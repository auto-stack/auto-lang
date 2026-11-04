//! PLAN-929 T-02：恢复→无预热首编辑最小整链（V-12 的 in-process 面）。
//!
//! 链路：冷建 → `to_manifest` → `encode_manifest` → `decode_manifest` →
//! `UnitCatalog::from_restored`（零源码生产——身份-only 惰性 body，字节级
//! 深重验）→ **立即**首个真实 λ/owner 编辑（恰目标物化、未改单元保持
//! headless 生产 0、无 dummy edit/整源 parse/预热）→ 失败修复（零变异）→
//! 伪造清单拒绝矩阵（使用前受控拒，V-05 面）→ 跨实例 foreign/stale 候选。
//!
//! 观察面：`body_work::capture_stage_attempts`（whole 轴=模块重建红）+
//! `ProductionMeter`（produced/reused/discarded 分账）+ headless 断言
//! （`SealedFn::item`/`SealedLambda::body` 为 `None` = 未物化）。

use ace_core::body_work::capture_stage_attempts;
use ace_core::source::cst::units::{self, DecodedManifest, RestoreError, UnitCatalog, MANIFEST_SCHEMA};
use ace_core::source::cst::{FrontBudget, FRONT_CAPS_VERSION};
use ace_core::source::SourceProfile;

use units::{decode_manifest, encode_manifest};

const CL: SourceProfile = SourceProfile::Closures;
const BUDGET: FrontBudget = FrontBudget::WORK_OWNED_64MIB;

const SRC: &str = "fn main() int {\n    let base int = 40\n    let f = (n int) => { return base + n }\n    let g = (m int) => { return f(m) + 1 }\n    return f(2) + g(3)\n}\nfn helper() int {\n    return 7\n}\n";

fn encode_decode(m: &units::CatalogManifest) -> DecodedManifest {
    decode_manifest(&encode_manifest(m)).expect("manifest decode")
}

fn restored() -> (UnitCatalog, DecodedManifest) {
    let cold = UnitCatalog::build(SRC, CL, BUDGET).expect("cold build");
    let manifest = cold.to_manifest(FRONT_CAPS_VERSION);
    let dm = encode_decode(&manifest);
    assert!(dm.matches_manifest(&manifest), "读回与当次导出全等");
    let cat = UnitCatalog::from_restored(SRC, &dm, CL, FRONT_CAPS_VERSION).expect("restore");
    (cat, dm)
}

/// T-02a：恢复本身零生产（whole 轴 0、无任何 per-unit 生产 attempt、计量
/// 全 0）；全部单元 headless（未物化）——恢复不做源码生产（§5.6）。
#[test]
fn t02_restore_itself_is_zero_production() {
    let cold = UnitCatalog::build(SRC, CL, BUDGET).expect("cold");
    assert!(cold.meter.produced > 0, "冷建有真实生产（对照面）");
    let manifest = cold.to_manifest(FRONT_CAPS_VERSION);
    let dm = decode_manifest(&encode_manifest(&manifest)).expect("decode");
    let (cat, attempts) = capture_stage_attempts(|| UnitCatalog::from_restored(SRC, &dm, CL, FRONT_CAPS_VERSION).expect("restore"));
    assert_eq!(attempts.whole_total(), 0, "恢复窗口零整源 parse（P0）");
    assert!(attempts.produced_units().is_empty(), "恢复窗口零 per-unit 生产");
    assert_eq!(cat.meter, Default::default(), "恢复计量全 0（无 borrowed 生产）");
    assert!(cat.fns.values().all(|f| f.item.is_none()), "全部 fn headless");
    assert!(cat.lambdas.values().all(|l| l.body.is_none()), "全部 λ headless");
    assert_eq!(cat.fns.len(), 2, "main+helper");
    assert_eq!(cat.lambdas.len(), 2, "main 的 2 个 λ");
    assert_eq!(cat.src(), SRC, "恢复源 = 调用方提供源");
}

/// T-02b：**无预热首编辑**——restore 后立即真实 λ 编辑：恰目标物化 1、
/// 兄弟 λ/owner/helper 生产 0 且保持 headless；源正确替换；无整源 parse
/// （whole 轴至多包装器 1 次且无 per-unit 重扫描）。
#[test]
fn t02_first_lambda_edit_after_restore_exact_production() {
    let (mut cat, _) = restored();
    let span = cat.lambda("main", 0).unwrap().span;
    let new_slice = "(n int) => { return base + n + 1 }";
    let expected_src = format!("{}{}{}", &SRC[..span.start as usize], new_slice, &SRC[span.end as usize..]);
    let (receipt, attempts) =
        capture_stage_attempts(|| cat.edit_lambda("main", 0, new_slice, BUDGET).expect("first edit after restore"));
    assert_eq!(receipt.produced.len(), 1, "恰目标 λ 生产");
    assert_eq!(receipt.produced[0].owner, "main");
    assert_eq!(receipt.produced[0].ordinal, 0);
    assert_eq!(receipt.reused.len(), 1, "兄弟 λ 复用（生产 0）");
    assert!(receipt.semantic_recheck_required, "目标 λ 文本已变 ⇒ 语义重验信号");
    assert!(attempts.whole_total() <= 1, "至多一次 λ 包装器片段解析，无整源重建");
    assert!(attempts.produced_units().is_empty() || attempts.produced_units().iter().all(|u| u == "main"), "无其他单元生产 attempt");
    assert_eq!(cat.meter.produced, 1, "恢复后首编辑：生产恰 1");
    assert_eq!(cat.meter.discarded, 0);
    assert_eq!(cat.src(), expected_src, "源恰目标切片替换");
    // 未改单元保持 headless（物化 0）且身份/坐标正确。
    assert!(cat.lambda("main", 1).unwrap().body.is_none(), "兄弟 λ 未物化");
    assert!(cat.lambda("main", 0).unwrap().body.is_some(), "目标 λ 已物化");
    assert!(cat.fns["main"].item.is_none(), "owner 未重产（λ 编辑不物化 owner）");
    assert!(cat.fns["helper"].item.is_none(), "无关函数 helper 未物化");
    let helper_span = cat.fns["helper"].span;
    assert!(helper_span.start > span.start as usize, "helper 坐标随漂移维护");
}

/// T-02c：restore 后首编辑**失败→修复**——坏片段 Diag 拒绝且目录零变异
/// （源/指纹/坐标/计量 produced 不动、discarded 如实 +1）；修复编辑成功。
#[test]
fn t02_first_edit_failure_zero_mutation_then_repair() {
    let (mut cat, _) = restored();
    let before_src = cat.src().to_string();
    let before_fns: Vec<_> = cat.fns.iter().map(|(k, v)| (k.clone(), v.content_sha, v.span)).collect();
    let before_lams: Vec<_> = cat.lambdas.iter().map(|(k, v)| (k.clone(), v.content_sha, v.span)).collect();
    let bad = "(n int) => { return base + n + }";
    let err = cat.edit_lambda("main", 0, bad, BUDGET).expect_err("坏片段必须拒绝");
    assert!(matches!(err, units::EditError::Diag(_)), "源码诊断分态：{err:?}");
    assert_eq!(cat.meter.produced, 0, "失败不产生已提交生产");
    assert_eq!(cat.meter.discarded, 1, "失败 attempt 如实计数（不默认 0）");
    assert_eq!(cat.src(), before_src, "源零变异");
    for (k, sha, span) in &before_fns {
        let f = &cat.fns[k];
        assert_eq!(f.content_sha, *sha, "fn 指纹不变：{k}");
        assert_eq!(f.span, *span, "fn 坐标不变：{k}");
    }
    for (k, sha, span) in &before_lams {
        let l = cat.lambdas.get(k).unwrap();
        assert_eq!(l.content_sha, *sha, "λ 指纹不变：{k:?}");
        assert_eq!(l.span, *span, "λ 坐标不变：{k:?}");
    }
    // 修复编辑（V-06/§5.6：失败后正常继续）。
    let ok = "(n int) => { return base + n + 2 }";
    let receipt = cat.edit_lambda("main", 0, ok, BUDGET).expect("修复编辑成功");
    assert_eq!(receipt.produced.len(), 1, "修复恰目标生产");
    assert_eq!(cat.meter.produced, 1);
}

/// T-02d：restore 后 owner 编辑（嵌套单元面）——同文 λ 语法共享（保持
/// headless、生产 0），owner 恰生产 1；λ 拓扑变化走具名非快路径。
#[test]
fn t02_owner_edit_after_restore_same_text_lambda_zero_production() {
    let (mut cat, _) = restored();
    // owner body 文本变化但两个 λ 文本字节不变：把 `let base int = 40`
    // 改为 `let base int = 41`（λ 文本不动）。
    let old_fn_span = cat.fns["main"].span;
    let old_fn = cat.src()[old_fn_span.start as usize..old_fn_span.end as usize].to_string();
    let new_fn = old_fn.replace("let base int = 40", "let base int = 41");
    let (receipt, attempts) =
        capture_stage_attempts(|| cat.edit_owner("main", &new_fn, BUDGET).expect("owner edit after restore"));
    assert_eq!(receipt.produced.len(), 1, "恰 owner 生产 1");
    assert_eq!(receipt.produced[0].kind, units::UnitKind::Fn);
    assert_eq!(receipt.reused.len(), 2, "两个同文 λ 复用（生产 0）");
    assert!(receipt.semantic_recheck_required, "owner 内容已变 ⇒ 重验信号");
    assert!(receipt.non_fast_path.is_none(), "同拓扑同文 λ 不走非快路径");
    assert!(attempts.whole_total() <= 1, "无整源重建");
    assert!(cat.lambda("main", 0).unwrap().body.is_none(), "同文 λ 保持 headless（物化 0）");
    assert!(cat.lambda("main", 1).unwrap().body.is_none(), "同文 λ 保持 headless（物化 0）");
    assert!(cat.fns["main"].item.is_some(), "owner 已物化");
    assert!(cat.fns["helper"].item.is_none(), "无关函数未物化");
}

/// T-02e：伪造/近邻清单**使用前**受控拒绝矩阵（V-05 面；全 typed 错误，
/// 不 panic、不半恢复）。
#[test]
fn t02_restore_rejection_matrix() {
    let cold = UnitCatalog::build(SRC, CL, BUDGET).expect("cold");
    let manifest = cold.to_manifest(FRONT_CAPS_VERSION);
    let good = decode_manifest(&encode_manifest(&manifest)).expect("decode");
    // caps 漂移。
    let mut m = good.clone();
    m.caps_version = "ac-source-front/9".into();
    assert!(matches!(
        UnitCatalog::from_restored(SRC, &m, CL, FRONT_CAPS_VERSION),
        Err(RestoreError::CapsMismatch { .. })
    ));
    // profile 漂移。
    let mut m = good.clone();
    m.profile = "scalar".into();
    assert!(matches!(
        UnitCatalog::from_restored(SRC, &m, CL, FRONT_CAPS_VERSION),
        Err(RestoreError::ProfileMismatch { .. })
    ));
    // schema 漂移（伪造 DecodedManifest 直构——decode 已拒文本面）。
    let mut m = good.clone();
    m.schema = "ac-source-units/2".into();
    assert!(matches!(
        UnitCatalog::from_restored(SRC, &m, CL, FRONT_CAPS_VERSION),
        Err(RestoreError::SchemaMismatch { .. })
    ));
    // 源漂移（module 指纹不符）。
    let mutated_src = SRC.replace("return 7", "return 8");
    assert!(matches!(
        UnitCatalog::from_restored(&mutated_src, &good, CL, FRONT_CAPS_VERSION),
        Err(RestoreError::ModuleHashMismatch { .. })
    ));
    // 单元内容指纹伪造。
    let mut m = good.clone();
    if let Some(u) = m.units.iter_mut().find(|u| u.kind == units::UnitKind::Fn && u.owner == "helper") {
        u.content_sha ^= 1;
    }
    assert!(matches!(
        UnitCatalog::from_restored(SRC, &m, CL, FRONT_CAPS_VERSION),
        Err(RestoreError::UnitHashMismatch { ref owner, .. }) if owner == "helper"
    ));
    // span 越界。
    let mut m = good.clone();
    if let Some(u) = m.units.iter_mut().find(|u| u.kind == units::UnitKind::Fn && u.owner == "helper") {
        u.current_span.1 = SRC.len() + 10;
    }
    assert!(matches!(
        UnitCatalog::from_restored(SRC, &m, CL, FRONT_CAPS_VERSION),
        Err(RestoreError::SpanOutOfBounds { .. })
    ));
    // 单元重叠。
    let mut m = good.clone();
    if let Some(u) = m.units.iter_mut().find(|u| u.kind == units::UnitKind::Fn && u.owner == "helper") {
        u.current_span.0 = 0;
        u.current_span.1 = SRC.len();
        // 重算指纹使该单元自身仍匹配（重叠检测不被指纹先行拒绝）。
        u.content_sha = units::content_sha(SRC.as_bytes());
    }
    assert!(matches!(
        UnitCatalog::from_restored(SRC, &m, CL, FRONT_CAPS_VERSION),
        Err(RestoreError::OverlappingUnits { .. })
    ));
    // 重复键。
    let mut m = good.clone();
    let dup = m.units.iter().find(|u| u.kind == units::UnitKind::Fn && u.owner == "helper").cloned().unwrap();
    m.units.push(dup);
    assert!(matches!(
        UnitCatalog::from_restored(SRC, &m, CL, FRONT_CAPS_VERSION),
        Err(RestoreError::DuplicateUnit { .. })
    ));
}

/// T-02f：跨实例 foreign / stale 候选（恢复面沿 928 合同）：同源同清单两次
/// 恢复 ⇒ 不同 instance；A 的候选提交进 B 一律 typed 拒（零变异）；stale
/// 同判。恢复目录上的零编辑重放也成立（生产 0，无解析）。
#[test]
fn t02_restore_foreign_stale_zero_edit_replay() {
    let (mut a, _dm) = restored();
    let (mut b, _) = restored();
    assert_ne!(a.instance, b.instance, "同源两恢复目录实例身份不同");
    // foreign：A 暂存零编辑候选 → 提交进 B。owner 切片字节全等 ⇒ 零编辑
    // 快路径（headless owner 不解析、候选 produced=0）。
    let ospan = a.fns["main"].span;
    let slice = a.src()[ospan.start as usize..ospan.end as usize].to_string();
    let cand = a.stage_owner_edit("main", &slice, BUDGET).expect("zero-edit stage on restored");
    assert_eq!(cand.produced(), 0, "零编辑候选零生产（headless 不解析）");
    assert!(cand.semantic_recheck().is_empty());
    let err = b.commit_staged(cand).expect_err("foreign candidate");
    assert!(matches!(err, units::EditError::StaleForeign(_)));
    // A 上零编辑提交：诚实回执（生产 0）。
    let cand = a.stage_owner_edit("main", &slice, BUDGET).expect("re-stage");
    let r = a.commit_staged(cand).expect("zero-edit commit on restored");
    assert!(r.produced.is_empty());
    // stale：暂存后推进目录再提交。
    let new_fn_src = "fn main() int {\n    let base int = 42\n    let f = (n int) => { return base + n }\n    let g = (m int) => { return f(m) + 1 }\n    return f(2) + g(3)\n}\n";
    let cand2 = a.stage_owner_edit("main", new_fn_src, BUDGET).expect("stage real edit");
    let _ = a.edit_lambda("main", 1, "(m int) => { return f(m) + 2 }", BUDGET).expect("advance revision");
    let err = a.commit_staged(cand2).expect_err("stale candidate");
    assert!(matches!(err, units::EditError::StaleForeign(_)), "stale 拒绝：{err:?}");
}

/// T-02g：恢复目录跨编辑再导出——编辑后 `to_manifest` 反映新身份/坐标；
/// 二次恢复（新清单）继续首编辑（恢复链可续，无半恢复态）。
#[test]
fn t02_restore_chain_continues_across_edits() {
    let (mut cat, _) = restored();
    let r = cat.edit_lambda("main", 0, "(n int) => { return base + n + 5 }", BUDGET).expect("edit");
    let m2 = cat.to_manifest(FRONT_CAPS_VERSION);
    let dm2 = decode_manifest(&encode_manifest(&m2)).expect("re-encode/decode");
    let (mut cat2, attempts) =
        capture_stage_attempts(|| UnitCatalog::from_restored(&r.new_src, &dm2, CL, FRONT_CAPS_VERSION).expect("second restore"));
    assert_eq!(attempts.whole_total(), 0, "二次恢复同样零生产");
    assert_eq!(cat2.meter, Default::default());
    assert_eq!(cat2.src(), r.new_src);
    let again = cat2.edit_lambda("main", 0, "(n int) => { return base + n + 6 }", BUDGET).expect("edit on second restore");
    assert_eq!(again.produced.len(), 1);
}

/// T-02h：schema 常量与清单合同（防漂移）：MANIFEST_SCHEMA 与 codec/解码
/// 拒绝面一致（未知 schema 文本解码即拒）。
#[test]
fn t02_manifest_schema_contract() {
    assert_eq!(MANIFEST_SCHEMA, "ac-source-units/1");
    let cold = UnitCatalog::build(SRC, CL, BUDGET).expect("cold");
    let text = encode_manifest(&cold.to_manifest(FRONT_CAPS_VERSION));
    let bad = text.replacen(MANIFEST_SCHEMA, "ac-source-units/2", 1);
    assert!(decode_manifest(&bad).is_err(), "未知 schema 消费前拒");
}

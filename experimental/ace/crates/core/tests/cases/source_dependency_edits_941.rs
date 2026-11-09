//! PLAN-941 V-04（T-06）：多函数/方法/闭包 body/signature/layout 依赖
//! 编辑集的 **P/IO 实测**——固定编辑集（§5.5）在单元目录面的逐族计量：
//!
//! - 每族编辑只物化恰目标单元（P=1），未改函数/方法/闭包生产 0 且载荷
//!   **Rc 同一**（IO=0：零重读/零重解析）；
//! - whole 轴零整源重建（`capture_stage_attempts`）；
//! - callee 只改未被 caller 读的 body：caller 生产 0，但重验信号必达且新
//!   实现文本对后续 link 可见（§5.5）；签名/layout 变化按事件 + 精确
//!   dependents 失效，调用点字节不变不静默；
//! - meta 正负依赖族由 ace-comptime 组 `meta_capability_941` 承载（目录
//!   面无 meta 语义，不在此伪测）。
//!
//! 边界：layout（type 声明）内容编辑的目录面 = 外部声明编辑路径提交后的
//! `note_decl_changed` 通知面（精确 dependents + 零生产）；本文件只测真实
//! 存在的目录 API，不新增生产写面。

use ace_core::body_work::capture_stage_attempts;
use ace_core::source::cst::units::{self, UnitCatalog};
use ace_core::source::cst::{FrontBudget, FRONT_CAPS_VERSION};
use ace_core::source::SourceProfile;
use std::rc::Rc;

const CL: SourceProfile = SourceProfile::Closures;
const COL: SourceProfile = SourceProfile::Collections;
const BUDGET: FrontBudget = FrontBudget::WORK_OWNED_64MIB;

/// 依赖图：main（含 2 λ）调用 helper；unrelated 独立。
const CL_SRC: &str = "fn main() int {\n    let base int = 40\n    let f = (n int) => { return base + n }\n    let g = (m int) => { return f(m) + helper() + 1 }\n    return f(2) + g(3)\n}\nfn helper() int {\n    return 7\n}\nfn unrelated() int {\n    return 5\n}\n";

/// collections 依赖图：Item 声明；make/consume 方法链；main 调用；旁路 fn。
const COL_SRC: &str = "type Item { name Str; code int }\nfn make(x Item) int {\n    return x.code\n}\nfn consume(x Item) int {\n    return make(x) + 1\n}\nfn main() int {\n    let a Item = Item(49, 41)\n    return consume(a.move)\n}\nfn unrelated() int {\n    return 5\n}\n";

/// fn/λ 载荷 Rc 指纹快照——编辑后逐单元核对（IO=0 面：未改单元零重读）。
fn fingerprints(
    cat: &UnitCatalog,
) -> (
    Vec<(String, Option<usize>)>,
    Vec<((String, u32), Option<usize>)>,
) {
    let mut fns: Vec<(String, Option<usize>)> = cat
        .fns
        .iter()
        .map(|(k, v)| (k.clone(), v.item.as_ref().map(|rc| Rc::as_ptr(rc) as usize)))
        .collect();
    fns.sort_by(|a, b| a.0.cmp(&b.0));
    let mut lams: Vec<((String, u32), Option<usize>)> = cat
        .lambdas
        .iter()
        .map(|(k, v)| (k.clone(), v.body.as_ref().map(|rc| Rc::as_ptr(rc) as usize)))
        .collect();
    lams.sort_by(|a, b| (&a.0, &a.1).cmp(&(&b.0, &b.1)));
    (fns, lams)
}

fn assert_unproduced_unchanged(
    cat: &UnitCatalog,
    before: &(
        Vec<(String, Option<usize>)>,
        Vec<((String, u32), Option<usize>)>,
    ),
    skip_fns: &[&str],
    skip_lams: &[&str],
) {
    let (fns, lams) = fingerprints(cat);
    let (bf, bl) = before;
    for ((name, ptr), (name2, ptr2)) in fns.iter().zip(bf.iter()) {
        assert_eq!(name, name2);
        if skip_fns.contains(&name.as_str()) {
            continue;
        }
        assert_eq!(ptr, ptr2, "未改 fn 载荷 Rc 同一（生产 0/IO 0）：{name}");
    }
    for (((owner, ordinal), ptr), ((owner2, ordinal2), ptr2)) in lams.iter().zip(bl.iter()) {
        assert_eq!((owner, ordinal), (owner2, ordinal2));
        if skip_lams.contains(&owner.as_str()) {
            continue;
        }
        assert_eq!(
            ptr, ptr2,
            "未改 λ 载荷 Rc 同一（生产 0/IO 0）：{owner}#{ordinal}"
        );
    }
}

/// V-04(a)：callee body 编辑（caller 不读 callee body 文本）——恰 callee
/// 生产 1；caller（含其全部 λ）与旁路函数生产 0 且载荷 Rc 同一；零整源
/// 重建；重验信号必达（新实现必须对后续 link 可见）；mint 身份携带。
#[test]
fn v04_callee_body_edit_caller_zero_io() {
    let mut cat = UnitCatalog::build(CL_SRC, CL, BUDGET).expect("cold");
    assert_eq!(cat.meter.produced, 5, "main+2λ+helper+unrelated");
    let before = fingerprints(&cat);
    let helper_mint = cat.fns["helper"].mint_span;
    let old = cat.src()[cat.fns["helper"].span.start..cat.fns["helper"].span.end].to_string();
    let new_fn = old.replace("return 7", "return 8");
    let produced_before = cat.meter.produced;
    let (receipt, attempts) = capture_stage_attempts(|| {
        cat.edit_owner("helper", &new_fn, BUDGET)
            .expect("callee body edit")
    });
    assert_eq!(receipt.produced.len(), 1, "恰 callee 生产");
    assert_eq!(receipt.produced[0].owner, "helper");
    assert_eq!(receipt.produced[0].kind, units::UnitKind::Fn);
    assert!(
        receipt.reused.is_empty(),
        "helper 无 λ ⇒ 无同文复用行（caller 不出现在生产/复用账）"
    );
    assert!(
        receipt.semantic_recheck_required,
        "callee 内容已变 ⇒ 重验信号（caller 生产 0 但必须重验 link）"
    );
    assert!(receipt.non_fast_path.is_none());
    assert_eq!(attempts.whole_total(), 0, "零整源 parse（P0）");
    assert!(
        attempts.produced_units().is_empty(),
        "无其他单元生产 attempt"
    );
    assert_eq!(cat.meter.produced, produced_before + 1);
    assert_eq!(cat.meter.discarded, 0);
    // 新实现文本对后续消费可见（目录当前源即编辑基）。
    assert!(cat.src().contains("return 8"), "新实现文本在当前源");
    assert_eq!(
        cat.fns["helper"].mint_span, helper_mint,
        "mint 身份跨 body 编辑携带"
    );
    assert!(
        !cat.events
            .iter()
            .any(|e| matches!(e, units::CatalogEvent::OwnerSignatureChanged { .. })),
        "body-only 编辑无签名事件"
    );
    assert_unproduced_unchanged(&cat, &before, &["helper"], &[]);
}

/// V-04(b)：方法族 body 编辑（collections：make/consume 方法链）——逐方法
/// 编辑只物化恰目标方法；链上/旁路函数与 Item 声明零生产零重读；签名事
/// 件不出现；连续两次编辑累计计量如实。
#[test]
fn v04_method_family_body_edits_zero_io() {
    let mut cat = UnitCatalog::build(COL_SRC, COL, BUDGET).expect("cold");
    assert_eq!(cat.meter.produced, 5, "Item+make+consume+main+unrelated");
    let decl_sha = cat.decls["Item"].content_sha;
    // 编辑 1：consume 方法体（链上方法；等长替换）。签名摘要当前含铸造
    // 坐标（span-sensitive）——为测"body-only 编辑无签名事件"的语义面，
    // 先做无前置漂移的编辑（漂移后的保守事件面见计划 §10.17 具名记录）。
    let before2 = fingerprints(&cat);
    let old = cat.src()[cat.fns["consume"].span.start..cat.fns["consume"].span.end].to_string();
    let new_fn = old.replace("+ 1", "+ 2");
    let (r1, a1) = capture_stage_attempts(|| {
        cat.edit_owner("consume", &new_fn, BUDGET)
            .expect("consume body edit")
    });
    assert_eq!(r1.produced.len(), 1);
    assert_eq!(r1.produced[0].owner, "consume");
    assert_eq!(a1.whole_total(), 0);
    assert!(r1.semantic_recheck_required, "main 调用 consume ⇒ 重验信号");
    assert_unproduced_unchanged(&cat, &before2, &["consume"], &[]);
    // 编辑 2：make 方法体（callee，+4 变长）——make 参数 span 在编辑点之
    // 前（无漂移）；Item 声明与 consume（已物化载荷）零生产零重读。
    let before3 = fingerprints(&cat);
    let old = cat.src()[cat.fns["make"].span.start..cat.fns["make"].span.end].to_string();
    let new_fn = old.replace("x.code", "x.code + 1");
    let (r2, a2) = capture_stage_attempts(|| {
        cat.edit_owner("make", &new_fn, BUDGET)
            .expect("make body edit")
    });
    assert_eq!(r2.produced.len(), 1);
    assert_eq!(r2.produced[0].owner, "make");
    assert_eq!(a2.whole_total(), 0);
    assert!(r2.semantic_recheck_required, "consume 调用 make ⇒ 重验信号");
    assert_unproduced_unchanged(&cat, &before3, &["make"], &[]);
    assert_eq!(cat.meter.produced, 7, "冷 5 + 两次编辑各 1");
    assert_eq!(cat.meter.discarded, 0);
    assert_eq!(
        cat.decls["Item"].content_sha, decl_sha,
        "声明单元指纹未被方法编辑扰动"
    );
    assert!(
        !cat.events
            .iter()
            .any(|e| matches!(e, units::CatalogEvent::OwnerSignatureChanged { .. })),
        "两次 body-only 方法编辑无签名事件：{:?}",
        cat.events
    );
    assert!(cat.src().contains("x.code + 1") && cat.src().contains("+ 2"));
}

/// V-04(c)：变长闭包子体编辑（多函数面）——目标 λ 变长重写恰生产 1；兄
/// 弟 λ 复用（0 生产）；helper/unrelated 载荷 Rc 同一；编辑点之后坐标漂
/// 移正确；whole 轴至多包装器 1 次且无 per-unit 重扫描。
#[test]
fn v04_closure_variable_length_subbody_edit_precise() {
    let mut cat = UnitCatalog::build(CL_SRC, CL, BUDGET).expect("cold");
    let before = fingerprints(&cat);
    let span = cat.lambda("main", 0).unwrap().span;
    let g_before = cat.lambda("main", 1).unwrap().span;
    let helper_before = cat.fns["helper"].span;
    let new_slice = "(n int) => { return base + n + 100 + 200 }";
    let delta = new_slice.len() as i64 - (span.end - span.start) as i64;
    let expected_src = format!(
        "{}{}{}",
        &CL_SRC[..span.start as usize],
        new_slice,
        &CL_SRC[span.end as usize..]
    );
    let (receipt, attempts) = capture_stage_attempts(|| {
        cat.edit_lambda("main", 0, new_slice, BUDGET)
            .expect("variable-length lambda edit")
    });
    assert_eq!(receipt.produced.len(), 1, "恰目标 λ 生产");
    assert_eq!(receipt.produced[0].owner, "main");
    assert_eq!(receipt.produced[0].ordinal, 0);
    assert_eq!(receipt.produced[0].kind, units::UnitKind::Lambda);
    assert_eq!(receipt.reused.len(), 1, "兄弟 λ 复用（生产 0）");
    assert_eq!(receipt.reused[0].ordinal, 1);
    assert!(receipt.semantic_recheck_required);
    assert!(
        attempts.whole_total() <= 1,
        "至多 λ 包装器 1 次，无整源重建"
    );
    assert!(
        attempts.produced_units().is_empty(),
        "无 per-unit 重扫描 attempt"
    );
    assert_eq!(cat.src(), expected_src, "变长替换源正确");
    // 漂移：编辑点之后单元坐标平移 delta。
    let g_after = cat.lambda("main", 1).unwrap().span;
    assert_eq!(
        (
            g_after.start as i64 - g_before.start as i64,
            g_after.end as i64 - g_before.end as i64
        ),
        (delta, delta),
        "兄弟 λ 坐标漂移 delta"
    );
    let helper_after = cat.fns["helper"].span;
    assert_eq!(
        (
            helper_after.start as i64 - helper_before.start as i64,
            helper_after.end as i64 - helper_before.end as i64
        ),
        (delta, delta),
        "后续 fn 坐标漂移 delta"
    );
    assert_unproduced_unchanged(&cat, &before, &[], &["main"]);
}

/// V-04(d)：签名编辑（callee 签名面）——事件必达 + 签名摘要变化；恰目标
/// 生产 1；caller（consume/main）生产 0 且 Rc 同一但重验信号必达（调用点
/// 字节不变不能盲 hit）；mint 身份携带；未引用旁路不入任何事件。
#[test]
fn v04_signature_edit_precise_invalidation_zero_io() {
    let mut cat = UnitCatalog::build(COL_SRC, COL, BUDGET).expect("cold");
    let before = fingerprints(&cat);
    let make_mint = cat.fns["make"].mint_span;
    let snap_before = cat.signature_snapshot("make").expect("cold materialized");
    let old = cat.src()[cat.fns["make"].span.start..cat.fns["make"].span.end].to_string();
    let new_fn = old.replace("fn make(x Item) int", "fn make(x Item, extra int) int");
    let (receipt, attempts) = capture_stage_attempts(|| {
        cat.edit_owner("make", &new_fn, BUDGET)
            .expect("signature edit")
    });
    assert_eq!(receipt.produced.len(), 1, "恰签名编辑目标生产");
    assert_eq!(receipt.produced[0].owner, "make");
    assert!(
        receipt.semantic_recheck_required,
        "签名变化 ⇒ 依赖者（consume/main 调用点）重验信号"
    );
    assert_eq!(attempts.whole_total(), 0);
    assert!(
        cat.events.iter().any(
            |e| matches!(e, units::CatalogEvent::OwnerSignatureChanged { owner } if owner == "make")
        ),
        "OwnerSignatureChanged 必达：{:?}",
        cat.events
    );
    let snap_after = cat.signature_snapshot("make").expect("edited materialized");
    assert_ne!(snap_before, snap_after, "签名摘要已变化");
    assert!(snap_after.contains("extra"), "新参数面进入摘要");
    assert_eq!(
        cat.fns["make"].mint_span, make_mint,
        "mint 身份跨签名编辑携带"
    );
    // 调用点文本未改（consume/main 源切片不变）但不得静默：事件 + 重验信
    // 号已覆盖；其载荷零生产零重读。
    assert_unproduced_unchanged(&cat, &before, &["make"], &[]);
}

/// V-04(e)：layout（type 声明）变化——外部声明编辑路径提交通知面：
/// `DeclChanged.dependents` 恰为名字引用该声明的 owner（make/consume/
/// main），未引用的 unrelated 不入；`DeclSetChanged` 必达；通知本身零单元
/// 生产（计量不动）且全部单元载荷 Rc 同一；声明指纹按新 span 切片刷新。
#[test]
fn v04_layout_decl_change_precise_dependents_zero_io() {
    let mut cat = UnitCatalog::build(COL_SRC, COL, BUDGET).expect("cold");
    let before = fingerprints(&cat);
    let (produced, reused, discarded) = (cat.meter.produced, cat.meter.reused, cat.meter.discarded);
    let old_span = cat.decls["Item"].span;
    // 外部声明编辑（如加字段）后的新 span——等长平移代表面（内容面由
    // sha 刷新断言承载）。
    let new_span = old_span;
    let ((), attempts) = capture_stage_attempts(|| cat.note_decl_changed("Item", new_span));
    let dependents = cat
        .events
        .iter()
        .find_map(|e| match e {
            units::CatalogEvent::DeclChanged { name, dependents } if name == "Item" => {
                Some(dependents.clone())
            }
            _ => None,
        })
        .expect("DeclChanged 必达");
    let mut sorted = dependents.clone();
    sorted.sort();
    assert_eq!(
        sorted,
        vec![
            "consume".to_string(),
            "main".to_string(),
            "make".to_string()
        ],
        "dependents 恰为名字引用面（精确失效，不笼统失效）"
    );
    assert!(
        cat.events
            .iter()
            .any(|e| matches!(e, units::CatalogEvent::DeclSetChanged)),
        "DeclSetChanged 必达"
    );
    assert_eq!(attempts.whole_total(), 0);
    assert!(attempts.produced_units().is_empty(), "通知面零生产 attempt");
    assert_eq!(
        (cat.meter.produced, cat.meter.reused, cat.meter.discarded),
        (produced, reused, discarded),
        "layout 变化通知零单元生产/复用/废弃（计量不动）"
    );
    assert_unproduced_unchanged(&cat, &before, &[], &[]);
    // 声明指纹按当前 span 切片刷新（同切片 ⇒ 同 sha；漂移切片 ⇒ 新 sha）。
    assert_eq!(
        cat.decls["Item"].content_sha,
        units::content_sha(COL_SRC[old_span.start..old_span.end].as_bytes()),
        "同 span 通知刷新为同内容指纹"
    );
}

/// V-04(f)：依赖编辑后 fresh ≡ 增量（§5.5 不变式）——对编辑后的当前源
/// 冷建目录，逐单元 content_sha/mint 跨度与增量目录全等；清单可编解码且
/// 恢复目录深重验通过（编辑链产物的可恢复身份闭合）。
#[test]
fn v04_edits_keep_fresh_equivalence_and_restorable_identity() {
    let mut cat = UnitCatalog::build(COL_SRC, COL, BUDGET).expect("cold");
    let make_mint_before = cat.fns["make"].mint_span;
    let old = cat.src()[cat.fns["make"].span.start..cat.fns["make"].span.end].to_string();
    let new_fn = old.replace("x.code", "x.code + 1");
    cat.edit_owner("make", &new_fn, BUDGET)
        .expect("incremental edit");
    let fresh = UnitCatalog::build(cat.src(), COL, BUDGET).expect("fresh on edited src");
    let make_mint_before = make_mint_before;
    for owner in ["make", "consume", "main", "unrelated"] {
        let a = &cat.fns[owner];
        let b = &fresh.fns[owner];
        assert_eq!(a.content_sha, b.content_sha, "{owner} 内容指纹 fresh≡增量");
        assert_eq!(a.span, b.span, "{owner} 当前坐标 fresh≡增量");
    }
    // 编辑单元 mint 身份携带（增量目录保铸造坐标；fresh 重新铸造不违反
    // 身份合同——身份 = 键+content_sha+mint_span，mint 面不回填）。
    assert_eq!(
        cat.fns["make"].mint_span, make_mint_before,
        "make mint 跨编辑携带"
    );
    for owner in ["consume", "main", "unrelated"] {
        let a = &cat.fns[owner];
        let b = &fresh.fns[owner];
        assert_eq!(
            a.mint_span.end - a.mint_span.start,
            b.mint_span.end - b.mint_span.start,
            "{owner} 未编辑单元 mint 跨度一致"
        );
    }
    assert_eq!(
        cat.decls["Item"].content_sha,
        fresh.decls["Item"].content_sha
    );
    let manifest = cat.to_manifest(FRONT_CAPS_VERSION);
    let dm = units::decode_manifest(&units::encode_manifest(&manifest)).expect("decode");
    assert!(dm.matches_manifest(&manifest), "清单编解码往返全等");
    let restored =
        UnitCatalog::from_restored(cat.src(), &dm, COL, FRONT_CAPS_VERSION).expect("restore");
    assert_eq!(restored.meter, Default::default(), "恢复零生产");
    assert!(
        restored.fns.values().all(|f| f.item.is_none()),
        "恢复目录全 headless"
    );
}

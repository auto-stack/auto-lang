//! PLAN-929 T-07：声明/签名/依赖传播矩阵（V-03 的 core 目录面）。
//!
//! - 签名变化 ⇒ `OwnerSignatureChanged` 事件必达（调用点文本未改不静默）；
//! - type/enum 声明变化 ⇒ `DeclChanged{dependents}`（名字引用面）+ `DeclSetChanged`；
//!   恢复目录未物化单元不进入 dependents 扫描（§5.6 不补扫）；
//! - 前缀/空白漂移 ⇒ 当前坐标漂移、mint 身份与内容指纹携带；
//! - **fresh ≡ 增量**：编辑后的目录与对新源冷建的目录清单全等。

use ace_core::atom_text::Span;
use ace_core::source::cst::units::{self, UnitCatalog};
use ace_core::source::cst::{FrontBudget, FRONT_CAPS_VERSION};
use ace_core::source::SourceProfile;
use units::{decode_manifest, encode_manifest};

const CL: SourceProfile = SourceProfile::Closures;
const B: FrontBudget = FrontBudget::WORK_OWNED_64MIB;

const SRC: &str = "fn base(n int) int { return n * 2 }\nfn main() int {\n    let f = (x int) => { return base(x) + 1 }\n    return f(2)\n}\n";

/// V-03a：owner 签名变化（参数面）——事件必达、λ 拓扑不动、
/// 调用点文本（其他 owner）未改也不得静默。
#[test]
fn t07_signature_change_event_reaches_dependents_surface() {
    let mut cat = UnitCatalog::build(SRC, CL, B).expect("cold");
    assert!(cat.events.is_empty(), "冷建无事件");
    let old_fn = cat.src()[cat.fns["base"].span.start..cat.fns["base"].span.end].to_string();
    let new_fn = old_fn.replace("fn base(n int) int", "fn base(n int, k int) int");
    let r = cat.edit_owner("base", &new_fn, B).expect("owner edit");
    assert!(r.semantic_recheck_required, "签名变化 ⇒ 重验信号");
    assert!(
        cat.events.iter().any(
            |e| matches!(e, units::CatalogEvent::OwnerSignatureChanged { owner } if owner == "base")
        ),
        "OwnerSignatureChanged 必达：{:?}",
        cat.events
    );
    assert!(
        !cat.events
            .iter()
            .any(|e| matches!(e, units::CatalogEvent::LambdaTopologyChanged { .. })),
        "base 无 λ ⇒ 无拓扑事件"
    );
    // main 的调用点文本未改——事件面不依赖调用点文本（不静默）。
}

/// V-03b：type 声明变化——dependents 为目录内名字引用面（已物化 owner）；
/// 恢复目录（headless）不补扫（§5.6）。
#[test]
fn t07_decl_change_dependents_materialized_only() {
    // collections 面（type/enum 所在 profile）。
    let src = "type Counter {\n    value int\n    fn read() int { return .value }\n}\nfn use_it() int { let c Counter = Counter(1)\n    return c.read() }\nfn unrelated() int { return 5 }\n";
    let col = SourceProfile::Collections;
    let mut cat = UnitCatalog::build(src, col, B).expect("cold");
    cat.events.clear();
    let new_span = Span { start: 5, end: 20 };
    cat.note_decl_changed("Counter", new_span);
    let dep = cat
        .events
        .iter()
        .find_map(|e| match e {
            units::CatalogEvent::DeclChanged { dependents, .. } => Some(dependents.clone()),
            _ => None,
        })
        .expect("DeclChanged 必达");
    assert!(
        dep.contains(&"use_it".to_string()),
        "名字引用者入 dependents：{dep:?}"
    );
    assert!(
        !dep.contains(&"unrelated".to_string()),
        "无关函数不入 dependents"
    );
    assert!(
        cat.events
            .iter()
            .any(|e| matches!(e, units::CatalogEvent::DeclSetChanged)),
        "DeclSetChanged 必达"
    );
    // 恢复目录（headless）——dependents 扫描只在已物化单元上（不补扫）。
    let m = cat.to_manifest(FRONT_CAPS_VERSION);
    let dm = decode_manifest(&encode_manifest(&m)).expect("decode");
    let mut restored =
        UnitCatalog::from_restored(cat.src(), &dm, col, FRONT_CAPS_VERSION).expect("restore");
    restored.note_decl_changed("Counter", new_span);
    let dep2 = restored
        .events
        .iter()
        .find_map(|e| match e {
            units::CatalogEvent::DeclChanged { dependents, .. } => Some(dependents.clone()),
            _ => None,
        })
        .expect("DeclChanged on restored");
    assert!(dep2.is_empty(), "headless 目录不补扫未物化 owner（§5.6）");
}

/// V-03c：前缀/空白漂移——当前坐标漂移、mint 身份与内容指纹携带不变；
/// 漂移源上的冷建目录与原目录单元身份一致。
#[test]
fn t07_prefix_drift_identity_carried() {
    let cat = UnitCatalog::build(SRC, CL, B).expect("cold");
    let drifted = format!("// leading comment\n\n{SRC}");
    let cat2 = UnitCatalog::build(&drifted, CL, B).expect("cold on drifted");
    for owner in ["base", "main"] {
        let a = &cat.fns[owner];
        let b = &cat2.fns[owner];
        assert_eq!(a.content_sha, b.content_sha, "{owner} 内容指纹携带");
        assert_eq!(
            a.mint_span.end - a.mint_span.start,
            b.mint_span.end - b.mint_span.start,
            "{owner} mint 跨度不变（等长前缀漂移）"
        );
        assert!(b.span.start > a.span.start, "{owner} 当前坐标已漂移");
    }
    // λ 同携带。
    let la = cat.lambda("main", 0).unwrap();
    let lb = cat2.lambda("main", 0).unwrap();
    assert_eq!(la.content_sha, lb.content_sha, "λ 内容指纹携带");
}

/// V-03d：**fresh ≡ 增量**——owner 编辑后的目录清单与对新源冷建的清单全等
/// （单元键/指纹/坐标面完全一致；module 指纹同源）。
#[test]
fn t07_fresh_equals_incremental_manifest() {
    let mut cat = UnitCatalog::build(SRC, CL, B).expect("cold");
    let old_fn = cat.src()[cat.fns["base"].span.start..cat.fns["base"].span.end].to_string();
    let new_fn = old_fn.replace("return n * 2", "return n * 3");
    let r = cat.edit_owner("base", &new_fn, B).expect("edit");
    let m_inc = cat.to_manifest(FRONT_CAPS_VERSION);
    let fresh = UnitCatalog::build(&r.new_src, CL, B).expect("fresh on edited source");
    let m_fresh = fresh.to_manifest(FRONT_CAPS_VERSION);
    // 排序确定性：to_manifest 内部按 (owner, ordinal, kind) 排序——两清单全等。
    assert_eq!(
        encode_manifest(&m_inc),
        encode_manifest(&m_fresh),
        "fresh ≡ 增量（清单全等）"
    );
}

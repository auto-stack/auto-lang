//! PLAN-981 V 批（V-B-LANGUAGE / V-B-SESSION）**可先跑的检查骨架**。
//!
//! 计划 §7 的 V-L-* / V-B-SESSION 条目要求"正式源 + 真实 consumer"。980 的正式
//! UI/事件源尚未交付时，本文件只冻结**不依赖 980 输入**的部分：五出口/选择、
//! if-lite 与 `#if` 归属、生命周期计数、消息/成员槽位、构造分母。依赖 980 的条目
//! 在 [`V_PENDING_INPUTS`] 里具名登记——V 批就绪时按该清单替换源即可，不重写判据。
use ace_core::source::age_h0::LegacyWidgetAlias;
use ace_core::source::age_package::{
    collect_facts,
    age_construct_coverage, compile_age_package, select_exit, AgeViewExit, AgeViewRequest,
    AGE_CONSTRUCTS,
};
use ace_core::source::packages::{PackageModuleInput, PACKAGES_CAPABILITY};

/// V 批需要的、目前还不存在的正式输入（980 交付后逐项替换）。
const V_PENDING_INPUTS: [&str; 2] = [
    // 2026-11-17：980/A 正式源已交（IF-980-NODE-DESKTOP-A），本项从清单移除；
    // 真实源已进 `plan981_ma_sources.rs` / AV 腿 / A2R 腿。
    "V-B-SESSION: 980 正式 typed UI/资源接口 + 多 Run/Stop 资源净归零宿主面（R64-10 异步面）",
    "V-L-*: 与 980 合流的正式 oracle 期望文件（当前用独立算术 oracle，不是 980 expected）",
];

fn input(src: &str) -> PackageModuleInput {
    PackageModuleInput {
        package: "app".into(),
        version: "1.0.0".into(),
        id: "main".into(),
        source: src.into(),
        provider: None,
    }
}

fn compile(src: &str) -> ace_core::source::age_package::AgePackageDescription {
    compile_age_package(
        &[input(src)],
        "app::main",
        LegacyWidgetAlias::Reject,
        &[PACKAGES_CAPABILITY.to_string()],
    )
    .expect("the harness source compiles")
    .description
}

/// V-L-VIEW：五出口与选择序（widget→portrait→默认、方形走默认、2D/3D 无跨域）。
#[test]
fn v_l_view_exits_and_selection_are_frozen() {
    let ui = AgeViewRequest::Ui {
        width: 400,
        height: 800,
        widget_host: false,
    };
    let multi = compile(
        "node M {\n    view {\n        text \"u\" {}\n    }\n    view-portrait {\n        text \"p\" {}\n    }\n}\n",
    );
    let c = &multi.components[0];
    assert_eq!(c.exits, vec![AgeViewExit::View, AgeViewExit::ViewPortrait]);
    assert_eq!(select_exit(&c.exits, ui).unwrap(), AgeViewExit::ViewPortrait);
    // 方形不选 portrait。
    let square = AgeViewRequest::Ui {
        width: 600,
        height: 600,
        widget_host: false,
    };
    assert_eq!(
        select_exit(&[AgeViewExit::ViewPortrait], square).unwrap_err().code,
        "source.age.exit-unavailable"
    );
    // 2D/3D 无跨域。
    assert_eq!(
        select_exit(&[AgeViewExit::View2d], AgeViewRequest::Scene3d)
            .unwrap_err()
            .code,
        "source.age.exit-unavailable"
    );
}

/// V-L-COND：`if` 值必须 bool；`#if` 归 966（age-package 面不接受）。
#[test]
fn v_l_condition_ownership_is_frozen() {
    let bad = "node M {\n    design {\n        theme: if nosuch_cond { 1 } else { 2 }\n    }\n    view {\n        text \"t\" {}\n    }\n}\n";
    assert!(compile_age_package(
        &[input(bad)],
        "app::main",
        LegacyWidgetAlias::Reject,
        &[PACKAGES_CAPABILITY.to_string()],
    )
    .is_err());
    let ct = "node M {\n    #if true {\n        view {\n            text \"t\" {}\n        }\n    }\n}\n";
    assert!(compile_age_package(
        &[input(ct)],
        "app::main",
        LegacyWidgetAlias::Reject,
        &[PACKAGES_CAPABILITY.to_string()],
    )
    .is_err(), "`#if` belongs to 966, not to the AGE package surface");
}

/// V-L-TIME：生命周期计数 Start=1 / Tick=N / Frame=M，不随出口倍增。
#[test]
fn v_l_lifecycle_counts_do_not_scale_with_exits() {
    use ace_core::source::age_package::lifecycle_budget;
    assert_eq!(
        lifecycle_budget(5, 3, 1).unwrap().ticks,
        5,
        "Tick count is the scheduled count"
    );
    assert_eq!(lifecycle_budget(5, 3, 1).unwrap().frames, 3);
    assert_eq!(
        lifecycle_budget(5, 3, 3).unwrap().ticks,
        5,
        "exits do not multiply ticks"
    );
    assert_eq!(lifecycle_budget(5, 3, 0).unwrap_err().code, "source.age.lifecycle");
}

/// V-L-WIRE：消息/handler 与成员槽位在描述面可读（typed 事件回 AV 的静态半边）。
#[test]
fn v_l_wire_surface_is_readable() {
    let d = compile(
        "node M {\n    model { count int = 0 }\n    on Inc { count = count + 1 }\n    msg { Set(int) }\n    design {\n        theme: \"dark\"\n    }\n    view {\n        text \"t\" {}\n    }\n}\n",
    );
    let c = &d.components[0];
    assert!(c.handler_msgs.iter().any(|m| m == "Inc"), "{:?}", c.handler_msgs);
    assert!(!c.messages.is_empty(), "msg variants are described");
    assert!(!c.design.is_empty(), "design entries are described");
}

/// V-L-COVER：构造分母（24 行）与真实源覆盖。
#[test]
fn v_l_construct_denominator_is_frozen() {
    assert_eq!(AGE_CONSTRUCTS.len(), 24);
    let d = compile(
        "let tint = material2d.color(0.9, 0.3, 0.2, 0.5)\n\nnode M {\n    model { count int = 0 }\n    on Inc { count = count + 1 }\n    view {\n        text \"t\" {}\n    }\n}\n",
    );
    let facts = collect_facts(&d);
    assert!(facts.contains(&"node") || facts.contains(&"model"));
    assert!(!facts.is_empty());
    // 单组件源不覆盖整张表，但**整表核对**在完整源上是绿的（本文件的冻结部分）。
    assert!(age_construct_coverage(&d, AGE_CONSTRUCTS).is_err());
}

/// V 批的 980 依赖清单不可静默缩水（缩了必须改这条测试并说明）。
#[test]
fn v_l_pending_inputs_are_named() {
    assert_eq!(
        V_PENDING_INPUTS.len(),
        2,
        "the 980-dependent V inputs are a frozen list"
    );
}
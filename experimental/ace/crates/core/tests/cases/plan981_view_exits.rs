//! PLAN-981（SOURCE 批）：**一个 node 恰一个 view 出口**。
//!
//! 独立复审 major M1：`view_role` 只是被重复赋值，于是两个 `view-2d`（或两个
//! `view`、或 2D+3D 并存）会**静默合并**——后者还把两个出口的行融成一条扁平
//! 列表，丢失"哪些叶属于哪个出口"。规范明说 *single view exit per node*。
//!
//! 现在第二个出口是具名拒绝，且消息里同时给出两个出口的字节位置。
use ace_core::source::age_h0::LegacyWidgetAlias;
use ace_core::source::age_package::compile_age_package;
use ace_core::source::packages::{PackageModuleInput, PACKAGES_CAPABILITY};

fn input(id: &str, source: &str) -> PackageModuleInput {
    PackageModuleInput {
        package: "app".into(),
        version: "1.0.0".into(),
        id: id.into(),
        source: source.into(),
        provider: None,
    }
}

fn compile(src: &str) -> Result<
    ace_core::source::age_package::AgePackageCompilation,
    Vec<ace_core::atom_text::Diagnostic>,
> {
    compile_age_package(
        &[input("main", src)],
        "app::main",
        LegacyWidgetAlias::Reject,
        &[PACKAGES_CAPABILITY.to_string()],
    )
}

/// 两个 `view-2d` 出口。
#[test]
fn p981_duplicate_view2d_is_rejected() {
    let ds = compile(
        "node M {\n    view-2d {\n        shape2d a {\n            kind: quad\n        }\n    }\n    view-2d {\n        shape2d b {\n            kind: quad\n        }\n    }\n}\n",
    )
    .expect_err("a second view-2d is rejected");
    assert_eq!(ds[0].code, "source.age.view-exit", "{ds:?}");
    assert!(
        ds[0].message.contains("exactly one view exit"),
        "{}",
        ds[0].message
    );
}

/// 两个 `view`（文本）出口。
#[test]
fn p981_duplicate_view_is_rejected() {
    let ds = compile(
        "node M {\n    view {\n        text \"a\" {}\n    }\n    view {\n        text \"b\" {}\n    }\n}\n",
    )
    .expect_err("a second view is rejected");
    assert_eq!(ds[0].code, "source.age.view-exit", "{ds:?}");
}

/// 2D 与 3D 并存（规范点名"single view exit per node"）。
#[test]
fn p981_mixed_2d_and_3d_exits_are_rejected() {
    let ds = compile(
        "node M {\n    view-2d {\n        shape2d q {\n            kind: quad\n        }\n    }\n    view-3d {\n        cylinder3d c {\n            position: (0.0, 0.0, 0.0)\n        }\n    }\n}\n",
    )
    .expect_err("view-2d + view-3d is rejected");
    assert_eq!(ds[0].code, "source.age.view-exit", "{ds:?}");
}

/// 单个出口仍然合法（收紧不能破坏正常源）。
#[test]
fn p981_single_exit_is_accepted() {
    for src in [
        "node M {\n    view-2d {\n        shape2d q {\n            kind: quad\n        }\n    }\n}\n",
        "node M {\n    view {\n        text \"t\" {}\n    }\n}\n",
        "node M {\n    view-3d {\n        cylinder3d c {\n            position: (0.0, 0.0, 0.0)\n        }\n    }\n}\n",
        // 没有 view 出口（纯逻辑组件）也合法。
        "node M {\n    model { n int = 0 }\n}\n",
    ] {
        compile(src).unwrap_or_else(|ds| panic!("single/no exit must compile: {ds:?}"));
    }
}

// ---- B-LANGUAGE-I（AGE-L03）：五 view 面 ----

/// `view-portrait` / `view-widget` 是直属呈现出口；出口事实按成员规则落定，
/// 文本叶仍走同一投影（不是只认 `view`）。
#[test]
fn p981_portrait_and_widget_exits_are_recognised() {
    use ace_core::source::age_package::AgeViewExit;
    let out = compile("node M {\n    view-portrait {\n        text \"p\" {}\n    }\n}\n")
        .expect("a view-portrait exit compiles");
    let c = &out.description.components[0];
    assert_eq!(c.exit, AgeViewExit::ViewPortrait, "portrait exit fact");
    assert_eq!(c.texts.len(), 1, "the portrait text leaf is carried");

    let out = compile("node M {\n    view-widget {\n        text \"w\" {}\n    }\n}\n")
        .expect("a view-widget exit compiles");
    let c = &out.description.components[0];
    assert_eq!(c.exit, AgeViewExit::ViewWidget, "widget exit fact");
    assert_eq!(c.texts.len(), 1, "the widget text leaf is carried");
}

/// 五 view 不改变"恰一个出口"：任意两个直属出口并存仍具名拒绝。
#[test]
fn p981_duplicate_portrait_or_widget_exits_are_rejected() {
    for src in [
        "node M {\n    view-portrait {\n        text \"a\" {}\n    }\n    view-portrait {\n        text \"b\" {}\n    }\n}\n",
        "node M {\n    view-portrait {\n        text \"a\" {}\n    }\n    view-widget {\n        text \"b\" {}\n    }\n}\n",
        "node M {\n    view {\n        text \"a\" {}\n    }\n    view-widget {\n        text \"b\" {}\n    }\n}\n",
    ] {
        let ds = compile(src).expect_err("a second exit is rejected");
        assert_eq!(ds[0].code, "source.age.view-exit", "{ds:?}");
    }
}

/// 未知后缀仍在后缀 token 处具名拒绝——新增出口不会把 `view-2x` 合法化。
#[test]
fn p981_unknown_view_suffix_is_still_rejected() {
    let ds = compile(
        "node M {\n    view-2x {\n        shape2d q {\n            kind: quad\n        }\n    }\n}\n",
    )
    .expect_err("view-2x is not an AGE exit");
    assert_eq!(ds[0].code, "source.age.view-suffix", "{ds:?}");
}
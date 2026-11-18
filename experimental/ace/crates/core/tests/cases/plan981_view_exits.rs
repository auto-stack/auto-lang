//! PLAN-981（B-LANGUAGE-I）：**多出口语义**——不同键的直属出口可并存、同键重复拒绝。
//!
//! 独立复审 major M1：`view_role` 只是被重复赋值，于是两个 `view-2d`（或两个
//! `view`、或 2D+3D 并存）会**静默合并**——后者还把两个出口的行融成一条扁平
//! 现在是多出口：不同键可并存（AGE-L03），同键重复仍具名拒绝。
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

fn compile(
    src: &str,
) -> Result<
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
        ds[0].message.contains("declared twice"),
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

/// AGE-L03（多出口版）：不同键的直属出口可以并存；声明事实列出全部出口。
#[test]
fn p981_distinct_exits_coexist_and_are_all_recorded() {
    use ace_core::source::age_package::AgeViewExit;
    let out = compile(
        "node M {\n    view {\n        text \"u\" {}\n    }\n    view-portrait {\n        text \"p\" {}\n    }\n}\n",
    )
    .expect("distinct direct exits coexist");
    let c = &out.description.components[0];
    assert_eq!(
        c.exits,
        vec![AgeViewExit::View, AgeViewExit::ViewPortrait],
        "both declared exits are recorded in declaration order"
    );
    assert_eq!(
        c.exit,
        AgeViewExit::View,
        "the first exit stays the primary fact"
    );
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

/// 同键重复仍拒绝；不同键并存合法（AGE-L03）。
#[test]
fn p981_duplicate_portrait_or_widget_exits_are_rejected() {
    for src in [
        "node M {\n    view-portrait {\n        text \"a\" {}\n    }\n    view-portrait {\n        text \"b\" {}\n    }\n}\n",
        "node M {\n    view-widget {\n        text \"a\" {}\n    }\n    view-widget {\n        text \"b\" {}\n    }\n}\n",
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
/// R60-8：每个文本叶带**出口归属**（多出口并存时不再是一张扁平表）。
#[test]
fn p981_texts_carry_their_exit() {
    use ace_core::source::age_package::AgeViewExit;
    let out = compile(
        "node M {\n    view {\n        text \"u\" {}\n    }\n    view-portrait {\n        text \"p\" {}\n    }\n}\n",
    )
    .expect("distinct exits with texts coexist");
    let c = &out.description.components[0];
    assert_eq!(c.texts.len(), 2, "{c:?}");
    assert_eq!(
        c.text_exits,
        vec![Some(AgeViewExit::View), Some(AgeViewExit::ViewPortrait)],
        "each text leaf knows which exit it belongs to"
    );
}

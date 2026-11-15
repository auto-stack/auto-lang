//! PLAN-981（SOURCE 批，AC-B-SOURCE）：**UI / 2D / 3D 三 view**。
//!
//! AC-B-SOURCE 要求三种 view 都进入同一 verify。此前只有 UI（`view`）与 2D
//! （`view-2d`，来自 g01）有覆盖；3D 从未被验证，且**元素的种类规则与 2D 不同**：
//!
//! - `view-2d` 的叶是 `shape2d`，其种类是属性 `kind: circle|quad`；
//! - `view-3d` 的叶是元素名本身（`cylinder3d` / `model3d` / `group3d`），没有
//!   `kind` 属性。规范里**不存在** `shape3d`。
//!
//! 两套符号表不可互套——这条正是本文件要钉住的不变量。
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

/// UI view（文本叶）走同一前门。
#[test]
fn p981_ui_view_enters_the_shared_checker() {
    let out = compile("node U {\n    view {\n        text \"hello\" {}\n    }\n}\n")
        .expect("UI view compiles");
    let u = &out.description.components[0];
    assert_eq!(u.texts.len(), 1, "the text leaf is carried");
    assert!(u.scene.is_empty(), "a UI view has no scene rows");
}

/// 2D view：叶是 `shape2d`，种类由 `kind:` 属性给出。
#[test]
fn p981_view2d_leaf_is_shape2d_with_a_kind_attribute() {
    let out = compile(
        "node D {\n    view-2d {\n        shape2d q {\n            kind: quad\n            position: (0.0, 0.0)\n            size: (10.0, 10.0)\n        }\n    }\n}\n",
    )
    .expect("2D view compiles");
    let d = &out.description.components[0];
    assert_eq!(d.scene.len(), 1);
    assert_eq!(d.scene[0].kind, "shape2d");
    assert_eq!(d.scene[0].name, "q");
}

/// 3D view：叶是**元素名本身**，`cylinder3d` 按规范成立。
#[test]
fn p981_view3d_cylinder_element_compiles() {
    let out = compile(
        "node T {\n    view-3d {\n        cylinder3d tower {\n            position: (1.0, 2.0, 3.0)\n            visible: true\n        }\n    }\n}\n",
    )
    .expect("3D view compiles");
    let t = &out.description.components[0];
    assert_eq!(t.scene.len(), 1);
    assert_eq!(t.scene[0].kind, "cylinder3d", "the element name IS the kind");
    assert_eq!(t.scene[0].name, "tower");
    assert!(
        t.scene[0].attrs.iter().any(|(k, _)| k == "position"),
        "its three-axis position is carried"
    );
}

/// `group3d` 同样是合法 3D 元素。
#[test]
fn p981_view3d_group_element_compiles() {
    let out = compile(
        "node T {\n    view-3d {\n        group3d g {\n            position: (0.0, 0.0, 0.0)\n        }\n    }\n}\n",
    )
    .expect("group3d compiles");
    assert_eq!(out.description.components[0].scene[0].kind, "group3d");
}

/// `shape3d` **不是** AGE 元素（规范中不存在）——具名拒绝，不静默接受。
#[test]
fn p981_shape3d_is_not_an_age_element() {
    let ds = compile(
        "node T {\n    view-3d {\n        shape3d box {\n            position: (0.0, 0.0, 0.0)\n        }\n    }\n}\n",
    )
    .expect_err("shape3d is not part of the AGE 3D surface");
    assert_eq!(ds[0].code, "source.age.scene-element", "{ds:?}");
    assert!(ds[0].message.contains("shape3d"), "{}", ds[0].message);
}

/// 两套种类表不可互套：`kind:` 是 2D 属性，放到 3D 叶上必须具名拒绝
/// （而不是把 `circle` 当成 3D 种类静默接受）。
#[test]
fn p981_kind_attribute_is_2d_only() {
    let ds = compile(
        "node T {\n    view-3d {\n        cylinder3d c {\n            kind: circle\n        }\n    }\n}\n",
    )
    .expect_err("`kind` does not apply to a 3D element");
    assert_eq!(ds[0].code, "source.age.scene-attr", "{ds:?}");

    // 反向：2D 叶用 3D 元素名当 kind，也被几何符号表拒绝。
    let ds2 = compile(
        "node D {\n    view-2d {\n        shape2d q {\n            kind: cylinder3d\n        }\n    }\n}\n",
    )
    .expect_err("a 3D element name is not a 2D geometry kind");
    assert_eq!(ds2[0].code, "source.age.scene-kind", "{ds2:?}");
}

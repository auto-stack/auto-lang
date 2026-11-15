//! PLAN-981（SOURCE 批）：**场景属性的形状纪律**。
//!
//! 三个此前**静默接受**的构造，实测都通过了编译：
//!
//! 1. 同一行内**重复属性键**（`kind: quad` + `kind: circle`）——后值覆盖前值、零诊断；
//! 2. **属性不适用于该元素**（`yaw:` 写在 `shape2d` 上、`size:` 写在 `cylinder3d` 上）；
//! 3. 非 bool 的 `visible:`。
//!
//! 这些是同一族的最后一批：判据只看"名字认得"，不看"该不该出现在这儿"和"写了几次"。
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

/// 同一行重复写同一个属性名 → 具名拒绝（不是后值静默胜出）。
#[test]
fn p981_duplicate_attribute_key_is_rejected() {
    let ds = compile(
        "node M {\n    view-2d {\n        shape2d q {\n            kind: quad\n            kind: circle\n        }\n    }\n}\n",
    )
    .expect_err("a duplicated attribute key is rejected");
    assert_eq!(ds[0].code, "source.age.scene-attr-duplicate", "{ds:?}");
    assert!(
        ds[0].message.contains("twice"),
        "the diagnostic must say it was written twice: {}",
        ds[0].message
    );
}

/// 3D 专属属性写到 2D 叶上 → 具名拒绝。
#[test]
fn p981_3d_attribute_on_a_2d_leaf_is_rejected() {
    let ds = compile(
        "node M {\n    view-2d {\n        shape2d q {\n            kind: quad\n            yaw: 1.0\n        }\n    }\n}\n",
    )
    .expect_err("`yaw` does not apply to shape2d");
    assert_eq!(ds[0].code, "source.age.scene-attr", "{ds:?}");
    assert!(ds[0].message.contains("does not apply"), "{}", ds[0].message);
}

/// 2D 专属属性写到 3D 叶上 → 具名拒绝。
#[test]
fn p981_2d_attribute_on_a_3d_leaf_is_rejected() {
    let ds = compile(
        "node M {\n    view-3d {\n        cylinder3d c {\n            position: (0.0, 0.0, 0.0)\n            size: (1.0, 1.0)\n        }\n    }\n}\n",
    )
    .expect_err("`size` does not apply to cylinder3d");
    assert_eq!(ds[0].code, "source.age.scene-attr", "{ds:?}");
}

/// 正确的元素/属性搭配仍然合法（收紧不能破坏正常源）。
#[test]
fn p981_valid_element_attribute_pairs_still_compile() {
    // 3D 叶用 yaw
    compile(
        "node M {\n    view-3d {\n        cylinder3d c {\n            position: (0.0, 0.0, 0.0)\n            yaw: 1.5\n        }\n    }\n}\n",
    )
    .expect("yaw is legal on cylinder3d");
    // 2D 叶用 position/size/visible
    compile(
        "node M {\n    view-2d {\n        shape2d q {\n            kind: circle\n            position: (1.0, 2.0)\n            size: (3.0, 3.0)\n            visible: false\n        }\n    }\n}\n",
    )
    .expect("the normal shape2d form is legal");
    // 组件实例用资源（真正的资源，不是字面量）
    compile(
        "let red = material2d.color(0.9, 0.3, 0.2, 0.5)\n\
node Disc(tint: Material2D) {\n    view-2d {\n        shape2d d {\n            kind: circle\n            material: tint\n        }\n    }\n}\n\
node M {\n    view-2d {\n        Disc inst {\n            tint: red\n        }\n    }\n}\n",
    )
    .expect("a component instance bound to a real resource is legal");
}

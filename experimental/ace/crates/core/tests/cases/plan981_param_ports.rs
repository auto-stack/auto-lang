//! PLAN-981（SOURCE 批）：**node 参数作资源时也必须端口类型正确**。
//!
//! 独立复审 blocker B4：`material: tint` 里的 `tint` 是 node 自己的参数，此前被
//! 无条件当成"实例化时才绑定、无需检查"而跳过——于是
//! `node Disc(tint: int)` 把一个整数塞进 `Material2D` 位也无人过问。
//!
//! 参数确实不是模块全局资源，但它**仍然**必须是该位置期望的那个端口。这条不变量
//! 与"具名资源必须是该端口"是同一个判据（`attr_port`），只是事实来源不同。
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

/// 非端口类型的参数不能充当 `material:`。
#[test]
fn p981_int_parameter_cannot_be_a_material() {
    let ds = compile(
        "node Disc(x: float, tint: int) {\n    view-2d {\n        shape2d d {\n            kind: circle\n            material: tint\n        }\n    }\n}\n",
    )
    .expect_err("an int parameter is not a Material2D");
    assert_eq!(ds[0].code, "source.age.scene-resource-type", "{ds:?}");
    assert!(ds[0].message.contains("int"), "{}", ds[0].message);
}

/// 端口之间也不能互换：`Mesh2D` 参数不能当 `Material2D`。
#[test]
fn p981_mesh_parameter_cannot_be_a_material() {
    let ds = compile(
        "node Disc(tint: Mesh2D) {\n    view-2d {\n        shape2d d {\n            kind: circle\n            material: tint\n        }\n    }\n}\n",
    )
    .expect_err("a Mesh2D parameter is not a Material2D");
    assert_eq!(ds[0].code, "source.age.scene-resource-type", "{ds:?}");
    assert!(ds[0].message.contains("Mesh2D"), "{}", ds[0].message);
}

/// 正确端口类型的参数仍然合法。
#[test]
fn p981_material2d_parameter_is_accepted() {
    let out = compile(
        "node Disc(tint: Material2D) {\n    view-2d {\n        shape2d d {\n            kind: circle\n            material: tint\n        }\n    }\n}\n",
    )
    .expect("a Material2D parameter is the right port");
    let d = out
        .description
        .components
        .iter()
        .find(|c| c.name == "Disc")
        .expect("Disc");
    assert_eq!(d.scene.len(), 1);
}

/// g01 的真实形状（`material: tint`，`tint: Material2D`）必须继续通过——
/// 收紧不能破坏原源。
#[test]
fn p981_g01_parameter_shape_still_compiles() {
    let out = compile(
        "node Disc(x: float, y: float, diameter: float, tint: Material2D) {\n    view-2d {\n        shape2d disc {\n            kind: circle\n            position: (x, y)\n            size: (diameter, diameter)\n            material: tint\n        }\n    }\n}\n",
    )
    .expect("the original g01 component shape compiles");
    let d = out
        .description
        .components
        .iter()
        .find(|c| c.name == "Disc")
        .expect("Disc");
    assert_eq!(d.params.len(), 4);
}

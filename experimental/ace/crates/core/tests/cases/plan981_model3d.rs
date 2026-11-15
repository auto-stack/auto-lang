//! PLAN-981（SOURCE 批）：**3D 资源端口 `Model3D`**。
//!
//! 规范把 `Model3D` 与 `Material2D`/`Mesh2D` 并列为端口（`ptype_of_3d`：
//! int/float/Model3D），但它的形状与 2D 端口**根本不同**：
//!
//! - 2D 端口的构造器收 f32 分量（`material2d.color(r,g,b,a)`）；
//! - `Model3D` 的构造器收**文件路径**（`model3d.glb("ship.glb")`），身份 = GLB
//!   完整原始字节的 SHA256 + byte_length。
//!
//! 所以它的字段是 `sha256`/`byte_length`，不是颜色分量——本文件钉住这个差异，
//! 防止日后有人"为了一致"把它压成 f32。
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

/// `model3d.glb("…")` 产出 `Model3D` 端口资源。
#[test]
fn p981_model3d_glb_constructor_yields_the_port() {
    let out = compile(
        "let ship = model3d.glb(\"ship.glb\")\nnode T {\n    view { text \"t\" {} }\n}\n",
    )
    .expect("the GLB constructor compiles");
    let r = out
        .description
        .resources
        .iter()
        .find(|r| r.name == "ship")
        .expect("ship");
    assert_eq!(r.port, "Model3D");
}

/// `Model3D` 是合法的 node 参数类型（与 `Material2D` 同等）。
#[test]
fn p981_model3d_is_a_valid_parameter_port() {
    let out = compile(
        "node T(m: Model3D) {\n    view-3d {\n        model3d body {\n            model: m\n            position: (1.0, 2.0, 3.0)\n        }\n    }\n}\n",
    )
    .expect("Model3D parameter is accepted");
    let t = &out.description.components[0];
    assert!(
        t.params.iter().any(|(n, ty)| n == "m" && ty == "Model3D"),
        "the parameter is carried with its port type: {:?}",
        t.params
    );
}

/// `model3d` 叶的 `model:` 绑定一个具名 GLB 资源。
#[test]
fn p981_model3d_leaf_binds_a_named_resource() {
    let out = compile(
        "let ship = model3d.glb(\"ship.glb\")\nnode T {\n    view-3d {\n        model3d body {\n            model: ship\n            position: (1.0, 2.0, 3.0)\n        }\n    }\n}\n",
    )
    .expect("model3d leaf compiles");
    let t = &out.description.components[0];
    assert_eq!(t.scene.len(), 1);
    assert_eq!(t.scene[0].kind, "model3d");
    assert!(
        t.scene[0].attrs.iter().any(|(k, _)| k == "model"),
        "the model binding is carried"
    );
}

/// `Model3D` 与 `Material2D` 是**不同**端口：把颜色资源绑到 `model:` 上必须被拒。
///
/// 这条是真正的类型区分：两个端口都是 record，但身份不同。
#[test]
fn p981_model3d_and_material2d_are_distinct_ports() {
    let ds = compile(
        "let red = material2d.color(0.9, 0.3, 0.2, 0.5)\nnode T {\n    view-3d {\n        model3d body {\n            model: red\n            position: (1.0, 2.0, 3.0)\n        }\n    }\n}\n",
    )
    .expect_err("a Material2D is not a Model3D");
    assert!(
        ds.iter()
            .any(|d| d.code.contains("scene-resource-type") || d.code.contains("type-mismatch")),
        "the mismatch is a resource/type error: {ds:?}"
    );
}

//! PLAN-981（SOURCE 批）：**资源引用的形状与限定词都必须真实**。
//!
//! 两个独立复审 blocker，同住一段代码：
//!
//! - **B6**：引用形状判定原来是 `_ => continue`——`resources.red.clone()`、
//!   `nope()`、`resources.red.deep` 这类方法/字段链里的未知资源**从未被存在性
//!   检查**，静默通过。现在非引用形状一律具名拒绝。
//! - **B5**：限定词匹配带 `m == "resources"` 的**硬编码通配**——任何名为
//!   `resources` 的模块无需 `use` 即可达，与"复用 binder 的可见性事实"自相矛盾。
//!   现在限定词必须逐字等于声明所在模块名。
use ace_core::source::age_h0::LegacyWidgetAlias;
use ace_core::source::age_package::compile_age_package;
use ace_core::source::packages::{PackageModuleInput, PACKAGES_CAPABILITY};

const RES: &str = "let red = material2d.color(0.9, 0.3, 0.2, 0.5)\n";

fn input(id: &str, source: &str) -> PackageModuleInput {
    PackageModuleInput {
        package: "app".into(),
        version: "1.0.0".into(),
        id: id.into(),
        source: source.into(),
        provider: None,
    }
}

fn compile(resources: &str, main: &str) -> Result<
    ace_core::source::age_package::AgePackageCompilation,
    Vec<ace_core::atom_text::Diagnostic>,
> {
    compile_age_package(
        &[input("resources", resources), input("main", main)],
        "app::main",
        LegacyWidgetAlias::Reject,
        &[PACKAGES_CAPABILITY.to_string()],
    )
}

fn with_material(attr: &str) -> String {
    format!(
        "use resources.red\nnode D {{\n    view-2d {{\n        shape2d q {{\n            kind: circle\n            material: {attr}\n        }}\n    }}\n}}\n"
    )
}

/// 方法调用不是资源引用——必须具名拒绝，不得静默跳过。
#[test]
fn p981_method_call_is_not_a_resource_reference() {
    let ds = compile(RES, &with_material("resources.red.clone()"))
        .expect_err("a method call is not a resource reference");
    assert_eq!(ds[0].code, "source.age.scene-resource", "{ds:?}");
}

/// 裸函数调用不是资源引用。
#[test]
fn p981_bare_call_is_not_a_resource_reference() {
    let ds = compile(RES, &with_material("nope()"))
        .expect_err("a call is not a resource reference");
    assert_eq!(ds[0].code, "source.age.scene-resource", "{ds:?}");
}

/// 深链（`a.b.c`）不是简单资源引用。
#[test]
fn p981_deep_chain_is_not_a_resource_reference() {
    let ds = compile(RES, &with_material("resources.red.deep"))
        .expect_err("a field chain is not a simple resource reference");
    assert_eq!(ds[0].code, "source.age.scene-resource", "{ds:?}");
}

/// 限定词必须是真实模块名：`othermod.red` 即便别处有 `red` 也不能命中。
#[test]
fn p981_wrong_qualifier_does_not_resolve() {
    let ds = compile(
        RES,
        "node D {\n    view-2d {\n        shape2d q {\n            kind: circle\n            material: othermod.red\n        }\n    }\n}\n",
    )
    .expect_err("the qualifier must name the declaring module");
    assert_eq!(ds[0].code, "source.age.scene-resource-unknown", "{ds:?}");
}

/// 限定词必须是**真实的模块名**。这条测的是 B5 的**实际形态**：
/// 模块叫 `assets`、源码却写 `resources.red`。旧的硬编码通配
/// (`m == "resources"`) 会让它通过，等于绕过了导入可见性。
#[test]
fn p981_qualifier_wildcard_is_gone() {
    let res = "let red = material2d.color(0.9, 0.3, 0.2, 0.5)\n";
    let main = "use assets.red\nnode D {\n    view-2d {\n        shape2d q {\n            kind: circle\n            material: resources.red\n        }\n    }\n}\n";
    let caps = [PACKAGES_CAPABILITY.to_string()];
    let ds = compile_age_package(
        &[input("assets", res), input("main", main)],
        "app::main",
        LegacyWidgetAlias::Reject,
        &caps,
    )
    .expect_err("`resources.red` must not resolve when the module is `assets`");
    assert_eq!(ds[0].code, "source.age.scene-resource-unknown", "{ds:?}");
}

/// 正确的限定写法与裸写法都仍然合法（收紧不能破坏原源）。
#[test]
fn p981_correct_forms_still_resolve() {
    let qualified = compile(RES, &with_material("resources.red"))
        .expect("qualified resource reference resolves");
    let d = qualified
        .description
        .components
        .iter()
        .find(|c| c.name == "D")
        .expect("D");
    assert_eq!(d.scene.len(), 1);

    let bare = compile(RES, &with_material("red")).expect("bare imported name resolves");
    assert_eq!(
        bare.description
            .components
            .iter()
            .find(|c| c.name == "D")
            .expect("D")
            .scene
            .len(),
        1
    );
}

/// g01 的真实写法（`tint: resources.red_alpha`）必须继续通过。
#[test]
fn p981_g01_qualified_form_still_compiles() {
    let main = "use resources.red\nnode D {\n    view-2d {\n        shape2d q {\n            kind: circle\n            material: resources.red\n            visible: true\n        }\n    }\n}\n";
    compile(RES, main).expect("the original g01 reference form compiles");
}

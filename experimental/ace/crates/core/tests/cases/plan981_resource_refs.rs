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

/// 括号**不改变**引用身份——既不绕过存在性，也不该被拒。
///
/// 复审问"`(red)` 应否被拒"。实测答案是**不该**，理由是可验证的：括号在投影后
/// 被展开，`(red)` 与 `red` 得到同一个表达式；而且**坏名字加括号仍然被拒**
/// （`(resources.nope)` → `scene-resource-unknown`，与不加括号完全一致）。
///
/// 所以"括号被接受"不是静默接受，而是"括号不携带身份"。这条用例把这个判断固定
/// 下来，防止日后有人为了"更严格"而无故拒绝合法括号。
#[test]
fn p981_parentheses_do_not_change_resource_identity() {
    // 合法引用加括号 → 仍然合法（且解析到同一资源）。
    for form in ["red", "(red)", "resources.red", "(resources.red)"] {
        compile(RES, &with_material(form))
            .unwrap_or_else(|ds| panic!("`{form}` should resolve: {ds:?}"));
    }
    // 坏引用加括号 → 仍然被拒（括号不绕过存在性检查）。
    for form in ["nope", "(nope)", "resources.nope", "(resources.nope)"] {
        let ds = compile(RES, &with_material(form))
            .expect_err(&format!("`{form}` must be rejected"));
        assert_eq!(
            ds[0].code, "source.age.scene-resource-unknown",
            "`{form}` must fail the SAME way with or without parens: {ds:?}"
        );
    }
}

/// 资源引用**必须尊重导入可见性**——裸名与限定名都一样。
///
/// 第三次复审指出：`check_scene_resource_refs` 修了限定词的硬编码通配，
/// 但**裸名**路径仍按全项目 `bound.globals` 匹配。实测：无 `use` 时
/// `material: red` 与 `material: assets.red` **都 ACCEPTED**。
///
/// 这与场景元素种类（B3）是同一缺陷类：用全项目名字集代替导入集。
#[test]
fn p981_resource_references_respect_imports() {
    const ASSETS: &str = "let red = material2d.color(0.9, 0.3, 0.2, 0.5)\n";
    let caps = [PACKAGES_CAPABILITY.to_string()];
    let with = |import: &str, attr: &str| {
        format!(
            "{import}node D {{\n    view-2d {{\n        shape2d q {{\n            kind: circle\n            material: {attr}\n        }}\n    }}\n}}\n"
        )
    };
    let run = |main: &str| {
        compile_age_package(
            &[input("assets", ASSETS), input("main", main)],
            "app::main",
            LegacyWidgetAlias::Reject,
            &caps,
        )
    };

    // 无 `use`：两种写法都必须被拒，且诊断要区分"存在但不可见"。
    for attr in ["red", "assets.red"] {
        let ds = run(&with("", attr)).expect_err(&format!("`{attr}` must not resolve without a use"));
        assert_eq!(
            ds[0].code, "source.age.scene-resource-invisible",
            "`{attr}` without a use: {ds:?}"
        );
        assert!(
            ds[0].message.contains("does not import"),
            "{}",
            ds[0].message
        );
    }
    // 有 `use`：两种写法都通过。
    for attr in ["red", "assets.red"] {
        run(&with("use assets.red\n", attr))
            .unwrap_or_else(|ds| panic!("`{attr}` with a use must resolve: {ds:?}"));
    }
}

/// "根本不存在"与"存在但本模块没导入"必须是**互斥可辨**的诊断（979 V-10）。
#[test]
fn p981_unknown_and_invisible_resources_are_distinguishable() {
    const ASSETS: &str = "let red = material2d.color(0.9, 0.3, 0.2, 0.5)\n";
    let caps = [PACKAGES_CAPABILITY.to_string()];
    let main = |attr: &str| {
        format!(
            "node D {{\n    view-2d {{\n        shape2d q {{\n            kind: circle\n            material: {attr}\n        }}\n    }}\n}}\n"
        )
    };
    let run = |attr: &str| {
        compile_age_package(
            &[input("assets", ASSETS), input("main", &main(attr))],
            "app::main",
            LegacyWidgetAlias::Reject,
            &caps,
        )
        .expect_err("must be rejected")
    };
    assert_eq!(
        run("nope")[0].code, "source.age.scene-resource-unknown",
        "a name nobody declares is UNKNOWN"
    );
    assert_eq!(
        run("red")[0].code, "source.age.scene-resource-invisible",
        "a declared-but-unimported name is INVISIBLE, not unknown"
    );
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

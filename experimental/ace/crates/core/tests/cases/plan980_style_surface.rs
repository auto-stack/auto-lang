//! PLAN-980 P3（981 回签 `edfa4220c`）：`style` 声明面永久用例（**age-package only**）。
//!
//! 语义=str const/fn 糖。判据按回签逐点：
//! - 零参字面量 → const `Str`；`pub style` → 公共面；
//! - 本地 ident 复制及导入的公共字面量 style 可折叠，带参调用仍报 `style-call`；
//! - 参数化（无默认、无插值）→ fn 返回 `Str`；默认值 → `style-defaults`；插值 → `style-interp`；
//! - 重名 → `source.module-duplicate-declaration`；`style` 作普通标识符仍合法；
//! - 用法 `style: NAME` 在投影期替换为字面量（981 (c)3：不引入 module-global 读）。
use ace_core::source::age_h0::LegacyWidgetAlias;
use ace_core::source::age_package::compile_age_package;
use ace_core::source::packages::{PackageModuleInput, PACKAGES_CAPABILITY};

fn unit(pkg: &str, id: &str, src: &str) -> PackageModuleInput {
    PackageModuleInput {
        package: pkg.into(),
        version: "1.0.0".into(),
        id: id.into(),
        source: src.into(),
        provider: None,
    }
}

fn caps() -> Vec<String> {
    vec![PACKAGES_CAPABILITY.to_string()]
}

const NODE: &str = "node App {\n    view {\n        text \"x\" {}\n    }\n}\n";

fn main_with(style_lines: &str) -> String {
    format!("{style_lines}\n{NODE}")
}

fn compile(
    main: &str,
) -> Result<
    ace_core::source::age_package::AgePackageCompilation,
    Vec<ace_core::atom_text::Diagnostic>,
> {
    compile_age_package(
        &[unit("app", "main", main)],
        "app::main",
        LegacyWidgetAlias::Accept,
        &caps(),
    )
}

/// ①字面量 → const `Str`；`pub style` → 公共面；本地 ident 复制 → 同值拷贝。
#[test]
fn p980_style_literal_pub_and_local_copy() {
    let out = compile(&main_with(
        "pub style card = \"bg-card rounded\"\nstyle alias_card = card\n",
    ))
    .expect("literal + pub + local copy compile");
    assert_eq!(out.description.components.len(), 1);
    assert!(
        out.atom_text.contains("bg-card"),
        "the literal survives into the checked program: {}",
        &out.atom_text[..out.atom_text.len().min(400)]
    );
}

/// ②参数化（无默认、无插值）→ fn 返回 `Str`。
#[test]
fn p980_style_param_recipe_without_default_compiles() {
    compile(&main_with("style sized(p: str) = \"text-sm\"\n"))
        .expect("parameterised style without defaults compiles");
}

/// ③默认值 → `style-defaults`（指向默认值 token 的具名拒）。
#[test]
fn p980_style_default_parameter_is_named() {
    let err = compile(&main_with("style sized(p: str = \"text-sm\") = \"x\"\n"))
        .expect_err("default parameters are a named pending surface");
    assert_eq!(err[0].code, "source.age.style-defaults", "{err:?}");
}

/// ④参数插值 → `style-interp`（本刀具名边界）。
#[test]
fn p980_style_interpolation_is_named() {
    let err = compile(&main_with("style sized(p: str) = \"{p} x\"\n"))
        .expect_err("parameter interpolation is a named pending surface");
    assert_eq!(err[0].code, "source.age.style-interp", "{err:?}");
}

/// ⑤用法：同模块 `style: NAME` 投影期替换为字面量（下游无 global 读）。
#[test]
fn p980_style_usage_is_replaced_with_the_literal() {
    let src = "style card = \"bg-card rounded\"\n\nnode App {\n    view {\n        text \"x\" { style: card }\n    }\n}\n";
    let out = compile(src).expect("usage replacement compiles");
    assert!(
        out.atom_text.contains("bg-card"),
        "literal appears in the checked program"
    );
}

#[test]
fn plan125_text_literal_parentheses_keep_the_label_and_style() {
    use ace_core::source::age_component::{
        project_component_candidates, ComponentEntryValue, ComponentViewItem,
    };
    use ace_core::source::ast::ExprKind;
    let source = r#"
widget App {
    view {
        text "▸" (style: "duration-200") {}
        text "Next" {}
    }
}
"#;
    let nodes = project_component_candidates(source).expect("component projection accepts text props");
    let [ComponentViewItem::Element(styled), ComponentViewItem::Text(next)] =
        nodes[0].view.tree.as_slice()
    else {
        panic!("styled text and sibling project independently: {:?}", nodes[0].view.tree);
    };
    assert_eq!(styled.tag, "text");
    assert!(matches!(&styled.label.as_ref().unwrap().kind, ExprKind::StrLit(text) if text == "▸"));
    assert!(matches!(&styled.items[..], [ComponentViewItem::Entry(entry)]
        if entry.key == "style" && matches!(&entry.value,
            ComponentEntryValue::Expr(expr) if matches!(&expr.kind, ExprKind::StrLit(v) if v == "duration-200"))));
    assert!(matches!(&next.expr.kind, ExprKind::StrLit(text) if text == "Next"));
    compile(source).expect("the same form verifies on the package path");
}

/// ⑥公共字面量样式以受检包导入的身份绑定。
#[test]
fn p980_style_imported_reference_is_literal() {
    use ace_core::source::age_component::{ComponentEntryValue, ComponentViewItem};
    use ace_core::source::ast::ExprKind;
    let utils =
        "pub style base = \"bg-base\"\nnode U {\n    view {\n        text \"u\" {}\n    }\n}\n";
    let main =
        "use utils: base\n\nnode App {\n    view {\n        text \"x\" { style: base }\n    }\n}\n";
    let package = compile_age_package(
        &[unit("app", "main", main), unit("app", "utils", utils)],
        "app::main",
        LegacyWidgetAlias::Accept,
        &caps(),
    )
    .expect("checked public style import");
    let node = &package.bound.units["app::main"].age_components[0].0;
    let [ComponentViewItem::Element(element)] = node.view.tree.as_slice() else {
        panic!("expected a styled text element");
    };
    assert!(matches!(&element.items[..], [ComponentViewItem::Entry(entry)]
        if entry.key == "style" && matches!(&entry.value, ComponentEntryValue::Expr(expr)
            if matches!(&expr.kind, ExprKind::StrLit(text) if text == "bg-base"))));

    let private = utils.replace("pub style", "style");
    let diagnostic = compile_age_package(
        &[unit("app", "main", main), unit("app", "utils", &private)],
        "app::main",
        LegacyWidgetAlias::Accept,
        &caps(),
    ).expect_err("styles use package export visibility");
    assert_eq!(diagnostic[0].code, "source.module-private");
}

#[test]
fn p107_public_style_dependencies_only_require_referenced_recipes() {
    let styles = r#"
pub style gap = "gap-1"
pub style fancy(bg: str = "blue") = "bg-{bg}"
"#;
    let main = r#"
use stylekit.styles: gap, fancy
node App {
    view { text "Chart" { style: gap } }
}
"#;
    let import = |src| compile_age_package(
        &[unit("app", "main", src), unit("stylekit", "styles", styles)],
        "app::main",
        LegacyWidgetAlias::Accept,
        &caps(),
    );
    let package = import(main).expect("unused dependency recipe does not block a literal");
    assert!(package.description.components.iter().any(|component| component.name == "App"));
    let used = main.replace("style: gap", "style: fancy");
    let diagnostic = import(&used).expect_err("referenced unsupported recipe is rejected");
    assert_eq!(diagnostic[0].code, "source.age.style-defaults");
    assert!(diagnostic[0].message.contains("module `stylekit::styles`"));
}

/// ⑦带参调用形 → `style-call` 具名拒（不折叠）。
#[test]
fn p980_style_call_form_is_named() {
    let src = "style sized(p: str) = \"text-sm\"\n\nnode App {\n    view {\n        text \"x\" { style: sized(p: \"md\") }\n    }\n}\n";
    let err = compile(src).expect_err("parameterised style calls are not folded");
    assert_eq!(err[0].code, "source.age.style-call", "{err:?}");
}

/// ⑧重名 → 现有重复声明码（不新造码）。
#[test]
fn p980_style_duplicate_uses_the_existing_code() {
    let err = compile(&main_with("style a = \"x\"\nstyle a = \"y\"\n"))
        .expect_err("duplicate style names share the declaration namespace");
    assert_eq!(
        err[0].code, "source.module-duplicate-declaration",
        "{err:?}"
    );
}

/// ⑨`style` 作普通标识符仍合法（不进 token 表）。
#[test]
fn p980_style_word_stays_a_legal_identifier() {
    let src = "node App {\n    model {\n        var style int = 1\n    }\n    view {\n        text \"x\" {}\n    }\n}\n";
    compile(src).expect("`style` is not reserved outside the declaration head");
}

//! PLAN-810 V-06 轻文法测试：无 auto-lang 闭包内驱动 auto-ui-dialect
//! **完整 UI 文法**真算法（widget/store/fragment/style recipe 族）。

use auto_parser_api::DomainParseContext;
use auto_ui_dialect::grammar::{actions, store, view, widget};
use auto_ui_dialect::UiStmt;
use ui_grammar_consumer::ToyUiContext;

#[test]
fn widget_full_grammar_on_toy_port() {
    let cx_src = r#"widget Board(count: int) {
    msg { Inc }
    model { #[primary] var count int = 0 }
    computed { doubled => .count }
    watch { .count -> { } }
    on { .Inc -> { } }
    style { .b { color: red } }
    view { text "hi" }
}
"#;
    let mut cx = ToyUiContext::new(cx_src);
    let UiStmt::WidgetDecl(w) = widget::parse_widget_decl(&mut cx).expect("toy widget parse")
    else {
        panic!("expected WidgetDecl")
    };
    assert_eq!(w.name.as_str(), "Board");
    assert_eq!(w.props.len(), 1);
    assert_eq!(w.messages.len(), 1);
    assert!(w.model.expect("model").fields[0].is_primary);
    assert!(w.computed.is_some());
    assert_eq!(w.watch.len(), 1);
    assert_eq!(w.on.expect("on").handlers.len(), 1);
    let css = w.style.expect("style");
    assert!(css.contains("color"), "css: {css:?}");
    assert!(css.contains("red"), "css: {css:?}");
    assert!(matches!(
        w.view.expect("view").root,
        auto_ui_dialect::ast::ViewNode::Element { .. }
    ));
    assert!(cx.errors().is_empty(), "stray errors: {:?}", cx.errors());
}

#[test]
fn store_decl_on_toy_port() {
    let src = r#"store Kept {
    msg { Bump, Tick }
    model { var n int = 0 }
    computed { shown => .n }
    on { .Bump -> { } }
    timer { Tick (every_ms: 1000) }
}
"#;
    let mut cx = ToyUiContext::new(src);
    let UiStmt::StoreDecl(st) = store::parse_store_decl(&mut cx).expect("toy store parse") else {
        panic!("expected StoreDecl")
    };
    assert_eq!(st.name.as_str(), "Kept");
    assert!(st.model.is_some());
    assert!(st.computed.is_some());
    assert!(st.on.is_some());
    assert!(st.timer.is_some());
}

#[test]
fn view_fragment_binds_params_via_port() {
    let src = "view fn Chip(active: int) { text active }";
    let mut cx = ToyUiContext::new(src);
    let UiStmt::ViewFragmentDecl(f) =
        view::parse_view_block_or_fragment(&mut cx).expect("toy fragment parse")
    else {
        panic!("expected ViewFragmentDecl")
    };
    assert_eq!(f.name.as_str(), "Chip");
    assert_eq!(f.params.len(), 1);
    assert!(cx.binds().iter().any(|(n, _)| n == "active"));
}

#[test]
fn style_recipe_returns_ui_owned_declaration() {
    let mut cx = ToyUiContext::new("style card = .base");
    let stmt = actions::parse_style_recipe_decl_stmt(&mut cx).expect("toy recipe parse");
    let Some(UiStmt::StyleRecipeDecl(recipe)) = stmt else {
        panic!("expected StyleRecipeDecl")
    };
    assert_eq!(recipe.name.as_str(), "card");
}

#[test]
fn style_recipe_none_fallback_no_side_effect() {
    let mut cx = ToyUiContext::new("style ( 1 )");
    let out = actions::parse_style_recipe_decl_stmt(&mut cx).expect("parse ok");
    assert!(out.is_none(), "non-decl style must not be claimed");
}

#[test]
fn port_object_safety_and_upcast() {
    let mut cx = ToyUiContext::new("msg { Inc }");
    let wide: &mut dyn DomainParseContext = &mut cx;
    let decl = auto_ui_dialect::parse_msg_decl(wide).expect("806 single source via upcast");
    assert_eq!(decl.variants.len(), 1);
}

//! PLAN-810 V-06 轻文法测试：无 auto-lang 闭包内驱动 auto-ui-dialect
//! **完整 UI 文法**真算法（widget/store/fragment/style recipe 族）。
//! 字节级类型/表达式/诊断位置保真与全部生产路径对照在上层 driver
//!（crates/auto-lang/src/tests/plan810_full_ui_grammar.rs）。

use auto_ui_dialect::grammar::{actions, scene, store, view, widget};
use auto_ui_dialect::UiParseContext;
use ui_grammar_consumer::ToyUiContext;

fn widget_decl(src: &str) -> auto_parser_api::error::AutoResult<auto_parser_api::ast::Stmt> {
    widget::parse_widget_decl(&mut ToyUiContext::new(src))
}

/// widget 全块：msg/model/computed/watch/on/view/style(raw CSS 重建)——
/// 经 toy UiParseContext 端口产出与真 Parser 同一 WidgetDecl 类型。
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
    let stmt = widget::parse_widget_decl(&mut cx).expect("toy widget parse");
    let auto_parser_api::ast::Stmt::WidgetDecl(w) = stmt else {
        panic!("expected WidgetDecl")
    };
    assert_eq!(w.name.as_str(), "Board");
    assert_eq!(w.props.len(), 1);
    assert_eq!(w.messages.len(), 1);
    let model = w.model.expect("model");
    assert!(model.fields[0].is_primary);
    assert!(w.computed.is_some());
    assert_eq!(w.watch.len(), 1);
    assert_eq!(w.on.expect("on").handlers.len(), 1);
    // 受限 raw CSS 端口：token 重建（内容保真在上层 driver 钉住）。
    let css = w.style.expect("style");
    assert!(css.contains("color"), "css: {css:?}");
    assert!(css.contains("red"), "css: {css:?}");
    assert!(matches!(
        w.view.expect("view").root,
        auto_parser_api::ast::ui::ViewNode::Element { .. }
    ));
    // 副作用观察口：on 处理器参数 define、诊断无投机残留。
    assert!(cx.defined().is_empty() || cx.defined().iter().all(|p| !p.is_empty()));
    assert!(cx.errors().is_empty(), "stray errors: {:?}", cx.errors());
}

/// store 声明族：msg/model/computed/on/timer 经同一端口（timer 校验
/// 需 msg 变体命中——真校验算法在文法 crate 内）。
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
    let stmt = store::parse_store_decl(&mut cx).expect("toy store parse");
    let auto_parser_api::ast::Stmt::StoreDecl(st) = stmt else {
        panic!("expected StoreDecl")
    };
    assert_eq!(st.name.as_str(), "Kept");
    assert!(st.model.is_some());
    assert!(st.computed.is_some());
    assert!(st.on.is_some());
    assert!(st.timer.is_some());
}

/// view fragment：参数进入 bind_var 观察口（解析期符号面端口路径）。
#[test]
fn view_fragment_binds_params_via_port() {
    let src = "view fn Chip(active: int) { text active }";
    let mut cx = ToyUiContext::new(src);
    let stmt = view::parse_view_block_or_fragment(&mut cx).expect("toy fragment parse");
    let auto_parser_api::ast::Stmt::ViewFragmentDecl(f) = stmt else {
        panic!("expected ViewFragmentDecl, got {stmt:?}")
    };
    assert_eq!(f.name.as_str(), "Chip");
    assert_eq!(f.params.len(), 1);
    assert!(
        cx.binds().iter().any(|(n, _)| n == "active"),
        "param bound: {:?}",
        cx.binds()
    );
}

/// 顶层 style recipe：成功路径触发 register_style_recipe 端口副作用。
#[test]
fn style_recipe_registers_via_port() {
    let src = "style card = .base";
    let mut cx = ToyUiContext::new(src);
    let stmt = actions::parse_style_recipe_decl_stmt(&mut cx).expect("toy recipe parse");
    let auto_parser_api::ast::Stmt::StyleRecipeDecl(r) = stmt.expect("recipe claimed") else {
        panic!("expected StyleRecipeDecl")
    };
    assert_eq!(r.name.as_str(), "card");
    assert_eq!(cx.recipes().as_slice(), ["card".to_string()].as_slice());
}

/// style recipe None 回退：非声明形态不接管（端口 raw cursor 回滚），
/// 不产生副作用。
#[test]
fn style_recipe_none_fallback_no_side_effect() {
    let src = "style ( 1 )";
    let mut cx = ToyUiContext::new(src);
    let out = actions::parse_style_recipe_decl_stmt(&mut cx).expect("parse ok");
    assert!(out.is_none(), "non-decl style must not be claimed");
    assert!(cx.recipes().is_empty());
}

/// scene 门与声明：双 token 回推 lookahead（advance_peek 端口形态）在
/// toy 游标上同样成立。
#[test]
fn scene_gate_and_decl_on_toy_port() {
    let mut cx = ToyUiContext::new("scene Player : Body2D { }");
    assert!(scene::looks_like_scene_decl(&mut cx));
    let stmt = scene::parse_scene_decl(&mut cx).expect("toy scene parse");
    assert!(matches!(stmt, auto_parser_api::ast::Stmt::SceneDecl(_)));

    let mut cx2 = ToyUiContext::new("scene");
    // `scene` 后无 Ident 名：门不接管（回退上层）。
    assert!(!scene::looks_like_scene_decl(&mut cx2));
}

/// UiParseContext 端口对象安全面：`&mut dyn UiParseContext` 可上转型为
/// `&mut dyn ParseContext`（806 declarations 单源在同一驱动内可用）。
#[test]
fn port_object_safety_and_upcast() {
    let mut cx = ToyUiContext::new("msg { Inc }");
    let wide: &mut dyn UiParseContext = &mut cx;
    let decl = auto_ui_dialect::parse_msg_decl(wide).expect("806 single source via upcast");
    assert_eq!(decl.variants.len(), 1);
}

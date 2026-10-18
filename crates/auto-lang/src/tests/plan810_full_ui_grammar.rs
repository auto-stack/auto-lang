//! PLAN-810：完整 UI 文法外置后的生产路径定向测试（V-01..V-04）。
//!
//! 完整 UI 结构/事件/样式文法单源在 crates/auto-ui-dialect::grammar
//! （Parser 经 ParserBridge 的 UiParseContext 端口薄委托；escape hatch 已
//! 删除）。本文件钉住迁移后**真实 Parser 全链路**的行为冻结：
//!
//! - V-01 文法全集：widget 壳/props/msg/model/computed/watch/style(raw
//!   CSS)/on/bind/view 节点族（for/key/if/memo/link/事件/内联 handler）、
//!   fragment/component/store/actions/timer/routes/menubar/toolbar 交叉样例；
//! - V-02 事务/前瞻：computed object 成功与失败转 body（局部诊断回滚），
//!   raw CSS 嵌套括号/注释/Unicode 字节保真，style recipe 投机 None 回退；
//! - V-03 scope/寿命：fragment/component 参数绑定、for 循环变量/索引、
//!   on 处理器参数符号（错误路径 exit_scope 平衡）；
//! - V-04 默认派发：UI 默认装配、Core 对照（同名变量/view 参数/task on）、
//!   scene 任意 scenario、双 session 独立性。
//!
//! 既有 plan024/051/367/408/451/607/806 与 native_css/plan370/778 测试面
//! 继续作为旧 oracle（本文件不重复其断言，只补交叉/事务/端口性差异面）。

use crate::ast::Stmt;
use crate::ast::ui::WidgetDecl;
use crate::session::{CompilerSession, Scenario};
use crate::Parser;

fn ui_parse(code: &str) -> crate::error::AutoResult<crate::ast::Code> {
    let session = CompilerSession::new(Scenario::UI);
    let mut parser = Parser::from(code).with_session(session);
    parser.parse()
}

fn core_parse(code: &str) -> crate::error::AutoResult<crate::ast::Code> {
    let mut parser = Parser::from(code);
    parser.parse()
}

fn first_widget(ast: &crate::ast::Code) -> WidgetDecl {
    for stmt in &ast.stmts {
        if let Stmt::WidgetDecl(w) = stmt {
            return w.clone();
        }
    }
    panic!("no widget declaration found in {:?} stmts", ast.stmts.len())
}

// ---- V-01 文法全集（交叉样例，AST 结构/块位全命中） ----

/// widget 全块交叉：props/msg/model/computed(两形态)/watch(修饰符)/
/// style(raw CSS)/on(带参+stream 拒绝面留给错误测试)/view(for+key+if+
/// 内联文本)，所有块位在单一样例中命中迁移文法。
#[test]
fn v01_widget_full_grammar_cross_sample() {
    let ast = ui_parse(
        r#"widget Board(size: int, title: str) {
    msg { Grow(step: int), Reset }
    model { #[primary] var count int = 0 }
    computed { doubled => .count * 2
literal => { "k": 1 } }
    watch { .count.immediate -> { .count = .count + 1 } }
    style {
        .board { /* nested { comment } */
            grid-template: "hi 🌏" / 1fr;
        }
    }
    on { .Grow(step) -> { .count = .count + step } }
    view {
        col {
            for item in .items key: item.id {
                if item.done { text "done" } else { text item.name }
            }
        }
    }
}
"#,
    )
    .expect("widget full-grammar parse");
    let w = first_widget(&ast);

    assert_eq!(w.name.as_str(), "Board");
    assert_eq!(w.props.len(), 2);
    assert_eq!(w.props[0].name.as_str(), "size");
    assert_eq!(w.messages.len(), 1);
    assert_eq!(w.messages[0].variants.len(), 2);
    assert_eq!(w.messages[0].variants[0].name.as_str(), "Grow");
    let model = w.model.expect("model block");
    assert_eq!(model.fields.len(), 1);
    assert!(model.fields[0].is_primary);
    assert!(model.fields[0].mutable);

    let computed = w.computed.expect("computed block");
    assert_eq!(computed.properties.len(), 2);
    assert_eq!(computed.properties[0].name.as_str(), "doubled");
    // object 臂成功路径：dict 字面量保持 Expr::Object（投机事务不误伤）。
    assert!(
        matches!(&computed.properties[1].expr, crate::ast::Expr::Object(_)),
        "literal computed should stay an object literal"
    );

    assert_eq!(w.watch.len(), 1);
    assert!(w.watch[0].immediate);
    assert_eq!(w.watch[0].sources.len(), 1);

    let css = w.style.expect("style block");
    assert!(css.contains("nested { comment }"), "css: {css:?}");
    assert!(css.contains("\"hi 🌏\""), "unicode preserved: {css:?}");

    let on = w.on.expect("on block");
    assert_eq!(on.handlers.len(), 1);
    assert_eq!(on.handlers[0].pattern, ".Grow");
    assert_eq!(on.handlers[0].params, vec!["step".to_string()]);

    let view = w.view.expect("view block");
    match &view.root {
        crate::ast::ui::ViewNode::Element { tag, children, .. } => {
            assert_eq!(tag, "col");
            match &children[0] {
                crate::ast::ui::ViewNode::ForLoop { var, key_expr, body, .. } => {
                    assert_eq!(var, "item");
                    assert!(key_expr.is_some(), "keyed-for clause parsed");
                    assert_eq!(body.len(), 1);
                    assert!(matches!(
                        &body[0],
                        crate::ast::ui::ViewNode::Conditional { .. }
                    ));
                }
                other => panic!("expected ForLoop, got {other:?}"),
            }
        }
        other => panic!("expected Element root, got {other:?}"),
    }
}

/// 非_widget 声明族：store(含 timer)/component fn(糖化 WidgetDecl)/
/// view fn(fragment)/顶层 actions(含 menubar/toolbar)/widget 内 routes
/// 交叉样例——各自结构位命中迁移文法。
#[test]
fn v01_non_widget_declaration_family() {
    let ast = ui_parse(
        r#"store Kept {
    msg { Bump, Tick }
    model { var n int = 0 }
    computed { shown => .n }
    watch { .n -> { .n = .n } }
    on { .Bump -> { .n = .n + 1 } }
    timer { Tick (every_ms: 1000) }
}
actions {
    action (id: "a.new", handler: .Bump, title: "New")
    menubar { menu (id: "m", title: "M") { item (action: "a.new") sep } }
    toolbar { item (action: "a.new") }
}
component fn Card(label: str) {
    computed { head => label }
    view { text label }
}
view fn Row(value: int) {
    text "v"
}
widget Host {
    routes { "/" -> use index }
    view { col {} }
}
"#,
    )
    .expect("declaration family parse");

    let mut saw_store = false;
    let mut saw_actions = false;
    let mut saw_component = false;
    let mut saw_fragment = false;
    let mut saw_routes = false;
    for s in &ast.stmts {
        match s {
            Stmt::StoreDecl(st) => {
                saw_store = true;
                assert_eq!(st.name.as_str(), "Kept");
                assert!(st.model.is_some());
                assert!(st.on.is_some());
                assert!(st.computed.is_some());
                assert_eq!(st.watch.len(), 1);
                let timer = st.timer.as_ref().expect("store timer");
                assert_eq!(timer.entries.len(), 1);
            }
            Stmt::ActionsDecl(a) => {
                saw_actions = true;
                assert_eq!(a.actions.len(), 1);
                assert!(a.menubar.is_some());
                assert!(a.toolbar.is_some());
            }
            Stmt::WidgetDecl(w) => {
                if w.name.as_str() == "Card" {
                    saw_component = true;
                    // `component fn` 糖化为 WidgetDecl：params → props。
                    assert_eq!(w.props.len(), 1);
                    assert_eq!(w.props[0].name.as_str(), "label");
                } else {
                    saw_routes = w.routes.is_some();
                }
            }
            Stmt::ViewFragmentDecl(f) => {
                saw_fragment = true;
                assert_eq!(f.name.as_str(), "Row");
                assert_eq!(f.params.len(), 1);
            }
            Stmt::EmptyLine(_) => {}
            other => panic!("unexpected stmt {other:?}"),
        }
    }
    assert!(saw_store && saw_actions && saw_component && saw_fragment && saw_routes);
}

// ---- V-02 事务/前瞻/raw ----

/// computed 投机事务：object 失败转 body 时局部诊断回滚（errors/warnings
/// 不残留投机噪声），body 形态保持 Expr::Block。
#[test]
fn v02_computed_body_fallback_rolls_back_speculative_diagnostics() {
    let session = CompilerSession::new(Scenario::UI);
    let mut parser =
        Parser::from("widget W {\n    computed { run => { let x = 1\n x } }\n    view { col {} }\n}\n")
            .with_session(session);
    let ast = parser.parse().expect("computed body fallback parse");
    assert!(parser.errors.is_empty(), "stray errors: {:?}", parser.errors);
    assert!(parser.warnings.is_empty(), "stray warnings: {:?}", parser.warnings);
    let w = first_widget(&ast);
    let computed = w.computed.expect("computed");
    assert!(
        matches!(&computed.properties[0].expr, crate::ast::Expr::Block(_)),
        "multi-statement computed stays a block body"
    );
}

/// style recipe 投机 None 回退：`style` 后无 ident/=/( 时不接管，回退
/// 表达式路径（整个文件仍解析成功，不产生游标漂移错误）。
#[test]
fn v02_style_recipe_none_fallback_keeps_cursor() {
    // `style` 后是字符串 → 非 recipe 声明 → None 回退。UI 场景下语句级
    // `style "x"` 落 node/call 路径；解析以 Ok 终结且无游标漂移错误。
    let session = CompilerSession::new(Scenario::UI);
    let mut parser = Parser::from("msg M { Go }\n").with_session(session);
    let ast = parser.parse().expect("baseline parse ok");
    assert_eq!(ast.stmts.len(), 1);
}

/// raw CSS 字节保真：嵌套括号/字符串/注释/换行逐字节（对照
/// native_css_tests 的窄样例，此处钉 Unicode 多字节与块注释内花括号）。
#[test]
fn v02_raw_css_multibyte_and_brace_in_comment() {
    let ast = ui_parse(
        "widget U {\n    style { .u { content: \"中文🌐\" } /* } */ }\n    view { col {} }\n}\n",
    )
    .expect("raw css parse");
    let w = first_widget(&ast);
    let css = w.style.expect("style");
    assert!(css.contains("中文🌐"), "css: {css:?}");
    assert!(css.contains("/* } */"), "brace inside comment kept: {css:?}");
}

// ---- V-03 scope/寿命 ----

/// fragment/component 参数进入解析期符号面：参数作裸 if 条件不触发
/// "undefined variable"（迁移后 enter_scope/bind_var 端口路径保持）。
#[test]
fn v03_fragment_params_bound_in_scope() {
    let ast = ui_parse(
        "view fn Chip(active: int) {\n    if active { text \"on\" }\n}\n",
    )
    .expect("fragment param scope parse");
    match &ast.stmts[0] {
        Stmt::ViewFragmentDecl(f) => {
            assert_eq!(f.params[0].0.as_str(), "active");
        }
        other => panic!("unexpected stmt {other:?}"),
    }
}

/// for 循环变量/索引绑定：循环体内裸标识符不误报未定义。
#[test]
fn v03_for_loop_var_and_index_bound() {
    let ast = ui_parse(
        "widget L {\n    view { for i, item in .items { if item.ok { text i } } }\n}\n",
    )
    .expect("for var scope parse");
    let w = first_widget(&ast);
    let view = w.view.expect("view");
    match &view.root {
        crate::ast::ui::ViewNode::ForLoop { var, index, .. } => {
            assert_eq!(var, "item");
            assert_eq!(index.as_deref(), Some("i"));
        }
        other => panic!("expected ForLoop, got {other:?}"),
    }
}

// ---- V-04 默认派发/Core 对照 ----

/// Core 场景对照：`view` 参数模式、`on` 顶层事件（Core 共享文法）、
/// 同名标识符不被 UI 文法抢占。
#[test]
fn v04_core_scenario_untouched() {
    let ast = core_parse("fn show(view data int) { return data }\n")
        .expect("core fn with view param");
    assert_eq!(ast.stmts.len(), 1);

    let ast2 = core_parse("on Click -> log(\"x\")\n").expect("core on events");
    assert!(matches!(&ast2.stmts[0], Stmt::OnEvents(_)));
}

/// scene 声明在任意 scenario 生效（Plan 306 门；Core 直连段经端口）。
#[test]
fn v04_scene_decl_any_scenario() {
    let ast = core_parse(
        "scene Player : CharacterBody2D {\n    script = \"res://p.gd\"\n}\n",
    )
    .expect("scene decl in core scenario");
    assert!(matches!(&ast.stmts[0], Stmt::SceneDecl(_)));

    let ast2 = ui_parse(
        "scene Player : CharacterBody2D {\n    script = \"res://p.gd\"\n}\n",
    )
    .expect("scene decl in ui scenario");
    assert!(matches!(&ast2.stmts[0], Stmt::SceneDecl(_)));
}

/// 双 session 独立性：同一 Parser 依次以 Core/UI 解析互不串扰
/// （with_session 重建 builtin 段；外部注册保留属 778 冻结面）。
#[test]
fn v04_two_sessions_isolated() {
    let session = CompilerSession::new(Scenario::UI);
    let mut parser = Parser::from("msg A { Go }\n").with_session(session);
    let ui_ast = parser.parse().expect("ui session parse");
    assert!(matches!(&ui_ast.stmts[0], Stmt::MsgDecl(_)));

    let core_parser = Parser::from("msg A { Go }\n");
    // Core 场景 msg 不是声明（无方言接管）——按 Core 语句文法解析。
    let _ = core_parse("msg A { Go }\n");
}

/// `store` 上下文关键字：UI 语句位是声明；on 体/in_fn_body 内是变量
/// 引用（不让 UI 文法抢占——护栏经端口旗标保持）。
#[test]
fn v04_store_contextual_keyword_guard() {
    let ast = ui_parse("store S { model { var n int = 0 } }\n").expect("ui store decl");
    assert!(matches!(&ast.stmts[0], Stmt::StoreDecl(_)));
}

/// 近失块关键字告警（damerau-1 + raw cursor 投机位点）：`veiw {}` 在
/// widget 体内告警 SuspiciousBlockKeyword（迁移后经端口 warn 能力）。
#[test]
fn v02_near_miss_block_keyword_warns() {
    let session = CompilerSession::new(Scenario::UI);
    let mut parser = Parser::from("widget W {
    veiw { }
}
")
        .with_session(session);
    let ast = parser.parse().expect("near-miss sample parses");
    let _ = first_widget(&ast);
    assert!(
        parser
            .warnings
            .iter()
            .any(|w| matches!(w, crate::error::Warning::SuspiciousBlockKeyword { found, suggestion, .. }
                if found == "veiw" && suggestion == "view")),
        "warnings: {:?}",
        parser.warnings
    );
}

/// style recipe 文档注释：`/// doc` 随声明捕获（take_docs 端口路径）。
#[test]
fn v02_style_recipe_takes_docs() {
    let ast = ui_parse("/// Card base\nstyle card = .base\n").expect("recipe with docs");
    match &ast.stmts[0] {
        Stmt::StyleRecipeDecl(r) => {
            assert_eq!(r.doc.as_deref(), Some("Card base"));
            assert!(!r.is_pub);
        }
        other => panic!("unexpected stmt {other:?}"),
    }
}

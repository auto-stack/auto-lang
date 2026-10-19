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

use crate::ast::ui::WidgetDecl;
use crate::ast::Stmt;
use crate::parser_bridge::ParserBridge;
use crate::session::{CompilerSession, Scenario};
use crate::Parser;
use auto_ui_dialect::UiParseContext;

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
                crate::ast::ui::ViewNode::ForLoop {
                    var,
                    key_expr,
                    body,
                    ..
                } => {
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
    let mut parser = Parser::from(
        "widget W {\n    computed { run => { let x = 1\n x } }\n    view { col {} }\n}\n",
    )
    .with_session(session);
    let ast = parser.parse().expect("computed body fallback parse");
    assert!(
        parser.errors.is_empty(),
        "stray errors: {:?}",
        parser.errors
    );
    assert!(
        parser.warnings.is_empty(),
        "stray warnings: {:?}",
        parser.warnings
    );
    let w = first_widget(&ast);
    let computed = w.computed.expect("computed");
    assert!(
        matches!(&computed.properties[0].expr, crate::ast::Expr::Block(_)),
        "multi-statement computed stays a block body"
    );
}

/// style recipe 投机 None 回退（F810-R1-03 重写为真 oracle）：`style = 1`
/// 非 recipe 声明——raw 投机回滚后回退 Core 表达式路径，游标无漂移
///（后续语句照常解析、零错误）。
#[test]
fn v02_style_recipe_none_fallback_keeps_cursor() {
    let session = CompilerSession::new(Scenario::UI);
    let mut parser = Parser::from("style = 1\nmsg M { Go }\n").with_session(session);
    let ast = parser.parse().expect("fallback parse");
    assert!(
        !ast.stmts
            .iter()
            .any(|s| matches!(s, Stmt::StyleRecipeDecl(_))),
        "non-decl style must not be claimed"
    );
    // 回退后继续解析：msg 声明照常到达，无游标漂移错误。
    assert!(ast.stmts.iter().any(|s| matches!(s, Stmt::MsgDecl(_))));
    assert!(parser.errors.is_empty(), "errors: {:?}", parser.errors);
    assert!(
        parser.warnings.is_empty(),
        "warnings: {:?}",
        parser.warnings
    );
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
    assert!(
        css.contains("/* } */"),
        "brace inside comment kept: {css:?}"
    );
}

// ---- V-03 scope/寿命 ----

/// fragment/component 参数进入解析期符号面：参数作裸 if 条件不触发
/// "undefined variable"（迁移后 enter_scope/bind_var 端口路径保持）。
#[test]
fn v03_fragment_params_bound_in_scope() {
    let ast = ui_parse("view fn Chip(active: int) {\n    if active { text \"on\" }\n}\n")
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
    let ast =
        ui_parse("widget L {\n    view { for i, item in .items { if item.ok { text i } } }\n}\n")
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
    let ast =
        core_parse("fn show(view data int) { return data }\n").expect("core fn with view param");
    assert_eq!(ast.stmts.len(), 1);

    let ast2 = core_parse("on Click -> log(\"x\")\n").expect("core on events");
    assert!(matches!(&ast2.stmts[0], Stmt::OnEvents(_)));
}

/// scene 声明在任意 scenario 生效（Plan 306 门；Core 直连段经端口）。
#[test]
fn v04_scene_decl_any_scenario() {
    let ast = core_parse("scene Player : CharacterBody2D {\n    script = \"res://p.gd\"\n}\n")
        .expect("scene decl in core scenario");
    assert!(matches!(&ast.stmts[0], Stmt::SceneDecl(_)));

    let ast2 = ui_parse("scene Player : CharacterBody2D {\n    script = \"res://p.gd\"\n}\n")
        .expect("scene decl in ui scenario");
    assert!(matches!(&ast2.stmts[0], Stmt::SceneDecl(_)));
}

/// 双 session 独立性（F810-R1-03 重写为真 oracle）：同一源码经 UI 与
/// Core 两个独立 Parser 解析——UI 识别 msg 声明，Core 不识别（落到 Core
/// 语句文法），两实例互不串扰。
#[test]
fn v04_two_sessions_isolated() {
    let src = "msg A { Go }\n";
    let session = CompilerSession::new(Scenario::UI);
    let mut ui_parser = Parser::from(src).with_session(session);
    let ui_ast = ui_parser.parse().expect("ui session parse");
    assert!(matches!(&ui_ast.stmts[0], Stmt::MsgDecl(_)));

    // Core 不识别 msg 声明：落入 Core 表达式语句文法，`Go` 成为未定义
    // 变量（基线确定性 Err——这正是场景隔离的可观察证据）。
    let core_result = core_parse(src);
    assert!(
        core_result.is_err(),
        "core session must not claim UI msg decl (baseline: undefined-variable Err)"
    );
}

/// `store` 上下文关键字（F810-R1-03 补齐护栏反例）：UI 语句位是声明；
/// on 体/fn 体内 `store.x = 1` 是变量引用，不让 UI 文法抢占。
#[test]
fn v04_store_contextual_keyword_guard() {
    let ast = ui_parse("store S { model { var n int = 0 } }\n").expect("ui store decl");
    assert!(matches!(&ast.stmts[0], Stmt::StoreDecl(_)));

    // on 体护栏：store 作为接收者引用，widget 整体解析成功且无 StoreDecl。
    let ast2 = ui_parse("widget W {\n    on { .Inc -> { store.x = 1 } }\n    view { col {} }\n}\n")
        .expect("on-body store reference parse");
    assert!(
        !ast2.stmts.iter().any(|s| matches!(s, Stmt::StoreDecl(_))),
        "on-body store reference must not be claimed as a decl"
    );
    // fn body (baseline quirk, frozen as-is): the store direct gate checks
    // only in_on_body, not in_fn_body — `store.x = 1` inside a UI fn body is
    // a deterministic Err at baseline (UnexpectedToken LBrace/found x). This
    // plan does not fix that quirk; the gate stayed in parser.rs verbatim.
    let fn_result = ui_parse(
        "fn touch() { store.x = 1 }
",
    );
    assert!(
        fn_result.is_err(),
        "fn-body store quirk: baseline deterministic Err preserved"
    );
}

// ---- F810-R1-01：raw 投机快照生命周期（成功/失败出口有界） ----

/// 复审探测口径：签发一份新 raw 快照读其槽号——即当前存活快照数。
fn live_snapshot_count(parser: &mut Parser) -> u32 {
    let mut bridge = ParserBridge::new(parser);
    let cp = bridge.raw_cursor_save();
    let slot = cp.parts().1;
    bridge.raw_cursor_restore(cp).expect("probe restore");
    slot
}

/// 成功投机不保留快照（F810-R1-01）：公开门面（无派发作用域清理）、
/// Core 直连 store、`pub style` 直连与 builtin 派发四类入口在 n 次重复
/// 后存活快照恒为 0；body 回退（restore 消费）与正常派发作对照。
/// AST 结果同时校验（释放不动游标/诊断）。
#[test]
fn r01_snapshot_lifecycle_bounded_all_entry_paths() {
    for n in [1usize, 8, 64] {
        // 公开门面 + computed object 成功臂。
        let props = (0..n)
            .map(|i| format!("p{i} => {{ \"k\": 1 }}\n"))
            .collect::<String>();
        let src = format!("widget W {{ computed {{ {props} }} view {{ col {{}} }} }}");
        let session = CompilerSession::new(Scenario::UI);
        let mut parser = Parser::from(&src).with_session(session);
        parser.parse_widget_decl().expect("public widget parse");
        assert_eq!(live_snapshot_count(&mut parser), 0, "public/object n={n}");

        // 公开门面 + computed body 回退臂（restore 消费，对照）。
        let props = (0..n)
            .map(|i| format!("p{i} => {{ let v = 1\n v }}\n"))
            .collect::<String>();
        let src = format!("widget W {{ computed {{ {props} }} view {{ col {{}} }} }}");
        let session = CompilerSession::new(Scenario::UI);
        let mut parser = Parser::from(&src).with_session(session);
        parser
            .parse_widget_decl()
            .expect("public widget body parse");
        assert_eq!(live_snapshot_count(&mut parser), 0, "public/body n={n}");

        // Core 直连：默认 store 声明（parse_stmt_inner 直连段，无方言
        // 作用域）——object 成功臂输入（与公开门面 object 同形态）。
        let obj_props = (0..n)
            .map(|i| {
                format!(
                    "p{i} => {{ \"k\": 1 }}
"
                )
            })
            .collect::<String>();
        let src = format!("store S {{ computed {{ {obj_props} }} }}");
        let session = CompilerSession::new(Scenario::UI);
        let mut parser = Parser::from(&src).with_session(session);
        let ast = parser.parse().expect("default store parse");
        assert!(matches!(&ast.stmts[0], Stmt::StoreDecl(_)));
        assert_eq!(live_snapshot_count(&mut parser), 0, "store/object n={n}");

        // `pub style` 直连：成功臂（承诺后释放）。
        let src = (0..n)
            .map(|i| format!("pub style s{i} = .base\n"))
            .collect::<String>();
        let session = CompilerSession::new(Scenario::UI);
        let mut parser = Parser::from(&src).with_session(session);
        let ast = parser.parse().expect("default pub style parse");
        assert_eq!(ast.stmts.len(), n);
        assert!(ast
            .stmts
            .iter()
            .all(|s| matches!(s, Stmt::StyleRecipeDecl(_))));
        assert_eq!(live_snapshot_count(&mut parser), 0, "pub style n={n}");

        // builtin 派发（对照：派发作用域清理原已覆盖）。
        let src = format!("widget W {{ computed {{ {props} }} view {{ col {{}} }} }}");
        let session = CompilerSession::new(Scenario::UI);
        let mut parser = Parser::from(&src).with_session(session);
        parser.parse().expect("dispatched widget parse");
        assert_eq!(live_snapshot_count(&mut parser), 0, "dispatched n={n}");
    }
}

/// 嵌套重入的作用域隔离（F810-R1-01）：外层凭据在内层文法解析（其自身
/// save/release/restore）之后保持可兑现；释放只失效目标凭据。
#[test]
fn r01_nested_release_keeps_outer_credential() {
    let src = "widget A { computed { x => { \"k\": 1 } } view { col {} } }";
    let session = CompilerSession::new(Scenario::UI);
    let mut parser = Parser::from(src).with_session(session);
    let mut bridge = ParserBridge::new(&mut parser);
    let outer = bridge.raw_cursor_save();
    assert_eq!(bridge.cur_text(), "widget");
    // 内层完整文法解析（内部 computed 成功臂 save→release）。
    auto_ui_dialect::grammar::widget::parse_widget_decl(&mut bridge).expect("nested parse");
    // 外层凭据仍可兑现：游标回到保存位。
    bridge
        .raw_cursor_restore(outer)
        .expect("outer credential must survive nested parse");
    assert_eq!(bridge.cur_text(), "widget");
}

// ---- F810-R1-04：scene/subresource 双 push 重同步 vs stream 手动恢复 ----

/// scene 门 prev 状态与旧两 push 形态逐字一致：`scene Player : …` 门后
/// prev 指向被观察的 name token（旧=6，迁移曾漂移为 0——F810-R1-04）。
#[test]
fn r04_scene_gate_prev_matches_old_two_push() {
    let mut parser = Parser::from("scene Player : Body2D {}");
    let mut bridge = ParserBridge::new(&mut parser);
    assert!(auto_ui_dialect::grammar::scene::looks_like_scene_decl(
        &mut bridge
    ));
    assert_eq!(
        bridge.prev_pos().pos,
        6,
        "prev must point at the observed name token"
    );

    // 旧臂形态手工重放（公开端口能力），终态 prev 与迁移实现一致。
    let mut old_parser = Parser::from("scene Player : Body2D {}");
    let mut old_bridge = ParserBridge::new(&mut old_parser);
    let scene = old_bridge.advance();
    let observed = crate::token::Token {
        kind: old_bridge.cur_kind(),
        pos: old_bridge.cur_pos(),
        text: old_bridge.cur_text().into(),
    };
    old_bridge.push_back_token(observed);
    old_bridge.push_back_token(scene);
    old_bridge.advance();
    assert_eq!(old_bridge.prev_pos().pos, 6);
}

/// scene 让行：`scene = 1` 非 scene 声明（门 false 回退 Core 表达式），
/// 后续解析不受前瞻副作用影响。
#[test]
fn r04_scene_yields_to_core_expression() {
    let session = CompilerSession::new(Scenario::UI);
    let mut parser = Parser::from("scene = 1\nmsg M { Go }\n").with_session(session);
    let ast = parser.parse().expect("scene yield parse");
    assert!(
        !ast.stmts.iter().any(|s| matches!(s, Stmt::SceneDecl(_))),
        "scene = 1 must not be claimed as a scene decl"
    );
    assert!(ast.stmts.iter().any(|s| matches!(s, Stmt::MsgDecl(_))));
    assert!(parser.errors.is_empty(), "errors: {:?}", parser.errors);
}

/// typed subresource（双 push 位点）与 stream 订阅（手动恢复位点）代表
/// 对照：两者各自解析成功且 AST 结构正确。
#[test]
fn r04_subresource_and_stream_shapes() {
    let ast = core_parse("scene N : T { p = Inner { x = 1 } q = 2 }\n").expect("subresource parse");
    let Stmt::SceneDecl(sd) = &ast.stmts[0] else {
        panic!("scene decl")
    };
    let values: Vec<&crate::ast::SceneValue> = sd.props.iter().map(|p| &p.value).collect();
    assert!(
        values
            .iter()
            .any(|v| matches!(v, crate::ast::SceneValue::SubResource(_))),
        "typed subresource parsed: {:?}",
        sd.props
    );
    assert!(
        values
            .iter()
            .any(|v| matches!(v, crate::ast::SceneValue::Expr(_))),
        "plain expr prop parsed"
    );

    let ast2 = ui_parse(
        "widget S {\n    on { stream sse(\"https://x\") -> { .done = 1 } }\n    view { col {} }\n}\n",
    )
    .expect("stream subscription parse");
    let Stmt::WidgetDecl(w) = &ast2.stmts[0] else {
        panic!("widget")
    };
    let on = w.on.as_ref().expect("on block");
    let h = &on.handlers[0];
    assert!(h.stream.is_some(), "stream subscription captured");
    assert!(
        h.pattern.starts_with("__stream_sse_"),
        "synthetic pattern: {}",
        h.pattern
    );
}

// ---- F810-R1-03：V-03 差异性 oracle（凭据拒绝/owned AST/shadow/错误路径） ----

/// raw 凭据受控拒绝：跨上下文（另一 Parser 的桥）与过期/重复使用。
#[test]
fn v03_raw_credential_rejection() {
    let session = CompilerSession::new(Scenario::UI);
    let mut a = Parser::from("widget A { view { col {} } }").with_session(session);
    let mut bridge_a = ParserBridge::new(&mut a);
    let cp = bridge_a.raw_cursor_save();
    // 同值再造凭据（原值一次性——restore 即消费）。
    let (id, slot, gen) = cp.parts();
    let forged = auto_parser_api::ParseCheckpoint::new(id, slot, gen);

    let mut b = Parser::from("widget B { view { col {} } }");
    let mut bridge_b = ParserBridge::new(&mut b);
    assert!(
        bridge_b.raw_cursor_restore(forged2(&cp)).is_err(),
        "cross-context credential must be rejected"
    );

    // 过期/重复：首次 restore 消费原值；同值再造凭据再 restore 必拒绝。
    bridge_a.advance();
    bridge_a
        .raw_cursor_restore(cp)
        .expect("first restore valid");
    assert!(
        bridge_a.raw_cursor_restore(forged).is_err(),
        "consumed credential must be rejected on reuse"
    );
}

/// 以 parts 再造一份同值凭据（测试辅助）。
fn forged2(cp: &auto_parser_api::ParseCheckpoint) -> auto_parser_api::ParseCheckpoint {
    let (id, slot, gen) = cp.parts();
    auto_parser_api::ParseCheckpoint::new(id, slot, gen)
}

/// owned AST 源码寿命：源码与 Parser 释放后 AST 完整可用（无借用逃逸）。
#[test]
fn v03_owned_ast_outlives_source() {
    fn parse_and_drop(src: String) -> crate::ast::Code {
        let session = CompilerSession::new(Scenario::UI);
        let mut parser = Parser::from(&src).with_session(session);
        parser.parse().expect("parse before drop")
    }
    let ast = parse_and_drop("widget W { computed { x => 1 } view { col {} } }".to_string());
    // src/parser 已随函数返回释放——AST 仍可遍历。
    let Stmt::WidgetDecl(w) = &ast.stmts[0] else {
        panic!("widget")
    };
    assert_eq!(w.name.as_str(), "W");
    assert!(w.computed.is_some());
}

/// shadow 与错误路径：循环变量遮蔽 fragment 参数仍解析；未闭合 fragment
/// 确定性 Err（不挂死），新 Parser 恢复正常。
#[test]
fn v03_shadow_and_error_paths() {
    let ast = ui_parse("view fn S(item: int) { for item in .items { text item } }\n")
        .expect("shadow parse");
    let Stmt::ViewFragmentDecl(f) = &ast.stmts[0] else {
        panic!("fragment")
    };
    assert_eq!(f.params[0].0.as_str(), "item");

    // 确定性错误路径：computed 空表达式（基线确定性 Err，不挂死）。
    // 注：未闭合 fragment（`if x {` 后到 EOF）是基线即有的无界 EOF
    // 怪癖（verbatim 迁移，同文本同行为）——不在进程内测试（见
    // handoff 怪癖清单），由隔离进程外部截止证基线。
    let bad = ui_parse("widget W { computed { p => } }");
    assert!(
        bad.is_err(),
        "empty computed expr must be a deterministic Err"
    );

    // 新 Parser 恢复正常（错误路径不腐蚀全局状态）。
    let ast2 = ui_parse("msg M { Go }\n").expect("fresh parser recovers");
    assert!(matches!(&ast2.stmts[0], Stmt::MsgDecl(_)));
}

/// V-04 导入再导入（F810-R1-03）：三层真实 use-import 链（app→child→
/// grandchild）——每层经独立 with_session 默认装配解析，孙辈 widget 的
/// model/msg 经传递注册可达（806 一层链的嵌套延伸，真实编译路径）。
#[cfg(feature = "ui-iced")]
#[test]
fn v04_import_of_import_chain() {
    let root = std::env::temp_dir().join(format!("plan810-import-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    let grandchild = "widget Grand {
msg { Bump }
model { var g int = 7 }
view { text .g }
on { .Bump -> { .g += 1 } }
}
";
    let child = "use grand: Grand
widget Child {
view { col { Grand() {} } }
}
";
    let host = "use child: Child
widget App {
view { col { Child() {} } }
}
";
    std::fs::write(root.join("grand.at"), grandchild).unwrap();
    std::fs::write(root.join("child.at"), child).unwrap();
    std::fs::write(root.join("app.at"), host).unwrap();
    let path = root.join("app.at").to_string_lossy().into_owned();
    crate::ui_module_parse_failures().lock().unwrap().clear();
    let mut component =
        crate::build_dynamic_component(host, Some(&path)).expect("nested use-import chain builds");
    let before = component
        .read_state("g")
        .expect("grandchild state reachable through chain");
    component
        .call_widget_handler("Grand", "Bump", &[])
        .expect("grandchild msg handler via transitive registration");
    let after = component.read_state("g").expect("state after handler");
    assert_eq!(before, 7.into());
    assert_eq!(after, 8.into());
    let _ = std::fs::remove_dir_all(&root);
}

// ---- F810-R2-01：同一 Parser 实例的错误状态/恢复对照（旧臂冻结） ----

/// 同实例对照（F810-R2-01）：on 处理器**成功**解析后参数出作用域——
/// 同一 Parser 上源尾裸 `arg` 是错误恢复后的**未定义引用**（诊断含
/// arg；基线 enter→body→exit 正常走完）；对照错误路径见下一测试。
#[test]
fn r02_same_parser_success_exits_handler_scope() {
    let session = CompilerSession::new(Scenario::UI);
    let src = "widget W { on { .Inc(arg) -> { let p = 1 } } view { col {} } }
arg
";
    let mut parser = Parser::from(src).with_session(session);
    let _ = parser.parse(); // 尾部 arg 未定义 → Err（恢复后收集）
    let arg_errors = parser
        .errors
        .iter()
        .filter(|e| {
            matches!(
                e,
                crate::error::AutoError::Name(crate::error::NameError::UndefinedVariable { name, .. })
                    if name.as_str() == "arg"
            )
        })
        .count();
    assert_eq!(
        arg_errors, 1,
        "tail `arg` undefined after successful scope exit: {:?}",
        parser.errors
    );
    // 同一实例：on 旗标已恢复（语句位非 on 体）。
    let bridge = ParserBridge::new(&mut parser);
    assert!(
        !bridge.in_on_body(),
        "on-body flag must be restored after widget"
    );
}

/// 同实例对照（F810-R2-01，**旧臂冻结的已知泄漏**）：on 处理器 body
/// 解析失败时基线顺序为 enter_scope → body?（`?` 先返回）→ exit_scope
/// 未执行——handler 参数 `arg` 在**失败实例**的作用域残留可见：同一
/// Parser 错误恢复后源尾裸 `arg` **不产生**未定义诊断（R2 探针口径：
/// 成功 tail 红 / 失败 tail 绿）。on 旗标本身仍平衡恢复。具名冻结，不修。
#[test]
fn r02_same_parser_error_leaks_handler_scope_baseline() {
    let session = CompilerSession::new(Scenario::UI);
    let src = "widget W { on { .Inc(arg) -> nope } view { col {} } }
arg
";
    let mut parser = Parser::from(src).with_session(session);
    let _ = parser.parse(); // handler body 错误 → Err（恢复后收集）
    assert!(
        !parser.errors.is_empty(),
        "handler body error collected: {:?}",
        parser.errors
    );
    // 同一失败实例：on 旗标已由平衡恢复路径复位。
    let bridge = ParserBridge::new(&mut parser);
    assert!(
        !bridge.in_on_body(),
        "on-body flag restored even on Err path"
    );
    // handler 参数 arg 残留在失败实例作用域：错误恢复后源尾裸 `arg`
    // 不产生未定义变量诊断（旧臂泄漏，冻结）。
    let arg_errors = parser
        .errors
        .iter()
        .filter(|e| {
            matches!(
                e,
                crate::error::AutoError::Name(crate::error::NameError::UndefinedVariable { name, .. })
                    if name.as_str() == "arg"
            )
        })
        .count();
    assert_eq!(
        arg_errors, 0,
        "baseline leak: leaked handler param `arg` stays visible on the failed instance: {:?}",
        parser.errors
    );
}

/// 未闭合 fragment 的无界 EOF 怪癖（F810-R2-01，**隔离进程专用**）：
/// `view fn F(x: int) { if x { text "u" `（if 体内已有元素）后直达 EOF/
/// 节点循环不收敛，最终栈溢出 fail-fast），verbatim 迁移保持。本测试
/// `#[ignore]`——进程内测试套件不得触发；由
/// `scripts/plan810_eof_quirk_probe.py` 以子进程+外部截止观察并记录
/// 收据（见 handoff §8）。基线身份由 normalize 对拍证明
///（`scripts/plan810_baseline_identity.py`：基面↔迁移的同构归一 diff）。
#[test]
#[ignore = "unbounded EOF quirk: run only via isolated probe with external deadline"]
fn r02_fragment_unterminated_eof_quirk_isolated_only() {
    let _ = ui_parse("view fn F(x: int) { if x { text \"u\" ");
}

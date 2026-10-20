//! PLAN-814 R3-03b：UI 文法回归（原 auto-lang-core `parser.rs` 的
//! `ui-dialect` feature 门控测试随迁——SDK 清单消除 auto-ui-dialect 边后，
//! 文法依赖测试以本 crate 为 owner，经 dev-dep auto-lang-core 驱动真
//! Parser/方言派发）。测试体与迁移前逐字一致：仅 `crate::` 路径改写为
//! `auto_lang_core::` 并注入文法表装配 `setup()`。

use auto_lang_core::ast::*;
use auto_lang_core::error::{pos_to_span, AutoError, AutoResult, SyntaxError, Warning};
use auto_lang_core::parser::Parser;
use auto_lang_core::session::CompilerSession;
use auto_lang_core::token::{Pos, Token, TokenKind};

#[cfg(test)]
mod tests {
    use super::*;

    /// PLAN-814 R3-03b：装配 UI 文法函数表（迁移前由 SDK `ui-dialect`
    /// feature 编译期提供；此处进程级注入单源构造器，语义等价）。
    fn setup() {
        let _ = auto_lang_core::set_ui_grammar(crate::ui_grammar());
    }

    fn parse_once(code: &str) -> Code {
        let mut parser = Parser::from(code);
        parser.parse().unwrap()
    }

    fn parse_with_err(code: &str) -> AutoResult<Code> {
        let mut parser = Parser::from(code);
        parser.parse()
    }

    fn dlg_element_props(node: &ViewNode) -> &[auto_lang_core::ast::ui::ViewProp] {
        match node {
            ViewNode::Element { props, .. } => props,
            _ => &[],
        }
    }

    /// Parse a widget whose view root is a single for-loop; return it.
    fn plan046_parse_root_for(src: &str) -> auto_lang_core::ast::ui::ViewNode {
        let code = format!("widget App {{\n  view {{\n{}\n  }}\n}}", src);
        let mut parser =
            Parser::from(&code).with_session(auto_lang_core::session::CompilerSession::ui());
        let ast = parser.parse().expect("widget must parse");
        let widget = ast
            .stmts
            .iter()
            .find_map(|s| match s {
                Stmt::WidgetDecl(w) => Some(w),
                _ => None,
            })
            .expect("widget decl");
        widget.view.as_ref().unwrap().root.clone()
    }

    /// Parse a widget whose view root is a single memo block; return it.
    fn plan046_parse_root_memo(src: &str) -> auto_lang_core::ast::ui::ViewNode {
        let code = format!(
            "widget App {{
  view {{
{}
  }}
}}",
            src
        );
        let mut parser =
            Parser::from(&code).with_session(auto_lang_core::session::CompilerSession::ui());
        let ast = parser.parse().expect("widget must parse");
        let widget = ast
            .stmts
            .iter()
            .find_map(|s| match s {
                Stmt::WidgetDecl(w) => Some(w),
                _ => None,
            })
            .expect("widget decl");
        widget.view.as_ref().unwrap().root.clone()
    }

    /// 恒 Err 的方言：复现旧缺陷锚（mem::take 整表 + `?` 提前退出会丢整张
    /// 方言表）。Err 后三段注册必须原位。
    struct Plan778FailModelDialect;

    impl auto_lang_core::dialect::Dialect for Plan778FailModelDialect {
        fn matches(&self, s: &auto_lang_core::session::CompilerSession) -> bool {
            s.scenario == auto_lang_core::session::Scenario::UI
        }
        fn keywords(&self) -> &'static [&'static str] {
            &["model"]
        }
        fn try_parse_stmt(
            &self,
            _bridge: &mut auto_lang_core::parser_bridge::ParserBridge<'_, '_>,
            _keyword: &str,
        ) -> AutoResult<Option<Stmt>> {
            Err(SyntaxError::Generic {
                message: "plan778 intentional failure".to_string(),
                span: pos_to_span(auto_lang_core::token::Pos {
                    line: 0,
                    at: 0,
                    pos: 0,
                    len: 0,
                }),
            }
            .into())
        }
    }

    /// 消费 token 后返回 None 的方言：派发器必须在 None 后强制回滚游标，
    /// 内建方言从语句原位接管。
    struct Plan778ConsumeThenNoneDialect;

    impl auto_lang_core::dialect::Dialect for Plan778ConsumeThenNoneDialect {
        fn matches(&self, s: &auto_lang_core::session::CompilerSession) -> bool {
            s.scenario == auto_lang_core::session::Scenario::UI
        }
        fn keywords(&self) -> &'static [&'static str] {
            &["model"]
        }
        fn try_parse_stmt(
            &self,
            bridge: &mut auto_lang_core::parser_bridge::ParserBridge<'_, '_>,
            _keyword: &str,
        ) -> AutoResult<Option<Stmt>> {
            bridge.advance(); // 吞 "model"
            bridge.advance(); // 吞 "{"
            Ok(None) // 未迁形态让行——游标必须被派发器回滚
        }
    }

    /// Plan 451: actions 块解析——注册表/menubar/toolbar 全字段。
    #[test]
    fn test_actions_block_parse() {
        setup();
        let session = auto_lang_core::session::CompilerSession::ui();
        let src = "widget App {
    actions {
        action (id: \"file.new\", handler: .ActNew, title: \"新建\", icon: \"file-plus\", shortcut: \"Ctrl+N\")
        action (id: \"file.save\", handler: .ActSave, title: \"保存\", shortcut: \"Ctrl+S\",
                enabled_if: \".store.tab_count > 0\", checked_if: \".store.edits > 0\")
        menubar {
            menu (id: \"file\", title: \"文件\") { item (action: \"file.new\")  sep  item (action: \"file.save\") }
        }
        toolbar { item (action: \"file.new\")  sep  item (action: \"file.save\") }
    }
    on {
        .ActNew -> { console_log(\"x\") }
        .ActSave -> { console_log(\"x\") }
    }
}
";
        let mut parser = Parser::from(src).with_session(session);
        let ast = parser.parse().expect("actions block parses");
        let decl = match ast.stmts.first() {
            Some(auto_lang_core::ast::Stmt::WidgetDecl(d)) => d,
            _ => panic!("expected widget decl"),
        };
        let acts = decl.actions.as_ref().expect("actions block present");
        assert_eq!(acts.actions.len(), 2);
        assert_eq!(acts.actions[0].id, "file.new");
        assert_eq!(acts.actions[0].handler, ".ActNew");
        assert_eq!(acts.actions[0].shortcut.as_deref(), Some("Ctrl+N"));
        assert!(acts.actions[0].enabled_if.is_none());
        assert_eq!(
            acts.actions[1].enabled_if.as_deref(),
            Some(".store.tab_count > 0")
        );
        assert_eq!(
            acts.actions[1].checked_if.as_deref(),
            Some(".store.edits > 0")
        );
        let mb = acts.menubar.as_ref().expect("menubar");
        assert_eq!(mb.menus.len(), 1);
        assert_eq!(mb.menus[0].id, "file");
        assert_eq!(mb.menus[0].title, "文件");
        assert_eq!(mb.menus[0].items.len(), 3);
        assert!(matches!(
            mb.menus[0].items[1],
            auto_lang_core::ast::ui::MenuItemEntry::Sep
        ));
        let tb = acts.toolbar.as_ref().expect("toolbar");
        assert_eq!(tb.items.len(), 3);
    }

    /// Plan 451 P3: 顶层 `actions {}` 声明（模块级形态）；非声明用法的
    /// `actions`（表达式/调用）不受方言接管。
    #[test]
    fn test_top_level_actions_decl() {
        setup();
        let session = auto_lang_core::session::CompilerSession::ui();
        let src = "actions {
    action (id: \"file.new\", handler: .ActNew, title: \"新建\", shortcut: \"Ctrl+N\")
    menubar { menu (id: \"file\", title: \"文件\") { item (action: \"file.new\") } }
}
widget App {
    view { col { text \"hi\" {} } }
    on { .ActNew -> { console_log(\"x\") } }
}
";
        let mut parser = Parser::from(src).with_session(session);
        let ast = parser.parse().expect("top-level actions decl parses");
        let block = match ast.stmts.first() {
            Some(auto_lang_core::ast::Stmt::ActionsDecl(b)) => b,
            _ => panic!(
                "expected ActionsDecl, got {:?}",
                ast.stmts
                    .first()
                    .map(|s| matches!(s, auto_lang_core::ast::Stmt::ActionsDecl(_)))
            ),
        };
        assert_eq!(block.actions.len(), 1);
        assert_eq!(block.actions[0].id, "file.new");
        assert!(block.menubar.is_some());

        // `actions` 后非 `{`（表达式/调用形态）不受接管——仍按原语句解析。
        let session2 = auto_lang_core::session::CompilerSession::ui();
        let src2 = "let actions = 1
widget App { view { col { text \"hi\" {} } } }
";
        let mut parser2 = Parser::from(src2).with_session(session2);
        let ast2 = parser2
            .parse()
            .expect("plain `actions` identifier unaffected");
        assert!(
            !ast2
                .stmts
                .iter()
                .any(|s| matches!(s, auto_lang_core::ast::Stmt::ActionsDecl(_))),
            "no ActionsDecl for non-block `actions` usage"
        );
    }

    /// Plan 451 P2: enabled_if/checked_if 的裸表达式拼写与引号串等价。
    #[test]
    fn test_actions_cond_bare_expression() {
        setup();
        let session = auto_lang_core::session::CompilerSession::ui();
        let src = "widget App {
    actions {
        action (id: \"a\", handler: .ActA, title: \"x\", enabled_if: .n > 0)
        action (id: \"b\", handler: .ActB, title: \"y\", checked_if: (.m || .k) && !.z)
    }
    on { .ActA -> { console_log(\"x\") }  .ActB -> { console_log(\"x\") } }
}
";
        let mut parser = Parser::from(src).with_session(session);
        let ast = parser.parse().expect("bare conditions parse");
        let decl = match ast.stmts.first() {
            Some(auto_lang_core::ast::Stmt::WidgetDecl(d)) => d,
            _ => panic!("expected widget decl"),
        };
        let acts = decl.actions.as_ref().expect("actions");
        assert_eq!(acts.actions[0].enabled_if.as_deref(), Some(".n > 0"));
        assert_eq!(
            acts.actions[1].checked_if.as_deref(),
            Some("( .m || .k ) && ! .z")
        );
    }

    /// Plan 425 复审债务修复:块关键字拼写近失(如 `veiw {}`)在 view 可选化
    /// 后会静默解析为视图元素——现在告警提示正确拼写;合法元素不误报。
    #[test]
    fn test_widget_block_keyword_near_miss_warning() {
        setup();
        let session = auto_lang_core::session::CompilerSession::ui();
        let src = "widget App {
    veiw { col { text \"hi\" { } } }
}
";
        let mut parser = Parser::from(src).with_session(session);
        let ast = parser.parse().expect("near-miss still parses as element");
        assert!(matches!(
            ast.stmts.first(),
            Some(auto_lang_core::ast::Stmt::WidgetDecl(_))
        ));
        let hit = parser.warnings.iter().any(|w| {
            matches!(
                w,
                Warning::SuspiciousBlockKeyword { found, suggestion, .. }
                    if found == "veiw" && suggestion == "view"
            )
        });
        assert!(
            hit,
            "expected near-miss warning, got {} warnings",
            parser.warnings.len()
        );

        let session2 = auto_lang_core::session::CompilerSession::ui();
        let src2 = "widget App {
    col { text \"hi\" { } }
}
";
        let mut parser2 = Parser::from(src2).with_session(session2);
        parser2.parse().expect("legit element parses");
        assert!(
            !parser2
                .warnings
                .iter()
                .any(|w| matches!(w, Warning::SuspiciousBlockKeyword { .. })),
            "legit element must not warn"
        );
    }

    #[test]
    fn test_with_session_registers_ui_dialect() {
        setup();
        // PR-2: UI session 注册 UiDialect；Core session 方言表为空。
        let ui_parser =
            Parser::from("x = 1").with_session(auto_lang_core::session::CompilerSession::ui());
        // builtin 段恰 1 个（UiDialect），外部段为空（PLAN-778 三段表）
        assert_eq!(ui_parser.dialect_section_len(1), 1);
        assert_eq!(
            ui_parser.dialect_section_len(0) + ui_parser.dialect_section_len(2),
            0
        );

        let core_parser = Parser::from("x = 1");
        assert_eq!(core_parser.dialect_section_len(1), 0);
    }

    #[test]
    fn test_plan778_err_keeps_dialect_registration() {
        setup();
        let mut parser = Parser::from("model 123")
            .with_session(auto_lang_core::session::CompilerSession::ui())
            .with_dialect(
                Box::new(Plan778FailModelDialect),
                auto_lang_core::dialect::DialectPriority::Override,
            );
        assert!(parser.parse_stmt().is_err(), "方言恒 Err");
        // 旧缺陷下此处三段全空（Err 提前 return 未移回）。
        assert_eq!(parser.dialect_section_len(0), 1, "Err 后 pre 段注册不丢");
        assert_eq!(parser.dialect_section_len(1), 1, "Err 后 builtin 段不丢");
        assert_eq!(parser.dialect_section_len(2), 0);
    }

    #[test]
    fn test_plan778_none_rolls_back_cursor_for_next_dialect() {
        setup();
        let ast = Parser::from("model { count int = 1 }")
            .with_session(auto_lang_core::session::CompilerSession::ui())
            .with_dialect(
                Box::new(Plan778ConsumeThenNoneDialect),
                auto_lang_core::dialect::DialectPriority::Override,
            )
            .parse()
            .expect("None 回滚后内建必须从语句原位接管");
        match &ast.stmts[0] {
            Stmt::ModelBlock(b) => assert_eq!(b.fields.len(), 1),
            other => panic!("expected ModelBlock after rollback, got {:?}", other),
        }
    }

    #[test]
    fn test_plan778_with_session_preserves_external_registration() {
        setup();
        // 先注册、后 with_session：pre 段保留，builtin 段按新 session 重建。
        let parser = Parser::from("x = 1")
            .with_dialect(
                Box::new(Plan778ConsumeThenNoneDialect),
                auto_lang_core::dialect::DialectPriority::Append,
            )
            .with_session(auto_lang_core::session::CompilerSession::ui());
        assert_eq!(
            parser.dialect_section_len(2),
            1,
            "with_session 不清外部注册"
        );
        assert_eq!(
            parser.dialect_section_len(1),
            1,
            "builtin 段按 session 重建"
        );
    }

    #[test]
    fn test_top_level_view_block_now_parses_in_ui_scenario() {
        setup();
        // PR-2 修复：UI 场景下顶层 `view { }` 块现在可正确解析（原先落到 expr_stmt）。
        // view 树用小写标签语法（col/text 等），与现有 widget view 测试一致。
        let mut parser = Parser::from("view {\n  col {\n    text \"hello\"\n  }\n}")
            .with_session(auto_lang_core::session::CompilerSession::ui());
        let ast = parser.parse().unwrap();
        let last = ast.stmts.last().unwrap();
        assert!(
            matches!(last, Stmt::ViewBlock(_)),
            "expected ViewBlock, got {:?}",
            last
        );
    }

    /// Plan 041a⑥-b/c(已修,musk 041 Phase 5 T20):
    /// (b) 事件绑定值 `.msg(.field, fn($event))` 嵌套调用;
    /// (c) 括号表达式作 text 简写 `text (.row.idx + 1).to_string()`
    ///     ——`(` 后首 token 为 Dot 时按 primary-prop 表达式消歧
    ///     (props key 永不以 . 开头)。
    #[test]
    fn test_041a_event_binding_forms() {
        setup();
        // (b) 事件绑定多参 + 嵌套 fn 调用 + $event
        let code_b = concat!(
            "widget App {
",
            "  msg Msg { step(int, str) }
",
            "  view {
",
            "    input (value: .x, oninput: .step(.row.idx, stepValueOf($event)))
",
            "  }
",
            "}"
        );
        let mut parser =
            Parser::from(code_b).with_session(auto_lang_core::session::CompilerSession::ui());
        let ast = parser.parse().unwrap();
        assert!(ast.stmts.iter().any(|s| matches!(s, Stmt::WidgetDecl(_))));

        // (c) 括号表达式上的方法链
        let code_c = concat!(
            "widget App {
",
            "  view {
",
            "    span { text (.row.idx + 1).to_string() }
",
            "  }
",
            "}"
        );
        let mut parser =
            Parser::from(code_c).with_session(auto_lang_core::session::CompilerSession::ui());
        let ast = parser.parse().unwrap();
        assert!(ast.stmts.iter().any(|s| matches!(s, Stmt::WidgetDecl(_))));
    }

    /// Plan 041a⑥: 语句级 `.` 消息派发不得并入前一语句(对象字面量赋值后
    /// 换行 + 句首 .msg(...) 曾被 glue 成方法链)。最小复现守护。
    #[test]
    fn test_leading_dot_stmt_not_glued_to_previous() {
        setup();
        let code = concat!(
            "widget App {
",
            "  msg Msg { go(obj) }
",
            "  model { var m obj = {} }
",
            "  on {
",
            "    .go -> {
",
            "      let draft = {a: 1, b: 2};
",
            "      .go(draft)
",
            "    }
",
            "  }
",
            "}"
        );
        let count = |code: &str| -> usize {
            let mut parser =
                Parser::from(code).with_session(auto_lang_core::session::CompilerSession::ui());
            let ast = parser.parse().unwrap();
            let widget = ast
                .stmts
                .iter()
                .find_map(|s| match s {
                    Stmt::WidgetDecl(w) => Some(w),
                    _ => None,
                })
                .expect("widget decl");
            let handler = widget.on.as_ref().unwrap().handlers.first().unwrap();
            handler.body.stmts.len()
        };
        // 变体二分:带 ; / 不带 ; / 同行
        let with_semi = code;
        let no_semi = code.replace(
            "};
", "}
",
        );
        let same_line = code.replace(
            "};
      .go",
            "}
      let _z = .go",
        );
        assert_eq!(count(&with_semi), 2, "semi-terminated: two stmts");
        assert_eq!(count(&no_semi), 2, "newline-terminated: two stmts");
        let mut parser =
            Parser::from(code).with_session(auto_lang_core::session::CompilerSession::ui());
        let ast = parser.parse().unwrap();
        let widget = ast
            .stmts
            .iter()
            .find_map(|s| match s {
                Stmt::WidgetDecl(w) => Some(w),
                _ => None,
            })
            .expect("widget decl");
        assert!(widget.on.is_some(), "on block parsed");
        // 断言语句数与形状:let draft = {..}; 与 .go(draft) 是两条独立语句,
        // let 的值不得变成 {..}.go(...) 方法链。
        let handler = widget.on.as_ref().unwrap().handlers.first().unwrap();
        let stmts = &handler.body.stmts;
        assert_eq!(
            stmts.len(),
            2,
            "two separate stmts, got {}: {:#?}",
            stmts.len(),
            stmts
        );
        assert!(
            matches!(&stmts[0], auto_lang_core::ast::Stmt::Store(st)
                if matches!(st.kind, auto_lang_core::ast::StoreKind::Let)
                    && matches!(st.expr, auto_lang_core::ast::Expr::Object(_))),
            "let value must stay a plain object literal: {:#?}",
            stmts[0]
        );
    }

    #[test]
    fn test_widget_parses_via_dialect_in_ui_scenario() {
        setup();
        // PR-2: widget 经 UiDialect 派发，UI 场景下解析为 WidgetDecl。
        let code = "widget App {\n  model { count int = 0 }\n  view { text \"hello\" }\n}";
        let mut parser =
            Parser::from(code).with_session(auto_lang_core::session::CompilerSession::ui());
        let ast = parser.parse().unwrap();
        let last = ast.stmts.last().unwrap();
        assert!(matches!(last, Stmt::WidgetDecl(_)));
    }

    #[test]
    fn test_view_event_modifiers_and_event_object() {
        setup();
        // Generic DOM events: dot-modifier chains (onkeydown.up.prevent,
        // onwheel.document.capture) and the $event object parameter, in both
        // paren and brace prop styles.
        let code = concat!(
            "widget App {\n",
            "  model { var x int = 0 }\n",
            "  view {\n",
            "    col {\n",
            "      input (value: .x, onkeydown.up.prevent: .Move($event))\n",
            "      col { onwheel.document.capture: .Lock($event.key) }\n",
            "    }\n",
            "  }\n",
            "}"
        );
        let mut parser =
            Parser::from(code).with_session(auto_lang_core::session::CompilerSession::ui());
        let ast = parser.parse().unwrap();
        let widget = ast
            .stmts
            .iter()
            .find_map(|s| match s {
                Stmt::WidgetDecl(w) => Some(w),
                _ => None,
            })
            .expect("widget decl");
        let root = &widget.view.as_ref().unwrap().root;
        let children = match root {
            ViewNode::Element { children, .. } => children,
            other => panic!("expected element root, got {:?}", other),
        };
        // Paren style: modifier chain preserved in the event name.
        let input_events = match &children[0] {
            ViewNode::Element { events, .. } => events,
            other => panic!("expected input element, got {:?}", other),
        };
        assert_eq!(input_events[0].name, "onkeydown.up.prevent");
        assert_eq!(input_events[0].handler, ".Move");
        assert_eq!(input_events[0].params, vec!["$event".to_string()]);
        // Brace style: window/document target modifier + $event field access.
        let col_events = match &children[1] {
            ViewNode::Element { events, .. } => events,
            other => panic!("expected col element, got {:?}", other),
        };
        assert_eq!(col_events[0].name, "onwheel.document.capture");
        assert_eq!(col_events[0].handler, ".Lock");
        assert_eq!(col_events[0].params, vec!["$event.key".to_string()]);
    }

    #[test]
    fn test_view_event_arg_nested_self_dot() {
        setup();
        // Plan 043 M5 B-5: `.c.name` inside an event arg must parse as
        // `this.c.name` (one `this.` prefix + property access), NOT
        // `this.c` + `this.name` = `this.cthis.name`. The Vue backend strips
        // the `this.` prefix and would otherwise emit the bogus identifier
        // `cthis` while the v-for binds `c`.
        let code = concat!(
            "widget ToolSidebar {\n",
            "  model { var commands List<ToolEntry> = [] }\n",
            "  view {\n",
            "    for c in .commands {\n",
            "      button { onclick: .Pick(.c.name) }\n",
            "    }\n",
            "  }\n",
            "}"
        );
        let mut parser =
            Parser::from(code).with_session(auto_lang_core::session::CompilerSession::ui());
        let ast = parser.parse().unwrap();
        let widget = ast
            .stmts
            .iter()
            .find_map(|s| match s {
                Stmt::WidgetDecl(w) => Some(w),
                _ => None,
            })
            .expect("widget decl");
        let root = &widget.view.as_ref().unwrap().root;
        let for_loop = match root {
            ViewNode::ForLoop { body, .. } => &body[0],
            other => panic!("expected for loop root, got {:?}", other),
        };
        let btn_events = match for_loop {
            ViewNode::Element { events, .. } => events,
            other => panic!("expected button element, got {:?}", other),
        };
        assert_eq!(btn_events[0].handler, ".Pick");
        assert_eq!(btn_events[0].params, vec!["this.c.name".to_string()]);
    }

    #[test]
    fn test_view_quoted_custom_event_names() {
        setup();
        // Custom event names containing ':'/'-' (CustomEvent dispatched by
        // hand-written TS): on "autodown:slash-open".document — in both paren
        // and brace prop styles. Internally the name keeps its quotes:
        // on"autodown:slash-open".document.
        let code = concat!(
            "widget App {\n",
            "  model { var x int = 0 }\n",
            "  view {\n",
            "    col {\n",
            "      input (on \"autodown:slash-open\".document: .Open($event))\n",
            "      col { on \"autodown:slash-close\".document.capture: .Close($event.detail) }\n",
            "    }\n",
            "  }\n",
            "}"
        );
        let mut parser =
            Parser::from(code).with_session(auto_lang_core::session::CompilerSession::ui());
        let ast = parser.parse().unwrap();
        let widget = ast
            .stmts
            .iter()
            .find_map(|s| match s {
                Stmt::WidgetDecl(w) => Some(w),
                _ => None,
            })
            .expect("widget decl");
        let root = &widget.view.as_ref().unwrap().root;
        let children = match root {
            ViewNode::Element { children, .. } => children,
            other => panic!("expected element root, got {:?}", other),
        };
        let input_events = match &children[0] {
            ViewNode::Element { events, .. } => events,
            other => panic!("expected input element, got {:?}", other),
        };
        assert_eq!(input_events[0].name, "on\"autodown:slash-open\".document");
        assert_eq!(input_events[0].handler, ".Open");
        assert_eq!(input_events[0].params, vec!["$event".to_string()]);
        let col_events = match &children[1] {
            ViewNode::Element { events, .. } => events,
            other => panic!("expected col element, got {:?}", other),
        };
        assert_eq!(
            col_events[0].name,
            "on\"autodown:slash-close\".document.capture"
        );
        assert_eq!(col_events[0].handler, ".Close");
        assert_eq!(col_events[0].params, vec!["$event.detail".to_string()]);
    }

    #[test]
    fn test_view_style_obj_binding() {
        setup();
        // style_obj: { css_prop: expr } parses to a StyleBinding prop named
        // "style_obj" (inline-style form), distinct from style: (class map).
        let code = concat!(
            "widget App {\n",
            "  model { var y int = 0 }\n",
            "  view {\n",
            "    col {\n",
            "      style_obj: { top: f\"${.y}px\", \"z-index\": 50 }\n",
            "    }\n",
            "  }\n",
            "}"
        );
        let mut parser =
            Parser::from(code).with_session(auto_lang_core::session::CompilerSession::ui());
        let ast = parser.parse().unwrap();
        let widget = ast
            .stmts
            .iter()
            .find_map(|s| match s {
                Stmt::WidgetDecl(w) => Some(w),
                _ => None,
            })
            .expect("widget decl");
        let root = &widget.view.as_ref().unwrap().root;
        let props = match root {
            ViewNode::Element { props, .. } => props,
            other => panic!("expected element root, got {:?}", other),
        };
        let style_obj = props
            .iter()
            .find(|p| p.name == "style_obj")
            .expect("style_obj prop");
        match &style_obj.value {
            ViewPropValue::StyleBinding(entries) => {
                assert_eq!(entries.len(), 2);
                assert_eq!(entries[0].style_name, "top");
                assert_eq!(entries[1].style_name, "z-index");
            }
            other => panic!("expected StyleBinding, got {:?}", other),
        }
    }

    #[test]
    fn test_view_dyn_component_parens() {
        setup();
        // dyn (expr) { props } — the parenthesized expression becomes the
        // "is" prop; brace props/class parse like any other element.
        let code = concat!(
            "widget App {\n",
            "  model { var items list = [] }\n",
            "  view {\n",
            "    col {\n",
            "      for item in .items {\n",
            "        dyn (.item.icon) { size: 16, class: \"w-4 h-4\" }\n",
            "      }\n",
            "    }\n",
            "  }\n",
            "}"
        );
        let mut parser =
            Parser::from(code).with_session(auto_lang_core::session::CompilerSession::ui());
        let ast = parser.parse().unwrap();
        let widget = ast
            .stmts
            .iter()
            .find_map(|s| match s {
                Stmt::WidgetDecl(w) => Some(w),
                _ => None,
            })
            .expect("widget decl");
        let root = &widget.view.as_ref().unwrap().root;
        let children = match root {
            ViewNode::Element { children, .. } => children,
            other => panic!("expected element root, got {:?}", other),
        };
        let body = match &children[0] {
            ViewNode::ForLoop { body, .. } => body,
            other => panic!("expected for loop, got {:?}", other),
        };
        let (tag, props) = match &body[0] {
            ViewNode::Element { tag, props, .. } => (tag, props),
            other => panic!("expected dyn element, got {:?}", other),
        };
        assert_eq!(tag, "dyn");
        let is_prop = props.iter().find(|p| p.name == "is").expect("is prop");
        match &is_prop.value {
            ViewPropValue::Expr(Expr::Dot(obj, field)) => {
                assert_eq!(field.as_str(), "icon");
                // .item.icon → Dot(Dot(self, item), icon)
                match obj.as_ref() {
                    Expr::Dot(inner, item) => {
                        assert_eq!(item.as_str(), "item");
                        assert!(matches!(inner.as_ref(), Expr::Ident(n) if n.as_str() == "self"));
                    }
                    other => panic!("expected Dot(self, item), got {:?}", other),
                }
            }
            other => panic!("expected Dot expr for is, got {:?}", other),
        }
        assert!(props.iter().any(|p| p.name == "size"));
        assert!(props.iter().any(|p| p.name == "class"));
    }

    #[test]
    fn test_stmt_after_try_catch_parses() {
        setup();
        // Plan 029 T15 回归锁:try/catch 的 finally 前瞻曾无条件吞掉换行,
        // 导致其后任何语句(expect_eos 看不到分隔符)报
        // "Expected end of statement"。fn 体与 on 体都要能继续解析。
        let code = concat!(
            "fn c() str {\n",
            "    let mode = \"g\"\n",
            "    try {\n",
            "        let r = 1\n",
            "        mode = \"x\"\n",
            "    } catch {\n",
            "        let _i = 0\n",
            "    }\n",
            "    return mode\n",
            "}\n"
        );
        let mut parser =
            Parser::from(code).with_session(auto_lang_core::session::CompilerSession::ui());
        let ast = parser.parse().expect("stmt after try/catch must parse");
        match &ast.stmts[0] {
            Stmt::Fn(f) => assert_eq!(f.body.stmts.len(), 3, "let+try+return"),
            other => panic!("expected fn decl, got {:?}", other),
        }

        let code2 = concat!(
            "widget W {\n",
            "  msg Msg { S }\n",
            "  model { var e str = \"\" }\n",
            "  view { col { text \"hi\" } }\n",
            "  on {\n",
            "    .S -> {\n",
            "      try { .e = \"\" } catch { .e = \"f\" }\n",
            "      .e = \"after\"\n",
            "    }\n",
            "  }\n",
            "}"
        );
        let mut p2 =
            Parser::from(code2).with_session(auto_lang_core::session::CompilerSession::ui());
        let ast2 = p2
            .parse()
            .expect("stmt after try/catch in on-body must parse");
        let w = ast2
            .stmts
            .iter()
            .find_map(|s| match s {
                Stmt::WidgetDecl(w) => Some(w),
                _ => None,
            })
            .expect("widget");
        let on = w.on.as_ref().expect("on block");
        assert_eq!(on.handlers[0].body.stmts.len(), 2, "try + assignment after");
    }

    #[test]
    fn test_try_catch_finally_parses() {
        setup();
        // Plan 012 P2 (gap 4): optional `finally { }` after catch, in an
        // on-block handler body.
        let code = concat!(
            "widget App {\n",
            "  msg Msg { Save }\n",
            "  model { var error str = \"\" }\n",
            "  view { col { text \"hi\" } }\n",
            "  on {\n",
            "    .Save -> {\n",
            "      try {\n",
            "        .error = \"\"\n",
            "      } catch (e) {\n",
            "        .error = \"failed\"\n",
            "      } finally {\n",
            "        .error = \"done\"\n",
            "      }\n",
            "    }\n",
            "  }\n",
            "}"
        );
        let mut parser =
            Parser::from(code).with_session(auto_lang_core::session::CompilerSession::ui());
        let ast = parser.parse().unwrap();
        let widget = ast
            .stmts
            .iter()
            .find_map(|s| match s {
                Stmt::WidgetDecl(w) => Some(w),
                _ => None,
            })
            .expect("widget decl");
        let on = widget.on.as_ref().expect("on block");
        let try_stmt = on.handlers[0]
            .body
            .stmts
            .iter()
            .find_map(|s| match s {
                Stmt::Try(t) => Some(t),
                _ => None,
            })
            .expect("try stmt in handler body");
        assert_eq!(try_stmt.catch_param.as_deref(), Some("e"));
        assert!(try_stmt.finally_body.is_some(), "finally body parsed");
        assert_eq!(try_stmt.finally_body.as_ref().unwrap().stmts.len(), 1);
    }

    #[test]
    fn test_try_catch_without_finally_unchanged() {
        setup();
        // Regression lock: the Plan 010 form (no finally) still parses with
        // finally_body = None.
        let code = concat!(
            "widget App {\n",
            "  msg Msg { Save }\n",
            "  model { var error str = \"\" }\n",
            "  view { col { text \"hi\" } }\n",
            "  on {\n",
            "    .Save -> {\n",
            "      try { .error = \"\" } catch { .error = \"x\" }\n",
            "    }\n",
            "  }\n",
            "}"
        );
        let mut parser =
            Parser::from(code).with_session(auto_lang_core::session::CompilerSession::ui());
        let ast = parser.parse().unwrap();
        let widget = ast
            .stmts
            .iter()
            .find_map(|s| match s {
                Stmt::WidgetDecl(w) => Some(w),
                _ => None,
            })
            .expect("widget decl");
        let on = widget.on.as_ref().expect("on block");
        let try_stmt = on.handlers[0]
            .body
            .stmts
            .iter()
            .find_map(|s| match s {
                Stmt::Try(t) => Some(t),
                _ => None,
            })
            .expect("try stmt");
        assert!(try_stmt.catch_param.is_none(), "bare catch: no binding");
        assert!(try_stmt.finally_body.is_none(), "no finally clause");
    }

    #[test]
    fn test_try_catch_param_resolves_in_catch_body() {
        setup();
        // Regression: the `catch (e)` binding must be visible inside the
        // catch body — `rethrow(e)` used to fail name resolution with
        // UndefinedVariable because the param was never defined.
        let code = concat!(
            "widget App {\n",
            "  msg Msg { Save }\n",
            "  model { var error str = \"\" }\n",
            "  view { col { text \"hi\" } }\n",
            "  on {\n",
            "    .Save -> {\n",
            "      try { .error = \"\" } catch (e) { rethrow(e) }\n",
            "    }\n",
            "  }\n",
            "}"
        );
        let mut parser =
            Parser::from(code).with_session(auto_lang_core::session::CompilerSession::ui());
        let ast = parser.parse().unwrap();
        let widget = ast
            .stmts
            .iter()
            .find_map(|s| match s {
                Stmt::WidgetDecl(w) => Some(w),
                _ => None,
            })
            .expect("widget decl");
        let on = widget.on.as_ref().expect("on block");
        let try_stmt = on.handlers[0]
            .body
            .stmts
            .iter()
            .find_map(|s| match s {
                Stmt::Try(t) => Some(t),
                _ => None,
            })
            .expect("try stmt");
        assert_eq!(try_stmt.catch_param.as_deref(), Some("e"));
    }

    #[test]
    fn test_widget_watch_block() {
        setup();
        // watch { .a -> {...}  .b, .c.immediate -> {...}  .d.deep -> {...} }
        let code = concat!(
            "widget App {\n",
            "  model {\n",
            "    var q str = \"\"\n",
            "    var sel int = 0\n",
            "  }\n",
            "  view { col { text \"hi\" } }\n",
            "  watch {\n",
            "    .q -> { .sel = 0 }\n",
            "    .q, .sel.immediate -> { .sel = 1 }\n",
            "    .q.deep -> { .sel = 2 }\n",
            "  }\n",
            "}"
        );
        let mut parser =
            Parser::from(code).with_session(auto_lang_core::session::CompilerSession::ui());
        let ast = parser.parse().unwrap();
        let widget = ast
            .stmts
            .iter()
            .find_map(|s| match s {
                Stmt::WidgetDecl(w) => Some(w),
                _ => None,
            })
            .expect("widget decl");
        assert_eq!(widget.watch.len(), 3);
        assert_eq!(widget.watch[0].sources.len(), 1);
        assert_eq!(widget.watch[0].sources[0].as_str(), "q");
        assert!(!widget.watch[0].immediate);
        assert!(!widget.watch[0].deep);
        assert!(!widget.watch[0].body.stmts.is_empty());
        assert_eq!(widget.watch[1].sources.len(), 2);
        assert_eq!(widget.watch[1].sources[0].as_str(), "q");
        assert_eq!(widget.watch[1].sources[1].as_str(), "sel");
        assert!(widget.watch[1].immediate);
        assert!(!widget.watch[1].deep);
        assert!(widget.watch[2].deep);
    }

    #[test]
    fn test_widget_expose_block() {
        setup();
        // expose { .a, .b  .c } — dot-prefixed member names, comma-separated
        // and/or one per line.
        let code = concat!(
            "widget App {\n",
            "  model { var q str = \"\" }\n",
            "  view { col { text \"hi\" } }\n",
            "  expose {\n",
            "    .fit, .graphEl\n",
            "    .relayout\n",
            "  }\n",
            "}"
        );
        let mut parser =
            Parser::from(code).with_session(auto_lang_core::session::CompilerSession::ui());
        let ast = parser.parse().unwrap();
        let widget = ast
            .stmts
            .iter()
            .find_map(|s| match s {
                Stmt::WidgetDecl(w) => Some(w),
                _ => None,
            })
            .expect("widget decl");
        assert_eq!(widget.expose.len(), 3);
        assert_eq!(widget.expose[0].as_str(), "fit");
        assert_eq!(widget.expose[1].as_str(), "graphEl");
        assert_eq!(widget.expose[2].as_str(), "relayout");
    }

    #[test]
    fn test_struct_literal_fields_become_node_args() {
        setup();
        // Plan 043 M5: `var block = Block{ id: 1, command: "hi" }` in a store
        // handler must parse its fields as NAMED ARGS (not a node body).
        // Previously the braces fed parse_node_body, which can't parse
        // `field: value` as statements — the error was collected, fields were
        // dropped, and codegen emitted `{}` (command results never matched).
        // Needs the UI session so the PascalCase struct-construction fallback
        // applies (Block isn't registered in a standalone parse).
        let code = concat!(
            "store S {\n",
            "    on {\n",
            "        .Go -> {\n",
            "            var block = Block{ id: 1, command: \"hi\" }\n",
            "        }\n",
            "    }\n",
            "}\n",
        );
        let mut parser =
            Parser::from(code).with_session(auto_lang_core::session::CompilerSession::ui());
        let ast = parser.parse().unwrap();
        // Traverse: StoreDecl → on → handlers[0] → body stmts → the var Store.
        use auto_lang_core::ast::{Expr, Stmt};
        let stmt = ast
            .stmts
            .iter()
            .find(|s| matches!(s, Stmt::StoreDecl(_)))
            .expect("store decl");
        let Stmt::StoreDecl(decl) = stmt else {
            unreachable!()
        };
        let on = decl.on.as_ref().expect("on block");
        let handler = on
            .handlers
            .iter()
            .find(|h| h.pattern == ".Go")
            .expect(".Go handler");
        let var_store = handler
            .body
            .stmts
            .iter()
            .find_map(|s| match s {
                Stmt::Store(store) if store.name.as_str() == "block" => Some(store),
                _ => None,
            })
            .expect("var block store stmt");
        let Expr::Node(node) = &var_store.expr else {
            panic!(
                "block init should be a Node (struct literal), got: {:?}",
                var_store.expr
            );
        };
        assert_eq!(
            node.args.args.len(),
            2,
            "both fields as named args, got {:?}",
            node.args.args
        );
        assert!(
            node.args
                .args
                .iter()
                .any(|a| matches!(a, Arg::Pair(k, _) if k.as_str() == "id")),
            "id field present as named arg, got {:?}",
            node.args.args
        );
        assert!(
            node.args
                .args
                .iter()
                .any(|a| matches!(a, Arg::Pair(k, _) if k.as_str() == "command")),
            "command field present as named arg, got {:?}",
            node.args.args
        );
    }

    #[test]
    fn test_dot_rhs_field_access_not_struct_construction() {
        setup();
        // Plan 043 M5: `text cell.Text { }` in a view fn — the PascalCase
        // field `Text` is immediately followed by the element's props brace.
        // The struct-literal widening (any PascalCase ident + `{`) must NOT
        // fire for the RHS of a Dot: it would parse `Text{...}` as a Node,
        // swallow the props brace, and desync → "Expected term, got RBrace".
        // Struct literals are standalone `Type{...}` (`x = Block{...}`), never
        // `obj.Field{...}`.
        let code = concat!(
            "widget W {\n",
            "    view {\n",
            "        col {\n",
            "            RenderT(a: .a)\n",
            "        }\n",
            "    }\n",
            "}\n",
            "view fn RenderT(a Any) {\n",
            "    col {\n",
            "        for cell in a {\n",
            "            text cell.Text { }\n",
            "            if cell.Tagged != nil {\n",
            "                text cell.Tagged.text { }\n",
            "            }\n",
            "        }\n",
            "    }\n",
            "}\n",
        );
        let mut parser =
            Parser::from(code).with_session(auto_lang_core::session::CompilerSession::ui());
        let ast = parser
            .parse()
            .expect("widget view with PascalCase fields parses");
        // RenderT's view body must contain the for loop with the text element.
        use auto_lang_core::ast::Stmt;
        let view_fn = ast
            .stmts
            .iter()
            .find_map(|s| match s {
                Stmt::ViewFragmentDecl(vf) if vf.name.as_str() == "RenderT" => Some(vf),
                _ => None,
            })
            .expect("RenderT view fn decl");
        let root = view_fn.body.clone();
        // Root → col element → for loop → text element with a Dot primary prop.
        fn collect_text_primary(
            node: &auto_lang_core::ast::ui::ViewNode,
            out: &mut Vec<auto_lang_core::ast::Expr>,
        ) {
            use auto_lang_core::ast::ui::ViewNode;
            match node {
                ViewNode::Element {
                    tag,
                    props,
                    children,
                    ..
                } => {
                    if tag == "text" {
                        for p in props {
                            if p.name == "text" {
                                if let auto_lang_core::ast::ui::ViewPropValue::Expr(e) = &p.value {
                                    out.push(e.clone());
                                }
                            }
                        }
                    }
                    for c in children {
                        collect_text_primary(c, out);
                    }
                }
                ViewNode::ForLoop { body, .. } => {
                    for c in body {
                        collect_text_primary(c, out);
                    }
                }
                ViewNode::Conditional {
                    then_body,
                    else_body,
                    ..
                } => {
                    for c in then_body.iter().chain(else_body.iter().flatten()) {
                        collect_text_primary(c, out);
                    }
                }
                _ => {}
            }
        }
        let mut primaries = Vec::new();
        collect_text_primary(&root, &mut primaries);
        assert!(
            primaries.iter().any(|e| matches!(
                e,
                auto_lang_core::ast::Expr::Dot(obj, f)
                    if matches!(obj.as_ref(), auto_lang_core::ast::Expr::Ident(n) if n.as_str() == "cell")
                        && f.as_str() == "Text"
            )),
            "cell.Text parsed as field access (Dot), got: {:?}",
            primaries
        );
        assert!(
            primaries.iter().any(|e| matches!(
                e,
                auto_lang_core::ast::Expr::Dot(inner, f)
                    if f.as_str() == "text"
                        && matches!(
                            inner.as_ref(),
                            auto_lang_core::ast::Expr::Dot(o, ff)
                                if ff.as_str() == "Tagged"
                                    && matches!(o.as_ref(), auto_lang_core::ast::Expr::Ident(n) if n.as_str() == "cell")
                        )
            )),
            "cell.Tagged.text parsed as nested Dot, got: {:?}",
            primaries
        );
    }

    #[test]
    fn test_event_inline_lambda_parses() {
        setup();
        // Plan 448 B1: `onclick: () => { ... }` — inline lambda event value.
        // Parsing mints each lambda into a synthetic `__evt_<event>_<n>`
        // handler reference AND appends the matching on-handler + msg variant
        // to the decl (single source of truth shared with the decl-based VM
        // synthesis path). Legacy `.Inc` / `.Changed(value)` are untouched.
        let code = concat!(
            "widget App {\n",
            "    model { var count int = 0 }\n",
            "    view {\n",
            "        row {\n",
            "            button \"-\" { onclick: () => {.count -= 1} }\n",
            "            button \"=\" (onclick: () => {.count = 0})\n",
            "            button \"+\" { onclick: .Inc }\n",
            "            input (oninput: .Changed(value))\n",
            "        }\n",
            "    }\n",
            "}\n"
        );
        let session = auto_lang_core::session::CompilerSession::ui();
        let mut p = Parser::from(code).with_session(session);
        let ast = p.parse().expect("inline lambda event must parse");
        let w = ast
            .stmts
            .iter()
            .find_map(|s| match s {
                Stmt::WidgetDecl(w) => Some(w),
                _ => None,
            })
            .expect("widget");

        let events = |tag: &str| -> Vec<(String, Vec<String>, Option<usize>)> {
            fn walk(
                node: &ViewNode,
                tag: &str,
                out: &mut Vec<(String, Vec<String>, Option<usize>)>,
            ) {
                match node {
                    ViewNode::Element {
                        tag: t,
                        events,
                        children,
                        ..
                    } => {
                        if t == tag {
                            for e in events {
                                out.push((
                                    e.handler.clone(),
                                    e.params.clone(),
                                    e.inline.as_ref().map(|b| b.len()),
                                ));
                            }
                        }
                        for c in children {
                            walk(c, tag, out);
                        }
                    }
                    ViewNode::Component { .. } => {}
                    _ => {}
                }
            }
            let mut out = Vec::new();
            if let Some(v) = &w.view {
                walk(&v.root, tag, &mut out);
            }
            out
        };

        let btns = events("button");
        assert_eq!(btns.len(), 3);
        assert_eq!(btns[0].0, ".__evt_onclick_1", "minted handler reference");
        assert!(btns[0].1.is_empty());
        assert!(btns[0].2.is_none(), "inline consumed by minting");
        assert_eq!(btns[1].0, ".__evt_onclick_2", "paren-form minted too");
        assert!(btns[1].2.is_none());
        assert_eq!(btns[2].0, ".Inc", "legacy dot handler untouched");

        let inputs = events("input");
        assert_eq!(inputs.len(), 1);
        assert_eq!(inputs[0].0, ".Changed");
        assert_eq!(inputs[0].1, vec!["value".to_string()]);
        assert!(inputs[0].2.is_none());

        // The decl carries the minted on-handlers (bodies moved out of the
        // view) and the msg variants — the decl-based VM synthesis path
        // (VmBridge::new_from_decls) reads exactly these.
        let on = w.on.as_ref().expect("on block injected");
        let patterns: Vec<&str> = on.handlers.iter().map(|h| h.pattern.as_str()).collect();
        assert!(patterns.contains(&".__evt_onclick_1"));
        assert!(patterns.contains(&".__evt_onclick_2"));
        assert_eq!(
            on.handlers[0].body.stmts.len(),
            1,
            "lambda body moved into the handler"
        );
        let variants: Vec<&str> = w
            .messages
            .iter()
            .flat_map(|m| m.variants.iter().map(|v| v.name.as_str()))
            .collect();
        assert!(variants.contains(&"__evt_onclick_1"));
        assert!(variants.contains(&"__evt_onclick_2"));
    }

    #[test]
    fn test_bare_value_input_mints_oninput_sync() {
        setup();
        // Plan 448 C: `input { value: .x }` with no oninput/onchange mints an
        // empty-body `.__evt_oninput_<n>` binding (event + on-handler + msg
        // variant), so the typed text reaches state on every backend. Guards:
        // an explicit oninput/onchange/onenter and a multi-level dot value
        // (`.store.me.image`) suppress minting.
        let code = concat!(
            "widget App {\n",
            "    model { var email str = \"\" }\n",
            "    view {\n",
            "        col {\n",
            "            input { value: .email, placeholder: \"you@example.com\" }\n",
            "            input (value: .email, oninput: .Changed)\n",
            "            textarea { value: .email }\n",
            "            input { value: .store.me.email }\n",
            "            input { placeholder: \"no value\" }\n",
            "        }\n",
            "    }\n",
            "}\n"
        );
        let session = auto_lang_core::session::CompilerSession::ui();
        let mut p = Parser::from(code).with_session(session);
        let ast = p.parse().expect("bare value input must parse");
        let w = ast
            .stmts
            .iter()
            .find_map(|s| match s {
                Stmt::WidgetDecl(w) => Some(w),
                _ => None,
            })
            .expect("widget");

        #[derive(PartialEq, Debug)]
        struct Ev<'a> {
            tag: &'a str,
            name: &'a str,
            handler: &'a str,
            stmts: usize,
        }
        fn walk<'a>(node: &'a ViewNode, out: &mut Vec<Ev<'a>>) {
            match node {
                ViewNode::Element {
                    tag,
                    events,
                    children,
                    ..
                } => {
                    for e in events {
                        out.push(Ev {
                            tag,
                            name: &e.name,
                            handler: &e.handler,
                            stmts: e.inline.as_ref().map_or(0, |b| b.len()),
                        });
                    }
                    for c in children {
                        walk(c, out);
                    }
                }
                ViewNode::ForLoop { body, .. } => {
                    for c in body {
                        walk(c, out);
                    }
                }
                ViewNode::Conditional {
                    then_body,
                    else_body,
                    ..
                } => {
                    for c in then_body {
                        walk(c, out);
                    }
                    if let Some(els) = else_body {
                        for c in els {
                            walk(c, out);
                        }
                    }
                }
                _ => {}
            }
        }
        let mut evs = Vec::new();
        if let Some(v) = &w.view {
            walk(&v.root, &mut evs);
        }

        assert_eq!(evs.len(), 3, "only bare direct-state inputs/textarea mint");
        assert_eq!(
            evs[0],
            Ev {
                tag: "input",
                name: "oninput",
                handler: ".__bind_App_oninput_1",
                stmts: 0
            }
        );
        assert_eq!(
            evs[1],
            Ev {
                tag: "input",
                name: "oninput",
                handler: ".Changed",
                stmts: 0
            },
            "explicit oninput suppresses minting"
        );
        assert_eq!(
            evs[2],
            Ev {
                tag: "textarea",
                name: "oninput",
                handler: ".__bind_App_oninput_2",
                stmts: 0
            },
            "textarea mints too; counter is view-order deterministic"
        );

        // The minted handler is an empty-body on-handler + msg variant, so the
        // decl-based VM synthesis (VmBridge::new_from_decls) sees it and the
        // input_state_map writeback keys on its name.
        let on = w.on.as_ref().expect("on block injected");
        let minted = on
            .handlers
            .iter()
            .find(|h| h.pattern == ".__bind_App_oninput_1")
            .expect("minted on-handler");
        assert!(
            minted.body.stmts.is_empty(),
            "empty body — writeback happens pre-dispatch"
        );
        assert!(minted.params.is_empty(), "no phantom arg (audit B12(b))");
        let variants: Vec<&str> = w
            .messages
            .iter()
            .flat_map(|m| m.variants.iter().map(|v| v.name.as_str()))
            .collect();
        assert!(variants.contains(&"__bind_App_oninput_1"));
        assert!(variants.contains(&"__bind_App_oninput_2"));
    }

    /// PLAN-533 T5: 模态对话框自管开合铸造。无 `open` prop 的
    /// alert-dialog/dialog 根（vue 风格内建开合——gallery dialog 页形态）
    /// 铸造 state + toggle/close 双 handler + open 绑定，三轨同源。
    /// 显式 `open` 绑定不铸造（负例）。
    #[test]
    fn test_modal_dialog_mints_self_managed_open() {
        setup();
        let code = concat!(
            "widget App {\n",
            "    view {\n",
            "        dialog {\n",
            "            dialog-trigger \"Open Dialog\"\n",
            "            dialog-content {\n",
            "                dialog-header {\n",
            "                    dialog-title \"Edit Profile\"\n",
            "                }\n",
            "                dialog-footer {\n",
            "                    dialog-close \"Save changes\"\n",
            "                }\n",
            "            }\n",
            "        }\n",
            "    }\n",
            "}\n"
        );
        let session = auto_lang_core::session::CompilerSession::ui();
        let mut p = Parser::from(code).with_session(session);
        let ast = p.parse().expect("unbound dialog must parse");
        let w = ast
            .stmts
            .iter()
            .find_map(|s| match s {
                Stmt::WidgetDecl(w) => Some(w),
                _ => None,
            })
            .expect("widget");

        // 铸造 state：`var __dlg_open_1 bool = false`
        let model = w.model.as_ref().expect("model block minted");
        let field = model
            .fields
            .iter()
            .find(|f| f.name.as_str() == "__dlg_open_1")
            .expect("minted state var");
        assert!(matches!(field.init, Expr::Bool(false)), "init false");
        assert!(matches!(field.ty, Type::Bool), "typed bool");

        // 根节点补 open 绑定，形态与真实解析的 `.state` 完全一致。
        fn find_dialog<'a>(node: &'a ViewNode) -> Option<&'a ViewNode> {
            match node {
                ViewNode::Element { tag, children, .. } => {
                    if tag == "dialog" {
                        return Some(node);
                    }
                    children.iter().find_map(find_dialog)
                }
                _ => None,
            }
        }
        let dlg = find_dialog(&w.view.as_ref().unwrap().root).expect("dialog root");
        let open = dlg_element_props(dlg)
            .iter()
            .find(|p| p.name == "open")
            .expect("open prop minted");
        // 对照：真实解析 `(open: .show)` 的表达式形态。
        let bound_code = "widget B { view { dialog (open: .show) { } } }";
        let mut p2 =
            Parser::from(bound_code).with_session(auto_lang_core::session::CompilerSession::ui());
        let ast2 = p2.parse().expect("bound dialog parses");
        let w2 = ast2
            .stmts
            .iter()
            .find_map(|s| match s {
                Stmt::WidgetDecl(w) => Some(w),
                _ => None,
            })
            .expect("widget 2");
        let real = find_dialog(&w2.view.as_ref().unwrap().root).unwrap();
        let real_open = dlg_element_props(real)
            .iter()
            .find(|p| p.name == "open")
            .unwrap();
        let ViewPropValue::Expr(real_expr) = &real_open.value else {
            panic!("bound open prop must be an expr");
        };
        let ViewPropValue::Expr(mint_expr) = &open.value else {
            panic!("minted open prop must be an expr");
        };
        assert_eq!(
            format!("{:?}", mint_expr),
            format!("{:?}", real_expr).replace("show", "__dlg_open_1"),
            "minted open binding must match real `.state` parse shape"
        );

        // trigger/close 补 onclick。
        fn events_of<'a>(node: &'a ViewNode, tag: &str) -> Vec<(String, String)> {
            fn walk<'a>(node: &'a ViewNode, tag: &str, out: &mut Vec<(String, String)>) {
                if let ViewNode::Element {
                    tag: t,
                    events,
                    children,
                    ..
                } = node
                {
                    if t == tag {
                        for e in events {
                            out.push((e.name.clone(), e.handler.clone()));
                        }
                    }
                    for c in children {
                        walk(c, tag, out);
                    }
                }
            }
            let mut out = Vec::new();
            walk(node, tag, &mut out);
            out
        }
        let root = &w.view.as_ref().unwrap().root;
        let trig = events_of(root, "dialog-trigger");
        assert!(
            trig.iter()
                .any(|(n, h)| n == "onclick" && h == ".__dlg_toggle_1"),
            "trigger onclick minted: {trig:?}"
        );
        let close = events_of(root, "dialog-close");
        assert!(
            close
                .iter()
                .any(|(n, h)| n == "onclick" && h == ".__dlg_close_1"),
            "close onclick minted: {close:?}"
        );

        // 折叠进 on + msg（decl-based VM 合成路径读取面）。
        let on = w.on.as_ref().expect("on block injected");
        let patterns: Vec<&str> = on.handlers.iter().map(|h| h.pattern.as_str()).collect();
        assert!(
            patterns.contains(&".__dlg_toggle_1"),
            "toggle handler folded: {patterns:?}"
        );
        assert!(
            patterns.contains(&".__dlg_close_1"),
            "close handler folded: {patterns:?}"
        );
        let variants: Vec<&str> = w
            .messages
            .iter()
            .flat_map(|m| m.variants.iter().map(|v| v.name.as_str()))
            .collect();
        assert!(variants.contains(&"__dlg_toggle_1"));
        assert!(variants.contains(&"__dlg_close_1"));

        // 负例：显式 open 绑定不铸造。
        let w2_open = dlg_element_props(real);
        assert!(
            w2_open.iter().any(|p| p.name == "open"),
            "bound form has its own open"
        );
        let trig2 = events_of(&w2.view.as_ref().unwrap().root, "dialog-trigger");
        assert!(trig2.is_empty(), "bound form mints nothing: {trig2:?}");
        assert!(
            !w2.model
                .as_ref()
                .map(|m| m
                    .fields
                    .iter()
                    .any(|f| f.name.as_str().starts_with("__dlg_")))
                .unwrap_or(false),
            "bound form mints no state"
        );
    }

    /// PLAN-534 T7: hovercard 自管开合铸造（hover 形态）。无 `open` prop
    /// 的 hovercard 根铸造同一 `__dlg_open_<n>` state + open 绑定,但
    /// trigger 无显式事件时补 onmouseenter/onmouseleave（体:置 true/false）
    /// 而非 onclick toggle;显式绑定不覆盖（负例）。
    #[test]
    fn test_hover_card_mints_hover_enter_leave() {
        setup();
        let code = concat!(
            "widget App {\n",
            "    view {\n",
            "        hover-card {\n",
            "            hover-card-trigger {\n",
            "                text \"@mentor\"\n",
            "            }\n",
            "            hover-card-content {\n",
            "                text \" Mentor bio \"\n",
            "            }\n",
            "        }\n",
            "    }\n",
            "}\n"
        );
        let session = auto_lang_core::session::CompilerSession::ui();
        let mut p = Parser::from(code).with_session(session);
        let ast = p.parse().expect("unbound hovercard must parse");
        let w = ast
            .stmts
            .iter()
            .find_map(|s| match s {
                Stmt::WidgetDecl(w) => Some(w),
                _ => None,
            })
            .expect("widget");

        // 铸造 state：`var __dlg_open_1 bool = false`（与 dialog 机器同槽）。
        let model = w.model.as_ref().expect("model block minted");
        let field = model
            .fields
            .iter()
            .find(|f| f.name.as_str() == "__dlg_open_1")
            .expect("minted state var");
        assert!(matches!(field.init, Expr::Bool(false)), "init false");

        // 根节点补 open 绑定。
        fn find_tag<'a>(node: &'a ViewNode, tag: &str) -> Option<&'a ViewNode> {
            match node {
                ViewNode::Element {
                    tag: t, children, ..
                } => {
                    if t == tag {
                        return Some(node);
                    }
                    children.iter().find_map(|c| find_tag(c, tag))
                }
                _ => None,
            }
        }
        let root_node = &w.view.as_ref().unwrap().root;
        let hc = find_tag(root_node, "hover-card").expect("hover-card root");
        let open = dlg_element_props(hc)
            .iter()
            .find(|p| p.name == "open")
            .expect("open prop minted");
        let ViewPropValue::Expr(mint_expr) = &open.value else {
            panic!("minted open prop must be an expr");
        };
        assert!(
            format!("{mint_expr:?}").contains("__dlg_open_1"),
            "open binding must reference minted state: {mint_expr:?}"
        );

        // trigger 补 hover 进/出（非 onclick toggle）。
        fn events_of<'a>(node: &'a ViewNode, tag: &str) -> Vec<(String, String)> {
            fn walk<'a>(node: &'a ViewNode, tag: &str, out: &mut Vec<(String, String)>) {
                if let ViewNode::Element {
                    tag: t,
                    events,
                    children,
                    ..
                } = node
                {
                    if t == tag {
                        for e in events {
                            out.push((e.name.clone(), e.handler.clone()));
                        }
                    }
                    for c in children {
                        walk(c, tag, out);
                    }
                }
            }
            let mut out = Vec::new();
            walk(node, tag, &mut out);
            out
        }
        let trig = events_of(root_node, "hover-card-trigger");
        assert!(
            trig.iter()
                .any(|(n, h)| n == "onmouseenter" && h == ".__dlg_enter_1"),
            "trigger onmouseenter minted: {trig:?}"
        );
        assert!(
            trig.iter()
                .any(|(n, h)| n == "onmouseleave" && h == ".__dlg_leave_1"),
            "trigger onmouseleave minted: {trig:?}"
        );
        assert!(
            !trig.iter().any(|(n, _)| n == "onclick"),
            "hover form must NOT mint onclick toggle: {trig:?}"
        );

        // 折叠进 on + msg。
        let on = w.on.as_ref().expect("on block injected");
        let patterns: Vec<&str> = on.handlers.iter().map(|h| h.pattern.as_str()).collect();
        assert!(
            patterns.contains(&".__dlg_enter_1"),
            "enter handler folded: {patterns:?}"
        );
        assert!(
            patterns.contains(&".__dlg_leave_1"),
            "leave handler folded: {patterns:?}"
        );

        // 负例：显式 hover 绑定不覆盖。
        let bound_code = concat!(
            "widget B {\n",
            "    model { var opened bool = false }\n",
            "    view {\n",
            "        hover-card (open: .opened) {\n",
            "            hover-card-trigger (onmouseenter: .opened) {\n",
            "                text \"@mentor\"\n",
            "            }\n",
            "            hover-card-content { text \"bio\" }\n",
            "        }\n",
            "    }\n",
            "}\n"
        );
        let mut p2 =
            Parser::from(bound_code).with_session(auto_lang_core::session::CompilerSession::ui());
        let ast2 = p2.parse().expect("bound hovercard parses");
        let w2 = ast2
            .stmts
            .iter()
            .find_map(|s| match s {
                Stmt::WidgetDecl(w) => Some(w),
                _ => None,
            })
            .expect("widget 2");
        let trig2 = events_of(&w2.view.as_ref().unwrap().root, "hover-card-trigger");
        assert!(
            trig2
                .iter()
                .any(|(n, h)| n == "onmouseenter" && h == ".opened"),
            "explicit onmouseenter preserved: {trig2:?}"
        );
        assert!(
            !trig2.iter().any(|(n, _)| n == "onmouseleave"),
            "bound form mints no extra hover events: {trig2:?}"
        );
        assert!(
            !w2.model
                .as_ref()
                .map(|m| m
                    .fields
                    .iter()
                    .any(|f| f.name.as_str().starts_with("__dlg_")))
                .unwrap_or(false),
            "bound form mints no state"
        );
    }

    #[test]
    fn test_msg_decl_unnamed_and_legacy_name() {
        setup();
        // Plan 448 A: canonical `msg { ... }` (no type name — the name was
        // never semantic) plus legacy `msg Name { ... }` compat where the
        // name token is read and discarded.
        let code = concat!(
            "widget Counter {\n",
            "    msg { Inc, Dec, Set(int) }\n",
            "    model { var count int = 0 }\n",
            "    view { col { text \"hi\" } }\n",
            "    on { .Inc -> { .count = .count + 1 } }\n",
            "}\n"
        );
        let session = auto_lang_core::session::CompilerSession::ui();
        let mut p = Parser::from(code).with_session(session);
        let ast = p.parse().expect("unnamed msg decl must parse");
        let w = ast
            .stmts
            .iter()
            .find_map(|s| match s {
                Stmt::WidgetDecl(w) => Some(w),
                _ => None,
            })
            .expect("widget");
        assert_eq!(w.messages.len(), 1);
        let msg = &w.messages[0];
        assert_eq!(msg.variants.len(), 3);
        assert_eq!(msg.variants[0].name.as_str(), "Inc");
        assert_eq!(msg.variants[2].name.as_str(), "Set");
        assert_eq!(msg.variants[2].payload.len(), 1);

        let legacy = concat!("widget Counter {\n", "    msg Msg { Inc }\n", "}\n");
        let session = auto_lang_core::session::CompilerSession::ui();
        let mut p2 = Parser::from(legacy).with_session(session);
        let ast2 = p2.parse().expect("legacy named msg decl must still parse");
        let w2 = ast2
            .stmts
            .iter()
            .find_map(|s| match s {
                Stmt::WidgetDecl(w) => Some(w),
                _ => None,
            })
            .expect("widget");
        assert_eq!(w2.messages[0].variants.len(), 1);
        assert_eq!(w2.messages[0].variants[0].name.as_str(), "Inc");
    }

    #[test]
    fn test_widget_parsing_pipeline() {
        setup();
        // Test the full widget parsing pipeline
        let code = r#"
widget Counter {
    msg Msg { Inc, Dec }
    model { count int = 0 }
    view {
        col {
            text "Hello"
            button "-" { onclick: .Dec }
            button "+" { onclick: .Inc }
        }
    }
    on {
        Inc -> { count = count + 1 }
        Dec -> { count = count - 1 }
    }
}
"#;
        // Create parser with UI scenario enabled
        let session =
            auto_lang_core::session::CompilerSession::new(auto_lang_core::session::Scenario::UI);
        let mut parser = Parser::from(code).with_session(session);
        let result = parser.parse();

        match result {
            Ok(ast) => {
                // Check that we got a widget declaration
                let non_empty: Vec<_> = ast
                    .stmts
                    .iter()
                    .filter(|s| !matches!(s, Stmt::EmptyLine(_)))
                    .collect();
                assert_eq!(non_empty.len(), 1, "Should have one widget statement");

                if let Stmt::WidgetDecl(widget) = non_empty[0] {
                    assert_eq!(widget.name.as_str(), "Counter");
                    assert_eq!(widget.messages.len(), 1);
                    assert_eq!(widget.messages[0].variants.len(), 2);

                    // Check view tree exists
                    assert!(widget.view.is_some(), "View should be parsed");

                    // Check model exists
                    assert!(widget.model.is_some(), "Model should be parsed");

                    println!("Widget parsed successfully!");
                } else {
                    panic!("Expected WidgetDecl, got {:?}", non_empty[0]);
                }
            }
            Err(e) => {
                // Print detailed error for debugging
                panic!("Widget parsing failed: {:?}", e);
            }
        }
    }

    #[test]
    fn test_view_top_level_bare_siblings_with_param_events() {
        setup();
        // Plan 015 (auto-down 批次, docs/plans/DEBTS.md 015 🟡): bare top-level sibling
        // elements with parameterized events used to fail with
        // "Expected term, got RBrace" (the view block parsed a single root
        // node only); the same shape inside `col { }` worked. Now multiple
        // top-level siblings are wrapped in an implicit `col`.
        let code = r#"
widget W {
    msg Msg { A(int), B(int) }
    view {
        div { onclick: .A(1) }
        div { onclick: .B(2) }
    }
}
"#;
        let session =
            auto_lang_core::session::CompilerSession::new(auto_lang_core::session::Scenario::UI);
        let mut parser = Parser::from(code).with_session(session);
        let result = parser.parse();

        match result {
            Ok(ast) => {
                let non_empty: Vec<_> = ast
                    .stmts
                    .iter()
                    .filter(|s| !matches!(s, Stmt::EmptyLine(_)))
                    .collect();
                assert_eq!(non_empty.len(), 1, "Should have one widget statement");

                if let Stmt::WidgetDecl(widget) = non_empty[0] {
                    let view = widget.view.as_ref().expect("View should be parsed");
                    // Multiple top-level siblings → implicit col wrapper
                    if let ViewNode::Element { tag, children, .. } = &view.root {
                        assert_eq!(tag, "col", "siblings should be wrapped in an implicit col");
                        assert_eq!(children.len(), 2, "both sibling divs survive");
                        for child in children {
                            if let ViewNode::Element { tag, events, .. } = child {
                                assert_eq!(tag, "div");
                                assert_eq!(events.len(), 1);
                                assert_eq!(events[0].name, "onclick");
                                assert_eq!(events[0].params.len(), 1, "event arg must survive");
                            } else {
                                panic!("Expected div element child, got {:?}", child);
                            }
                        }
                    } else {
                        panic!("Expected implicit col wrapper, got {:?}", view.root);
                    }
                } else {
                    panic!("Expected WidgetDecl, got {:?}", non_empty[0]);
                }
            }
            Err(e) => {
                panic!("Widget parsing failed: {:?}", e);
            }
        }
    }

    #[test]
    fn test_view_single_root_stays_unwrapped() {
        setup();
        // Plan 015: a single top-level node must NOT gain a wrapper —
        // regression guard that the multi-sibling fix keeps the existing
        // single-root AST shape byte-identical.
        let code = r#"
widget W {
    msg Msg { A(int) }
    view {
        div { onclick: .A(1) }
    }
}
"#;
        let session =
            auto_lang_core::session::CompilerSession::new(auto_lang_core::session::Scenario::UI);
        let mut parser = Parser::from(code).with_session(session);
        let result = parser.parse();

        match result {
            Ok(ast) => {
                let non_empty: Vec<_> = ast
                    .stmts
                    .iter()
                    .filter(|s| !matches!(s, Stmt::EmptyLine(_)))
                    .collect();
                if let Stmt::WidgetDecl(widget) = non_empty[0] {
                    let view = widget.view.as_ref().expect("View should be parsed");
                    if let ViewNode::Element { tag, .. } = &view.root {
                        assert_eq!(tag, "div", "single root must stay the div itself");
                    } else {
                        panic!("Expected div root, got {:?}", view.root);
                    }
                } else {
                    panic!("Expected WidgetDecl, got {:?}", non_empty[0]);
                }
            }
            Err(e) => {
                panic!("Widget parsing failed: {:?}", e);
            }
        }
    }

    #[test]
    fn test_widget_with_routes() {
        setup();
        // Test widget with routes block (Plan 105)
        let code = r#"
widget App {
    routes {
        "/" -> HomePage {}
        "/button" -> ButtonPage {}
        "/user/:id" -> UserPage {}
    }
    view {
        col {
            outlet
        }
    }
}
"#;
        // Create parser with UI scenario enabled
        let session =
            auto_lang_core::session::CompilerSession::new(auto_lang_core::session::Scenario::UI);
        let mut parser = Parser::from(code).with_session(session);
        let result = parser.parse();

        match result {
            Ok(ast) => {
                let non_empty: Vec<_> = ast
                    .stmts
                    .iter()
                    .filter(|s| !matches!(s, Stmt::EmptyLine(_)))
                    .collect();
                assert_eq!(non_empty.len(), 1, "Should have one widget statement");

                if let Stmt::WidgetDecl(widget) = non_empty[0] {
                    assert_eq!(widget.name.as_str(), "App");

                    // Check routes exist
                    assert!(widget.routes.is_some(), "Routes should be parsed");

                    let routes = widget.routes.as_ref().unwrap();
                    assert_eq!(routes.routes.len(), 3);
                    assert_eq!(routes.routes[0].path, "/");
                    assert_eq!(routes.routes[0].module, "homepage"); // lowercase (backward compat)
                    assert_eq!(routes.routes[1].path, "/button");
                    assert_eq!(routes.routes[2].path, "/user/:id");
                    assert_eq!(routes.routes[2].params, vec!["id"]);

                    println!("Widget with routes parsed successfully!");
                } else {
                    panic!("Expected WidgetDecl, got {:?}", non_empty[0]);
                }
            }
            Err(e) => {
                panic!("Widget with routes parsing failed: {:?}", e);
            }
        }
    }

    #[test]
    fn test_msg_variant_named_payload() {
        setup();
        // Plan 435 挂账②:命名参数形态 `Change(value: str)`(gallery 官方组件在用)
        let code = r#"
widget T {
    msg Msg {
        Change(value: str)
        Complete(name: str, count: int)
        Set(bool)
        Go
    }
    model { n int = 0 }
    view { text "x" {} }
}
"#;
        let session =
            auto_lang_core::session::CompilerSession::new(auto_lang_core::session::Scenario::UI);
        let mut parser = Parser::from(code).with_session(session);
        let ast = parser.parse().expect("parse");
        let mut variants = vec![];
        for stmt in &ast.stmts {
            if let Stmt::WidgetDecl(w) = stmt {
                if let Some(msg) = w.messages.first() {
                    variants = msg.variants.clone();
                }
            }
        }
        assert_eq!(variants.len(), 4);
        assert_eq!(variants[0].name.as_str(), "Change");
        assert_eq!(variants[0].payload_names, vec![Some("value".to_string())]);
        assert_eq!(variants[1].payload_names[0], Some("name".to_string()));
        assert_eq!(variants[1].payload_names[1], Some("count".to_string()));
        // 位置形态与无参形态不受影响
        assert_eq!(variants[2].payload.len(), 1);
        assert!(variants[2].payload_names.iter().all(|n| n.is_none()));
        assert!(variants[3].payload.is_empty());
        assert!(variants[3].payload_names.is_empty());
    }

    #[test]
    fn test_hyphenated_tag_names() {
        setup();
        // Test that hyphenated tag names parse correctly in view blocks (Task 3)
        let code = r#"
widget Test {
    view {
        preview-card (id: "test") {
            button-primary {}
        }
    }
}
"#;
        // Create parser with UI scenario enabled
        let session =
            auto_lang_core::session::CompilerSession::new(auto_lang_core::session::Scenario::UI);
        let mut parser = Parser::from(code).with_session(session);
        let result = parser.parse();

        match result {
            Ok(ast) => {
                let non_empty: Vec<_> = ast
                    .stmts
                    .iter()
                    .filter(|s| !matches!(s, Stmt::EmptyLine(_)))
                    .collect();
                assert_eq!(non_empty.len(), 1, "Should have one widget statement");

                if let Stmt::WidgetDecl(widget) = non_empty[0] {
                    assert_eq!(widget.name.as_str(), "Test");

                    // Check view tree exists
                    assert!(widget.view.is_some(), "View should be parsed");

                    println!("Hyphenated tag names parsed successfully!");
                } else {
                    panic!("Expected WidgetDecl, got {:?}", non_empty[0]);
                }
            }
            Err(e) => {
                panic!("Hyphenated tag names parsing failed: {:?}", e);
            }
        }
    }

    #[test]
    fn test_string_as_primary_prop_h1() {
        setup();
        // h1 "Title" → h1 (text: "Title")
        let code = r#"widget Test { view { col { h1 "ContextMenu" } } }"#;
        let session =
            auto_lang_core::session::CompilerSession::new(auto_lang_core::session::Scenario::UI);
        let mut parser = Parser::from(code).with_session(session);
        let result = parser.parse();
        assert!(result.is_ok());

        if let Ok(ast) = result {
            if let Stmt::WidgetDecl(widget) = &ast.stmts[0] {
                if let Some(view) = &widget.view {
                    if let ViewNode::Element { tag, children, .. } = &view.root {
                        assert_eq!(tag, "col");
                        assert_eq!(children.len(), 1);
                        if let ViewNode::Element { tag, props, .. } = &children[0] {
                            assert_eq!(tag, "h1");
                            assert_eq!(props.len(), 1);
                            assert_eq!(props[0].name, "text");
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn test_string_as_primary_prop_text() {
        setup();
        // text "content" creates a text element with primary prop
        let code = r#"widget Test { view { col { text "Hello World" } } }"#;
        let session =
            auto_lang_core::session::CompilerSession::new(auto_lang_core::session::Scenario::UI);
        let mut parser = Parser::from(code).with_session(session);
        let result = parser.parse();
        assert!(result.is_ok());

        if let Ok(ast) = result {
            if let Stmt::WidgetDecl(widget) = &ast.stmts[0] {
                if let Some(view) = &widget.view {
                    if let ViewNode::Element { children, .. } = &view.root {
                        assert_eq!(children.len(), 1);
                        // text "Hello World" is parsed as an Element with tag "text"
                        if let ViewNode::Element { tag, props, .. } = &children[0] {
                            assert_eq!(tag, "text");
                            assert!(props.iter().any(|p| p.name == "text"));
                        } else {
                            panic!("Expected Element node with tag 'text'");
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn test_string_with_special_chars() {
        setup();
        // String with em dash and other special characters should work
        let code = r#"widget Test { view { text "Displays a menu — such as a set of actions" } }"#;
        let session =
            auto_lang_core::session::CompilerSession::new(auto_lang_core::session::Scenario::UI);
        let mut parser = Parser::from(code).with_session(session);
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_string_as_primary_prop_with_additional_props() {
        setup();
        // button "Click" (class: "btn") { onclick: .Test }
        let code =
            r#"widget Test { view { button "Click me" (class: "btn") { onclick: .Test } } }"#;
        let session =
            auto_lang_core::session::CompilerSession::new(auto_lang_core::session::Scenario::UI);
        let mut parser = Parser::from(code).with_session(session);
        let result = parser.parse();
        assert!(result.is_ok());

        if let Ok(ast) = result {
            if let Stmt::WidgetDecl(widget) = &ast.stmts[0] {
                if let Some(view) = &widget.view {
                    if let ViewNode::Element {
                        tag, props, events, ..
                    } = &view.root
                    {
                        assert_eq!(tag, "button");
                        // Should have text prop from string literal
                        assert!(props.iter().any(|p| p.name == "text"));
                        // Should have class prop from parentheses (parser now keeps "class" as-is)
                        assert!(props.iter().any(|p| p.name == "class" || p.name == "style"));
                        // Should have onclick event
                        assert!(!events.is_empty());
                    }
                }
            }
        }
    }

    #[test]
    fn test_gt_syntax_not_text_shorthand() {
        setup();
        // Test: `> "text"` should be parsed as element with tag ">" and no primary prop
        let code = r#"widget Test { view { col { > "Hello" } } }"#;
        let session =
            auto_lang_core::session::CompilerSession::new(auto_lang_core::session::Scenario::UI);
        let mut parser = Parser::from(code).with_session(session);
        let result = parser.parse();

        // Parsing succeeds - `>` is treated as an element tag name
        assert!(result.is_ok());

        // Navigate to the child node and verify it's an element with tag ">"
        if let Ok(ast) = result {
            if let Stmt::WidgetDecl(widget) = &ast.stmts[0] {
                if let Some(view) = &widget.view {
                    // Root is col, find the child
                    if let ViewNode::Element { tag, children, .. } = &view.root {
                        assert_eq!(tag, "col");
                        // The child should be an element with tag ">"
                        assert_eq!(children.len(), 1);
                        if let ViewNode::Element { tag, .. } = &children[0] {
                            assert_eq!(
                                tag, ">",
                                "The '>' should be parsed as element tag, not text syntax"
                            );
                            // No primary prop for ">" element
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn test_pipe_syntax_works_for_text() {
        setup();
        // Verify the correct pipe syntax works for text
        let code = r#"widget Test { view { col { | "Hello" } } }"#;
        let session =
            auto_lang_core::session::CompilerSession::new(auto_lang_core::session::Scenario::UI);
        let mut parser = Parser::from(code).with_session(session);
        let result = parser.parse();
        assert!(result.is_ok(), "Pipe syntax should work for text nodes");
    }

    #[test]
    fn test_aura_dot_prefixed_handler_syntax() {
        setup();
        // Test: on { .Inc => { .count = .count + 1 } }
        let code = r#"
widget Counter {
    msg Msg { Inc, Dec }
    model { var count int = 0 }
    view {
        col {
            text "Counter Demo"
            text f"Count: ${.count}"
            row {
                button "-" { onclick: .Dec }
                button "+" { onclick: .Inc }
            }
        }
    }
    on {
        .Inc -> { .count = .count + 1 }
        .Dec -> { .count = .count - 1 }
    }
}
"#;
        let session =
            auto_lang_core::session::CompilerSession::new(auto_lang_core::session::Scenario::UI);
        let mut parser = Parser::from(code).with_session(session);
        let result = parser.parse();

        match result {
            Ok(ast) => {
                let non_empty: Vec<_> = ast
                    .stmts
                    .iter()
                    .filter(|s| !matches!(s, Stmt::EmptyLine(_)))
                    .collect();
                assert_eq!(non_empty.len(), 1, "Should have one widget statement");

                if let Stmt::WidgetDecl(widget) = non_empty[0] {
                    assert_eq!(widget.name.as_str(), "Counter");
                    assert_eq!(widget.messages.len(), 1);
                    assert_eq!(widget.messages[0].variants.len(), 2);

                    // Check model has mutable count field
                    assert!(widget.model.is_some());
                    let model = widget.model.as_ref().unwrap();
                    assert_eq!(model.fields.len(), 1);
                    assert_eq!(model.fields[0].name.as_str(), "count");
                    assert!(
                        model.fields[0].mutable,
                        "count should be mutable with var keyword"
                    );

                    // Check on block has dot-prefixed handlers
                    assert!(widget.on.is_some());
                    let on = widget.on.as_ref().unwrap();
                    assert_eq!(on.handlers.len(), 2);

                    // Verify handler patterns are dot-prefixed
                    assert_eq!(on.handlers[0].pattern, ".Inc");
                    assert_eq!(on.handlers[1].pattern, ".Dec");

                    println!("Dot-prefixed handler syntax parsed successfully!");
                } else {
                    panic!("Expected WidgetDecl, got {:?}", non_empty[0]);
                }
            }
            Err(e) => {
                panic!("Dot-prefixed handler syntax parsing failed: {:?}", e);
            }
        }
    }

    #[test]
    fn test_aura_model_var_keyword() {
        setup();
        // Test: model { var count int = 0 } creates mutable field
        //      model { count int = 0 } creates immutable field (default)
        let mutable_code = r#"
widget Test {
    model { var count int = 0 }
    view { col { text "test" } }
}
"#;
        let immutable_code = r#"
widget Test {
    model { count int = 0 }
    view { col { text "test" } }
}
"#;

        // Test mutable field
        let session =
            auto_lang_core::session::CompilerSession::new(auto_lang_core::session::Scenario::UI);
        let mut parser = Parser::from(mutable_code).with_session(session);
        let result = parser.parse();
        assert!(result.is_ok());

        if let Ok(ast) = result {
            if let Stmt::WidgetDecl(widget) = &ast.stmts[0] {
                let model = widget.model.as_ref().unwrap();
                assert!(model.fields[0].mutable, "Field with var should be mutable");
            }
        }

        // Test immutable field (default)
        let session2 =
            auto_lang_core::session::CompilerSession::new(auto_lang_core::session::Scenario::UI);
        let mut parser2 = Parser::from(immutable_code).with_session(session2);
        let result2 = parser2.parse();
        assert!(result2.is_ok());

        if let Ok(ast) = result2 {
            if let Stmt::WidgetDecl(widget) = &ast.stmts[0] {
                let model = widget.model.as_ref().unwrap();
                assert!(
                    !model.fields[0].mutable,
                    "Field without var should be immutable"
                );
            }
        }
    }

    /// Plan 043 M5 #1: msg variants with multiple payload params must parse.
    /// Previously `Complete(str, int)` failed with "Expected term, got RBrace"
    /// because the payload parser consumed only a single type.
    #[test]
    fn test_msg_multi_param_payload() {
        setup();
        let code = r#"
widget Shell {
    msg Msg { Init, Complete(str, int), RunSmart(int, str, []str), SetTag(str) }
    model { count int = 0 }
    view { col { text "hi" } }
}
"#;
        let session =
            auto_lang_core::session::CompilerSession::new(auto_lang_core::session::Scenario::UI);
        let mut parser = Parser::from(code).with_session(session);
        let result = parser.parse();

        let ast = result.expect("multi-param msg should parse");
        let non_empty: Vec<_> = ast
            .stmts
            .iter()
            .filter(|s| !matches!(s, Stmt::EmptyLine(_)))
            .collect();
        let widget = match non_empty[0] {
            Stmt::WidgetDecl(w) => w,
            other => panic!("expected WidgetDecl, got {:?}", other),
        };
        let msg = &widget.messages[0];
        assert_eq!(msg.variants.len(), 4, "all 4 variants should parse");

        // Init — unit variant, empty payload.
        assert!(msg.variants[0].payload.is_empty(), "Init has no payload");

        // Complete(str, int) — two payload types.
        assert_eq!(msg.variants[1].name.as_str(), "Complete");
        assert_eq!(
            msg.variants[1].payload.len(),
            2,
            "Complete has 2 payload types"
        );

        // RunSmart(int, str, []str) — three payload types.
        assert_eq!(msg.variants[2].name.as_str(), "RunSmart");
        assert_eq!(
            msg.variants[2].payload.len(),
            3,
            "RunSmart has 3 payload types"
        );

        // SetTag(str) — single payload still works (regression guard).
        assert_eq!(msg.variants[3].name.as_str(), "SetTag");
        assert_eq!(
            msg.variants[3].payload.len(),
            1,
            "SetTag has 1 payload type"
        );
    }

    /// Plan 043 M5 #1 regression: the full Plan 043 shell_store msg declaration
    /// (15 variants, mixing unit / single-param user-type / multi-param with
    /// []str) must parse end-to-end. This is the exact form that failed at
    /// `ClearScreen }` in the real shell_store.at.
    #[test]
    fn test_msg_full_shellstore_declaration() {
        setup();
        let code = r#"
type CommandResult { code int }
type CommandOutput { line str }

store S {
    model { x int = 0 }
    msg Msg { Init, RunCommand(str), RunResult(CommandResult), RunOutput(CommandOutput),
              Cancel, RunSmart(int, str, []str), Complete(str, int),
              NavigateHistory(bool), OpenHistorySearch, CloseHistorySearch,
              SelectHistory(str), ToggleSidebar, PickTool(str), RefreshGit, ClearScreen }
}
"#;
        let session =
            auto_lang_core::session::CompilerSession::new(auto_lang_core::session::Scenario::UI);
        let mut parser = Parser::from(code).with_session(session);
        let result = parser.parse();
        let ast = result.expect("full shell_store msg should parse");
        let store = ast
            .stmts
            .iter()
            .find_map(|s| match s {
                Stmt::StoreDecl(s) => Some(s),
                _ => None,
            })
            .expect("store declaration present");
        let msg = &store.messages[0];
        assert_eq!(msg.variants.len(), 15, "all 15 variants parse");
        // Spot-check the multi-param ones.
        let runsmart = msg
            .variants
            .iter()
            .find(|v| v.name.as_str() == "RunSmart")
            .unwrap();
        assert_eq!(runsmart.payload.len(), 3, "RunSmart has 3 payload types");
        let complete = msg
            .variants
            .iter()
            .find(|v| v.name.as_str() == "Complete")
            .unwrap();
        assert_eq!(complete.payload.len(), 2, "Complete has 2 payload types");
    }

    /// Plan 043 M5: view conditions must accept `None`/`nil` literals
    /// (e.g. `if .block.output != None`). parse_condition_expr previously had
    /// no NoneKW/Nil branch, so the `None` was left unconsumed and the parser
    /// desynced, surfacing as "Expected term, got RBrace" at the arm's brace.
    #[test]
    fn test_view_condition_none_comparison() {
        setup();
        let code = r#"
widget W {
    model { x str = "" }
    view {
        col {
            if .x != None {
                text "set"
            }
            if .x == None {
                text "unset"
            }
        }
    }
}
"#;
        let session =
            auto_lang_core::session::CompilerSession::new(auto_lang_core::session::Scenario::UI);
        let mut parser = Parser::from(code).with_session(session);
        let ast = parser.parse().expect("view None comparison should parse");
        let non_empty: Vec<_> = ast
            .stmts
            .iter()
            .filter(|s| !matches!(s, Stmt::EmptyLine(_)))
            .collect();
        assert_eq!(non_empty.len(), 1, "one widget statement");
    }

    /// Plan 043 M5: `Type{ field: value }` construction must parse even when
    /// the type is imported (`use types:`) and not resolvable in the current
    /// file, or fully undeclared. atom() previously gated struct construction
    /// on `is_type` (lookup_ident_type), which returns None for unresolved
    /// imports. Now widened for PascalCase idents (UI scenario only) so
    /// `PromptContext{ git_branch: "", git_status: None }` parses.
    #[test]
    fn test_struct_literal_construction_unresolved_type() {
        setup();
        // Undeclared PascalCase type — must construct.
        let code = r#"
store S {
    model { var x PromptContext = PromptContext{ git_branch: "", git_status: None } }
}
"#;
        let session =
            auto_lang_core::session::CompilerSession::new(auto_lang_core::session::Scenario::UI);
        let mut parser = Parser::from(code).with_session(session);
        let ast = parser
            .parse()
            .expect("undeclared PascalCase struct literal should construct");
        let _ = ast; // parsed OK is the contract

        // Lowercase idents must NOT be treated as struct construction (they
        // stay regular identifiers), even in the UI scenario.
        let code2 = r#"
widget W {
    model { var x str = "" }
    view { col { text "hi" } }
}
"#;
        let session2 =
            auto_lang_core::session::CompilerSession::new(auto_lang_core::session::Scenario::UI);
        let mut parser2 = Parser::from(code2).with_session(session2);
        let _ = parser2
            .parse()
            .expect("lowercase model fields parse normally");
    }

    /// Plan 043 M5: a parameterized on-handler must bind ALL its params, both
    /// for bare names `.Run(a, b, c)` and for name+type `.Run(a int, b int)`.
    /// Previously the param parser kept only even-indexed tokens (assuming
    /// `name type` pairs), which dropped every other bare name — so
    /// `.RunSmart(block_id, name, args)` lost `name`.
    #[test]
    fn test_on_handler_multi_param_binding() {
        setup();
        fn handler_params(code: &str) -> Vec<String> {
            let session = auto_lang_core::session::CompilerSession::new(
                auto_lang_core::session::Scenario::UI,
            );
            let mut parser = Parser::from(code).with_session(session);
            let ast = parser.parse().expect("should parse");
            let store = ast
                .stmts
                .iter()
                .find_map(|s| match s {
                    Stmt::StoreDecl(s) => Some(s),
                    _ => None,
                })
                .expect("store present");
            store.on.as_ref().expect("on block present").handlers[0]
                .params
                .clone()
        }

        // 3 bare params — all must survive.
        let p = handler_params(
            r#"
store S {
    model { x int = 0 }
    msg Msg { Run(int, str, []str) }
    on { .Run(block_id, name, args) -> { .x = 0 } }
}
"#,
        );
        assert_eq!(
            p,
            vec![
                "block_id".to_string(),
                "name".to_string(),
                "args".to_string()
            ],
            "all 3 bare param names bound"
        );

        // name+type form — names kept, types dropped.
        let p = handler_params(
            r#"
store S {
    model { x int = 0 }
    msg Msg { Add(int, int) }
    on { .Add(a int, b int) -> { .x = a } }
}
"#,
        );
        assert_eq!(
            p,
            vec!["a".to_string(), "b".to_string()],
            "name+type params: names kept, types dropped"
        );

        // Single param (regression guard).
        let p = handler_params(
            r#"
store S {
    model { x int = 0 }
    msg Msg { Set(int) }
    on { .Set(v) -> { .x = v } }
}
"#,
        );
        assert_eq!(p, vec!["v".to_string()], "single param still works");
    }

    /// Plan 043 M5 #2: a computed property with a multi-statement block body
    /// must parse. Previously only single expressions parsed (`x => .foo`);
    /// `x => { var y = ...; return y }` failed with "Expected term, got RBrace".
    #[test]
    fn test_computed_multiline_body() {
        setup();
        let code = r#"
widget Counter {
    model { count int = 0 }
    computed {
        doubled => count * 2
        summary => {
            var label = "count="
            label = label + count
            return label
        }
    }
    view { col { text "hi" } }
}
"#;
        let session =
            auto_lang_core::session::CompilerSession::new(auto_lang_core::session::Scenario::UI);
        let mut parser = Parser::from(code).with_session(session);
        let ast = parser
            .parse()
            .expect("multiline computed body should parse");

        let non_empty: Vec<_> = ast
            .stmts
            .iter()
            .filter(|s| !matches!(s, Stmt::EmptyLine(_)))
            .collect();
        let widget = match non_empty[0] {
            Stmt::WidgetDecl(w) => w,
            other => panic!("expected WidgetDecl, got {:?}", other),
        };
        let computed = widget.computed.as_ref().expect("computed block present");
        assert_eq!(computed.properties.len(), 2);

        // First property: single expression (regression guard — still works).
        assert!(
            !matches!(computed.properties[0].expr, Expr::Block(_)),
            "single-expression computed stays a bare expr"
        );

        // Second property: multi-statement block body.
        match &computed.properties[1].expr {
            Expr::Block(body) => {
                assert_eq!(body.stmts.len(), 3, "summary body has 3 statements");
                // Last statement is the return.
                assert!(
                    matches!(body.stmts.last(), Some(Stmt::Return(_))),
                    "body ends in a return"
                );
            }
            other => panic!(
                "expected Expr::Block for multiline computed, got {:?}",
                other
            ),
        }
    }

    /// PLAN-607 T-01: Style Recipe declaration parsing tests (constant, parametric, composite, pub, doc, and fallback).
    #[test]
    fn test_style_recipe_parse() {
        setup();
        let code = r#"
/// Base card style
style card_base = "bg-card rounded-xl shadow-sm border border-border"

style pill(bg: str = "bg-primary", fg: str = "text-primary-foreground", pad: str = "px-4 py-2") =
    "{pad} {bg} {fg} rounded-full"

style pill_danger = pill(bg: "bg-destructive", fg: "text-destructive-foreground")

pub style brand_btn = "bg-brand text-white"

style = 123
"#;
        let mut parser =
            Parser::from(code).with_session(auto_lang_core::session::CompilerSession::ui());
        let code_ast = parser
            .parse()
            .expect("should parse style recipes successfully");

        // 1. card_base
        let card_base = code_ast
            .stmts
            .iter()
            .find_map(|s| match s {
                Stmt::StyleRecipeDecl(r) if r.name == "card_base" => Some(r),
                _ => None,
            })
            .expect("card_base style recipe should exist");
        assert_eq!(card_base.doc.as_deref(), Some("Base card style"));
        assert!(!card_base.is_pub);
        assert_eq!(card_base.params.len(), 0);

        // 2. pill
        let pill = code_ast
            .stmts
            .iter()
            .find_map(|s| match s {
                Stmt::StyleRecipeDecl(r) if r.name == "pill" => Some(r),
                _ => None,
            })
            .expect("pill style recipe should exist");
        assert_eq!(pill.params.len(), 3);
        assert_eq!(pill.params[0].name.as_str(), "bg");
        assert!(pill.params[0].default_value.is_some());
        assert_eq!(pill.params[1].name.as_str(), "fg");
        assert_eq!(pill.params[2].name.as_str(), "pad");

        // 3. pill_danger
        let pill_danger = code_ast
            .stmts
            .iter()
            .find_map(|s| match s {
                Stmt::StyleRecipeDecl(r) if r.name == "pill_danger" => Some(r),
                _ => None,
            })
            .expect("pill_danger style recipe should exist");
        assert_eq!(pill_danger.params.len(), 0);

        // 4. brand_btn (pub)
        let brand_btn = code_ast
            .stmts
            .iter()
            .find_map(|s| match s {
                Stmt::StyleRecipeDecl(r) if r.name == "brand_btn" => Some(r),
                _ => None,
            })
            .expect("brand_btn style recipe should exist");
        assert!(brand_btn.is_pub);

        // 5. style = 123 should NOT be a StyleRecipeDecl
        let non_recipe = code_ast.stmts.iter().any(|s| match s {
            Stmt::StyleRecipeDecl(r) if r.name == "style" => true,
            _ => false,
        });
        assert!(
            !non_recipe,
            "style = 123 must not be parsed as a StyleRecipeDecl"
        );
    }

    /// `for x in .items key: x.id { }` → key_expr = Dot(Ident(x), id).
    #[test]
    fn plan046_keyed_for_item_field_chain() {
        setup();
        let root =
            plan046_parse_root_for("    for x in .items key: x.id {\n      text \"r\"\n    }");
        match root {
            ViewNode::ForLoop { var, key_expr, .. } => {
                assert_eq!(var, "x");
                let key_expr = key_expr.expect("key clause must parse");
                assert!(
                    matches!(key_expr, Expr::Dot(ref l, ref n)
                        if matches!(**l, Expr::Ident(ref v) if v.as_str() == "x")
                            && n.as_str() == "id"),
                    "unexpected key expr: {:?}",
                    key_expr
                );
            }
            other => panic!("expected ForLoop root, got {:?}", other),
        }
    }

    /// `key: .id` → self-rooted state chain (same shape as expression position).
    #[test]
    fn plan046_keyed_for_state_chain() {
        setup();
        let root =
            plan046_parse_root_for("    for x in .items key: .user.id {\n      text \"r\"\n    }");
        match root {
            ViewNode::ForLoop { key_expr, .. } => {
                let key_expr = key_expr.expect("key clause must parse");
                let src = format!("{:?}", key_expr);
                assert!(src.contains("self"), "state chain is self-rooted: {}", src);
                assert!(src.contains("user") && src.contains("id"), "{}", src);
            }
            other => panic!("expected ForLoop root, got {:?}", other),
        }
    }

    /// Str literal key (`key: "abc"`) and Int literal key (`key: 5`).
    #[test]
    fn plan046_keyed_for_scalar_literals() {
        setup();
        let root =
            plan046_parse_root_for("    for x in .items key: \"abc\" {\n      text \"r\"\n    }");
        match root {
            ViewNode::ForLoop {
                key_expr: Some(Expr::Str(s)),
                ..
            } => assert_eq!(s, "abc"),
            other => panic!("expected Str key, got {:?}", other),
        }
        let root = plan046_parse_root_for("    for x in .items key: 5 {\n      text \"r\"\n    }");
        match root {
            ViewNode::ForLoop {
                key_expr: Some(Expr::Int(v)),
                ..
            } => assert_eq!(v, 5),
            other => panic!("expected Int key, got {:?}", other),
        }
    }

    /// Index form composes: `for i, x in .items key: x.id`.
    #[test]
    fn plan046_keyed_for_with_index_var() {
        setup();
        let root =
            plan046_parse_root_for("    for i, x in .items key: x.id {\n      text \"r\"\n    }");
        match root {
            ViewNode::ForLoop {
                index, key_expr, ..
            } => {
                assert_eq!(index.as_deref(), Some("i"));
                assert!(key_expr.is_some(), "key clause must survive index form");
            }
            other => panic!("expected ForLoop root, got {:?}", other),
        }
    }

    /// Keyless loop is byte-identical modulo the new None field (AC-01 前提).
    #[test]
    fn plan046_keyless_for_has_no_key_expr() {
        setup();
        let root = plan046_parse_root_for("    for x in .items {\n      text \"r\"\n    }");
        match root {
            ViewNode::ForLoop {
                var,
                key_expr,
                body,
                ..
            } => {
                assert_eq!(var, "x");
                assert!(key_expr.is_none(), "keyless loop must have key_expr None");
                assert_eq!(body.len(), 1);
            }
            other => panic!("expected ForLoop root, got {:?}", other),
        }
    }

    /// `memo (deps: .a, .b) { … }` → deps 拍平为两个表达式、exact=false。
    #[test]
    fn plan046_memo_block_parses_deps_and_body() {
        setup();
        let root = plan046_parse_root_memo(
            "    memo (deps: .a, .b) {
      text \"hi\"
    }",
        );
        match root {
            ViewNode::MemoBlock {
                deps, exact, body, ..
            } => {
                assert_eq!(deps.len(), 2, "逗号列表拍平为逐项 deps");
                assert!(!exact);
                assert_eq!(body.len(), 1);
                let src = format!("{:?}", deps);
                assert!(src.contains("a") && src.contains("b"), "{}", src);
            }
            other => panic!("expected MemoBlock root, got {:?}", other),
        }
    }

    /// `exact: true` 旗标与 `deps: .x, exact: true` 混排。
    #[test]
    fn plan046_memo_block_exact_and_mixed_params() {
        setup();
        let root = plan046_parse_root_memo(
            "    memo (exact: true, deps: .x) {
      text \"h\"
    }",
        );
        match root {
            ViewNode::MemoBlock { deps, exact, .. } => {
                assert!(exact, "exact 旗标在册");
                assert_eq!(deps.len(), 1);
            }
            other => panic!("expected MemoBlock, got {:?}", other),
        }
        let root = plan046_parse_root_memo(
            "    memo (deps: .a, .b, exact: false) {
      text \"h\"
    }",
        );
        match root {
            ViewNode::MemoBlock { deps, exact, .. } => {
                assert!(!exact);
                assert_eq!(deps.len(), 2, "deps 在 exact 前截断正确");
            }
            other => panic!("expected MemoBlock, got {:?}", other),
        }
    }

    /// PLAN-046 T-05：`outlet (memo: true)` 语料 prop 解析；裸 outlet 兼容；
    /// 非 `memo: true` 头参指名报错。
    #[test]
    fn plan046_outlet_prop_parsing() {
        setup();
        let code = concat!(
            "widget App {
",
            "  view {
",
            "    col {
",
            "      outlet (memo: true)
",
            "    }
",
            "  }
",
            "}"
        );
        let mut parser =
            Parser::from(code).with_session(auto_lang_core::session::CompilerSession::ui());
        let ast = parser.parse().expect("outlet prop must parse");
        let widget = ast
            .stmts
            .iter()
            .find_map(|s| match s {
                Stmt::WidgetDecl(w) => Some(w),
                _ => None,
            })
            .expect("widget decl");
        let children = match &widget.view.as_ref().unwrap().root {
            ViewNode::Element { children, .. } => children,
            other => panic!("expected col root, got {:?}", other),
        };
        match &children[0] {
            ViewNode::Outlet { memo } => assert_eq!(*memo, Some(true), "memo prop 在册"),
            other => panic!("expected Outlet, got {:?}", other),
        }

        // 裸 outlet：memo=None（未设；PLAN-708 T-01 三态——缺省语义由
        // builder 三态表裁定，解析层不折叠）。
        let code = "widget App {
  view {
    outlet
  }
}";
        let mut parser =
            Parser::from(code).with_session(auto_lang_core::session::CompilerSession::ui());
        let ast = parser.parse().expect("bare outlet must parse");
        let widget = ast
            .stmts
            .iter()
            .find_map(|s| match s {
                Stmt::WidgetDecl(w) => Some(w),
                _ => None,
            })
            .expect("widget decl");
        match &widget.view.as_ref().unwrap().root {
            ViewNode::Outlet { memo } => assert_eq!(*memo, None),
            other => panic!("expected Outlet, got {:?}", other),
        }

        // PLAN-708 T-01（S-01）: `outlet (memo: false)` 显式关（新接受形态）。
        let code = "widget App {
  view {
    outlet (memo: false)
  }
}";
        let mut parser =
            Parser::from(code).with_session(auto_lang_core::session::CompilerSession::ui());
        let ast = parser.parse().expect("explicit-false outlet must parse");
        let widget = ast
            .stmts
            .iter()
            .find_map(|s| match s {
                Stmt::WidgetDecl(w) => Some(w),
                _ => None,
            })
            .expect("widget decl");
        match &widget.view.as_ref().unwrap().root {
            ViewNode::Outlet { memo } => assert_eq!(*memo, Some(false)),
            other => panic!("expected Outlet, got {:?}", other),
        }

        // 非 memo: true 头参 → 解析错误。
        let code = "widget App {
  view {
    outlet (foo: 1)
  }
}";
        let mut parser =
            Parser::from(code).with_session(auto_lang_core::session::CompilerSession::ui());
        assert!(parser.parse().is_err(), "non-memo outlet header must error");
    }

    /// 嵌套形态：memo 块体内 for/if 照常解析。
    #[test]
    fn plan046_memo_block_body_nodes() {
        setup();
        let root = plan046_parse_root_memo(
            "    memo (deps: .items) {
      for x in .items {
        text \"r\"
      }
    }",
        );
        match root {
            ViewNode::MemoBlock { body, .. } => {
                assert_eq!(body.len(), 1);
                assert!(matches!(body[0], ViewNode::ForLoop { .. }), "体内 for 在册");
            }
            other => panic!("expected MemoBlock, got {:?}", other),
        }
    }
}

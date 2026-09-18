//! PLAN-024: named view parsing (`view mini { ... }`).
//!
//! Language extension for the desktop dashboard (auto-os shell S10): a widget
//! may declare additional named views alongside its main `view`. All named
//! views share the widget's store/model; the main `view` slot is untouched.
//! Guards: duplicate named views and duplicate main `view` are parse errors
//! (main-view duplicates were previously a SILENT OVERRIDE — regression fix),
//! while `view fn` fragments (top level) and `view` parameter mode in `fn`
//! signatures keep their existing semantics.

#[cfg(test)]
mod plan024_named_view_tests {
    use crate::ast::{Stmt, ViewNode};
    use crate::error::AutoError;
    use crate::parser::Parser;
    use crate::session::CompilerSession;

    fn parse_ui(code: &str) -> Result<crate::ast::Code, AutoError> {
        let mut parser = Parser::from(code).with_session(CompilerSession::ui());
        parser.parse()
    }

    fn widget_decl(code: &crate::ast::Code) -> &crate::ast::WidgetDecl {
        code.stmts
            .iter()
            .find_map(|s| match s {
                Stmt::WidgetDecl(w) => Some(w),
                _ => None,
            })
            .expect("widget decl")
    }

    /// `view mini { ... }` lands in `named_views`; the main `view` slot is
    /// untouched.
    #[test]
    fn named_view_parses_into_decl() {
        let code = parse_ui(
            r#"
widget Clock {
    model { now str = "" }
    view {
        col { text { "Clock" } }
    }
    view mini {
        col { text { "Mini" } }
    }
}
"#,
        )
        .expect("parse");
        let w = widget_decl(&code);
        assert!(w.view.is_some(), "main view must stay in its own slot");
        assert_eq!(w.named_views.len(), 1);
        assert_eq!(w.named_views[0].0.as_str(), "mini");
    }

    /// Multiple named views on one widget.
    #[test]
    fn multiple_named_views() {
        let code = parse_ui(
            r#"
widget Clock {
    view { col { text { "Main" } } }
    view mini { col { text { "Mini" } } }
    view wide { col { text { "Wide" } } }
}
"#,
        )
        .expect("parse");
        let w = widget_decl(&code);
        assert_eq!(w.named_views.len(), 2);
        assert_eq!(w.named_views[0].0.as_str(), "mini");
        assert_eq!(w.named_views[1].0.as_str(), "wide");
    }

    /// Named view body parses as a real view tree (element root reachable).
    #[test]
    fn named_view_body_is_view_tree() {
        let code = parse_ui(
            r#"
widget Clock {
    view { col { text { "Main" } } }
    view mini {
        col {
            text { "12:00" }
        }
    }
}
"#,
        )
        .expect("parse");
        let w = widget_decl(&code);
        match &w.named_views[0].1.root {
            ViewNode::Element { tag, .. } => assert_eq!(tag, "col"),
            other => panic!("expected element root, got {:?}", std::mem::discriminant(other)),
        }
    }

    /// Duplicate named view name is a parse error.
    #[test]
    fn duplicate_named_view_errors() {
        let result = parse_ui(
            r#"
widget Clock {
    view { col { text { "Main" } } }
    view mini { col { text { "A" } } }
    view mini { col { text { "B" } } }
}
"#,
        );
        assert!(result.is_err(), "duplicate `view mini` must be an error");
    }

    /// Duplicate main `view` is a parse error — was a SILENT OVERRIDE before
    /// PLAN-024 (regression fix pinned here).
    #[test]
    fn duplicate_main_view_errors() {
        let result = parse_ui(
            r#"
widget Clock {
    view { col { text { "A" } } }
    view { col { text { "B" } } }
}
"#,
        );
        assert!(result.is_err(), "duplicate `view` block must be an error");
    }

    /// Top-level `view fn` fragment dispatch is unaffected (peek fn wins over
    /// named-view parsing).
    #[test]
    fn view_fn_top_level_still_works() {
        let code = parse_ui(
            r#"
view fn Row(active: bool) {
    button {
        style: if active { "on" } else { "off" }
    }
}
"#,
        );
        assert!(code.is_ok(), "top-level `view fn` must keep parsing");
    }

    /// `view` as a parameter-mode keyword in `fn` signatures is unaffected —
    /// the named-view dispatch only lives at widget-body statement position.
    #[test]
    fn fn_view_param_mode_still_works() {
        let code = parse_ui("fn foo(view x int) int { return x }");
        assert!(code.is_ok(), "`fn f(view x int)` must keep parsing");
    }

    /// A widget without any view still parses (Plan 425 view optionality),
    /// named_views empty.
    #[test]
    fn viewless_widget_keeps_empty_named_views() {
        let code = parse_ui(
            r#"
widget Counter {
    model { count int = 0 }
}
"#,
        )
        .expect("parse");
        let w = widget_decl(&code);
        assert!(w.view.is_none());
        assert!(w.named_views.is_empty());
    }

    /// Regression (PLAN-024 work T-01): the main-view dispatch must CONSUME
    /// the opening `{` before parse_view_root_nodes. When it leaked, `{`
    /// parsed as a bogus element child and the real root got wrapped in an
    /// implicit col — element events landed one level deep and extract lost
    /// them (caught by aura::extract event-modifier corpus during full `cargo t`).
    #[test]
    fn main_view_root_consumes_brace_no_implicit_wrap() {
        let code = parse_ui(
            r#"
widget Nav {
    view {
        col {
            onclick.self: .X
        }
    }
    on { .X -> {} }
}
"#,
        )
        .expect("parse");
        let w = widget_decl(&code);
        match &w.view.as_ref().expect("main view").root {
            ViewNode::Element { tag, events, children, .. } => {
                assert_eq!(tag, "col", "single root must not be wrapped");
                assert!(
                    events.iter().any(|e| e.name == "onclick.self"),
                    "events must sit on the root element, children: {:?}",
                    children.len()
                );
            }
            other => panic!("expected element root, got {:?}", other),
        }
    }
}

/// PLAN-024 vue 生成：named_view_codes 产物（Mini.vue 源）——generate
/// 管线克隆换根复用（script/store 段共享），无 mini 源恒空。
#[test]
fn vue_named_view_codes_generated() {
    let dir = std::env::temp_dir().join("plan024_vue_mini");
    let _ = std::fs::create_dir_all(&dir);
    let path = dir.join("app.at");
    std::fs::write(
        &path,
        r#"
widget App {
    model { n int = 0 }
    view { col { text { "Main" } } }
    view mini { col { text { "12:00" } } }
}
"#,
    )
    .unwrap();
    let result = crate::ui_gen::generate_component_from_file(
        &path,
        crate::ui_gen::ComponentGenOptions::default(),
    )
    .expect("generate");
    assert_eq!(result.named_view_codes.len(), 1);
    let (wname, vname, code) = &result.named_view_codes[0];
    assert_eq!(wname, "App");
    assert_eq!(vname, "mini");
    assert!(code.contains("12:00"), "mini template in SFC");
    assert!(code.contains("<template>"), "valid SFC shape");
    let _ = std::fs::remove_file(&path);
}

/// PLAN-024 vue 生成：无 named view 的源产物恒空（零回归面）。
#[test]
fn vue_named_view_codes_empty_without_mini() {
    let dir = std::env::temp_dir().join("plan024_vue_nomini");
    let _ = std::fs::create_dir_all(&dir);
    let path = dir.join("app.at");
    std::fs::write(
        &path,
        r#"
widget App {
    model { n int = 0 }
    view { col { text { "Main" } } }
}
"#,
    )
    .unwrap();
    let result = crate::ui_gen::generate_component_from_file(
        &path,
        crate::ui_gen::ComponentGenOptions::default(),
    )
    .expect("generate");
    assert!(result.named_view_codes.is_empty());
    let _ = std::fs::remove_file(&path);
}

/// PLAN-024 走查诊断：面板 chrome 样式串逐类解析——w-[920px] 任意值类
/// 必须落 Width(920)（渲染实机面板曾收缩为内容宽，定位用）。
/// fix-tabs-merged-look：测试体消费 `crate::ui`，须挂 ui feature 门——
/// 否则 `cargo tv/tf`（test-vm-files 无 ui）编译失败（024 落地引入的
/// 基线破损，此处机械修复解锁门禁）。
#[cfg(feature = "ui")]
#[test]
fn dashboard_panel_classes_parse() {
    use crate::ui::style::Style;
    let s = Style::parse(
        "bg-card/80 border rounded-xl shadow-xl overflow-hidden w-[920px] h-[212px]",
    )
    .expect("panel style parses");
    let dbg = format!("{s:?}");
    assert!(dbg.contains("920"), "width arbitrary class applied: {dbg}");
}

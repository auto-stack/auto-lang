//! Tests for `view fn` fragment parsing (Plan 367 P2-3).
//!
//! Root cause of sidebar.at failing to parse: `view fn` parameters were
//! collected but NOT registered in the inference context (infer_ctx). When
//! the fragment body used a parameter as a bare `if` condition
//! (`style: if active { ... }`), `check_symbol` flagged the parameter as an
//! "undefined variable", which surfaced (after error recovery) as
//! "Expected term, got RBrace". This made sidebar.at fail entirely, so
//! NavTree was never registered → rendered as FALLBACK (empty note list) in
//! VM mode.
//!
//! Fix: register each `view fn` parameter in infer_ctx (bind_var) so the
//! symbol checker recognizes parameters used in the body.

#[cfg(test)]
mod plan367_viewfn_tests {
    use crate::session::CompilerSession;

    fn parse_ok(code: &str) -> bool {
        let session = CompilerSession::ui();
        let mut parser = crate::Parser::from(code).with_session(session);
        match parser.parse() {
            Ok(_) => true,
            Err(e) => {
                eprintln!("viewfn: PARSE FAIL: {:?}", e);
                false
            }
        }
    }

    /// view fn with a parameter used as a bare `if` condition in a style value.
    /// This is the NoteRow shape from sidebar.at — was failing before the fix.
    #[test]
    fn view_fn_param_as_if_condition() {
        assert!(parse_ok(
            r#"
view fn Row(active: bool) {
    button {
        style: if active { "on" } else { "off" }
    }
}
"#
        ));
    }

    /// view fn with multiple typed params + the if-condition shape.
    #[test]
    fn view_fn_multi_param_if_condition() {
        assert!(parse_ok(
            r#"
view fn Row(note: Note, active: bool, indent: str) {
    button {
        style: if active { "on" } else { "off" }
    }
}
"#
        ));
    }

    /// The verbatim NoteRow fragment from sidebar.at.
    #[test]
    fn view_fn_verbatim_noterow() {
        assert!(parse_ok(
            r#"
view fn NoteRow(note: Note, active: bool, indent: str) {
    button {
        text note.title { style: "block truncate" }
        text note.time { style: "block text-xs text-muted-foreground mt-0.5" }
        style: if active {
            "w-full text-left py-2 rounded-lg bg-accent text-accent-foreground"
        } else {
            "w-full text-left py-2 rounded-lg text-foreground hover:bg-accent/50 transition-colors"
        }
    }
}
"#
        ));
    }

    /// view fn with no params + an if still works (regression guard).
    #[test]
    fn view_fn_no_params_if() {
        assert!(parse_ok(r#"view fn R() { button { style: if true { "on" } else { "off" } } }"#));
    }

    /// Two consecutive view fns + a widget (the real sidebar.at shape).
    #[test]
    fn view_fns_then_widget() {
        assert!(parse_ok(
            r#"
view fn A(x: str) { text "a" }
view fn B(y: str) { text "b" }
widget W(a: str) {
    msg Msg { Go }
    view { col { text "w" } }
    on { .Go -> {} }
}
"#
        ));
    }

    /// The REAL sidebar.at parses in full, and declares NavTree.
    #[test]
    fn real_sidebar_at_parses_with_navtree() {
        let path = [
            std::env::var("CARGO_MANIFEST_DIR")
                .ok()
                .map(|d| std::path::PathBuf::from(d)
                    .join("../../examples/ui/015-notes/src/front/sidebar.at")),
            Some(std::path::PathBuf::from(
                "examples/ui/015-notes/src/front/sidebar.at",
            )),
        ]
        .into_iter()
        .flatten()
        .find(|p| p.exists());
        let Some(path) = path else {
            eprintln!("viewfn: SKIPPED — sidebar.at not found");
            return;
        };
        let code = std::fs::read_to_string(&path).unwrap();
        let session = CompilerSession::ui();
        let mut parser = crate::Parser::from(code.as_str()).with_session(session);
        let ast = parser.parse().expect("sidebar.at must parse cleanly");
        let has_navtree = ast.stmts.iter().any(|s| {
            matches!(s, crate::ast::Stmt::WidgetDecl(d) if d.name.to_string() == "NavTree")
        });
        assert!(has_navtree, "sidebar.at must declare the NavTree widget");
    }
}

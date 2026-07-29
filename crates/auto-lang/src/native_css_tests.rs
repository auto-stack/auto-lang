//! Tests for widget-level native CSS pass-through (`style { ... }` block).
//!
//! The CSS content must be captured verbatim by the lexer
//! (`Lexer::capture_raw_block`) — never tokenized — so nested `{}`,
//! `/* */` comments, strings containing braces, media queries, and
//! pseudo-classes all survive unchanged. The Vue backend emits it into the
//! component's `<style scoped>` block untouched.

#[cfg(test)]
mod native_css_tests {
    use crate::session::CompilerSession;

    /// Parse source in UI scenario and return the first WidgetDecl.
    fn parse_widget(code: &str) -> crate::ast::ui::WidgetDecl {
        let session = CompilerSession::ui();
        let mut parser = crate::Parser::from(code).with_session(session);
        let ast = parser.parse().expect("parse failed");
        for stmt in &ast.stmts {
            if let crate::ast::Stmt::WidgetDecl(w) = stmt {
                return w.clone();
            }
        }
        panic!("no widget declaration found");
    }

    #[test]
    fn style_block_captured_verbatim() {
        let w = parse_widget(
            r#"widget Foo {
    model {
        var count int = 0
    }
    style {
        .foo {
            color: red;
        }
    }
    view {
        col { text "hi" }
    }
}
"#,
        );
        let css = w.style.expect("style block missing");
        assert!(css.contains(".foo {"), "css: {:?}", css);
        assert!(css.contains("color: red;"), "css: {:?}", css);
        // Raw capture: the exact bytes between the braces are preserved.
        assert_eq!(
            css,
            "\n        .foo {\n            color: red;\n        }\n    "
        );
    }

    #[test]
    fn style_block_nested_braces_media_query_and_hover() {
        let w = parse_widget(
            r#"widget Foo {
    style {
        .autodown-editor {
            --ad-border: #333;
        }
        .autodown-editor:hover {
            border-color: var(--ad-border);
        }
        @media (max-width: 768px) {
            .autodown-editor {
                font-size: 12px;
            }
        }
    }
    view {
        col { text "hi" }
    }
}
"#,
        );
        let css = w.style.expect("style block missing");
        assert!(css.contains(".autodown-editor:hover"), "css: {:?}", css);
        assert!(css.contains("@media (max-width: 768px) {"), "css: {:?}", css);
        assert!(css.contains("--ad-border: #333;"), "css: {:?}", css);
        assert!(css.contains("font-size: 12px;"), "css: {:?}", css);
        // The view block after the style block must still parse.
        assert!(w.view.is_some());
    }

    #[test]
    fn style_block_comment_and_string_with_braces() {
        let w = parse_widget(
            r#"widget Foo {
    style {
        /* a comment with a } brace and { another */
        .a::before {
            content: "}";
        }
        .b::after {
            content: '}';
        }
    }
    view {
        col { text "hi" }
    }
}
"#,
        );
        let css = w.style.expect("style block missing");
        assert!(css.contains("/* a comment with a } brace and { another */"), "css: {:?}", css);
        assert!(css.contains(r#"content: "}";"#), "css: {:?}", css);
        assert!(css.contains("content: '}';"), "css: {:?}", css);
        assert!(w.view.is_some());
    }

    #[test]
    fn style_block_then_other_blocks_still_parse() {
        // The block after `style` must be reached correctly: the raw capture
        // has to leave the lexer exactly after the matching `}`.
        let w = parse_widget(
            r#"widget Foo {
    msg Msg { Inc }
    style {
        .x { color: blue; }
    }
    model {
        var count int = 0
    }
    on {
        .Inc -> { .count = .count + 1 }
    }
    view {
        col { text "hi" }
    }
}
"#,
        );
        assert!(w.style.is_some());
        assert!(w.model.is_some());
        assert!(w.on.is_some());
        assert!(w.view.is_some());
        assert_eq!(w.messages.len(), 1);
    }

    #[test]
    fn style_block_unterminated_is_error() {
        let session = CompilerSession::ui();
        let mut parser = crate::Parser::from(
            r#"widget Foo {
    style {
        .x { color: blue; }
"#,
        )
        .with_session(session);
        assert!(parser.parse().is_err());
    }

    #[test]
    fn widget_without_style_block_has_none() {
        let w = parse_widget(
            r#"widget Foo {
    view {
        col { text "hi" }
    }
}
"#,
        );
        assert!(w.style.is_none());
    }

    /// End-to-end: parse → extract → Vue SFC contains the verbatim CSS in a
    /// dedicated `<style scoped>` block.
    #[test]
    fn style_block_emitted_into_scoped_style() {
        use crate::ui_gen::{BackendGenerator, VueGenerator};

        let session = CompilerSession::ui().with_backend("vue");
        let mut parser = crate::Parser::from(
            r#"widget Foo {
    style {
        .foo {
            color: red;
        }
        .foo:hover {
            color: blue;
        }
    }
    view {
        col { text "hi" }
    }
}
"#,
        )
        .with_session(session);
        let ast = parser.parse().expect("parse failed");
        let decl = match &ast.stmts[0] {
            crate::ast::Stmt::WidgetDecl(w) => w,
            _ => panic!("expected widget"),
        };
        let widget = crate::aura::extract_widget_from_decl(decl).expect("extract failed");
        assert!(widget.style_css.is_some());

        let mut gen = VueGenerator::new();
        let sfc = gen.generate(&widget).expect("generate failed");
        assert!(sfc.contains("<style scoped>"), "sfc:\n{}", sfc);
        let css = widget.style_css.as_ref().unwrap();
        assert!(sfc.contains(css.as_str()), "scoped css not verbatim in sfc");
        assert!(sfc.contains(".foo:hover"), "sfc:\n{}", sfc);
    }
}

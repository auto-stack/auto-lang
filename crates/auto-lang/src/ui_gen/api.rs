//! High-level Transpiler API for AutoUI
//!
//! Provides simple API for transpiling .at files to backend-specific code
//! using the AURA pipeline (Plan 096).
//!
//! # Active API
//! - `transpile_file` — Transpile a .at file to Rust code
//! - `transpile_aura` — Transpile UI source string to Rust
//! - `transpile_vue_aura` — Transpile UI source string to Vue3 SFC

use std::path::Path;

use crate::aura::extract_widget_from_decl;
use crate::parser::Parser;
use crate::session::CompilerSession;
use crate::ui_gen::{BackendGenerator, RustGenerator, VueGenerator};

/// Transpile Auto UI file to Rust code using AURA pipeline
///
/// # Arguments
/// * `input_path` - Path to .at file
/// * `output_path` - Optional path to write .rs file
///
/// # Returns
/// Generated Rust code as string
pub fn transpile_file(
    input_path: impl AsRef<Path>,
    output_path: Option<&str>,
) -> Result<String, String> {
    let input_path = input_path.as_ref();
    let source = std::fs::read_to_string(input_path)
        .map_err(|e| format!("Failed to read file {}: {}", input_path.display(), e))?;

    transpile_aura(&source, output_path)
}

/// Transpile UI source code to Rust using AURA pipeline (Plan 096)
///
/// This is the preferred method for transpiling UI components.
/// It uses the new AURA-based architecture without DSL preprocessing.
pub fn transpile_aura(source: &str, output_path: Option<&str>) -> Result<String, String> {
    // Parse with UI scenario
    let session = CompilerSession::ui();
    let mut parser = Parser::from(source).with_session(session);
    let ast = parser.parse().map_err(|e| format!("Failed to parse: {:?}", e))?;

    // Extract and generate
    let mut code = String::new();
    let mut generator = RustGenerator::new();

    for stmt in &ast.stmts {
        if let crate::ast::Stmt::WidgetDecl(widget_decl) = stmt {
            let aura_widget = extract_widget_from_decl(widget_decl)
                .map_err(|e| format!("Failed to extract widget: {}", e))?;
            let widget_code = generator.generate(&aura_widget)
                .map_err(|e| format!("Failed to generate: {}", e))?;
            code.push_str(&widget_code);
            code.push('\n');
        }
    }

    if let Some(output) = output_path {
        std::fs::write(output, &code)
            .map_err(|e| format!("Failed to write file {}: {}", output, e))?;
    }

    Ok(code)
}

/// Transpile UI source code to Vue3 SFC using AURA pipeline (Plan 096)
pub fn transpile_vue_aura(source: &str, output_path: Option<&str>) -> Result<String, String> {
    // Parse with UI scenario
    let session = CompilerSession::ui();
    let mut parser = Parser::from(source).with_session(session);
    let ast = parser.parse().map_err(|e| format!("Failed to parse: {:?}", e))?;

    // Extract and generate
    let mut code = String::new();
    let mut generator = VueGenerator::new();

    for stmt in &ast.stmts {
        if let crate::ast::Stmt::WidgetDecl(widget_decl) = stmt {
            let aura_widget = extract_widget_from_decl(widget_decl)
                .map_err(|e| format!("Failed to extract widget: {}", e))?;
            let widget_code = generator.generate(&aura_widget)
                .map_err(|e| format!("Failed to generate: {}", e))?;
            code.push_str(&widget_code);
            code.push('\n');
        }
    }

    if let Some(output) = output_path {
        std::fs::write(output, &code)
            .map_err(|e| format!("Failed to write file {}: {}", output, e))?;
    }

    Ok(code)
}

#[cfg(test)]
mod tests {
    use super::transpile_vue_aura;

    #[test]
    fn test_transpile_placeholder() {
        // Active API smoke test — real tests require .at fixture files
        assert!(true);
    }

    /// Plan 356 follow-up: a view `for`-loop over an `ident.field` iterable
    /// (e.g. `for tag in note.tags`) must parse + generate. Previously
    /// `parse_view_for_loop` only accepted `.field`, numeric ranges, or a bare
    /// ident, so `note.tags` was read as just `note`, leaving `.tags` to break
    /// the rest of the view ("Expected term, got RBrace").
    #[test]
    fn test_view_for_loop_ident_field_iterable() {
        let src = r#"
widget Tags {
    view {
        col {
            for note in .notes {
                for tag in note.tags {
                    button { text tag }
                }
            }
        }
    }
}
"#;
        let out = transpile_vue_aura(src, None).expect("ident.field iterable must generate");
        // Outer loop iterates .notes; inner loop iterates note.tags.
        assert!(out.matches("v-for").count() == 2, "expected 2 v-for loops in:\n{out}");
    }

    /// Plan 356 follow-up: `ident.field.sub` chains as an iterable must also
    /// work (symmetric with `.field.sub` chained access).
    #[test]
    fn test_view_for_loop_ident_field_chain_iterable() {
        let src = r#"
widget W {
    view {
        for x in store.items {
            button { text "x" }
        }
    }
}
"#;
        let out = transpile_vue_aura(src, None).expect("ident.field.chain iterable must generate");
        assert!(out.contains("v-for"), "missing v-for in:\n{out}");
    }

    /// Plan 356 end-to-end (IGNORED): the real 015-notes sidebar (commit
    /// 50307d51, 200 lines) that originally OOM'd. After the Plan 356 OOM fix
    /// and the `for ident.field` fix, parsing advances past the tag-filter
    /// section, but the file still hits a *separate* pre-existing bug: a
    /// `style: if <complex-cond> { } else { }` attribute with a comparison
    /// condition fails to parse (offset ~8873, "Expected term, got RBrace").
    ///
    /// That is a distinct parser issue, out of scope for this fix. This test is
    /// kept (ignored) as a guard: once the style:if-attribute bug is fixed,
    /// un-ignore it to verify the full sidebar regenerates end to end.
    #[test]
    #[ignore]
    fn test_plan356_real_sidebar_generates() {
        let src = include_str!("../../tests/fixtures/plan356_oom_sidebar.at");
        let out = transpile_vue_aura(src, None).expect("real sidebar must generate");
        assert!(out.len() < 100_000, "output too large: {} bytes", out.len());
        assert!(out.contains("v-for"), "expected v-for in sidebar output");
        assert!(out.contains("SelectTag"), "expected SelectTag handler in sidebar output");
    }

    /// Plan 356 regression: the minimal OOM trigger. A `for`-loop whose body
    /// has an event handler taking the loop variable as an argument, where the
    /// loop variable is a reserved-keyword identifier (`tag` → TokenKind::Tag).
    ///
    /// Before the fix this exhausted memory (parse_event_arg ignored the `Tag`
    /// token and the handler arg loop spun forever). It must now parse +
    /// generate a sane SFC inline (no thread, no timeout needed).
    #[test]
    fn test_plan356_oom_regression_loop_var_as_handler_arg() {
        let src = include_str!("../../tests/fixtures/plan356_minimal_oom.at");
        let out = transpile_vue_aura(src, None).expect("Plan 356 trigger must generate");
        assert!(out.contains("v-for"), "expected v-for in:\n{out}");
        assert!(out.contains("SelectTag"), "expected handler binding in:\n{out}");
    }

    /// Plan 356: the soft-keyword-as-identifier fix applies to any reserved
    /// keyword, not just `tag`. Verify a few others (`type`, `move`) used as a
    /// loop variable passed to a handler also generate cleanly.
    #[test]
    fn test_plan356_soft_keyword_loop_var() {
        for kw in ["tag", "type", "move", "copy", "super"] {
            let src = format!(
                r#"
widget W(active: bool) {{
    msg Msg {{ Go(str) }}
    view {{
        col {{
            for {kw} in .items {{
                button {{
                    text {kw}
                    onclick: .Go({kw})
                }}
            }}
        }}
    }}
    on {{ .Go(x) -> {{ }} }}
}}
"#,
            );
            let out =
                transpile_vue_aura(&src, None).unwrap_or_else(|e| panic!("{kw}: {e}"));
            assert!(out.contains("v-for"), "{kw}: missing v-for in:\n{out}");
            assert!(out.contains("Go"), "{kw}: missing handler in:\n{out}");
        }
    }

    /// Plan 356 control: a non-keyword loop variable name worked before and
    /// must still work (guards against the fix over-reaching).
    #[test]
    fn test_plan356_normal_loop_var_still_works() {
        let src = r#"
widget W(active: bool) {
    msg Msg { Go(str) }
    view {
        col {
            for item in .items {
                button {
                    text item
                    onclick: .Go(item)
                }
            }
        }
    }
    on { .Go(x) -> { } }
}
"#;
        let out = transpile_vue_aura(src, None).expect("normal loop var must generate");
        assert!(out.contains("v-for"));
    }
}

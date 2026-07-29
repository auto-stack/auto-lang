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

    /// DOM escape hatch, end to end (plain Tailwind mode):
    /// `ref: "menuEl"` in the view → `ref="menuEl"` template attribute +
    /// `const menuEl = ref<HTMLElement | null>(null)` script declaration;
    /// `.menuEl.xxx` in `on` handlers → `menuEl.value!.xxx`;
    /// `document.*` / `window.*` pass through unchanged.
    #[test]
    fn test_dom_escape_hatch_plain_mode() {
        let src = r#"
widget App {
    msg Msg {
        Open
        Scrolled
    }
    model {
        var menu_left int = 0
    }
    view {
        col {
            button "open" {
                ref: "triggerEl",
                onclick: .Open
            }
            col {
                ref: "menuEl"
                text "menu"
            }
            col {
                ref: "scrollEl",
                onwheel: .Scrolled($event)
                text "content"
            }
        }
    }
    on {
        .Open -> {
            let r = .triggerEl.getBoundingClientRect()
            .menu_left = r.left
            let w = window.innerWidth
            let h = window.innerHeight
            let ae = document.activeElement
        }

        .Scrolled(e) -> {
            .scrollEl.scrollTop = .scrollEl.scrollTop + e.deltaY
            let sh = .scrollEl.scrollHeight
            let ch = .scrollEl.clientHeight
            let q = .menuEl.querySelector(".item")
        }
    }
}
"#;
        let out = transpile_vue_aura(src, None).expect("dom escape hatch must generate");
        // Template ref attributes
        assert!(out.contains("ref=\"triggerEl\""), "sfc:\n{out}");
        assert!(out.contains("ref=\"menuEl\""), "sfc:\n{out}");
        assert!(out.contains("ref=\"scrollEl\""), "sfc:\n{out}");
        // Script declarations
        assert!(out.contains("const triggerEl = ref<HTMLElement | null>(null)"), "sfc:\n{out}");
        assert!(out.contains("const menuEl = ref<HTMLElement | null>(null)"), "sfc:\n{out}");
        assert!(out.contains("const scrollEl = ref<HTMLElement | null>(null)"), "sfc:\n{out}");
        // Handler body: ref access maps onto the DOM element
        assert!(out.contains("triggerEl.value!.getBoundingClientRect()"), "sfc:\n{out}");
        assert!(out.contains("scrollEl.value!.scrollTop = scrollEl.value!.scrollTop + e.deltaY"), "sfc:\n{out}");
        assert!(out.contains("scrollEl.value!.scrollHeight"), "sfc:\n{out}");
        assert!(out.contains("scrollEl.value!.clientHeight"), "sfc:\n{out}");
        assert!(out.contains("menuEl.value!.querySelector('.item')"), "sfc:\n{out}");
        // document/window pass-through
        assert!(out.contains("window.innerWidth"), "sfc:\n{out}");
        assert!(out.contains("window.innerHeight"), "sfc:\n{out}");
        assert!(out.contains("document.activeElement"), "sfc:\n{out}");
    }

    /// Same escape hatch in shadcn mode: layout elements (col) and buttons
    /// are mapped through the shadcn branch, which whitelists props — the
    /// `ref` prop must survive there too.
    #[test]
    fn test_dom_escape_hatch_shadcn_mode() {
        let src = r#"
widget App {
    msg Msg {
        Open
    }
    model {
        var menu_left int = 0
    }
    view {
        col {
            button "open" {
                ref: "triggerEl",
                onclick: .Open
            }
            col {
                ref: "menuEl"
                text "menu"
            }
        }
    }
    on {
        .Open -> {
            let r = .triggerEl.getBoundingClientRect()
            .menu_left = r.left
        }
    }
}
"#;
        use crate::session::CompilerSession;
        use crate::ui_gen::{BackendGenerator, VueGenerator, VueMode};
        let session = CompilerSession::ui().with_backend("vue");
        let mut parser = crate::Parser::from(src).with_session(session);
        let ast = parser.parse().expect("parse");
        let mut generated = None;
        for stmt in &ast.stmts {
            if let crate::ast::Stmt::WidgetDecl(w) = stmt {
                let widget = crate::aura::extract_widget_from_decl(w).expect("extract");
                let mut gen = VueGenerator::new().with_mode(VueMode::Shadcn);
                generated = Some(gen.generate(&widget).expect("generate"));
            }
        }
        let out = generated.expect("widget");
        assert!(out.contains("ref=\"triggerEl\""), "sfc:\n{out}");
        assert!(out.contains("ref=\"menuEl\""), "sfc:\n{out}");
        assert!(out.contains("const triggerEl = ref<HTMLElement | null>(null)"), "sfc:\n{out}");
        assert!(out.contains("const menuEl = ref<HTMLElement | null>(null)"), "sfc:\n{out}");
        assert!(out.contains("triggerEl.value!.getBoundingClientRect()"), "sfc:\n{out}");
    }

    /// Plan 356 follow-up #2: a reserved-keyword identifier (e.g. `tag`,
    /// lexed as TokenKind::Tag) used inside an `if` condition — specifically
    /// as the right-hand side of a comparison in a `style: if` attribute:
    ///   style: if .active_tag == tag { ... } else { ... }
    ///
    /// Previously the expression atom parser matched only TokenKind::Ident,
    /// so `tag` here produced "Expected term, got RBrace" (the real 015-notes
    /// sidebar's offset-8873 error). Must now parse + generate.
    #[test]
    fn test_soft_keyword_in_if_condition() {
        let src = r#"
widget W(active_tag: str) {
    view {
        button {
            style: if .active_tag == tag {
                "bg-blue-500"
            } else {
                "bg-gray-100"
            }
        }
    }
}
"#;
        let out = transpile_vue_aura(src, None).expect("soft-kw in if-condition must generate");
        assert!(out.contains(":class"), "expected :class binding in:\n{out}");
    }

    /// Plan 356 follow-up #2: a reserved-keyword identifier as a comparison
    /// operand in an `if` condition (the shape the real sidebar uses:
    /// `if .active_tag == tag`). Covers several soft keywords.
    ///
    /// NOTE: a bare soft keyword as the *whole* condition (`if tag {}`) is a
    /// separate, narrower pre-existing issue with `style:`-attribute parsing
    /// and is intentionally not covered here — the sidebar doesn't use it.
    #[test]
    fn test_soft_keyword_as_condition_value() {
        for kw in ["tag", "type", "move", "copy", "super"] {
            let src = format!(
                "widget W(x: str) {{ view {{ button {{ style: if .x == {kw} {{ \"a\" }} else {{ \"b\" }} }} }} }}",
            );
            transpile_vue_aura(&src, None)
                .unwrap_or_else(|e| panic!("compare-operand {kw}: {e}"));
        }
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

    /// Plan 356 end-to-end: the real 015-notes sidebar (commit 50307d51, 200
    /// lines) that originally OOM'd. This is the full integration guard — it
    /// exercises every trigger fixed across the three Plan 356 commits:
    ///   - `onclick: .SelectTag(tag)`        (reserved-kw loop var → OOM)
    ///   - `for tag in note.tags`            (ident.field iterable)
    ///   - `style: if .active_tag == tag {}` (reserved-kw in if-condition)
    /// It must parse AND generate end to end.
    #[test]
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

    /// Plan 358 D1 stress guard: the `for` + `style:if` + `msg`/`on`
    /// combination that OOM'd (1.7GB+) on the real 015-notes sidebar. The
    /// root cause was a `format!("{:?}", widget.view_tree)` dark-mode scan in
    /// `generate_sfc` whose Debug output exploded on large trees with
    /// `Expr::If` nodes (fixed in a86c183c by removing the Debug format).
    ///
    /// This test generates a widget with 10 for-loops x 60 `style:if`
    /// branches (600 if-nodes) plus `msg`/`on`, and asserts generation
    /// completes well under the Plan 358 budget (< 5s; measured ~0.1s) so a
    /// reintroduced tree-wide Debug format — or any other superlinear blowup
    /// on this pattern — fails loudly instead of OOMing a build.
    #[test]
    fn test_plan358_d1_for_style_if_msg_on_stress() {
        let mut src = String::from(
            "widget NavTree(active: bool) {\n    msg Msg { SelectTag(str) }\n    view {\n        col {\n",
        );
        for l in 0..10 {
            src.push_str(&format!(
                "            for tag{l} in .items {{\n                button {{\n                    text tag{l}\n                    onclick: .SelectTag(tag{l})\n"
            ));
            for i in 0..60 {
                src.push_str(&format!(
                    "                    style: if tag{l} == \"t{i}\" {{ \"l{l}c{i}a\" }} else if .active {{ \"l{l}c{i}b\" }} else {{ \"l{l}c{i}c\" }}\n"
                ));
            }
            src.push_str("                }\n            }\n");
        }
        src.push_str("        }\n    }\n    on { .SelectTag(t) -> { } }\n}\n");

        let start = std::time::Instant::now();
        let out = transpile_vue_aura(&src, None).expect("D1 trigger pattern must generate");
        let elapsed = start.elapsed();
        assert!(
            elapsed.as_secs() < 5,
            "generation took {:?}, over the 5s Plan 358 budget (possible D1 regression)",
            elapsed
        );
        assert_eq!(out.matches("v-for").count(), 10, "expected 10 v-for loops");
        assert!(out.contains("SelectTag"), "expected handler binding");
    }
}

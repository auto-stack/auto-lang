//! Snapshot tests for .at → AuraWidget extraction (Plan 362 Phase 3).
//!
//! These tests capture the extracted AuraWidget structures from key 015-notes
//! .at files. Unlike raw SFC output (which has HashMap non-determinism),
//! AuraWidget extraction is deterministic (parser → AST → extract).
//!
//! When the generator or extractor changes, `cargo test` will show which
//! snapshots changed. Review diffs with `cargo insta review`.
//!
//! Run: `cargo test -p auto-lang -- ui_snapshots`
//! Accept changes: `cargo insta review` (or `INSTA_UPDATE=always`)

use auto_lang::ui_gen::{generate_component_from_file, ComponentGenOptions};
use std::path::Path;

/// Helper: resolve a path relative to the workspace root.
fn workspace_path(rel: &str) -> String {
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .join("..")
        .join("..")
        .join(rel)
        .to_string_lossy()
        .to_string()
}

/// Extract widgets from an .at file and return a deterministic debug representation.
/// Uses AuraWidget debug output which is stable across runs.
fn snapshot_widgets(path: &str, snapshot_name: &str) {
    let opts = ComponentGenOptions::default();
    let result = generate_component_from_file(Path::new(path), opts)
        .expect("generate_component_from_file should succeed");

    // Build a stable representation: widget names + counts + validation warnings
    let mut out = String::new();
    out.push_str(&format!("File: {}\n", path));
    out.push_str(&format!("Widget count: {}\n", result.widgets.len()));
    for w in &result.widgets {
        out.push_str(&format!(
            "Widget: {} | props: {} | state: {} | handlers: {} | messages: {}\n",
            w.name,
            w.props.len(),
            w.state_vars.len(),
            w.handlers.len(),
            w.messages.len(),
        ));
        for prop in &w.props {
            out.push_str(&format!("  prop: {} ({:?})\n", prop.name, prop.type_info));
        }
        for state in &w.state_vars {
            out.push_str(&format!("  state: {} ({:?})\n", state.name, state.type_info));
        }
        let mut handler_names: Vec<&str> = w.handlers.keys().map(|s| s.as_str()).collect();
        handler_names.sort();
        for h in &handler_names {
            out.push_str(&format!("  handler: {}\n", h));
        }
    }
    out.push_str(&format!("API imports: {:?}\n", result.detected_api_imports));
    out.push_str(&format!("Store deps: {:?}\n", result.detected_store_deps));
    out.push_str(&format!("Store composables: {}\n", result.store_composables.len()));
    out.push_str(&format!("Validation warnings: {}\n", result.validation_warnings.len()));

    // SFC output validation: does it compile to non-empty?
    for (name, code) in &result.all_widget_codes {
        out.push_str(&format!(
            "SFC '{}': {} bytes, has template: {}, has script: {}, has style: {}\n",
            name,
            code.len(),
            code.contains("<template>"),
            code.contains("<script"),
            code.contains("<style>"),
        ));
    }

    insta::assert_snapshot!(snapshot_name, out);
}

// ============================================================================
// 015-notes Widget Snapshots
// ============================================================================

#[test]
fn snapshot_sidebar() {
    snapshot_widgets(
        &workspace_path("examples/ui/015-notes/src/front/sidebar.at"),
        "sidebar",
    );
}

#[test]
fn snapshot_editor() {
    snapshot_widgets(
        &workspace_path("examples/ui/015-notes/src/front/editor.at"),
        "editor",
    );
}

#[test]
fn snapshot_app() {
    snapshot_widgets(
        &workspace_path("examples/ui/015-notes/src/front/app.at"),
        "app",
    );
}

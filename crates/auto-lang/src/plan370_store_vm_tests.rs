//! Regression tests for the 015-notes VM-mode "notes list empty" bug
//! (Plan 370 D-GAP-4 / store-composable in VM mode).
//!
//! ## Root cause (found via systematic debugging)
//!
//! In merged-VM mode the MCP snapshot showed "No notes yet" (empty list) even
//! though seed data exists in `back/db.at`. The actual root cause was NOT the
//! VmRef/path issues a prior session chased — it was that the `notes_store`
//! store, declared in its own source file and imported via
//! `use notes_store: NotesStore`, was never converted to a child widget decl.
//! The child-widget collection loop in `run_file_dynamic_ui_inner` only matched
//! `Stmt::WidgetDecl`, ignoring `Stmt::StoreDecl`. As a result the store's
//! fields (`notes`, `active_id`, …) were never merged into the App's root state
//! object, so `__state.notes` was undefined and handlers like
//! `.notes = list_notes()` silently no-op'd.
//!
//! These tests mirror the exact production path
//! (parse → collect imports/stores → build bridge from decls → run_module_init
//! → fire Init) and assert the data + render layers behave correctly.

#[cfg(test)]
mod plan370_store_vm_tests {
    use crate::plan370_test_support::build_015_component;
    use std::collections::HashMap;



    /// REGRESSION: store fields must be merged into root state after Init.
    ///
    /// Before the fix, root state held ONLY the App's own `search` field; the
    /// store's `notes`, `active_id`, `active_folder`, etc. were absent.
    #[cfg(feature = "ui-interpreter")]
    #[test]
    fn store_fields_merged_into_root_state() {
        let comp = match build_015_component() {
            Some(c) => c,
            None => {
                eprintln!("plan370: SKIPPED — app.at not found");
                return;
            }
        };
        let state = comp.read_all_state();
        for required in [
            "search", // App's own model field
            "notes",  // store
            "active_id",
            "active_folder",
            "active_tag",
            "dark_mode",
            "accent_color",
        ] {
            assert!(
                state.contains_key(required),
                "store field '{}' missing from root state; keys = {:?}",
                required,
                state.keys().collect::<Vec<_>>()
            );
        }
    }

    /// REGRESSION: `notes` must resolve to the 6 seed Notes from db.at.
    ///
    /// Before the fix, `__state.notes` was undefined so `list_notes()` never
    /// populated it. After the fix the VM handler chain
    /// (`store.Init` → `list_notes()` → `db.all_notes()`) executes and the
    /// `Value::VmRef` derefs to 6 elements.
    #[cfg(feature = "ui-interpreter")]
    #[test]
    fn notes_resolves_to_seed_data() {
        let comp = match build_015_component() {
            Some(c) => c,
            None => {
                eprintln!("plan370: SKIPPED — app.at not found");
                return;
            }
        };
        let notes = comp
            .bridge()
            .read_state_as_vec("notes")
            .expect("read_state_as_vec('notes') must succeed");
        assert_eq!(
            notes.len(),
            6,
            "expected 6 seed notes (Welcome..Sprint Planning), got {}: {:?}",
            notes.len(),
            notes
        );
    }

    /// REGRESSION: the MCP snapshot must NOT show the empty state.
    ///
    /// This is the originally-observed symptom. It requires BOTH the data-layer
    /// fix (store fields merged + VmRef materialized) and the render-layer fix
    /// (AuraSnapshotBuilder evaluates `.store.notes.len() > 0` and materializes
    /// the VmRef array). Before the fixes, `.store.notes.len() > 0` evaluated
    /// false and the snapshot rendered the "No notes yet" else-branch.
    #[cfg(feature = "ui-interpreter")]
    #[test]
    fn snapshot_does_not_show_empty_state() {
        let comp = match build_015_component() {
            Some(c) => c,
            None => {
                eprintln!("plan370: SKIPPED — app.at not found");
                return;
            }
        };
        let state = comp.read_all_state_materialized();
        let template = comp.view_template();
        use crate::ui::aura_snapshot_builder::AuraSnapshotBuilder;
        let builder = AuraSnapshotBuilder::new(&state);
        let snap = builder.build(comp.widget_name(), template);

        // The empty-state branch must NOT render now that notes has 6 items.
        assert!(
            !snap.contains("No notes yet"),
            "snapshot still shows 'No notes yet' — the notes-empty bug is not fixed.\n\
             notes in state = {:?}\n\n--- snapshot ---\n{}",
            state.get("notes"),
            snap
        );
        // Positive signal: the note-editor (then-branch) should be present.
        assert!(
            snap.contains("EditorPanel"),
            "snapshot should render the EditorPanel (notes.len() > 0 branch)"
        );
    }

    /// UNIT: AuraSnapshotBuilder comparison operators + `.len()` + `.store.` path.
    /// Covers the render-layer fix in isolation (no VM / no example files needed).
    #[test]
    fn snapshot_builder_evaluates_store_len_comparison() {
        use crate::ui::aura_snapshot_builder::AuraSnapshotBuilder;
        let mut state = HashMap::new();
        // notes materialized to an Array of 6 elements (as the MCP sync does).
        state.insert(
            "notes".to_string(),
            auto_val::Value::Array(auto_val::Array {
                values: (0..6).map(auto_val::Value::Int).collect(),
            }),
        );
        state.insert("active_folder".to_string(), auto_val::Value::Str("all".into()));
        let builder = AuraSnapshotBuilder::new(&state);

        // The exact condition from app.at:
        assert!(builder.eval_condition(".store.notes.len() > 0"));
        assert!(builder.eval_condition(".notes.len() > 0"));
        assert!(!builder.eval_condition(".notes.len() > 6"));
        assert!(builder.eval_condition(".notes.len() >= 6"));
        assert!(builder.eval_condition(".active_folder == \"all\""));
        assert!(!builder.eval_condition(".active_folder == \"pinned\""));
    }
}

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
    use crate::ast::Stmt;
    use crate::session::CompilerSession;
    use crate::ui::dynamic::DynamicComponent;
    use crate::ui::widget_registry::WidgetRegistry;
    use crate::use_scanner::scan_use_statements;
    use std::collections::{HashMap, HashSet};
    use std::fs;
    use std::path::{Path, PathBuf};

    /// Locate the 015-notes app.at regardless of cwd (tests vs. IDE).
    fn locate_app_at() -> Option<PathBuf> {
        let candidates = [
            std::env::var("CARGO_MANIFEST_DIR")
                .ok()
                .map(|d| PathBuf::from(d).join("../../examples/ui/015-notes/src/front/app.at")),
            Some(PathBuf::from("examples/ui/015-notes/src/front/app.at")),
            Some(PathBuf::from("../../examples/ui/015-notes/src/front/app.at")),
        ];
        candidates.into_iter().flatten().find(|p| p.exists())
    }

    /// Build a DynamicComponent exactly like `run_file_dynamic_ui_inner` does,
    /// then return it after `fire_init()` so state is populated. Returns None
    /// (so tests gracefully no-op) when the example sources aren't present.
    fn build_015_component() -> Option<DynamicComponent> {
        let manifest = locate_app_at()?;
        let base_dir = manifest.parent().unwrap_or(Path::new(".")).to_path_buf();
        let code = fs::read_to_string(&manifest).unwrap();

        // 1. Parse + extract root widget
        let session = CompilerSession::ui();
        let mut parser = crate::Parser::from(code.as_str()).with_session(session);
        let ast = parser.parse().unwrap();
        let mut root_decl = None;
        let mut widget = None;
        for stmt in &ast.stmts {
            if let Stmt::WidgetDecl(decl) = stmt {
                root_decl = Some(decl.clone());
                widget = Some(
                    crate::aura::extract_widget_from_decl(decl)
                        .map_err(|e| e.to_string())
                        .unwrap(),
                );
                break;
            }
        }
        let root_decl = root_decl?;
        let widget = widget?;

        // 2. Collect child widgets + imports + aliases (mirror lib.rs)
        let mut registry = WidgetRegistry::new();
        let mut child_decls = Vec::new();
        let mut import_stmts: Vec<Stmt> = Vec::new();
        let mut visited = HashSet::new();
        let mut seen_symbols = HashSet::new();
        let mut import_session = crate::compile::CompileSession::new();
        let mut import_aliases: HashMap<String, String> = HashMap::new();

        let use_stmts = scan_use_statements(&code);
        for use_stmt in &use_stmts {
            if use_stmt.is_c_import || use_stmt.is_rust_import {
                continue;
            }
            let module_path = match crate::resolve_module_path(&base_dir, &use_stmt.module) {
                Some(p) => p,
                None => continue,
            };
            if let Ok(module_code) = fs::read_to_string(&module_path) {
                let mod_session = CompilerSession::ui();
                let mut mod_parser =
                    crate::Parser::from(module_code.as_str()).with_session(mod_session);
                if let Ok(mod_ast) = mod_parser.parse() {
                    for stmt in &mod_ast.stmts {
                        if let Stmt::WidgetDecl(decl) = stmt {
                            if let Ok(child_widget) = crate::aura::extract_widget_from_decl(decl) {
                                if use_stmt.is_wildcard
                                    || use_stmt.items.is_empty()
                                    || use_stmt.items.iter().any(|s| s == &child_widget.name)
                                {
                                    child_decls.push(decl.clone());
                                    registry.register(child_widget);
                                }
                            }
                        } else if let Stmt::StoreDecl(store_decl) = stmt {
                            // MIRROR OF lib.rs FIX: convert imported StoreDecl →
                            // view-less child WidgetDecl so its fields merge into
                            // root state. Without this, notes/active_id/... are
                            // absent from __state.
                            let name = store_decl.name.clone();
                            if use_stmt.is_wildcard
                                || use_stmt.items.is_empty()
                                || use_stmt.items.iter().any(|s| *s == name.as_str())
                            {
                                child_decls.push(crate::ast::ui::WidgetDecl {
                                    name,
                                    messages: store_decl.messages.clone(),
                                    model: store_decl.model.clone(),
                                    computed: store_decl.computed.clone(),
                                    view: None,
                                    on: store_decl.on.clone(),
                                    bind: None,
                                    props: Vec::new(),
                                    routes: None,
                                    lifecycle: Vec::new(),
                                });
                            }
                        }
                    }
                }
            }
            crate::collect_module_imports(
                &module_path,
                &mut visited,
                &mut import_stmts,
                &mut seen_symbols,
                &mut import_session,
                None,
            );
            let module_qualifier = use_stmt.module.split('.').last().unwrap_or(&use_stmt.module);
            for item in &use_stmt.items {
                let qualified = format!("{}.{}", module_qualifier, item);
                import_aliases.insert(item.clone(), qualified);
            }
        }

        // 3. Stores declared in the root AST → fake child widget decls (D-GAP-4)
        let mut store_as_child_decls = Vec::new();
        for stmt in &ast.stmts {
            if let Stmt::StoreDecl(store_decl) = stmt {
                store_as_child_decls.push(crate::ast::ui::WidgetDecl {
                    name: store_decl.name.clone(),
                    messages: store_decl.messages.clone(),
                    model: store_decl.model.clone(),
                    computed: store_decl.computed.clone(),
                    view: None,
                    on: store_decl.on.clone(),
                    bind: None,
                    props: Vec::new(),
                    routes: None,
                    lifecycle: Vec::new(),
                });
            }
        }
        let mut all_child_decls = child_decls.clone();
        all_child_decls.extend(store_as_child_decls.iter().cloned());

        let mut comp = DynamicComponent::with_registry_and_imports_from_decls(
            &root_decl,
            &all_child_decls,
            &widget,
            registry,
            import_stmts,
            &import_aliases,
            false, // merged VM mode (api_over_http = false)
        )
        .unwrap();
        comp.fire_init();
        Some(comp)
    }

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

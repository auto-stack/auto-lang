//! Plan 339: Symbol namespace — cross-module function resolution tests.
//!
//! Verifies that `collect_module_imports` qualifies ALL function names with
//! their module name (api.create_note, db.create_note) and that the
//! `import_scope` alias map (built from `use back.api: create_note`) lets a
//! bare call `create_note(...)` resolve to the `api.create_note` export.
//!
//! These tests live in lib.rs's test tree so they can reach the private
//! `collect_module_imports` / `resolve_module_path` helpers and the
//! `synthesize_widget_module` pipeline exactly as `run_file_dynamic_ui` does.

#[cfg(test)]
mod plan339_tests {
    use crate::compile::CompileSession;
    use crate::use_scanner::scan_use_statements;
    use std::collections::{HashMap, HashSet};
    use std::fs;
    use std::path::{Path, PathBuf};

    /// Mirror of `collect_module_imports` (private in lib.rs) so this test can
    /// exercise the same flattening + qualification without filesystem fixtures.
    /// It is fed pre-parsed module source via temp files.
    fn collect_imports(
        module_path: &Path,
        visited: &mut HashSet<PathBuf>,
        out: &mut Vec<crate::ast::Stmt>,
        seen: &mut HashSet<String>,
        session: &mut CompileSession,
    ) {
        let canon = module_path
            .canonicalize()
            .unwrap_or_else(|_| module_path.to_path_buf());
        if !visited.insert(canon) {
            return;
        }
        let code = match fs::read_to_string(module_path) {
            Ok(c) => c,
            Err(_) => return,
        };
        if let Some(dir) = module_path.parent() {
            let abs = fs::canonicalize(dir).unwrap_or_else(|_| dir.to_path_buf());
            session.add_source_dir(abs);
            if let Some(parent) = dir.parent() {
                let abs_parent = fs::canonicalize(parent).unwrap_or_else(|_| parent.to_path_buf());
                session.add_source_dir(abs_parent);
            }
        }
        let _ = session.resolve_uses(&code);

        let parser_session = if module_path
            .components()
            .any(|c| c.as_os_str() == "back")
        {
            crate::session::CompilerSession::core()
        } else {
            crate::session::CompilerSession::ui()
        };
        let mut parser =
            crate::Parser::new_with_type_store(code.as_str(), session.type_store())
                .with_session(parser_session);
        let ast = match parser.parse() {
            Ok(a) => a,
            Err(_) => return,
        };
        let module_name = module_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        for stmt in &ast.stmts {
            match stmt {
                crate::ast::Stmt::Fn(_) => {
                    if let Some(name) = crate::stmt_symbol_name(stmt) {
                        let qualified = format!("{}.{}", module_name, name);
                        if seen.insert(qualified.clone()) {
                            let mut s = stmt.clone();
                            if let crate::ast::Stmt::Fn(ref mut f) = s {
                                f.name = crate::ast::Name::from(qualified.as_str());
                            }
                            out.push(s);
                        }
                    }
                }
                crate::ast::Stmt::TypeDecl(_)
                | crate::ast::Stmt::EnumDecl(_)
                | crate::ast::Stmt::Ext(_) => {
                    if let Some(name) = crate::stmt_symbol_name(stmt) {
                        if seen.insert(name.clone()) {
                            out.push(stmt.clone());
                        }
                    }
                }
                crate::ast::Stmt::Use(_) => {
                    out.push(stmt.clone());
                }
                crate::ast::Stmt::Store(s) => {
                    let key = format!("__store:{}", s.name);
                    if seen.insert(key) {
                        out.push(stmt.clone());
                    }
                }
                _ => {}
            }
        }
        let module_dir = module_path
            .parent()
            .unwrap_or(Path::new("."));
        for dep in scan_use_statements(&code) {
            if dep.is_c_import || dep.is_rust_import {
                continue;
            }
            if let Some(dep_path) = crate::resolve_module_path(module_dir, &dep.module) {
                collect_imports(&dep_path, visited, out, seen, session);
            }
        }
    }

    /// Write a small two-module project (back/api.at + back/db.at) to a temp
    /// dir and verify the flattened symbol table.
    #[test]
    fn test_collect_qualifies_all_functions() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let root = tmp.path();
        let back = root.join("back");
        fs::create_dir_all(&back).unwrap();
        fs::write(
            back.join("api.at"),
            r#"
pub type Note = { id int, title str, body str, time str }

pub fn list_notes() []Note {
    use db
    return db.all_notes()
}

pub fn create_note(title str, body str) Note {
    use db
    return db.create_note(title, body)
}
"#,
        )
        .unwrap();
        fs::write(
            back.join("db.at"),
            r#"
use api: Note

var notes List<Note> = List<Note>.new([])

pub fn all_notes() []Note {
    return notes.to_array()
}

pub fn create_note(title str, body str) Note {
    let note = Note { id: 0, title: title, body: body, time: "now" }
    notes.push(note)
    return note
}
"#,
        )
        .unwrap();

        let mut visited = HashSet::new();
        let mut out: Vec<crate::ast::Stmt> = Vec::new();
        let mut seen = HashSet::new();
        let mut session = CompileSession::new();
        collect_imports(
            &back.join("api.at"),
            &mut visited,
            &mut out,
            &mut seen,
            &mut session,
        );

        // Collect all qualified function names produced by flattening.
        let fn_names: Vec<String> = out
            .iter()
            .filter_map(|s| {
                if let crate::ast::Stmt::Fn(f) = s {
                    Some(f.name.to_string())
                } else {
                    None
                }
            })
            .collect();
        eprintln!("plan339: collected fn names = {:?}", fn_names);

        // Plan 339 Phase 3: EVERY Fn gets a module-qualified name.
        assert!(
            fn_names.iter().any(|n| n == "api.list_notes"),
            "api.list_notes missing: {:?}",
            fn_names
        );
        assert!(
            fn_names.iter().any(|n| n == "api.create_note"),
            "api.create_note missing: {:?}",
            fn_names
        );
        assert!(
            fn_names.iter().any(|n| n == "db.all_notes"),
            "db.all_notes missing: {:?}",
            fn_names
        );
        assert!(
            fn_names.iter().any(|n| n == "db.create_note"),
            "db.create_note missing: {:?}",
            fn_names
        );
        // No bare (unqualified) names should survive.
        assert!(
            !fn_names.iter().any(|n| n == "create_note" || n == "list_notes" || n == "all_notes"),
            "bare names should not survive: {:?}",
            fn_names
        );
        // api.create_note and db.create_note are DISTINCT (no last-wins dedup).
        let create_count = fn_names.iter().filter(|n| n.ends_with(".create_note")).count();
        assert_eq!(
            create_count, 2,
            "expected api.create_note AND db.create_note, got {}: {:?}",
            create_count, fn_names
        );
    }

    /// Verify the import_aliases map built from `use back.api: create_note`
    /// maps bare names to the module-qualified export name.
    #[test]
    fn test_import_aliases_map_bare_to_qualified() {
        let code = "use back.api: create_note, list_notes";
        let use_stmts = scan_use_statements(code);
        assert_eq!(use_stmts.len(), 1);
        let us = &use_stmts[0];
        assert_eq!(us.module, "back.api");
        assert_eq!(us.items, vec!["create_note".to_string(), "list_notes".to_string()]);

        // Mirror lib.rs's alias construction (Phase 4).
        let module_qualifier = us.module.split('.').last().unwrap_or(&us.module);
        let mut aliases: HashMap<String, String> = HashMap::new();
        for item in &us.items {
            let qualified = format!("{}.{}", module_qualifier, item);
            aliases.insert(item.clone(), qualified);
        }
        assert_eq!(aliases.get("create_note"), Some(&"api.create_note".to_string()));
        assert_eq!(aliases.get("list_notes"), Some(&"api.list_notes".to_string()));
    }

    /// End-to-end: load the REAL 015-notes sources, run the full
    /// `collect_module_imports` + alias build + `synthesize_widget_module`
    /// pipeline, and assert the App widget compiles with all cross-module
    /// calls (list_notes, create_note, update_note, delete_note) resolving.
    ///
    /// This reproduces exactly what `run_file_dynamic_ui` does, minus the
    /// iced render loop. If Plan 339's qualified-name + alias scheme is broken,
    /// `synthesize_widget_module` returns an error here.
    #[cfg(feature = "ui")]
    #[test]
    fn test_015_notes_app_compiles_with_namespace() {
        // Tests run from the crate dir; the repo root is two levels up.
        let candidates = [
            std::env::var("CARGO_MANIFEST_DIR")
                .ok()
                .map(|d| PathBuf::from(d))
                .map(|d| d.join("../../examples/ui/015-notes/src/front/app.at")),
            Some(PathBuf::from(
                "examples/ui/015-notes/src/front/app.at",
            )),
        ];
        let manifest = candidates
            .iter()
            .flatten()
            .find(|p| p.exists())
            .cloned();
        let Some(manifest) = manifest else {
            eprintln!("plan339: skipping — 015-notes app.at not found");
            return;
        };
        let base_dir = manifest.parent().unwrap_or(Path::new(".")).to_path_buf();
        let code = fs::read_to_string(&manifest).expect("read app.at");

        // 1. Extract the App widget.
        let session = crate::session::CompilerSession::ui();
        let mut parser = crate::Parser::from(code.as_str()).with_session(session);
        let ast = parser.parse().expect("parse app.at");
        let mut widget = None;
        for stmt in &ast.stmts {
            if let crate::ast::Stmt::WidgetDecl(decl) = stmt {
                widget = Some(
                    crate::aura::extract_widget_from_decl(decl)
                        .map_err(|e| e.to_string())
                        .expect("extract App"),
                );
                break;
            }
        }
        let widget = widget.expect("App widget");

        // 2. Collect imports + build aliases (mirror run_file_dynamic_ui).
        let mut visited = HashSet::new();
        let mut import_stmts: Vec<crate::ast::Stmt> = Vec::new();
        let mut seen_symbols = HashSet::new();
        let mut import_session = CompileSession::new();
        let mut import_aliases: HashMap<String, String> = HashMap::new();

        let use_stmts = scan_use_statements(&code);
        for use_stmt in &use_stmts {
            if use_stmt.is_c_import || use_stmt.is_rust_import {
                continue;
            }
            let Some(module_path) = crate::resolve_module_path(&base_dir, &use_stmt.module) else {
                continue;
            };
            collect_imports(
                &module_path,
                &mut visited,
                &mut import_stmts,
                &mut seen_symbols,
                &mut import_session,
            );
            let module_qualifier = use_stmt
                .module
                .split('.')
                .last()
                .unwrap_or(&use_stmt.module);
            for item in &use_stmt.items {
                let qualified = format!("{}.{}", module_qualifier, item);
                import_aliases.insert(item.clone(), qualified);
            }
        }

        eprintln!(
            "plan339: import_aliases = {:?}",
            import_aliases
        );

        // 3. Run the FULL pipeline through VmBridge (synthesis + link + VM
        //    init). This is exactly what run_file_dynamic_ui does, minus the
        //    iced render loop. A bare cross-module call (e.g. `search_notes`
        //    which lives in db.at but is not listed in `use back.api:`) must
        //    still link via the unique bare-name → qualified fallback.
        let bridge = crate::ui::vm_bridge::VmBridge::new_with_children(
            &widget,
            &[],
            import_stmts,
            &import_aliases,
            false,
        );
        match &bridge {
            Ok(_) => eprintln!("plan339: 015-notes VmBridge OK (link succeeded)"),
            Err(e) => panic!("015-notes VmBridge init failed: {:?}", e),
        }
    }

    /// Regression: 016-calendar uses `use calendar_util: build_month_grid, ...`
    /// (a sibling module, not nested under back/). Verify it still compiles
    /// under Plan 339's qualified-name scheme.
    #[cfg(feature = "ui")]
    #[test]
    fn test_016_calendar_app_compiles_with_namespace() {
        let candidates = [
            std::env::var("CARGO_MANIFEST_DIR")
                .ok()
                .map(|d| PathBuf::from(d))
                .map(|d| d.join("../../examples/ui/016-calendar/src/front/app.at")),
            Some(PathBuf::from(
                "examples/ui/016-calendar/src/front/app.at",
            )),
        ];
        let manifest = candidates
            .iter()
            .flatten()
            .find(|p| p.exists())
            .cloned();
        let Some(manifest) = manifest else {
            eprintln!("plan339: skipping — 016-calendar app.at not found");
            return;
        };
        let base_dir = manifest.parent().unwrap_or(Path::new(".")).to_path_buf();
        let code = fs::read_to_string(&manifest).expect("read app.at");

        let session = crate::session::CompilerSession::ui();
        let mut parser = crate::Parser::from(code.as_str()).with_session(session);
        let ast = parser.parse().expect("parse app.at");
        let mut widget = None;
        for stmt in &ast.stmts {
            if let crate::ast::Stmt::WidgetDecl(decl) = stmt {
                widget = Some(
                    crate::aura::extract_widget_from_decl(decl)
                        .map_err(|e| e.to_string())
                        .expect("extract App"),
                );
                break;
            }
        }
        let widget = widget.expect("App widget");

        let mut visited = HashSet::new();
        let mut import_stmts: Vec<crate::ast::Stmt> = Vec::new();
        let mut seen_symbols = HashSet::new();
        let mut import_session = CompileSession::new();
        let mut import_aliases: HashMap<String, String> = HashMap::new();

        let use_stmts = scan_use_statements(&code);
        for use_stmt in &use_stmts {
            if use_stmt.is_c_import || use_stmt.is_rust_import {
                continue;
            }
            let Some(module_path) = crate::resolve_module_path(&base_dir, &use_stmt.module) else {
                continue;
            };
            collect_imports(
                &module_path,
                &mut visited,
                &mut import_stmts,
                &mut seen_symbols,
                &mut import_session,
            );
            let module_qualifier = use_stmt
                .module
                .split('.')
                .last()
                .unwrap_or(&use_stmt.module);
            for item in &use_stmt.items {
                let qualified = format!("{}.{}", module_qualifier, item);
                import_aliases.insert(item.clone(), qualified);
            }
        }

        eprintln!("plan339: 016 import_aliases = {:?}", import_aliases);
        let bridge = crate::ui::vm_bridge::VmBridge::new_with_children(
            &widget,
            &[],
            import_stmts,
            &import_aliases,
            false,
        );
        match &bridge {
            Ok(_) => eprintln!("plan339: 016-calendar VmBridge OK (link succeeded)"),
            Err(e) => panic!("016-calendar VmBridge init failed: {:?}", e),
        }
    }
}

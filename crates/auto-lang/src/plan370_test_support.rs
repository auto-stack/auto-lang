//! Shared test support for Plan 370: build a DynamicComponent from the REAL
//! 015-notes example sources, mirroring `run_file_dynamic_ui_inner`.
//!
//! Used by `plan370_store_vm_tests` (data/render layer) and
//! `plan370_015_behavior_tests` (D1-D7 handler behavior). Both need to
//! construct the real 015-notes App (with its `use notes_store` store +
//! `use back.api` imports) exactly as production does.

#![cfg(test)]

use crate::ast::Stmt;
use crate::session::CompilerSession;
use crate::ui::dynamic::DynamicComponent;
use crate::ui::widget_registry::WidgetRegistry;
use crate::use_scanner::scan_use_statements;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

/// Locate the 015-notes app.at regardless of cwd (tests vs. IDE).
pub(crate) fn locate_app_at() -> Option<PathBuf> {
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
/// then return it after `fire_init()` so state is populated.
///
/// Walks the production path: parse app.at → extract root widget → collect
/// `use`-imported child widgets AND stores (StoreDecl → view-less child decl,
/// the D-GAP-4 fix) → collect module imports + aliases → build via
/// `with_registry_and_imports_from_decls` → fire_init.
///
/// Returns None (so tests gracefully no-op) when the example sources aren't
/// present (e.g. running the crate in isolation without the examples/ tree).
#[cfg(feature = "ui-interpreter")]
pub(crate) fn build_015_component() -> Option<DynamicComponent> {
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
                        // D-GAP-4: convert imported StoreDecl → view-less child
                        // WidgetDecl so its fields merge into root state.
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

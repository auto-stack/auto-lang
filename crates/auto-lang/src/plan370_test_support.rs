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
    locate_example_app_at("015-notes")
}

/// Locate any example's app.at by example dir name (e.g. "015-notes", "021-block-static").
pub(crate) fn locate_example_app_at(example: &str) -> Option<PathBuf> {
    let rel = format!("examples/ui/{}/src/front/app.at", example);
    let candidates = [
        std::env::var("CARGO_MANIFEST_DIR")
            .ok()
            .map(|d| PathBuf::from(d).join(format!("../../{}", rel))),
        Some(PathBuf::from(&rel)),
        Some(PathBuf::from(format!("../../{}", rel))),
        // Plan 409 §8: top-level examples (widgets-gallery) live directly under
        // examples/, not examples/ui/.
        std::env::var("CARGO_MANIFEST_DIR")
            .ok()
            .map(|d| PathBuf::from(d).join(format!("../../examples/{}/src/front/app.at", example))),
        Some(PathBuf::from(format!("examples/{}/src/front/app.at", example))),
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
    build_example_component("015-notes")
}

/// Build a DynamicComponent from any example's app.at (same production path
/// as `run_file_dynamic_ui_inner`): parse → root widget → collect use-imported
/// child widgets + stores → module imports → build → fire_init.
///
/// Returns None (graceful no-op) when the example sources aren't present.
#[cfg(feature = "ui-interpreter")]
pub(crate) fn build_example_component(example: &str) -> Option<DynamicComponent> {
    let manifest = locate_example_app_at(example)?;
    build_component_from_app(&manifest)
}

/// Path-based variant of `build_example_component` for fixtures outside
/// examples/ (e.g. test/ui corpora).
#[cfg(feature = "ui-interpreter")]
pub(crate) fn build_component_from_app(manifest: &Path) -> Option<DynamicComponent> {
    let base_dir = manifest.parent().unwrap_or(Path::new(".")).to_path_buf();
    // PLAN-050 T9: 与生产 build_dynamic_component 同款 i18n 查表装载。
    crate::ui::i18n_lookup::load_from_dir(&base_dir);
    let code = fs::read_to_string(manifest).unwrap();

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
        let module_path = match crate::resolve_use_module(&base_dir, use_stmt) {
            crate::UseModuleResolution::Module(p) => p,
            crate::UseModuleResolution::StoreFiles(found) => {
                // Plan 442 A2: legacy `use store: X` facade — production
                // fallback shared via resolve_use_module.
                for (_, path) in found {
                    crate::collect_module_imports(
                        &path,
                        &mut visited,
                        &mut import_stmts,
                        &mut seen_symbols,
                        &mut import_session,
                        None,
                    );
                }
                continue;
            }
            crate::UseModuleResolution::None => continue,
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
                                setup: None, // Plan 426 field; test support defaults
                                actions: None,
            timer: None,
                                view: None,
                                on: store_decl.on.clone(),
                                bind: None,
                                props: Vec::new(),
                                routes: None,
                                lifecycle: Vec::new(),
                                style: None,
                                ext_imports: Vec::new(),
                                watch: Vec::new(),
                                expose: Vec::new(),
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
    let mut root_store_names = HashSet::new();
    for stmt in &ast.stmts {
        if let Stmt::StoreDecl(store_decl) = stmt {
            root_store_names.insert(store_decl.name.to_string());
            store_as_child_decls.push(crate::ast::ui::WidgetDecl {
                name: store_decl.name.clone(),
                messages: store_decl.messages.clone(),
                model: store_decl.model.clone(),
                computed: store_decl.computed.clone(),
                                setup: None, // Plan 426 field; test support defaults
                                actions: None,
            timer: None,
                view: None,
                on: store_decl.on.clone(),
                bind: None,
                props: Vec::new(),
                routes: None,
                lifecycle: Vec::new(),
                style: None,
                ext_imports: Vec::new(),
                watch: Vec::new(),
                expose: Vec::new(),
            });
        }
    }
    // Stores collected from imported modules (incl. the Plan 442 A2 legacy
    // facade path) → view-less child decls, deduped against root + children
    // (mirrors the production import_stmts conversion).
    for stmt in &import_stmts {
        if let Stmt::StoreDecl(store_decl) = stmt {
            let sname = store_decl.name.to_string();
            if !root_store_names.contains(&sname)
                && !child_decls.iter().any(|d| d.name.to_string() == sname)
            {
                store_as_child_decls.push(crate::ast::ui::WidgetDecl {
                    name: store_decl.name.clone(),
                    messages: store_decl.messages.clone(),
                    model: store_decl.model.clone(),
                    computed: store_decl.computed.clone(),
                    setup: None,
                    actions: None,
            timer: None,
                    view: None,
                    on: store_decl.on.clone(),
                    bind: None,
                    props: Vec::new(),
                    routes: None,
                    lifecycle: Vec::new(),
                    style: None,
                    ext_imports: Vec::new(),
                    watch: Vec::new(),
                    expose: Vec::new(),
                });
            }
        }
    }
    let mut all_child_decls = child_decls.clone();
    all_child_decls.extend(store_as_child_decls.iter().cloned());

    // Mirror of production Plan 403: root module's own top-level fn/type/enum
    // declarations compile into the VM alongside imported ones.
    for stmt in &ast.stmts {
        match stmt {
            Stmt::Fn(_) | Stmt::TypeDecl(_) | Stmt::EnumDecl(_) => {
                import_stmts.push(stmt.clone());
            }
            _ => {}
        }
    }

    // Mirror of production Plan 442 A3: `use.web` ext imports (adapter-chain
    // loading + platform stubs) via the shared production helper.
    let mut ext_widget_decls: Vec<crate::ast::ui::WidgetDecl> = Vec::new();
    crate::load_ext_imports_for_vm(
        &base_dir,
        &ast,
        &root_decl,
        &all_child_decls,
        &mut visited,
        &mut import_stmts,
        &mut seen_symbols,
        &mut import_session,
        None,
        &mut import_aliases,
        &mut ext_widget_decls,
    )
    .expect("ext imports");
    // PLAN-051 C4: adapter widget 注册（与生产 build_dynamic_component_inner
    // 同款——use.web component 的 VM widget 形态进视图 registry）。
    for wd in &ext_widget_decls {
        if let Ok(w) = crate::aura::extract_widget_from_decl(wd) {
            all_child_decls.push(wd.clone());
            registry.register(w);
        }
    }

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

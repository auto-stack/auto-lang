//! Rust UI (ICED/GPUI) project generation utilities
//!
//! This module generates Rust code from AURA widget definitions,
//! targeting ICED or GPUI backends via the auto_lang::ui runtime.
//!
//! Workflow:
//! 1. Read .at files from a `front/` directory
//! 2. Parse with AURA pipeline (CompilerSession::ui with "rust" backend)
//! 3. Extract WidgetDecl AST nodes -> AuraWidget
//! 4. Generate Rust code via RustGenerator
//! 5. Wrap in main() with backend selection (ICED/GPUI)
//! 6. Write to `rust/<name>.rs`

use std::fs;
use std::path::{Path, PathBuf};

use auto_lang::api::types::{ApiModule, ApiEndpoint};

use auto_lang::ui_gen::rust::RustGenerator;
use auto_lang::ui_gen::BackendGenerator;
use auto_lang::Parser;
use auto_lang::session::CompilerSession;
use colored::Colorize;

use crate::AutoResult;

/// Generate Rust UI code from .at files in a project directory.
///
/// Resolve the front/ source directory for a project.
fn find_front_dir(project_dir: &Path) -> PathBuf {
    if project_dir.join("src").join("front").exists() {
        project_dir.join("src").join("front")
    } else if project_dir.join("source").join("front").exists() {
        project_dir.join("source").join("front")
    } else if project_dir.join("front").exists() {
        project_dir.join("front")
    } else {
        project_dir.join("src").join("front")
    }
}

/// Check if the generated Rust project needs to be regenerated.
/// Returns (needs_full_regen, needs_code_regen).
fn needs_regeneration(project_dir: &Path, rust_dir: &Path) -> (bool, bool) {
    let cargo_toml = rust_dir.join("Cargo.toml");
    let main_rs = rust_dir.join("src").join("main.rs");

    if !cargo_toml.exists() || !main_rs.exists() {
        return (true, true);
    }

    // Check if any .at source file is newer than main.rs
    let front_dir = find_front_dir(project_dir);
    if let Ok(at_files) = collect_at_files(&front_dir) {
        if let Ok(main_meta) = fs::metadata(&main_rs) {
            if let Ok(main_time) = main_meta.modified() {
                for at_file in &at_files {
                    if let Ok(at_meta) = fs::metadata(at_file) {
                        if let Ok(at_time) = at_meta.modified() {
                            if at_time > main_time {
                                return (false, true);
                            }
                        }
                    }
                }
            }
        }
    }

    // Check if default feature in Cargo.toml matches expected
    if let Ok(content) = fs::read_to_string(&cargo_toml) {
        if !content.contains("default = [\"ui-iced\"]") {
            return (true, true);
        }
    }

    (false, false)
}

/// Plan 374 / Plan 371 Task 22a: pre-scan `.at` files for `StoreDecl`s.
///
/// Shared by both `generate_rust_ui` (full regen) and `regenerate_code_only`
/// (incremental regen). Previously the incremental path skipped this and
/// passed an empty store list, starving the store-composable pipeline
/// (struct/enum generation + `store.X` rewriting registration) and producing
/// 86 errors on store-composable apps like 015-notes.
fn collect_store_decls(at_files: &[std::path::PathBuf]) -> Vec<auto_lang::ast::ui::StoreDecl> {
    let mut all_stores: Vec<auto_lang::ast::ui::StoreDecl> = Vec::new();
    for at_path in at_files {
        if let Ok(code) = std::fs::read_to_string(at_path) {
            let session = CompilerSession::ui().with_backend("rust");
            let mut parser = Parser::from(code.as_str()).with_session(session);
            if let Ok(ast) = parser.parse() {
                for stmt in &ast.stmts {
                    if let auto_lang::ast::Stmt::StoreDecl(ref store) = stmt {
                        all_stores.push(store.clone());
                    }
                }
            }
        }
    }
    all_stores
}

/// Plan 371 Task 22c: scan ALL `.at` files for each component's own scalar
/// state fields (name, rust_type), so a parent in one file can see a child's
/// fields defined in another file.
fn collect_component_state_fields(
    at_files: &[std::path::PathBuf],
) -> std::collections::HashMap<String, Vec<(String, String)>> {
    let mut map: std::collections::HashMap<String, Vec<(String, String)>> =
        std::collections::HashMap::new();
    for at_path in at_files {
        if let Ok(code) = std::fs::read_to_string(at_path) {
            let session = CompilerSession::ui().with_backend("rust");
            let mut parser = Parser::from(code.as_str()).with_session(session);
            if let Ok(ast) = parser.parse() {
                for stmt in &ast.stmts {
                    if let auto_lang::ast::Stmt::WidgetDecl(widget_decl) = stmt {
                        let fields: Vec<(String, String)> = widget_decl
                            .model
                            .as_ref()
                            .map(|m| {
                                m.fields
                                    .iter()
                                    .filter_map(|v| {
                                        let ty = match v.ty {
                                            auto_lang::ast::Type::Bool => Some("bool"),
                                            auto_lang::ast::Type::Int => Some("i32"),
                                            auto_lang::ast::Type::Float
                                            | auto_lang::ast::Type::Double => Some("f64"),
                                            auto_lang::ast::Type::StrOwned
                                            | auto_lang::ast::Type::StrFixed(_) => Some("String"),
                                            _ => None,
                                        };
                                        let ty = ty.or_else(|| match &v.init {
                                            auto_lang::ast::Expr::Bool(_) => Some("bool"),
                                            auto_lang::ast::Expr::Int(_) => Some("i32"),
                                            auto_lang::ast::Expr::Float(_, _)
                                            | auto_lang::ast::Expr::Double(_, _) => Some("f64"),
                                            auto_lang::ast::Expr::Str(_) => Some("String"),
                                            _ => None,
                                        });
                                        ty.map(|t| (v.name.to_string(), t.to_string()))
                                    })
                                    .collect()
                            })
                            .unwrap_or_default();
                        map.insert(widget_decl.name.to_string(), fields);
                    }
                }
            }
        }
    }
    map
}

/// Semantic info about a child component, collected via cross-file pre-scan.
/// Used by the Rust generator to replace hardcoded special-case logic
/// (Init forwarding, prop writeback, delete handling) with .at-source-driven
/// general code. Mirrors `collect_component_state_fields` in structure.
/// Defined in `auto_lang::ui_gen::rust` (the generator crate) and re-exported
/// here so `auto-man` can use it without a reverse dependency.
pub use auto_lang::ui_gen::rust::ComponentSemantics;

/// Cross-file pre-scan: for each widget, collect which props its handlers write.
/// Scans `on` blocks for assignments `self.<prop>.<field> = ...` where `<prop>`
/// matches a declared widget prop (so private state like `editing` is excluded).
pub fn collect_component_semantics(
    at_files: &[std::path::PathBuf],
) -> std::collections::HashMap<String, ComponentSemantics> {
    use auto_lang::ast::Stmt;

    let mut map: std::collections::HashMap<String, ComponentSemantics> =
        std::collections::HashMap::new();
    for at_path in at_files {
        if let Ok(code) = std::fs::read_to_string(at_path) {
            let session = CompilerSession::ui().with_backend("rust");
            let mut parser = Parser::from(code.as_str()).with_session(session);
            if let Ok(ast) = parser.parse() {
                for stmt in &ast.stmts {
                    if let Stmt::WidgetDecl(widget_decl) = stmt {
                        let prop_names: std::collections::HashSet<String> = widget_decl
                            .props
                            .iter()
                            .map(|p| p.name.to_string())
                            .collect();
                        let mut semantics = ComponentSemantics::default();
                        // Scan handler bodies for `self.<prop>.<field> = ...` assignments.
                        if let Some(on_block) = &widget_decl.on {
                            for handler in &on_block.handlers {
                                scan_written_props(&handler.body, &prop_names, &mut semantics.written_props);
                            }
                        }
                        // Dedup while preserving order.
                        semantics.written_props.sort();
                        semantics.written_props.dedup();
                        map.insert(widget_decl.name.to_string(), semantics);
                    }
                }
            }
        }
    }
    map
}

/// Recursively scan statements for prop-write assignments.
/// Matches `self.<prop>.<field> = rhs` (two-level Dot on left, Op::Asn),
/// where `<prop>` is one of the widget's declared props.
fn scan_written_props(
    body: &auto_lang::ast::Body,
    prop_names: &std::collections::HashSet<String>,
    out: &mut Vec<String>,
) {
    use auto_lang::ast::{Expr, Stmt};
    use auto_val::Op;
    for stmt in &body.stmts {
        match stmt {
            Stmt::Expr(Expr::Bina(left, op, _)) if matches!(op, Op::Asn) => {
                // left = self.<prop>.<field>  → Dot(Dot(Ident("self"), prop), field)
                if let Expr::Dot(inner, _field) = left.as_ref() {
                    if let Expr::Dot(obj, prop) = inner.as_ref() {
                        if let Expr::Ident(name) = obj.as_ref() {
                            if name.as_str() == "self" || name.as_str() == "." {
                                let prop_str = prop.as_str().to_string();
                                if prop_names.contains(&prop_str)
                                    && !out.contains(&prop_str)
                                {
                                    out.push(prop_str);
                                }
                            }
                        }
                    }
                }
            }
            // Recurse into control-flow constructs that contain nested bodies.
            Stmt::Expr(Expr::If(if_stmt)) => {
                for branch in &if_stmt.branches {
                    scan_written_props(&branch.body, prop_names, out);
                }
                if let Some(ref else_body) = if_stmt.else_ {
                    scan_written_props(else_body, prop_names, out);
                }
            }
            Stmt::For(for_stmt) => {
                scan_written_props(&for_stmt.body, prop_names, out);
            }
            Stmt::Block(inner_body) => {
                scan_written_props(inner_body, prop_names, out);
            }
            _ => {}
        }
    }
}

/// Regenerate only main.rs (skip Cargo.toml to preserve cargo cache).
fn regenerate_code_only(project_dir: &Path, rust_dir: &Path) -> AutoResult<()> {
    let front_dir = find_front_dir(project_dir);
    let at_files = collect_at_files(&front_dir)?;
    if at_files.is_empty() {
        return Ok(());
    }

    let pac_path = project_dir.join("pac.at");
    let project_name = if pac_path.exists() {
        parse_pac_name(&pac_path).unwrap_or_else(|| "MyApp".to_string())
    } else {
        "MyApp".to_string()
    };

    // Plan 374 / Plan 371 Task 22a: pre-scan all .at files for StoreDecls,
    // exactly like `generate_rust_ui` does. Previously this passed `&[]`,
    // which starved `compile_at_file`'s store registration + struct/enum
    // generation — producing 86 "no field `store`" / "cannot find type
    // `StoreMsg`" errors on incremental regen of store-composable apps.
    let all_stores = collect_store_decls(&at_files);
    if !all_stores.is_empty() {
        println!("  {} {} store composable(s) found", "Found".bright_green(), all_stores.len());
    }
    // Plan 371 Task 22c: cross-file component state fields.
    let component_fields = collect_component_state_fields(&at_files);
    // Plan 371 L1: cross-file component semantics (written props).
    let component_semantics = collect_component_semantics(&at_files);

    let mut all_components = String::new();
    let mut all_api_imports: Vec<String> = Vec::new();
    for at_path in &at_files {
        match compile_at_file(at_path, &all_stores, &component_fields, &component_semantics) {
            Ok((code, api_imports)) => {
                all_components.push_str(&code);
                all_components.push('\n');
                all_api_imports.extend(api_imports);
            }
            Err(e) => {
                let file_name = at_path.file_name().unwrap_or_default().to_string_lossy();
                println!("{} Failed to compile {}: {}", "Warning:".bright_yellow(), file_name, e);
            }
        }
    }

    if all_components.trim().is_empty() {
        return Ok(());
    }

    // Deduplicate API imports and generate API client once
    deduplicate_imports(&mut all_api_imports);
    if !all_api_imports.is_empty() {
        all_components.push('\n');
        all_components.push_str(&generate_api_client(project_dir, &all_api_imports));
    }

    let full_code = wrap_example(&project_name, &all_components);
    let main_rs = rust_dir.join("src").join("main.rs");
    fs::write(&main_rs, &full_code)
        .map_err(|e| format!("Failed to write {}: {}", main_rs.display(), e))?;

    Ok(())
}

/// `project_dir` is the workspace root (where pac.at lives).
/// `output_dir` overrides the default `rust/` output directory.
/// `_project` is reserved for future full-project scaffolding.
pub fn generate_rust_ui(
    project_dir: &Path,
    output_dir: Option<&Path>,
    _project: bool,
) -> AutoResult<()> {
    println!("{}", "Generating Rust UI code".bright_cyan());

    let front_dir = find_front_dir(project_dir);

    if !front_dir.exists() {
        return Err(format!(
            "Front directory not found: {}",
            front_dir.display()
        )
        .into());
    }

    // Collect .at files
    let at_files = collect_at_files(&front_dir)?;
    if at_files.is_empty() {
        println!("{}", "  No .at files found in front directory".bright_yellow());
        return Ok(());
    }

    println!(
        "{} {} files found",
        "  Found".bright_green(),
        at_files.len()
    );

    // Determine output directory — use shared workspace at examples/rust-workspace/
    let ws_dir = ensure_shared_workspace(project_dir);
    let member_name = front_member_name(project_dir);
    let default_output = ws_dir.join(&member_name);
    let output = output_dir
        .map(|p| p.to_path_buf())
        .unwrap_or(default_output);

    fs::create_dir_all(&output)
        .map_err(|e| format!("Failed to create output directory: {}", e))?;

    // Get project name from pac.at
    let pac_path = project_dir.join("pac.at");
    let project_name = if pac_path.exists() {
        parse_pac_name(&pac_path).unwrap_or_else(|| "MyApp".to_string())
    } else {
        "MyApp".to_string()
    };

    // Plan 374 Task 2-3: Pre-scan all .at files to collect StoreDecls.
    let all_stores = collect_store_decls(&at_files);
    if !all_stores.is_empty() {
        println!("  {} {} store composable(s) found", "Found".bright_green(), all_stores.len());
    }

    // Plan 371 Task 22c: cross-file component state fields.
    let component_fields = collect_component_state_fields(&at_files);
    // Plan 371 L1: cross-file component semantics (written props).
    let component_semantics = collect_component_semantics(&at_files);

    // Compile each .at file and collect generated components
    let mut all_components = String::new();
    let mut all_api_imports: Vec<String> = Vec::new();
    for at_path in &at_files {
        let file_name = at_path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy();
        println!("  {} {}", "Parsing".bright_cyan(), file_name);

        match compile_at_file(at_path, &all_stores, &component_fields, &component_semantics) {
            Ok((code, api_imports)) => {
                all_components.push_str(&code);
                all_components.push('\n');
                all_api_imports.extend(api_imports);
            }
            Err(e) => {
                println!(
                    "{} Failed to compile {}: {}",
                    "Warning:".bright_yellow(),
                    file_name,
                    e
                );
            }
        }
    }

    if all_components.trim().is_empty() {
        println!(
            "{}",
            "  No components generated (no WidgetDecl nodes found)".bright_yellow()
        );
        return Ok(());
    }

    // Deduplicate API imports and generate API client once
    deduplicate_imports(&mut all_api_imports);
    if !all_api_imports.is_empty() {
        all_components.push('\n');
        all_components.push_str(&generate_api_client(project_dir, &all_api_imports));
    }

    // Wrap in main() boilerplate
    let main_widget = extract_main_widget(&all_components);
    let full_code = wrap_example(&project_name, &all_components);

    // Write output as a Cargo project
    let src_dir = output.join("src");
    fs::create_dir_all(&src_dir)
        .map_err(|e| format!("Failed to create src directory: {}", e))?;

    let main_rs = src_dir.join("main.rs");
    fs::write(&main_rs, &full_code)
        .map_err(|e| format!("Failed to write {}: {}", main_rs.display(), e))?;

    // Generate Cargo.toml with workspace dependencies
    let cargo_toml = generate_cargo_toml(&project_name, project_dir);
    let cargo_path = output.join("Cargo.toml");
    fs::write(&cargo_path, &cargo_toml)
        .map_err(|e| format!("Failed to write {}: {}", cargo_path.display(), e))?;

    // Note: no per-member .cargo/config.toml needed — the workspace-level
    // .cargo/config.toml sets target-dir for all members.

    // Update workspace members to include the newly created project
    let _ = ensure_shared_workspace(project_dir);

    println!();
    println!(
        "{} {}",
        "  Generated".bright_green(),
        output.display()
    );
    println!(
        "{} {} (main widget)",
        "  Entry".bright_green(),
        main_widget
    );
    println!();
    println!(
        "{}",
        "  Rust UI project generated successfully!".bright_green().bold()
    );

    Ok(())
}

/// Extract API function names from `use back.api: fn1, fn2, ...` statements.
fn extract_api_imports_from_ast(ast: &auto_lang::ast::Code) -> Vec<String> {
    let mut imports = Vec::new();
    for stmt in &ast.stmts {
        if let auto_lang::ast::Stmt::Use(ref use_stmt) = stmt {
            if is_api_use(use_stmt) {
                imports.extend(use_stmt.items.iter().map(|s| s.as_str().to_string()));
            }
        }
    }
    imports
}

/// Check if a `use` statement targets `back.api`
fn is_api_use(use_stmt: &auto_lang::ast::Use) -> bool {
    if use_stmt.paths.len() == 2
        && use_stmt.paths[0].as_str() == "back"
        && use_stmt.paths[1].as_str() == "api"
    {
        return true;
    }
    if let Some(ref mp) = use_stmt.module_path {
        if mp.display() == "back.api" {
            return true;
        }
    }
    false
}

/// Compile a single .at file to Rust UI code.
/// Returns (generated_code, api_imports) so callers can deduplicate API stubs.
fn compile_at_file(
    at_path: &Path,
    stores: &[auto_lang::ast::ui::StoreDecl],
    component_fields: &std::collections::HashMap<String, Vec<(String, String)>>,
    component_semantics: &std::collections::HashMap<String, ComponentSemantics>,
) -> AutoResult<(String, Vec<String>)> {
    let code = fs::read_to_string(at_path)
        .map_err(|e| format!("Failed to read {}: {}", at_path.display(), e))?;

    // Parse with UI scenario targeting rust backend
    let session = CompilerSession::ui().with_backend("rust");
    let mut parser = Parser::from(code.as_str());
    parser = parser.with_session(session);
    let ast = parser
        .parse()
        .map_err(|e| format!("Parse error: {:?}", e))?;

    let mut output = String::new();
    let mut generator = RustGenerator::new();

    // Extract API imports from `use back.api: ...` statements
    let api_imports = extract_api_imports_from_ast(&ast);

    // Plan 374 Task 1: Register view fn fragments BEFORE extracting widgets.
    auto_lang::aura::extract::clear_view_fragments();
    for stmt in &ast.stmts {
        if let auto_lang::ast::Stmt::ViewFragmentDecl(frag) = stmt {
            auto_lang::aura::extract::register_view_fragment(frag);
        }
    }

    // Plan 374 Task 2: Register store names for store.X rewriting (all files),
    // but only GENERATE the store struct in the file that declares it.
    for store in stores {
        generator.register_store("store", store.name.as_str());
    }
    // Check if THIS file contains any StoreDecl — only generate from here.
    let file_has_store = ast.stmts.iter().any(|s| matches!(s, auto_lang::ast::Stmt::StoreDecl(_)));
    if file_has_store {
    for store in stores {
        let fake_decl = auto_lang::ast::ui::WidgetDecl {
            name: store.name.clone(),
            messages: store.messages.clone(),
            model: store.model.clone(),
            computed: store.computed.clone(),
            view: None,
            on: store.on.clone(),
            bind: None,
            props: Vec::new(),
            routes: None,
            lifecycle: Vec::new(),
            style: None,
            ext_imports: Vec::new(),
            watch: Vec::new(),
            expose: Vec::new(),
        };
        match auto_lang::aura::extract_widget_from_decl(&fake_decl) {
            Ok(aura_widget) => {
                let rust_code = generator.generate(&aura_widget).map_err(|e| e.to_string())?;
                output.push_str(&rust_code);
                output.push('\n');
            }
            Err(e) => {
                eprintln!("  Warning: store '{}' extraction failed: {}", store.name, e);
            }
        }
    }
    }

    // Plan 371 Task 22c: register every component's own scalar state fields.
    for (name, fields) in component_fields.iter() {
        generator.register_component_state(name, fields.clone());
    }
    // Plan 371 L1: register every component's semantics (written props).
    for (name, sem) in component_semantics.iter() {
        generator.register_component_semantics(name, sem.clone());
    }

    // Extract AURA widgets from AST
    for stmt in &ast.stmts {
        if let auto_lang::ast::Stmt::WidgetDecl(widget_decl) = stmt {
            let mut aura_widget = auto_lang::aura::extract_widget_from_decl(widget_decl)
                .map_err(|e| e.to_string())?;
            aura_widget.api_imports = api_imports.clone();

            let rust_code = generator
                .generate(&aura_widget)
                .map_err(|e| e.to_string())?;

            output.push_str(&rust_code);
            output.push('\n');
        }
    }

    // Plan 374: Generate Rust type aliases from Auto type declarations.
    // User-defined types (like Note) are aliased to serde_json::Value since
    // the code generator uses Value-based dynamic field access (["field"]).
    for stmt in &ast.stmts {
        if let auto_lang::ast::Stmt::TypeDecl(type_decl) = stmt {
            output.push_str(&generate_type_struct(type_decl));
            output.push('\n');
        }
    }

    // NOTE: API stubs are NOT generated here — callers collect all imports
    // across files and generate stubs once to avoid duplicates.

    Ok((output, api_imports))
}

/// Deduplicate API imports, preserving order.
fn deduplicate_imports(imports: &mut Vec<String>) {
    let mut seen = std::collections::HashSet::new();
    imports.retain(|s| seen.insert(s.clone()));
}

/// Generate API client functions for Rust UI.
/// Parses the API definition from src/back/api.at and generates reqwest HTTP calls (Plan 388 W1: migrated from ureq).
/// Falls back to heuristic stubs if the API file can't be parsed.
fn generate_api_client(project_dir: &Path, api_imports: &[String]) -> String {
    // AUTO_HTTP_PORT lets multiple `auto run` instances coexist; default 8080.
    let base_url = crate::util::http_base_url();

    // Try to parse api.at to get real endpoint definitions
    let api_module = parse_api_module(project_dir);

    // Plan 347: In merged mode (AUTO_VM_MERGE != "0", the default), generate
    // in-process direct-call functions instead of HTTP client code.
    let merged_mode = std::env::var("AUTO_VM_MERGE").as_deref() != Ok("0");

    if merged_mode {
        if let Some(module) = &api_module {
            return generate_merged_api_client(module);
        }
        // Fallback to stubs if no api.at found
        return generate_api_stubs(api_imports);
    }

    // Split mode: generate HTTP client functions.
    if let Some(module) = &api_module {
        let mut code = String::new();
        // Plan 349 step 1: Generate a TLS-aware HTTP client helper.
        code.push_str(&generate_http_client_helper());
        // Plan 349 step 2-3: Generate upload/download utility functions.
        code.push_str(&generate_http_utility_functions());
        // Plan 350 step 4: Generate WebSocket client functions.
        code.push_str(&generate_ws_functions());

        for endpoint in &module.endpoints {
            code.push_str(&generate_endpoint_fn(endpoint, &base_url));
            code.push('\n');
        }
        return code;
    }

    // Fallback: heuristic stubs based on function name convention
    generate_api_stubs(api_imports)
}

/// Parse the API module from src/back/api.at
fn parse_api_module(project_dir: &Path) -> Option<ApiModule> {
    let back_dir = if project_dir.join("src").join("back").exists() {
        project_dir.join("src").join("back")
    } else if project_dir.join("back").exists() {
        project_dir.join("back")
    } else {
        return None;
    };

    let api_file = back_dir.join("api.at");
    if !api_file.exists() {
        return None;
    }

    let content = fs::read_to_string(&api_file).ok()?;
    crate::api_gen::try_full_parse(&content)
        .or_else(|| crate::api_gen::extract_api_lenient(&content))
}

/// Plan 349 step 1 / Plan 388 W1: Generate a TLS-aware HTTP client helper.
/// The generated API client uses `reqwest::blocking` so TLS can be configured
/// via a `ClientBuilder` (ureq cannot configure certificates per-request).
/// Set AUTO_TLS_SKIP_VERIFY=1 to skip certificate verification (dev/test);
/// set AUTO_TLS_CA_CERT=/path/to/ca.pem to add a custom CA (PEM). The client is
/// built once (OnceLock) and reused; per-call cloning is cheap (Arc-backed).
/// Plan 349 步骤 7/8 (W6): AUTO_COOKIE_STORE=1 / AUTO_GZIP=1 / AUTO_BROTLI=1 enable
/// cookie persistence and response decompression, mirroring the VM-side
/// RequestBuilder.cookie_store/gzip/brotli flags.
fn generate_http_client_helper() -> String {
    r#"// Plan 349/388: TLS-aware reqwest blocking client helper.
// Set AUTO_TLS_SKIP_VERIFY=1 to skip certificate verification (dev/test).
// Set AUTO_TLS_CA_CERT=/path/to/ca.pem to add a custom CA (PEM).
// Plan 349 步骤 7/8 (W6): Set AUTO_COOKIE_STORE=1 / AUTO_GZIP=1 / AUTO_BROTLI=1 to
// enable cookie persistence and response decompression.
fn _tls_skip_verify() -> bool {
    std::env::var("AUTO_TLS_SKIP_VERIFY").as_deref() == Ok("1")
}

fn _http_client() -> reqwest::blocking::Client {
    use std::sync::OnceLock;
    static CLIENT: OnceLock<reqwest::blocking::Client> = OnceLock::new();
    CLIENT.get_or_init(|| {
        let mut builder = reqwest::blocking::Client::builder();
        if _tls_skip_verify() {
            builder = builder.danger_accept_invalid_certs(true);
        }
        if let Ok(ca_path) = std::env::var("AUTO_TLS_CA_CERT") {
            if let Ok(pem) = std::fs::read(&ca_path) {
                if let Ok(cert) = reqwest::Certificate::from_pem(&pem) {
                    builder = builder.add_root_certificate(cert);
                }
            }
        }
        // Plan 349 步骤 7/8 (W6): cookie store + compression, env-driven (default off).
        if std::env::var("AUTO_COOKIE_STORE").as_deref() == Ok("1") {
            builder = builder.cookie_store(true);
        }
        if std::env::var("AUTO_GZIP").as_deref() == Ok("1") {
            builder = builder.gzip(true);
        }
        if std::env::var("AUTO_BROTLI").as_deref() == Ok("1") {
            builder = builder.brotli(true);
        }
        // Fall back to a plain client only if the builder itself fails (e.g. a
        // malformed PEM): failures are then *stricter* (verification stays on,
        // custom CA absent → connection errors), never silently weaker.
        builder.build().unwrap_or_default()
    })
    .clone()
}

"#.to_string()
}


fn generate_endpoint_fn(endpoint: &ApiEndpoint, base_url: &str) -> String {
    let fn_name = &endpoint.fn_name;
    let method = endpoint.method().to_uppercase();
    let path = endpoint.path();
    let path = path.strip_prefix('/').unwrap_or(&path);

    // Separate path params from body params
    let full_path = format!("/{}", path); // keep for :param matching
    let path_params: Vec<_> = endpoint.params.iter()
        .filter(|p| full_path.contains(&format!(":{}", p.name)))
        .collect();
    let body_params: Vec<_> = endpoint.params.iter()
        .filter(|p| !full_path.contains(&format!(":{}", p.name)))
        .collect();

    // Build URL — if path has :param, use format!()
    let has_path_params = !path_params.is_empty();
    let url_expr = if has_path_params {
        let mut url_fmt = path.to_string();
        let mut format_args: Vec<String> = Vec::new();
        for p in &path_params {
            url_fmt = url_fmt.replace(&format!(":{}", p.name), "{}");
            format_args.push(p.name.clone());
        }
        format!("&format!(\"{}/{}\", {})", base_url, url_fmt, format_args.join(", "))
    } else {
        format!("\"{}/{}\"", base_url, path)
    };

    // Function parameters
    let params: Vec<String> = endpoint.params.iter()
        .map(|p| format!("{}: {}", p.name, auto_type_to_rust(&p.ty)))
        .collect();
    let param_list = params.join(", ");

    // Return type — use serde_json::Value for the Rust UI since widgets work with Value
    let return_type = &endpoint.return_type;
    let is_void = return_type == "void";
    let is_vec = return_type.starts_with("[]");
    let is_option = return_type.starts_with("?");

    let (rust_return_type, value_type) = if is_void {
        (String::new(), String::new())
    } else if is_vec {
        ("Vec<serde_json::Value>".to_string(), "Vec<serde_json::Value>".to_string())
    } else if is_option {
        ("Option<serde_json::Value>".to_string(), "Option<serde_json::Value>".to_string())
    } else {
        ("serde_json::Value".to_string(), "serde_json::Value".to_string())
    };

    // Generate function body based on HTTP method
    let ureq_method = method.to_lowercase();
    // DELETE never needs a return value in the UI — treat as void
    // Option-returning PUT/PATCH are also fire-and-forget (non-blocking) — treat as void
    let is_fire_and_forget = is_void || method == "DELETE" || (method != "GET" && is_option);
    let body = if method == "GET" {
        generate_get_fn_body(ureq_method, url_expr, is_fire_and_forget, &value_type)
    } else if method == "DELETE" {
        generate_delete_fn_body(url_expr)
    } else {
        // POST, PUT — send JSON body
        generate_write_fn_body(ureq_method, url_expr, &body_params, is_void, &value_type)
    };

    if is_fire_and_forget {
        format!("fn {}({}) {{\n{}}}\n", fn_name, param_list, body)
    } else {
        format!("fn {}({}) -> {} {{\n{}}}\n", fn_name, param_list, rust_return_type, body)
    }
}

/// Generate body for GET requests (reqwest::blocking, Plan 388 W1)
fn generate_get_fn_body(method: String, url_expr: String, is_void: bool, return_type: &str) -> String {
    if is_void {
        format!("    let _ = _http_client().{}({}).send();\n", method, url_expr)
    } else if return_type.starts_with("Vec<") {
        format!(
            "    _http_client().{}({})\n        .send().ok()\n        .and_then(|r| r.json::<{}>().ok())\n        .unwrap_or_default()\n",
            method, url_expr, return_type
        )
    } else if return_type.starts_with("Option<") {
        // Deserialize as Value, let .ok() produce Option<Value> naturally
        format!(
            "    _http_client().{}({})\n        .send().ok()\n        .and_then(|r| r.json::<serde_json::Value>().ok())\n",
            method, url_expr
        )
    } else {
        format!(
            "    _http_client().{}({})\n        .send().ok()\n        .and_then(|r| r.json::<{}>().ok())\n        .unwrap_or_default()\n",
            method, url_expr, return_type
        )
    }
}

/// Generate body for DELETE requests (non-blocking via background thread)
fn generate_delete_fn_body(url_expr: String) -> String {
    let url_owned = if url_expr.starts_with('&') || url_expr.starts_with("format!") {
        format!("    let url = {}.to_string();\n", url_expr.trim_start_matches('&'))
    } else {
        format!("    let url = {}.to_string();\n", url_expr)
    };
    format!("{}    std::thread::spawn(move || {{ let _ = _http_client().delete(&url).send(); }});\n", url_owned)
}

/// Generate body for POST/PUT requests (with JSON body)
///
/// For void return types (e.g., update_note): non-blocking via background thread.
/// For value return types (e.g., create_note): returns a local JSON placeholder and
/// spawns a background thread for the actual HTTP call. The returned placeholder
/// contains the params so the UI can display immediately.
fn generate_write_fn_body(method: String, url_expr: String, body_params: &[&auto_lang::api::types::ApiParam], is_void: bool, return_type: &str) -> String {
    let json_fields: Vec<String> = body_params.iter()
        .map(|p| format!("\"{}\": {}", p.name, p.name))
        .collect();
    let json_body = format!("serde_json::json!({{{}}})", json_fields.join(", "));

    if is_void {
        // Non-blocking: spawn background thread for fire-and-forget
        let url_owned = if url_expr.starts_with('&') || url_expr.starts_with("format!") {
            format!("    let url = {}.to_string();\n", url_expr.trim_start_matches('&'))
        } else {
            format!("    let url = {}.to_string();\n", url_expr)
        };
        format!(
            "{}    let body = {};\n    std::thread::spawn(move || {{ let _ = _http_client().{}(&url).json(&body).send(); }});\n",
            url_owned, json_body, method
        )
    } else if return_type.starts_with("Vec<") {
        // Vec return — keep blocking (rare for POST/PUT)
        format!(
            "    _http_client().{}({})\n        .json(&{})\n        .send().ok()\n        .and_then(|r| r.json::<{}>().ok())\n        .unwrap_or_default()\n",
            method, url_expr, json_body, return_type
        )
    } else if return_type.starts_with("Option<") {
        // Option return (e.g., update_note) — non-blocking: fire-and-forget in background thread
        let url_owned = if url_expr.starts_with('&') || url_expr.starts_with("format!") {
            format!("    let url = {}.to_string();\n", url_expr.trim_start_matches('&'))
        } else {
            format!("    let url = {}.to_string();\n", url_expr)
        };
        format!(
            "{}    let body = {};\n    std::thread::spawn(move || {{ let _ = _http_client().{}(&url).json(&body).send(); }});\n",
            url_owned, json_body, method
        )
    } else {
        // Value return (e.g., create_note → serde_json::Value)
        // Non-blocking: return a local placeholder with the params, POST in background.
        // The actual server-generated ID won't be available, but the UI works immediately.
        let local_fields: Vec<String> = body_params.iter()
            .map(|p| format!("\"{}\": {}", p.name, p.name))
            .collect();
        let local_json = format!("serde_json::json!({{{}}})", local_fields.join(", "));
        let url_owned = if url_expr.starts_with('&') || url_expr.starts_with("format!") {
            format!("    let url = {}.to_string();\n", url_expr.trim_start_matches('&'))
        } else {
            format!("    let url = {}.to_string();\n", url_expr)
        };
        format!(
            "{}    let body = {};\n    let local_result = {local_json};\n    std::thread::spawn(move || {{ let _ = _http_client().{}(&url).json(&body).send(); }});\n    local_result\n",
            url_owned, json_body, method, local_json = local_json
        )
    }
}

/// Convert Auto type string to Rust type for API function signatures
fn auto_type_to_rust(ty: &str) -> String {
    match ty {
        "int" => "i32".to_string(),
        "i64" => "i64".to_string(),
        "str" => "String".to_string(),
        "bool" => "bool".to_string(),
        "void" => "()".to_string(),
        s if s.starts_with("[]") => {
            let inner = &s[2..];
            format!("Vec<{}>", auto_type_to_rust(inner))
        }
        s if s.starts_with("?") => {
            let inner = &s[1..];
            format!("Option<{}>", auto_type_to_rust(inner))
        }
        s => s.to_string(),
    }
}

/// Build the RHS expression that stores a merged-client parameter into a JSON
/// object field. `[]str`/array params (`Vec<String>`) become a JSON array so
/// the field keeps its array shape (e.g. Note.tags); scalars use Value::from.
fn merged_param_to_value(ty: &str, param_name: &str) -> String {
    if ty.starts_with("[]") {
        format!(
            "serde_json::Value::Array({}.iter().map(|s| serde_json::Value::from(s.clone())).collect())",
            param_name
        )
    } else {
        format!("serde_json::Value::from({}.clone())", param_name)
    }
}

/// Generate heuristic stub functions when API module can't be parsed.
fn generate_api_stubs(api_imports: &[String]) -> String {
    let mut code = String::new();
    code.push_str("// API function stubs (no api.at found — using heuristic placeholders)\n");
    for fn_name in api_imports {
        let lower = fn_name.to_lowercase();
        if lower.starts_with("list_") || lower.starts_with("list") {
            code.push_str(&format!(
                "fn {}() -> Vec<serde_json::Value> {{ vec![] }}\n\n", fn_name
            ));
        } else if lower.starts_with("create_") {
            code.push_str(&format!(
                "fn {}(_title: String, _body: String) -> serde_json::Value {{ serde_json::json!({{\"id\": 0, \"title\": _title, \"body\": _body, \"time\": \"now\"}}) }}\n\n",
                fn_name
            ));
        } else if lower.starts_with("update_") {
            code.push_str(&format!(
                "fn {}(_id: i32, _title: String, _body: String) {{ }}\n\n", fn_name
            ));
        } else if lower.starts_with("delete_") {
            code.push_str(&format!(
                "fn {}(_id: i32) {{ }}\n\n", fn_name
            ));
        } else if lower.starts_with("get_") {
            code.push_str(&format!(
                "fn {}(_id: i32) -> Option<serde_json::Value> {{ None }}\n\n", fn_name
            ));
        } else {
            code.push_str(&format!(
                "fn {}() {{ }}\n\n", fn_name
            ));
        }
    }
    code
}

/// Plan 347: Generate JSON initial data for the merged-mode global store.
/// Returns a string like `[json!({"id":0,"title":"Welcome",...}), ...]`
/// (without the outer `vec![]` wrapper).
fn generate_json_initial_data(module: &auto_lang::api::ApiModule) -> String {
    let primary_type = crate::api_gen::primary_type_name_pub(module);
    let primary_type = match primary_type {
        Some(t) => t,
        None => return "[]".to_string(),
    };
    let api_type = match module.types.iter().find(|t| t.name == primary_type) {
        Some(t) => t,
        None => return "[]".to_string(),
    };

    let mut items = vec![];
    for i in 0..3i64 {
        let fields: Vec<String> = api_type.fields.iter().map(|f| {
            let val = match f.ty.as_str() {
                "int" | "i64" => format!("{}", i),
                "bool" => match f.name.as_str() {
                    // pinned: first note pinned so the "Pinned" tab isn't empty.
                    "pinned" => if i == 0 { "true" } else { "false" }.to_string(),
                    _ => "false".to_string(),
                },
                s if s.starts_with("[]") => {
                    // Array field (e.g. tags) — emit a non-empty JSON array so
                    // tag filters and renders have data to show.
                    let sample = match f.name.as_str() {
                        "tags" => match i { 0 => "intro", 1 => "home", _ => "work" },
                        _ => "item",
                    };
                    format!("[\"{}\"]", sample)
                }
                _ => {
                    let sample = match f.name.as_str() {
                        "title" | "name" => match i { 0 => "Welcome", 1 => "Shopping List", _ => "Meeting Notes" },
                        "body" | "description" | "content" => match i { 0 => "This is your notes app. Click on any note to view it.", 1 => "Milk, Eggs, Bread, Cheese", _ => "Q3 roadmap discussion with the team" },
                        "time" | "date" | "created_at" => match i { 0 => "Just now", 1 => "2 hours ago", _ => "Yesterday" },
                        // folder: spread the 3 seed notes across the categories
                        // ("" / "personal" / "work") so each folder tab shows at
                        // least one note. Without this all notes get a generic
                        // value and folder-filtered lists render empty.
                        "folder" | "category" => match i { 0 => "", 1 => "personal", _ => "work" },
                        _ => "Sample",
                    };
                    format!("\"{}\"", sample)
                }
            };
            format!("\"{}\": {}", f.name, val)
        }).collect();
        items.push(format!("serde_json::json!({{{}}})", fields.join(", ")));
    }
    format!("[{}]", items.join(", "))
}

/// Plan 347: Generate in-process API functions for Rust+Rust merged mode.
/// Instead of HTTP calls, functions operate on a global `static DATA: Mutex<Vec<Value>>`.
/// Function signatures match the HTTP version so widget call sites don't change.
fn generate_merged_api_client(module: &auto_lang::api::ApiModule) -> String {
    let mut code = String::new();
    code.push_str("// API functions (auto-generated, in-process merged mode — no HTTP)\n\n");

    // Generate JSON initial data (not strong-typed structs).
    let initial_items = generate_json_initial_data(module);
    code.push_str("use std::sync::{LazyLock, Mutex};\n");
    code.push_str("use serde_json::Value;\n\n");
    code.push_str(&format!(
        "static API_DATA: LazyLock<Mutex<Vec<Value>>> = LazyLock::new(|| {{\n    Mutex::new(vec!{})\n}});\n",
        initial_items
    ));
    code.push_str("static API_NEXT_ID: LazyLock<Mutex<i64>> = LazyLock::new(|| Mutex::new(100));\n\n");

    for endpoint in &module.endpoints {
        let fn_name = &endpoint.fn_name;
        let method = endpoint.method().to_uppercase();
        let params: Vec<&str> = endpoint.params.iter().map(|p| p.name.as_str()).collect();
        let path_params: Vec<_> = endpoint.params.iter()
            .filter(|p| endpoint.path().contains(&format!(":{}", p.name)))
            .collect();
        let body_params: Vec<_> = endpoint.params.iter()
            .filter(|p| !endpoint.path().contains(&format!(":{}", p.name)))
            .collect();

        match method.as_str() {
            "GET" => {
                if params.is_empty() {
                    // list: return all
                    code.push_str(&format!(
                        "fn {}() -> Vec<Value> {{\n    API_DATA.lock().unwrap().clone()\n}}\n\n", fn_name
                    ));
                } else {
                    // get by id/path param
                    let id_param = path_params.first().map(|p| p.name.as_str()).unwrap_or("id");
                    code.push_str(&format!(
                        "fn {}({}: i32) -> Option<Value> {{\n    API_DATA.lock().unwrap().iter().find(|n| n[\"id\"].as_i64() == Some({} as i64)).cloned()\n}}\n\n",
                        fn_name, id_param, id_param
                    ));
                }
            }
            "POST" => {
                let body_fields: Vec<String> = body_params.iter()
                    .map(|p| format!("\"{}\": {}", p.name, merged_param_to_value(&p.ty, &p.name)))
                    .collect();
                code.push_str(&format!(
                    "fn {}({}) -> Value {{\n    let mut data = API_DATA.lock().unwrap();\n    let id = {{ let mut next = API_NEXT_ID.lock().unwrap(); *next += 1; *next }};\n    let item = serde_json::json!({{\"id\": id, {}}});\n    data.push(item.clone());\n    item\n}}\n\n",
                    fn_name,
                    body_params.iter().map(|p| format!("{}: {}", p.name, auto_type_to_rust(&p.ty))).collect::<Vec<_>>().join(", "),
                    body_fields.join(", ")
                ));
            }
            "PUT" => {
                // PUT is fire-and-forget in the UI (callers don't use the
                // return value), so return () — matching split mode's
                // fire-and-forget PUT. Param types come from the declared type
                // (e.g. []str → Vec<String>), and array params are stored as
                // JSON arrays rather than a single Value::from(String).
                let id_param = path_params.first().map(|p| p.name.as_str()).unwrap_or("id");
                let body_fields: Vec<String> = body_params.iter()
                    .map(|p| {
                        let rhs = merged_param_to_value(&p.ty, &p.name);
                        format!("item[\"{}\"] = {}", p.name, rhs)
                    })
                    .collect();
                code.push_str(&format!(
                    "fn {}({}: i32, {}) {{\n    let mut data = API_DATA.lock().unwrap();\n    if let Some(item) = data.iter_mut().find(|n| n[\"id\"].as_i64() == Some({} as i64)) {{\n        {};\n    }}\n}}\n\n",
                    fn_name, id_param,
                    body_params.iter().map(|p| format!("{}: {}", p.name, auto_type_to_rust(&p.ty))).collect::<Vec<_>>().join(", "),
                    id_param,
                    body_fields.join("; ")
                ));
            }
            "PATCH" => {
                // PATCH: partial update — same structure as PUT (id + body fields).
                let id_param = path_params.first().map(|p| p.name.as_str()).unwrap_or("id");
                let body_fields: Vec<String> = body_params.iter()
                    .map(|p| {
                        let rhs = merged_param_to_value(&p.ty, &p.name);
                        format!("item[\"{}\"] = {}", p.name, rhs)
                    })
                    .collect();
                let all_params: Vec<String> = path_params.iter()
                    .map(|p| format!("{}: i32", p.name))
                    .chain(body_params.iter().map(|p| format!("{}: {}", p.name, auto_type_to_rust(&p.ty))))
                    .collect();
                let path_filter: Vec<String> = path_params.iter()
                    .map(|p| format!("n[\"{}\"].as_i64() == Some({} as i64)", p.name, p.name))
                    .collect();
                code.push_str(&format!(
                    "fn {}({}) {{\n    let mut data = API_DATA.lock().unwrap();\n    if let Some(item) = data.iter_mut().find(|n| {}) {{\n        {};\n    }}\n}}\n\n",
                    fn_name,
                    all_params.join(", "),
                    path_filter.join(" && "),
                    body_fields.join("; ")
                ));
            }
            "DELETE" => {
                if path_params.is_empty() {
                    // No path param (e.g. DELETE /api/todos/completed) — operate on all
                    // matching items. Use body params as filter criteria if present.
                    if body_params.is_empty() {
                        // Clear all (unlikely but safe).
                        code.push_str(&format!(
                            "fn {}() {{\n    let mut data = API_DATA.lock().unwrap();\n    data.clear();\n}}\n\n",
                            fn_name
                        ));
                    } else {
                        // Retain items NOT matching the body filter (e.g. done != true).
                        let retain_cond: Vec<String> = body_params.iter()
                            .map(|p| format!("n[\"{}\"] != {}", p.name, merged_param_to_value(&p.ty, &p.name)))
                            .collect();
                        let params_sig: Vec<String> = body_params.iter()
                            .map(|p| format!("{}: {}", p.name, auto_type_to_rust(&p.ty)))
                            .collect();
                        code.push_str(&format!(
                            "fn {}({}) {{\n    let mut data = API_DATA.lock().unwrap();\n    data.retain(|n| {});\n}}\n\n",
                            fn_name,
                            params_sig.join(", "),
                            retain_cond.join(" && ")
                        ));
                    }
                } else {
                    let id_param = path_params.first().map(|p| p.name.as_str()).unwrap_or("id");
                    code.push_str(&format!(
                        "fn {}({}: i32) {{\n    let mut data = API_DATA.lock().unwrap();\n    data.retain(|n| n[\"id\"].as_i64() != Some({} as i64));\n}}\n\n",
                        fn_name, id_param, id_param
                    ));
                }
            }
            _ => {
                code.push_str(&format!("fn {}() {{}}\n\n", fn_name));
            }
        }
    }

    code
}

/// Plan 349 step 2 / Plan 388 W2/W3: Generate multipart upload + download
/// (resume + progress) functions for a2r. Mirrors the VM natives
/// `http.upload` / `http.download` / `http.download_resume` /
/// `http.download_with_progress` and the chainable `RequestBuilder.multipart_*`.
fn generate_http_utility_functions() -> String {
    r#"// Plan 349/388: File upload (multipart) + download utilities (a2r)

fn upload_file(url: &str, file_path: &str) -> serde_json::Value {
    let url = url.to_string();
    let file_path = file_path.to_string();
    std::thread::spawn(move || {
        let form = match reqwest::blocking::multipart::Form::new()
            .file("file", &file_path) { Ok(f) => f, Err(_) => return serde_json::Value::Null };
        let resp = match _http_client()
            .post(&url)
            .multipart(form)
            .send() { Ok(r) => r, Err(_) => return serde_json::Value::Null };
        let text = match resp.text() { Ok(t) => t, Err(_) => return serde_json::Value::Null };
        serde_json::from_str(&text).unwrap_or(serde_json::Value::Null)
    }).join().unwrap_or(serde_json::Value::Null)
}

fn upload_file_with_fields(url: &str, file_path: &str, fields: &serde_json::Value) -> serde_json::Value {
    let url = url.to_string();
    let file_path = file_path.to_string();
    let fields = fields.clone();
    std::thread::spawn(move || {
        let mut form = reqwest::blocking::multipart::Form::new();
        if let Some(obj) = fields.as_object() {
            for (k, v) in obj {
                if let Some(s) = v.as_str() {
                    form = form.text(k.clone(), s.to_string());
                }
            }
        }
        if let Ok(part) = reqwest::blocking::multipart::Part::file(&file_path) {
            form = form.part("file", part);
        }
        let resp = match _http_client()
            .post(&url)
            .multipart(form)
            .send() { Ok(r) => r, Err(_) => return serde_json::Value::Null };
        let text = match resp.text() { Ok(t) => t, Err(_) => return serde_json::Value::Null };
        serde_json::from_str(&text).unwrap_or(serde_json::Value::Null)
    }).join().unwrap_or(serde_json::Value::Null)
}

// Plan 388 W2: chainable multipart builder — mirrors the VM's
// RequestBuilder.multipart_file(field, path) / multipart_text(field, value).
// Usage: multipart_form().text("title", "x").file("upload", "a.bin").send(url)
fn multipart_form() -> MultiPart {
    MultiPart { fields: Vec::new(), files: Vec::new() }
}

struct MultiPart {
    fields: Vec<(String, String)>,
    files: Vec<(String, String)>,
}

impl MultiPart {
    fn text(mut self, field: &str, value: &str) -> Self {
        self.fields.push((field.to_string(), value.to_string()));
        self
    }

    fn file(mut self, field: &str, path: &str) -> Self {
        self.files.push((field.to_string(), path.to_string()));
        self
    }

    fn send(self, url: &str) -> serde_json::Value {
        let url = url.to_string();
        let fields = self.fields;
        let files = self.files;
        std::thread::spawn(move || {
            let mut form = reqwest::blocking::multipart::Form::new();
            for (k, v) in fields {
                form = form.text(k, v);
            }
            for (k, path) in files {
                match reqwest::blocking::multipart::Part::file(&path) {
                    Ok(part) => form = form.part(k, part),
                    Err(_) => return serde_json::Value::Null,
                }
            }
            let resp = match _http_client()
                .post(&url)
                .multipart(form)
                .send() { Ok(r) => r, Err(_) => return serde_json::Value::Null };
            let text = match resp.text() { Ok(t) => t, Err(_) => return serde_json::Value::Null };
            serde_json::from_str(&text).unwrap_or(serde_json::Value::Null)
        }).join().unwrap_or(serde_json::Value::Null)
    }
}

fn download_file(url: &str, file_path: &str) -> bool {
    let url = url.to_string();
    let file_path = file_path.to_string();
    std::thread::spawn(move || {
        let resp = match _http_client().get(&url).send() { Ok(r) => r, Err(_) => return false };
        use std::io::Write;
        let mut file = match std::fs::File::create(&file_path) { Ok(f) => f, Err(_) => return false };
        match resp.bytes() {
            Ok(b) => file.write_all(&b).is_ok(),
            Err(_) => false,
        }
    }).join().unwrap_or(false)
}

fn download_file_resume(url: &str, file_path: &str, offset: u64) -> bool {
    let url = url.to_string();
    let file_path = file_path.to_string();
    std::thread::spawn(move || {
        let range = format!("bytes={}-", offset);
        let resp = match _http_client()
            .get(&url).header("Range", &range).send() { Ok(r) => r, Err(_) => return false };
        use std::io::Write;
        let mut file = match std::fs::OpenOptions::new().append(true).open(&file_path) {
            Ok(f) => f, Err(_) => return false
        };
        match resp.bytes() {
            Ok(b) => file.write_all(&b).is_ok(),
            Err(_) => false,
        }
    }).join().unwrap_or(false)
}

// Plan 388 W3: non-blocking download with progress events — mirrors the VM's
// http.download_with_progress iterator. Returns an mpsc::Receiver the caller
// polls; each event is {"done": bool, "written": u64, "total": u64, ...}.
fn download_with_progress(url: &str, file_path: &str) -> std::sync::mpsc::Receiver<serde_json::Value> {
    let (tx, rx) = std::sync::mpsc::channel::<serde_json::Value>();
    let url = url.to_string();
    let file_path = file_path.to_string();
    std::thread::spawn(move || {
        let mut resp = match _http_client().get(&url).send() {
            Ok(r) => r,
            Err(_) => { let _ = tx.send(serde_json::json!({"done": true, "error": "request failed"})); return; }
        };
        let total = resp.content_length().unwrap_or(0);
        let file = match std::fs::File::create(&file_path) {
            Ok(f) => f,
            Err(_) => { let _ = tx.send(serde_json::json!({"done": true, "error": "open file"})); return; }
        };
        struct ProgressWriter {
            inner: std::fs::File,
            tx: std::sync::mpsc::Sender<serde_json::Value>,
            total: u64,
            written: u64,
        }
        impl std::io::Write for ProgressWriter {
            fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
                let n = self.inner.write(buf)?;
                self.written += n as u64;
                let _ = self.tx.send(serde_json::json!({
                    "done": false, "written": self.written, "total": self.total
                }));
                Ok(n)
            }
            fn flush(&mut self) -> std::io::Result<()> { self.inner.flush() }
        }
        let mut writer = ProgressWriter { inner: file, tx: tx.clone(), total, written: 0 };
        let ok = resp.copy_to(&mut writer).is_ok();
        let _ = tx.send(serde_json::json!({
            "done": true, "written": writer.written, "total": total, "ok": ok
        }));
    });
    rx
}

"#.to_string()
}

/// Plan 350 step 4 / Plan 388 W4: Generate WebSocket client functions for a2r
/// (ws_connect / ws_send / ws_on_message / ws_close — mirrors the VM's
/// `ws.connect/send/on_message/close`). Incoming text/binary frames are queued
/// per connection and drained by `ws_on_message` (non-blocking, UI-pollable).
fn generate_ws_functions() -> String {
    r#"// Plan 350/388: WebSocket client (a2r)
use std::sync::Mutex;
use std::collections::HashMap;

lazy_static::lazy_static! {
    static ref WS_CONNS: Mutex<HashMap<i32, WsConn>> = Mutex::new(HashMap::new());
    static ref WS_NEXT_ID: std::sync::atomic::AtomicI32 = std::sync::atomic::AtomicI32::new(1);
}

struct WsConn {
    sender: Option<std::sync::mpsc::Sender<String>>,
    receiver: std::sync::mpsc::Receiver<String>,
    close: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

fn ws_connect(url: &str) -> i32 {
    let id = WS_NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let (tx, rx) = std::sync::mpsc::channel::<String>();
    let (msg_tx, msg_rx) = std::sync::mpsc::channel::<String>();
    let close = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let close_reader = close.clone();
    let url = url.to_string();

    std::thread::spawn(move || {
        use tungstenite::Message;
        let (mut socket, _) = match tungstenite::connect(&url) {
            Ok(pair) => pair,
            Err(_) => return,
        };
        // Short read timeout so the loop can service outgoing messages while
        // waiting for incoming frames — a plain blocking read() would deadlock
        // an echo conversation until the peer speaks first (the outgoing
        // channel would never be polled). Applied to the underlying TcpStream
        // for plain and native-TLS (wss://) streams; the `_` arm covers
        // rustls-enabled builds (MaybeTlsStream is #[non_exhaustive]; the
        // Rustls variant is feature-gated and only exists with rustls-tls).
        use tungstenite::stream::MaybeTlsStream;
        let timeout = Some(std::time::Duration::from_millis(50));
        match socket.get_ref() {
            MaybeTlsStream::Plain(tcp) => { let _ = tcp.set_read_timeout(timeout); }
            MaybeTlsStream::NativeTls(tls) => { let _ = tls.get_ref().set_read_timeout(timeout); }
            _ => {}
        }
        loop {
            if close_reader.load(std::sync::atomic::Ordering::SeqCst) { break; }
            // Check for outgoing messages (non-blocking).
            if let Ok(msg) = rx.try_recv() {
                if socket.send(Message::Text(msg.into())).is_err() { break; }
            }
            match socket.read() {
                // Deliver incoming frames to the ws_on_message queue.
                Ok(Message::Text(t)) => { let _ = msg_tx.send(t); }
                Ok(Message::Binary(b)) => { let _ = msg_tx.send(String::from_utf8_lossy(&b).into_owned()); }
                Ok(Message::Close(_)) => break,
                // Read timeout / would-block: no frame ready yet — not a
                // connection error, keep servicing outgoing + waiting.
                Err(e) => {
                    use tungstenite::Error;
                    let is_timeout = matches!(&e, Error::Io(ioe)
                        if ioe.kind() == std::io::ErrorKind::WouldBlock
                        || ioe.kind() == std::io::ErrorKind::TimedOut);
                    if !is_timeout { break; }
                }
                _ => {}
            }
        }
        // Dropping `socket` closes the TCP/WS connection (ws_close relies on this).
    });

    WS_CONNS.lock().unwrap().insert(id, WsConn { sender: Some(tx), receiver: msg_rx, close });
    id
}

fn ws_send(handle: i32, message: &str) -> bool {
    WS_CONNS.lock().unwrap()
        .get(&handle)
        .and_then(|conn| conn.sender.as_ref())
        .and_then(|tx| tx.send(message.to_string()).ok())
        .is_some()
}

fn ws_on_message(handle: i32) -> Vec<String> {
    let mut out = Vec::new();
    if let Some(conn) = WS_CONNS.lock().unwrap().get(&handle) {
        while let Ok(m) = conn.receiver.try_recv() {
            out.push(m);
        }
    }
    out
}

fn ws_close(handle: i32) {
    if let Some(conn) = WS_CONNS.lock().unwrap().get(&handle) {
        // Flag the reader thread to exit (dropping the socket → connection
        // actually closes), then sever the send path.
        conn.close.store(true, std::sync::atomic::Ordering::SeqCst);
    }
    WS_CONNS.lock().unwrap().remove(&handle);
}

"#.to_string()
}

/// Wrap generated components in a main() function with ICED/GPUI backend selection.
fn wrap_example(project_name: &str, components: &str) -> String {
    let main_widget = extract_main_widget(components);
    let main_msg = format!("{}Msg", main_widget);

    // Strip duplicate imports — RustGenerator already emits them
    let cleaned = components.trim()
        .replace("use auto_lang::ui::{Component, View};\n", "")
        .replace("use auto_lang::ui::{Component, View};", "");

    // Detect async init: look for __InitLoaded variant in generated code
    let async_init_func = extract_init_api_func(cleaned.trim());

    let iced_entry = if let Some(ref func_name) = async_init_func {
        // Async init: use run_app_with_task_devtools with boot task that loads
        // data in background (Plan 311 P2-A: DevTools-wired counterpart of
        // run_app_with_task, so F12 works for init-API apps like 015-notes).
        // The async {} wrapper ensures spawn_blocking is only called when Iced's Tokio
        // runtime polls the future — NOT eagerly in main() before the runtime starts.
        format!(
            r#"println!("Running with Iced backend");
        let __init = std::cell::RefCell::new(Some(
            iced::Task::perform(
                async {{ tokio::task::spawn_blocking(|| {func_name}()).await.unwrap_or_default() }},
                |r| {main_msg}::__InitLoaded(r)
            )
        ));
        return auto_lang::ui::iced::run_app_with_task_devtools(move || {{
            let task = __init.borrow_mut().take().unwrap_or_else(iced::Task::none);
            ({main_widget}::default(), task)
        }});"#,
            func_name = func_name,
            main_msg = main_msg,
            main_widget = main_widget,
        )
    } else {
        // No async init: use standard run_app. Wrapped in run_app_devtools so
        // F12 opens the rust-mode DevTools inspector (Plan 311).
        format!(
            r#"println!("Running with Iced backend");
        return auto_lang::ui::iced::run_app_devtools::<{main_widget}>();"#,
            main_widget = main_widget,
        )
    };

    format!(
        r#"// Auto-generated from Auto language by a2rust-ui

use auto_lang::ui::{{Component, View}};

{cleaned}

fn main() -> auto_lang::ui::AppResult<()> {{
    #[cfg(feature = "ui-iced")]
    {{
        {iced_entry}
    }}
    #[cfg(feature = "ui-gpui")]
    {{
        println!("Running with GPUI backend");
        return auto_lang::ui::gpui::run_app::<{main_widget}>("{project_name}");
    }}
    #[cfg(not(any(feature = "ui-iced", feature = "ui-gpui")))]
    {{
        Err("No backend enabled! Use --features ui-iced or ui-gpui".into())
    }}
}}
"#,
        cleaned = cleaned.trim(),
        iced_entry = iced_entry,
        main_widget = main_widget,
        project_name = to_snake_case(project_name),
    )
}

/// Extract the main widget name from generated components.
/// Looks for "App" struct first, then falls back to the first `pub struct` found.
fn extract_main_widget(components: &str) -> String {
    // Look for "pub struct App"
    for line in components.lines() {
        let trimmed = line.trim();
        if trimmed == "pub struct App {" {
            return "App".to_string();
        }
    }

    // Fallback: find first pub struct
    for line in components.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("pub struct ") {
            if let Some(name) = rest.split_whitespace().next() {
                // Remove trailing brace if present
                let name = name.trim_end_matches('{').trim();
                return name.to_string();
            }
        }
    }

    // Last resort
    "App".to_string()
}

/// Detect async init by looking for `__InitLoaded` in the generated code.
/// If found, extract the API function name (e.g., "list_notes") that should be
/// called in the background boot task.
fn extract_init_api_func(components: &str) -> Option<String> {
    // Only look for async init if __InitLoaded variant exists
    if !components.contains("__InitLoaded") {
        return None;
    }

    // Find the API function to call: look for `fn list_*()` or `fn get_*()` definitions.
    // These are the GET endpoints that load data for Init.
    for line in components.lines() {
        let trimmed = line.trim();
        // Match patterns like: fn list_notes() -> Vec<...> {
        if let Some(rest) = trimmed.strip_prefix("fn ") {
            if let Some(paren_pos) = rest.find('(') {
                let func_name = &rest[..paren_pos];
                let lower = func_name.to_lowercase();
                if lower.starts_with("list_") || lower.starts_with("get_") {
                    return Some(func_name.to_string());
                }
            }
        }
    }
    None
}

/// Collect all .at files in a directory (non-recursive).
fn collect_at_files(dir: &Path) -> AutoResult<Vec<PathBuf>> {
    let mut files = Vec::new();

    if !dir.exists() {
        return Ok(files);
    }

    for entry in fs::read_dir(dir).map_err(|e| format!("Failed to read dir: {}", e))? {
        let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        let path = entry.path();

        if path.extension().map(|e| e == "at").unwrap_or(false) {
            let file_name = path.file_name().unwrap_or_default().to_string_lossy();
            // Skip pac.at (project config)
            if file_name == "pac.at" {
                continue;
            }
            files.push(path);
        }
    }

    files.sort();
    Ok(files)
}

/// Parse project name from pac.at file.
fn parse_pac_name(pac_path: &Path) -> Option<String> {
    let content = fs::read_to_string(pac_path).ok()?;
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("name:") {
            if let Some(colon_pos) = line.find(':') {
                let value = line[colon_pos + 1..].trim();
                let value = value.trim_matches('"').trim_matches('\'');
                let value = value.trim_end_matches(',');
                if !value.is_empty() {
                    return Some(value.to_string());
                }
            }
        }
    }
    None
}

/// Convert CamelCase to snake_case.
fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 {
                result.push('_');
            }
            result.push(c.to_ascii_lowercase());
        } else {
            result.push(c);
        }
    }
    result
}

/// Generate Cargo.toml content for the Rust UI project (workspace member version).
///
/// No `[workspace]` section — this project is a member of the shared workspace
/// at `examples/rust-workspace/`. Dependencies use `workspace = true` to inherit
/// from the workspace-level `[workspace.dependencies]`.
fn generate_cargo_toml(project_name: &str, _project_dir: &Path) -> String {
    let snake_name = to_snake_case(project_name);

    format!(
        r#"[package]
name = "{snake_name}"
version = "0.1.0"
edition = "2021"

[features]
ui-gpui = ["auto-lang/ui-gpui"]
ui-iced = ["auto-lang/ui-iced"]
default = ["ui-iced", "auto-lang/default"]

[dependencies]
auto-lang.workspace = true
serde_json.workspace = true
reqwest = {{ version = "0.12", features = ["blocking", "json", "multipart", "cookies", "gzip", "brotli"] }}
tungstenite = {{ version = "0.24", features = ["native-tls"] }}
lazy_static = "1"
tokio.workspace = true
iced.workspace = true
"#
    )
}

/// Write `.cargo/config.toml` with shared target-dir pointing to workspace root's target/.
pub fn write_shared_cargo_config(project_dir: &Path, gen_subdir: &str) -> std::io::Result<()> {
    let cargo_dir = project_dir.join("gen").join(gen_subdir).join("rust");
    let config_dir = cargo_dir.join(".cargo");
    fs::create_dir_all(&config_dir)?;

    // Compute relative path from cargo_dir back to workspace root's target/
    let target_rel = find_workspace_target_path(&cargo_dir);

    let config = format!(
        "[build]\ntarget-dir = \"{}\"\n",
        target_rel.replace('\\', "/")
    );
    fs::write(config_dir.join("config.toml"), config)
}

/// Find relative path from a generated rust/ dir to the workspace root's target/ directory.
fn find_workspace_target_path(cargo_dir: &Path) -> String {
    // Walk up from cargo_dir to find workspace root (identified by crates/ directory)
    let mut ups = 0usize;
    let mut dir = cargo_dir.to_path_buf();
    for _ in 0..10 {
        if dir.join("crates").exists() {
            // Found workspace root — build relative path: ../../.../target
            let mut rel = (0..ups).map(|_| "..").collect::<Vec<_>>().join("/");
            if !rel.is_empty() {
                rel.push('/');
            }
            rel.push_str("target");
            return rel;
        }
        if !dir.pop() {
            break;
        }
        ups += 1;
    }
    // Fallback: absolute path to auto-lang/target
    let abs = std::env::current_dir()
        .unwrap_or_default()
        .join("target");
    abs.to_string_lossy().to_string().replace('\\', "/")
}

/// Get the shared Rust workspace directory for all generated UI projects.
///
/// Located inside the repo at `examples/rust-workspace/` and excluded from the
/// main workspace via `exclude` in the root `Cargo.toml`. All generated Rust
/// projects become members of this single workspace, enabling cross-project
/// compilation artifact reuse.
pub fn get_rust_workspace_dir() -> PathBuf {
    PathBuf::from("examples/rust-workspace")
}

/// Compute the relative path from the shared workspace dir to auto-lang crate.
fn compute_auto_lang_rel_path(project_dir: &Path) -> String {
    // Walk up from project_dir to find the workspace root (has crates/auto-lang)
    let mut dir = project_dir.to_path_buf();
    for _ in 0..10 {
        if dir.join("crates").join("auto-lang").exists() {
            let auto_lang_abs = dir.join("crates").join("auto-lang");
            let workspace_dir = get_rust_workspace_dir();
            // Compute relative path from workspace_dir to auto_lang_abs
            return compute_relative_path(&workspace_dir, &auto_lang_abs);
        }
        if !dir.pop() {
            break;
        }
    }
    // Fallback
    "../../autostack/auto-lang/crates/auto-lang".to_string()
}

/// Compute relative path from `from` to `to` using only `..` and directory names.
fn compute_relative_path(from: &Path, to: &Path) -> String {
    // Canonicalize both paths for reliable comparison
    let from_abs = std::fs::canonicalize(from)
        .unwrap_or_else(|_| from.to_path_buf());
    let to_abs = std::fs::canonicalize(to)
        .unwrap_or_else(|_| to.to_path_buf());

    let from_parts: Vec<&std::ffi::OsStr> = from_abs.iter().collect();
    let to_parts: Vec<&std::ffi::OsStr> = to_abs.iter().collect();

    // Find common prefix length
    let common = from_parts.iter().zip(to_parts.iter())
        .take_while(|(a, b)| a == b)
        .count();

    // Go up from `from` to the common ancestor
    let ups = from_parts.len() - common;
    let mut result: Vec<String> = (0..ups).map(|_| "..".to_string()).collect();

    // Then descend to `to`
    for part in &to_parts[common..] {
        result.push(part.to_string_lossy().to_string());
    }

    result.join("/").replace('\\', "/")
}

/// Compute the relative path from the shared workspace dir to the auto-lang target/ directory.
fn compute_target_rel_path(project_dir: &Path) -> String {
    let mut dir = project_dir.to_path_buf();
    for _ in 0..10 {
        if dir.join("crates").exists() {
            let target_abs = dir.join("target");
            let workspace_dir = get_rust_workspace_dir();
            return compute_relative_path(&workspace_dir, &target_abs);
        }
        if !dir.pop() {
            break;
        }
    }
    "../../autostack/auto-lang/target".to_string()
}

/// Check whether a directory is a *complete* Cargo crate — i.e. it declares at
/// least one target, so cargo won't reject it with "no targets specified in the
/// manifest" when the workspace is resolved.
///
/// Mirrors cargo's own target discovery: a `src/main.rs` or `src/lib.rs`, or an
/// explicit `[lib]` / `[[bin]]` section in `Cargo.toml`.
///
/// An incomplete member (e.g. a half-generated frontend crate whose `src/` was
/// removed) would otherwise poison the whole workspace: cargo validates *every*
/// member when resolving the workspace manifest, so one bad member fails even an
/// unrelated `cargo run --manifest-path <other-member>`.
fn has_cargo_targets(dir: &Path) -> bool {
    if dir.join("src").join("main.rs").exists() || dir.join("src").join("lib.rs").exists() {
        return true;
    }
    // Fall back to explicit target declarations in Cargo.toml.
    if let Ok(content) = fs::read_to_string(dir.join("Cargo.toml")) {
        // A `[lib]` table or any `[[bin]]` array counts as a target. Match on a
        // line-starting header to avoid catching these strings in comments.
        for line in content.lines() {
            let trimmed = line.trim_start();
            if trimmed.starts_with("[lib]") || trimmed.starts_with("[[bin]]") {
                return true;
            }
        }
    }
    false
}

/// Ensure the shared Rust workspace exists and is configured.
///
/// Creates/updates:
/// - `examples/rust-workspace/Cargo.toml` (virtual manifest with all members)
/// - `examples/rust-workspace/.cargo/config.toml` (target-dir pointing to auto-lang/target/)
///
/// Returns the workspace directory path.
pub fn ensure_shared_workspace(project_dir: &Path) -> PathBuf {
    let ws_dir = get_rust_workspace_dir();
    fs::create_dir_all(&ws_dir).ok();

    let ws_cargo = ws_dir.join("Cargo.toml");

    // Scan existing member directories (each subdirectory with a Cargo.toml).
    // Skip incomplete members (no src/main.rs, src/lib.rs, [lib], or [[bin]]):
    // cargo rejects them with "no targets specified" when resolving the
    // workspace, which would poison unrelated member runs. They get re-included
    // automatically once generate_rust_ui writes their src/.
    let mut members: Vec<String> = Vec::new();
    if let Ok(entries) = fs::read_dir(&ws_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() && path.join("Cargo.toml").exists() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    // Skip .cargo directory
                    if name == ".cargo" {
                        continue;
                    }
                    if !has_cargo_targets(&path) {
                        eprintln!(
                            "⚠ Skipping incomplete workspace member '{}' \
                             (no src/lib.rs, src/main.rs, [lib], or [[bin]] — \
                             re-included once generated)",
                            name
                        );
                        continue;
                    }
                    members.push(name.to_string());
                }
            }
        }
    }
    members.sort();

    let auto_lang_rel = compute_auto_lang_rel_path(project_dir);
    let target_rel = compute_target_rel_path(project_dir);

    let members_toml = members.iter()
        .map(|m| format!("    \"{}\"", m))
        .collect::<Vec<_>>()
        .join(",\n");

    let content = format!(
r#"[workspace]
members = [
{members_toml}
]
resolver = "2"

[workspace.dependencies]
auto-lang = {{ path = "{auto_lang_rel}" }}
serde_json = "1"
tokio = {{ version = "1", features = ["rt"] }}
iced = {{ version = "0.14.0", features = ["tokio", "advanced"] }}
axum = "0.7"
serde = {{ version = "1", features = ["derive"] }}
tower-http = {{ version = "0.5", features = ["cors"] }}
"#
    );

    // Always rewrite to update members list
    let _ = fs::write(&ws_cargo, &content);

    // Write .cargo/config.toml with target-dir
    let config_dir = ws_dir.join(".cargo");
    fs::create_dir_all(&config_dir).ok();
    let config = format!("[build]\ntarget-dir = \"{}\"\n", target_rel.replace('\\', "/"));
    let _ = fs::write(config_dir.join("config.toml"), config);

    ws_dir
}

/// Get the member directory name for a frontend project in the shared workspace.
fn front_member_name(project_dir: &Path) -> String {
    project_dir.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("myapp")
        .to_string()
}

/// Get the member directory name for a backend project in the shared workspace.
pub fn back_member_name(project_dir: &Path) -> String {
    let base = project_dir.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("myapp");
    format!("{}-back", base)
}

/// Run the generated Rust UI project.
/// Start the API backend server if a backend exists in the shared workspace.
/// Returns the child process handle so the caller can clean it up on exit.
pub fn start_api_server(project_dir: &Path) -> Option<std::process::Child> {
    // Backend lives in the shared workspace at examples/rust-workspace/{name}-back/
    let ws_dir = get_rust_workspace_dir();
    let back_name = back_member_name(project_dir);
    let api_backend_dir = ws_dir.join(&back_name);
    if !api_backend_dir.join("Cargo.toml").exists() {
        return None;
    }

    println!();
    println!("{}", "▶ Starting API backend server (Rust axum)...".bright_cyan());

    // Plan 354: Kill any process occupying the backend port before starting.
    let port = crate::util::http_port();
    crate::util::kill_process_on_port(port);

    let cargo_toml = api_backend_dir.join("Cargo.toml");

    // Plan 328: Sanitize Cargo package name — cargo rejects names starting
    // with a digit (e.g. 015-notes-back). Fix in-place if stale Cargo.toml
    // has an unsanitized name.
    if let Ok(content) = std::fs::read_to_string(&cargo_toml) {
        for line in content.lines() {
            if let Some(rest) = line.strip_prefix("name = \"") {
                if let Some(name) = rest.strip_suffix("\"") {
                    if name.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false) {
                        let fixed = content.replace(
                            &format!("name = \"{}\"", name),
                            &format!("name = \"app-{}\"", name),
                        );
                        let _ = std::fs::write(&cargo_toml, fixed);
                        println!("  {} Fixed package name: {} → app-{}", "⚠".bright_yellow(), name, name);
                    }
                    break;
                }
            }
        }
    }

    println!("  cargo run --manifest-path {}", cargo_toml.display());
    let api_server = std::process::Command::new("cargo")
        .args(["run", "--manifest-path", cargo_toml.to_str().unwrap_or(".")])
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .spawn();

    match api_server {
        Ok(child) => {
            println!("  {} API server starting (PID: {})...", "✓".bright_green(), child.id());

            // Wait for the server to become ready by polling the port
            println!("  Waiting for API server to be ready...");
            let max_wait = std::time::Duration::from_secs(60);
            let start = std::time::Instant::now();
            let mut ready = false;

            // AUTO_HTTP_PORT (default 8080) — must match the backend's bind address.
            let port = crate::util::http_port();
            let probe_addr: std::net::SocketAddr = format!("127.0.0.1:{}", port).parse().unwrap();

            while start.elapsed() < max_wait {
                std::thread::sleep(std::time::Duration::from_millis(500));
                match std::net::TcpStream::connect_timeout(
                    &probe_addr,
                    std::time::Duration::from_secs(1),
                ) {
                    Ok(_) => {
                        ready = true;
                        break;
                    }
                    Err(_) => continue,
                }
            }

            if ready {
                println!("  {} API server is ready on http://127.0.0.1:{}", "✓".bright_green(), port);
            } else {
                println!("  {} API server did not respond within {}s, continuing anyway...",
                    "⚠".bright_yellow(), max_wait.as_secs());
            }

            Some(child)
        }
        Err(e) => {
            println!("  {} Failed to start API server: {}", "⚠".bright_yellow(), e);
            println!("  Continuing without backend...");
            None
        }
    }
}

/// Stop an API server child process (if running).
pub fn stop_api_server(child: &mut Option<std::process::Child>) {
    if let Some(c) = child {
        let _ = c.kill();
        println!("  {} API server (Rust) stopped", "✓".bright_green());
    }
}

/// Plan 340 fix: Start the AutoVM HTTP server (not the a2r Rust backend) for
/// VM+VM split mode. Spawns a background thread running
/// `auto_lang::run_file(api.at)`, which detects #[api] routes and enters
/// `serve_async`. Polls the port until ready (or timeout). The thread runs
/// for the process lifetime (cleaned up when the process exits).
///
/// Returns true if the server became ready, false on timeout / missing api.at.
pub fn start_vm_server(project_dir: &Path) -> bool {
    let api_path = project_dir.join("src").join("back").join("api.at");
    if !api_path.exists() {
        eprintln!("  {} VM split mode requires src/back/api.at (not found)", "⚠".bright_yellow());
        return false;
    }

    let port = crate::util::http_port();
    println!();
    println!(
        "  {} Starting AutoVM HTTP server (VM backend, port {})...",
        "▶".bright_cyan(), port
    );
    println!("    api.at: {}", api_path.display());

    let api_str = api_path.to_string_lossy().to_string();
    // Spawn the VM server on a large-stack background thread (the flattened
    // api.at + db.at + types.at exceeds the 1MB default during parse/codegen).
    std::thread::Builder::new()
        .stack_size(32 * 1024 * 1024)
        .name("auto-vm-http-server".into())
        .spawn(move || {
            if let Err(e) = auto_lang::run_file(&api_str) {
                eprintln!("[AutoVM server] error: {}", e);
            }
        })
        .expect("spawn VM server thread");

    // Wait for the server to become ready by polling the port.
    println!("    Waiting for AutoVM server to be ready...");
    let max_wait = std::time::Duration::from_secs(60);
    let start = std::time::Instant::now();
    let probe_addr: std::net::SocketAddr = format!("127.0.0.1:{}", port).parse().unwrap();
    while start.elapsed() < max_wait {
        std::thread::sleep(std::time::Duration::from_millis(500));
        if std::net::TcpStream::connect_timeout(&probe_addr, std::time::Duration::from_secs(1)).is_ok() {
            println!(
                "  {} AutoVM server ready on http://127.0.0.1:{}",
                "✓".bright_green(), port
            );
            return true;
        }
    }
    println!(
        "  {} AutoVM server did not respond within {}s, continuing anyway...",
        "⚠".bright_yellow(), max_wait.as_secs()
    );
    false
}

pub fn run_rust_ui(project_dir: &Path, args: Vec<String>) -> AutoResult<()> {
    // Rust project now lives in the shared workspace at examples/rust-workspace/{name}/
    let ws_dir = get_rust_workspace_dir();
    let member_name = front_member_name(project_dir);
    let rust_dir = ws_dir.join(&member_name);
    let (full, code) = needs_regeneration(project_dir, &rust_dir);

    if full {
        println!("{}", "Generating Rust UI project...".bright_cyan());
        generate_rust_ui(project_dir, None, false)?;
    } else if code {
        println!("{}", "Regenerating Rust UI code (source changed)...".bright_cyan());
        regenerate_code_only(project_dir, &rust_dir)?;
    }

    // Plan 347: In merged mode, skip the backend HTTP server — API functions
    // are in-process (generated as direct-call code in main.rs).
    let merged_mode = std::env::var("AUTO_VM_MERGE").as_deref() != Ok("0");
    let mut _api_child = if merged_mode {
        println!();
        println!(
            "  {} rust+rust merged mode: backend runs in-process (no HTTP server)",
            "✓".bright_green()
        );
        None
    } else {
        start_api_server(project_dir)
    };

    println!(
        "{}",
        if merged_mode {
            "Running Rust UI app (backend: rust-ui, merged in-process)".bright_cyan()
        } else {
            "Running Rust UI app (backend: rust-ui, split over HTTP)".bright_cyan()
        }
    );

    // Set CWD to src/front/ so local assets (e.g. avatar.png) can be found
    // by the Iced renderer's load_image_bytes(). The cargo subprocess uses
    // --manifest-path instead of current_dir so it can find Cargo.toml, but
    // the final binary (profile-card.exe) inherits this CWD for asset resolution.
    //
    // IMPORTANT: resolve the manifest to an ABSOLUTE path BEFORE changing CWD.
    // `rust_dir` is relative to the repo root (get_rust_workspace_dir returns
    // "examples/rust-workspace"), so after set_current_dir(front_dir) the
    // relative path no longer resolves and `cargo run --manifest-path` fails
    // with "manifest path does not exist".
    let cargo_toml = {
        let rel = rust_dir.join("Cargo.toml");
        // Prefer canonicalize (resolves symlinks/relative parts); fall back to
        // anchoring the relative path to the pre-CWD-change current directory.
        std::fs::canonicalize(&rel).unwrap_or_else(|_| {
            let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
            cwd.join(&rel)
        })
    };

    let front_dir = project_dir.join("src").join("front");
    let original_dir = std::env::current_dir().ok();
    let _ = std::env::set_current_dir(&front_dir);

    let mut cmd = std::process::Command::new("cargo");
    cmd.args(["run", "--manifest-path", cargo_toml.to_str().unwrap_or(".")]);
    for arg in &args {
        cmd.arg(arg);
    }

    let status = cmd.status()?;

    // Restore original CWD
    if let Some(dir) = original_dir {
        let _ = std::env::set_current_dir(dir);
    }

    stop_api_server(&mut _api_child);

    if !status.success() {
        return Err(format!("Cargo run failed with status: {}", status).into());
    }

    Ok(())
}

/// Run the UI via the AutoLang interpreter (--render=vm mode).
/// Starts the same API backend server as --render=rust, but runs
/// the frontend through the in-process interpreter instead of
/// transpiling to Rust.
pub fn run_vm_ui(project_dir: &Path, _args: Vec<String>) -> AutoResult<()> {
    // Plan 334: vm+vm 同进程合并。前端 widget VM（经 Plan 333）直接链接后端
    // 函数访问数据（list_notes → db.all_notes → notes 全局），不需要独立的 Axum
    // HTTP 后端进程。跳过 start_api_server 可消除冗余 cargo 编译、端口占用、
    // 启动等待。
    //
    // Plan 340: --no-merge（AUTO_VM_MERGE=0）切换到分离模式：启动独立后端 HTTP
    // 进程，前端 VM 通过 HTTP 调用后端 API（codegen 把 #[api] 调用改写成 HTTP）。
    // AUTO_VM_WITH_HTTP=1 是旧开关，等价于分离模式（向后兼容）。
    //
    // Plan 340 fix: 分离模式启动独立后端 HTTP 进程。后端可以是 AutoVM server
    // （VM+VM split）或 a2r 转译的 Rust axum server（VM+Rust）。
    // AUTO_BACKEND_IMPL=rust → Rust 后端（start_api_server）
    // AUTO_BACKEND_IMPL=vm 或未设 → AutoVM 后端（start_vm_server）
    let split_mode = std::env::var("AUTO_VM_MERGE").as_deref() == Ok("0")
        || std::env::var("AUTO_VM_WITH_HTTP").as_deref() == Ok("1");
    let backend_impl = std::env::var("AUTO_BACKEND_IMPL").unwrap_or_else(|_| "vm".to_string());
    let mut _api_child = if split_mode {
        if backend_impl == "rust" {
            // VM+Rust split: ensure the Rust axum server is generated, then
            // start it. generate_api("rust") writes Cargo.toml + main.rs +
            // api.rs + types.rs from #[api] annotations (idempotent).
            if let Err(e) = crate::api_gen::generate_api(project_dir, "rust") {
                eprintln!("  {} Failed to generate Rust backend: {}", "⚠".bright_yellow(), e);
            }
            start_api_server(project_dir)
        } else {
            // VM+VM split: AutoVM HTTP server as backend.
            start_vm_server(project_dir);
            None
        }
    } else {
        println!();
        println!(
            "  {} vm+vm merged mode: backend runs in-process (no HTTP server)",
            "✓".bright_green()
        );
        None
    };

    let entry = project_dir.join("src").join("front").join("app.at");
    if !entry.exists() {
        stop_api_server(&mut _api_child);
        return Err(format!("Frontend entry not found: {}", entry.display()).into());
    }

    println!(
        "{}",
        if split_mode {
            format!("Running VM interpreter UI (backend: {}, split over HTTP)", backend_impl).bright_cyan()
        } else {
            "Running VM interpreter UI (backend: vm, merged)".bright_cyan()
        }
    );

    // Change CWD to src/front/ so `use` imports resolve correctly
    let front_dir = project_dir.join("src").join("front");
    let original_dir = std::env::current_dir().ok();
    let _ = std::env::set_current_dir(&front_dir);

    let result = auto_lang::run_file(entry.to_str().unwrap_or("src/front/app.at"));

    // Restore original CWD
    if let Some(dir) = original_dir {
        let _ = std::env::set_current_dir(dir);
    }

    stop_api_server(&mut _api_child);

    match result {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("VM UI error: {}", e).into()),
    }
}

/// Generate a Rust type alias from an Auto TypeDecl.
/// Uses serde_json::Value as the underlying type to maintain compatibility
/// with the dynamic-typing code generator (field access via ["field"]).
fn generate_type_struct(type_decl: &auto_lang::ast::TypeDecl) -> String {
    format!("pub type {} = serde_json::Value;\n", type_decl.name)
}

/// Convert an Auto Type to a Rust type string (standalone, mirrors RustGenerator::auto_type_to_rust).
fn type_to_rust_str(ty: &auto_lang::ast::Type) -> String {
    use auto_lang::ast::Type;
    match ty {
        Type::Int => "i32".to_string(),
        Type::Uint => "u32".to_string(),
        Type::I64 => "i64".to_string(),
        Type::U64 => "u64".to_string(),
        Type::Float => "f32".to_string(),
        Type::Double => "f64".to_string(),
        Type::Bool => "bool".to_string(),
        Type::StrFixed(_) | Type::StrOwned | Type::StrSlice => "String".to_string(),
        Type::Void => "()".to_string(),
        Type::Array(arr) => format!("Vec<{}>", type_to_rust_str(&arr.elem)),
        Type::RuntimeArray(arr) => format!("Vec<{}>", type_to_rust_str(&arr.elem)),
        Type::List(inner) => format!("Vec<{}>", type_to_rust_str(inner)),
        Type::Slice(sl) => format!("Vec<{}>", type_to_rust_str(&sl.elem)),
        Type::Map(k, v) => format!("std::collections::HashMap<{}, {}>", type_to_rust_str(k), type_to_rust_str(v)),
        Type::User(td) => td.name.to_string(),
        Type::Unknown => "serde_json::Value".to_string(),
        _ => "serde_json::Value".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_snake_case() {
        assert_eq!(to_snake_case("MyApp"), "my_app");
        assert_eq!(to_snake_case("HelloWorld"), "hello_world");
        assert_eq!(to_snake_case("App"), "app");
        assert_eq!(to_snake_case("already_snake"), "already_snake");
        assert_eq!(to_snake_case("lowercase"), "lowercase");
    }

    #[test]
    fn test_extract_main_widget_prefers_app() {
        let code = r#"
pub struct Counter {
    pub count: i32,
}

pub struct App {
    pub title: String,
}
"#;
        assert_eq!(extract_main_widget(code), "App");
    }

    #[test]
    fn test_extract_main_widget_fallback_first_struct() {
        let code = r#"
pub struct Counter {
    pub count: i32,
}

pub struct Timer {
    pub seconds: i32,
}
"#;
        assert_eq!(extract_main_widget(code), "Counter");
    }

    #[test]
    fn test_extract_main_widget_empty() {
        let code = "// no structs here";
        assert_eq!(extract_main_widget(code), "App");
    }

    #[test]
    fn test_parse_pac_name() {
        let dir = std::env::temp_dir().join("auto_test_pac");
        fs::create_dir_all(&dir).ok();
        let pac_path = dir.join("pac.at");
        fs::write(&pac_path, r#"name: "TestProject""#).ok();
        assert_eq!(parse_pac_name(&pac_path), Some("TestProject".to_string()));
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_gen_015_notes_rust() {
        // Quick generation test for 015-notes Rust UI code
        let project_dir = std::path::PathBuf::from(
            env!("CARGO_MANIFEST_DIR")
        ).join("..").join("..").join("examples").join("ui").join("015-notes");
        if !project_dir.exists() {
            eprintln!("Skipping: project dir not found at {:?}", project_dir);
            return;
        }
        let result = super::generate_rust_ui(&project_dir, None, false);
        match result {
            Ok(()) => println!("Generation succeeded!"),
            Err(e) => panic!("Generation failed: {}", e),
        }
    }

    #[test]
    fn test_collect_at_files_skips_pac() {
        let dir = std::env::temp_dir().join("auto_test_collect");
        fs::create_dir_all(&dir).ok();
        fs::write(dir.join("app.at"), "").ok();
        fs::write(dir.join("pac.at"), "name: test").ok();
        fs::write(dir.join("other.at"), "").ok();

        let files = collect_at_files(&dir).unwrap();
        let names: Vec<String> = files
            .iter()
            .map(|f| f.file_name().unwrap().to_string_lossy().to_string())
            .collect();

        assert!(names.contains(&"app.at".to_string()));
        assert!(names.contains(&"other.at".to_string()));
        assert!(!names.contains(&"pac.at".to_string()));

        fs::remove_dir_all(&dir).ok();
    }

    // --- Plan 388 W1: API client migrated from ureq to reqwest::blocking + TLS ---

    #[test]
    fn test_w1_http_client_helper_emits_tls_client() {
        // The split-mode helper must build a reqwest::blocking client honouring
        // AUTO_TLS_SKIP_VERIFY / AUTO_TLS_CA_CERT (OnceLock-cached), so endpoint
        // functions can configure TLS (ureq could not).
        let helper = generate_http_client_helper();
        assert!(helper.contains("fn _http_client() -> reqwest::blocking::Client"), "missing _http_client");
        assert!(helper.contains("AUTO_TLS_SKIP_VERIFY"), "missing skip-verify env");
        assert!(helper.contains("AUTO_TLS_CA_CERT"), "missing CA env");
        assert!(helper.contains("danger_accept_invalid_certs"), "missing skip-verify wiring");
        assert!(helper.contains("add_root_certificate"), "missing custom CA wiring");
        assert!(helper.contains("OnceLock"), "client should be built once");
    }

    #[test]
    fn test_w1_get_body_uses_reqwest_client() {
        // GET body must route through _http_client() and reqwest's .send()/.json(),
        // never emit `ureq::`.
        for (ret, expected) in [
            ("serde_json::Value", "_http_client().get("),
            ("Vec<serde_json::Value>", "r.json::<Vec<serde_json::Value>>().ok()"),
            ("Option<serde_json::Value>", "r.json::<serde_json::Value>().ok()"),
        ] {
            let body = generate_get_fn_body("get".into(), "&format!(\"http://x/{}\", id)".into(), false, ret);
            assert!(body.contains(expected), "GET({ret}) missing {expected}: {body}");
            assert!(!body.contains("ureq"), "GET({ret}) still emits ureq: {body}");
        }
        let void = generate_get_fn_body("get".into(), "\"http://x\"".into(), true, "");
        assert!(void.contains("_http_client().get(\"http://x\").send()"), "void GET: {void}");
        assert!(!void.contains("ureq"), "void GET still emits ureq: {void}");
    }

    #[test]
    fn test_w1_write_and_delete_bodies_use_reqwest_client() {
        // POST/PUT fire-and-forget + blocking bodies, and DELETE, must use
        // _http_client() and reqwest .json(&body)/.send().
        let params: Vec<&auto_lang::api::types::ApiParam> = vec![];
        let void = generate_write_fn_body("post".into(), "\"http://x\"".into(), &params, true, "");
        assert!(void.contains("_http_client().post(&url).json(&body).send()"), "void POST: {void}");
        assert!(!void.contains("ureq"), "void POST still emits ureq: {void}");

        let blocking = generate_write_fn_body(
            "post".into(),
            "\"http://x\"".into(),
            &params,
            false,
            "serde_json::Value",
        );
        // Value-returning POST is non-blocking: placeholder + background thread.
        assert!(blocking.contains("_http_client().post(&url).json(&body).send()"), "blocking POST: {blocking}");
        assert!(blocking.contains("local_result"), "blocking POST must return placeholder: {blocking}");
        assert!(!blocking.contains("ureq"), "blocking POST still emits ureq: {blocking}");

        let del = generate_delete_fn_body("\"http://x/1\"".into());
        assert!(del.contains("_http_client().delete(&url).send()"), "DELETE: {del}");
        assert!(!del.contains("ureq"), "DELETE still emits ureq: {del}");
    }

    // --- Plan 388 W2/W3: multipart chain + download progress ---

    #[test]
    fn test_w2_multipart_chainable_builder() {
        let utils = generate_http_utility_functions();
        // Chainable builder mirroring VM RequestBuilder.multipart_file/multipart_text
        assert!(utils.contains("fn multipart_form() -> MultiPart"), "missing multipart_form");
        assert!(utils.contains("struct MultiPart"), "missing MultiPart struct");
        assert!(utils.contains("fn text(mut self, field: &str, value: &str) -> Self"), "missing .text()");
        assert!(utils.contains("fn file(mut self, field: &str, path: &str) -> Self"), "missing .file()");
        assert!(utils.contains("fn send(self, url: &str) -> serde_json::Value"), "missing .send()");
        assert!(utils.contains(".multipart(form)"), "send must attach multipart form");
        // Existing generic uploads kept
        assert!(utils.contains("fn upload_file("), "upload_file must remain");
        assert!(utils.contains("fn upload_file_with_fields("), "upload_file_with_fields must remain");
        // Plan 388 review: all HTTP clients must be TLS-aware (_http_client),
        // not raw reqwest builders that ignore AUTO_TLS_*.
        assert!(!utils.contains("reqwest::blocking::Client::new"), "utils must not build raw clients");
        assert!(!utils.contains("reqwest::blocking::get"), "utils must not use blocking::get");
        assert!(utils.contains("_http_client()"), "utils must use TLS-aware _http_client");
    }

    #[test]
    fn test_w3_download_progress_iterator() {
        let utils = generate_http_utility_functions();
        assert!(utils.contains("fn download_with_progress(url: &str, file_path: &str) -> std::sync::mpsc::Receiver<serde_json::Value>"),
            "missing download_with_progress");
        assert!(utils.contains("fn download_file(url: &str, file_path: &str) -> bool"), "download_file must remain");
        assert!(utils.contains("fn download_file_resume("), "download_file_resume must remain");
        assert!(utils.contains("copy_to"), "progress must stream via copy_to (not buffer-all)");
        assert!(utils.contains("content_length()"), "progress must report total");
        // Plan 388 review: downloads must go through the TLS-aware client too.
        assert!(utils.contains("_http_client().get"), "download must use TLS-aware _http_client");
    }

    // --- Plan 388 W4: WebSocket client ---

    #[test]
    fn test_w4_ws_on_message_delivers_frames() {
        let ws = generate_ws_functions();
        for f in ["ws_connect", "ws_send", "ws_on_message", "ws_close"] {
            assert!(ws.contains(&format!("fn {}(", f)), "missing {f}");
        }
        // Reader thread must forward frames to a per-conn queue consumed by ws_on_message
        assert!(ws.contains("msg_tx.send"), "reader must deliver frames to a queue");
        assert!(ws.contains("receiver: std::sync::mpsc::Receiver<String>"), "WsConn must hold the incoming queue");
        assert!(ws.contains("fn ws_on_message(handle: i32) -> Vec<String>"), "ws_on_message must drain to Vec<String>");
        // No unused Arc import
        assert!(!ws.contains("use std::sync::{Arc, Mutex}"), "Arc import must be gone");
        // Plan 388 review: read timeout must cover the MaybeTlsStream variants
        // that exist in generated projects (plain ws:// + native-TLS wss://,
        // matching the `native-tls` tungstenite feature in the Cargo template) —
        // otherwise the outgoing-channel deadlock persists for TLS connections.
        // The `_` arm satisfies #[non_exhaustive] (and rustls-enabled builds).
        assert!(ws.contains("MaybeTlsStream::Plain(tcp) => { let _ = tcp.set_read_timeout(timeout); }"),
            "plain ws:// must get a read timeout");
        assert!(ws.contains("MaybeTlsStream::NativeTls(tls) => { let _ = tls.get_ref().set_read_timeout(timeout); }"),
            "native-TLS wss:// must get a read timeout");
        assert!(ws.contains("_ => {}"), "match must be exhaustive (non_exhaustive enum)");
        // Plan 388 review: ws_close must actually terminate the reader thread
        // (close flag → socket dropped), not just sever the send path.
        assert!(ws.contains("close_reader.load"), "reader must check the close flag");
        assert!(ws.contains("conn.close.store(true"), "ws_close must set the close flag");
    }

    #[test]
    fn test_w6_helper_emits_cookie_gzip_brotli_env() {
        // Plan 349 步骤 7/8 (W6): the generated _http_client() helper must wire cookie
        // store + gzip + brotli behind env flags, mirroring the VM-side
        // RequestBuilder.cookie_store/gzip/brotli flags. Defaults stay off so
        // existing behavior is unchanged.
        let helper = generate_http_client_helper();
        assert!(helper.contains("AUTO_COOKIE_STORE"), "missing cookie env flag");
        assert!(helper.contains("cookie_store(true)"), "cookie_store not wired");
        assert!(helper.contains("AUTO_GZIP"), "missing gzip env flag");
        assert!(helper.contains(".gzip(true)"), "gzip not wired");
        assert!(helper.contains("AUTO_BROTLI"), "missing brotli env flag");
        assert!(helper.contains(".brotli(true)"), "brotli not wired");
    }

    #[test]
    fn test_w6_cargo_toml_has_compression_cookie_features() {
        // Plan 349 步骤 7/8 (W6): generated Cargo.toml must declare the reqwest features
        // the helper relies on (cookies + gzip + brotli), alongside the
        // pre-existing blocking/json/multipart.
        let toml = generate_cargo_toml("test_proj", std::path::Path::new("/tmp/test"));
        assert!(toml.contains("\"cookies\""), "missing cookies feature: [{}]", toml);
        assert!(toml.contains("\"gzip\""), "missing gzip feature: [{}]", toml);
        assert!(toml.contains("\"brotli\""), "missing brotli feature: [{}]", toml);
    }
}

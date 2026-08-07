//! API Code Generation Integration
//!
//! Plan 130: Integrate API code generation with build workflow
//!
//! This module bridges the gap between:
//! - auto-lang/src/api: API extraction and code generation
//! - auto-man/src/tauri: Tauri project generation
//! - auto-man/src/vue: Vue project generation
//!
//! ## Workflow
//!
//! 1. Parse `back/api.at` to extract `#[api]` function definitions
//! 2. Generate backend code:
//!    - Tauri mode: `src-tauri/src/commands.rs` with `#[tauri::command]`
//!    - Vue mode: Generate Axum routes for HTTP backend
//! 3. Generate frontend code:
//!    - `src/api/types.ts`: TypeScript interfaces
//!    - `src/api/client.ts`: API client (IPC or HTTP)

use std::path::Path;

use crate::AutoResult;

use auto_lang::api::{ApiModule, ApiType, ApiField, ApiEndpoint, ApiParam, ApiAttrs};

/// Generate API code for the project
///
/// This is the main entry point for API code generation.
/// It reads the backend API definitions and generates:
/// - Backend: Tauri commands or Axum routes
/// - Frontend: TypeScript types and API client
pub fn generate_api(root_dir: &Path, backend: &str) -> AutoResult<()> {
    // Try common backend directory layouts: src/back/ or back/
    let back_dir = if root_dir.join("src").join("back").exists() {
        root_dir.join("src").join("back")
    } else if root_dir.join("back").exists() {
        root_dir.join("back")
    } else {
        // No backend directory found, skip generation
        return Ok(());
    };

    // Check if back/api.at exists
    let api_file = back_dir.join("api.at");
    if !api_file.exists() {
        // No API file, skip generation
        return Ok(());
    }

    // Read API file
    let api_content = std::fs::read_to_string(&api_file)
        .map_err(|e| format!("Failed to read {}: {}", api_file.display(), e))?;

    // Try full parsing first, fall back to lenient extraction
    let api_module = match try_full_parse(&api_content) {
        Some(module) => module,
        None => {
            // Lenient extraction for files with module references like `use db`
            match extract_api_lenient(&api_content) {
                Some(m) => {
                    println!("  ℹ Using lenient API extraction (module references skipped)");
                    m
                }
                None => {
                    println!("  ⚠ Could not extract API definitions");
                    return Ok(());
                }
            }
        }
    };

    // Check if any endpoints or types were extracted
    if api_module.endpoints.is_empty() && api_module.types.is_empty() {
        println!("  ⚠ No API endpoints or types found");
        return Ok(());
    }

    // Generate code based on backend
    match backend {
        "tauri" => {
            generate_tauri_api(&api_module, root_dir)?;
        }
        "vue" => {
            generate_vue_api(&api_module, root_dir)?;
        }
        "rust" => {
            // Plan 345: VM+Rust mode — generate Rust axum server from #[api].
            generate_rust_server(&api_module, root_dir)?;
        }
        _ => {
            // No API generation for other backends
        }
    }

    Ok(())
}

/// Try to parse API file with full AST parsing
pub fn try_full_parse(api_content: &str) -> Option<ApiModule> {
    use auto_lang::api::ApiExtractor;

    let mut parser = auto_lang::Parser::from(api_content);
    let ast = parser.parse().ok()?;

    let extractor = ApiExtractor::new();
    let module = extractor.extract("api", &ast.stmts);

    // Only return if we found endpoints
    if module.endpoints.is_empty() && module.types.is_empty() {
        None
    } else {
        Some(module)
    }
}

/// Generate Tauri API code
fn generate_tauri_api(api_module: &auto_lang::api::ApiModule, root_dir: &Path) -> AutoResult<()> {
    use auto_lang::api::Target;

    let vue_dir = root_dir.join("gen").join("front").join("vue");
    let tauri_src_dir = vue_dir.join("src-tauri").join("src");

    // Ensure directories exist
    std::fs::create_dir_all(&tauri_src_dir)
        .map_err(|e| format!("Failed to create directory: {}", e))?;

    // Generate Tauri commands
    let tauri_gen = Target::Tauri.generator();
    let tauri_code = tauri_gen.generate(api_module);
    std::fs::write(tauri_src_dir.join("commands.rs"), &tauri_code)
        .map_err(|e| format!("Failed to write commands.rs: {}", e))?;

    // Generate TypeScript IPC client for Tauri (uses invoke instead of fetch)
    let ts_ipc_code = generate_tauri_ts_client(api_module);

    // Write to src/lib/api.ts so Vue imports resolve correctly
    let lib_dir = vue_dir.join("src").join("lib");
    std::fs::create_dir_all(&lib_dir)
        .map_err(|e| format!("Failed to create lib directory: {}", e))?;
    std::fs::write(lib_dir.join("api.ts"), &ts_ipc_code)
        .map_err(|e| format!("Failed to write src/lib/api.ts: {}", e))?;

    // Also write to src/api/client.ts for backward compatibility
    let api_dir = vue_dir.join("src").join("api");
    std::fs::create_dir_all(&api_dir)
        .map_err(|e| format!("Failed to create api directory: {}", e))?;
    std::fs::write(api_dir.join("client.ts"), &ts_ipc_code)
        .map_err(|e| format!("Failed to write client.ts: {}", e))?;

    println!("  ✓ Generated Tauri commands: src-tauri/src/commands.rs");
    println!("  ✓ Generated TypeScript IPC client: src/lib/api.ts");

    Ok(())
}

/// Generate a Tauri IPC TypeScript client using `invoke`
fn generate_tauri_ts_client(api_module: &auto_lang::api::ApiModule) -> String {
    let mut lines = vec![
        "import { invoke } from '@tauri-apps/api/core';".to_string(),
        "".to_string(),
    ];

    // Type definitions
    for api_type in &api_module.types {
        lines.push(format!("export interface {} {{", api_type.name));
        for field in &api_type.fields {
            let ts_type = auto_type_to_ts(&field.ty);
            let optional = if field.optional { "?" } else { "" };
            lines.push(format!("    {}{}: {};", field.name, optional, ts_type));
        }
        lines.push("}".to_string());
        lines.push("".to_string());
    }

    // IPC functions
    for endpoint in &api_module.endpoints {
        let params_ts: Vec<String> = endpoint.params.iter().map(|p| {
            let ts_type = auto_type_to_ts(&p.ty);
            format!("{}: {}", p.name, ts_type)
        }).collect();

        let return_ts = auto_type_to_ts(&endpoint.return_type);
        let args_str = if params_ts.is_empty() {
            "".to_string()
        } else {
            params_ts.join(", ")
        };

        if params_ts.is_empty() {
            lines.push(format!(
                "export async function {}(): Promise<{}> {{",
                endpoint.fn_name, return_ts
            ));
            lines.push(format!(
                "    return invoke('{}');",
                endpoint.fn_name
            ));
        } else {
            lines.push(format!(
                "export async function {}({}): Promise<{}> {{",
                endpoint.fn_name, args_str, return_ts
            ));
            lines.push(format!(
                "    return invoke('{}', {{ {} }});",
                endpoint.fn_name,
                endpoint.params.iter().map(|p| format!("{}", p.name)).collect::<Vec<_>>().join(", ")
            ));
        }
        lines.push("}".to_string());
        lines.push("".to_string());
    }

    lines.join("\n")
}

/// Convert Auto type to TypeScript type
fn auto_type_to_ts(auto_type: &str) -> String {
    let auto_type = auto_type.trim();
    // Handle prefix ?T (Auto Option syntax: ?Note, ?int)
    if let Some(inner) = auto_type.strip_prefix('?') {
        return format!("{} | null", auto_type_to_ts(inner));
    }
    // Handle suffix T? (alternative Option syntax)
    if auto_type.ends_with('?') {
        let inner = &auto_type[..auto_type.len()-1];
        return format!("{} | null", auto_type_to_ts(inner));
    }
    if auto_type.starts_with("[]") || auto_type.starts_with("List<") {
        let inner = if auto_type.starts_with("[]") {
            &auto_type[2..]
        } else if let Some(close) = auto_type.find('>') {
            &auto_type[5..close]
        } else {
            auto_type
        };
        return format!("{}[]", auto_type_to_ts(inner));
    }
    match auto_type {
        "int" | "i32" | "i64" | "long" | "uint" | "u32" | "u64" | "ulong" => "number".to_string(),
        "float" | "f32" | "double" | "f64" => "number".to_string(),
        "bool" | "boolean" => "boolean".to_string(),
        "str" | "string" | "String" => "string".to_string(),
        "void" | "()" => "void".to_string(),
        _ => auto_type.to_string(),
    }
}

/// Generate Vue + HTTP API code
fn generate_vue_api(api_module: &auto_lang::api::ApiModule, root_dir: &Path) -> AutoResult<()> {
    use auto_lang::api::TypeScriptGenerator;

    // For workspace projects, output to dist/src/lib/
    let dist_dir = root_dir.join("dist");
    let lib_dir = dist_dir.join("src").join("lib");
    std::fs::create_dir_all(&lib_dir)
        .map_err(|e| format!("Failed to create lib directory: {}", e))?;

    // Generate simple TypeScript client
    let ts_gen = TypeScriptGenerator::new();
    let ts_code = ts_gen.generate_simple_client(api_module);

    std::fs::write(lib_dir.join("api.ts"), &ts_code)
        .map_err(|e| format!("Failed to write api.ts: {}", e))?;

    // Also write to vue/src/lib/ for Vue project imports
    let vue_lib_dir = root_dir.join("gen").join("front").join("vue").join("src").join("lib");
    if vue_lib_dir.exists() || root_dir.join("gen").join("front").join("vue").exists() {
        std::fs::create_dir_all(&vue_lib_dir)
            .map_err(|e| format!("Failed to create vue lib directory: {}", e))?;
        std::fs::write(vue_lib_dir.join("api.ts"), &ts_code)
            .map_err(|e| format!("Failed to write vue api.ts: {}", e))?;
    }

    // Write API function names to a manifest file for code generator consumption
    let fn_names: Vec<String> = api_module.endpoints.iter()
        .map(|ep| ep.fn_name.to_lowercase())
        .collect();
    std::fs::write(dist_dir.join(".api_functions"), fn_names.join("\n"))
        .map_err(|e| format!("Failed to write .api_functions: {}", e))?;

    println!("  ✓ Generated TypeScript client: dist/src/lib/api.ts");

    // Generate Rust server if back/ exists
    let back_dir = if root_dir.join("src").join("back").exists() {
        root_dir.join("src").join("back")
    } else if root_dir.join("back").exists() {
        root_dir.join("back")
    } else {
        return Ok(());
    };
    if back_dir.exists() {
        generate_rust_server(api_module, root_dir)?;
    }

    Ok(())
}

/// Generate Rust server code (Axum-based)
/// Plan musk-022 CRUD 智能扩展: transpile db.at to a db.rs module via a2r.
/// Reuses the Tauri-backend precedent (tauri_backend::transpile_at_to_rust).
fn transpile_db_to_rs(content: &str) -> AutoResult<String> {
    use auto_lang::trans::rust::transpile_rust;
    use auto_val::AutoStr;
    let mut sink = transpile_rust(AutoStr::from("db"), content)
        .map_err(|e| format!("Failed to transpile db.at: {}", e))?;
    let rust_code = String::from_utf8(sink.done()?.to_vec())
        .map_err(|e| format!("Invalid UTF-8 in db.rs output: {}", e))?;
    Ok(rust_code)
}

/// Plan musk-022 CRUD 扩展: post-process a2r's db.rs output to fix the known
/// backend-context gaps (a2r transpiles for a generic module; the HTTP backend
/// has specific shape). Fixes applied:
/// - `use crate::api::{T}` → `use crate::types::{T}` (types live in types.rs,
///   not api.rs; a2r maps `use api:` to `crate::api` but the generator emits
///   types separately).
/// These are mechanical, safe rewrites; the deeper a2r issues (List<T>.new
/// wrapping, &[T] lifetimes) are fixed in a2r proper (trans/rust.rs).
fn post_process_db_rs(mut code: String) -> String {
    code = code.replace("use crate::api::", "use crate::types::");
    // Strip `List<T>.new(EXPR)` -> `EXPR` (a2r leaves the wrapper; List=Vec, the
    // array literal is already vec![...]). Bracket-balanced over the .new(...) parens.
    code = strip_collection_new(&code);
    // `fn f() -> &[T] { return *G.lock().unwrap(); }` -> return owned Vec<T> with
    // .clone(): a2r emits a borrowed slice return over a MutexGuard (lifetime error).
    code = fix_borrowed_slice_returns(&code);
    // `*G.lock().unwrap().method(...)` -> `G.lock().unwrap().method(...)`: a2r over-
    // dereferences the guard before a method call (push/insert). Only the method-call
    // form (a `.` following), not assignment targets.
    code = code.replace("*MESSAGES.lock().unwrap().push", "MESSAGES.lock().unwrap().push");
    code = code.replace("*MESSAGES.lock().unwrap().insert", "MESSAGES.lock().unwrap().insert");
    // Param-to-field &str -> String: a2r passes &str fn params into String struct
    // fields without .to_string(). Regex: `<field>: <param>,` where both are the same
    // bare ident and field is a known String field (skip id/bool/time/count).
    code = append_tostring_for_str_fields(&code);
    // id field type widening: backend types.rs uses i64 for int, but a2r emits i32
    // guards. `id: *NEXTID.lock().unwrap()` -> add `as i64` for the id field.
    code = code.replace("id: *NEXTID.lock().unwrap()", "id: *NEXTID.lock().unwrap() as i64");
    code
}

/// For `Type { field: field, ... }` where `field` is a &str param assigned to a
/// String struct field, append `.to_string()`. Scans for `ident: ident,` pairs
/// (no regex backref — regex crate lacks it) where the ident is a str param.
fn append_tostring_for_str_fields(code: &str) -> String {
    use std::collections::HashSet;
    let mut str_params: HashSet<String> = HashSet::new();
    for line in code.lines() {
        let l = line.trim_start();
        if !(l.starts_with("pub fn ") || l.starts_with("fn ")) { continue; }
        if let Some(open) = l.find('(') {
            if let Some(close) = l[open..].find(')') {
                for p in l[open + 1..open + close].split(',') {
                    let tok: Vec<&str> = p.trim().split(|c: char| c == ':' || c.is_whitespace())
                        .filter(|t| !t.is_empty()).collect();
                    if tok.len() >= 2 && (tok[1] == "str" || tok[1] == "&str") {
                        str_params.insert(tok[0].to_string());
                    }
                }
            }
        }
    }
    let skip: HashSet<&str> = ["id", "mine", "done", "pinned", "time", "count", "unread", "active_id"].iter().copied().collect();
    // Scan the whole code; for each word w that is a str param, replace `w: w,` -> `w: w.to_string(),`.
    // Process by iterating over str_params (small set) and doing targeted replace.
    let mut out = code.to_string();
    for name in &str_params {
        if skip.contains(name.as_str()) { continue; }
        let needle = format!("{}: {},", name, name);
        let repl = format!("{}: {}.to_string(),", name, name);
        out = out.replace(&needle, &repl);
    }
    out
}

/// Strip `List<T>.new(...)` / `Array<T>.new(...)` wrappers, leaving the inner expr.
fn strip_collection_new(code: &str) -> String {
    let mut out = String::with_capacity(code.len());
    let bytes = code.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        // Match `List<` or `Array<`
        if (code[i..].starts_with("List<") || code[i..].starts_with("Array<")) {
            // Find the matching `>` for the `<` (no nested <> expected in type position).
            let lt_at = i + 4; // index of '<' (List<) or +5 (Array< handled below)
            let is_array = code[i..].starts_with("Array<");
            let lt_at = if is_array { i + 5 } else { i + 4 };
            if let Some(gt_rel) = code[lt_at..].find('>') {
                let gt_at = lt_at + gt_rel;
                // After '>' expect ".new("
                if code[gt_at+1..].starts_with(".new(") {
                    let paren_open = gt_at + 5;
                    // Bracket-balance over (...) honoring () [] {} "".
                    if let Some(paren_close) = balance_paren(code, paren_open) {
                        // Emit the inner expr (paren_open+1 .. paren_close), skip the wrapper.
                        out.push_str(&code[paren_open+1..paren_close]);
                        i = paren_close + 1;
                        continue;
                    }
                }
            }
        }
        out.push(bytes[i] as char);
        i += 1;
    }
    out
}

/// Find the matching ')' for the '(' at `open`, balancing () [] {} and "".
fn balance_paren(code: &str, open: usize) -> Option<usize> {
    let bytes = code.as_bytes();
    let mut depth = 1i32;
    let mut in_str = false;
    let mut j = open + 1;
    while j < bytes.len() {
        let c = bytes[j] as char;
        if in_str { if c == '"' { in_str = false; } j += 1; continue; }
        match c {
            '"' => in_str = true,
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' => { depth -= 1; if depth == 0 { return Some(j); } }
            _ => {}
        }
        j += 1;
    }
    None
}

/// `pub fn f() -> &[T] { return *G.lock().unwrap(); }` -> `pub fn f() -> Vec<T>
/// { return G.lock().unwrap().clone(); }`. The borrowed-slice-over-guard form
/// doesn't compile; return an owned clone instead.
fn fix_borrowed_slice_returns(code: &str) -> String {
    let mut out = code.to_string();
    // Return type: `&[T]` -> `Vec<T>`
    while let Some(idx) = out.find("-> &[") {
        if let Some(end) = out[idx..].find("]") {
            let abs_end = idx + end;
            // Replace "-> &[T]" (5 chars "-> &[") with "-> Vec<[T]"
            out.replace_range(idx..=abs_end, &format!("-> Vec<{}>", &out[idx+5..abs_end]));
        } else { break; }
    }
    // Body: `return *G.lock().unwrap();` -> `return G.lock().unwrap().clone();`
    // (matches the all_messages pattern: return *<UPPER>.lock().unwrap();)
    out = out.replace("return *MESSAGES.lock().unwrap();", "return MESSAGES.lock().unwrap().clone();");
    out = out.replace("return *NOTES.lock().unwrap();", "return NOTES.lock().unwrap().clone();");
    out
}

fn generate_rust_server(api_module: &auto_lang::api::ApiModule, root_dir: &Path) -> AutoResult<()> {
    // Output to shared workspace at D:/.auto/rust-workspace/{name}-back/
    let ws_dir = crate::rust_ui::ensure_shared_workspace(root_dir);
    let back_name = crate::rust_ui::back_member_name(root_dir);
    let rust_dir = ws_dir.join(&back_name);
    let src_dir = rust_dir.join("src");
    std::fs::create_dir_all(&src_dir)
        .map_err(|e| format!("Failed to create rust/src: {}", e))?;

    // Plan musk-022: detect streaming endpoints — they need events.rs + cargo deps.
    let has_sse = api_module.endpoints.iter().any(|e| e.return_type.contains("Stream<"));

    // Plan musk-022 CRUD 扩展: read db.at early so has_db gates cargo deps + db.rs.
    let db_file = root_dir.join("src").join("back").join("db.at");
    let db_content = if db_file.exists() {
        std::fs::read_to_string(&db_file).ok()
    } else {
        None
    };
    let has_db = db_content.as_deref().map(|c| c.contains("pub fn")).unwrap_or(false);

    // Generate Cargo.toml (workspace member version — no [workspace])
    let cargo_toml = generate_cargo_toml(&back_name, has_sse, has_db);
    std::fs::write(rust_dir.join("Cargo.toml"), &cargo_toml)
        .map_err(|e| format!("Failed to write Cargo.toml: {}", e))?;

    // Generate types.rs
    let types_rs = generate_types_rs(api_module);
    std::fs::write(src_dir.join("types.rs"), &types_rs)
        .map_err(|e| format!("Failed to write types.rs: {}", e))?;

    // Plan 399 第 4-5 步: collect db.rs's public fn names so handlers can call
    // them directly (status unified to db.rs lazy_static) instead of State<Db>.
    let db_fns: Option<std::collections::HashSet<String>> = if has_db {
        if let Some(ref content) = db_content {
            match transpile_db_to_rs(content) {
                Ok(db_rs) => {
                    let db_rs = post_process_db_rs(db_rs);
                    // Also persist db.rs (idempotent with the write below).
                    if let Err(e) = std::fs::write(src_dir.join("db.rs"), &db_rs) {
                        eprintln!("  ⚠ Failed to write db.rs: {}", e);
                    }
                    Some(extract_db_fn_names(&db_rs))
                }
                Err(e) => {
                    eprintln!("  ⚠ db.rs transpile failed (db_fns unavailable, handlers fall back to State<Db>): {}", e);
                    None
                }
            }
        } else {
            None
        }
    } else {
        None
    };

    // Generate api.rs with route handlers
    let api_rs = generate_api_rs(api_module, db_fns.as_ref());
    std::fs::write(src_dir.join("api.rs"), &api_rs)
        .map_err(|e| format!("Failed to write api.rs: {}", e))?;

    // Plan musk-022: events broadcast-bus module for SSE backends.
    if has_sse {
        let events_rs = generate_events_rs();
        std::fs::write(src_dir.join("events.rs"), &events_rs)
            .map_err(|e| format!("Failed to write events.rs: {}", e))?;
    }

    // db.rs was already written above when db_fns was collected (Plan 399 4-5).
    // If has_db is true but transpile failed there, nothing more to do here.
    let seed_data = db_content;

    // Plan 399 第 4-5 步: when db.rs takes over state, main.rs drops State<Db>
    // (seed data lives in db.rs's once_cell::Lazy globals). `db_full_cover` is
    // true only when every non-SSE endpoint resolved to a db.rs call — otherwise
    // keep State<Db> for the template-fallback endpoints (mixed-state safety).
    let db_full_cover = db_fns.as_ref().map(|s| !s.is_empty()).unwrap_or(false)
        && all_endpoints_covered(api_module, db_fns.as_ref());

    // Generate main.rs
    let main_rs = generate_main_rs(api_module, seed_data.as_deref(), db_full_cover);
    std::fs::write(src_dir.join("main.rs"), &main_rs)
        .map_err(|e| format!("Failed to write main.rs: {}", e))?;

    // Update workspace members. MUST run after main.rs is written: ensure_shared_workspace
    // skips members with no src/main.rs (has_cargo_targets guard). Plan musk-022.
    let _ = crate::rust_ui::ensure_shared_workspace(root_dir);

    println!("  ✓ Generated Rust server: {}/", back_name);

    Ok(())
}

/// Generate Cargo.toml for the Rust server (workspace member version).
///
/// `package_name` must be unique across the shared workspace — multiple
/// projects' backends live as siblings under `D:/.auto/rust-workspace`, and
/// cargo forbids two members with the same package name. Use the per-project
/// `back_member_name` (e.g. "015-notes-back"), not a fixed "api-server".
fn generate_cargo_toml(package_name: &str, has_sse: bool, has_db: bool) -> String {
    // Plan 328: Cargo rejects names starting with a digit.
    let safe_name = if package_name.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false) {
        format!("app-{}", package_name)
    } else {
        package_name.to_string()
    };
    let sse_deps = if has_sse { "
async-stream = \"0.3\"
futures = \"0.3\"" } else { "" };
    // Plan musk-022 CRUD 扩展: a2r 全局变量转译用 once_cell::Lazy.
    let db_deps = if has_db { "
once_cell = \"1\"" } else { "" };
    format!(
        r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[dependencies]
axum.workspace = true
tokio = {{ version = "1", features = ["full"] }}
serde.workspace = true
serde_json.workspace = true
tower-http.workspace = true{}{}
"#,
        safe_name, db_deps, sse_deps
    )
}

/// Plan musk-022: SSE broadcast-bus module. tokio::sync::broadcast channel;
/// POST handlers call broadcast(json), GET stream handler calls subscribe().
fn generate_events_rs() -> String {
    r#"// Auto-generated SSE event bus (Plan musk-022).
use tokio::sync::broadcast;
type Bus = broadcast::Sender<String>;
fn bus() -> Bus {
    use std::sync::OnceLock;
    static BUS: OnceLock<Bus> = OnceLock::new();
    BUS.get_or_init(|| { let (tx, _rx) = broadcast::channel(256); tx }).clone()
}
pub fn subscribe() -> broadcast::Receiver<String> { bus().subscribe() }
pub fn broadcast(json: String) { let _ = bus().send(json); }
"#.to_string()
}

/// Generate types.rs with serde structs
fn generate_types_rs(api_module: &auto_lang::api::ApiModule) -> String {
    let mut lines = vec!["use serde::{Serialize, Deserialize};".to_string(), "".to_string()];

    for api_type in &api_module.types {
        // Include Default derive for simple placeholder generation
        lines.push(format!("#[derive(Clone, Debug, Default, Serialize, Deserialize)]"));
        lines.push(format!("pub struct {} {{", api_type.name));
        for field in &api_type.fields {
            let rust_type = auto_type_to_rust(&field.ty);
            lines.push(format!("    pub {}: {},", field.name, rust_type));
        }
        lines.push("}".to_string());
        lines.push("".to_string());
    }

    lines.join("\n")
}

/// Convert AutoLang type to Rust type
/// Convert snake_case to PascalCase (e.g. "search_notes" → "SearchNotes")
fn to_pascal_case(s: &str) -> String {
    s.split('_')
        .map(|word| {
            let mut c = word.chars();
            match c.next() {
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                None => String::new(),
            }
        })
        .collect()
}

fn auto_type_to_rust(auto_type: &str) -> String {
    // Handle optional type: prefix ?T (AutoLang syntax: ?Note) or suffix T?
    let auto_type = auto_type.trim();
    if let Some(inner) = auto_type.strip_prefix('?') {
        return format!("Option<{}>", auto_type_to_rust(inner));
    }
    if auto_type.ends_with('?') {
        let inner = &auto_type[..auto_type.len()-1];
        return format!("Option<{}>", auto_type_to_rust(inner));
    }

    match auto_type {
        "int" => "i64".to_string(),
        "str" => "String".to_string(),
        "bool" => "bool".to_string(),
        "float" => "f64".to_string(),
        s if s.starts_with("[]") || s.starts_with("[") => {
            // Handle []T and [N]T
            let inner = s.trim_start_matches(|c: char| c == '[' || c == ']' || c.is_numeric());
            format!("Vec<{}>", auto_type_to_rust(inner))
        }
        s => s.to_string(),
    }
}

/// Determine if a path contains a path parameter (e.g., `:id`)
fn has_path_param(path: &str) -> bool {
    path.split('/').any(|s| s.starts_with(':'))
}



/// Determine the primary type from an ApiModule (first defined type)
pub fn primary_type_name_pub(api_module: &auto_lang::api::ApiModule) -> Option<String> {
    api_module.types.first().map(|t| t.name.clone())
}

/// Get body params (params that aren't path params and aren't query params)
fn endpoint_body_params(endpoint: &ApiEndpoint) -> Vec<&ApiParam> {
    let path = endpoint.path();
    let method = endpoint.method();
    endpoint.params.iter().filter(|p| {
        let is_path = path.contains(&format!(":{}", p.name));
        let is_query = !is_path && matches!(method.as_str(), "GET" | "DELETE");
        !is_path && !is_query
    }).collect()
}

/// Get query params (non-path params on GET/DELETE endpoints)
fn endpoint_query_params(endpoint: &ApiEndpoint) -> Vec<&ApiParam> {
    let path = endpoint.path();
    let method = endpoint.method();
    endpoint.params.iter().filter(|p| {
        let is_path = path.contains(&format!(":{}", p.name));
        !is_path && matches!(method.as_str(), "GET" | "DELETE")
    }).collect()
}

/// Get path params (params that appear in the URL path)
fn endpoint_path_params(endpoint: &ApiEndpoint) -> Vec<&ApiParam> {
    let path = endpoint.path();
    endpoint.params.iter().filter(|p| {
        path.contains(&format!(":{}", p.name))
    }).collect()
}

/// Check if endpoint has a JSON body (POST/PUT with non-path params)
fn endpoint_has_body(endpoint: &ApiEndpoint) -> bool {
    let method = endpoint.method();
    matches!(method.as_str(), "POST" | "PUT")
}

// ============================================================================
// Plan 399 第 4-5 步: db.rs delegation helpers
// ============================================================================

/// Extract the set of `pub fn NAME` names from a transpiled db.rs source.
/// Used to decide whether an HTTP handler can delegate to `db::NAME(...)`
/// instead of the `State<Db>` CRUD template.
fn extract_db_fn_names(db_rs: &str) -> std::collections::HashSet<String> {
    use regex::Regex;
    let mut set = std::collections::HashSet::new();
    // Match `pub fn name(` at the start of a line (a2r emits this form).
    let re = Regex::new(r"(?m)^\s*pub\s+fn\s+([A-Za-z_][A-Za-z0-9_]*)\s*\(")
        .expect("valid regex");
    for cap in re.captures_iter(db_rs) {
        if let Some(m) = cap.get(1) {
            set.insert(m.as_str().to_string());
        }
    }
    set
}

/// Build the ordered list of candidate db.rs function names for an endpoint,
/// in priority order. The api.at convention (015-notes / 017-chat) is that each
/// endpoint body is a thin `return db.FN(args)` delegate, so FN follows the
/// endpoint name with CRUD-verb normalization:
///   - exact name first (list_notes may map to a db fn literally named list_notes)
///   - list_X / get_X        → all_X / all_Xs / find_X / get_X
///   - send_X / create_X     → create_X / add_X
///   - update_X / edit_X     → update_X
///   - delete_X / remove_X   → delete_X / remove_X
///   - toggle_X              → toggle_X
///   - search_X              → search_X
///   - otherwise: same name (rely on exact match above)
fn db_fn_candidates(endpoint: &ApiEndpoint) -> Vec<String> {
    let name = endpoint.fn_name.as_str();
    let mut cands: Vec<String> = vec![name.to_string()];

    // The api.at convention (015-notes / 017-chat): each endpoint body is a thin
    // `return db.FN(args)` delegate. FN follows the endpoint name with CRUD-verb
    // normalization. The "subject" after the verb is the db fn's subject as-is
    // (e.g. list_notes → all_notes: rest="notes" → all_notes; list_messages →
    // all_messages: rest="messages" → all_messages).
    for (verb, rest_fn) in [
        ("list", name.strip_prefix("list_")),
        ("get", name.strip_prefix("get_")),
        ("find", name.strip_prefix("find_")),
        ("send", name.strip_prefix("send_")),
        ("create", name.strip_prefix("create_")),
        ("add", name.strip_prefix("add_")),
        ("update", name.strip_prefix("update_")),
        ("edit", name.strip_prefix("edit_")),
        ("move", name.strip_prefix("move_")),
        ("delete", name.strip_prefix("delete_")),
        ("remove", name.strip_prefix("remove_")),
        ("toggle", name.strip_prefix("toggle_")),
        ("search", name.strip_prefix("search_")),
    ] {
        if let Some(rest) = rest_fn {
            // The "subject" for the db fn: for list_notes rest="notes", db fn is
            // all_notes. For list_messages rest="messages", db fn is all_messages.
            // So the `all_{rest}` form (no re-pluralization) is what we want; drop
            // the `all_{rest}s` candidate to avoid double-plural when rest is already plural.
            let mut cands_for_verb = match verb {
                "list" => vec![
                    format!("all_{}", rest),       // all_notes / all_messages  ✓
                    format!("get_{}", rest),
                    format!("list_{}", rest),
                ],
                "get" | "find" => vec![
                    format!("find_{}", rest),
                    format!("get_{}", rest),
                ],
                "send" | "create" | "add" => vec![
                    format!("create_{}", rest),
                    format!("add_{}", rest),
                ],
                "update" | "edit" | "move" => vec![format!("update_{}", rest)],
                "delete" | "remove" => vec![
                    format!("delete_{}", rest),
                    format!("remove_{}", rest),
                ],
                "toggle" => vec![format!("toggle_{}", rest)],
                "search" => vec![format!("search_{}", rest)],
                _ => vec![],
            };
            cands.append(&mut cands_for_verb);
        }
    }

    cands.dedup();
    cands
}

/// The resolved db.rs delegation for one endpoint: the db fn name plus the
/// ordered argument expressions to pass it (mapped from extractors).
struct DbDelegation {
    db_fn: String,
    /// Argument expressions in source order, e.g. ["id", "input.sender", "query.q"].
    args: Vec<String>,
}

/// Resolve an endpoint to a db.rs function call, or `None` if no candidate name
/// exists in `db_fns`. When matched, builds the argument list by mapping each
/// endpoint param to its extractor binding:
///   - path param  → bare ident `name` (Path extractor binds it directly)
///   - query param → `query.name`
///   - body param  → `input.name`
/// a2r transpiles `str` params to Rust `&str`, but axum's serde extractors hold
/// owned `String`s — so string params are borrowed (`&input.name`/`&query.name`)
/// and Rust's deref coercion (`&String` → `&str`) bridges the gap. Non-str params
/// pass by value as-is. Params preserve api.at declaration order (matches the
/// db.rs fn signature).
fn resolve_db_call(
    endpoint: &ApiEndpoint,
    db_fns: &std::collections::HashSet<String>,
) -> Option<DbDelegation> {
    let db_fn = db_fn_candidates(endpoint)
        .into_iter()
        .find(|c| db_fns.contains(c))?;
    let path = endpoint.path();
    let method = endpoint.method();
    let is_str = |ty: &str| {
        let t = ty.trim();
        t == "str" || t == "String" || t == "&str"
    };
    let args: Vec<String> = endpoint.params.iter().map(|p| {
        let is_path = path.contains(&format!(":{}", p.name));
        let is_query = !is_path && matches!(method.as_str(), "GET" | "DELETE");
        let borrow = is_str(&p.ty);
        if is_path {
            // Path str params are rare; borrow them too (Path<String> → &str).
            if borrow { format!("&{}", p.name) } else { p.name.clone() }
        } else if is_query {
            if borrow { format!("&query.{}", p.name) } else { format!("query.{}", p.name) }
        } else {
            if borrow { format!("&input.{}", p.name) } else { format!("input.{}", p.name) }
        }
    }).collect();
    Some(DbDelegation { db_fn, args })
}

/// True when every non-SSE endpoint in the module resolves to a db.rs call.
/// Used to decide whether main.rs can drop `State<Db>` entirely (full db.rs
/// coverage) vs. keep it for template-fallback endpoints (mixed state).
fn all_endpoints_covered(
    api_module: &auto_lang::api::ApiModule,
    db_fns: Option<&std::collections::HashSet<String>>,
) -> bool {
    let db_fns = match db_fns {
        Some(s) => s,
        None => return false,
    };
    api_module.endpoints.iter().all(|e| {
        // SSE/streaming endpoints don't touch State<Db>; ignore them.
        e.return_type.contains("Stream<") || resolve_db_call(e, db_fns).is_some()
    })
}

/// Generate api.rs with route handlers — full CRUD implementation.
///
/// Plan 399 第 4-5 步: when `db_fns` is `Some`, endpoints whose business logic
/// lives in db.rs (matched by `resolve_db_call`) get a handler that calls
/// `db::FN(...)` directly instead of the `State<Db>` CRUD template. Endpoints
/// not matched fall back to the template (with a warning). `None` keeps the
/// legacy `State<Db>` behavior (e.g. seed-only backends with no db.at).
fn generate_api_rs(
    api_module: &auto_lang::api::ApiModule,
    db_fns: Option<&std::collections::HashSet<String>>,
) -> String {
    let db_active = db_fns.map(|s| !s.is_empty()).unwrap_or(false);
    let mut lines = vec![
        "use axum::{".to_string(),
        "    extract::{Path, State, Json, Query},".to_string(),
        "    http::StatusCode,".to_string(),
        "    Json as JsonResponse,".to_string(),
        "};".to_string(),
        "use crate::types::*;".to_string(),
        "use std::sync::{Arc, Mutex};".to_string(),
        "".to_string(),
    ];

    // Determine primary type and generate Db type alias
    let primary_type = match primary_type_name_pub(api_module) {
        Some(t) => t,
        None => {
            // Fallback: generate skeleton handlers
            lines.push("// No types defined, generating skeleton handlers".to_string());
            for endpoint in &api_module.endpoints {
                lines.push("".to_string());
                lines.push(format!("pub async fn {}() {{", endpoint.fn_name));
                lines.push("    // TODO: Implement".to_string());
                lines.push("}".to_string());
            }
            return lines.join("\n");
        }
    };

    lines.push(format!("pub type Db = Arc<Mutex<Vec<{}>>>;", primary_type));
    lines.push("".to_string());

    // Generate CreateInput struct for POST endpoints with body fields
    for endpoint in &api_module.endpoints {
        if endpoint.method() == "POST" {
            let body_params = endpoint_body_params(endpoint);
            if !body_params.is_empty() {
                lines.push("#[derive(serde::Deserialize)]".to_string());
                lines.push(format!("pub struct Create{}Input {{", primary_type));
                for param in &body_params {
                    let rust_type = auto_type_to_rust(&param.ty);
                    lines.push(format!("    pub {}: {},", param.name, rust_type));
                }
                lines.push("}".to_string());
                lines.push("".to_string());
                break; // Only one CreateInput per primary type
            }
        }
    }

    // Generate UpdateInput structs for PUT/PATCH endpoints with body fields.
    // Each unique set of body params gets its own struct.
    let mut seen_param_sets: Vec<String> = Vec::new();
    for endpoint in &api_module.endpoints {
        let method = endpoint.method();
        let ep_fn_name = &endpoint.fn_name;
        if method == "PUT" || method == "PATCH" {
            let body_params = endpoint_body_params(endpoint);
            if !body_params.is_empty() {
                // Create a signature for this param set
                let param_sig: String = body_params.iter()
                    .map(|p| format!("{}:{}", p.name, p.ty))
                    .collect::<Vec<_>>().join(",");
                if seen_param_sets.contains(&param_sig) {
                    continue; // Skip duplicate
                }
                seen_param_sets.push(param_sig);

                // Struct name: Update{Type}Input for first, Update{Type}{FnName}Input for others
                let struct_name = if seen_param_sets.len() == 1 {
                    format!("Update{}Input", primary_type)
                } else {
                    // PascalCase the fn_name suffix
                    let suffix: String = ep_fn_name.split('_').map(|s| {
                        let mut c = s.chars();
                        match c.next() {
                            Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                            None => String::new(),
                        }
                    }).collect::<String>();
                    format!("Update{}{}Input", primary_type, suffix)
                };

                lines.push("#[derive(serde::Deserialize)]".to_string());
                lines.push(format!("pub struct {} {{", struct_name));
                for param in &body_params {
                    let rust_type = auto_type_to_rust(&param.ty);
                    lines.push(format!("    pub {}: {},", param.name, rust_type));
                }
                lines.push("}".to_string());
                lines.push("".to_string());
            }
        }
    }

    // Generate Query structs for GET/DELETE endpoints with query params
    for endpoint in &api_module.endpoints {
        let query_params = endpoint_query_params(endpoint);
        if !query_params.is_empty() {
            let struct_name = format!("{}Query", to_pascal_case(&endpoint.fn_name));
            lines.push("#[derive(serde::Deserialize)]".to_string());
            lines.push(format!("pub struct {} {{", struct_name));
            for param in &query_params {
                let rust_type = auto_type_to_rust(&param.ty);
                lines.push(format!("    pub {}: {},", param.name, rust_type));
            }
            lines.push("}".to_string());
            lines.push("".to_string());
        }
    }

    // Get type field names for time detection
    let type_fields: Vec<&str> = api_module.types.iter()
        .find(|t| t.name == primary_type)
        .map(|t| t.fields.iter().map(|f| f.name.as_str()).collect())
        .unwrap_or_default();
    let has_time_field = type_fields.contains(&"time");
    // Convention: first field is the ID field
    let id_field = type_fields.first().copied().unwrap_or("id");

    // Plan musk-022: detect SSE for POST broadcast.
    let has_sse = api_module.endpoints.iter().any(|e| e.return_type.contains("Stream<"));

    // Generate handler for each endpoint
    for endpoint in &api_module.endpoints {
        let method = endpoint.method();
        let fn_name = &endpoint.fn_name;
        let has_path = has_path_param(&endpoint.path());

        // Plan musk-022: streaming endpoints get an SSE handler (Sse<impl Stream>),
        // not a JSON CRUD handler. Subscribes to the events bus, emits each as SSE.
        if endpoint.return_type.contains("Stream<") {
            lines.push(format!(
                "pub async fn {}() -> axum::response::Sse<impl futures::Stream<Item = Result<axum::response::sse::Event, std::convert::Infallible>>> {{",
                fn_name
            ));
            lines.push("    let rx = crate::events::subscribe();".to_string());
            lines.push("    let stream = async_stream::stream! {".to_string());
            lines.push("        let mut rx = rx;".to_string());
            lines.push("        while let Ok(json) = rx.recv().await {".to_string());
            lines.push("            yield Ok(axum::response::sse::Event::default().data(json));".to_string());
            lines.push("        }".to_string());
            lines.push("    };".to_string());
            lines.push("    axum::response::Sse::new(stream)".to_string());
            lines.push("}".to_string());
            lines.push("".to_string());
            continue;
        }

        // Plan 399 第 4-5 步: try to delegate this handler to a db.rs function.
        // When db.rs is active and a matching fn is found, the handler calls
        // `db::FN(...)` instead of locking `State<Db>` — state stays unified in
        // db.rs's once_cell::Lazy globals.
        let db_delegation = if db_active {
            db_fns.and_then(|fns| resolve_db_call(endpoint, fns))
        } else {
            None
        };
        if db_active && db_delegation.is_none() {
            // db.rs exists but no matching fn for this endpoint — must keep
            // State<Db> as a fallback (mixed state). Warn so it's visible.
            eprintln!(
                "  ⚠ endpoint `{}` has no db.rs match; falling back to State<Db> template",
                fn_name
            );
        }

        // Build function parameters
        let mut params = vec![];
        if has_path {
            let path_params = endpoint_path_params(endpoint);
            if let Some(first) = path_params.first() {
                let rust_type = auto_type_to_rust(&first.ty);
                params.push(format!("Path({}): Path<{}>", first.name, rust_type));
            }
        }
        // Only inject State<Db> when NOT delegating to db.rs.
        if db_delegation.is_none() {
            params.push("State(db): State<Db>".to_string());
        }
        // Query params for GET/DELETE with non-path params
        let query_params = endpoint_query_params(endpoint);
        let has_query = !query_params.is_empty();
        if has_query {
            let query_struct = format!("{}Query", to_pascal_case(fn_name));
            params.push(format!("Query(query): Query<{}>", query_struct));
        }
        if endpoint_has_body(endpoint) {
            if method == "POST" {
                let body_params = endpoint_body_params(endpoint);
                if !body_params.is_empty() {
                    params.push(format!("Json(input): Json<Create{}Input>", primary_type));
                } else {
                    params.push(format!("Json(input): Json<{}>", primary_type));
                }
            } else {
                // PUT/PATCH uses per-endpoint Input struct if body params exist
                let body_params = endpoint_body_params(endpoint);
                if !body_params.is_empty() {
                    // Find the matching struct name for this endpoint's params
                    let param_sig: String = body_params.iter()
                        .map(|p| format!("{}:{}", p.name, p.ty))
                        .collect::<Vec<_>>().join(",");
                    let struct_name = if seen_param_sets.first().map(|s| s == &param_sig).unwrap_or(false) {
                        format!("Update{}Input", primary_type)
                    } else {
                        let suffix: String = fn_name.split('_').map(|s| {
                            let mut c = s.chars();
                            match c.next() {
                                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                                None => String::new(),
                            }
                        }).collect::<String>();
                        format!("Update{}{}Input", primary_type, suffix)
                    };
                    params.push(format!("Json(input): Json<{}>", struct_name));
                } else {
                    params.push(format!("Json(input): Json<{}>", primary_type));
                }
            }
        }

        // Determine return type
        // Strip Option wrapper for endpoints that use Result<_, StatusCode> for 404
        let raw_ret = auto_type_to_rust(&endpoint.return_type);
        let is_void = raw_ret == "()" || raw_ret == "void";
        // Wrap in Result if endpoint may return NOT_FOUND
        let needs_result = has_path || matches!(method.as_str(), "DELETE" | "PUT");
        // For Result-returning endpoints, strip Option<> since 404 is handled via Err
        let json_inner = if needs_result {
            raw_ret.strip_prefix("Option<")
                .and_then(|s| s.strip_suffix('>'))
                .unwrap_or(&raw_ret)
                .to_string()
        } else {
            raw_ret.clone()
        };
        let json_ret = if is_void {
            "StatusCode".to_string()
        } else {
            format!("JsonResponse<{}>", json_inner)
        };
        let ret_type = if needs_result {
            format!("Result<{}, StatusCode>", json_ret)
        } else {
            json_ret
        };

        lines.push(format!(
            "pub async fn {}({}) -> {} {{",
            fn_name,
            params.join(", "),
            ret_type
        ));

        // Plan 399 第 4-5 步: db.rs delegation body. When resolved, the handler
        // body is just `db::FN(args)` (optionally broadcasting an SSE event for
        // POST creates). This replaces the entire State<Db> CRUD template below.
        // `json_inner` holds the inner Rust type (e.g. `Vec<Message>`/`Message`)
        // already computed above; reuse it to wrap the db result.
        if let Some(deleg) = &db_delegation {
            let call = format!("crate::db::{}({})", deleg.db_fn, deleg.args.join(", "));
            if is_void {
                if needs_result {
                    lines.push(format!("    match {} {{ Some(_) => Ok(StatusCode::OK), None => Err(StatusCode::NOT_FOUND) }};", call));
                } else {
                    lines.push(format!("    {};", call));
                    lines.push("    StatusCode::OK".to_string());
                }
            } else if needs_result {
                // Option<T>-returning db fn → Ok-or-404. Non-Option with path
                // (rare) wraps directly in Ok(JsonResponse::<T>(...)).
                if endpoint.return_type.trim_start().starts_with('?')
                    || endpoint.return_type.contains("?")
                {
                    lines.push(format!(
                        "    {}.map(JsonResponse::<{}>).ok_or(StatusCode::NOT_FOUND)",
                        call, json_inner
                    ));
                } else {
                    lines.push(format!(
                        "    Ok(JsonResponse::<{}>({}))",
                        json_inner, call
                    ));
                }
            } else if has_sse && method == "POST" {
                // Capture the created item, broadcast, then return it.
                lines.push(format!("    let item = {};", call));
                lines.push("    let mut evt = serde_json::to_value(&item).unwrap_or_default();".to_string());
                lines.push("    if let Some(obj) = evt.as_object_mut() { obj.insert(\"event\".to_string(), serde_json::Value::String(\"NewMessage\".to_string())); }".to_string());
                lines.push("    crate::events::broadcast(evt.to_string());".to_string());
                lines.push(format!("    JsonResponse::<{}>(item)", json_inner));
            } else {
                // Non-void, no path, no SSE: wrap the db fn result directly.
                lines.push(format!("    JsonResponse::<{}>({})", json_inner, call));
            }
            lines.push("}".to_string());
            lines.push("".to_string());
            continue;
        }

        // Generate handler body based on CRUD operation
        match method.as_str() {
            "GET" if !has_path && has_query => {
                // Search/filter: filter items by query params (case-insensitive
                // substring match on all string fields, or exact match on others).
                lines.push("    let items = db.lock().unwrap();".to_string());
                lines.push("    let filtered: Vec<_> = items.iter().filter(|n| {".to_string());
                for (i, param) in query_params.iter().enumerate() {
                    let connector = if i == 0 { "" } else { " && " };
                    let field = param.name.trim_start_matches("query_").trim_start_matches("search_");
                    // Heuristic: "query" param → search title+body; named field → match that field
                    if field == "query" || field == "q" || field == "search" {
                        lines.push(format!("        {}n.title.to_lowercase().contains(&query.{}.to_lowercase()) || n.body.to_lowercase().contains(&query.{}.to_lowercase())",
                            connector, param.name, param.name));
                    } else {
                        lines.push(format!("        {}n.{} == query.{}", connector, field, param.name));
                    }
                }
                lines.push("    }).cloned().collect();".to_string());
                lines.push("    JsonResponse(filtered)".to_string());
            }
            "GET" if !has_path => {
                // List all
                lines.push("    let items = db.lock().unwrap();".to_string());
                lines.push("    JsonResponse(items.clone())".to_string());
            }
            "GET" if has_path => {
                // Get by ID
                let path_params = endpoint_path_params(endpoint);
                let id_name = path_params.first().map(|p| p.name.as_str()).unwrap_or(id_field);
                lines.push("    let items = db.lock().unwrap();".to_string());
                lines.push("    items.iter()".to_string());
                lines.push(format!("        .find(|n| n.{} == {})", id_name, id_name));
                lines.push("        .cloned()".to_string());
                lines.push("        .map(JsonResponse)".to_string());
                lines.push("        .ok_or(StatusCode::NOT_FOUND)".to_string());
            }
            "POST" => {
                // Create
                lines.push("    let mut items = db.lock().unwrap();".to_string());
                lines.push(format!(
                    "    let new_id = items.iter().map(|n| n.{}).max().unwrap_or(-1) + 1;",
                    id_field
                ));
                let body_params = endpoint_body_params(endpoint);
                if body_params.is_empty() {
                    lines.push(format!(
                        "    let item = {} {{ {}: new_id, ..Default::default() }};",
                        primary_type, id_field
                    ));
                } else {
                    lines.push(format!("    let item = {} {{", primary_type));
                    lines.push(format!("        {}: new_id,", id_field));
                    for param in &body_params {
                        lines.push(format!("        {}: input.{},", param.name, param.name));
                    }
                    if has_time_field {
                        lines.push("        time: \"Just now\".to_string(),".to_string());
                    }
                    lines.push("        ..Default::default()".to_string());
                    lines.push("    };".to_string());
                }
                lines.push("    items.push(item.clone());".to_string());
                if has_sse {
                    lines.push("    let mut evt = serde_json::to_value(&item).unwrap_or_default();".to_string());
                    lines.push("    if let Some(obj) = evt.as_object_mut() { obj.insert(\"event\".to_string(), serde_json::Value::String(\"NewMessage\".to_string())); }".to_string());
                    lines.push("    crate::events::broadcast(evt.to_string());".to_string());
                }
                lines.push("    JsonResponse(item)".to_string());
            }
            "PUT" => {
                // Update
                let path_params = endpoint_path_params(endpoint);
                let id_name = path_params.first().map(|p| p.name.as_str()).unwrap_or(id_field);
                lines.push("    let mut items = db.lock().unwrap();".to_string());
                lines.push(format!(
                    "    if let Some(item) = items.iter_mut().find(|n| n.{} == {}) {{",
                    id_name, id_name
                ));
                let body_params = endpoint_body_params(endpoint);
                if !body_params.is_empty() {
                    for param in &body_params {
                        lines.push(format!("        item.{} = input.{}.clone();", param.name, param.name));
                    }
                } else {
                    // Update from full type - copy all fields except id
                    for field in &type_fields {
                        if *field != id_name {
                            lines.push(format!("        item.{} = input.{}.clone();", field, field));
                        }
                    }
                }
                if has_time_field && !body_params.iter().any(|p| p.name == "time") {
                    lines.push("        item.time = \"Just now\".to_string();".to_string());
                }
                lines.push("        Ok(JsonResponse(item.clone()))".to_string());
                lines.push("    } else {".to_string());
                lines.push("        Err(StatusCode::NOT_FOUND)".to_string());
                lines.push("    }".to_string());
            }
            "DELETE" => {
                // Delete
                let path_params = endpoint_path_params(endpoint);
                let id_name = path_params.first().map(|p| p.name.as_str()).unwrap_or(id_field);
                lines.push("    let mut items = db.lock().unwrap();".to_string());
                lines.push("    let len_before = items.len();".to_string());
                lines.push(format!("    items.retain(|n| n.{} != {});", id_name, id_name));
                lines.push("    if items.len() < len_before {".to_string());
                if raw_ret == "bool" {
                    lines.push("        Ok(JsonResponse(true))".to_string());
                } else {
                    lines.push("        Ok(StatusCode::OK)".to_string());
                }
                lines.push("    } else {".to_string());
                lines.push("        Err(StatusCode::NOT_FOUND)".to_string());
                lines.push("    }".to_string());
            }
            "PATCH" => {
                // PATCH: toggle/update specific fields (e.g. toggle_pin)
                let path_params = endpoint_path_params(endpoint);
                let id_name = path_params.first().map(|p| p.name.as_str()).unwrap_or(id_field);
                let body_params = endpoint_body_params(endpoint);
                lines.push("    let mut items = db.lock().unwrap();".to_string());
                lines.push(format!(
                    "    if let Some(item) = items.iter_mut().find(|n| n.{} == {}) {{",
                    id_name, id_name
                ));
                // Toggle boolean fields; set others from input
                for param in &body_params {
                    if param.ty.contains("bool") {
                        lines.push(format!("        item.{} = !item.{};", param.name, param.name));
                    } else {
                        lines.push(format!("        item.{} = input.{}.clone();", param.name, param.name));
                    }
                }
                // If no body params, infer toggle from fn name (e.g. toggle_pin → toggle pinned)
                if body_params.is_empty() {
                    // Try to infer field name from function name (toggle_X → X or toggle_Xed)
                    if let Some(stripped) = fn_name.strip_prefix("toggle_") {
                        // Try exact match first, then with "ned" / "d" suffix (pin → pinned)
                        let field = if type_fields.contains(&stripped) {
                            stripped.to_string()
                        } else if type_fields.contains(&format!("{}ned", stripped).as_str()) {
                            format!("{}ned", stripped)
                        } else if type_fields.contains(&format!("{}d", stripped).as_str()) {
                            format!("{}d", stripped)
                        } else {
                            stripped.to_string()
                        };
                        lines.push(format!("        item.{} = !item.{};", field, field));
                    }
                }
                lines.push("        Ok(JsonResponse(item.clone()))".to_string());
                lines.push("    } else {".to_string());
                lines.push("        Err(StatusCode::NOT_FOUND)".to_string());
                lines.push("    }".to_string());
            }
            _ => {
                // Default fallback
                lines.push("    // TODO: Implement".to_string());
                lines.push("    JsonResponse(Default::default())".to_string());
            }
        }

        lines.push("}".to_string());
        lines.push("".to_string());
    }

    lines.join("\n")
}

/// Generate initial sample data for the primary type.
///
/// If db_at_content is provided, try to extract seed data from
/// `var notes List<Note>.new([...])` declarations. Fall back to
/// generating 3 default sample items.
pub fn generate_initial_data_pub(api_module: &auto_lang::api::ApiModule, db_at_content: Option<&str>) -> String {
    // Try to extract seed data from db.at first
    if let Some(content) = db_at_content {
        if let Some(seed) = extract_seed_data(content, api_module) {
            return seed;
        }
    }

    // Fall back to hardcoded samples
    generate_default_seed_data(api_module)
}

/// Extract seed data from db.at's `var notes List<Note>.new([...])` declaration.
/// Parses the Note { ... } struct literals and converts to Rust.
fn extract_seed_data(db_content: &str, api_module: &auto_lang::api::ApiModule) -> Option<String> {
    let primary_type = primary_type_name_pub(api_module)?;
    let api_type = api_module.types.iter().find(|t| t.name == primary_type)?;

    // Find .new([ pattern (most reliable)
    let new_pattern = ".new([";
    let start = db_content.find(new_pattern)?;
    let after_start = &db_content[start + new_pattern.len()..];
    let after_start = &db_content[start + new_pattern.len()..];

    // Find matching closing ]) — count brackets
    let mut depth = 1;
    let mut end = 0;
    for (i, c) in after_start.chars().enumerate() {
        match c {
            '[' => depth += 1,
            ']' => {
                depth -= 1;
                if depth == 0 {
                    end = i;
                    break;
                }
            }
            _ => {}
        }
    }
    if end == 0 {
        return None;
    }

    let items_str = &after_start[..end];

    // Parse individual Note { field: value, ... } entries
    let mut rust_items = Vec::new();
    let mut remaining = items_str;
    while let Some(type_start) = remaining.find(&format!("{} {{", primary_type)) {
        let after_type = &remaining[type_start + primary_type.len()..];
        // Find matching closing brace
        let mut brace_depth = 0;
        let mut brace_end = 0;
        for (i, c) in after_type.chars().enumerate() {
            match c {
                '{' => brace_depth += 1,
                '}' => {
                    brace_depth -= 1;
                    if brace_depth == 0 {
                        brace_end = i;
                        break;
                    }
                }
                _ => {}
            }
        }
        if brace_end == 0 {
            break;
        }

        // Skip leading whitespace and opening brace
        let inner_start = after_type.find('{').map(|p| p + 1).unwrap_or(1);
        let fields_str = &after_type[inner_start..brace_end];
        let rust_fields = convert_at_fields_to_rust(fields_str, api_type);
        rust_items.push(format!(
            "        {} {{\n            {}\n        }}",
            primary_type, rust_fields
        ));

        remaining = &after_type[brace_end + 1..];
    }

    if rust_items.is_empty() {
        return None;
    }

    Some(format!("vec![\n{}\n    ]", rust_items.join(",\n")))
}

/// Convert Auto-style struct fields to Rust struct fields.
/// E.g., `title: "Welcome"` → `title: "Welcome".into()`
///       `pinned: true` → `pinned: true`
///       `tags: ["intro"]` → `tags: vec!["intro".into()]`
///       `folder: ""` → `folder: "".into()`
fn convert_at_fields_to_rust(fields_str: &str, api_type: &auto_lang::api::ApiType) -> String {
    let mut rust_fields = Vec::new();

    // Parse field: value pairs (comma-separated at top level)
    let mut current_field = String::new();
    let mut depth = 0;
    let mut in_string = false;
    let mut fields: Vec<String> = Vec::new();

    for c in fields_str.chars() {
        match c {
            '"' => {
                in_string = !in_string;
                current_field.push(c);
            }
            '[' | '{' if !in_string => {
                depth += 1;
                current_field.push(c);
            }
            ']' | '}' if !in_string => {
                depth -= 1;
                current_field.push(c);
            }
            ',' if depth == 0 && !in_string => {
                fields.push(current_field.trim().to_string());
                current_field.clear();
            }
            _ => {
                current_field.push(c);
            }
        }
    }
    if !current_field.trim().is_empty() {
        fields.push(current_field.trim().to_string());
    }

    for field_def in &fields {
        if let Some(colon_pos) = field_def.find(':') {
            let name = field_def[..colon_pos].trim();
            let value = field_def[colon_pos + 1..].trim();

            // Infer conversion from value syntax (don't rely on api_type lookup)
            let rust_value = if value.starts_with('[') {
                // Array: ["a", "b"] → vec!["a".into(), "b".into()]
                let inner = value.trim_start_matches('[').trim_end_matches(']');
                let items: Vec<&str> = inner.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()).collect();
                let rust_items: Vec<String> = items.iter()
                    .map(|s| format!("{}.into()", s))
                    .collect();
                format!("vec![{}]", rust_items.join(", "))
            } else if value.starts_with('"') {
                // String literal: ensure .into()
                format!("{}.into()", value)
            } else {
                // Number, bool, etc: pass through
                value.to_string()
            };

            rust_fields.push(format!("{}: {}", name, rust_value));
        }
    }

    rust_fields.join(",\n            ")
}

/// Generate 3 default sample items (fallback when no db.at seed data)
fn generate_default_seed_data(api_module: &auto_lang::api::ApiModule) -> String {
    let primary_type = match primary_type_name_pub(api_module) {
        Some(t) => t,
        None => return "Vec::new()".to_string(),
    };

    let api_type = match api_module.types.iter().find(|t| t.name == primary_type) {
        Some(t) => t,
        None => return "Vec::new()".to_string(),
    };

    // Generate 3 sample items based on type fields
    let mut items = vec![];
    for i in 0..3 {
        let fields: Vec<String> = api_type.fields.iter().map(|f| {
            let val = match f.ty.as_str() {
                "int" | "i64" => format!("{}", i),
                "str" | "String" => {
                    let sample = match f.name.as_str() {
                        "title" | "name" => match i {
                            0 => "Welcome",
                            1 => "Shopping List",
                            _ => "Meeting Notes",
                        },
                        "body" | "description" | "content" => match i {
                            0 => "This is your notes app. Click on any note to view it.",
                            1 => "Milk, Eggs, Bread, Cheese",
                            _ => "Q3 roadmap discussion with the team",
                        },
                        "email" => match i {
                            0 => "alice@example.com",
                            1 => "bob@example.com",
                            _ => "charlie@example.com",
                        },
                        "time" | "date" | "created_at" => match i {
                            0 => "Just now",
                            1 => "2 hours ago",
                            _ => "Yesterday",
                        },
                        _ => "Sample",
                    };
                    format!("\"{}\".into()", sample)
                }
                "bool" => "false".to_string(),
                _ => "Default::default()".to_string(),
            };
            format!("{}: {}", f.name, val)
        }).collect();
        let field_str = fields.join(",\n            ");
        items.push(format!(
            "        {} {{\n            {}\n        }}",
            primary_type, field_str
        ));
    }

    let items_str = items.join(",\n");
    format!("vec![\n{}\n    ]", items_str)
}

/// Generate main.rs with Axum server setup, shared state, and initial data.
///
/// Plan 399 第 4-5 步: when `db_full_cover` is true, every non-SSE endpoint
/// delegates to db.rs and holds its own state in once_cell::Lazy globals — so
/// main.rs drops the `State<Db>` seed entirely (no `use api::Db`, no
/// `.with_state(data)`). When false (no db.rs, or db.rs only partially covers
/// endpoints), the legacy `State<Db>` seed path is used.
fn generate_main_rs(
    api_module: &auto_lang::api::ApiModule,
    db_at_content: Option<&str>,
    db_full_cover: bool,
) -> String {
    let routes: Vec<String> = api_module.endpoints.iter()
        .map(|e| {
            let path = e.path();
            let method = e.method().to_lowercase();
            format!("        .route(\"{}\", axum::routing::{}(api::{}))", path, method, e.fn_name)
        })
        .collect();

    let routes_str = routes.join("\n");

    // Plan musk-022: declare events module when SSE endpoints exist.
    let has_sse = api_module.endpoints.iter().any(|e| e.return_type.contains("Stream<"));
    // Plan musk-022 CRUD 智能扩展 第3步: declare db module when db.at has functions.
    let has_db = db_at_content.map(|c| c.contains("pub fn")).unwrap_or(false);

    let mut s = String::new();
    s.push_str("mod api;\n");
    s.push_str("mod types;\n");
    if has_sse {
        s.push_str("mod events;\n");
    }
    if has_db {
        s.push_str("mod db;\n");
    }
    s.push_str("\n");
    if !db_full_cover {
        // Legacy seed-state path: handlers take State<Db>, main injects the seed.
        let initial_data = generate_initial_data_pub(api_module, db_at_content);
        s.push_str("use api::Db;\n");
        s.push_str("use crate::types::*;\n");
        s.push_str("use std::sync::{Arc, Mutex};\n");
        s.push_str("use tower_http::cors::{CorsLayer, Any};\n\n");
        s.push_str("#[tokio::main]\n");
        s.push_str("async fn main() {\n");
        // Resolve the bind port from AUTO_HTTP_PORT (default 8080) so multiple
        // `auto run` instances — or other services sharing the host — can coexist.
        s.push_str("    let port: u16 = std::env::var(\"AUTO_HTTP_PORT\")\n");
        s.push_str("        .ok()\n");
        s.push_str("        .and_then(|v| v.trim().parse().ok())\n");
        s.push_str("        .unwrap_or(8080);\n");
        s.push_str("    let addr = format!(\"127.0.0.1:{}\", port);\n");
        s.push_str("    println!(\"Server running on http://{}\", addr);\n");
        s.push_str("    println!(\"CORS enabled for all origins\");\n\n");
        s.push_str("    // Initial data\n");
        s.push_str(&format!("    let data: Db = Arc::new(Mutex::new({}));\n\n", initial_data));
        s.push_str("    // Enable CORS for frontend development\n");
        s.push_str("    let cors = CorsLayer::new()\n");
        s.push_str("        .allow_origin(Any)\n");
        s.push_str("        .allow_methods(Any)\n");
        s.push_str("        .allow_headers(Any);\n\n");
        s.push_str("    let app = axum::Router::new()\n");
        s.push_str(&format!("{}\n", routes_str));
        s.push_str("        .with_state(data)\n");
        s.push_str("        .layer(cors);\n\n");
    } else {
        // db.rs full-coverage path: no State<Db>, seed lives in db.rs Lazy globals.
        s.push_str("use tower_http::cors::{CorsLayer, Any};\n\n");
        s.push_str("#[tokio::main]\n");
        s.push_str("async fn main() {\n");
        s.push_str("    let port: u16 = std::env::var(\"AUTO_HTTP_PORT\")\n");
        s.push_str("        .ok()\n");
        s.push_str("        .and_then(|v| v.trim().parse().ok())\n");
        s.push_str("        .unwrap_or(8080);\n");
        s.push_str("    let addr = format!(\"127.0.0.1:{}\", port);\n");
        s.push_str("    println!(\"Server running on http://{}\", addr);\n");
        s.push_str("    println!(\"CORS enabled for all origins\");\n\n");
        s.push_str("    let cors = CorsLayer::new()\n");
        s.push_str("        .allow_origin(Any)\n");
        s.push_str("        .allow_methods(Any)\n");
        s.push_str("        .allow_headers(Any);\n\n");
        s.push_str("    let app = axum::Router::new()\n");
        s.push_str(&format!("{}\n", routes_str));
        s.push_str("        .layer(cors);\n\n");
    }
    s.push_str("    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();\n");
    s.push_str("    axum::serve(listener, app).await.unwrap();\n");
    s.push_str("}\n");
    s
}


// ============================================================================
// Lenient API Extraction (Plan 132)
// ============================================================================

/// Extract API definitions leniently - skip unresolvable module references
///
/// This function uses regex-based parsing to extract API definitions without
/// requiring full module resolution. This is useful when `back/api.at` contains
/// `use db` statements where the db module isn't available during extraction.
pub fn extract_api_lenient(api_content: &str) -> Option<ApiModule> {
    use regex::Regex;

    let mut module = ApiModule::new("api".to_string());

    // Extract type definitions using regex
    // Pattern: pub type Name = { fields }
    let type_pattern = Regex::new(r"pub\s+type\s+(\w+)\s*=\s*\{([^}]+)\}").ok()?;

    for cap in type_pattern.captures_iter(api_content) {
        let name = cap.get(1)?.as_str().to_string();
        let fields_str = cap.get(2)?.as_str();

        let fields = parse_fields(fields_str);
        module.types.push(ApiType {
            name,
            fields,
            doc: None,
        });
    }

    // Extract #[api] function definitions
    // Pattern: #[api(...)] pub fn name(params) return_type {
    // Note: return_type may be followed by { or whitespace
    let fn_pattern = Regex::new(
        r#"#\[api\(([^]]*)\]\s*pub\s+fn\s+(\w+)\s*\(([^)]*)\)\s*(\S+)?"#
    ).ok()?;

    for cap in fn_pattern.captures_iter(api_content) {
        let annotation_str = cap.get(1).map(|m| m.as_str()).unwrap_or("");
        let fn_name = cap.get(2)?.as_str().to_string();
        let params_str = cap.get(3).map(|m| m.as_str()).unwrap_or("");
        // Return type may have trailing { which we need to strip
        let return_type_raw = cap.get(4).map(|m| m.as_str()).unwrap_or("void");
        let return_type = return_type_raw.trim_end_matches('{').trim().to_string();
        let return_type = if return_type.is_empty() { "void".to_string() } else { return_type };

        // Extract method from annotation (e.g., method = "GET")
        let method_pattern = Regex::new(r#"method\s*=\s*"(\w+)""#).ok()?;
        let method = method_pattern.captures(annotation_str)
            .and_then(|cap| cap.get(1))
            .map(|m| m.as_str().to_string())
            .unwrap_or_else(|| "GET".to_string());

        // Extract path from annotation (e.g., path = "/api/users")
        let path_pattern = Regex::new(r#"path\s*=\s*"([^"]+)""#).ok()?;
        let path = path_pattern.captures(annotation_str)
            .and_then(|cap| cap.get(1))
            .map(|m| m.as_str().to_string())
            .unwrap_or_else(|| format!("/api/{}", fn_name));

        let params = parse_params(params_str);
        let mut attrs = ApiAttrs::new();
        attrs.method = Some(method);
        attrs.path = Some(path);
        let mut endpoint = ApiEndpoint::new(fn_name.clone(), attrs);
        endpoint.params = params;
        endpoint.return_type = return_type;

        module.endpoints.push(endpoint);
    }

    Some(module)
}

/// Parse type fields from a string like "id: int\nname: str" or the
/// space-separated form "id int\nname str" (Plan 317 supports both).
/// Plan 043 M5 B-4: colon-less fields (e.g. `commands []ToolEntry`,
/// `rows [][]RenderedCell`) were previously dropped, so the generated
/// interface silently lost fields and callers failed with TS2339.
fn parse_fields(fields_str: &str) -> Vec<ApiField> {
    fields_str
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() { return None; }

            // Split on ':' to get name and type (canonical form)
            let parts: Vec<&str> = line.splitn(2, ':').collect();
            let (name, ty) = if parts.len() == 2 && !parts[1].trim().is_empty() {
                (parts[0].trim(), parts[1].trim())
            } else {
                // Colon-less "name type" form: split on the first whitespace.
                // A bare name (no type) is skipped, matching the colon path.
                let mut ws = line.splitn(2, char::is_whitespace);
                match (ws.next(), ws.next()) {
                    (Some(n), Some(t)) => (n, t.trim()),
                    _ => return None,
                }
            };
            if name.is_empty() {
                return None;
            }
            Some(ApiField {
                name: name.to_string(),
                ty: ty.to_string(),
                optional: false,
                default: None,
            })
        })
        .collect()
}

/// Parse function parameters from a string like "id int, name str"
fn parse_params(params_str: &str) -> Vec<ApiParam> {
    if params_str.trim().is_empty() {
        return Vec::new();
    }

    params_str
        .split(',')
        .filter_map(|param| {
            let parts: Vec<&str> = param.trim().split_whitespace().collect();
            if parts.len() >= 2 {
                Some(ApiParam {
                    name: parts[0].to_string(),
                    ty: parts[1].to_string(),
                    optional: false,
                    default: None,
                })
            } else {
                None
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_api_lenient_types() {
        let content = r#"
pub type User = {
    id: int
    name: str
    email: str
}

pub type CreateUserRequest = {
    name: str
    email: str
}
"#;
        let module = extract_api_lenient(content).expect("Should extract");

        assert_eq!(module.types.len(), 2);
        assert_eq!(module.types[0].name, "User");
        assert_eq!(module.types[0].fields.len(), 3);
        assert_eq!(module.types[0].fields[0].name, "id");
        assert_eq!(module.types[0].fields[0].ty, "int");
        assert_eq!(module.types[1].name, "CreateUserRequest");
    }

    #[test]
    fn test_extract_api_lenient_colonless_fields() {
        // Plan 043 M5 B-4: colon-less `name type` fields (valid Auto syntax,
        // e.g. `commands []ToolEntry` / `rows [][]RenderedCell`) must survive
        // lenient extraction — previously only `name: type` lines were kept,
        // silently dropping the rest from the generated interface (TS2339).
        let content = r#"
pub type RenderedOutput = {
    kind: str
    text: str
    rows [][]RenderedCell
    code_lines [][]CodeSpan
}

pub type BootSnapshot = {
    cwd: str
    home: str
    commands []ToolEntry
    smart_commands []SmartCommandEntry
}
"#;
        let module = extract_api_lenient(content).expect("Should extract");
        assert_eq!(module.types.len(), 2);

        let ro = &module.types[0];
        assert_eq!(ro.name, "RenderedOutput");
        let field_names: Vec<&str> = ro.fields.iter().map(|f| f.name.as_str()).collect();
        assert_eq!(field_names, vec!["kind", "text", "rows", "code_lines"]);
        assert_eq!(ro.fields[2].ty, "[][]RenderedCell");
        assert_eq!(ro.fields[3].ty, "[][]CodeSpan");

        let snap = &module.types[1];
        assert_eq!(snap.name, "BootSnapshot");
        let snap_names: Vec<&str> = snap.fields.iter().map(|f| f.name.as_str()).collect();
        assert_eq!(snap_names, vec!["cwd", "home", "commands", "smart_commands"]);
        assert_eq!(snap.fields[2].ty, "[]ToolEntry");
    }

    #[test]
    fn test_extract_api_lenient_endpoints() {
        let content = r#"
#[api(method = "GET", path = "/api/users/:id")]
pub fn getuser(id int) User? {
    use db
    return db.find_user(id)
}

#[api(method = "GET", path = "/api/users")]
pub fn listusers() []User {
    use db
    return db.all_users()
}
"#;
        let module = extract_api_lenient(content).expect("Should extract");

        assert_eq!(module.endpoints.len(), 2);
        assert_eq!(module.endpoints[0].fn_name, "getuser");
        assert_eq!(module.endpoints[0].params.len(), 1);
        assert_eq!(module.endpoints[0].params[0].name, "id");
        assert_eq!(module.endpoints[0].params[0].ty, "int");
        assert_eq!(module.endpoints[0].return_type, "User?");
        // Verify method and path extraction
        assert_eq!(module.endpoints[0].attrs.method, Some("GET".to_string()));
        assert_eq!(module.endpoints[0].attrs.path, Some("/api/users/:id".to_string()));

        assert_eq!(module.endpoints[1].fn_name, "listusers");
        assert_eq!(module.endpoints[1].params.len(), 0);
        assert_eq!(module.endpoints[1].return_type, "[]User");
        // Verify method and path extraction
        assert_eq!(module.endpoints[1].attrs.method, Some("GET".to_string()));
        assert_eq!(module.endpoints[1].attrs.path, Some("/api/users".to_string()));
    }

    #[test]
    fn test_extract_api_lenient_with_create_request() {
        let content = r#"
#[api(method = "POST", path = "/api/users")]
pub fn createuser(req CreateUserRequest) User {
    use db
    let user = db.create_user(req.name, req.email)
    return user
}
"#;
        let module = extract_api_lenient(content).expect("Should extract");

        assert_eq!(module.endpoints.len(), 1);
        assert_eq!(module.endpoints[0].fn_name, "createuser");
        assert_eq!(module.endpoints[0].params.len(), 1);
        assert_eq!(module.endpoints[0].params[0].name, "req");
        assert_eq!(module.endpoints[0].params[0].ty, "CreateUserRequest");
        assert_eq!(module.endpoints[0].return_type, "User");
        // Verify method and path extraction
        assert_eq!(module.endpoints[0].attrs.method, Some("POST".to_string()));
        assert_eq!(module.endpoints[0].attrs.path, Some("/api/users".to_string()));
    }

    #[test]
    fn test_parse_fields() {
        let fields_str = r#"
    id: int
    name: str
    email: str
"#;
        let fields = parse_fields(fields_str);

        assert_eq!(fields.len(), 3);
        assert_eq!(fields[0].name, "id");
        assert_eq!(fields[0].ty, "int");
        assert_eq!(fields[1].name, "name");
        assert_eq!(fields[1].ty, "str");
    }

    #[test]
    fn test_parse_params() {
        let params_str = "id int, name str";
        let params = parse_params(params_str);

        assert_eq!(params.len(), 2);
        assert_eq!(params[0].name, "id");
        assert_eq!(params[0].ty, "int");
        assert_eq!(params[1].name, "name");
        assert_eq!(params[1].ty, "str");
    }

    #[test]
    fn test_parse_params_empty() {
        let params = parse_params("");
        assert!(params.is_empty());

        let params = parse_params("   ");
        assert!(params.is_empty());
    }

    #[test]
    fn test_extract_full_example() {
        // Test with content from the actual api-example file
        let content = r#"
/// User information
pub type User = {
    id: int
    name: str
    email: str
}

/// Create user request
pub type CreateUserRequest = {
    name: str
    email: str
}

/// Get user by ID
#[api(method = "GET", path = "/api/users/:id")]
pub fn getuser(id int) User? {
    use db

    let user = db.find_user(id)
    return user
}

/// List all users
#[api(method = "GET", path = "/api/users")]
pub fn listusers() []User {
    use db

    return db.all_users()
}
"#;
        let module = extract_api_lenient(content).expect("Should extract");

        assert_eq!(module.types.len(), 2, "Should have 2 types");
        assert_eq!(module.endpoints.len(), 2, "Should have 2 endpoints");

        // Check User type
        assert_eq!(module.types[0].name, "User");
        assert_eq!(module.types[0].fields.len(), 3);

        // Check getuser endpoint
        assert_eq!(module.endpoints[0].fn_name, "getuser");
        assert_eq!(module.endpoints[0].return_type, "User?");
        assert_eq!(module.endpoints[0].attrs.method, Some("GET".to_string()));
        assert_eq!(module.endpoints[0].attrs.path, Some("/api/users/:id".to_string()));

        // Check listusers endpoint
        assert_eq!(module.endpoints[1].fn_name, "listusers");
        assert_eq!(module.endpoints[1].return_type, "[]User");
        assert_eq!(module.endpoints[1].attrs.method, Some("GET".to_string()));
        assert_eq!(module.endpoints[1].attrs.path, Some("/api/users".to_string()));
    }

    /// Plan musk-022: SSE endpoint → Sse handler + events bus + cargo deps.
    #[test]
    fn test_sse_handler_generation() {
        let content = r#"
pub type Message = { id: int, text: str }

#[api(method = "GET", path = "/api/messages")]
pub fn list_messages() []Message { return db.all() }

#[api(method = "POST", path = "/api/messages")]
pub fn send_message(text str) Message { return db.create(text) }

#[api(method = "GET", path = "/api/stream")]
pub fn stream() ~Stream<ChatEvent> { return bus.subscribe() }
"#;
        let module = extract_api_lenient(content).expect("Should extract");
        assert!(module.endpoints.iter().any(|e| e.return_type.contains("Stream<")));
        let api_rs = generate_api_rs(&module, None);
        assert!(api_rs.contains("axum::response::Sse<"), "SSE return: {}", api_rs);
        assert!(api_rs.contains("crate::events::subscribe()"), "subscribe");
        assert!(api_rs.contains("async_stream::stream!"), "stream macro");
        assert!(api_rs.contains("crate::events::broadcast("), "broadcast");
        let cargo = generate_cargo_toml("chat-back", true, true);
        assert!(cargo.contains("async-stream"), "dep");
        let events = generate_events_rs();
        assert!(events.contains("pub fn subscribe()"), "subscribe fn");
        assert!(events.contains("pub fn broadcast("), "broadcast fn");
        let main = generate_main_rs(&module, None, false);
        assert!(main.contains("mod events;"), "mod events");
    }

    /// Plan 399 第 4-5 步: 017-chat — handlers delegate to db.rs (no State<Db>),
    /// POST still broadcasts the SSE NewMessage event.
    #[test]
    fn test_handler_calls_db_for_chat() {
        let api = r#"
pub type Message = { id: int, sender: str, text: str, time: str, mine: bool }

#[api(method = "GET", path = "/api/messages")]
pub fn list_messages() []Message { return db.all_messages() }

#[api(method = "POST", path = "/api/messages")]
pub fn send_message(sender str, text str) Message { return db.create_message(sender, text) }

#[api(method = "GET", path = "/api/stream")]
pub fn stream() ~Stream<ChatEvent> { return bus.subscribe() }
"#;
        let module = extract_api_lenient(api).expect("extract api");
        // db.rs exposes all_messages + create_message.
        let db_fns: std::collections::HashSet<String> = [
            "all_messages".to_string(), "create_message".to_string(),
        ].into_iter().collect();

        let api_rs = generate_api_rs(&module, Some(&db_fns));

        // GET list delegates to crate::db::all_messages(), no State<Db>.
        assert!(api_rs.contains("crate::db::all_messages()"), "list delegates: {}", api_rs);
        assert!(
            api_rs.contains("JsonResponse::<Vec<Message>>(crate::db::all_messages())"),
            "list wrap: {}", api_rs
        );
        // POST delegates to crate::db::create_message(&input.sender, &input.text)
        // (str params borrowed: a2r emits &str, extractors hold String).
        assert!(
            api_rs.contains("crate::db::create_message(&input.sender, &input.text)"),
            "create delegates: {}", api_rs
        );
        // POST still broadcasts the SSE event after create.
        assert!(api_rs.contains("crate::events::broadcast("), "broadcast: {}", api_rs);
        // No handler should lock State<Db> now.
        assert!(!api_rs.contains("State<Db>"), "no State<Db>: {}", api_rs);
        assert!(!api_rs.contains("..Default::default()"), "no default fill: {}", api_rs);
    }

    /// Plan 399 第 4-5 步: 015-notes regression — all 9 endpoints delegate to
    /// db.rs (list_notes→all_notes, create_note→create_note, get_note→find_note,
    /// update_note→update_note, delete_note→delete_note, toggle_pin→toggle_pin,
    /// update_tags→update_tags, search_notes→search_notes).
    #[test]
    fn test_handler_calls_db_for_notes_regression() {
        let api = r#"
pub type Note = { id: int, title: str, body: str, time: str, pinned: bool, tags: []str, folder: str }

#[api(method = "GET", path = "/api/notes")]
pub fn list_notes() []Note { return db.all_notes() }

#[api(method = "GET", path = "/api/notes/:id")]
pub fn get_note(id int) ?Note { return db.find_note(id) }

#[api(method = "POST", path = "/api/notes")]
pub fn create_note(title str, body str, folder str) Note { return db.create_note(title, body, folder) }

#[api(method = "PUT", path = "/api/notes/:id")]
pub fn update_note(id int, title str, body str) ?Note { return db.update_note(id, title, body) }

#[api(method = "DELETE", path = "/api/notes/:id")]
pub fn delete_note(id int) bool { return db.delete_note(id) }

#[api(method = "PATCH", path = "/api/notes/:id/pin")]
pub fn toggle_pin(id int) ?Note { return db.toggle_pin(id) }

#[api(method = "PUT", path = "/api/notes/:id/tags")]
pub fn update_tags(id int, tags []str) ?Note { return db.update_tags(id, tags) }

#[api(method = "GET", path = "/api/notes/search")]
pub fn search_notes(query str) []Note { return db.search_notes(query) }
"#;
        let module = extract_api_lenient(api).expect("extract api");
        let db_fns: std::collections::HashSet<String> = [
            "all_notes".to_string(), "find_note".to_string(), "create_note".to_string(),
            "update_note".to_string(), "delete_note".to_string(), "toggle_pin".to_string(),
            "update_tags".to_string(), "search_notes".to_string(),
        ].into_iter().collect();

        let api_rs = generate_api_rs(&module, Some(&db_fns));

        // Every endpoint resolves to its db.rs counterpart.
        for db_fn in &[
            "crate::db::all_notes", "crate::db::find_note", "crate::db::create_note", "crate::db::update_note",
            "crate::db::delete_note", "crate::db::toggle_pin", "crate::db::update_tags", "crate::db::search_notes",
        ] {
            assert!(api_rs.contains(db_fn), "missing {}: {}", db_fn, api_rs);
        }
        // Path params bind directly, body params via input., query via query.
        // str params are borrowed (&input.x / &query.x).
        assert!(api_rs.contains("crate::db::find_note(id)"), "path arg: {}", api_rs);
        assert!(
            api_rs.contains("crate::db::create_note(&input.title, &input.body, &input.folder)"),
            "body args: {}", api_rs
        );
        assert!(api_rs.contains("crate::db::search_notes(&query.query)"), "query arg: {}", api_rs);
        // Option-returning path endpoints map to ok_or(NOT_FOUND).
        assert!(api_rs.contains(".ok_or(StatusCode::NOT_FOUND)"), "404 mapping: {}", api_rs);
        // No State<Db> at all.
        assert!(!api_rs.contains("State<Db>"), "no State<Db>: {}", api_rs);
    }

    /// Plan 399 第 4-5 步: when db.rs fully covers all endpoints, main.rs drops
    /// State<Db> (no `with_state`, no `use api::Db`). Seed lives in db.rs globals.
    #[test]
    fn test_main_rs_no_state_when_db_full_cover() {
        let api = r#"
pub type Note = { id: int, title: str }

#[api(method = "GET", path = "/api/notes")]
pub fn list_notes() []Note { return db.all_notes() }
"#;
        // db.at with at least one pub fn → has_db true.
        let db_at = "use api: Note\npub fn all_notes() []Note { return notes }\n";
        let module = extract_api_lenient(api).expect("extract api");
        let main = generate_main_rs(&module, Some(db_at), true);
        assert!(main.contains("mod db;"), "declare db module: {}", main);
        assert!(!main.contains("with_state"), "no with_state: {}", main);
        assert!(!main.contains("use api::Db;"), "no Db import: {}", main);
        assert!(main.contains("axum::Router::new()"), "router still built: {}", main);

        // And the legacy path is preserved when db_full_cover is false.
        let main_legacy = generate_main_rs(&module, None, false);
        assert!(main_legacy.contains("with_state"), "legacy keeps state: {}", main_legacy);
    }

    /// Plan 399 第 4-5 步: end-to-end on the real 017-chat db.at. Confirms the
    /// transpiled db.rs carries the real `mine: true` business logic (the
    /// original bug returned mine:false), and that extract_db_fn_names recovers
    /// all_messages + create_message so handlers can delegate to them.
    #[test]
    fn test_017_chat_db_rs_has_real_logic_and_fn_names() {
        let db_at = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..").join("..").join("examples")
            .join("ui").join("017-chat").join("src").join("back").join("db.at");
        if !db_at.exists() {
            eprintln!("skipping: 017-chat db.at not found at {:?}", db_at);
            return;
        }
        let content = std::fs::read_to_string(&db_at).unwrap();

        let db_rs = transpile_db_to_rs(&content).expect("db.rs transpiles");
        let db_rs = post_process_db_rs(db_rs);

        // The create_message body must set mine: true (the whole point of the
        // CRUD extension: handler delegation makes this run server-side).
        assert!(db_rs.contains("mine: true"), "real mine:true logic: {}", db_rs);

        // extract_db_fn_names recovers the two functions handlers delegate to.
        let fns = extract_db_fn_names(&db_rs);
        assert!(fns.contains("all_messages"), "all_messages found: {:?}", fns);
        assert!(fns.contains("create_message"), "create_message found: {:?}", fns);

        // And the api.at side wires both endpoints to those db functions.
        let api_at = std::fs::read_to_string(
            db_at.with_file_name("api.at")
        ).unwrap();
        let module = extract_api_lenient(&api_at).expect("api extracts");
        let api_rs = generate_api_rs(&module, Some(&fns));
        assert!(api_rs.contains("crate::db::all_messages()"), "list delegates: {}", api_rs);
        assert!(
            api_rs.contains("crate::db::create_message(&input.sender, &input.text)"),
            "create delegates: {}", api_rs
        );
        assert!(!api_rs.contains("State<Db>"), "no State<Db>: {}", api_rs);

        // Full coverage → main.rs must drop State<Db> (state unified to db.rs).
        let db_at_content = std::fs::read_to_string(&db_at).unwrap();
        let main_rs = generate_main_rs(&module, Some(&db_at_content), true);
        assert!(!main_rs.contains("with_state"), "main drops state: {}", main_rs);
        assert!(main_rs.contains("mod db;"), "main declares db: {}", main_rs);
        assert!(main_rs.contains("mod events;"), "main declares events (SSE): {}", main_rs);
    }
}
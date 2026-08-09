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
    // Plan 399 §7: parse + transpile on a 16MB stack. db.at with deep nesting
    // (for + if/else + multi-field struct literals) overflows Windows's 1MB
    // main-thread stack (parser frames are large; cf. run_autovm lib.rs:341).
    // This mirrors the repo's established pattern. UTF-8-safety fixes in
    // strip_collection_new (post_process) handle the second §7 bug separately.
    // Error is converted to String inside the thread (AutoError is !Send).
    let content = content.to_string();
    let handle = std::thread::Builder::new()
        .stack_size(16 * 1024 * 1024)
        .spawn(move || -> Result<String, String> {
            let mut sink = transpile_rust(AutoStr::from("db"), &content)
                .map_err(|e| format!("Failed to transpile db.at: {}", e))?;
            let rust_code = String::from_utf8(sink.done().map_err(|e| e.to_string())?.to_vec())
                .map_err(|e| format!("Invalid UTF-8 in db.rs output: {}", e))?;
            Ok(rust_code)
        })
        .map_err(|e| format!("Failed to spawn db.rs transpile thread: {}", e))?;
    handle
        .join()
        .map_err(|_| "db.rs transpile thread panicked".to_string())?
        .map_err(|e| e.into())
}

/// Plan musk-022 CRUD 扩展: post-process a2r's db.rs output to fix the known
/// backend-context gaps (a2r transpiles for a generic module; the HTTP backend
/// has specific shape). All rewrites below are mechanical, safe string fixes.
///
/// NOTE (Plan 399 调研 2026-08-07): these are **workarounds**, not redundant —
/// the corresponding a2r root causes are NOT fixed in `trans/rust.rs`. Verified:
/// - `use api:` → `crate::api` : a2r `use_stmt` has no api→types remap
///   (`rust.rs:11490` maps any non-stdlib bare module to `crate::<name>`).
/// - `List<T>.new(EXPR)` wrapper : a2r `call` has no List/Array interception
///   (`rust.rs:3659`); `GenName` emits `List<T>` verbatim.
/// - `&[T]` return over a MutexGuard : a2r `Type::Slice` always emits `&[T]`
///   regardless of return position (`rust.rs:1184`), no owned-Vec rewrite.
/// - str param → String field missing `.to_string()` : no such logic in a2r.
/// Do NOT delete these rewrites unless the a2r root cause is fixed first.
fn post_process_db_rs(mut code: String) -> String {
    code = code.replace("use crate::api::", "use crate::types::");
    // Strip `List<T>.new(EXPR)` -> `EXPR` (a2r leaves the wrapper; List=Vec, the
    // array literal is already vec![...]). Bracket-balanced over the .new(...) parens.
    code = strip_collection_new(&code);
    // Plan 399 Phase 11.3: &[T] return + global clone now handled in a2r
    // (rust_return_type_name emits Vec<T>; write_return_expr adds .clone() for
    // global var returns). Keeping fix_borrowed_slice_returns as a no-op fallback
    // is unnecessary once a2r is verified — comment out to confirm.
    // code = fix_borrowed_slice_returns(&code);
    // Plan 399 §3/P11.6: over-deref before method calls. Generalize the hardcoded
    // MESSAGES to any global + any mutating method: `*VAR.lock().unwrap().METHOD(`
    // → drop `*`. a2r over-dereferences the MutexGuard before a method call
    // (push/insert/extend/etc. need &mut self via DerefMut, not a moved value).
    // P11.6 a2r根治(改 Expr::Ident)回归面广,保留这个后处理正则作为完整覆盖.
    {
        use regex::Regex;
        if let Ok(re) = Regex::new(r"\*(\w+)\.lock\(\)\.unwrap\(\)\.(push|insert|extend|pop|remove|retain|clear|sort_by|sort|swap|truncate|drain|splice|resize)\(") {
            code = re.replace_all(&code, "$1.lock().unwrap().$2(").to_string();
        }
    }
    // Param-to-field &str -> String: a2r passes &str fn params into String struct
    // fields without .to_string(). Regex: `<field>: <param>,` where both are the same
    // bare ident and field is a known String field (skip id/bool/time/count).
    code = append_tostring_for_str_fields(&code);
    // id field type widening: backend types.rs uses i64 for int, but a2r emits i32
    // guards. `id: *NEXTID.lock().unwrap()` -> add `as i64` for the id field.
    code = code.replace("id: *NEXTID.lock().unwrap()", "id: *NEXTID.lock().unwrap() as i64");
    // Plan 399 Phase 11.1: a2r now emits i64 for `int` (rust_type_name Type::Int
    // => i64 + as i64 casts). The blunt code.replace("i32","i64") is no longer
    // needed — comment out to confirm 015/017 still compile.
    // code = code.replace("i32", "i64");
    // Plan 399 §3: a2r emits `let results: Vec<T> = ...` without `mut` but then
    // calls results.push(). Add `mut` to `let NAME:` that is followed (in the same
    // fn) by `NAME.push`. Simple per-line heuristic: `let X = vec![]` / `let X:` → `let mut X`.
    // Plan 399 Phase 11.5: a2r now scans the fn body for mutated `let` bindings
    // (scan_mutated_bindings) and emits `let mut` — this post-process is removed.
    // code = add_mut_to_let_collections(&code);
    // Plan 399 §3: a2r iterates borrowed (`for note in &*G.lock()`) but moves
    // fields out of the shared reference in struct literals (`tags: note.tags`)
    // and returns (`return Some(note)`).
    // Plan 399 Phase 11.4: a2r now clones borrowed-iter field reads in struct
    // ctors (write_expr_for_struct_field). The return-Some(note) case still
    // needs this post-process (a2r doesn't yet clone bare iter-var returns) —
    // keep it until that's covered.
    code = append_clone_for_borrowed_fields(&code);
    code
}

/// Plan 399 §3: in `Type { ..., field: note.x, ... }` struct literals, `note` is
/// a `&Note` (borrowed iterator), so moving `note.tags`/`note.title` errors.
/// Append `.clone()` to every `<ident>.<ident>` read inside struct-ctor fields.
/// Also fix `return Some(<ident>)` over a borrow → `Some(<ident>.clone())`.
fn append_clone_for_borrowed_fields(code: &str) -> String {
    use regex::Regex;
    let mut out = code.to_string();
    // `field: name.attr` (not already cloned, not a method call) → add .clone()
    // Matches `word.word` followed by `,` or ` }` (struct field end). Avoids
    // `word.word(...)` calls and already-`.clone()`d reads.
    let re = match Regex::new(r"(\b\w+:\s*)(\w+)\.(\w+)(,|\s*\})") {
        Ok(r) => r,
        Err(_) => return out,
    };
    out = re.replace_all(&out, "${1}${2}.${3}.clone()${4}").to_string();
    // `return Some(name)` → `return Some(name.clone())` when name is a bare ident
    // (covers `for note in &*G.lock() { ... return Some(note) }`).
    let re2 = match Regex::new(r"return Some\((\w+)\)") {
        Ok(r) => r,
        Err(_) => return out,
    };
    out = re2.replace_all(&out, "return Some($1.clone())").to_string();
    out
}

/// Plan 399 §3: add `mut` to `let NAME = vec![...]` / `let NAME: Vec<...> = ...`
/// when the binding is later mutated (a2r omits `mut`). Conservative: only
/// prefixes `let ` → `let mut ` for lines that look like a fresh Vec binding.
fn add_mut_to_let_collections(code: &str) -> String {
    use regex::Regex;
    let re = match Regex::new(r"(?m)^(\s*)let (\w+)(:\s*Vec<| =\s*vec!\[)") {
        Ok(r) => r,
        Err(_) => return code.to_string(),
    };
    re.replace_all(code, "${1}let mut ${2}${3}").to_string()
}

/// For `Type { field: field, ... }` where `field` is a &str param assigned to a
/// String struct field, append `.to_string()`. Scans for `ident: ident,` pairs
/// (no regex backref — regex crate lacks it) where the ident is a str param.
fn append_tostring_for_str_fields(code: &str) -> String {
    use std::collections::HashSet;
    let mut str_params: HashSet<String> = HashSet::new();
    // Plan 399 §3: slice params (&[String]) assigned to Vec<String> fields need
    // .to_vec() (a2r passes them straight: `tags: tags` where tags: &[String]).
    let mut slice_params: HashSet<String> = HashSet::new();
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
                    // Slice param: `name &[String]` or `name &[...]`.
                    if tok.len() >= 2 && tok[1].starts_with("&[") {
                        slice_params.insert(tok[0].to_string());
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
        // Plan 399 §3: a2r emits `field: field` for str params; add .to_string().
        // Match both `,` (mid-struct) and ` }` (last field, no trailing comma).
        for sep in [",", " }"] {
            let needle = format!("{}: {}{}", name, name, sep);
            let repl = format!("{}: {}.to_string(){}", name, name, sep);
            out = out.replace(&needle, &repl);
        }
    }
    // Plan 399 §3: slice params (`tags: &[String]`) → `field: tags.to_vec()`.
    for name in &slice_params {
        for sep in [",", " }"] {
            let needle = format!("{}: {}{}", name, name, sep);
            let repl = format!("{}: {}.to_vec(){}", name, name, sep);
            out = out.replace(&needle, &repl);
        }
    }
    out
}

/// Strip `List<T>.new(...)` / `Array<T>.new(...)` wrappers, leaving the inner expr.
/// Plan 399 §7: made UTF-8 safe — `List</Array<` are ASCII so byte-indexing for
/// matching is fine, but the byte cursor must skip whole UTF-8 sequences when
/// it lands on a non-ASCII lead byte (e.g. emoji in seed data like 📄), else
/// `code[i..]` slices into a multi-byte char and panics.
fn strip_collection_new(code: &str) -> String {
    let mut out = String::with_capacity(code.len());
    let bytes = code.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        // Only attempt the List/Array match at an ASCII lead byte — landing mid
        // multi-byte char means we just copy the byte run through.
        if bytes[i] < 0x80 && (code[i..].starts_with("List<") || code[i..].starts_with("Array<")) {
            let is_array = code[i..].starts_with("Array<");
            let lt_at = if is_array { i + 5 } else { i + 4 };
            if let Some(gt_rel) = code[lt_at..].find('>') {
                let gt_at = lt_at + gt_rel;
                if code.get(gt_at+1..).map_or(false, |s| s.starts_with(".new(")) {
                    let paren_open = gt_at + 5;
                    if let Some(paren_close) = balance_paren(code, paren_open) {
                        out.push_str(&code[paren_open+1..paren_close]);
                        i = paren_close + 1;
                        continue;
                    }
                }
            }
        }
        // Advance one whole UTF-8 char so the cursor stays on a char boundary.
        let ch_len = utf8_len(bytes[i]);
        out.push_str(&code[i..i + ch_len]);
        i += ch_len;
    }
    out
}

/// Byte length of the UTF-8 char whose lead byte is `b`. Plan 399 §7.
fn utf8_len(b: u8) -> usize {
    if b < 0x80 { 1 }
    else if b < 0xC0 { 1 } // continuation byte (shouldn't be a lead, stay safe)
    else if b < 0xE0 { 2 }
    else if b < 0xF0 { 3 }
    else { 4 }
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
    // Plan 399 §3: generalize the hardcoded MESSAGES/NOTES to any global (e.g.
    // FOLDERS) — `return *<UPPER>.lock().unwrap();` over a guard can't move.
    {
        use regex::Regex;
        if let Ok(re) = Regex::new(r"return \*(\w+)\.lock\(\)\.unwrap\(\);") {
            out = re.replace_all(&out, "return $1.lock().unwrap().clone();").to_string();
        }
    }
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

    // Plan 399 Phase 13 (§10): mixed state (some endpoints delegate to db.rs
    // Lazy, others lock State<Db>) silently diverges — writes to one store are
    // invisible to the other. Fail fast at generation time instead of emitting
    // a server that compiles but corrupts data. Escape hatch for incremental
    // db.at migration: AUTO_ALLOW_PARTIAL_DB=1 keeps the old mixed behavior.
    if has_db && db_fns.as_ref().map(|s| !s.is_empty()).unwrap_or(false) && !db_full_cover {
        let allow_partial = std::env::var("AUTO_ALLOW_PARTIAL_DB")
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
            .unwrap_or(false);
        let uncovered: Vec<&str> = api_module.endpoints.iter()
            .filter(|e| !e.return_type.contains("Stream<"))
            .filter(|e| match db_fns.as_ref() {
                Some(fns) => resolve_db_call(*e, fns).is_none(),
                None => true,
            })
            .map(|e| e.fn_name.as_str())
            .collect();
        if allow_partial {
            eprintln!(
                "  ⚠ PARTIAL db.rs coverage (AUTO_ALLOW_PARTIAL_DB=1): endpoints {:?} \
                 fall back to State<Db> — state WILL diverge (db.rs Lazy ≠ State<Db> seed).",
                uncovered
            );
        } else {
            return Err(format!(
                "db.rs exists but endpoints {:?} have no matching db.rs function. \
                 This produces a mixed-state server (db.rs Lazy vs State<Db>) whose writes \
                 silently diverge. Either (a) add db.rs functions for these endpoints, or \
                 (b) set AUTO_ALLOW_PARTIAL_DB=1 to accept the mixed state during migration.",
                uncovered
            ).into());
        }
    }

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
    // Plan 405: db.at 字符串操作(+ 拼接/contains)会让 a2r 生成 `use a2r_std`
    // (StringBuilder 等), 而 a2r_std 在 auto-lang crate 里 → 必须加 auto-lang
    // 依赖, 否则 `unresolved import a2r_std`。任何用字符串的后端都会触发。
    let db_deps = if has_db { "
auto-lang.workspace = true
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

/// Check if endpoint has a JSON body to extract.
/// Plan 399 §9: PATCH with body params (e.g. `set_pinned(id, pinned bool)`)
/// was previously excluded, which meant no `Json(input)` extractor — but
/// `resolve_db_call` still emitted `&input.X` → unbound `input` → compile error.
/// POST/PUT always have a body (create/update the whole resource); PATCH only
/// has a body when it declares body params (toggle_pin has none → no body).
fn endpoint_has_body(endpoint: &ApiEndpoint) -> bool {
    let method = endpoint.method();
    match method.as_str() {
        "POST" | "PUT" => true,
        "PATCH" => !endpoint_body_params(endpoint).is_empty(),
        _ => false,
    }
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

/// Plan 399 §8: extract the db function name from the endpoint's body AST when
/// it is a thin `return db.FN(...)` delegate (the documented api.at convention).
/// Reads `Stmt::Return(Expr::Call{ name: Dot(Ident("db"), FN) })`. Returns the
/// FN name (verified to exist in db_fns) — far more reliable than the
/// name-heuristic `db_fn_candidates`, which fails on synonyms/plurals/aliases.
fn extract_db_fn_from_body(
    endpoint: &ApiEndpoint,
    db_fns: &std::collections::HashSet<String>,
) -> Option<String> {
    use auto_lang::ast::{Expr, Stmt};
    let body = endpoint.body.as_ref()?;
    // Scan statements for `return db.FN(...)` (the whole-body convention).
    for stmt in &body.stmts {
        if let Stmt::Return(expr) = stmt {
            if let Expr::Call(call) = expr.as_ref() {
                if let Expr::Dot(receiver, method) = call.name.as_ref() {
                    if let Expr::Ident(recv_name) = receiver.as_ref() {
                        if recv_name.as_ref() == "db" && db_fns.contains(method.as_ref()) {
                            return Some(method.as_ref().to_string());
                        }
                    }
                }
            }
        }
    }
    None
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
    // Plan 399 §8: prefer the body's explicit `db.FN(...)` call over the
    // name-heuristic (handles synonyms/plurals/aliases that the heuristic misses).
    let db_fn = extract_db_fn_from_body(endpoint, db_fns)
        .or_else(|| {
            db_fn_candidates(endpoint)
                .into_iter()
                .find(|c| db_fns.contains(c))
        })?;
    let path = endpoint.path();
    let method = endpoint.method();
    let is_str = |ty: &str| {
        let t = ty.trim();
        t == "str" || t == "String" || t == "&str"
    };
    // Plan 399 §3: a2r turns `[]str` into `&[String]` params, but extractors hold
    // `Vec<String>` — borrow (`&input.tags`) so `&Vec<String>` derefs to `&[String]`.
    let is_slice_str = |ty: &str| {
        let t = ty.trim();
        t == "[]str" || t == "[]String" || t == "&[String]"
    };
    let args: Vec<String> = endpoint.params.iter().map(|p| {
        let is_path = path.contains(&format!(":{}", p.name));
        let is_query = !is_path && matches!(method.as_str(), "GET" | "DELETE");
        let borrow = is_str(&p.ty) || is_slice_str(&p.ty);
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

/// Plan 399 §6: infer the SSE broadcast discriminator value for a POST endpoint.
/// The frontend store dispatches by `data.<discriminator_field>` (default field
/// "event"), and the value names the ChatEvent variant to route to. Convention:
/// - POST returning the primary entity (a "create") → `"New{TypeName}"`
///   (e.g. POST /api/messages → Message → "NewMessage").
/// - POST whose fn name contains "typing" (void, signals presence) → `"Typing"`.
/// Returns None when the endpoint should not broadcast.
fn broadcast_event_name(endpoint: &ApiEndpoint, primary_type: &str) -> Option<String> {
    if endpoint.method() != "POST" {
        return None;
    }
    let fn_name = endpoint.fn_name.to_lowercase();
    if fn_name.contains("typing") {
        return Some("Typing".to_string());
    }
    // Default: a create broadcasts "New<Type>".
    Some(format!("New{}", primary_type))
}

/// Generate api.rs with route handlers — full CRUD implementation.
///
/// Plan 399 第 4-5 步: when `db_fns` is `Some`, endpoints whose business logic
/// lives in db.rs (matched by `resolve_db_call`) get a handler that calls
/// `db::FN(...)` directly instead of the `State<Db>` CRUD template. Endpoints
/// not matched fall back to the template (with a warning). `None` keeps the
/// legacy `State<Db>` behavior (e.g. seed-only backends with no db.at).

/// Plan 400 Phase 2: Check if an endpoint's body is a "thin delegation"
/// (`return db.FN(args)` or simple let+return). Thin delegations go through
/// route B; only non-thin bodies with real logic (if/for/while/multi-statement)
/// go through the a2r body-transpilation path.
fn is_thin_delegation(endpoint: &ApiEndpoint) -> bool {
    let body = match &endpoint.body {
        Some(b) => b,
        None => return true,
    };
    // Thin delegation = no control-flow statements (if/for/while/match).
    // A body with only return/let-return is "thin" (goes to route B or CRUD).
    for stmt in &body.stmts {
        match stmt {
            auto_lang::ast::Stmt::If(_) | auto_lang::ast::Stmt::For(_) => return false,
            _ => {}
        }
    }
    true
}

/// Plan 400 Phase 2: transpile body statements via a2r into indented Rust lines.
fn try_transpile_body(
    body: &auto_lang::ast::Body,
    endpoint: &ApiEndpoint,
    api_module: &auto_lang::api::ApiModule,
) -> Result<Vec<String>, String> {
    use auto_lang::trans::rust::RustTrans;
    use auto_val::AutoStr;
    use auto_lang::ast::Type;

    let mut trans = RustTrans::new(AutoStr::from("api_handler"));
    for api_type in &api_module.types {
        let fields: Vec<(&str, Type)> = api_type
            .fields
            .iter()
            .map(|f| (f.name.as_str(), Type::StrOwned))
            .collect();
        trans.register_type(&api_type.name, fields);
    }
    let params: Vec<(AutoStr, Type)> = endpoint
        .params
        .iter()
        .map(|p| {
            let ty = match p.ty.as_str() {
                "int" | "uint" | "u64" | "i64" => Type::Int,
                "bool" => Type::Bool,
                "float" => Type::Float,
                _ => Type::StrOwned,
            };
            (AutoStr::from(p.name.as_str()), ty)
        })
        .collect();
    trans.transpile_body_stmts(body, &params).map_err(|e| e.to_string())
}

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

    // Generate CreateInput struct(s) for POST endpoints with body fields.
    // Plan 399 Phase 12: each unique body-param set gets its own struct (was:
    // only one CreateInput per primary type, so a void POST like set_typing
    // with different params than the create POST got the wrong struct → 422).
    // Mirrors the UpdateInput dedup logic below.
    let mut seen_create_sets: Vec<String> = Vec::new();
    let mut create_struct_for_sig: Vec<(String, String)> = Vec::new(); // (param_sig, struct_name)
    for endpoint in &api_module.endpoints {
        if endpoint.method() == "POST" {
            let body_params = endpoint_body_params(endpoint);
            if !body_params.is_empty() {
                let param_sig: String = body_params.iter()
                    .map(|p| format!("{}:{}", p.name, p.ty))
                    .collect::<Vec<_>>().join(",");
                if seen_create_sets.contains(&param_sig) {
                    continue;
                }
                seen_create_sets.push(param_sig.clone());
                let ep_fn_name = &endpoint.fn_name;
                let struct_name = if seen_create_sets.len() == 1 {
                    format!("Create{}Input", primary_type)
                } else {
                    let suffix: String = ep_fn_name.split('_').map(|s| {
                        let mut c = s.chars();
                        match c.next() {
                            Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                            None => String::new(),
                        }
                    }).collect::<String>();
                    format!("Create{}{}Input", primary_type, suffix)
                };
                create_struct_for_sig.push((param_sig, struct_name.clone()));
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
        // Plan 399 §6: SSE broadcast event name (was hardcoded "NewMessage").
        let bcast_evt = broadcast_event_name(endpoint, &primary_type);

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
                    // Plan 399 Phase 12: pick the CreateInput struct matching this
                    // endpoint's body-param signature (was: always Create{Type}Input).
                    let param_sig: String = body_params.iter()
                        .map(|p| format!("{}:{}", p.name, p.ty))
                        .collect::<Vec<_>>().join(",");
                    let struct_name = create_struct_for_sig.iter()
                        .find(|(sig, _)| sig == &param_sig)
                        .map(|(_, name)| name.clone())
                        .unwrap_or_else(|| format!("Create{}Input", primary_type));
                    params.push(format!("Json(input): Json<{}>", struct_name));
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

        // Plan 400 Phase 2: a2r body transpilation. Non-thin bodies with real
        // logic get transpiled via a2r instead of CRUD template. Disable: AUTO_A2R_BODY=0.
        let a2r_body_enabled = std::env::var("AUTO_A2R_BODY").map(|v| v != "0").unwrap_or(true);
        if a2r_body_enabled && !is_thin_delegation(endpoint) {
            if let Some(body) = &endpoint.body {
                match try_transpile_body(body, endpoint, api_module) {
                    Ok(stmts) => {
                        for s in &stmts {
                            lines.push(s.clone());
                        }
                        lines.push("}".to_string());
                        lines.push("".to_string());
                        continue;
                    }
                    Err(e) => {
                        eprintln!(
                            "  ⚠ endpoint `{}` a2r body failed ({}); fallback to template",
                            fn_name, e
                        );
                    }
                }
            }
        }

        // Plan 399 第 4-5 步: db.rs delegation body. When resolved, the handler
        // body is just `db::FN(args)` (optionally broadcasting an SSE event for
        // POST creates). This replaces the entire State<Db> CRUD template below.
        // `json_inner` holds the inner Rust type (e.g. `Vec<Message>`/`Message`)
        // already computed above; reuse it to wrap the db result.
        if let Some(deleg) = &db_delegation {
            let call = format!("crate::db::{}({})", deleg.db_fn, deleg.args.join(", "));
            if is_void {
                // Plan 399 §6/Phase 12: a void POST that broadcasts (e.g. typing
                // signal) emits a single-value SSE payload. The frontend ChatEvent
                // declares `Typing(str)` (single-value variant), so the broadcast
                // uses a fixed "name" field carrying the typing user's name — the
                // store handler reads `evt.name`. (Object variants like NewMessage
                // broadcast the whole struct, which already matches.)
                if has_sse && bcast_evt.as_deref() == Some("Typing") {
                    lines.push(format!("    {};", call));
                    // Build {"event":"Typing","name": <first str body param>}.
                    // The typing endpoint conventionally takes a single `sender`
                    // str param naming who is typing.
                    let name_field = endpoint_body_params(endpoint)
                        .first().map(|p| p.name.as_str()).unwrap_or("sender");
                    lines.push(format!(
                        "    let evt = serde_json::json!({{ \"event\": \"Typing\", \"name\": input.{} }});",
                        name_field
                    ));
                    lines.push("    crate::events::broadcast(evt.to_string());".to_string());
                    lines.push("    StatusCode::OK".to_string());
                } else if needs_result {
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
                // Plan 399 §6: event name is New{Type} (was hardcoded "NewMessage").
                lines.push(format!("    let item = {};", call));
                lines.push("    let mut evt = serde_json::to_value(&item).unwrap_or_default();".to_string());
                lines.push(format!(
                    "    if let Some(obj) = evt.as_object_mut() {{ obj.insert(\"event\".to_string(), serde_json::Value::String(\"{}\".to_string())); }}",
                    bcast_evt.as_deref().unwrap_or("NewMessage")
                ));
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
                    lines.push(format!(
                        "    if let Some(obj) = evt.as_object_mut() {{ obj.insert(\"event\".to_string(), serde_json::Value::String(\"{}\".to_string())); }}",
                        bcast_evt.as_deref().unwrap_or("NewMessage")
                    ));
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

    /// Plan 399 §9: PATCH endpoint WITH body params must get a Json(input)
    /// extractor (previously endpoint_has_body excluded PATCH → unbound input).
    /// PATCH WITHOUT body (toggle_pin) must still get no Json extractor.
    #[test]
    fn test_patch_with_body_gets_json_extractor() {
        let api = r#"
pub type Task = { id: int, title: str, done: bool }

#[api(method = "PATCH", path = "/api/tasks/:id")]
pub fn set_done(id int, done bool) ?Task { return db.set_done(id, done) }

#[api(method = "PATCH", path = "/api/tasks/:id/pin")]
pub fn toggle_pin(id int) ?Task { return db.toggle_pin(id) }
"#;
        let module = extract_api_lenient(api).expect("extract");
        let db_fns: std::collections::HashSet<String> = [
            "set_done".to_string(), "toggle_pin".to_string(),
        ].into_iter().collect();
        let api_rs = generate_api_rs(&module, Some(&db_fns));

        // PATCH+body (set_done): must have Json extractor AND delegate &input.done.
        assert!(
            api_rs.contains("Json(input): Json<") && api_rs.contains("set_done"),
            "PATCH+body has Json extractor: {}", api_rs
        );
        assert!(
            api_rs.contains("crate::db::set_done(id, input.done)"),
            "PATCH+body delegates bool param (no borrow): {}", api_rs
        );

        // PATCH no body (toggle_pin): must NOT have a Json extractor (only Path).
        // Match "Json(input)" specifically — the return type JsonResponse also contains "Json".
        let toggle_sig = api_rs.lines()
            .find(|l| l.contains("pub async fn toggle_pin"))
            .unwrap_or_else(|| panic!("toggle_pin handler missing: {}", api_rs));
        assert!(!toggle_sig.contains("Json(input)"), "PATCH no-body has no Json extractor: {}", toggle_sig);
        assert!(toggle_sig.contains("Path(id)"), "PATCH no-body has Path: {}", toggle_sig);
    }

    /// Plan 399 §6: SSE broadcast event name is no longer hardcoded. A create
    /// POST broadcasts "New{Type}", a typing POST (void, fn name has "typing")
    /// broadcasts "Typing" with the input payload.
    #[test]
    fn test_sse_broadcast_event_name_not_hardcoded() {
        let api = r#"
pub type Message = { id: int, text: str }

#[api(method = "POST", path = "/api/messages")]
pub fn send_message(text str) Message { return db.create(text) }

#[api(method = "POST", path = "/api/typing")]
pub fn set_typing(sender str) { return db.set_typing(sender) }

#[api(method = "GET", path = "/api/stream")]
pub fn stream() ~Stream<ChatEvent> { return bus.subscribe() }
"#;
        let module = extract_api_lenient(api).expect("extract");
        let db_fns: std::collections::HashSet<String> = [
            "create".to_string(), "set_typing".to_string(),
        ].into_iter().collect();
        let api_rs = generate_api_rs(&module, Some(&db_fns));

        // create POST broadcasts "NewMessage" (was hardcoded before §6).
        assert!(api_rs.contains("\"NewMessage\""), "create broadcasts NewMessage: {}", api_rs);
        // typing POST (void) broadcasts "Typing" with the name field (Phase 12
        // protocol: single-value variant uses a fixed "name" field, not &input).
        assert!(api_rs.contains("\"Typing\""), "typing broadcasts Typing: {}", api_rs);
        assert!(
            api_rs.contains("serde_json::json!") && api_rs.contains("\"name\": input.sender"),
            "typing broadcasts json! with name field: {}", api_rs
        );
    }

    /// Plan 399 §8: db fn resolution prefers the body's `db.FN(...)` call over
    /// the name-heuristic. A synonym endpoint (lookup, not in the verb whitelist)
    /// delegates correctly via the body — the heuristic alone would miss it.
    #[test]
    fn test_db_fn_resolved_from_body_over_heuristic() {
        let api = r#"
pub type User = { id: int, name: str }

#[api(method = "GET", path = "/api/users/:id")]
pub fn lookup(id int) ?User { return db.find_user(id) }
"#;
        // full_parse recovers the body (the `return db.find_user(id)` AST).
        let module = try_full_parse(api).expect("full_parse");
        assert_eq!(module.endpoints.len(), 1);
        assert!(module.endpoints[0].body.is_some(), "body captured");
        // db.rs has find_user. The endpoint fn name "lookup" is NOT in the
        // heuristic verb whitelist, so db_fn_candidates would fail — but the
        // body-based resolver finds find_user directly.
        let db_fns: std::collections::HashSet<String> = ["find_user".to_string()].into_iter().collect();
        let api_rs = generate_api_rs(&module, Some(&db_fns));
        assert!(
            api_rs.contains("crate::db::find_user(id)"),
            "synonym endpoint delegates via body: {}", api_rs
        );
        assert!(!api_rs.contains("State<Db>"), "no State<Db>: {}", api_rs);
    }

    /// Plan 399 Phase 13 (§10): mixed-state detection. When db.rs exists but
    /// doesn't cover all endpoints, `all_endpoints_covered` returns false and
    /// the uncovered endpoints can be collected (generate_rust_server then
    /// hard-errors unless AUTO_ALLOW_PARTIAL_DB=1). This test exercises the
    /// detection logic without the full generate_api write path.
    #[test]
    fn test_mixed_state_detection_collects_uncovered() {
        let api = r#"
pub type Note = { id: int, title: str }

#[api(method = "GET", path = "/api/notes")]
pub fn list_notes() []Note { return db.all_notes() }

#[api(method = "POST", path = "/api/notes")]
pub fn create_note(title str) Note { return db.create_note(title) }

#[api(method = "POST", path = "/api/notes/duplicate")]
pub fn duplicate(id int) Note { return db.clone_note(id) }
"#;
        let module = try_full_parse(api).expect("full_parse");
        // db.rs has all_notes + create_note but NOT clone_note (duplicate's body).
        let db_fns: std::collections::HashSet<String> = [
            "all_notes".to_string(), "create_note".to_string(),
        ].into_iter().collect();
        // Not full cover — duplicate is uncovered.
        assert!(!all_endpoints_covered(&module, Some(&db_fns)),
            "duplicate endpoint should make coverage partial");
        // Collect uncovered (same logic as generate_rust_server's hard check).
        let uncovered: Vec<&str> = module.endpoints.iter()
            .filter(|e| !e.return_type.contains("Stream<"))
            .filter(|e| resolve_db_call(e, &db_fns).is_none())
            .map(|e| e.fn_name.as_str())
            .collect();
        assert_eq!(uncovered, vec!["duplicate"], "only duplicate is uncovered: {:?}", uncovered);
    }

    /// Plan 399 §7: regenerate the REAL 015-notes backend (default stack) and
    /// assert it now uses full db-delegation. Marked #[ignore] because it
    /// writes the shared workspace on disk. Previously this overflowed; the fix
    /// (transpile_db_to_rs runs on a 16MB stack + strip_collection_new UTF-8
    /// safe) makes 015's complex db.at (12 fns + for/if-else + emoji seeds)
    /// transpile and generate cleanly.
    #[test]
    #[ignore]
    fn regen_real_015_backend_db_delegation() {
        let project = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..").join("..").join("examples").join("ui").join("015-notes");
        if !project.exists() {
            eprintln!("skip: 015-notes not found");
            return;
        }
        // stage A: api.at extracts (default stack — parse_api is shallow).
        let api_content = std::fs::read_to_string(project.join("src").join("back").join("api.at"))
            .expect("api.at");
        let module = try_full_parse(&api_content)
            .or_else(|| extract_api_lenient(&api_content))
            .expect("api extracts");
        assert!(module.endpoints.len() >= 8, "8 endpoints: {}", module.endpoints.len());

        // stage B: full generate (transpile_db_to_rs has its own 16MB stack).
        generate_api(&project, "rust").expect("generate_api 015 ok");

        let ws = crate::rust_ui::ensure_shared_workspace(&project);
        let back = crate::rust_ui::back_member_name(&project);
        let api_rs = std::fs::read_to_string(ws.join(&back).join("src").join("api.rs"))
            .expect("api.rs exists");
        let main_rs = std::fs::read_to_string(ws.join(&back).join("src").join("main.rs"))
            .expect("main.rs exists");
        // Full db-delegation (no State<Db>).
        assert!(!api_rs.contains("State<Db>"), "no State<Db>: {}", api_rs);
        assert!(api_rs.contains("crate::db::all_notes"), "list delegates: {}", api_rs);
        assert!(api_rs.contains("crate::db::create_note"), "create delegates: {}", api_rs);
        // []str param (update_tags) — borrowed (&input.tags): a2r emits &[String]
        // params, extractors hold Vec<String>, &Vec derefs to &[String].
        assert!(
            api_rs.contains("crate::db::update_tags(id, &input.tags)"),
            "[]str param borrowed: {}", api_rs
        );
        assert!(!main_rs.contains("with_state"), "main no with_state: {}", main_rs);
        assert!(main_rs.contains("mod db;"), "main declares db: {}", main_rs);
    }

    /// Plan 400 Phase 2: a non-thin-delegation body (contains real logic like
    /// if/else, not just `return db.FN(...)`) should be transpiled via a2r and
    /// injected into the handler — NOT fall through to the CRUD template.
    #[test]
    fn test_a2r_body_non_thin_delegation() {
        let api = r#"
pub type Item = { id: int, name: str }

#[api(method = "GET", path = "/api/items/:id")]
pub fn get_item(id int) Item {
    if id > 0 {
        return Item { id: id, name: "positive" }
    }
    return Item { id: id, name: "zero" }
}
"#;
        let module = try_full_parse(api).expect("full_parse");
        assert_eq!(module.endpoints.len(), 1);
        // The body has an if-statement → NOT a thin delegation.
        assert!(!is_thin_delegation(&module.endpoints[0]), "if-body is non-thin");
        // No db.rs → no delegation possible.
        let api_rs = generate_api_rs(&module, None);
        // The a2r path should have transpiled the if-statement into the handler.
        // Look for evidence: "if" keyword from the transpiled body (not the
        // CRUD template which has no if-statements).
        assert!(
            api_rs.contains("if id") || api_rs.contains("if (id"),
            "a2r body transpiled if-statement into handler:\n{}",
            api_rs
        );
        // Should NOT contain the CRUD template's `db.lock()` (that's the
        // fallback path which we bypassed).
        assert!(
            !api_rs.contains("db.lock()"),
            "non-thin body should not use CRUD template:\n{}",
            api_rs
        );
    }

    /// Plan 400 Phase 2: AUTO_A2R_BODY=0 disables the a2r body path, falling
    /// back to CRUD template even for non-thin bodies.
    #[test]
    fn test_a2r_body_disabled_falls_back() {
        std::env::set_var("AUTO_A2R_BODY", "0");
        let api = r#"
pub type Item = { id: int, name: str }

#[api(method = "GET", path = "/api/items/:id")]
pub fn get_item(id int) Item {
    if id > 0 { return Item { id: id, name: "positive" } }
    return Item { id: id, name: "zero" }
}
"#;
        let module = try_full_parse(api).expect("full_parse");
        let api_rs = generate_api_rs(&module, None);
        std::env::remove_var("AUTO_A2R_BODY");
        // With a2r disabled, falls back to CRUD template.
        assert!(
            api_rs.contains("db.lock()"),
            "AUTO_A2R_BODY=0 should use CRUD template:\n{}",
            api_rs
        );
    }
}
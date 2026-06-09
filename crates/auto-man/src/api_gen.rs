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
        _ => {
            // No API generation for other backends
        }
    }

    Ok(())
}

/// Try to parse API file with full AST parsing
fn try_full_parse(api_content: &str) -> Option<ApiModule> {
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
fn generate_rust_server(api_module: &auto_lang::api::ApiModule, root_dir: &Path) -> AutoResult<()> {
    let rust_dir = root_dir.join("gen").join("back").join("rust");
    let src_dir = rust_dir.join("src");
    std::fs::create_dir_all(&src_dir)
        .map_err(|e| format!("Failed to create rust/src: {}", e))?;

    // Generate Cargo.toml
    let cargo_toml = generate_cargo_toml();
    std::fs::write(rust_dir.join("Cargo.toml"), &cargo_toml)
        .map_err(|e| format!("Failed to write Cargo.toml: {}", e))?;

    // Generate types.rs
    let types_rs = generate_types_rs(api_module);
    std::fs::write(src_dir.join("types.rs"), &types_rs)
        .map_err(|e| format!("Failed to write types.rs: {}", e))?;

    // Generate api.rs with route handlers
    let api_rs = generate_api_rs(api_module);
    std::fs::write(src_dir.join("api.rs"), &api_rs)
        .map_err(|e| format!("Failed to write api.rs: {}", e))?;

    // Generate main.rs
    let main_rs = generate_main_rs(api_module);
    std::fs::write(src_dir.join("main.rs"), &main_rs)
        .map_err(|e| format!("Failed to write main.rs: {}", e))?;

    println!("  ✓ Generated Rust server: rust/");

    Ok(())
}

/// Generate Cargo.toml for the Rust server
fn generate_cargo_toml() -> String {
    r#"[package]
name = "api-server"
version = "0.1.0"
edition = "2021"

[dependencies]
axum = "0.7"
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tower-http = { version = "0.5", features = ["cors"] }

[workspace]
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
fn primary_type_name(api_module: &auto_lang::api::ApiModule) -> Option<String> {
    api_module.types.first().map(|t| t.name.clone())
}

/// Get body params (params that aren't path params)
fn endpoint_body_params(endpoint: &ApiEndpoint) -> Vec<&ApiParam> {
    let path = endpoint.path();
    endpoint.params.iter().filter(|p| {
        !path.contains(&format!(":{}", p.name))
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

/// Generate api.rs with route handlers — full CRUD implementation
fn generate_api_rs(api_module: &auto_lang::api::ApiModule) -> String {
    let mut lines = vec![
        "use axum::{".to_string(),
        "    extract::{Path, State, Json},".to_string(),
        "    http::StatusCode,".to_string(),
        "    Json as JsonResponse,".to_string(),
        "};".to_string(),
        "use crate::types::*;".to_string(),
        "use std::sync::{Arc, Mutex};".to_string(),
        "".to_string(),
    ];

    // Determine primary type and generate Db type alias
    let primary_type = match primary_type_name(api_module) {
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

    // Get type field names for time detection
    let type_fields: Vec<&str> = api_module.types.iter()
        .find(|t| t.name == primary_type)
        .map(|t| t.fields.iter().map(|f| f.name.as_str()).collect())
        .unwrap_or_default();
    let has_time_field = type_fields.contains(&"time");
    // Convention: first field is the ID field
    let id_field = type_fields.first().copied().unwrap_or("id");

    // Generate handler for each endpoint
    for endpoint in &api_module.endpoints {
        let method = endpoint.method();
        let fn_name = &endpoint.fn_name;
        let has_path = has_path_param(&endpoint.path());

        // Build function parameters
        let mut params = vec![];
        if has_path {
            let path_params = endpoint_path_params(endpoint);
            if let Some(first) = path_params.first() {
                let rust_type = auto_type_to_rust(&first.ty);
                params.push(format!("Path({}): Path<{}>", first.name, rust_type));
            }
        }
        params.push("State(db): State<Db>".to_string());
        if endpoint_has_body(endpoint) {
            if method == "POST" {
                let body_params = endpoint_body_params(endpoint);
                if !body_params.is_empty() {
                    params.push(format!("Json(input): Json<Create{}Input>", primary_type));
                } else {
                    params.push(format!("Json(input): Json<{}>", primary_type));
                }
            } else {
                // PUT uses the full type
                params.push(format!("Json(input): Json<{}>", primary_type));
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

        // Generate handler body based on CRUD operation
        match method.as_str() {
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
                    lines.push("    };".to_string());
                }
                lines.push("    items.push(item.clone());".to_string());
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

/// Generate initial sample data for the primary type
fn generate_initial_data(api_module: &auto_lang::api::ApiModule) -> String {
    let primary_type = match primary_type_name(api_module) {
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

/// Generate main.rs with Axum server setup, shared state, and initial data
fn generate_main_rs(api_module: &auto_lang::api::ApiModule) -> String {
    let routes: Vec<String> = api_module.endpoints.iter()
        .map(|e| {
            let path = e.path();
            let method = e.method().to_lowercase();
            format!("        .route(\"{}\", axum::routing::{}(api::{}))", path, method, e.fn_name)
        })
        .collect();

    let initial_data = generate_initial_data(api_module);
    let routes_str = routes.join("\n");

    let mut s = String::new();
    s.push_str("mod api;\n");
    s.push_str("mod types;\n\n");
    s.push_str("use api::Db;\n");
    s.push_str("use crate::types::*;\n");
    s.push_str("use std::sync::{Arc, Mutex};\n");
    s.push_str("use tower_http::cors::{CorsLayer, Any};\n\n");
    s.push_str("#[tokio::main]\n");
    s.push_str("async fn main() {\n");
    s.push_str("    println!(\"Server running on http://127.0.0.1:8080\");\n");
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
    s.push_str("    let listener = tokio::net::TcpListener::bind(\"127.0.0.1:8080\").await.unwrap();\n");
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
fn extract_api_lenient(api_content: &str) -> Option<ApiModule> {
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

/// Parse type fields from a string like "id: int\nname: str"
fn parse_fields(fields_str: &str) -> Vec<ApiField> {
    fields_str
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() { return None; }

            // Split on ':' to get name and type
            let parts: Vec<&str> = line.splitn(2, ':').collect();
            if parts.len() == 2 {
                let name = parts[0].trim().to_string();
                let ty = parts[1].trim().to_string();
                Some(ApiField {
                    name,
                    ty,
                    optional: false,
                    default: None,
                })
            } else {
                None
            }
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
}

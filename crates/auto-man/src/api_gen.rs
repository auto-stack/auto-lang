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

/// Generate API code for the project
///
/// This is the main entry point for API code generation.
/// It reads the backend API definitions and generates:
/// - Backend: Tauri commands or Axum routes
/// - Frontend: TypeScript types and API client
pub fn generate_api(root_dir: &Path, backend: &str) -> AutoResult<()> {
    let back_dir = root_dir.join("back");

    // Check if back/api.at exists
    let api_file = back_dir.join("api.at");
    if !api_file.exists() {
        // No API file, skip generation
        return Ok(());
    }

    // Read API file
    let api_content = std::fs::read_to_string(&api_file)
        .map_err(|e| format!("Failed to read {}: {}", api_file.display(), e))?;

    // Parse API definitions using auto-lang's API module
    use auto_lang::api::ApiExtractor;

    // Parse the API file (may fail if there are module references like `use db`)
    // In that case, we try to extract what we can
    let mut parser = auto_lang::Parser::from(&api_content);
    let ast = match parser.parse() {
        Ok(ast) => ast,
        Err(e) => {
            // Try lenient parsing - just extract function signatures
            // For now, just skip API generation if parsing fails
            println!("  ⚠ Could not parse API file (module references not supported yet)");
            println!("    Error: {}", e);
            return Ok(());
        }
    };

    // Extract API module
    let extractor = ApiExtractor::new();
    let api_module = extractor.extract("api", &ast.stmts);

    // Check if any endpoints were extracted
    if api_module.endpoints.is_empty() {
        println!("  ⚠ No API endpoints found in api.at");
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

/// Generate Tauri API code
fn generate_tauri_api(api_module: &auto_lang::api::ApiModule, root_dir: &Path) -> AutoResult<()> {
    use auto_lang::api::Target;

    let vue_dir = root_dir.join("vue");
    let tauri_src_dir = vue_dir.join("src-tauri").join("src");

    // Ensure directories exist
    std::fs::create_dir_all(&tauri_src_dir)
        .map_err(|e| format!("Failed to create directory: {}", e))?;

    // Generate Tauri commands
    let tauri_gen = Target::Tauri.generator();
    let tauri_code = tauri_gen.generate(api_module);
    std::fs::write(tauri_src_dir.join("commands.rs"), &tauri_code)
        .map_err(|e| format!("Failed to write commands.rs: {}", e))?;

    // Generate TypeScript client
    let ts_gen = Target::TypeScript.generator();
    let ts_code = ts_gen.generate(api_module);

    let api_dir = vue_dir.join("src").join("api");
    std::fs::create_dir_all(&api_dir)
        .map_err(|e| format!("Failed to create api directory: {}", e))?;
    std::fs::write(api_dir.join("client.ts"), &ts_code)
        .map_err(|e| format!("Failed to write client.ts: {}", e))?;

    println!("  ✓ Generated Tauri commands: src-tauri/src/commands.rs");
    println!("  ✓ Generated TypeScript client: src/api/client.ts");

    Ok(())
}

/// Generate Vue + HTTP API code
fn generate_vue_api(api_module: &auto_lang::api::ApiModule, root_dir: &Path) -> AutoResult<()> {
    use auto_lang::api::Target;

    let vue_dir = root_dir.join("vue");

    // Generate TypeScript client with HTTP backend
    let ts_gen = Target::TypeScript.generator();
    let ts_code = ts_gen.generate(api_module);

    let api_dir = vue_dir.join("src").join("api");
    std::fs::create_dir_all(&api_dir)
        .map_err(|e| format!("Failed to create api directory: {}", e))?;
    std::fs::write(api_dir.join("client.ts"), &ts_code)
        .map_err(|e| format!("Failed to write client.ts: {}", e))?;

    println!("  ✓ Generated TypeScript client: src/api/client.ts");

    // Note: For Vue + HTTP mode, the backend server is a separate project
    // Users should run `cargo run` in the back/ directory

    Ok(())
}

//! Tauri Backend Generator (Plan 151)
//!
//! This module generates a complete Rust backend crate from AutoLang source files,
//! specifically for Tauri IPC mode. It transpiles `api.at` + `db.at` to Rust,
//! creating an independent `rust/` crate that integrates with Tauri's thin shell.
//!
//! ## Architecture
//!
//! ```text
//! api-example/
//! ├── back/                    # Auto source code
//! │   ├── api.at               # API interface definitions
//! │   └── db.at                # Database operations + global state
//! │
//! └── rust/                    # Generated Rust crate
//!     ├── Cargo.toml           # Dependencies: serde, once_cell, tauri (optional)
//!     └── src/
//!         ├── lib.rs           # Module exports
//!         ├── types.rs         # User, CreateUserRequest types
//!         ├── db.rs            # Global state + database functions
//!         └── commands.rs      # #[tauri::command] wrappers
//! ```
//!
//! ## Transpilation Mapping
//!
//! ### Types
//!
//! | Auto | Rust |
//! |------|------|
//! | `type User = { id: int, name: str, email: str }` | `#[derive(Serialize, Deserialize)] pub struct User { pub id: i32, pub name: String, pub email: String }` |
//! | `?User` | `Option<User>` |
//! | `[]User` | `Vec<User>` |
//!
//! ### Global State
//!
//! | Auto | Rust |
//! |------|------|
//! | `var users List<User> = List<User>.new([...])` | `static USERS: Lazy<Mutex<Vec<User>>> = Lazy::new(\|\| Mutex::new(vec![...]));` |
//! | `var nextid int = 4` | `static NEXT_ID: Lazy<Mutex<i32>> = Lazy::new(\|\| Mutex::new(4));` |
//!
//! ### Functions
//!
//! | Auto | Rust |
//! |------|------|
//! | `pub fn find_user(id int) ?User` | `pub fn find_user(id: i32) -> Option<User>` |
//! | `return Some(user)` / `return None` | `return Some(user);` / `return None;` |
//! | `#[api(method = "GET", path = "/api/users/:id")]` | `#[tauri::command]` |

use std::path::Path;
use std::fs;

use crate::AutoResult;

/// Generate complete Tauri backend Rust crate from AutoLang source
///
/// This is the main entry point for Tauri backend generation.
/// It transpiles `back/api.at` and `back/db.at` to a complete Rust crate.
pub fn generate_tauri_backend(root_dir: &Path) -> AutoResult<()> {
    let back_dir = root_dir.join("src").join("back");
    let rust_dir = root_dir.join("gen").join("back").join("rust");

    // Check if back/ directory exists
    if !back_dir.exists() {
        return Err("back/ directory not found".into());
    }

    // Read api.at and db.at
    let api_content = read_at_file(&back_dir, "api.at")?;
    let db_content = read_at_file(&back_dir, "db.at")?;

    // Parse and transpile using a2r transpiler
    let _api_rust = transpile_at_to_rust(&api_content)?;
    let db_rust = transpile_at_to_rust(&db_content)?;

    // Extract types and commands from api.at
    let types_rs = extract_types(&api_content)?;
    let commands_rs = extract_commands(&api_content)?;

    // Create rust/ directory structure
    let src_dir = rust_dir.join("src");
    fs::create_dir_all(&src_dir)
        .map_err(|e| format!("Failed to create rust/src: {}", e))?;

    // Generate Cargo.toml
    let cargo_toml = generate_cargo_toml();
    fs::write(rust_dir.join("Cargo.toml"), &cargo_toml)
        .map_err(|e| format!("Failed to write Cargo.toml: {}", e))?;

    // Generate lib.rs
    let lib_rs = generate_lib_rs();
    fs::write(src_dir.join("lib.rs"), &lib_rs)
        .map_err(|e| format!("Failed to write lib.rs: {}", e))?;

    // Generate types.rs
    fs::write(src_dir.join("types.rs"), &types_rs)
        .map_err(|e| format!("Failed to write types.rs: {}", e))?;

    // Generate db.rs (transpiled from db.at)
    fs::write(src_dir.join("db.rs"), &db_rust)
        .map_err(|e| format!("Failed to write db.rs: {}", e))?;

    // Generate commands.rs
    fs::write(src_dir.join("commands.rs"), &commands_rs)
        .map_err(|e| format!("Failed to write commands.rs: {}", e))?;

    println!("  ✓ Generated Tauri backend crate: rust/");
    println!("    - Cargo.toml");
    println!("    - src/lib.rs");
    println!("    - src/types.rs");
    println!("    - src/db.rs");
    println!("    - src/commands.rs");

    Ok(())
}

/// Read an .at file from back/ directory
fn read_at_file(back_dir: &Path, name: &str) -> AutoResult<String> {
    let path = back_dir.join(name);
    if !path.exists() {
        return Err(format!("{} not found", name).into());
    }
    fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read {}: {}", name, e).into())
}

/// Transpile AutoLang code to Rust using a2r transpiler
fn transpile_at_to_rust(content: &str) -> AutoResult<String> {
    use auto_lang::trans::rust::transpile_rust;
    use auto_val::AutoStr;

    let mut sink = transpile_rust(AutoStr::from("db"), content)
        .map_err(|e| format!("Failed to transpile: {}", e))?;

    let rust_code = String::from_utf8(sink.done()?.to_vec())
        .map_err(|e| format!("Invalid UTF-8 in output: {}", e))?;

    Ok(rust_code)
}

/// Extract type definitions from api.at
fn extract_types(_content: &str) -> AutoResult<String> {
    // For now, return a simple placeholder
    // TODO: Implement proper type extraction from AST
    Ok(r#"
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserRequest {
    pub name: String,
    pub email: String,
}
"#.to_string())
}

/// Extract Tauri commands from api.at
fn extract_commands(_content: &str) -> AutoResult<String> {
    // For now, return a simple placeholder
    // TODO: Implement proper command extraction from AST
    Ok(r#"
use crate::db;
use crate::types::*;
use tauri::command;

#[command]
pub fn getuser(id: i32) -> Option<User> {
    db::find_user(id)
}

#[command]
pub fn listusers() -> Vec<User> {
    db::all_users()
}

#[command]
pub fn createuser(req: CreateUserRequest) -> User {
    db::create_user(req.name, req.email)
}

#[command]
pub fn deleteuser(id: i32) -> bool {
    db::delete_user(id)
}

#[command]
pub fn searchusers(query: String) -> Vec<User> {
    db::search_users(query)
}
"#.to_string())
}

/// Generate Cargo.toml for the backend crate
fn generate_cargo_toml() -> String {
    r#"[package]
name = "api-example-backend"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { version = "1", features = ["derive"] }
once_cell = "1.19"
tauri = { version = "2", optional = true }

[features]
default = []
tauri = ["dep:tauri"]
"#.to_string()
}

/// Generate lib.rs module exports
fn generate_lib_rs() -> String {
    r#"mod types;
mod db;
#[cfg(feature = "tauri")]
mod commands;

pub use types::*;
pub use db::*;
#[cfg(feature = "tauri")]
pub use commands::*;
"#.to_string()
}

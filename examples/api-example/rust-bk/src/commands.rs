//! Tauri IPC Commands
//!
//! Auto-generated from: examples/api-example/back/api.at
//!
//! This module provides #[tauri::command] wrappers for the database functions.
//! Each command is a thin wrapper that calls the corresponding db:: function.

use crate::db;
use crate::types::*;
use tauri::command;

// ============================================================================
// User Commands
// ============================================================================

/// Get user by ID
///
/// Auto source:
/// ```auto
/// #[api(method = "GET", path = "/api/users/:id")]
/// pub fn getuser(id int) User {
///     use db
///     let user = db.find_user(id)
///     return user
/// }
/// ```
#[command]
pub fn getuser(id: i32) -> Option<User> {
    db::find_user(id)
}

/// List all users
///
/// Auto source:
/// ```auto
/// #[api(method = "GET", path = "/api/users")]
/// pub fn listusers() []User {
///     use db
///     return db.all_users()
/// }
/// ```
#[command]
pub fn listusers() -> Vec<User> {
    db::all_users()
}

/// Create a new user
///
/// Auto source:
/// ```auto
/// #[api(method = "POST", path = "/api/users")]
/// pub fn createuser(req CreateUserRequest) User {
///     use db
///     let user = db.create_user(req.name, req.email)
///     return user.await
/// }
/// ```
#[command]
pub fn createuser(name: String, email: String) -> User {
    db::create_user(name, email)
}

/// Delete a user by ID
///
/// Auto source:
/// ```auto
/// #[api(method = "DELETE", path = "/api/users/:id")]
/// pub fn deleteuser(id int) bool {
///     use db
///     return db.delete_user(id)
/// }
/// ```
#[command]
pub fn deleteuser(id: i32) -> bool {
    db::delete_user(id)
}

/// Search users by name or email
///
/// Auto source:
/// ```auto
/// #[api(method = "GET", path = "/api/users/search")]
/// pub fn searchusers(query str) []User {
///     use db
///     return db.search_users(query)
/// }
/// ```
#[command]
pub fn searchusers(query: String) -> Vec<User> {
    db::search_users(query)
}

// ============================================================================
// Command Registration Helper
// ============================================================================

/// Register all commands with a Tauri app builder
///
/// Usage in src-tauri/src/main.rs:
/// ```ignore
/// use api_example_backend::commands::register_commands;
///
/// fn main() {
///     tauri::Builder::default()
///         .invoke_handler(register_commands())
///         .run(tauri::generate_context!())
///         .expect("error while running tauri application");
/// }
/// ```
pub fn register_commands<R: tauri::Runtime>() -> impl Fn(tauri::ipc::Invoke<R>) -> bool + Send + Sync + 'static {
    tauri::generate_handler![
        getuser,
        listusers,
        createuser,
        deleteuser,
        searchusers,
    ]
}

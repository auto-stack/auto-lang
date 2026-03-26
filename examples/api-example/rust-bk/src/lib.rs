//! API Example Backend
//!
//! Auto-generated backend for api-example Tauri application.
//!
//! This crate provides:
//! - `types`: User and request types (from api.at)
//! - `db`: Database operations (from db.at)
//! - `commands`: Tauri IPC command wrappers (from #[api] functions)
//!
//! ## Usage
//!
//! ### As a library (for testing)
//! ```rust
//! use api_example_backend::{User, db};
//!
//! let users = db::all_users();
//! println!("Found {} users", users.len());
//! ```
//!
//! ### With Tauri (in src-tauri/src/main.rs)
//! ```ignore
//! use api_example_backend::commands::register_commands;
//!
//! fn main() {
//!     tauri::Builder::default()
//!         .invoke_handler(register_commands())
//!         .run(tauri::generate_context!())
//!         .expect("error while running tauri application");
//! }
//! ```

pub mod types;
pub mod db;

#[cfg(feature = "tauri")]
pub mod commands;

// Re-export commonly used types at crate root
pub use types::{User, CreateUserRequest};

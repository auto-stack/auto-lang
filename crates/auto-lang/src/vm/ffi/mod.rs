//! Plan 094: Hybrid FFI Bridge
//!
//! This module implements a hybrid FFI architecture that combines:
//! - **Static FFI**: `#[rust_fn]` macro for built-in stdlib functions
//! - **Dynamic FFI**: `use.rust` + sandbox for user crates (Plan 092)
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                    Hybrid FFI Architecture                   │
//! ├─────────────────────────────────────────────────────────────┤
//! │                                                             │
//! │   Static FFI              Dynamic FFI (Plan 092)           │
//! │   #[rust_fn]              use.rust + sandbox               │
//! │   IDs: 0-9999             IDs: 10000+                      │
//! │                                                             │
//! │   • File.read_text        • serde_json::from_str           │
//! │   • File.write_text       • tokio::net::TcpStream          │
//! │   • Env.get               • user_crate::*                  │
//! │                                                             │
//! │   Zero overhead            ABI verified                     │
//! │   Compile-time             Runtime loaded                   │
//! │                                                             │
//! └─────────────────────────────────────────────────────────────┘
//!                               │
//!                               ▼
//!               Unified NativeInterface.get(id)
//!                               │
//!                               ▼
//!                          CALL_NAT opcode
//! ```

mod convert;
mod error;
pub mod stdlib;
pub mod rust_stdlib;
pub mod http_server;  // Plan 321/322: AutoHttpServer unified shim
pub mod websocket;   // Plan 350: WebSocket client
// Plan 216 Phase 2: C FFI runtime
pub mod c_ffi;

pub use convert::VMConvertible;
pub use error::FFIError;
pub use stdlib::register_stdlib_ffi;
pub use c_ffi::CFfiRuntime;

/// Inventory-collected FFI registration entry (Plan 198).
///
/// Each `#[rust_fn("Name.method")]` annotated function generates one of these
/// via `inventory::submit!`. At VM init, `build_from_inventory()` iterates
/// all submissions and registers them by looking up the ID from BIGVM_NATIVES.
pub struct StaticFFIRegistration {
    pub name: &'static str,
    pub shim: fn(&mut crate::vm::task::AutoTask, &crate::vm::engine::AutoVM) -> Result<(), crate::vm::engine::VMError>,
}
inventory::collect!(StaticFFIRegistration);

/// Maximum ID for static FFI bindings
pub const STATIC_ID_MAX: u16 = 10000;

/// Starting ID for dynamic FFI bindings
pub const DYNAMIC_ID_START: u16 = 10000;

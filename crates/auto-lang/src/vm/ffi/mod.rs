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

pub use convert::VMConvertible;
pub use error::FFIError;
pub use stdlib::register_stdlib_ffi;

/// Maximum ID for static FFI bindings
pub const STATIC_ID_MAX: u16 = 10000;

/// Starting ID for dynamic FFI bindings
pub const DYNAMIC_ID_START: u16 = 10000;

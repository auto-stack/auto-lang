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
// Plan 216 Phase 2: C FFI runtime
pub mod c_ffi;

pub use convert::VMConvertible;
pub use error::FFIError;
pub use stdlib::register_stdlib_ffi;
pub use c_ffi::CFfiRuntime;

/// Maximum ID for static FFI bindings
pub const STATIC_ID_MAX: u16 = 10000;

/// Starting ID for dynamic FFI bindings
pub const DYNAMIC_ID_START: u16 = 10000;

// Plan 198: Registration entry for #[rust_fn] shims, collected via inventory crate.
inventory::collect!(StaticFFIRegistration);

/// Registration entry for a #[rust_fn] shim, collected by the inventory crate
/// at link time. Each entry maps a canonical name (e.g., "auto.file.read_text") to its
/// shim function pointer.
pub struct StaticFFIRegistration {
    pub name: &'static str,
    pub shim: fn(&mut crate::vm::task::AutoTask, &crate::vm::engine::AutoVM) -> Result<(), crate::vm::engine::VMError>,
}

/// Auto-register all #[rust_fn] shims collected by the inventory crate.
///
/// Iterates all `StaticFFIRegistration` entries submitted by `#[rust_fn]` macros,
/// resolves their canonical name to an ID via BIGVM_NATIVES, and registers
/// the shim in the NativeInterface dispatch table.
pub fn register_all_rust_fn(natives: &mut crate::vm::native::NativeInterface) {
    for reg in inventory::iter::<StaticFFIRegistration> {
        if let Some(id) = crate::vm::native_registry::BIGVM_NATIVES
            .lock()
            .ok()
            .and_then(|r| r.resolve_qualified(reg.name))
        {
            natives.register_with_name(id, reg.name, reg.shim);
        }
    }
}

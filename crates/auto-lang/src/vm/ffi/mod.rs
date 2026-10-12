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
// PLAN-705 T-02: 统一 live-op 登记表（register/complete/take/cancel 单次
// 终结协议）+ 全局完成通知 —— 服务端 HTTP handler 异步生命周期的结果面。
pub mod async_http;
// Plan 430: shim-metadata 生成的 std 追加段(dispatch 3000 优先于手写臂)
mod generated_std;
// Plan 430 C2: 三方 crate 方法 shim 包的运行期注册表与 dispatch
pub mod dep_methods;
pub mod http_server;
pub mod rust_stdlib; // Plan 321/322: AutoHttpServer unified shim

pub mod http_stream; // PLAN-707: 外部 HTTP/SSE 流统一资源表与生产者
                     // PLAN-727 T-05: 文件传输桥——共享传输核心（a2r_std::http::transfer）的宿主
                     // 注册表/park/legacy 迁移面。
pub mod http_server_file; // PLAN-729: 服务端文件响应描述符桥
pub mod http_transfer;
pub mod http_upload; // PLAN-730: 服务端上传句柄/会话/收据桥
                     // PLAN-699: Axum/Hyper HTTP/1.1 transport (network thread + owned bridge).
pub mod http_transport;
pub mod websocket; // Plan 350: WebSocket client
                   // auto-os Plan 013 T2: AutoTerm 引擎桥(auto.term.*,libloading → autoterm_core.dll)
pub mod term_engine;
// Plan 442 C2: axum → AutoVM serve adapter (Router/extractor marshalling)
pub mod axum_adapter;
// Plan 442 C2: musk backend extern response constructors (VM-side shims)
pub mod musk_response_ctor;
// Plan 216 Phase 2: C FFI runtime
pub mod c_ffi;

pub use convert::VMConvertible;
// Plan 377 §4.3: heap-aware 64-bit encode/decode（>2^48 BigInt 堆装箱兜底）
pub use c_ffi::CFfiRuntime;
pub use convert::{decode_i64_full, decode_u64_full, encode_i64_with_heap, encode_u64_with_heap};
pub use error::FFIError;
pub use stdlib::register_stdlib_ffi;

/// Inventory-collected FFI registration entry (Plan 198).
///
/// Each `#[rust_fn("Name.method")]` annotated function generates one of these
/// via `inventory::submit!`. At VM init, `build_from_inventory()` iterates
/// all submissions and registers them by looking up the ID from BIGVM_NATIVES.
pub struct StaticFFIRegistration {
    pub name: &'static str,
    pub shim: fn(
        &mut crate::vm::task::AutoTask,
        &crate::vm::engine::AutoVM,
    ) -> Result<(), crate::vm::engine::VMError>,
    pub parameters: &'static [&'static str],
    pub returns: &'static str,
    pub producer: &'static str,
}
inventory::collect!(StaticFFIRegistration);

/// Maximum ID for static FFI bindings
pub const STATIC_ID_MAX: u16 = 10000;

/// Starting ID for dynamic FFI bindings
pub const DYNAMIC_ID_START: u16 = 10000;

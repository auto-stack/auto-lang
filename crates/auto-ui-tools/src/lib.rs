//! # auto-ui-tools
//!
//! UI engineering tool package (PLAN-815). M815-A delivered the real
//! `auto-ui-tool` process with two narrow protocol operations — `describe`
//! and `validate_project` (§5.5 honesty contract: unimplemented operations
//! are rejected before any side effect).
//!
//! **M815-B / T-04** (this revision) lands the generation closure extracted
//! from auto-man: the Vue/Rust/Jet/Ark/Tauri/VSCode project generators,
//! gallery/wm/shadcn assets, the JS package runner and the display-field
//! reader — all consuming the M814-A public surface (auto-ui-compiler +
//! auto-lang-core), the 809 I-PROJECT request (via `PacSource`) and the
//! auto-api-gen single-algorithm API generation. Real build/run execution
//! over the protocol lands with T-05 (M814-B gate); the run paths that
//! depend on the not-yet-migrated iced runtime/back_proxy stay behind in
//! auto-man compat modules.
//!
//! Closure discipline: this crate depends only on the protocol crate, the
//! one-source project model, the M814-A public UI/compiler/SDK packages,
//! auto-man-core (declaration single source) and auto-api-gen — no auto-man,
//! no legacy auto-lang compat big lib. Clients (PLAN-816 CLI) talk to the
//! binary over framed stdio; they must not link this crate as a library
//! dependency.

pub mod resources;
pub mod validate;

// ---- PLAN-815 T-05: 托管执行闭包（协议循环/工作操作/进程托管/运行路径/
// bin stdio 分离） ----
pub mod host;
pub mod proc_host;
pub mod run_vm;
pub mod run_vue;
pub mod stdio_split;
pub mod tool_ops;

// ---- PLAN-815 T-04: UI engineering tool closure (from auto-man) ----
pub mod ark;
pub mod gallery_assets;
pub mod gallery_cache;
pub mod image_viewer;
pub mod jet;
pub mod pkg;
#[cfg(test)]
mod plan075_bp_tests;
pub mod project_display_reader;
#[cfg(test)]
mod project_display_reader_tests;
pub mod rust_ui;
pub mod shadcn_build;
pub mod tauri;
pub mod tauri_backend;
pub mod util;
pub mod vscode;
pub mod vue;
pub mod vue_shadcn;
pub mod wm_assets;

use std::io::{Read, Write};

use auto_tool_protocol::ToolDescriptor;

/// Tool identity on the wire.
pub const TOOL_NAME: &str = "auto-ui-tool";

/// The capability list (A-stage narrow ops + T-05 execution ops). See
/// [`host::tool_descriptor`].
pub fn tool_descriptor() -> ToolDescriptor {
    host::tool_descriptor()
}

/// Run the tool host over framed stdio until EOF. M815-A kept the
/// synchronous loop; T-05 moves to the threaded host (protocol loop thread
/// + OS-main-thread UI executor) with the caller-visible contract
/// unchanged — see [`host::run_tool`].
pub fn run_tool<R: Read + Send + 'static, W: Write + Send + 'static, L: Write + Send + 'static>(
    input: R,
    control: W,
    log: L,
    max_frame_bytes: usize,
) -> i32 {
    host::run_tool(input, control, log, max_frame_bytes)
}

//! # auto-terminal-core
//!
//! Terminal host-domain core (PLAN-808 / R03 terminal subdomain). Owns:
//! - the plain value contract (cells/colors/geometry/snapshot);
//! - the runtime `autoterm_core.dll` engine service (library resolution,
//!   symbol contract, session handle table, row/cursor/viewport sampling,
//!   backlog state, palette queries);
//! - [`TerminalHostProvider`], the 34-method light capability port consumed
//!   by the Auto VM and by UI hosts.
//!
//! Dependency closure: std + `libloading` only. This crate must NOT depend
//! on auto-lang, UI, auto-man or auto-term; the terminal engine is a runtime
//! resource loaded through the documented DLL search order. UI hosts adapt
//! this core to their view/input registry; non-UI consumers can use the
//! service directly.
//!
//! Frozen semantics (PLAN-808 T-01 `attachments/808/contract-baseline.md`):
//! every absent/ invalid-handle sentinel and the 34 native shapes are the
//! pre-extraction values; do not adjust them silently.

pub mod engine;
pub mod provider;
pub mod retired;
pub mod types;

pub use engine::{shared, EngineService};
pub use provider::{default_provider, TerminalHostProvider};
pub use retired::retired_cache;
pub use types::{
    decode_style_color, TerminalCell, TerminalColor, TerminalGeometry, TerminalRetiredCache,
    TerminalSnapshot, MAX_RESIZE_COLS, MAX_RESIZE_ROWS, MIN_RESIZE_COLS, PALETTE_SLOTS,
};

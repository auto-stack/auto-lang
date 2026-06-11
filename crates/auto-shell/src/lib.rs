//! AutoShell - A modern shell environment using AutoLang
//!
//! This library provides the core functionality for the AutoShell REPL,
//! command execution, and pipeline system.
//!
//! ## Architecture
//!
//! - `ash-core` crate — Pure logic, zero terminal dependencies
//! - `frontend/` — Terminal-dependent code (will become `ash-tui` crate)
//! - `cmd/`, `completions/`, `data/`, `shell/` — Mixed layer, migrating

// Core layer: re-export ash-core crate as `core` module for backward compatibility
pub use ash_core as core;

// Frontend layer (terminal-dependent, will become ash-tui crate)
pub mod frontend;

// Legacy modules (will migrate into ash-core or frontend over time)
pub mod cmd;
pub mod completions;
pub mod data;
pub mod menu;
pub mod prompt;
pub mod shell;
pub mod signal;

// Re-export core modules at crate root for backward compatibility
pub use ash_core::bookmarks;
pub use ash_core::parser;
pub use ash_core::pipeline;

// Re-export frontend modules at crate root for backward compatibility
pub use frontend::repl;
pub use frontend::term;

pub use repl::Repl;
pub use shell::Shell;

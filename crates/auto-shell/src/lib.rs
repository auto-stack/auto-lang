//! AutoShell - A modern shell environment using AutoLang
//!
//! This library provides the core functionality for the AutoShell REPL,
//! command execution, and pipeline system.
//!
//! ## Internal Layering (migrating toward ash-core + ash-tui split)
//!
//! - `core/` — Pure logic, zero terminal dependencies. Will become `ash-core` crate.
//! - Everything else — Terminal-dependent code. Will become `ash-tui` crate.

// Core layer (pure logic, zero terminal deps)
pub mod core;

// Legacy modules (will migrate into core/ or frontend/ over time)
pub mod cmd;
pub mod completions;
pub mod data;
pub mod repl;
pub mod shell;
pub mod term;

// Re-export core modules at crate root for backward compatibility
pub use core::bookmarks;
pub use core::parser;

pub use repl::Repl;
pub use shell::Shell;

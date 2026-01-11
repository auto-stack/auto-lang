//! AutoShell - A modern shell environment using AutoLang
//!
//! This library provides the core functionality for the AutoShell REPL,
//! command execution, and pipeline system.

pub mod cmd;
pub mod completions;
pub mod data;
pub mod parser;
pub mod repl;
pub mod shell;
pub mod term;

pub use shell::Shell;
pub use repl::Repl;

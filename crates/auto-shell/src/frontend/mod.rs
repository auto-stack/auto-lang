//! Frontend module — terminal-dependent rendering layer
//!
//! This module contains code that depends on terminal libraries (reedline,
//! ratatui, nu-ansi-term, crossterm). It will eventually become the `ash-tui`
//! crate.
//!
//! ## Architecture
//!
//! - `renderer/` — ratatui Buffer → ANSI string conversion bridge
//! - `repl` — Read-Eval-Print Loop (reedline-driven)
//! - `term/` — Terminal utilities (highlight, prompt)
//! - `completions_reedline` — reedline Completer adapter

pub mod renderer;
pub mod repl;
pub mod term;
pub mod completions_reedline;

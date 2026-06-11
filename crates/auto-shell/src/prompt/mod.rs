//! AshPrompt — modular prompt engine for AutoShell
//!
//! Inspired by Starship's architecture, but with minimal dependencies:
//! - `rayon` for parallel module rendering
//! - `toml` for configuration
//! - `nu-ansi-term` for ANSI styling (already a dependency)
//!
//! # Quick start
//!
//! ```ignore
//! use auto_shell::prompt::{AshPrompt, config::AshConfig};
//!
//! let prompt = AshPrompt::new(AshConfig::load());
//! // Use with reedline: Reedline::create().with_prompt(prompt)
//! ```

pub mod config;
pub mod context;
pub mod engine;
pub mod module;
pub mod modules;

pub use config::AshConfig;
pub use context::AshContext;
pub use engine::AshPrompt;
pub use module::{PromptModule, PromptSegment, SegmentStyle};

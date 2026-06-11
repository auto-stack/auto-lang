//! Auto-completion module
//!
//! Re-exports completion logic from ash-core and provides reedline integration.

// Re-export everything from ash-core completions
pub use ash_core::completions::{
    auto, command, file, Completion, CompletionKind, get_completions,
};

// Frontend-only: reedline integration
pub mod reedline;

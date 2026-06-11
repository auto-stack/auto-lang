//! Auto-completion module
//!
//! Re-exports completion logic from ash-core and provides reedline integration.

// Re-export everything from ash-core completions
pub use ash_core::completions::{
    auto, command, file, flag, types,
    Completion, CompletionKind,
    get_completions, get_completions_with_context,
    CompletionSignature, CompletionArgument,
};

// Frontend-only: reedline integration
pub mod reedline;

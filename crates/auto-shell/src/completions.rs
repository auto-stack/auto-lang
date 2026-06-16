//! Auto-completion module
//!
//! Re-exports completion logic from ash-core and provides reedline integration.

// Re-export everything from ash-core completions
pub use ash_core::completions::{
    auto, command, file, flag, provider, spec, types,
    Completion, CompletionKind,
    get_completions, get_completions_with_context,
    CompletionContext, CompletionProvider,
    CompletionSignature, CompletionArgument,
    CompletionSpec, SubcommandSpec, FlagSpec, ArgSpec,
    WhenCondition, CompletionSource, ParseMode,
};

// Frontend-only: reedline integration
pub mod reedline;

// Plan 315: three-tier spec loading + runtime probe helpers.
pub mod spec_tiers;

// External command completion definitions
pub mod definitions;

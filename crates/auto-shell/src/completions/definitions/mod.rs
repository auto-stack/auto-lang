//! External command completion definitions
//!
//! Each file defines a `CompletionSpec` for a well-known external command
//! using the builder pattern from `ash_core::completions::spec`.

pub mod cargo;
pub mod git;

use ash_core::completions::CompletionProvider;

/// Register all built-in external command completion specs.
pub fn register_all(provider: &mut CompletionProvider) {
    provider.register(git::spec());
    provider.register(cargo::spec());
}

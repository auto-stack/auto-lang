//! Completion-aware command signature types
//!
//! Mirrors `auto-shell::cmd::{Signature, Argument}` but lives in `ash-core`
//! so the pure completion logic can access command metadata without depending
//! on the full auto-shell crate. Converted via `From<Signature>`.

/// Completion-aware command signature.
#[derive(Clone, Debug)]
pub struct CompletionSignature {
    pub name: String,
    pub description: String,
    pub arguments: Vec<CompletionArgument>,
}

/// Completion-aware argument descriptor.
#[derive(Clone, Debug)]
pub struct CompletionArgument {
    pub name: String,
    pub description: String,
    pub required: bool,
    pub is_flag: bool,
    pub short: Option<char>,
}

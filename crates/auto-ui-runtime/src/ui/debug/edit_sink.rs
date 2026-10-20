//! Edit sink trait -- abstraction for applying debug edits.
//!
//! Phase 1 stub. The trait is defined so the DebugLayer can hold a reference
//! to an edit sink from day one, but no concrete implementations exist yet.
//!
//! Future phases will provide:
//! - `TranspiledEditSink` -- writes edits back to `.at` source files
//! - `VmEditSink`         -- patches VM state directly (instant hot reload)

/// Error type for edit-sink operations (stub).
#[derive(Debug)]
pub struct DebugError {
    pub message: String,
}

impl std::fmt::Display for DebugError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "DebugError: {}", self.message)
    }
}

impl std::error::Error for DebugError {}

/// Trait for backends that can apply debug edits.
///
/// In the current phase this is a placeholder.  Phase 4 will flesh out
/// `TranspiledEditSink` with actual `.at` file round-tripping.
pub trait DebugEditSink {
    /// Apply a batch of debug edits.
    ///
    /// Edits should be applied bottom-up (descending byte offset) so that
    /// earlier byte offsets remain valid.
    fn apply(&self, edits: &[DebugEdit]) -> Result<(), DebugError>;
}

/// A single debug edit (stub for Phase 1).
///
/// Variants will be expanded in Phase 4 (editing) and Phase 5 (widget tree
/// manipulation).
#[derive(Debug, Clone)]
pub enum DebugEdit {
    /// Stub variant -- will be replaced with real variants in Phase 4.
    _Phase4Stub,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debug_error_display() {
        let err = DebugError {
            message: "something went wrong".to_string(),
        };
        assert_eq!(format!("{}", err), "DebugError: something went wrong");
    }
}

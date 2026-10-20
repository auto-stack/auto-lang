// PLAN-807 — directory comparison lives in `auto-text-core::diff::dirs`;
// this historical path re-exports it with unchanged type identity.
pub use auto_text_core::diff::dirs::{
    DirCounts, DirDiffError, DirDiffIter, DirDiffOptions, DirEntryDiff, DirStatus,
};

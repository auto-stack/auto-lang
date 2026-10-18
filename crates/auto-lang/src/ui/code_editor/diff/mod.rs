// PLAN-703 engine / PLAN-807 — the diff engine moved to the
// `auto-text-core` crate (single source: interning, patience anchors,
// histogram core, grouping, envelope semantics). This module is a thin
// re-export preserving the historical in-crate paths
// (`crate::ui::code_editor::diff::*`, `…::diff::dirs::*`,
// `…::diff::envelope::*`) with UNCHANGED type and function identity.
// `envelope` keeps the one UI-owned piece: the editor-registry key→snapshot
// lookup feeding the pure core face.
pub use auto_text_core::diff::{
    diff_lines, diff_lines_parallel, diff_lines_windowed, diff_snapshots, refine_inline, DiffOpts,
    DiffOut, Hunk, Refinement,
};

pub mod dirs;
pub mod envelope;

pub use dirs::{DirCounts, DirDiffError, DirDiffIter, DirDiffOptions, DirEntryDiff, DirStatus};

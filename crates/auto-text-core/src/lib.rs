//! Text foundation — persistent rope, file-backed paging, snapshots, diff.
//!
//! PLAN-807: extracted verbatim from `auto-lang`'s `ui/code_editor/core/rope`
//! and `ui/code_editor/diff` so text data and algorithms live in ONE place
//! consumable without the language/VM/UI stack. `auto-lang` keeps thin
//! re-export facades at the old paths (`ui::code_editor::core::rope`,
//! `ui::code_editor::diff`); the editor registry and key→snapshot lookup
//! stay UI-side (the pure envelope takes snapshots).
//!
//! Scope boundaries (deliberate, honest limits):
//! - The rope is the editor-kernel document store (line convention, digest
//!   and paging invariants documented in `rope`); it is not a general
//!   CRDT/OT structure.
//! - `write_backed` is the streaming merge save with external-modification
//!   refusal; no stronger save transactionality is offered.
//! - Diff is the histogram engine with patience anchors and the downstream
//!   envelope JSON contract; window calls are projections of the full
//!   result, never re-diffs of a slice.
//!
//! Dependency discipline: `imara-diff` + std only. No
//! auto-lang/auto-man/auto-ui/auto-vm types cross this boundary (pinned by
//! `test/fixtures/modularity/text-core-consumer`).

pub mod rope;

pub mod diff;

pub use rope::{PageConfig, PageState, PageStore, Rope, RopeSnapshot};

//! Pinned snapshot of the AutoDown headless block parser.
//!
//! `markdown_parser.rs` / `ial.rs` / `block_model.rs` are copied verbatim from
//! the auto-down repository (a2r-generated; see `PROVENANCE.md` for the source
//! revision and SHA-256 fingerprints). They must not be edited by hand —
//! changes go through explicit re-vendoring with updated fingerprints.
//!
//! Raw source positions are NOT populated by this parser (`BlockNode::source`
//! is always `rng(0, 0)`); consumers needing byte-accurate payloads pair the
//! tree with an external raw-fence locator (`auto_test_case::raw_ranges`).
//!
//! Known non-invoked hazard kept out of the consumption surface: the
//! `block_model` editing helpers `spans_insert`/`spans_delete`/`spans_split_at`
//! mix byte and char indexing — do not call them (the parser itself does not).

// a2r-generated snapshot: warnings are silenced wholesale on purpose; the code
// is frozen, and lint churn here would only invite hand edits.
#![allow(warnings)]

pub mod block_model;
pub mod ial;
pub mod markdown_parser;

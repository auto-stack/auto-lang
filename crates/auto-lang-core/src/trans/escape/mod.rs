//! Ownership escape analysis — Plan 310 Phase 1.
//!
//! Decides, for each local binding in a function body, whether it can be
//! represented as a Rust borrow or must fall back to a clone / smart pointer.
//! See [`analyzer::EscapeAnalyzer`] for the algorithm and
//! [`escape_map::EscapeMap`] for the decision table.
//!
//! ## Phase 1 status
//!
//! The analyzer runs and populates an [`EscapeMap`], but the transpiler does
//! **not** yet consult it — transpiled output bytes are unchanged. Phase 2
//! wires the map into `Expr::View`/`Expr::Mut` generation.

pub mod analyzer;
pub mod escape_map;
pub mod report;

pub use analyzer::EscapeAnalyzer;
pub use escape_map::{BindingId, EscapeMap, OwnershipTier};

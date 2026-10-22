//! PLAN-817 CTE core: the consumer mapping (§5 of the contract decision)
//! over the landed 796/ac-core public APIs. This crate owns NO execution
//! engine and NO store — the probe crate wires those; everything here is
//! schedulable, verifiable logic over `SemaInput` and `hir::Bundle`.

pub mod consumer;
pub mod dependencies;
pub mod schedule;

pub use consumer::{backfill_and_verify, resolve_bindings, substitute_and_infer, ResolvedBinding};
pub use dependencies::{build_manifest, manifest_fp, manifest_fresh};
pub use schedule::{resolve_queries, QueryResolution};

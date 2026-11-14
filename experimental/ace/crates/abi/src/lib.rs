//! PLAN-797 first bounded AutoABI candidate profile + bidirectional native
//! interop probe — host-side harness.
//!
//! Scope (plan §0/§1): freeze a bounded Windows x64 candidate profile and
//! prove three REAL native interop legs with independently compiled
//! objects — AutoABI caller -> AutoABI callee, AutoABI caller -> C callee
//! (real MSVC cl.exe object), C caller -> AutoABI callee — linked into
//! real PEs with the merged ac-core link entries and executed under the
//! contained deadline executor.
//!
//! Non-goals, stated up front so no green test can be over-claimed:
//! * This is NOT an Auto-source compiler: modules are emitted from a
//!   frozen, bounded declaration IR (see [`decl`]), never through an Auto
//!   parser. Reports must say so (plan §5.2).
//! * `v0.1-candidate` is a candidate label only; the profile is frozen
//!   after independent review + merge gates, not by this crate (§0).
//! * The internal ABI-A–E surfaces of PLAN-774 stay untouched; nothing
//!   here extends or claims compatibility with them (§1).

#![cfg(windows)]

pub mod bridge;
pub mod build;
pub mod cache;
pub(crate) mod cache_paths;
pub mod ctool;
mod ctool_capture;
mod ctool_dependencies;
mod ctool_environment;
mod ctool_output;
mod ctool_text;
pub mod decl;
pub mod descriptor;
pub mod emit;
pub mod layout;
pub mod link;
/// PLAN-967 T-14: the provider build cache (precise BuildIdentity
/// over the existing 783 ManagedStore).
#[cfg(windows)]
pub mod provider_build_cache;
pub mod resource_provider;
pub mod run;

/// PLAN-967 T-03: the F0 certification recipe on the worker transport.
#[cfg(windows)]
pub mod rust_f0_worker;
/// PLAN-967 F0: the Rust provider build unit (frozen source + generated
/// shim -> one cdylib closure) and its in-process loader.
#[cfg(windows)]
pub mod rust_provider;
pub mod semref;
mod source_crt;
pub mod source_foreign;
pub mod source_provider;
mod source_request_work;
pub mod source_store;
mod source_tool_scratch;
pub mod trace;
pub mod typed_decl;
pub mod typed_emit;
pub mod validate;
pub mod worker;
pub mod worker_bridge;
pub mod worker_source;

/// PLAN-937 CA-D1: the neutral foreign-interface contract shared with
/// the A2C C side (both real consumers bind to this one model).
pub use ace_foreign_contract;
/// Re-export the semantic contract crate so integration tests can mint
/// refs through the SAME real library the implementation binds to.
pub use ace_semantics_contract;

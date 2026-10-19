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

pub mod build;
pub mod cache;
pub mod ctool;
pub mod decl;
pub mod descriptor;
pub mod emit;
pub mod layout;
pub mod link;
pub mod run;
pub mod semref;
pub mod trace;
pub mod validate;
pub mod worker;

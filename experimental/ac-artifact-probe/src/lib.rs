//! ac-artifact-probe — PLAN-783 AC/ACC unified compiler-artifact contract
//! and cache feasibility probe.
//!
//! Standalone experimental workspace. It consumes the ac-core public API
//! (`auto-ac-prototype`) for the real vertical line and reuses
//! `auto-cache`'s `BlobStore` through an isolated adapter for the block
//! layer. It is NOT a production cache entry point, and ac-core must never
//! gain a dependency on product crates through this probe.
//!
//! Layers (see `schema`): A00–A12 artifact contract registry. The probe
//! implements a restricted executable subset end to end (source snapshot →
//! declaration summary → core/scalar HIR → COFF object → staged link); all
//! other layers are registered seams only.

pub mod codec;
pub mod identity;
pub mod layout;
pub mod schema;
pub mod store;
pub mod subproc;
pub mod sysinfo;

/// Probe identity carried into every artifact manifest and receipt.
pub const PROBE_ID: &str = concat!("ac-artifact-probe/", env!("CARGO_PKG_VERSION"));

/// Experiment-wide hash algorithm identity (part of every content key).
pub const HASH_ALGO: &str = "blake3-1.5";

/// Default subprocess deadline (plan §3): 60 seconds. Tool preparation that
/// legitimately needs longer must record it in the receipt.
pub const SUBPROC_DEADLINE_SECS: u64 = 60;

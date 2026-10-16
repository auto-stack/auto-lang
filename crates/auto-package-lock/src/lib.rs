//! pac.lock core — lock model, codec and verification rules.
//!
//! PLAN-795: extracted Target-free from `auto-man`'s `lock.rs` so consumers
//! without the language/VM/renderer stack can read, write and verify
//! `pac.lock` files. The rules below are single-sourced here; `auto-man`
//! keeps a thin compatibility facade (Target→entry projection, Git commit
//! observation) that delegates to this crate.
//!
//! Scope boundaries (deliberate, honest limits):
//! - This lock is the AutoMan `pac.lock` contract — it is NOT the Cargo
//!   distribution lock of PLAN-792.
//! - Verification checks lock presence, version match and (PLAN-635 D6)
//!   materialized-path existence for local path deps. It is not a full Git
//!   content check and not a supply-chain integrity guarantee.
//! - Version stamping: the caller supplies the tool version
//!   (`automan_version`); this crate never substitutes its own package
//!   version.
//!
//! Dependency discipline: only `auto-val` (leaf data crate), serde/toml/
//! serde_json and std. No `auto-lang`/`auto-man`/UI/build types cross this
//! boundary (pinned by `test/fixtures/modularity/package-lock-consumer`).

pub mod codec;
pub mod model;
pub mod verify;

pub use codec::{is_json_path, LOCK_JSON_SUFFIX};
pub use model::{
    find_entry, sort_dependencies, LockEntry, LockFile, LockedEntryView, ResolvedDependency,
};
pub use verify::verify_dependencies;

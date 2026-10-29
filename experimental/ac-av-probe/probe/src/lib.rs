//! auto-av-probe — PLAN-798 probe layer.
//!
//! Consumes the landed ac-core public API (Checked HIR front door, native
//! reference) and the PLAN-783 ManagedStore; hosts the per-function unit
//! cache, the request driver with spans/observations and the bounded worker
//! binary. The HIR→MIR lowering itself lives in the shared `auto-mir-lower`
//! crate since PLAN-868 — this probe is a consumer, not the provider.
//! Nothing here feeds product paths.

pub mod bench;
pub mod cache;
pub mod closures_fixture;
pub mod closures_865_support;
pub mod closures_session;
pub mod coff_admission;
pub mod collections_864_session;
pub mod collections_864_support;
pub mod collections_fixture;
pub mod collections_session;
pub mod data_driver;
pub mod data_session;
pub mod driver;
pub mod dynamic_fixture;
pub mod dynamic_session;
pub mod fn_hir_codec;
pub mod formal_828;
pub mod formal_campaign;
pub mod formal_worker;
pub mod function_queries;
pub mod load256;
pub mod mempeek;
pub mod native_exec;
pub mod observations;
pub mod owned_driver;
pub mod owned_fixture;
pub mod owned_session;
pub mod resident;
pub mod semrefs;
pub mod session;
pub mod text_runtime;
pub mod typed_fixture;
pub mod wire;
pub mod worker_restore;

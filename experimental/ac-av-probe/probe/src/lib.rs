//! auto-av-probe — PLAN-798 probe layer.
//!
//! Consumes the landed ac-core public API (Checked HIR front door, native
//! reference) and the PLAN-783 ManagedStore; owns the HIR→MIR adapter, the
//! per-function unit cache, the request driver with spans/observations and
//! the bounded worker binary. Nothing here feeds product paths.

pub mod bench;
pub mod cache;
pub mod data_driver;
pub mod data_session;
pub mod driver;
pub mod fn_hir_codec;
pub mod formal_828;
pub mod formal_campaign;
pub mod formal_worker;
pub mod function_queries;
pub mod hir_adapter;
pub mod load256;
pub mod mempeek;
pub mod native_admission;
pub mod native_reference;
pub mod observations;
pub mod owned_driver;
pub mod owned_fixture;
pub mod semrefs;
pub mod session;
pub mod typed_fixture;
pub mod worker_restore;

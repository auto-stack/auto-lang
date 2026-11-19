//! auto-av-probe — PLAN-798 probe layer.
//!
//! Consumes the landed ac-core public API (Checked HIR front door, native
//! reference) and the PLAN-783 ManagedStore; hosts the per-function unit
//! cache, the request driver with spans/observations and the bounded worker
//! binary. The HIR→MIR lowering itself lives in the shared `auto-mir-lower`
//! crate since PLAN-868 — this probe is a consumer, not the provider.
//! Nothing here feeds product paths.

pub mod age_component;
// PLAN-981（IF-981-GFX-HOST-PORT）：AGE 图形宿主会话（注册/生命周期/每 Tick 预算）。
pub mod age_gfx;
pub mod age_h0;
pub mod age_package;
pub mod bench;
pub mod cache;
pub mod closures_865_support;
pub mod closures_fixture;
pub mod closures_fn_session;
pub mod closures_session;
pub mod closures_source_recovery;
pub mod coff_admission;
pub mod collections_864_session;
pub mod collections_864_support;
pub mod elf_admission; // PLAN-1040
                       // PLAN-919: the observed (lifecycle-channel) source→AV seam.
pub mod collections_fixture;
pub mod collections_session;
pub mod data_driver;
pub mod resource_events;
pub mod sched_worker;
pub mod scheduler_run; // PLAN-980 STATE：UI 异步宿主的 scheduler 一次性执行入口
                       // PLAN-934 T-08: fact-driven scalar source entry (product seam).
pub mod data_session;
pub mod driver;
pub mod dynamic_fixture;
pub mod dynamic_session;
pub mod fn_hir_codec;
pub mod formal_828;
pub mod formal_campaign;
pub mod formal_worker;
pub mod function_queries;
// PLAN-1078: the ordinary AV language-source recipe producer.
pub mod language_source_worker;
pub mod load256;
pub mod mempeek;
pub mod native_exec;
pub mod observations;
pub mod owned_driver;
pub mod owned_fixture;
pub mod owned_session;
pub mod query_entry;
pub mod resident;
pub mod resource_915;
pub mod semrefs;
pub mod session;
#[cfg(feature = "test-case-batch")]
pub mod test_observations;
pub mod text_runtime;
pub mod typed_fixture;
pub mod wire;
pub mod worker_restore;

/// Typed console host probe (PLAN-863).
pub mod console_host;

/// Console recipe observation producer (PLAN-863).
pub mod console_worker;

#[cfg(test)]
mod formal_turnaround_tests;

/// PLAN-950 finite module source consumer over existing function units.
pub mod modules;

#[path = "cases/aggregates_codegen_tier.rs"]
mod aggregates_codegen_tier;
#[path = "cases/cache_tier.rs"]
mod cache_tier;
#[path = "cases/choose_instances_935.rs"]
mod choose_instances_935;
#[path = "cases/generic_records_963.rs"]
mod generic_records_963;
#[path = "cases/const_instances_963.rs"]
mod const_instances_963;
#[path = "cases/cli.rs"]
mod cli;
#[path = "cases/closures_compat.rs"]
mod closures_compat;
#[path = "cases/closures_native_924_performance.rs"]
mod closures_native_924_performance;
#[path = "cases/closures_native_tier.rs"]
mod closures_native_tier;
#[path = "cases/closures_restore_tier.rs"]
mod closures_restore_tier;
#[path = "cases/closures_typed_tier.rs"]
mod closures_typed_tier;
#[path = "cases/codegen_tier.rs"]
mod codegen_tier;
#[path = "cases/collections_driver_tier.rs"]
mod collections_driver_tier;
#[path = "cases/collections_frontend_tier.rs"]
mod collections_frontend_tier;
#[path = "cases/collections_incremental_tier.rs"]
mod collections_incremental_tier;
#[path = "cases/collections_layout_tier.rs"]
mod collections_layout_tier;
#[path = "cases/collections_observation_tier.rs"]
mod collections_observation_tier;
#[path = "cases/collections_place_tier.rs"]
mod collections_place_tier;
#[path = "cases/collections_resources_tier.rs"]
mod collections_resources_tier;
#[path = "cases/collections_seam_tier.rs"]
mod collections_seam_tier;
#[path = "cases/collections_source_tier.rs"]
mod collections_source_tier;
#[path = "cases/collections_typed_tier.rs"]
mod collections_typed_tier;
#[path = "cases/collections_workload_tier.rs"]
mod collections_workload_tier;
#[path = "cases/console_lane.rs"]
mod console_lane;
#[path = "cases/data_owned_tier.rs"]
mod data_owned_tier;
#[path = "cases/first_case.rs"]
mod first_case;
#[path = "cases/gap_closure.rs"]
mod gap_closure;
#[path = "cases/host_console_compat.rs"]
mod host_console_compat;
#[path = "cases/incremental_churn_gc_1000.rs"]
mod incremental_churn_gc_1000;
#[path = "cases/incremental_tier.rs"]
mod incremental_tier;
#[path = "cases/numeric_codegen_tier.rs"]
mod numeric_codegen_tier;
#[path = "cases/parity_tier.rs"]
mod parity_tier;
#[path = "cases/record_method_tier.rs"]
mod record_method_tier;
#[path = "cases/resources_tier.rs"]
mod resources_tier;
#[path = "cases/shadow_tier.rs"]
mod shadow_tier;
#[path = "cases/source_diagnostics_911.rs"]
mod source_diagnostics_911;
#[path = "cases/transaction_tier.rs"]
mod transaction_tier;
#[path = "cases/turnaround_tier.rs"]
mod turnaround_tier;

#[path = "cases/owned_text_incremental.rs"]
mod owned_text_incremental;

#[path = "cases/owned_text_performance.rs"]
mod owned_text_performance;

// PLAN-919: per-owner lifecycle events (R01/R04/R06 ac cells).
#[path = "cases/collections_resource_events_919.rs"]
mod collections_resource_events_919;

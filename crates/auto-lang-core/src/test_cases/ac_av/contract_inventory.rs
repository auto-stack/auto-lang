//! Read-only access to the original parent's frozen tables and limits.
//! No worker location, input compilation, cache query or guest execution.
pub(crate) fn snapshot() -> serde_json::Value {
    serde_json::json!({
        "source": "crates/auto-lang-core/src/test_cases/ac_av.rs",
        "protocol": "avworker/1",
        "limits": {
            "response_bytes": super::READ_CAP,
            "request_bytes": super::REQUEST_CAP,
            "stderr_bytes": super::STDERR_CAP,
            "reader_grace_ms": super::READER_GRACE.as_millis() as u64,
            "default_session_wall_ms": super::SESSION_WALL.as_millis() as u64,
            "default_build_wall_ms": super::BUILD_WALL.as_millis() as u64,
            "override_environment": {
                "AV798_SESSION_WALL_MS": std::env::var("AV798_SESSION_WALL_MS").ok(),
                "AV798_BUILD_WALL_MS": std::env::var("AV798_BUILD_WALL_MS").ok(),
            },
        },
        "tables": {
            "data_typed": {"columns":["fixture", "kind", "bits"], "rows":super::DATA_TYPED_FIXTURES},
            "capture": {"columns":["id", "first_clone_attempt", "last_clone_attempt", "control_attempt", "expected_live_after_drop"], "rows":super::CAPTURE_CASES},
            "resource_sweep": {"columns":["id", "value", "arena_attempts", "value_attempts"], "rows":super::RESOURCE_SWEEP_CASES},
            "place_sequences": {"columns":["mutation", "trap", "fresh_value", "legal_value"], "rows":super::PLACE_SEQUENCES},
            "owned": {"columns":["id", "result_json", "creation_sizes", "release_ids"], "rows":super::OWNED_CASES},
            "collections": {"columns":["id", "result_json", "creation_sizes", "release_ids"], "rows":super::COLLECTIONS_CASES},
            "closures": {"columns":["id", "result_json", "creation_sizes", "release_ids"], "rows":super::CLOSURES_CASES},
            "dynamic": {"columns":["id", "result_json", "creation_sizes", "release_ids"], "rows":super::DYNAMIC_CASES},
            "closures_source": {"columns":["id", "value", "diagnostic", "fresh_units"], "rows":super::CLOSURES_SOURCE_CASES},
            "closures_fn": {"columns":["id", "value", "diagnostic", "fresh_units"], "rows":super::CLOSURES_FN_CASES},
            "closures_recovery_edit": {"columns":["id", "edited_value"], "rows":super::CLOSURES_RECOVERY_EDIT_VALUES},
        },
    })
}

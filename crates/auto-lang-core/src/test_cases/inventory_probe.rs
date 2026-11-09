//! Typed probe declarations for read-only obligation discovery.
use auto_test_case::contract::{CheckSpec, ProbeStep};
use auto_test_case::ir::ResolvedOracle;

pub(super) fn append(
    row: &mut serde_json::Value,
    check: &CheckSpec,
    oracle: &ResolvedOracle,
    fixture_base: &std::path::Path,
) {
    let options = &check.options;
    row["probe_bindings"] = serde_json::json!({
        "av":options.probe_av.as_ref().map(|v| serde_json::json!({
            "entry":v.entry, "caps":v.caps, "session":v.session, "profile":v.profile,
            "resource":v.resource.as_ref().map(|r| serde_json::json!({
                "profile":r.profile, "max_steps":r.max_steps, "wall_ms":r.wall_ms,
                "cancel_after_ms":r.cancel_after_ms,
            })),
            "invalidation":v.invalidation.as_ref().map(|d| serde_json::json!({
                "stable_defs":d.stable_defs,"rebuilt_defs":d.rebuilt_defs,
            })),
        })),
        "native":options.probe_mn.as_ref().map(|v| serde_json::json!({
            "entry":v.entry, "caps":v.caps, "session":v.session, "diagnostics":v.diagnostics,
            "resource":v.resource.as_ref().map(|r| serde_json::json!({
                "budget_bytes":r.budget_bytes,"cancel_after_work":r.cancel_after_work,
                "fail_nth":r.fail_nth,"fail_seam":r.fail_seam,"fail_text_nth":r.fail_text_nth,
                "fail_ledger_nth":r.fail_ledger_nth,
            })),
            "declaration":v.declaration.as_ref().map(|d| serde_json::json!({
                "min_frontend_invalidations":d.min_frontend_invalidations,
                "max_codegen_units":d.max_codegen_units,"stable_defs":d.stable_defs,
            })),
            "typed":v.typed.as_ref().map(|t| serde_json::json!({"fixture":t.fixture,"args":t.args})),
        })),
        "a2x_hir":options.probe_a2x_hir.as_ref().map(|v| serde_json::json!({
            "profile":v.profile,"entry":v.entry,"caps":v.caps,
        })),
        "cte":options.probe_cte.as_ref().map(|v| serde_json::json!({
            "entry":v.entry,"caps":v.caps,"session":v.session,
        })),
        "abi":options.probe_abi.as_ref().map(|v| serde_json::json!({"leg":v.leg})),
        "sema":options.probe_sema.as_ref().map(|v| serde_json::json!({
            "package":v.package,"module":v.module,"kind":v.kind,"fn_name":v.fn_name,
        })),
    });
    row["declared_probe_steps"] = options
        .probe_steps
        .as_ref()
        .map(|steps| {
            steps
                .iter()
                .map(|step| match step {
                    ProbeStep::Query => serde_json::json!({"op":"query"}),
                    ProbeStep::Run => serde_json::json!({"op":"run"}),
                    ProbeStep::Restart => serde_json::json!({"op":"restart"}),
                    ProbeStep::EditSource { from, to } => {
                        serde_json::json!({"op":"edit-source","from":from,"to":to})
                    }
                    ProbeStep::PartialRebuild { from, to, defs } => {
                        serde_json::json!({"op":"partial-rebuild","from":from,"to":to,"defs":defs})
                    }
                    ProbeStep::DeclarationInvalidate { from, to } => {
                        serde_json::json!({"op":"declaration-invalidate","from":from,"to":to})
                    }
                    ProbeStep::ResidentSlice { slices } => {
                        serde_json::json!({"op":"resident-slice","slices":slices})
                    }
                    ProbeStep::ResidentApply => serde_json::json!({"op":"resident-apply"}),
                    ProbeStep::ResidentFinish => serde_json::json!({"op":"resident-finish"}),
                })
                .collect::<Vec<_>>()
        })
        .into();
    row["resolved_probe_oracles"] = oracle
        .probe
        .as_ref()
        .map(|steps| {
            steps.iter().map(|s| serde_json::json!({
        "cache":s.cache,"compute_calls":s.compute_calls,"bindings_file":s.bindings_file,
        "bindings_sha256":s.bindings_sha256,"stages_include":s.stages_include,
    })).collect::<Vec<_>>()
        })
        .into();
    row["typed_oracle"] = check
        .expect
        .typed
        .as_ref()
        .map(|t| serde_json::json!({"kind":t.kind,"bits":t.bits}))
        .into();
    row["av_channel"] = serde_json::json!(oracle.av_channel);
    // The original /1 metadata keeps its old projection. Only /2 adds the
    // already-resolved oracle sidecars to its explicit dependency list.
    let paths = row["input_paths"].as_array_mut().expect("inventory paths");
    for sidecar in oracle
        .probe
        .iter()
        .flatten()
        .filter_map(|step| step.bindings_file.as_ref())
    {
        paths.push(serde_json::json!(fixture_base.join(sidecar)));
    }
}

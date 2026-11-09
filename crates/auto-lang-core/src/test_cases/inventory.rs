//! Read-only atomic discovery through the same parser/resolver as execution.
//! Module governance must never reconstruct case/check semantics from regexes.
use std::path::Path;

use auto_test_case::admission::Limits;
use auto_test_case::discovery::{load_suite, CASE_FENCE_LANGUAGE};
use auto_test_case::ir::{resolve_case, resolve_oracle};
use auto_test_case::suite::apply_defaults;

#[path = "inventory_probe.rs"]
mod probe;
#[path = "inventory_wrappers.rs"]
mod wrappers;

/// The original parent constants/tables, not a reconstructed language model.
pub fn parent_contract_inventory() -> serde_json::Value {
    serde_json::json!({
        "schema":"test-parent-contract-inventory/1", "scope":"discovery", "executed":0,
        "parents": {
            "av":super::ac_av::contract_inventory::snapshot(),
            "native":super::ac_mn::contract_inventory(),
            "abi":super::abi::contract_inventory(),
        },
    })
}

/// Production file discoverers shared with entry.rs and sys_env.rs. Discovery
/// never claims that a Rust wrapper selected or executed the returned cases.
pub fn legacy_inventory(test_root: &Path) -> Result<serde_json::Value, String> {
    use crate::test_runner;
    use auto_test_case::sha256::{hex, sha256};
    fn file(path: &Path) -> Result<serde_json::Value, String> {
        if !path.is_file() {
            return Ok(serde_json::json!({"path":path,"present":false,"sha256":null}));
        }
        let bytes = std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
        Ok(serde_json::json!({"path":path,"present":true,"sha256":hex(&sha256(&bytes))}))
    }
    if !test_root.is_dir() {
        return Err("legacy test input root missing".into());
    }
    let mut rows = Vec::new();
    for case in test_runner::discover_vm_tests(&test_root.join("vm")) {
        let oracles = [case.expected_out, case.expected_result, case.expected_error]
            .into_iter()
            .flatten()
            .map(|p| file(&p))
            .collect::<Result<Vec<_>, _>>()?;
        rows.push(
            serde_json::json!({"family":"vm","id":case.name,"source":file(&case.source_file)?,
            "bootstrap":case.is_bootstrap,"oracles":oracles}),
        );
    }
    for (dir, name) in [("a2r", "a2r"), ("cookbook", "cookbook")] {
        for case in test_runner::discover_a2r_tests(&test_root.join(dir), name) {
            rows.push(
                serde_json::json!({"family":name,"id":case.name,"source":file(&case.source_file)?,
                "oracles":[file(&case.expected_file)?]}),
            );
        }
    }
    for case in test_runner::discover_a2c_tests(&test_root.join("a2c")) {
        let mut oracles = vec![file(&case.expected_c)?, file(&case.expected_h)?];
        if let Some(path) = case.expected_error {
            oracles.push(file(&path)?);
        }
        rows.push(serde_json::json!({"family":"a2c","id":case.name,"source":file(&case.source_file)?,"oracles":oracles}));
    }
    for case in test_runner::discover_a2ts_tests(&test_root.join("a2ts")) {
        rows.push(
            serde_json::json!({"family":"a2ts","id":case.name,"source":file(&case.source_file)?,
            "oracles":[file(&case.expected_file)?]}),
        );
    }
    rows.sort_by_key(|r| {
        (
            r["family"].as_str().unwrap().to_owned(),
            r["id"].as_str().unwrap().to_owned(),
        )
    });
    if rows.is_empty() {
        return Err("legacy runtime discovery selected zero cases".into());
    }
    Ok(
        serde_json::json!({"schema":"test-legacy-inventory/1","scope":"discovery","executed":0,
                         "root":test_root,"cases":rows,"wrapper_inventory":wrappers::discover(test_root)?}),
    )
}

pub fn suite_inventory(path: &Path, limits: &Limits) -> Result<serde_json::Value, String> {
    suite_inventory_impl(path, limits, false)
}

/// PLAN-941: independent current expectations for an evidence consumer.
/// Uses the execution parser's own typed fingerprints, without executing a
/// check or reading a previous child verdict. The original /1 stays unchanged.
pub fn suite_identity_inventory(path: &Path, limits: &Limits) -> Result<serde_json::Value, String> {
    suite_inventory_impl(path, limits, true)
}

fn suite_inventory_impl(
    path: &Path,
    limits: &Limits,
    identities: bool,
) -> Result<serde_json::Value, String> {
    let loaded = load_suite(path, limits).map_err(|e| e.to_string())?;
    if !loaded.errors.is_empty() {
        return Err(loaded
            .errors
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join("\n"));
    }
    let mut checks = Vec::new();
    for file in &loaded.cases {
        let fences: Vec<_> = file
            .doc
            .fences()
            .iter()
            .filter(|f| f.language == CASE_FENCE_LANGUAGE)
            .collect();
        if fences.len() != file.specs.len() {
            return Err(format!("case fence/spec mismatch: {}", file.path.display()));
        }
        for (spec, fence) in file.specs.iter().zip(fences) {
            let case = resolve_case(spec.clone(), &file.doc, &loaded.fixture_base, limits)
                .map_err(|e| e.to_string())?;
            let files: Vec<_> = case
                .files
                .iter()
                .map(|f| {
                    serde_json::json!({
                        "virtual_path": f.virtual_path, "sha256": f.sha256,
                    })
                })
                .collect();
            for check in &spec.checks {
                let (effective, origins) = apply_defaults(check, &loaded.suite.defaults);
                let oracle = resolve_oracle(&spec.id, &effective, &loaded.fixture_base, limits)
                    .map_err(|e| e.to_string())?;
                let artifacts: Vec<_> = oracle
                    .artifacts
                    .iter()
                    .map(|a| {
                        serde_json::json!({
                            "path": a.path, "file": a.file, "sha256": a.sha256,
                        })
                    })
                    .collect();
                let defaults: Vec<_> = origins
                    .iter()
                    .map(|o| {
                        serde_json::json!({
                            "field": o.field, "source": o.source,
                        })
                    })
                    .collect();
                let skip = effective.skip.as_ref().or(spec.status_skip.as_ref());
                let mut row = serde_json::json!({
                    "suite": loaded.suite.id,
                    "case": spec.id, "check": effective.id,
                    "case_file": file.path, "case_file_sha256": file.sha256,
                    "span": {"open_line": fence.open_line, "close_line": fence.close_line,
                             "payload_start": fence.payload.start, "payload_end": fence.payload.end},
                    "source_sha256": case.source_sha256, "source_files": files,
                    "source_ref": format!("{:?}", spec.source.payload),
                    "input_paths": std::iter::once(&spec.source.payload).chain(spec.files.iter().map(|f| &f.payload))
                        .filter_map(|p|match p {auto_test_case::contract::PayloadSpec::File(rel)=>Some(loaded.fixture_base.join(rel)),_=>None})
                        .chain(oracle.artifacts.iter().map(|a|loaded.fixture_base.join(&a.file))).collect::<Vec<_>>(),
                    "oracle_sha256": oracle.oracle_sha256, "oracle_artifacts": artifacts,
                    "expected_exit": oracle.exit,
                    "adapter": effective.adapter.as_str(), "action": effective.action.as_str(),
                    "recipe": effective.options.probe_recipe,
                    "profile": effective.options.probe_av.as_ref().and_then(|a| a.profile.as_ref()),
                    "tier": effective.tier, "product_from": effective.product_from,
                    "state": if skip.is_some() { "SKIP" } else { "RUN" },
                    "skip_reason": skip.map(|s| &s.reason), "skip_owner": skip.and_then(|s| s.owner.as_ref()),
                    "defaults": defaults,
                });
                if identities {
                    row["source_origin"] = serde_json::json!({"kind":match &spec.source.payload {
                        auto_test_case::contract::PayloadSpec::File(_) => "file",
                        auto_test_case::contract::PayloadSpec::Text(_) => "text",
                        auto_test_case::contract::PayloadSpec::Block(_) => "block",
                    }});
                    probe::append(&mut row, &effective, &oracle, &loaded.fixture_base);
                    row["contract_sha256"] = super::fingerprint(spec).into();
                    row["check_sha256"] = super::fingerprint(&effective).into();
                    use auto_test_case::contract::{Compare, Completion, MessageMatcher};
                    row["completion"] = match &oracle.completion {
                        Completion::Success => serde_json::json!({ "kind": "success" }),
                        Completion::Error {
                            stage,
                            code,
                            message,
                        } => serde_json::json!({
                            "kind": "error", "stage": stage.as_str(), "code": code,
                            "message": message.as_ref().map(|m| match m {
                                MessageMatcher::Exact(text) => serde_json::json!({"kind":"exact", "text":text}),
                                MessageMatcher::Contains(text) => serde_json::json!({"kind":"contains", "text":text}),
                            }),
                        }),
                    };
                    let budget = effective.options.effective_budget();
                    row["ac_options"] = serde_json::json!({
                        "profile": effective.options.effective_profile().as_str(),
                        "entry": effective.options.effective_entry(),
                        "cache": effective.options.effective_cache().as_str(),
                        "grammar": if effective.adapter == auto_test_case::contract::Adapter::Syntax {
                            Some(effective.options.effective_grammar())
                        } else { None },
                        "capabilities": effective.options.effective_capabilities(),
                        "budget": {"build_ms":budget.build_ms, "run_ms":budget.run_ms, "total_ms":budget.total_ms},
                    });
                    if let Some(abi) = &effective.options.probe_abi {
                        use auto_test_case::contract::ProbeStep;
                        row["abi_options"] = serde_json::json!({"leg": abi.leg});
                        let steps = effective.options.probe_steps.as_deref().unwrap_or_default();
                        let mut exported = Vec::new();
                        for step in steps {
                            exported.push(match step {
                                ProbeStep::Query => serde_json::json!({"op":"query"}),
                                ProbeStep::Restart => serde_json::json!({"op":"restart"}),
                                ProbeStep::EditSource { from, to } => {
                                    serde_json::json!({"op":"edit-source", "from":from, "to":to})
                                }
                                _ => {
                                    return Err("unsupported ABI step in identity inventory".into())
                                }
                            });
                        }
                        row["abi_steps"] = exported.into();
                        row["probe_oracles"] = oracle.probe.as_ref().map(|steps| steps.iter().map(|s| serde_json::json!({
                            "cache":s.cache, "compute_calls":s.compute_calls, "bindings_sha256":s.bindings_sha256,
                            "stages_include":s.stages_include,
                        })).collect::<Vec<_>>()).into();
                    }
                    row["artifact_modes"] = oracle.artifacts.iter().map(|a| serde_json::json!({
                        "path":a.path, "compare":match a.compare { Compare::Bytes => "bytes", Compare::Lf => "lf" },
                    })).collect::<Vec<_>>().into();
                }
                checks.push(row);
            }
        }
    }
    if checks.is_empty() {
        return Err("suite inventory selected zero atomic checks".into());
    }
    Ok(serde_json::json!({
        "schema": if identities { "test-case-inventory/2" } else { "test-case-inventory/1" }, "scope": "discovery", "executed": 0,
        "suite": loaded.suite.id, "suite_path": loaded.suite_path,
        "fixture_base": loaded.fixture_base,
        "suite_sha256": loaded.suite_sha256, "checks": checks,
    }))
}

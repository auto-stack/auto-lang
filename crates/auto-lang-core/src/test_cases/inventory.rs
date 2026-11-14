//! Read-only atomic discovery through the same parser/resolver as execution.
//! Module governance must never reconstruct case/check semantics from regexes.
use std::path::Path;

use auto_test_case::admission::Limits;
use auto_test_case::contract::{
    Compare, Completion, EntryMode, MessageMatcher, Options, PayloadSpec, ProbeStep, SkipSpec,
    StreamExpect,
};
use auto_test_case::discovery::{load_suite, CaseFile, CaseFileFormat, CASE_FENCE_LANGUAGE};
use auto_test_case::ir::{resolve_case, resolve_oracle};
use auto_test_case::sha256::{hex, sha256};
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
        let spans = file_case_spans(file)?;
        for (spec, (open_line, close_line)) in file.specs.iter().zip(spans) {
            let check_sections = doc_check_sections(file, open_line);
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
                            "path": a.path, "file": a.file,
                            "inline": a.origin.map(|(open, close)| serde_json::json!({
                                "open_line": open, "close_line": close,
                            })),
                            "sha256": a.sha256,
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
                    "format": match file.format {
                        CaseFileFormat::LegacyFences => "fences",
                        CaseFileFormat::Document => "document",
                    },
                    "span": {"open_line": open_line, "close_line": close_line},
                    "doc_section_line": check_sections.get(effective.id.as_str()).copied(),
                    "source_sha256": case.source_sha256, "source_files": files,
                    "source_ref": format!("{:?}", spec.source.payload),
                    "input_paths": std::iter::once(&spec.source.payload).chain(spec.files.iter().map(|f| &f.payload))
                        .filter_map(|p|match p {auto_test_case::contract::PayloadSpec::File(rel)=>Some(loaded.fixture_base.join(rel)),_=>None})
                        .chain(oracle.artifacts.iter().filter_map(|a|a.file.as_ref().map(|f|loaded.fixture_base.join(f)))).collect::<Vec<_>>(),
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
                                    return Err("unsupported ABI step in identity inventory".into());
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

fn file_case_spans(file: &CaseFile) -> Result<Vec<(usize, Option<usize>)>, String> {
    let spans: Vec<(usize, Option<usize>)> = match file.format {
        CaseFileFormat::LegacyFences => {
            let fences: Vec<_> = file
                .doc
                .fences()
                .iter()
                .filter(|f| f.language == CASE_FENCE_LANGUAGE)
                .collect();
            if fences.len() != file.specs.len() {
                return Err(format!("case fence/spec mismatch: {}", file.path.display()));
            }
            fences.iter().map(|f| (f.open_line, f.close_line)).collect()
        }
        CaseFileFormat::Document => file
            .doc_spans
            .iter()
            .map(|s| (s.heading_line, None))
            .collect(),
    };
    if spans.len() != file.specs.len() {
        return Err(format!("case span/spec mismatch: {}", file.path.display()));
    }
    Ok(spans)
}

fn doc_check_sections(
    file: &CaseFile,
    open_line: usize,
) -> std::collections::BTreeMap<&str, usize> {
    match file.format {
        CaseFileFormat::Document => file
            .doc_spans
            .iter()
            .find(|s| s.heading_line == open_line)
            .map(|s| {
                s.check_spans
                    .iter()
                    .map(|c| (c.id.as_str(), c.section_line))
                    .collect()
            })
            .unwrap_or_default(),
        CaseFileFormat::LegacyFences => std::collections::BTreeMap::new(),
    }
}

fn skip_json(s: &SkipSpec) -> serde_json::Value {
    serde_json::json!({
        "reason": s.reason,
        "issue": s.issue,
        "owner": s.owner,
    })
}

fn compare_str(c: Compare) -> &'static str {
    match c {
        Compare::Bytes => "bytes",
        Compare::Lf => "lf",
    }
}

fn stream_json(s: &Option<StreamExpect>) -> serde_json::Value {
    match s {
        None => serde_json::Value::Null,
        Some(st) => {
            let b = st.text.as_bytes();
            serde_json::json!({
                "compare": compare_str(st.compare),
                "bytes": b.len(),
                "sha256": hex(&sha256(b)),
                "text": st.text,
            })
        }
    }
}

fn entry_mode_str(mode: EntryMode) -> &'static str {
    match mode {
        EntryMode::Program => "program",
        EntryMode::Script => "script",
        EntryMode::Expression => "expression",
        EntryMode::Module => "module",
    }
}

fn probe_step_json(step: &ProbeStep) -> serde_json::Value {
    match step {
        ProbeStep::Query => serde_json::json!({"op": "query"}),
        ProbeStep::Restart => serde_json::json!({"op": "restart"}),
        ProbeStep::Run => serde_json::json!({"op": "run"}),
        ProbeStep::EditSource { from, to } => {
            serde_json::json!({"op": "edit-source", "from": from, "to": to})
        }
        ProbeStep::PartialRebuild { from, to, defs } => {
            serde_json::json!({"op": "partial-rebuild", "from": from, "to": to, "defs": defs})
        }
        ProbeStep::DeclarationInvalidate { from, to } => {
            serde_json::json!({"op": "declaration-invalidate", "from": from, "to": to})
        }
        ProbeStep::ResidentSlice { slices } => {
            serde_json::json!({"op": "resident-slice", "slices": slices})
        }
        ProbeStep::ResidentApply => serde_json::json!({"op": "resident-apply"}),
        ProbeStep::ResidentFinish => serde_json::json!({"op": "resident-finish"}),
    }
}

fn options_json(opt: &Options) -> serde_json::Value {
    serde_json::json!({
        "module_name": opt.module_name,
        "a2r_int_profile": opt.a2r_int_profile,
        "a2r_resource_profile": opt.a2r_resource_profile,
        "ac_profile": opt.ac_profile.map(|p| p.as_str()),
        "ac_target": opt.ac_target,
        "ac_entry": opt.ac_entry,
        "ac_cache": opt.ac_cache.map(|c| c.as_str()),
        "ac_budget": opt.ac_budget.map(|b| serde_json::json!({
            "build_ms": b.build_ms,
            "run_ms": b.run_ms,
            "total_ms": b.total_ms,
        })),
        "ac_capabilities": opt.ac_capabilities,
        "ac_support_lib": opt.ac_support_lib,
        "syntax_grammar": opt.syntax_grammar,
        "syntax_edits": opt.syntax_edits.as_ref().map(|e| format!("{e:?}")),
        "probe_recipe": opt.probe_recipe,
        "probe_steps": opt.probe_steps.as_ref().map(|steps| steps.iter().map(probe_step_json).collect::<Vec<_>>()),
        "probe_sema": opt.probe_sema.as_ref().map(|s| format!("{s:?}")),
        "probe_abi": opt.probe_abi.as_ref().map(|a| format!("{a:?}")),
        "probe_av": opt.probe_av.as_ref().map(|av| serde_json::json!({
            "entry": av.entry,
            "caps": av.caps,
            "session": av.session,
            "profile": av.profile,
            "invalidation": av.invalidation.is_some(),
            "resource": av.resource.is_some(),
        })),
        "probe_cte": opt.probe_cte.as_ref().map(|c| format!("{c:?}")),
        "probe_mn": opt.probe_mn.as_ref().map(|mn| serde_json::json!({
            "entry": mn.entry,
            "caps": mn.caps,
            "session": mn.session,
            "typed": mn.typed.is_some(),
            "diagnostics": mn.diagnostics,
            "declaration": mn.declaration.is_some(),
            "resource": mn.resource.is_some(),
            "history": mn.history.is_some(),
            "turnaround": mn.turnaround.is_some(),
        })),
        "probe_a2x_hir": opt.probe_a2x_hir.as_ref().map(|a| format!("{a:?}")),
        "avm_fail_alloc_at": opt.avm_fail_alloc_at,
    })
}

/// PLAN-1057 §5.8 read-only canonical projection (`autolang.case-migration/1`)
/// for side-by-side semantic equivalence & origin traceability verification.
pub fn suite_migration_inventory(
    path: &Path,
    limits: &Limits,
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
    let mut rows = Vec::new();
    for file in &loaded.cases {
        let spans = file_case_spans(file)?;
        for (spec, (open_line, close_line)) in file.specs.iter().zip(spans) {
            let check_sections = doc_check_sections(file, open_line);
            let case = resolve_case(spec.clone(), &file.doc, &loaded.fixture_base, limits)
                .map_err(|e| e.to_string())?;
            let entry_file = case
                .files
                .first()
                .ok_or_else(|| format!("case {} has no resolved entry source", spec.id))?;
            let extra_files: Vec<_> = case
                .files
                .iter()
                .skip(1)
                .map(|rfile| {
                    serde_json::json!({
                        "virtual_path": rfile.virtual_path,
                        "bytes": rfile.bytes.len(),
                        "sha256": rfile.sha256,
                    })
                })
                .collect();
            let mut input_paths = Vec::new();
            if let PayloadSpec::File(rel) = &spec.source.payload {
                input_paths.push(loaded.fixture_base.join(rel));
            }
            for f in &spec.files {
                if let PayloadSpec::File(rel) = &f.payload {
                    input_paths.push(loaded.fixture_base.join(rel));
                }
            }
            for (ordinal, check) in spec.checks.iter().enumerate() {
                let (effective, _origins) = apply_defaults(check, &loaded.suite.defaults);
                let oracle = resolve_oracle(&spec.id, &effective, &loaded.fixture_base, limits)
                    .map_err(|e| e.to_string())?;
                let completion_json = match &oracle.completion {
                    Completion::Success => serde_json::json!({
                        "kind": "success",
                        "stage": null,
                        "code": null,
                        "message": null,
                    }),
                    Completion::Error {
                        stage,
                        code,
                        message,
                    } => serde_json::json!({
                        "kind": "error",
                        "stage": stage.as_str(),
                        "code": code,
                        "message": message.as_ref().map(|m| match m {
                            MessageMatcher::Exact(t) => serde_json::json!({"kind": "exact", "text": t}),
                            MessageMatcher::Contains(t) => serde_json::json!({"kind": "contains", "text": t}),
                        }),
                    }),
                };
                let artifacts_semantic: Vec<_> = oracle
                    .artifacts
                    .iter()
                    .map(|a| {
                        serde_json::json!({
                            "path": a.path,
                            "compare": compare_str(a.compare),
                            "bytes": a.bytes.len(),
                            "sha256": a.sha256,
                            "text_utf8": std::str::from_utf8(&a.bytes).ok(),
                        })
                    })
                    .collect();
                let artifacts_origin: Vec<_> = oracle
                    .artifacts
                    .iter()
                    .map(|a| {
                        serde_json::json!({
                            "path": a.path,
                            "file": a.file,
                            "origin_lines": a.origin.map(|(o, c)| serde_json::json!({"open": o, "close": c})),
                            "sha256": a.sha256,
                        })
                    })
                    .collect();
                rows.push(serde_json::json!({
                    "semantic": {
                        "case": {
                            "id": spec.id,
                            "tags": spec.tags,
                            "links": spec.links,
                            "entry_mode": entry_mode_str(spec.entry),
                            "status_skip": spec.status_skip.as_ref().map(skip_json),
                            "source": {
                                "syntax": spec.source.syntax.as_str(),
                                "virtual_path": entry_file.virtual_path,
                                "bytes": entry_file.bytes.len(),
                                "sha256": entry_file.sha256,
                            },
                            "files": extra_files,
                        },
                        "check": {
                            "id": effective.id,
                            "ordinal": ordinal,
                            "adapter": effective.adapter.as_str(),
                            "action": effective.action.as_str(),
                            "tier": effective.tier,
                            "product_from": effective.product_from,
                            "skip": effective.skip.as_ref().map(skip_json),
                        },
                        "options": options_json(&effective.options),
                        "expect": {
                            "completion": completion_json,
                            "exit": oracle.exit,
                            "av_channel": oracle.av_channel,
                            "result": oracle.result.as_ref().map(|r| serde_json::json!({
                                "format": r.format,
                                "text": r.text,
                            })),
                            "stdout": stream_json(&oracle.stdout),
                            "stderr": stream_json(&oracle.stderr),
                            "trace": stream_json(&oracle.trace),
                            "artifacts": artifacts_semantic,
                            "perf": oracle.perf.map(|p| serde_json::json!({"total_ms_max": p.total_ms_max})),
                            "typed": effective.expect.typed.as_ref().map(|t| serde_json::json!({"kind": t.kind, "bits": t.bits})),
                            "probe": oracle.probe.as_ref().map(|steps| format!("{steps:?}")),
                        },
                    },
                    "origin": {
                        "suite": loaded.suite.id,
                        "suite_path": loaded.suite_path,
                        "suite_sha256": loaded.suite_sha256,
                        "fixture_base": loaded.fixture_base,
                        "case_file": file.path,
                        "case_file_sha256": file.sha256,
                        "format": match file.format {
                            CaseFileFormat::LegacyFences => "fences",
                            CaseFileFormat::Document => "document",
                        },
                        "case_open_line": open_line,
                        "case_close_line": close_line,
                        "doc_section_line": check_sections.get(effective.id.as_str()).copied(),
                        "source_ref": format!("{:?}", spec.source.payload),
                        "input_paths": input_paths,
                        "source_sha256": case.source_sha256,
                        "oracle_sha256": oracle.oracle_sha256,
                        "artifacts": artifacts_origin,
                        "case_dir": super::sanitize(&spec.id),
                        "check_dir": super::sanitize(&effective.id),
                    },
                }));
            }
        }
    }
    if rows.is_empty() {
        return Err("suite migration inventory selected zero atomic checks".into());
    }
    Ok(serde_json::json!({
        "schema": "autolang.case-migration/1",
        "suite": loaded.suite.id,
        "suite_path": loaded.suite_path,
        "suite_sha256": loaded.suite_sha256,
        "fixture_base": loaded.fixture_base,
        "checks": rows,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn cases_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../auto-lang/test/cases")
    }

    #[test]
    fn migration_inventory_projects_exact_legacy_scalar_console_and_immutable_rows() {
        let base_inv = suite_migration_inventory(
            &cases_root().join("ace-m1/ace-m1-base.suite.ad"),
            &Limits::defaults(),
        )
        .unwrap();
        assert_eq!(base_inv["schema"], "autolang.case-migration/1");
        let base_rows = base_inv["checks"].as_array().unwrap();
        let b02_a2r = base_rows
            .iter()
            .find(|r| {
                r["semantic"]["case"]["id"] == "ace.m1.b02"
                    && r["semantic"]["check"]["id"] == "a2r-snapshot"
            })
            .unwrap();
        assert_eq!(b02_a2r["semantic"]["case"]["entry_mode"], "program");
        assert_eq!(
            b02_a2r["semantic"]["case"]["tags"],
            serde_json::json!(["ace-m1", "four-path", "scalar"])
        );
        assert_eq!(
            b02_a2r["semantic"]["case"]["links"],
            serde_json::json!(["PLAN-852"])
        );
        assert_eq!(b02_a2r["semantic"]["options"]["module_name"], "m1_b02");
        assert_eq!(
            b02_a2r["semantic"]["expect"]["artifacts"][0]["path"],
            "m1_b02.rs"
        );
        assert_eq!(b02_a2r["origin"]["case_dir"], "ace.m1.b02");
        assert_eq!(b02_a2r["origin"]["check_dir"], "a2r-snapshot");

        let con_inv = suite_migration_inventory(
            &cases_root().join("ace-m1/console-863/console-863.suite.ad"),
            &Limits::defaults(),
        )
        .unwrap();
        let con_rows = con_inv["checks"].as_array().unwrap();
        let o07 = con_rows
            .iter()
            .find(|r| r["semantic"]["case"]["id"] == "ace.m1.o07")
            .unwrap();
        assert_eq!(o07["semantic"]["expect"]["av_channel"], "unit");
        assert_eq!(o07["semantic"]["expect"]["exit"], serde_json::Value::Null);
        assert_eq!(o07["semantic"]["expect"]["stdout"]["text"], "7\n");
        assert_eq!(o07["semantic"]["expect"]["stderr"]["text"], "unit\n");

        let o08 = con_rows
            .iter()
            .find(|r| r["semantic"]["case"]["id"] == "ace.m1.o08")
            .unwrap();
        assert_eq!(o08["semantic"]["case"]["entry_mode"], "program");
        assert_eq!(o08["semantic"]["expect"]["av_channel"], "value");
        assert_eq!(o08["semantic"]["expect"]["exit"], 42);

        let o10 = con_rows
            .iter()
            .find(|r| r["semantic"]["case"]["id"] == "ace.m1.o10")
            .unwrap();
        assert_eq!(o10["semantic"]["expect"]["completion"]["kind"], "error");
        assert_eq!(o10["semantic"]["expect"]["completion"]["stage"], "run");
        assert_eq!(
            o10["semantic"]["expect"]["completion"]["code"],
            "av.trap.div-by-zero"
        );
        assert_eq!(o10["semantic"]["expect"]["stdout"]["text"], "before\n");
        assert_eq!(o10["semantic"]["expect"]["stderr"]["text"], "fail\n");

        let o12 = con_rows
            .iter()
            .find(|r| r["semantic"]["case"]["id"] == "ace.m1.o12")
            .unwrap();
        assert_eq!(
            o12["semantic"]["options"]["probe_av"]["caps"],
            serde_json::json!([])
        );
        assert_eq!(o12["semantic"]["expect"]["completion"]["stage"], "parse");
        assert_eq!(
            o12["semantic"]["expect"]["completion"]["code"],
            "host.unauthorized"
        );

        let imm_inv = suite_migration_inventory(
            &cases_root().join("ace-m1/immutable-avm-920.suite.ad"),
            &Limits::defaults(),
        )
        .unwrap();
        let imm_rows = imm_inv["checks"].as_array().unwrap();
        let top = imm_rows
            .iter()
            .find(|r| r["semantic"]["case"]["id"] == "ace.m1.ia920.top")
            .unwrap();
        assert_eq!(top["semantic"]["case"]["entry_mode"], "script");
        assert_eq!(top["semantic"]["expect"]["completion"]["stage"], "type");
        assert_eq!(
            top["semantic"]["expect"]["completion"]["code"],
            "auto_name_E0203"
        );
        assert_eq!(top["origin"]["case_dir"], "ace.m1.ia920.top");
        assert_eq!(top["origin"]["check_dir"], "autovm");
    }
}

//! PLAN-1042 `ac-a2py/*` recipe adapter (`probe/run` seam):
//! bridges the checked HIR -> `ace-a2x-python` (`daily-emit-py` / `run-py` ops on
//! the prebuilt `a2x-worker`) into `auto-lang-core::test_cases` and `auto-test-case`.
//!
//! Admission & trust rules enforced parent-side:
//! * Emit recipes (`ac-a2py/hir-{scalar,record-text,collections}-emit-v1`) invoke
//!   `daily-emit-py` (`spawned_python == 0`, `execution_claimed == false`) and
//!   byte-compare the emitted files against the golden oracle sidecars.
//! * The execution recipe (`ac-a2py/hir-stdlib-run-v1`) invokes `run-py`,
//!   which MUST spawn real OS Python (`spawned_python == 1`, `execution_claimed == true`,
//!   non-empty `python_identity`), emit via `ace-a2x-python/modules/1`, and return
//!   observed `stdout_hex` / `stderr_hex` / `exit_code` for parent-side comparison.
//! * Missing Python, emit-only impersonation, legacy `PythonTrans`, or AutoVM
//!   fallback NEVER pass: they produce controlled errors/rejections.

use std::path::Path;
use std::process::Command;
use std::time::Instant;

use auto_test_case::compare::{compare_bytes, Mismatch};
use auto_test_case::contract::{CheckSpec, Completion, ProbeStep, Syntax};
use auto_test_case::ir::{ResolvedCase, ResolvedOracle};
use auto_test_case::report::{ArtifactRef, Status};
use auto_test_case::sha256::{hex, sha256};

pub use super::ac_a2c_hir::{
    worker_binary, path_domain_problems, A2X_WORKER_PROTOCOL, A2X_WORKER_SHA_ENV,
};
use super::avm::Verdict;

pub const A2PY_HIR_RECIPE: &str = "ac-a2py/hir-scalar-emit-v1";
pub const A2PY_RT_HIR_RECIPE: &str = "ac-a2py/hir-record-text-emit-v1";
pub const A2PY_COLL_HIR_RECIPE: &str = "ac-a2py/hir-collections-emit-v1";
pub const A2PY_STDLIB_RUN_RECIPE: &str = "ac-a2py/hir-stdlib-run-v1";
pub const PY_PRODUCER: &str = "ace-a2x-python/modules/1";

pub fn is_a2py_recipe(recipe: &str) -> bool {
    matches!(
        recipe,
        A2PY_HIR_RECIPE | A2PY_RT_HIR_RECIPE | A2PY_COLL_HIR_RECIPE | A2PY_STDLIB_RUN_RECIPE
    )
}

pub fn decode_hex(s: &str) -> Result<Vec<u8>, String> {
    if s.len() % 2 != 0 {
        return Err(format!("odd hex length: {}", s.len()));
    }
    let mut out = Vec::with_capacity(s.len() / 2);
    let bytes = s.as_bytes();
    for i in (0..bytes.len()).step_by(2) {
        let hi = (bytes[i] as char)
            .to_digit(16)
            .ok_or_else(|| format!("invalid hex char at byte {i}"))?;
        let lo = (bytes[i + 1] as char)
            .to_digit(16)
            .ok_or_else(|| format!("invalid hex char at byte {}", i + 1))?;
        out.push(((hi << 4) | lo) as u8);
    }
    Ok(out)
}

/// Parent-side admission verification for Python worker responses.
pub fn validate_worker_response(
    resp: &serde_json::Value,
    recipe: &str,
    disk_manifest_sha: &str,
) -> Result<(), String> {
    if resp["status"] != "ok" {
        return Err(format!("worker controlled failure: {}", resp["detail"]));
    }
    if resp["protocol"].as_str() != Some(A2X_WORKER_PROTOCOL)
        || resp["recipe"].as_str() != Some(recipe)
    {
        return Err(format!(
            "worker identity mismatch: protocol={:?} recipe={:?}",
            resp["protocol"], resp["recipe"]
        ));
    }
    if resp["target"].as_str() != Some("python")
        || resp["producer"].as_str() != Some(PY_PRODUCER)
    {
        return Err(format!(
            "python producer mismatch: target={:?} producer={:?}",
            resp["target"], resp["producer"]
        ));
    }
    if resp["manifest_sha256"].as_str() != Some(disk_manifest_sha) {
        return Err("worker manifest hash disagrees with the emitted file".into());
    }
    let is_run = recipe == A2PY_STDLIB_RUN_RECIPE;
    if is_run {
        if resp["mode"].as_str() != Some("run-py")
            || resp["execution_claimed"].as_bool() != Some(true)
            || resp["spawned_python"].as_i64() != Some(1)
        {
            return Err(format!(
                "execution recipe `{recipe}` requires real Python execution (`mode=run-py`, `execution_claimed=true`, `spawned_python=1`); emit-only cannot pass"
            ));
        }
        let py = resp.get("python_identity").and_then(|v| v.as_object());
        let has_identity = py
            .map(|o| {
                o.get("exe").and_then(|x| x.as_str()).is_some_and(|s| !s.is_empty())
                    && o.get("version")
                        .and_then(|x| x.as_str())
                        .is_some_and(|s| !s.is_empty())
                    && o.get("version_sha256")
                        .and_then(|x| x.as_str())
                        .is_some_and(|s| s.len() == 64)
                    && o.get("os")
                        .and_then(|x| x.as_str())
                        .is_some_and(|s| !s.is_empty())
            })
            .unwrap_or(false);
        if !has_identity {
            return Err("execution response missing complete `python_identity`".into());
        }
    } else if resp["mode"].as_str() != Some("daily-emit-py")
        || resp["execution_claimed"].as_bool() != Some(false)
        || resp["spawned_python"].as_i64() != Some(0)
        || resp["spawned_rustc"].as_i64() != Some(0)
        || resp["spawned_cl"].as_i64() != Some(0)
    {
        return Err(format!(
            "daily Python emit `{recipe}` must stay spawn-zero with execution_claimed=false"
        ));
    }
    Ok(())
}

pub fn run_check(
    case: &ResolvedCase,
    check: &CheckSpec,
    oracle: &ResolvedOracle,
    _project_dir: &Path,
    artifact_dir: &Path,
    _run_dir: &Path,
) -> Result<Verdict, String> {
    let started = Instant::now();
    if case.spec.source.syntax != Syntax::Auto {
        return Err(format!(
            "ac-a2py consumes auto source files; syntax {:?} is not accepted",
            case.spec.source.syntax.as_str()
        ));
    }
    let recipe = check
        .options
        .probe_recipe
        .as_deref()
        .ok_or("probe check without a recipe (contract violation)")?;
    if !is_a2py_recipe(recipe) {
        return Err("probe check with an unregistered a2py recipe (contract violation)".into());
    }
    let binding = check
        .options
        .probe_a2x_hir
        .as_ref()
        .ok_or("probe check without an a2x_hir binding (contract violation)")?;
    let profile_ok = match recipe {
        A2PY_RT_HIR_RECIPE => {
            matches!(binding.profile.as_str(), "record" | "text" | "text-record")
        }
        A2PY_COLL_HIR_RECIPE => binding.profile == "collections",
        A2PY_STDLIB_RUN_RECIPE => matches!(
            binding.profile.as_str(),
            "core" | "scalar" | "record" | "text" | "text-record" | "collections"
        ),
        _ => matches!(binding.profile.as_str(), "core" | "scalar"),
    };
    if !profile_ok {
        return Err(format!(
            "{recipe} does not accept profile {:?}",
            binding.profile
        ));
    }
    let steps = check
        .options
        .probe_steps
        .as_ref()
        .ok_or("probe check without steps (contract violation)")?;
    if steps.len() != 1 || !matches!(steps[0], ProbeStep::Run) {
        return Err(format!("{recipe} takes exactly one `run` step"));
    }
    let source_bytes = case
        .files
        .first()
        .map(|f| f.bytes.clone())
        .ok_or_else(|| "a2x_hir adapter needs the resolved entry input".to_string())?;

    let worker = worker_binary()?;
    let worker_bytes = std::fs::read(&worker).map_err(|e| format!("read worker: {e}"))?;
    let worker_sha = hex(&sha256(&worker_bytes));
    if let Ok(pinned) = std::env::var(A2X_WORKER_SHA_ENV) {
        if !pinned.is_empty() && pinned != worker_sha {
            return Ok(Verdict {
                status: Status::Error,
                detail: Some(format!(
                    "worker content credential mismatch: pinned {pinned} != observed {worker_sha}"
                )),
                ..Verdict::pass()
            });
        }
    }

    std::fs::create_dir_all(artifact_dir).map_err(|e| format!("artifact dir: {e}"))?;
    let source_path = artifact_dir.join("source.at");
    std::fs::write(&source_path, &source_bytes).map_err(|e| format!("stage source: {e}"))?;
    let actual_dir = artifact_dir.join("actual");
    let nonce = hex(&sha256(
        format!(
            "{}|{}|a2x-hir-py",
            std::process::id(),
            started.elapsed().as_nanos()
        )
        .as_bytes(),
    ));
    let op = if recipe == A2PY_STDLIB_RUN_RECIPE {
        "run-py"
    } else {
        "daily-emit-py"
    };
    let request = serde_json::json!({
        "op": op,
        "recipe": recipe,
        "source_path": source_path,
        "profile": binding.profile,
        "entry": binding.entry,
        "caps": binding.caps,
        "out_dir": actual_dir,
        "nonce": nonce,
    });
    let request_text = request.to_string();
    let exchange = super::execution_trace::WorkerExchange::start(
        artifact_dir,
        A2X_WORKER_PROTOCOL,
        recipe,
        &worker_sha,
        request_text.as_bytes(),
    )?;
    let out = Command::new(&worker)
        .arg(&request_text)
        .output()
        .map_err(|e| format!("spawn a2x worker {}: {e}", worker.display()))?;
    if let Some(exchange) = exchange {
        exchange.finish(&out.stdout, &out.stderr, out.status.code(), None)?;
    }
    if !out.status.success() {
        return Ok(Verdict {
            status: Status::Error,
            detail: Some(format!(
                "a2x worker failed (exit {:?}): {}",
                out.status.code(),
                String::from_utf8_lossy(&out.stdout).trim()
            )),
            ..Verdict::pass()
        });
    }
    let resp: serde_json::Value = serde_json::from_slice(&out.stdout)
        .map_err(|e| format!("worker response not json: {e}"))?;
    let manifest_bytes = std::fs::read(actual_dir.join("manifest.json"))
        .map_err(|e| format!("read manifest: {e}"))?;
    let manifest_sha = hex(&sha256(&manifest_bytes));
    if let Err(reason) = validate_worker_response(&resp, recipe, &manifest_sha) {
        return Ok(Verdict {
            status: Status::Error,
            detail: Some(reason),
            ..Verdict::pass()
        });
    }

    match &oracle.completion {
        Completion::Success => {
            let mut mismatches = Vec::new();
            let mut refs = Vec::new();
            if recipe == A2PY_STDLIB_RUN_RECIPE {
                let actual_exit = resp["exit_code"].as_i64().map(|x| x as i32);
                if let Some(want_exit) = oracle.exit {
                    if actual_exit != Some(want_exit) {
                        mismatches.push(Mismatch {
                            observation: "exit".into(),
                            kind: "value",
                            expected: want_exit.to_string(),
                            actual: format!("{actual_exit:?}"),
                            first_diff: None,
                        });
                    }
                } else if actual_exit != Some(0) {
                    mismatches.push(Mismatch {
                        observation: "exit".into(),
                        kind: "nonzero",
                        expected: "0".into(),
                        actual: format!("{actual_exit:?}"),
                        first_diff: None,
                    });
                }
                if let Some(want_out) = &oracle.stdout {
                    let got_hex = resp["stdout_hex"].as_str().unwrap_or("");
                    let got_bytes = decode_hex(got_hex).map_err(|e| format!("stdout_hex: {e}"))?;
                    if let Some(m) = compare_bytes(
                        want_out.text.as_bytes(),
                        &got_bytes,
                        want_out.compare,
                        "stdout",
                    ) {
                        mismatches.push(m);
                    }
                }
                if let Some(want_err) = &oracle.stderr {
                    let got_hex = resp["stderr_hex"].as_str().unwrap_or("");
                    let got_bytes = decode_hex(got_hex).map_err(|e| format!("stderr_hex: {e}"))?;
                    if let Some(m) = compare_bytes(
                        want_err.text.as_bytes(),
                        &got_bytes,
                        want_err.compare,
                        "stderr",
                    ) {
                        mismatches.push(m);
                    }
                }
            } else {
                let mut expected_paths: Vec<String> = Vec::new();
                for problem in path_domain_problems(
                    &oracle
                        .artifacts
                        .iter()
                        .map(|w| w.path.clone())
                        .collect::<Vec<_>>(),
                ) {
                    mismatches.push(Mismatch {
                        observation: "artifact paths".into(),
                        kind: "path-domain",
                        expected: "relative normalized unique paths within the output domain".into(),
                        actual: problem,
                        first_diff: None,
                    });
                }
                for want in &oracle.artifacts {
                    expected_paths.push(want.path.clone());
                    let path = actual_dir.join(&want.path);
                    match std::fs::read(&path) {
                        Err(_) => mismatches.push(Mismatch {
                            observation: format!("artifact {}", want.path),
                            kind: "missing",
                            expected: format!("{} bytes", want.bytes.len()),
                            actual: "absent".to_string(),
                            first_diff: None,
                        }),
                        Ok(actual) => {
                            if let Some(m) = compare_bytes(
                                &want.bytes,
                                &actual,
                                want.compare,
                                &format!("artifact {}", want.path),
                            ) {
                                mismatches.push(m);
                            }
                            refs.push(ArtifactRef {
                                logical_path: want.path.clone(),
                                stored_path: path.display().to_string(),
                                sha256: hex(&sha256(&actual)),
                                bytes: Some(actual.len()),
                            });
                        }
                    }
                }
                let extras: Vec<String> = std::fs::read_dir(&actual_dir)
                    .map(|rd| {
                        rd.filter_map(|e| e.ok())
                            .filter(|e| e.path().is_file())
                            .filter(|e| {
                                let name = e.file_name().to_string_lossy().to_string();
                                !expected_paths.contains(&name) && name != "source.at"
                            })
                            .map(|e| e.file_name().to_string_lossy().to_string())
                            .collect()
                    })
                    .unwrap_or_default();
                if !extras.is_empty() {
                    mismatches.push(Mismatch {
                        observation: "artifacts".into(),
                        kind: "extra",
                        expected: format!("exactly {:?}", expected_paths),
                        actual: format!("extra files: {:?}", extras),
                        first_diff: None,
                    });
                }
            }
            let status = if mismatches.is_empty() {
                Status::Pass
            } else {
                Status::Fail
            };
            Ok(Verdict {
                status,
                mismatches,
                detail: Some(format!(
                    "{recipe} verified via worker {worker_sha} (manifest {manifest_sha})"
                )),
                artifacts: refs,
                native: None,
            })
        }
        other => Err(format!(
            "{recipe} currently expects success completions (got {:?})",
            other
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use auto_test_case::admission::Limits;
    use auto_test_case::atom::parse_contract;
    use auto_test_case::contract::decode_case;
    use auto_test_case::document::CaseDocument;
    use auto_test_case::ir::{resolve_case, resolve_oracle};

    #[test]
    fn reject_emit_only_impersonation_for_run_recipe() {
        let fake_emit_resp = serde_json::json!({
            "status": "ok",
            "protocol": A2X_WORKER_PROTOCOL,
            "recipe": A2PY_STDLIB_RUN_RECIPE,
            "target": "python",
            "producer": PY_PRODUCER,
            "mode": "daily-emit-py",
            "execution_claimed": false,
            "spawned_python": 0,
            "manifest_sha256": "abc"
        });
        let err = validate_worker_response(&fake_emit_resp, A2PY_STDLIB_RUN_RECIPE, "abc")
            .expect_err("emit-only must NEVER pass execution recipe");
        assert!(err.contains("emit-only cannot pass"), "got {err}");

        let fake_no_identity = serde_json::json!({
            "status": "ok",
            "protocol": A2X_WORKER_PROTOCOL,
            "recipe": A2PY_STDLIB_RUN_RECIPE,
            "target": "python",
            "producer": PY_PRODUCER,
            "mode": "run-py",
            "execution_claimed": true,
            "spawned_python": 1,
            "manifest_sha256": "abc"
        });
        let err = validate_worker_response(&fake_no_identity, A2PY_STDLIB_RUN_RECIPE, "abc")
            .expect_err("missing python_identity must fail");
        assert!(err.contains("python_identity"), "got {err}");
    }

    #[test]
    fn reject_wrong_producer_or_tampered_manifest() {
        let wrong_prod = serde_json::json!({
            "status": "ok",
            "protocol": A2X_WORKER_PROTOCOL,
            "recipe": A2PY_HIR_RECIPE,
            "target": "python",
            "producer": "legacy-python-trans",
            "mode": "daily-emit-py",
            "execution_claimed": false,
            "spawned_python": 0,
            "spawned_rustc": 0,
            "spawned_cl": 0,
            "manifest_sha256": "abc"
        });
        assert!(validate_worker_response(&wrong_prod, A2PY_HIR_RECIPE, "abc").is_err());

        let valid_emit = serde_json::json!({
            "status": "ok",
            "protocol": A2X_WORKER_PROTOCOL,
            "recipe": A2PY_HIR_RECIPE,
            "target": "python",
            "producer": PY_PRODUCER,
            "mode": "daily-emit-py",
            "execution_claimed": false,
            "spawned_python": 0,
            "spawned_rustc": 0,
            "spawned_cl": 0,
            "manifest_sha256": "abc"
        });
        assert!(validate_worker_response(&valid_emit, A2PY_HIR_RECIPE, "def").is_err());
        assert!(validate_worker_response(&valid_emit, A2PY_HIR_RECIPE, "abc").is_ok());
    }

    #[test]
    fn contract_admits_a2py_recipes_and_rejects_missing_binding() {
        let valid_case = r#"
case {
    schema: "autolang.case/2"
    id: "py1"
    tags: ["a2py"]
    links: ["PLAN-1042"]
    source: { text: "fn main() int { return 7 }", syntax: "auto", path: "main.at" }
    entry: { mode: "program" }
    checks: [
        {
            id: "py1"
            adapter: "probe"
            action: "run"
            options: {
                recipe: "ac-a2py/hir-stdlib-run-v1"
                a2x_hir: { profile: "core", entry: "main", caps: [] }
                steps: [{ op: "run" }]
            }
            expect: { completion: "success", exit: 0, stdout: { text: "value:7\n", compare: "lf" } }
        }
    ]
}
"#;
        let doc = parse_contract(valid_case).expect("parse case contract");
        decode_case(&doc, &Limits::defaults()).expect("expected valid ac-a2py case");

        let bad_profile = r#"
case {
    schema: "autolang.case/2"
    id: "py3"
    tags: ["a2py"]
    links: ["PLAN-1042"]
    source: { text: "fn main() int { return 7 }", syntax: "auto", path: "main.at" }
    entry: { mode: "program" }
    checks: [
        {
            id: "py3"
            adapter: "probe"
            action: "run"
            options: {
                recipe: "ac-a2py/hir-collections-emit-v1"
                a2x_hir: { profile: "scalar", entry: "main", caps: [] }
                steps: [{ op: "run" }]
            }
            expect: { completion: "success" }
        }
    ]
}
"#;
        let bad_v = parse_contract(bad_profile).expect("parse case contract");
        assert!(decode_case(&bad_v, &Limits::defaults()).is_err());

        let bad_no_binding = r#"
case {
    schema: "autolang.case/2"
    id: "py2"
    tags: ["a2py"]
    links: ["PLAN-1042"]
    source: { text: "fn main() int { return 7 }", syntax: "auto", path: "main.at" }
    entry: { mode: "program" }
    checks: [
        {
            id: "py2"
            adapter: "probe"
            action: "run"
            options: {
                recipe: "ac-a2py/hir-stdlib-run-v1"
                steps: [{ op: "run" }]
            }
            expect: { completion: "success", exit: 0 }
        }
    ]
}
"#;
        let bad_v2 = parse_contract(bad_no_binding).expect("parse case contract");
        assert!(decode_case(&bad_v2, &Limits::defaults()).is_err());
    }

    #[test]
    fn worker_run_py_executes_real_python_and_rejects_negative_controls() {
        let worker = worker_binary().expect("prebuilt a2x-worker must exist for ace-adapters");
        let tmp = std::env::temp_dir().join(format!("plan1042_adapter_e2e_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();
        let src_file = tmp.join("source.at");
        std::fs::write(&src_file, b"fn main() int { return 7 }\n").unwrap();

        let out_dir = tmp.join("run_out");
        let req = serde_json::json!({
            "op": "run-py",
            "recipe": A2PY_STDLIB_RUN_RECIPE,
            "source_path": src_file,
            "profile": "core",
            "entry": "main",
            "caps": [],
            "out_dir": out_dir,
            "nonce": "n1",
        });
        let out = Command::new(&worker)
            .arg(req.to_string())
            .output()
            .expect("spawn worker");
        assert!(
            out.status.success(),
            "run-py failed: {}",
            String::from_utf8_lossy(&out.stdout)
        );
        let resp: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
        let disk_manifest = std::fs::read(out_dir.join("manifest.json")).unwrap();
        let disk_sha = hex(&sha256(&disk_manifest));
        validate_worker_response(&resp, A2PY_STDLIB_RUN_RECIPE, &disk_sha).unwrap();
        let stdout_bytes = decode_hex(resp["stdout_hex"].as_str().unwrap()).unwrap();
        assert_eq!(stdout_bytes, b"value:7\n");

        // Negative controls: legacy trans, use_vm, emit_only, expected on worker.
        for extra_key in [
            ("use_legacy_python_trans", serde_json::json!(true)),
            ("use_vm", serde_json::json!(true)),
            ("emit_only", serde_json::json!(true)),
            ("expected", serde_json::json!("value:7\n")),
        ] {
            let mut neg = req.clone();
            neg[extra_key.0] = extra_key.1;
            let neg_out = Command::new(&worker)
                .arg(neg.to_string())
                .output()
                .expect("spawn neg worker");
            assert!(
                !neg_out.status.success(),
                "worker must reject `{}`",
                extra_key.0
            );
        }

        let case_md = r#"
```test-case
case {
    schema: "autolang.case/2"
    id: "py_run_e2e"
    tags: ["a2py"]
    links: ["PLAN-1042"]
    source: { text: "fn main() int { return 7 }\n", syntax: "auto", path: "main.at" }
    entry: { mode: "program" }
    checks: [
        {
            id: "c1"
            adapter: "probe"
            action: "run"
            options: {
                recipe: "ac-a2py/hir-stdlib-run-v1"
                a2x_hir: { profile: "core", entry: "main", caps: [] }
                steps: [{ op: "run" }]
            }
            expect: {
                completion: "success"
                exit: 0
                stdout: { text: "value:7\n", compare: "lf" }
            }
        }
    ]
}
```
"#;
        let limits = Limits::defaults();
        let doc = CaseDocument::parse_with(case_md.as_bytes(), &limits).unwrap();
        let tc = doc.fences().first().unwrap();
        let cval = parse_contract(std::str::from_utf8(doc.payload_bytes(tc)).unwrap()).unwrap();
        let spec = decode_case(&cval, &limits).unwrap();
        let check = spec.checks[0].clone();
        let oracle = resolve_oracle(&spec.id, &check, &tmp, &limits).unwrap();
        let resolved = resolve_case(spec, &doc, &tmp, &limits).unwrap();
        let v = run_check(&resolved, &check, &oracle, &tmp, &tmp.join("art"), &tmp)
            .expect("run_check ok");
        assert_eq!(v.status, Status::Pass, "verdict mismatch: {:?}", v.detail);

        let _ = std::fs::remove_dir_all(&tmp);
    }
}

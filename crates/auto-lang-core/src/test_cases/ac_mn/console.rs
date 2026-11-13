//! PLAN-947: the `ac/mn/console-run-v1` TestCase judge — formal AutoDown
//! execution of the console/1 native lane through the unified probe/run
//! seam. The parent builds the request, spawns the mn-worker under the
//! shared contained channel, admits the identity echoes, and judges the
//! observed outcome PARENT-SIDE against the declared expectations. The
//! worker never sees the expected outcomes.
//!
//! Judgement faces (the console taxonomy):
//! - `completion: "success"` + `exit: <i32|70-trap>` + byte-exact streams;
//! - `completion: {stage, code}` — a diagnosed frontend/admission/resource
//!   rejection (console budget = `resource.console-output`; an unauthorized
//!   run = `host.unauthorized` with dispatch count zero and empty streams).

use super::{call_worker_observed, worker_binary, Verdict};
use std::path::Path;
use std::time::Instant;

pub const RECIPE: &str = "ac/mn/console-run-v1";

pub fn run_check_console(
    case: &auto_test_case::ir::ResolvedCase,
    check: &auto_test_case::contract::CheckSpec,
    oracle: &auto_test_case::ir::ResolvedOracle,
    artifact_dir: &Path,
) -> Result<Verdict, String> {
    use auto_test_case::compare::Mismatch;
    let started = Instant::now();
    let mn = check
        .options
        .probe_mn
        .as_ref()
        .ok_or("console check without an mn binding (contract violation)")?;
    if !matches!(
        check.options.probe_steps.as_deref(),
        Some([auto_test_case::contract::ProbeStep::Run])
    ) {
        return Err(format!(
            "{RECIPE} consumes exactly one run step (contract violation)"
        ));
    }
    if mn.entry != "d_main" || mn.session.is_some() || mn.diagnostics.is_some() {
        return Err(format!(
            "{RECIPE} requires the d_main entry with no session/profile (contract violation)"
        ));
    }
    auto_test_case::contract::validate_console_oracle(
        oracle.stdout.as_ref(),
        oracle.stderr.as_ref(),
        oracle.exit,
        None,
        &oracle.completion,
    )?;
    let source = case
        .files
        .first()
        .map(|f| f.bytes.clone())
        .ok_or("console case without materialized source bytes")?;
    let _ = worker_binary()?;
    let nonce = auto_test_case::sha256::hex(&auto_test_case::sha256::sha256(
        format!("{}|mn-console|{}", std::process::id(), source.len()).as_bytes(),
    ));
    let store_root = artifact_dir.join("mn-console-store");
    std::fs::create_dir_all(&store_root).map_err(|e| format!("store root: {e}"))?;
    let mut request = serde_json::json!({
        "recipe": RECIPE,
        "nonce": nonce,
        "source_b64": super::b64(&source),
        "entry": "d_main",
        "caps": mn.caps,
        "store_root": store_root.to_string_lossy(),
        "steps": [{"op": "run"}],
    });
    let request_text = request.to_string();
    let request_hash =
        auto_test_case::sha256::hex(&auto_test_case::sha256::sha256(request_text.as_bytes()));
    request["request_hash"] = serde_json::json!(request_hash);

    let response = call_worker_observed(&request.to_string(), Some(artifact_dir))?;
    let resp: serde_json::Value =
        serde_json::from_str(&response).map_err(|e| format!("console worker JSON: {e}"))?;
    let mut mismatches: Vec<Mismatch> = Vec::new();
    let echo_check = |mismatches: &mut Vec<Mismatch>, field: &str, want: &str| {
        let got = resp.get(field).and_then(|v| v.as_str()).unwrap_or("");
        if got != want {
            mismatches.push(Mismatch {
                observation: field.into(),
                kind: "protocol".into(),
                expected: want.into(),
                actual: got.to_string(),
                first_diff: None,
            });
        }
    };
    echo_check(&mut mismatches, "schema", "mnworker/1");
    echo_check(&mut mismatches, "nonce", nonce.as_str());
    echo_check(&mut mismatches, "request_hash", request_hash.as_str());
    let ok = resp.get("ok").cloned().unwrap_or(serde_json::Value::Null);
    if ok.get("recipe").and_then(|v| v.as_str()) != Some(RECIPE) {
        mismatches.push(Mismatch {
            observation: "recipe".into(),
            kind: "protocol".into(),
            expected: RECIPE.into(),
            actual: format!("{:?}", ok.get("recipe")),
            first_diff: None,
        });
    }
    if ok.get("worker_version").and_then(|v| v.as_str()) != Some("mnworker/1.0") {
        mismatches.push(Mismatch {
            observation: "worker_version".into(),
            kind: "protocol".into(),
            expected: "mnworker/1.0".into(),
            actual: format!("{:?}", ok.get("worker_version")),
            first_diff: None,
        });
    }
    let step = ok
        .get("steps")
        .and_then(|s| s.as_array())
        .and_then(|a| a.first())
        .cloned()
        .unwrap_or(serde_json::Value::Null);
    let request_wall_ms = started.elapsed().as_secs_f64() * 1000.0;
    std::fs::write(
        artifact_dir.join("probe-receipt.json"),
        serde_json::to_string_pretty(&serde_json::json!({
            "recipe": RECIPE,
            "protocol": "mnworker/1",
            "nonce": nonce,
            "request_hash": request_hash,
            "step": step,
            "request_wall_ms": request_wall_ms,
        }))
        .unwrap_or_default(),
    )
    .map_err(|e| format!("write receipt: {e}"))?;

    // ---- judgement (parent-side, independent oracle) --------------------
    use auto_test_case::contract::Completion;
    let outcome = step.get("outcome").and_then(|v| v.as_str()).unwrap_or("");
    match &oracle.completion {
        Completion::Success => {
            if outcome != "success" {
                mismatches.push(Mismatch {
                    observation: "completion".into(),
                    kind: "outcome".into(),
                    expected: "success".into(),
                    actual: format!(
                        "{outcome} {:?}",
                        step.get("diag").and_then(|d| d.get("code"))
                    ),
                    first_diff: None,
                });
            } else {
                if let Some(want) = oracle.exit {
                    let got = step.get("exit_code").and_then(|v| v.as_i64());
                    if got != Some(want as i64) {
                        mismatches.push(Mismatch {
                            observation: "exit".into(),
                            kind: "exit".into(),
                            expected: want.to_string(),
                            actual: format!("{got:?}"),
                            first_diff: None,
                        });
                    }
                }
                compare_hex_stream(&step, "stdout", oracle.stdout.as_ref(), &mut mismatches)?;
                compare_hex_stream(&step, "stderr", oracle.stderr.as_ref(), &mut mismatches)?;
            }
        }
        Completion::Error { stage, code, .. } => {
            let diag = step.get("diag").cloned().unwrap_or(serde_json::Value::Null);
            let got_code = diag.get("code").and_then(|v| v.as_str()).unwrap_or("");
            // The same stage normalization the AV console judge applies
            // (front-door/admission rejections are parse-stage).
            let raw_stage = diag.get("stage").and_then(|v| v.as_str()).unwrap_or("");
            let got_stage = match raw_stage {
                "admission" | "frontend" | "parse" | "text" => "parse",
                "resolve" | "bind" => "resolve",
                "typecheck" => "typecheck",
                other => other,
            };
            if outcome != "diag" && outcome != "resource" {
                mismatches.push(Mismatch {
                    observation: "completion".into(),
                    kind: "outcome".into(),
                    expected: format!("error {}/{stage:?}", stage.as_str()),
                    actual: format!("{outcome} {got_stage}/{got_code}"),
                    first_diff: None,
                });
            }
            if got_stage != stage.as_str() {
                mismatches.push(Mismatch {
                    observation: "completion.stage".into(),
                    kind: "stage".into(),
                    expected: stage.as_str().to_string(),
                    actual: got_stage.to_string(),
                    first_diff: None,
                });
            }
            if let Some(want) = code {
                if got_code != want {
                    mismatches.push(Mismatch {
                        observation: "completion.code".into(),
                        kind: "code".into(),
                        expected: want.clone(),
                        actual: got_code.to_string(),
                        first_diff: None,
                    });
                }
            }
        }
    }
    let exit_code = step
        .get("exit_code")
        .and_then(|v| v.as_i64())
        .map(|v| v as i32);
    let worker_sha = super::bound_worker_sha(RECIPE)?;
    let native = auto_test_case::report::NativeRecord {
        exit_code,
        stage_spans: vec![("request_wall_ms".to_string(), request_wall_ms as u64)],
        cache_mode: None,
        cache_hits: vec![],
        counters: vec![],
        gaps: vec![],
        toolchain: vec![
            ("mn-worker-exe-sha256".to_string(), worker_sha),
            ("recipe".to_string(), RECIPE.to_string()),
            (
                "mir-lowering".to_string(),
                super::super::ac_cte::EXPECTED_MIR_LOWERING_VERSION.to_string(),
            ),
        ],
    };
    let mut verdict = if mismatches.is_empty() {
        Verdict::pass()
    } else {
        Verdict::fail(mismatches)
    };
    verdict.native = Some(native);
    Ok(verdict)
}

fn compare_hex_stream(
    step: &serde_json::Value,
    field: &str,
    want: Option<&auto_test_case::contract::StreamExpect>,
    mismatches: &mut Vec<auto_test_case::compare::Mismatch>,
) -> Result<(), String> {
    use base64::Engine as _;
    let hex_text = step
        .get(format!("{field}_hex"))
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let bytes = super::hex_stream(hex_text)?;
    let run = serde_json::json!({format!("{field}_b64"): base64::engine::general_purpose::STANDARD.encode(&bytes)});
    crate::test_cases::ac::compare_stream_field(&run, field, want, mismatches)
}

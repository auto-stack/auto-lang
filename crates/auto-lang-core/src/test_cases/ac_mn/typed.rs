//! PLAN-891 `ac/mir-native-typed-v1`: the formal native typed-entry check.
//!
//! The parent materializes the case's file-ized fixture (mn-typed-fixture/1),
//! sends its bytes + the declared arguments to the mn-worker, and judges the
//! machine frame INDEPENDENTLY: the expected (kind, canonical bits) live in
//! the CASE's expect.typed — they never enter the request. The worker only
//! ever reports what the real PE returned.

use super::{call_worker_observed, worker_binary, TYPED_RECIPE};
use crate::test_cases::avm::Verdict;
use auto_test_case::compare::Mismatch;
use auto_test_case::contract::{CheckSpec, Completion, Stage};
use auto_test_case::ir::{ResolvedCase, ResolvedOracle};
use auto_test_case::report::{NativeRecord, Status};
use auto_test_case::sha256::{hex, sha256};
use base64::Engine as _;
use std::path::Path;
use std::time::Instant;

/// The frozen typed ABI tag the worker must echo (mirror of
/// `ace_native::config::TYPED_ABI`).
pub const EXPECTED_TYPED_ABI: &str = "msvc-x64/priv-typed-data-v1";

pub fn run_check_typed(
    case: &ResolvedCase,
    check: &CheckSpec,
    oracle: &ResolvedOracle,
    artifact_dir: &Path,
) -> Result<Verdict, String> {
    let started = Instant::now();
    let mn = check
        .options
        .probe_mn
        .as_ref()
        .ok_or("typed check without an mn binding (contract violation)")?;
    let typed = mn
        .typed
        .as_ref()
        .ok_or("typed check without a typed binding (contract violation)")?;
    let steps = check
        .options
        .probe_steps
        .as_ref()
        .ok_or("typed check without steps (contract violation)")?;
    if !matches!(steps.as_slice(), [auto_test_case::contract::ProbeStep::Run]) {
        return Err(format!(
            "{TYPED_RECIPE} consumes exactly one run step (contract violation)"
        ));
    }
    // The fixture IS the case's declared source file: same bytes ride the
    // request and (independently) any AV-side comparison of the same file.
    let source_file = match &case.spec.source.payload {
        auto_test_case::contract::PayloadSpec::File(f) => f.clone(),
        other => {
            return Err(format!(
                "typed case source must be a fixture file, got {other:?}"
            ))
        }
    };
    if source_file != typed.fixture {
        return Err(format!(
            "typed fixture `{}` does not match the case source file `{source_file}`",
            typed.fixture
        ));
    }
    let fixture_bytes = case
        .files
        .first()
        .map(|f| f.bytes.clone())
        .ok_or("typed case without materialized fixture bytes")?;

    let _ = worker_binary()?; // build/locate the mn-worker (same binary)
    let nonce = hex(&sha256(
        format!("{}|typed|{}", std::process::id(), fixture_bytes.len()).as_bytes(),
    ));
    let store_root = artifact_dir.join("mn-typed-store");
    std::fs::create_dir_all(&store_root).map_err(|e| format!("store root: {e}"))?;
    let workdir = artifact_dir.join("mn-typed-work");
    std::fs::create_dir_all(&workdir).map_err(|e| format!("workdir: {e}"))?;
    let mut request = serde_json::json!({
        "recipe": TYPED_RECIPE,
        "nonce": nonce,
        "typed_fixture_b64": base64::engine::general_purpose::STANDARD.encode(&fixture_bytes),
        "entry": mn.entry,
        "args": typed
            .args
            .iter()
            .map(|(k, b)| serde_json::json!([k, b]))
            .collect::<Vec<_>>(),
        "store_root": store_root.to_string_lossy(),
        "workdir": workdir.to_string_lossy(),
    });
    let request_text = request.to_string();
    let request_hash = hex(&sha256(request_text.as_bytes()));
    request["request_hash"] = serde_json::json!(request_hash);

    let response = call_worker_observed(&request.to_string(), Some(artifact_dir))?;
    let resp: serde_json::Value = serde_json::from_str(&response)
        .map_err(|e| format!("typed worker response is not JSON: {e}"))?;

    // ---- admission: identity echoes come back untouched ----------------
    let mut mismatches: Vec<Mismatch> = Vec::new();
    if resp.get("schema").and_then(|v| v.as_str()) != Some("mnworker/1") {
        mismatches.push(Mismatch {
            observation: "schema".into(),
            kind: "protocol".into(),
            expected: "mnworker/1".into(),
            actual: resp
                .get("schema")
                .and_then(|v| v.as_str())
                .unwrap_or("<missing>")
                .to_string(),
            first_diff: None,
        });
    }
    if resp.get("nonce").and_then(|v| v.as_str()) != Some(nonce.as_str()) {
        mismatches.push(Mismatch {
            observation: "nonce".into(),
            kind: "protocol".into(),
            expected: nonce.clone(),
            actual: format!("{:?}", resp.get("nonce")),
            first_diff: None,
        });
    }
    if resp.get("request_hash").and_then(|v| v.as_str()) != Some(request_hash.as_str()) {
        mismatches.push(Mismatch {
            observation: "request_hash".into(),
            kind: "protocol".into(),
            expected: request_hash.clone(),
            actual: format!("{:?}", resp.get("request_hash")),
            first_diff: None,
        });
    }
    let ok = resp.get("ok").cloned().unwrap_or(serde_json::Value::Null);
    if ok.get("recipe").and_then(|v| v.as_str()) != Some(TYPED_RECIPE) {
        mismatches.push(Mismatch {
            observation: "recipe".into(),
            kind: "protocol".into(),
            expected: TYPED_RECIPE.into(),
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
    if ok.get("mn_abi").and_then(|v| v.as_str()) != Some(EXPECTED_TYPED_ABI) {
        mismatches.push(Mismatch {
            observation: "mn_abi".into(),
            kind: "protocol".into(),
            expected: EXPECTED_TYPED_ABI.into(),
            actual: format!("{:?}", ok.get("mn_abi")),
            first_diff: None,
        });
    }
    let run_invocations = ok
        .get("counters")
        .and_then(|c| c.get("run_invocations"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    if run_invocations < 1 {
        mismatches.push(Mismatch {
            observation: "counters".into(),
            kind: "always-execute".into(),
            expected: ">=1 run invocation".into(),
            actual: format!("{run_invocations}"),
            first_diff: None,
        });
    }

    // ---- judgement (parent-side, independent oracle) --------------------
    let typed_frame = ok.get("typed").cloned().unwrap_or(serde_json::Value::Null);
    let status = typed_frame
        .get("status")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let observed: String;
    match status {
        "value" => {
            let kind = typed_frame
                .get("kind")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let bits = typed_frame
                .get("bits")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            match &oracle.completion {
                Completion::Success => {
                    if let Some(want) = &check.expect.typed {
                        if let Some(violation) =
                            crate::test_cases::ac_av::typed_expect_shape_violation(
                                &want.kind, &want.bits,
                            )
                        {
                            return Err(format!("typed expect is not canonical: {violation}"));
                        }
                        if kind != want.kind {
                            mismatches.push(Mismatch {
                                observation: "typed".into(),
                                kind: "typed-tag".into(),
                                expected: want.kind.clone(),
                                actual: kind.to_string(),
                                first_diff: None,
                            });
                        }
                        if bits != want.bits {
                            mismatches.push(Mismatch {
                                observation: "typed".into(),
                                kind: "typed-bits".into(),
                                expected: want.bits.clone(),
                                actual: bits.to_string(),
                                first_diff: None,
                            });
                        }
                    } else {
                        mismatches.push(Mismatch {
                            observation: "typed".into(),
                            kind: "missing-axis".into(),
                            expected: "a typed expect (kind, bits) for a value frame".into(),
                            actual: "none declared".into(),
                            first_diff: None,
                        });
                    }
                }
                Completion::Error { stage, .. } => {
                    mismatches.push(Mismatch {
                        observation: "completion".into(),
                        kind: "outcome".into(),
                        expected: format!("{stage:?} rejection"),
                        actual: format!("typed value {kind}:{bits}"),
                        first_diff: None,
                    });
                }
            }
            observed = format!("value {kind}:{bits}");
        }
        "trap" | "resource" | "error" => {
            let reason = typed_frame
                .get("reason")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let code = format!("mn.typed.{status}");
            // A named non-success frame maps onto the probe adapter's run
            // stage; the reason rides the receipt.
            match &oracle.completion {
                Completion::Error {
                    stage: want_stage,
                    code: want_code,
                    ..
                } => {
                    if want_stage != &Stage::Run {
                        mismatches.push(Mismatch {
                            observation: "stage".into(),
                            kind: "stage".into(),
                            expected: format!("{want_stage:?}"),
                            actual: format!("{:?}", Stage::Run),
                            first_diff: None,
                        });
                    }
                    if let Some(want) = want_code {
                        if want != &code {
                            mismatches.push(Mismatch {
                                observation: "code".into(),
                                kind: "code".into(),
                                expected: want.clone(),
                                actual: code.clone(),
                                first_diff: None,
                            });
                        }
                    }
                }
                other => {
                    mismatches.push(Mismatch {
                        observation: "completion".into(),
                        kind: "outcome".into(),
                        expected: format!("{other:?}"),
                        actual: format!("{status} frame (reason {reason})"),
                        first_diff: None,
                    });
                }
            }
            observed = format!("{status} frame (reason {reason})");
        }
        other => {
            mismatches.push(Mismatch {
                observation: "typed".into(),
                kind: "typed-outcome".into(),
                expected: "a value/trap/resource/error frame".into(),
                actual: format!("{other:?}"),
                first_diff: None,
            });
            observed = format!("{other:?}");
        }
    }
    // A diagnosed worker rejection (fixture/verify/admit) maps through the
    // same stage vocabulary as the scalar recipe.
    if let Some(diag) = resp.get("diagnostic") {
        let code = diag.get("code").and_then(|v| v.as_str()).unwrap_or("");
        let message = diag.get("message").and_then(|v| v.as_str()).unwrap_or("");
        let stage = if code.starts_with("mn.unsupported.fixture")
            || code.starts_with("mn.unsupported.typed-verify")
        {
            Stage::Parse
        } else {
            Stage::Typecheck
        };
        match &oracle.completion {
            Completion::Error {
                stage: want_stage,
                code: want_code,
                ..
            } => {
                if want_stage != &stage {
                    mismatches.push(Mismatch {
                        observation: "stage".into(),
                        kind: "stage".into(),
                        expected: format!("{want_stage:?}"),
                        actual: format!("{stage:?}"),
                        first_diff: None,
                    });
                }
                if let Some(want) = want_code {
                    if want != code {
                        mismatches.push(Mismatch {
                            observation: "code".into(),
                            kind: "code".into(),
                            expected: want.clone(),
                            actual: code.to_string(),
                            first_diff: None,
                        });
                    }
                }
            }
            other => {
                mismatches.push(Mismatch {
                    observation: "completion".into(),
                    kind: "outcome".into(),
                    expected: format!("{other:?}"),
                    actual: format!("diagnosed {code}: {message}"),
                    first_diff: None,
                });
            }
        }
    }

    // ---- receipt --------------------------------------------------------
    let receipt = serde_json::json!({
        "recipe": TYPED_RECIPE,
        "nonce": nonce,
        "request_hash": request_hash,
        "fixture_sha256": hex(&sha256(&fixture_bytes)),
        "args": typed.args,
        "entry": mn.entry,
        "worker_response": resp,
        "request_wall_ms": started.elapsed().as_secs_f64() * 1000.0,
    });
    let receipt_path = artifact_dir.join("mn-typed-receipt.json");
    std::fs::write(
        &receipt_path,
        serde_json::to_vec_pretty(&receipt).unwrap_or_default(),
    )
    .map_err(|e| format!("write typed receipt: {e}"))?;

    let native = NativeRecord {
        exit_code: ok
            .get("exit_code")
            .and_then(|v| v.as_i64())
            .map(|v| v as i32),
        stage_spans: vec![(
            "request_wall_ms".to_string(),
            (started.elapsed().as_secs_f64() * 1000.0) as u64,
        )],
        cache_mode: None,
        cache_hits: vec![],
        counters: vec![],
        toolchain: vec![
            ("recipe".to_string(), TYPED_RECIPE.to_string()),
            (
                "mir-lowering".to_string(),
                super::EXPECTED_MIR_LOWERING_VERSION.to_string(),
            ),
            (
                "mn-worker-exe-sha256".to_string(),
                super::bound_worker_sha(TYPED_RECIPE)?,
            ),
            ("mn-typed-abi".to_string(), EXPECTED_TYPED_ABI.to_string()),
        ],
        gaps: vec![],
    };
    let status = if mismatches.is_empty() {
        Status::Pass
    } else {
        Status::Fail
    };
    Ok(Verdict {
        status,
        mismatches,
        detail: Some(format!(
            "{TYPED_RECIPE}: typed run judged (outcome {observed}); receipt mn-typed-receipt.json"
        )),
        artifacts: vec![],
        native: Some(native),
    })
}

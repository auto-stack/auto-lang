//! PLAN-941 F06: actual data worker diagnostics, independently named oracles.
use auto_tool_execution::{ExecutionSpec, ExitKind, StdioPolicy, TreeScope};
use base64::Engine;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

static NEXT: AtomicU64 = AtomicU64::new(0);

fn scratch() -> PathBuf {
    let base = std::env::var_os("AUTO_PLAN_ARTIFACTS")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    let path = base.join(format!(
        "data-diag-941-{}-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&path).unwrap();
    path
}

fn source(name: &str) -> String {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../../crates/auto-lang/test/cases/ace-m1/src");
    std::fs::read_to_string(root.join(format!("{name}.at"))).unwrap()
}

fn worker(src: &str, recipe: &str, store: Option<&Path>) -> auto_tool_execution::ExecutionResult {
    worker_caps(src, recipe, store, &[])
}

fn worker_caps(
    src: &str,
    recipe: &str,
    store: Option<&Path>,
    caps: &[&str],
) -> auto_tool_execution::ExecutionResult {
    let dir = scratch();
    let mut request = json!({
        "schema": "avworker/1", "recipe": recipe, "nonce": "f".repeat(64),
        "source_b64": base64::engine::general_purpose::STANDARD.encode(src.as_bytes()),
        "entry": "d_main", "caps": caps, "store_root": store.unwrap_or(&dir.join("store")),
        "steps": [{"op": "run"}]
    });
    request["request_hash"] = Value::String(format!(
        "{:x}",
        Sha256::digest(serde_json::to_vec(&request).unwrap())
    ));
    let mut input = serde_json::to_vec(&request).unwrap();
    input.push(b'\n');
    std::fs::write(dir.join("request.jsonl"), &input).unwrap();
    let mut spec = ExecutionSpec::new([env!("CARGO_BIN_EXE_av-worker"), "case-serve"]);
    spec.stdin = StdioPolicy::Piped;
    spec.stdin_data = Some(input);
    spec.deadline = Some(Duration::from_secs(30));
    spec.tree_scope = TreeScope::ProcessTree;
    let result = auto_tool_execution::run(&spec).unwrap();
    let stdout = result.stdout.as_deref().unwrap();
    let stderr = result.stderr.as_deref().unwrap();
    std::fs::write(dir.join("stdout"), stdout).unwrap();
    std::fs::write(dir.join("stderr"), stderr).unwrap();
    assert!(stdout.len() <= 64 * 1024 && stderr.len() <= 64 * 1024);
    if result.kind == ExitKind::Success {
        let response: Value = serde_json::from_slice(stdout).unwrap();
        assert_eq!(response["schema"], "avworker/1");
        assert_eq!(response["nonce"], request["nonce"]);
        assert_eq!(response["request_hash"], request["request_hash"]);
    }
    result
}

fn negative(name: &str, expected_code: &str) {
    let result = worker(&source(name), "ac-av/data-run-v1", None);
    assert_eq!(
        result.kind,
        ExitKind::Success,
        "{:?}",
        result.stderr_lossy()
    );
    assert_eq!(result.stderr.as_deref().unwrap(), b"");
    let response: Value = serde_json::from_slice(result.stdout.as_deref().unwrap()).unwrap();
    let step = &response["ok"]["steps"][0];
    assert_eq!(step["outcome"], "diag");
    assert_eq!(step["diag"]["stage"], "frontend");
    assert_eq!(step["diag"]["code"], expected_code, "{name}: {response}");
    assert!(step["value"].is_null() && step["trap"].is_null());
}

#[test]
fn original_m09_worker_code() {
    negative("M09", "source.not-callable");
}
#[test]
fn original_m10_worker_code() {
    negative("M10", "verify.immutable-write");
}
#[test]
fn original_m11_worker_code() {
    negative("M11", "verify.mut-conflict");
}
#[test]
fn original_r09_worker_code() {
    negative("R09", "source.type-mismatch");
}

#[test]
fn typed_frontend_retains_ordered_primary_stage_code_message_and_span() {
    use ace_av::data_driver::{request_data, request_data_staged, DataRequestError};
    for (name, stage, code) in [
        ("M09", "bind", "source.not-callable"),
        ("M10", "verify", "verify.immutable-write"),
        ("M11", "verify", "verify.mut-conflict"),
        ("R09", "verify", "source.type-mismatch"),
    ] {
        let src = source(name);
        let original = ace_core::source::compile_record_source(&src, "data.at", &[]).unwrap_err();
        let error = request_data_staged(None, "941", &src, "d_main", &[]).unwrap_err();
        let DataRequestError::Frontend(retained) = &error else {
            panic!("{error:?}");
        };
        assert_eq!(retained.len(), original.len());
        assert_eq!(retained[0].stage.as_str(), stage);
        assert_eq!(retained[0].code, code);
        for (actual, want) in retained.iter().zip(original.iter()) {
            assert_eq!(actual.stage, want.stage);
            assert_eq!(actual.code, want.code);
            assert_eq!(actual.message, want.message);
            assert_eq!(actual.span, want.span);
        }
        let legacy = request_data(None, "941", &src, "d_main", &[]).unwrap_err();
        assert_eq!(
            legacy,
            format!("frontend: {} {}", original[0].code, original[0].message)
        );
        assert_eq!(legacy, error.to_string());
    }
}

#[test]
fn classification_uses_types_even_when_messages_look_like_other_stages() {
    use ace_av::data_driver::{request_data_staged, DataRequestError};
    let mut error = request_data_staged(None, "941", &source("M09"), "d_main", &[]).unwrap_err();
    let DataRequestError::Frontend(ds) = &mut error else {
        panic!("frontend required");
    };
    ds[0].message = "persist: mir verify: encode: misleading human text".into();
    let diag = error.diagnostic().unwrap();
    assert_eq!(
        (diag.stage.as_str(), diag.code.as_str()),
        ("frontend", "source.not-callable")
    );
    assert_eq!(
        diag.message,
        "persist: mir verify: encode: misleading human text"
    );
    assert!(
        DataRequestError::Infrastructure("frontend: source.not-callable fake".into())
            .diagnostic()
            .is_none()
    );
    assert!(DataRequestError::Frontend(vec![]).diagnostic().is_none());
    let mut no_code =
        ace_core::source::compile_record_source(&source("M09"), "data.at", &[]).unwrap_err();
    no_code[0].code.clear();
    assert!(DataRequestError::Frontend(no_code.clone())
        .diagnostic()
        .is_none());
    no_code[0].code = "source.infra".into();
    assert!(DataRequestError::Frontend(no_code).diagnostic().is_none());
    assert!(DataRequestError::FrontDoor(ace_av::driver::StageDiag {
        stage: "admission".into(),
        code: "source.infra".into(),
        message: "frontend: fake".into()
    })
    .diagnostic()
    .is_none());
}

#[test]
fn actual_front_door_resource_and_missing_entry_keep_their_typed_causes() {
    use ace_av::data_driver::{request_data_staged, DataRequestError};
    use ace_av_core::contract::{ResourceKind, MAX_INPUT_BYTES};
    let error = request_data_staged(None, "941", &" ".repeat(MAX_INPUT_BYTES + 1), "d_main", &[])
        .unwrap_err();
    let DataRequestError::FrontDoor(raw) = &error else {
        panic!("{error:?}")
    };
    assert_eq!(raw.stage, "admission");
    assert_eq!(raw.code, ResourceKind::InputTooLarge.code());
    assert_eq!(
        error.diagnostic().unwrap().code,
        ResourceKind::InputTooLarge.code()
    );
    let error = request_data_staged(None, "941", "fn main() int { return 42 }", "d_missing", &[])
        .unwrap_err();
    assert!(matches!(error, DataRequestError::Lower(_)));
    let diag = error.diagnostic().unwrap();
    assert_eq!(diag.stage, "frontend");
    assert_eq!(diag.code, "lower.entry");
}

#[test]
fn typed_nonfrontend_admission_failures_are_not_frontend_errors() {
    use ace_av::data_driver::DataRequestError;
    use ace_av_core::{
        abc::AbcDecodeError, abc_verify::AbcAdmissionError, mir_verify::MirVerifyError,
    };
    let mir = MirVerifyError::Invalid {
        code: "mir.entry-sig".into(),
        message: "frontend: fake".into(),
        span: None,
    };
    let codec = AbcDecodeError::Format {
        code: "abc.magic".into(),
        message: "frontend: fake".into(),
    };
    for (error, code) in [
        (DataRequestError::MirVerify(mir.clone()), "mir.entry-sig"),
        (DataRequestError::Encode(codec.clone()), "abc.magic"),
        (
            DataRequestError::AbcAdmit(AbcAdmissionError::Verify(mir)),
            "mir.entry-sig",
        ),
        (
            DataRequestError::AbcAdmit(AbcAdmissionError::Decode(codec)),
            "abc.magic",
        ),
    ] {
        let diag = error.diagnostic().unwrap();
        assert_eq!(diag.stage, "verify");
        assert_eq!(diag.code, code);
    }
    assert!(
        DataRequestError::AbcAdmit(AbcAdmissionError::Identity("frontend: fake".into()))
            .diagnostic()
            .is_none()
    );
}

#[test]
fn legitimate_records_keep_values_in_both_apis_and_the_worker() {
    use ace_av::data_driver::{request_data, request_data_staged};
    for (name, want) in [("M01", 0), ("R01", 19), ("R02", 9)] {
        let src = source(name);
        for outcome in [
            request_data(None, "941", &src, "d_main", &[]).unwrap(),
            request_data_staged(None, "941", &src, "d_main", &[]).unwrap(),
        ] {
            let Some(ace_av_core::contract::ExecOutcome::Success { value, .. }) = outcome.av else {
                panic!("success required")
            };
            assert_eq!(value.as_i32(), Some(want));
        }
        let actual = worker(&src, "ac-av/data-run-v1", None);
        assert_eq!(actual.kind, ExitKind::Success);
        let value: Value = serde_json::from_slice(actual.stdout.as_deref().unwrap()).unwrap();
        assert_eq!(value["ok"]["steps"][0]["outcome"], "success");
        assert_eq!(value["ok"]["steps"][0]["value"], want);
    }
}

#[test]
fn core_and_owned_compatibility_consumers_stay_accepted() {
    for recipe in ["ac-av/core-run-v1", "ac-av/owned-run-v1"] {
        let actual = worker("fn main() int { return 42 }", recipe, None);
        assert_eq!(actual.kind, ExitKind::Success);
        let value: Value = serde_json::from_slice(actual.stdout.as_deref().unwrap()).unwrap();
        assert_eq!(value["ok"]["steps"][0]["outcome"], "success");
        assert_eq!(value["ok"]["steps"][0]["value"], 42);
    }
}

#[test]
fn real_data_trace_and_guest_trap_do_not_become_diagnostics() {
    let trace="type P { a int; b int } fn main() int { let p = P { b: mark_b(), a: mark_a() }; return p.a + p.b }";
    let result = worker_caps(trace, "ac-av/data-run-v1", None, &["hir.test.trace.v1"]);
    assert_eq!(result.kind, ExitKind::Success);
    let value: Value = serde_json::from_slice(result.stdout.as_deref().unwrap()).unwrap();
    let step = &value["ok"]["steps"][0];
    assert_eq!(step["outcome"], "success");
    assert_eq!(step["value"], 3);
    assert_eq!(step["trace"], json!(["b", "a"]));
    let result = worker(&source("B14"), "ac-av/data-run-v1", None);
    assert_eq!(result.kind, ExitKind::Success);
    let value: Value = serde_json::from_slice(result.stdout.as_deref().unwrap()).unwrap();
    let step = &value["ok"]["steps"][0];
    assert_eq!(step["outcome"], "trap");
    assert_eq!(step["trap"], "av.trap.div-by-zero");
    assert!(step["diag"].is_null());
}

#[test]
fn actual_store_failure_is_infrastructure_not_a_source_diagnostic() {
    use ace_av::data_driver::{request_data_staged, DataRequestError};
    let root = scratch();
    let occupied = root.join("file-not-store");
    std::fs::write(&occupied, b"occupied").unwrap();
    let error = request_data_staged(
        Some(&occupied),
        "941",
        "fn main() int { return 42 }",
        "d_main",
        &[],
    )
    .unwrap_err();
    assert!(matches!(error, DataRequestError::Infrastructure(_)));
    assert!(error.diagnostic().is_none());
    let result = worker(
        "fn main() int { return 42 }",
        "ac-av/data-run-v1",
        Some(&occupied),
    );
    assert!(matches!(result.kind, ExitKind::Failure(_)));
    assert!(result.stdout.as_deref().unwrap().is_empty());
    assert!(std::fs::read(&occupied).unwrap() == b"occupied");
}

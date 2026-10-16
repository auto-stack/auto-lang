//! PLAN-788 R1 repair counterexamples (review-r1.md F01/F03/F05/F08/F09/
//! F10): each test reproduces a finding's red shape against the REPAIRED
//! worker and pins the invariant. Windows-only (native toolchain).

#![cfg(windows)]

use ac_case_worker::exec;
use ac_case_worker::protocol::{Envelope, InputSpec, ReqBudget, WorkerRequest, WORKER_SCHEMA};
use base64::Engine;
use std::path::PathBuf;

fn b64(s: &str) -> String {
    base64::engine::general_purpose::STANDARD.encode(s.as_bytes())
}

static SEQ: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

fn scratch(family: &str) -> PathBuf {
    let p = std::env::temp_dir().join(format!(
        "ac788-r1-{}-{}-{}",
        family,
        std::process::id(),
        SEQ.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
    ));
    let _ = std::fs::remove_dir_all(&p);
    std::fs::create_dir_all(&p).unwrap();
    p
}

fn req(
    kind: &str,
    source: &str,
    build_ms: u64,
    total_ms: u64,
    store: Option<&str>,
) -> WorkerRequest {
    let n = SEQ.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let s = scratch("req").display().to_string();
    WorkerRequest {
        schema: WORKER_SCHEMA.into(),
        kind: kind.into(),
        nonce: format!("n-{}", n),
        request_hash: format!("h-{}", n),
        input: InputSpec::Source {
            source_b64: b64(source),
        },
        profile: "core".into(),
        entry: "main".into(),
        budget: ReqBudget {
            build_ms,
            run_ms: 5_000,
            total_ms,
        },
        scratch: s,
        store_root: store.map(|s| s.to_string()),
        cache: None,
        project: None,
        include_hir: None,
    }
}

const P04: &str = "fn main() int { var i = 0; while i < 3 { i = i + 1 }; return i }";

#[test]
fn r1_budget_one_ms_total_is_enforced_not_ignored() {
    // F05 red shape: build=1ms/total=1ms on a REAL source build used to
    // succeed; every exit now checks the declared budget.
    let err = exec::execute(&req("run", P04, 1, 1, None)).unwrap_err();
    assert!(
        err.code == "budget.total" || err.code == "budget.build" || err.code == "budget.run",
        "a 1ms budget must fail a real build, got {} ({})",
        err.code,
        err.message
    );
}

#[test]
fn r1_protocol_rejects_unknown_fields() {
    // F03: an unknown field on the envelope/request is a decode error —
    // never parsed-then-ignored.
    let raw = r#"{"op":"request","request":{"schema":"acworker/1","kind":"check","nonce":"x","request_hash":"y","input":{"kind":"source","source_b64":"eA=="},"profile":"core","entry":"main","budget":{"build_ms":1000,"run_ms":1000,"total_ms":2000},"scratch":"C:\\","surprise_field":1}}"#;
    let e: Result<Envelope, _> = serde_json::from_str(raw);
    assert!(e.is_err(), "deny_unknown_fields must reject the request");
}

#[test]
fn r1_product_run_invalid_exe_is_infra_error() {
    // F02 red shape: a consumer pointing at a NON-executable product file
    // is an infrastructure error, never a PASS.
    let dir = scratch("bad-exe");
    let exe = dir.join("app.exe");
    std::fs::write(&exe, b"not a PE executable").unwrap();
    let mut r = req("run", P04, 60_000, 70_000, None);
    r.input = InputSpec::Product {
        exe_path: exe.display().to_string(),
    };
    let err = exec::execute(&r).unwrap_err();
    assert!(
        err.code.starts_with("run.") || err.code == "run.failed",
        "invalid product exe must be a run failure, got {} ({})",
        err.code,
        err.message
    );
}

#[test]
fn r1_product_run_valid_exe_under_containment() {
    // F01 positive: the consumer run goes through the SAME contained,
    // budgeted executor (Job + deadline + caps), exit observed.
    let work = scratch("prod");
    let build = exec::execute(&req("run", P04, 60_000, 70_000, None)).unwrap();
    let exe_src = build
        .artifacts
        .iter()
        .find(|a| a.kind == "exe")
        .expect("build produced an exe");
    // Stage a copy to exercise the product path with a fresh file.
    let exe = work.join("app.exe");
    std::fs::copy(&exe_src.path, &exe).unwrap();
    let mut r = req("run", P04, 60_000, 70_000, None);
    r.input = InputSpec::Product {
        exe_path: exe.display().to_string(),
    };
    let ok = exec::execute(&r).unwrap();
    assert_eq!(ok.run.as_ref().unwrap().exit_code, Some(3));
    assert!(ok.artifacts.is_empty(), "product runs produce no artifacts");
}

#[test]
fn r1_backend_missing_entry_typed_through_store_path() {
    // F09 red shape: a valid source whose entry is ABSENT — with the store
    // attached the driver's Backend error used to surface as infra
    // (driver.failed); it must be the typed backend/entry.not-found
    // observation the error-completion oracle matches.
    let store = scratch("store-be").join("store");
    std::fs::create_dir_all(&store).unwrap();
    let src = "fn helper() int { return 1 }";
    let mut r = req("run", src, 60_000, 70_000, Some(store.to_str().unwrap()));
    r.entry = "main".into();
    let ok = exec::execute(&r).unwrap();
    assert!(
        ok.cache.is_none(),
        "a language failure is never a cache record"
    );
    let d = &ok.diagnostics[0];
    assert_eq!(d.stage, "backend", "{:#?}", ok.diagnostics);
    assert_eq!(d.code, "entry.not-found");
}

#[test]
fn r1_negative_trace_carries_frontend_spans() {
    // F10 red shape: a resolve-stage failure used to return trace=[];
    // the observer spans must travel with the failure.
    let ok = exec::execute(&req(
        "check",
        "fn main() int { return x }",
        60_000,
        70_000,
        None,
    ))
    .unwrap();
    assert_eq!(ok.diagnostics[0].stage, "resolve");
    let stages: Vec<&str> = ok.trace.iter().map(|s| s.stage.as_str()).collect();
    assert!(stages.contains(&"parse"), "trace {:?}", stages);
    assert!(stages.contains(&"resolve"), "trace {:?}", stages);
}

#[test]
fn r1_cache_nonparticipating_mode_rejected() {
    // F08 mirror: fresh/read-only on a non-participating combination is an
    // admission error, never a silent no-op.
    let mut r = req("check", P04, 60_000, 70_000, Some("unused"));
    r.cache = Some("fresh".into());
    let err = exec::execute(&r).unwrap_err();
    assert_eq!(err.code, "request.cache", "{}", err.message);
}

#[test]
fn r1_worker_exe_sha256_axis_always_present() {
    // F07: every native row's identity axis — present on the DIRECT path
    // too (check), not only driver runs.
    let ok = exec::execute(&req("check", P04, 60_000, 70_000, None)).unwrap();
    assert!(
        !ok.toolchain.worker_exe_sha256.is_empty(),
        "worker exe hash is the identity axis on every path"
    );
}

//! PLAN-788 T-03 worker protocol surface (V-02/V-05 anchors at the worker
//! level): typed stage attribution through the observation seam, budget
//! admission boundaries, session semantics, and one FULL IPC round-trip
//! through the real binary (begin → request → response). Windows-only for
//! execution paths; the negative cases run anywhere.

#![cfg(windows)]

use ac_case_worker::exec;
use ac_case_worker::protocol::{InputSpec, ReqBudget, WorkerRequest, WORKER_SCHEMA};
use base64::Engine;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

fn b64(s: &str) -> String {
    base64::engine::general_purpose::STANDARD.encode(s.as_bytes())
}

fn b64d(s: &str) -> Vec<u8> {
    base64::engine::general_purpose::STANDARD.decode(s).unwrap()
}

fn scratch(name: &str) -> PathBuf {
    let p = std::env::temp_dir().join(format!("ac788-worker-{}", name));
    let _ = std::fs::remove_dir_all(&p);
    std::fs::create_dir_all(&p).unwrap();
    p
}

fn req(kind: &str, input: InputSpec, run_ms: u64) -> WorkerRequest {
    // Unique scratch per call: tests run in parallel THREADS in one process
    // — a shared dir would let one test's cleanup delete another's staged
    // obj mid-link.
    static SEQ: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let n = SEQ.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let s = scratch(&format!("req-{}-{}-{}", kind, std::process::id(), n))
        .display()
        .to_string();
    WorkerRequest {
        schema: WORKER_SCHEMA.into(),
        kind: kind.into(),
        nonce: "n-1".into(),
        request_hash: "h-1".into(),
        input,
        profile: "core".into(),
        entry: "main".into(),
        budget: ReqBudget {
            build_ms: 60_000,
            run_ms,
            total_ms: 70_000,
        },
        scratch: s,
        store_root: None,
        cache: None,
        project: None,
        include_hir: None,
        grammar: None,
        edits: None,
        capabilities: Vec::new(),
        support_lib: None,
    }
}

// ---- typed stage attribution through the observer seam (V-02) ---------

#[test]
fn ac_stage_parse_boundary_attributed() {
    let r = req(
        "check",
        InputSpec::Source {
            source_b64: b64("fn main( int { return 3 }"),
        },
        5_000,
    );
    let ok = exec::execute(&r).unwrap();
    assert!(ok.run.is_none());
    let d = &ok.diagnostics[0];
    assert_eq!(
        d.stage, "parse",
        "observer boundary, not Stage::Text guess: {:#?}",
        ok.diagnostics
    );
    assert!(
        d.code.starts_with("source."),
        "frontend code verbatim: {}",
        d.code
    );
}

#[test]
fn ac_stage_resolve_boundary_attributed() {
    let r = req(
        "check",
        InputSpec::Source {
            source_b64: b64("fn main() int { return x }"),
        },
        5_000,
    );
    let ok = exec::execute(&r).unwrap();
    let d = &ok.diagnostics[0];
    assert_eq!(d.stage, "resolve", "{:#?}", ok.diagnostics);
    assert_eq!(d.code, "source.undefined");
}

#[test]
fn ac_stage_typecheck_boundary_attributed() {
    // `1 < 2` is bool; returning it from an int fn is a type error.
    let r = req(
        "check",
        InputSpec::Source {
            source_b64: b64("fn main() int { return 1 < 2 }"),
        },
        5_000,
    );
    let ok = exec::execute(&r).unwrap();
    let d = &ok.diagnostics[0];
    assert_eq!(d.stage, "typecheck", "{:#?}", ok.diagnostics);
    assert_eq!(d.code, "source.type-mismatch");
}

#[test]
fn ac_stage_raw_hir_bind_rejection_typed() {
    let src = std::fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../ac-core/fixtures/invalid/dangling-expr-ref.atom"),
    )
    .unwrap();
    let r = req("check", InputSpec::Hir { hir_b64: b64(&src) }, 5_000);
    let ok = exec::execute(&r).unwrap();
    let d = &ok
        .diagnostics
        .iter()
        .find(|d| d.code == "bind.dangling-ref")
        .expect("typed bind rejection");
    assert_eq!(d.stage, "bind");
    let span = &src[d.span_start..d.span_end];
    assert!(span.contains("e_missing"), "span points at `{}`", span);
}

#[test]
fn ac_stage_valid_check_carries_canonical_hir_and_trace() {
    let src = std::fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../ac-core/fixtures/source/valid/p04-while-retest.at"),
    )
    .unwrap();
    let r = req(
        "check",
        InputSpec::Source {
            source_b64: b64(&src),
        },
        5_000,
    );
    let ok = exec::execute(&r).unwrap();
    assert!(ok.diagnostics.is_empty());
    let hir = b64d(ok.hir_b64.as_deref().unwrap());
    assert!(hir.starts_with(b"hir("), "canonical HIR artifact bytes");
    assert_eq!(ok.artifacts.len(), 1);
    assert_eq!(ok.artifacts[0].kind, "hir");
    let stages: Vec<&str> = ok.trace.iter().map(|s| s.stage.as_str()).collect();
    for expect in ["parse", "resolve", "typecheck", "emit", "bind", "verify"] {
        assert!(
            stages.contains(&expect),
            "trace stages {:?} missing {}",
            stages,
            expect
        );
    }
}

#[test]
fn ac_stage_build_run_end_to_end_with_manifests() {
    let src = std::fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../ac-core/fixtures/source/valid/p04-while-retest.at"),
    )
    .unwrap();
    let r = req(
        "run",
        InputSpec::Source {
            source_b64: b64(&src),
        },
        5_000,
    );
    let ok = exec::execute(&r).unwrap();
    assert!(ok.diagnostics.is_empty(), "{:#?}", ok.diagnostics);
    let run = ok.run.as_ref().unwrap();
    assert_eq!(run.exit_code, Some(3), "p04 oracle");
    assert!(b64d(&run.stdout_b64).is_empty());
    assert!(b64d(&run.stderr_b64).is_empty());
    // Manifest hashes bind the REAL files in scratch — parent re-verifies.
    //
    // PLAN-871 T-05: the product shape is BACKEND-DEPENDENT. The shared-MIR
    // backend links a MULTI-UNIT closure and has no single `app.obj`
    // (freeze F4 item 2 — the same reason `ac-native` no longer emits
    // `<stem>.obj`): its manifest is `hir + exe`. The legacy backend keeps
    // `hir + obj + exe`. The reported backend axis decides, so this test
    // judges BOTH shapes instead of pinning one backend.
    let mn_backend = ok.toolchain.backend.starts_with("mn:");
    assert_eq!(
        ok.artifacts.len(),
        if mn_backend { 2 } else { 3 },
        "backend {:?} product manifest",
        ok.toolchain.backend
    );
    if mn_backend {
        assert!(
            ok.artifacts.iter().all(|a| a.logical != "app.obj"),
            "the shared-MIR backend must not publish a single app.obj"
        );
    }
    for a in &ok.artifacts {
        let bytes = std::fs::read(&a.path).unwrap();
        assert_eq!(bytes.len(), a.bytes, "{} length", a.logical);
        let sha = sha256_hex(&bytes);
        assert_eq!(sha, a.sha256, "{} hash", a.logical);
    }
    let exe = ok.artifacts.iter().find(|a| a.kind == "exe").unwrap();
    assert_eq!(&std::fs::read(&exe.path).unwrap()[0..2], b"MZ");
}

// ---- budget boundaries (V-05 anchors at the worker level) -------------

#[test]
fn process_budget_run_deadline_kills_real_program() {
    // A REAL AC program that loops forever: the run budget must kill it and
    // surface a budget.run infrastructure error (never a hang, never a
    // truncated success, never a language diagnostic).
    let src = "fn main() int { var i = 0; while 0 < 1 { i = i + 1 }; return i }";
    let r = req(
        "run",
        InputSpec::Source {
            source_b64: b64(src),
        },
        800,
    );
    let t = std::time::Instant::now();
    let err = exec::execute(&r).unwrap_err();
    assert_eq!(err.code, "budget.run", "{}", err.message);
    assert!(
        t.elapsed().as_millis() < 10_000,
        "killed on the budget, not elsewhere ({:?})",
        t.elapsed()
    );
}

#[test]
fn process_budget_admission_rejects_over_cap() {
    let mut r = req(
        "check",
        InputSpec::Source {
            source_b64: b64("fn main() int { return 0 }"),
        },
        5_000,
    );
    r.budget.build_ms = 60_001;
    let err = exec::execute(&r).unwrap_err();
    assert_eq!(err.code, "request.budget");

    let r = req(
        "check",
        InputSpec::Source {
            source_b64: b64("fn main() int { return 0 }"),
        },
        5_001,
    );
    let err = exec::execute(&r).unwrap_err();
    assert_eq!(err.code, "request.budget");

    let mut r = req(
        "check",
        InputSpec::Source {
            source_b64: b64("fn main() int { return 0 }"),
        },
        5_000,
    );
    r.schema = "acworker/2".into();
    let err = exec::execute(&r).unwrap_err();
    assert_eq!(err.code, "request.schema");

    let mut r = req(
        "check",
        InputSpec::Source {
            source_b64: b64("fn main() int { return 0 }"),
        },
        5_000,
    );
    r.kind = "emit".into();
    let err = exec::execute(&r).unwrap_err();
    assert_eq!(err.code, "request.kind");

    let mut r = req(
        "check",
        InputSpec::Source {
            source_b64: b64("fn main() int { return 0 }"),
        },
        5_000,
    );
    r.profile = "text".into();
    let err = exec::execute(&r).unwrap_err();
    assert_eq!(err.code, "request.profile");
}

#[test]
fn process_budget_scratch_admission() {
    let mut r = req(
        "check",
        InputSpec::Source {
            source_b64: b64("fn main() int { return 0 }"),
        },
        5_000,
    );
    r.scratch = "relative/path".into();
    assert_eq!(exec::execute(&r).unwrap_err().code, "request.scratch");

    let mut r = req(
        "check",
        InputSpec::Source {
            source_b64: b64("fn main() int { return 0 }"),
        },
        5_000,
    );
    r.scratch = std::env::temp_dir()
        .join("ac788-worker-nonexistent-dir-xyz")
        .display()
        .to_string();
    assert_eq!(exec::execute(&r).unwrap_err().code, "request.scratch");
}

// ---- full IPC round-trip through the real binary -----------------------

#[test]
fn protocol_session_roundtrip_via_binary() {
    let exe = env!("CARGO_BIN_EXE_ac-case-worker");
    let src = std::fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../ac-core/fixtures/source/valid/p04-while-retest.at"),
    )
    .unwrap();
    let request = req(
        "run",
        InputSpec::Source {
            source_b64: b64(&src),
        },
        5_000,
    );
    let mut child = Command::new(exe)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn worker");
    {
        let mut stdin = child.stdin.take().unwrap();
        writeln!(stdin, r#"{{"op":"begin","schema":"{}"}}"#, WORKER_SCHEMA).unwrap();
        let line = serde_json::to_string(&ac_case_worker::Envelope::Request {
            request: request.clone(),
        })
        .unwrap();
        writeln!(stdin, "{}", line).unwrap();
        writeln!(stdin, r#"{{"op":"end"}}"#).unwrap();
        // stdin drops here → worker sees EOF even if it ignored `end`
    }
    let out = child.wait_with_output().unwrap();
    assert!(out.status.success(), "worker exit: {:?}", out.status);
    let stdout = String::from_utf8(out.stdout).unwrap();
    let mut lines = stdout.lines();
    let resp_line = lines.next().expect("one response line");
    let resp: ac_case_worker::WorkerResponse = serde_json::from_str(resp_line).unwrap();
    assert_eq!(resp.schema, WORKER_SCHEMA);
    assert_eq!(resp.nonce, "n-1", "nonce echoes the request");
    assert_eq!(resp.request_hash, "h-1", "hash echoes the request");
    assert!(resp.error.is_none(), "{:#?}", resp.error);
    let ok = resp.ok.unwrap();
    assert_eq!(ok.run.unwrap().exit_code, Some(3));
    // Exactly one response line for exactly one request.
    assert!(lines.next().is_none(), "no stray lines: {:?}", stdout);
}

fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut h = Sha256::new();
    h.update(bytes);
    h.finalize().iter().map(|b| format!("{:02x}", b)).collect()
}

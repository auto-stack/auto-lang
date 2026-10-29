//! PLAN-871 T-04 backend switch counterexamples (V-02): the ac-case worker
//! resolves `AC_NATIVE_BACKEND` PER REQUEST (freeze F1) and switches the
//! core/scalar build/run arms to the shared-MIR backend, with a per-profile
//! rollback that is observable in the response's `toolchain.backend` axis.
//!
//! Windows-only (native toolchain). The env var is process-global, so every
//! test that touches it holds [`ENV_LOCK`] and restores the ambient value.

#![cfg(windows)]

use ac_case_worker::exec;
use ac_case_worker::protocol::{InputSpec, ReqBudget, WorkerRequest, WORKER_SCHEMA};
use base64::Engine;
use std::path::PathBuf;

static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
static SEQ: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

const VAR: &str = "AC_NATIVE_BACKEND";
const MN_PREFIX: &str = "mn:";

fn b64(s: &str) -> String {
    base64::engine::general_purpose::STANDARD.encode(s.as_bytes())
}

fn scratch(family: &str) -> PathBuf {
    let p = std::env::temp_dir().join(format!(
        "ac871-{}-{}-{}",
        family,
        std::process::id(),
        SEQ.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
    ));
    let _ = std::fs::remove_dir_all(&p);
    std::fs::create_dir_all(&p).unwrap();
    p
}

fn req(kind: &str, profile: &str, source: &str, store: Option<&PathBuf>) -> WorkerRequest {
    let n = SEQ.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    WorkerRequest {
        schema: WORKER_SCHEMA.into(),
        kind: kind.into(),
        nonce: format!("n-{n}"),
        request_hash: format!("h-{n}"),
        input: InputSpec::Source {
            source_b64: b64(source),
        },
        profile: profile.into(),
        entry: "main".into(),
        budget: ReqBudget {
            build_ms: 60_000,
            run_ms: 5_000,
            total_ms: 70_000,
        },
        scratch: scratch("req").display().to_string(),
        store_root: store.map(|s| s.display().to_string()),
        cache: None,
        project: store.map(|_| "ac-871".to_string()),
        include_hir: None,
        grammar: None,
        edits: None,
        capabilities: Vec::new(),
        support_lib: None,
    }
}

fn logicals(req: &WorkerRequest) -> Vec<String> {
    let out = exec::execute(req).expect("worker request runs");
    out.artifacts.iter().map(|a| a.logical.clone()).collect()
}

fn backend_of(req: &WorkerRequest) -> String {
    let out = exec::execute(req).expect("worker request runs");
    out.toolchain.backend
}

const SRC: &str = "fn main() int { return 14 }";

#[test]
fn t04_default_table_is_mn_for_core_and_scalar() {
    let _g = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    std::env::remove_var(VAR);
    for profile in ["core", "scalar"] {
        let out = exec::execute(&req("run", profile, SRC, None)).expect("run executes");
        assert_eq!(out.run.as_ref().and_then(|r| r.exit_code), Some(14));
        assert!(
            out.toolchain.backend.starts_with(MN_PREFIX),
            "profile {profile} default must be mn, got {:?}",
            out.toolchain.backend
        );
        // mn links a multi-unit closure: the build manifest carries no
        // single `app.obj` (freeze F4 item 2), only hir + exe.
        let logs = logicals(&req("build", profile, SRC, None));
        assert_eq!(
            logs,
            vec!["hir.atom".to_string(), "app.exe".to_string()],
            "profile {profile} mn build products"
        );
    }
}

#[test]
fn t04_legacy_switch_is_retired_after_g_delete() {
    // PLAN-871 T-06: the legacy HIR→CLIF backend (and its obj product) is
    // gone. A request that still resolves to it is refused EXPLICITLY with
    // the retirement message — never silently served by mn.
    let _g = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    std::env::set_var(VAR, "legacy");
    let err = exec::execute(&req("run", "core", SRC, None))
        .expect_err("legacy backend must be refused after retirement");
    assert_eq!(err.code, "request.backend", "{}", err.message);
    assert!(
        err.message.contains("retired") && err.message.contains("T-06"),
        "retirement must be explicit: {}",
        err.message
    );
    std::env::remove_var(VAR);
}

#[test]
fn t04_per_profile_rollback_is_observable_and_refused() {
    let _g = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    // A profile-scoped rollback still RESOLVES (the axis stays observable in
    // reports), but the request is refused: the backend no longer exists.
    std::env::set_var(VAR, "core=legacy");
    let err = exec::execute(&req("run", "core", SRC, None))
        .expect_err("core=legacy must be refused after retirement");
    assert_eq!(err.code, "request.backend");
    // The sibling profile keeps the switched default.
    assert!(backend_of(&req("run", "scalar", SRC, None)).starts_with(MN_PREFIX));
    // And the whole-table flip is the other direction.
    std::env::set_var(VAR, "mn");
    assert!(backend_of(&req("run", "core", SRC, None)).starts_with(MN_PREFIX));
    std::env::remove_var(VAR);
}

fn counter(ok: &ac_case_worker::OkOut, name: &str) -> u32 {
    ok.cache
        .as_ref()
        .and_then(|c| c.counters.iter().find(|(n, _)| n == name))
        .map(|(_, v)| *v)
        .unwrap_or(0)
}

fn hit(ok: &ac_case_worker::OkOut, name: &str) -> String {
    ok.cache
        .as_ref()
        .and_then(|c| c.hits.iter().find(|(n, _)| n == name))
        .map(|(_, v)| v.clone())
        .unwrap_or_default()
}

#[test]
fn t04_malformed_switch_is_an_explicit_refusal() {
    let _g = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    std::env::set_var(VAR, "widget=mn");
    let err = exec::execute(&req("check", "core", SRC, None))
        .expect_err("a malformed AC_NATIVE_BACKEND must refuse the request");
    assert_eq!(err.code, "request.backend", "{}", err.message);
    assert!(err.message.contains("widget"), "{}", err.message);
    std::env::remove_var(VAR);
}

#[test]
fn t04_store_backed_mn_run_executes_and_reports_backend() {
    let _g = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    std::env::remove_var(VAR);
    let store = scratch("store");
    let out = exec::execute(&req("run", "core", SRC, Some(&store))).expect("store run executes");
    assert_eq!(out.run.as_ref().and_then(|r| r.exit_code), Some(14));
    assert!(out.toolchain.backend.starts_with(MN_PREFIX));
    assert!(
        out.cache.is_some(),
        "a store-backed mn request reports its cache observation"
    );
}

fn read_repo(rel: &str) -> String {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(rel);
    std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("read {}: {e}", p.display()))
}

#[test]
fn t04_hir_atom_input_builds_and_runs_through_mn() {
    let _g = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    std::env::remove_var(VAR);
    let atom = read_repo("experimental/ac-core/fixtures/native/add-2-3.atom");
    let mut r = req("run", "core", "", None);
    r.input = InputSpec::Hir {
        hir_b64: b64(&atom),
    };
    r.entry = "test_entry".into();
    let out = exec::execute(&r).expect("HIR atom run executes");
    assert_eq!(out.run.as_ref().and_then(|x| x.exit_code), Some(5));
    assert!(out.toolchain.backend.starts_with(MN_PREFIX));
    let logs: Vec<String> = out.artifacts.iter().map(|a| a.logical.clone()).collect();
    assert!(logs.contains(&"hir.atom".to_string()), "{logs:?}");
    assert!(logs.contains(&"app.exe".to_string()), "{logs:?}");
}

#[test]
fn t04_trace_support_library_travels_to_the_mn_link() {
    let _g = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    std::env::remove_var(VAR);
    // The landed trace-marks support library (built on first use).
    let support = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../ac-core/test-support")
        .canonicalize()
        .expect("test-support dir");
    let lib = support.join("target/debug/ac_trace_support.lib");
    if !lib.is_file() {
        let st = std::process::Command::new("cargo")
            .args(["build", "--locked"])
            .current_dir(&support)
            .status()
            .expect("spawn cargo for test-support");
        assert!(st.success(), "test-support build failed");
    }
    let src = "fn pair(a int, b int) int { return a * 10 + b }\n\
               fn main() int { return pair(b: mark_b(), a: mark_a()) }";
    let mut r = req("run", "core", src, None);
    r.capabilities = vec!["hir.test.trace.v1".to_string()];
    r.support_lib = Some(lib.display().to_string());
    let out = exec::execute(&r).expect("trace run executes");
    assert_eq!(out.run.as_ref().and_then(|x| x.exit_code), Some(12));
    let stderr = out
        .run
        .as_ref()
        .map(|x| {
            base64::engine::general_purpose::STANDARD
                .decode(x.stderr_b64.as_bytes())
                .unwrap()
        })
        .unwrap();
    assert_eq!(String::from_utf8_lossy(&stderr), "ba");
}

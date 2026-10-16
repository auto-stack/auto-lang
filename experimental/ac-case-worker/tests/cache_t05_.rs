//! PLAN-788 T-05 cache/trace adapter (V-08/V-09/V-10 anchors at the worker
//! level): real unified-store hit/miss/invalidate semantics through the
//! request path, truthful counters, read-only discipline, and the store
//! lifecycle boundaries. Windows-only (native toolchain).

#![cfg(windows)]

use ac_case_worker::exec;
use ac_case_worker::protocol::{InputSpec, ReqBudget, WorkerRequest, WORKER_SCHEMA};
use base64::Engine;
use std::path::PathBuf;

fn b64(s: &str) -> String {
    base64::engine::general_purpose::STANDARD.encode(s.as_bytes())
}

fn scratch(family: &str) -> PathBuf {
    let p = std::env::temp_dir().join(format!("ac788-cache-{}-{}", family, std::process::id()));
    let _ = std::fs::remove_dir_all(&p);
    std::fs::create_dir_all(&p).unwrap();
    p
}

static SEQ: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

fn req(kind: &str, source: &str, cache: Option<&str>, store: Option<&str>) -> WorkerRequest {
    let n = SEQ.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let s = scratch(&format!("req-{}", n)).display().to_string();
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
            build_ms: 60_000,
            run_ms: 5_000,
            total_ms: 70_000,
        },
        scratch: s,
        store_root: store.map(|s| s.to_string()),
        cache: cache.map(|s| s.to_string()),
        project: None,
        include_hir: None,
    }
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

const P04_3: &str = "fn main() int { var i = 0; while i < 3 { i = i + 1 }; return i }";
const P04_4: &str = "fn main() int { var i = 0; while i < 4 { i = i + 1 }; return i }";

#[test]
fn native_cache_fresh_then_object_hit() {
    let root = scratch("store-fresh-hit");
    let store = root.join("store").display().to_string();

    let ok = exec::execute(&req("run", P04_3, Some("reuse"), Some(&store))).unwrap();
    assert_eq!(ok.run.as_ref().unwrap().exit_code, Some(3));
    assert_eq!(hit(&ok, "object"), "lowered", "first request computes");
    assert_eq!(counter(&ok, "parse"), 1);
    assert_eq!(counter(&ok, "verify"), 1);
    assert_eq!(counter(&ok, "lower"), 1);
    assert_eq!(counter(&ok, "link"), 1, "hit or miss, the link is real");
    assert_eq!(counter(&ok, "run"), 1, "worker-owned run leg counted");

    // Second request over the SAME on-disk store: object hit — the
    // frontend/verify/lower are provably skipped (counters), link+run real.
    let ok = exec::execute(&req("run", P04_3, Some("reuse"), Some(&store))).unwrap();
    assert_eq!(
        ok.run.as_ref().unwrap().exit_code,
        Some(3),
        "oracle re-observed"
    );
    assert_eq!(hit(&ok, "object"), "hit");
    assert_eq!(counter(&ok, "parse"), 0, "frontend skipped on object hit");
    assert_eq!(counter(&ok, "verify"), 0, "verify skipped on object hit");
    assert_eq!(counter(&ok, "lower"), 0, "lowering skipped on object hit");
    assert_eq!(counter(&ok, "link"), 1);
    assert_eq!(counter(&ok, "run"), 1);
}

#[test]
fn native_cache_fresh_mode_forces_recompute_on_warm_store() {
    let root = scratch("store-fresh-mode");
    let store = root.join("store").display().to_string();
    exec::execute(&req("run", P04_3, Some("reuse"), Some(&store))).unwrap();

    let ok = exec::execute(&req("run", P04_3, Some("fresh"), Some(&store))).unwrap();
    assert_eq!(ok.run.as_ref().unwrap().exit_code, Some(3));
    assert_eq!(
        hit(&ok, "object"),
        "lowered",
        "fresh forbids reading old results"
    );
    assert_eq!(counter(&ok, "parse"), 1, "fresh recomputes the frontend");
    assert_eq!(counter(&ok, "verify"), 1);
    assert_eq!(counter(&ok, "lower"), 1);
}

#[test]
fn native_invalidation_body_edit_full_miss() {
    let root = scratch("store-body-edit");
    let store = root.join("store").display().to_string();
    exec::execute(&req("run", P04_3, Some("reuse"), Some(&store))).unwrap();

    // Same-signature body edit (3→4): the snapshot changes → every layer
    // keyed under it misses; the object is recomputed and the NEW oracle
    // (exit 4) is what actually runs.
    let ok = exec::execute(&req("run", P04_4, Some("reuse"), Some(&store))).unwrap();
    assert_eq!(ok.run.as_ref().unwrap().exit_code, Some(4));
    assert_eq!(
        hit(&ok, "object"),
        "lowered",
        "body change must miss the object"
    );
    assert_eq!(counter(&ok, "parse"), 1);
    assert_eq!(counter(&ok, "lower"), 1);
}

#[test]
fn native_store_read_only_missing_root_is_infra_error() {
    let root = scratch("store-ro-missing");
    let missing = root.join("never-created").display().to_string();
    let err = exec::execute(&req("run", P04_3, Some("read-only"), Some(&missing))).unwrap_err();
    assert_eq!(err.code, "store.missing-root", "{}", err.message);
}

#[test]
fn native_store_read_only_consumes_hits_and_never_publishes() {
    let root = scratch("store-ro");
    let store = root.join("store").display().to_string();
    // Warm the store (publishing allowed).
    exec::execute(&req("run", P04_3, Some("reuse"), Some(&store))).unwrap();

    // Read-only consumes the hit…
    let ok = exec::execute(&req("run", P04_3, Some("read-only"), Some(&store))).unwrap();
    assert_eq!(ok.run.as_ref().unwrap().exit_code, Some(3));
    assert_eq!(hit(&ok, "object"), "hit");
    assert_eq!(counter(&ok, "parse"), 0);
    assert_eq!(counter(&ok, "store_puts"), 0, "read-only never writes");

    // …and a read-only MISS (body edit) recomputes and runs for real
    // without publishing: a later reuse request must still miss.
    let ok = exec::execute(&req("run", P04_4, Some("read-only"), Some(&store))).unwrap();
    assert_eq!(ok.run.as_ref().unwrap().exit_code, Some(4));
    assert_eq!(hit(&ok, "object"), "lowered");
    assert_eq!(counter(&ok, "store_puts"), 0);
    let ok = exec::execute(&req("run", P04_4, Some("reuse"), Some(&store))).unwrap();
    assert_eq!(
        hit(&ok, "object"),
        "lowered",
        "the read-only miss published nothing — reuse still computes"
    );
    assert_eq!(counter(&ok, "parse"), 1);
}

#[test]
fn native_cache_language_failure_replays_typed_not_cached() {
    let root = scratch("store-neg");
    let store = root.join("store").display().to_string();
    let bad = "fn main() int { return x }";
    let ok = exec::execute(&req("check", bad, Some("reuse"), Some(&store))).unwrap();
    // check never routes through the driver (its observation IS the
    // frontend) — typed diagnostics, no cache record.
    let d = &ok.diagnostics[0];
    assert_eq!(d.stage, "resolve");
    assert_eq!(d.code, "source.undefined");

    // build/run language failures through the driver replay typed too.
    let ok = exec::execute(&req("run", bad, Some("reuse"), Some(&store))).unwrap();
    assert!(
        ok.cache.is_none(),
        "a language failure is never a cache record"
    );
    let d = &ok.diagnostics[0];
    assert_eq!(
        d.stage, "resolve",
        "typed replay of the driver's Frontend error"
    );
    assert_eq!(d.code, "source.undefined");
    assert!(ok.run.is_none());
}

#[test]
fn native_cache_trace_records_stage_spans() {
    let root = scratch("store-trace");
    let store = root.join("store").display().to_string();
    let ok = exec::execute(&req("run", P04_3, Some("reuse"), Some(&store))).unwrap();
    let stages: Vec<&str> = ok.trace.iter().map(|s| s.stage.as_str()).collect();
    // R1 F10: the driver path reports the REAL frontend boundaries
    // (parse/resolve/typecheck/emit/bind from the observer), not a merged
    // "frontend" blob.
    for expect in [
        "parse", "resolve", "typecheck", "emit", "bind", "verify", "lower", "link", "run",
        "total",
    ] {
        assert!(
            stages.contains(&expect),
            "trace {:?} missing {}",
            stages,
            expect
        );
    }
    // No cached PASS: the run span exists on EVERY request — the oracle is
    // re-observed each time (second request included).
    let ok = exec::execute(&req("run", P04_3, Some("reuse"), Some(&store))).unwrap();
    assert!(
        ok.trace.iter().any(|s| s.stage == "run"),
        "hit request still runs for real"
    );
    assert!(ok.trace.iter().any(|s| s.stage == "link"));
}

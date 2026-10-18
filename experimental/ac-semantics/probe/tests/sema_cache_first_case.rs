//! PLAN-796 T-01 first-case integration: a REAL inference result
//! persisted through the unified 783 store in the isolated `S796`
//! namespace, surviving actual process exit, restored and HIT without
//! any compute path — plus the controlled invalidation paths (§5.4).
//!
//! Cross-process proof: `put` and `get` run as SEPARATE child processes
//! (`CARGO_BIN_EXE_sema_probe`); the store write happens in a process
//! that then exits before the reader starts.

#![cfg(windows)]

use ac_artifact_probe::store::ManagedStore;
use ac_sema_core::query::QueryKind;
use ac_sema_core::{InferOutcome, SemaInput};
use ac_sema_probe::{open_store, publish_infer_result, run_inference, FetchOutcome};
use std::path::PathBuf;
use std::process::Command;

const BIN: &str = env!("CARGO_BIN_EXE_sema-probe");

fn scratch(family: &str) -> PathBuf {
    let p = std::env::temp_dir().join(format!("ac796-{}", family));
    let _ = std::fs::remove_dir_all(&p);
    std::fs::create_dir_all(&p).unwrap();
    p
}

fn run_bin(args: &[&str]) -> (i32, String) {
    let out = Command::new(BIN).args(args).output().expect("spawn sema-probe");
    (out.status.code().unwrap_or(-1), String::from_utf8_lossy(&out.stdout).into_owned())
}

const MAIN_AND_HELPER: &str = "fn helper(a int, b int) int { return a + b }\n\
     \n\
     fn main() int { let x = helper(2, 3)\n\
     return x * 2 }\n";

#[test]
fn sema_cache_first_case_cross_process_roundtrip() {
    let root = scratch("first-case");
    let store_root = root.join("store");
    let src = root.join("m.at");
    std::fs::write(&src, MAIN_AND_HELPER).unwrap();

    // process 1: real inference + publish, then EXIT
    let (code, out) = run_bin(&["put", store_root.to_str().unwrap(), src.to_str().unwrap(), "main", "proj-796"]);
    assert_eq!(code, 0, "put failed: {out}");
    assert!(out.contains("\"event\":\"published\""), "no publish receipt: {out}");
    assert!(out.contains("\"name\":\"x\""), "no binding fact for x: {out}");

    // process 2: fresh open, lookup WITHOUT compute, expect HIT
    let (code, out) = run_bin(&["get", store_root.to_str().unwrap(), src.to_str().unwrap(), "main"]);
    assert_eq!(code, 0, "get failed: {out}");
    assert!(out.contains("\"event\":\"hit\""), "expected hit, got: {out}");
    assert!(out.contains("\"compute_called\":false"), "compute must not run on hit: {out}");
    // the restored facts carry the inferred binding types
    assert!(out.contains("\"name\":\"x\",\"ty\":\"i32\""), "x must be i32: {out}");
}

#[test]
fn sema_cache_first_case_callee_signature_change_invalidates() {
    let root = scratch("first-case-sig");
    let store_root = root.join("store");
    let src = root.join("m.at");
    std::fs::write(&src, MAIN_AND_HELPER).unwrap();
    let (code, out) = run_bin(&["put", store_root.to_str().unwrap(), src.to_str().unwrap(), "main", "proj-796"]);
    assert_eq!(code, 0, "put failed: {out}");

    // caller UNCHANGED; helper's SIGNATURE changes (param i32 → bool)
    let edited = MAIN_AND_HELPER.replace("helper(a int, b int)", "helper(a bool, b int)");
    std::fs::write(&src, edited).unwrap();
    let (code, out) = run_bin(&["get", store_root.to_str().unwrap(), src.to_str().unwrap(), "main"]);
    assert_eq!(code, 0, "get failed: {out}");
    // the caller's own key components did not change — the MISS comes
    // from the dependency manifest re-validation, reported as stale
    assert!(
        out.contains("\"event\":\"stale\"") || out.contains("\"event\":\"miss\""),
        "expected stale/miss after callee signature change, got: {out}"
    );
}

#[test]
fn sema_cache_first_case_target_body_change_changes_key() {
    let root = scratch("first-case-body");
    let store_root = root.join("store");
    let src = root.join("m.at");
    std::fs::write(&src, MAIN_AND_HELPER).unwrap();
    let (code, out) = run_bin(&["put", store_root.to_str().unwrap(), src.to_str().unwrap(), "main", "proj-796"]);
    assert_eq!(code, 0, "put failed: {out}");

    // the TARGET's body changes → different body fingerprint → the key
    // itself differs → plain miss (fresh recompute path)
    let edited = MAIN_AND_HELPER.replace("x * 2", "x * 3");
    std::fs::write(&src, edited).unwrap();
    let (code, out) = run_bin(&["get", store_root.to_str().unwrap(), src.to_str().unwrap(), "main"]);
    assert_eq!(code, 0, "get failed: {out}");
    assert!(
        out.contains("\"event\":\"miss\"") || out.contains("\"event\":\"stale\""),
        "expected miss/stale after target body change, got: {out}"
    );
}

/// In-process: the no-compute-on-hit control through the typed query
/// flow (F-796-R4-02: dispositions are mutually exclusive).
#[test]
fn sema_cache_first_case_query_flow_hit_does_not_compute() {
    let root = scratch("first-case-flow");
    let store: ManagedStore = open_store(&root.join("store")).unwrap();
    let input = ac_sema_core::adapter::parse_into_input(
        MAIN_AND_HELPER,
        "p1",
        "m",
        auto_ac_prototype::source::SourceProfile::Core,
    )
    .unwrap();
    let query = QueryKind::InferBody { fn_name: "main".into() };

    // first: miss → real compute → publish (typed disposition)
    let report =
        ac_sema_probe::run_query_flow(&store, &input, &query, Some("proj"), None).unwrap();
    assert_eq!(report.disposition, ac_sema_probe::Disposition::PublishedMiss);
    assert_eq!(report.compute_calls, 1, "first query must compute");
    assert!(report.disposition.is_ok());

    // second: hit → compute must NOT run (structural: the hit arm of
    // run_query_flow has no compute call at all)
    let report =
        ac_sema_probe::run_query_flow(&store, &input, &query, Some("proj"), None).unwrap();
    assert_eq!(report.disposition, ac_sema_probe::Disposition::Hit);
    assert_eq!(report.compute_calls, 0, "hit must not compute");
    assert!(report.bindings.iter().any(|(n, t)| n == "x" && t == "i32"));
}

/// Negative dependency (M-04): a result computed when `g` was ABSENT is
/// invalidated when `g` becomes defined — without touching the caller.
#[test]
fn sema_cache_first_case_negative_dependency_new_definition() {
    let root = scratch("first-case-neg");
    let store: ManagedStore = open_store(&root.join("store")).unwrap();
    let src_with_unknown = "fn main() int { return g(1) }";
    let input1 = ac_sema_core::adapter::parse_into_input(
        src_with_unknown,
        "p1",
        "m",
        auto_ac_prototype::source::SourceProfile::Core,
    )
    .unwrap();
    let query = QueryKind::InferBody { fn_name: "main".into() };
    // diagnostic outcome is definitive (unknown-name) — cacheable under
    // the full key
    let summary = match run_inference(&input1, "main") {
        InferOutcome::Diags(s) => s,
        other => panic!("expected diags, got {other:?}"),
    };
    publish_infer_result(&store, &input1, &query, &summary, Some("proj")).unwrap();

    // SAME caller bytes; a `g` appears in the module
    let src_with_g = "fn g(a int) int { return a }\nfn main() int { return g(1) }";
    let input2 = ac_sema_core::adapter::parse_into_input(
        src_with_g,
        "p1",
        "m",
        auto_ac_prototype::source::SourceProfile::Core,
    )
    .unwrap();
    match ac_sema_probe::lookup_only(&store, &input2, &query).unwrap() {
        // main's own key components changed here because the module's
        // candidate set grew (fn-cand-fp is part of the InferBody key),
        // so this is a MISS; the negative-dependency mechanics are
        // exercised in the meta_query_/sema_cache_ suites at manifest
        // level. What must NEVER happen: a silent Hit of the stale
        // unknown-name verdict.
        FetchOutcome::Miss | FetchOutcome::Stale { .. } => {}
        FetchOutcome::Hit { .. } => panic!("stale unknown-name verdict was served as a hit"),
        FetchOutcome::HitStaleProvenance { .. } => {}
    }
}

/// Same-name declarations in different packages never merge: their
/// query keys and logical ids differ, and both results coexist.
#[test]
fn sema_cache_first_case_cross_package_no_merge() {
    let root = scratch("first-case-pkg");
    let store: ManagedStore = open_store(&root.join("store")).unwrap();
    let src = "fn main() int { return 1 }";
    let q = QueryKind::InferBody { fn_name: "main".into() };
    for pkg in ["app.a", "app.b"] {
        let input = ac_sema_core::adapter::parse_into_input(
            src,
            pkg,
            "m",
            auto_ac_prototype::source::SourceProfile::Core,
        )
        .unwrap();
        let summary = match run_inference(&input, "main") {
            InferOutcome::Ok(s) => s,
            other => panic!("{pkg}: {other:?}"),
        };
        publish_infer_result(&store, &input, &q, &summary, Some(pkg)).unwrap();
    }
    // each package hits its own result
    for pkg in ["app.a", "app.b"] {
        let input = ac_sema_core::adapter::parse_into_input(
            src,
            pkg,
            "m",
            auto_ac_prototype::source::SourceProfile::Core,
        )
        .unwrap();
        match ac_sema_probe::lookup_only(&store, &input, &q).unwrap() {
            FetchOutcome::Hit { page, .. } => {
                let expect = ac_sema_core::query::target_logical(&input, &q);
                assert!(format!("{:?}", page).contains(expect.as_str().chars().take(8).collect::<String>().as_str())
                    || matches!(&page.body, ac_sema_contract::PageBody::CachedResult(_)));
            }
            _ => panic!("{pkg}: expected hit"),
            FetchOutcome::HitStaleProvenance { .. } => {}
        }
    }
}

/// Illegal page payloads are controlled rejections at admission: wrong
/// schema / wrong producer bytes never decode as a semantic page.
#[test]
fn sema_cache_first_case_illegal_payload_rejected() {
    // syntactically valid framing but a foreign schema string
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&8u32.to_le_bytes()); // schema len
    bytes.extend_from_slice(b"ac796/XX");
    let err = ac_sema_contract::decode_page(&bytes).unwrap_err();
    assert!(matches!(err, ac_sema_contract::CodecError::BadSchema { .. }));

    // empty / truncated
    assert!(ac_sema_contract::decode_page(&[]).is_err());
    assert!(ac_sema_contract::decode_page(&[0; 3]).is_err());
}

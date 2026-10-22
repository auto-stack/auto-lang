//! cte_first_ — the first real vertical (V-01/V-05 core): real AV
//! execution, consumer backfill changing inference facts + HIR + native
//! behavior, frozen store persistence, cross-process restore, AV-forbid
//! counter-control and cross-project hits.
//!
//! Independent expectations are HAND-COMPUTED here (20+22=42, 9<10=true,
//! branch exits 1/2) — never produced by the AV or the freeze under test.

use auto_cte_probe::driver::{CteDriver, StepStatus};
use std::collections::BTreeMap;
use std::path::Path;
use std::time::Duration;

const SUM42_DECK: &str = r#"
# first vertical: cte_sum(20,22)=42 backfilled into main.ct
deck cte817/1
module main
source <<
fn cte_sum(a int, b int) int {
    return a + b
}

fn main() int {
    let ct = 0
    return ct
}
>>
query q1 = call cte_sum(20, 22)
bind main.ct = q1 expect i32 placeholder 0
entry main
"#;

const BOOL_TRUE_DECK: &str = r#"
# bool branch: cte_flag(9) -> true flips main's branch
deck cte817/1
module main
source <<
fn cte_flag(x int) bool {
    return x < 10
}

fn main() int {
    let gate = false
    if gate {
        return 1
    }
    return 2
}
>>
query q1 = call cte_flag(9)
bind main.gate = q1 expect bool placeholder false
entry main
"#;

fn bool_deck(x: i32) -> String {
    BOOL_TRUE_DECK.replace("call cte_flag(9)", &format!("call cte_flag({x})"))
}

#[test]
fn cte_first_sum42_real_av_and_backfill() {
    let tmp = tempfile::tempdir().unwrap();
    let mut d = CteDriver::ephemeral();
    let out = d.query_step(SUM42_DECK, Some(tmp.path()));
    assert_eq!(out.status, StepStatus::Ok, "diag: {:?}", out.message);
    assert_eq!(out.cache, Some("miss"), "ephemeral driver has no store");
    assert_eq!(out.ct_execute, 1, "exactly one real AV call");
    assert_eq!(out.meta_query, 0);
    assert_eq!(
        out.values.get("main.ct").map(|v| v.render()),
        Some("i32:42".to_string()),
        "real AV result"
    );
    assert_eq!(
        out.consumer_types.get("main.ct").map(String::as_str),
        Some("i32"),
        "796 inference derives i32 after substitution"
    );
    assert_eq!(
        out.hir_proof.get("main.ct").map(String::as_str),
        Some("42"),
        "backfilled HIR carries Constant{{42}}"
    );
    assert_eq!(out.native_exit, Some(42), "native observable (independent)");
    assert!(out.peak_private_bytes.unwrap_or(0) > 0, "peak sampled");
    assert!(out.spans.contains_key("av_execute"), "trace spans present");
}

#[test]
fn cte_first_bool_branch_flips_native_exit() {
    let tmp = tempfile::tempdir().unwrap();
    let mut d = CteDriver::ephemeral();
    let t = d.query_step(&bool_deck(9), Some(tmp.path()));
    assert_eq!(t.status, StepStatus::Ok, "{:?}", t.message);
    assert_eq!(
        t.values.get("main.gate").map(|v| v.render()),
        Some("bool:true".into())
    );
    assert_eq!(
        t.consumer_types.get("main.gate").map(String::as_str),
        Some("bool")
    );
    assert_eq!(
        t.hir_proof.get("main.gate").map(String::as_str),
        Some("0<1")
    );
    assert_eq!(
        t.native_exit,
        Some(1),
        "true branch taken (was 2 before backfill)"
    );

    let f = d.query_step(&bool_deck(99), Some(tmp.path()));
    assert_eq!(f.status, StepStatus::Ok, "{:?}", f.message);
    assert_eq!(
        f.values.get("main.gate").map(|v| v.render()),
        Some("bool:false".into())
    );
    assert_eq!(
        f.hir_proof.get("main.gate").map(String::as_str),
        Some("0<0")
    );
    assert_eq!(
        f.native_exit,
        Some(2),
        "false branch keeps the original arm"
    );
}

#[test]
fn cte_first_store_hit_skips_av_same_process() {
    let tmp = tempfile::tempdir().unwrap();
    let store = tmp.path().join("store");
    let mut d = CteDriver::with_store(&store, "proj-a").unwrap();
    let first = d.query_step(SUM42_DECK, None);
    assert_eq!(first.status, StepStatus::Ok, "{:?}", first.message);
    assert_eq!(first.cache, Some("miss"));
    assert_eq!(first.ct_execute, 1);

    let second = d.query_step(SUM42_DECK, None);
    assert_eq!(second.status, StepStatus::Ok, "{:?}", second.message);
    assert_eq!(second.cache, Some("hit"), "frozen layer hit");
    assert_eq!(second.ct_execute, 0, "a hit never executes the AV");
    assert_eq!(second.ct_execute, 0);
    // Consumer work is redone on EVERY hit (never cached PASS):
    assert_eq!(
        second.hir_proof.get("main.ct").map(String::as_str),
        Some("42")
    );
    assert_eq!(
        second.consumer_types.get("main.ct").map(String::as_str),
        Some("i32")
    );
    // and the hit skipped the av_execute span entirely.
    assert!(!second.spans.contains_key("av_execute"));
}

#[test]
fn cte_first_store_restore_new_process() {
    let tmp = tempfile::tempdir().unwrap();
    let store = tmp.path().join("store");
    // Worker #1: the miss that publishes.
    let r1 = call_worker(&store, SUM42_DECK);
    let s1 = &r1["ok"]["steps"][0];
    assert_eq!(s1["cache"], "miss", "worker1: {:?}", r1);
    assert_eq!(s1["ct_execute"], 1);
    assert_eq!(s1["exit"], 42);

    // Worker #2 — a REAL new process — restores from disk and hits.
    let r2 = call_worker(&store, SUM42_DECK);
    let s2 = &r2["ok"]["steps"][0];
    assert_eq!(s2["cache"], "hit", "worker2: {:?}", r2);
    assert_eq!(s2["ct_execute"], 0, "restored hit executes no AV");
    assert_eq!(s2["exit"], 42, "consumer re-derived in the new process");
}

#[test]
fn cte_first_av_forbidden_counter_control() {
    let tmp = tempfile::tempdir().unwrap();
    let store = tmp.path().join("store");
    let _work = tmp.path().join("native");

    // With AV forbidden a miss CANNOT complete — controlled infra error.
    let mut d = CteDriver::with_store(&store, "cc").unwrap();
    d.set_forbid_av(true);
    let blocked = d.query_step(SUM42_DECK, None);
    assert_eq!(blocked.status, StepStatus::Diag);
    assert!(
        !blocked.language_class,
        "resource/infra class, not a language verdict"
    );
    assert_eq!(blocked.code.as_deref(), Some("cte.infra"));
    assert_eq!(blocked.ct_execute, 0);

    // Populate normally, then forbid again: the HIT still completes.
    let mut d2 = CteDriver::with_store(&store, "cc").unwrap();
    let miss = d2.query_step(SUM42_DECK, None);
    assert_eq!(miss.status, StepStatus::Ok, "{:?}", miss.message);
    assert_eq!(miss.cache, Some("miss"));
    let mut d3 = CteDriver::with_store(&store, "cc").unwrap();
    d3.set_forbid_av(true);
    let hit = d3.query_step(SUM42_DECK, None);
    assert_eq!(
        hit.status,
        StepStatus::Ok,
        "hit under AV-forbid: {:?}",
        hit.message
    );
    assert_eq!(hit.cache, Some("hit"));
    assert_eq!(hit.ct_execute, 0);
    assert_eq!(hit.hir_proof.get("main.ct").map(String::as_str), Some("42"));
}

#[test]
fn cte_first_cross_project_hit() {
    let tmp = tempfile::tempdir().unwrap();
    let store = tmp.path().join("store");
    let mut a = CteDriver::with_store(&store, "proj-a").unwrap();
    let mut b = CteDriver::with_store(&store, "proj-b").unwrap();
    let first = a.query_step(SUM42_DECK, None);
    assert_eq!(first.cache, Some("miss"));
    let second = b.query_step(SUM42_DECK, None);
    assert_eq!(
        second.status,
        StepStatus::Ok,
        "another project shares the frozen value: {:?}",
        second.message
    );
    assert_eq!(
        second.cache,
        Some("hit"),
        "content identity, not project identity"
    );
    assert_eq!(second.ct_execute, 0);
}

// ---------------------------------------------------------------------------
// helpers
// ---------------------------------------------------------------------------

/// Spawn the REAL worker binary (`cte_probe_worker case-serve`) in a
/// fresh process — the cross-process restore path.
fn call_worker(store: &Path, deck: &str) -> serde_json::Value {
    use base64::Engine as _;
    use std::io::Write;
    use std::process::{Command, Stdio};
    let exe = env!("CARGO_BIN_EXE_cte_probe_worker");
    let req = serde_json::json!({
        "recipe": "cte/scalar-query-v1",
        "nonce": "n1",
        "deck_b64": base64::engine::general_purpose::STANDARD.encode(deck),
        "store_root": store.display().to_string().replace('/', "\\"),
        "session": "restore-test",
        "steps": [ { "op": "query" } ],
        "request_hash": "h1",
    });
    let mut child = Command::new(exe)
        .arg("case-serve")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn cte_probe_worker");
    child
        .stdin
        .take()
        .unwrap()
        .write_all(req.to_string().as_bytes())
        .unwrap();
    let out = child
        .wait_with_output()
        .expect("worker completes (bounded by test harness)");
    assert!(out.status.success(), "worker exit: {:?}", out.status);
    let line = String::from_utf8_lossy(&out.stdout);
    let v: serde_json::Value = serde_json::from_str(line.trim()).expect("one JSON line");
    assert_eq!(v["schema"], "cteworker/1");
    assert_eq!(v["nonce"], "n1");
    assert_eq!(v["request_hash"], "h1");
    v
}

/// Suppress the unused-warning for Duration (used by future turnaround
/// helpers kept adjacent to this family).
#[allow(dead_code)]
fn _keep(_d: Duration, _m: BTreeMap<String, String>) {}

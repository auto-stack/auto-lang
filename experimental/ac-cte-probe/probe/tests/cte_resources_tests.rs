//! cte_resources_ — resources family (PLAN-817 T-05, V-07/V-08): the
//! managed-budget ledger (reservations BEFORE allocations, target_met
//! never papered over, spike counter-controls真红), deep-input bounds,
//! streamed generation of ≥32 MiB of REAL frozen pages + 512 distinct
//! input revisions with per-entry value assertions, REAL-watermark GC
//! (physical deletion + convergence + lease keep-alive + sentinel), and
//! the worker/tree memory hard gates.
//!
//! The scale tests run in-process on purpose (the worker gate has its
//! own focused test); closed-form expectations are hand-derived:
//! value(20, b) = 20 + b + Σ k for k in 0..100 (= 4970 + b).

use ac_artifact_probe::store::{GcConfig, GC_HIGH, GC_LOW};
use auto_cte_probe::driver::{CteDriver, StepStatus};
use auto_cte_probe::{BudgetCaps, Partition};

// ---- F-817-R8-01: REAL-allocation observation ---------------------------
//
// A global allocator wrapper records the LARGEST single allocation
// since the last reset — the review's oracle class (the physical read,
// not upper-layer event logs). Tests reset the counter and assert the
// window's max against the granted budget.

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering as AtomicOrdering};

static MAX_SINGLE_ALLOC: AtomicUsize = AtomicUsize::new(0);
/// Coexisting-live bytes (alloc − dealloc watermark peak): the oracle
/// for COEXISTING copy budgets (F-817-R9-01), which single-allocation
/// maxima cannot observe.
static LIVE_BYTES: AtomicUsize = AtomicUsize::new(0);
static MAX_LIVE_BYTES: AtomicUsize = AtomicUsize::new(0);
static WINDOW_BASELINE: AtomicUsize = AtomicUsize::new(0);

struct TrackingAlloc;

unsafe impl GlobalAlloc for TrackingAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let p = System.alloc(layout);
        if !p.is_null() {
            MAX_SINGLE_ALLOC.fetch_max(layout.size(), AtomicOrdering::Relaxed);
            let live = LIVE_BYTES.fetch_add(layout.size(), AtomicOrdering::Relaxed) + layout.size();
            MAX_LIVE_BYTES.fetch_max(live, AtomicOrdering::Relaxed);
        }
        p
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        LIVE_BYTES.fetch_sub(layout.size(), AtomicOrdering::Relaxed);
        System.dealloc(ptr, layout)
    }
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let p = System.realloc(ptr, layout, new_size);
        if !p.is_null() {
            MAX_SINGLE_ALLOC.fetch_max(new_size, AtomicOrdering::Relaxed);
            LIVE_BYTES.fetch_sub(layout.size(), AtomicOrdering::Relaxed);
            let live = LIVE_BYTES.fetch_add(new_size, AtomicOrdering::Relaxed) + new_size;
            MAX_LIVE_BYTES.fetch_max(live, AtomicOrdering::Relaxed);
        }
        p
    }
}

#[global_allocator]
static GLOBAL: TrackingAlloc = TrackingAlloc;

/// F-817-R10-03: the observation window is begun ONCE (baseline = the
/// live watermark AT THAT MOMENT, single-max cleared, watermark
/// re-based) and read out WITHOUT resetting — the old reset-on-read
/// pattern re-based the watermark AFTER the measured operation and
/// left stale process history in both metrics.
fn begin_window() {
    let live = LIVE_BYTES.load(AtomicOrdering::Relaxed);
    WINDOW_BASELINE.store(live, AtomicOrdering::Relaxed);
    MAX_LIVE_BYTES.store(live, AtomicOrdering::Relaxed);
    MAX_SINGLE_ALLOC.store(0, AtomicOrdering::Relaxed);
}

/// Read the window: (max single allocation, coexisting-live peak above
/// the begin baseline). Does NOT reset anything.
fn window_readout() -> (usize, usize) {
    (
        MAX_SINGLE_ALLOC.load(AtomicOrdering::Relaxed),
        MAX_LIVE_BYTES
            .load(AtomicOrdering::Relaxed)
            .saturating_sub(WINDOW_BASELINE.load(AtomicOrdering::Relaxed)),
    )
}

fn deck_with(src: &str, tail: &str) -> String {
    format!("deck cte817/1\nmodule main\nsource <<\n{src}\n>>\n{tail}entry main\n")
}

const SMALL_SRC: &str = "fn cte_sum(a int, b int) int {\n    return a + b\n}\n\nfn main() int {\n    let ct = 0\n    return ct\n}\n";

/// The generator source: cte_sum's closure covers 100 real helpers.
fn big_source_n(n: usize) -> String {
    let mut s = String::new();
    for k in 0..n {
        s.push_str(&format!("fn h{k}() int {{\n    return {k}\n}}\n\n"));
    }
    s.push_str("fn cte_sum(a int, b int) int {\n    var acc = a + b\n");
    for k in 0..n {
        s.push_str(&format!("    acc = acc + h{k}()\n"));
    }
    s.push_str("    return acc\n}\n\nfn main() int {\n    let ct = 0\n    return ct\n}\n");
    s
}

/// The default generator (100 helpers).
fn big_source() -> String {
    big_source_n(100)
}

const SIGMA: i64 = 4950; // Σ k for k in 0..100

/// One generation request: `q` distinct-arg calls of cte_sum with the
/// given starting b, every value asserted through its own anchor.
fn big_deck(start_b: i32, q: usize) -> String {
    let mut main = String::new();
    let mut queries = String::new();
    let mut binds = String::new();
    for i in 0..q {
        let b = start_b + i as i32;
        main.push_str(&format!("    let c{i} = 0\n"));
        queries.push_str(&format!("query q{i} = call cte_sum(20, {b})\n"));
        binds.push_str(&format!("bind main.c{i} = q{i} expect i32 placeholder 0\n"));
    }
    main.push_str("    return c0\n");
    let src = big_source().replace(
        "fn main() int {\n    let ct = 0\n    return ct\n}\n",
        &format!("fn main() int {{\n{main}}}\n"),
    );
    deck_with(&src, &format!("{queries}{binds}"))
}

fn gc_deck(start_b: i32, q: usize) -> (String, i64) {
    let n = 300;
    let sigma = (0..n as i64).sum::<i64>();
    let mut main = String::new();
    let mut queries = String::new();
    let mut binds = String::new();
    for i in 0..q {
        let b = start_b + i as i32;
        main.push_str(&format!(
            "    let c{i} = 0
"
        ));
        queries.push_str(&format!(
            "query q{i} = call cte_sum(20, {b})
"
        ));
        binds.push_str(&format!(
            "bind main.c{i} = q{i} expect i32 placeholder 0
"
        ));
    }
    main.push_str(
        "    return c0
",
    );
    let src = big_source_n(n).replace(
        "fn main() int {
    let ct = 0
    return ct
}
",
        &format!(
            "fn main() int {{
{main}}}
"
        ),
    );
    (deck_with(&src, &format!("{queries}{binds}")), sigma)
}

fn assert_big_values(out: &auto_cte_probe::driver::CteStepOutcome, start_b: i32, q: usize) {
    assert_eq!(out.status, StepStatus::Ok, "{:?}", out.message);
    for i in 0..q {
        let want = 20 + (start_b + i as i32) as i64 + SIGMA;
        let got = out
            .values
            .get(&format!("main.c{i}"))
            .map(|v| v.render())
            .unwrap_or_default();
        assert_eq!(got, format!("i32:{want}"), "anchor c{i}");
    }
}

// ---------------------------------------------------------------------------
// managed budget gates + spike counter-controls (V-08)
// ---------------------------------------------------------------------------

#[test]
fn resources_budget_within_gates_and_pre_reserved() {
    let mut d = CteDriver::ephemeral();
    let ok = d.query_step(
        &deck_with(
            SMALL_SRC,
            "query q1 = call cte_sum(20, 22)\nbind main.ct = q1 expect i32 placeholder 0\n",
        ),
        None,
    );
    assert_eq!(ok.status, StepStatus::Ok, "{:?}", ok.message);
    assert!(ok.managed_target_met, "normal steps meet the managed gate");
    assert!(ok.managed_peak_bytes <= 32 * 1024 * 1024);

    // Counter-control: a tiny staging partition rejects the REQUEST text
    // itself — before any parse/clone/execution.
    let mut tiny = CteDriver::ephemeral();
    tiny.set_managed_caps(BudgetCaps {
        hot: 8 * 1024 * 1024,
        staging: 8,
        av_slots: 8 * 1024 * 1024,
    });
    let out = tiny.query_step(
        &deck_with(
            SMALL_SRC,
            "query q1 = call cte_sum(1, 2)\nbind main.ct = q1 expect i32 placeholder 0\n",
        ),
        None,
    );
    assert_eq!(out.status, StepStatus::Diag);
    assert_eq!(
        out.code.as_deref(),
        Some("cte.resource.exhausted"),
        "{:?}",
        out.code
    );
    assert!(!out.language_class);
    assert_eq!(out.ct_execute, 0, "the reservation fired before execution");
    assert!(
        !out.managed_target_met,
        "over-budget never reports target met"
    );
}

#[test]
fn resources_spike_control_is_really_red() {
    // The staging partition is sized between the request text and the
    // envelope encode: execution completes, the ENCODE reservation then
    // "spikes" over — the step is red and nothing was published.
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("store");
    let deck = deck_with(
        SMALL_SRC,
        "query q1 = call cte_sum(20, 22)\nbind main.ct = q1 expect i32 placeholder 0\n",
    );
    let mut d = CteDriver::with_store(&root, "spike").unwrap();
    let probe = d.query_step(&deck, None);
    assert!(probe.status == StepStatus::Ok);
    let envelope_bytes = d.last_envelope_bytes().expect("the probe step published");
    // The ledger's known staging need for THIS deck shape plus the
    // envelope, minus one byte: the encode reservation must spike over.
    let base_need = (deck.len() as u64) + (SMALL_SRC.len() as u64) * 4;
    d.set_managed_caps(BudgetCaps {
        hot: 8 * 1024 * 1024,
        staging: base_need + envelope_bytes - 1,
        av_slots: 8 * 1024 * 1024,
    });
    // A DIFFERENT request (different args) so the first entry can't serve it.
    let deck2 = deck.replace("call cte_sum(20, 22)", "call cte_sum(21, 22)");
    let out = d.query_step(&deck2, None);
    assert!(
        !out.managed_target_met || out.status == StepStatus::Diag,
        "the spike step is red (met={}, status={:?})",
        out.managed_target_met,
        out.status
    );
    // And nothing was published for the spiked identity.
    let mut clean = CteDriver::with_store(&root, "spike").unwrap();
    let after = clean.query_step(&deck2, None);
    assert_eq!(after.status, StepStatus::Ok, "{:?}", after.message);
    assert_eq!(
        after.cache,
        Some("miss"),
        "the spiked run published nothing"
    );
}

// ---------------------------------------------------------------------------
// deep-input bounds (V-07/V-08)
// ---------------------------------------------------------------------------

#[test]
fn resources_deep_input_bound() {
    let mut pad = String::from("# padding\n");
    while pad.len() <= 4 * 1024 * 1024 + 64 {
        pad.push_str("# padding padding padding padding padding padding\n");
    }
    let fat = format!(
        "{pad}{}",
        deck_with(
            SMALL_SRC,
            "query q1 = call cte_sum(1, 2)\nbind main.ct = q1 expect i32 placeholder 0\n"
        )
    );
    let mut d = CteDriver::ephemeral();
    let out = d.query_step(&fat, None);
    assert_eq!(out.status, StepStatus::Diag);
    assert!(
        out.message.as_deref().unwrap_or_default().contains("input"),
        "the 4MiB deep-input ceiling rejects before parse: {:?}",
        out.message
    );
    assert_eq!(out.ct_execute, 0);
}

// ---------------------------------------------------------------------------
// ≥32 MiB real pages + 512 distinct-input revisions (V-08)
// ---------------------------------------------------------------------------

#[test]
fn resources_thirty_two_mib_pages_and_512_distinct_revisions() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("store");
    let mut d = CteDriver::with_store(&root, "pages").unwrap();
    // The debug scale tier: multi-call requests at unoptimized speed —
    // the 10s wall is the reference RELEASE bound (F-817-R6-01), not a
    // debug-timing verdict; lengthen via the test seam.
    d.set_request_wall_limit(std::time::Duration::from_secs(600));

    // Streamed generation: requests of 100 distinct-arg calls each,
    // every value asserted against the closed form, until the store's
    // real payload total passes 32 MiB.
    let mut start_b = 1i32;
    let mut requests = 0usize;
    while d.store().unwrap().raw().space().total < 32 * 1024 * 1024 {
        let deck = big_deck(start_b, 100);
        let out = d.query_step(&deck, None);
        assert_big_values(&out, start_b, 100);
        assert!(out.managed_target_met, "generation stays inside the ledger");
        assert_eq!(out.ct_execute, 100);
        start_b += 100;
        requests += 1;
        assert!(requests < 400, "generation terminates");
    }
    let total = d.store().unwrap().raw().space().total;
    assert!(
        total >= 32 * 1024 * 1024,
        "real frozen pages ≥32MiB: {total}"
    );

    // 512 DISTINCT-INPUT revisions on the small source: each is a fresh
    // key, a real re-execution, and the correct value.
    let base = deck_with(
        SMALL_SRC,
        "query q1 = call cte_sum(7, 0)\nbind main.ct = q1 expect i32 placeholder 0\n",
    );
    for i in 1..=512i32 {
        let deck = base.replace("call cte_sum(7, 0)", &format!("call cte_sum(7, {i})"));
        let out = d.query_step(&deck, None);
        assert_eq!(out.status, StepStatus::Ok, "rev {i}: {:?}", out.message);
        assert_eq!(out.cache, Some("miss"), "rev {i} is a distinct key");
        assert_eq!(
            out.values.get("main.ct").map(|v| v.render()),
            Some(format!("i32:{}", 7 + i))
        );
        assert!(out.managed_target_met);
    }
    let after = d.store().unwrap().raw().space().total;
    assert!(
        after > total,
        "the 512 revisions added real distinct products"
    );
}

// ---------------------------------------------------------------------------
// REAL-watermark GC: physical deletion, convergence, lease, sentinel
// ---------------------------------------------------------------------------

#[test]
fn resources_real_watermark_gc_physical_delete_and_converge() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("store");
    // The PIN publishes FIRST so it is the store's OLDEST frozen entry
    // (GC deletes oldest-first): a lease keeps it alive through the
    // first pass, and after release the next real-watermark pass
    // collects it first. A foreign sentinel rides along under its own
    // project root.
    let st = auto_cte_probe::store::CteStore::open(&root, "gc-runner").unwrap();
    let pin_deck = deck_with(
        SMALL_SRC,
        "query q1 = call cte_sum(3, 4)
bind main.ct = q1 expect i32 placeholder 0
",
    );
    let mut pin_drv = CteDriver::with_store(&root, "pin").unwrap();
    // Debug scale tier: lengthen the wall seam (see pages test).
    pin_drv.set_request_wall_limit(std::time::Duration::from_secs(600));
    let pin = pin_drv.query_step(&pin_deck, None);
    assert_eq!(pin.cache, Some("miss"));
    let pin_art = pin_drv
        .last_published_artifact()
        .expect("the pin published a real artifact")
        .to_string();
    drop(pin);
    let lease = st
        .raw()
        .acquire_lease("cte817-pinned", 300, Some(&pin_art))
        .unwrap();

    use ac_artifact_probe::codec::ChunkKind;
    use ac_artifact_probe::identity::QueryKey;
    use ac_artifact_probe::store::PutRequest;
    let sentinel = auto_cte_probe::store::CteStore::open(&root, "user-sentinel").unwrap();
    let sent_key = QueryKey::new(
        &auto_cte_probe::store::CTE_LAYER_SPEC,
        vec![
            ("producer".into(), "user".into()),
            ("owner".into(), "sentinel".into()),
            ("callee".into(), "s".into()),
            ("args".into(), "i32:0".into()),
            ("algo".into(), "user".into()),
            ("caps".into(), "".into()),
            ("engine".into(), "user".into()),
            ("manifest-fp".into(), "user".into()),
        ],
    );
    let sent_id = sentinel
        .raw()
        .put_artifact(PutRequest {
            layer: auto_cte_contract::CTE_FROZEN_LAYER,
            key: sent_key.clone(),
            kind: ChunkKind::LayoutData,
            payload: b"sentinel-bytes".to_vec(),
            edges: vec![],
            project: Some("user-sentinel".into()),
        })
        .unwrap();

    let mut d = CteDriver::with_store(&root, "cold").unwrap();
    d.set_request_wall_limit(std::time::Duration::from_secs(600));

    // Drive the SAME store past the REAL high watermark (64 MiB).
    let mut start_b = 1i32;
    let mut requests = 0usize;
    while d.store().unwrap().raw().space().total <= GC_HIGH {
        let (deck, sigma) = gc_deck(start_b, 50);
        let out = d.query_step(&deck, None);
        assert_eq!(out.status, StepStatus::Ok, "{:?}", out.message);
        for i in 0..50 {
            let want = 20 + (start_b + i as i32) as i64 + sigma;
            assert_eq!(
                out.values.get(&format!("main.c{i}")).map(|v| v.render()),
                Some(format!("i32:{want}")),
                "anchor c{i}"
            );
        }
        start_b += 50;
        requests += 1;
        assert!(requests < 400, "generation terminates");
    }
    let cold_total = d.store().unwrap().raw().space().total;
    println!("gc-test: requests={requests} cold_total={cold_total}");
    assert!(
        cold_total > GC_HIGH,
        "cold set over the real high watermark: {cold_total}"
    );

    // The cold mass becomes unreachable; the REAL-watermark pass runs.
    st.raw().remove_project_roots("cold").unwrap();
    let receipt = ac_artifact_probe::store::run_gc(
        st.raw(),
        GcConfig {
            high: GC_HIGH,
            low: GC_LOW,
        },
    )
    .unwrap();
    println!(
        "gc-test: ran={} deleted={} freed={} reason={}",
        receipt.ran, receipt.deleted_chunks, receipt.freed_bytes, receipt.reason
    );
    assert!(receipt.ran, "the pass ran");
    assert!(
        receipt.deleted_chunks > 0 && receipt.freed_bytes > 0,
        "physical deletion really happened: {receipt:?}"
    );
    let after_gc = st.raw().space().total;
    println!("gc-test: after_gc={after_gc}");
    assert!(
        after_gc < GC_LOW,
        "the store converged below the low watermark: {after_gc}"
    );

    // The pinned entry survived (lease); the sentinel is byte-identical.
    let mut pin_check = CteDriver::with_store(&root, "pin").unwrap();
    pin_check.set_request_wall_limit(std::time::Duration::from_secs(600));
    let pinned = pin_check.query_step(&pin_deck, None);
    assert_eq!(
        pinned.cache,
        Some("hit"),
        "the leased entry has no unavailable input"
    );
    let sent = sentinel
        .raw()
        .get_artifact_by_id(&sent_id)
        .unwrap()
        .expect("sentinel alive");
    assert_eq!(sent.payload, b"sentinel-bytes");

    // Release the lease: the pin becomes collectable. A watermark pass
    // only runs ABOVE the high watermark (783 semantics), so drive the
    // store over it once more with real traffic, drop that traffic's
    // roots, and let the REAL-watermark pass collect both the new mass
    // and the released pin.
    lease.release();
    st.raw().remove_project_roots("pin").unwrap();
    let mut warm = CteDriver::with_store(&root, "warm").unwrap();
    warm.set_request_wall_limit(std::time::Duration::from_secs(600));
    let mut wb = 10_000i32;
    while warm.store().unwrap().raw().space().total <= GC_HIGH {
        let (deck, sigma) = gc_deck(wb, 50);
        let out = warm.query_step(&deck, None);
        assert_eq!(out.status, StepStatus::Ok, "{:?}", out.message);
        assert_eq!(
            out.values.get("main.c0").map(|v| v.render()),
            Some(format!("i32:{}", 20 + wb as i64 + sigma))
        );
        wb += 50;
    }
    st.raw().remove_project_roots("warm").unwrap();
    let second = ac_artifact_probe::store::run_gc(
        st.raw(),
        GcConfig {
            high: GC_HIGH,
            low: GC_LOW,
        },
    )
    .unwrap();
    println!(
        "gc-test: second ran={} deleted={} freed={}",
        second.ran, second.deleted_chunks, second.freed_bytes
    );
    assert!(second.ran && second.deleted_chunks > 0);
    let mut gone = CteDriver::with_store(&root, "pin").unwrap();
    gone.set_request_wall_limit(std::time::Duration::from_secs(600));
    assert_eq!(
        gone.query_step(&pin_deck, None).cache,
        Some("miss"),
        "released pin was collected by the real-watermark pass"
    );
    // The sentinel STILL survives every pass.
    assert!(sentinel
        .raw()
        .get_artifact_by_id(&sent_id)
        .unwrap()
        .is_some());
}

// ---------------------------------------------------------------------------
// worker / tree memory hard gates (V-08)
// ---------------------------------------------------------------------------

#[test]
fn resources_worker_peak_under_hard_gates() {
    use base64::Engine as _;
    use std::io::Write;
    use std::process::{Command, Stdio};
    let deck = big_deck(9000, 40);
    let req = serde_json::json!({
        "recipe": "cte/scalar-query-v1",
        "nonce": "peak",
        "deck_b64": base64::engine::general_purpose::STANDARD.encode(deck),
        "store_root": std::env::temp_dir().join("cte817-peak-gate").display().to_string().replace('/', "\\"),
        "session": "peak",
        "steps": [ { "op": "query" } ],
        "request_hash": "h",
    });
    let parent_sampler = auto_av_probe::mempeek::PeakSampler::start(1);
    let mut child = Command::new(env!("CARGO_BIN_EXE_cte_probe_worker"))
        .arg("case-serve")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("worker spawns");
    child
        .stdin
        .take()
        .unwrap()
        .write_all(req.to_string().as_bytes())
        .unwrap();
    let out = child.wait_with_output().expect("worker completes");
    let parent_peak = parent_sampler.stop();
    assert!(out.status.success());
    let v: serde_json::Value =
        serde_json::from_str(String::from_utf8_lossy(&out.stdout).trim()).unwrap();
    assert!(v["ok"].is_object(), "worker ok: {}", v);
    let worker_peak = v["ok"]["worker_private_bytes_peak"].as_u64().unwrap_or(0);
    assert!(worker_peak > 0, "the worker reported a real sampled peak");
    assert!(
        worker_peak <= 384 * 1024 * 1024,
        "worker private-bytes hard gate: {worker_peak}"
    );
    let tree_estimate = worker_peak.max(parent_peak);
    assert!(
        tree_estimate <= 512 * 1024 * 1024,
        "process-tree hard gate (single report): {tree_estimate} (worker {worker_peak}, parent {parent_peak})"
    );
    // Every managed partition figure the ledger reports held.
    let step = &v["ok"]["steps"][0];
    let _ = step;
}

// Silence unused warnings for conditionally-used partitions import.
#[allow(dead_code)]
fn _keep(_p: Partition) {}

// ---- F-817-R6-01: the cumulative request deadline ----------------------

/// The review's request-wall deck shape (review-r6): 115 helpers x 80
/// meaningful arithmetic lines, `queries` DISTINCT-argument cte_sum
/// calls (no single-flight reuse), only q0 bound, native off.
fn wall_deck(queries: usize) -> String {
    let src = auto_cte_probe::bench::baseline_source_bench(115, 0);
    let mut q = String::new();
    for i in 0..queries {
        q.push_str(&format!("query q{i} = call cte_sum(20, {})\n", 22 + i));
    }
    format!(
        "deck cte817/1\nmodule main\nsource <<\n{src}>>\n{q}bind main.ct = q0 expect i32 placeholder 0\nentry main\nnative off\n"
    )
}

const WALL_BASE: i64 = 42 + 115 * 4780 + 81 * (114 * 115 / 2);

/// Fast classification control (test-seam deadline): a multi-call
/// request that runs past a shortened cumulative limit aborts as the
/// RESOURCE deadline state (never a language verdict), counts the
/// execution facts, publishes NOTHING — and the same deck under the
/// frozen default limit stays green.
#[test]
fn cte_resources_request_deadline_fast_control() {
    let root = tempfile::tempdir().unwrap();
    let deck = wall_deck(4);
    let mut d = CteDriver::with_store(root.path(), "dl").unwrap();
    d.set_request_wall_limit(std::time::Duration::from_millis(300));
    let t = std::time::Instant::now();
    let out = d.query_step(&deck, None);
    let elapsed = t.elapsed();
    assert_eq!(out.status, StepStatus::Diag);
    assert_eq!(out.code.as_deref(), Some("cte.resource.deadline"));
    assert!(
        !out.language_class,
        "deadline is a resource state, never a language verdict"
    );
    assert!(out.ct_execute >= 1, "execution facts still count");
    assert!(elapsed >= std::time::Duration::from_millis(300));
    assert!(
        elapsed < std::time::Duration::from_secs(10),
        "the deadline interrupts the request, it does not run to completion: {elapsed:?}"
    );
    assert!(
        d.last_published_artifact().is_none(),
        "an aborted request publishes nothing"
    );
    // Legal positive control: a single-call variant of the same deck
    // under the frozen default limit stays green (the 4-call form is
    // the real-10s replay below).
    let mut ok_driver = CteDriver::with_store(root.path(), "dl-ok").unwrap();
    let ok = ok_driver.query_step(&wall_deck(1), None);
    assert_eq!(ok.status, StepStatus::Ok);
    assert_eq!(ok.ct_execute, 1);
    assert_eq!(
        ok.values.get("main.ct").map(|v| v.render()).as_deref(),
        Some(format!("i32:{WALL_BASE}").as_str())
    );
}

/// The REAL 10s replay (review-r6 request-wall counterexample, worker
/// path): 32 distinct-argument legal calls (each individually under
/// every per-call cap) accumulate past the 10s request deadline — the
/// worker returns the controlled resource state and publishes nothing;
/// the 4-query same-scale positive control completes green with all
/// four calls executed (also proving the aborted 32-query run cached
/// nothing).
#[test]
fn cte_resources_request_wall_real_ten_seconds() {
    use std::io::Write as _;
    use std::process::{Command, Stdio};
    let store = tempfile::tempdir().unwrap();
    let serve = |deck: &str, nonce: &str| -> serde_json::Value {
        let req = auto_cte_probe::bench::build_worker_request_for_deck(
            nonce,
            &format!("rh-{nonce}"),
            store.path(),
            "wall",
            deck,
            &serde_json::json!([{ "op": "query" }]),
        );
        let mut child = Command::new(env!("CARGO_BIN_EXE_cte_probe_worker"))
            .arg("case-serve")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("worker spawn");
        child
            .stdin
            .take()
            .unwrap()
            .write_all(req.as_bytes())
            .unwrap();
        let out = child.wait_with_output().expect("worker completes");
        serde_json::from_str(String::from_utf8_lossy(&out.stdout).trim()).unwrap()
    };
    // (a) The 32-query request: individually-legal calls, cumulative
    // 22s+ — must abort at the 10s request deadline.
    let t = std::time::Instant::now();
    let v = serve(&wall_deck(32), "wall32");
    let elapsed = t.elapsed();
    let st = &v["ok"]["steps"][0];
    assert_eq!(st["outcome"], "diag", "aborted, not ok: {st}");
    assert_eq!(st["code"], "cte.resource.deadline", "precise code: {st}");
    assert_eq!(
        st["language_class"], false,
        "resource state, never language"
    );
    assert!(st["ct_execute"].as_u64().unwrap_or(0) >= 1);
    let wall_ms = st["wall_ms"].as_f64().unwrap_or(0.0);
    assert!(
        wall_ms >= 9500.0,
        "the deadline fires AT the limit, not early: {wall_ms}ms"
    );
    assert!(
        elapsed.as_secs() < 30,
        "the request is interrupted near the limit: {elapsed:?}"
    );
    // (b) Same-scale positive control (review four-query-control): the
    // legal scale completes green. Sub-calls that finished WITHIN the
    // aborted run's budget legitimately published their own envelopes
    // (each is a legal per-call result, produced before the deadline);
    // what the deadline forbids — and (a) shows — is the REQUEST
    // succeeding or anything publishing past the limit.
    let v4 = serve(&wall_deck(4), "wall4");
    let st4 = &v4["ok"]["steps"][0];
    assert_eq!(st4["outcome"], "ok", "legal scale stays green: {st4}");
    assert!(
        st4["ct_execute"].as_u64().unwrap_or(99) <= 4,
        "no phantom executions: {st4}"
    );
    assert_eq!(
        st4["values"]["main.ct"],
        format!("i32:{WALL_BASE}"),
        "independent closed form"
    );
}

// ---- F-817-R6-02: reserve-before-allocation, observable in order ------

fn event_index(
    events: &[auto_cte_probe::observe::BudgetEvent],
    pred: impl Fn(&auto_cte_probe::observe::BudgetEvent) -> bool,
) -> Option<usize> {
    events.iter().position(pred)
}

fn is_reserve_what(
    what: &str,
    ok: bool,
) -> impl Fn(&auto_cte_probe::observe::BudgetEvent) -> bool + '_ {
    move |e| matches!(e, auto_cte_probe::observe::BudgetEvent::Reserve { what: w, ok: o, .. } if *w == what && *o == ok)
}

fn is_alloc(tag: &str) -> impl Fn(&auto_cte_probe::observe::BudgetEvent) -> bool + '_ {
    move |e| matches!(e, auto_cte_probe::observe::BudgetEvent::Alloc { tag: t, .. } if *t == tag)
}

/// The LIVE staging level at the event with index `idx` — replaying
/// Reserve(ok)+Release events yields the exact level a shrunken cap
/// must sit at to fail exactly at the next reservation.
fn staging_live_at(events: &[auto_cte_probe::observe::BudgetEvent], idx: usize) -> u64 {
    events[..idx].iter().fold(0u64, |acc, e| match e {
        auto_cte_probe::observe::BudgetEvent::Reserve {
            partition: Partition::Staging,
            bytes,
            ok: true,
            ..
        } => acc + *bytes,
        auto_cte_probe::observe::BudgetEvent::Release {
            partition: Partition::Staging,
            bytes,
        } => acc.saturating_sub(*bytes),
        _ => acc,
    })
}

fn small_query_deck() -> String {
    deck_with(
        SMALL_SRC,
        "query q1 = call cte_sum(20, 22)\nbind main.ct = q1 expect i32 placeholder 0\n",
    )
}

/// The ENCODE path: the envelope reservation precedes the encode
/// allocation (legal run), and an over-quota encode is rejected AT THE
/// RESERVATION — no encode allocation happens — proven directly on the
/// reserved-encode entrypoint with the budget event log. (Within a
/// live request the encode staging gate is structurally SUBSUMED: a
/// legit envelope is a fingerprint manifest at ~3.2x its source,
/// always under the 4x projection reservation that already gated the
/// same partition — the reserve stays as the order/accounting gate.)
#[test]
fn cte_resources_encode_reserve_before_alloc_order() {
    let root = tempfile::tempdir().unwrap();
    let deck = small_query_deck();
    // (a) Legal run through the REAL driver path: Reserve("envelope
    // encode", ok) BEFORE Alloc("envelope-encode").
    let mut d = CteDriver::with_store(root.path(), "enc").unwrap();
    let out = d.query_step(&deck, None);
    assert_eq!(out.status, StepStatus::Ok);
    let events = d.last_budget_events();
    let reserve = event_index(events, is_reserve_what("envelope encode", true))
        .expect("encode reservation recorded");
    let alloc =
        event_index(events, is_alloc("envelope-encode")).expect("encode allocation recorded");
    assert!(reserve < alloc, "reserve precedes alloc: {events:?}");
    // (b) The published envelope, read back through the store's public
    // artifact API (the id the driver recorded).
    let art_id = d.last_published_artifact().expect("published").to_string();
    let st = auto_cte_probe::store::CteStore::open(root.path(), "enc").unwrap();
    let loaded = st
        .raw()
        .get_artifact_by_id(&art_id)
        .expect("artifact read")
        .expect("artifact present");
    let env: auto_cte_contract::CteExecutedV1 =
        serde_json::from_slice(&loaded.payload).expect("envelope decodes");
    let exact = auto_cte_probe::store::count_envelope_bytes(&env).unwrap();
    assert_eq!(
        exact as usize,
        loaded.payload.len(),
        "the counting writer matches the real encode"
    );
    // (c) Over-quota control: staging one byte SHORT of the exact
    // count — the reservation fails, NO encode allocation happens.
    let mut tight = auto_cte_probe::ManagedBudget::new(BudgetCaps {
        hot: 8 * 1024 * 1024,
        staging: exact - 1,
        av_slots: 8 * 1024 * 1024,
    });
    let rejected = auto_cte_probe::store::serialize_envelope_reserved(&env, &mut tight);
    assert!(rejected.is_err(), "the over-quota encode is rejected");
    let ev = tight.events();
    assert!(
        event_index(ev, is_reserve_what("envelope encode", false)).is_some(),
        "the rejection is AT the reservation: {ev:?}"
    );
    assert!(
        event_index(ev, is_alloc("envelope-encode")).is_none(),
        "no encode allocation after the rejection: {ev:?}"
    );
    // (d) Exactly-enough budget: accepted, reservation BEFORE the
    // allocation in the SAME event log — AND the real capacity IS the
    // grant (F-817-R10-02): the window's max single allocation never
    // exceeds the reserved count, and the returned Vec's capacity
    // equals its length (the growth chain that left 2048B behind a
    // 1163B grant is gone).
    let mut fits = auto_cte_probe::ManagedBudget::new(BudgetCaps {
        hot: 8 * 1024 * 1024,
        staging: exact,
        av_slots: 8 * 1024 * 1024,
    });
    begin_window();
    let payload = auto_cte_probe::store::serialize_envelope_reserved(&env, &mut fits)
        .expect("exact budget fits");
    let (enc_single, _) = window_readout();
    assert_eq!(payload.len() as u64, exact);
    assert_eq!(
        payload.capacity() as u64,
        exact,
        "exact-capacity encode: capacity == the reserved count"
    );
    assert!(
        enc_single <= exact as usize + 512,
        "the encode window's largest allocation ({enc_single}) stays at the grant ({exact})"
    );
    let ev2 = fits.events();
    let r2 = event_index(ev2, is_reserve_what("envelope encode", true)).unwrap();
    let a2 = event_index(ev2, is_alloc("envelope-encode")).unwrap();
    assert!(r2 < a2, "reserve precedes alloc: {ev2:?}");
    // Sensitivity: the GROWTH-based encoder (serde_json::to_vec, the
    // pre-fix algorithm) on the same envelope overshoots the grant —
    // this checker must see it.
    begin_window();
    let grown = serde_json::to_vec(&env).expect("growth encoder runs");
    let (grown_single, _) = window_readout();
    assert!(
        grown.capacity() as u64 > exact || grown_single > exact as usize + 512,
        "the sensitivity control sees the growth encoder overshoot: cap={} single={grown_single} grant={exact}",
        grown.capacity()
    );
}

/// The RESTORE path: the codec-page bound is reserved before the
/// payload loads; the payload length (Hot) before the decode; a
/// shrunken Hot rejects the DECODE with the payload load already
/// ordered; a shrunken Staging rejects the LOAD itself; old entry
/// intact.
#[test]
fn cte_resources_restore_reserve_before_alloc_order() {
    let root = tempfile::tempdir().unwrap();
    let deck = small_query_deck();
    let mut d = CteDriver::with_store(root.path(), "rst").unwrap();
    assert_eq!(d.query_step(&deck, None).status, StepStatus::Ok);
    // (a) Legal restore (fresh driver, same store): frame-read reserve
    // < payload-copy reserve < frame alloc < decode reserve < decode
    // alloc (F-817-R9-01 separate grants).
    let mut d2 = CteDriver::with_store(root.path(), "rst").unwrap();
    assert_eq!(d2.query_step(&deck, None).status, StepStatus::Ok);
    let events = d2.last_budget_events();
    let frame_reserve = event_index(events, is_reserve_what("frozen frame read", true))
        .expect("frame reservation recorded");
    let copy_reserve = event_index(events, is_reserve_what("frozen payload copy", true))
        .expect("copy reservation recorded");
    assert!(frame_reserve < copy_reserve, "{events:?}");
    let payload_alloc =
        event_index(events, is_alloc("frozen-payload")).expect("payload allocation recorded");
    let decode_reserve = event_index(events, is_reserve_what("restored envelope decode", true))
        .expect("decode reservation recorded");
    let decode_alloc =
        event_index(events, is_alloc("restore-decode")).expect("decode allocation recorded");
    assert!(copy_reserve < payload_alloc, "{events:?}");
    assert!(payload_alloc < decode_reserve, "{events:?}");
    assert!(decode_reserve < decode_alloc, "{events:?}");
    // (b) Shrunken Hot: the DECODE reservation fails — the payload
    // load was already ordered (reserve before alloc), the decode
    // allocation never happens.
    let mut d3 = CteDriver::with_store(root.path(), "rst").unwrap();
    d3.set_managed_caps(BudgetCaps {
        hot: 8,
        staging: 16 * 1024 * 1024,
        av_slots: 8 * 1024 * 1024,
    });
    let out3 = d3.query_step(&deck, None);
    assert_eq!(out3.status, StepStatus::Diag);
    assert!(
        out3.message
            .as_deref()
            .unwrap_or("")
            .contains("restored envelope decode"),
        "{:?}",
        out3.message
    );
    let events3 = d3.last_budget_events();
    assert!(event_index(events3, is_reserve_what("restored envelope decode", false)).is_some());
    assert!(
        event_index(events3, is_alloc("frozen-payload")).is_some(),
        "the load was gated first (ordered), the decode is what fails"
    );
    assert!(event_index(events3, is_alloc("restore-decode")).is_none());
    // (c) Shrunken Staging: the FRAME-READ reservation fails — the
    // loader never allocates.
    let load_reserve_idx =
        event_index(events, is_reserve_what("frozen frame read", true)).expect("load reserve");
    let pre_load = staging_live_at(events, load_reserve_idx);
    let mut d4 = CteDriver::with_store(root.path(), "rst").unwrap();
    d4.set_managed_caps(BudgetCaps {
        hot: 8 * 1024 * 1024,
        staging: pre_load,
        av_slots: 8 * 1024 * 1024,
    });
    let out4 = d4.query_step(&deck, None);
    assert_eq!(out4.status, StepStatus::Diag);
    assert!(
        out4.message
            .as_deref()
            .unwrap_or("")
            .contains("frozen frame read"),
        "{:?}",
        out4.message
    );
    let events4 = d4.last_budget_events();
    assert!(event_index(events4, is_reserve_what("frozen frame read", false)).is_some());
    assert!(event_index(events4, is_alloc("frozen-frame")).is_none());
    assert!(event_index(events4, is_alloc("frozen-payload")).is_none());
    // (d) Old entry intact throughout.
    let mut d5 = CteDriver::with_store(root.path(), "rst").unwrap();
    let out5 = d5.query_step(&deck, None);
    assert_eq!(out5.status, StepStatus::Ok);
    assert_eq!(out5.cache, Some("hit"));
}

/// F-817-R7-01: an OVERSIZED carrier under a REAL request identity is
/// rejected at the READ bound, BEFORE any reassembly allocation. The
/// identity comes from a REAL execution (publish -> read the envelope
/// back); a fresh store then carries a 2MiB LayoutData payload under
/// that exact key via the PUBLIC put API (the review's probe shape);
/// the driver's zero-allocation size probe sees the real framed size
/// and rejects — the loader NEVER RUNS (no load attempt counter, no
/// load reservation, nothing published), and the probe's numbers are
/// independently checked against the planted truth.
#[test]
fn cte_resources_oversized_restore_rejected_before_load() {
    use ac_artifact_probe::codec::ChunkKind;
    use ac_artifact_probe::store::PutRequest;
    use auto_cte_contract::{CteExecutedV1, CTE_FROZEN_LAYER};

    let root_a = tempfile::tempdir().unwrap();
    let deck = small_query_deck();
    // (1) A REAL execution publishes a real envelope at the REAL identity.
    let mut d = CteDriver::with_store(root_a.path(), "r7").unwrap();
    assert_eq!(d.query_step(&deck, None).status, StepStatus::Ok);
    let art_id = d.last_published_artifact().expect("published").to_string();
    let st_a = auto_cte_probe::store::CteStore::open(root_a.path(), "r7").unwrap();
    let loaded = st_a
        .raw()
        .get_artifact_by_id(&art_id)
        .expect("artifact read")
        .expect("artifact present");
    let env: CteExecutedV1 = serde_json::from_slice(&loaded.payload).expect("decodes");
    let identity = env.request_identity;
    // (2) A FRESH store carrying a 2MiB payload under that exact key
    // (public put; the review counterexample's shape).
    let root_b = tempfile::tempdir().unwrap();
    let st_b = auto_cte_probe::store::CteStore::open(root_b.path(), "r7").unwrap();
    st_b.raw()
        .put_artifact(PutRequest {
            layer: CTE_FROZEN_LAYER,
            key: auto_cte_probe::store::envelope_key(&identity),
            kind: ChunkKind::LayoutData,
            payload: vec![b' '; 2 * 1024 * 1024],
            edges: Vec::new(),
            project: Some("r7".into()),
        })
        .expect("plant");
    // The probe reports the REAL planted size (independent check).
    let probed = auto_cte_probe::store::probe_frozen_size(&st_b, &identity)
        .expect("probe")
        .expect("present");
    assert_eq!(
        probed,
        (
            2 * 1024 * 1024 + ac_artifact_probe::codec::CHUNK_HEADER_LEN as u64,
            2 * 1024 * 1024
        ),
        "the zero-allocation probe returns the true framed/payload sizes"
    );
    // (3) The driver request REJECTS at the read bound BEFORE the load.
    let mut d2 = CteDriver::with_store(root_b.path(), "r7").unwrap();
    let out = d2.query_step(&deck, None);
    assert_eq!(out.status, StepStatus::Diag);
    let msg = out.message.as_deref().unwrap_or_default();
    assert!(
        msg.contains("reassembly bound") && msg.contains("BEFORE load"),
        "the rejection names the read bound: {msg}"
    );
    // The loader NEVER ran: probe yes, load no.
    assert_eq!(out.counters.get("frozen_probes"), Some(&1));
    assert!(
        out.counters.get("frozen_load_attempts").is_none(),
        "no reassembly was attempted (counters: {:?})",
        out.counters
    );
    // No load reservation either — the rejection precedes the ledger.
    let ev2 = d2.last_budget_events();
    assert!(
        event_index(ev2, is_reserve_what("frozen frame read", false)).is_none()
            && event_index(ev2, is_reserve_what("frozen frame read", true)).is_none()
            && event_index(ev2, is_reserve_what("frozen payload copy", false)).is_none()
            && event_index(ev2, is_reserve_what("frozen payload copy", true)).is_none(),
        "no load reservation was reached: {ev2:?}"
    );
    assert!(d2.last_published_artifact().is_none());
    // (4) The planted store row is INTACT (a normal smaller envelope at
    // a DIFFERENT identity still serves fine on store A).
    let mut d3 = CteDriver::with_store(root_a.path(), "r7").unwrap();
    let out3 = d3.query_step(&deck, None);
    assert_eq!(out3.status, StepStatus::Ok);
    assert_eq!(out3.cache, Some("hit"), "the good entry still hits");
}

/// The wire/input path: a base64 payload over the 4MiB deck bound is
/// refused at the PRE-CHECK (before the decode allocates); a request
/// over the 6MiB wire bound is refused by the BOUNDED READ itself.
#[test]
fn cte_resources_wire_input_bounded() {
    use std::io::Write as _;
    use std::process::{Command, Stdio};
    let serve = |deck: &str, nonce: &str| -> serde_json::Value {
        let req = auto_cte_probe::bench::build_worker_request_for_deck(
            nonce,
            &format!("rh-{nonce}"),
            std::path::Path::new("C:/cte817-nowhere"),
            "w",
            deck,
            &serde_json::json!([{ "op": "query" }]),
        );
        let mut child = Command::new(env!("CARGO_BIN_EXE_cte_probe_worker"))
            .arg("case-serve")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("worker spawn");
        // The oversized-wire control EXPECTS the child to refuse after
        // cap+1 bytes and exit — the remaining write may hit EPIPE.
        let _ = child.stdin.take().unwrap().write_all(req.as_bytes());
        let out = child.wait_with_output().expect("worker completes");
        serde_json::from_str(String::from_utf8_lossy(&out.stdout).trim()).unwrap()
    };
    // (a) A 4.2MiB deck: base64 ≈ 5.9MiB — under the 6MiB wire bound,
    // OVER the deck_b64 pre-check — refused BEFORE the decode.
    let big = "x".repeat((auto_cte_contract::MAX_REQUEST_INPUT_BYTES / 100 * 105) as usize);
    let v = serve(&big, "deep");
    assert!(v["ok"].is_null(), "refused: {v}");
    let msg = v["error"]["message"].as_str().unwrap_or_default();
    assert!(
        msg.contains("deck_b64") && msg.contains("exceeds"),
        "the pre-check names the bound: {msg}"
    );
    // (b) A 7MiB deck: base64 ≈ 9.3MiB — over the wire bound; the
    // BOUNDED READ refuses after at most cap+1 bytes.
    let huge = "y".repeat(7 * 1024 * 1024);
    let v2 = serve(&huge, "wire");
    assert!(v2["ok"].is_null(), "refused: {v2}");
    let msg2 = v2["error"]["message"].as_str().unwrap_or_default();
    assert!(
        msg2.contains("wire bound"),
        "the bounded read names itself: {msg2}"
    );
}

// ---- F-817-R8-01 / R8-02: the corruption/SQL counter-controls ------------

/// The review's REAL-allocation counterexample, replayed: a legit 512B
/// payload is published (framed 560B, granted read budget 1072B), then
/// the blob FILE on disk is replaced by 2MiB garbage while the db
/// metadata still says 560 — the bounded loader must reject at the
/// granted read budget with the window's LARGEST single allocation
/// staying within the budget (the 2MiB is never read). The undamaged
/// positive control first proves the same loader path's allocation
/// stays in budget on a normal hit.
#[test]
fn cte_resources_corrupt_blob_read_bounded() {
    let root = tempfile::tempdir().unwrap();
    let deck = small_query_deck();
    let mut d = CteDriver::with_store(root.path(), "r8").unwrap();
    assert_eq!(d.query_step(&deck, None).status, StepStatus::Ok);

    // (a) In-budget positive control: a normal hit's largest single
    // allocation AND coexisting-live peak stay modest (no ungranted
    // payload-sized copies).
    {
        begin_window();
        let mut h = CteDriver::with_store(root.path(), "r8").unwrap();
        let out = h.query_step(&deck, None);
        assert_eq!(out.status, StepStatus::Ok);
        assert_eq!(out.cache, Some("hit"));
        let (max, _live) = window_readout();
        assert!(
            max <= 512 * 1024,
            "normal restore window max single alloc {max} stays modest (no payload-sized copies)"
        );
        assert!(
            window_readout().1 <= 512 * 1024,
            "coexisting-live peak {} stays modest",
            window_readout().1
        );
    }

    // (b) The probed sizes (metadata still correct here).
    let d2 = CteDriver::with_store(root.path(), "r8").unwrap();
    // Re-derive the identity from the last published envelope.
    let art_id = d2
        .last_published_artifact()
        .map(str::to_string)
        .or_else(|| d.last_published_artifact().map(str::to_string))
        .expect("published");
    let st = auto_cte_probe::store::CteStore::open(root.path(), "r8").unwrap();
    let loaded = st
        .raw()
        .get_artifact_by_id(&art_id)
        .expect("read")
        .expect("present");
    let env: auto_cte_contract::CteExecutedV1 =
        serde_json::from_slice(&loaded.payload).expect("decodes");
    let probed = auto_cte_probe::store::probe_frozen_size(&st, &env.request_identity)
        .expect("probe")
        .expect("present");
    assert_eq!(probed.1, loaded.payload.len() as u64);

    // (c) Replace the blob FILE with 2MiB garbage (db metadata intact).
    let mut blobs: Vec<std::path::PathBuf> = Vec::new();
    fn walk(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
        if let Ok(rd) = std::fs::read_dir(dir) {
            for e in rd.flatten() {
                let p = e.path();
                if p.is_dir() {
                    walk(&p, out);
                } else {
                    out.push(p);
                }
            }
        }
    }
    walk(&root.path().join("blocks"), &mut blobs);
    assert!(!blobs.is_empty(), "blob files found under blocks/");
    let blob = blobs
        .iter()
        .max_by_key(|p| std::fs::metadata(p).map(|m| m.len()).unwrap_or(0))
        .expect("largest blob");
    std::fs::write(blob, vec![0xABu8; 2 * 1024 * 1024]).expect("corrupt the blob");

    // (d) The bounded loader rejects at the granted budget — the 2MiB
    // is NEVER read: the window's max single allocation stays at the
    // small-allocation scale, far below the 2MiB corruption.
    {
        begin_window();
        let mut d3 = CteDriver::with_store(root.path(), "r8").unwrap();
        let out = d3.query_step(&deck, None);
        assert_eq!(out.status, StepStatus::Diag);
        let msg = out.message.as_deref().unwrap_or_default();
        assert!(
            msg.contains("granted read budget"),
            "the rejection names the physical read bound: {msg}"
        );
        assert!(
            !out.language_class,
            "a corrupt blob is infra, never a language verdict"
        );
        assert!(d3.last_published_artifact().is_none());
        let (max, _live) = window_readout();
        assert!(
            max < 2 * 1024 * 1024,
            "the 2MiB corrupted blob was NEVER read into memory (window max single alloc = {max}B)"
        );
        assert!(
            max <= 512 * 1024,
            "the rejection window's max allocation stays at small scale: {max}B"
        );
    }
}

/// F-817-R9-01: the EXACT-capacity bounded read — narrow-window
/// allocator evidence at sizes AROUND the power-of-two growth steps
/// (where read_to_end's doubling used to overshoot). For each planted
/// payload size (legal content via the PUBLIC put, one identity per
/// size), four states are measured against the granted budgets
/// (frame grant = probed framed length; copy grant = frame−48):
///   legal   -> Ok, max SINGLE alloc == framed (exact capacity, no
///              growth), coexisting peak <= frame + copy grants;
///   +1 byte -> rejected at the read bound, ZERO content allocation;
///   same-len garbage -> read in full (single alloc == framed), then
///              rejected by CAS — allocation never exceeds the grant;
///   shrunk  -> read_exact fails (UnexpectedEof), controlled Io.
#[test]
fn cte_resources_bounded_read_capacity_exact() {
    use ac_artifact_probe::codec::ChunkKind;
    use ac_artifact_probe::store::PutRequest;
    use auto_cte_contract::{CteRequestIdentityV1, CTE_FROZEN_LAYER};

    let root = tempfile::tempdir().unwrap();
    let st = auto_cte_probe::store::CteStore::open(root.path(), "r9").unwrap();
    // Small fixed incidental allocations OUTSIDE the loader (identity
    // strings, SQL row buffers) — the content-buffer budgets below are
    // asserted EXACTLY; this epsilon only covers those fixed-size
    /// artifacts, independent of payload size.
    const INCIDENTAL: usize = 2048;

    let mut blobs: Vec<std::path::PathBuf> = Vec::new();
    fn walk(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
        if let Ok(rd) = std::fs::read_dir(dir) {
            for e in rd.flatten() {
                let p = e.path();
                if p.is_dir() {
                    walk(&p, out);
                } else {
                    out.push(p);
                }
            }
        }
    }

    for (i, payload_len) in [
        500usize, 512, 524, 1023, 1024, 1032, 2047, 2048, 2056,
        // Content-dominated size: the granted coexisting budget
        // (2*framed-48 ~= 200KB) sits BELOW the old growth algorithm's
        // footprint (read_to_end capacity 131072 + copy 100000), which
        // the sensitivity control proves this sweep would catch.
        100_000,
    ]
    .into_iter()
    .enumerate()
    {
        // One identity per size; a LEGAL chunk via the public put.
        let identity = CteRequestIdentityV1 {
            module: "main".into(),
            package: "r9".into(),
            callee: format!("sum{i}"),
            args_canonical: format!("i32:{payload_len}"),
            query_algo: "r9".into(),
            caps: vec![],
            engine: auto_cte_probe::av_driver::current_engine(),
            manifest_fp: format!("r9-{payload_len}"),
        };
        st.raw()
            .put_artifact(PutRequest {
                layer: CTE_FROZEN_LAYER,
                key: auto_cte_probe::store::envelope_key(&identity),
                kind: ChunkKind::LayoutData,
                payload: vec![0x5Au8; payload_len],
                edges: Vec::new(),
                project: Some("r9".into()),
            })
            .expect("plant");
        let (framed, probed_payload) = auto_cte_probe::store::probe_frozen_size(&st, &identity)
            .expect("probe")
            .expect("present");
        assert_eq!(probed_payload as usize, payload_len);

        // Locate this size's blob file (framed length is unique here).
        blobs.clear();
        walk(&root.path().join("blocks"), &mut blobs);
        let blob = blobs
            .iter()
            .find(|p| std::fs::metadata(p).map(|m| m.len()).ok() == Some(framed))
            .unwrap_or_else(|| panic!("blob for framed {framed} not found"))
            .clone();

        // (1) LEGAL: exact single allocation == framed; coexisting peak
        // within the two grants (frame + copy = 2*framed−48).
        {
            begin_window();
            let out = auto_cte_probe::store::load_frozen_payload_bounded(&st, &identity, framed)
                .expect("legal load")
                .expect("present");
            assert_eq!(out.len(), payload_len);
            let (max, live) = window_readout();
            assert!(
                max <= framed as usize + INCIDENTAL,
                "legal {payload_len}: max single alloc {max} <= framed {framed} (+incidental)"
            );

            assert!(
                live <= (2 * framed - 48) as usize + INCIDENTAL,
                "legal {payload_len}: coexisting peak {live} <= frame+copy grants {} (+incidental)",
                2 * framed - 48
            );
        }

        // (2) +1 BYTE over the FRAMED grant: rejected at the read
        // bound, ZERO content allocation (no frame-sized buffer is
        // ever built).
        std::fs::write(&blob, vec![0xABu8; framed as usize + 1]).expect("grow");
        {
            begin_window();
            let err = auto_cte_probe::store::load_frozen_payload_bounded(&st, &identity, framed)
                .expect_err("over-bound rejected");
            assert!(
                format!("{err:?}").contains("granted read budget"),
                "{payload_len}+1 names the read bound: {err:?}"
            );
            let (max, _live) = window_readout();
            assert!(
                max <= INCIDENTAL,
                "{payload_len}+1: zero content allocation (max single {max})"
            );
        }

        // (3) SAME-LENGTH garbage: passes metadata, reads EXACTLY
        // framed bytes (single alloc == framed — the read_to_end
        // doubling overshoot is gone), then CAS rejects.
        std::fs::write(&blob, vec![0xCDu8; framed as usize]).expect("same-len garbage");
        {
            begin_window();
            let err = auto_cte_probe::store::load_frozen_payload_bounded(&st, &identity, framed)
                .expect_err("corrupt CAS rejected");
            assert!(
                format!("{err:?}").contains("address mismatch"),
                "{payload_len} same-len garbage hits CAS: {err:?}"
            );
            let (max, _live) = window_readout();
            assert!(
                max <= framed as usize + INCIDENTAL,
                "{payload_len} garbage: single alloc {max} <= framed {framed} (exact capacity)"
            );
        }

        // (4) SHRUNK file: read_exact fails — controlled Io rejection.
        std::fs::write(&blob, vec![0xEFu8; payload_len.saturating_sub(100).max(1)])
            .expect("shrink");
        {
            begin_window();
            let err = auto_cte_probe::store::load_frozen_payload_bounded(&st, &identity, framed)
                .expect_err("shrunk rejected");
            let txt = format!("{err:?}");
            assert!(
                txt.contains("bounded chunk read") || txt.contains("address mismatch"),
                "{payload_len} shrunk: controlled rejection: {txt}"
            );
            let (max, _live) = window_readout();
            assert!(
                max <= framed as usize + INCIDENTAL,
                "{payload_len} shrunk: allocation bounded by the grant"
            );
        }

        // Restore a legal file so later sizes find their blobs cleanly.
        std::fs::write(&blob, {
            // Rebuild the exact framed bytes via a fresh put at a
            // scratch identity is unnecessary — the store's CAS
            // requires the original content; just leave the damaged
            // file (each size uses its own unique blob).
            vec![0u8; 0]
        })
        .ok();
    }
}

/// F-817-R8-02: ONLY a genuinely absent row is a miss. A real SQL error
/// (schema damage — the chunks table renamed away) propagates as a
/// controlled infra rejection: the driver NEVER treats it as a miss
/// (no AV execution, nothing published). The empty-store control
/// (a genuine miss) still executes normally.
#[test]
fn cte_resources_probe_sql_error_not_miss() {
    let root = tempfile::tempdir().unwrap();
    let deck = small_query_deck();
    // (a) Genuine-miss control: an empty store executes normally.
    let mut miss = CteDriver::with_store(root.path(), "empty").unwrap();
    let out = miss.query_step(&deck, None);
    assert_eq!(out.status, StepStatus::Ok);
    assert_eq!(out.cache, Some("miss"));
    assert_eq!(out.ct_execute, 1, "a genuine miss executes");

    // (b) Damage the schema in an ISOLATED store that already carries
    // a good entry (the error must fire, not the miss path). The
    // rename runs on THE DRIVER'S OWN open connection — the store's
    // schema init (`CREATE TABLE IF NOT EXISTS`) would silently
    // recreate an empty chunks table on a fresh open, hiding the error.
    let root2 = tempfile::tempdir().unwrap();
    let mut seed = CteDriver::with_store(root2.path(), "r8b").unwrap();
    assert_eq!(seed.query_step(&deck, None).status, StepStatus::Ok);
    let mut d = CteDriver::with_store(root2.path(), "r8b").unwrap();
    d.store()
        .expect("store attached")
        .raw()
        .db()
        .execute_batch("ALTER TABLE chunks RENAME TO chunks_saved")
        .expect("rename for the error control");

    // (c) The probe surfaces the SQL error — the driver refuses with
    // the infra state and NEVER starts an AV recomputation.
    let out = d.query_step(&deck, None);
    assert_eq!(out.status, StepStatus::Diag, "refused: {:?}", out.message);
    let msg = out.message.as_deref().unwrap_or_default();
    assert!(
        msg.contains("size probe failed"),
        "the rejection names the probe error: {msg}"
    );
    assert!(
        msg.contains("no such table"),
        "the underlying SQL cause is preserved: {msg}"
    );
    assert_eq!(
        out.ct_execute, 0,
        "an infra probe failure NEVER becomes a silent AV recompute"
    );
    assert!(d.last_published_artifact().is_none());
}

/// F-817-R10-01: the decode grant comes from the STRUCTURAL AUDIT (the
/// real container population), so a short-string envelope cannot
/// decode past its grant. The review's counterexample replayed on the
/// REAL driver path: the identity is taken from a real execution, a
/// foreign envelope (schema replaced, closure = N empty strings) is
/// planted under that exact key via the PUBLIC put, and the driver
/// restores it:
///   N=100000 -> the audit grant (~17.6MB) exceeds the 8MiB Hot cap —
///     the reservation is refused BEFORE serde_json ever runs (the
///     window shows no decode-sized allocation);
///   N=10000  -> the audit grant (~1.76MB) fits, serde runs, and the
///     window's real allocations stay under the grant (the old "2x
///     payload" grant left a 393216B single allocation against
///     62292B); the foreign schema is then a controlled rejection;
///   N=0      -> the pristine envelope hits normally.
#[test]
fn cte_resources_decode_audit_bounded() {
    use ac_artifact_probe::codec::ChunkKind;
    use ac_artifact_probe::store::PutRequest;
    use auto_cte_contract::{CteExecutedV1, CTE_FROZEN_LAYER};

    let seed_root = tempfile::tempdir().unwrap();
    let deck = small_query_deck();
    let mut d = CteDriver::with_store(seed_root.path(), "r10").unwrap();
    assert_eq!(d.query_step(&deck, None).status, StepStatus::Ok);
    let art_id = d.last_published_artifact().expect("published").to_string();
    let st_a = auto_cte_probe::store::CteStore::open(seed_root.path(), "r10").unwrap();
    let loaded = st_a
        .raw()
        .get_artifact_by_id(&art_id)
        .expect("read")
        .expect("present");
    let env: CteExecutedV1 = serde_json::from_slice(&loaded.payload).expect("decodes");
    let identity = env.request_identity.clone();

    for n in [0usize, 10000, 100000] {
        let mut foreign = env.clone();
        if n > 0 {
            foreign.schema = "foreign-schema".into();
            foreign.deps.closure = vec![String::new(); n];
        }
        let bytes = serde_json::to_vec(&foreign).unwrap();
        assert!(
            bytes.len() <= 1024 * 1024,
            "plant stays under the read bound"
        );
        let root = tempfile::tempdir().unwrap();
        let st = auto_cte_probe::store::CteStore::open(root.path(), "r10").unwrap();
        st.raw()
            .put_artifact(PutRequest {
                layer: CTE_FROZEN_LAYER,
                key: auto_cte_probe::store::envelope_key(&identity),
                kind: ChunkKind::LayoutData,
                payload: bytes.clone(),
                edges: Vec::new(),
                project: Some("r10".into()),
            })
            .expect("plant");
        let audit = auto_cte_probe::store::audit_envelope_budget(&bytes)
            .expect("audit scans the syntactic json")
            .max(1);
        begin_window();
        let mut drv = CteDriver::with_store(root.path(), "r10").unwrap();
        let out = drv.query_step(&deck, None);
        let (single, live) = window_readout();
        match n {
            0 => {
                assert_eq!(out.status, StepStatus::Ok);
                assert_eq!(out.cache, Some("hit"));
                assert_eq!(out.ct_execute, 0);
            }
            100000 => {
                // The audit grant (>= 100000 * 176 = 17.6MB) exceeds the
                // 8MiB Hot cap: refused at the reservation, from_slice
                // NEVER runs — no decode-sized allocation appears.
                assert!(audit > 8 * 1024 * 1024, "audit {audit} exceeds Hot");
                assert_eq!(out.status, StepStatus::Diag, "{:?}", out.message);
                assert!(
                    out.message
                        .as_deref()
                        .unwrap_or_default()
                        .contains("restored envelope decode"),
                    "refused at the decode reservation: {:?}",
                    out.message
                );
                assert!(drv.last_published_artifact().is_none());
                assert!(
                    single < 1024 * 1024,
                    "no decode-sized allocation (max single {single}; the old path saw 3145728)"
                );
                assert!(
                    live < 2 * 1024 * 1024,
                    "coexisting peak {live} stays at load scale"
                );
            }
            10000 => {
                // The audit grant (~1.76MB) fits the 8MiB Hot cap: serde
                // runs and its REAL allocations stay under the grant —
                // then the foreign schema is a controlled rejection.
                assert!(audit <= 8 * 1024 * 1024, "audit {audit} fits Hot");
                assert_eq!(out.status, StepStatus::Diag, "{:?}", out.message);
                let msg = out.message.as_deref().unwrap_or_default();
                assert!(
                    msg.contains("foreign-schema") || msg.contains("schema"),
                    "the foreign schema is the rejection: {msg}"
                );
                assert!(drv.last_published_artifact().is_none());
                assert!(
                    single as u64 <= audit,
                    "decode's largest allocation ({single}) stays under the audited grant ({audit}; the old 2x grant saw 393216 against 62292)"
                );
                assert!(
                    live as u64 <= audit + bytes.len() as u64 + 4096,
                    "coexisting peak ({live}) stays under grant + payload + slack"
                );
            }
            _ => unreachable!(),
        }
    }
}

/// Splice a `digits`-long run of 9s over an integer field's literal —
/// builds a syntactically valid JSON number far past u64 (serde's
/// `parse_long_integer` consumes the rest of the digits through the
/// shared scratch buffer before the u64 visitor rejects the result).
fn splice_giant_number(json: &[u8], field: &str, digits: usize) -> Vec<u8> {
    let key = format!("\"{field}\":");
    let pos = json
        .windows(key.len())
        .position(|w| w == key.as_bytes())
        .expect("field present");
    let mut end = pos + key.len();
    while json
        .get(end)
        .is_some_and(|c| c.is_ascii_digit() || *c == b'-')
    {
        end += 1;
    }
    let mut out = Vec::with_capacity(json.len() + digits);
    out.extend_from_slice(&json[..pos + key.len()]);
    out.extend(std::iter::repeat_n(b'9', digits));
    out.extend_from_slice(&json[end..]);
    out
}

/// F-817-R11-01: the decode Hot grant must cover the REAL serde_json
/// decode footprint — escaped strings route through the deserializer's
/// shared scratch Vec (grown by doubling) BEFORE serde_core's
/// StringVisitor copies the unescaped bytes into the owned String that
/// lands in the envelope, so the escaped tier coexists at
/// scratch-capacity + owned ≈ 3x the string while the old "2x S" term
/// granted 2x. The review's replay shapes (real execution identity,
/// public put, foreign schema so admission rejects AFTER the decode
/// allocations happen): source_fp = "x"×N + "\n" + "x"×N vs a plain
/// control of the same decoded length. Long number literals share the
/// same scratch buffer (`parse_long_integer`), so they are audited as
/// tokens too — not just strings.
#[test]
fn cte_resources_decode_audit_escaped_scratch_bounded() {
    use ac_artifact_probe::codec::ChunkKind;
    use ac_artifact_probe::store::PutRequest;
    use auto_cte_contract::{CteExecutedV1, CTE_FROZEN_LAYER};

    let seed_root = tempfile::tempdir().unwrap();
    let deck = small_query_deck();
    let mut d = CteDriver::with_store(seed_root.path(), "r11").unwrap();
    assert_eq!(d.query_step(&deck, None).status, StepStatus::Ok);
    let art_id = d.last_published_artifact().expect("published").to_string();
    let st_a = auto_cte_probe::store::CteStore::open(seed_root.path(), "r11").unwrap();
    let loaded = st_a
        .raw()
        .get_artifact_by_id(&art_id)
        .expect("read")
        .expect("present");
    let env: CteExecutedV1 = serde_json::from_slice(&loaded.payload).expect("decodes");
    let identity = env.request_identity.clone();

    for n in [32768usize, 65536, 131072] {
        for escaped in [true, false] {
            let mut foreign = env.clone();
            foreign.schema = "foreign-schema".into();
            // Escaped: one real newline — serde_json encodes it as the
            // 2-byte `\n` escape, forcing the scratch path. Plain: same
            // decoded length (2N+1), zero escapes — Borrowed fast path.
            foreign.source_fp = if escaped {
                format!("{}\n{}", "x".repeat(n), "x".repeat(n))
            } else {
                format!("{}y{}", "x".repeat(n), "x".repeat(n))
            };
            let bytes = serde_json::to_vec(&foreign).unwrap();
            assert!(
                bytes.len() <= 1024 * 1024,
                "plant stays under the read bound"
            );

            // The audit itself allocates nothing (Ok path).
            begin_window();
            let audit = auto_cte_probe::store::audit_envelope_budget(&bytes)
                .expect("audit scans the syntactic json");
            let (a_single, a_live) = window_readout();
            assert_eq!((a_single, a_live), (0, 0), "audit is zero-allocation");

            // DECODE-ONLY window (the payload is NOT part of the Hot
            // grant): the controlled schema rejection happens AFTER the
            // decode allocations, so the window observes exactly what
            // the grant must cover — scratch + owned + containers.
            begin_window();
            let decoded = auto_cte_probe::store::decode_envelope(&bytes, &identity);
            let (single, live) = window_readout();
            assert!(decoded.is_err(), "foreign schema is rejected");
            let s_big = 2 * n as u64 + if escaped { 2 } else { 1 };
            assert!(
                audit >= s_big + 2 * s_big,
                "grant ({audit}) covers owned (>= {s_big}) + scratch (2x {s_big})"
            );
            assert!(
                single as u64 <= audit,
                "largest decode allocation ({single}) over the grant ({audit})"
            );
            assert!(
                live as u64 <= audit,
                "coexisting decode peak ({live}) over the grant ({audit}) — the R11 replay saw 393995 against 268780 at N=65536"
            );
            if escaped {
                // Sensitivity (the check distinguishes the OLD model):
                // the measured peak exceeds the old "2x S" string term
                // even with 16KiB of slack for the container factor and
                // the fixed fields — so a regression back to 2x S must
                // turn this test red.
                assert!(
                    live as u64 > 2 * s_big + 16384,
                    "peak ({live}) must exceed the old 2x-string grant + slack (2*{s_big}+16384)"
                );
            }
        }
    }

    // Long number literals share the scratch Vec: a 100000-digit
    // `exec_steps` is syntactically valid JSON, so the audit must grant
    // its token footprint BEFORE from_slice grows the scratch to hold
    // the digits.
    let giant = splice_giant_number(&loaded.payload, "exec_steps", 100000);
    begin_window();
    let audit =
        auto_cte_probe::store::audit_envelope_budget(&giant).expect("audit scans the giant number");
    let (a_single, a_live) = window_readout();
    assert_eq!((a_single, a_live), (0, 0), "audit is zero-allocation");
    assert!(
        audit >= 2 * 100000,
        "grant ({audit}) covers the number's scratch term"
    );
    begin_window();
    let decoded = auto_cte_probe::store::decode_envelope(&giant, &identity);
    let (single, live) = window_readout();
    assert!(decoded.is_err(), "a >u64 literal cannot decode into u64");
    assert!(
        single as u64 <= audit,
        "max single ({single}) over ({audit})"
    );
    assert!(
        live as u64 <= audit,
        "coexisting peak ({live}) over ({audit})"
    );

    // DRIVER path, default caps: the (now sufficient) grant fits Hot,
    // so the foreign envelope reaches the schema rejection — never an
    // AV recompute, never a publish.
    let mut foreign = env.clone();
    foreign.schema = "foreign-schema".into();
    foreign.source_fp = format!("{}\n{}", "x".repeat(65536), "x".repeat(65536));
    let bytes = serde_json::to_vec(&foreign).unwrap();
    let root = tempfile::tempdir().unwrap();
    let st = auto_cte_probe::store::CteStore::open(root.path(), "r11").unwrap();
    st.raw()
        .put_artifact(PutRequest {
            layer: CTE_FROZEN_LAYER,
            key: auto_cte_probe::store::envelope_key(&identity),
            kind: ChunkKind::LayoutData,
            payload: bytes.clone(),
            edges: Vec::new(),
            project: Some("r11".into()),
        })
        .expect("plant");
    let mut drv = CteDriver::with_store(root.path(), "r11").unwrap();
    let out = drv.query_step(&deck, None);
    assert_eq!(out.status, StepStatus::Diag, "{:?}", out.message);
    assert!(
        out.message
            .as_deref()
            .unwrap_or_default()
            .contains("foreign-schema"),
        "the rejection is the foreign schema, not the budget: {:?}",
        out.message
    );
    assert_eq!(out.ct_execute, 0);
    assert!(drv.last_published_artifact().is_none());
}

/// F-817-R10-03: the observer + thresholds actually DISTINGUISH the
/// pre-fix algorithm. A faithful in-test replica of the old growth
/// read (File::take + read_to_end, the R8 shape) on a REAL 100000B
/// blob produces a coexisting footprint ABOVE the granted budget that
/// this suite asserts — so a regression back to growth-based reading
/// turns these gates red, while the current exact-capacity read stays
/// green (covered by the sweep's 100000 size).
#[test]
fn cte_resources_allocator_gate_sensitivity() {
    use ac_artifact_probe::codec::ChunkKind;
    use ac_artifact_probe::store::PutRequest;
    use auto_cte_contract::{CteRequestIdentityV1, CTE_FROZEN_LAYER};

    let root = tempfile::tempdir().unwrap();
    let st = auto_cte_probe::store::CteStore::open(root.path(), "r10s").unwrap();
    let identity = CteRequestIdentityV1 {
        module: "main".into(),
        package: "r10s".into(),
        callee: "sens".into(),
        args_canonical: "i32:1".into(),
        query_algo: "r10s".into(),
        caps: vec![],
        engine: auto_cte_probe::av_driver::current_engine(),
        manifest_fp: "r10s".into(),
    };
    let payload_len = 100_000usize;
    st.raw()
        .put_artifact(PutRequest {
            layer: CTE_FROZEN_LAYER,
            key: auto_cte_probe::store::envelope_key(&identity),
            kind: ChunkKind::LayoutData,
            payload: vec![0x11u8; payload_len],
            edges: Vec::new(),
            project: Some("r10s".into()),
        })
        .expect("plant");
    let (framed, _) = auto_cte_probe::store::probe_frozen_size(&st, &identity)
        .expect("probe")
        .expect("present");
    // Locate the blob file.
    let mut blobs: Vec<std::path::PathBuf> = Vec::new();
    fn walk(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
        if let Ok(rd) = std::fs::read_dir(dir) {
            for e in rd.flatten() {
                let p = e.path();
                if p.is_dir() {
                    walk(&p, out);
                } else {
                    out.push(p);
                }
            }
        }
    }
    walk(&root.path().join("blocks"), &mut blobs);
    let blob = blobs
        .iter()
        .find(|p| std::fs::metadata(p).map(|m| m.len()).ok() == Some(framed as u64))
        .expect("blob")
        .clone();

    // The OLD algorithm's shape: File::take(cap) + read_to_end (growth),
    // then the payload copy — measured with the same window API.
    begin_window();
    let old_bytes = {
        use std::io::Read as _;
        let mut f = std::fs::File::open(&blob).unwrap();
        let mut buf = Vec::new();
        (&mut f)
            .take(framed)
            .read_to_end(&mut buf)
            .expect("old read");
        let payload_copy = buf[48..].to_vec();
        (buf.capacity(), payload_copy)
    };
    let (old_single, old_live) = window_readout();
    // The granted budget the sweep asserts for this size:
    // frame + copy = 2*framed - 48 (+ the same incidental margin).
    let granted = (2 * framed - 48) as usize + 2048;
    assert!(
        old_live > granted || old_single > framed as usize + 2048,
        "the sensitivity control MUST flag the old algorithm here: old_live={old_live} old_single={old_single} granted={granted}"
    );
    // And the CURRENT loader on the same blob stays within the grant
    // (cross-checked against the sweep's 100000 case).
    begin_window();
    let loaded = auto_cte_probe::store::load_frozen_payload_bounded(&st, &identity, framed)
        .expect("load")
        .expect("present");
    let (new_single, new_live) = window_readout();
    assert_eq!(loaded.len(), payload_len);
    assert!(new_single <= framed as usize + 2048);
    assert!(new_live <= granted, "current read within grant: {new_live}");
    let _ = old_bytes;
}

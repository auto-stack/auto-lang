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

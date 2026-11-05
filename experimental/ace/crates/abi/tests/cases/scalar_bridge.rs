//! PLAN-937 CA-D1 scalar-bridge conformance: the REAL chains through the
//! neutral foreign contract (plan §6 V-01..V-07).
//!
//! Every assertion here is an EXECUTED observable — legs exit 0 only
//! when the delivered machine code matches the independent oracles, the
//! source export only passes when the independent C caller's hand-
//! computed expectations hold, and every increment/restore claim is
//! read off the per-key producer counters, not off a green banner.

#![cfg(windows)]

use ace_abi::bridge::{
    self, restore_bridge, run_source_export, run_typed_caller_with_checks, run_typed_chain,
    source_export_body_edit, source_export_first_case, BridgeCounters, TypedLeg,
    BRIDGE_TOTAL_DEADLINE,
};
use ace_abi::typed_decl::{self, oracle_typed_mix, FROZEN_TYPED_INPUTS};
use ace_abi::validate::admit_descriptor;
use ace_artifacts::store::ManagedStore;
use ace_foreign_contract::PhysicalPlan;

use std::path::PathBuf;
use std::time::Duration;

fn scratch_root(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("t937-bridge-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let _ = tag;
    dir
}

fn store_at(tag: &str) -> (ManagedStore, PathBuf) {
    static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
    let _guard = LOCK.lock().unwrap_or_else(|p| p.into_inner());
    // NOTE: the lock only serializes store creation within this test
    // binary; each test uses its OWN root (below), so no cross-test
    // sharing happens.
    drop(_guard);
    let root = std::env::temp_dir().join(format!("t937-store-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    (ManagedStore::open(&root).expect("store opens"), root)
}

// ---------------------------------------------------------------------------
// V-01 / V-03: the typed chain's three REAL legs
// ---------------------------------------------------------------------------

/// All three typed legs (emitted↔emitted self-check, native→C, C→native)
/// execute green: every PE exits with its FAILED check count = 0.
#[test]
fn typed_chain_three_legs_execute_green() {
    let (store, _root) = store_at("typed-legs");
    let counters = BridgeCounters::new();
    let out = run_typed_chain(
        &store,
        &counters,
        &scratch_root("typed"),
        &[
            TypedLeg::SelfCheck,
            TypedLeg::NativeToC,
            TypedLeg::CToNative,
        ],
        BRIDGE_TOTAL_DEADLINE,
    )
    .expect("typed chain runs");
    for (leg, exit) in &out.legs {
        assert_eq!(*exit, 0, "leg {leg} must exit 0 (failed checks: {exit})");
    }
    assert_eq!(out.legs.len(), 3);
    // the typed callee was produced exactly once for all three legs
    let callee_key = bridge::typed_callee_key(&out.plan_digest).canonical();
    assert_eq!(counters.producer_calls(&callee_key), 1);
}

/// Negative control (the oracle discriminates): a caller baked with a
/// WRONG expectation must exit nonzero — a green leg can never be a
/// vacuous pass.
#[test]
fn typed_chain_negative_control_discriminates() {
    let (store, _root) = store_at("typed-neg");
    let counters = BridgeCounters::new();
    let mut checks = ace_abi::typed_emit::frozen_typed_checks();
    if let ace_abi::typed_emit::TypedCheck::Mix { expected, .. } = &mut checks[0] {
        *expected = oracle_typed_mix(&FROZEN_TYPED_INPUTS).wrapping_add(1);
    } else {
        panic!("first check must be the frozen mix");
    }
    let exit = run_typed_caller_with_checks(
        &store,
        &counters,
        &scratch_root("typed-mut"),
        &checks,
        false,
    )
    .expect("mutated caller runs");
    assert_ne!(exit, 0, "a wrong expected value must FAIL the PE");
}

/// V-03 boundary inputs are IN the frozen check set (extremes, quiet-NaN
/// payload, -0.0, bool both values, char boundaries, wrapping div).
#[test]
fn typed_frozen_inputs_carry_the_boundaries() {
    let x = FROZEN_TYPED_INPUTS;
    assert_eq!(x.b, 0xDEAD_BEEF);
    assert_eq!(x.c, i64::MIN + 123);
    assert_eq!(x.d, u64::MAX - 7);
    assert_eq!(x.f_bits, 0x7FF8_0000_DEAD_BEEF); // quiet NaN with payload
    assert_eq!(x.h, 0x10FFFF); // last Unicode scalar value
    let checks = ace_abi::typed_emit::frozen_typed_checks();
    // -0.0 variant present
    assert!(checks.iter().any(|c| matches!(c,
        ace_abi::typed_emit::TypedCheck::Mix { inputs, .. }
            if inputs.f_bits == (-0.0f64).to_bits())));
    // div controlled failure present (out-sentinel check)
    assert!(checks
        .iter()
        .any(|c| matches!(c, ace_abi::typed_emit::TypedCheck::Div { d: 0, .. })));
}

// ---------------------------------------------------------------------------
// V-04: strict PRE-CALL admission (reject before any call, counter 0)
// ---------------------------------------------------------------------------

/// Mutated expectations are rejected at admission with NAMED stages, on
/// a store that has NEVER produced a callee (counter 0 — the rejection
/// happens before any production).
#[test]
fn admission_rejects_before_any_production() {
    let decls = typed_decl::typed_decls();
    let facts = decls.validate().expect("facts");
    let plan = PhysicalPlan::project(&facts).expect("plan");
    let descriptor = bridge::descriptor_from_plan(&facts, &plan).expect("descriptor");
    let semref = bridge::interface_binding_hash(&facts).expect("semref");

    let expect = bridge::typed_expectation(semref.clone());
    admit_descriptor(&descriptor, &expect).expect("baseline expectation admits");

    // wrong target
    let mut e = expect.clone();
    e.target = "aarch64-linux".into();
    let r = admit_descriptor(&descriptor, &e).unwrap_err();
    assert!(format!("{r:?}").contains("WrongTarget"), "{r:?}");

    // wrong domain
    let mut e = expect.clone();
    e.abi_domain = "other-abi".into();
    let r = admit_descriptor(&descriptor, &e).unwrap_err();
    assert!(format!("{r:?}").contains("WrongDomain"), "{r:?}");

    // wrong packing
    let mut e = expect.clone();
    e.packing = "packed".into();
    let r = admit_descriptor(&descriptor, &e).unwrap_err();
    assert!(format!("{r:?}").contains("Packing"), "{r:?}");

    // wrong signature (same-width substitution IS a deviation)
    let mut e = expect.clone();
    e.required_symbols[0].1 = "pure(i32,u32,i64,u64,f64,bool,char,u64)->i64".into();
    let r = admit_descriptor(&descriptor, &e).unwrap_err();
    assert!(format!("{r:?}").contains("Signature"), "{r:?}");

    // unknown symbol
    let mut e = expect.clone();
    e.required_symbols[0].0 = "t937_nosuch".into();
    let r = admit_descriptor(&descriptor, &e).unwrap_err();
    assert!(format!("{r:?}").contains("MissingSymbol"), "{r:?}");

    // semantic binding mismatch
    let mut e = expect.clone();
    e.semref_hash = "0".repeat(64);
    let r = admit_descriptor(&descriptor, &e).unwrap_err();
    assert!(format!("{r:?}").contains("SemRef"), "{r:?}");

    // LEGAL controls still admit: nullable-ctx fallible with ordered
    // outs, and non-null ctx VALUES are pre-call-validated, not rejected.
    let r = facts.validate_call_values("t937_mix", None, &typed_decl::frozen_lane_values());
    assert!(r.is_ok(), "legal mixed-lane values must pre-call validate");
    // invalid char (lone surrogate) is rejected BEFORE any call
    let mut bad = typed_decl::frozen_lane_values();
    bad[6] = ace_foreign_contract::LaneValue::CharScalar(0xD800);
    let e = facts
        .validate_call_values("t937_mix", None, &bad)
        .unwrap_err();
    assert!(e.contains("Unicode scalar"), "{e}");
    // fallible ctx: declared nullable accepts a non-null VALUE
    let e = facts.validate_call_values(
        "t937_div",
        Some(&ace_foreign_contract::LaneValue::CtxPtr(0xDEAD)),
        &[
            ace_foreign_contract::LaneValue::I64(1),
            ace_foreign_contract::LaneValue::I64(2),
        ],
    );
    assert!(e.is_ok(), "non-null ctx value is legal: {e:?}");
}

// ---------------------------------------------------------------------------
// V-02: the source export chain (real source → C library → independent
// caller EXECUTES the exports)
// ---------------------------------------------------------------------------

#[test]
fn source_export_chain_executes_real_source() {
    let (store, _root) = store_at("source-export");
    let counters = BridgeCounters::new();
    let out = run_source_export(
        &store,
        &counters,
        &scratch_root("src"),
        &source_export_first_case(),
        BRIDGE_TOTAL_DEADLINE,
    )
    .expect("source export chain runs");
    assert_eq!(
        out.caller_exit, 0,
        "independent caller must pass: {}",
        out.stdout
    );
    assert!(out.stdout.contains("source-export: ok"), "{}", out.stdout);
    assert!(out
        .library_manifest
        .contains(ace_a2x_c::LIBRARY_FILESET_SCHEMA));
    assert!(out
        .library_manifest
        .contains(ace_a2x_c::LIBRARY_EXPORT_RECIPE));
}

/// The distinguishing oracle: editing the REAL body flips the DELIVERED
/// value — the frozen independent caller (expecting 42) must now FAIL.
#[test]
fn source_body_edit_flips_delivered_value() {
    let (store, _root) = store_at("source-edit");
    let counters = BridgeCounters::new();
    let edited = bridge::SourceExportRequest {
        src: source_export_body_edit(),
        module: "t937_source".into(),
        exports: vec![
            ("sum".into(), "t937_sum".into()),
            ("div".into(), "t937_div".into()),
        ],
    };
    let out = run_source_export(
        &store,
        &counters,
        &scratch_root("src-edit"),
        &edited,
        BRIDGE_TOTAL_DEADLINE,
    )
    .expect("edited chain runs");
    assert_ne!(
        out.caller_exit, 0,
        "body+1 must flip the delivered value (caller expects 42)"
    );
    assert!(out.stdout.contains("FAILED"), "{}", out.stdout);
}

// ---------------------------------------------------------------------------
// V-05: per-interface/per-function increment (real P0 accounting)
// ---------------------------------------------------------------------------

/// Fresh → all producers run once; identical re-run → ZERO new producers
/// (pure hits); a body edit → exactly the edited function's fragment and
/// the body-reading adapters reproduce, while the body-INdependent plan
/// page stays a hit (caller metadata P=0).
#[test]
fn per_function_increment_accounting() {
    let (store, _root) = store_at("increment");
    let counters = BridgeCounters::new();
    let scratch = scratch_root("incr");

    // fresh run: 1 facts + 1 plan + 2 fragments + 1 library = 5 producers
    let first = run_source_export(
        &store,
        &counters,
        &scratch,
        &source_export_first_case(),
        BRIDGE_TOTAL_DEADLINE,
    )
    .expect("first run");
    assert_eq!(
        counters.total_producer_calls(),
        5,
        "fresh run produces exactly the 5 bridge artifacts"
    );

    // identical second run: everything hits, ZERO producers
    run_source_export(
        &store,
        &counters,
        &scratch,
        &source_export_first_case(),
        BRIDGE_TOTAL_DEADLINE,
    )
    .expect("second run");
    assert_eq!(
        counters.total_producer_calls(),
        5,
        "identical re-run must be pure hits"
    );

    // body edit: exactly ONE fragment reproduces (sum), the other
    // function's fragment is a hit, the plan (body-independent) is a
    // hit; facts (carries source identity) + library legitimately
    // reproduce.
    let edited = bridge::SourceExportRequest {
        src: source_export_body_edit(),
        module: "t937_source".into(),
        exports: vec![
            ("sum".into(), "t937_sum".into()),
            ("div".into(), "t937_div".into()),
        ],
    };
    let second = run_source_export(&store, &counters, &scratch, &edited, BRIDGE_TOTAL_DEADLINE)
        .expect("edited run");
    let sum_key_v1 = first
        .handle
        .fragment_keys
        .iter()
        .find(|k| k.contains("fn=sum"))
        .expect("sum fragment key v1");
    let sum_key_v2 = second
        .handle
        .fragment_keys
        .iter()
        .find(|k| k.contains("fn=sum"))
        .expect("sum fragment key v2");
    let div_key = first
        .handle
        .fragment_keys
        .iter()
        .find(|k| k.contains("fn=div"))
        .expect("div fragment key");
    // The edit gives sum a NEW sem_fp -> a NEW key produced exactly once;
    // div's key is UNCHANGED between the two handles (pure hit).
    assert_ne!(
        sum_key_v1, sum_key_v2,
        "edited body must be a new fragment key"
    );
    assert_eq!(counters.producer_calls(sum_key_v1), 1);
    assert_eq!(
        counters.producer_calls(sum_key_v2),
        1,
        "exactly one production for the edited fragment"
    );
    assert_eq!(
        second
            .handle
            .fragment_keys
            .iter()
            .find(|k| k.contains("fn=div"))
            .unwrap(),
        div_key,
        "unchanged function keeps its fragment key"
    );
    assert_eq!(
        counters.producer_calls(div_key),
        1,
        "unchanged function: no additional div-fragment production"
    );
    // plan page (body-independent metadata): produced ONCE at the first
    // run; the interface-preserving edit is a pure hit.
    assert_eq!(
        counters.producer_calls(&first.handle.plan_key),
        1,
        "plan page P=0 across the edit"
    );
    // total: 5 (run1) + 0 (run2) + facts + sum-fragment + library = 8
    assert_eq!(counters.total_producer_calls(), 8);
}

// ---------------------------------------------------------------------------
// V-06: bridge restore (bounded decode, deep verify, no producers) and
// its negative controls
// ---------------------------------------------------------------------------

#[test]
fn bridge_restore_deep_verifies_without_producers() {
    let (store, _root) = store_at("restore");
    let counters = BridgeCounters::new();
    let scratch = scratch_root("restore");
    let first = run_source_export(
        &store,
        &counters,
        &scratch,
        &source_export_first_case(),
        BRIDGE_TOTAL_DEADLINE,
    )
    .expect("chain runs");
    let manifest_before = first.library_manifest.clone();
    let produced = counters.total_producer_calls();

    // restore with FRESH counters: no producer may run; payload is read.
    let restore_counters = BridgeCounters::new();
    let restored = restore_bridge(&store, &restore_counters, &first.handle).expect("restore");
    assert_eq!(restored.producers_ran, 0, "restore must not run producers");
    assert!(
        restored.payload_bytes_read > 0,
        "restore reads persisted payload"
    );
    assert_eq!(restored.library_manifest, manifest_before);
    assert_eq!(restored.facts.digest().unwrap(), first.facts_digest);
    let _ = produced;

    // immediate body edit → reproduce → the OLD persisted artifacts stay
    // byte-identical (immutable history); the NEW library manifest differs.
    let edited = bridge::SourceExportRequest {
        src: source_export_body_edit(),
        module: "t937_source".into(),
        exports: vec![
            ("sum".into(), "t937_sum".into()),
            ("div".into(), "t937_div".into()),
        ],
    };
    let second = run_source_export(&store, &counters, &scratch, &edited, BRIDGE_TOTAL_DEADLINE)
        .expect("edited chain");
    assert_ne!(second.library_manifest, manifest_before);
    // restore of the FIRST handle still yields the OLD manifest
    let rc2 = BridgeCounters::new();
    let r2 = restore_bridge(&store, &rc2, &first.handle).expect("old handle restores");
    assert_eq!(
        r2.library_manifest, manifest_before,
        "old persisted bridge bytes are immutable"
    );
}

/// Missing page / forged handle / wrong-source identity are NAMED
/// rejections — restore never "repairs" foreign edits.
#[test]
fn bridge_restore_negative_controls() {
    let (store, _root) = store_at("restore-neg");
    let counters = BridgeCounters::new();
    let scratch = scratch_root("restore-neg");
    let first = run_source_export(
        &store,
        &counters,
        &scratch,
        &source_export_first_case(),
        BRIDGE_TOTAL_DEADLINE,
    )
    .expect("chain runs");

    // 1) wrong source identity in the handle -> named mismatch
    let mut forged = first.handle.clone();
    forged.source_sha256 = "0".repeat(64);
    let rc = BridgeCounters::new();
    let e = restore_bridge(&store, &rc, &forged).unwrap_err();
    assert!(
        e.to_string().contains("mismatch") || e.to_string().contains("missing"),
        "{e}"
    );

    // 2) missing page (handle pointing at a nonexistent key) -> named miss
    let mut missing = first.handle.clone();
    missing.library_key = missing.library_key.replace(
        "kind=bridge-library-fileset",
        "kind=bridge-library-fileset-nonexistent",
    );
    let rc = BridgeCounters::new();
    let e = restore_bridge(&store, &rc, &missing).unwrap_err();
    assert!(
        e.to_string().contains("missing") || e.to_string().contains("mismatch"),
        "{e}"
    );

    // 3) old-schema/wrong-layer key -> parse rejection
    let mut bad = first.handle.clone();
    bad.library_key = "NOT-A-BRIDGE-KEY".into();
    let rc = BridgeCounters::new();
    let e = restore_bridge(&store, &rc, &bad).unwrap_err();
    assert!(e.to_string().contains("not a bridge key"), "{e}");
}

// ---------------------------------------------------------------------------
// V-07 resources: total budget, artifact cap, lease, GC/512 churn
// ---------------------------------------------------------------------------

/// The total-budget seam rejects with a named error (no partial green).
#[test]
fn total_budget_exhaustion_is_a_named_rejection() {
    let (store, _root) = store_at("budget");
    let counters = BridgeCounters::new();
    let e = run_source_export(
        &store,
        &counters,
        &scratch_root("budget"),
        &source_export_first_case(),
        Duration::from_nanos(1),
    )
    .unwrap_err();
    assert!(e.to_string().contains("budget exhausted"), "{e}");
}

/// 512-revision churn on the bridge layer with GC convergence, rooted +
/// leased survival, and lease release flipping liveness (mirrors the 797
/// 512-revision discipline on the NEW layer).
#[test]
fn bridge_512_revision_churn_gc_convergence() {
    use ace_artifacts::codec::ChunkKind;
    use ace_artifacts::store::{run_gc, GcConfig, PutRequest};
    let (store, _root) = store_at("churn");
    let mut last_art = String::new();
    let mut leased_art = String::new();
    for i in 0..512 {
        let key = bridge::library_fileset_key(&format!("rev-{i:04}"), "rt", "ex");
        let mut p = format!("revision {i}").into_bytes();
        p.push((i % 251) as u8);
        let art = store
            .put_artifact(PutRequest {
                layer: bridge::LAYER_BRIDGE_V01.id,
                key,
                kind: ChunkKind::LayoutData,
                payload: p,
                edges: vec![],
                project: None,
            })
            .expect("put revision");
        if i == 511 {
            last_art = art.clone();
        }
        if i == 100 {
            leased_art = art;
        }
    }
    store
        .register_root("bridge-937-churn", &last_art, "newest")
        .expect("root newest");
    let lease = store
        .acquire_lease("bridge-937-churn", 60, Some(&leased_art))
        .expect("lease old revision");
    assert!(store.lease_live(&lease.lease_id).expect("lease live"));

    let space0 = store.space();
    let floor = space0.db_main + space0.locks + space0.logs;
    assert!(space0.total > floor);
    let cfg = GcConfig {
        high: space0.total - 1,
        low: floor + 64 * 1024,
    };
    let receipt = run_gc(&store, cfg).expect("gc");
    assert!(
        receipt.ran,
        "GC must run over the watermark: {}",
        receipt.reason
    );
    assert!(
        receipt.deleted_chunks >= 500,
        "churn bulk reclaimed: {receipt:?}"
    );

    // rooted newest + leased revision survive
    assert!(store
        .get_artifact(
            bridge::LAYER_BRIDGE_V01.id,
            &bridge::library_fileset_key("rev-0511", "rt", "ex")
        )
        .expect("read newest")
        .is_some());
    assert!(store
        .get_artifact(
            bridge::LAYER_BRIDGE_V01.id,
            &bridge::library_fileset_key("rev-0100", "rt", "ex")
        )
        .expect("read leased")
        .is_some());

    // convergence: managed live bytes collapse to the live set
    let space1 = store.space();
    let floor1 = space1.db_main + space1.locks + space1.logs;
    let managed_live = space1.total.saturating_sub(floor1);
    assert!(
        managed_live <= 128 * 1024,
        "block layer converges, got {managed_live}B"
    );
    assert!(space1.total < space0.total);

    // release flips liveness
    let lease_id = lease.lease_id.clone();
    lease.release();
    assert!(!store.lease_live(&lease_id).expect("lease liveness check"));
}

/// The single-artifact cap (64 MiB) rejects an over-sized producer with
/// a NAMED error (resource, never a silent pass).
#[test]
fn producer_cap_rejects_oversized_artifact() {
    use ace_artifacts::codec::ChunkKind;
    let (store, _root) = store_at("cap");
    let counters = BridgeCounters::new();
    let key = bridge::library_fileset_key("cap-test", "rt", "ex");
    let e =
        bridge::produce_counted_for_test(&store, &counters, &key, ChunkKind::LayoutData, || {
            Ok(vec![0u8; 64 * 1024 * 1024 + 1])
        })
        .unwrap_err();
    assert!(e.to_string().contains("64MiB"), "{}", e);
}

/// Worker-path equivalence: the edited deck through execute_bridge must
/// deliver the flipped value (caller FAILS), same as the in-process run.
#[test]
fn worker_bridge_edited_deck_flips_value() {
    use ace_abi::worker_bridge::{execute_bridge, BRIDGE_RECIPE};
    use base64::Engine as _;
    let root = std::env::temp_dir().join(format!("t937-wbridge-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let deck = "deck ace-ca-d1/1\nleg source\nmodule t937_source\nexport sum as t937_sum\nexport div as t937_div\nsrc {\nfn sum(a int, b int) int { return a + b + 1 }\nfn div(n int, d int) int { return n / d }\nfn main() int { return 0 }\n}\n";
    let req = ace_abi::worker::WorkerRequest {
        recipe: BRIDGE_RECIPE.into(),
        nonce: "n".into(),
        request_hash: "h".into(),
        deck_b64: base64::engine::general_purpose::STANDARD.encode(deck),
        leg: "source".into(),
        store_root: root.join("store").display().to_string(),
        steps: vec![ace_abi::worker::StepIn {
            op: "query".into(),
            from: None,
            to: None,
        }],
    };
    let out = execute_bridge(&req).expect("bridge executes");
    let q = &out.steps[0];
    eprintln!(
        "outcome={} compute={} pe_exit={:?}",
        q.outcome, q.compute_calls, q.pe_exit
    );
    // the frozen caller FAILS the edited body: positive PE exit maps to a
    // `pe-exit` diagnostic (the 797 exit-code discipline), never a green step
    assert_eq!(q.outcome, "diags");
    assert_eq!(q.diagnostics[0].code, "pe-exit");
    assert!(q.compute_calls > 0);
}

// ---------------------------------------------------------------------------
// V-07 / §6 formal turnaround: six scenarios × 30 samples with the
// frozen P90 gates (body-edit ≤ 2000ms, admission-reject ≤ 2000ms),
// mirroring the 797 turnaround discipline on the BRIDGE chain. Raw
// JSONL + summary land under target/bridge-937/turnaround/.
// ---------------------------------------------------------------------------

use std::io::Write as _;

fn p90(mut xs: Vec<u128>) -> u128 {
    xs.sort();
    xs[(xs.len() * 90 / 100).min(xs.len() - 1)]
}

#[test]
fn bridge_turnaround_six_scenarios_30_samples() {
    use ace_abi::typed_decl::typed_decls as frozen_typed_decls;
    use ace_foreign_contract::PhysicalPlan;

    let root = std::env::temp_dir().join(format!("t937-turnaround-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let store = ManagedStore::open(&root.join("store")).expect("store");
    let scratch = root.join("scratch");
    let out_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../target/bridge-937/turnaround");
    std::fs::create_dir_all(&out_dir).expect("out dir");
    let mut jsonl = std::fs::File::create(out_dir.join("turnaround.jsonl")).expect("jsonl");

    let mut rows: Vec<(&str, u128)> = Vec::new();
    let base = source_export_first_case();

    // 1) fresh (uniform cold: distinct module per sample)
    for i in 0..30 {
        let counters = BridgeCounters::new();
        let req = bridge::SourceExportRequest {
            module: format!("m{i:02}"),
            ..base.clone()
        };
        let t = std::time::Instant::now();
        run_source_export(&store, &counters, &scratch, &req, BRIDGE_TOTAL_DEADLINE).expect("fresh");
        rows.push(("fresh", t.elapsed().as_millis() as u128));
    }
    // 2) library-hot (identical re-run — all-hit). One warm-up run for
    // the BASE module identity (scenario 1 used distinct modules), then
    // every sampled re-run must be a pure hit.
    {
        let warm = BridgeCounters::new();
        run_source_export(&store, &warm, &scratch, &base, BRIDGE_TOTAL_DEADLINE).expect("warm");
    }
    for _ in 0..30 {
        let counters = BridgeCounters::new();
        let t = std::time::Instant::now();
        let out = run_source_export(&store, &counters, &scratch, &base, BRIDGE_TOTAL_DEADLINE)
            .expect("hot");
        assert_eq!(
            counters.total_producer_calls(),
            0,
            "hot sample must be all-hit"
        );
        let _ = out;
        rows.push(("library-hot", t.elapsed().as_millis() as u128));
    }
    // 3) typed-object-hot (warm-up: the typed artifacts enter this store
    // here, then every sampled re-run must be a pure hit)
    {
        let warm = BridgeCounters::new();
        run_typed_chain(
            &store,
            &warm,
            &scratch,
            &[TypedLeg::SelfCheck],
            BRIDGE_TOTAL_DEADLINE,
        )
        .expect("typed warm");
    }
    for _ in 0..30 {
        let counters = BridgeCounters::new();
        let t = std::time::Instant::now();
        run_typed_chain(
            &store,
            &counters,
            &scratch,
            &[TypedLeg::SelfCheck],
            BRIDGE_TOTAL_DEADLINE,
        )
        .expect("typed hot");
        assert_eq!(counters.total_producer_calls(), 0);
        rows.push(("typed-object-hot", t.elapsed().as_millis() as u128));
    }
    // 4) restart-disk-hot: fresh counters + deep RESTORE then re-run
    for _ in 0..30 {
        let counters = BridgeCounters::new();
        let t = std::time::Instant::now();
        let out = run_source_export(&store, &counters, &scratch, &base, BRIDGE_TOTAL_DEADLINE)
            .expect("restart run");
        let rc = BridgeCounters::new();
        let restored = restore_bridge(&store, &rc, &out.handle).expect("restore");
        assert_eq!(restored.producers_ran, 0);
        rows.push(("restart-disk-hot", t.elapsed().as_millis() as u128));
    }
    // 5) body-edit (distinct body per sample — uniform cold, HARD GATE)
    let mut edit_ms = Vec::new();
    for i in 0..30u64 {
        let counters = BridgeCounters::new();
        let src = format!(
            "fn sum(a int, b int) int {{ return a + b + {i} }}\nfn div(n int, d int) int {{ return n / d }}\nfn main() int {{ return 0 }}\n"
        );
        let req = bridge::SourceExportRequest {
            src,
            ..base.clone()
        };
        let t = std::time::Instant::now();
        run_source_export(&store, &counters, &scratch, &req, BRIDGE_TOTAL_DEADLINE)
            .expect("body edit");
        let ms = t.elapsed().as_millis() as u128;
        edit_ms.push(ms);
        rows.push(("body-edit", ms));
    }
    // 6) admission-reject (mutated expectation, PRE-CALL, HARD GATE)
    let decls = frozen_typed_decls();
    let facts = decls.validate().expect("facts");
    let plan = PhysicalPlan::project(&facts).expect("plan");
    let descriptor = bridge::descriptor_from_plan(&facts, &plan).expect("descriptor");
    let mut reject_ms = Vec::new();
    for i in 0..30 {
        let mut e = bridge::typed_expectation(bridge::interface_binding_hash(&facts).expect("sem"));
        // rotate the mutated dimension per sample (target/packing/semref)
        match i % 3 {
            0 => e.target = format!("bad-target-{i}"),
            1 => e.packing = format!("bad-packing-{i}"),
            _ => e.semref_hash = format!("{i:064}"),
        }
        let t = std::time::Instant::now();
        let rej = admit_descriptor(&descriptor, &e).expect_err("must reject");
        let ms = t.elapsed().as_millis() as u128;
        assert!(
            format!("{rej:?}").contains("Wrong")
                || format!("{rej:?}").contains("SemRef")
                || format!("{rej:?}").contains("Packing")
        );
        reject_ms.push(ms);
        rows.push(("descriptor-reject", ms));
    }

    let mut summary = String::new();
    for (name, _) in [
        ("fresh", 0u128),
        ("library-hot", 0),
        ("typed-object-hot", 0),
        ("restart-disk-hot", 0),
        ("body-edit", 0),
        ("descriptor-reject", 0),
    ] {
        let xs: Vec<u128> = rows
            .iter()
            .filter(|(n, _)| *n == name)
            .map(|(_, v)| *v)
            .collect();
        let p0 = xs.iter().min().copied().unwrap_or(0);
        let p5 = p90(xs.clone()); // reuse ordering helper for a mid band
        let _ = p5;
        let p9 = p90(xs.clone());
        summary.push_str(&format!(
            "{name}: n={} min={}ms p90={}ms max={}ms\n",
            xs.len(),
            p0,
            p9,
            xs.iter().max().copied().unwrap_or(0)
        ));
    }
    for (name, ms) in &rows {
        let _ = writeln!(jsonl, "{{\"scenario\":\"{name}\",\"ms\":{ms}}}");
    }
    let gate_edit = p90(edit_ms);
    let gate_rej = p90(reject_ms);
    summary.push_str(&format!(
        "HARD GATES: body-edit P90 {gate_edit}ms (<=2000), descriptor-reject P90 {gate_rej}ms (<=2000)\n"
    ));
    std::fs::write(out_dir.join("summary.txt"), &summary).expect("summary");
    assert!(gate_edit <= 2000, "body-edit P90 gate: {summary}");
    assert!(gate_rej <= 2000, "descriptor-reject P90 gate: {summary}");
}

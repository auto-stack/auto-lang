//! PLAN-939 S07 wide typed-resource bridge conformance: the wide chain
//! (element-count i64 view / RecMix wide-field closure / the original
//! wide synchronous callback) across the REAL peers.
//!
//! Every assertion is an EXECUTED observable — legs exit 0 only when the
//! delivered machine code matches the independent oracles (Rust here,
//! Cranelift bodies, and the hand-written freestanding C peers), and the
//! negative control proves a wrong expectation cannot pass vacuously.

#![cfg(windows)]

use ace_abi::bridge::{
    self, run_wide_caller_with_checks, run_wide_chain, wide_expectation, BridgeCounters,
    WideLeg, BRIDGE_TOTAL_DEADLINE,
};
use ace_abi::typed_decl::wide_decls;
use ace_abi::typed_emit::{frozen_wide_checks, WideCheck};
use ace_artifacts::store::ManagedStore;
use ace_foreign_contract::PhysicalPlan;

use std::path::PathBuf;

fn scratch_root(tag: &str) -> PathBuf {
    // per-test root: parallel tests in this binary must never delete one
    // another's in-flight scratch (the tag IS the isolation).
    let dir = std::env::temp_dir().join(format!("t939w-bridge-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    dir
}

fn store_at(tag: &str) -> (ManagedStore, PathBuf) {
    static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
    let _guard = LOCK.lock().unwrap_or_else(|p| p.into_inner());
    drop(_guard);
    let root = std::env::temp_dir().join(format!("t939w-store-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    (ManagedStore::open(&root).expect("store opens"), root)
}

/// All three wide legs (emitted↔emitted self-check, native→C with the
/// hand-written C callee, C→native with the hand-written C caller whose
/// callback travels the fnptr lane into native) execute green.
#[test]
fn wide_chain_three_legs_execute_green() {
    let (store, _root) = store_at("wide-legs");
    let counters = BridgeCounters::new();
    let out = run_wide_chain(
        &store,
        &counters,
        &scratch_root("wide"),
        &[
            WideLeg::SelfCheck,
            WideLeg::NativeToC,
            WideLeg::CToNative,
        ],
        BRIDGE_TOTAL_DEADLINE,
    )
    .expect("wide chain runs");
    for (leg, exit) in &out.legs {
        assert_eq!(*exit, 0, "leg {leg} must exit 0 (failed checks: {exit})");
    }
    assert_eq!(out.legs.len(), 3);
    // the wide callee was produced exactly once for all three legs
    let callee_key = bridge::wide_callee_key(&out.plan_digest).canonical();
    assert_eq!(counters.producer_calls(&callee_key), 1);
}

/// Negative control (the oracle discriminates): a caller baked with a
/// WRONG element-sum expectation must exit nonzero.
#[test]
fn wide_chain_negative_control_discriminates() {
    let (store, _root) = store_at("wide-neg");
    let counters = BridgeCounters::new();
    let mut checks = frozen_wide_checks();
    let WideCheck::Sum { expected, .. } = &mut checks[0] else {
        panic!("first wide check must be the frozen element sum");
    };
    *expected = expected.wrapping_add(1);
    let exit = run_wide_caller_with_checks(&store, &counters, &scratch_root("wide-mut"), &checks)
        .expect("mutated wide caller runs");
    assert_ne!(exit, 0, "a wrong expected value must FAIL the PE");
}

/// The wide descriptor projects the RESOURCE lane semantics — the
/// element-count view spells `buf:i64` (never a bare u64 len), the
/// record view spells its nominal + role, the callback spells its
/// physical fnptr type — and admits against the hand-frozen wide
/// expectation (V-01 len-unit + layout admission).
#[test]
fn wide_descriptor_spells_resource_lanes_and_admits() {
    let facts = wide_decls().validate().expect("wide decls validate");
    let plan = PhysicalPlan::project(&facts).expect("plan projects");
    let descriptor = bridge::descriptor_from_plan(&facts, &plan).expect("descriptor projects");
    let find = |sym: &str| {
        descriptor
            .symbols
            .iter()
            .find(|s| s.name == sym)
            .unwrap_or_else(|| panic!("symbol {sym} present"))
            .form
            .signature()
    };
    assert_eq!(find("t939_wide_sum"), "pure(buf:i64)->i64");
    assert_eq!(find("t939_crec_sum"), "pure(rec:RecMix:const)->i64");
    assert_eq!(
        find("t939_wide_fold"),
        "pure(callback(ctx:i64,i64)->i64,i64,buf:i64)->i64"
    );
    // the record table carries the RecMix closure (wide fields)...
    let rec = descriptor
        .records
        .iter()
        .find(|r| r.name == "RecMix")
        .expect("RecMix in the record table");
    assert_eq!(rec.fields.len(), 4);
    // ...and admission against the independent consumer expectation
    // (own closure + semantic ref) passes BEFORE any production.
    let semref = bridge::interface_binding_hash(&facts).expect("semref");
    let admission =
        ace_abi::validate::admit_descriptor(&descriptor, &wide_expectation(semref)).expect(
            "wide descriptor admits against the hand-frozen expectation",
        );
    assert_eq!(admission.admitted_symbols.len(), 3);
}

/// The frozen check set carries the boundaries: element-count prefixes
/// (len-unit evidence), the empty view, the U+10FFFF tag, the near-MIN
/// wide field, the wide-beyond-i32 ctx seed, and the empty fold.
#[test]
fn wide_frozen_inputs_carry_the_boundaries() {
    use ace_abi::typed_decl::t939w as w;
    assert_eq!(w::SUM_ITEMS[4], i64::MIN + 7);
    assert_eq!(w::SUM_ITEMS[6], i64::MIN);
    assert_eq!(w::SUM_ITEMS[7], i64::MAX);
    assert!(w::FOLD_CTX > i32::MAX as i64, "wide ctx beyond the i32 domain");
    assert_eq!(w::REC_TAG, 0x10FFFF);
    let checks = frozen_wide_checks();
    let sums = checks
        .iter()
        .filter(|c| matches!(c, WideCheck::Sum { .. }))
        .count();
    let folds = checks
        .iter()
        .filter(|c| matches!(c, WideCheck::Fold { .. }))
        .count();
    assert!(sums >= 4, "prefix/empty sum cases present");
    assert!(folds >= 3, "prefix/empty fold cases present");
}

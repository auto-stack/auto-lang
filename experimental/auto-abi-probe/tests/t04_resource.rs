//! PLAN-797 T-04: real ownership/error/callback fixtures with a REAL
//! ledger (plan §8 T-04; V-04/V-05). The provider is the C resource
//! module (interop_resource.c): private arena, per-slot live/generation
//! ledger, provider-owned destroy, Nth-allocation failure injection with
//! rollback, borrowed-buffer rules, and strictly synchronous callbacks.
//! Two independent consumers drive it: an AutoABI (Cranelift) caller and
//! a hand-written C caller with its own oracles.

#![cfg(windows)]

use auto_abi_probe::ctool;
use auto_abi_probe::emit::{emit_caller_suite, CbKind, SuiteCheck};
use auto_abi_probe::link::link_pe;
use auto_abi_probe::run;

use std::path::PathBuf;

fn manifest() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

const SRC_BYTES: &[u8] = &[1u8, 2, 3, 4, 5, 6, 7, 8];

/// The AutoABI consumer's full resource protocol script. Expected values
/// are derived here from the frozen contract (not read back from the
/// provider): ledger transitions are enumerated by hand.
fn resource_checks() -> Vec<SuiteCheck> {
    vec![
        // lifecycle: create -> set -> get -> destroy, ledger agrees
        SuiteCheck::HCreate {
            symbol: "p797_h_create",
            size: 64,
            dst_reg: 0,
            expect_fail: false,
        },
        SuiteCheck::HSet {
            symbol: "p797_h_set",
            handle_reg: 0,
            idx: 0,
            val: 4242,
            expect_status: 0,
        },
        SuiteCheck::HGet {
            symbol: "p797_h_get",
            handle_reg: 0,
            idx: 0,
            expected: 4242,
            expect_status: 0,
        },
        SuiteCheck::HDestroy {
            symbol: "p797_h_destroy",
            handle_reg: 0,
            expect_status: 0,
        },
        SuiteCheck::Ledger {
            symbol: "p797_ledger",
            expect_live: 0,
            expect_allocs: 1,
            expect_frees: 1,
        },
        // double destroy of the SAME handle is rejected (V-04)
        SuiteCheck::HDestroy {
            symbol: "p797_h_destroy",
            handle_reg: 0,
            expect_status: -1,
        },
        // forged handle (valid-looking index, wrong generation) rejected
        SuiteCheck::HCreate {
            symbol: "p797_h_create",
            size: 64,
            dst_reg: 1,
            expect_fail: false,
        },
        SuiteCheck::HDestroy {
            symbol: "p797_h_destroy",
            handle_reg: 1,
            expect_status: 0,
        },
        SuiteCheck::HCreate {
            symbol: "p797_h_create",
            size: 64,
            dst_reg: 2,
            expect_fail: false,
        },
        // reg 1 is now STALE (slot reused with a new generation)
        SuiteCheck::HDestroy {
            symbol: "p797_h_destroy",
            handle_reg: 1,
            expect_status: -1,
        },
        // h_get on a stale handle is likewise rejected, out untouched
        SuiteCheck::HGet {
            symbol: "p797_h_get",
            handle_reg: 1,
            idx: 0,
            expected: 0,
            expect_status: -1,
        },
        SuiteCheck::HDestroy {
            symbol: "p797_h_destroy",
            handle_reg: 2,
            expect_status: 0,
        },
        SuiteCheck::Ledger {
            symbol: "p797_ledger",
            expect_live: 0,
            expect_allocs: 3,
            expect_frees: 3,
        },
        // buffers (V-04): empty ok, null+nonzero rejected with out intact
        SuiteCheck::BufSumOk {
            symbol: "p797_buf_sum",
            bytes: &[],
            expected: 0,
        },
        SuiteCheck::BufSumNull {
            symbol: "p797_buf_sum",
            len: 8,
            sentinel: 777,
        },
        SuiteCheck::BufSumOk {
            symbol: "p797_buf_sum",
            bytes: SRC_BYTES,
            expected: 36,
        },
        // copy: capacity rejection keeps dst untouched; ok path copies
        SuiteCheck::BufCopyCap {
            symbol: "p797_buf_copy",
            cap: 4,
            src: SRC_BYTES,
        },
        SuiteCheck::BufCopyOk {
            symbol: "p797_buf_copy",
            src: SRC_BYTES,
            expect: SRC_BYTES,
        },
        // failure injection: the SECOND allocation inside two_step fails
        // -> partial allocation rolled back, caller out untouched (V-04)
        SuiteCheck::FailNth {
            symbol: "p797_fail_nth_alloc",
            n: 2,
        },
        SuiteCheck::TwoStep {
            symbol: "p797_two_step",
            x: 9,
            expect_status: -1,
            sentinel: 555666,
            dst_reg: None,
        },
        SuiteCheck::Ledger {
            symbol: "p797_ledger",
            expect_live: 0,
            expect_allocs: 4,
            expect_frees: 4,
        },
        // success composite: transient A released internally; B flows to
        // reg 3 and is destroyed through the provider's own entry
        SuiteCheck::TwoStep {
            symbol: "p797_two_step",
            x: 9,
            expect_status: 0,
            sentinel: 555666,
            dst_reg: Some(3),
        },
        SuiteCheck::HDestroy {
            symbol: "p797_h_destroy",
            handle_reg: 3,
            expect_status: 0,
        },
        SuiteCheck::Ledger {
            symbol: "p797_ledger",
            expect_live: 0,
            expect_allocs: 6,
            expect_frees: 6,
        },
        // callbacks (V-05): two DIFFERENT callbacks with explicit ctx in
        // sequence — the second result proves the first was never stored
        SuiteCheck::Fold {
            symbol: "p797_fold",
            cb: CbKind::Ctx1kAddItem,
            ctx: 7,
            items: &[3, -1, 10],
            expected: 3 * 7000 + 12,
        },
        SuiteCheck::Fold {
            symbol: "p797_fold",
            cb: CbKind::Ctx2kItem3,
            ctx: 7,
            items: &[3, -1, 10],
            expected: 3 * 14000 + 3 * 12,
        },
        SuiteCheck::StoredCb {
            symbol: "p797_stored_cb",
        },
    ]
}

/// Leg B shape: AutoABI (Cranelift) caller drives the C provider.
#[test]
fn abi_resource_autoabi_consumer_full_protocol() {
    let caller =
        emit_caller_suite("p797_res_caller", &resource_checks()).expect("emit resource caller");

    let dir = run::scratch("abi_resource_autoabi");
    let src = manifest().join("cside/interop_resource.c");
    let c_obj = dir.join("provider.obj");
    ctool::compile_c(&src, &manifest().join("cside"), &c_obj)
        .expect("cl compiles the resource provider");

    let caller_obj = dir.join("caller.obj");
    std::fs::write(&caller_obj, &caller).unwrap();
    let exe = link_pe(&dir, &[caller_obj, c_obj], "p797_entry").expect("link");
    let pe = run::run_pe(&exe).expect("run");
    assert_eq!(
        pe.code, 0,
        "resource protocol exit {} (flags); out={:?} err={:?}",
        pe.code, pe.stdout, pe.stderr
    );
}

/// Leg C shape: the hand-written C consumer with its OWN oracles drives
/// the same provider — independent consumer evidence for every V-04/V-05
/// rule including the generation-reuse stale-handle case.
#[test]
fn abi_resource_c_consumer_full_protocol() {
    let dir = run::scratch("abi_resource_c");
    let provider_src = manifest().join("cside/interop_resource.c");
    let caller_src = manifest().join("cside/interop_resource_caller.c");
    let provider_obj = dir.join("provider.obj");
    let caller_obj = dir.join("caller.obj");
    ctool::compile_c(&provider_src, &manifest().join("cside"), &provider_obj)
        .expect("cl compiles provider");
    ctool::compile_c(&caller_src, &manifest().join("cside"), &caller_obj)
        .expect("cl compiles C consumer");

    let exe = link_pe(&dir, &[caller_obj, provider_obj], "p797_res_entry").expect("link");
    let pe = run::run_pe(&exe).expect("run");
    assert_eq!(
        pe.code, 0,
        "C consumer flags; out={:?} err={:?}",
        pe.stdout, pe.stderr
    );
}

/// Missed-rollback control: WITHOUT injection the same two_step succeeds
/// and the ledger shows the transient release — running the injected
/// script's ledger expectations against an UNinjected run must fail
/// (exit != 0), proving the rollback checks are actually wired to the
/// real ledger rather than to constants.
#[test]
fn abi_resource_rollback_control_discriminates() {
    let checks = vec![
        // no FailNth here: two_step succeeds, but we assert the FAILURE
        // contract (status -1 + sentinel + live 0) — it must NOT hold.
        SuiteCheck::TwoStep {
            symbol: "p797_two_step",
            x: 9,
            expect_status: -1,
            sentinel: 555666,
            dst_reg: None,
        },
        SuiteCheck::Ledger {
            symbol: "p797_ledger",
            expect_live: 0,
            expect_allocs: 0,
            expect_frees: 0,
        },
    ];
    let caller = emit_caller_suite("p797_res_ctl", &checks).expect("emit control caller");
    let dir = run::scratch("abi_resource_ctl");
    let src = manifest().join("cside/interop_resource.c");
    let c_obj = dir.join("provider.obj");
    ctool::compile_c(&src, &manifest().join("cside"), &c_obj).unwrap();
    let caller_obj = dir.join("caller.obj");
    std::fs::write(&caller_obj, &caller).unwrap();
    let exe = link_pe(&dir, &[caller_obj, c_obj], "p797_entry").expect("link");
    let pe = run::run_pe(&exe).expect("run");
    assert_ne!(
        pe.code, 0,
        "uninjected run against failure expectations must fail"
    );
}

/// F-797-R4-04/05/06 permanent counterexamples, consumer-side: an
/// independent C oracle links BOTH providers (A + the renamed-shim B)
/// and proves the negative-index boundary, cross-provider rejection,
/// and real capacity reclamation.
#[test]
fn abi_resource_edges_independent_c_control() {
    let dir = run::scratch("abi_resource_edges");
    let cside = manifest().join("cside");
    let edges = dir.join("edges.obj");
    let prov_a = dir.join("provider_a.obj");
    let prov_b = dir.join("provider_b.obj");
    ctool::compile_c(&cside.join("interop_resource_edges.c"), &cside, &edges)
        .expect("cl compiles the edges consumer");
    ctool::compile_c(&cside.join("interop_resource.c"), &cside, &prov_a)
        .expect("cl compiles provider A");
    ctool::compile_c(&cside.join("interop_resource_b.c"), &cside, &prov_b)
        .expect("cl compiles provider B (renamed shim)");

    let exe =
        link_pe(&dir, &[edges, prov_a, prov_b], "p797_res_edges_entry").expect("link edges PE");
    let pe = run::run_pe(&exe).expect("run edges PE");
    assert_eq!(
        pe.code, 0,
        "edge rules (F04/F05/F06) must all hold; out={:?} err={:?}",
        pe.stdout, pe.stderr
    );
}

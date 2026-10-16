//! PLAN-797 T-03: full three-leg interop suite (plan §8 T-03; V-02,
//! V-03 call-side share). Every leg links objects from INDEPENDENT
//! producers (AutoABI emitter vs real cl.exe) into one real PE and
//! executes it; heterogeneous params/returns, nested C->AutoABI->C
//! chains, the fallible status/out contract, record views at frozen
//! offsets, and an observable counter are all exercised with
//! independent oracles, plus single-check negative controls that must
//! exit exactly 1 (proving the checks discriminate).

#![cfg(windows)]

use auto_abi_probe::build::{consume_and_build_caller, produce_callee_suite, suite_expectation};
use auto_abi_probe::ctool;
use auto_abi_probe::decl::{self, AbiScalar, FnDecl, BodyKind};
use auto_abi_probe::emit::SuiteCheck;
use auto_abi_probe::link::link_pe;
use auto_abi_probe::run;

use std::path::PathBuf;

fn manifest() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn compile_c(name: &str, case: &str) -> PathBuf {
    let dir = run::scratch(case);
    let src = manifest().join("cside").join(name);
    let obj = dir.join(name).with_extension("obj");
    ctool::compile_c(&src, &manifest().join("cside"), &obj)
        .unwrap_or_else(|e| panic!("cl compile {name}: {e}"));
    obj
}

fn write_obj(dir: &std::path::Path, name: &str, bytes: &[u8]) -> PathBuf {
    let p = dir.join(name);
    std::fs::write(&p, bytes).unwrap_or_else(|e| panic!("write {name}: {e}"));
    p
}

/// The AutoABI suite declaration set (mix/inner/facc/outer/div/rec/bump).
fn suite_defs() -> Vec<FnDecl> {
    vec![
        FnDecl { symbol: "p797_mix", params: decl::MIX_PARAMS, ret: AbiScalar::I64, body: BodyKind::MixV1 },
        FnDecl { symbol: "p797_inner", params: &[AbiScalar::I64], ret: AbiScalar::I64, body: BodyKind::InnerV1 },
        FnDecl { symbol: "p797_facc", params: &[AbiScalar::F64, AbiScalar::F64], ret: AbiScalar::F64, body: BodyKind::FAccV1 },
        FnDecl { symbol: "p797_outer", params: &[AbiScalar::I32, AbiScalar::F64], ret: AbiScalar::I64, body: BodyKind::OuterV1 },
        FnDecl { symbol: "p797_div", params: &[AbiScalar::I64, AbiScalar::I64, AbiScalar::Ptr], ret: AbiScalar::I32, body: BodyKind::FallibleDivV1 },
        FnDecl { symbol: "p797_rec_sum", params: &[AbiScalar::Ptr], ret: AbiScalar::I64, body: BodyKind::RecSumV1 },
        FnDecl { symbol: "p797_bump", params: &[AbiScalar::I32], ret: AbiScalar::I64, body: BodyKind::BumpV1 },
    ]
}

/// The full positive check set (expected values from the independent
/// Rust oracles). `prefix` selects the callee namespace: "" for the
/// AutoABI suite, "p797_c" for the C suite.
fn full_checks(prefix: &str) -> Vec<SuiteCheck> {
    let mix_expected = decl::oracle_mix_v1(&decl::FROZEN_INPUTS);
    vec![
        SuiteCheck::Mix {
            symbol: suffix(prefix, "mix"),
            inputs: decl::FROZEN_INPUTS,
            expected: mix_expected,
            swap_fh: false,
        },
        SuiteCheck::Facc {
            symbol: suffix(prefix, "facc"),
            a: decl::FROZEN_FACC.0,
            b: decl::FROZEN_FACC.1,
            expected: decl::oracle_facc_v1(decl::FROZEN_FACC.0, decl::FROZEN_FACC.1),
        },
        SuiteCheck::Outer {
            symbol: suffix(prefix, "outer"),
            a: decl::FROZEN_OUTER.0,
            b: decl::FROZEN_OUTER.1,
            expected: decl::oracle_outer_v1(decl::FROZEN_OUTER.0, decl::FROZEN_OUTER.1),
        },
        SuiteCheck::DivOk {
            symbol: suffix(prefix, "div"),
            n: -100,
            d: 7,
            expected_q: decl::oracle_div_v1(-100, 7).ok().unwrap(),
        },
        SuiteCheck::DivZero {
            symbol: suffix(prefix, "div"),
            n: 12345,
            sentinel: 47000,
        },
        SuiteCheck::RecSum {
            symbol: suffix(prefix, "rec_sum"),
            rec: decl::FROZEN_REC,
            expected: decl::oracle_rec_sum_v1(&decl::FROZEN_REC),
            bad_tag_offset: false,
        },
        SuiteCheck::BumpSeq {
            symbol: suffix(prefix, "bump"),
            xs: decl::FROZEN_BUMPS,
            expected_total: decl::oracle_bump_total(decl::FROZEN_BUMPS),
        },
    ]
}

fn suffix(prefix: &str, name: &str) -> &'static str {
    match (prefix, name) {
        ("", "mix") => "p797_mix",
        ("", "facc") => "p797_facc",
        ("", "outer") => "p797_outer",
        ("", "div") => "p797_div",
        ("", "rec_sum") => "p797_rec_sum",
        ("", "bump") => "p797_bump",
        ("p797_c", "mix") => "p797_cmix",
        ("p797_c", "facc") => "p797_cfacc",
        ("p797_c", "outer") => "p797_couter",
        ("p797_c", "div") => "p797_cdiv",
        ("p797_c", "rec_sum") => "p797_crec_sum",
        ("p797_c", "bump") => "p797_cbump",
        _ => unreachable!("unknown suite symbol {prefix}/{name}"),
    }
}

/// Leg A: AutoABI caller -> AutoABI callee, full suite through the
/// produce/consume recipes (descriptor admission gates the consumer
/// emit — the two sides never share a codegen unit).
#[test]
fn abi_calls_leg_a_full_suite() {
    let defs = suite_defs();
    let produced = produce_callee_suite("p797_a_callee", &defs, false).expect("produce");
    let descriptor_bytes = produced.descriptor.encode().unwrap();
    let (_admission, caller) = consume_and_build_caller(
        &descriptor_bytes,
        &suite_expectation(&defs),
        "p797_a_caller",
        &full_checks(""),
    )
    .expect("consumer admits and builds");

    let dir = run::scratch("abi_calls_leg_a");
    let objs = vec![
        write_obj(&dir, "caller.obj", &caller),
        write_obj(&dir, "callee.obj", &produced.obj),
    ];
    let exe = link_pe(&dir, &objs, "p797_entry").expect("link");
    let pe = run::run_pe(&exe).expect("run");
    assert_eq!(pe.code, 0, "leg A suite; out={:?} err={:?}", pe.stdout, pe.stderr);
}

/// Leg B: AutoABI caller -> C callee suite (real cl.exe object).
#[test]
fn abi_calls_leg_b_full_suite() {
    // The C suite has no `outer` (outer's nested target is AutoABI
    // side); use the C cdiv/cfacc/crec_sum/cbump/cmix surface.
    let checks = full_checks("p797_c")
        .into_iter()
        .filter(|c| !matches!(c, SuiteCheck::Outer { .. }))
        .collect::<Vec<_>>();
    let caller = auto_abi_probe::emit::emit_caller_suite("p797_b_caller", &checks)
        .expect("emit caller");

    let c_obj = compile_c("interop_callee.c", "abi_calls_leg_b");
    let dir = c_obj.parent().unwrap();
    let objs = vec![write_obj(dir, "caller.obj", &caller), c_obj.clone()];
    let exe = link_pe(dir, &objs, "p797_entry").expect("link");
    let pe = run::run_pe(&exe).expect("run");
    assert_eq!(pe.code, 0, "leg B suite; out={:?} err={:?}", pe.stdout, pe.stderr);
}

/// Leg C: C caller (own entry, own oracles, defines p797_inner) ->
/// AutoABI callee module that IMPORTS p797_inner: the nested chain is
/// C -> AutoABI -> C inside one real PE. The callee decl set therefore
/// EXCLUDES p797_inner (the C caller's object provides it).
#[test]
fn abi_calls_leg_c_full_suite_nested() {
    let defs = suite_defs()
        .into_iter()
        .filter(|d| d.symbol != "p797_inner")
        .collect::<Vec<_>>();
    let produced = produce_callee_suite("p797_c_callee", &defs, true).expect("produce");

    let c_obj = compile_c("interop_caller.c", "abi_calls_leg_c");
    let dir = c_obj.parent().unwrap();
    let objs = vec![c_obj.clone(), write_obj(dir, "callee.obj", &produced.obj)];
    let exe = link_pe(dir, &objs, "p797_c_suite_entry").expect("link");
    let pe = run::run_pe(&exe).expect("run");
    assert_eq!(pe.code, 0, "leg C suite; out={:?} err={:?}", pe.stdout, pe.stderr);
}

/// Negative control: swapped f/h inputs to mix must fail the baked
/// expectation — exit EXACTLY 1 (single check, so the exit code counts
/// the failed checks; a leg that cannot discriminate would exit 0).
#[test]
fn abi_calls_swapped_args_control_exits_1() {
    let defs = suite_defs();
    let produced = produce_callee_suite("p797_sw_callee", &defs, false).expect("produce");
    let descriptor_bytes = produced.descriptor.encode().unwrap();
    let checks = vec![SuiteCheck::Mix {
        symbol: "p797_mix",
        inputs: decl::FROZEN_INPUTS,
        expected: decl::oracle_mix_v1(&decl::FROZEN_INPUTS),
        swap_fh: true,
    }];
    let (_a, caller) = consume_and_build_caller(
        &descriptor_bytes,
        &suite_expectation(&defs),
        "p797_sw_caller",
        &checks,
    )
    .expect("admit + build");

    let dir = run::scratch("abi_calls_swapped");
    let objs = vec![
        write_obj(&dir, "caller.obj", &caller),
        write_obj(&dir, "callee.obj", &produced.obj),
    ];
    let exe = link_pe(&dir, &objs, "p797_entry").expect("link");
    let pe = run::run_pe(&exe).expect("run");
    assert_eq!(pe.code, 1, "swapped-args control must exit exactly 1");
}

/// Negative control: writing `tag` at a WRONG offset makes the callee's
/// frozen offsets diverge — exit exactly 1.
#[test]
fn abi_calls_wrong_record_offset_control_exits_1() {
    let defs = suite_defs();
    let produced = produce_callee_suite("p797_wo_callee", &defs, false).expect("produce");
    let descriptor_bytes = produced.descriptor.encode().unwrap();
    let checks = vec![SuiteCheck::RecSum {
        symbol: "p797_rec_sum",
        rec: decl::FROZEN_REC,
        expected: decl::oracle_rec_sum_v1(&decl::FROZEN_REC),
        bad_tag_offset: true,
    }];
    let (_a, caller) = consume_and_build_caller(
        &descriptor_bytes,
        &suite_expectation(&defs),
        "p797_wo_caller",
        &checks,
    )
    .expect("admit + build");

    let dir = run::scratch("abi_calls_wrong_offset");
    let objs = vec![
        write_obj(&dir, "caller.obj", &caller),
        write_obj(&dir, "callee.obj", &produced.obj),
    ];
    let exe = link_pe(&dir, &objs, "p797_entry").expect("link");
    let pe = run::run_pe(&exe).expect("run");
    assert_eq!(pe.code, 1, "wrong-offset control must exit exactly 1");
}

/// Negative control: expecting the total of SIX bumps after FIVE calls
/// (missed-call shape) must fail — the counter makes call counts
/// observable.
#[test]
fn abi_calls_missed_call_control_exits_1() {
    let defs = suite_defs();
    let produced = produce_callee_suite("p797_mc_callee", &defs, false).expect("produce");
    let descriptor_bytes = produced.descriptor.encode().unwrap();
    let expected_six = decl::oracle_bump_total(decl::FROZEN_BUMPS) + 5;
    let checks = vec![SuiteCheck::BumpSeq {
        symbol: "p797_bump",
        xs: decl::FROZEN_BUMPS,
        expected_total: expected_six,
    }];
    let (_a, caller) = consume_and_build_caller(
        &descriptor_bytes,
        &suite_expectation(&defs),
        "p797_mc_caller",
        &checks,
    )
    .expect("admit + build");

    let dir = run::scratch("abi_calls_missed");
    let objs = vec![
        write_obj(&dir, "caller.obj", &caller),
        write_obj(&dir, "callee.obj", &produced.obj),
    ];
    let exe = link_pe(&dir, &objs, "p797_entry").expect("link");
    let pe = run::run_pe(&exe).expect("run");
    assert_eq!(pe.code, 1, "missed-call control must exit exactly 1");
}

/// The consumer recipe refuses a tampered descriptor before ANY caller
/// object exists (V-01 economics inside the build path): wrong target in
/// the producer descriptor -> Rejection, no obj bytes.
#[test]
fn abi_calls_consumer_rejects_tampered_boundary() {
    let defs = suite_defs();
    let mut produced = produce_callee_suite("p797_tamper", &defs, false).expect("produce");
    produced.descriptor.target = "x86_64-unknown-linux-gnu".into();
    let bytes = produced.descriptor.encode().unwrap();
    let err = consume_and_build_caller(
        &bytes,
        &suite_expectation(&defs),
        "p797_tamper_caller",
        &full_checks(""),
    )
    .unwrap_err();
    assert!(matches!(
        err,
        auto_abi_probe::build::ConsumeError::Rejected(
            auto_abi_probe::validate::Rejection::WrongTarget { .. }
        )
    ));
}

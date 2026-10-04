//! PLAN-741 T-04/T-05 native execution tests: fixtures carrying zero-param
//! HIR test entries are lowered to COFF, linked with rust-lld, and executed;
//! exit codes are the observable results. Windows only — this is the real
//! native gate (plan §6). Run with `--test-threads=1` (per plan command).

#![cfg(windows)]

mod common;

fn build_and_run(fixture: &str, case: &str) -> i32 {
    let rel = format!("experimental/ac-core/fixtures/native/{}", fixture);
    match common::build_fixture(&rel, "d_entry") {
        Ok(obj) => match common::link_and_run(case, &obj) {
            Ok(code) => code,
            Err(e) => panic!("{}: {}", case, e),
        },
        Err(e) => panic!("{}: build failed:\n{}", fixture, e),
    }
}

#[test]
fn add_2_3_returns_5() {
    assert_eq!(build_and_run("add-2-3.atom", "add-2-3"), 5);
}

#[test]
fn add_neg2_3_returns_1() {
    assert_eq!(build_and_run("add-neg2-3.atom", "add-neg2-3"), 1);
}

#[test]
fn count_3_returns_3() {
    assert_eq!(build_and_run("count-3.atom", "count-3"), 3);
}

#[test]
fn count_0_returns_0() {
    assert_eq!(build_and_run("count-0.atom", "count-0"), 0);
}

#[test]
fn count_neg2_returns_0() {
    assert_eq!(build_and_run("count-neg2.atom", "count-neg2"), 0);
}

#[test]
fn bool_condition_selects_native_branch() {
    // QA-01: a bool comparison result stored into a bool local (let +
    // assign), then branch on it. r1 panicked the Cranelift frontend here
    // (icmp i8 into an i32 variable).
    assert_eq!(build_and_run("bool-if.atom", "bool-if"), 3);
}

#[test]
fn bool_abi_roundtrip_param_and_return() {
    // QA-01: bool across the call ABI — bool return value, bool argument,
    // bool param driving a branch. less(2,3)=true, less(7,2)=false, so
    // pick(true,5) + pick(false,9) = 5.
    assert_eq!(build_and_run("bool-abi.atom", "bool-abi"), 5);
}

#[test]
fn swapped_bindings_call_native() {
    // QA-02: swapped bindings with matching types must pass verify and run
    // natively — param0 receives eval_args[1] (i32 9), param1 the bool.
    assert_eq!(build_and_run("binding-swap.atom", "binding-swap"), 9);
}

#[test]
fn add_overflow_traps_70() {
    // i32::MAX + 1 must take the explicit trap path, not wrap to a normal
    // result.
    assert_eq!(
        build_and_run("overflow-add-max.atom", "overflow-add-max"),
        70
    );
}

#[test]
fn mul_overflow_traps_70() {
    assert_eq!(
        build_and_run("overflow-mul-max2.atom", "overflow-mul-max2"),
        70
    );
}

#[test]
fn object_exports_entry_and_start_symbols() {
    // The object must carry the generic ac_start wrapper and the entry
    // function's declared name so the PE links with /entry:ac_start.
    let rel = "experimental/ac-core/fixtures/native/add-2-3.atom";
    let obj = common::build_fixture(rel, "d_entry").expect("build");
    use object::{Object, ObjectSymbol};
    let data = object::read::File::parse(&obj[..]).expect("parse COFF");
    let mut symbols = std::collections::BTreeSet::new();
    for sym in data.symbols() {
        if let Ok(name) = sym.name() {
            symbols.insert(name.to_string());
        }
    }
    for expected in ["ac_start", "test_entry", "add", "ExitProcess"] {
        assert!(
            symbols.iter().any(|s| s.contains(expected)),
            "object missing symbol `{}` (has {:?})",
            expected,
            symbols
        );
    }
}

//! Independent third-review controls; uses committed harness/shim unchanged.
#![cfg(test)]
use ac_data_probe_774 as probe;

fn budget_case(case: &str, budget: i64) -> probe::ProbeOutcome {
    probe::run_probe(case, |cx| {
        cx.data("len", &[0; 4], true);
        cx.set_main(Box::new(move |fb, fx| {
            let outp = probe::data_addr(fb, fx, "len");
            let b = probe::const_ptr(fb, budget);
            probe::call_shim(fb, fx, "p774_set_budget", &[b]);
            let seed = probe::const_i32(fb, 7);
            let n = probe::const_i32(fb, 2);
            let p = probe::call_shim(fb, fx, "p774_own_make", &[seed, n, outp])[0];
            let v = probe::expect_non_null(fb, p);
            let got_len = probe::load_i32(fb, outp, 0);
            let vl = probe::expect_eq_i32(fb, got_len, 2);
            probe::call_shim(fb, fx, "p774_own_free", &[p]);
            let inv = probe::call_shim(fb, fx, "p774_check_invariant", &[])[0];
            let x = probe::acc(fb, v, vl);
            probe::acc(fb, x, inv)
        }));
    })
}

#[test]
fn maximum_u32_budget_control() {
    let out = budget_case("review-r3-budget-u32-max", 0xFFFF_FFFF);
    probe::assert_pass(&out, &[]);
    assert_eq!(probe::summary_of(&out), (1, 1, 1, 0, 0, 2));
}

#[test]
fn i64_budget_4gib_must_not_turn_into_zero() {
    let out = budget_case("review-r3-budget-4gib", 0x1_0000_0000);
    probe::assert_pass(&out, &[]);
    assert_eq!(probe::summary_of(&out), (1, 1, 1, 0, 0, 2));
}

#[test]
fn i64_budget_4gib_plus_one_must_not_turn_into_one() {
    let out = budget_case("review-r3-budget-4gib-plus1", 0x1_0000_0001);
    probe::assert_pass(&out, &[]);
    assert_eq!(probe::summary_of(&out), (1, 1, 1, 0, 0, 2));
}

#[test]
fn growth_failure_retry_really_crosses_capacity_with_all_owners_live() {
    let out = probe::run_probe("review-r3-real-growth-retry", |cx| {
        cx.data("len", &[0; 4], true);
        cx.set_main(Box::new(|fb, fx| {
            let outp = probe::data_addr(fb, fx, "len");
            let seed = probe::const_i32(fb, 7);
            let one = probe::const_i32(fb, 1);
            let two = probe::const_i32(fb, 2);
            probe::call_shim(fb, fx, "p774_fail_ledger_nth", &[two]);
            let mut owners = Vec::new();
            for _ in 0..16 {
                owners.push(probe::call_shim(fb, fx, "p774_own_make", &[seed, one, outp])[0]);
            }
            let failed = probe::call_shim(fb, fx, "p774_own_make", &[seed, one, outp])[0];
            let mut v = probe::expect_null(fb, failed);
            let code = probe::load_i32(fb, outp, 0);
            let vc = probe::expect_eq_i32(fb, code, -5);
            v = probe::acc(fb, v, vc);
            let zero = probe::const_i32(fb, 0);
            probe::call_shim(fb, fx, "p774_fail_ledger_nth", &[zero]);
            // No owner is freed before retry: inserting 17 must actually grow.
            let retry = probe::call_shim(fb, fx, "p774_own_make", &[seed, one, outp])[0];
            let nn = probe::expect_non_null(fb, retry);
            v = probe::acc(fb, v, nn);
            owners.push(retry);
            for &p in &owners {
                let got = probe::call_shim(fb, fx, "p774_borrow_read", &[p, one, zero])[0];
                let vr = probe::expect_eq_i32(fb, got, 7);
                v = probe::acc(fb, v, vr);
            }
            for p in owners.into_iter().rev() { probe::call_shim(fb, fx, "p774_own_free", &[p]); }
            let inv = probe::call_shim(fb, fx, "p774_check_invariant", &[])[0];
            probe::acc(fb, v, inv)
        }));
    });
    probe::assert_pass(&out, &[]);
    assert_eq!(probe::summary_of(&out), (18, 17, 17, 0, 0, 17));
}

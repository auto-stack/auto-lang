//! V-05: fault injection at every declared allocation point and the failure
//! category matrix (probe-cases.md §V-05; decisions.md §D-01/D-06). Every
//! failure crosses the ABI as a VALUE (exit 0); already-owned resources are
//! reclaimed per contract (live=0 on normal control flow).

#![cfg(windows)]

use ac_data_probe_774 as probe;

/// V05-1: AP-1 (owned construction) allocation failure.
#[test]
fn v05_1_make_fails() {
    let out = probe::run_probe("v05-1", |cx| {
        cx.data("outlen", &[0; 4], true);
        cx.set_main(Box::new(|fb, fx| {
            let outp = probe::data_addr(fb, fx, "outlen");
            let one = probe::const_i32(fb, 1);
            probe::call_shim(fb, fx, "p774_fail_nth_alloc", &[one]);
            let seed = probe::const_i32(fb, 7);
            let len = probe::const_i32(fb, 16);
            let p = probe::call_shim(fb, fx, "p774_own_make", &[seed, len, outp]).remove(0);
            let vn = probe::expect_null(fb, p);
            let code = probe::load_i32(fb, outp, 0);
            let vc = probe::expect_eq_i32(fb, code, probe::status::ERR_ENV_OOM);
            let inv = probe::call_shim(fb, fx, "p774_check_invariant", &[]).remove(0);
            let v = probe::acc(fb, vn, vc);
            probe::acc(fb, v, inv)
        }));
    });
    probe::assert_pass(&out, &["OWN-MAKE seed=7 len=16"]);
    assert_eq!(probe::summary_of(&out), (1, 0, 0, 0, 0, 0));
}

/// V05-2: AP-2 (clone) failure — the original is still reclaimed.
#[test]
fn v05_2_clone_fails_original_freed() {
    let out = probe::run_probe("v05-2", |cx| {
        cx.data("outlen", &[0; 4], true);
        cx.set_main(Box::new(|fb, fx| {
            let outp = probe::data_addr(fb, fx, "outlen");
            let two = probe::const_i32(fb, 2);
            probe::call_shim(fb, fx, "p774_fail_nth_alloc", &[two]);
            let seed = probe::const_i32(fb, 7);
            let len = probe::const_i32(fb, 16);
            let p = probe::call_shim(fb, fx, "p774_own_make", &[seed, len, outp]).remove(0);
            let v1 = probe::expect_non_null(fb, p);
            let q = probe::call_shim(fb, fx, "p774_own_clone", &[p, len, outp]).remove(0);
            let v2 = probe::expect_null(fb, q);
            let code = probe::load_i32(fb, outp, 0);
            let v3 = probe::expect_eq_i32(fb, code, probe::status::ERR_ENV_OOM);
            probe::call_shim(fb, fx, "p774_own_free", &[p]);
            let inv = probe::call_shim(fb, fx, "p774_check_invariant", &[]).remove(0);
            let v = probe::acc(fb, v1, v2);
            let v = probe::acc(fb, v, v3);
            probe::acc(fb, v, inv)
        }));
    });
    probe::assert_pass(&out, &["CLONE len=16"]);
    assert_eq!(probe::summary_of(&out), (2, 1, 1, 0, 0, 16));
}

/// V05-3: AP-3 (concat build) failure.
#[test]
fn v05_3_concat_fails() {
    let out = probe::run_probe("v05-3", |cx| {
        cx.data("a", b"Ab", false);
        cx.data("b", b"cd", false);
        cx.data("outlen", &[0; 4], true);
        cx.set_main(Box::new(|fb, fx| {
            let pa = probe::data_addr(fb, fx, "a");
            let pb = probe::data_addr(fb, fx, "b");
            let outp = probe::data_addr(fb, fx, "outlen");
            let one = probe::const_i32(fb, 1);
            probe::call_shim(fb, fx, "p774_fail_nth_alloc", &[one]);
            let la = probe::const_i32(fb, 2);
            let lb = probe::const_i32(fb, 2);
            let r = probe::call_shim(fb, fx, "p774_own_concat", &[pa, la, pb, lb, outp]).remove(0);
            let vn = probe::expect_null(fb, r);
            let code = probe::load_i32(fb, outp, 0);
            let vc = probe::expect_eq_i32(fb, code, probe::status::ERR_ENV_OOM);
            let inv = probe::call_shim(fb, fx, "p774_check_invariant", &[]).remove(0);
            let v = probe::acc(fb, vn, vc);
            probe::acc(fb, v, inv)
        }));
    });
    probe::assert_pass(&out, &["CONCAT a=2 b=2"]);
    assert_eq!(probe::summary_of(&out), (1, 0, 0, 0, 0, 0));
}

/// V05-4: AP-4 (literal promotion) failure.
#[test]
fn v05_4_lit_promotion_fails() {
    let out = probe::run_probe("v05-4", |cx| {
        cx.data("lit", b"Auto", false);
        cx.data("outlen", &[0; 4], true);
        cx.set_main(Box::new(|fb, fx| {
            let lp = probe::data_addr(fb, fx, "lit");
            let outp = probe::data_addr(fb, fx, "outlen");
            let one = probe::const_i32(fb, 1);
            probe::call_shim(fb, fx, "p774_fail_nth_alloc", &[one]);
            let four = probe::const_i32(fb, 4);
            let r = probe::call_shim(fb, fx, "p774_own_from_lit", &[lp, four, outp]).remove(0);
            let vn = probe::expect_null(fb, r);
            let code = probe::load_i32(fb, outp, 0);
            let vc = probe::expect_eq_i32(fb, code, probe::status::ERR_ENV_OOM);
            let inv = probe::call_shim(fb, fx, "p774_check_invariant", &[]).remove(0);
            let v = probe::acc(fb, vn, vc);
            probe::acc(fb, v, inv)
        }));
    });
    probe::assert_pass(&out, &["LIT len=4"]);
    assert_eq!(probe::summary_of(&out), (1, 0, 0, 0, 0, 0));
}

/// V05-5: zero length is valid — no allocator call at all.
#[test]
fn v05_5_zero_length() {
    let out = probe::run_probe("v05-5", |cx| {
        cx.data("outlen", &[0; 4], true);
        cx.set_main(Box::new(|fb, fx| {
            let outp = probe::data_addr(fb, fx, "outlen");
            let seed = probe::const_i32(fb, 7);
            let zero = probe::const_i32(fb, 0);
            let p = probe::call_shim(fb, fx, "p774_own_make", &[seed, zero, outp]).remove(0);
            let vn = probe::expect_null(fb, p);
            let code = probe::load_i32(fb, outp, 0);
            let vc = probe::expect_eq_i32(fb, code, 0);
            probe::call_shim(fb, fx, "p774_own_free", &[p]); // free(null) is a no-op
            let inv = probe::call_shim(fb, fx, "p774_check_invariant", &[]).remove(0);
            let v = probe::acc(fb, vn, vc);
            probe::acc(fb, v, inv)
        }));
    });
    probe::assert_pass(&out, &["OWN-MAKE seed=7 len=0"]);
    assert_eq!(probe::summary_of(&out), (0, 0, 0, 0, 0, 0));
}

/// V05-6: concat length overflow is rejected BEFORE any buffer read or
/// allocation attempt (attempts delta 0).
#[test]
fn v05_6_concat_overflow() {
    let out = probe::run_probe("v05-6", |cx| {
        cx.data("a", b"Ab", false);
        cx.data("b", b"cd", false);
        cx.data("outlen", &[0; 4], true);
        cx.set_main(Box::new(|fb, fx| {
            let pa = probe::data_addr(fb, fx, "a");
            let pb = probe::data_addr(fb, fx, "b");
            let outp = probe::data_addr(fb, fx, "outlen");
            let half = probe::const_i32(fb, 0x4000_0000);
            let r =
                probe::call_shim(fb, fx, "p774_own_concat", &[pa, half, pb, half, outp]).remove(0);
            let vn = probe::expect_null(fb, r);
            let code = probe::load_i32(fb, outp, 0);
            let vc = probe::expect_eq_i32(fb, code, probe::status::ERR_LEN_OVERFLOW);
            let inv = probe::call_shim(fb, fx, "p774_check_invariant", &[]).remove(0);
            let v = probe::acc(fb, vn, vc);
            probe::acc(fb, v, inv)
        }));
    });
    probe::assert_pass(&out, &["CONCAT a=1073741824 b=1073741824"]);
    assert_eq!(probe::summary_of(&out), (0, 0, 0, 0, 0, 0));
}

/// V05-7/8/9: conversion boundaries and the explicit shim error channel.
#[test]
fn v05_7_8_9_boundaries_and_shim_error() {
    let out = probe::run_probe("v05-789", |cx| {
        cx.set_main(Box::new(|fb, fx| {
            let neg = probe::const_i32(fb, -5);
            let r1 = probe::call_shim(fb, fx, "p774_len_from_i32", &[neg]).remove(0);
            let v1 = probe::expect_eq_i32(fb, r1, probe::status::ERR_LEN_NEGATIVE);
            let big = probe::const_ptr(fb, 0x8000_0000);
            let r2 = probe::call_shim(fb, fx, "p774_len_to_i32", &[big]).remove(0);
            let v2 = probe::expect_eq_i32(fb, r2, probe::status::ERR_LEN_TOO_LARGE);
            let r3 = probe::call_shim(fb, fx, "p774_shim_error", &[]).remove(0);
            let v3 = probe::expect_eq_i32(fb, r3, probe::status::ERR_SHIM);
            let v = probe::acc(fb, v1, v2);
            probe::acc(fb, v, v3)
        }));
    });
    probe::assert_pass(&out, &[]);
}

/// V05-10: concat success counter-control (the real bytes, the real sum).
#[test]
fn v05_10_concat_success() {
    let out = probe::run_probe("v05-10", |cx| {
        cx.data("a", b"Ab", false);
        cx.data("b", b"cd", false);
        cx.data("outlen", &[0; 4], true);
        cx.set_main(Box::new(|fb, fx| {
            let pa = probe::data_addr(fb, fx, "a");
            let pb = probe::data_addr(fb, fx, "b");
            let outp = probe::data_addr(fb, fx, "outlen");
            let la = probe::const_i32(fb, 2);
            let lb = probe::const_i32(fb, 2);
            let r = probe::call_shim(fb, fx, "p774_own_concat", &[pa, la, pb, lb, outp]).remove(0);
            let vn = probe::expect_non_null(fb, r);
            let len = probe::load_i32(fb, outp, 0);
            let vl = probe::expect_eq_i32(fb, len, 4);
            let sum = probe::call_shim(fb, fx, "p774_borrow_sum", &[r, len]).remove(0);
            let vs = probe::expect_eq_i32(fb, sum, 362);
            probe::call_shim(fb, fx, "p774_own_free", &[r]);
            let inv = probe::call_shim(fb, fx, "p774_check_invariant", &[]).remove(0);
            let v = probe::acc(fb, vn, vl);
            let v = probe::acc(fb, v, vs);
            probe::acc(fb, v, inv)
        }));
    });
    probe::assert_pass(&out, &["CONCAT a=2 b=2"]);
    assert_eq!(probe::summary_of(&out), (1, 1, 1, 0, 0, 4));
}

/// V05-11: borrow_read bounds.
#[test]
fn v05_11_borrow_read_bounds() {
    let out = probe::run_probe("v05-11", |cx| {
        cx.data("outlen", &[0; 4], true);
        cx.set_main(Box::new(|fb, fx| {
            let outp = probe::data_addr(fb, fx, "outlen");
            let seed = probe::const_i32(fb, 7);
            let len = probe::const_i32(fb, 16);
            let p = probe::call_shim(fb, fx, "p774_own_make", &[seed, len, outp]).remove(0);
            let sixteen = probe::const_i32(fb, 16);
            let r1 = probe::call_shim(fb, fx, "p774_borrow_read", &[p, sixteen, sixteen]).remove(0);
            let v1 = probe::expect_eq_i32(fb, r1, probe::status::ERR_INDEX_OOB);
            let neg = probe::const_i32(fb, -1);
            let r2 = probe::call_shim(fb, fx, "p774_borrow_read", &[p, sixteen, neg]).remove(0);
            let v2 = probe::expect_eq_i32(fb, r2, probe::status::ERR_INDEX_OOB);
            probe::call_shim(fb, fx, "p774_own_free", &[p]);
            let inv = probe::call_shim(fb, fx, "p774_check_invariant", &[]).remove(0);
            let v = probe::acc(fb, v1, v2);
            probe::acc(fb, v, inv)
        }));
    });
    probe::assert_pass(&out, &[]);
}

/// V05-12: category distinctness matrix — the frozen runtime status codes
/// are pairwise distinct, the model codes live in a separate namespace, and
/// the fatal trap is neither (an OS exit code, not a status value).
#[test]
fn v05_12_category_matrix() {
    let codes = [
        probe::status::ERR_LEN_OVERFLOW,
        probe::status::ERR_LEN_NEGATIVE,
        probe::status::ERR_LEN_TOO_LARGE,
        probe::status::ERR_BUDGET,
        probe::status::ERR_ENV_OOM,
        probe::status::ERR_SHIM,
        probe::status::ERR_INVALID_UTF8,
        probe::status::ERR_INVALID_SCALAR,
        probe::status::ERR_RANGE_REVERSED,
        probe::status::ERR_RANGE_OOB,
        probe::status::ERR_INDEX_OOB,
    ];
    for i in 0..codes.len() {
        for j in (i + 1)..codes.len() {
            assert_ne!(codes[i], codes[j], "runtime status codes must be distinct");
        }
    }
    // Model (front-door) codes are a disjoint namespace; the trap exit code
    // (0xC0000005) is an OS termination, neither a status nor a rejection.
    for m in ac_data_probe_774::rules::REJECTION_CODES {
        assert!(m.starts_with("verify."), "model code namespace: {m}");
        assert!(!m.contains("depth"), "no depth-based rejection exists: {m}");
    }
}

/// V05-13 (review r1 P774-R1-03): per-point injection WITH another owned
/// resource already live — the surviving owner is still reclaimed.
#[test]
fn v05_13_ap3_ap4_fault_with_live_owner() {
    // AP-4 (literal promotion) fails while a 16-byte buffer is live
    let lit = probe::run_probe("v05-13-lit", |cx| {
        cx.data("lit", b"Auto", false);
        cx.data("outlen", &[0; 4], true);
        cx.set_main(Box::new(|fb, fx| {
            let outp = probe::data_addr(fb, fx, "outlen");
            let two = probe::const_i32(fb, 2);
            probe::call_shim(fb, fx, "p774_fail_nth_alloc", &[two]);
            let seed = probe::const_i32(fb, 7);
            let len = probe::const_i32(fb, 16);
            let p = probe::call_shim(fb, fx, "p774_own_make", &[seed, len, outp]).remove(0);
            let v1 = probe::expect_non_null(fb, p);
            let lp = probe::data_addr(fb, fx, "lit");
            let four = probe::const_i32(fb, 4);
            let q = probe::call_shim(fb, fx, "p774_own_from_lit", &[lp, four, outp]).remove(0);
            let v2 = probe::expect_null(fb, q);
            let code = probe::load_i32(fb, outp, 0);
            let v3 = probe::expect_eq_i32(fb, code, probe::status::ERR_ENV_OOM);
            probe::call_shim(fb, fx, "p774_own_free", &[p]);
            let inv = probe::call_shim(fb, fx, "p774_check_invariant", &[]).remove(0);
            let v = probe::acc(fb, v1, v2);
            let v = probe::acc(fb, v, v3);
            probe::acc(fb, v, inv)
        }));
    });
    probe::assert_pass(&lit, &["LIT len=4"]);
    assert_eq!(probe::summary_of(&lit), (2, 1, 1, 0, 0, 16));

    // AP-3 (concat build) fails while a 16-byte buffer is live
    let cat = probe::run_probe("v05-13-concat", |cx| {
        cx.data("a", b"Ab", false);
        cx.data("b", b"cd", false);
        cx.data("outlen", &[0; 4], true);
        cx.set_main(Box::new(|fb, fx| {
            let outp = probe::data_addr(fb, fx, "outlen");
            let two = probe::const_i32(fb, 2);
            probe::call_shim(fb, fx, "p774_fail_nth_alloc", &[two]);
            let seed = probe::const_i32(fb, 7);
            let len = probe::const_i32(fb, 16);
            let p = probe::call_shim(fb, fx, "p774_own_make", &[seed, len, outp]).remove(0);
            let v1 = probe::expect_non_null(fb, p);
            let pa = probe::data_addr(fb, fx, "a");
            let pb = probe::data_addr(fb, fx, "b");
            let la = probe::const_i32(fb, 2);
            let lb = probe::const_i32(fb, 2);
            let q = probe::call_shim(fb, fx, "p774_own_concat", &[pa, la, pb, lb, outp]).remove(0);
            let v2 = probe::expect_null(fb, q);
            let code = probe::load_i32(fb, outp, 0);
            let v3 = probe::expect_eq_i32(fb, code, probe::status::ERR_ENV_OOM);
            probe::call_shim(fb, fx, "p774_own_free", &[p]);
            let inv = probe::call_shim(fb, fx, "p774_check_invariant", &[]).remove(0);
            let v = probe::acc(fb, v1, v2);
            let v = probe::acc(fb, v, v3);
            probe::acc(fb, v, inv)
        }));
    });
    probe::assert_pass(&cat, &["CONCAT a=2 b=2"]);
    assert_eq!(probe::summary_of(&cat), (2, 1, 1, 0, 0, 16));
}

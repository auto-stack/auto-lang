//! Independent PLAN-774 r3 review controls; no implementation modifications.
//! Run via an external/ignored manifest pointing its [lib] to this file.
#![cfg(test)]
use ac_data_probe_774 as probe;
use probe::rules::{run_script, Op};

#[test]
fn borrowed_return_requires_live_provenance() {
    let out = run_script(&[Op::Make(0), Op::Free(0), Op::ReturnViewParam(0)]);
    assert!(
        out.rejection.is_some(),
        "freed local slot accepted as returned parameter view"
    );
}

#[test]
fn active_views_and_mut_controls() {
    assert!(run_script(&[
        Op::Make(0),
        Op::View(0),
        Op::View(0),
        Op::EndView(0),
        Op::EndView(0),
        Op::Mut(0),
        Op::EndMut(0),
        Op::Free(0)
    ])
    .rejection
    .is_none());
    for tail in [Op::Mut(0), Op::View(0), Op::MoveOut(0), Op::Free(0)] {
        assert_eq!(
            run_script(&[Op::Make(0), Op::Mut(0), tail])
                .rejection
                .unwrap()
                .code,
            "verify.mut-conflict"
        );
    }
}

fn live_allocations(n: usize) -> probe::ProbeOutcome {
    probe::run_probe(&format!("review-r1-live-{n}"), |cx| {
        cx.data("len", &[0; 4], true);
        cx.set_main(Box::new(move |fb, fx| {
            let outp = probe::data_addr(fb, fx, "len");
            let one = probe::const_i32(fb, 1);
            let mut ptrs = Vec::new();
            for _ in 0..n {
                ptrs.push(probe::call_shim(fb, fx, "p774_own_make", &[one, one, outp])[0]);
            }
            for p in ptrs.into_iter().rev() {
                probe::call_shim(fb, fx, "p774_own_free", &[p]);
            }
            probe::call_shim(fb, fx, "p774_check_invariant", &[])[0]
        }));
    })
}

#[test]
fn sixty_four_live_control() {
    let out = live_allocations(64);
    probe::assert_pass(&out, &[]);
    assert_eq!(probe::summary_of(&out), (64, 64, 64, 0, 0, 64));
}

#[test]
fn sixty_five_live_resources_restore_budget() {
    let out = live_allocations(65);
    eprintln!("65-live PE code={:?}\n{}", out.code, out.stderr);
    probe::assert_pass(&out, &[]);
    assert_eq!(probe::summary_of(&out), (65, 65, 65, 0, 0, 65));
}

#[test]
fn released_sixty_five_restore_full_byte_allowance() {
    let out = probe::run_probe("review-r1-budget-after-65", |cx| {
        cx.data("len", &[0; 4], true);
        cx.set_main(Box::new(|fb, fx| {
            let outp = probe::data_addr(fb, fx, "len");
            let budget = probe::const_ptr(fb, 65);
            probe::call_shim(fb, fx, "p774_set_budget", &[budget]);
            let one = probe::const_i32(fb, 1);
            let mut ptrs = Vec::new();
            for _ in 0..65 {
                ptrs.push(probe::call_shim(fb, fx, "p774_own_make", &[one, one, outp])[0]);
            }
            for p in ptrs.into_iter().rev() {
                probe::call_shim(fb, fx, "p774_own_free", &[p]);
            }
            let n = probe::const_i32(fb, 65);
            let p = probe::call_shim(fb, fx, "p774_own_make", &[one, n, outp])[0];
            let v = probe::expect_non_null(fb, p);
            probe::call_shim(fb, fx, "p774_own_free", &[p]);
            let inv = probe::call_shim(fb, fx, "p774_check_invariant", &[])[0];
            probe::acc(fb, v, inv)
        }));
    });
    probe::assert_pass(&out, &[]);
    assert_eq!(probe::summary_of(&out), (66, 66, 66, 0, 0, 65));
}

#[test]
fn zero_byte_budget_rejects_one_byte() {
    let out = probe::run_probe("review-r1-zero-budget", |cx| {
        cx.data("len", &[0; 4], true);
        cx.set_main(Box::new(|fb, fx| {
            let outp = probe::data_addr(fb, fx, "len");
            let zero = probe::const_ptr(fb, 0);
            probe::call_shim(fb, fx, "p774_set_budget", &[zero]);
            let one = probe::const_i32(fb, 1);
            let p = probe::call_shim(fb, fx, "p774_own_make", &[one, one, outp])[0];
            let code = probe::load_i32(fb, outp, 0);
            let v = probe::expect_eq_i32(fb, code, probe::status::ERR_BUDGET);
            probe::call_shim(fb, fx, "p774_own_free", &[p]);
            let inv = probe::call_shim(fb, fx, "p774_check_invariant", &[])[0];
            probe::acc(fb, v, inv)
        }));
    });
    probe::assert_pass(&out, &[]);
    assert_eq!(probe::summary_of(&out), (0, 0, 0, 0, 0, 0));
}

#[test]
fn unicode_valid_invalid_sibling_controls() {
    let out = probe::run_probe("review-r1-utf8", |cx| {
        let fixtures: &[(&str, &[u8])] = &[
            ("truncated", &[0xC2]),
            ("overlong", &[0xE0, 0x80, 0x80]),
            ("surrogate", &[0xED, 0xA0, 0x80]),
            ("overmax", &[0xF4, 0x90, 0x80, 0x80]),
            ("max", &[0xF4, 0x8F, 0xBF, 0xBF]),
        ];
        for &(name, bytes) in fixtures {
            cx.data(name, bytes, false);
        }
        cx.set_main(Box::new(|fb, fx| {
            let mut v = probe::const_i32(fb, 0);
            for (name, len, want) in [
                ("truncated", 1, -7),
                ("overlong", 3, -7),
                ("surrogate", 3, -7),
                ("overmax", 4, -7),
                ("max", 4, 1),
            ] {
                let p = probe::data_addr(fb, fx, name);
                let n = probe::const_i32(fb, len);
                let r = probe::call_shim(fb, fx, "p774_text_scalars", &[p, n])[0];
                let check = probe::expect_eq_i32(fb, r, want);
                v = probe::acc(fb, v, check);
            }
            let p = probe::const_ptr(fb, 0);
            let n = probe::const_i32(fb, 0);
            let r = probe::call_shim(fb, fx, "p774_text_scalars", &[p, n])[0];
            let check = probe::expect_eq_i32(fb, r, 0);
            probe::acc(fb, v, check)
        }));
    });
    probe::assert_pass(&out, &[]);
}

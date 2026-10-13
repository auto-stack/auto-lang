//! V-04: illegal states rejected at the FRONT DOOR — the probe's rule model
//! (decisions.md §D-04), with native dispatch ledgers proving the rejected
//! op never reaches machine code, and legal siblings running as real PEs.
//! The R2 refcount-handle family documents why it CANNOT provide this door.

#![cfg(windows)]

use ac_data_probe_774 as probe;
use ac_data_probe_774::rules;

fn rejected(ops: &[rules::Op], want_index: usize, want_code: &str) -> rules::RuleOutcome {
    let out = rules::run_script(ops);
    let r = out
        .rejection
        .unwrap_or_else(|| panic!("expected rejection at {want_index}, script ran to completion"));
    assert_eq!(r.index, want_index, "rejection index");
    assert_eq!(r.code, want_code, "rejection code");
    out
}

/// V04-M1: use after free — view on a freed slot.
#[test]
fn v04_m1_use_after_free() {
    let out = rejected(
        &[rules::Op::Make(0), rules::Op::Free(0), rules::Op::View(0)],
        3,
        "verify.use-after-free",
    );
    // The rejected op never dispatched natively; the legal prefix did.
    assert_eq!(out.ledger.get("borrow_read"), None);
    assert_eq!(out.ledger.get("own_make"), Some(&1));
    assert_eq!(out.ledger.get("own_free"), Some(&1));
    // Legal sibling runs as a real PE with live=0.
    let pe = probe::run_probe("v04-m1-sib", |cx| {
        cx.data("outlen", &[0; 4], true);
        cx.set_main(Box::new(|fb, fx| {
            let outp = probe::data_addr(fb, fx, "outlen");
            let seed = probe::const_i32(fb, 7);
            let len = probe::const_i32(fb, 16);
            let p = probe::call_shim(fb, fx, "p774_own_make", &[seed, len, outp]).remove(0);
            probe::call_shim(fb, fx, "p774_own_free", &[p]);
            probe::call_shim(fb, fx, "p774_check_invariant", &[]).remove(0)
        }));
    });
    probe::assert_pass(&pe, &[]);
    assert_eq!(probe::summary_of(&pe), (1, 1, 1, 0, 0, 16));
}

/// V04-M2: double move.
#[test]
fn v04_m2_double_move() {
    let out = rejected(
        &[
            rules::Op::Make(0),
            rules::Op::MoveOut(0),
            rules::Op::MoveOut(0),
        ],
        3,
        "verify.double-move",
    );
    // move_transfer has no native op by design (a register/value move); its
    // ledger entry stays absent — the dispatch-proof for moves is that no
    // cleanup or read ever fires for the dead source.
    assert_eq!(out.ledger.get("move_transfer"), None);
    // Legal sibling: one move, single cleanup at the destination slot.
    let pe = probe::run_probe("v04-m2-sib", |cx| {
        cx.data("outlen", &[0; 4], true);
        cx.set_main(Box::new(|fb, fx| {
            let outp = probe::data_addr(fb, fx, "outlen");
            let seed = probe::const_i32(fb, 7);
            let len = probe::const_i32(fb, 16);
            let p = probe::call_shim(fb, fx, "p774_own_make", &[seed, len, outp]).remove(0);
            let dst = fb.declare_var(cranelift_codegen::ir::types::I64);
            fb.def_var(dst, p);
            let q = fb.use_var(dst);
            probe::call_shim(fb, fx, "p774_own_free", &[q]);
            probe::call_shim(fb, fx, "p774_check_invariant", &[]).remove(0)
        }));
    });
    probe::assert_pass(&pe, &[]);
    assert_eq!(probe::summary_of(&pe), (1, 1, 1, 0, 0, 16));
}

/// V04-M5: use after move.
#[test]
fn v04_m5_use_after_move() {
    let out = rejected(
        &[
            rules::Op::Make(0),
            rules::Op::MoveOut(0),
            rules::Op::View(0),
        ],
        3,
        "verify.use-after-move",
    );
    assert_eq!(out.ledger.get("borrow_read"), None);
    // Legal sibling: view first, end the view, then move.
    let ok = rules::run_script(&[
        rules::Op::Make(0),
        rules::Op::View(0),
        rules::Op::EndView(0),
        rules::Op::MoveOut(0),
    ]);
    assert!(ok.rejection.is_none());
    assert_eq!(ok.ledger.get("borrow_read"), Some(&1));
}

/// V04-M3: mut under an active view.
#[test]
fn v04_m3_mut_conflict() {
    let out = rejected(
        &[rules::Op::Make(0), rules::Op::View(0), rules::Op::Mut(0)],
        3,
        "verify.mut-conflict",
    );
    assert_eq!(out.ledger.get("rec_mut"), None);
    // Legal sibling: the record view ends before the mutation — real PE.
    let pe = probe::run_probe("v04-m3-sib", |cx| {
        cx.data(
            "rec",
            &{
                let mut b = Vec::new();
                b.extend_from_slice(&1i32.to_le_bytes());
                b.extend_from_slice(&5i32.to_le_bytes());
                b.extend_from_slice(&233i32.to_le_bytes());
                b
            },
            true,
        );
        cx.set_main(Box::new(|fb, fx| {
            let p = probe::data_addr(fb, fx, "rec");
            let before = probe::call_shim(fb, fx, "p774_rec_view", &[p]).remove(0);
            let vb = probe::expect_eq_i32(fb, before, 710);
            let ten = probe::const_i32(fb, 10);
            probe::call_shim(fb, fx, "p774_rec_mut", &[p, ten]);
            let after = probe::call_shim(fb, fx, "p774_rec_view", &[p]).remove(0);
            let va = probe::expect_eq_i32(fb, after, 729);
            probe::acc(fb, vb, va)
        }));
    });
    probe::assert_pass(
        &pe,
        &["REC-VIEW flag=1 count=5 tag=233", "REC-MUT addend=10"],
    );
}

/// V04-M4: returning a view of a locally owned value is an escape.
#[test]
fn v04_m4_borrow_escape() {
    let out = rejected(
        &[rules::Op::Make(0), rules::Op::ReturnViewLocal(0)],
        2,
        "verify.borrow-escape",
    );
    assert_eq!(out.ledger.get("borrow_return"), None);
    // Legal sibling: returning a view of a borrowed PARAM — the owner
    // outlives the frame. Real PE with a nested function.
    let pe = probe::run_probe("v04-m4-sib", |cx| {
        cx.data("lit", b"A", false);
        cx.func(
            "view_of_param",
            &[
                cranelift_codegen::ir::types::I64,
                cranelift_codegen::ir::types::I32,
            ],
            Box::new(|fb, fx, params| {
                let zero = probe::const_i32(fb, 0);
                probe::call_shim(fb, fx, "p774_borrow_read", &[params[0], params[1], zero])
                    .remove(0)
            }),
        );
        cx.set_main(Box::new(|fb, fx| {
            let ptr = probe::data_addr(fb, fx, "lit");
            let one = probe::const_i32(fb, 1);
            let r = probe::call_shim(fb, fx, "view_of_param", &[ptr, one]).remove(0);
            probe::expect_eq_i32(fb, r, 65)
        }));
    });
    probe::assert_pass(&pe, &[]);
}

/// V04-R2: the refcount-handle family CANNOT front-door-reject a stale use —
/// after drop, the recycled slot is silently misread with no error. This is
/// the elimination evidence for candidate R2 (decisions.md §D-06).
#[test]
fn v04_r2_silent_stale_read() {
    let pe = probe::run_probe("v04-r2", |cx| {
        cx.set_main(Box::new(|fb, fx| {
            let p11 = probe::const_i32(fb, 0x11);
            let h1 = probe::call_shim(fb, fx, "p774_h_alloc", &[p11]).remove(0);
            let vh1 = probe::expect_eq_i32(fb, h1, 1);
            probe::call_shim(fb, fx, "p774_h_drop", &[h1]);
            let p22 = probe::const_i32(fb, 0x22);
            let h2 = probe::call_shim(fb, fx, "p774_h_alloc", &[p22]).remove(0);
            let vh2 = probe::expect_eq_i32(fb, h2, 1); // slot 1 recycled
                                                       // The stale handle h1 (== 1) reads the RECYCLED value with no
                                                       // error — the machine side confirms the silent misread (34), the
                                                       // exact behavior class R1's front door rejects.
            let stale = probe::call_shim(fb, fx, "p774_h_get", &[h1]).remove(0);
            let vstale = probe::expect_eq_i32(fb, stale, 34);
            let v1 = probe::acc(fb, vh1, vh2);
            probe::acc(fb, v1, vstale)
        }));
    });
    // exit 0 asserts the stale read RETURNED the recycled value (34): the
    // violation is real, observable, and entirely silent.
    probe::assert_pass(&pe, &["H-ALLOC h=1", "H-DROP h=1 rc=0", "H-GET h=1 v=34"]);
}

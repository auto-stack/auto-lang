//! V-04: illegal states rejected at the FRONT DOOR — the probe's rule model
//! (decisions.md §D-04), with native dispatch ledgers proving the rejected
//! op never reaches machine code, and legal siblings running as real PEs.
//! The R2 refcount-handle family documents why it CANNOT provide this door.

#![cfg(windows)]

use ace_data_conformance as probe;
use ace_data_conformance::rules;

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

// ---- review r1 (P774-R1-02): provenance, storage/mode entries, real PEs ----

/// V04-M4b: ReturnViewParam must verify the slot IS a live borrowed param —
/// a freed local, a moved-out slot, or a move-in owned param all reject.
#[test]
fn v04_m4b_provenance_required() {
    // freed local masquerading as a "param" view (r1 counterexample)
    let out = rejected(
        &[
            rules::Op::Make(0),
            rules::Op::Free(0),
            rules::Op::ReturnViewParam(0),
        ],
        3,
        "verify.use-after-free",
    );
    assert_eq!(out.ledger.get("borrow_return"), None);
    // move-in owned param: the view would escape this frame's cleanup boundary
    let out2 = rejected(
        &[rules::Op::ParamOwned(0), rules::Op::ReturnViewParam(0)],
        2,
        "verify.borrow-escape",
    );
    assert_eq!(out2.ledger.get("borrow_return"), None);
    // the legal form: a registered borrowed-in param
    let ok = rules::run_script(&[rules::Op::ParamBorrowed(0), rules::Op::ReturnViewParam(0)]);
    assert!(ok.rejection.is_none());
    assert_eq!(ok.ledger.get("borrow_return"), Some(&1));
}

/// V04-M4c: the legal borrow return as a REAL PE — the borrowed pointer AND
/// length cross the call frame and the CALLER reads through them afterwards.
#[test]
fn v04_m4c_real_borrow_return_pe() {
    let pe = probe::run_probe("v04-m4c", |cx| {
        cx.data("lit", b"Auto", false);
        cx.data("cells", &[0; 16], true); // [0..8] ptr cell, [8..12] len cell
        cx.func(
            "give_view",
            &[
                cranelift_codegen::ir::types::I64,
                cranelift_codegen::ir::types::I32,
                cranelift_codegen::ir::types::I64,
            ],
            Box::new(|fb, _fx, params| {
                // store the borrowed (ptr, len) into the caller cells: the
                // borrow is RETURNED, not consumed inside the callee
                probe::store_i64(fb, params[2], 0, params[0]);
                probe::store_i32(fb, params[2], 8, params[1]);
                probe::const_i32(fb, 0)
            }),
        );
        cx.set_main(Box::new(|fb, fx| {
            let lit = probe::data_addr(fb, fx, "lit");
            let cells = probe::data_addr(fb, fx, "cells");
            let four = probe::const_i32(fb, 4);
            let r = probe::call_shim(fb, fx, "give_view", &[lit, four, cells]).remove(0);
            let vr = probe::expect_eq_i32(fb, r, 0);
            let got_ptr = probe::load_i64(fb, cells, 0);
            let got_len = probe::load_i32(fb, cells, 8);
            let vl = probe::expect_eq_i32(fb, got_len, 4);
            let zero = probe::const_i32(fb, 0);
            let byte =
                probe::call_shim(fb, fx, "p774_borrow_read", &[got_ptr, got_len, zero]).remove(0);
            let vb = probe::expect_eq_i32(fb, byte, 65);
            let v1 = probe::acc(fb, vr, vl);
            probe::acc(fb, v1, vb)
        }));
    });
    probe::assert_pass(&pe, &[]);
}

/// V04-M5 sibling as a REAL PE (make -> read through a view -> move -> one
/// cleanup at the destination slot).
#[test]
fn v04_m5_sibling_real_pe() {
    let pe = probe::run_probe("v04-m5-sib", |cx| {
        cx.data("outlen", &[0; 4], true);
        cx.set_main(Box::new(|fb, fx| {
            let outp = probe::data_addr(fb, fx, "outlen");
            let seed = probe::const_i32(fb, 7);
            let len = probe::const_i32(fb, 16);
            let p = probe::call_shim(fb, fx, "p774_own_make", &[seed, len, outp]).remove(0);
            let zero = probe::const_i32(fb, 0);
            let byte = probe::call_shim(fb, fx, "p774_borrow_read", &[p, len, zero]).remove(0);
            let vb = probe::expect_eq_i32(fb, byte, 7);
            let dst = fb.declare_var(cranelift_codegen::ir::types::I64);
            fb.def_var(dst, p);
            let q = fb.use_var(dst);
            probe::call_shim(fb, fx, "p774_own_free", &[q]);
            let inv = probe::call_shim(fb, fx, "p774_check_invariant", &[]).remove(0);
            probe::acc(fb, vb, inv)
        }));
    });
    probe::assert_pass(&pe, &[]);
    assert_eq!(probe::summary_of(&pe), (1, 1, 1, 0, 0, 16));
}

/// V04-storage entries: field/container borrows have REACHABLE rejection
/// operations; storing an owned clone is the legal sibling.
#[test]
fn v04_field_and_container_entries() {
    let f = rejected(
        &[rules::Op::Make(0), rules::Op::StoreFieldView(0)],
        2,
        "verify.field-borrow",
    );
    assert_eq!(f.ledger.get("own_make"), Some(&1));
    let c = rejected(
        &[rules::Op::Make(0), rules::Op::StoreContainerView(0)],
        2,
        "verify.container-borrow",
    );
    assert_eq!(c.ledger.get("own_make"), Some(&1));
    // legal siblings in the model ...
    let ok = rules::run_script(&[
        rules::Op::Make(0),
        rules::Op::StoreOwnedClone(0),
        rules::Op::Free(0),
    ]);
    assert!(ok.rejection.is_none());
    assert_eq!(ok.ledger.get("own_clone"), Some(&1));
    // ... and as a real PE: field/container stores keep the value live
    // until scope exit (the clone is freed exactly once).
    let pe = probe::run_probe("v04-store-sib", |cx| {
        cx.data("outlen", &[0; 4], true);
        cx.set_main(Box::new(|fb, fx| {
            let outp = probe::data_addr(fb, fx, "outlen");
            let seed = probe::const_i32(fb, 7);
            let len = probe::const_i32(fb, 16);
            let p = probe::call_shim(fb, fx, "p774_own_make", &[seed, len, outp]).remove(0);
            let q = probe::call_shim(fb, fx, "p774_own_clone", &[p, len, outp]).remove(0);
            probe::call_shim(fb, fx, "p774_own_free", &[p]);
            probe::call_shim(fb, fx, "p774_own_free", &[q]);
            probe::call_shim(fb, fx, "p774_check_invariant", &[]).remove(0)
        }));
    });
    probe::assert_pass(&pe, &[]);
    assert_eq!(probe::summary_of(&pe), (2, 2, 2, 0, 0, 32));
}

/// V04 mode requirement: passing an owned value to a `move Str` param
/// without the call-site `.move` rejects; the explicit form transfers and
/// the CALLEE takes over cleanup (real PE sibling).
#[test]
fn v04_move_param_mode_requirement() {
    let bad = rejected(
        &[rules::Op::Make(0), rules::Op::CallMoveParamImplicit(0)],
        2,
        "verify.ownership-required",
    );
    assert_eq!(bad.ledger.get("own_free"), None);
    let ok = rules::run_script(&[rules::Op::Make(0), rules::Op::CallMoveParamExplicit(0)]);
    assert!(ok.rejection.is_none());
    // the callee's cleanup dispatch fires exactly once
    assert_eq!(ok.ledger.get("own_free"), Some(&1));

    let pe = probe::run_probe("v04-moveparam-sib", |cx| {
        cx.data("outlen", &[0; 4], true);
        cx.func(
            "consume",
            &[cranelift_codegen::ir::types::I64],
            Box::new(|fb, fx, params| {
                // move-in param: the CALLEE owns cleanup
                probe::call_shim(fb, fx, "p774_own_free", &[params[0]]);
                probe::const_i32(fb, 0)
            }),
        );
        cx.set_main(Box::new(|fb, fx| {
            let outp = probe::data_addr(fb, fx, "outlen");
            let seed = probe::const_i32(fb, 7);
            let len = probe::const_i32(fb, 16);
            let p = probe::call_shim(fb, fx, "p774_own_make", &[seed, len, outp]).remove(0);
            let r = probe::call_shim(fb, fx, "consume", &[p]).remove(0);
            let v = probe::expect_eq_i32(fb, r, 0);
            let inv = probe::call_shim(fb, fx, "p774_check_invariant", &[]).remove(0);
            probe::acc(fb, v, inv)
        }));
    });
    probe::assert_pass(&pe, &[]);
    assert_eq!(probe::summary_of(&pe), (1, 1, 1, 0, 0, 16));
}

/// R2 fairness control (review r1 P774-R1-04): with per-slot active
/// tracking the stale read IS detectable — showing exactly the extra
/// machinery a front-door-safe R2 would need on top of the handle table.
#[test]
fn v04_r2_checked_variant_detects_stale() {
    let pe = probe::run_probe("v04-r2-checked", |cx| {
        cx.set_main(Box::new(|fb, fx| {
            let p11 = probe::const_i32(fb, 0x11);
            let h1 = probe::call_shim(fb, fx, "p774_h_alloc", &[p11]).remove(0);
            probe::call_shim(fb, fx, "p774_h_drop", &[h1]);
            let stale = probe::call_shim(fb, fx, "p774_h_get_checked", &[h1]).remove(0);
            probe::expect_eq_i32(fb, stale, probe::status::ERR_INDEX_OOB)
        }));
    });
    probe::assert_pass(&pe, &["H-ALLOC h=1", "H-DROP h=1 rc=0"]);
}

// ---- review r2 (P774-R2-01/02): source x mode matrix, static literal,
// incoming mut borrow ----

/// The r2 reviewer's control: a view region over a borrowed param ends and
/// the param remains returnable — no panic, no invalidation.
#[test]
fn v04_r2_borrowed_param_view_region_ends() {
    let out = rules::run_script(&[
        rules::Op::ParamBorrowed(0),
        rules::Op::View(0),
        rules::Op::EndView(0),
        rules::Op::ReturnViewParam(0),
    ]);
    assert!(out.rejection.is_none(), "got {:?}", out.rejection);
    assert_eq!(out.ledger.get("borrow_read"), Some(&1));
    assert_eq!(out.ledger.get("borrow_return"), Some(&1));
}

/// Representative source x mode matrix (r2): every legal combination runs,
/// every illegal one rejects with the frozen code — no legal input panics.
#[test]
fn v04_r2_source_mode_matrix() {
    use rules::Op;
    #[rustfmt::skip]
    let legal: &[&[Op]] = &[
        // local owned: view/end, mut/end, move, free, clone, store clone
        &[Op::Make(0), Op::View(0), Op::EndView(0), Op::Mut(0), Op::EndMut(0), Op::Clone(0), Op::MoveOut(0)],
        // borrowed param: view/end, return, clone
        &[Op::ParamBorrowed(0), Op::View(0), Op::EndView(0), Op::Clone(0), Op::ReturnViewParam(0)],
        // incoming mut param: mutate through it, then the frame ends
        &[Op::ParamMutBorrowed(0), Op::Mut(0)],
        // move-in param: view/end, mutate, free (callee cleanup), move out
        &[Op::ParamOwned(0), Op::View(0), Op::EndView(0), Op::Mut(0), Op::EndMut(0), Op::MoveOut(0)],
        &[Op::ParamOwned(0), Op::Free(0)],
        // static literal: view/end, return the view
        &[Op::StaticLiteral(0), Op::View(0), Op::EndView(0), Op::ReturnViewParam(0)],
        // explicit move call transfers; implicit rejects
        &[Op::Make(0), Op::CallMoveParamExplicit(0)],
        &[Op::ParamBorrowed(0), Op::StoreOwnedClone(0)],
    ];
    for (i, script) in legal.iter().enumerate() {
        let out = rules::run_script(script);
        assert!(
            out.rejection.is_none(),
            "legal[{i}] rejected: {:?}",
            out.rejection
        );
    }
    #[rustfmt::skip]
    let illegal: &[(&[Op], &str)] = &[
        // readonly borrowed param is not a mutable source
        (&[Op::ParamBorrowed(0), Op::Mut(0)], "verify.mut-conflict"),
        // static literal is not mutable / not ownable / not freeable
        (&[Op::StaticLiteral(0), Op::Mut(0)], "verify.mut-conflict"),
        (&[Op::StaticLiteral(0), Op::MoveOut(0)], "verify.ownership-required"),
        (&[Op::StaticLiteral(0), Op::Free(0)], "verify.ownership-required"),
        (&[Op::StaticLiteral(0), Op::CallMoveParamExplicit(0)], "verify.ownership-required"),
        // borrowed / mut-borrowed params are not ownable or freeable
        (&[Op::ParamBorrowed(0), Op::Free(0)], "verify.ownership-required"),
        (&[Op::ParamBorrowed(0), Op::MoveOut(0)], "verify.ownership-required"),
        (&[Op::ParamMutBorrowed(0), Op::Free(0)], "verify.ownership-required"),
        (&[Op::ParamMutBorrowed(0), Op::MoveOut(0)], "verify.ownership-required"),
        // readonly view during an incoming exclusive mut borrow
        (&[Op::ParamMutBorrowed(0), Op::View(0)], "verify.mut-conflict"),
        // returning a view of an incoming mut borrow (no first-batch
        // reborrow downgrade) or of a move-in param escapes
        (&[Op::ParamMutBorrowed(0), Op::ReturnViewParam(0)], "verify.borrow-escape"),
        (&[Op::ParamOwned(0), Op::ReturnViewParam(0)], "verify.borrow-escape"),
        // views over locals still cannot escape
        (&[Op::Make(0), Op::View(0), Op::ReturnViewParam(0)], "verify.borrow-escape"),
        (&[Op::Make(0), Op::ReturnViewLocal(0)], "verify.borrow-escape"),
        // dead / moved sources reject regardless of use kind
        (&[Op::Make(0), Op::Free(0), Op::ReturnViewLocal(0)], "verify.use-after-free"),
        (&[Op::Make(0), Op::MoveOut(0), Op::Clone(0)], "verify.use-after-move"),
    ];
    for (i, (script, code)) in illegal.iter().enumerate() {
        let out = rules::run_script(script);
        let r = out
            .rejection
            .unwrap_or_else(|| panic!("illegal[{i}] accepted"));
        assert_eq!(r.code, *code, "illegal[{i}] code mismatch");
    }
}

/// Static literal return as a REAL PE: the literal lives in rodata; a view
/// of it crosses the frame and the caller reads through it (the m4c shape,
/// restated for the restored D-03 source).
#[test]
fn v04_r2_static_literal_return_pe() {
    let pe = probe::run_probe("v04-r2-lit", |cx| {
        cx.data("lit", b"Auto", false); // read-only data section = static literal
        cx.data("cells", &[0; 16], true);
        cx.func(
            "give_view",
            &[
                cranelift_codegen::ir::types::I64,
                cranelift_codegen::ir::types::I32,
                cranelift_codegen::ir::types::I64,
            ],
            Box::new(|fb, _fx, params| {
                probe::store_i64(fb, params[2], 0, params[0]);
                probe::store_i32(fb, params[2], 8, params[1]);
                probe::const_i32(fb, 0)
            }),
        );
        cx.set_main(Box::new(|fb, fx| {
            let lit = probe::data_addr(fb, fx, "lit");
            let cells = probe::data_addr(fb, fx, "cells");
            let four = probe::const_i32(fb, 4);
            let r = probe::call_shim(fb, fx, "give_view", &[lit, four, cells]).remove(0);
            let vr = probe::expect_eq_i32(fb, r, 0);
            let ptr = probe::load_i64(fb, cells, 0);
            let len = probe::load_i32(fb, cells, 8);
            let zero = probe::const_i32(fb, 0);
            let byte = probe::call_shim(fb, fx, "p774_borrow_read", &[ptr, len, zero]).remove(0);
            let vb = probe::expect_eq_i32(fb, byte, 65);
            probe::acc(fb, vr, vb)
        }));
    });
    probe::assert_pass(&pe, &[]);
}

/// Incoming `mut str` param as a REAL PE: the callee mutates EXCLUSIVELY
/// through its mut-borrowed record param; the caller observes before/after.
#[test]
fn v04_r2_incoming_mut_param_pe() {
    let pe = probe::run_probe("v04-r2-mutparam", |cx| {
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
        cx.func(
            "mut_through_param",
            &[cranelift_codegen::ir::types::I64],
            Box::new(|fb, fx, params| {
                let ten = probe::const_i32(fb, 10);
                probe::call_shim(fb, fx, "p774_rec_mut", &[params[0], ten]);
                probe::const_i32(fb, 0)
            }),
        );
        cx.set_main(Box::new(|fb, fx| {
            let p = probe::data_addr(fb, fx, "rec");
            let before = probe::call_shim(fb, fx, "p774_rec_view", &[p]).remove(0);
            let vb = probe::expect_eq_i32(fb, before, 710);
            let r = probe::call_shim(fb, fx, "mut_through_param", &[p]).remove(0);
            let vr = probe::expect_eq_i32(fb, r, 0);
            let after = probe::call_shim(fb, fx, "p774_rec_view", &[p]).remove(0);
            let va = probe::expect_eq_i32(fb, after, 729);
            let v1 = probe::acc(fb, vb, vr);
            probe::acc(fb, v1, va)
        }));
    });
    probe::assert_pass(
        &pe,
        &["REC-VIEW flag=1 count=5 tag=233", "REC-MUT addend=10"],
    );
}

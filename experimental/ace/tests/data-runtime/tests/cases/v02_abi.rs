//! V-02: the five frozen ABI categories over real machine calls
//! (probe-cases.md §V-02; decisions.md §D-05).

#![cfg(windows)]

use ace_data_conformance as probe;

fn rec_bytes(flag: i32, count: i32, tag: i32) -> Vec<u8> {
    let mut b = Vec::new();
    b.extend_from_slice(&flag.to_le_bytes());
    b.extend_from_slice(&count.to_le_bytes());
    b.extend_from_slice(&tag.to_le_bytes());
    b
}

/// V02-1 ABI-A mixed scalars + V02-1s heterogeneous order control.
#[test]
fn v02_1_mix_and_swap_control() {
    let base = probe::run_probe("v02-1", |cx| {
        cx.set_main(Box::new(|fb, fx| {
            let a = probe::const_i32(fb, 17);
            let b = probe::const_i32(fb, 1);
            let c = probe::const_i32(fb, 0x4E2D);
            let d = probe::const_i32(fb, 96);
            let r = probe::call_shim(fb, fx, "p774_mix", &[a, b, c, d]).remove(0);
            probe::expect_eq_i32(fb, r, 20127)
        }));
    });
    probe::assert_pass(&base, &["MIX a=17 b=1 c=20013 d=96"]);

    // Control: swap the a/d argument VALUES; the observation channel must
    // reflect the physical order, proving a machine-level swap would be
    // caught by the oracle trace.
    let swap = probe::run_probe("v02-1s", |cx| {
        cx.set_main(Box::new(|fb, fx| {
            let a = probe::const_i32(fb, 96);
            let b = probe::const_i32(fb, 1);
            let c = probe::const_i32(fb, 0x4E2D);
            let d = probe::const_i32(fb, 17);
            let r = probe::call_shim(fb, fx, "p774_mix", &[a, b, c, d]).remove(0);
            probe::expect_eq_i32(fb, r, 20127)
        }));
    });
    probe::assert_pass(&swap, &["MIX a=96 b=1 c=20013 d=17"]);
    assert!(
        !swap.stderr.contains("MIX a=17 b=1 c=20013 d=96"),
        "swap control must differ from the base oracle trace"
    );
}

/// V02-2 ABI-A register/stack boundary (args 5-7 travel on the stack) +
/// V02-2s boundary-swap control.
#[test]
fn v02_2_many_and_boundary_swap() {
    let base = probe::run_probe("v02-2", |cx| {
        cx.set_main(Box::new(|fb, fx| {
            let args: Vec<_> = (100..=106)
                .map(|v| probe::const_i32(fb, v as i64))
                .collect();
            let r = probe::call_shim(fb, fx, "p774_many", &args).remove(0);
            probe::expect_eq_i32(fb, r, 721)
        }));
    });
    probe::assert_pass(&base, &["MANY a=100 b=101 c=102 d=103 e=104 f=105 g=106"]);

    let swap = probe::run_probe("v02-2s", |cx| {
        cx.set_main(Box::new(|fb, fx| {
            let mut v: Vec<_> = (100..=106)
                .map(|x| probe::const_i32(fb, x as i64))
                .collect();
            v.swap(3, 4); // d (last register arg) <-> e (first stack arg)
            let r = probe::call_shim(fb, fx, "p774_many", &v).remove(0);
            probe::expect_eq_i32(fb, r, 721)
        }));
    });
    probe::assert_pass(&swap, &["MANY a=100 b=101 c=102 d=104 e=103 f=105 g=106"]);
}

/// V02-3 ABI-B borrowed buffer: callee reads, never frees.
#[test]
fn v02_3_borrow_sum() {
    let out = probe::run_probe("v02-3", |cx| {
        cx.data("auto", b"Auto", false);
        cx.set_main(Box::new(|fb, fx| {
            let ptr = probe::data_addr(fb, fx, "auto");
            let len = probe::const_i32(fb, 4);
            let r = probe::call_shim(fb, fx, "p774_borrow_sum", &[ptr, len]).remove(0);
            let v = probe::expect_eq_i32(fb, r, 409);
            let inv = probe::call_shim(fb, fx, "p774_check_invariant", &[]).remove(0);
            probe::acc(fb, v, inv)
        }));
    });
    probe::assert_pass(&out, &["BSUM len=4"]);
    assert_eq!(probe::summary_of(&out), (0, 0, 0, 0, 0, 0));
}

/// V02-4 ABI-C owned return: content pattern, length channel, caller frees.
#[test]
fn v02_4_own_make() {
    let out = probe::run_probe("v02-4", |cx| {
        cx.data("outlen", &[0; 4], true);
        cx.set_main(Box::new(|fb, fx| {
            let outp = probe::data_addr(fb, fx, "outlen");
            let seed = probe::const_i32(fb, 7);
            let len = probe::const_i32(fb, 16);
            let ptr = probe::call_shim(fb, fx, "p774_own_make", &[seed, len, outp]).remove(0);
            let nn = probe::expect_non_null(fb, ptr);
            let got_len = probe::load_i32(fb, outp, 0);
            let vl = probe::expect_eq_i32(fb, got_len, 16);
            let sixteen = probe::const_i32(fb, 16);
            let sum = probe::call_shim(fb, fx, "p774_borrow_sum", &[ptr, sixteen]).remove(0);
            let vs = probe::expect_eq_i32(fb, sum, 232);
            probe::call_shim(fb, fx, "p774_own_free", &[ptr]);
            let v1 = probe::acc(fb, nn, vl);
            let v2 = probe::acc(fb, vs, v1);
            let inv = probe::call_shim(fb, fx, "p774_check_invariant", &[]).remove(0);
            probe::acc(fb, v2, inv)
        }));
    });
    probe::assert_pass(&out, &["OWN-MAKE seed=7 len=16"]);
    assert_eq!(probe::summary_of(&out), (1, 1, 1, 0, 0, 16));
}

/// V02-5 ABI-D record view + V02-5s field-swap control.
#[test]
fn v02_5_rec_view_and_field_swap() {
    let out = probe::run_probe("v02-5", |cx| {
        cx.data("rec", &rec_bytes(1, 5, 233), true);
        cx.set_main(Box::new(|fb, fx| {
            let p = probe::data_addr(fb, fx, "rec");
            let r = probe::call_shim(fb, fx, "p774_rec_view", &[p]).remove(0);
            probe::expect_eq_i32(fb, r, 710)
        }));
    });
    probe::assert_pass(&out, &["REC-VIEW flag=1 count=5 tag=233"]);

    // Control: store the VALUES into swapped field slots; the shim reads by
    // the frozen offsets, so the trace exposes the swap.
    let swap = probe::run_probe("v02-5s", |cx| {
        cx.data("rec", &rec_bytes(1, 233, 5), true);
        cx.set_main(Box::new(|fb, fx| {
            let p = probe::data_addr(fb, fx, "rec");
            let r = probe::call_shim(fb, fx, "p774_rec_view", &[p]).remove(0);
            probe::expect_eq_i32(fb, r, 482)
        }));
    });
    probe::assert_pass(&swap, &["REC-VIEW flag=1 count=233 tag=5"]);
}

/// V02-6 ABI-E record mut: count += addend, flag flips; observed via view.
#[test]
fn v02_6_rec_mut() {
    let out = probe::run_probe("v02-6", |cx| {
        cx.data("rec", &rec_bytes(1, 5, 233), true);
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
    probe::assert_pass(&out, &["REC-MUT addend=10"]);
}

/// V02-7 ABI-E record out-return: callee fully initializes caller storage.
#[test]
fn v02_7_rec_out() {
    let out = probe::run_probe("v02-7", |cx| {
        cx.data("out", &[0; 12], true);
        cx.set_main(Box::new(|fb, fx| {
            let outp = probe::data_addr(fb, fx, "out");
            let seed = probe::const_i32(fb, 5);
            probe::call_shim(fb, fx, "p774_rec_out", &[seed, outp]);
            let r = probe::call_shim(fb, fx, "p774_rec_view", &[outp]).remove(0);
            probe::expect_eq_i32(fb, r, 218)
        }));
    });
    probe::assert_pass(&out, &["REC-OUT seed=5"]);
}

/// V02-8 nested calls: the main body (view, call f_inner, view) where
/// f_inner mutates — a second level of real machine calls.
#[test]
fn v02_8_nested_calls() {
    let out = probe::run_probe("v02-8", |cx| {
        cx.data("rec", &rec_bytes(1, 5, 233), true);
        cx.func(
            "f_inner",
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
            let inner = probe::call_shim(fb, fx, "f_inner", &[p]).remove(0);
            let vi = probe::expect_eq_i32(fb, inner, 0);
            let after = probe::call_shim(fb, fx, "p774_rec_view", &[p]).remove(0);
            let va = probe::expect_eq_i32(fb, after, 729);
            let v1 = probe::acc(fb, vb, vi);
            probe::acc(fb, v1, va)
        }));
    });
    probe::assert_pass(
        &out,
        &["REC-VIEW flag=1 count=5 tag=233", "REC-MUT addend=10"],
    );
}

/// V02-9 (review r1 P774-R1-04): nested record layout — Outer { inner:
/// Inner{x,y} inline @0..8, flag @8 } via rec2_out/rec2_view, mixed with a
/// flat Rec1 op in the same PE.
#[test]
fn v02_9_nested_record_layout() {
    let out = probe::run_probe("v02-9", |cx| {
        cx.data("outer", &[0; 12], true);
        cx.data("rec", &rec_bytes(1, 5, 233), true);
        cx.set_main(Box::new(|fb, fx| {
            let outer = probe::data_addr(fb, fx, "outer");
            let seed = probe::const_i32(fb, 5);
            probe::call_shim(fb, fx, "p774_rec2_out", &[seed, outer]);
            let r2 = probe::call_shim(fb, fx, "p774_rec2_view", &[outer]).remove(0);
            // checksum = x + 2*y + 3*flag = 5 + 12 + 3 = 20
            let v2 = probe::expect_eq_i32(fb, r2, 20);
            // mixed flat-record call in the same PE
            let p = probe::data_addr(fb, fx, "rec");
            let r1 = probe::call_shim(fb, fx, "p774_rec_view", &[p]).remove(0);
            let v1 = probe::expect_eq_i32(fb, r1, 710);
            probe::acc(fb, v1, v2)
        }));
    });
    probe::assert_pass(&out, &["REC2-OUT seed=5", "REC2-VIEW x=5 y=6 flag=1"]);
}

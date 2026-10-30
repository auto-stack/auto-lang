//! PLAN-770 T-03 text op faces (774 V-07 oracle subset on the 770 runtime):
//! scalar/byte counting, char_at, slice, codepoint casts, and the new
//! text_eq/ne value equality including NUL non-truncation.

#![cfg(windows)]
use ace_text_conformance::{
    acc, call_shim, const_i32, data_addr, expect_eq_i32, load_i32, run_probe, summary_of,
};

fn out_cell(
    fb: &mut cranelift_frontend::FunctionBuilder,
    fx: &ace_text_conformance::Fx,
) -> cranelift_codegen::ir::Value {
    data_addr(fb, fx, "outcell")
}

fn observe(fb: &mut cranelift_frontend::FunctionBuilder, fx: &ace_text_conformance::Fx) {
    call_shim(fb, fx, "ac_rt_check_invariant", &[]);
}

#[test]
fn v07_unicode_counts_and_char_at() {
    let out = run_probe("v07_counts", |cx| {
        cx.data("t1", b"A\xE4\xB8\xAD\xF0\x9F\x98\x80", false); // A中😀
        cx.data("t2", b"e\xCC\x81", false); // e + U+0301
        cx.data("t3", b"\x00", false); // NUL
        cx.set_main(|fb, fx| {
            let t1 = data_addr(fb, fx, "t1");
            let t2 = data_addr(fb, fx, "t2");
            let t3 = data_addr(fb, fx, "t3");
            let n8 = const_i32(fb, 8);
            let n3 = const_i32(fb, 3);
            let n1 = const_i32(fb, 1);
            let i0 = const_i32(fb, 0);
            let i1 = const_i32(fb, 1);
            let i2 = const_i32(fb, 2);
            let i3 = const_i32(fb, 3);
            let sc1 = call_shim(fb, fx, "ac_rt_text_scalars", &[t1, n8])[0];
            let f1 = expect_eq_i32(fb, sc1, 3);
            let by1 = call_shim(fb, fx, "ac_rt_text_bytes", &[t1, n8])[0];
            let f2 = expect_eq_i32(fb, by1, 8);
            let sc2 = call_shim(fb, fx, "ac_rt_text_scalars", &[t2, n3])[0];
            let f3 = expect_eq_i32(fb, sc2, 2);
            let by2 = call_shim(fb, fx, "ac_rt_text_bytes", &[t2, n3])[0];
            let f4 = expect_eq_i32(fb, by2, 3);
            let sc3 = call_shim(fb, fx, "ac_rt_text_scalars", &[t3, n1])[0];
            let f5 = expect_eq_i32(fb, sc3, 1);
            let c0 = call_shim(fb, fx, "ac_rt_char_at", &[t1, n8, i0])[0];
            let f6 = expect_eq_i32(fb, c0, 65);
            let c1 = call_shim(fb, fx, "ac_rt_char_at", &[t1, n8, i1])[0];
            let f7 = expect_eq_i32(fb, c1, 20013);
            let c2 = call_shim(fb, fx, "ac_rt_char_at", &[t1, n8, i2])[0];
            let f8 = expect_eq_i32(fb, c2, 128512);
            let cn = call_shim(fb, fx, "ac_rt_char_at", &[t3, n1, i0])[0];
            let f9 = expect_eq_i32(fb, cn, 0); // NUL is a legal char
            let cx3 = call_shim(fb, fx, "ac_rt_char_at", &[t1, n8, i3])[0];
            let f10 = expect_eq_i32(fb, cx3, -10);
            let f = acc(fb, f1, f2);
            let f = acc(fb, f, f3);
            let f = acc(fb, f, f4);
            let f = acc(fb, f, f5);
            let f = acc(fb, f, f6);
            let f = acc(fb, f, f7);
            let f = acc(fb, f, f8);
            let f = acc(fb, f, f9);
            let f = acc(fb, f, f10);
            observe(fb, fx);
            f
        });
    });
    assert_eq!(out.code, Some(0), "stderr:\n{}", out.stderr);
    assert_eq!(summary_of(&out), (0, 0, 0, 0, 0, 0));
}

#[test]
fn v07_slice_ranges() {
    let out = run_probe("v07_slice", |cx| {
        cx.data("outcell", &[0; 16], true);
        cx.data("t1", b"A\xE4\xB8\xAD\xF0\x9F\x98\x80", false);
        cx.set_main(|fb, fx| {
            let t1 = data_addr(fb, fx, "t1");
            let sp = out_cell(fb, fx);
            let n = const_i32(fb, 8);
            let i0 = const_i32(fb, 0);
            let i1 = const_i32(fb, 1);
            let i3 = const_i32(fb, 3);
            // slice(0,1): len 1
            call_shim(fb, fx, "ac_rt_text_slice", &[t1, n, i0, i1, sp]);
            let g1 = load_i32(fb, sp, 0);
            let f1 = expect_eq_i32(fb, g1, 1);
            // slice(1,3): len 7 (cross-scalar byte range)
            call_shim(fb, fx, "ac_rt_text_slice", &[t1, n, i1, i3, sp]);
            let g2 = load_i32(fb, sp, 0);
            let f2 = expect_eq_i32(fb, g2, 7);
            // slice(1,1): empty range legal, len 0
            call_shim(fb, fx, "ac_rt_text_slice", &[t1, n, i1, i1, sp]);
            let g3 = load_i32(fb, sp, 0);
            let f3 = expect_eq_i32(fb, g3, 0);
            let f = acc(fb, f1, f2);
            let f = acc(fb, f, f3);
            observe(fb, fx);
            f
        });
    });
    assert_eq!(out.code, Some(0), "stderr:\n{}", out.stderr);
    assert_eq!(summary_of(&out), (0, 0, 0, 0, 0, 0));
}

#[test]
fn v07_codepoint_roundtrip() {
    let out = run_probe("v07_cp", |cx| {
        cx.set_main(|fb, fx| {
            let cpn = const_i32(fb, 0x1F600);
            let cp = call_shim(fb, fx, "ac_rt_char_from_codepoint", &[cpn])[0];
            let f1 = expect_eq_i32(fb, cp, 128512);
            let back = call_shim(fb, fx, "ac_rt_char_to_codepoint", &[cp])[0];
            let f2 = expect_eq_i32(fb, back, 128512);
            let surn = const_i32(fb, 0xD800);
            let sur = call_shim(fb, fx, "ac_rt_char_from_codepoint", &[surn])[0];
            let f3 = expect_eq_i32(fb, sur, -8);
            let f = acc(fb, f1, f2);
            let f = acc(fb, f, f3);
            observe(fb, fx);
            f
        });
    });
    assert_eq!(out.code, Some(0), "stderr:\n{}", out.stderr);
    assert!(out.stderr.contains("RT-FAIL code=-8"));
}

#[test]
fn v07_text_equality_is_byte_exact() {
    let out = run_probe("v07_eq", |cx| {
        cx.data("a1", b"Auto", false);
        cx.data("a2", b"Auto", false);
        cx.data("a3", b"Autu", false);
        cx.data("n1", b"a\x00b", false);
        cx.data("n2", b"a\x00b", false);
        cx.data("n3", b"a", false);
        cx.set_main(|fb, fx| {
            let a1 = data_addr(fb, fx, "a1");
            let a2 = data_addr(fb, fx, "a2");
            let a3 = data_addr(fb, fx, "a3");
            let n1 = data_addr(fb, fx, "n1");
            let n2 = data_addr(fb, fx, "n2");
            let n3 = data_addr(fb, fx, "n3");
            let c4 = const_i32(fb, 4);
            let c3 = const_i32(fb, 3);
            let c1 = const_i32(fb, 1);
            let c0 = const_i32(fb, 0);
            let eq = call_shim(fb, fx, "ac_rt_text_eq", &[a1, c4, a2, c4])[0];
            let f1 = expect_eq_i32(fb, eq, 1);
            let ne = call_shim(fb, fx, "ac_rt_text_ne", &[a1, c4, a3, c4])[0];
            let f2 = expect_eq_i32(fb, ne, 1);
            let eqd = call_shim(fb, fx, "ac_rt_text_eq", &[a1, c4, a3, c4])[0];
            let f3 = expect_eq_i32(fb, eqd, 0);
            // NUL does not truncate: "a\0b" == "a\0b" (1) but != "a" (0)
            let eqn = call_shim(fb, fx, "ac_rt_text_eq", &[n1, c3, n2, c3])[0];
            let f4 = expect_eq_i32(fb, eqn, 1);
            let nen = call_shim(fb, fx, "ac_rt_text_eq", &[n1, c3, n3, c1])[0];
            let f5 = expect_eq_i32(fb, nen, 0);
            // empty == empty
            let eqe = call_shim(fb, fx, "ac_rt_text_eq", &[a1, c0, a3, c0])[0];
            let f6 = expect_eq_i32(fb, eqe, 1);
            let f = acc(fb, f1, f2);
            let f = acc(fb, f, f3);
            let f = acc(fb, f, f4);
            let f = acc(fb, f, f5);
            let f = acc(fb, f, f6);
            observe(fb, fx);
            f
        });
    });
    assert_eq!(out.code, Some(0), "stderr:\n{}", out.stderr);
}

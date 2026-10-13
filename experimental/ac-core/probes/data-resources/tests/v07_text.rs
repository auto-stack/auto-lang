//! V-07: Unicode / indexing boundaries — byte vs scalar are both asserted
//! (probe-cases.md §V-07; decisions.md §D-01/D-02).

#![cfg(windows)]

use ac_data_probe_774 as probe;

/// T1 = "A中😀" — 8 UTF-8 bytes, 3 scalars.
const T1: &[u8] = &[0x41, 0xE4, 0xB8, 0xAD, 0xF0, 0x9F, 0x98, 0x80];
/// T2 = "e" + U+0301 combining acute — 3 bytes, 2 scalars.
const T2: &[u8] = &[0x65, 0xCC, 0x81];
/// T3 = NUL — 1 byte, 1 scalar.
const T3: &[u8] = &[0x00];
/// T4 = invalid UTF-8 — decode must fail.
const T4: &[u8] = &[0xFF, 0xFE];

/// V07-1/2/3: byte and scalar counts never mix.
#[test]
fn v07_counts() {
    let out = probe::run_probe("v07-counts", |cx| {
        cx.data("t1", T1, false);
        cx.data("t2", T2, false);
        cx.data("t3", T3, false);
        cx.set_main(Box::new(|fb, fx| {
            let l1 = probe::const_i32(fb, 8);
            let l2 = probe::const_i32(fb, 3);
            let l3 = probe::const_i32(fb, 1);
            let p1 = probe::data_addr(fb, fx, "t1");
            let p2 = probe::data_addr(fb, fx, "t2");
            let p3 = probe::data_addr(fb, fx, "t3");
            let s1 = probe::call_shim(fb, fx, "p774_text_scalars", &[p1, l1]).remove(0);
            let b1 = probe::call_shim(fb, fx, "p774_text_bytes", &[p1, l1]).remove(0);
            let s2 = probe::call_shim(fb, fx, "p774_text_scalars", &[p2, l2]).remove(0);
            let b2 = probe::call_shim(fb, fx, "p774_text_bytes", &[p2, l2]).remove(0);
            let s3 = probe::call_shim(fb, fx, "p774_text_scalars", &[p3, l3]).remove(0);
            let b3 = probe::call_shim(fb, fx, "p774_text_bytes", &[p3, l3]).remove(0);
            let e1 = probe::expect_eq_i32(fb, s1, 3);
            let e2 = probe::expect_eq_i32(fb, b1, 8);
            let e3 = probe::expect_eq_i32(fb, s2, 2);
            let e4 = probe::expect_eq_i32(fb, b2, 3);
            let e5 = probe::expect_eq_i32(fb, s3, 1);
            let e6 = probe::expect_eq_i32(fb, b3, 1);
            let v = probe::acc(fb, e1, e2);
            let v = probe::acc(fb, v, e3);
            let v = probe::acc(fb, v, e4);
            let v = probe::acc(fb, v, e5);
            probe::acc(fb, v, e6)
        }));
    });
    probe::assert_pass(
        &out,
        &[
            "TEXT-SCALARS n=3",
            "TEXT-BYTES n=8",
            "TEXT-SCALARS n=2",
            "TEXT-BYTES n=3",
        ],
    );
}

/// V07-4: invalid UTF-8 fails the runtime decode with the frozen code.
#[test]
fn v07_invalid_utf8() {
    let out = probe::run_probe("v07-invalid", |cx| {
        cx.data("t4", T4, false);
        cx.set_main(Box::new(|fb, fx| {
            let p = probe::data_addr(fb, fx, "t4");
            let l = probe::const_i32(fb, 2);
            let r = probe::call_shim(fb, fx, "p774_text_scalars", &[p, l]).remove(0);
            probe::expect_eq_i32(fb, r, probe::status::ERR_INVALID_UTF8)
        }));
    });
    probe::assert_pass(&out, &[]);
}

/// V07-5/6/7: char_at — scalar positions, code-point values, oob rejection,
/// NUL is a legal char.
#[test]
fn v07_char_at() {
    let out = probe::run_probe("v07-charat", |cx| {
        cx.data("t1", T1, false);
        cx.data("t3", T3, false);
        cx.set_main(Box::new(|fb, fx| {
            let p1 = probe::data_addr(fb, fx, "t1");
            let l1 = probe::const_i32(fb, 8);
            let p3 = probe::data_addr(fb, fx, "t3");
            let l3 = probe::const_i32(fb, 1);
            let i0 = probe::const_i32(fb, 0);
            let i1 = probe::const_i32(fb, 1);
            let i2 = probe::const_i32(fb, 2);
            let i3 = probe::const_i32(fb, 3);
            let im1 = probe::const_i32(fb, -1);
            let r0 = probe::call_shim(fb, fx, "p774_char_at", &[p1, l1, i0]).remove(0);
            let r1 = probe::call_shim(fb, fx, "p774_char_at", &[p1, l1, i1]).remove(0);
            let r2 = probe::call_shim(fb, fx, "p774_char_at", &[p1, l1, i2]).remove(0);
            let r3 = probe::call_shim(fb, fx, "p774_char_at", &[p1, l1, i3]).remove(0);
            let rm1 = probe::call_shim(fb, fx, "p774_char_at", &[p1, l1, im1]).remove(0);
            let rn = probe::call_shim(fb, fx, "p774_char_at", &[p3, l3, i0]).remove(0);
            let e1 = probe::expect_eq_i32(fb, r0, 65);
            let e2 = probe::expect_eq_i32(fb, r1, 20013);
            let e3 = probe::expect_eq_i32(fb, r2, 128512);
            let e4 = probe::expect_eq_i32(fb, r3, probe::status::ERR_RANGE_OOB);
            let e5 = probe::expect_eq_i32(fb, rm1, probe::status::ERR_RANGE_OOB);
            let e6 = probe::expect_eq_i32(fb, rn, 0);
            let v = probe::acc(fb, e1, e2);
            let v = probe::acc(fb, v, e3);
            let v = probe::acc(fb, v, e4);
            let v = probe::acc(fb, v, e5);
            probe::acc(fb, v, e6)
        }));
    });
    probe::assert_pass(
        &out,
        &[
            "CHAR-AT i=0 v=65",
            "CHAR-AT i=1 v=20013",
            "CHAR-AT i=2 v=128512",
        ],
    );
}

/// V07-8..12: slicing by scalar range — legal empty range, reversed and
/// out-of-bounds rejections.
#[test]
fn v07_slice() {
    let ok = probe::run_probe("v07-slice-ok", |cx| {
        cx.data("t1", T1, false);
        cx.data("t2", T2, false);
        cx.data("slen", &[0; 4], true);
        cx.set_main(Box::new(|fb, fx| {
            let p1 = probe::data_addr(fb, fx, "t1");
            let l1 = probe::const_i32(fb, 8);
            let p2 = probe::data_addr(fb, fx, "t2");
            let l2 = probe::const_i32(fb, 3);
            let sp = probe::data_addr(fb, fx, "slen");
            let s0 = probe::const_i32(fb, 0);
            let s1 = probe::const_i32(fb, 1);
            let s2 = probe::const_i32(fb, 2);
            let s3 = probe::const_i32(fb, 3);

            // slice(T1,0,1): len 1, first byte 0x41
            let a = probe::call_shim(fb, fx, "p774_text_slice", &[p1, l1, s0, s1, sp]).remove(0);
            let la = probe::load_i32(fb, sp, 0);
            let va = probe::expect_eq_i32(fb, la, 1);
            let one = probe::const_i32(fb, 1);
            let ba = probe::call_shim(fb, fx, "p774_borrow_read", &[a, one, s0]).remove(0);
            let vba = probe::expect_eq_i32(fb, ba, 0x41);
            let va = probe::acc(fb, va, vba);

            // slice(T1,1,3): len 7, byte sum 1264
            let b = probe::call_shim(fb, fx, "p774_text_slice", &[p1, l1, s1, s3, sp]).remove(0);
            let lb = probe::load_i32(fb, sp, 0);
            let vb = probe::expect_eq_i32(fb, lb, 7);
            let seven = probe::const_i32(fb, 7);
            let sb = probe::call_shim(fb, fx, "p774_borrow_sum", &[b, seven]).remove(0);
            let vsb = probe::expect_eq_i32(fb, sb, 1264);
            let vb = probe::acc(fb, vb, vsb);

            // slice(T1,1,1): empty range is legal
            let _ = probe::call_shim(fb, fx, "p774_text_slice", &[p1, l1, s1, s1, sp]).remove(0);
            let lc = probe::load_i32(fb, sp, 0);
            let vc = probe::expect_eq_i32(fb, lc, 0);

            // slice(T2,0,2): whole string, len 3
            let _ = probe::call_shim(fb, fx, "p774_text_slice", &[p2, l2, s0, s2, sp]).remove(0);
            let ld = probe::load_i32(fb, sp, 0);
            let vd = probe::expect_eq_i32(fb, ld, 3);

            let v1 = probe::acc(fb, va, vb);
            let v2 = probe::acc(fb, vc, vd);
            probe::acc(fb, v1, v2)
        }));
    });
    probe::assert_pass(
        &ok,
        &[
            "SLICE s=0 e=1 len=1",
            "SLICE s=1 e=3 len=7",
            "SLICE s=1 e=1 len=0",
        ],
    );

    let bad = probe::run_probe("v07-slice-reject", |cx| {
        cx.data("t1", T1, false);
        cx.data("slen", &[0; 4], true);
        cx.set_main(Box::new(|fb, fx| {
            let p1 = probe::data_addr(fb, fx, "t1");
            let l1 = probe::const_i32(fb, 8);
            let sp = probe::data_addr(fb, fx, "slen");
            let s0 = probe::const_i32(fb, 0);
            let s1 = probe::const_i32(fb, 1);
            let s2 = probe::const_i32(fb, 2);
            let s4 = probe::const_i32(fb, 4);
            let im1 = probe::const_i32(fb, -1);
            // reversed: (2,1)
            let _ = probe::call_shim(fb, fx, "p774_text_slice", &[p1, l1, s2, s1, sp]).remove(0);
            let lr = probe::load_i32(fb, sp, 0);
            let vr = probe::expect_eq_i32(fb, lr, probe::status::ERR_RANGE_REVERSED);
            // out of bounds: (0,4)
            let _ = probe::call_shim(fb, fx, "p774_text_slice", &[p1, l1, s0, s4, sp]).remove(0);
            let lo = probe::load_i32(fb, sp, 0);
            let vo = probe::expect_eq_i32(fb, lo, probe::status::ERR_RANGE_OOB);
            // negative start: (-1,2)
            let _ = probe::call_shim(fb, fx, "p774_text_slice", &[p1, l1, im1, s2, sp]).remove(0);
            let ln = probe::load_i32(fb, sp, 0);
            let vn = probe::expect_eq_i32(fb, ln, probe::status::ERR_RANGE_OOB);
            let v1 = probe::acc(fb, vr, vo);
            probe::acc(fb, v1, vn)
        }));
    });
    probe::assert_pass(&bad, &[]);
}

/// V07-13/14: explicit char<->code point conversion with surrogate and
/// range rejections.
#[test]
fn v07_codepoint_conversion() {
    let out = probe::run_probe("v07-cp", |cx| {
        cx.set_main(Box::new(|fb, fx| {
            let ok_cp = probe::const_i32(fb, 0x1F600);
            let surr = probe::const_i32(fb, 0xD800);
            let neg = probe::const_i32(fb, -1);
            let over = probe::const_i32(fb, 0x110000);
            let r1 = probe::call_shim(fb, fx, "p774_char_from_codepoint", &[ok_cp]).remove(0);
            let r2 = probe::call_shim(fb, fx, "p774_char_from_codepoint", &[surr]).remove(0);
            let r3 = probe::call_shim(fb, fx, "p774_char_from_codepoint", &[neg]).remove(0);
            let r4 = probe::call_shim(fb, fx, "p774_char_from_codepoint", &[over]).remove(0);
            let zh = probe::const_i32(fb, 20013);
            let r5 = probe::call_shim(fb, fx, "p774_char_to_codepoint", &[zh]).remove(0);
            let e1 = probe::expect_eq_i32(fb, r1, 128512);
            let e2 = probe::expect_eq_i32(fb, r2, probe::status::ERR_INVALID_SCALAR);
            let e3 = probe::expect_eq_i32(fb, r3, probe::status::ERR_INVALID_SCALAR);
            let e4 = probe::expect_eq_i32(fb, r4, probe::status::ERR_INVALID_SCALAR);
            let e5 = probe::expect_eq_i32(fb, r5, 20013);
            let v = probe::acc(fb, e1, e2);
            let v = probe::acc(fb, v, e3);
            let v = probe::acc(fb, v, e4);
            probe::acc(fb, v, e5)
        }));
    });
    probe::assert_pass(&out, &["FROM-CP cp=128512"]);
}

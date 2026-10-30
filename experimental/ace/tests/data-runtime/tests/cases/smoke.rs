//! Smoke: one real PE calling one shim op end to end (V02-1 values).
//! Validates the whole pipeline (Cranelift caller, rust-lld + shim link,
//! contained executor, trace channel) before the full case files land.

#![cfg(windows)]

use ace_data_conformance as probe;

#[test]
fn smoke_mix_real_pe() {
    let out = probe::run_probe("smoke-mix", |cx| {
        cx.set_main(Box::new(|fb, fx| {
            let a = probe::const_i32(fb, 17);
            let b = probe::const_i32(fb, 1);
            let c = probe::const_i32(fb, 0x4E2D);
            let d = probe::const_i32(fb, 96);
            let r = probe::call_shim(fb, fx, "p774_mix", &[a, b, c, d]).remove(0);
            probe::expect_eq_i32(fb, r, 20127)
        }));
    });
    probe::assert_pass(&out, &["MIX a=17 b=1 c=20013 d=96"]);
}

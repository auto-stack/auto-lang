// Permanent host-side accounting-state probe (r4 P774-R4-01).
//
// The cross-u32 accumulated-live-bytes boundary cannot be exercised in a
// real PE without multi-GiB allocations, so this footer — appended to the
// CURRENT shim source with only the crate-level no_std/no_builtins/
// panic_handler scaffolding stripped (same adapter the r4 review used) —
// seeds logical accounting state and drives the ORIGINAL try_alloc/
// owned_free code. Tiny physical buffers; no big-memory claims.

#[test]
fn aggregate_live_bytes_full_precision_across_u32_boundary() {
    let mut out_len = 0i32;
    let a = p774_own_make(7, 1, &mut out_len);
    let b = p774_own_make(7, 1, &mut out_len);
    assert!(!a.is_null() && !b.is_null());
    unsafe {
        let (base, _, count) = led_state();
        assert_eq!(count, 2);
        (*base.add(0)).len = i32::MAX as u32;
        (*base.add(1)).len = i32::MAX as u32;
    }
    // two logical i32::MAX resources: exactly 2^32 - 2 live bytes
    c().live_bytes = (u32::MAX as u64) - 1;
    c().peak = (u32::MAX as u64) - 1;
    // exact budget: 2^32 - 2 + 4 = 2^32 + 2 must be ADMITTED (==)
    p774_set_budget(0x1_0000_0002);
    let (q, _) = try_alloc(4).expect("exact-budget request must be admitted");
    assert_eq!(c().live_bytes, 0x1_0000_0002, "aggregate lost precision");
    // one over: rejected before the allocator
    assert!(try_alloc(2).is_err(), "over-budget request admitted");
    assert_eq!(c().live_bytes, 0x1_0000_0002);
    // release the 4-byte resource: remainder is EXACT, allowance restored
    owned_free(q);
    assert_eq!(c().live_bytes, (u32::MAX as u64) - 1, "release remainder lost precision");
    // re-request 3 bytes within the restored allowance: admitted
    let (r, _) = try_alloc(3).expect("re-request within restored allowance");
    assert_eq!(c().live_bytes, 0x1_0000_0001, "post-release accumulate lost precision");
    owned_free(r);
    // restore real lengths and the matching live-byte total, then release
    // everything: accounting balances exactly
    unsafe {
        let (base, _, _) = led_state();
        (*base.add(0)).len = 1;
        (*base.add(1)).len = 1;
    }
    c().live_bytes = 2;
    owned_free(b);
    owned_free(a);
    let c0 = c();
    assert_eq!(c0.allocs, c0.frees);
    assert_eq!(c0.live, 0);
    assert_eq!(c0.live_bytes, 0);
    assert_eq!(c0.invalid_frees, 0);
}

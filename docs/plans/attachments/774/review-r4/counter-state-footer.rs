#[test]
fn aggregate_live_bytes_must_not_saturate_or_admit_over_budget() {
    // Two logical i32::MAX resources: valid individual lengths, sum = 2^32-2.
    // Use tiny physical allocations and seed only bookkeeping state to avoid
    // allocating GiBs. All subsequent operations use the original shim code.
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
    c().live_bytes = ((u32::MAX as u64) - 1) as _;
    c().peak = ((u32::MAX as u64) - 1) as _;
    // Adding 4 consumes the budget exactly: 2^32 + 2 logical bytes.
    p774_set_budget(0x1_0000_0002);
    let (q, _) = try_alloc(4).expect("exact budget should accept 4-byte request");
    let accounted = c().live_bytes as u64;
    let extra = try_alloc(2);
    let extra_admitted = extra.is_ok();
    eprintln!("expected_live=4294967298; actual_live={accounted}; extra_2_admitted={extra_admitted}");
    if let Ok((p, _)) = extra { owned_free(p); }
    owned_free(q);
    owned_free(b);
    owned_free(a);
    assert_eq!(accounted, 0x1_0000_0002, "aggregate counter silently saturated");
    assert!(!extra_admitted, "request above the exact live-byte budget was admitted");
}
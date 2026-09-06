use.py anno_mod: scale, half, count_up, find_lo, mix, fakey

fn tap_ok(n int, name str) {
    print("ok " + n.to(str) + " - " + name)
}

fn tap_not_ok(n int, name str, diag str) {
    print("not ok " + n.to(str) + " - " + name + " # " + diag)
}

fn main() {
    // 1. float annotation: D4-authorized scalar coercion feeds comparison
    if scale(2.0) == 4 { tap_ok(1, "test_scale_float") } else { tap_not_ok(1, "test_scale_float", "got " + scale(2.0).to(str)) }

    // 2. non-integer float
    if half(5.0) == 2.5 { tap_ok(2, "test_half_float") } else { tap_not_ok(2, "test_half_float", "got " + half(5.0).to(str)) }

    // 3. int annotation
    if count_up(5) == 15 { tap_ok(3, "test_int_anno") } else { tap_not_ok(3, "test_int_anno", "got " + count_up(5).to(str)) }

    // 4. nullable annotation: None is a value (null), not an error
    var r = find_lo(9)
    if r == null {
        tap_ok(4, "test_nullable_none")
    } else {
        tap_not_ok(4, "test_nullable_none", "got " + r.to(str))
    }

    // 5. nullable hit path: inner int marshals
    if find_lo(1) == 20 { tap_ok(5, "test_nullable_hit") } else { tap_not_ok(5, "test_nullable_hit", "got " + find_lo(1).to(str)) }

    // 6. py_int explicit coercion bridge (480)
    if py_int("42") == 42 { tap_ok(6, "test_py_int_explicit") } else { tap_not_ok(6, "test_py_int_explicit", "int coercion failed") }

    // 7. py_float explicit coercion bridge (465)
    if py_float("2.5") == 2.5 { tap_ok(7, "test_py_float_explicit") } else { tap_not_ok(7, "test_py_float_explicit", "float coercion failed") }

    // 8. mixed float args
    if mix(1.5, 2.5) == 4 { tap_ok(8, "test_mixed_args") } else { tap_not_ok(8, "test_mixed_args", "got " + mix(1.5, 2.5).to(str)) }

    // 9. lying annotation (float promised, int returned): GIL float() coercion
    if fakey(1.0) == 7 { tap_ok(9, "test_lying_annotation_coerced") } else { tap_not_ok(9, "test_lying_annotation_coerced", "got " + fakey(1.0).to(str)) }
}

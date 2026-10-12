import anno_mod
from anno_mod import scale, half, count_up, find_lo, mix, fakey


def tap_ok(n, name):
    print(f"ok {n} - {name}")


def tap_not_ok(n, name, diag):
    print(f"not ok {n} - {name} # {diag}")


def main():
    if scale(2.0) == 4:
        tap_ok(1, "test_scale_float")
    else:
        tap_not_ok(1, "test_scale_float", f"got {scale(2.0)}")

    if half(5.0) == 2.5:
        tap_ok(2, "test_half_float")
    else:
        tap_not_ok(2, "test_half_float", f"got {half(5.0)}")

    if count_up(5) == 15:
        tap_ok(3, "test_int_anno")
    else:
        tap_not_ok(3, "test_int_anno", f"got {count_up(5)}")

    r = find_lo(9)
    if r is None:
        tap_ok(4, "test_nullable_none")
    else:
        tap_not_ok(4, "test_nullable_none", f"got {r}")

    if find_lo(1) == 20:
        tap_ok(5, "test_nullable_hit")
    else:
        tap_not_ok(5, "test_nullable_hit", f"got {find_lo(1)}")

    if int("42") == 42:
        tap_ok(6, "test_py_int_explicit")
    else:
        tap_not_ok(6, "test_py_int_explicit", "int coercion failed")

    if float("2.5") == 2.5:
        tap_ok(7, "test_py_float_explicit")
    else:
        tap_not_ok(7, "test_py_float_explicit", "float coercion failed")

    if mix(1.5, 2.5) == 4:
        tap_ok(8, "test_mixed_args")
    else:
        tap_not_ok(8, "test_mixed_args", f"got {mix(1.5, 2.5)}")

    if fakey(1.0) == 7:
        tap_ok(9, "test_lying_annotation_coerced")
    else:
        tap_not_ok(9, "test_lying_annotation_coerced", f"got {fakey(1.0)}")


if __name__ == "__main__":
    main()

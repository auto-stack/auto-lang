"""py_torch_infer native Python oracle (Plan 539 W1).

Mirrors tests/auto/infer.at case-for-case. Prints TAP lines; the parity
comparator joins the three backends by test name.
"""
import torch


def tap_ok(n, name):
    print(f"ok {n} - {name}")


def tap_not_ok(n, name, diag):
    print(f"not ok {n} - {name} # {diag}")


def main():
    t = torch.arange(6)

    # 1. add scalar
    a1 = t + 1
    if int(a1.sum()) == 21:
        tap_ok(1, "test_add_scalar")
    else:
        tap_not_ok(1, "test_add_scalar", f"got {a1.sum()}")

    # 2. mul scalar (elementwise)
    a2 = t * 2
    if int(a2.sum()) == 30:
        tap_ok(2, "test_mul_scalar")
    else:
        tap_not_ok(2, "test_mul_scalar", f"got {a2.sum()}")

    # 3. sub tensor-tensor
    base = t + 10
    a3 = base - t
    if int(a3.sum()) == 60:
        tap_ok(3, "test_sub_tensors")
    else:
        tap_not_ok(3, "test_sub_tensors", f"got {a3.sum()}")

    # 4. div scalar
    a4 = t / 2
    if int(a4.sum()) == 7:
        tap_ok(4, "test_div_scalar")
    else:
        tap_not_ok(4, "test_div_scalar", f"got {a4.sum()}")

    # 5. unary neg
    a5 = -t
    if int(a5.sum()) == -15:
        tap_ok(5, "test_neg")
    else:
        tap_not_ok(5, "test_neg", f"got {a5.sum()}")

    # 6. elementwise square
    a6 = t * t
    if int(a6.sum()) == 55:
        tap_ok(6, "test_mul_elementwise")
    else:
        tap_not_ok(6, "test_mul_elementwise", f"got {a6.sum()}")

    # 7. reflected mul
    a7 = 2 * t
    if int(a7.sum()) == 30:
        tap_ok(7, "test_reflect_mul")
    else:
        tap_not_ok(7, "test_reflect_mul", f"got {a7.sum()}")

    # 8. elementwise eq
    a8 = t == t
    if int(a8.sum()) == 6:
        tap_ok(8, "test_eq_elementwise")
    else:
        tap_not_ok(8, "test_eq_elementwise", f"got {a8.sum()}")

    # 9. matmul
    ma = torch.arange(6).reshape(2, 3)
    mb = torch.arange(6).reshape(3, 2)
    mm = ma.matmul(mb)
    mm00 = mm[0][1]
    if int(mm00) == 13:
        tap_ok(9, "test_matmul")
    else:
        tap_not_ok(9, "test_matmul", f"got {mm00}")

    # 10. slice getitem
    sl = slice(None, 1)
    row0 = mm[sl]
    row1 = mm[1]
    if int(row0.sum()) == 23:
        tap_ok(10, "test_getitem_slice")
    else:
        tap_not_ok(10, "test_getitem_slice", f"got {row0.sum()}")

    # 11. setitem
    ma[0, 0] = 100
    if int(ma.sum()) == 115:
        tap_ok(11, "test_setitem")
    else:
        tap_not_ok(11, "test_setitem", f"got {ma.sum()}")

    # 12. callable direct
    tensor_cls = torch.tensor
    t12 = tensor_cls([1, 2, 3])
    if int(t12.sum()) == 6:
        tap_ok(12, "test_call0")
    else:
        tap_not_ok(12, "test_call0", f"got {t12.sum()}")

    # 13. kwargs channel
    g2 = torch.arange(6).reshape(2, 3)
    cols = g2.sum(dim=0)
    if int(cols.sum()) == 15:
        tap_ok(13, "test_kwargs_dim")
    else:
        tap_not_ok(13, "test_kwargs_dim", f"got {cols.sum()}")

    # 14. no_grad context
    x = torch.arange(3) / 1
    x.requires_grad_(True)
    ng = torch.no_grad()
    ygrad = 1
    with ng:
        y = x * 2
        ygrad = int(y.requires_grad)
    if ygrad == 0:
        tap_ok(14, "test_with_no_grad")
    else:
        tap_not_ok(14, "test_with_no_grad", f"got {ygrad}")

    # 15/16. May channel semantics: Python exceptions are native — emulate
    # the Auto fallback/unwrap contract.
    try:
        t.no_such_method()
        msg = "unexpected"
    except AttributeError:
        msg = "fallback"
    if msg == "fallback":
        tap_ok(15, "test_may_exception")
    else:
        tap_not_ok(15, "test_may_exception", f"got {msg}")

    total = t.sum()
    if int(total) == 15:
        tap_ok(16, "test_may_ok")
    else:
        tap_not_ok(16, "test_may_ok", f"got {total}")

    # 17. null print/str fidelity (Plan 550 T07)
    if str(None) == "None":
        tap_ok(17, "test_print_none")
    else:
        tap_not_ok(17, "test_print_none", f"got {None}")


if __name__ == "__main__":
    main()

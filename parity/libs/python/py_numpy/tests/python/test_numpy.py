"""Python oracle for py_numpy parity (Plan 461).

Emits TAP output so the auto-parity runner can parse it with the same TAP
parser used for the AutoVM and a2py backends. Test names MUST match the Auto
test file (tests/auto/numpy.at) because the comparator joins backends by name.

Each assertion mirrors the Auto test 1:1, but computes through native numpy —
this is the oracle side of the three-way comparison (AutoVM vs a2py vs native
Python).
"""
import numpy as np


def tap_ok(n, name):
    print("ok {} - {}".format(n, name))


def tap_not_ok(n, name, diag):
    print("not ok {} - {} # {}".format(n, name, diag))


if __name__ == "__main__":
    if int(np.sin(0.0)) == 0:
        tap_ok(1, "test_sin_zero")
    else:
        tap_not_ok(1, "test_sin_zero", "got {}".format(np.sin(0.0)))
    if int(np.sqrt(16.0)) == 4:
        tap_ok(2, "test_sqrt")
    else:
        tap_not_ok(2, "test_sqrt", "got {}".format(np.sqrt(16.0)))

    if int(np.sum(np.arange(5))) == 10:
        tap_ok(3, "test_arange_sum")
    else:
        tap_not_ok(3, "test_arange_sum", "got {}".format(np.sum(np.arange(5))))
    if int(np.arange(5).mean()) == 2:
        tap_ok(4, "test_arange_mean")
    else:
        tap_not_ok(4, "test_arange_mean", "got {}".format(np.arange(5).mean()))
    if int(np.arange(9).max()) == 8:
        tap_ok(5, "test_arange_max")
    else:
        tap_not_ok(5, "test_arange_max", "got {}".format(np.arange(9).max()))

    if int(np.dot(np.arange(3), np.arange(3))) == 5:
        tap_ok(6, "test_dot")
    else:
        tap_not_ok(6, "test_dot", "got {}".format(np.dot(np.arange(3), np.arange(3))))

    m = np.arange(12).reshape(3, 4)
    if m.shape[0] == 3:
        tap_ok(7, "test_reshape_rows")
    else:
        tap_not_ok(7, "test_reshape_rows", "got {}".format(m.shape[0]))
    if m.shape[1] == 4:
        tap_ok(8, "test_reshape_cols")
    else:
        tap_not_ok(8, "test_reshape_cols", "got {}".format(m.shape[1]))

    if str(np.arange(3)) == "[0 1 2]":
        tap_ok(9, "test_array_str")
    else:
        tap_not_ok(9, "test_array_str", "got {}".format(str(np.arange(3))))
    if str(np.arange(3).dtype) == "int64":
        tap_ok(10, "test_dtype_str")
    else:
        tap_not_ok(10, "test_dtype_str", "got {}".format(str(np.arange(3).dtype)))

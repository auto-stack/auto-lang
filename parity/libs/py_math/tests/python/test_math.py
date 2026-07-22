"""Python oracle for py_math parity (Plan 369).

Emits TAP output (`ok N - <name>` / `not ok N - <name> # <diag>`) so the
auto-parity runner can parse it with the same TAP parser used for the AutoVM
and a2py backends. Test names MUST match the Auto test file
(tests/auto/test_math.at) because the comparator joins backends by name.
"""
import math


def tap_ok(n, name):
    print("ok {} - {}".format(n, name))


def tap_not_ok(n, name, diag):
    print("not ok {} - {} # {}".format(n, name, diag))


if __name__ == "__main__":
    # Integer results. The function set is chosen so that every case is
    # deterministic and yields a plain integer, keeping the P0 skeleton a clean
    # green baseline. (See README for the known AutoVM divergences on gcd/fabs
    # that motivated excluding those two from the skeleton.)
    if math.ceil(3.2) == 4:
        tap_ok(1, "test_ceil")
    else:
        tap_not_ok(1, "test_ceil", "got {}".format(math.ceil(3.2)))
    if math.floor(3.8) == 3:
        tap_ok(2, "test_floor")
    else:
        tap_not_ok(2, "test_floor", "got {}".format(math.floor(3.8)))
    if math.factorial(5) == 120:
        tap_ok(3, "test_factorial")
    else:
        tap_not_ok(3, "test_factorial", "got {}".format(math.factorial(5)))
    if math.trunc(3.7) == 3:
        tap_ok(4, "test_trunc")
    else:
        tap_not_ok(4, "test_trunc", "got {}".format(math.trunc(3.7)))
    if math.isqrt(17) == 4:
        tap_ok(5, "test_isqrt")
    else:
        tap_not_ok(5, "test_isqrt", "got {}".format(math.isqrt(17)))
    if int(math.sqrt(16)) == 4:
        tap_ok(6, "test_sqrt_int")
    else:
        tap_not_ok(6, "test_sqrt_int", "got {}".format(math.sqrt(16)))
    if int(math.pow(2, 10)) == 1024:
        tap_ok(7, "test_pow_int")
    else:
        tap_not_ok(7, "test_pow_int", "got {}".format(math.pow(2, 10)))
    # Plan 369 P1: expanded coverage. Float-returning functions are converted
    # to int for comparison (int(log2(8)) == 3); boolean-returning predicates
    # are compared against 1/0 (isfinite(5) == 1). Constants math.pi / math.e
    # are intentionally omitted: the AutoVM returns 0 for module-attribute
    # access (a known FFI limitation), so a constant test would diverge.
    if int(math.log2(8)) == 3:
        tap_ok(8, "test_log2")
    else:
        tap_not_ok(8, "test_log2", "got {}".format(math.log2(8)))
    if int(math.log10(1000)) == 3:
        tap_ok(9, "test_log10")
    else:
        tap_not_ok(9, "test_log10", "got {}".format(math.log10(1000)))
    if math.isfinite(5) == 1:
        tap_ok(10, "test_isfinite")
    else:
        tap_not_ok(10, "test_isfinite", "got {}".format(math.isfinite(5)))
    if math.isinf(5) == 0:
        tap_ok(11, "test_isinf")
    else:
        tap_not_ok(11, "test_isinf", "got {}".format(math.isinf(5)))
    if math.isnan(5) == 0:
        tap_ok(12, "test_isnan")
    else:
        tap_not_ok(12, "test_isnan", "got {}".format(math.isnan(5)))
    if int(math.sin(0)) == 0:
        tap_ok(13, "test_sin_zero")
    else:
        tap_not_ok(13, "test_sin_zero", "got {}".format(math.sin(0)))
    if int(math.cos(0)) == 1:
        tap_ok(14, "test_cos_zero")
    else:
        tap_not_ok(14, "test_cos_zero", "got {}".format(math.cos(0)))

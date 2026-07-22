"""Python oracle for py_random parity (Plan 369).

Emits TAP output (`ok N - <name>` / `not ok N - <name> # <diag>`) so the
auto-parity runner can parse it with the same TAP parser used for the AutoVM
and a2py backends. Test names MUST match the Auto test file
(tests/auto/random.at) because the comparator joins backends by name.

Design note
-----------
Random numbers are non-deterministic unless seeded. Every test re-seeds before
generating a value, so the sequence is reproducible. The parity comparator
only looks at TAP pass/fail (not diagnostics), so to detect value divergences
the comparison must be baked into the pass/fail: each test re-seeds, generates
a value, and checks it against a hard-coded expected value captured from this
same Python oracle.

This Python oracle IS the source of truth, so by construction it emits `ok`
for every case. The Auto test (random.at) performs the same seed + generate +
compare against the same expected values; if AutoVM's PyFFI reproduces the
seeded sequence it emits `ok` too (consistent), otherwise `not ok`
(classified as an AutoVM bug).

Expected values were captured from CPython 3 `random.seed(n); random.randint(a, b)`:
    seed(42)  randint(1, 100)   -> 82
    seed(42)  randint(1, 1000)  -> 655
    seed(100) randint(1, 100)   -> 19
    seed(42)  randint(1, 100)   -> 82   (re-peatability)
    seed(42)  randint(1, 10)    -> 2
    seed(7)   randint(1, 50)    -> 21
    seed(42)  randint(0, 5)     -> 5
    seed(0)   randint(1, 1000)  -> 865

`random.randrange` is intentionally omitted: its 1- and 3-argument forms hit
the same multi-argument FFI corruption as `math.gcd`/`lcm`/`perm` (a stray
value leaks to stdout and the return value is wrong), which would corrupt the
TAP stream. See the README for details.
"""
import random


def tap_ok(n, name):
    print("ok {} - {}".format(n, name))


def tap_not_ok(n, name, diag):
    print("not ok {} - {} # {}".format(n, name, diag))


if __name__ == "__main__":
    random.seed(42)
    v = random.randint(1, 100)
    if v == 82:
        tap_ok(1, "test_seed_randint")
    else:
        tap_not_ok(1, "test_seed_randint", "got {}".format(v))

    random.seed(42)
    v = random.randint(1, 1000)
    if v == 655:
        tap_ok(2, "test_seed_randint_1000")
    else:
        tap_not_ok(2, "test_seed_randint_1000", "got {}".format(v))

    random.seed(100)
    v = random.randint(1, 100)
    if v == 19:
        tap_ok(3, "test_seed100_randint")
    else:
        tap_not_ok(3, "test_seed100_randint", "got {}".format(v))

    random.seed(42)
    v = random.randint(1, 100)
    if v == 82:
        tap_ok(4, "test_seed_randint_again")
    else:
        tap_not_ok(4, "test_seed_randint_again", "got {}".format(v))

    random.seed(42)
    v = random.randint(1, 10)
    if v == 2:
        tap_ok(5, "test_seed_randint_small")
    else:
        tap_not_ok(5, "test_seed_randint_small", "got {}".format(v))

    random.seed(7)
    v = random.randint(1, 50)
    if v == 21:
        tap_ok(6, "test_seed7_randint")
    else:
        tap_not_ok(6, "test_seed7_randint", "got {}".format(v))

    random.seed(42)
    v = random.randint(0, 5)
    if v == 5:
        tap_ok(7, "test_seed_randint_zero_low")
    else:
        tap_not_ok(7, "test_seed_randint_zero_low", "got {}".format(v))

    random.seed(0)
    v = random.randint(1, 1000)
    if v == 865:
        tap_ok(8, "test_seed0_randint")
    else:
        tap_not_ok(8, "test_seed0_randint", "got {}".format(v))

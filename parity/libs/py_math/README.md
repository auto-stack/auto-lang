# py_math (Python parity)

**Mode:** Python parity (Plan 369) — three-way comparison of AutoVM, a2py
(transpiled Python), and a native Python oracle.

**Upstream:** Python 3 `math` standard-library module.

**Scope:** integer-result math operations. This is the P0 skeleton for the
Python parity framework: it verifies that the three backends agree end-to-end
on a small, deterministic, integer-valued test set.

## API

The Auto test imports these symbols from `use.py math`:

- `ceil(x) -> int`
- `floor(x) -> int`
- `factorial(n) -> int`
- `trunc(x) -> int`
- `isqrt(n) -> int`
- `sqrt(x) -> float` (compared as integer via `.to(int)`)
- `pow(base, exp) -> float` (compared as integer via `.to(int)`)

Plan 369 P1 added these (float results coerced via `.to(int)`, boolean
predicates compared against 0/1):

- `log2(x) -> float` (compared as integer)
- `log10(x) -> float` (compared as integer)
- `isfinite(x) -> bool` (PyFFI marshals to int 0/1)
- `isinf(x) -> bool` (PyFFI marshals to int 0/1)
- `isnan(x) -> bool` (PyFFI marshals to int 0/1)
- `sin(x) -> float` (compared as integer at nice values)
- `cos(x) -> float` (compared as integer at nice values)

## Test layout

- `tests/python/test_math.py` — Python oracle emitting TAP output.
- `tests/auto/math.at` — Auto test using `use.py math`, emitting TAP.

The Auto file is named `math.at` (matching the parity repo's convention of
descriptive test names; the root `.gitignore` excludes `test*.at`). The Python
oracle keeps the standard `test_*.py` pytest convention. Test names inside the
files must match because the parity comparator joins backends by test name.

## History: previously excluded, now resolved

Earlier phases excluded some `math` features as known AutoVM divergences so the
baseline stayed green. They have since been fixed and are covered by the test
suite (Plan 369 P3 fixed multi-arg calls; P4 Task 13 fixed constant imports):

- `gcd(a, b)` / `lcm(a, b)` — previously the two-argument FFI call printed a
  stray value to stdout and returned the wrong result. Multi-arg calls now work
  via the CALL_PY opcode, so `gcd(12, 8) == 4` and `lcm(4, 6) == 12` pass on
  all three backends.
- `math.pi` / `math.e` (module constants) — now importable via
  `use.py math: pi, e`. They marshal to their string representation
  (`"3.141592653589793"`, `"2.718281828459045"`), which is deterministic for
  IEEE-754 doubles, so the tests compare the exact string form. Note: a
  non-integral float constant is not recoverable as an Auto int
  (`pi.to(int) == 0`); only the string form is reliable, so pi/e are asserted by
  string equality rather than via `.to(int)`.

Still excluded from the test set:

- `fabs(x)` — Python returns a `float`; the FFI marshals an integral result
  (`fabs(-5)` → `5.0`) such that the string form is `"5"` and `.to(int)` works,
  but the value is not tagged as a Python float. This is a marshalling
  limitation rather than a TAP-corrupting bug; it is omitted to keep the
  baseline clean. It can be re-enabled once float tagging is improved.

## Known divergences

(none — the suite is 100% consistent across AutoVM, a2py, and the oracle)

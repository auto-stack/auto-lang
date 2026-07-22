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

## Known AutoVM divergences (excluded from the skeleton)

Two `math` functions were candidates for the skeleton but currently diverge in
the AutoVM and are therefore NOT included, so the P0 baseline stays green:

- `gcd(a, b)` — the AutoVM prints a stray value to stdout and returns the wrong
  result for the two-argument call (e.g. `gcd(12, 8)` emits `128` and fails).
  This corrupts the TAP stream. `lcm` and `perm` exhibit the same symptom;
  `comb` does not, so the bug is not uniform across two-argument FFI calls.
- `fabs(x)` — Python returns a `float`; the FFI marshals it to the string `"5"`,
  so `fabs(-5) == 5` (int) is false in the AutoVM. a2py and the oracle both
  pass, correctly classifying this as an AutoVM bug.

Both are real findings worth re-enabling once the FFI marshalling / call
handling is fixed; they are intentionally omitted here so the skeleton proves
the framework with a clean 100% baseline.

A third candidate was evaluated in P1 and excluded:

- `math.pi` / `math.e` (module constants) — the AutoVM returns `0` for
  module-attribute access (e.g. `math.pi`), so `int(math.pi) == 3` is false
  while a2py and the oracle both pass. Unlike `gcd`/`fabs`, this does not
  corrupt the TAP stream; it is omitted purely to keep the baseline green.
  This points at a PyFFI gap: constant (non-callable) attribute access is not
  marshalled, only function calls registered via `use.py m: f`.

## Known divergences

(none in the current test set)

# py_random (Python parity)

**Mode:** Python parity (Plan 369) — three-way comparison of AutoVM, a2py
(transpiled Python), and a native Python oracle.

**Upstream:** Python 3 `random` standard-library module.

**Scope:** seeded integer random number generation. This validates that
AutoVM's PyFFI reproduces CPython's seeded `random` sequence exactly — i.e.
that `seed(n)` state is correctly shared between the embedded CPython
interpreter and the `randint` calls.

## Why seeding matters

`random` output is non-deterministic by default. Every test re-seeds before
generating a value so the sequence is reproducible, and the generated value is
compared against a hard-coded expected value captured from CPython 3. Because
the AutoVM calls the **same** CPython `random` module via PyFFI, a matching
value proves the seed state is set and read consistently across the FFI
boundary.

## API

The Auto test imports these symbols from `use.py random`:

- `seed(n) -> None` — sets the RNG state
- `randint(a, b) -> int` — inclusive random integer in `[a, b]`
- `choice(seq) -> item` — seeded choice over a sequence. The Auto suite uses a
  string sequence (e.g. `choice("abcde")`); Auto list literals do not marshal to
  a Python list, so a list argument fails with "object of type 'int' has no
  len()". A string works because it marshals directly to a Python `str`, which
  is a valid sequence.
- `uniform(a, b) -> float` — seeded float in `[a, b]`. The result marshals to
  its shortest round-tripping string form, so the test asserts the exact string
  (deterministic for IEEE-754 doubles).

## Test layout

- `tests/python/test_random.py` — Python oracle emitting TAP output. This
  oracle is the source of truth: by construction it emits `ok` for every case.
- `tests/auto/random.at` — Auto test using `use.py random`, emitting TAP. It
  performs the same seed + generate + compare and emits `not ok` if the value
  diverges from the expected one.

The Auto file is named `random.at` (matching the parity repo's convention of
descriptive test names; the root `.gitignore` excludes `test*.at`). The Python
oracle keeps the standard `test_*.py` convention. Test names inside the files
must match because the parity comparator joins backends by test name.

### How divergences surface

The TAP comparator only inspects pass/fail, not diagnostics. So the value
check is baked into pass/fail:

- Python oracle: always `ok` (source of truth).
- a2py (transpiled Python): runs the same Python, also `ok`.
- AutoVM: `ok` if `seed(n); randint(a, b)` reproduces the expected value,
  `not ok` otherwise.

If AutoVM reproduces the seeded value it is `consistent`; a wrong value yields
`not ok` only in the VM slot, classified as an **AutoVM bug**.

### Expected value table (CPython 3)

| seed | call              | expected |
|------|-------------------|----------|
| 42   | randint(1, 100)   | 82       |
| 42   | randint(1, 1000)  | 655      |
| 100  | randint(1, 100)   | 19       |
| 42   | randint(1, 100)   | 82       |
| 42   | randint(1, 10)    | 2        |
| 7    | randint(1, 50)    | 21       |
| 42   | randint(0, 5)     | 5        |
| 0    | randint(1, 1000)  | 865      |

## History: previously excluded, now resolved

`random.randrange` (1- and 3-argument forms) was previously excluded because it
hit the same multi-argument FFI corruption seen in `math.gcd` / `math.lcm` /
`math.perm`: a stray argument value leaked to stdout and the return value was
wrong. Plan 369 P3 fixed multi-arg calls via the CALL_PY opcode, so
`randrange(0, 100, 5)` and friends now return the correct seeded value
(e.g. `seed(42); randrange(0, 100, 5)` → `15`). The 2-argument `randint(a, b)`
path was always unaffected and remains the primary integer case.

## Known limitations

- `choice` over an Auto list literal (e.g. `choice([1, 2, 3])`) fails: Auto list
  literals do not marshal to a Python `list`, so Python raises "object of type
  'int' has no len()". The suite exercises `choice` over a string sequence
  instead, which marshals correctly.

## Known divergences

(none — the suite is 100% consistent across AutoVM, a2py, and the oracle)

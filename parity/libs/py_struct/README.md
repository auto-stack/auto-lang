# py_struct Parity

**Python module:** `struct` (stdlib)
**C backend:** C module
**Mode:** Python parity (Plan 369) — three-way comparison of AutoVM, a2py
(transpiled Python), and a native Python oracle.
**Scope:** `calcsize`, plus `pack`/`unpack` round-trips.

## API

The Auto test imports these symbols from `use.py struct`:

- `calcsize(fmt) -> int` — single string arg, int return.
- `pack(fmt, *values) -> bytes` — multi-arg FFI (format string + values).
- `unpack(fmt, buffer) -> tuple` — returns a tuple of the decoded values.

`pack` returns `bytes` and `unpack` returns a `tuple`; both marshal to opaque
heap values in the AutoVM. The test reaches the decoded integers via
`py_call(tup, "__getitem__", i)` (Plan 369 Task 12 object-method shim), which
lets a pack→unpack round-trip be compared against the original input.

## Test layout

- `tests/python/test_struct.py` — Python oracle emitting TAP output.
- `tests/auto/struct.at` — Auto test using `use.py struct`, emitting TAP.

Test names match across the two files because the parity comparator joins
backends by test name.

## History (previously blocking, now resolved)

- `pack`/`unpack` — previously untestable because the PyFFI only forwarded the
  first argument to the Python call. Plan 369 P3 fixed multi-arg calls via the
  CALL_PY opcode (e.g. `pack(">I", 258)`, `pack(">HH", 1, 2)`).

## Known divergences

(none — the suite is 100% consistent across AutoVM, a2py, and the oracle)

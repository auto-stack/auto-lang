# py_uuid Parity

**Python module:** `uuid` (stdlib)
**C backend:** `_uuid` C module
**Mode:** Python parity (Plan 369) — three-way comparison of AutoVM, a2py
(transpiled Python), and a native Python oracle.

## Scope

`uuid5(namespace, name)` — the deterministic, SHA-1 based UUID generator. It is
the only fully deterministic constructor in the `uuid` module (`uuid1` is
time/MAC based, `uuid4` is random), so it is the natural parity target.

## API

The Auto test imports these symbols from `use.py uuid`:

- `uuid5(namespace, name)` — deterministic UUID; the first argument is a UUID
  object (a module constant), the second is the name string.
- `NAMESPACE_DNS`, `NAMESPACE_URL` — Python module constants (non-callable
  attributes), imported directly via `use.py uuid: NAMESPACE_DNS`.

The returned `UUID` object marshals to an opaque heap value in the AutoVM, so
the test reaches its string form via `py_call(u, "__str__")` (Plan 369 Task 12
object-method shim).

## Test layout

- `tests/python/test_uuid.py` — Python oracle emitting TAP output.
- `tests/auto/uuid.at` — Auto test using `use.py uuid`, emitting TAP.

Test names match across the two files because the parity comparator joins
backends by test name.

## History (previously blocking, now resolved)

These were documented limitations in earlier phases and are no longer
applicable:

- `NAMESPACE_DNS` / `NAMESPACE_URL` — Python module constants. Plan 369 P3
  added constant imports via `register_constant`; P4 Task 13 additionally
  fixed the parser's symbol checker so imported constants resolve in argument
  position (previously they only resolved in method-call position like
  `pi.to(str)`).
- `uuid5(namespace, name)` — needs a UUID object as the first argument and a
  two-argument FFI call. Plan 369 P3 fixed multi-arg calls via the CALL_PY
  opcode.

## Known divergences

(none — the suite is 100% consistent across AutoVM, a2py, and the oracle)

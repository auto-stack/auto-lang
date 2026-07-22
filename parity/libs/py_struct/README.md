# py_struct Parity

**Python module:** `struct` (stdlib)
**C backend:** C module
**Scope:** `calcsize` only (single string arg, int return).
**NOT covered:** `pack`/`unpack` — require multi-arg FFI (format string + values) which is
currently broken in PyFFI (only first arg is passed). See DIV-PY-MULTIARG-1.

## Known divergences

- `pack`/`unpack` untestable due to PyFFI multi-arg limitation

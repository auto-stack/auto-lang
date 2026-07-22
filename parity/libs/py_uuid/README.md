# py_uuid Parity

**Python module:** `uuid` (stdlib)
**C backend:** `_uuid` C module
**Scope:** Currently BLOCKED — uuid5 requires module constant (NAMESPACE_DNS) and
UUID object argument, neither supported by PyFFI.

## Known limitations

- `NAMESPACE_DNS` — Python module constant, not importable via `use.py` (DIV-PY-CONST-1)
- `uuid5(namespace, name)` — needs UUID object as first arg + 2-arg FFI

## Status

Python oracle runs the full uuid5 reference suite. Auto test is a stub.

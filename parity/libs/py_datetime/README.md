# py_datetime Parity

**Python module:** `datetime` (stdlib)
**C backend:** `_datetime` C module
**Scope:** Currently BLOCKED — most datetime operations require multi-arg FFI or
Python object method calls, neither of which work in current PyFFI.

## Known limitations

- `date(y, m, d)` requires 3-arg FFI (DIV-PY-MULTIARG-1) — only first arg passed
- `d.isoformat()` / `d.year` — Python object method/attribute access not supported
- `datetime.date.today()` — bare module dot-call doesn't resolve

## Status

The Python oracle (`test_datetime.py`) contains the full expected test suite.
The Auto test (`datetime.at`) is a stub that documents the limitation.
When PyFFI multi-arg support is added, enable the matching Auto tests.

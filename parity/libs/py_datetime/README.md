# py_datetime Parity

**Python module:** `datetime` (stdlib)
**C backend:** `_datetime` C module
**Mode:** Python parity (Plan 369) — three-way comparison of AutoVM, a2py
(transpiled Python), and a native Python oracle.
**Scope:** VM parity achieved for object construction, methods, and attributes.

## Status

Plan 369 Task 10 fixed multi-arg Python FFI calls (`date(y, m, d)`).
Plan 369 Task 12 added Python object method calls and attribute access via the
`py_call(obj, method, ...args)` and `py_getattr(obj, attr)` VM built-ins, backed
by opaque `PyObjectHandle` storage in the VM heap (the live Python object is kept
rather than stringified).

The Auto test (`datetime.at`) exercises all five cases and matches the Python
oracle on every backend (100% three-way consistency):
- `test_date_isoformat` — `py_call(d, "isoformat")`
- `test_date_add_30_days` — `py_call(d, "__add__", timedelta(30))`
- `test_date_components` — `py_getattr(d, "year"|"month"|"day")`
- `test_weekday` — `py_call(d, "weekday")`
- `test_toordinal` — `py_call(d, "toordinal")`

The a2py transpiler lowers `py_call(o, "m", ...)` to `o.m(...)` and
`py_getattr(o, "a")` to `o.a`, so the transpiled Python is valid and produces
the same TAP output as the oracle.

## Known limitations

- `datetime.date.today()` bare module dot-call is not exercised here (classmethod
  dispatch is a separate concern from instance method calls).

## Known divergences

(none — the suite is 100% consistent across AutoVM, a2py, and the oracle)

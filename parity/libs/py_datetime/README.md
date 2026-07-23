# py_datetime Parity

**Python module:** `datetime` (stdlib)
**C backend:** `_datetime` C module
**Scope:** VM parity achieved for object construction, methods, and attributes.

## Status

Plan 369 Task 10 fixed multi-arg Python FFI calls (`date(y, m, d)`).
Plan 369 Task 12 added Python object method calls and attribute access via the
`py_call(obj, method, ...args)` and `py_getattr(obj, attr)` VM built-ins, backed
by opaque `PyObjectHandle` storage in the VM heap (the live Python object is kept
rather than stringified).

The Auto test (`datetime.at`) now exercises all five cases and matches the Python
oracle on the AutoVM backend:
- `test_date_isoformat` — `py_call(d, "isoformat")`
- `test_date_add_30_days` — `py_call(d, "__add__", timedelta(30))`
- `test_date_components` — `py_getattr(d, "year"|"month"|"day")`
- `test_weekday` — `py_call(d, "weekday")`
- `test_toordinal` — `py_call(d, "toordinal")`

## Known limitations

- The a2r (Auto-to-Python transpiler) backend does not understand `py_call` /
  `py_getattr` — it emits them verbatim into the transpiled Python, which then
  crashes with `NameError: name 'py_call' is not defined`. This is a transpiler
  limitation, not a VM bug; the harness classifies it as "test case issue".
  Fixing it would require the transpiler to lower `py_call(o, "m", ...)` to
  `o.m(...)` and `py_getattr(o, "a")` to `o.a`.
- `datetime.date.today()` bare module dot-call is not exercised here (classmethod
  dispatch is a separate concern from instance method calls).

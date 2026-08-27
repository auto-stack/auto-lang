# py_numpy (Python sci-compute parity, Plan 461)

numpy parity through the Python FFI bridge (`use.py numpy: ...` plus
`py_call`/`py_getattr` on opaque handles). Three-way: AutoVM vs a2py vs native
Python.

## Scope

Scalar ufuncs (`sin`, `sqrt`), reductions over Python-side arrays
(`sum`/`mean`/`max`), `dot`, reshape + shape tuple access, and deterministic
string forms (`str(array)`, dtype name).

## Calling conventions exercised

- Idiom A (item import) for module functions: `use.py numpy: sin, arange, sum`.
- Idiom B for members of returned objects: `py_call(arr, "mean")`,
  `py_getattr(arr, "dtype")` → `py_getattr(..., "name")`.
- Tuples (`shape`) marshal as handles — read elements via
  `py_call(shape, "__getitem__", i)`.
- Arrays are created Python-side (`arange`); Auto lists are NOT passed into
  Python (DIV-PY-AUTOLIST-1).

## Files

- `tests/auto/numpy.at` — Auto TAP cases (10)
- `tests/python/test_numpy.py` — native Python oracle

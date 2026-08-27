# py_pandas (Python sci-compute parity, Plan 461)

pandas parity through the Python FFI bridge. Three-way: AutoVM vs a2py vs
native Python.

## Scope

DataFrame construction from a numpy handle (`DataFrame(arange(12).reshape(4,3))`),
shape/len access, column aggregations (`sum().min()/max()`), row selection
(`iloc[0].min()`), `.values` round-trip, and runtime class-name assertion
(`Series`).

## Notes

- `df.values` and `df.shape` are attributes — read via `py_getattr`, not
  `py_call`.
- Expected values: column sums of arange(12) reshaped 4x3 are 18 / 22 / 26.
- Auto dicts/lists are not passed into Python (DIV-PY-AUTOLIST-1); the frame is
  built Python-side.

## Files

- `tests/auto/pandas.at` — Auto TAP cases (8)
- `tests/python/test_pandas.py` — native Python oracle

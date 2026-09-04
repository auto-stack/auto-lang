# py_torch (Python sci-compute parity, Plan 461)

torch parity through the Python FFI bridge. Three-way: AutoVM vs a2py vs
native Python.

## Scope

Tensor creation Python-side (`arange`/`ones`/`zeros`/`linspace`), reductions
(`sum`), functional activations (`relu`, `abs`), and deterministic runtime
type strings (`torch.LongTensor` / `torch.FloatTensor`).

## Notes

- Plan 539 W2 update: 0-dim tensors now stay OPAQUE handles (only exact
  Python floats marshal to f64) — extract scalars with `py_float(x)`
  (`float(x)`). Real float returns (`.item()`, math functions) still land
  as f64 directly.
- Never use arithmetic operators on handles (`t - 0` degrades the handle to
  an int); use torch functional forms instead.
- `use.py` has no import aliasing and torch/numpy both export `arange`, so
  both modules cannot be imported in one file — the numpy↔torch `from_numpy`
  interop case is therefore deferred.

## Files

- `tests/auto/torch.at` — Auto TAP cases (7)
- `tests/python/test_torch.py` — native Python oracle

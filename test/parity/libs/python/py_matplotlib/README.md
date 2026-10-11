# py_matplotlib (Python sci-compute parity, Plan 461)

matplotlib (pyplot) parity through the Python FFI bridge. Three-way: AutoVM vs
a2py vs native Python.

## Scope

The rendering contract is FILE OUTPUT (headless): `plot` + `savefig` +
artifact assertions (`os.path.getsize > 0`, `> 1000`). No interactive windows;
the default backend resolves headless inside the embedded interpreter and in
plain CPython alike.

## Notes

- Line data comes from numpy handles; Auto list literals must NOT be passed
  to pyplot (DIV-PY-AUTOLIST-1 — it crashes the VM side).
- The zero-arg `pyplot.figure()` mis-marshals an argument through the FFI
  (spike finding); pyplot state creates figures implicitly, so it is skipped.
- `os.makedirs` uses positional exist_ok: `makedirs(dir, 511, 1)` (kwargs are
  not supported by the FFI). Output lands in `py_matplotlib_tmp/` (gitignored).
- Dotted module paths work: `use.py matplotlib.pyplot: plot, savefig, close`.

## Files

- `tests/auto/matplotlib.at` — Auto TAP cases (3)
- `tests/python/test_matplotlib.py` — native Python oracle

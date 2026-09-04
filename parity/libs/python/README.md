# libs/python/ — Python parity suites (Plan 369/461/539)

Three-way parity against a Python oracle: AutoVM (embedded CPython via
PyO3, `--features python`) vs a2py (transpiled Python) vs native Python.
Parity mode is auto-detected from each lib's `tests/python/` directory.

## Calling conventions (Plan 539 W0)

The FFI surface as of Plan 539 W0 — all forms below are asserted by the
W1/W2 suites and probed in `scratch/p539/` during development:

- **Item import** — `use.py torch: arange, ones` (also submodules:
  `use.py torch.nn: Linear` works). Non-callable attributes (e.g.
  `math.pi`) register as zero-arg constants (Plan 369).
- **Method calls** — `py_call(obj, "method", args...)` positional, and
  `py_call(obj, "method", pos..., name: value)` for keyword arguments
  (DIV-PY-KWARGS-1 cleared; Auto named-arg syntax is `k: v`, **not**
  `k = v` — the latter parses as an assignment expression and passes
  positionally). a2py lowers both to `obj.method(pos..., name=value)`.
- **May-valued calls** — `py_call_may(obj, "method", args...)` returns
  `Result.Ok(value)` / `Result.Err("PyException <Type>: <msg>")`; consume
  with `expr.?` (propagate) or `expr.?(default)` (fallback). `py_call`
  stays strict — its exceptions abort unless wrapped in try/catch.
- **Iteration** — `py_iter(x)` → iterator handle, `py_next(it)` → value
  or null (StopIteration); `for x in handle` iterates sized/indexed py
  objects (tensors, lists, dicts) through the index channel. Loaders and
  generators use the manual `while b != null` pattern — Auto's
  `for x, y in z` tuple-destructure form does not unpack like Python and
  must not be used on py handles.
- **Attribute access** — `py_getattr(obj, "attr")` (a2py: `obj.attr`).

## Constants & module handles (DIV-PY-CONST-1)

- **Same-name items across modules silently last-win** (`use.py torch:
  arange` + `use.py numpy: arange` — the second import shadows the
  first). Avoid importing same-named items from two py modules in one
  file; there is no alias syntax (`use.py torch as t` does not exist —
  deliberately out of scope, recorded on the python-parity roadmap).
- **Module handle workaround** — get an opaque module handle via
  importlib and route constants/members through `py_getattr`:

  ```
  use.py importlib: import_module
  var torch = import_module("torch")
  var no_grad = py_getattr(torch, "no_grad")   # context manager class
  var t = py_call(py_getattr(torch, "arange"), "__call__", 5)
  ```

- **Bare module dot-call is broken** — `use.py torch` (no items) plus
  `torch.arange(5)` fails with "Undefined variable" (Plan 300-era
  feature bit-rotted; probe p8a, Plan 539). Use the importlib handle
  above instead.

## Marshalling notes

- Auto array arguments marshal to Python lists (nested arrays and
  objects too — DIV-PY-AUTOLIST-1 cleared in Plan 539 W0).
- Scalar marshalling (W2 rule): only EXACT Python floats (`.item()`,
  math functions, numpy scalars) push a real f64; 0-dim TENSORS stay
  opaque handles so `backward` keeps working — extract with
  `py_float(x)`. `.to(str)`/`.to(int)`/`.to(float)` are tag-dispatched.
- Training loop (Plan 539 W2): kwargs constructors (`Linear(2, 1, bias:
  false)`, `SGD(params, lr: 0.1)`), seeded convergence loop (forward →
  MSELoss → zero_grad/backward/step), tuple/dict round-trips — see the
  `py_torch_train` suite.
- Known a2py gap: Auto float literals emit as Python ints (`1.0` → `1`)
  — pre-existing, suite authors must keep this in mind for dtype
  sensitive assertions.

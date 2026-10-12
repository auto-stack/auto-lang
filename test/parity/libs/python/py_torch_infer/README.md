# py_torch_infer (Plan 539 W1 inference-idiom parity)

torch inference idioms through the Python FFI bridge. Three-way: AutoVM vs
a2py vs native Python. Builds on `py_torch` (Plan 461) — this suite covers
the W1 operator/idiom layer cleared by Plan 539.

## Scope (16 cases)

- **dunder routing** — `+ - * /` on handle operands (scalar and tensor),
  unary `-`, elementwise `==` (torch semantics: comparisons yield bool
  tensors), reflected operators (`2 * t` via `__rmul__`).
- **`*` is elementwise** — matmul is the explicit `py_matmul(a, b)` function
  form (→ `a.matmul(b)` in a2py); no operator overload, per the Plan 539
  parity decision.
- **indexing** — `py_getitem` (multi-index → tuple key), `py_slice(null, 1)`
  for unbounded slices (`mm[:1]`), `py_setitem`.
- **callable direct** — `py_call0(fn, args)` (class constructors, modules).
- **kwargs** — `py_call(obj, "m", dim: 0)` (Auto named-arg syntax is
  `k: v`).
- **context manager** — `py_with(no_grad_instance, (ctx) => { ... })`.
- **May channel** — `py_call_may(...).?(default)` fallback and Ok unwrap.

## Notes

- `*`/`==` semantics pinned: elementwise, matching torch/numpy and a2py
  output verbatim — there is no `*`-as-matmul path anywhere.
- Composite receivers use intermediate variables: a2py does not
  parenthesize nested expressions in method receivers (`t == t.sum()`
  would mis-bind).
- 0-dim tensor results stay OPAKE handles (W2 rule) — extract scalars
  with `py_float(x)`; never `.to(str)`/`.to(int)` a tensor handle.
- `py_with(ctx, () => { body })` compiles to an INLINE bracket —
  `py_enter(ctx); body; py_exit(ctx)` — because py handles in closure
  locals degrade to raw ids (DIV-PY-CLOSURE-1, pre-existing). The body
  runs in the enclosing scope; an exception in the body skips
  `__exit__` (unlike Python's guaranteed-exit).

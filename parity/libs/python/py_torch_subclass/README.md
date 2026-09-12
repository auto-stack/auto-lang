# py_torch_subclass (Plan 602 — Python class derivation factory)

Custom `nn.Module` / `Dataset` subclasses whose methods are Auto closures,
driven three-way: AutoVM vs a2py vs native Python. Everything seeded
(`torch.manual_seed(0)`) and CPU-deterministic.

## Scope (4 cases)

- **custom module forward** — `py_subclass("MyNet", nn.Module, {...})`:
  Python-source `__init__` (exec inline, self-contained imports) builds the
  layer; the Auto `forward` closure is the callback. Instantiate via
  `py_call0` (Python type call), drive a `ones(4)` batch through, pin the
  first output element (seeded).
- **Dataset protocol** — Auto `__len__` (self-only callback) and Auto
  `__getitem__` (self + index, n-arg ABI) drive the Python protocol through
  the derived class.
- **Auto-driven training loop** — 10 ticks of `zero_grad`/`backward`/`step`
  over the derived model; seeded convergence (`first > last`, `last < 0.001`).
- **reentrancy depth probe (D5)** — `outer`(Python) → `mid`(Auto closure) →
  `leaf`(Python) at ≥3 nested shim levels; arithmetic result pinned (18).

## Conventions

- **self-first handle**: a callback closure's first parameter is the instance
  handle; the Auto side keeps operating on Python state through it
  (`py_getattr(self, ...)`, `py_call(self, ...)`). Declare-and-ignore is fine
  for `__len__`.
- **n-arg callback ABI**: Python-side call arguments marshal in tuple order;
  the closure's declared arity must match exactly (arity mismatch → Python
  `TypeError` with expected/actual).
- **window discipline (T02)**: callbacks fire only inside a host
  py_call-family shim window (GIL on the VM thread, num_workers=0). Python
  never runs outside a shim in embedded mode, so the out-of-window
  RuntimeError is pinned by py_ffi unit tests rather than a corpus case.
- **base classes ride module-getattr** — `use.py` imports bind callables; a
  bare imported class reference would CALL the type. Fetch bases via
  `py_getattr(module, "Class")`.
- **Str methods are self-contained** — the factory exec's the class template
  with only `__base__` in the namespace; method sources import what they need
  (`import torch.nn as nn` inside `__init__`).
- **num_workers=0** (single-threaded GIL on the VM thread; DataLoader workers
  would fire callbacks off-window — unsupported, see roadmap §7.3).
- **torch CPU, `manual_seed(0)`** before every construction.

## Known divergences

- none recorded for this suite; the derived-class mechanics are byte-equal
  across the three backends on the pinned cases.

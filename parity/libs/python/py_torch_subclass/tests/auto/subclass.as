// py_torch_subclass Auto test — TAP output format (Plan 602 py class
// derivation factory). Three-way: AutoVM vs a2py vs native Python.
//
// The parity runner sets the working directory to the library root, so
// `use.py torch` resolves the installed torch package through the embedded
// CPython (PyO3) bridge.
//
// Test names MUST match tests/python/test_subclass.py (comparator joins by
// name).
//
// Conventions (Plan 602):
// - py_subclass(name, base, methods): Str values are Python source methods
//   (exec inline); closure values are Auto callback methods — their first
//   parameter is the instance handle (self), and Python-side calls marshal
//   arguments in tuple order (n-arg callback ABI, arity mismatch = TypeError).
// - Class handles instantiate via py_call0 (Python type call: __init__ runs
//   normally).
// - Base classes ride the module-getattr channel (use.py imports bind
//   callables — a bare `object` reference would CALL the type).
// - Callbacks are only legal inside a host py_call-family shim window
//   (num_workers=0, GIL on the VM thread).

use.py importlib: import_module

fn tap_ok(n int, name str) {
    print("ok " + n.to(str) + " - " + name)
}

fn tap_not_ok(n int, name str, diag str) {
    print("not ok " + n.to(str) + " - " + name + " # " + diag)
}

fn main() {
    // Base classes ride the module-getattr channel (use.py imports bind
    // callables — a bare `object` reference would CALL the type).
    var object_t = py_getattr(import_module("builtins"), "object")

    // 1. custom module: Python-source __init__ builds the layer, Auto
    //    forward is the callback (self-first handle convention).
    var MyAdder = py_subclass("MyAdder", object_t, {
        __init__: "def __init__(self, k):\n        self.k = k",
        forward: (self, x) => py_getattr(self, "k") + x
    })
    var m = py_call0(MyAdder, 10)
    var y = py_call(m, "forward", 5)
    if y == 15 { tap_ok(1, "test_custom_module_forward") } else { tap_not_ok(1, "test_custom_module_forward", "got " + y.to(str)) }
}

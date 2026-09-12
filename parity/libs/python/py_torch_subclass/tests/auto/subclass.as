// py_torch_subclass Auto test — TAP output format (Plan 602 py class
// derivation factory). Three-way: AutoVM vs a2py vs native Python.
//
// The parity runner sets the working directory to the library root, so
// `use.py importlib` resolves through the embedded CPython (PyO3) bridge.
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
// - Window discipline (T02, num_workers=0): Auto callbacks fire only inside
//   a host py_call-family shim window; a callback reaching Python without
//   one raises RuntimeError (guard pinned by py_ffi unit tests, not by
//   corpus — Python never runs outside a shim in embedded mode).
// - torch CPU, seed fixed (manual_seed(0)) before every construction.
use.py importlib: import_module

fn tap_ok(n int, name str) {
    print("ok " + n.to(str) + " - " + name)
}

fn tap_not_ok(n int, name str, diag str) {
    print("not ok " + n.to(str) + " - " + name + " # " + diag)
}

fn main() {
    var torch = import_module("torch")
    var nn = py_getattr(torch, "nn")
    var object_t = py_getattr(import_module("builtins"), "object")

    // ---- 1. custom nn.Module: Python-source __init__ builds the layer,
    //         Auto forward is the callback (self-first handle convention).
    py_call0(py_getattr(torch, "manual_seed"), 0)
    var MyNet = py_subclass("MyNet", py_getattr(nn, "Module"), {
        __init__: "def __init__(self):\n        import torch.nn as nn\n        super().__init__()\n        self.lin = nn.Linear(4, 1)",
        forward: (self, x) => py_call0(py_getattr(self, "lin"), x)
    })
    var net = py_call0(MyNet)
    var xb = py_call0(py_getattr(torch, "ones"), 4)
    var y1 = py_call0(net, xb)
    var v1 = py_float(py_getitem(y1, 0))
    if v1 == -0.7075908780097961 {
        tap_ok(1, "test_custom_module_forward")
    } else {
        tap_not_ok(1, "test_custom_module_forward", "got " + v1.to(str))
    }

    // ---- 2. Dataset subclass: Auto __len__ (self-only callback) and
    //         Auto __getitem__ (self + index) drive the protocol.
    var data_mod = py_getattr(py_getattr(torch, "utils"), "data")
    var MyData = py_subclass("MyData", py_getattr(data_mod, "Dataset"), {
        __init__: "def __init__(self):\n        self.data = [3, 1, 4, 1, 5]",
        __len__: (self) => py_call0(py_getattr(py_getattr(self, "data"), "__len__")),
        __getitem__: (self, i) => py_getitem(py_getattr(self, "data"), i)
    })
    var ds = py_call0(MyData)
    var dlen = py_call(ds, "__len__")
    var d0 = py_call(ds, "__getitem__", 0)
    var d4 = py_call(ds, "__getitem__", 4)
    if dlen == 5 && d0 == 3 && d4 == 5 {
        tap_ok(2, "test_dataset_len_getitem")
    } else {
        tap_not_ok(2, "test_dataset_len_getitem", "len=" + dlen.to(str) + " d0=" + d0.to(str) + " d4=" + d4.to(str))
    }

    // ---- 3. Auto-driven training loop (num_workers=0 discipline: the
    //         loop crosses a shim every iteration, callbacks stay legal).
    py_call0(py_getattr(torch, "manual_seed"), 0)
    var Reg = py_subclass("Reg", py_getattr(nn, "Module"), {
        __init__: "def __init__(self):\n        import torch.nn as nn\n        super().__init__()\n        self.lin = nn.Linear(2, 1)",
        forward: (self, x) => py_call0(py_getattr(self, "lin"), x)
    })
    var reg = py_call0(Reg)
    var mse = py_call0(py_getattr(nn, "MSELoss"))
    var params = py_call(py_getattr(reg, "parameters"), "__call__")
    var sgd = py_call0(py_getattr(py_getattr(torch, "optim"), "SGD"), params, 0.05)
    var xin = py_call0(py_getattr(torch, "ones"), 2)
    var target = py_call0(py_getattr(torch, "zeros"), 1)
    var first = 0.0
    var last = 0.0
    var i = 0
    while i < 10 {
        var pred = py_call0(reg, xin)
        var loss = py_call0(mse, pred, target)
        py_call(sgd, "zero_grad")
        py_call(loss, "backward")
        py_call(sgd, "step")
        var lv = py_float(loss)
        if i == 0 { first = lv }
        if i == 9 { last = lv }
        i = i + 1
    }
    if first > last && last < 0.001 {
        tap_ok(3, "test_training_loop")
    } else {
        tap_not_ok(3, "test_training_loop", "first=" + first.to(str) + " last=" + last.to(str))
    }

    // ---- 4. reentrancy depth probe (D5): Auto→py→callback→py→callback→py
    //         chain at least 3 nested shim levels deep, final scalar pinned.
    var Chain = py_subclass("Chain", object_t, {
        __init__: "def __init__(self):\n        self.k = 7",
        outer: "def outer(self, x):\n        return self.mid(x + 1)",
        leaf: "def leaf(self, v):\n        return v * 2",
        mid: (self, v) => py_call(self, "leaf", py_getattr(self, "k") + v)
    })
    var chain = py_call0(Chain)
    var r4 = py_call(chain, "outer", 1)
    // outer(1) → mid(2) → py_getattr k [shim #2] → py_call leaf [shim #3] → 9*2
    if r4 == 18 {
        tap_ok(4, "test_reentry_depth3")
    } else {
        tap_not_ok(4, "test_reentry_depth3", "got " + r4.to(str))
    }
}

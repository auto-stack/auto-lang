// py_torch_train Auto test — TAP output format (Plan 539 W2 training-loop
// parity). Three-way: AutoVM vs a2py vs native Python.
//
// The parity runner sets the working directory to the library root, so
// `use.py torch...` resolves the installed torch package through the
// embedded CPython (PyO3) bridge.
//
// Test names MUST match tests/python/test_train.py (comparator joins by
// name).
//
// Conventions (Plan 539 W2):
// - kwargs constructors via item-import direct calls (`Linear(2, 3, bias:
//   false)`, `SGD(..., lr: 0.1)`).
// - Everything seeded (`torch.manual_seed(0)`) — deterministic CPU asserts.
// - 0-dim scalars assert via `.to(int)` on the marshalled f64 (floats
//   print without ".0" — use int asserts or exact fractional strings).
// - tuple returns flatten to Auto Lists (index via `x[0]`, count via
//   for-in — `.len()` on py values is unreliable, DIV-PY / P539-D2).
use.py torch: manual_seed, arange, randn, tensor, zeros
use.py torch.nn: Linear
use.py torch.optim: SGD
use.py importlib: import_module
use.py builtins: dict

fn tap_ok(n int, name str) {
    print("ok " + n.to(str) + " - " + name)
}

fn tap_not_ok(n int, name str, diag str) {
    print("not ok " + n.to(str) + " - " + name + " # " + diag)
}

fn main() {
    var torch = import_module("torch")
    var nn = import_module("torch.nn")
    manual_seed(0)

    // 1. kwargs constructor: Linear without bias.
    var layer = Linear(2, 3, bias: false)
    var w = py_getattr(layer, "weight")
    var shape = py_getattr(w, "shape")
    var dims = 0
    for d in shape { dims = dims + 1 }
    var d0 = shape[0]
    if dims == 2 { if py_float(d0) == 3 {
        tap_ok(1, "test_linear_kwargs_shape")
    } else { tap_not_ok(1, "test_linear_kwargs_shape", "d0 " + py_float(d0).to(str)) } } else {
        tap_not_ok(1, "test_linear_kwargs_shape", "dims " + dims.to(str))
    }

    // 2. bias=None via kwargs round-trips to Auto null.
    var b = py_getattr(layer, "bias")
    if b == null { tap_ok(2, "test_linear_bias_null") } else { tap_not_ok(2, "test_linear_bias_null", "not null") }

    // 3. forward pass: model(x) direct call (py_call0) + matmul chain.
    var x = randn(4, 2)
    var y = py_call0(layer, x)
    var yshape = py_getattr(y, "shape")
    var rows = 0
    for r in yshape { rows = rows + 1 }
    if py_float(yshape[0]) == 4 { if py_float(yshape[1]) == 3 {
        tap_ok(3, "test_forward_shape")
    } else { tap_not_ok(3, "test_forward_shape", "c1 " + py_float(yshape[1]).to(str)) } } else {
        tap_not_ok(3, "test_forward_shape", "r0 " + py_float(yshape[0]).to(str))
    }

    // 4. MSELoss + backward + step: seeded deterministic convergence.
    manual_seed(0)
    var model = Linear(2, 1, bias: false)
    var xin = py_call(arange(8), "reshape", 4, 2) / 2
    var target = py_call0(py_getattr(torch, "ones"), 4, 1)
    var loss_fn = py_call0(py_getattr(nn, "MSELoss"))
    var params = py_call(py_getattr(model, "parameters"), "__call__")
    var opt = SGD(params, lr: 0.1)
    var first_loss = 0.0
    var i = 0
    while i < 60 {
        var out = py_call0(model, xin)
        var loss = py_call0(loss_fn, out, target)
        py_call(opt, "zero_grad")
        py_call(loss, "backward")
        py_call(opt, "step")
        if i == 0 { first_loss = py_float(loss) }
        i = i + 1
    }
    // final loss on the trained model (0-dim tensors marshal to f64)
    var out2 = py_call0(model, xin)
    var lf = py_float(py_call0(loss_fn, out2, target))
    if lf < first_loss { if lf < 1 {
        tap_ok(4, "test_train_converges")
    } else { tap_not_ok(4, "test_train_converges", "lf " + lf.to(str)) } } else {
        tap_not_ok(4, "test_train_converges", "l0 " + first_loss.to(str) + " lf " + lf.to(str))
    }

    // 5. .item() scalar channel: float straight through (f64 direct).
    var s = py_call0(py_getattr(torch, "tensor"), [2.0, 3.0])
    var total = py_float(py_call(s, "sum"))
    if total == 5 { tap_ok(5, "test_item_scalar") } else { tap_not_ok(5, "test_item_scalar", "got " + total.to(str)) }

    // 6. exception capture: py_call_may on a bad method -> fallback.
    var bad = py_call_may(model, "no_such_method")
    var msg = bad.?("E")
    if msg.to(str) == "E" { tap_ok(6, "test_may_exception") } else { tap_not_ok(6, "test_may_exception", msg.to(str)) }

    // 7. Auto list argument into torch: tensor([...]) round trip.
    var t7 = py_call0(py_getattr(torch, "tensor"), [10, 20, 30])
    var s7 = py_call(t7, "sum")
    var s7f = py_float(py_call(t7, "sum"))
    if s7f == 60 { tap_ok(7, "test_list_arg_roundtrip") } else { tap_not_ok(7, "test_list_arg_roundtrip", "got " + s7f.to(str)) }

    // 8. tuple flatten: size() returns (rows, cols) -> Auto List.
    var m8 = py_call0(py_getattr(torch, "zeros"), 2, 5)
    var sz = py_call(m8, "size")
    var nd = 0
    for d8 in sz { nd = nd + 1 }
    if nd == 2 { if py_float(sz[0]) == 2 { if py_float(sz[1]) == 5 {
        tap_ok(8, "test_tuple_flatten")
    } else { tap_not_ok(8, "test_tuple_flatten", "c " + py_float(sz[1]).to(str)) } } else {
        tap_not_ok(8, "test_tuple_flatten", "r " + py_float(sz[0]).to(str))
    } } else { tap_not_ok(8, "test_tuple_flatten", "nd " + nd.to(str)) }

    // 9. dict round-trip: kwargs-built dict getitem + state_dict length.
    // (state_dict VALUES stringify under the Plan-300 dict marshal — tensor
    // values are not kept; length + scalar dict getitem are the supported
    // surface, DIV registered.)
    var d9 = dict(mode: 7, rank: 9)
    var mode = py_getitem(d9, "mode")
    var sd = py_call(model, "state_dict")
    var sdl = py_call(sd, "__len__")
    if mode == 7 { if sdl == 1 {
        tap_ok(9, "test_state_dict_getitem")
    } else { tap_not_ok(9, "test_state_dict_getitem", "len " + sdl.to(str)) } } else {
        tap_not_ok(9, "test_state_dict_getitem", "mode " + mode.to(str))
    }

    // 10. no_grad inference inside the inline with-bracket.
    var ng = py_call0(py_getattr(torch, "no_grad"))
    var gflag = 1
    py_with(ng, () => {
        var xg = py_call0(py_getattr(torch, "arange"), 3)
        var xg2 = py_call(arange(3), "float")
        py_call(xg2, "requires_grad_", true)
        var yg = xg2 * 2
        gflag = py_getattr(yg, "requires_grad").to(int)
    })
    if gflag == 0 { tap_ok(10, "test_no_grad_inference") } else { tap_not_ok(10, "test_no_grad_inference", "g " + gflag.to(str)) }
}

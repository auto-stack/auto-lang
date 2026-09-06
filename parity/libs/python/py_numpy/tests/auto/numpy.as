// py_numpy Auto test — TAP output format (Plan 461 Python sci-compute parity).
// The parity runner sets the working directory to the library root
// (parity/libs/python/py_numpy), so `use.py numpy` resolves the installed
// numpy package through the embedded CPython (PyO3) bridge.
//
// Test names MUST match tests/python/test_numpy.py because the comparator
// joins backends by name.
//
// Calling conventions exercised here (Plan 461 spike conclusions):
// - Idiom A: item import (`use.py numpy: sin, sqrt, ...`) for module functions.
// - Idiom B: `py_call(handle, "method", args...)` / `py_getattr(handle, attr)`
//   for everything on objects returned as opaque PyObjectHandles (ndarrays).
// - Numeric returns marshal through the string pool with whole-number floats
//   formatted without a trailing `.0` (py_auto_marshal_return f64 quirk), so
//   assertions use `.to(int)` or exact string equality.
// - Auto lists are NOT passed into Python (VM-side list→Python marshalling is
//   broken — see known-divergences DIV-PY-AUTOLIST-1); all arrays are created
//   Python-side (arange etc.) and stay on the Python side of the boundary.
use.py numpy: sin, sqrt, sum, dot, arange

fn tap_ok(n int, name str) {
    print("ok " + n.to(str) + " - " + name)
}

fn tap_not_ok(n int, name str, diag str) {
    print("not ok " + n.to(str) + " - " + name + " # " + diag)
}

fn main() {
    // Scalar ufuncs — same shape as the py_math suite.
    if sin(0.0).to(int) == 0 { tap_ok(1, "test_sin_zero") } else { tap_not_ok(1, "test_sin_zero", "got " + sin(0.0).to(str)) }
    if sqrt(16.0).to(int) == 4 { tap_ok(2, "test_sqrt") } else { tap_not_ok(2, "test_sqrt", "got " + sqrt(16.0).to(str)) }

    // Reductions over a Python-side array (handle returned by arange).
    if sum(arange(5)).to(int) == 10 { tap_ok(3, "test_arange_sum") } else { tap_not_ok(3, "test_arange_sum", "got " + sum(arange(5)).to(str)) }
    if py_call(arange(5), "mean").to(int) == 2 { tap_ok(4, "test_arange_mean") } else { tap_not_ok(4, "test_arange_mean", "got " + py_call(arange(5), "mean").to(str)) }
    if py_call(arange(9), "max").to(int) == 8 { tap_ok(5, "test_arange_max") } else { tap_not_ok(5, "test_arange_max", "got " + py_call(arange(9), "max").to(str)) }

    // Two-handle function: dot(arange(3), arange(3)) = 0+1+4.
    if dot(arange(3), arange(3)).to(int) == 5 { tap_ok(6, "test_dot") } else { tap_not_ok(6, "test_dot", "got " + dot(arange(3), arange(3)).to(str)) }

    // Shape access on a 2-D handle: tuples marshal as handles, so elements are
    // read via __getitem__.
    var m = py_call(arange(12), "reshape", 3, 4)
    var shape = py_getattr(m, "shape")
    if py_call(shape, "__getitem__", 0).to(int) == 3 { tap_ok(7, "test_reshape_rows") } else { tap_not_ok(7, "test_reshape_rows", "got " + py_call(shape, "__getitem__", 0).to(str)) }
    if py_call(shape, "__getitem__", 1).to(int) == 4 { tap_ok(8, "test_reshape_cols") } else { tap_not_ok(8, "test_reshape_cols", "got " + py_call(shape, "__getitem__", 1).to(str)) }

    // Deterministic string forms of ndarray and dtype handles.
    if py_call(arange(3), "__str__").to(str) == "[0 1 2]" { tap_ok(9, "test_array_str") } else { tap_not_ok(9, "test_array_str", "got " + py_call(arange(3), "__str__").to(str)) }
    // dtype is a C-level object whose __str__ slot does not resolve to a bound
    // method through getattr — read the `name` attribute instead.
    if py_getattr(py_getattr(arange(3), "dtype"), "name").to(str) == "int64" { tap_ok(10, "test_dtype_str") } else { tap_not_ok(10, "test_dtype_str", "got " + py_getattr(py_getattr(arange(3), "dtype"), "name").to(str)) }
}

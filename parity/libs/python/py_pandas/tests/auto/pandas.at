// py_pandas Auto test — TAP output format (Plan 461 Python sci-compute parity).
// The parity runner sets the working directory to the library root
// (parity/libs/python/py_pandas), so `use.py pandas` resolves the installed
// pandas package through the embedded CPython (PyO3) bridge.
//
// Test names MUST match tests/python/test_pandas.py because the comparator
// joins backends by name.
//
// Conventions (Plan 461 spike): the DataFrame is built Python-side from a
// reshaped arange handle — Auto lists/dicts are NOT passed into Python
// (DIV-PY-AUTOLIST-1). All results collapse to scalars or deterministic strings
// before assertion. Tuples (df.shape) marshal as handles, read via
// __getitem__; `.item()` is unnecessary because 0-d results marshal straight
// to scalar strings.
use.py numpy: arange
use.py pandas: DataFrame

fn tap_ok(n int, name str) {
    print("ok " + n.to(str) + " - " + name)
}

fn tap_not_ok(n int, name str, diag str) {
    print("not ok " + n.to(str) + " - " + name + " # " + diag)
}

fn main() {
    var df = DataFrame(py_call(arange(12), "reshape", 4, 3))

    var shape = py_getattr(df, "shape")
    if py_call(shape, "__getitem__", 0).to(int) == 4 { tap_ok(1, "test_df_rows") } else { tap_not_ok(1, "test_df_rows", "got " + py_call(shape, "__getitem__", 0).to(str)) }
    if py_call(shape, "__getitem__", 1).to(int) == 3 { tap_ok(2, "test_df_cols") } else { tap_not_ok(2, "test_df_cols", "got " + py_call(shape, "__getitem__", 1).to(str)) }

    if py_call(df, "__len__").to(int) == 4 { tap_ok(3, "test_df_len") } else { tap_not_ok(3, "test_df_len", "got " + py_call(df, "__len__").to(str)) }

    // Column sums are 18 / 22 / 26 for arange(12) reshaped 4x3.
    if py_call(py_call(df, "sum"), "min").to(int) == 18 { tap_ok(4, "test_colsum_min") } else { tap_not_ok(4, "test_colsum_min", "got " + py_call(py_call(df, "sum"), "min").to(str)) }
    if py_call(py_call(df, "sum"), "max").to(int) == 26 { tap_ok(5, "test_colsum_max") } else { tap_not_ok(5, "test_colsum_max", "got " + py_call(py_call(df, "sum"), "max").to(str)) }

    // df.sum() returns a Series — assert its runtime class name (deterministic).
    var s = py_call(df, "sum")
    if py_call(py_getattr(py_getattr(s, "__class__"), "__name__"), "__str__").to(str) == "Series" { tap_ok(6, "test_series_class") } else { tap_not_ok(6, "test_series_class", "got " + py_call(py_getattr(py_getattr(s, "__class__"), "__name__"), "__str__").to(str)) }

    // Row selection via iloc[0] (first row 0,1,2) — its min is 0.
    var row0 = py_call(py_getattr(df, "iloc"), "__getitem__", 0)
    if py_call(row0, "min").to(int) == 0 { tap_ok(7, "test_iloc0_min") } else { tap_not_ok(7, "test_iloc0_min", "got " + py_call(row0, "min").to(str)) }

    // .values is an attribute (ndarray) — read it via py_getattr; min is 0.
    if py_call(py_getattr(df, "values"), "min").to(int) == 0 { tap_ok(8, "test_values_min") } else { tap_not_ok(8, "test_values_min", "got " + py_call(py_getattr(df, "values"), "min").to(str)) }
}

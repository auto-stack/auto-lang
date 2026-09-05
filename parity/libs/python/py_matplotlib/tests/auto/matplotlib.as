// py_matplotlib Auto test — TAP output format (Plan 461 Python sci-compute
// parity). The parity runner sets the working directory to the library root
// (parity/libs/python/py_matplotlib), so `use.py matplotlib.pyplot` resolves
// the installed package and output files land in the lib directory.
//
// Test names MUST match tests/python/test_matplotlib.py because the
// comparator joins backends by name.
//
// Conventions (Plan 461 spike):
// - Rendering contract is FILE OUTPUT (savefig), not interactive windows —
//   the headless default backend resolves inside the embedded interpreter.
// - Line data comes from numpy handles; Auto lists are not passed into
//   Python (DIV-PY-AUTOLIST-1 — passing a list literal crashes the VM).
// - The zero-arg pyplot.figure() mis-marshals an argument (spike finding);
//   pyplot state implicitly creates figures, so it is not called.
// - makedirs uses positional exist_ok (kwargs are not supported by the FFI):
//   os.makedirs(name, mode=0o777, exist_ok=True) -> makedirs(name, 511, 1).
use.py numpy: arange
use.py matplotlib.pyplot: plot, savefig, close
use.py os.path: getsize
use.py os: makedirs

fn tap_ok(n int, name str) {
    print("ok " + n.to(str) + " - " + name)
}

fn tap_not_ok(n int, name str, diag str) {
    print("not ok " + n.to(str) + " - " + name + " # " + diag)
}

fn main() {
    makedirs("py_matplotlib_tmp", 511, 1)

    var x = arange(5)
    plot(x, x)
    savefig("py_matplotlib_tmp/plot.png")
    close("all")

    var sz = getsize("py_matplotlib_tmp/plot.png")
    if sz.to(int) > 0 { tap_ok(1, "test_plot_save_nonempty") } else { tap_not_ok(1, "test_plot_save_nonempty", "got " + sz.to(str)) }
    if sz.to(int) > 1000 { tap_ok(2, "test_png_bytes_gt1k") } else { tap_not_ok(2, "test_png_bytes_gt1k", "got " + sz.to(str)) }

    // Replot with different data into a second file — idempotent rerun check.
    var y = arange(9)
    plot(y, y)
    savefig("py_matplotlib_tmp/plot2.png")
    close("all")
    var sz2 = getsize("py_matplotlib_tmp/plot2.png")
    if sz2.to(int) > 0 { tap_ok(3, "test_replot_save_nonempty") } else { tap_not_ok(3, "test_replot_save_nonempty", "got " + sz2.to(str)) }
}

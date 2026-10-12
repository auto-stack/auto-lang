"""Python oracle for py_matplotlib parity (Plan 461).

Emits TAP output so the auto-parity runner can parse it with the same TAP
parser used for the AutoVM and a2py backends. Test names MUST match the Auto
test file (tests/auto/matplotlib.at) because the comparator joins backends by
name.

Runs the same plot/savefig sequence natively and asserts the identical file
artifacts (non-empty, >1k bytes). Rendering contract is file output, not
interactive windows.
"""
import os

import numpy as np
import matplotlib.pyplot as plt


def tap_ok(n, name):
    print("ok {} - {}".format(n, name))


def tap_not_ok(n, name, diag):
    print("not ok {} - {} # {}".format(n, name, diag))


if __name__ == "__main__":
    out = "py_matplotlib_tmp"
    os.makedirs(out, mode=0o777, exist_ok=True)

    x = np.arange(5)
    plt.plot(x, x)
    plt.savefig(os.path.join(out, "plot.png"))
    plt.close("all")

    sz = os.path.getsize(os.path.join(out, "plot.png"))
    if sz > 0:
        tap_ok(1, "test_plot_save_nonempty")
    else:
        tap_not_ok(1, "test_plot_save_nonempty", "got {}".format(sz))
    if sz > 1000:
        tap_ok(2, "test_png_bytes_gt1k")
    else:
        tap_not_ok(2, "test_png_bytes_gt1k", "got {}".format(sz))

    y = np.arange(9)
    plt.plot(y, y)
    plt.savefig(os.path.join(out, "plot2.png"))
    plt.close("all")
    sz2 = os.path.getsize(os.path.join(out, "plot2.png"))
    if sz2 > 0:
        tap_ok(3, "test_replot_save_nonempty")
    else:
        tap_not_ok(3, "test_replot_save_nonempty", "got {}".format(sz2))

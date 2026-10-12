"""Python oracle for py_pandas parity (Plan 461).

Emits TAP output so the auto-parity runner can parse it with the same TAP
parser used for the AutoVM and a2py backends. Test names MUST match the Auto
test file (tests/auto/pandas.at) because the comparator joins backends by name.
"""
import numpy as np
import pandas as pd


def tap_ok(n, name):
    print("ok {} - {}".format(n, name))


def tap_not_ok(n, name, diag):
    print("not ok {} - {} # {}".format(n, name, diag))


if __name__ == "__main__":
    df = pd.DataFrame(np.arange(12).reshape(4, 3))

    if df.shape[0] == 4:
        tap_ok(1, "test_df_rows")
    else:
        tap_not_ok(1, "test_df_rows", "got {}".format(df.shape[0]))
    if df.shape[1] == 3:
        tap_ok(2, "test_df_cols")
    else:
        tap_not_ok(2, "test_df_cols", "got {}".format(df.shape[1]))

    if len(df) == 4:
        tap_ok(3, "test_df_len")
    else:
        tap_not_ok(3, "test_df_len", "got {}".format(len(df)))

    # Column sums are 18 / 22 / 26 for arange(12) reshaped 4x3.
    if df.sum().min() == 18:
        tap_ok(4, "test_colsum_min")
    else:
        tap_not_ok(4, "test_colsum_min", "got {}".format(df.sum().min()))
    if df.sum().max() == 26:
        tap_ok(5, "test_colsum_max")
    else:
        tap_not_ok(5, "test_colsum_max", "got {}".format(df.sum().max()))

    if df.sum().__class__.__name__ == "Series":
        tap_ok(6, "test_series_class")
    else:
        tap_not_ok(6, "test_series_class", "got {}".format(df.sum().__class__.__name__))

    if df.iloc[0].min() == 0:
        tap_ok(7, "test_iloc0_min")
    else:
        tap_not_ok(7, "test_iloc0_min", "got {}".format(df.iloc[0].min()))

    if df.values.min() == 0:
        tap_ok(8, "test_values_min")
    else:
        tap_not_ok(8, "test_values_min", "got {}".format(df.values.min()))

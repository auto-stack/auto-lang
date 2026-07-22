"""datetime module parity tests.

NOTE: Most datetime operations are currently untestable via PyFFI because:
1. date(y,m,d) requires 3-arg FFI (broken — DIV-PY-MULTIARG-1)
2. Python object method calls (d.isoformat(), d.year) are not supported
3. Bare module dot-calls (datetime.date.today()) don't resolve

This oracle validates the EXPECTED behavior so that when PyFFI is fixed,
the Auto tests can be enabled. For now, these are Python-only reference outputs.
"""
from datetime import date, timedelta


def tap_ok(n, name):
    print(f"ok {n} - {name}")


def tap_not_ok(n, name, diag):
    print(f"not ok {n} - {name} # {diag}")


if __name__ == "__main__":
    # These are the cases that SHOULD work once PyFFI supports multi-arg calls.
    # For now, the Auto test only covers a subset (see datetime.at).

    d = date(2026, 1, 1)
    # isoformat
    iso = d.isoformat()
    if iso == "2026-01-01":
        tap_ok(1, "test_date_isoformat")
    else:
        tap_not_ok(1, "test_date_isoformat", f"got {iso}")

    # date + timedelta
    d2 = d + timedelta(days=30)
    if d2.isoformat() == "2026-01-31":
        tap_ok(2, "test_date_add_30_days")
    else:
        tap_not_ok(2, "test_date_add_30_days", f"got {d2.isoformat()}")

    # components
    if d.year == 2026 and d.month == 1 and d.day == 1:
        tap_ok(3, "test_date_components")
    else:
        tap_not_ok(3, "test_date_components", f"got {d.year}-{d.month}-{d.day}")

    # weekday
    if d.weekday() == 3:  # 2026-01-01 is Thursday (3)
        tap_ok(4, "test_weekday")
    else:
        tap_not_ok(4, "test_weekday", f"got {d.weekday()}")

    # toordinal
    if d.toordinal() == 739251:
        tap_ok(5, "test_toordinal")
    else:
        tap_not_ok(5, "test_toordinal", f"got {d.toordinal()}")

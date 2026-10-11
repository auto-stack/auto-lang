/// datetime module parity tests.
///
/// Plan 369 Task 10 fixed multi-arg Python FFI calls (date(y,m,d)).
/// Plan 369 Task 12 fixed Python object method calls and attribute access
/// via the py_call(obj, method, ...args) and py_getattr(obj, attr) built-ins,
/// backed by opaque PyObjectHandle storage in the VM heap.
///
/// Each test mirrors a case in tests/python/test_datetime.py and prints a
/// TAP line so the parity harness can compare Auto / Python / Rust outputs.
use.py datetime: date, timedelta

fn tap_ok(n int, name str) {
    print("ok " + n.to(str) + " - " + name)
}

fn tap_not_ok(n int, name str, diag str) {
    print("not ok " + n.to(str) + " - " + name + " # " + diag)
}

fn main() {
    var d = date(2026, 1, 1)

    // isoformat (method, no args)
    var iso = py_call(d, "isoformat")
    if iso == "2026-01-01" {
        tap_ok(1, "test_date_isoformat")
    } else {
        tap_not_ok(1, "test_date_isoformat", "got " + iso.to(str))
    }

    // date + timedelta(days=30). timedelta is constructed via kwargs-free
    // positional: timedelta(30) == timedelta(days=30).
    var td = timedelta(30)
    var d2 = py_call(d, "__add__", td)
    var iso2 = py_call(d2, "isoformat")
    if iso2 == "2026-01-31" {
        tap_ok(2, "test_date_add_30_days")
    } else {
        tap_not_ok(2, "test_date_add_30_days", "got " + iso2.to(str))
    }

    // components (attribute access)
    var y = py_getattr(d, "year")
    var m = py_getattr(d, "month")
    var day = py_getattr(d, "day")
    if y == 2026 {
        if m == 1 {
            if day == 1 {
                tap_ok(3, "test_date_components")
            } else {
                tap_not_ok(3, "test_date_components", "day got " + day.to(str))
            }
        } else {
            tap_not_ok(3, "test_date_components", "month got " + m.to(str))
        }
    } else {
        tap_not_ok(3, "test_date_components", "year got " + y.to(str))
    }

    // weekday (method, no args, returns int)
    var w = py_call(d, "weekday")
    if w == 3 {
        tap_ok(4, "test_weekday")
    } else {
        tap_not_ok(4, "test_weekday", "got " + w.to(str))
    }

    // toordinal (method, no args, returns int)
    var ord = py_call(d, "toordinal")
    if ord == 739617 {
        tap_ok(5, "test_toordinal")
    } else {
        tap_not_ok(5, "test_toordinal", "got " + ord.to(str))
    }
}

// py_json Auto test — TAP output format (Plan 369 P6 Task 25).
//
// Targets the Python `json` standard-library module via the PyFFI. Covers the
// round-trip loads -> dict access -> dumps cycle on FLAT structures (a
// top-level dict/list whose values are primitives):
//
//   json.loads(str)                    -> dict | list  (opaque handle)
//   py_call(d, "__getitem__", key)     -> value (str/int primitive)
//   py_call(d, "__contains__", key)    -> bool -> int (key membership)
//   json.dumps(handle)                 -> str          (clean scalar; round-trips)
//   top-level loads("[...]")           -> list handle (len + element access)
//
// Flat-only rationale: the suite deliberately uses FLAT JSON (no nested
// containers). When a top-level loads result (a dict/list handle) is indexed
// and the VALUE is itself a container (dict or list), the AutoVM PyFFI does
// NOT return an opaque handle for the nested container — it collapses it to the
// integer 0. So nested-JSON access is currently divergent and is excluded (see
// the README "Known limitations"). Flat structures, where every value is a
// primitive (str/int/bool), work cleanly on all three backends.
//
// JSON literal note: Auto treats '...' as a CHARACTER literal, not a string,
// so JSON text is written with double quotes and escaped inner quotes
// ("{\"name\": \"Alice\"}"). The same form is used in the Python oracle so the
// literals are byte-for-byte identical.
//
// A dict/list returned by loads is held as an opaque PyObjectHandle; Auto reads
// it via py_call(h, "method", ...args). A handle does not stringify (to(str)
// yields a raw handle marker), so every assertion goes through __getitem__,
// __contains__, __len__, or dumps (which returns a clean str).
//
// Test names MUST match tests/python/test_json.py because the comparator joins
// backends by name.
use.py json: dumps, loads

fn tap_ok(n int, name str) {
    print("ok " + n.to(str) + " - " + name)
}

fn tap_not_ok(n int, name str, diag str) {
    print("not ok " + n.to(str) + " - " + name + " # " + diag)
}

fn main() {
    var SRC = "{\"name\": \"Alice\", \"age\": 30, \"flag\": true}"

    // 1. loads a flat dict; __getitem__ returns a string value.
    var d = loads(SRC)
    var name = py_call(d, "__getitem__", "name")
    if name == "Alice" {
        tap_ok(1, "test_loads_get_str")
    } else {
        tap_not_ok(1, "test_loads_get_str", "got " + name.to(str))
    }

    // 2. __getitem__ returns an int value; a bool value marshals to int 1.
    var age = py_call(d, "__getitem__", "age")
    var flag = py_call(d, "__getitem__", "flag")
    if age == 30 {
        if flag == 1 {
            tap_ok(2, "test_loads_get_int_bool")
        } else {
            tap_not_ok(2, "test_loads_get_int_bool", "flag got " + flag.to(str))
        }
    } else {
        tap_not_ok(2, "test_loads_get_int_bool", "age got " + age.to(str))
    }

    // 3. dumps round-trips the loaded dict back to its canonical JSON text.
    var s = dumps(d)
    if s == SRC {
        tap_ok(3, "test_dumps_roundtrip")
    } else {
        tap_not_ok(3, "test_dumps_roundtrip", "got " + s.to(str))
    }

    // 4. __contains__ checks key membership (bool -> int).
    var has_name = py_call(d, "__contains__", "name")
    var has_missing = py_call(d, "__contains__", "missing")
    if has_name == 1 {
        if has_missing == 0 {
            tap_ok(4, "test_contains")
        } else {
            tap_not_ok(4, "test_contains", "missing got " + has_missing.to(str))
        }
    } else {
        tap_not_ok(4, "test_contains", "name got " + has_name.to(str))
    }

    // 5. A top-level JSON array loads to a list handle: len + element + dumps.
    var arr = loads("[1, 2, 3]")
    var an = py_call(arr, "__len__")
    var a0 = py_call(arr, "__getitem__", 0)
    var arrs = dumps(arr)
    if an == 3 {
        if a0 == 1 {
            if arrs == "[1, 2, 3]" {
                tap_ok(5, "test_loads_list_roundtrip")
            } else {
                tap_not_ok(5, "test_loads_list_roundtrip", "dumps got " + arrs.to(str))
            }
        } else {
            tap_not_ok(5, "test_loads_list_roundtrip", "[0] got " + a0.to(str))
        }
    } else {
        tap_not_ok(5, "test_loads_list_roundtrip", "len got " + an.to(str))
    }
}

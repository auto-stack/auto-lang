"""Python oracle for py_json parity (Plan 369 P6 Task 25).

Emits TAP output (`ok N - <name>` / `not ok N - <name> # <diag>`) so the
auto-parity runner can parse it with the same TAP parser used for the AutoVM
and a2py backends. Test names MUST match the Auto test file
(tests/auto/json.at) because the comparator joins backends by name.

Scope
-----
Plan 369 P6 Task 25 targets the Python `json` standard-library module. The
suite covers the round-trip `loads` -> dict access -> `dumps` cycle on FLAT
structures (a top-level dict/list whose values are primitives):

- `json.loads(str)`           -> dict | list   (opaque handle on the Auto side)
- `py_call(d, "__getitem__", key)` -> dict value (str/int primitive)
- `py_call(d, "__contains__", key)` -> bool -> int (key membership)
- `json.dumps(handle)`        -> str          (clean scalar; round-trips)
- top-level `json.loads("[...]")` -> list handle (len + element access)

This oracle IS the source of truth, so by construction it emits `ok` for every
case. The Auto test (json.at) performs the same operations and checks them
against the same expected values; if AutoVM's PyFFI reproduces the result it
emits `ok` too (consistent), otherwise `not ok` (an AutoVM bug).

Flat-only rationale
-------------------
The suite deliberately uses FLAT JSON (no nested containers). When a top-level
`loads` result (a dict/list handle) is indexed and the VALUE is itself a
container (dict or list), the AutoVM PyFFI does NOT return an opaque handle for
the nested container — it collapses it to the integer 0. So nested-JSON access
is currently divergent and is excluded (see the README "Known limitations").
Flat structures, where every value is a primitive (str/int/bool), work cleanly
on all three backends.

JSON literal note
-----------------
Auto treats `'...'` as a character literal, not a string, so JSON text is
written with double quotes and escaped inner quotes (`"{\"name\": \"Alice\"}"`)
in BOTH this oracle and the Auto test, keeping the literal byte-for-byte
identical across backends.
"""
import json


def tap_ok(n, name):
    print("ok {} - {}".format(n, name))


def tap_not_ok(n, name, diag):
    print("not ok {} - {} # {}".format(n, name, diag))


if __name__ == "__main__":
    SRC = '{"name": "Alice", "age": 30, "flag": true}'

    # 1. loads a flat dict; __getitem__ returns a string value.
    d = json.loads(SRC)
    name = d.__getitem__("name")
    if name == "Alice":
        tap_ok(1, "test_loads_get_str")
    else:
        tap_not_ok(1, "test_loads_get_str", "got {}".format(name))

    # 2. __getitem__ returns an int value; a bool value marshals to int 1.
    age = d.__getitem__("age")
    flag = d.__getitem__("flag")
    if age == 30 and flag == 1:
        tap_ok(2, "test_loads_get_int_bool")
    else:
        tap_not_ok(2, "test_loads_get_int_bool", "got {} {}".format(age, flag))

    # 3. dumps round-trips the loaded dict back to its canonical JSON text.
    s = json.dumps(d)
    if s == SRC:
        tap_ok(3, "test_dumps_roundtrip")
    else:
        tap_not_ok(3, "test_dumps_roundtrip", "got {}".format(s))

    # 4. __contains__ checks key membership (bool -> int).
    has_name = d.__contains__("name")
    has_missing = d.__contains__("missing")
    if has_name == 1 and has_missing == 0:
        tap_ok(4, "test_contains")
    else:
        tap_not_ok(4, "test_contains", "got {} {}".format(has_name, has_missing))

    # 5. A top-level JSON array loads to a list handle: len + element + dumps.
    arr = json.loads("[1, 2, 3]")
    an = arr.__len__()
    a0 = arr.__getitem__(0)
    arrs = json.dumps(arr)
    if an == 3 and a0 == 1 and arrs == "[1, 2, 3]":
        tap_ok(5, "test_loads_list_roundtrip")
    else:
        tap_not_ok(5, "test_loads_list_roundtrip", "got {} {} {}".format(an, a0, arrs))

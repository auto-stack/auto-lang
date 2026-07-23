"""Python oracle for py_string parity (Plan 369 P4 Task 18).

Emits TAP output (`ok N - <name>` / `not ok N - <name> # <diag>`) so the
auto-parity runner can parse it with the same TAP parser used for the AutoVM
and a2py backends. Test names MUST match the Auto test file
(tests/auto/string.at) because the comparator joins backends by name.

Design note
-----------
Auto strings live on the stack as string-tagged values, not as PyObjectHandles.
But the `py_call(obj, "method", ...args)` built-in accepts a stack string as
its first argument: PyFFI marshals it to a Python `str` (via `pop_auto_py_arg`)
and calls the method on it. The result is then marshalled back:

- A method returning a `str`  -> Auto string (compares equal to literals).
- A method returning a `bool` -> Auto int (Python True/False -> 1/0).
- A method returning an `int` -> Auto int.
- A method returning a `list` -> opaque PyObjectHandle (cannot be stringified;
  probe via `__len__` / `__getitem__`, as exercised in py_list).

So every assertion here uses the same method names the Auto test calls via
`py_call`. The a2py transpiler lowers `py_call(s, "m", ...)` to `s.m(...)`, so
the transpiled Python is valid and matches this oracle exactly.

This Python oracle IS the source of truth, so by construction it emits `ok`
for every case. The Auto test (string.at) calls the same methods on the same
inputs and checks them against the same expected values; if AutoVM's PyFFI
reproduces the result it emits `ok` too (consistent), otherwise `not ok` (an
AutoVM bug).
"""
import builtins  # noqa: F401  (documented import surface; oracle uses globals)


def tap_ok(n, name):
    print("ok {} - {}".format(n, name))


def tap_not_ok(n, name, diag):
    print("not ok {} - {} # {}".format(n, name, diag))


if __name__ == "__main__":
    # 1. upper / lower (str -> str).
    if "hello".upper() == "HELLO" and "WORLD".lower() == "world":
        tap_ok(1, "test_upper_lower")
    else:
        tap_not_ok(1, "test_upper_lower",
                   "got {} {}".format("hello".upper(), "WORLD".lower()))

    # 2. title / capitalize (str -> str).
    if "hello world".title() == "Hello World" and "hello world".capitalize() == "Hello world":
        tap_ok(2, "test_title_capitalize")
    else:
        tap_not_ok(2, "test_title_capitalize",
                   "got {} {}".format("hello world".title(), "hello world".capitalize()))

    # 3. replace (multi-arg: old, new; and 3-arg with count).
    if "hello".replace("l", "L") == "heLLo" and "aaaa".replace("a", "b", 2) == "bbaa":
        tap_ok(3, "test_replace")
    else:
        tap_not_ok(3, "test_replace",
                   "got {} {}".format("hello".replace("l", "L"), "aaaa".replace("a", "b", 2)))

    # 4. strip / lstrip / rstrip.
    if "  hi  ".strip() == "hi" and "  hi  ".lstrip() == "hi  " and "  hi  ".rstrip() == "  hi":
        tap_ok(4, "test_strip")
    else:
        tap_not_ok(4, "test_strip",
                   "got |{}| |{}| |{}|".format("  hi  ".strip(), "  hi  ".lstrip(), "  hi  ".rstrip()))

    # 5. startswith / endswith (bool -> int).
    sw = "hello world".startswith("hello")
    ew = "hello world".endswith("world")
    if sw == 1 and ew == 1:
        tap_ok(5, "test_startswith_endswith")
    else:
        tap_not_ok(5, "test_startswith_endswith", "got {} {}".format(sw, ew))

    # 6. find (returns int, -1 when missing) and count (returns int).
    if "hello world".find("world") == 6 and "hello world".find("xyz") == -1 and "hello".count("l") == 2:
        tap_ok(6, "test_find_count")
    else:
        tap_not_ok(6, "test_find_count",
                   "got {} {} {}".format("hello world".find("world"),
                                         "hello world".find("xyz"),
                                         "hello".count("l")))

    # 7. split (returns a list) + join (str.join(list) -> str).
    parts = "a,b,c".split(",")
    joined = "-".join(parts)
    if parts.__len__() == 3 and parts.__getitem__(2) == "c" and joined == "a-b-c":
        tap_ok(7, "test_split_join")
    else:
        tap_not_ok(7, "test_split_join",
                   "got {} {} {}".format(parts.__len__(), parts.__getitem__(2), joined))

    # 8. __len__ (int) and __getitem__ (str -> 1-char str).
    if "hello".__len__() == 5 and "hello".__getitem__(1) == "e":
        tap_ok(8, "test_len_getitem")
    else:
        tap_not_ok(8, "test_len_getitem",
                   "got {} {}".format("hello".__len__(), "hello".__getitem__(1)))

"""Python oracle for py_list parity (Plan 369 P4 Task 17).

Emits TAP output (`ok N - <name>` / `not ok N - <name> # <diag>`) so the
auto-parity runner can parse it with the same TAP parser used for the AutoVM
and a2py backends. Test names MUST match the Auto test file
(tests/auto/list.at) because the comparator joins backends by name.

Design note
-----------
Plan 369 P4 Task 16 found that Auto list literals (e.g. `[1, 2, 3]`) do NOT
marshal to a Python `list`, so `choice([1,2,3])` failed. This suite approaches
the problem from the other direction: Python functions that RETURN lists
(`sorted`, `list`, `reversed`), and whether Auto can manipulate those returned
list objects.

A Python list returned through PyFFI is held as an opaque `PyObjectHandle` in
the AutoVM heap (the live Python object is kept, not stringified). Auto
interacts with it via the `py_call(handle, "method", ...args)` and
`py_getattr(handle, "attr")` built-ins:

- `py_call(lst, "__len__")`           -> `len(lst)`
- `py_call(lst, "__getitem__", i)`    -> `lst[i]`
- `py_call(lst, "__contains__", x)`   -> `x in lst`  (returns bool -> int)
- `py_call(lst, "__add__", other)`    -> `lst + other`
- `py_call(lst, "__mul__", n)`        -> `lst * n`
- `py_call(lst, "count", x)`          -> `lst.count(x)`
- `py_call(lst, "index", x)`          -> `lst.index(x)`
- `py_call(lst, "__reversed__")`      -> `reversed(lst)` (iterator; feed to `list()`)

The list itself cannot be printed directly (Auto's `to(str)` on a list handle
yields a raw handle marker), so every assertion goes through element access,
length, or a method that returns a primitive.

The a2py transpiler lowers `py_call(o, "m", ...)` to `o.m(...)` and
`py_getattr(o, "a")` to `o.a`, so the transpiled Python is valid and produces
the same TAP output as this oracle.

This Python oracle IS the source of truth, so by construction it emits `ok`
for every case. The Auto test (list.at) performs the same operations and
checks them against the same expected values; if AutoVM's PyFFI reproduces the
result it emits `ok` too (consistent), otherwise `not ok` (an AutoVM bug).
"""
import builtins  # noqa: F401  (documented import surface; oracle uses globals)


def tap_ok(n, name):
    print("ok {} - {}".format(n, name))


def tap_not_ok(n, name, diag):
    print("not ok {} - {} # {}".format(n, name, diag))


if __name__ == "__main__":
    # 1. sorted() returns a list; __len__ gives its size.
    lst = sorted("dcba")
    if lst.__len__() == 4:
        tap_ok(1, "test_sorted_returns_list_len")
    else:
        tap_not_ok(1, "test_sorted_returns_list_len", "got {}".format(lst.__len__()))

    # 2. __getitem__ reads sorted elements in order.
    e = (lst.__getitem__(0) + lst.__getitem__(1)
         + lst.__getitem__(2) + lst.__getitem__(3))
    if e == "abcd":
        tap_ok(2, "test_sorted_getitem")
    else:
        tap_not_ok(2, "test_sorted_getitem", "got {}".format(e))

    # 3. list() builds a list from a string.
    chars = list("hello")
    if chars.__len__() == 5 and chars.__getitem__(0) == "h":
        tap_ok(3, "test_list_from_string")
    else:
        tap_not_ok(3, "test_list_from_string",
                   "got {} {}".format(chars.__len__(), chars.__getitem__(0)))

    # 4. __contains__ (the `in` operator) returns a bool -> int.
    has = chars.__contains__("l")
    if has == 1:
        tap_ok(4, "test_list_contains")
    else:
        tap_not_ok(4, "test_list_contains", "got {}".format(has))

    # 5. __add__ concatenates two lists.
    ab = list("ab").__add__(list("cd"))
    if ab.__len__() == 4 and ab.__getitem__(0) == "a" and ab.__getitem__(3) == "d":
        tap_ok(5, "test_list_add")
    else:
        tap_not_ok(5, "test_list_add",
                   "got {} {} {}".format(ab.__len__(), ab.__getitem__(0), ab.__getitem__(3)))

    # 6. __mul__ repeats a list.
    aaa = list("ab").__mul__(3)
    if aaa.__len__() == 6 and aaa.__getitem__(0) == "a" and aaa.__getitem__(5) == "b":
        tap_ok(6, "test_list_mul")
    else:
        tap_not_ok(6, "test_list_mul",
                   "got {} {} {}".format(aaa.__len__(), aaa.__getitem__(0), aaa.__getitem__(5)))

    # 7. count() and index() methods on a list.
    n = list("mississippi").count("s")
    i = list("mississippi").index("p")
    if n == 4 and i == 8:
        tap_ok(7, "test_list_count_index")
    else:
        tap_not_ok(7, "test_list_count_index", "got {} {}".format(n, i))

    # 8. __reversed__ returns an iterator; list() materialises it back to a list.
    rev = list(list("abcd").__reversed__())
    if rev.__getitem__(0) == "d" and rev.__getitem__(3) == "a":
        tap_ok(8, "test_reversed_via_list")
    else:
        tap_not_ok(8, "test_reversed_via_list",
                   "got {} {}".format(rev.__getitem__(0), rev.__getitem__(3)))

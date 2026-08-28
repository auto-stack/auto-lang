# py_list (Python parity)

**Mode:** Python parity (Plan 369) — three-way comparison of AutoVM, a2py
(transpiled Python), and a native Python oracle.

**Upstream:** Python 3 builtins (`sorted`, `list`, `reversed`).

**Scope:** Python list objects returned across the PyFFI boundary, and the
Auto operations that manipulate them.

## Why this library exists

Plan 369 P4 Task 16 (`py_random`) found that Auto **list literals** such as
`[1, 2, 3]` do not marshal to a Python `list`, so passing one to a Python
function (e.g. `random.choice([1,2,3])`) failed. This library approaches list
parity from the opposite direction: Python functions that **return** lists, and
whether Auto can manipulate those returned list objects.

A Python list returned through PyFFI is held as an opaque `PyObjectHandle` in
the AutoVM heap (the live object is kept rather than stringified). Auto
interacts with it via the `py_call(handle, "method", ...args)` and
`py_getattr(handle, "attr")` built-ins:

| Auto (PyFFI)                              | Python equivalent      |
|-------------------------------------------|------------------------|
| `py_call(lst, "__len__")`                 | `len(lst)`             |
| `py_call(lst, "__getitem__", i)`          | `lst[i]`               |
| `py_call(lst, "__contains__", x)`         | `x in lst` (bool->int) |
| `py_call(lst, "__add__", other)`          | `lst + other`          |
| `py_call(lst, "__mul__", n)`              | `lst * n`              |
| `py_call(lst, "count", x)`                | `lst.count(x)`         |
| `py_call(lst, "index", x)`                | `lst.index(x)`         |
| `py_call(lst, "__reversed__")` + `list()` | `list(reversed(lst))`  |

## How Auto reads a list

A list handle **cannot be printed directly**: Auto's `to(str)` on a list handle
yields a raw handle marker (e.g. `"4000000"`), not the list contents. So every
assertion in the suite goes through element access (`__getitem__`), length
(`__len__`), or a method that returns a primitive (`count`, `index`,
`__contains__`). Strings of single characters returned by `__getitem__` compare
equal to Auto string literals, so element-level equality checks work.

## API

The Auto test imports these symbols from `use.py builtins`:

- `sorted(iterable) -> list` — sorted list of elements (here, characters of a
  string, which is a valid Python iterable).
- `list(iterable) -> list` — constructor. Accepts a string (char list) or a
  `PyObjectHandle` iterator (e.g. the result of `__reversed__`), demonstrating
  that handles can be passed back into Python functions.

## Test layout

- `tests/python/test_list.py` — Python oracle emitting TAP output. This oracle
  is the source of truth: by construction it emits `ok` for every case.
- `tests/auto/list.at` — Auto test using `use.py builtins`, emitting TAP. It
  performs the same operations and checks them against the same expected
  values.

The Auto file is named `list.at` (matching the parity repo's convention of
descriptive test names; the root `.gitignore` excludes `test*.at`). The Python
oracle keeps the standard `test_*.py` convention. Test names inside the files
must match because the parity comparator joins backends by test name.

The a2py transpiler lowers `py_call(o, "m", ...)` to `o.m(...)` and
`py_getattr(o, "a")` to `o.a`, so the transpiled Python is valid and produces
the same TAP output as the oracle.

### How divergences surface

The TAP comparator only inspects pass/fail, not diagnostics. So the value
check is baked into pass/fail:

- Python oracle: always `ok` (source of truth).
- a2py (transpiled Python): runs the same Python, also `ok`.
- AutoVM: `ok` if the operation reproduces the expected value, `not ok`
  otherwise.

## Test cases

| # | Name                          | Operation                                   |
|---|-------------------------------|---------------------------------------------|
| 1 | `test_sorted_returns_list_len`| `sorted("dcba")` -> `__len__` == 4          |
| 2 | `test_sorted_getitem`         | `__getitem__(0..3)` joins to `"abcd"`       |
| 3 | `test_list_from_string`       | `list("hello")` -> len 5, first `"h"`       |
| 4 | `test_list_contains`          | `__contains__("l")` -> 1                    |
| 5 | `test_list_add`               | `list("ab") + list("cd")` -> len 4, ends    |
| 6 | `test_list_mul`               | `list("ab") * 3` -> len 6, ends             |
| 7 | `test_list_count_index`       | `"mississippi"` -> count 4, index 8         |
| 8 | `test_reversed_via_list`      | `list(reversed("abcd"))` -> `"d"`, `"a"`    |

## Known limitations

- Auto list literals (`[1, 2, 3]`) still do not marshal to a Python `list`, so
  this suite never constructs a list on the Auto side — it only consumes lists
  produced by Python. Constructing a Python list from Auto is an open PyFFI
  marshalling gap (see the `py_random` `choice` limitation).
- A list handle does not stringify in Auto (`to(str)` yields a raw handle
  marker), so the suite never prints or compares whole lists — only lengths,
  elements, and method results.

## Known divergences

(none — the suite is 100% consistent across AutoVM, a2py, and the oracle)

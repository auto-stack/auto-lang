# py_string (Python parity)

**Mode:** Python parity (Plan 369) — three-way comparison of AutoVM, a2py
(transpiled Python), and a native Python oracle.

**Upstream:** Python 3 `str` methods.

**Scope:** Python string methods invoked from Auto via the `py_call` built-in.

## Why this library exists

Auto strings are the one Auto value type that marshals cleanly to Python
(plan 369 P4 Task 16 confirmed Auto strings marshal as Python `str`, which is
why `random.choice("abcde")` worked but `choice([1,2,3])` did not). This
library verifies that Python's full `str` method surface is reachable from
Auto, and that the results marshal back correctly.

## How `py_call` works on Auto strings

Auto strings live on the stack as string-tagged values, **not** as
`PyObjectHandle`s. But the `py_call(obj, "method", ...args)` built-in accepts a
stack string as its first argument: PyFFI marshals it to a Python `str` (via
`pop_auto_py_arg`) and calls the named method on it. The result is then
marshalled back:

| Python return type | Auto marshalling                          |
|-------------------|-------------------------------------------|
| `str`             | Auto string (compares equal to literals)  |
| `bool`            | Auto int (`True`/`False` -> `1`/`0`)      |
| `int`             | Auto int                                  |
| `list`            | opaque `PyObjectHandle` (see py_list)     |

So `py_call("hello", "upper")` -> `"HELLO"`, and `py_call(s, "startswith",
prefix)` -> `1`/`0`. A method that returns a list (e.g. `split`) yields a
handle that cannot be stringified directly; it is probed via `__len__` /
`__getitem__` (exercised in the `test_split_join` case, and more thoroughly in
the `py_list` library).

### Linking requirement

`py_call` is only linked in when a `use.py` import is present in the file. This
test imports a single builtin (`use.py builtins: sorted`) purely to activate
the Python FFI surface; the strings under test are plain Auto stack strings
and `sorted` is never called.

### Chaining

`py_call` results can be chained — `py_call(py_call(s, "upper"), "lower")` —
because the result of one `py_call` is itself a value that `pop_auto_py_arg`
can re-marshal.

## Test layout

- `tests/python/test_string.py` — Python oracle emitting TAP output. This
  oracle is the source of truth: by construction it emits `ok` for every case.
- `tests/auto/string.at` — Auto test using `use.py builtins`, emitting TAP. It
  calls the same methods on the same inputs and checks them against the same
  expected values.

The Auto file is named `string.at` (matching the parity repo's convention of
descriptive test names; the root `.gitignore` excludes `test*.at`). The Python
oracle keeps the standard `test_*.py` convention. Test names inside the files
must match because the parity comparator joins backends by test name.

The a2py transpiler lowers `py_call(s, "m", ...)` to `s.m(...)`, so the
transpiled Python is valid and produces the same TAP output as the oracle.

## Test cases

| # | Name                       | Methods exercised                          |
|---|----------------------------|--------------------------------------------|
| 1 | `test_upper_lower`         | `upper`, `lower`                           |
| 2 | `test_title_capitalize`    | `title`, `capitalize`                      |
| 3 | `test_replace`             | `replace` (2-arg and 3-arg with count)     |
| 4 | `test_strip`               | `strip`, `lstrip`, `rstrip`                |
| 5 | `test_startswith_endswith` | `startswith`, `endswith` (bool -> int)     |
| 6 | `test_find_count`          | `find` (incl. -1 missing), `count`         |
| 7 | `test_split_join`          | `split` (-> list handle), `join`           |
| 8 | `test_len_getitem`         | `__len__`, `__getitem__`                    |

## Known limitations

- `py_call` is unavailable without a `use.py` import in the file (the linker
  does not resolve the symbol). The test imports an otherwise-unused builtin
  (`sorted`) solely to activate the FFI surface.
- String methods returning containers (`split`, `partition`) yield opaque list
  handles that cannot be printed directly in Auto; they must be probed via
  dunders (handled here for `split`, and covered exhaustively in `py_list`).

## Known divergences

(none — the suite is 100% consistent across AutoVM, a2py, and the oracle)

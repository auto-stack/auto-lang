# py_os (Python parity)

**Mode:** Python parity (Plan 369) — three-way comparison of AutoVM, a2py
(transpiled Python), and a native Python oracle.

**Upstream:** Python 3 `os` and `os.path` standard-library modules.

**Scope:** pure, deterministic path/string functions from `os.path` plus the
read-only `os.listdir` / `os.path.exists`. Side-effecting operations
(`makedirs`, `rename`, `remove`, `getcwd`) are deliberately excluded (see
Known limitations) so the three backends agree without coordinating filesystem
state.

## Why this library exists

`os.path` exercises two interesting PyFFI surfaces that earlier Python-parity
libs did not cover together:

1. **Multi-arg functions returning a `str`** (`join`, `basename`, `dirname`,
   `normpath`) — the common case; clean scalar returns.
2. **A function returning a Python `tuple`** (`splitext`) — held as an opaque
   `PyObjectHandle` and read via `py_call(sp, "__getitem__", i)`.
3. **A module-level constant** (`os.sep`) — imported via `use.py os: sep`,
   marshalling to its string form.
4. **`os.listdir` returning a `list`** handle, whose length is read via
   `py_call(lst, "__len__")`.

## API

The Auto test imports these symbols:

From `use.py os`:
- `listdir(path) -> list[str]` — directory entry names (handle).
- `sep` — the platform path separator constant (str).

From `use.py os.path`:
- `basename(path) -> str`
- `dirname(path) -> str`
- `splitext(path) -> tuple(str root, str ext)` (handle)
- `join(a, b, c) -> str`
- `normpath(path) -> str`
- `exists(path) -> bool` (PyFFI marshals to int 0/1)

## How Auto reads a tuple / list

A `tuple` or `list` returned through PyFFI is held as an opaque
`PyObjectHandle` in the AutoVM heap (the live object is kept, not stringified).
Auto interacts with it via the `py_call(handle, "method", ...args)` built-in:

| Auto (PyFFI)                         | Python equivalent |
|--------------------------------------|--------------------|
| `py_call(sp, "__getitem__", i)`      | `sp[i]`            |
| `py_call(lst, "__len__")`            | `len(lst)`         |

A handle does not stringify in Auto (`to(str)` yields a raw handle marker),
so every assertion goes through element access, length, or a primitive-returning
method.

## Platform portability

`os.sep`, `os.path.join`, and `os.path.normpath` produce platform-specific
separators (`\\` on Windows, `/` on POSIX). Rather than hardcoding a literal
that would only pass on one platform, both the oracle and the Auto test assert
the `join` / `normpath` result against `os.sep` (the constant) and build the
expected string dynamically:

```
var expected = "a" + sepstr + "b" + sepstr + "c"
if join("a", "b", "c") == expected { ... }
```

Because the oracle, the a2py backend, and the AutoVM all run on the same host,
they observe the same `os.sep` and agree. `basename`, `dirname`, and `splitext`
on forward-slash input are platform-agnostic (the result is identical
everywhere), so those are asserted against plain literals.

## Test layout

- `tests/python/test_os.py` — Python oracle emitting TAP output. This oracle is
  the source of truth: by construction it emits `ok` for every case.
- `tests/auto/os.at` — Auto test using `use.py os` / `use.py os.path`, emitting
  TAP. It performs the same operations and checks them against the same
  expected values.

The Auto file is named `os.at` (matching the parity repo's convention of
descriptive test names; the root `.gitignore` excludes `test*.at`). The Python
oracle keeps the standard `test_*.py` convention. Test names inside the files
must match because the parity comparator joins backends by name.

### How divergences surface

The TAP comparator only inspects pass/fail, not diagnostics. So the value
check is baked into pass/fail:

- Python oracle: always `ok` (source of truth).
- a2py (transpiled Python): runs the same Python, also `ok`.
- AutoVM: `ok` if the operation reproduces the expected value, `not ok`
  otherwise.

## Test cases

| # | Name                    | Operation                                            |
|---|-------------------------|------------------------------------------------------|
| 1 | `test_basename`         | `basename("/tmp/test.txt")` == `"test.txt"`          |
| 2 | `test_dirname`          | `dirname("/tmp/test.txt")` == `"/tmp"`               |
| 3 | `test_splitext`         | `splitext(...)` tuple -> root `/tmp/test`, ext `.txt`|
| 4 | `test_join_uses_sep`    | `join("a","b","c")` == `"a"+sep+"b"+sep+"c"`         |
| 5 | `test_normpath`         | `normpath("a/b/../c")` == `"a"+sep+"c"`              |
| 6 | `test_exists_dot`       | `exists(".")` == 1                                   |
| 7 | `test_exists_missing`   | `exists("definitely_no_such_dir_xyz_123")` == 0      |
| 8 | `test_listdir_len`      | `listdir(".")` length >= 1                           |

## Known limitations

- **`os.getcwd()` fails**: a no-arg PyFFI call from Auto passes one argument,
  and `nt.getcwd()` rejects it with `TypeError: nt.getcwd() takes no arguments
  (1 given)`. This is a general no-arg-call PyFFI gap (also affects other
  zero-argument stdlib functions), not specific to `os`. Not exercised here.
- **Side-effecting operations excluded**: `makedirs`, `rename`, `remove`,
  `mkdir`, `rmdir` mutate the filesystem. For a parity test that must agree
  across three independently-run backends without shared state, such
  operations are a poor fit and are omitted. Pure, read-only functions are
  preferred.
- A tuple/list handle does not stringify in Auto (`to(str)` yields a raw handle
  marker), so the suite never prints whole tuples/lists — only their elements
  and length.

## Known divergences

(none — the suite is 100% consistent across AutoVM, a2py, and the oracle)

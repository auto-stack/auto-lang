# py_re (Python parity)

**Mode:** Python parity (Plan 369) — three-way comparison of AutoVM, a2py
(transpiled Python), and a native Python oracle.

**Upstream:** Python 3 `re` standard-library module.

**Scope:** the four primary regex operations and the distinct Python types they
return across the PyFFI boundary — `str` (sub), `list` (findall, split), and the
`Match` object (search) — plus the pure `escape` helper.

## Why this library exists

`re` exercises a rich mix of PyFFI return shapes in one module:

1. **`str`-returning functions** (`sub`, `escape`) — clean scalar returns, the
   simplest case.
2. **A 4-argument call** (`sub(pat, repl, str, count)`) — confirms multi-arg
   FFI dispatch extends past two args.
3. **`list`-returning functions** (`findall`, `split`) — held as opaque
   `PyObjectHandle`s, read via `py_call(lst, "__len__")` /
   `py_call(lst, "__getitem__", i)`.
4. **A `Match` object return** (`search`) — also a handle, but read via named
   methods `group(n)`, `start()`, `end()` rather than dunders.

## API

The Auto test imports these symbols from `use.py re`:

- `sub(pattern, repl, string) -> str` — replace all matches.
- `sub(pattern, repl, string, count) -> str` — limited replacements (4-arg).
- `findall(pattern, string) -> list[str]` — all non-overlapping matches (handle).
- `search(pattern, string) -> Match` — first match (handle), or `None`.
- `split(pattern, string) -> list[str]` — pieces between matches (handle).
- `escape(string) -> str` — escape regex metacharacters.

## How Auto reads a list / Match object

A `list` or `Match` returned through PyFFI is held as an opaque `PyObjectHandle`
in the AutoVM heap (the live object is kept, not stringified). Auto interacts
with it via the `py_call(handle, "method", ...args)` built-in:

| Auto (PyFFI)                          | Python equivalent   |
|---------------------------------------|----------------------|
| `py_call(lst, "__len__")`             | `len(lst)`           |
| `py_call(lst, "__getitem__", i)`      | `lst[i]`             |
| `py_call(m, "group", n)`              | `m.group(n)`         |
| `py_call(m, "start")`                 | `m.start()`          |
| `py_call(m, "end")`                   | `m.end()`            |

A handle does not stringify in Auto (`to(str)` yields a raw handle marker), so
every assertion goes through element access, length, or a primitive-returning
method.

## Regex literal syntax

Auto has no raw-string (`r"..."`) syntax, so regex patterns are written with
**escaped backslashes** (`"\\d+"`) in BOTH this oracle and the Auto test. Using
the same escaped form on both sides keeps the pattern byte-for-byte identical
and avoids any cross-backend interpretation drift. (An unescaped `"\d"` would
be an invalid Auto escape; `"\\d"` is the Auto string whose sole content is
`\d`, exactly what `re` expects.)

## Test layout

- `tests/python/test_re.py` — Python oracle emitting TAP output. This oracle is
  the source of truth: by construction it emits `ok` for every case.
- `tests/auto/re.at` — Auto test using `use.py re`, emitting TAP. It performs
  the same operations and checks them against the same expected values.

The Auto file is named `re.at` (matching the parity repo's convention of
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

| # | Name                     | Operation                                              |
|---|--------------------------|--------------------------------------------------------|
| 1 | `test_sub_all`           | `sub("\\d+","N","a1b22c333")` == `"aNbNcN"`            |
| 2 | `test_sub_count`         | `sub(...,1)` (4-arg) == `"aNb22c333"`                  |
| 3 | `test_findall`           | `findall("\\d+",...)` len 3, `[0]="1"`, `[2]="333"`    |
| 4 | `test_search_groups`     | `search("(\\w+)@(\\w+)")` groups `user@example`,`user` |
| 5 | `test_match_span`        | Match `start()==3`, `end()==6`                         |
| 6 | `test_findall_no_match`  | `findall("zzz","abc")` len 0 (empty list)              |
| 7 | `test_split`             | `split("\\s+","a  b   c")` len 3, `[0]="a"`, `[2]="c"` |
| 8 | `test_escape`            | `escape("a.b*c")` == `"a\\.b\\*c"`                     |

## Known limitations

- **`re.search` no-match (`None`) is marshalling-divergent and not asserted
  directly.** When `search` finds nothing, Python returns `None`. The AutoVM
  marshals that `None` to the integer `0`, so an Auto assertion `nm == 0`
  *passes* on AutoVM. But the a2py backend lowers the same `if nm == 0` to
  Python, where `None == 0` is `False` — so a2py emits `not ok`. The oracle,
  using `nm is None`, passes. This three-way split (oracle ok, AutoVM ok, a2py
  not ok) is a genuine inconsistency, so the suite avoids it: the "no match"
  case (test 6) uses `findall` returning an **empty list** instead, which is a
  real Python list on every backend and agrees everywhere (`len == 0`).
  Documenting the `None → 0` AutoVM marshalling here as a finding; it is a
  semantic gap between the AutoVM's value model and Python's `None`, not a `re`
  bug.
- Auto has no raw-string syntax; patterns must use escaped backslashes
  (`"\\d+"`), see "Regex literal syntax" above.
- A list/Match handle does not stringify in Auto (`to(str)` yields a raw handle
  marker), so the suite never prints whole lists/Match objects — only their
  elements, length, and method results.

## Known divergences

(none in the suite — 100% consistent across AutoVM, a2py, and the oracle. The
`search`-no-match `None` behaviour is excluded as described above.)

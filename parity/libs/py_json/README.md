# py_json (Python parity)

**Mode:** Python parity (Plan 369) — three-way comparison of AutoVM, a2py
(transpiled Python), and a native Python oracle.

**Upstream:** Python 3 `json` standard-library module.

**Scope:** the round-trip `loads` → dict access → `dumps` cycle on **flat**
JSON structures (a top-level dict or list whose values are primitives: str, int,
bool). Covers reading values, key membership, and the serialize/deserialize
round-trip.

## Why this library exists

`json` exercises the "Python object handle flows back into a Python function"
pattern that earlier libs only touched lightly:

1. **`loads(str)` → dict/list handle** — the top-level container survives the
   PyFFI boundary as an opaque `PyObjectHandle`.
2. **`py_call(handle, "method", ...)` on a *returned* object** — reading dict
   values via `__getitem__`, membership via `__contains__`.
3. **A handle passed *back* into a Python function** — `dumps(handle)` takes the
   handle from `loads` and returns a clean `str`, proving handles round-trip
   through the FFI in both directions.

## API

The Auto test imports these symbols from `use.py json`:

- `loads(json_str) -> dict | list` — parse JSON text to a container (handle).
- `dumps(obj) -> str` — serialize a container (handle) back to JSON text.

Container handles are manipulated with the `py_call(handle, "method", ...args)`
built-in (`__getitem__`, `__contains__`, `__len__`).

## How Auto reads a dict / list

A dict or list returned by `loads` is held as an opaque `PyObjectHandle` in the
AutoVM heap (the live object is kept, not stringified). Auto interacts with it
via `py_call`:

| Auto (PyFFI)                              | Python equivalent        |
|-------------------------------------------|--------------------------|
| `py_call(d, "__getitem__", key)`          | `d[key]`                 |
| `py_call(d, "__contains__", key)`         | `key in d` (bool -> int) |
| `py_call(lst, "__len__")`                 | `len(lst)`               |
| `py_call(d, "keys")` + `list()`           | `list(d.keys())`         |
| `py_call(d, "get", key, default)`         | `d.get(key, default)`    |
| `dumps(handle)`                           | `json.dumps(obj)`        |

A handle does not stringify in Auto (`to(str)` yields a raw handle marker), so
every assertion goes through `__getitem__`, `__contains__`, `__len__`, or
`dumps` (which returns a clean `str`). Python `bool` values marshal to int
(`true` → `1`), matching how the oracle compares them.

## Flat-only: nested containers are a known PyFFI gap

The suite uses **flat** JSON exclusively (no dict/list nested inside another
container). When a top-level `loads` result (a dict/list handle) is indexed and
the **value is itself a container** (a nested dict or list), the AutoVM PyFFI
does **not** return an opaque handle for the nested container — it collapses it
to the integer `0`:

```
loads('{"outer": {"inner": 42}}')   -> dict handle d           (ok)
py_call(d, "__getitem__", "outer")  -> 0  (NOT the inner dict) (broken)
```

The same collapse happens for a list value inside a dict, and for the
`re.search`-no-match `None` (see `py_re` README). The common root cause: a
container/non-string value returned from a *method call on a handle* (as opposed
to a top-level `use.py` function call) is not preserved as an opaque handle — it
is flattened to a scalar int. Flat structures, where every indexed value is a
primitive (str/int/bool), avoid this entirely and agree on all three backends.

## JSON literal syntax

Auto treats `'...'` as a **character literal**, not a string, so JSON text must
be written with double quotes and escaped inner quotes:

```
var SRC = "{\"name\": \"Alice\", \"age\": 30}"
```

The same escaped form is used in the Python oracle so the literals are
byte-for-byte identical across backends. (Auto also reserves the identifier
`as`, so no variable is named `as`.)

## Test layout

- `tests/python/test_json.py` — Python oracle emitting TAP output. This oracle
  is the source of truth: by construction it emits `ok` for every case.
- `tests/auto/json.at` — Auto test using `use.py json`, emitting TAP. It
  performs the same operations and checks them against the same expected
  values.

The Auto file is named `json.at` (matching the parity repo's convention of
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

| # | Name                        | Operation                                                |
|---|-----------------------------|----------------------------------------------------------|
| 1 | `test_loads_get_str`        | `loads(...)` → `__getitem__("name")` == `"Alice"`        |
| 2 | `test_loads_get_int_bool`   | `__getitem__("age")` == 30, `__getitem__("flag")` == 1   |
| 3 | `test_dumps_roundtrip`      | `dumps(loads(SRC))` == `SRC`                             |
| 4 | `test_contains`             | `__contains__("name")` == 1, `__contains__("missing")` == 0 |
| 5 | `test_loads_list_roundtrip` | top-level `loads("[1,2,3]")` len 3, `[0]`==1, dumps ok   |

## Known limitations

- **Nested container access collapses to `0`.** Indexing a top-level container
  handle for a value that is itself a dict/list returns the integer `0`, not a
  nested handle. This affects `{"outer": {...}}`, `{"items": [...]}`, etc. The
  suite is flat-only to stay consistent. (Same root cause as the `re.search`
  no-match `None → 0` collapse documented in `py_re`.)
- **Cannot construct a Python dict/list from Auto literals.** Auto dict/list
  literals do not marshal to Python containers (see the `py_list` / `py_random`
  limitation notes), so the suite only ever *consumes* containers produced by
  `loads` — it never builds one on the Auto side to feed to `dumps`.
- A container handle does not stringify in Auto (`to(str)` yields a raw handle
  marker), so the suite never prints whole dicts/lists — only their values
  (via `__getitem__`) and the `dumps` string.
- Auto reserves the identifier `as`; no variable is so named.
- `dumps` formatting options (`indent`, `sort_keys`, `separators`) are not
  exercised — the default canonical form is used and round-trips exactly.

## Known divergences

(none in the suite — 100% consistent across AutoVM, a2py, and the oracle.
Nested-container access is excluded as described above.)

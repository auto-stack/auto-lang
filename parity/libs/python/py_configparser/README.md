# py_configparser (Python parity)

**Mode:** Python parity (Plan 369) — three-way comparison of AutoVM, a2py
(transpiled Python), and a native Python oracle.

**Upstream:** Python 3 `configparser` standard-library module.

**Scope:** INI parsing on **flat** config (sections whose values are scalars:
str / int / bool). Covers `read_string`, value lookup (`get` / `getint` /
`getboolean`), and `sections()` counting.

## Why this library exists

`configparser` exercises the "construct a class instance, then drive it with
`py_call`" pattern, complementing the `loads` → handle pattern from `py_json`:

1. **`ConfigParser()` class construction** — like `date()` in `py_datetime`,
   the no-arg constructor returns an opaque `PyObjectHandle`.
2. **`py_call(handle, "read_string", text)`** — a void method that mutates
   the parser in place (the handle is kept live across the call).
3. **`py_call(handle, "get", section, key)`** — a 2-positional-arg method
   returning a clean `str` scalar (the common, reliable path).
4. **`py_call(handle, "sections")` → list handle → `__len__`** — a method that
   returns a container, read via the list-`__len__` pattern.

## API

The Auto test imports this symbol from `use.py configparser`:

- `ConfigParser` — the parser class (constructed with no args).

Parser instances are driven with `py_call(cfg, "method", ...args)`:

| Auto (PyFFI)                                  | Python equivalent            |
|-----------------------------------------------|------------------------------|
| `ConfigParser()`                              | `configparser.ConfigParser()`|
| `py_call(cfg, "read_string", ini)`            | `cfg.read_string(ini)`       |
| `py_call(cfg, "get", sec, key)`               | `cfg.get(sec, key)` -> str   |
| `py_call(cfg, "getint", sec, key)`            | `cfg.getint(sec, key)` -> int|
| `py_call(cfg, "getboolean", sec, key)`        | `cfg.getboolean(...)` -> int |
| `py_call(cfg, "sections")` + `__len__`        | `len(cfg.sections())`        |

## How Auto reads config values

A `ConfigParser` instance is held as an opaque `PyObjectHandle` in the AutoVM
heap (the live object is kept, not stringified). Auto interacts with it via
`py_call`. Returned scalars (`str` from `get`, `int` from `getint`/`getboolean`,
`int` from `sections().__len__()`) come back as clean Auto values and compare
to literals normally.

## Flat-only: scalar values

The suite reads only **scalar** config values (str / int / bool). It does not
exercise multi-line values, interpolation, or DEFAULT sections, which are
orthogonal to the PyFFI round-trip being validated.

## INI literal syntax

Auto treats `'...'` as a **character literal**, not a string, so INI text is
written with double quotes and escaped newlines:

```
var INI = "[db]\nhost = localhost\nport = 5432\n[app]\nname = myapp\ndebug = true\n"
```

The same escaped form is used in the Python oracle so the literal is
byte-for-byte identical across backends.

## Test layout

- `tests/python/test_configparser.py` — Python oracle emitting TAP output.
  This oracle is the source of truth: by construction it emits `ok` for every
  case.
- `tests/auto/configparser.at` — Auto test using `use.py configparser`,
  emitting TAP. It performs the same operations and checks them against the
  same expected values.

The Auto file is named `configparser.at` (matching the parity repo's
convention of descriptive test names; the root `.gitignore` excludes
`test*.at`). The Python oracle keeps the standard `test_*.py` convention.
Test names inside the files must match because the parity comparator joins
backends by name.

## Test cases

| # | Name                          | Operation                                                      |
|---|-------------------------------|----------------------------------------------------------------|
| 1 | `test_get_str`                | `get("db","host")` == `"localhost"`                           |
| 2 | `test_getint`                 | `getint("db","port")` == 5432                                 |
| 3 | `test_getboolean_true`        | `getboolean("app","debug")` == 1 (True marshals to int 1)     |
| 4 | `test_sections_len`           | `sections().__len__()` == 2                                   |
| 5 | `test_get_second_section`     | `get("app","name")` == `"myapp"`                              |

## Known limitations

- **Method-call `False` does not marshal reliably.** A Python `False` returned
  from a method call on a handle (`has_option` / `has_section` on a missing
  key) does not become a clean `int 0` in Auto — `.to(str)` yields an
  empty/unstable marker and `== 0` is unreliable. By contrast the `True` case
  (`getboolean("debug")` on a true value, or `has_section` on an existing
  section) marshals cleanly to `1`. The suite therefore only asserts the True
  branch of booleans and never the False branch; section existence is checked
  via `sections().__len__()` rather than `has_section`. (This is the same
  class of asymmetry as the `py_re` / `py_json` handle-return quirks.)
- **4-arg `get` hits the extra-arg path.** `get(section, key, fallback)` (and
  any 4-positional-arg method) raises `TypeError: takes 3 positional arguments
  but 4 were given` under the AutoVM PyFFI. The suite uses plain 2-key
  `get` / `getint` / `getboolean` only.
- **No-arg constructor is fine here** (`ConfigParser()` works), unlike the
  `hashlib` constructors which suffer the no-arg extra-arg bug (see
  `py_hashlib`).
- Auto treats `'...'` as a character literal and reserves the identifier `as`;
  the suite uses `"..."` strings and no `as` variable.

## Known divergences

(none in the suite — 100% consistent across AutoVM, a2py, and the oracle.
Method-call `False` and 4-arg `get` are excluded as described above.)

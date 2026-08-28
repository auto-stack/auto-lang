# py_sys (Python parity)

**Mode:** Python parity (Plan 369) — three-way comparison of AutoVM, a2py
(transpiled Python), and a native Python oracle.

**Upstream:** Python 3 `sys` standard-library module.

**Scope:** read-only module constants (`platform`, `version`, `byteorder`,
`maxunicode`) and the `version_info` struct. **`maxsize` and `executable` are
NOT asserted** — they diverge across backends for documented reasons (see
"Known divergences").

## Why this library exists

`sys` is the canonical "module-level constants" library. Earlier py_ libs
imported *functions* and *classes*; `sys` imports plain *constants* and a
*namedtuple object*, exercising:

1. **Module constants via `use.py`** — `platform` / `version` / `byteorder`
   arrive as clean Auto strings; `maxunicode` arrives as a clean Auto int.
   These compare with `==` and relational operators normally.
2. **A namedtuple as an opaque handle** — `version_info` is a Python object,
   read via `py_getattr(vi, "major")` → clean int (the `py_getattr` read path,
   same as `py_hashlib`'s attribute reads).

## API

The Auto test imports these symbols from `use.py sys`:

- `platform` → str (OS identifier, e.g. `"win32"`, `"linux"`).
- `version` → str (the full version line).
- `byteorder` → str (`"little"` / `"big"`).
- `maxunicode` → int (`0x10FFFF` == `1114111` on Python 3).
- `version_info` → namedtuple object; fields read via `py_getattr`:

| Auto (PyFFI)                       | Python equivalent            |
|------------------------------------|------------------------------|
| `platform`                         | `sys.platform` -> str        |
| `version`                          | `sys.version` -> str         |
| `byteorder`                        | `sys.byteorder` -> str       |
| `maxunicode`                       | `sys.maxunicode` -> int      |
| `py_getattr(version_info, "major")`| `sys.version_info.major` -> int |

## Test layout

- `tests/python/test_sys.py` — Python oracle emitting TAP output. This oracle
  is the source of truth: by construction it emits `ok` for every case.
- `tests/auto/sys.at` — Auto test using `use.py sys`, emitting TAP. It
  performs the same operations and checks them against the same expected
  values.

The Auto file is named `sys.at` (matching the parity repo's convention of
descriptive test names; the root `.gitignore` excludes `test*.at`). The Python
oracle keeps the standard `test_*.py` convention. Test names inside the files
must match because the parity comparator joins backends by name.

## Test cases

| # | Name                      | Operation                                              |
|---|---------------------------|--------------------------------------------------------|
| 1 | `test_platform`           | `platform == "win32"`                                  |
| 2 | `test_version_major`      | `py_getattr(version_info, "major") == 3`               |
| 3 | `test_byteorder`          | `byteorder == "little"`                                |
| 4 | `test_maxunicode`         | `maxunicode == 1114111`                                |
| 5 | `test_version_roundtrip`  | `version.to(str) == version` (str constant round-trips)|

## Known limitations

- **`maxsize` overflows to double.** `sys.maxsize` is `9223372036854775807`,
  which exceeds Auto's int range and arrives as an f64. `.to(str)` renders the
  rounded `"9223372036854776000"`, and relational compares against int
  literals (`maxsize > 5`, `maxsize > 0`) silently return **false** (f64 vs
  int comparison is broken in the VM). So no value assertion on `maxsize` can
  agree across backends; it is imported only to activate the PyFFI and is
  excluded from the suite. (The smaller `maxunicode` == 1114111 fits in a
  clean int and is reliable.)
- **`executable` differs by host process.** The AutoVM embeds Python, so
  `sys.executable` is the `auto.exe` path; the a2py/oracle backends run as a
  Python script, so `sys.executable` is the `python.exe` path. The values
  differ by construction. Auto string methods (`.length()`, `.find()`,
  `.contains()`) are unavailable, so substring/shape checks aren't feasible
  either. `executable` is therefore excluded.
- **No Auto string methods.** Auto strings support `==` and `+` but not
  `.length()` / `.find()` / `.contains()`, so the suite uses only equality
  and round-trip checks.
- Auto treats `'...'` as a character literal and reserves the identifier `as`;
  the suite uses `"..."` strings and no `as` variable.

## Known divergences

- **`maxsize`** — diverges by value: Auto renders `9223372036854776000`
  (f64-rounded), the oracle renders `9223372036854775807` (exact). Excluded
  from assertions (imported only to activate the PyFFI).
- **`executable`** — diverges structurally: Auto sees `auto.exe`, the
  a2py/oracle see `python.exe`. Excluded from assertions.

The five asserted cases are 100% consistent across AutoVM, a2py, and the
oracle.

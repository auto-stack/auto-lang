# py_hashlib (Python parity)

**Mode:** Python parity (Plan 369) — three-way comparison of AutoVM, a2py
(transpiled Python), and a native Python oracle.

**Upstream:** Python 3 `hashlib` standard-library module.

**Scope:** hash-object construction via `new(algo_name)` and its read-only
metadata attributes (`name`, `digest_size`, `block_size`) across md5, sha256,
and sha512. **Data hashing (computing digests) is NOT covered** — it is
blocked by the PyFFI bytes limitation documented below.

## Why this library exists

`hashlib` was the first library whose *intended* use (computing a hash of
data) is entirely blocked by the PyFFI, so it instead validates the
`py_getattr(handle, "attr")` read path on a freshly constructed object:

1. **`new("sha256")` construction** — a single-string-arg constructor that
   returns an opaque `PyObjectHandle`. (The bare constructors `sha256()` /
   `md5()` are blocked by the no-arg extra-arg bug; `new(name)` avoids it
   because it has one real positional arg.)
2. **`py_getattr(h, "name")` → str** — reading a string attribute.
3. **`py_getattr(h, "digest_size")` → int** — reading an int attribute that
   compares cleanly to a literal.

## API

The Auto test imports this symbol from `use.py hashlib`:

- `new(algo_name) -> hash object` — construct a hash object for a named
  algorithm (`"md5"`, `"sha256"`, `"sha512"`, ...).

Hash objects are read with `py_getattr(handle, "attr")`:

| Auto (PyFFI)                          | Python equivalent        |
|---------------------------------------|--------------------------|
| `new("sha256")`                       | `hashlib.new("sha256")`  |
| `py_getattr(h, "name")`               | `h.name` -> str          |
| `py_getattr(h, "digest_size")`        | `h.digest_size` -> int   |
| `py_getattr(h, "block_size")`         | `h.block_size` -> int    |

## The bytes limitation (why no digests)

hashlib's data-taking constructors and `update()` require Python `bytes`. The
AutoVM PyFFI marshals an Auto `str` to a Python **`str`**, not `bytes`, so
every data path fails:

```
sha256("hello")   -> TypeError: Strings must be encoded before hashing
md5("hello")      -> same
h.update("hello") -> TypeError: object supporting the buffer API required
```

The bare no-arg constructors (`sha256()`, `md5()`, ...) additionally hit the
PyFFI extra-arg bug: a `pending_native_arg_count` of 0 falls back to the
`inspect.signature`-derived param count, which (because `data=b''` has a
default) still ends up passing a stray non-bytes value, producing the same
"object supporting the buffer API required" error. So an empty hash object
cannot be constructed via `sha256()` either.

The **only** reliable construction path is `new(algo_name)` with a single
string positional arg, which yields a fresh empty hash object whose metadata
attributes (`name`, `digest_size`, `block_size`) are then readable. This suite
therefore validates hash-object **metadata** (algorithm identity + sizes), not
digests. There is no `hexdigest()` / `update()` coverage because those require
a `bytes` payload that Auto cannot currently produce. To lift this limitation,
the PyFFI would need either an Auto `bytes` type that marshals to Python
`bytes`, or a `py_call` overload that encodes str→bytes before the call.

## Test layout

- `tests/python/test_hashlib.py` — Python oracle emitting TAP output. This
  oracle is the source of truth: by construction it emits `ok` for every case.
- `tests/auto/hashlib.at` — Auto test using `use.py hashlib`, emitting TAP. It
  performs the same operations and checks them against the same expected
  values.

The Auto file is named `hashlib.at` (matching the parity repo's convention of
descriptive test names; the root `.gitignore` excludes `test*.at`). The Python
oracle keeps the standard `test_*.py` convention. Test names inside the files
must match because the parity comparator joins backends by name.

## Test cases

| # | Name                    | Operation                                                |
|---|-------------------------|----------------------------------------------------------|
| 1 | `test_md5_name`         | `new("md5").name` == `"md5"`                             |
| 2 | `test_md5_digest_size`  | `new("md5").digest_size` == 16                           |
| 3 | `test_sha256_name`      | `new("sha256").name` == `"sha256"`                       |
| 4 | `test_sha256_sizes`     | `new("sha256")`: `digest_size` == 32, `block_size` == 64 |
| 5 | `test_sha512_sizes`     | `new("sha512")`: `digest_size` == 64, `block_size` == 128|

## Known limitations

- **No data hashing.** `sha256(str)` / `md5(str)` / `h.update(str)` all fail
  because the PyFFI marshals Auto `str` to Python `str`, not `bytes`. Digests
  cannot be computed through the FFI today.
- **No bare no-arg constructors.** `sha256()` / `md5()` hit the extra-arg bug
  (a stray non-bytes arg is passed). Use `new(name)` instead.
- **`new` is the only reliable entry point.** It takes a single string
  positional arg (the algorithm name) and returns a fresh empty hash object.
- Metadata-only coverage: `name` / `digest_size` / `block_size` are exercised;
  `hexdigest()` / `digest()` / `update()` are not (bytes payload required).
- Auto treats `'...'` as a character literal and reserves the identifier `as`;
  the suite uses `"..."` strings and no `as` variable.

## Known divergences

(none in the suite — 100% consistent across AutoVM, a2py, and the oracle.
Data hashing is excluded as described above; the suite validates only the
metadata that the FFI can round-trip.)

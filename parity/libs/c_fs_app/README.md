# c_fs_app (Plan 368 F1 — consumer parity)

File read/write consumer application. Verifies that Auto consuming the `fs.*`
stdlib (backed by std::fs) produces the same behavior as a native Rust app
using std::fs directly — three-way (AutoVM / a2r / native Rust).

This is **consumer mode**: Auto calls library capabilities (`fs.read_text` /
`fs.write_text` / `fs.exists` / `fs.create_dir`) to perform file operations,
rather than re-implementing library internals. The Rust oracle calls std::fs
directly.

## API

| Function | Signature | Calls (Auto) | Calls (Rust oracle) |
|----------|-----------|--------------|---------------------|
| `write_and_read` | `(path, content) -> str` | `fs.write_text` + `fs.read_text` | `fs::write` + `fs::read_to_string` |
| `check_exists` | `(path) -> int` | `fs.exists` | `fs::metadata` |
| `mkdir_write_read` | `(dir, filename, content) -> str` | `fs.create_dir` + write + read | `fs::create_dir_all` + write + read |

## Determinism

Each backend uses a fixed relative subdir (`c_fs_app_tmp`) under its own
working directory (Auto: lib root; Rust: `tests/rust/`). They are
self-contained (write→read round-trip); only the TAP output is compared, not
shared files. The Rust oracle goes further: each `#[test]` writes into its
own unique sub-subdir so the tests are race-free when cargo runs them on
parallel threads (Auto/a2r run all assertions in a single sequential `main()`).

7 test cases, names mirror the Rust oracle exactly:
`test_write_read_basic`, `test_write_read_empty`, `test_write_overwrite`,
`test_exists_yes`, `test_exists_no`, `test_mkdir_write_read`,
`test_nested_exists`.

## a2r notes (Plan 368 — transpiler gaps discovered then fixed)

Two pre-existing a2r transpiler gaps were hit while writing this lib and
have since been **fixed** (Plan 368 FU-2), so the source now uses natural
string patterns:

1. **Owned `str` var registered as `StrSlice`** — `var x str = <concat>` was
   tracked as `StrSlice` but transpiled to owned `String`, so re-using it
   (e.g. `fs.write_text(fullpath)` then `fs.read_text(fullpath)`) moved it
   (E0382). Fixed in `needs_as_str`: only function *params* declared `str`
   are truly `&str`; locals render to owned `String` and need `.as_str()`
   at `&str` use sites (which also borrows instead of moving).

2. **Inline-concat args to user functions** — `f(base + "/x")` transpiled to
   `f(format!(...))` without `.as_str()` (E0308). Fixed in
   `needs_borrow_unknown_callee`: inline-concat args are now borrowed via
   `expr_contains_string`.

A small a2r-std alignment fix was also applied earlier: `a2r_std::fs::read_text`
/`read_to_string` return `String` (empty on error) to match the VM's
`auto.fs.read_text` native (`shim_file_read_text` → `unwrap_or_default()`),
so the same `.at` source behaves identically across VM/a2r/Rust.

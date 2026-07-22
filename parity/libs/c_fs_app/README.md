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

## a2r notes (Plan 368 F1 — discovered transpiler gaps)

The consumer source works around two pre-existing a2r transpiler limitations
(recorded for a future a2r improvement, out of this plan's scope):

1. **Owned `str` var registered as `StrSlice`.** `var x str = <concat>` is
   tracked in `local_var_types` as `StrSlice` but transpiled to Rust `String`.
   Re-using it (e.g. passing to `fs.write_text` then `fs.read_text`) triggers
   `E0382 use of moved value`. Workaround in `c_fs_app.at`: pass the path as
   an **inline** concatenation at each call site (inline concats go through
   `expr_as_str` and are correctly `.as_str()`-borrowed).

2. **Inline-concat args to user functions.** `f(base + "/x")` transpiles to
   `f(format!(...), ...)` without `.as_str()`, failing `E0308` when `f`
   takes `&str`. Workaround in `basic.at`: bind the concat to a local `str`
   variable first (variable-form string args are auto-`.as_str()`'d).

Both quirks are documented inline in the `.at` sources. A small a2r-std
alignment fix was also needed: `a2r_std::fs::read_text`/`read_to_string` now
return `String` (empty on error) to match the VM's `auto.fs.read_text`
native (`shim_file_read_text` → `unwrap_or_default()`), so the same `.at`
source behaves identically across VM/a2r/Rust.

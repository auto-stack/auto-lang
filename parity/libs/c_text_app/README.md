# c_text_app (Plan 368 F5 — consumer parity)

Text batch-processor consumer application. Combines `auto.fs` (file read/write)
with text transforms (replace / trim / lower): read a file → transform → write
back → read back. Verifies that Auto composing stdlib capabilities for text
batch processing behaves identically to a native Rust app using std::fs +
`str::replace` / `trim` / `to_lowercase` — three-way (AutoVM / a2r / native Rust).

This is **consumer mode**: Auto calls library capabilities (`fs.read_text` /
`fs.write_text`) plus string methods to do application-level text processing.

## API

| Function | Signature | Transforms | Rust oracle |
|----------|-----------|------------|-------------|
| `transform_replace` | `(path, old, new) -> str` | replace all `old` with `new` | `content.replace(old, new)` |
| `transform_trim` | `(path) -> str` | trim leading/trailing whitespace | `content.trim()` |
| `transform_lower` | `(path) -> str` | lowercase | `content.to_lowercase()` |

## Determinism

Each backend uses a fixed relative dir (`c_text_app_tmp`) under its own working
directory; the Rust oracle uses unique per-test file names so parallel `#[test]`
threads do not collide. ASCII-only inputs so Auto `lower()` and Rust
`to_lowercase` agree. 6 test cases, names mirror the Rust oracle exactly:
`test_replace_basic/no_match`, `test_trim_both/none`, `test_lower_mixed/already`.

## a2r notes (Plan 368 F5)

* A small a2r gap was fixed alongside this lib: `str.lower()` / `str.upper()`
  (the stdlib `str.at` method names) were not in the a2r method-remap table
  (only `to_lower`/`to_upper` were). They now remap to `to_lowercase` /
  `to_uppercase`, matching the VM.
* Same documented str-concat quirks as the other consumer libs are sidestepped
  by passing paths as string literals.
* Text-transform results are bound to a local var before returning
  (`return <method-call>` drops the value in the VM — same family as
  string_utils / Plan 348 C2).

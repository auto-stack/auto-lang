# string_utils (L1)

Pure ASCII string-manipulation parity library. Hand-rolled implementations of
`reverse` / `to_lower` / `to_upper` / `trim` / `contains` / `replace` over
byte-level indexing, so the three-way parity (AutoVM vs a2r-transpiled Rust vs
native Rust std) is fully deterministic. No external dependencies, no IO.

Like `cli_app`, this fills a2r's "pure Rust output" coverage — it exercises the
transpiler's handling of `StringBuilder`, `char_at`, `len`, nested loops, and
early-break control flow without any third-party crate.

## API

| Function | Signature | Mirrors |
|----------|-----------|---------|
| `reverse(s)` | `str -> str` | Rust `s.chars().rev().collect()` |
| `to_lower(s)` | `str -> str` | ASCII-only A-Z fold (not Unicode) |
| `to_upper(s)` | `str -> str` | ASCII-only a-z fold |
| `trim(s)` | `str -> str` | strips `' ' \t \n \r` both ends |
| `contains(h, n)` | `(str, str) -> int` | 1 if found, 0 otherwise |
| `replace(s, src, dest)` | `(str, str, str) -> str` | all occurrences |

22 test cases, names mirror the Rust oracle exactly.

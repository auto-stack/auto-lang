# cli_app Replication

**Upstream:** Rust `std` (text statistics in the style of the `wc` core subset).
**Scope:** three pure functions over a string input — `count_lines`,
`count_words`, `count_chars`.
**Auto features tested:** string indexing (`char_at`), length (`len`), loops,
boolean operators (`&&` / `||` / `!`), integer accumulation, pure computation
with **no external dependencies and no IO**.

This use case exists to fill a gap in a2r test coverage: most other libraries
in this parity workspace wrap a non-trivial Rust crate (base64, url, regex,
serde_json, ...). `cli_app` instead targets the "pure Rust output, no external
library" path — the Auto implementation and the Rust oracle both depend on
nothing but `std`, so the three-way comparison (AutoVM vs a2r-transpiled Rust
vs native Rust std) is fully deterministic and IO-free.

## Why string input, not files?

The original plan considered a `wc`-style program that reads files. File IO
would force the three backends to share fixture files and add non-determinism
(paths, line endings, missing files). Instead `count_*` take the content as a
string, which keeps everything pure-computation and identical across all three
backends.

## API

- `count_lines(s str) int` — number of lines (see definition below)
- `count_words(s str) int` — number of whitespace-separated non-empty tokens
- `count_chars(s str) int` — byte length of the string (ASCII tests only)

## Line-count definition (must match across all three backends)

The rule, identical on the Auto and Rust sides:

- empty string `""` -> `0`
- otherwise, if the string ends with `\n` -> number of `\n` occurrences
- otherwise -> number of `\n` occurrences `+ 1`

| input       | lines |
|-------------|-------|
| `""`        | 0     |
| `"a"`       | 1     |
| `"a\n"`     | 1     |
| `"a\nb"`    | 2     |
| `"a\nb\n"`  | 2     |
| `"\n"`      | 1     |
| `"\n\n"`    | 2     |

This is exactly `str::lines().count()` in Rust std. On the Auto side it is
implemented by counting `\n` (code point 10) and adding 1 unless the last
character is `\n` or the string is empty.

## Word-count definition

Number of non-empty tokens produced by splitting on ASCII whitespace
(`space`, `\t`, `\n`, `\r`). Matches `str::split_whitespace().count()`.
`""` -> 0, `"   "` -> 0, `"a b c"` -> 3.

## Char-count definition

Byte length of the string, matching `str::len()`. All test inputs are ASCII so
there is no byte-vs-code-point ambiguity.

## Coverage

- `tests/auto/basic.at` — 18 cases: empty / single / trailing-newline /
  multi-line / paragraph for `count_lines`; empty / whitespace-only / single /
  multi-word / leading-trailing whitespace for `count_words`; empty / single /
  word / with-newline for `count_chars`.
- `tests/auto/edge_cases.at` — 14 cases: runs of newlines, CRLF, leading
  newline, many lines; tab / mixed-whitespace / CR word separation; tab and
  CRLF char counts; a long string; and a combined three-function sanity check
  over one shared input.

Total: 32 test cases, all mirrored in `tests/rust/tests/cli_app.rs`.

## Implementation notes

The Auto implementation uses `str.char_at(i)` (returns the code point as
`int`) and `str.len()` for length, matching the conventions established by the
`base64` and `url` libraries in this workspace. Logical operators are
`&&` / `||` / `!` (Plan 072 reverted the `and` / `or` keyword forms).

## Known divergences

(none yet)

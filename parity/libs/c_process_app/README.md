# c_process_app (Plan 367 F4 — consumer parity)

CLI argument-parsing consumer application. Verifies that Auto's parsing of a
simulated argv string (the core of any "read `process.args()` then
parse/dispatch" CLI app) behaves identically to a native Rust app using
`str::split(' ')` — three-way (AutoVM / a2r / native Rust).

This is **consumer mode**: it represents the parsing/dispatch logic a CLI app
runs after reading `process.args()`. Because real argv differs per process
(design doc §F4), the suite tests the **parsing logic over a FIXED args
string**, not the real argv.

## API

| Function | Signature | Behavior | Rust oracle |
|----------|-----------|----------|-------------|
| `parse_count` | `(args_str) -> int` | count non-empty space-separated args | `split(' ').filter(non_empty).count()` |
| `parse_nth` | `(args_str, n) -> str` | nth non-empty arg (0-based); `"<none>"` if out of range | `split(' ').filter(...).nth(n)` |

## Determinism

Pure computation over a fixed input string — fully deterministic and
race-free under cargo's parallel `#[test]` threads. 9 test cases, names
mirror the Rust oracle exactly:
`test_count_basic/single/two/empty/extra_spaces`,
`test_nth_first/middle/last/out_of_range`.

## VM quirks worked around (Plan 367 F4)

The implementation avoids several pre-existing AutoVM quirks (each documented
inline in `c_process_app.at`):

1. **`str.split` returns `[]str` but the list return is corrupted across a
   cross-module call** (the test imports the lib). Worked around with a
   char-by-char state machine that returns only int/str primitives.
2. **`int.to(str)` on a char codepoint yields its decimal digits** (e.g. `99`
   → `"99"`, not `"c"`). Worked around with `StringBuilder.append_char(code)`
   + `.build()` (the approach proven by `string_utils`).
3. **`return` inside a `for` loop body is mis-codegenned.** Worked around with
   a `built`/`done` flag — the result is captured, and the `return` happens
   after the loop.
4. **`print("")` swallows the preceding output line.** Worked around by using
   a non-empty `"<none>"` sentinel for the out-of-range case instead of `""`.

A small a2r alignment fix accompanied this lib: `StringBuilder::build` now
takes `&self` (returns a clone) in both `a2r-std` and the embedded
`auto_lang::a2r_std`, matching the VM's non-consuming `sb.build()` semantics
so an `.at` source can build() more than once / after a conditional path.

# base64 Replication

**Upstream:** base64 crate v0.22.0
**Scope:** `encode` (standard alphabet, padded) and `decode` (standard alphabet, padded).
**Auto features tested:** string operations, byte manipulation, loops, error handling (Result).

## API

- `encode(input str) str` — encode a string to base64 (standard alphabet, padded)
- `decode(input str) Result[str, str]` — decode base64 to string, `Err` on invalid input

## Implementation notes

The Auto implementation deliberately avoids bitwise *operator* chaining
(`a.and(3).shl(4).or(b)`), which the current AutoVM miscomputes (a known VM
bug). Instead each bit-group is built up from intermediate `let`/`var`
bindings so every native call is applied to a named value.

Single-character output is produced via `StringBuilder.new(n).append_char(code).build()`
because `int.as(char)` does not yield a single-char string in the current VM.

## Known divergences

(none yet)

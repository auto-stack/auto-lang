# Spec Update: Batch 02 — Lexical Structure

**Date**: 2026-04-16
**Plans Referenced**: Plan 026 (complete), Plan 095 (complete), Plan 120 (complete), Plan 121 (complete), Plan 122 (complete), Plan 124 (complete), Plan 126 (complete), Plan 131 (complete), Plan 168 (complete), Plan 169 (done)
**Source Files**: `token.rs`, `lexer.rs`
**Sections Updated**: Language Overview (key features), Lexical Structure (identifiers, keywords, operators, literals)

## Old Content

### Keywords
Old spec listed 24 keywords: `alias, as, break, const, else, enum, false, fn, for, has, if, in, is, let, mut, nil, null, on, tag, true, type, union, use, var, when`

### Operators
16 operators listed in a single flat table.

### Literals
- Integers: only `digit+ ("u" | "u8" | "i8")?`
- Floats: basic scientific notation
- Strings: single-quoted and C-string only
- No multi-line strings

### Identifiers
No hyphen support documented.

## New Content

### Keywords
Updated to 56 keywords from `token.rs`, organized by category:
- Declarations: added `spec`, `ext`, `static`, `shared`, `impl`, `node`
- Ownership: added `view`, `move`, `copy`, `take`, `hold`
- Option/Result: added `None`, `Some`, `Ok`, `Err`
- Concurrency: added `task`, `spawn`, `await`, `reply`, `go`
- Modules: added `pac`, `super`, `dep`
- Boolean logic: added `and`, `or`
- UI/Routing: added `routes`, `outlet`, `link`, `route`, `nav`
- Removed `while` (not a keyword; Auto uses `for cond { }`)

### Operators
- Added modulo `%` and `%=` assignment
- Added null-safe operators: `??`, `?.`, `.?` (Plan 120)
- Added ownership dot-operators: `.view`, `.mut`, `.move`, `.take` (Plan 122)
- Added `~` (tilde, async marker, Plan 124)
- Added compile-time tokens: `#if`, `#for`, `#is`, `#{` (Plan 095)
- Organized into categorized tables instead of flat list

### Literals
- Added hex integers `0x1A`
- Added underscore separators `1_000_000`
- Added float suffixes `42f`, `42d`, `3.14f`, `3.14d`
- Added multi-line strings `"""..."""` (Plan 169)

### Identifiers
- Added hyphen support within identifiers (e.g., `preview-card`)

## Notes

- `while` was removed from keywords list — Auto uses `for cond { }` instead
- `.take` is deprecated in favor of `.move` (Plan 122)
- `#if`/`#for`/`#is`/`#{` are compound tokens, not `#` + keyword
- Boundary check: `#ifx` is lexed as `Hash` + `Ident("ifx")`, not `HashIf`

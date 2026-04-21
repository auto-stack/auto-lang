# Plan 169: Multi-Line Strings (`"""..."""`)

**Status:** DONE

**Context:** AutoLang has no multi-line string syntax. Strings must fit on one line. Adding `"""..."""` (Python-style triple quotes) enables embedded newlines, useful for prompts, templates, and code generation.

**Why `"""` over `r#"..."#`:** AutoLang uses `#` heavily for annotations (`#[pub]`, `#[derive(...)]`). Adding `r#"..."#` would create confusion. Triple quotes are simple, widely recognized, and don't conflict.

---

## Scope

| Component | Change? | Reason |
|-----------|---------|--------|
| Lexer (`lexer.rs`) | Yes | New `multi_str()` method + dispatch |
| Token (`token.rs`) | No | `TokenKind::Str` works as-is |
| Parser (`parser.rs`) | No | `Expr::Str(AutoStr)` holds any content |
| AST (`ast.rs`) | No | Same |
| VM codegen (`vm/codegen.rs`) | No | `LOAD_STR` with byte content handles it |
| `trans.rs` | Yes | New shared `escape_str()` helper |
| `trans/rust.rs` | Yes | Use `escape_str()` for string emission |
| `trans/c.rs` | Yes | Same |
| `trans/python.rs` | Yes | Same |
| `trans/javascript.rs` | Yes | Same |
| `trans/ts_expr.rs` | Yes | Same |

---

## Implementation Notes

### `escape_str()` in `trans.rs`
Shared helper that escapes `\`, `"`, `\n`, `\r`, `\t`, `\0` for embedding in double-quoted string literals across all transpilers.

### `multi_str()` in lexer
Reads `"""..."""` preserving literal newlines. When encountering runs of `"` characters, counts all consecutive quotes — only the LAST 3 close the string, extras become content (e.g., `""""` = one `"` content + closing `"""`). This avoids stray `"` tokens that would start unintended regular strings.

### `next_step()` dispatch
On `"`, clones the char iterator to peek ahead. If the next two chars are also `"`, dispatches to `multi_str()`; otherwise falls through to `str()`.

---

## Edge Cases

| Source | Lexer Token | Rust Output |
|--------|------------|-------------|
| `""""""` | `Str("")` | `""` |
| `"""hello"""` | `Str("hello")` | `"hello"` |
| `"""a\nb"""` | `Str("a\nb")` | `"a\nb"` |
| `"""a"b"""` | `Str("a\"b")` | `"a\"b"` |
| `"""a""b"""` | `Str("a\"\"b")` | `"a\"\"b"` |
| `"hello"` | `Str("hello")` | `"hello"` (unchanged) |

---

## Tests

- 7 lexer unit tests: simple, newlines, embedded quotes (single/double), escapes, empty, regression
- 1 a2r integration test: `test/a2r/163_multi_str/`
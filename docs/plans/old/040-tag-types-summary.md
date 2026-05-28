# Tag Types Implementation Summary

## Status: COMPLETE (5/5 tasks)

All core tasks are done. The `tag` keyword is now **DEPRECATED** and redirects to `enum` (heterogeneous). The unified `enum` keyword (Plan 156) supersedes `tag`, but the `Tag` AST nodes and `Cover::Tag`/`TagUncover` paths remain active for both `tag` and heterogeneous `enum` pattern matching.

### Completed Tasks

#### 1. Tag Variant Construction
**Syntax:** `Tag.Variant(args)` or `Enum.Variant(args)` (heterogeneous)

**Implementation:**
- `parser.rs`: Extended `Tag` AST to include methods field
- `eval.rs`: Added tag construction detection in `eval_call()`
- `eval.rs`: Implemented `eval_tag_construction()` to create Node with variant/payload

#### 2. Tag Pattern Matching
**Syntax:** `is target { Tag.Variant(var) -> ... }`

**Implementation:**
- `eval.rs`: Extended `eval_is()` to handle TagCover patterns
- `parser.rs`: `tag_cover()` and `is_branch_cond_expr_inner()` both produce `Cover::Tag(TagCover)`
- Works identically for both `tag` keyword and heterogeneous `enum` keyword

#### 3. Tag AST Extension
- `ast/tag.rs`: `Tag` struct with `methods` field
- `ast/cover.rs`: `TagCover`, `TagUncover` (with `binding` field), `Cover::Tag` enum

#### 4. Tag Parser Extension
- `parser.rs`: `fn` parsing branch in `tag_stmt()`, `tag_cover()` for pattern matching
- `parser.rs`: `enter_scope()` + `self.define()` for binding variables in match arms

#### 5. Tag Transpilation (was listed as pending, now complete)
**All four transpiler backends support tag types:**

| Backend | Implementation | Status |
|---|---|---|
| **C** (`trans/c.rs`) | `tag()`, `tag_enum()`, `tag_struct()`, `tag_method_decl/impl()`, `enum_decl_to_tag()` -- generates `enum XKind` + `struct X { enum XKind tag; union { ... } as; }` | Complete |
| **Rust** (`trans/rust.rs`) | `Cover::Tag` generates `Enum::Variant(bindings)` match arms; `TagUncover` emits binding variable name | Complete |
| **TypeScript** (`trans/ts_expr.rs`, `ts_stmt.rs`) | `tag_decl()` generates discriminated union type; `TagUncover` generates `src.value` field access | Complete |
| **Python** (`trans/python.rs`) | `tag_decl()` generates `@dataclass` with `kind` discriminator + factory methods | Complete |

### TagUncover Fix (2026-05-28)

Previously `Expr::Uncover(TagUncover)` in the Rust transpiler emitted a comment placeholder `/* TagUncover: src */` instead of actual code. Fixed by:
- Adding `binding: AutoStr` field to `TagUncover` struct (`ast/cover.rs`)
- Setting `binding` in parser when creating `TagUncover` (`parser.rs` two sites)
- Emitting `uncover.binding` in Rust transpiler (`trans/rust.rs`)

### Known Issues

#### Tag Method Parsing (Low Priority)
Methods inside tag bodies fail to parse. Workaround: use `ext` blocks.

#### `tag` keyword DEPRECATED
`tag` redirects to `enum_stmt()` in the parser. Use heterogeneous `enum` instead:
```auto
enum Atom { Int int, Float float }
```

### Test Coverage

- **a2r**: `06_pattern_matching/004_hetero_enum` (single-expr body), `008_hetero_enum_multistmt` (multi-stmt body with binding)
- **a2p**: `test_02_005_tag` (Python tag transpilation)
- **a2c**: Heterogeneous enums reuse tag code path via `enum_decl_to_tag()` (tested in `06_pattern_matching/`)
- **a2ts**: Heterogeneous enums reuse tag code path (tested in ArkTS tests)

# Plan: Update Auto Language Specification from Implementation

## Context

The TAPL book was written against an outdated spec (v0.1, dated "2025"). The actual lexer has **62 keywords** while the spec lists **24**. The parser supports **34 statement types** and **50+ expression types** — many are undocumented. At least 10 major language features (specs/traits, actors, async, compile-time, etc.) are completely absent from the spec. We need to update the spec to match the implemented reality so the TAPL book (and future development) has an accurate reference.

**Goal**: Analyze lexer/parser code and all 176 implementation plans, then systematically update the spec at `auto-lang/docs/language/specification.md`. Each batch of changes is documented in a companion "spec update file" under `auto-lang/docs/language/spec-updates/`.

---

## Approach

12 batches, ordered by dependency. Each batch:
1. Reads relevant source code (lexer keywords, parser AST nodes)
2. Reads relevant plan files for context
3. Updates specific spec sections
4. Creates a spec-update file documenting what changed and why

### Source Files (authoritative)

| File | Purpose |
|------|---------|
| `crates/auto-lang/src/token.rs` | All tokens and 62 keywords |
| `crates/auto-lang/src/lexer.rs` | Literal parsing, comment handling |
| `crates/auto-lang/src/ast.rs` | Stmt (34 variants), Expr (50+ variants) |
| `crates/auto-lang/src/ast/types.rs` | Type enum (25+ variants), TypeDecl, generics |
| `crates/auto-lang/src/ast/fun.rs` | Fn, Param, ParamMode, Closure |
| `crates/auto-lang/src/ast/spec.rs` | SpecDecl, SpecImpl |
| `crates/auto-lang/src/ast/enums.rs` | EnumDecl, EnumItem, EnumKind |
| `crates/auto-lang/src/ast/ext.rs` | Ext (type extensions) |
| `crates/auto-lang/src/ast/task.rs` | TaskDef, TaskOnBlock |
| `crates/auto-lang/src/ast/comptime.rs` | HashIf, HashFor, HashBrace |
| `crates/auto-lang/src/ast/store.rs` | StoreKind (let/var/mut/const/shared) |
| `docs/plans/*.md` (176 files) | Implementation plans with status markers |

---

## Batches

### Batch 1: Metadata & TOC Restructure
- Update version 0.1 → 0.2, date → 2026-04
- Restructure TOC: add 11 new sections (Specs/Traits, Generics, Closures, Option/Result, Concurrency, Async, Compile-Time, Ownership, Modules, UI, Routing)
- **Spec update file**: `spec-updates/batch-01-metadata.md`

### Batch 2: Lexical Structure (Section 3)
- Update keywords: 24 → 62 (categorized table matching token.rs)
- Add operators: `??`, `?.`, `.view`, `.mut`, `.move`, `.take`, `~`, `%`/`%=`
- Add literals: triple-quoted `"""`, C strings `c"..."`, hex `0x`, suffixes `i64`/`u64`/`usize`/`f`/`d`
- Add compile-time tokens: `#if`, `#for`, `#is`, `#{}`
- **Plans**: 026, 095, 120, 121, 122, 124, 126, 131, 168, 169
- **Spec update file**: `spec-updates/batch-02-lexical.md`

### Batch 3: Types (Section 5)
- Add missing primitive types: `usize`, `i64`/`u64`, `String`, `cstr`
- Add compound types: `[N]T`, `[]T`, `List<T>`, `Map<K,V>`, `*T`, `&T`
- Add `?T` (Option), `!T` (Result), `Handle<T>`, `linear<T>`
- Add function types: `fn(params) ret`
- Add generic type instances
- **Plans**: 048, 052, 120, 121, 160, 155
- **Spec update file**: `spec-updates/batch-03-types.md`

### Batch 4: Expressions (Section 6)
- Add all 50+ expression types from Expr enum
- Closures: `x => expr`, `(a, b) => expr`
- Ownership: `.view`, `.mut`, `.move`
- Type conversion: `.as(Type)`, `.to(Type)`
- Option/Result: `Some(v)`, `None`, `Ok(v)`, `Err(e)`
- Async: `~{ }`, `.await`, `.go`
- Null coalescing: `??`, error propagation: `.?`
- Compile-time: `#{ expr }`
- Full precedence table from parser.rs
- **Plans**: 026, 060, 095, 120, 122, 124, 126, 161, 162, 165
- **Spec update file**: `spec-updates/batch-04-expressions.md`

### Batch 5: Statements & Control Flow (Sections 7-8)
- Add `shared` storage modifier
- Add `return`, `reply` statements
- Enhance `is` with struct destructuring, option/result patterns
- Document `when` blocks
- **Plans**: 120, 122, 124, 165, 168
- **Spec update file**: `spec-updates/batch-05-statements.md`

### Batch 6: Functions (Section 9)
- Generic functions: `fn foo<T>(x: T) -> T`
- Parameter modes: `view` default, `mut`, `move`
- Static methods: `static fn new()`
- Pub visibility: `pub fn foo()`
- Closures: full documentation
- **Plans**: 035, 048, 060, 088, 122, 163
- **Spec update file**: `spec-updates/batch-06-functions.md`

### Batch 7: Type Definitions, Enums, Tags (Sections 11-12)
- Single inheritance: `type Dog is Animal { ... }`
- Generic types: `type List<T> { ... }`, const generics
- Spec implementation: `type X has SpecY`, `type X as SpecY`
- Enum kinds: scalar, with-repr, homogeneous, heterogeneous (ADT), generic
- `ext` blocks: `ext str { fn method() { ... } }`
- **Plans**: 019, 021, 035, 048, 057, 059, 163
- **Spec update file**: `spec-updates/batch-07-type-defs.md`

### Batch 8: NEW — Specs & Traits (Section 13)
- Spec declaration: `spec Printable { fn print() }`
- Generic specs: `spec Iterable<T> { ... }`
- Default methods, spec bounds, polymorphic dispatch
- Transpilation behavior (C: vtables, Rust: native traits)
- **Plans**: 019, 057, 059
- **Spec update file**: `spec-updates/batch-08-specs.md`

### Batch 9: NEW — Generics, Closures, Option/Result (Sections 14-16)
- Generics: type params, const params, constraints, monomorphization
- Closures: syntax forms, capture semantics, iterator usage
- Option/Result: `?T`/`!T`, constructors, error propagation, pattern matching
- **Plans**: 048, 049, 052, 057, 059, 060, 120, 165
- **Spec update file**: `spec-updates/batch-09-generics-closures-option.md`

### Batch 10: NEW — Concurrency: Tasks & Async (Sections 17-18)
- Task definition, lifecycle hooks, message handling
- `spawn`, `send`, `Handle<T>`
- `~T` async type, `~{ }` blocks, `.await`, `.go`
- Ask/Reply RPC, backpressure
- **Plans**: 121, 124, 126
- **Spec update file**: `spec-updates/batch-10-concurrency.md`

### Batch 11: NEW — Compile-Time, Ownership, Modules (Sections 19-21)
- `#if`/`#for`/`#is`/`#{}` compile-time execution
- Ownership trinity: `view`/`mut`/`move`, `.clone()`, `hold`
- `use` statements, `pac`/`super` paths, `dep` declarations, `pub` exports
- **Plans**: 095, 122, 131, 167, 168, 163
- **Spec update file**: `spec-updates/batch-11-comptime-ownership-modules.md`

### Batch 12: NEW — UI, Routing, Final Cleanup (Sections 22-23)
- Widget/model/view/msg (contextual in UI scenario)
- Routes/outlet/link/nav
- Final pass: update appendices, cross-references, deprecation notes
- **Plans**: 096, 105, 106
- **Spec update file**: `spec-updates/batch-12-ui-routing-cleanup.md`

---

## Spec Update File Format

Each file at `docs/language/spec-updates/batch-NN-name.md`:

```markdown
# Spec Update: Batch NN — [Title]
**Date**: 2026-04-XX
**Plans Referenced**: Plan XXX (status), Plan XXX (status), ...
**Source Files**: token.rs, ast/types.rs, ...
**Sections Updated**: Section N (update type)

## Old Content
[literal copy of replaced text, or "NEW SECTION"]

## New Content
[new spec text]

## Notes
[caveats, deprecated features, backend limitations]
```

---

## Verification

After all batches:
1. **Keyword coverage**: Every keyword in `token.rs` keyword_kind() appears in the spec
2. **Token coverage**: Every TokenKind variant is documented
3. **AST coverage**: Every Stmt and Expr variant has spec documentation
4. **Type coverage**: Every Type variant is in the types section
5. **Plan cross-ref**: Every completed plan (✅) has corresponding spec content
6. **Consistency**: No contradictions between spec sections
7. Run the project's test suite to ensure nothing broke (read-only spec changes shouldn't affect tests)

# Plan 242: a2r Feature Gap Tracker

## Background

The Auto-to-Rust transpiler (a2r) has reached a functional state with 144 passing tests across 17 categorized directories (Plan 170/204/220/223). However, significant gaps remain compared to the C transpiler (a2c) in terms of feature parity, type system fidelity, and production readiness.

This tracker serves as a **living document** that inventories all outstanding a2r features, workarounds, and partial implementations. Items are sorted by estimated implementation difficulty (easiest first). As items are completed, this file should be updated with completion dates and the corresponding plan status should be synchronized in `docs/plan-indices/` and `docs/plan-reports/`.

> **Exclusion**: Lifetime annotation generation (`'a`) is recognized as a complex, cross-cutting concern and is deliberately **excluded** from this tracker. It will be addressed in a dedicated plan after the ownership model (Item 11) matures.

---

## Tracker Table

| # | Feature | Status | Plan | Difficulty | Owner | Blockers | Completion Date |
|---|---------|--------|------|------------|-------|----------|-----------------|
| 1 | Generic constraints output (`<T: Trait>`) | ⏳ Planned | 166 / 364 | ⭐ Low | — | fn-level done (Plan 166); type/impl-level + multi-bound tracked in Plan 364 W3 | — |
| 2 | HashMap literal transpilation | ⚠️ Workaround | 159 | ⭐⭐ Low-Mid | — | Type context propagation in transpiler | — |
| 3 | `.to(Type)` method keyword | ⏳ Planned | 162 | ⭐⭐ Low-Mid | — | None (`.as(Type)` provides reference pattern) | — |
| 4 | `Result<T>` error handling chains | 🔧 Partial | 159 | ⭐⭐ Mid | — | None | — |
| 5 | Struct destructuring in `is` (`{x,y}`) | ⏳ Planned | 165 | ⭐⭐⭐ Mid | — | Pattern parser extensions | — |
| 6 | External trait impl (`ext Type for Trait`) | ⏳ Planned | 164 | ⭐⭐⭐ Mid | — | Parser/AST ext block extensions | — |
| 7 | `String` vs `&str` distinction | 🔧 Partial | 159 | ⭐⭐⭐⭐ Mid-High | — | Type system or transpiler heuristic enhancements | — |
| 8 | Complex closure type inference | 🔧 Partial | 159 | ⭐⭐⭐⭐ Mid-High | — | Type inference engine enhancements | — |
| 9 | Platform-specific files (`.rs.at`, `#[rs]`) | 🔧 Partial | 083 | ⭐⭐⭐⭐ High | — | Compiler file-loading pipeline refinements | — |
| 10 | a2rs backend stdlib (Redis/SQLite) | 🔧 Partial | 119 | ⭐⭐⭐⭐ High | — | Plan 121 async system maturity; 6 cookbook DB stubs handed off from Plan 240 Phase 10 | — |
| 11 | Ownership and borrowing model (beyond `Rc<T>`/`clone()`) | ⚠️ Workaround | — | ⭐⭐⭐⭐⭐ Very High | — | Precise ownership analysis in transpiler | — |
| 12 | a2rs async model (blocking → tokio) | ⚠️ Workaround | 119 / 355 | ⭐⭐⭐⭐⭐ Very High | — | Plan 355 a2r async/await; 13 cookbook async stubs handed off from Plan 240 Phase 12 | — |
| 13 | Core a2r completeness / a2c parity | 🔧 Partial | 007 / 067 | ⭐⭐⭐⭐⭐ Very High | — | Many edge cases across expr/stmt/types | — |
| 14 | Rust Cookbook systematic test suite | ✅ Done | 240 | ⭐⭐⭐⭐⭐ Very High | — | Core suite complete (163 .at, 124/124 a2r pass); DB/async/cc stubs handed to #10/#12/#17 | 2026-07-14 |
| 15 | a2r UI generator (GPUI/ICED) | ⏳ Planned | 180 | ⭐⭐⭐⭐⭐ Extreme | — | AURA → GPUI mapping layer; auto-ui integration | — |
| 16 | Self-hosting a2r transpiler (in Auto) | 🔧 Partial | 229 / 237 | ⭐⭐⭐⭐⭐ Maximum | — | Generics, pattern matching, trait system completion | — |
| 17 | Build-time codegen (`dep cc`) + `memmap2` FFI | ⏳ Planned | — | ⭐⭐⭐⭐ High | — | build-time codegen + memmap2 FFI bridge; 4 cookbook stubs handed off from Plan 240 Phase 13 | — |

**Legend**
- ⏳ Planned = Not yet started
- 🔧 Partial = Started but incomplete
- ⚠️ Workaround = Functional via temporary/ suboptimal mechanism
- ✅ Done = Fully implemented and tested

---

## Feature Details

### 1. Generic Constraints Output (`<T: Trait>`)
**Current state**: `#[with(T as Trait)]` annotations are parsed and stored in the AST, but a2r ignores them when emitting function/type signatures.
**Desired state**: `fn foo<T>() #[with(T as Clone)] { ... }` → `fn foo<T: Clone>() { ... }`
**Files likely touched**: `crates/auto-lang/src/trans/rust.rs`
**Acceptance criteria**:
- [ ] `#[with(T as Trait)]` on `fn` emits `<T: Trait>`
- [ ] Multiple bounds emit `<T: Trait1 + Trait2>`
- [ ] Associated specs (traits) are mapped correctly

---

### 2. HashMap Literal Transpilation
**Current state**: Map literals with type annotations emit struct/object syntax instead of idiomatic `HashMap::from([...])`.
**Desired state**: `let m Map<str, int> = {"a": 1}` → `let m: HashMap<String, i32> = HashMap::from([(String::from("a"), 1)]);`
**Files likely touched**: `crates/auto-lang/src/trans/rust.rs` (expression emission)
**Acceptance criteria**:
- [ ] Map literals in a2r output use `HashMap::from([...])`
- [ ] Key/value types are correctly converted (e.g., `str` keys become `String::from(...)`)
- [ ] Empty map literals emit `HashMap::new()`

---

### 3. `.to(Type)` Method Keyword
**Current state**: Only `.as(Type)` (reinterpret cast) is implemented (Plan 161).
**Desired state**: `.to(Type)` performs converting/allocating casts: `x.to(str)` → `x.to_string()`, `x.to(int)` → `x.parse::<i32>().unwrap()`.
**Files likely touched**: `parser.rs` (Pratt parser / `dot_item()`), `ast.rs` (new `Expr::To`), `trans/rust.rs`, `trans/c.rs`, `trans/typescript.rs`
**Acceptance criteria**:
- [ ] Parser intercepts `.to(Type)` and produces `Expr::To`
- [ ] a2r emits correct converting methods per type pair
- [ ] Numeric-to-numeric degrades to `as` cast when no conversion method exists

---

### 4. `Result<T>` Error Handling Chains
**Current state**: Basic `Result<T>` type mapping and `?` / `is` patterns work.
**Desired state**: Chain methods like `.map_err()`, `.unwrap_or()`, `.and_then()` map to their Rust equivalents seamlessly.
**Files likely touched**: `trans/rust.rs` (method name mapping), stdlib method registry
**Acceptance criteria**:
- [ ] `.map_err(fn)` → `.map_err(fn)`
- [ ] `.unwrap_or(default)` → `.unwrap_or(default)`
- [ ] `.and_then(fn)` → `.and_then(fn)`
- [ ] Auto-generated `Result` methods are documented in a2r method mapping table

---

### 5. Struct Destructuring in `is` (`{x, y}`)
**Current state**: `is` supports equality and enum variant matching.
**Desired state**: Support Rust-style struct patterns: `is point { Point { x, y } => ... }`
**Files likely touched**: `parser.rs` (pattern parsing), `ast.rs` (`Pattern` enum), `type_checker.rs`, `trans/rust.rs`
**Acceptance criteria**:
- [ ] Parser accepts `{field1, field2}` patterns in `is` arms
- [ ] Type checker validates field names against struct definition
- [ ] a2r emits `Point { x, y } =>` in `match` arms

---

### 6. External Trait Implementation (`ext Type for Trait`)
**Current state**: `ext Type { ... }` works for inherent methods.
**Desired state**: `ext Point for Display { ... }` transpiles to `impl Display for Point { ... }` in a2r.
**Files likely touched**: `parser.rs`, `ast.rs` (`Ext` node), `trans/rust.rs`
**Acceptance criteria**:
- [ ] Parser accepts `ext Type for Trait { ... }`
- [ ] AST stores the target trait reference
- [ ] a2r emits `impl Trait for Type { ... }`
- [ ] a2c produces a clear error or alternative output

---

### 7. `String` vs `&str` Distinction
**Current state**: Auto `str` is largely mapped to Rust `String`. Function parameters and returns do not distinguish owned vs borrowed strings.
**Desired state**: String literals and function parameters that do not need ownership emit `&str`; owned data emits `String`.
**Files likely touched**: `trans/rust.rs`, potentially `type_checker.rs`
**Acceptance criteria**:
- [ ] String literals in function arguments auto-borrow as `&str` when possible
- [ ] Struct fields annotated `Str` emit `String`; `str` parameters can emit `&str`
- [ ] Method calls like `.to_str()` or `.as_str()` are context-aware
- [ ] No regressions in existing 144 a2r tests

---

### 8. Complex Closure Type Inference
**Current state**: Simple closures may work; closures passed as arguments to higher-order functions often require explicit type annotations.
**Desired state**: Closure signatures are inferred from the expected type of the call site (e.g., `.map(|x| x * 2)` infers `x` type from the iterable).
**Files likely touched**: `type_checker.rs`, `ast.rs`
**Acceptance criteria**:
- [ ] Closures passed to `map`/`filter`/`reduce` infer parameter types
- [ ] Multi-parameter closures infer correctly
- [ ] Generic closures with trait bounds are handled or produce clear errors

---

### 9. Platform-Specific Files (`.rs.at`, `#[rs]`)
**Current state**: `.rs.at` loading and `#[rs]` target selection exist but are not production-complete.
**Desired state**: Robust multi-target compilation where `.rs.at` seamlessly overrides `.at` methods for Rust targets, and `#[rs]`/`#[c]`/`#[vm]` reliably control emission.
**Files likely touched**: `compile_session.rs`, `parser.rs`, `trans/rust.rs`
**Acceptance criteria**:
- [ ] `CompileDest::TransRust` consistently loads `.at` then `.rs.at`
- [ ] `#[rs]` functions are emitted only for Rust; `#[c]` only for C
- [ ] Unannotated functions emit for all targets
- [ ] No duplicate symbol errors during multi-file Rust transpilation

---

### 10. a2rs Backend Stdlib (Redis / SQLite)
**Current state**: Only VM-side HTTP FFI (Plan 102) is implemented. Redis and SQLite have API designs but no Rust implementations.
**Desired state**: AutoLang APIs for Redis (`redis` crate) and SQLite (`rusqlite` crate) transpile to idiomatic Rust.
**Files likely touched**: `stdlib/auto/redis.*`, `stdlib/auto/sqlite.*`, `trans/rust.rs`
**Acceptance criteria**:
- [ ] Redis client API transpiles to `redis` crate calls
- [ ] SQLite API transpiles to `rusqlite` calls
- [ ] Connection pooling and error handling are supported
- [ ] Async variants work once Item 12 is complete

> **Cookbook handoff (from Plan 240 Phase 10):** 6 database stub `.at` files (`database/postgres/*`, `database/sqlite/*`) currently print fake DDL. De-stubbing them lands here once `dep rusqlite` FFI is available.

---

### 11. Ownership and Borrowing Model (Beyond `Rc<T>` / `clone()`)
**Current state**: Conservative strategy: `&T` for params, `clone()` for moves, `Rc<T>` for shared references.
**Desired state**: Precise ownership analysis minimizes unnecessary clones and replaces `Rc<T>` with borrowing where possible.
**Files likely touched**: `trans/rust.rs` (broad architectural change)
**Acceptance criteria**:
- [ ] Transpiler analyzes variable liveness to decide move vs borrow
- [ ] Shared immutable references use `&T` instead of `Rc<T>` when safe
- [ ] Mutable borrows are emitted accurately without over-borrowing
- [ ] Performance of generated Rust is comparable to hand-written code

---

### 12. a2rs Async Model (Blocking → Tokio)
**Current state**: Backend stdlib recommends blocking APIs as a workaround.
**Desired state**: Full tokio-based async for a2rs, including `async fn`, `.await`, `tokio::spawn`, and `tokio::main`.
**Files likely touched**: `stdlib/auto/async.*`, `trans/rust.rs`, runtime bindings
**Acceptance criteria**:
- [ ] `async fn` in Auto emits `async fn` in Rust
- [ ] `.await` is emitted correctly at call sites
- [ ] `#[tokio::main]` is auto-inserted for executables
- [ ] Redis/SQLite HTTP backends support async variants

> **Cookbook handoff (from Plan 240 Phase 12):** 13 async stub `.at` files (`asynchronous/*`, async `database/*`) are blocked on the VM-embedded tokio runtime. a2r-side transpilation is tracked in **Plan 355** (async/await). De-stubbing these tests lands here once both the VM runtime and a2r emit are ready.

---

### 13. Core a2r Completeness / a2c Parity
**Current state**: 144 a2r tests vs ~106 a2c tests, but a2c covers more edge cases in C interop, pointers, and stdlib.
**Desired state**: a2r supports all non-C-specific features that a2c supports.
**Files likely touched**: `trans/rust.rs`, test suite
**Acceptance criteria**:
- [ ] Audit a2c-only tests and port applicable ones to a2r
- [ ] a2r test count reaches parity with a2c (or justifiable exceptions documented)
- [ ] All `#[ignore]` a2r tests have associated tickets or are resolved

---

### 14. Rust Cookbook Systematic Test Suite
**Current state**: ✅ Core suite complete (Plan 240, archived). 163 `.at` files across all Cookbook chapters, assert-based (Phase 14), de-stubbed for all non-architecture-blocked files (Phase 15), FAIL-driven VM/transpiler fix loop run (Phase 16). Result: 124/124 a2r pass, 236/236 transpiler pass.
**Deferred to other items**: infrastructure-blocked stubs were handed off rather than completed in 240 — database (6 files) → item #10; async/tokio (13 files) → item #12 / Plan 355; build-time codegen + memmap2 (4 files) → item #17.
**Desired state**: (achieved for the core) A curated suite of a2r tests derived from Rust Cookbook examples covering collections, filesystem, concurrency, encoding, etc.
**Files touched**: `test/cookbook/` (163 `.at` files), `src/tests/a2r_tests.rs`
**Acceptance criteria**:
- [x] 50+ new tests derived from Rust Cookbook idioms (163 delivered)
- [x] All tests pass string-match a2r verification
- [x] Categories mirror Rust Cookbook chapters

---

### 15. a2r UI Generator (GPUI / ICED)
**Current state**: Planned. GPUI/ICED backends exist in standalone `auto-ui` but are not wired into `auto gen`.
**Desired state**: `auto gen --target rust-ui` produces a compileable GPUI project from AURA widgets.
**Files likely touched**: New generator module, `auto-ui` integration
**Acceptance criteria**:
- [ ] AURA widgets transpile to GPUI `div`/`text`/`button` etc.
- [ ] Style properties map to GPUI typed style methods
- [ ] Event handlers delegate to a2ts or inline Rust closures
- [ ] Generated project builds with `cargo run`

---

### 16. Self-Hosting a2r Transpiler (in Auto)
**Current state**: Phase 1 (AAVM parser/evaluator/bytecode) is complete. Phase 2 (a2r transpiler written in Auto) and Phase E (AAVM a2r transpiler) remain.
**Desired state**: AutoLang can transpile itself to Rust via an Auto-written a2r transpiler.
**Files likely touched**: `auto/compiler/transpiler.at` (new)
**Acceptance criteria**:
- [ ] Auto-written transpiler passes all a2r test categories
- [ ] Can transpile `auto/compiler/*.at` to Rust
- [ ] Generated Rust compiler builds and passes a subset of tests

---

### 17. Build-Time Codegen (`dep cc`) + `memmap2` FFI
**Current state**: Not started. No `dep cc` build-time codegen support; no `memmap2` FFI bridge. 4 cookbook stub files block on this (handed off from Plan 240 Phase 13).
**Desired state**: `dep cc` enables `cc::Build` build-time C/C++ compilation; `dep memmap2` (or equivalent) provides memory-mapped file I/O, both transpiling to idiomatic Rust and runnable in the VM where feasible.
**Files likely touched**: build system / `use.rust` FFI bridge, `trans/rust.rs`, VM FFI shims
**Acceptance criteria**:
- [ ] `dep cc` build-time codegen works for the 3 `devtools/cc_*` cookbook stubs
- [ ] `dep memmap2` FFI bridge unblocks `safety/001_memmap`
- [ ] All 4 handed-off stub `.at` files de-stubbed and asserting real behavior

---

## Execution Roadmap

### Phase A: Quick Wins (Items 1–4)
Focus on low-risk, high-value transpiler-only changes that improve correctness and expressiveness.
- **1** Generic constraints
- **2** HashMap literals
- **3** `.to(Type)`
- **4** Result chains

### Phase B: Language Features (Items 5–8)
Extend parser, AST, and type checker to support richer Rust idioms.
- **5** Struct destructuring
- **6** External trait impl
- **7** String/&str distinction
- **8** Closure inference

### Phase C: Platform & Runtime (Items 9–12)
Solidify Rust-target compilation pipeline and backend services.
- **9** `.rs.at` / `#[rs]` hardening
- **10** Redis/SQLite stdlib
- **11** Ownership model improvements
- **12** Async tokio migration

### Phase D: Validation & Ecosystem (Items 13–16)
Large-scale validation, UI generator, and the self-hosting milestone.
- **13** a2c parity push
- **14** Rust Cookbook tests
- **15** a2rust-ui generator
- **16** Self-hosting transpiler

### Phase E: Build Tooling & FFI (Item 17)
Build-time codegen and memory-mapped I/O bridging — unblocks the last Cookbook stubs.
- **17** `dep cc` codegen + `memmap2` FFI

---

## Changelog

| Date | Change | Author |
|------|--------|--------|
| 2026-05-09 | Initial tracker created from plan-indices / plan-reports audit | — |
| 2026-07-14 | Item #14 (Cookbook suite) marked ✅ Done — Plan 240 core complete & archived; DB/async stubs handed to #10/#12; new item #17 (cc codegen + memmap2) added from Plan 240 Phase 13 | — |

---

## Related Documents

- `docs/plan-indices/06-transpilers.md`
- `docs/plan-reports/06-transpilers.md`
- `docs/plan-indices/05-stdlib.md`
- `docs/plan-reports/05-stdlib.md`
- `docs/plan-indices/13-self-hosting.md`
- `docs/plan-reports/13-self-hosting.md`
- `docs/plan-indices/16-shell-tools.md`
- `docs/plan-reports/16-shell-tools.md`
- `docs/plans/241-a2r-string-type-cleanup.md`

# 06 - Transpilers and Code Generation

## Overview

AutoLang supports a comprehensive suite of transpiler backends that convert AutoLang source code into target languages including C, Rust, TypeScript, Python, and JavaScript. The system also includes a reverse transpiler (r2a) for importing Rust code back into AutoLang. Each backend follows a shared architectural pattern: parse AutoLang source, walk the AST, and emit idiomatic target-language code. The transpilers differ in maturity: a2c is the most mature with 106 tests, followed by a2r at 144 tests, a2ts at 24, and a2p/a2j at 10 and 9 tests respectively. UI-specific generators produce output for Vue, ArkTS, GPUI, and VSCode extensions. Across 22 plans, 10 are completed, 3 are partially implemented, and 9 remain in planning stages.

## Plan Index

| Plan | Title | Status | Summary |
|------|-------|--------|---------|
| 007 | Implement a2r Transpiler | Partial | Auto-to-Rust transpiler; Phase 1 core infrastructure complete, Phases 2-6 pending |
| 022 | Python Transpiler (a2p) | Done | Complete 10-phase implementation: expressions, control flow, functions, pattern matching, classes |
| 023 | JavaScript Transpiler (a2j) | Done | Complete 11-phase implementation: all JS features in single phase, 9/9 tests passing |
| 062 | C Transpiler Generics | Done | Monomorphization for a2c: type specialization, array tests, miette error messages (127 tests) |
| 067 | Strengthen Rust Transpiler | Partial | Gap analysis to bring a2r to feature parity with a2c (38% coverage, 90/238 tests) |
| 083 | a2r with .rs.at and #[rs] | Partial | Platform-specific Rust implementation files and #[rs] annotation support |
| 100 | a2js to a2ts Migration | Partial | Upgrade JavaScript generator to TypeScript with ArkTS variant support |
| 161 | a2r List + Auto Features | Done | #[rs] target selector, .as(Type) cast, and a2r List<T> support |
| 162 | .to(Type) Method Keyword | Planned | Explicit type conversion method keyword complementing .as(Type) reinterpret cast |
| 163 | a2r Core Struct Support | Done | 5 core struct features: static fn, nested fields, enum tag values, Option/Result, user attrs |
| 164 | a2r ext for Trait | Planned | External trait implementation via ext Type for Trait syntax in a2r |
| 165 | Struct Destructuring in is | Planned | Rust-style {field1, field2} struct destructuring in is match arms |
| 166 | a2r Generic Constraints | Planned | Emit #[with(T as Trait)] annotations as <T: Trait> in Rust output |
| 170 | a2r Test Reorganization | Done | Reorganized ~60 a2r tests into 17 categorized directories, 144 tests passing |
| 171 | a2c Test Reorganization | Done | Reorganized 239 a2c test directories into 25 categorized directories, 106 tests passing |
| 172 | a2ts Test Reorganization | Done | Reorganized 24 a2ts tests into 10 categorized directories, all passing |
| 173 | r2a Rust-to-Auto Transpiler | Done | Reverse transpiler: Rust to AutoLang via syn crate, 116 tests across 4 phases |
| 174 | Conditional UI Backends | Planned | ui-headless feature flag for UI-less builds, skipping GPUI/ICED dependencies |
| 175 | Migrate auto-ui into auto-lang | Planned | Move GPUI and ICED backends from standalone auto-ui into auto-lang workspace |
| 180 | a2rust-ui Generator | Planned | Wire RustGenerator into auto gen for Rust UI backend (GPUI examples) |
| 181 | a2vscode Generator | Planned | VSCode extension generator from AURA widgets using a2vue + webview panel |
| 187 | a2ts Vue Adapter | Planned | Replace Vue generator's inline JS with a2ts delegation for proper TypeScript output |
| 204 | a2r Transpiler Completeness | Planned | Close a2r feature gap: Result, spec, struct, enum, stdlib method mapping |
| 213 | a2py Python Transpiler Maturation | Planned | Expand Python transpiler from 18% to 80%+ coverage (Option/Result, closures, generics) |
| 215 | a2ts TypeScript Transpiler Maturation | Planned | Expand TypeScript transpiler from 24 to 80+ tests (Option/Result, collections, async) |
| 216 | C FFI Bindgen | Complete | Auto-bindgen for C headers with libloading runtime, a2c auto-bind, CLI integration |

## Status

**Implemented**: a2p (Python, 10 tests), a2j (JavaScript, 9 tests), a2c generics monomorphization, r2a reverse transpiler (116 tests), a2r core struct support (static fn, pub, tokio main, mut self, field attrs), a2r list implementation and .as(Type) cast, test suite reorganization for a2r/a2c/a2ts, C FFI bindgen with libloading runtime and CLI integration (Plan 216).

**Partial**: a2r (144 tests, 38% parity with a2c, ongoing gap closure), a2r .rs.at platform-specific files, a2ts migration from a2js (Phase 2-3 done, Phase 4 pending).

**Planned**: .to(Type) method keyword, ext-for-trait in a2r, struct destructuring, generic constraints, conditional UI backends, auto-ui migration, a2rust-ui generator, a2vscode generator, a2ts Vue adapter, a2r transpiler completeness (Plan 204), a2py Python transpiler maturation (Plan 213), a2ts TypeScript transpiler maturation (Plan 215).

## Design

### Forward Transpiler Architecture

All forward transpilers follow a shared pattern. Each implements a `Trans` trait that accepts a parsed AutoLang AST (`Code`) and writes output to a `Sink`. The core methods are `expr()` for expression translation, `stmt()` for statement dispatch, and type-specific helpers like `fn_decl()`, `type_decl()`, `enum_decl()`, and `is_stmt()`. The a2c transpiler at `crates/auto-lang/src/trans/c.rs` serves as the architectural reference at over 2000 lines, supporting 40+ expression variants and 16+ statement types with dual-file generation (.c and .h).

The a2r transpiler at `crates/auto-lang/src/trans/rust.rs` follows the same structure but generates single-file Rust modules instead of dual header/implementation files. A key design decision for a2r is how to handle ownership and borrowing: the transpiler prefers references for function parameters (`&T`), uses `clone()` when ownership transfer is needed, and adds `mut` keywords where mutable borrows are required. The type mapping converts AutoLang primitives to their Rust equivalents: `Int` to `i32`, `Float` to `f64`, `Str` to `String`, `Array(arr)` to `[T; N]` or `Vec<T>`, and `Ptr(ptr)` to `&T` or `Box<T>` with smart detection.

### C Transpiler and Monomorphization

The a2c transpiler supports generic type monomorphization, which specializes generic types at compile time. For example, `Box<int>` generates a `box_int` type in C, while return types like `str` map to `char*`. Plan 062 brought the test suite to 127 passing tests with miette error display helpers and proper handling of generic fields. Tests for unimplemented features (generic tags, nested arrays, const generics) are marked as `#[ignore]` with descriptive reasons, providing a clear roadmap for future work.

### Python and JavaScript Transpilers

The a2p (Python) and a2j (JavaScript) transpilers are complete and production-ready. Both benefit from the dynamic nature of their target languages, which eliminates type system friction. A notable design insight is that AutoLang f-strings map directly to Python f-strings and JavaScript template literals with only minor syntax changes: `$name` becomes `{name}` in Python and `${name}` in JavaScript, while the surrounding quotes change to backticks for JS template literals.

The Python transpiler maps AutoLang `type` declarations to `@dataclass` decorated classes, `enum` to `enum.Enum`, and `is` pattern matching to Python 3.10+ `match/case` (requiring Python 3.10 as a minimum). The JavaScript transpiler produces ES6+ code with `class` for structs, `Object.freeze()` for enums, `switch/case` for pattern matching, and `const`/`let` based on AutoLang's `let`/`var` distinction. The JavaScript transpiler was notably completed faster than estimated because all core features fit naturally into a single implementation phase.

### TypeScript Migration and ArkTS

Plan 100 describes upgrading the JavaScript generator to TypeScript with an ArkTS variant for HarmonyOS frontend development. The migration creates a shared `ts_common.rs` module for TypeScript generation logic, upgrades `trans/javascript.rs` to `trans/typescript.rs` with type annotations, and updates the Vue generator to emit `lang="ts"` with `ref<T>()` type annotations. Phase 2 (a2ts core) and Phase 3 (Vue generator TypeScript upgrade) are complete, producing `interface` instead of `class` for struct types and `const enum` for enums. Phase 4 (shared logic extraction) remains pending.

### Reverse Transpiler: r2a

The r2a transpiler at `crates/auto-lang/src/trans/r2a.rs` converts Rust source code back into AutoLang using the `syn` crate v2 for parsing. Rather than building an intermediate AutoLang AST, r2a performs direct syn-to-text conversion via a `R2aTrans` struct. This covers 116 tests across four phases: core functions/variables/control flow, impl/trait/spec/ext blocks, generics with trait bounds, and async/comment degradation with module support.

Key syntax mappings include `fn add(a: i32, b: i32) -> i32` to `fn add(a int, b int) int`, `let mut count = 0` to `var count = 0`, `trait Flyer` to `spec Flyer`, `impl Flyer for Bird` to `ext Bird for Flyer`, and `impl Bird` to `ext Bird`. The transpiler uses method self-parameter detection to determine `static fn` (no self), plain `fn` (`&self`), or `mut fn` (`&mut self`). Features that AutoLang cannot express (lifetimes, `unsafe`, `dyn Trait`) degrade to comment markers rather than errors.

### Rust Transpiler Feature Development

The a2r transpiler has undergone significant feature development. Plan 161 introduced `.as(Type)` cast expressions (zero-overhead reinterpret, mapping to Rust's `as` operator), pointer null-check methods (`is_null()` / `is_not_null()`), and `#[rs]` target selection for controlling which transpiler backend generates a given function. Plan 163 added five core struct features: `static fn` for associated functions without `&self`, `#[pub]` visibility propagation to AST nodes, automatic `#[tokio::main]` detection when `await` appears in main, `mut fn` for `&mut self` methods, and per-field serde attribute passthrough.

Plans 164, 165, and 166 describe planned features that complete the Rust transpiler's OOP support. Plan 164 adds `ext Type for Trait` syntax allowing external trait implementations like `ext Point for Display`. Plan 165 introduces struct destructuring in `is` match arms, supporting patterns like `Point { x, y }`. Plan 166 emits `#[with(T as Trait)]` annotations as Rust generic constraints `<T: Trait>`.

### Type Conversion: .as() vs .to()

AutoLang distinguishes between two type conversion mechanisms. The `.as(Type)` method (Plan 161, implemented) performs zero-overhead reinterpretation casts: `x.as(u32)` produces `(x as u32)` in Rust and `((unsigned int)(x))` in C. The `.to(Type)` method (Plan 162, planned) performs lossy conversions with potential allocation: `x.to(str)` produces `x.to_string()` in Rust, `x.to(int)` produces `x.parse::<i32>().unwrap()`, and numeric-to-numeric conversions degrade to `as` casts. Both use the same parser interception pattern in the Pratt parser and `dot_item()` function, but differ semantically and in their AST representation (`Expr::Cast` vs `Expr::To`).

### Test Suite Reorganization

Plans 170, 171, and 172 reorganized all transpiler test suites from chaotic flat-numbered directories into categorized structures. The a2r suite went from ~60 active tests with 35 orphaned directories and 14 number conflicts to 144 tests across 17 categorized directories (01_basics through 17_autocode). The a2c suite consolidated 239 directories (88 orphaned, 26 stubs, ~50 number conflicts) into ~90 tests across 25 categories. The a2ts suite organized 24 tests into 10 categories aligned with the a2r/a2c numbering scheme. All suites use a consistent `NN_category/NNN_test_name` directory format with individual `#[test]` functions for VSCode test discovery.

Each reorganization also cleaned up legacy artifacts: inline tests were converted to file-based tests, stale `.wrong.*` output files were removed, redundant enum smoke tests were consolidated from ~24 to 3 representatives, and `may_*` tests were replaced by proper Option/Result equivalents.

### Platform-Specific Files and Annotations

Plan 083 introduces `.rs.at` platform-specific implementation files, following the established pattern where `.c.at` files provide C-specific implementations and `.vm.at` files provide VM-specific implementations. The `#[rs]` annotation functions as a target selector controlling the `should_skip` logic: unannotated functions are generated for all backends, while `#[c]`, `#[vm]`, and `#[rs]` restrict generation to specific targets. Files like `stdlib/auto/list.rs.at` provide Rust-specific `ext List` methods, and `stdlib/auto/storage.rs.at` provides `ext Heap` storage implementations using `use.rust` for direct Rust standard library imports.

### UI Backend Generators

The transpiler system extends beyond language targets to UI-specific code generation. Plan 174 proposes a `ui-headless` feature flag for building without GPUI/ICED dependencies, using a no-op renderer that builds VTree in memory for testing. Plan 175 describes migrating the standalone `auto-ui` project (with GPUI and ICED backend runners, 20+ examples, and transpiler APIs) into the auto-lang workspace as feature-gated modules. Plan 180 wires the existing `RustGenerator` into `auto gen` for generating Rust UI examples with typed Tailwind-like style methods on `ViewBuilder`. Plan 181 generates complete VSCode extension projects from AURA widgets, delegating UI rendering to the existing a2vue generator and wrapping output in an extension scaffold with webview panels and IPC messaging. Plan 187 replaces the Vue generator's inline JavaScript transpilation with delegation to the a2ts transpiler, enabling TypeScript output with proper type annotations for handler bodies.

## Open Questions

- How to handle AutoLang's ownership model in the a2r transpiler: start with `Rc<T>` for shared references and optimize to lifetimes later, or emit lifetimes from the start?
- Whether `.to()` should become a keyword (blocking use as a variable name) or remain a method keyword that only activates in `.to(Type)` patterns.
- Whether the a2vscode generator should support multi-panel views and per-widget placement via annotations in its initial implementation.
- How to ensure AURA rewrites (StateRef, API calls) propagate through all nested expression levels when delegating from the Vue adapter to the a2ts transpiler.

## Source Plans

Plans 007, 022, 023, 062, 067, 083, 100, 161, 162, 163, 164, 165, 166, 170, 171, 172, 173, 174, 175, 180, 181, 187, 204, 213, 215, 216.

- [204-a2r-transpiler-completeness.md](../plans/204-a2r-transpiler-completeness.md)
- [213-a2py-maturation.md](../plans/213-a2py-maturation.md)
- [215-a2ts-maturation.md](../plans/215-a2ts-maturation.md)
- [216-cffi-bindgen.md](../plans/216-cffi-bindgen.md)

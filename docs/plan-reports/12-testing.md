# 12 - Testing Infrastructure

## Overview

AutoLang's testing infrastructure evolved from inline Rust unit tests embedded in source files into a comprehensive, file-based testing framework spanning the VM evaluator, three transpiler backends (a2r, a2c, a2ts), and the AutoDown document processor. The most significant architectural shift was the migration from monolithic inline test files toward categorized directory structures where each test case is a standalone `.at` source file paired with expected output files. This transformation touched over 400 test cases across the codebase and established a consistent organizational pattern that makes tests discoverable, maintainable, and extensible.

## Plan Index

| Plan | Title | Status | Summary |
|------|-------|--------|---------|
| 110 | AutoDown Comprehensive Test Suite | Partial | Establish ~88 tests for AutoDown covering lexer, parser, transpilers, math, and edge cases |
| 158 | Fix Test Regressions (270 Failures) | Done | Resolve all 270 failing tests from unified enum, Box<Node>, and parser changes |
| 170 | A2R Test Suite Reorganization | Done | Reorganize a2r tests from flat numbering into 17 categorized directories; 144 tests |
| 171 | A2C Test Suite Reorganization | Done | Reorganize a2c tests from 239 directories into 25 categorized directories; 106 tests |
| 172 | A2TS Test Suite Reorganization | Done | Reorganize a2ts tests into 10 categorized directories aligned with a2r/a2c; 24 tests |
| 179 | Migrate vm_tests.rs to File-Based Tests | Partial | Migrate ~130 inline VM tests to file-based `.at` tests across 16 category directories |
| 191 | Assert and Precise Linker Errors | Planned | Add assert/assert_eq/assert_ne intrinsics and propagate source positions into linker errors |

## Status

**Implemented:** All three transpiler test suites (a2r, a2c, a2ts) have been reorganized from chaotic flat-numbered directories into consistent categorized structures. The massive 270-test regression from the unified enum and parser refactoring has been fully resolved, bringing the codebase from 270 failures to zero across approximately 2,533 tests. The a2r suite expanded from ~60 active tests to 144 by recovering previously ignored tests and converting inline string-literal tests into file-based ones. The a2c suite was trimmed from 239 directories (88 orphaned, 21 redundant enum smoke tests, 3 duplicate question tests) down to roughly 90 well-organized test cases across 25 categories.

**Partial:** The VM file-based test migration (Plan 179) has established the directory structure and framework across 16 categories, with roughly 167 file-based tests created. The inline `vm_tests.rs` file has been slimmed down but still retains tests that require direct bytecode construction, AST inspection, or config-mode parsing. The AutoDown test suite (Plan 110) has its directory structure and test runner pattern defined but needs completion of the full 88-test suite covering lexer, parser, transpiler snapshots, math, and error recovery phases.

**Planned:** Plan 191 proposes adding `assert`, `assert_eq`, and `assert_ne` as native VM intrinsics (IDs 4-6) with shim functions that panic on failure, and propagating source positions from AST `Call` nodes through `RelocEntry` into a structured `LinkError` type so that "Undefined symbol" errors point to the exact call site rather than falling back to heuristic text search.

## Design

### Transpiler Test Suite Architecture

The three transpiler backends -- a2r (Auto to Rust), a2c (Auto to C), and a2ts (Auto to TypeScript) -- share a unified test organization pattern established through Plans 170, 171, and 172. Before reorganization, each suite suffered from the same problems: flat sequential numbering with huge gaps (000-999 with many missing), number collisions where different features shared the same prefix, orphaned directories left over from renumbering passes, and stale `.wrong.*` output files accumulating without cleanup.

The reorganized structure uses two-level directory hierarchies: category directories like `01_basics/`, `02_types/`, `03_control_flow/` contain numbered test subdirectories like `001_hello/`, `002_sqrt/`. Each test case consists of an `input.at` source file paired with expected output files -- `.expected.rs` for a2r, `.expected.c` and `.expected.h` for a2c, `.expected.ts` for a2ts. Categories 01 through 13 are shared across all three transpilers, while higher numbers capture backend-specific features: a2c has categories 18-25 for C interop, option types, storage strategies, iterators, stdlib, and type checking; a2ts has category 18 for TypeScript-specific interop.

The test runner pattern is consistent across backends. A helper function like `test_a2r(case)` accepts a `"category/NNN_name"` path, reads the `.at` source and `.expected.*` files, invokes the transpiler, and compares output. On mismatch, a `.wrong.*` file is written for manual review. Each test case gets an individual `#[test]` function for VSCode test discovery, named with the pattern `test_{category}_{NNN}_{name}`. The a2r suite has a special multi-file test (`14_modules/005_multi_file`) that verifies project-level transpilation produces the correct file structure including `Cargo.toml`, `main.rs`, and nested module files.

The migration was executed via Python scripts that encoded the full old-to-new mapping tables, performed bulk file moves, and cleaned up orphaned directories. The a2c cleanup was the most aggressive, removing 88 orphaned directories, 5 `may_*` tests replaced by Option/Result, approximately 21 redundant two-variant enum smoke tests (keeping only 3 representatives), and 3 duplicate `question_*` tests.

### VM File-Based Testing Framework

Plan 179 defined the migration of the monolithic `vm_tests.rs` file -- which had grown to 3,048 lines of inline Rust test code -- into a file-based testing framework. The key insight is that many inline tests simply call `run(code)` and assert on the result string, making them natural candidates for standalone `.at` source files with `.expected.result`, `.expected.out`, or `.expected.error` counterparts.

The file-based test directory structure mirrors the transpiler pattern with 16 categories: `01_basics` through `16_option_result`. Each test directory contains an `input.at` file and an expected output file. The `.expected.result` suffix captures the VM's return value, `.expected.out` captures stdout from `print()` calls, and `.expected.error` captures expected runtime errors. This covers the vast majority of inline tests.

Certain tests cannot be migrated because they depend on capabilities exclusive to inline Rust code. Direct bytecode tests (`test_vm_ret_constant`, `test_vm_const_i32_add`) construct VM instructions without going through the parser. AST inspection tests verify the parser's internal data structures. Config-mode tests use `AutoConfig::new()`. Node syntax tests exercise parser-level features not yet fully implemented. AtomReader tests validate a separate subsystem. These remain in `vm_tests.rs`, which was slimmed from 3,048 lines to approximately 11 retained test functions.

The deduplication strategy is noteworthy: the inline test file contained `*_main` variants that wrapped identical logic inside `fn main() int { ... }` blocks. Since the `run()` helper handles both script mode and main-function mode, only the shorter script-mode version was kept in the file-based migration, reducing the total from approximately 130 inline tests to around 100 unique file-based tests.

### Regression Fix Methodology (Plan 158)

Plan 158 documents the systematic resolution of 270 test failures introduced by five consecutive commits that changed the unified enum system, `Box<Node>` dereferencing, parser output format, C transpiler type tracking, and added debug print statements. The fix was organized into five phases prioritized by difficulty and impact.

Phase 1 addressed "easy wins" -- 47 failures from UI generator indentation changes (resolved by accepting new formatting), `dstr`-to-`String` type migration (requiring 10 new native function implementations from IDs 177-186), parser AST format changes (adding `(mode view)` to parameters, renaming `path` to `module_path`), and resolver error message format updates. Phase 2 tackled type system issues, primarily the C transpiler's `infer_expr_type()` function missing cases for `Meta::Type` constructors and the enum pattern matching code generation producing redundant type conversions. The A2R transpiler was fully fixed by adding `Expr::Dot` handling with method name mapping (e.g., `append` to `push_str`, `length` to `len`) and proper `var`/`let mut` semantics including mutable borrow tracking.

Phases 3 and 4 addressed VM runtime features: implementing missing operations like binary `Mod`, dynamic function calls, and `List.capacity` registration; enforcing inline storage capacity limits for `ListData::push()` and `ListData::insert()`; and fixing AutoVM REPL history path handling and variable persistence. Phase 5 removed approximately 20 debug `eprintln!` statements from the C transpiler.

The most subtle bugs discovered during regression fixing involved native ID registration mismatches where `HashMap` native IDs differed between the dynamic registry and hardcoded shim constants, and a stack corruption issue where `SET_ELEM` opcodes for array element assignment did not set `last_expr_type = Void`, causing the code generator to emit a spurious `POP` instruction that corrupted the local variable frame.

### AutoDown Test Suite Design

Plan 110 outlines a comprehensive test suite for the AutoDown document processor, which at 4,273 lines of implementation had only broken tests with syntax errors. The proposed suite uses a hybrid approach: inline unit tests for lexer and parser components (approximately 25 lexer tests and 30 parser tests), and snapshot-based integration tests for the Typst and HTML transpilers (11 test cases covering basic documents, headers, lists, math, control flow, interpolation, components, tables, code blocks, edge cases, and error recovery).

The test runner follows the established snapshot pattern: a `test_autodown(name)` function reads a `.ad` input file, parses it, transpiles to both Typst and HTML, and compares against `.expected.typ` and `.expected.html` files. Additional inline tests cover math parser functions (sum, prod, integral, sqrt, trigonometric) and operators (superscript, subscript, fraction, implicit multiplication), plus error handling for unclosed math blocks, missing braces, and invalid escape sequences. The target is approximately 88 tests across all phases.

### Assert Intrinsics and Linker Error Precision (Plan 191)

The planned assert system (Plan 191) introduces `assert`, `assert_eq`, and `assert_ne` as native intrinsics registered with IDs 4-6 in the VM. The shim functions operate on the task stack: `assert` pops a single condition value, `assert_eq` and `assert_ne` pop two values for comparison. On failure, they return `VMError::RuntimeError` with a descriptive message. These are registered as intrinsics in the code generator alongside `print`, and marked as void-returning to prevent the code generator from emitting cleanup instructions for their non-existent return values. The plan acknowledges that the ideal long-term solution uses comptime evaluation (`#{file}`, `#{line}`) to inject source locations, but since comptime is not yet implemented, the native intrinsic approach serves as a practical starting point.

The linker error precision improvement addresses the problem where "Undefined symbol" errors point to the wrong source location -- often a doc comment at the top of the file instead of the actual call site. The root cause is that AST `Call` nodes do not carry source positions, so the linker has no information about where the call originated. The fix adds an `Optional<Pos>` field to the `Call` struct, propagates this through `RelocEntry` into a new `LinkError` type that carries the symbol name, module name, and source position. The error consumer in `lib.rs` uses the structured error's position to generate a precise span, falling back to the existing heuristic text search only when no position is available.

## Open Questions

- Whether `assert` should be implemented as native intrinsics now or deferred until comptime evaluation is available to provide richer error messages with file and line information.
- Whether the remaining inline VM tests that require AST inspection or direct bytecode construction should eventually gain file-based counterparts through enhanced test infrastructure.
- The AutoDown test suite target of 88 tests is ambitious; the phased rollout may need prioritization given that the implementation is currently only partially complete.

## Source Plans

- `docs/plans/110-autodown-comprehensive-tests.md`
- `docs/plans/158-fix-test-regressions.md`
- `docs/plans/170-a2r-test-reorganization.md`
- `docs/plans/171-a2c-test-reorganization.md`
- `docs/plans/172-a2ts-test-reorganization.md`
- `docs/plans/179-migrate-vm-tests-to-file-based.md`
- `docs/plans/191-assert-and-precise-linker-errors.md`

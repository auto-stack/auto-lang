# 12 - Testing

## Overview
AutoLang's testing infrastructure matured from inline Rust unit tests to a comprehensive file-based test framework covering the VM, transpilers (a2r, a2c, a2ts), and the AutoDown document processor. Major reorganization efforts brought chaotic test suites into categorized, discoverable directory structures, while regression fixes stabilized the codebase across rapid feature development.

## Plan Summary

| Plan | Title | Status | Summary |
|------|-------|--------|---------|
| 110 | AutoDown Comprehensive Test Suite | 🔧 | Establish test suite for AutoDown covering lexer, parser, transpilers, math, and edge cases |
| 158 | Fix Test Regressions (270 Failures) | ✅ | Fix all 270 failing tests introduced by unified enum, Box<Node>, and parser changes |
| 170 | A2R Test Suite Reorganization | ✅ | Reorganize a2r tests from chaotic numbering into categorized directory structure; 144 tests passing |
| 171 | A2C Test Suite Reorganization | ✅ | Reorganize a2c tests from 239 directories into categorized structure; 106 tests passing, orphans removed |
| 172 | A2TS Test Suite Reorganization | ✅ | Reorganize a2ts tests into categorized structure aligned with a2r/a2c; 24 tests passing |
| 179 | Migrate vm_tests.rs to File-Based vm_file Tests | 🔧 | Migrate ~130 inline VM tests to file-based .at test files; ~167 file-based tests, vm_tests.rs slimmed |
| 191 | Assert and Precise Linker Errors | ✅ | Add assert/assert_eq/assert_ne intrinsics and propagate source positions into linker error spans |
| 199 | VM Interactive Debugger | ✅ | SOURCE_LINE opcodes, call stack, disassembler, GDB-style debugger, AI agent debug API |
| 209 | ac-examples Modernization | ✅ | 33/33 examples pass; Phase 0 complete |
| 210 | Book Listing Test Harness | ✅ | Auto-discovery test harness for 1136 code listings |
| 211 | Stdlib Test Coverage 80%+ | ✅ | VM + a2r tests for all stdlib modules (~60 new tests) |
| 260 | Auto Test Framework | ✅ | `auto test` CLI command, discovers and runs `#[test]` functions in .at files |
| 262 | File-Based Test Framework | ✅ | Auto-discover VM file-based tests in `test/vm/` (~427 tests) |
| 261 | Migrate Rust Tests to Auto `#[test]` | ✅ | 8 test files migrated to `tests/auto/` (dstr, infer, list, field_access, memory, etc.) |
| 263 | A2R Transpiler Test Runner | ✅ | Declarative test discovery via `tests/*.at` for a2r, vm, a2c, a2ts (~900 tests) |

## Status Summary
- Completed: 8 | Partial: 3 | Planned: 3 | Deprecated: 0

## Key Achievements
- All three transpiler test suites (a2r, a2c, a2ts) reorganized into consistent categorized directory structures with clear numbering
- 270 test regressions from unified enum and parser refactoring resolved to 0 failures across 2,533+ tests
- VM test migration from 3,048-line inline test file to ~167 file-based .at tests with expected output comparison

## Remaining Work
- Complete AutoDown test suite with snapshot tests for Typst, HTML, and edge-case coverage
- Finish VM file-based test migration for remaining inline tests that require AST inspection or bytecode-level verification
- Add assert/assert_eq/assert_ne as native intrinsics and improve linker error spans to point to exact call sites (Plan 191)
- Plan 209: Rewrite 33 ac-examples using modern language features — DONE (33/33 pass)
- Plan 210: Build auto-discovery test harness for 1136 book code listings
- Plan 211: Add ~60 stdlib tests to reach 80%+ coverage

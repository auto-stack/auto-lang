# 06 - Transpilers

## Overview
AutoLang supports multiple transpiler backends (a2c, a2r, a2ts, a2p, a2j) for cross-platform code generation, plus a reverse r2a transpiler for importing Rust code. Test suites for each backend have been reorganized into categorized directory structures. UI-specific generators produce Vue, ArkTS, Tauri, and VSCode extension output.

## Plan Summary

| Plan | Title | Status | Summary |
|------|-------|--------|---------|
| 007 | Implement a2r Transpiler | 🔧 | Auto-to-Rust transpiler following a2c architecture patterns; basic phase 1 complete |
| 022 | Python Transpiler (a2p) | ✅ | Complete 10-phase implementation: expressions, control flow, functions, pattern matching, classes |
| 023 | JavaScript Transpiler (a2j) | ✅ | Complete 11-phase implementation: all JS features in single phase, 9/9 tests passing |
| 062 | C Transpiler Generics | ✅ | Monomorphization for a2c: type specialization, array tests, miette error messages (127 tests) |
| 067 | Strengthen Rust Transpiler | ⏳ | Gap analysis to bring a2r to feature parity with a2c (34 vs 161 tests) |
| 083 | a2r with .rs.at and #[rs] | 🔧 | Platform-specific Rust implementation files and #[rs] annotation support |
| 100 | a2js to a2ts Migration | 🔧 | Upgrade JavaScript generator to TypeScript with ArkTS variant support |
| 161 | a2r List + Auto Features | ✅ | #[rs] target selector, .as(Type) cast, and a2r List<T> support |
| 162 | .to(Type) Method Keyword | ⏳ | Explicit type conversion method keyword complementing .as(Type) reinterpret cast |
| 163 | a2r Core Struct Support | ✅ | 5 core struct features: static fn, nested fields, enum tag values, Option/Result, user attrs |
| 164 | a2r ext for Trait | ⏳ | External trait implementation via ext Type for Trait syntax in a2r |
| 165 | Struct Destructuring in is | ⏳ | Rust-style {field1, field2} struct destructuring in is match arms |
| 166 | a2r Generic Constraints | ⏳ | Emit #[with(T as Trait)] annotations as <T: Trait> in Rust output |
| 170 | a2r Test Reorganization | ✅ | Reorganized ~60 a2r tests into categorized structure, 144 tests passing |
| 171 | a2c Test Reorganization | ✅ | Reorganized 239 a2c test directories into categorized structure, 106 tests passing |
| 172 | a2ts Test Reorganization | ✅ | Reorganized 24 a2ts tests into categorized structure, all passing |
| 173 | r2a Rust-to-Auto Transpiler | ✅ | Reverse transpiler: Rust to AutoLang via syn crate, 116 tests across 4 phases |
| 174 | Conditional UI Backends | ⏳ | ui-headless feature flag for UI-less builds, skipping GPUI/ICED dependencies |
| 175 | Migrate auto-ui into auto-lang | ⏳ | Move GPUI and ICED backends from standalone auto-ui into auto-lang workspace |
| 180 | a2rust-ui Generator | ⏳ | Wire RustGenerator into auto gen for Rust UI backend (GPUI examples) |
| 181 | a2vscode Generator | ⏳ | VSCode extension generator from AURA widgets using a2vue + webview panel |
| 187 | a2ts Vue Adapter | ⏳ | Replace Vue generator's inline JS with a2ts delegation for proper TypeScript output |

## Status Summary
- Completed: 10 | Partial: 3 | Planned: 9 | Deprecated: 0

## Key Achievements
- Complete transpiler suite: a2c (106 tests), a2r (144 tests), a2ts (24 tests), a2p (10 tests), a2j (9 tests)
- Reverse r2a transpiler with 116 tests for Rust-to-AutoLang code import
- C transpiler monomorphization enabling generic type specialization
- Test suite reorganization across all backends with categorized directory structures

## Remaining Work
- Close a2r feature gap with a2c (advanced data types, closures, async patterns)
- Complete a2ts migration from a2js with full TypeScript type annotations
- Implement UI backend generators (a2rust-ui, a2vscode) and migrate auto-ui
- Add struct destructuring, generic constraints, and external trait impl to a2r

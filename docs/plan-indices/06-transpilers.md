# 06 - Transpilers

## Overview
AutoLang supports multiple transpiler backends (a2c, a2r, a2ts, a2p, a2j) for cross-platform code generation, plus a reverse r2a transpiler for importing Rust code. Test suites for each backend have been reorganized into categorized directory structures. UI-specific generators produce Vue, ArkTS, Tauri, and VSCode extension output.

## Plan Summary

| Plan | Title | Status | Summary |
|------|-------|--------|---------|
| 007 | Implement a2r Transpiler | 🔧 | Auto-to-Rust transpiler following a2c architecture patterns; basic phase 1 complete |
| 417 | “Auto 作 Rust 脚本层”发布收尾（359 residuals） | ✅ | 全部完成:Phase E 五项(E1-E5)+D2/D3+双 demo+A1/A2 落地页+359 回填(2026-08-22 收官);附带修复 a2r 双重 await,登记 DIV-A2R-STRPARAM-1 三库回归 |
| 427 | a2r 字符串形参借用回归修复（DIV-A2R-STRPARAM-1） | ✅ | 修复 3f6aa1be(396 §2.4)引入的 is_str_slice_var 误判(str_slice_pattern_bindings 专属集合);serde_json/url/base64 三库恢复 100%,L1 回到 260 例/10 库;golden 008+rustc 冒烟防线 |
| 359 | Auto as Rust's Script Layer 全程落地 | ✅ | 三向 parity 体系+生态用例库+Script-to-Ship tour+落地页;165 checkbox 工件级核验后回填(2026-08-22,417-final 批收官);后续 base64/url/serde_json a2r 回归见 DIV-A2R-STRPARAM-1 |
| 415 | a2r 剩余大件拆粒度（242 收尾批） | 📋 | HashMap::from 发射/SQLite→Redis stdlib/GPUI spike/自举/dep cc+memmap2 五子项;2026-08-22 立项 |
| 022 | Python Transpiler (a2p) | ✅ | Complete 10-phase implementation: expressions, control flow, functions, pattern matching, classes |
| 023 | JavaScript Transpiler (a2j) | ✅ | Complete 11-phase implementation: all JS features in single phase, 9/9 tests passing |
| 062 | C Transpiler Generics | ✅ | Monomorphization for a2c: type specialization, array tests, miette error messages (127 tests) |
| 067 | Strengthen Rust Transpiler | 🔧 | Gap analysis to bring a2r to feature parity with a2c (34 vs 161 tests) |
| 083 | a2r with .rs.at and #[rs] | 🔧 | Platform-specific Rust implementation files and #[rs] annotation support |
| 100 | a2js to a2ts Migration | 🔧 | Upgrade JavaScript generator to TypeScript with ArkTS variant support |
| 161 | a2r List + Auto Features | ✅ | #[rs] target selector, .as(Type) cast, and a2r List<T> support |
| 162 | .to(Type) Method Keyword | ✅ | Explicit type conversion method keyword（Expr::To ast.rs:507 + rust.rs:3509 + golden 002_to_convert，2026-08-20 核实） |
| 163 | a2r Core Struct Support | ✅ | 5 core struct features: static fn, nested fields, enum tag values, Option/Result, user attrs |
| 164 | a2r ext for Trait | ✅ | External trait implementation via ext Type for Trait syntax（parser.rs:4749 可选 TraitName + 4697 + rust.rs:1580，2026-08-20 核实） |
| 165 | Struct Destructuring in is | ✅ | Rust-style {field1, field2} struct destructuring in is match arms（ast.rs:372 StructPattern + golden 002_struct_destructure，2026-08-20 核实） |
| 166 | a2r Generic Constraints | ✅ | Emit #[with(T as Trait)] as <T: Trait>（Plan 166 fn 级 + Plan 364 W3 多 bound type/impl 级，2026-08-20 核实） |
| 170 | a2r Test Reorganization | ✅ | Reorganized ~60 a2r tests into categorized structure, 144 tests passing |
| 171 | a2c Test Reorganization | ✅ | Reorganized 239 a2c test directories into categorized structure, 106 tests passing |
| 172 | a2ts Test Reorganization | ✅ | Reorganized 24 a2ts tests into categorized structure, all passing |
| 173 | r2a Rust-to-Auto Transpiler | ✅ | Reverse transpiler: Rust to AutoLang via syn crate, 116 tests across 4 phases |
| 174 | Conditional UI Backends | ⏳ | ui-headless feature flag for UI-less builds, skipping GPUI/ICED dependencies |
| 175 | Migrate auto-ui into auto-lang | ⏳ | Move GPUI and ICED backends from standalone auto-ui into auto-lang workspace |
| 180 | a2rust-ui Generator | ⏳ | Wire RustGenerator into auto gen for Rust UI backend (GPUI examples) |
| 181 | a2vscode Generator | ⏳ | VSCode extension generator from AURA widgets using a2vue + webview panel |
| 187 | a2ts Vue Adapter | ⏳ | Replace Vue generator's inline JS with a2ts delegation for proper TypeScript output |
| 204 | a2r Transpiler Completeness | ✅ | All 6 phases: Result, spec, struct, enum, stdlib method mapping, safe output |
| 213 | a2py Python Transpiler Maturation | ✅ | Expand Python transpiler from 18% to 80%+ coverage (Option/Result, closures, generics) |
| 283 | a2py Pythonic Maturation | ✅ | Import system, stdlib/builtin/method mapping, @dataclass/@staticmethod, struct destructuring, type tracking |
| 215 | a2ts TypeScript Transpiler Maturation | ✅ | Expand TypeScript transpiler from 24 to 80+ tests (Option/Result, collections, async) |
| 216 | C FFI Bindgen | ✅ | Auto-bindgen for C headers with libloading runtime, a2c auto-bind, CLI integration |
| 219 | Playground Source Map | ✅ | Source mapping for transpiler output to enable clickable error locations |
| 220 | a2r Transpiler Improvement | ✅ | Rust transpiler improvements: better type mapping, enum support, stdlib coverage |
| 223 | a2r Step-00 Transpiler Fixes | ✅ | Lexer pos drift fix, multi-arg enum variants, is-expression, return/break in match arms |
| 232 | a2r Lexer Compilation | ✅ | `.sub()`/`.slice()` handler + post_process() for types |
| 240 | Rust Cookbook a2r Tests | ✅ | Systematic a2r test suite — 163 .at files, 124/124 pass, all assert-based; DB/async/cc stubs handed off to Plan 242 #10/#12/#17 |
| 241 | a2r String Type Cleanup | ✅ | Fix get_or/insert .to_string() heuristics, return newline support, not-in-if |
| 264 | a2r Dot to Double Colon | ✅ | module_types mapping, qualify_type_name(), use stmt path handling for `.` → `::` |
| 308 | Godot Demo Reverse Translation | ✅ | 4 个官方 Godot demo 逆向翻译为 a2gd 回归测试：6 fixture（test/a2gd/tscn/godot_demos/）+ 11 测试函数（gdscript.rs 5 + tscn.rs 6），5 条 documented gaps 留档 |
| 364 | a2r COSMIC Replication Readiness | ✅ | W1-W7 全落地：dotted 注解/Fn.attrs/multi-bound `T: A + B`/move 闭包/`~{}` 全语句/~Stream parity/path 依赖 + Phase 8 F1-F3（Try 降级与 F4 deferred 已登记债务簿） |
| 391 | a2r Parity Debt from Musk | ✅ | D1-D6 六项 a2r 限制修复（.len() cast、Option<&T> 标注、List<str> split、:: 路径、() 类型、trait impl 错误）+ §8 多段路径 codegen（std::env::var 用 ::） |
| 392 | a2r Codegen Fixes from Musk | ✅ | E4 sort_by + E5 HashMap.get 误归因（E1/E2/E3 移交 Plan 393） |
| 393 | a2r Method Dispatch Fixes | ✅ | E1 .append 过宽、E2 Ok(None) 误改、E3 HashMap::insert 漏分号 |
| 395 | Turbofish Generic Call Args | ✅ | 调用泛型实参 `method<Type>(args)` / `fn<Type>(args)` → Rust `::<T>`，AST 字段 + parser 回溯 + 3 发射点 + golden |
| 396 | a2r 改进（auto-ai 滚动聚合） | ✅ | §2.1-§2.6 六条根因全根治：借用推理 B/C/D/E + 三段限定 unit-variant 模式剥模块段 + a2r_std time i64 对齐；auto-ai 三转译 crate 首次同时全绿，§2 范围 sed 全部毕业（golden 340/340） |
| 397 | Spec Supertrait + Arc<Fn> Spec-Param | ✅ | `pub spec Tool: Send + Sync` → `trait Tool: Send + Sync`，Arc<Fn> spec-param golden 确认 |

## Status Summary
- Completed: 30 | Partial: 3 | Planned: 9 | Deprecated: 0

## Key Achievements
- Complete transpiler suite: a2c (106 tests), a2r (144 tests), a2ts (24 tests), a2p (96 tests), a2j (9 tests)
- Reverse r2a transpiler with 116 tests for Rust-to-AutoLang code import
- C transpiler monomorphization enabling generic type specialization
- Test suite reorganization across all backends with categorized directory structures

## Remaining Work
- Complete a2ts migration with TypeScript type annotations and expanded test coverage (Plan 215)
- Implement UI backend generators (a2rust-ui, a2vscode) and migrate auto-ui
- a2r remaining gaps tracked in Plan 242 (HashMap::from, closure inference, Redis/SQLite backend, GPUI, self-hosting, dep cc)

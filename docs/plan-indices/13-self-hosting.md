# 13 - Self-Hosting Compiler & Metaprogramming

## Overview
Plans covering the path to a self-hosted AutoLang compiler, including generic types, pattern matching, trait system completion, bootstrap strategy, expression/array support, and compile-time execution. These form the critical backbone for AutoLang to compile itself.

## Plan Summary

| Plan | Title | Status | Summary |
|------|-------|--------|---------|
| 028 | Generic Types and Monomorphization | ⏳ | Full generics with monomorphization for type-safe containers and algorithms |
| 029 | Pattern Matching System | ⏳ | Comprehensive pattern matching extending `is` statement with structs, enums, guards |
| 030 | Trait System Completion | ⏳ | Full trait/polymorphism system with generic traits, associated types, dynamic dispatch |
| 031 | Bootstrap Strategy | ⏳ | Three-stage bootstrap to resolve compiler-stdlib circular dependency |
| 033 | Self-Hosting Compiler | ⏳ | AutoLang compiler written in AutoLang targeting C via a2c transpiler |
| 037 | Expression and Array Support | ✅ | Complex expressions, array indexing, array return types -- fully implemented |
| 095 | Compile-Time Execution Engine (CTEE) | ✅ | `#if`, `#for`, `#is`, `#{}` comptime constructs using embedded AutoVM |
| 229 | Self-Hosting via a2r | ✅ Complete | Auto 自举编译器 — All phases complete: Phase 1-4 (token+lexer+parser+eval+typeinfer+codegen+BVM+a2r+self-hosting+bootstrap verification), 235 tests, bootstrap.exe passes self-test |
| 233 | AAVM Parser (P0+P1) | ✅ | tokenize_list() + Pratt parser + 37 tests |
| 234-P1 | AAVM Parser P1 | ✅ | 10 features: closure/fstr/is/enum/use/ext/spec/alias/object |
| 236 | AAVM Evaluator | ✅ | Tree-walking eval + AST restructuring + 16 tests |
| 237 | AAVM Architecture Gap Closure | ✅ | Phase A-E complete (value encoding, type inference, bytecode compiler, BVM string/map/list ops, a2r transpiler) |
| 239 | AAVM List/Map Bytecode | ✅ | BVM heap + 8 opcodes (LIST_NEW/PUSH/GET/LEN, MAP_*) |
| 432 | AAVM v2 核心移植 | ✅ | 六层管线(token/lexer/parser/typeinfo/codegen/engine)纯 Rust 模式 Auto 移植,AutoVM 内自举跑通 helloworld+fib(10)(M3 主里程碑);M1-M5 六道闸门全绿(corpus_m4 30 文件:bootstrap 038-052 回收 + 数组四件套);divergences 22 处定稿,lib-legacy 封存;附带宿主修复 D26 字符串池 RC/D26 堆侧 UAF/D30 负 int 哨兵池界 |
| 433 | a2r 闭环 | ✅ | AAVM v2 经 Rust 版 a2r 转译为纯 Rust(零 a2r_std,rustc metadata 329→0 错);AAVM-Rust 二进制 corpus_m4 30/30 与 Rust 参考一致(0.1-2ms/例);parity 四向矩阵(①参考②aavm_rust③aavm_vm④golden)全绿一条命令可复现;自举回路演示落档;242 #16/415-D 收口(AA2R 五向 → 434);a2r 修复 12 项 + .at 改写六文件(divergences D32-D37);VM 枚举载荷传参缺陷挂 242 |

## Status Summary
- Completed: 9 | Partial: 0 | Planned: 5 | Deprecated: 0

## Key Achievements
- Plan 037 completed in ~1 week (vs 6-10 week estimate) after discovering most features already worked
- Plan 095 CTEE fully implemented with lexer tokens, AST nodes, parser support, and VmInterpreter-based evaluation
- Compile-time `#if`/`#for`/`#is`/`#{}` constructs enable conditional compilation and metaprogramming
- Plan 229 fully complete: Auto self-hosting compiler via a2r — all 4 phases done, bootstrap.exe compiles and passes self-test (run_eval + run_a2r)

## Remaining Work
- Plans 028-031 form a deep dependency chain blocking full self-hosting (estimated 30-50 weeks combined)
- Generic types and monomorphization (Plan 028) is the critical first step, needed by Plans 029, 030, and 033
- Bootstrap strategy (Plan 031) must resolve the compiler-stdlib chicken-and-egg problem before Plan 033 can begin
- AAVM bootstrap test suite: 74 directories covering token/lexer/parser/eval/typeinfer/bytecode/BVM ops

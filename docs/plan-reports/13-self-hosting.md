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

## Status Summary
- Completed: 2 | Partial: 0 | Planned: 5 | Deprecated: 0

## Key Achievements
- Plan 037 completed in ~1 week (vs 6-10 week estimate) after discovering most features already worked
- Plan 095 CTEE fully implemented with lexer tokens, AST nodes, parser support, and VmInterpreter-based evaluation
- Compile-time `#if`/`#for`/`#is`/`#{}` constructs enable conditional compilation and metaprogramming

## Remaining Work
- Plans 028-031 form a deep dependency chain blocking self-hosting (estimated 30-50 weeks combined)
- Generic types and monomorphization (Plan 028) is the critical first step, needed by Plans 029, 030, and 033
- Bootstrap strategy (Plan 031) must resolve the compiler-stdlib chicken-and-egg problem before Plan 033 can begin

# 13 - Self-Hosting Compiler

## Overview

Self-hosting is the capstone goal for AutoLang: a compiler written in AutoLang that can compile itself, reducing the project's dependency on the Rust implementation. This chapter covers seven plans that form the full trajectory from today's feature gaps to a bootstrapped, self-compiling toolchain. Two plans are already complete (expression/array support and the compile-time execution engine), while five remain in planning, forming a deep dependency chain estimated at 30-50 weeks of sequential work. The key technical insight is that self-hosting requires not just rewriting the compiler in AutoLang, but first equipping the language with generics, pattern matching, traits, and a bootstrap strategy to resolve the circular dependency between the compiler and its own standard library.

## Plan Index

| Plan | Title | Status | Summary |
|------|-------|--------|---------|
| 028 | Generic Types and Monomorphization | Planned | Full generics with monomorphization for type-safe containers and algorithms |
| 029 | Pattern Matching System | Planned | Struct, enum, nested, guard, and OR patterns extending the `is` statement |
| 030 | Trait System Completion | Planned | Generic traits, associated types, trait bounds, and dynamic dispatch with vtables |
| 031 | Bootstrap Strategy | Planned | Three-stage bootstrap to resolve the compiler-stdlib circular dependency |
| 033 | Self-Hosting Compiler | Planned | AutoLang compiler written in AutoLang, targeting C via a2c transpiler |
| 037 | Expression and Array Support | Complete | Complex expressions, array indexing, array return types |
| 095 | Compile-Time Execution Engine (CTEE) | Complete | `#if`, `#for`, `#is`, `#{}` comptime constructs using embedded AutoVM |
| 229 | Self-Hosting via a2r | Partial Phase 2 | Auto 自举编译器 — Phase 1 fully complete (token+lexer+parser+eval+typeinfer+codegen+vm+BVM string/map/list ops with 74 test dirs), Phase 2 (a2r transpiler in Auto) + Phase 3 (self-bootstrapping) remaining |
| 233 | AAVM Parser P0+P1 | Complete | tokenize_list() + Pratt parser + 20 P0 tests, then 10 P1 features (closure/fstr/is/enum/use/ext/spec/alias/object) for 37 total tests |
| 234-P1 | AAVM Parser P1 | Complete | 10 high-priority features: closure/fstr/is/enum/use/ext/spec/alias/object |
| 236 | AAVM Evaluator | Complete | Tree-walking evaluator + AST restructuring + 16 tests |
| 237 | AAVM Architecture Gap Closure | Partial Phase E | Phase A-D complete (value encoding, type inference, bytecode compiler, BVM string/map/list ops), Phase E (a2r transpiler in Auto) remaining |
| 239 | AAVM List/Map Bytecode | Complete | BVM heap + 8 opcodes (LIST_NEW/PUSH/GET/LEN, MAP_NEW/INSERT/GET/LEN) |

## Status

**Implemented**: Plan 037 (expression/array support, completed in ~1 week versus a 6-10 week estimate after discovering most features already worked), Plan 095 (CTEE with lexer tokens, AST nodes, parser support, and VmInterpreter-based evaluation), Plans 233, 234-P1, 236, 239 (AAVM parser, evaluator, and bytecode milestones — self-hosting Phase 1 compiler front-end complete with 74 test directories).

**Partial**: Plan 229 (Phase 1 complete, Phase 2 a2r transpiler + Phase 3 self-bootstrapping remaining), Plan 237 (Phase A-D complete, Phase E a2r transpiler remaining).

**Planned**: Plans 028-031 and 033. These form a deep dependency chain: Plan 028 (generics) is needed by Plans 029, 030, and 033; Plan 031 (bootstrap) must resolve the compiler-stdlib chicken-and-egg problem before Plan 033 can begin. Combined estimated timeline is 30-50 weeks for the planned work alone.

## Design

### Completed Foundation: Expressions, Arrays, and Compile-Time Execution

Plan 037 discovered that most of its planned features already worked -- complex expressions in for-loop ranges, array indexing with expressions like `arr[idx+1]`, and static method declarations were all functional. The one genuine gap was array return types: functions like `fn get_numbers() []int` failed at the parser. This was fixed by teaching the parser to accept `LSquare` and `Star` tokens in return-type position and updating the C transpiler to emit a pointer return with an out-parameter for size. The generated C for `fn get_numbers() []int` produces `int* get_numbers(int* out_size)` with a static array literal, and call sites declare a size variable and pass its address. All 554 tests (49 evaluator, 99 transpiler) passed after the change. The actual implementation took roughly one week, a dramatic improvement over the 6-10 week estimate, because the plan's discovery phase revealed that Phases 1, 2, and 4 needed no changes.

Plan 095 implemented the Compile-Time Execution Engine, which embeds the AutoVM inside the compiler to execute code at compile time. The engine supports four comptime constructs: `#if` for conditional compilation (e.g., platform-specific code paths), `#for` for compile-time loop unrolling (e.g., generating repetitive code), `#is` for compile-time pattern matching, and `#{}` for evaluating expressions whose results are substituted as literals. The implementation reuses the existing `VmInterpreter` rather than building a custom evaluator, which ensures all language features available at runtime are also available at compile time. Built-in constants like `OS`, `ARCH`, `DEBUG`, and `VERSION` are injected into the compile-time environment. The CTEE works as an AST transformation pass: after parsing, it evaluates all `#`-prefixed constructs, prunes false branches, unrolls loops, and substitutes computed values before the transpiler sees the code. Error reporting uses a `ComptimeError` type with codes E0401-E0406 and miette integration for source-span annotations.

### Generic Types and Monomorphization (Plan 028)

Generics are the single most critical missing feature for self-hosting. The current compiler can only express concrete types, forcing massive code duplication: a separate `IntSymbolTable`, `StrSymbolTable`, and so on for every type-keyed collection. Plan 028 introduces `type Vec<T>`, `type HashMap<K, V>`, generic functions like `fn id<T>(x T) T`, type parameter inference, and trait bounds via `where` clauses. The implementation follows four phases.

Phase 1 extends the type system with `Type::GenericVar(Name)` for type variables and `Type::App { func, args }` for generic applications like `Vec<int>`. Phase 2 adds parser support for `<T, U: Clone>` parameter lists, generic type references, and trait-bound syntax. Phase 3 implements type inference through unification: when a generic function is called with concrete arguments, the inference engine matches parameter types against argument types, collecting type-variable bindings and checking trait bounds. Phase 4 implements monomorphization -- the process of generating a specialized copy of each generic function for every concrete type combination it is used with. For example, `swap<int>` generates `swap_int(int*, int*)` and `swap<str>` generates `swap_str(str*, str*)` in the C output. A specialization cache prevents duplicate instantiations. Name mangling encodes the type arguments into the generated C function name (e.g., `HashMap_int_Symbol_get`).

The monomorphization approach mirrors Rust's strategy and produces zero-cost abstractions: the generated C code is identical to what a hand-written, type-specialized version would look like. Estimated timeline is 12-16 weeks, making it the longest single plan in the self-hosting track.

### Pattern Matching (Plan 029)

AutoLang already has a basic `is` statement for equality matching, but the self-hosting compiler needs far more: struct destructuring, enum variant matching, nested patterns, OR patterns, pattern guards, tuple matching, slice/array patterns, and compile-time exhaustiveness checking. Plan 029 designs a `Pattern` enum in the AST covering all these forms -- `Pattern::Struct`, `Pattern::EnumVariant`, `Pattern::Tuple`, `Pattern::Or`, `Pattern::Guard`, `Pattern::Slice`, and so on.

The implementation is parser-intensive. New parsing functions handle struct patterns (`Point{x, y}`), enum patterns (`Option::Some(val)`), nested patterns, and OR patterns (`1 | 2 | 3`). Type checking for patterns requires two passes: inference (determining the type of each pattern binding) and exhaustiveness checking (verifying all possible values are covered, with error codes like `auto_pattern_E0001` for non-exhaustive matches and `auto_pattern_W0001` for unreachable patterns). Code generation translates pattern matching into C if-else chains with discriminant checks for enum variants and field-access chains for struct patterns. The exhaustiveness algorithm builds a decision tree and checks coverage for all variants of enum types. Timeline is 10-14 weeks.

### Trait System Completion (Plan 030)

AutoLang has a basic `spec` keyword for trait declarations, but lacks generic type parameters on traits, associated types, trait bounds on generic functions, trait inheritance, and dynamic dispatch. Plan 030 fills all these gaps. Generic specs like `spec Iterable<T>` allow abstract iteration over any collection. Associated types like `type Item` inside a spec let traits declare output types without making them parameters. Trait bounds like `fn sort<T>(arr [T]) where T: Comparable` constrain what types a generic function accepts. Trait inheritance like `spec Writer : Reader` builds trait hierarchies. Dynamic dispatch via trait objects (`spec Reader`) generates C vtables -- structs of function pointers -- so that a `ReaderObj` contains a data pointer and a vtable pointer, and virtual calls go through `reader.vtable->read_line(reader.data)`.

The implementation has four phases: type system extensions (adding `Type::TraitObject`, `TraitRef`, `TraitBound` to the AST), trait resolution (a `TraitResolver` that maintains registries of trait declarations and implementations, with coherence checking to prevent overlapping impls), static dispatch through monomorphization (direct function calls with mangled names), and dynamic dispatch through vtable generation. Static dispatch is always preferred for performance; dynamic dispatch is opt-in when the programmer explicitly uses a trait object type. The trait system is modeled closely on Rust's, with blanket implementations (`impl<T> Reader for T`) and associated type projection. Timeline is 12-16 weeks, and it depends on Plan 028 being complete first.

### Bootstrap Strategy (Plan 031)

The central challenge of self-hosting is circular dependency: the compiler needs the standard library (HashMap, StringBuilder, String) to be written, but the standard library needs the compiler to transpile it. Plan 031 resolves this with a three-stage bootstrap process, following the same pattern used by Go, Rust, and GCC.

Stage 1 builds a minimal compiler written in AutoLang, transpiled to C by the existing Rust compiler. This minimal compiler supports only core language features: basic types, fixed arrays, structs, functions, control flow, and C FFI. It explicitly excludes HashMap, StringBuilder, ownership, generics, pattern matching, and traits. It uses C arrays for symbol tables with O(n) lookup (acceptable for small programs), C strings for all text, and manual `malloc`/`free` for memory. The Stage 1 compiler is roughly 2,000-3,000 lines of AutoLang.

Stage 2 builds the standard library components (HashMap, StringBuilder, String) in pure C, compiled with GCC into a static library `libstdlib.a`. These are C implementations with clean APIs that the Stage 1 compiler can link against.

Stage 3 builds the full-featured compiler using the Stage 1 compiler plus the stdlib. Now HashMap, StringBuilder, and String are available, enabling the full compiler to use efficient data structures. This compiler is feature-complete and can compile itself.

Stage 4 is validation: the Stage 3 compiler compiles itself, producing a Stage 4 binary that should be bit-for-bit identical (or functionally equivalent) to the Stage 3 binary. Build orchestration is handled by shell scripts (`bootstrap.sh`, `build_stage1.sh`, `build_stdlib.sh`, `validate.sh`) and an `auto-man.yaml` configuration file. Timeline for the bootstrap strategy alone is 4-6 weeks.

### Self-Hosting Compiler (Plan 033)

Plan 033 is the culmination: an AutoLang compiler written in AutoLang that targets C through the a2c transpiler. The compiler follows the standard pipeline (lexer, parser, symbol table, type checker, C transpiler), with each component implemented as an AutoLang module under `auto/`. The existing Rust implementation (4,399-line parser, 2,505-line C transpiler, 1,794-line type inference engine) serves as the reference implementation.

The plan is divided into eight phases spanning 43-62 weeks. Phase 1 builds the token system (50+ token kinds). Phase 2 implements the lexer (5-6 weeks). Phase 3 builds the hierarchical symbol table using HashMap. Phase 4 defines the full AST (expressions, statements, types). Phase 5 implements the recursive-descent parser with precedence climbing (8-10 weeks, the longest phase). Phase 6 implements type checking with unification and coercion. Phase 7 implements the C transpiler using StringBuilder for output. Phase 8 creates the compiler driver and achieves self-compilation.

The module structure places core libraries under `auto/lib/` (token.at, pos.at, error.at, ast.at, symbol.at, type_check.at) and compiler components under `auto/compiler/` (lexer.at, parser.at, transpiler.at, compiler.at). The compiler driver reads a source file, lexes, parses, type-checks, generates C, writes the C file, and optionally invokes GCC to produce an executable. Build configuration uses auto-man with a YAML file specifying sources, output, and toolchain options.

Key technical decisions include targeting C rather than Rust (simpler build, wider tooling support, alignment with Auto-Man's embedded ecosystem), using arena allocation to mitigate C's memory safety concerns, and accepting up to 10x slower performance than the Rust compiler initially. The self-hosted compiler need not match the Rust compiler's feature set immediately; it can start with a minimal viable subset and add features incrementally through self-compilation.

### Dependency Chain and Critical Path

The plans form a strict dependency chain. Plan 028 (generics) must complete before Plan 030 (traits) can begin, because traits need generic type parameters. Plan 029 (pattern matching) can proceed in parallel with Plan 028, but its code-generation phase depends on Plan 028's type system extensions. Plan 031 (bootstrap) can begin planning immediately, but its execution requires the stdlib foundation from Plan 027 (not covered in this chapter). Plan 033 (self-hosting) depends on all of the above plus Plans 024, 025, and 027 from other chapters.

The combined estimated timeline for just the planned work in this chapter is 30-50 weeks (generics: 12-16, pattern matching: 10-14, traits: 12-16, bootstrap: 4-6, self-hosting: 43-62), but significant parallelism exists. The critical path runs generics to traits to bootstrap to self-hosting, with pattern matching overlapping the early phases.

## Open Questions

1. Should the Stage 1 minimal compiler be written in AutoLang (more authentic, validates language design) or in C directly (faster to implement, less risky)?
2. Should range patterns be supported in `is` statements (e.g., `is x { 1..10 => ... }`)?
3. Should pattern matching be an expression or a statement? Rust treats it as an expression; AutoLang currently treats `is` as a statement.
4. Should higher-kinded types be supported in the trait system (e.g., `spec Functor<F>` where F is a type constructor)?
5. How should the orphan rule be enforced -- prevent impls of foreign traits for foreign types?
6. Should comptime mode in the VM block all non-deterministic operations (file I/O, randomness, time) at the FFI boundary?

## Source Plans

- Plan 028: Generic Types and Monomorphization
- Plan 029: Pattern Matching System
- Plan 030: Trait System Completion
- Plan 031: Bootstrap Strategy
- Plan 033: Self-Hosting Compiler
- Plan 037: Expression and Array Support
- Plan 095: Compile-Time Execution Engine (CTEE)
- Plan 229: Self-Hosting via a2r
- Plan 233: AAVM Parser P0+P1
- Plan 234-P1: AAVM Parser P1
- Plan 236: AAVM Evaluator
- Plan 237: AAVM Architecture Gap Closure
- Plan 239: AAVM List/Map Bytecode

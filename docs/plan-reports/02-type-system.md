# 02 - Type System

## Overview

AutoLang's type system evolved from a simple set of built-in primitives into a comprehensive system supporting parametric polymorphism, trait-based abstraction, single inheritance, closures, and generic constraints. This transformation unfolded across 14 plans spanning from early composition fixes to full generic spec support with monomorphized vtable generation. The core achievement is a type system that feels lightweight in syntax (no `->` for returns, space-separated fields) yet delivers Rust-grade capabilities including ownership-aware borrow semantics, compile-time trait conformance checking, and platform-aware storage strategies. Twelve of the fourteen plans are complete, with one partial implementation (storage injection) and one remaining design gap (type alias syntax).

## Plan Index

| Plan | Title | Status | Summary |
|------|-------|--------|---------|
| 018 | Type Composition Improvements | Partial | Fix `has` composition with method resolution; evaluator and transpiler gaps remain |
| 019 | Spec Trait System | Complete | Full `spec` keyword system with vtable generation, member-level delegation, and default methods |
| 021 | Single Inheritance | Complete | `is` keyword inheritance with flat struct layout in both C and Rust transpilers |
| 025 | String Type Redesign | Complete | 20 string functions, CStr FFI type, 37 unit tests, comprehensive documentation |
| 026 | Property Keywords | Complete | `view`/`mut`/`take` converted from prefix to postfix dot notation |
| 048 | Generic Type Definitions | Complete | Full generic type support with substitution; stdlib `May<T>` and `List<T>` working |
| 049 | May Operators to Generic Types | Complete | `?T`, `.?`, `??` operators migrated from hardcoded `Type::May` to generic `tag May<T>` |
| 055 | Storage Injection | Planned | Platform-aware storage strategy (MCU=Fixed, PC=Dynamic); infrastructure done, type aliases needed |
| 056 | Dot Expression Field Access | Complete | Dedicated `Expr::Dot` type, field read/write, move-safe field access, transpiler support |
| 057 | Generic Specs | Complete | Traits with type parameters, monomorphized vtable generation, spec conformance validation |
| 058 | Type Alias Syntax | Complete | `type X = Y` syntax with parser, evaluator, and C transpiler typedef generation |
| 059 | Generic Type Fields | Complete | Generic fields in structs, `*const T`/`*mut T` qualifiers, `impl<T>` blocks |
| 060 | Closure Syntax | Complete | JS/TS-style `x => x * 2`, variable capture, type inference, C function pointer transpilation |
| 061 | Generic Constraints | Complete | `<T: Spec>` bounds enabling Plan 051 Auto Flow iterator system |
| 191 | Assert and Precise Linker Errors | Complete | Add assert/assert_eq/assert_ne intrinsics and propagate source positions into linker error spans |
| 193 | Conv Type Conversion System | Complete | Unified `.to()` method with Conv<From, To> spec for type-safe conversions |
| 194 | Monomorphic Dispatch for Generic Methods | Complete | Compile-time type-based dispatch for HashMap/HashSet generic APIs |

## Status

**Implemented**: Plans 019, 021, 025, 026, 048, 049, 056, 057, 058, 059, 060, 061, 191, 193, 194

**Partial**: Plan 018 (`has` composition is a no-op in evaluator and transpiler), Plan 055 (infrastructure complete but type alias syntax blocks the seamless `List<T>` user experience)

**Planned**: Full polymorphic arrays with `[]Flyer` trait objects; associated types in specs; deeper variable capture semantics for closures

## Design

### Composition, Inheritance, and Delegation

AutoLang supports three distinct composition mechanisms. The `has` keyword (Plan 018) provides type-level composition where a type embeds another type's fields and methods. While parsing and C transpiler support were implemented, the evaluator still treats `has` as a no-op, meaning runtime field mixing from composed types does not work. This remains an open gap.

The `is` keyword (Plan 021) implements single inheritance using flat struct layout. A `type Dog is Animal` declaration causes the parser to copy all parent fields and methods into the child type. The C transpiler generates a flat struct containing parent fields directly (not nested), and the Rust transpiler does the same. Method overriding works naturally: child methods replace parent methods of the same name. The system validates at parse time that the parent type exists and is a user-defined type, rejecting attempts to inherit from built-in types like `int`.

The most sophisticated composition mechanism is member-level delegation (Plan 019), which uses `has member Type for Spec` syntax inside a type body. This lets a type explicitly delegate implementation of a specific spec to a named member. For example, `type Starship as Engine { has core WarpDrive for Engine }` means that when `ship.start()` is called and `start` is an `Engine` method, the call forwards to `ship.core.start()`. The method resolution order checks the type's own methods first, then walks the delegation chain in declaration order. Both C and Rust transpilers generate wrapper functions that perform this forwarding, with the C transpiler using vtable entries and the Rust transpiler generating `impl Engine for Starship { fn start(&self) { self.core.start() } }`.

### Spec Trait System

The spec system (Plan 019) is AutoLang's trait abstraction, implemented across eight phases with support from lexer through transpiler. Specs are declared with `spec Name { fn method_sig() }` and implemented with `type Name as SpecName { ... }`. The implementation spans six key modules.

At the lexer level, `spec` was added as `TokenKind::Spec`. The parser recognizes spec declarations and the `as` clause in type declarations, supporting multiple specs via comma separation (`type Foo as Spec1, Spec2`). The AST gained a `SpecDecl` node with method signatures, and later a `body` field for default method implementations.

The trait checker validates conformance at parse time: every method declared in a spec must have a corresponding implementation in the type, with matching parameter counts and return types. The evaluator registers specs in the universe scope and dispatches trait-bounded method calls through a vtable lookup. The C transpiler generates vtable structs with function pointers, creating a `TypeName_SpecName_vtable` static instance for each spec implementation. The Rust transpiler leverages native Rust traits, generating `trait Name { ... }` and `impl Name for Type { ... }` blocks.

Phase 8.5 added default method bodies to specs, enabling forwarding patterns like `list.map()` automatically delegating to `list.iter().map()` through the `Iterable<T>` spec. This involved adding a `body: Option<Box<Expr>>` field to `SpecMethod`, updating the parser to handle optional method bodies in spec declarations, and implementing a spec-level method resolution fallback when a method is not found directly on the type.

### Generic Types and Type Parameters

Generic type support (Plan 048) introduced parametric polymorphism to AutoLang. The syntax `type List<T>` or `tag May<T>` declares a type parameter that is substituted with concrete types at use sites like `List<int>`. The implementation involved extending the parser to handle `<T>` parameter lists in type declarations, adding a `GenericParam` structure to the AST, and implementing type substitution that replaces parameter names with concrete types throughout field declarations and method signatures.

The stdlib was converted to use generics: `tag May<T>` in `stdlib/auto/may.at` and `type List<T>` in `stdlib/auto/list.at`. The `?T` syntax sugar (Plan 049) was migrated from the hardcoded `Type::May(Box<Type>)` AST variant to the generic `tag May<T>` system, preserving the convenient `?int` notation while routing through the generic type machinery.

Generic specs (Plan 057) extended traits with type parameters, enabling declarations like `spec Storage<T> { fn data() *T }` and implementations like `type Heap<T> as Storage<T>`. The C transpiler performs monomorphization, generating specialized vtables for each concrete instantiation (e.g., `Storage_int_vtable` with `int (*get)(void *self)`). Type argument counts are validated at parse time against the spec's generic parameter count.

Generic type fields (Plan 059) resolved the remaining gap where struct fields could not use generic types. Fields like `iter I` in `type MapIter<I, T>` and `list *const List<T, S>` in `type ListIter<T, S>` now parse correctly. This plan also added `*const T` and `*mut T` pointer qualifiers and `impl<T, S> Type<T, S>` generic impl block syntax.

### Generic Constraints

Generic constraints (Plan 061) completed the parametric polymorphism story by allowing type parameters to be bounded by specs: `<T: Spec>`. This enables the compiler to verify that concrete types satisfy the required interface at compile time. The `TypeParam` structure in `ast/types.rs` gained a `constraint` field, and the parser's `parse_type_param()` function was updated to parse the constraint syntax. Type inference helpers in `infer/expr.rs` use constraint information to validate method calls on bounded type parameters. This feature was the final unblocker for Plan 051's Auto Flow iterator system, enabling all eight phases of the iterator design to complete.

### Single Inheritance and Field Access

Single inheritance (Plan 021) and dot expression field access (Plan 056) form the object-oriented foundation. Inheritance uses a flat struct strategy where parent fields are copied directly into the child struct, avoiding nested layout complexity. Field access uses a dedicated `Expr::Dot(object, field)` AST node rather than repurposing the binary `Expr::Bina` with `Op::Dot`, which cleanly separates field access from arithmetic operations and enables correct move semantics: reading `obj.field` does not move the object.

The evaluator distinguishes four dot-expression patterns: static method calls (`List.new()`), instance method calls (`list.push(1)`), field reads (`let x = obj.field`), and field assignments (`obj.field = value`). The C and Rust transpilers both generate natural dot-notation code in their respective languages. Instance construction supports both named parameters (`Point { x: 1, y: 2 }`) and positional parameters (`Point(1, 2)`).

### String Types and Property Keywords

The string redesign (Plan 025) delivered a comprehensive operations library with 20 functions covering search (`str_contains`, `str_find`), transform (`str_trim`, `str_replace`), split/join, comparison, and utilities. A new `CStr` type in `crates/auto-val/src/cstr.rs` provides null-terminated UTF-8 strings for safe C FFI with five dedicated functions. All 37 unit tests pass.

Property keywords (Plan 026) realigned `view`, `mut`, and `take` with AutoLang's unified dot notation philosophy. These borrow/ownership operators moved from prefix syntax (`view s`) to postfix property syntax (`s.view`), while preserving the prefix position for function parameter annotations (`fn process(mut data int)`). The change was implemented as a hard break with no backward compatibility period, touching approximately 40 test cases. The AST structure remained unchanged since `Expr::View`, `Expr::Mut`, and `Expr::Take` still wrap a single inner expression; only the parsing direction changed.

### Storage Injection

Storage injection (Plan 055) aims to make generic containers platform-aware. The vision is that `List<int>` automatically selects `Fixed<64>` storage on MCU targets and `Dynamic` storage on PC targets. The infrastructure is complete: a `Target` enum detects MCU vs PC via environment variables, `Universe::inject_environment()` populates storage-related environment values, and `List<T, S>` accepts a storage parameter. However, the seamless user experience depends on type alias syntax (`type List<T> = List<T, DefaultStorage>`), which Plan 058 has now implemented. The remaining gap is wiring the alias into the prelude so that bare `List<int>` resolves to the target-appropriate storage strategy. Users can currently write `List<int, Heap>` or `List<int, InlineInt64>` explicitly.

### Type Aliases

Type alias syntax (Plan 058) enables `type X = Y` declarations for simplified type notation. The parser recognizes both simple aliases (`type IntAlias = int`) and generic aliases (`type List<T> = List<T, DefaultStorage>`). The evaluator stores aliases in a dedicated registry and resolves them during type checking with substitution for generic parameters. The C transpiler generates `typedef` statements for simple aliases. Recursive aliases are detected and rejected to prevent infinite loops.

### Closures

Closure syntax (Plan 060) brings functional programming to AutoLang with JS/TS-style arrow syntax. Single-parameter closures use no parentheses (`x => x * 2`), while multi-parameter closures require them (`(a, b) => a + b`). Block bodies are supported (`(x) => { let y = x * 2; y + 10 }`), and type annotations follow AutoLang's space-separated convention (`(a int, b int) => a + b`).

The implementation stores closure data in the evaluator indexed by unique ID to avoid circular dependencies with the value crate. When a closure is called, the evaluator pushes a new scope, binds parameters, evaluates the body, and pops the scope. Variable capture from enclosing scopes is partially implemented. The C transpiler generates function pointer types (`int (*)(int, int)`) and standalone function definitions, with type inference determining parameter and return types from annotations and body expressions.

## Open Questions

- **Polymorphic arrays**: `[]Flyer` trait object arrays are designed but not fully implemented. The C transpiler generates `unknown` types for these, and runtime type inference for heterogeneous trait collections remains open.
- **Associated types**: Deferred from Plan 059. The simpler approach of using direct return types (`fn iter() Iter<T>`) works for current needs, but complex trait systems with multiple interrelated types may require associated types in the future.
- **`has` composition runtime**: Plan 018's evaluator still treats `has` as a no-op. Full field mixing and method resolution from composed types at runtime needs implementation.
- **Variable capture completeness**: Closures currently support parameter-only operation. Full capture of enclosing scope variables (by-value for primitives, by-reference for complex types) remains to be implemented.
- **Generic constraints extensibility**: Current `<T: Spec>` bounds could be extended with associated types, multiple bounds, and more complex bound expressions.

## Source Plans

- Plan 018: Type Composition Improvements
- Plan 019: Spec Trait System
- Plan 021: Single Inheritance
- Plan 025: String Type Redesign
- Plan 026: Property Keywords
- Plan 048: Generic Type Definitions
- Plan 049: May Operators to Generic Types
- Plan 055: Storage Injection
- Plan 056: Dot Expression Field Access
- Plan 057: Generic Specs
- Plan 058: Type Alias Syntax
- Plan 059: Generic Type Fields
- Plan 060: Closure Syntax
- Plan 061: Generic Constraints
- [191-assert-and-precise-linker-errors.md](../plans/191-assert-and-precise-linker-errors.md)
- [193-conv-type-conversion.md](../plans/193-conv-type-conversion.md)
- [194-monomorphic-dispatch.md](../plans/194-monomorphic-dispatch.md)

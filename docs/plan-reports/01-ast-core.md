# 01 - AST Representation, Atom Format, and Node Unification

## Overview

This report covers the foundational AST infrastructure for AutoLang, spanning twelve implementation plans that established how the compiler represents, serializes, and manipulates abstract syntax trees. The work progressed through three major phases: first, building a complete AST-to-Atom serialization pipeline with traits and test coverage; second, modernizing the core Node data structure with IndexMap and unifying redundant child-storage fields; and third, creating a three-layer construction API (chain methods, Builder pattern, and macro DSL) that reduced tree-building code by 60--70 percent. Nine of the twelve plans are fully implemented, with three remaining in planning status.

## Plan Index

| Plan | Title | Status | Summary |
|------|-------|--------|---------|
| 001 | VM Function Integration | Completed | Native Rust VM functions with registry, module-based dispatch for stdlib APIs |
| 002 | ToAtom AST Testing | Completed | Comprehensive markdown-based tests for all 33 `to_atom()` implementations |
| 003 | ToNode Trait Refactoring | Completed | New `ToNode` trait returning `Node` directly instead of `Value` wrapper, eliminating 42 unwrap sites |
| 004 | ToAtom Refactor Plan | Completed | Refactored `ToAtom` to return `AutoStr` instead of `Value` for text serialization |
| 005 | ToAtom Text Refactor | Completed | Implemented `AtomWriter` trait for all AST types with Lisp-style S-expression output |
| 006 | Fix AtomWriter Implementations | Completed | Fixed output to match hand-written test expectations for operators, params, types |
| 011 | Auto-Atom Refactoring | Planned | Production-ready atom library with `AtomError`, query API, JSON support, schema validation |
| 012 | Node Refactoring IndexMap | Completed | Migrated NodeBody and Obj from BTreeMap+Vec to IndexMap for O(1) lookups |
| 013 | Unify Args and Props | Planned | Eliminate separate `Args` structure, unify with `props` using `num_args` boundary counter |
| 014 | Unify Body, Nodes, Kids | Planned | Merge `body`, `body_ref`, and `nodes` fields into single `kids` field with `Kid` enum |
| 015 | Atom Builder API | Completed | Chain construction methods and Builder pattern for Node/Array/Obj/Atom (~735 LOC, 77 tests) |
| 016 | Atom Macro DSL | Completed | `value!`, `atom!`, `node!` proc macros with `#{var}` interpolation via AutoLang parser |

## Status

**Implemented**: Plans 001, 002, 003, 004, 005, 006, 012, 015, 016 (9 plans)

**Planned**: Plans 011 (auto-atom refactoring), 013 (unify args/props), 014 (unify body/nodes/kids)

## Design

### VM Function Registry and Stdlib Dispatch (Plan 001)

The VM function integration plan established the architecture for calling native Rust functions from the AutoLang interpreter. The design centers on a centralized `VmRegistry` using a module-based organization scheme. Each module (such as `auto.io`) groups related functions and type methods. The registry supports both standalone functions (`VmFunction` signature) and instance methods (`VmMethod` signature that receives `&mut Value` as the receiver).

The dispatch pipeline works in stages. When the evaluator encounters a `use` statement targeting a VM module, it looks up the module in the global registry and registers imported functions in the current scope as `FnKind::VmFunction` declarations. When a call is made, `eval_vm_fn_call()` performs a cached lookup---the first invocation hits the registry, and subsequent calls use a per-universe `vm_function_cache` HashMap for O(1) dispatch. Method calls on type instances (such as `file.read_text()`) follow a separate path through `eval_vm_method_call()`, which passes the instance as the first argument.

Key design decisions include lazy loading of modules on first `use`, thread-safe access via `Mutex`, and separate function/method type signatures. Error handling covers four categories: `ModuleNotFound` for unknown module paths, `FunctionNotFound` for missing functions within a module, `MethodNotFound` for missing type methods, and `RuntimeError` for errors raised by the VM function itself (such as file-not-found). Error messages include available module names to guide the user. The IO module serves as the reference implementation, registering `open` as a standalone function and `close`/`read_text` as methods on the `File` type.

The plan identified three main risk areas: method call parsing (the parser may not represent `file.close()` as a distinct `Expr::Dot` node), function signature mismatches between Rust's multi-argument dispatch and the single-argument `VmFunction` type, and Rust borrow checker conflicts from the `&mut Value` parameter in method calls. Mitigations included adding `Expr::Dot` support, creating wrapper functions per method, and using interior mutability or cloning.

### AST Serialization Pipeline (Plans 002--006)

The AST serialization work progressed through five plans that built, tested, and refined the Atom representation layer.

**Plan 002** introduced the `ToAtom` trait returning `Value` and defined the ATOM format specification. The mapping from AST types to ATOM nodes follows a consistent pattern: literals become simple nodes like `int(42)` or `str("hello")`, binary operators become `bina(op, left, right)`, and complex statements become nodes with properties and children. Implementation proceeded dependency-first---foundation types (Type, Key, Pair, Param, Arg, Body, Branch), then expressions, then statements, then top-level containers---covering all 50+ AST structs across 20 modules. The plan also created `atom_helpers.rs` with an `AtomBuilder` utility providing shortcuts like `int_node()`, `str_node()`, and `ident_node()`.

**Plan 002's testing component** established a markdown-based test format where each test file contains input code and expected ATOM output separated by `---`. Five test files were planned covering literals, expressions, functions, control flow, types, and declarations, targeting all 33 `to_atom()` implementations.

**Plan 003** addressed a core API problem: 32 of the 35 `ToAtom` implementations returned `Value::Node`, requiring callers to write `.to_atom().to_node().unwrap()` in 42 locations. The solution introduced a parallel `ToNode` trait that returns `Node` directly for complex AST structures, while `ToAtom` remains for primitive types that return `Value::Str`, `Value::Int`, or `Value::Pair`. The existing `ToAtom` implementations for node-producing types were converted to delegate: `fn to_atom(&self) -> Value { Value::Node(self.to_node()) }`. This eliminated all 42 unwrap calls and made the type-level distinction between "this produces a node" and "this produces a primitive value" explicit in the trait system. Eighteen AST module files were modified to implement `ToNode`.

**Plan 004** further refined the trait by changing `ToAtom`'s return type from `Value` to `AutoStr`. The rationale was that `ToAtom` serves as a text serialization mechanism, not a tree construction one---`ToNode` handles tree building. This separation allowed each trait to optimize for its purpose: `ToNode` for structural manipulation, `ToAtom` for string output. The `ToAtomStr` helper trait provided a blanket implementation: any type implementing `AtomWriter` could call `to_atom_str()` to get a cached string representation.

**Plan 005** implemented the `AtomWriter` trait across all AST types. `AtomWriter` defines `fn write_atom(&self, f: &mut impl io::Write) -> AutoResult<()>`, a streaming interface that avoids intermediate string allocation. The output format uses Lisp-style S-expressions: `(if (branch cond body) (else else-body))`, `(fn name=add params=(params ...) return=int body=(body ...))`. Implementation followed six complexity tiers from primitives to top-level types.

**Plan 006** fixed the AtomWriter output to match hand-written test expectations. Seven specific issues were resolved: binary operators had unnecessary quotes (`bina('+', 1, 2)` instead of `bina(+, 1, 2)`), if-statement formatting had incorrect newlines, function parameters used verbose format (`param(name("a"), type(int))` instead of `(a, int)`), function return types were missing, struct constructors were rendered as `call` instead of `node`, member format was overly verbose, and TypeDecl format did not match expectations. The struct constructor detection proved the most challenging problem because the TypeDecl is not in scope during method parsing; a heuristic based on uppercase-first-letter detection was used as a fallback.

### Node Structure Modernization (Plan 012)

The Node structure migration from BTreeMap+Vec to IndexMap addressed five critical issues in the previous design: O(log n) lookups, memory overhead from duplicate storage (index vector plus BTreeMap), manual synchronization bugs, incorrect display order (sorted instead of insertion order), and high maintenance complexity.

IndexMap was chosen because it provides O(1) average lookups while preserving insertion order---the same library used by rustc, tokio, and serde. The migration removed the `index: Vec<ValueKey>` field from `NodeBody`, changed `BTreeMap<ValueKey, NodeItem>` to `IndexMap<ValueKey, NodeItem>`, and eliminated all `self.index.push()` calls. The `Obj` type received the same treatment. The `to_astr()` method was simplified from manual index-based iteration to a direct `self.map.iter()` call that automatically yields items in insertion order.

The migration produced 11 new insertion-order tests and passed all 349 existing tests (285 auto-lang + 28 auto-val + 18 auto-atom + 2 auto-xml + 19 doc tests). Performance improved by 2--10x for lookups on structures with more than 100 items, with approximately 20--30% memory reduction.

A notable behavior change accompanied the migration: Display and serialization now iterate in insertion order rather than sorted (alphabetical) order. Code that relied on sorted output was updated to explicitly call `.sorted()` when needed. The migration took approximately one week across six phases: dependency setup, NodeBody refactoring, Obj refactoring, testing and validation, documentation updates, and a migration guide.

### Planned Unifications (Plans 013, 014)

Two related refactoring plans remain to be implemented, both targeting further simplification of the Node structure.

**Plan 013** proposes eliminating the separate `Args` structure (which holds `Vec<Arg>` with positional and named variants) and unifying it with the `props` IndexMap field. The key insight is that IndexMap preserves insertion order, so a `num_args: usize` boundary counter can distinguish arguments (added first, at construction time) from body properties (added later, in the node body). The unified `props` IndexMap would store args at indices `0..num_args` and body properties at indices `num_args..`. New accessor methods---`is_arg()`, `get_arg()`, `args_iter()`, `body_props_iter()`---would enforce the boundary. This eliminates redundant storage, inconsistent access patterns, and dual lookup logic.

**Plan 014** proposes merging three separate child-related fields (`nodes: Vec<Node>`, `body: NodeBody`, `body_ref: MetaID`) into a single `kids: Kids` field. The `Kids` type wraps an `IndexMap<ValueKey, Kid>` where `Kid` is an enum with `Node(Node)` and `Lazy(MetaID)` variants, giving type-safe distinction between eagerly evaluated children and lazily evaluated references. A separate `lazy: Option<MetaID>` field on `Kids` stores the deferred body reference for LAZY tempo mode. The key design decisions include: keeping `body_ref` temporarily during migration (some code still checks `body_ref != MetaID::Nil`), using the `Kid` enum for type safety rather than storing all children as `Value` and checking types at runtime, and adopting a two-phase migration strategy (add `kids` alongside old fields, migrate code incrementally, then remove old fields). The estimated implementation time is 3--4 days across six phases, affecting approximately 20 changes in eval.rs, 30 in universe.rs, 5 in parser.rs, and 10 in the transpiler layer.

### Three-Layer Construction API (Plans 015, 016)

Plans 015 and 016 created a progressive API for constructing Atom/Node/Array/Obj trees, reducing typical construction code by 60--70%.

**Layer 1: Chain methods** (Plan 015, Phase 1, ~305 LOC, 33 tests) added `with_*` methods that return `self` for fluent chaining. For Node: `with_prop(key, value)`, `with_child(node)`, `with_text(text)`, `with_arg(arg)`, and batch variants. Array got `with(value)` and `with_values(iter)`. Obj got `with(key, value)` and `with_pairs(iter)`. Atom received convenience constructors like `node_with_props()`, `array_from()`, and `obj_from()`. These methods are pure additions with zero breaking changes.

**Layer 2: Builder pattern** (Plan 015, Phase 2, ~430 LOC, 44 tests) introduced dedicated builder types---`NodeBuilder`, `ArrayBuilder`, `ObjBuilder`, `AtomBuilder`---with conditional construction methods. `prop_if(condition, key, value)` adds a property only when the condition is true; `child_if(condition, node)` does the same for children. This enables runtime configuration-driven tree construction without branching code. The builders support deferred construction: configure first, call `build()` once. They are compatible with both the old args system and the planned unified props system.

**Layer 3: Macro DSL** (Plan 016, ~620 LOC implementation + ~380 LOC tests) took an unexpected but beneficial design turn. The original plan called for `macro_rules!` declarative macros, but the actual implementation uses procedural macros backed by the AutoLang parser. The `value!`, `atom!`, and `node!` macros convert their TokenStream input to a string, parse it through `AtomReader`, and produce the corresponding Value/Atom/Node at runtime. This approach gives complete AutoLang syntax support automatically---any syntax the parser handles works in the macro without additional macro rules. The macros are implemented in a separate `auto-lang-macros` crate, re-exported through the main `auto-lang` crate. The `ToAutoValue` trait lives in `auto-val/src/to_value.rs` (~102 LOC) and covers all primitive types plus reference types.

The macro system supports variable interpolation through the `#{var}` syntax. A `ToAutoValue` trait converts Rust types (`i32`, `f64`, `bool`, `&str`, and more) to AutoLang values, enabling mixed literal-and-interpolation expressions like `value!{ name: #{name}, count: #{count}, active: true }`. The interpolation detection scans for `Punct('#') + Group(Brace, Ident)` patterns and generates `var.to_auto_value()` calls. When no interpolation is detected, the macro passes the raw string to `AtomReader` directly, avoiding the overhead of token-by-token processing.

Testing produced 30 total tests across three files: 9 proc_macro_tests, 13 value_macro_tests (including 4 interpolation tests), 3 to_value unit tests, and 5 doc tests, all passing with 100% pass rate and zero compiler warnings.

The three layers serve distinct use cases: chain methods for simple static construction, Builder for conditional runtime construction, and macros for declarative configuration-like syntax. The recommendation is to use macros for static configuration structures and test data, Builder for complex runtime logic and dynamic generation, and chain methods for quick inline construction where neither macros nor Builder are warranted.

### Planned Auto-Atom Refactoring (Plan 011)

The auto-atom crate currently operates as a minimal prototype at ~190 lines with 5 panic calls, no documentation, only 3 tests, and no query or manipulation API. The refactoring plan targets a production-ready library at ~1,500 lines organized across 10 phases over 3--4 weeks.

The highest-priority work (P0, Week 1) replaces all 5 panic calls with a custom `AtomError` enum using thiserror, creates comprehensive Rustdoc documentation on all public APIs, expands tests from 3 to 50+, and fixes the `merge_atom()` function in universe.rs to support nested node structures. The `AtomError` enum provides variants for `InvalidType`, `ConversionFailed`, `AccessError`, `SerializationError`, `ValidationError`, and `MissingField`, with an `AtomResult<T>` type alias used throughout.

The second priority (P1, Weeks 2--3) adds a query API with 15+ methods (`get`, `get_path`, `find`, `filter`, `has`, `keys`, `values`, and type checks), a manipulation API (`add`, `remove`, `update`, `merge`), JSON serialization behind a feature flag using serde/serde_json, and performance benchmarks using Criterion. Path navigation supports both dot notation (`"users.0.name"`) and bracket notation (`"users[0].name"`).

Lower-priority items (P2, Week 4) include schema validation via an `AtomSchema` enum, YAML/TOML/CBOR format support, XPath-like query language, and pretty-print visualization. All breaking changes (such as `Atom::new()` returning `AtomResult<Atom>` instead of `Atom`) would be documented in a migration guide with deprecated wrapper methods during a transition period.

## Open Questions

- **Plan 011 scope**: The auto-atom refactoring plan is ambitious (190 lines to ~1,500 lines, 3--4 weeks). The priority of error handling, JSON serialization, and schema validation relative to other compiler work needs evaluation.
- **Plan 013 invariant enforcement**: The `num_args` boundary counter relies on the invariant that args are always added before body properties. Runtime invariant checks in debug mode could help catch violations, but the enforcement strategy is not yet finalized.
- **Plan 014 migration path**: Merging `body`, `nodes`, and `body_ref` into `kids` requires updating approximately 60 downstream call sites. Whether to use a two-phase migration (add alongside, then remove old) or a single breaking change needs to be decided based on the current number of dependents.
- **Macro performance**: The proc-macro approach parses strings at runtime via `AtomReader`, which is slower than direct construction. Whether this matters in practice (macros are typically used for static configuration, not hot paths) has not been benchmarked.

## Source Plans

- `docs/plans/001-vm-function-integration.md`
- `docs/plans/002-to-atom-ast.md`
- `docs/plans/003-to-node-trait-refactoring.md`
- `docs/plans/004-to_atom_refactor_plan.md`
- `docs/plans/005-to-atom-text-refactor-plan.md`
- `docs/plans/006-fix-atomwriter-implementations.md`
- `docs/plans/011-auto-atom-refactoring.md`
- `docs/plans/012-node-refactoring-indexmap.md`
- `docs/plans/013-unify-args-and-props.md`
- `docs/plans/014-unify-body-nodes-kids.md`
- `docs/plans/015-atom-builder-api.md`
- `docs/plans/016-atom-macro-dsl.md`

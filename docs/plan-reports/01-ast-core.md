# 01 - AST Representation, Atom Format, and Node Unification

## Overview
This report covers the core AST infrastructure for AutoLang, including the Atom data interchange format, ToAtom/ToNode trait system, Node structure improvements (IndexMap migration, args/props unification), and the Builder/macro API for tree construction. These plans form the foundation for how the compiler represents, serializes, and manipulates abstract syntax trees.

## Plan Summary

| Plan | Title | Status | Summary |
|------|-------|--------|---------|
| 001 | VM Function Integration | ✅ | Integrate native Rust VM functions with AutoLang interpreter for stdlib APIs |
| 002 | ToAtom AST Testing | ✅ | Comprehensive testing for all 33 `to_atom()` implementations in ATOM format |
| 003 | ToNode Trait Refactoring | ✅ | Refactor ToAtom to ToNode trait returning `Node` directly instead of `Value` wrapper |
| 004 | ToAtom Refactor Plan | ✅ | Refactor ToAtom trait to return `AutoStr` instead of `Value` for text serialization |
| 005 | ToAtom Text Refactor | ✅ | Implement `AtomWriter` trait for all AST types with Lisp-style S-expression output |
| 006 | Fix AtomWriter Implementations | ✅ | Fix AtomWriter output to match hand-written test expectations (operators, params, types) |
| 011 | Auto-Atom Refactoring | ⏳ | Transform auto-atom from prototype to production-ready library with error handling, query API, JSON support |
| 012 | Node Refactoring IndexMap | ✅ | Migrate NodeBody and Obj from BTreeMap+Vec to IndexMap for O(1) lookups and insertion-order preservation |
| 013 | Unify Args and Props | ⏳ | Eliminate separate `Args` structure, unify with `props` using IndexMap `num_args` boundary counter |
| 014 | Unify Body, Nodes, Kids | ⏳ | Unify `body`, `body_ref`, and `nodes` fields into single `kids` field with `Kid` enum |
| 015 | Atom Builder API | ✅ | Add chain construction methods and Builder pattern for Node/Array/Obj/Atom (~735 LOC, 77 tests) |
| 016 | Atom Macro DSL | ✅ | Implement `value!`, `atom!`, `node!` proc macros with variable interpolation via AutoLang parser |

Status codes: Completed, Planned, Partial/In Progress, Deprecated

## Status Summary
- Completed: 9 | Partial: 0 | Planned: 3 | Deprecated: 0

## Key Achievements
- Full Atom serialization pipeline: ToNode/ToAtom traits, AtomWriter, and test suite covering 33+ AST types
- Node structure modernized with IndexMap for O(1) lookups, insertion-order preservation, and 349+ tests passing
- Three-layer construction API: chain methods (simple), Builder pattern (conditional), and macro DSL (declarative)
- Proc macro system using AutoLang parser for `value!`/`atom!`/`node!` macros with `#{var}` interpolation

## Remaining Work
- Plan 011 (auto-atom refactoring): Error handling with `AtomError`, query/manipulation API, JSON serialization, and schema validation
- Plan 013 (unify args/props): Remove `Args`/`Arg` types, implement `num_args` boundary counter, update all downstream code
- Plan 014 (unify body/nodes/kids): Merge three child fields into `Kids` type with `Kid::Node`/`Kid::Lazy` variants

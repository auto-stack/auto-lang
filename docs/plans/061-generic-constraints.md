# Plan 061: Generic Constraints via #[with(...)]

**Status**: ✅ Complete (All Phases)
**Priority**: P0 (Pre-requisite for Plan 051 Phase 4-8)
**Dependencies**: None
**Timeline**: ~6 hours completed  

## Objective

Implement `#[with(...)]` annotation syntax for declaring constrained generic type parameters.

## Target Syntax

```auto
// Simple generics (existing, unchanged)
fn identity<T>(x T) T

// Constrained generics (new)
#[with(I as Iter<T>, T, U)]
fn map(iter I, f T=>U) MapIter<I, T, U>

#[with(T as Clone)]
fn duplicate(x T) T { return x.clone() }
```

## Design Summary

See [docs/design/generic-constraints.md](file:///d:/autostack/auto-lang/docs/design/generic-constraints.md) for full design rationale.

| Scenario | Syntax | Example |
|----------|--------|---------|
| Simple generic | `fn foo<T>(...)` | `fn identity<T>(x T)` |
| Constrained generic | `#[with(T as Spec)]` | `#[with(T as Clone)]`<br>`fn dup(x T)` |

## Implementation Phases

### Phase 1: Parser Extension (2-3 hours)

#### [MODIFY] [parser.rs](file:///d:/autostack/auto-lang/crates/auto-lang/src/parser.rs)

1. Add `parse_with_attr()` function
   - Parse `with(T, U as Iter<V>, ...)` syntax
   - Recognize `as` keyword for constraints
   - Return `Vec<TypeParam>` with constraints populated

2. Integrate with `parse_fn()` and `parse_type_decl()`
   - Check for `#[with(...)]` attribute
   - Merge with any `<T>` declarations (with overrides)

### Phase 2: Integration (1-2 hours)

- Connect parsed constraints to existing `TypeParam.constraint` field
- Ensure monomorphization respects constraints
- Add constraint info to type checking (if applicable)

### Phase 3: Testing (1 hour)

- Create `test/a2c/110_with_constraint/` test case
- Run existing generic tests for regression
- Verify backward compatibility

## Success Criteria

- [x] `#[with(T)]` parses correctly (no constraint)
- [x] `#[with(T as Spec)]` parses with constraint populated
- [x] `#[with(A as Clone, B as Debug)]` handles multiple params
- [x] Existing `<T>` syntax continues to work
- [x] a2c test passes (test_110_with_constraint)
- [x] Type argument inference during function calls
- [x] Constraint validation using TraitChecker
- [x] type_args stored in Call AST for debugging

## Related Plans

- **Plan 051**: Auto Flow (blocked on this for Phase 4-8)
- **Plan 057**: Generic Specs
- **Plan 060**: Closure Syntax

## References

- Design doc: [docs/design/generic-constraints.md](file:///d:/autostack/auto-lang/docs/design/generic-constraints.md)
- AST location: `TypeParam.constraint` in `ast/types.rs:223`

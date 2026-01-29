# Plan 062: Fix Remaining Failing A2C Transpiler Tests

**Status**: Draft
**Created**: 2025-01-30
**Priority**: High

## Overview

Currently 18 a2c transpiler tests are failing out of 134 total tests. This plan analyzes the failures and provides a strategic roadmap to fix them systematically.

## Current Status

- **Passing**: 116 tests (86.6%)
- **Failing**: 18 tests (13.4%)
- **Blocked**: Test files not yet created for advanced features

## Root Cause Analysis

### Category 1: Test Files Not Created (12 tests)

Most failing tests (080-125 series) have test functions defined but no `.at` source files or expected outputs. These are placeholders for future features.

| Test ID | Test Name | Status | Blocker |
|---------|-----------|--------|---------|
| 080 | array_nested | No files | Generic array syntax |
| 080 | array_zero_size | No files | Zero-size array validation |
| 080 | array_slice | No files | Slice type syntax |
| 090 | type_alias | No files | Type alias syntax |
| 095 | storage_module | No files | Spec/module syntax |
| 096 | storage_usage | No files | Storage usage |
| 097 | list_storage | Parsing error | Generic type `>` parsing |
| 102 | generic_field | No files | Generic struct fields |
| 104 | terminal_operators | No files | Terminal operators |
| 111 | io_specs | No files | IO spec syntax |
| 121 | terminal_operators | No files | Terminal operators (duplicate?) |
| 122 | bang_operator | Exists but wrong output | Bang operator syntax |
| 123 | extended_adapters | No files | Adapter syntax |
| 124 | predicates | No files | Predicate syntax |
| 125 | collect | No files | Iterator collect |

### Category 2: Feature Not Implemented (5 tests)

| Test ID | Test Name | Blocker | Complexity |
|---------|-----------|---------|------------|
| 021 | type_error | Type checking not integrated | High |
| 060 | generic_tag | "generics not yet implemented" | Very High |
| 092 | const_generics | Const generic parameters | Very High |
| 097 | list_storage | Generic `>` parsing issue | Medium |
| 122 | bang_operator | `!` operator not implemented | Medium |

## Strategic Approach

### Phase 1: Quick Wins (Tests That Exist) - 2 tests

#### 1.1 Fix test_097_list_storage (List<T, Storage> parsing)
**Problem**: Parser treats `>` in `List<int, Heap>` as comparison operator
**Root Cause**: Same issue as test_101 - generic type angle bracket parsing
**Solution**: Already fixed in previous work, just needs test file validation
**Estimated Effort**: 1 hour
**Dependencies**: None

#### 1.2 Fix test_122_bang_operator (Postfix `!` operator)
**Problem**: Bang operator `list.iter()!` not implemented
**User Clarification**: Should use `.!` syntax, not `!` directly
**Solution**:
- Add `TokenKind::Bang` to lexer
- Parse postfix `.!` operator
- Transpile to `.collect()` call
**Estimated Effort**: 4-6 hours
**Dependencies**: Parser, lexer, C transpiler

### Phase 2: Generic Type Parsing Fix (Foundation) - Impact: 5+ tests

#### 2.1 Fix Generic Type Greater-Than Parsing
**Problem**: `>` in `List<int, Heap>` parsed as comparison instead of generic close
**Affected Tests**:
- test_097_list_storage
- test_102_generic_field
- test_060_generic_tag
- test_095_storage_module
- Any future tests with `<T>` syntax

**Current Behavior**:
```auto
let list = List<int, Heap>.new()
```

Parses as: `List < int` (comparison), `Heap > .new()` (comparison)

**Required Fix**:
The parser already has some generic type support in `parse_ident_or_generic_type()` but it's not being used consistently in expression context. Need to:
1. Improve pratt parser to recognize `Type<Params>` as single unit in expressions
2. Prevent `<` and `>` from being treated as comparison operators when in type context
3. Handle generic types in: method calls, struct construction, variable declarations

**Key Files**:
- `crates/auto-lang/src/parser.rs`: `expr_pratt_with_left()`, `parse_ident_or_generic_type()`
- `crates/auto-lang/src/lexer.rs`: Token handling

**Estimated Effort**: 8-12 hours
**Dependencies**: Parser, lexer

#### 2.2 Implement Empty/Zero-Size Arrays
**Test**: test_080_array_zero_size
**Problem**: Parser rejects `arr[0]` with "Array size must be greater than 0"
**Solution**:
- Remove size validation or allow size 0 for flexible arrays
- Transpile to C flexible array member or `arr[]`
**Estimated Effort**: 4-6 hours
**Dependencies**: Parser, C transpiler

### Phase 3: Type Checking Integration (1 test)

#### 3.1 Integrate Type Checking for test_021_type_error
**Problem**: Type inference system exists but not integrated with parser
**Current State**:
- Type inference implemented: `crates/auto-lang/src/infer/`
- Documented: "Phase 5: Parser integration (deferred per user request)"
- Test expects: Type mismatch error when passing `int` to `str` field

**Required Work**:
1. Call `infer_expr()` after parsing store statements
2. Add type checking for struct initialization arguments
3. Report type errors with miette
4. Update test expectations if needed

**Key Files**:
- `crates/auto-lang/src/parser.rs`: Add type checking calls
- `crates/auto-lang/src/trans/c.rs`: Optionally skip type-error code

**Estimated Effort**: 12-16 hours
**Dependencies**: Type inference system, parser integration

**Note**: This was explicitly deferred by user - may need confirmation to proceed.

### Phase 4: Advanced Features (Long-term) - 10 tests

These tests require significant language feature implementation:

#### 4.1 Generic Tags (test_060_generic_tag)
**Required**:
- Generic type parameters in tag definitions
- Tag instantiation with type arguments
- Type substitution in tag fields

**Estimated Effort**: 20-30 hours

#### 4.2 Const Generics (test_092_const_generics)
**Required**:
- Const generic parameters: `type Array<N, T>`
- Const expressions in type parameters
- Compile-time evaluation of const expressions

**Estimated Effort**: 30-40 hours

#### 4.3 Type Aliases (test_090_type_alias)
**Required**:
- Parse `type Name = Type` syntax
- Type substitution in transpilation
- Support for generic type aliases

**Estimated Effort**: 8-12 hours

#### 4.4 Storage System (test_095, test_096, test_097)
**Required**:
- Spec declarations (already partially implemented)
- Storage trait implementations
- Type-level storage parameters
- Integration with generic types

**Estimated Effort**: 40-60 hours

#### 4.5 Generic Struct Fields (test_102_generic_field)
**Required**:
- Parse generic fields: `type Foo<T> { field: T }`
- Type substitution in field access
- Transpiler support for generic structs

**Estimated Effort**: 16-24 hours

#### 4.6 Terminal Operators (test_104, test_121)
**Required**:
- Terminal operator syntax
- Short-circuit evaluation
- C transpiler support

**Estimated Effort**: 12-20 hours

#### 4.7 IO Specs (test_111)
**Required**:
- Spec declarations for IO operations
- Spec implementation in types
- IO method resolution

**Estimated Effort**: 24-32 hours

#### 4.8 Bang Operator (test_122)
**Clarification**: Use `.!` syntax (not `!`)
**Required**:
- Add `TokenKind::Bang`
- Parse postfix `.!` operator
- Transform to `.collect()` during parsing/transpilation

**Estimated Effort**: 6-8 hours

#### 4.9 Adapters/Predicates/Collect (test_123, test_124, test_125)
**Required**:
- Adapter syntax and implementation
- Predicate syntax and evaluation
- Iterator collect protocol

**Estimated Effort**: 20-30 hours

## Recommended Implementation Order

### Sprint 1: Fix Generic Type Parsing (Foundation)
**Impact**: Unlocks 5+ tests
**Effort**: 8-12 hours

1. Fix `>` parsing in generic types (2.1, 2.2)
2. Validate test_097_list_storage works
3. Check if test_102_generic_field unblocks

### Sprint 2: Quick Feature Implementations
**Impact**: +3-4 tests
**Effort**: 10-16 hours

1. Implement zero-size arrays (2.2)
2. Implement bang operator with `.!` syntax (1.2)
3. Create test file for type_alias if simple

### Sprint 3: Medium Complexity Features
**Impact**: +2-3 tests
**Effort**: 20-30 hours

1. Type aliases (4.3)
2. Generic struct fields (4.5)
3. Terminal operators (4.6) if needed

### Sprint 4: Major Features (Long-term)
**Impact**: +8-10 tests
**Effort**: 80-150 hours

1. Storage system (4.4)
2. Generic tags (4.1)
3. Const generics (4.2)
4. IO specs (4.7)
5. Adapters/predicates/collect (4.9)

### Sprint 5: Type Checking (Deferred)
**Impact**: +1 test
**Effort**: 12-16 hours
1. Integrate type checking (3.1) - **ONLY if user confirms**

## Detailed Implementation Plans

### Sprint 1: Fix Generic Type Greater-Than Parsing

**Problem Analysis**:
The pratt parser correctly parses `List<int, Heap>` as a type in type position, but fails when it appears in expression position (like `List<int, Heap>.new()`). The parser splits it into comparisons:
- `List < int` (is List less than int?)
- `Heap > .new()` (is Heap greater than .new()?)

**Solution Approach**:

1. **Option A**: Modify `parse_primary()` to recognize `Ident < ... >` pattern
   - Pros: Centralized handling
   - Cons: May conflict with comparison parsing

2. **Option B**: Use `parse_type()` in expression context for GenName
   - Pros: Reuses existing type parser
   - Cons: May have precedence issues

3. **Option C**: Add lookahead in pratt parser
   - Pros: Most accurate
   - Cons: More complex

**Recommended**: Option A with enhanced `parse_primary()`

**Implementation Steps**:

1. Modify `parse_primary()` to detect generic type pattern:
```rust
fn parse_primary(&mut self) -> AutoResult<Expr> {
    // ... existing code ...

    // Check for generic type: Ident < Type, ... >
    if self.is_kind(TokenKind::Ident) {
        let save = self.clone();
        self.next(); // consume ident

        // Lookahead to see if this is Type<Params> or just Ident
        if self.is_kind(TokenKind::Lt) && self.next_token_is_type() {
            // This is a generic type instantiation
            self = save;
            return self.parse_ident_or_generic_type();
        }

        self = save;
        // ... rest of ident handling
    }
}
```

2. Update `is_returnable()` to handle GenName
3. Add GenName support in method call resolution
4. Test with `List<int, Heap>.new()`

**Test Cases**:
- `List<int, Heap>.new()` - method call on generic type
- `MyType<string, int>` - nested generics
- `map.get(key)` - regular method call (should not break)

### Sprint 2: Bang Operator (with `.!` syntax)

**User Clarification**: Use `list.iter().!` not `list.iter()!`

**Syntax**:
```auto
let collected = list.iter().!  // Eagerly collect iterator
```

**Desugaring**:
```auto
let collected = list.iter().collect()
```

**Implementation Steps**:

1. **Lexer** (`crates/auto-lang/src/lexer.rs`):
   - Add `TokenKind::Bang` (for `!`)
   - Add `TokenKind::DotBang` (for `.!`) if needed

2. **Parser** (`crates/auto-lang/src/parser.rs`):
   - In `expr_pratt_with_left()`, after parsing `Expr::Dot`
   - Check if next token is `Bang`
   - If yes, create `Expr::DotBang` or desugar immediately

3. **Transpiler** (`crates/auto-lang/src/trans/c.rs`):
   - Transpile `.!` to `.collect()` call
   - Or handle as special case in expression handling

**Code Structure**:
```rust
// In parser, after dot expression handling
if self.is_kind(TokenKind::Bang) {
    self.next(); // consume !
    // Desugar: expr.!  =>  expr.collect()
    let collect_method = Expr::Ident("collect".into());
    let args = Args::new();
    return Ok(Expr::Call(Call {
        name: Box::new(Expr::Dot(Box::new(expr), collect_method)),
        args,
        ret: Type::Unknown,  // Will be inferred
        type_args: Vec::new(),
    }));
}
```

### Sprint 3: Type Aliases

**Syntax**:
```auto
type IntArray = [10]int
type StringList = List<string>
```

**Implementation Steps**:

1. **Parser**:
   - Add type alias statement parsing
   - Store in scope with type substitution info

2. **Transpiler**:
   - Simple aliases: direct name replacement
   - Generic aliases: type substitution

3. **Test**:
   - Create `test/a2c/090_type_alias/type_alias.at`
   - Test basic aliasing
   - Test generic aliasing

## Success Metrics

### Sprint 1 Success Criteria
- [ ] test_097_list_storage passes
- [ ] `List<int, Heap>.new()` transpiles correctly
- [ ] No regression in existing 116 passing tests
- [ ] Generic types work in: method calls, struct fields, arrays

### Sprint 2 Success Criteria
- [ ] test_122_bang_operator passes (with `.!` syntax)
- [ ] test_080_array_zero_size passes
- [ ] 120+ tests passing total

### Sprint 3 Success Criteria
- [ ] test_090_type_alias passes
- [ ] test_102_generic_field passes
- [ ] 125+ tests passing total

### Overall Success
- [ ] All 18 failing tests fixed
- [ ] 134/134 tests passing (100%)
- [ ] No regressions introduced
- [ ] Code quality maintained (zero warnings)

## Risk Assessment

### High Risk Items

1. **Generic Type Parsing Fix**: Core parser change, could break existing tests
   - **Mitigation**: Comprehensive test suite, incremental changes
   - **Rollback Plan**: Git revert + fix in branches

2. **Type Checking Integration**: Major architectural change
   - **Mitigation**: User explicitly deferred, optional feature
   - **Rollback Plan**: Feature flag to disable

### Medium Risk Items

1. **Bang Operator**: New operator syntax
   - **Mitigation**: Clear syntax (`.!`), desugaring approach
   - **Rollback Plan**: Parse as error until implemented

2. **Const Generics**: Very complex feature
   - **Mitigation**: Phase-based implementation
   - **Rollback Plan**: Reject at parse level initially

## Dependencies

### Cross-Feature Dependencies

```
Generic Type Parsing (Foundation)
├── Generic Struct Fields (depends on)
├── Storage System (depends on)
├── Generic Tags (depends on)
└── Type Aliases (depends on)

Type Checking Integration (Independent)
├── Type Aliases (benefits from)
└── Generic Fields (validates)

Zero-Size Arrays (Independent)
└── Arrays (depends on)

Bang Operator (Independent)
└── Iterators (depends on)
```

## Estimated Timeline

| Sprint | Duration | Tests Fixed | Cumulative |
|--------|----------|-------------|-------------|
| 1 | 1-2 days | 5 | 121 |
| 2 | 1-2 days | 3-4 | 124-125 |
| 3 | 3-5 days | 2-3 | 126-128 |
| 4 | 2-3 weeks | 8-10 | 134-138 |
| 5 | 2-3 days | 1 | 135 |

**Total**: 3-4 weeks for all 18 tests (excluding type checking if deferred)

## Open Questions

1. **Type Checking**: Should we proceed with integrating the type inference system? (User previously deferred)

2. **Test File Creation**: Who creates the .at files for tests that don't have them? Should we:

   **Option A**: Create placeholder test files now
   - Pros: Tests defined and ready
   - Cons: May need updates when features implemented

   **Option B**: Create test files alongside implementation
   - Pros: Tests match implementation
   - Cons: Harder to track progress

   **Option C**: Mark tests as `#[ignore]` until ready
   - Pros: Clear status
   - Cons: Test pollution

   **Recommendation**: Option C with clear TODO comments

3. **Generic Type Syntax**: Should we support Rust-style `Vec::<T>` turbofish for disambiguation?

4. **Backwards Compatibility**: How to handle breaking changes in generic type syntax?

## Next Steps

1. **Immediate**: Start Sprint 1 - Fix generic type `>` parsing
2. **Short-term**: Complete Sprint 2 - Quick wins
3. **Medium-term**: Evaluate user priorities for Sprint 3 vs 4
4. **Long-term**: Plan major feature implementation based on user needs

## Appendix: Test Categories

### By Implementation Status

| Category | Count | Tests |
|----------|-------|-------|
| Files exist & parseable | 2 | 097, 122 |
| Parse errors (generic `>`) | 5 | 097, 102, 095, 060, etc. |
| No test files | 11 | 080, 090, 104, 111, 121, 123, 124, 125, etc. |
| Feature not implemented | 5 | 021, 060, 092, 122 |

### By Feature Area

| Feature | Tests | Effort |
|---------|-------|--------|
| Generic types | 8 | High |
| Type checking | 1 | Very High |
| Arrays (advanced) | 3 | Medium |
| Operators | 3 | Medium |
| Storage system | 3 | Very High |
| Iterators/collect | 3 | High |
| Specs/traits | 2 | Very High |
| Type aliases | 1 | Low-Medium |

### By Complexity

| Complexity | Count | Tests |
|------------|-------|-------|
| Trivial (test files) | 11 | Missing .at files |
| Low | 2 | type_error (deferred), bang_operator |
| Medium | 3 | Zero-size arrays, type_alias, generic_field |
| High | 4 | generic_tags, terminal_operators, io_specs, list_storage |
| Very High | 4 | const_generics, storage_modules, adapters, predicates |

---

**Document Status**: Draft - Ready for review
**Next Review**: After Sprint 1 completion
**Related Plans**:
- Plan 051: Iterator system
- Plan 052: Storage-based types
- Plan 060: Closure syntax
- Plan 061: Generic constraints

# Plan 059: Generic Type Fields & Advanced Type System Features

**Status**: 📋 Ready for Implementation
**Priority**: P1 (Core Language Infrastructure)
**Dependencies**: None (Standalone Enhancement)
**Timeline**: 20-30 hours

## Objective

Implement advanced type system features needed for Plan 051 (Auto Flow) and other modern language features:

1. **Generic type fields in structs** - Enable `type MapIter<I, T> { iter I }`
2. **Associated types with constraints** - Enable `type IterT impl Iter<T>`
3. **Borrow/self syntax** - Enable `.IterT`, `self.iter()`
4. **Default implementations in specs** - Enable forwarding methods
5. **Closure syntax** - Enable `|x| x * 2` lambda expressions

## Background

### Why These Features Are Needed

**Plan 051 Auto Flow** requires:
```auto
type MapIter<I, T, U> {
    iter I  // Generic type field - DOESN'T WORK
}

spec Iterable<T> {
    type IterT impl Iter<T>  // Associated type with constraint - DOESN'T WORK
    fn iter() .IterT         // Borrow syntax - DOESN'T WORK

    fn map<U>(f: fn(T)U) MapIter<.IterT, ...> {  // Default impl - DOESN'T WORK
        return .iter().map(f)
    }
}
```

**Current Workarounds** (cumbersome):
- Use pointers: `iter *Iter<int>` instead of `iter Iter<int>`
- Manual forwarding: users write `list.iter().map()` instead of `list.map()`
- No closures: use named functions instead

## Implementation Phases

### Phase 1: Generic Type Fields in Structs (P0) - 6-10 hours

#### 1.1 Investigation

**Task**: Understand why `parse_type()` doesn't recognize `List<int>` in struct field context

**Current Behavior**:
```auto
// This FAILS to parse:
type MyStruct {
    field List<int>  // Error: "Expected end of statement, got LBrace<{>"
}
```

**Root Cause Analysis**:
- Parser has `parse_generic_instance()` at line 4822
- `type_member()` calls `parse_type()` at line 4211
- `parse_type()` calls `parse_ident_or_generic_type()` at line 5117
- `parse_ident_or_generic_type()` checks `next_token_is_type()` at line 4716
- But something prevents proper recognition in struct field context

**Investigation Steps**:
1. Add debug logging to `parse_type()` in struct field context
2. Check if `next_token_is_type()` returns correct value
3. Verify `parse_generic_instance()` is being called
4. Identify where the parsing diverges

#### 1.2 Fix Implementation

**Target**: Make this work:
```auto
type MyStruct {
    field List<int>    // Generic type field
    data May<string>   // Also test with May<T>
}

fn main() {
    let s = MyStruct { field: nil, data: nil }
}
```

**Files to Modify**:
- `crates/auto-lang/src/parser.rs` (lines around 4206-4225)

**Success Criteria**:
- ✅ Generic type fields parse correctly
- ✅ `List<int>`, `Map<int, str>`, `May<int>` all work as field types
- ✅ Nested generics: `type Nested { inner List<List<int>> }`
- ✅ a2c test passes

---

### Phase 2: Associated Types with Constraints (P1) - 4-6 hours

#### 2.1 Syntax Design

**Proposed Syntax**:
```auto
spec Iterable<T> {
    type IterT impl Iter<T>  // Associated type with constraint
    fn iter() IterT
}
```

**Alternative Syntax** (if needed):
```auto
spec Iterable<T> where IterT: Iter<T> {
    type IterT
    fn iter() IterT
}
```

#### 2.2 Implementation

**Files**:
- `crates/auto-lang/src/parser.rs` - Parse `impl` constraint syntax
- `crates/auto-lang/src/ast.rs` - Add constraint to associated type representation
- `crates/auto-lang/src/trans/c.rs` - Transpile constraints to C

**Success Criteria**:
- ✅ `type IterT impl Iter<T>` parses correctly
- ✅ Multiple constraints: `type IterT impl Iter<T> + Clone`
- ✅ a2c test generates appropriate C code

---

### Phase 3: Borrow and Self Syntax (P1) - 4-6 hours

#### 3.1 Borrow Syntax for Return Types

**Target**:
```auto
spec Iterable<T> {
    fn iter() .IterT  // Return "borrow of Self's IterT"
}
```

**Implementation**:
- Parse `.` prefix on return types in method declarations
- Transpile to C pointer return types
- Update type checker to handle borrowed types

#### 3.2 Self Syntax

**Target**:
```auto
impl Iterable<T> for List<T> {
    fn iter() .IterT {  // Same as above, explicit Self
        return .iter
    }
}
```

**Success Criteria**:
- ✅ `.IterT` parses in function signatures
- ✅ `.iter()` parses as "access self.iter"
- ✅ C transpiler generates correct pointer code

---

### Phase 4: Default Implementations in Specs (P1) - 6-8 hours

#### 4.1 Syntax Design

**Target**:
```auto
spec Iterable<T> {
    type IterT impl Iter<T>
    fn iter() .IterT

    // Default implementation (can be overridden)
    fn map<U>(f: fn(T)U) MapIter<.IterT, U> {
        return .iter().map(f)
    }
}
```

**Implementation**:
1. Extend `SpecDecl` to include default method implementations
2. Parse method bodies in spec declarations
3. During impl resolution, check if method has default
4. If no explicit impl, use default from spec

**Success Criteria**:
- ✅ Default methods in specs parse correctly
- ✅ Types using spec get default methods automatically
- ✅ Default methods can be overridden in impl blocks
- ✅ a2c transpilation works with defaults

---

### Phase 5: Closure Syntax (P1) - 8-12 hours

#### 5.1 Syntax Design

**Proposed Syntax**:
```auto
// Closure with single expression
let double = |x| x * 2

// Closure with block
let complex = |x, y| {
    let temp = x + y
    temp * 2
}

// Closure with type annotations
let add = |x int, y int| int { x + y }
```

**Type Inference**:
- Single param: `|x|` infers type from usage
- Multiple params: `|x, y|` requires type annotation or usage
- Return type: inferred from body or specified

#### 5.2 Implementation

**Files**:
- `crates/auto-lang/src/lexer.rs` - Add pipe token `|` to lexer
- `crates/auto-lang/src/parser.rs` - Parse closure syntax
- `crates/auto-lang/src/ast.rs` - Add Closure expression type
- `crates/auto-lang/src/trans/c.rs` - Transpile to C function pointers
- `crates/auto-lang/src/eval.rs` - Evaluate closures

**C Transpilation Strategy**:
```c
// AutoLang:
let double = |x| x * 2

// Generated C:
typedef struct {
    int (*impl)(void*);
    void* env;
} Closure_int;

int double_impl(void* env) {
    int x = *(int*)env;
    return x * 2;
}

int env = 42;
Closure_int closure = { double_impl, &env };
```

**Success Criteria**:
- ✅ Closures parse correctly
- ✅ Captures variables from enclosing scope
- ✅ Type inference works for parameters and return
- ✅ C transpilation generates working code
- ✅ a2c tests for closures

---

## File Structure

```
crates/auto-lang/src/
├── parser.rs             # Add closure parsing, generic type field fixes
├── lexer.rs              # Add pipe token for closures
├── ast.rs                # Add Closure expr, constraints to associated types
├── trans/
│   └── c.rs             # Transpile closures, constraints, borrowed types
└── eval.rs               # Evaluate closures

crates/auto-lang/test/a2c/
├── 103_generic_field/     # Test generic type fields
├── 104_associated_types/  # Test associated type constraints
├── 105_borrow_syntax/     # Test .IterT and self syntax
├── 106_default_impl/      # Test default impl in specs
└── 107_closures/          # Test closure syntax
```

## Integration with Existing Plans

### Plan 051 (Auto Flow) - Primary Beneficiary
**Currently Blocked On**:
- Phase 2: Map/Filter adapters (need generic type fields)
- Iterable auto-forwarding (needs default impls)

**Unblocks After**:
- Phase 1 ✅: Generic type fields → Map/Filter adapters work
- Phase 4 ✅: Default impls → Auto-forwarding works
- Phase 5 ✅: Closures → Functional chaining works naturally

### Plan 057 (Generic Specs) - Enhancement
**Currently Supports**:
- Generic specs with type parameters
- Basic spec implementations

**Needs Enhancement**:
- Associated type constraints (Phase 2)
- Default method implementations (Phase 4)

### Plan 052 (Storage-Based List) - Enhancement
**Currently Supports**:
- `List<T, S>` with storage abstraction

**Will Enable**:
- Generic iterator: `type ListIter<T, S> { list *const List<T, S> }`

### Plan 055 (Environment Injection) - Compatible
No changes needed - works independently.

## Testing Strategy

### Phase 1 Tests

**Test 103: Generic Type Fields**
```auto
use auto.list: List

type Container {
    data List<int>
    items List<List<string>>
}

fn main() {
    let c = Container { data: nil, items: nil }
}
```

**Expected C**:
```c
typedef struct {
    unknown data;
    unknown items;
} Container;
```

### Phase 2 Tests

**Test 104: Associated Type Constraints**
```auto
spec Iterable<T> {
    type IterT impl Iter<T>
    fn iter() IterT
}

type MyIter {
    current int
}

impl Iter<int> for MyIter {
    fn next() May<int> { ... }
}

impl Iterable<int> for MyIter {
    type IterT = MyIter
    fn iter() MyIter { ... }
}
```

### Phase 3 Tests

**Test 105: Borrow Syntax**
```auto
type MyList {
    data int
}

fn iter() .MyList {  // Return borrowed MyList pointer
    return self
}
```

### Phase 4 Tests

**Test 106: Default Implementations**
```auto
spec Container<T> {
    fn size() int
}

impl Container<int> for Container<int> {
    fn size() int { 0 }  // Explicit impl
}

fn test() {
    let c = Container<int> { }
    let s = c.size()  // Uses default if not explicitly impl'd
}
```

### Phase 5 Tests

**Test 107: Closures**
```auto
fn main() {
    let list = List.new()
    list.push(1)
    list.push(2)
    list.push(3)

    // Map using closure
    let doubled = list.iter().map(|x| x * 2)
}
```

## Success Criteria

### Phase 1: Generic Type Fields ✅
- [ ] `type Foo { field List<int> }` parses
- [ ] Nested generics work: `field List<List<int>>`
- [ ] Multiple generic fields
- [ ] Mixed with regular fields
- [ ] a2c test 103 passes

### Phase 2: Associated Types ✅
- [ ] `type IterT impl Iter<T>` parses
- [ ] Constraint checking at compile time
- [ ] Multiple constraints supported
- [ ] a2c test 104 passes

### Phase 3: Borrow Syntax ✅
- [ ] `.IterT` parses as type
- [ ] `.field` parses as access
- [ ] Return type transpiles correctly
- [ ] a2c test 105 passes

### Phase 4: Default Implementations ✅
- [ ] Default methods in specs parse
- [ ] Default used when no explicit impl
- [ ] Explicit impl overrides default
- [ ] a2c test 106 passes

### Phase 5: Closures ✅
- [ ] `|x| x * 2` syntax parses
- [ ] Multi-parameter closures: `|x, y| x + y`
- [ ] Variable capture works
- [ ] Type inference for params/return
- [ ] a2c test 107 passes

## Timeline Summary

| Phase | Duration | Dependencies | Status |
|-------|----------|-------------|--------|
| Phase 1 | 6-10 hours | None | Ready |
| Phase 2 | 4-6 hours | Phase 1 | Ready |
| Phase 3 | 4-6 hours | Phase 2 | Ready |
| Phase 4 | 6-8 hours | Phase 2 | Ready |
| Phase 5 | 8-12 hours | None | Ready |
| **Total** | **28-42 hours** | | |

## Risks and Mitigations

### Risk 1: Parser Complexity

**Impact**: High - Parser is complex and easy to break

**Mitigation**:
- Extensive testing with a2c tests before/after changes
- Add debug logging to understand parsing flow
- Create minimal test cases for each feature
- Test parser regression with existing test suite

### Risk 2: C Transpilation

**Impact**: High - C is low-level and unforgiving

**Mitigation**:
- Design C representation before implementing
- Test with simple cases first
- Verify memory safety (no leaks, use-after-free)
- Ensure thread safety if relevant

### Risk 3: Type Inference

**Impact**: Medium - Type inference can be complex

**Mitigation**:
- Start with explicit type annotations required
- Add inference incrementally
- Clear error messages when inference fails
- Document inference rules clearly

### Risk 4: Closure Capture

**Impact**: Medium - Variable capture has edge cases

**Mitigation**:
- Start with by-value capture (copy)
- Add by-reference capture later (&)
- Clearly document capture semantics
- Test captured variable lifetime

## Future Enhancements (Beyond This Plan)

1. **Generic Constraints**: `where T: Clone` syntax
2. **Higher-Kinded Types**: `type Functor<F>` where F is a type constructor
3. **Existential Types**: `type Foo = for<'a> Fn<'a>(&'a str) -> bool`
4. **Trait Objects**: Dynamic dispatch via trait objects
5. **Generic Methods on impls**: `impl<T> List<T> where T: Display`

## Related Plans

- **Plan 051**: Auto Flow - Primary beneficiary of all phases
- **Plan 057**: Generic Specs - Enhanced by Phases 2, 4
- **Plan 052**: Storage-Based List - Enhanced by Phase 1
- **Plan 055**: Environment Injection - No changes needed
- **Plan 058**: Type Aliases - Independent, but complementary

## Status

**📋 READY FOR INVESTIGATION**

This plan provides a systematic approach to implementing critical type system features needed for modern language features. The investigation in Phase 1 is the highest priority as it unblocks Plan 051 and enables future iterator work.

**Recommended Starting Point**: Phase 1 Investigation (2-3 hours)
- Understand the parsing issue
- Identify root cause
- Propose fix
- Test with minimal example

**After Phase 1**: Can proceed with implementation or document findings for future work.

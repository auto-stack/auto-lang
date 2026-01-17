# May<T> Design Feasibility Analysis

**Date**: 2025-01-17
**Author**: Claude Code
**Related Documents**:
- [May Type Design Document](../design/may-type.md)
- [Plan 027: Standard Library C Foundation](../plans/027-stdlib-c-foundation.md)

---

## Executive Summary

The updated May<T> design is **technically feasible** but requires **significant parser, evaluator, and transpiler work** to implement the syntactic sugar features (`?T`, `.?`, `??`). The core `tag May<T>` type system is well-designed and aligns with modern language features (Rust's `enum`, Swift's `enum`).

**Key Finding**: The design document specifies three distinct layers:
1. **Type system** (`tag May<T>`) - Feasible, requires tag type support
2. **Syntactic sugar** (`?T` for `May<T>`) - Feasible, requires parser changes
3. **Operators** (`.?` propagation, `??` coalescing) - Complex, requires early return semantics

**Recommendation**: Implement in **3 phases**:
1. **Phase 1**: Tag type system + basic May<T> (no syntax sugar)
2. **Phase 2**: `?T` syntactic sugar in parser
3. **Phase 3**: `.?` and `??` operators with early return

**Estimated Timeline**: 6-8 weeks (not 4 weeks as originally planned)

---

## 1. Design Analysis

### 1.1 Core Type Definition

**Design Document Specification**:
```auto
tag May<T> {
    nil Nil
    err Err
    val T
}
```

**Feasibility**: ✅ **FEASIBLE**

**Analysis**:
- The `tag` syntax is already defined in AutoLang's language specification
- AST support exists: `crates/auto-lang/src/ast/tag.rs`
- This is a discriminated union (sum type), similar to Rust's `enum`
- C transpilation is straightforward: `enum` for tag + `union` for payload

**C Translation**:
```c
typedef enum {
    May_Nil = 0x00,
    May_Err = 0x02,
    May_Val = 0x01
} MayTag;

typedef struct {
    MayTag tag;
    union {
        void* nil;     // Unused, for completeness
        void* err;     // Error payload (pointer or error code)
        T val;         // Value payload (generic)
    } data;
} May;
```

**Implementation Requirements**:
1. ✅ Tag type syntax in AST - Already exists
2. ⏸️ Tag type support in parser - Partially implemented
3. ⏸️ Tag type support in evaluator - Needs implementation
4. ⏸️ Tag type transpilation to C - Needs implementation in a2c

### 1.2 Syntactic Sugar: `?T`

**Design Document Specification**:
```auto
// These are equivalent:
let x: May<int> = May_value(42)
let x: ?int = May_value(42)

fn get_value() ?str { ... }
```

**Feasibility**: ✅ **FEASIBLE** (requires parser work)

**Analysis**:
- Lexer needs to recognize `?` as a prefix operator in type contexts
- Parser needs to expand `?T` → `May<T>` during type parsing
- This is a purely syntactic transformation, no runtime changes needed

**Implementation Requirements**:
1. **Lexer changes** (`crates/auto-lang/src/lexer.rs`):
   - Add `TokenKind::Question` for `?` token (already exists)
   - Distinguish between prefix `?` (type context) and postfix `?.` (operator)

2. **Parser changes** (`crates/auto-lang/src/parser.rs`):
   - In `parse_type()` function, detect `?T` pattern
   - Expand `?T` to `May<T>` automatically
   - Update AST to store expanded form

**Example Implementation**:
```rust
// In parser.rs, parse_type() function
fn parse_type(&mut self) -> AutoResult<Type> {
    if self.is_kind(TokenKind::Question) {
        self.next(); // Consume '?'
        let inner = self.parse_type()?;
        return Ok(Type::May(Box::new(inner))); // New Type variant
    }
    // ... rest of type parsing
}
```

### 1.3 Creation Functions

**Design Document Specification**:
```auto
fn some(v T) ?T
fn nil() ?T
fn err(e Err) ?T
```

**Feasibility**: ✅ **FEASIBLE** (already implemented as May_value, May_empty, May_error)

**Analysis**:
- These are already implemented as `May_value()`, `May_empty()`, `May_error()`
- Just need to add `some()`, `nil()`, `err()` as aliases
- No parser/evaluator changes needed

**Implementation**: Add to `crates/auto-lang/src/libs/may.rs`:
```rust
pub fn some(args: &Args) -> Value {
    may_value(args)
}

pub fn nil(args: &Args) -> Value {
    may_empty(args)
}

pub fn err(args: &Args) -> Value {
    may_error(args)
}
```

### 1.4 Methods on `?T`

**Design Document Specification**:
```auto
ext ?T {
    fn is_some() bool
    fn is_nil() bool
    fn is_err() bool
}
```

**Feasibility**: ⚠️ **FEASIBLE** (requires ext statement support for generic types)

**Analysis**:
- `ext` statement support exists (Plan 035)
- Currently implemented as global functions: `May_is_empty()`, `May_is_value()`, `May_is_error()`
- Need to convert to method syntax and add `is_some()` alias

**Current Status**:
- ✅ Plan 035 implemented `ext` statements
- ✅ `ext ?T` syntax is supported
- ⏸️ Need to add methods to `?T` type

**Implementation**:
```auto
// In stdlib/may/may.at
ext ?T {
    fn is_some() bool {
        // Call existing May_is_value
        May_is_value(this)
    }

    fn is_nil() bool {
        May_is_empty(this)
    }

    fn is_err() bool {
        May_is_error(this)
    }
}
```

### 1.5 `.?` Propagation Operator

**Design Document Specification**:
```auto
fn get_first_line(path str) ?str {
    let line = File.open(path).?.readline().?
    return line.view
}

// Compiler expands to:
let _tmp1 = File.open(path)
if _tmp1.is_err() or _tmp1.is_nil() {
    return _tmp1  // Early return
}
let _tmp2 = _tmp1.unwrap()
let _tmp3 = _tmp2.readline()
if _tmp3.is_err() or _tmp3.is_nil() {
    return _tmp3
}
let line = _tmp3.unwrap()
```

**Feasibility**: ⚠️ **COMPLEX** (requires early return semantics)

**Analysis**:
- **Major complexity**: Early return from function body
- Requires parser to detect `.?` operator
- Requires AST transformation to inject if-checks
- Requires context awareness (current function's return type)
- Requires C transpiler support

**Implementation Challenges**:

1. **Parser complexity**:
   - Need to distinguish `x.?` (method call) from `x.?.y` (propagation)
   - Need to track function return types during parsing
   - Need to generate early return code

2. **Evaluator complexity**:
   - Need to implement early return semantics
   - Need to check if current function returns `?T`
   - Need to handle non-`?T` return types as compile error

3. **C transpiler complexity**:
   - Need to generate early return C code
   - Need to handle temporary variables correctly
   - Need to preserve error values through returns

**Example C Transpilation**:
```c
// Auto source:
fn read_file(path str) ?str {
    let file = File.open(path).?
    let content = File.read(file).?
    return content
}

// Generated C:
May_str read_file(AutoStr path) {
    May _tmp1 = File_open(path);
    if (_tmp1.tag != May_Val) {
        return _tmp1;  // Early return
    }
    File file = _tmp1.data.val;

    May _tmp2 = File_read(file);
    if (_tmp2.tag != May_Val) {
        return _tmp2;  // Early return
    }
    return _tmp2;
}
```

**Implementation Requirements**:
1. ✅ Lexer support for `?.` token sequence (already exists)
2. ⏸️ Parser to detect `.?` vs `method()` call
3. ⏸️ AST transformation for early return
4. ⏸️ Evaluator to implement early return
5. ⏸️ C transpiler to generate early return code

### 1.6 `??` Null-Coalescing Operator

**Design Document Specification**:
```auto
let age = get_age().? ?? 18

// Compiler expands to:
let _tmp = get_age().?
if _tmp.is_some() {
    let age = _tmp.unwrap()
} else {
    let age = 18
}
```

**Feasibility**: ✅ **FEASIBLE** (simpler than `.?` operator)

**Analysis**:
- Simpler than `.?` because no early return needed
- Just syntactic sugar for if-else expression
- Parser can transform to ternary or if-else

**Example C Transpilation**:
```c
// Auto source:
let age = get_age().? ?? 18

// Generated C:
May _tmp = get_age_propagated();
int age = (_tmp.tag == May_Val) ? _tmp.data.val : 18;
```

**Implementation Requirements**:
1. ✅ Lexer support for `??` token (already exists as double question)
2. ⏸️ Parser to recognize `??` binary operator
3. ⏸️ Parser to transform to if-else expression
4. ⏸️ C transpiler to generate ternary operator

### 1.7 Pattern Matching with `is`

**Design Document Specification**:
```auto
let t = some(5)

is t {
    nil => {print("t is nil!")}
    err(e) => {print(`error: $e`)}
    n => {print(n)}
}
```

**Feasibility**: ✅ **FEASIBLE** (already implemented)

**Analysis**:
- `is` statement for pattern matching already exists
- Support for tag variant matching exists
- Binding variables in patterns (`err(e)`) works

**Current Status**:
- ✅ `is` statement implemented
- ✅ Pattern matching implemented
- ✅ Tag variant matching works

**No additional work needed.**

---

## 2. Implementation Phases

### Phase 1: Tag Type System + Basic May<T> (2-3 weeks)

**Objective**: Implement `tag May<T>` without syntactic sugar

**Deliverables**:
1. ✅ Tag type parsing support
2. ✅ Tag type evaluation support
3. ✅ Tag type C transpilation
4. ✅ Basic May<T> implementation using `tag` syntax
5. ✅ Methods: `is_some()`, `is_nil()`, `is_err()`
6. ✅ Functions: `some()`, `nil()`, `err()`
7. ✅ Pattern matching with `is`

**Files to Modify**:
1. `crates/auto-lang/src/parser.rs` - Tag type parsing
2. `crates/auto-lang/src/eval.rs` - Tag type evaluation
3. `crates/auto-lang/src/trans/c.rs` - Tag type C transpilation
4. `stdlib/may/may.at` - Rewrite using `tag` syntax

**Success Criteria**:
```auto
// This should work:
tag May<T> {
    nil Nil
    err Err
    val T
}

let x = May.val(42)
is x {
    nil => print("nil"),
    err(e) => print(f"error: $e"),
    val(v) => print(f"value: $v")
}
```

### Phase 2: `?T` Syntactic Sugar (1-2 weeks)

**Objective**: Add `?T` as shorthand for `May<T>`

**Deliverables**:
1. ✅ Lexer recognizes `?` in type context
2. ✅ Parser expands `?T` → `May<T>`
3. ✅ Type checker validates `?T` usage
4. ✅ C transpiler handles `?T` types

**Files to Modify**:
1. `crates/auto-lang/src/lexer.rs` - Type context detection
2. `crates/auto-lang/src/parser.rs` - Type expansion
3. `crates/auto-lang/src/trans/c.rs` - C type generation

**Success Criteria**:
```auto
// This should work:
fn divide(a int, b int) ?int {
    if b == 0 {
        return err("division by zero")
    }
    return some(a / b)
}

let x: ?int = some(42)
```

### Phase 3: `.?` and `??` Operators (2-3 weeks)

**Objective**: Implement propagation and coalescing operators

**Deliverables**:
1. ✅ `.?` operator with early return
2. ✅ `??` operator with default values
3. ✅ Parser transformations for both operators
4. ✅ Evaluator early return semantics
5. ✅ C transpiler support

**Files to Modify**:
1. `crates/auto-lang/src/parser.rs` - Operator detection
2. `crates/auto-lang/src/eval.rs` - Early return
3. `crates/auto-lang/src/trans/c.rs` - C code generation

**Success Criteria**:
```auto
// This should work:
fn read_file(path str) ?str {
    let file = File.open(path).?
    let content = File.read(file).?
    return content
}

let age = get_age().? ?? 18
```

---

## 3. Technical Challenges

### Challenge 1: Early Return Semantics (`.?` operator)

**Problem**: The `.?` operator needs to trigger early return from the enclosing function.

**Solution Options**:

**Option A**: AST transformation (preferred)
- Transform `.?` into if-check during parsing
- Generate explicit early return code
- Pros: Explicit, easy to debug
- Cons: Complex AST transformation

**Option B**: Runtime exception
- Throw exception when `.?` encounters nil/err
- Catch at function boundary
- Pros: Simpler parser
- Cons: Performance overhead, not C-like

**Recommendation**: Option A (AST transformation)

### Challenge 2: Type Context Detection (`?` token)

**Problem**: Lexer needs to distinguish `?T` (type context) from `x.?` (operator).

**Solution**:
- Parser-level distinction, not lexer-level
- Lexer always emits `Question` token
- Parser determines context based on position
- If `?` precedes a type, it's type context
- If `.?` follows expression, it's propagation operator

### Challenge 3: Generic Type Transpilation to C

**Problem**: C doesn't have generics, but `May<T>` is generic.

**Current Solution**:
- Use monomorphization (generate separate C struct for each T)
- Example: `May_int`, `May_str`, `May_File`

**Future Enhancement**:
- Use `void*` for value payload (current approach)
- Type erasure with runtime checks

### Challenge 4: Niche Optimization

**Problem**: Design document mentions "0-byte overhead for pointer types" using niche optimization.

**Feasibility**: ⚠️ **NOT IMMEDIATELY FEASIBLE**

**Analysis**:
- Niche optimization requires special pointer values (0x0, 0x1) to represent states
- This is an advanced compiler optimization
- Rust implements this, but it's complex
- Should be deferred to future optimization phase

**Recommendation**: Start with standard tag+union layout, optimize later.

---

## 4. Dependencies

### Required Before Starting:

1. ✅ **Plan 024** (Ownership System) - COMPLETE
2. ✅ **Plan 025** (String Type Redesign) - COMPLETE
3. ⏸️ **Tag type support in parser** - PARTIAL (needs completion)
4. ⏸️ **Tag type support in evaluator** - NEEDED
5. ⏸️ **Tag type C transpilation** - NEEDED

### Blocking Issues:

1. **Tag type parsing incomplete**: Parser can parse `tag` definitions but doesn't fully support all features
2. **Tag type evaluation missing**: Evaluator doesn't support tag variant construction and matching
3. **C transpiler incomplete**: a2c doesn't transpile `tag` types to C enum+union

### Unblocking Plan:

**Week 1**: Complete tag type parsing and evaluation
- Tag variant construction
- Tag pattern matching
- Tag methods

**Week 2**: Implement tag type C transpilation
- Generate C enums for tags
- Generate C unions for payloads
- Test round-trip conversion

**Week 3+**: Begin May<T> implementation once tag types are working

---

## 5. Risk Assessment

### High Risk Items:

1. **`.?` operator complexity** (Risk: High)
   - Early return semantics are complex
   - Requires significant parser/evaluator work
   - Mitigation: Implement in stages, start with basic version

2. **Tag type C transpilation** (Risk: Medium)
   - C doesn't have native sum types
   - Need to generate correct enum+union code
   - Mitigation: Start with simple cases, test thoroughly

### Medium Risk Items:

1. **Generic type support** (Risk: Medium)
   - C doesn't have generics
   - Need monomorphization or void*
   - Mitigation: Use void* initially, optimize later

2. **Performance of `.?` operator** (Risk: Medium)
   - May generate lots of temporary variables
   - Could impact code size
   - Mitigation: Benchmark, optimize if needed

### Low Risk Items:

1. **`??` operator** (Risk: Low)
   - Simple syntactic sugar
   - Easy to implement

2. **`?T` syntax** (Risk: Low)
   - Simple text substitution
   - No runtime impact

---

## 6. Recommendations

### Immediate Actions (Week 1-2):

1. **Complete tag type support** (blocking May<T>)
   - Finish tag type parser
   - Implement tag type evaluator
   - Implement tag type C transpilation

2. **Test tag types thoroughly**
   - Create test cases for tag variants
   - Test pattern matching
   - Test C transpilation

### Short-term (Week 3-4):

3. **Implement basic May<T> without syntax sugar**
   - Use `tag May<T>` syntax
   - Implement `some()`, `nil()`, `err()` functions
   - Implement `is_some()`, `is_nil()`, `is_err()` methods
   - Test thoroughly

### Medium-term (Week 5-6):

4. **Add `?T` syntactic sugar**
   - Parser expansion
   - C transpilation support
   - Testing

### Long-term (Week 7-8):

5. **Implement `.?` and `??` operators**
   - Start with `??` (simpler)
   - Then implement `.?` (more complex)
   - Comprehensive testing

### Defer to Future:

6. **Niche optimization** (Phase 2 optimization)
7. **Error message linking system** (Phase 2 feature)
8. **Rich error types** (Phase 2 feature)

---

## 7. Conclusion

The May<T> design is **technically feasible** and aligns well with AutoLang's architecture. However, the implementation will take **6-8 weeks**, not the 4 weeks originally estimated in Plan 027.

**Key Success Factors**:
1. Complete tag type support first (blocking dependency)
2. Implement in phases (basic → syntax sugar → operators)
3. Test thoroughly at each phase
4. Defer advanced features (niche optimization) to future

**Next Steps**:
1. Update Plan 027 with revised timeline (6-8 weeks instead of 4)
2. Break Phase 1b into 3 sub-phases
3. Start with tag type completion (blocking dependency)
4. Implement basic May<T> before adding syntax sugar

**Feasibility Verdict**: ✅ **FEASIBLE** (with revised timeline)

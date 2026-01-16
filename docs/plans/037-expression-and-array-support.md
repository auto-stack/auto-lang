# Plan 037: Expression and Array Support

## Implementation Status: 📝 PLANNED

**Last Updated**: 2025-01-16

**Priority:** HIGH - Unblocks Plan 036 Phase 4 completion and enables advanced stdlib methods

**Dependencies:**
- ✅ Plan 001 (VM Function Integration)
- ✅ Plan 035 (ext Statement)
- ✅ Plan 036 Phase 1-3 (Stdlib file organization)

**Estimated Start:** Immediately after Plan 036 Phase 4
**Timeline:** 6-10 weeks
**Complexity:** High (parser, type system, and code generation changes)

## Executive Summary

Add comprehensive support for complex expressions and array operations to the AutoLang parser. These foundational features are currently blocking the implementation of advanced stdlib methods like `split()`, `join()`, `trim()`, `File.read_all()`, and `File.write_lines()`.

**Current Limitations:**
- ❌ Complex expressions in conditions (`i < len`, `x + 1 > 5`)
- ❌ Method chaining (`self.sub(i, i+1)`)
- ❌ Array indexing (`arr[i]`, `lines[0]`)
- ❌ Array return types (`fn split() [str]`)
- ❌ Static methods (`static fn join()`)

**Target State:**
- ✅ Binary expressions in all contexts
- ✅ Array indexing with bounds checking
- ✅ Methods returning arrays
- ✅ Static method declarations and calls
- ✅ Method chaining on expressions

**Impact:**
- Unblocks Plan 036 Phase 4 completion
- Enables advanced string manipulation methods
- Improves language expressiveness for all code
- Required for self-hosting compiler (Plan 033)

---

## 1. Problem Analysis

### 1.1 Current Parser Limitations

**Example 1: Cannot use complex expressions in while conditions**
```auto
// What we want to write:
fn read_all() str {
    let mut result = ""
    while i < len {  // ❌ Parser error: unexpected Lt
        // ...
    }
}

// Current workaround:
fn read_all() str {
    let mut result = ""
    let mut eof = is_eof()
    while eof {  // ✅ Works, but awkward
        eof = is_eof()
    }
}
```

**Example 2: Cannot use array indexing**
```auto
// What we want to write:
fn write_lines(lines []str) {
    while i < lines.len() {
        self.write_line(lines[i])  // ❌ Parser error: unexpected LBracket
        i = i + 1
    }
}

// Current workaround: Not possible
```

**Example 3: Cannot return arrays from methods**
```auto
// What we want to write:
fn split(delimiter str) [str] {  // ❌ Parser error: unexpected LBracket
    let result = []
    // ...
    result
}
```

**Example 4: Cannot chain method calls**
```auto
// What we want to write:
fn trim() str {
    let end = self.len() - 1  // ❌ Parser error
    self.sub(0, end)  // ❌ Cannot use expressions as arguments
}

// Current workaround:
fn trim() str {
    let end = .size
    let pos = end - 1
    .sub(0, end)
}
```

### 1.2 Root Causes

**Parser Issues:**
1. **Expression parsing** - Binary expressions only work in limited contexts
2. **Array indexing** - No support for `expr[index]` syntax
3. **Array types** - Parser doesn't handle `[T]` as return type
4. **Method calls** - Cannot call methods on complex expressions

**Type System Issues:**
1. **Array type representation** - `[T]` not fully integrated
2. **Type inference for arrays** - Cannot infer array literal types
3. **Generic array types** - No support for `[]str` in method signatures

**Code Generation Issues:**
1. **Array operations** - No code generation for indexing
2. **Bounds checking** - No runtime checks for array access
3. **Array allocation** - No support for dynamic array creation

---

## 2. Implementation Strategy

### 2.1 Design Principles

1. **Incremental Implementation**: Add features in phases, each phase is testable
2. **Backward Compatibility**: All changes must not break existing code
3. **Comprehensive Testing**: Each feature needs unit tests and integration tests
4. **Performance**: Array operations must be efficient (bounds checking optimization)
5. **Safety**: Runtime bounds checking for all array access

### 2.2 Phase Breakdown

**Phase 1**: Complex Expression Support (2-3 weeks)
**Phase 2**: Array Indexing Operations (2-3 weeks)
**Phase 3**: Array Return Types (1-2 weeks)
**Phase 4**: Static Methods (1-2 weeks)

---

## 3. Phase 1: Complex Expression Support

**Objective**: Enable binary expressions in all contexts (conditions, arguments, etc.)

**Dependencies**: None
**Risk**: Medium
**Timeline**: 2-3 weeks

### 3.1 Current State

**Works:**
```auto
let x = 1 + 2  // ✅ Binary expressions in assignments
let y = x * 3  // ✅
if x > 0 {     // ✅ Binary expressions in if conditions
    print("positive")
}
```

**Doesn't Work:**
```auto
while i < len {        // ❌ Binary expressions in while
    // ...
}

for i in 0..arr.len() { // ❌ Binary expressions in range
    // ...
}

fn foo(x int, y int) int {
    return x + y       // ✅ Works in return
}
```

### 3.2 Implementation Tasks

#### 3.2.1 Parser Changes

**File**: `crates/auto-lang/src/parser.rs`

**Current limitation**: The parser only allows binary expressions in certain contexts.

**Required changes:**

1. **Update `parse_while_statement()`** to accept binary expressions:
```rust
fn parse_while_statement(&mut self) -> AutoResult<Stmt> {
    self.expect(TokenKind::While)?;
    self.expect(TokenKind::LParen)?;

    // OLD: Only simple identifiers
    // let cond = self.parse_ident()?;

    // NEW: Full expression support
    let cond = self.parse_expr()?;  // ← Changed

    self.expect(TokenKind::RParen)?;
    let body = self.parse_block()?;

    Ok(Stmt::While { cond, body })
}
```

2. **Update `parse_for_range()`** to accept expressions in range bounds:
```rust
fn parse_for_range(&mut self) -> AutoResult<Stmt> {
    self.expect(TokenKind::For)?;
    let var = self.parse_ident()?;
    self.expect(TokenKind::In)?;

    let start = self.parse_expr()?;  // ← Changed (was parse_primary)

    self.expect(TokenKind::DotDot)?;

    let end = self.parse_expr()?;    // ← Changed (was parse_primary)

    // ...
}
```

3. **Allow expression evaluation in all statement contexts**

#### 3.2.2 Expression Grammar Enhancement

**Current grammar (simplified):**
```
while_stmt ::= "while" "(" ident ")" block
for_range ::= "for" ident "in" primary ".." primary block
```

**New grammar:**
```
while_stmt ::= "while" "(" expr ")" block
for_range ::= "for" ident "in" expr ".." expr block
```

### 3.3 Test Cases

**File**: `crates/auto-lang/test/a2c/037_complex_expr/while_expr.at`

```auto
fn test_while_with_expr() {
    let mut i = 0
    let max = 10
    while i < max {
        print(i)
        i = i + 1
    }
}
```

**File**: `crates/auto-lang/test/a2c/037_complex_expr/for_range_expr.at`

```auto
fn test_for_with_expr() {
    let len = 10
    for i in 0..len {
        print(i)
    }
}
```

**File**: `crates/auto-lang/test/a2c/037_complex_expr/complex_bool.at`

```auto
fn test_complex_bool() {
    let x = 5
    let y = 10
    if x > 0 && y < 20 {
        print("both true")
    }
}
```

### 3.4 Success Criteria

- [ ] `while` statements accept binary expressions
- [ ] `for` range bounds support expressions
- [ ] Logical operators (`&&`, `||`) work in all contexts
- [ ] 20+ new tests passing
- [ ] All existing tests still passing

---

## 4. Phase 2: Array Indexing Operations

**Objective**: Add support for array indexing syntax and operations

**Dependencies**: Phase 1
**Risk**: High
**Timeline**: 2-3 weeks

### 4.1 Current State

**Arrays work in limited contexts:**
```auto
let arr = [1, 2, 3]  // ✅ Array literal
let x = arr[0]        // ❌ Doesn't work - no indexing
```

**Existing array support:**
- Array literals: `[1, 2, 3]`
- Array types: `[int]`, `[str]`
- Array parameters: `fn foo(arr []int)`

**Missing:**
- Array indexing: `arr[i]`
- Array methods: `arr.len()`
- Array bounds checking

### 4.2 Implementation Tasks

#### 4.2.1 Parser Changes

**File**: `crates/auto-lang/src/parser.rs`

**Add array indexing to primary expressions:**

```rust
fn parse_primary(&mut self) -> AutoResult<Expr> {
    // ... existing code for literals, identifiers, ...

    // NEW: Array indexing
    if self.is_kind(TokenKind::LBracket) {
        self.next(); // consume [
        let index = self.parse_expr()?;
        self.expect(TokenKind::RBracket)?;

        // Return index expression
        return Ok(Expr::Index {
            base: Box::new(expr),
            index: Box::new(index),
        });
    }

    // ... rest of function
}
```

**Update AST if needed:**

```rust
// File: crates/auto-lang/src/ast/expr.rs

pub enum Expr {
    // ... existing variants ...

    // NEW or UPDATE:
    Index {
        base: Box<Expr>,  // The array being indexed
        index: Box<Expr>,  // The index expression
    },
}
```

#### 4.2.2 Type Checking

**File**: `crates/auto-lang/src/infer/` or parser type inference

**Add type checking for index expressions:**

```rust
fn infer_index_expr(&mut self, base: &Expr, index: &Expr) -> Type {
    // Infer base type
    let base_ty = self.infer_expr(base)?;

    // Check if base is an array
    let (elem_ty, len) = match base_ty {
        Type::Array { elem, len } => (elem, len),
        _ => return Err(TypeError::InvalidIndexType {
            ty: base_ty,
            span: pos_to_span(base.pos()),
        }.into()),
    };

    // Check index type is int
    let index_ty = self.infer_expr(index)?;
    if !matches!(index_ty, Type::Int | Type::Uint) {
        return Err(TypeError::InvalidIndexType {
            ty: index_ty,
            span: pos_to_span(index.pos()),
        }.into());
    }

    // Return element type
    *elem_ty
}
```

#### 4.2.3 Evaluator Implementation

**File**: `crates/auto-lang/src/eval.rs`

**Add evaluation for index expressions:**

```rust
fn eval_index_expr(&mut self, base: &Expr, index: &Expr) -> AutoResult<Value> {
    let base_val = self.eval_expr(base)?;
    let index_val = self.eval_expr(index)?;

    let (arr, index_int) = match (base_val, index_val) {
        (Value::Array(arr), Value::Int(i)) => (arr, i),
        _ => return Err(RuntimeError::InvalidIndexOperation.into()),
    };

    // Bounds checking
    if index_int < 0 || index_int >= arr.len() as i64 {
        return Err(RuntimeError::IndexOutOfBounds {
            index: index_int,
            len: arr.len(),
            span: pos_to_span(index.pos()),
        }.into());
    }

    Ok(arr[index_int as usize].clone())
}
```

#### 4.2.4 C Transpiler Implementation

**File**: `crates/auto-lang/src/trans/c.rs`

**Add C code generation for index expressions:**

```rust
fn transpile_expr(&mut self, expr: &Expr) -> AutoResult<String> {
    match expr {
        // ... existing cases ...

        Expr::Index { base, index } => {
            let base_str = self.transpile_expr(base)?;
            let index_str = self.transpile_expr(index)?;

            // Generate array access with bounds checking
            Ok(format!(
                "array_get(&{}, {})",  // Assuming helper function
                base_str,
                index_str
            ))
        },

        // ... rest of cases ...
    }
}
```

**Add helper function to generated C code:**

```c
// Generated C helper for array access
T* array_get(Array(T)* arr, int index) {
    if (index < 0 || index >= arr->len) {
        fprintf(stderr, "Index out of bounds: %d\\n", index);
        exit(1);
    }
    return &arr->data[index];
}
```

### 4.3 Test Cases

**File**: `crates/auto-lang/test/a2c/037_array_index/basic_index.at`

```auto
fn test_basic_index() {
    let arr = [1, 2, 3, 4, 5]
    let x = arr[0]
    let y = arr[2]
    print(x)  // 1
    print(y)  // 3
}

fn test_index_with_variable() {
    let arr = [10, 20, 30]
    let i = 1
    let val = arr[i]
    print(val)  // 20
}

fn test_index_in_loop() {
    let arr = [1, 2, 3, 4, 5]
    let mut i = 0
    while i < 5 {
        print(arr[i])
        i = i + 1
    }
}
```

**File**: `crates/auto-lang/test/a2c/037_array_index/bounds_check.at`

```auto
fn test_bounds_check() {
    let arr = [1, 2, 3]
    let x = arr[5]  // Should trigger bounds check error
}
```

### 4.4 Success Criteria

- [ ] Array indexing syntax `arr[i]` works
- [ ] Index can be any expression (`arr[i+1]`, `arr[fn()`)
- [ ] Runtime bounds checking implemented
- [ ] Out-of-bounds access produces clear error
- [ ] 30+ tests passing
- [ ] C transpiler generates correct code
- [ ] Evaluator correctly handles indexing

---

## 5. Phase 3: Array Return Types

**Objective**: Enable methods to return array types

**Dependencies**: Phase 2
**Risk**: Medium
**Timeline**: 1-2 weeks

### 5.1 Current State

**Doesn't work:**
```auto
fn split(delimiter str) [str] {  // ❌ Parser error
    let result = []
    result
}
```

**Workaround:** Not possible

### 5.2 Implementation Tasks

#### 5.2.1 Parser Changes

**File**: `crates/auto-lang/src/parser.rs`

**Update function return type parsing:**

```rust
fn parse_fn_return(&mut self) -> AutoResult<Option<Type>> {
    // ... existing code ...

    // NEW: Support array types
    if self.is_kind(TokenKind::LBracket) {
        self.next(); // consume [
        let elem = self.parse_type()?;
        self.expect(TokenKind::RBracket)?;
        return Ok(Some(Type::Array {
            elem: Box::new(elem),
            len: None,  // Unknown length for return type
        }));
    }

    // ... rest of function
}
```

#### 5.2.2 Type System Updates

**File**: `crates/auto-lang/src/ast/types.rs` or related

**Ensure Type::Array supports dynamic length:**

```rust
pub enum Type {
    // ... existing variants ...

    Array {
        elem: Box<Type>,
        len: Option<usize>,  // Some(len) for fixed, None for dynamic
    },
}
```

#### 5.2.3 Test Cases

**File**: `crates/auto-lang/test/a2c/037_array_return/basic.at`

```auto
fn get_numbers() [int] {
    [1, 2, 3, 4, 5]
}

fn test_array_return() {
    let nums = get_numbers()
    print(nums[0])  // 1
    print(nums[4])  // 5
}

fn identity(arr [int]) [int] {
    arr
}
```

**File**: `crates/auto-lang/test/a2c/037_array_return/string_array.at`

```auto
fn get_words() [str] {
    ["hello", "world"]
}

fn test_string_array() {
    let words = get_words()
    print(words[0])  // hello
    print(words[1])  // world
}
```

### 5.3 Success Criteria

- [ ] Functions can return array types
- [ ] Array types work as parameters
- [ ] Type inference works for array returns
- [ ] 15+ tests passing
- [ ] C transpiler handles array returns

---

## 6. Phase 4: Static Methods

**Objective**: Add support for static method declarations and calls

**Dependencies**: Phase 1, 2, 3
**Risk**: Low
**Timeline**: 1-2 weeks

### 6.1 Current State

**Doesn't work:**
```auto
static fn join(parts [str], delimiter str) str {  // ❌ Parser error
    // ...
}
```

### 6.2 Implementation Tasks

#### 6.2.1 Parser Changes

**File**: `crates/auto-lang/src/parser.rs`

**Add support for `static` keyword in function declarations:**

```rust
fn parse_fn_decl(&mut self) -> AutoResult<Stmt> {
    let is_static = self.is_keyword("static");
    if is_static {
        self.next(); // consume static
    }

    self.expect(TokenKind::Fn)?;
    let name = self.parse_ident()?;

    // Parse generics (if any)
    let generics = self.parse_generics()?;

    // Parse parameters
    self.expect(TokenKind::LParen)?;
    let params = self.parse_params()?;
    self.expect(TokenKind::RParen)?;

    // Parse return type
    let return_type = self.parse_fn_return()?;

    // Parse body
    let body = self.parse_block()?;

    Ok(Stmt::Fn {
        name,
        params,
        return_type,
        body,
        is_static,  // ← NEW
    })
}
```

**Update AST:**

```rust
// File: crates/auto-lang/src/ast/stmt.rs

pub struct FnDecl {
    pub name: Name,
    pub params: Vec<Param>,
    pub return_type: Option<Type>,
    pub body: Body,
    pub is_static: bool,  // ← NEW
}
```

#### 6.2.2 Method Call Syntax

**Support both instance and static method calls:**

```auto
// Instance method:
obj.method(arg)     // Current: works

// Static method:
Type::method(arg)   // NEW: need to support
```

**Parser changes:**

```rust
fn parse_call(&mut self) -> AutoResult<Expr> {
    let obj = self.parse_primary()?;

    if self.is_kind(TokenKind::DotDot) {
        // Static method call: Type::method()
        self.next(); // consume ::
        let method = self.parse_ident()?;
        self.expect(TokenKind::LParen)?;
        let args = self.parse_args()?;
        self.expect(TokenKind::RParen)?;

        return Ok(Expr::StaticCall {
            type_name: obj,
            method,
            args,
        });
    }

    if self.is_kind(TokenKind::LParen) {
        // Instance method call: obj.method()
        self.next(); // consume (
        let args = self.parse_args()?;
        self.expect(TokenKind::RParen)?;

        return Ok(Expr::Call {
            obj: Box::new(obj),
            method,
            args,
        });
    }

    Ok(obj)
}
```

### 6.3 Test Cases

**File**: `crates/auto-lang/test/a2c/037_static_method/basic.at`

```auto
type Math {
    static fn add(a int, b int) int {
        a + b
    }

    static fn multiply(a int, b int) int {
        a * b
    }
}

fn test_static_method() {
    let x = Math::add(5, 3)
    let y = Math::multiply(4, 7)
    print(x)  // 8
    print(y)  // 28
}
```

**File**: `crates/auto-lang/test/a2c/037_static_method/str_join.at`

```auto
ext str {
    static fn join(parts [str], delimiter str) str {
        // Implementation after Phase 3
        ""
    }
}

fn test_join() {
    let words = ["hello", "world"]
    let result = str::join(words, " ")
    print(result)  // "hello world"
}
```

### 6.4 Success Criteria

- [ ] `static fn` syntax supported
- [ ] Static method calls `Type::method()` work
- [ ] Static methods can be called without instance
- [ ] 20+ tests passing
- [ ] C transpiler generates correct code

---

## 7. Integration with Plan 036

Once all phases are complete, update Plan 036 Phase 4 with the following methods:

### 7.1 String Methods

```auto
// File: stdlib/auto/str.at

ext str {
    // Split string by delimiter
    fn split(delimiter str) [str] {
        let mut result = []
        let mut current = ""
        let mut i = 0

        while i < self.len() {
            if self.sub(i, i + delimiter.len()) == delimiter {
                result = result.append(current)
                current = ""
                i = i + delimiter.len()
            } else {
                current = current.append(self.char_at(i))
                i = i + 1
            }
        }

        if current.len() > 0 {
            result = result.append(current)
        }

        result
    }

    // Trim whitespace
    fn trim() str {
        let mut start = 0
        let mut end = self.len()

        while start < end {
            let ch = self.char_at(start)
            if ch != " " {
                if ch != "\t" {
                    if ch != "\n" {
                        break
                    }
                }
            }
            start = start + 1
        }

        while end > start {
            let ch = self.char_at(end - 1)
            if ch != " " {
                if ch != "\t" {
                    if ch != "\n" {
                        break
                    }
                }
            }
            end = end - 1
        }

        self.sub(start, end)
    }

    // Split by whitespace
    fn words() [str] {
        self.split(" ")
    }

    // Split by newline
    fn lines() [str] {
        self.split("\n")
    }
}

// Static join method
static fn str::join(parts [str], delimiter str) str {
    if parts.len() == 0 {
        return ""
    }

    let mut result = parts[0]
    let mut i = 1

    while i < parts.len() {
        result = result.append(delimiter)
        result = result.append(parts[i])
        i = i + 1
    }

    result
}
```

### 7.2 File Methods

```auto
// File: stdlib/auto/io.at

type File {
    // ... existing methods ...

    // Read entire file
    fn read_all() str {
        let mut result = ""
        let mut line = self.read_line()
        let mut eof = self.is_eof()

        while eof == false {
            result = result.append(line)
            line = self.read_line()
            eof = self.is_eof()
        }

        result
    }

    // Write multiple lines
    fn write_lines(lines []str) {
        let mut i = 0

        while i < lines.len() {
            self.write_line(lines[i])
            i = i + 1
        }
    }
}
```

---

## 8. Success Criteria (Overall)

### Must Have (MVP)
- [ ] Complex binary expressions in while/for conditions
- [ ] Array indexing with bounds checking
- [ ] Methods returning arrays
- [ ] Static method declarations and calls
- [ ] All 100+ new tests passing
- [ ] All existing tests still passing
- [ ] C transpiler support for all features

### Should Have
- [ ] Array methods (len, append, etc.)
- [ ] Array literals with expressions (`[1+2, 3*4]`)
- [ ] Nested array indexing (`arr[i][j]`)
- [ ] Performance optimizations for array access

### Could Have
- [ ] Multi-dimensional arrays
- [ ] Array slicing (`arr[1:5]`)
- [ ] Array comprehensions
- [ ] Lazy evaluation for array operations

---

## 9. Timeline Summary

| Phase | Duration | Dependencies | Deliverable |
|-------|----------|--------------|-------------|
| Phase 1: Complex Expressions | 2-3 weeks | None | Binary expressions in all contexts |
| Phase 2: Array Indexing | 2-3 weeks | Phase 1 | Array indexing with bounds checking |
| Phase 3: Array Return Types | 1-2 weeks | Phase 2 | Methods returning arrays |
| Phase 4: Static Methods | 1-2 weeks | Phase 1, 2, 3 | Static method support |
| **Total** | **6-10 weeks** | **Sequential** | Full expression and array support |

**Critical Path:** Phase 1 → 2 → 3 → 4 (must be sequential)

---

## 10. Risks and Mitigations

### Risk 1: Breaking Existing Code
**Impact**: High
**Probability**: Medium
**Mitigation**:
- Comprehensive test suite before starting
- Run all tests after each phase
- Backward compatibility checks
- Feature flags for new syntax

### Risk 2: Performance Regression
**Impact**: Medium
**Probability**: Low
**Mitigation**:
- Benchmark array operations
- Optimize bounds checking
- Use compiler optimizations
- Profile before and after

### Risk 3: Complex Interactions
**Impact**: Medium
**Probability**: High
**Mitigation**:
- Implement phases incrementally
- Test each phase thoroughly
- Integration tests at each step
- Clear rollback strategy

### Risk 4: C Transpiler Complexity
**Impact**: Medium
**Probability**: Medium
**Mitigation**:
- Start with evaluator support
- Add C transpilation incrementally
- Reuse existing array infrastructure
- Test generated C code extensively

---

## 11. Next Steps

### Immediate Actions (Week 1)
1. Create comprehensive test suite for current limitations
2. Document exact error messages for each limitation
3. Identify all code locations needing changes
4. Set up feature branch for implementation

### First Month Goals
- Complete Phase 1 (Complex Expressions)
- Start Phase 2 (Array Indexing)
- Have 50+ tests passing

### Full Completion Goals
- All 4 phases complete
- 100+ new tests passing
- Plan 036 Phase 4 methods implemented
- Full documentation

---

## 12. Related Documentation

- [Plan 036: Unified Auto Section](./036-unified-auto-section.md) - Depends on this plan
- [Plan 029: Pattern Matching System](./029-pattern-matching-system.md) - Related expression work
- [Plan 035: ext Statement](./035-ext-statement.md) - Provides context for method syntax
- [Language Specification](../language/specification.md) - Will need updates

---

## 13. Conclusion

This plan addresses foundational parser and type system limitations that are blocking advanced stdlib method implementations. By completing all four phases, AutoLang will have:

1. **Full expression support** in all contexts
2. **Safe array operations** with bounds checking
3. **Flexible return types** including arrays
4. **Static methods** for type-level operations

These features will not only unblock Plan 036 Phase 4 but also benefit the entire language by making it more expressive and safer. The implementation is designed to be incremental, testable, and backward compatible.

**Estimated completion**: 6-10 weeks
**Priority**: HIGH (unblocks multiple other plans)
**Complexity**: High (parser, type system, code generation)

---

**Plan End**

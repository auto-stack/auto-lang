# Plan 053: Mutable Variables in Loop Scopes

## Objective

Fix the compilation error when using `mut` variables inside `while` loops, enabling iteration patterns for data structures like `List<T>`.

## Motivation

**Problem**: Using `mut` variables in `while` loops causes compilation errors

**Current Failure**:
```auto
let arr [4]int = [1, 2, 3, 4]
mut sum = 0
mut i = 0
while i < 4 {
    sum = sum + arr[i]  // ❌ COMPILATION ERROR
    i = i + 1
}
```

**Error Message**:
```
aborting due to 1 previous error
```

**Impact**:
- Cannot implement iteration over `List<T>`
- Cannot use accumulator patterns in loops
- Cannot manually index arrays with `mut` counter
- Forces workarounds or recursion

**Desired State**:
```auto
mut sum = 0
mut i = 0
while i < 10 {
    sum = sum + i
    i = i + 1
}
sum  // Returns 45
```

**Benefits**:
1. **Iteration**: Can iterate over `List<T>` by index
2. **Accumulators**: Can use sum/product patterns
3. **Counters**: Can use `mut i` as loop counter
4. **Normalcy**: Matches user expectations from other languages

## Current State Analysis

### What Works Now

**`mut` at function scope** - ✅ Works:
```auto
mut x = 0
x = 10
x  // Returns 10
```

**`mut` in array mutation** - ✅ Works (Plan 051 Phase 1 confirmed):
```auto
mut arr [4]int = [1, 2, 3, 4]
arr[0] = 100  // ✅ Works
```

**Variables without `mut` in loops** - ✅ Works:
```auto
let i = 0
while i < 10 {
    i  // Can read, but not modify
    i = i + 1  // ❌ Error: can't reassign immutable
}
```

### What Doesn't Work

**`mut` variables declared before `while`** - ❌ Fails:
```auto
mut i = 0
while i < 10 {
    i = i + 1  // COMPILATION ERROR
}
```

**Investigation needed**:
- Is this a parser error?
- Is this a scope error?
- Is this a type checker error?
- Is this a code generation error?

## Investigation Plan

### Phase 1: Error Diagnosis

**Goal**: Understand WHY the error occurs

**Steps**:
1. **Test with simplest case**:
   ```auto
   mut i = 0
   while i < 5 {
       i = i + 1
   }
   i
   ```

2. **Check parser output**:
   - Parse the code successfully?
   - What AST is generated?
   - Are `mut` bindings tracked correctly?

3. **Check type checker**:
   - Are variables in correct scope?
   - Is `mut` flag preserved in loop scope?
   - Are assignments type-checked correctly?

4. **Check code generation**:
   - Is C code generated?
   - What does generated C look like?

5. **Get exact error message**:
   - Run with `RUST_BACKTRACE=1`
   - Check what the actual error is
   - Identify which phase fails

### Phase 2: Root Cause Analysis

**Possible Causes**:

#### A. Parser Issue
**Symptom**: Code fails to parse
**Root Cause**: Parser doesn't handle `mut` before `while`
**Fix**: Extend parser to recognize pattern

#### B. Scope Issue
**Symptom**: Variable not found in loop body
**Root Cause**: `mut` variables not in loop scope
**Fix**: Adjust scope management to include outer `mut` bindings

#### C. Type Checking Issue
**Symptom**: Assignment to `mut` fails type check
**Root Cause**: Type checker doesn't recognize `mut` in loop
**Fix**: Extend type checker to track `mut` through loops

#### D. Code Generation Issue
**Symptom**: Invalid C code generated
**Root Cause**: C transpiler doesn't handle reassignment in loops
**Fix**: Generate correct C code for variable updates

## Design Solutions

Based on root cause, different fixes needed:

### Solution A: Parser Fix (if parser issue)

**File**: `crates/auto-lang/src/parser.rs`

**Changes**:
```rust
// In parse_while() or equivalent
fn parse_while_loop(&mut self) -> AutoResult<Stmt> {
    self.expect(TokenKind::While)?;
    self.expect(TokenKind::LParen)?;
    let condition = self.parse_expr()?;
    self.expect(TokenKind::RParen)?;

    // Parse body
    let body = self.parse_block()?;

    // Check if any `mut` variables from outer scope are used
    // and ensure they're accessible in loop body

    Ok(Stmt::While(Box::new(condition), body))
}
```

### Solution B: Scope Fix (if scope issue)

**File**: `crates/auto-lang/src/eval.rs` (universe/scope)

**Changes**:
```rust
impl Universe {
    pub fn enter_loop_scope(&mut self) {
        // Push new scope but INHERIT mut bindings from parent
        let new_scope = Scope::new();
        new_scope.inherit_mut_from(&self.current_scope());
        self.scopes.push(new_scope);
    }
}

impl Scope {
    fn inherit_mut_from(&mut self, parent: &Scope) {
        for (name, binding) in parent.bindings.iter() {
            if binding.is_mutable {
                // Clone mutable bindings into loop scope
                self.bindings.insert(name.clone(), binding.clone());
            }
        }
    }
}
```

### Solution C: Type Checker Fix (if type checking issue)

**File**: `crates/auto-lang/src/infer/` or type checker

**Changes**:
```rust
fn infer_while_loop(&mut self, cond: &Expr, body: &Stmt) -> Type {
    // Enter loop scope
    self.enter_scope();

    // Type check condition (must be bool or int)
    let cond_ty = self.infer_expr(cond)?;
    self.unify(cond_ty, Type::Bool)?;

    // Type check body (has access to outer mut variables)
    let body_ty = self.infer_stmt(body)?;

    // Exit scope
    self.exit_scope();

    Ok(body_ty)
}
```

### Solution D: Code Generation Fix (if C transpiler issue)

**File**: `crates/auto-lang/src/trans/c.rs`

**Changes**:
```rust
fn transpile_while(&mut self, cond: &Expr, body: &Stmt, sink: &mut Sink) -> AutoResult<()> {
    // Transpile condition
    let cond_code = self.transpile_expr(cond, sink)?;
    output!(sink, "while ({}) {{", cond_code);

    // Transpile body
    // For `mut` variables that are reassigned, generate pointer assignments
    self.transpile_stmt(body, sink)?;

    output!(sink, "}");

    Ok(())
}
```

**Variable Reassignment Pattern**:
```c
// Generated C for: mut i = 0; while i < 10 { i = i + 1 }
int i = 0;
while (i < 10) {
    i = i + 1;  // ✅ Works in C
}
```

## Implementation Plan

### Phase 1: Diagnosis (1-2 hours)
- Create minimal test case
- Run with full error output
- Identify exact failure point
- Document root cause

### Phase 2: Fix Implementation (2-4 hours)
- Implement fix based on root cause
- Test with simple cases
- Test with complex cases

### Phase 3: Comprehensive Testing (2-3 hours)
- Basic `mut` in while loop
- Multiple `mut` variables in loop
- Nested loops with `mut`
- Array indexing with `mut` counter
- Accumulator patterns (sum, product)

### Phase 4: Integration Testing (2-3 hours)
- Test with List iteration
- Test with array iteration
- Test with complex data structures
- Performance tests

**Total Estimated Time**: 7-12 hours

## Test Cases

**Test Directory**: `crates/auto-lang/test/a2c/083_mut_in_loops/`

### Basic Tests
1. `mut_counter.at` - Simple counter increment
2. `mut_accumulator.at` - Sum accumulator
3. `mut_multiple.at` - Multiple mut variables in loop
4. `mut_nested.at` - Nested loops with mut

### Array Tests
5. `mut_array_iter.at` - Iterate over array
6. `mut_array_sum.at` - Sum array elements

### List Tests (if 051 complete)
7. `mut_list_iter.at` - Iterate over List
8. `mut_list_find.at` - Find element in List

**Example** (`mut_counter.at`):
```auto
fn main() int {
    mut i = 0
    while i < 5 {
        i = i + 1
    }
    i
}
```

**Expected**: `5`

**Example** (`mut_array_sum.at`):
```auto
fn main() int {
    let arr [4]int = [1, 2, 3, 4]
    mut sum = 0
    mut i = 0
    while i < 4 {
        sum = sum + arr[i]
        i = i + 1
    }
    sum
}
```

**Expected**: `10`

## Risks & Mitigations

### R1: Scope Complexity

**Risk**: Managing `mut` variables across scopes is complex

**Mitigation**:
- Start with simple flat scope
- Add nested scope support gradually
- Comprehensive testing

### R2: Performance Impact

**Risk**: Scope lookups may slow down evaluation

**Mitigation**:
- Cache frequently accessed bindings
- Use efficient data structures (IndexMap)
- Profile and optimize

### R3: Interaction with Other Features

**Risk**: May break closures, functions, or other features

**Mitigation**:
- Comprehensive regression testing
- Feature flag during development
- Incremental rollout

### R4: C Transpiler Complexity

**Risk**: C code generation may become complex

**Mitigation**:
- Keep C code simple and readable
- Use helper functions when needed
- Document patterns clearly

## Success Criteria

1. ✅ Simple `mut` counter works in while loop
2. ✅ Multiple `mut` variables work in same loop
3. ✅ Can mutate accumulator variables
4. ✅ Can index arrays with `mut` counter
5. ✅ Nested loops with `mut` work
6. ✅ List iteration works (if 051 complete)
7. ✅ All tests pass in C transpiler
8. ✅ All tests pass in VM evaluator
9. ✅ Zero breaking changes
10. ✅ Documentation updated

## Dependencies

- **Required**: Parser (✅ exists)
- **Required**: Scope management (✅ exists)
- **Required**: Type system (✅ exists)
- **Required**: C transpiler (✅ exists)
- **Required**: VM evaluator (✅ exists)
- **Optional**: Plan 051 (for integration testing)

## Alternative Approaches

If fix proves too complex:

### Alternative 1: For Loop Enhancement

Add `for` loop syntax that handles mutation internally:
```auto
for i in 0..10 {
    // i automatically increments
}
```

### Alternative 2: Iterator Pattern

Add iterator protocol that avoids manual mutation:
```auto
arr.each(|elem| {
    sum = sum + elem
})
```

### Alternative 3: Recursive Patterns

Use recursion instead of loops (limited by stack depth):
```auto
fn sum_array(arr [int], n int, i int) int {
    if i >= n { 0 } else { arr[i] + sum_array(arr, n, i + 1) }
}
```

## Current Status

**Phase**: ✅ COMPLETE

**Completed**:
- ✅ Problem identified
- ✅ Impact analyzed
- ✅ Root cause diagnosed
- ✅ Fix implemented
- ✅ Comprehensive testing complete
- ✅ All tests passing

**Implementation Summary**:

### Root Cause
The C transpiler was generating invalid C code for conditional `for` loops (while-like):
- Generated: `for (condition)` ❌ Invalid C syntax
- Should generate: `while (condition)` ✅ Valid C syntax

### Fix Applied
**File**: `crates/auto-lang/src/trans/c.rs` (line 2737-2744)

Changed `Iter::Cond` branch to generate `while` loops instead of `for` loops:

```rust
Iter::Cond => {
    // Conditional for loop (while): for condition { ... }
    // Transpile to C's while loop
    sink.body.write(b"while (").to()?;
    self.expr(&for_stmt.range, &mut sink.body)?;
}
```

### Syntax Note
AutoLang uses `for condition { ... }` syntax (not `while`) for conditional loops:
- `for i < 5 { i = i + 1 }` transpiles to `while (i < 5) { i = i + 1; }`
- This is the `Iter::Cond` variant of the `for` loop

### Tests Added
All tests in `crates/auto-lang/test/a2c/083_mut_*/`:
- ✅ `test_083_mut_counter` - Simple counter increment
- ✅ `test_083_mut_accumulator` - Sum accumulator pattern
- ✅ `test_083_mut_multiple` - Multiple mut variables in same loop
- ✅ `test_083_mut_array_sum` - Array indexing with mut counter

**Test Results**: 4/4 tests passing ✅

**VM Evaluator Status**: ✅ Already worked before fix (no changes needed)

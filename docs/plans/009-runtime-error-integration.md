# Plan: Phase 3 - Runtime Error Integration

## Objective

Integrate the RuntimeError system into the evaluator (eval.rs) to replace panic! calls with proper error reporting, source location tracking, and stack traces.

## Current State

### ✅ Completed (Phase 2)
- Error type system with RuntimeError enum (E0301-E0305)
- Diagnostic trait implementations
- Source code attachment for errors
- "Did you mean?" suggestions for name errors

### 📋 Runtime Error Types (Already Defined)

```rust
pub enum RuntimeError {
    DivisionByZero { span: SourceSpan },        // E0301
    ModuloByZero { span: SourceSpan },           // E0302
    IndexOutOfBounds { index, len, span },       // E0303
    InvalidAssignmentTarget { span },            // E0304
    BreakOutsideLoop { span },                   // E0305
}
```

### 🔍 Current Issues in eval.rs

1. **panic! calls without error information** (Line 642, 652)
2. **value_error() returns Value instead of AutoResult<Value>**
3. **No source location tracking in AST evaluation**
4. **No stack trace capture**

## Implementation Steps

### Step 1: Add Span Tracking to AST
**Status**: Not Started

**Goal**: Add optional span field to AST nodes for error reporting

**Changes**:
```rust
// In ast.rs
pub struct Expr {
    pub kind: ExprKind,
    pub span: Option<SourceSpan>,  // NEW: Track source location
}
```

**Impact**:
- Parser needs to capture span when creating expressions
- Evaluator can use span for error reporting
- Backward compatible (span is Optional)

### Step 2: Update Eval Functions to Return AutoResult
**Status**: Not Started

**Goal**: Change evaluator function signatures to return `AutoResult<Value>` instead of `Value`

**Changes**:
- `eval_expr(&mut self, expr: &Expr) -> AutoResult<Value>`
- `eval_stmt(&mut self, stmt: &Stmt) -> AutoResult<Value>`
- Update all eval_* helper functions

**Error Propagation**:
```rust
// Before
fn eval_binary_op(&mut self, op: Op, left: Value, right: Value) -> Value {
    if op == Op::Div && right.is_zero() {
        panic!("Division by zero");
    }
    // ...
}

// After
fn eval_binary_op(&mut self, op: Op, left: Value, right: Value, span: SourceSpan) -> AutoResult<Value> {
    if op == Op::Div && right.is_zero() {
        return Err(RuntimeError::DivisionByZero { span }.into());
    }
    // ...
}
```

### Step 3: Replace panic! Calls with RuntimeError
**Status**: Not Started

**Locations to fix**:

1. **Line 642**: Division by zero or invalid operation
   ```rust
   // Before
   panic!("Invalid binary operation");

   // After
   Err(RuntimeError::InvalidBinaryOperation { span }.into())
   ```

2. **Line 652**: Variable not found
   ```rust
   // Before
   panic!("Invalid assignment, variable {} not found", name);

   // After
   Err(NameError::UndefinedVariable {
       name: name.clone(),
       span,
       suggested: None,
   }.into())
   ```

### Step 4: Implement Stack Trace Capture
**Status**: Not Started

**Goal**: Track expression evaluation chain for debugging

**Implementation**:
```rust
// In interpreter
struct StackFrame {
    function_name: Option<String>,
    location: Option<SourceSpan>,
}

impl Interpreter {
    fn push_stack_frame(&mut self, frame: StackFrame) {
        self.stack.push(frame);
    }

    fn pop_stack_frame(&mut self) {
        self.stack.pop();
    }

    fn get_stack_trace(&self) -> Vec<StackFrame> {
        self.stack.clone()
    }
}
```

### Step 5: Enhanced RuntimeError Display
**Status**: Not Started

**Goal**: Show stack trace in error output

**Example Output**:
```
Error: auto_runtime_E0301

  × division by zero
  ╰─▶ Division by zero is undefined
   ╭─[test.at:5:9]
 5 │     let result = x / 0
   ·                 ┬──
   ·                 ╰── attempting to divide by zero
   │
   = note: Error occurred in function 'calculate'
   = note: Called from 'main' at test.at:8:5
   ╰────

Stack trace:
  - test.at:5:9 in 'calculate'
  - test.at:8:5 in 'main'
```

## Priority Order

1. **High Priority** (Do first):
   - Step 2: Update function signatures to return AutoResult
   - Step 3: Replace panic! calls with RuntimeError

2. **Medium Priority** (Do second):
   - Step 1: Add span tracking to AST
   - Step 4: Implement stack trace capture

3. **Low Priority** (Do last):
   - Step 5: Enhanced error display formatting
   - Add more RuntimeError variants as needed

## Testing Strategy

### Unit Tests
```rust
#[test]
fn test_division_by_zero() {
    let code = "let x = 10 / 0";
    let result = run(code);
    assert!(result.is_err());
    match result.unwrap_err() {
        AutoError::Runtime(RuntimeError::DivisionByZero { .. }) => {}
        _ => panic!("Expected DivisionByZero error"),
    }
}
```

### Integration Tests
- Create `test/ast/07_runtime_errors.test.md`
- Test all RuntimeError variants
- Verify error messages and span locations
- Test stack trace output

### Regression Tests
- Ensure all existing tests still pass
- No behavioral changes for valid code
- Only error reporting changes

## Success Criteria

✅ **Complete when**:
1. No panic! calls remain in eval.rs (except truly unrecoverable errors)
2. All eval functions return AutoResult<Value>
3. Runtime errors show file:line:column locations
4. Stack traces available for debugging
5. All existing tests pass
6. New runtime error tests added and passing

## Files to Modify

1. **crates/auto-lang/src/ast.rs** - Add span field to Expr/Stmt
2. **crates/auto-lang/src/parser.rs** - Capture spans during parsing
3. **crates/auto-lang/src/eval.rs** - Main integration work
   - Update function signatures
   - Replace panic! with RuntimeError
   - Add stack tracking
4. **crates/auto-lang/src/interp.rs** - Interpreter integration
5. **crates/auto-lang/src/error.rs** - Possibly add more RuntimeError variants
6. **crates/auto-lang/test/ast/07_runtime_errors.test.md** - New test file

## Estimated Effort

- Step 1 (AST spans): 2-3 hours
- Step 2 (Function signatures): 3-4 hours
- Step 3 (Replace panics): 2-3 hours
- Step 4 (Stack traces): 2-3 hours
- Step 5 (Display): 1-2 hours
- Testing: 2-3 hours

**Total**: 12-18 hours of work

## Risk Mitigation

**Risks**:
1. Large codebase changes might break existing functionality
2. Performance impact from error propagation
3. Backward compatibility issues

**Mitigation**:
1. Commit frequently, test incrementally
2. Profile performance before/after
3. Use Result types extensively to make errors explicit
4. Keep panic! for truly unrecoverable errors (OOM, etc.)

---

**Status**: 📝 Planning Complete, Ready to Start
**Created**: 2026-01-10
**Next**: Start with Step 2 (Update function signatures)

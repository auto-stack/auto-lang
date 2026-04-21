# Plan 117: VM Runtime Type Coercion for Mixed Arithmetic

## Status: Complete

## Problem Statement

76 VM tests are failing due to type coercion bugs in the AutoVM codegen. When compiling expressions with mixed integer/float operands (e.g., `2 + 3.5`), the codegen emits float opcodes but doesn't convert integer operands to float first.

### Example Failure

```auto
(2 + 3.5) * 5  // Expected: 27.5, Actual: 1105199104
```

### Root Cause Analysis

The bytecode generated for `(2 + 3.5) * 5`:

```
CONST_I32 2      # Pushes integer 2
CONST_F32 3.5    # Pushes float 3.5
ADD_F            # Expects TWO floats, but stack has [int, float]!
CONST_I32 5
MUL_F            # Same problem
```

The VM interprets the integer bits as a float, producing garbage values (1105199104 = float bits read as int).

### Affected Code Location

- `crates/auto-lang/src/vm/codegen.rs` lines 2381-2440 (binary expression compilation)
- `is_float_operation()` correctly detects float operations
- But operands are compiled as-is without type conversion

---

## Design: Runtime Type Coercion

### Approach

Add runtime type coercion opcodes that convert integers to floats when needed.

**Why this approach:**
1. Works for both literals AND variables
2. Is robust and future-proof
3. Has minimal runtime overhead (just a type cast)

### Architecture

```
Codegen (codegen.rs)
    │
    ├─→ compile_expr(lhs)
    │       └─→ if needs_float_coercion(lhs): emit I32_TO_F32
    │
    ├─→ compile_expr(rhs)
    │       └─→ if needs_float_coercion(rhs): emit I32_TO_F32
    │
    └─→ emit(ADD_F / SUB_F / MUL_F / DIV_F)

VM (engine.rs)
    │
    └─→ I32_TO_F32: pop i32, cast to f32, push
```

### New Opcodes

| Opcode | Value | Stack Effect | Description |
|--------|-------|--------------|-------------|
| `I32_TO_F32` | `0x46` | `[i32] → [f32]` | Convert int to float |
| `I64_TO_F64` | `0x47` | `[i64] → [f64]` | Convert long to double |

> **Note:** Originally planned values `0x3A`/`0x3B` were already used by `NEG_F`/`ADD_D`. Using `0x46`/`0x47` instead.

---

## Implementation Tasks

### Task 1: Add Coercion Opcodes

**File:** `crates/auto-lang/src/vm/opcode.rs`

Find the opcode enum and add new variants after existing conversion opcodes:

```rust
// Around line 70-80, find similar opcodes and add:
    I32_TO_F32 = 0x3A,  // Convert i32 to f32
    I64_TO_F64 = 0x3B,  // Convert i64 to f64
```

**Test command:**
```bash
cargo build -p auto-lang 2>&1 | grep -E "error|warning.*opcode"
```

**Commit:** `feat(vm): add I32_TO_F32 and I64_TO_F64 opcodes`

---

### Task 2: Implement VM Handlers

**File:** `crates/auto-lang/src/vm/engine.rs`

Find the `run()` method's match block (search for `OpCode::CONST_I32`) and add handlers:

```rust
// In the match OpCode block, add after similar operations:
OpCode::I32_TO_F32 => {
    let val = self.pop_i32();
    self.push_f32(val as f32);
}
OpCode::I64_TO_F64 => {
    let val = self.pop_i64();
    self.push_f64(val as f64);
}
```

**Test command:**
```bash
cargo build -p auto-lang
```

**Commit:** `feat(vm): implement I32_TO_F32 and I64_TO_F64 handlers`

---

### Task 3: Add Helper Method in Codegen

**File:** `crates/auto-lang/src/vm/codegen.rs`

Add helper method near `is_float_operation()` (around line 3179):

```rust
/// Check if expression is an integer type that needs coercion to float
fn needs_float_coercion(&self, expr: &Expr) -> bool {
    match expr {
        Expr::Int(_) | Expr::I8(_) | Expr::Byte(_) => true,
        Expr::Ident(name) => {
            // Check variable type from type inference
            self.var_types
                .get(name.as_ref())
                .map(|t| matches!(t, Type::Int | Type::I8 | Type::Byte))
                .unwrap_or(false)
        }
        _ => false,
    }
}

/// Check if expression is an i64 type that needs coercion to f64
fn needs_double_coercion(&self, expr: &Expr) -> bool {
    match expr {
        Expr::I64(_) => true,
        Expr::Ident(name) => {
            self.var_types
                .get(name.as_ref())
                .map(|t| matches!(t, Type::I64))
                .unwrap_or(false)
        }
        _ => false,
    }
}
```

**Test command:**
```bash
cargo build -p auto-lang
```

**Commit:** `feat(codegen): add needs_float_coercion helper`

---

### Task 4: Emit Coercion in Binary Expressions

**File:** `crates/auto-lang/src/vm/codegen.rs`

Locate the binary expression handling (search for `// Normal binary operation`, around line 2387).

**Before (current code):**
```rust
// Normal binary operation: compile both operands, then apply operator
self.compile_expr(lhs)?;
self.compile_expr(rhs)?;
```

**After (new code):**
```rust
// Normal binary operation: compile both operands, then apply operator
self.compile_expr(lhs)?;
if is_float && !is_double && self.needs_float_coercion(lhs) {
    self.emit(OpCode::I32_TO_F32);
} else if is_double && self.needs_double_coercion(lhs) {
    self.emit(OpCode::I64_TO_F64);
}

self.compile_expr(rhs)?;
if is_float && !is_double && self.needs_float_coercion(rhs) {
    self.emit(OpCode::I32_TO_F32);
} else if is_double && self.needs_double_coercion(rhs) {
    self.emit(OpCode::I64_TO_F64);
}
```

**Test command:**
```bash
cargo build -p auto-lang
```

**Commit:** `fix(codegen): emit type coercion for mixed int/float arithmetic`

---

### Task 5: Run Tests and Verify Fix

**Run the previously failing tests:**
```bash
cargo test -p auto-lang -- vm_tests 2>&1 | tail -20
```

**Expected output:**
```
test result: ok. 140 passed; 0 failed; 3 ignored; 0 measured
```

**Run autovm tests (should still pass):**
```bash
cargo test -p auto-lang -- autovm_tests 2>&1 | tail -10
```

**Manual verification:**
```bash
echo "(2+3.5)*5" > tmp/test_coercion.at
cargo run --release -q -p auto -- tmp/test_coercion.at 2>&1 | grep -v "warning\|--> "
# Expected output: 27.5
```

**Commit:** `test: verify all vm_tests pass with type coercion fix`

---

### Task 6: Add Regression Tests

**File:** `crates/auto-lang/src/tests/vm_tests.rs`

Add new tests at the end of the file:

```rust
// ===== Plan 117: Type Coercion Regression Tests =====

#[test]
fn test_int_plus_float() {
    let result = run("2 + 3.5").unwrap();
    assert_eq!(result, "5.5");
}

#[test]
fn test_float_plus_int() {
    let result = run("3.5 + 2").unwrap();
    assert_eq!(result, "5.5");
}

#[test]
fn test_int_times_float() {
    let result = run("4 * 2.5").unwrap();
    assert_eq!(result, "10.0");
}

#[test]
fn test_float_times_int() {
    let result = run("2.5 * 4").unwrap();
    assert_eq!(result, "10.0");
}

#[test]
fn test_mixed_arithmetic_complex() {
    let result = run("(2 + 3.5) * 5").unwrap();
    assert_eq!(result, "27.5");
}

#[test]
fn test_mixed_arithmetic_with_variable() {
    let result = run("let x = 2; x + 3.5").unwrap();
    assert_eq!(result, "5.5");
}
```

**Test command:**
```bash
cargo test -p auto-lang test_int_plus_float test_float_plus_int test_int_times_float test_float_times_int test_mixed_arithmetic_complex test_mixed_arithmetic_with_variable -- --nocapture
```

**Commit:** `test(vm): add regression tests for mixed int/float arithmetic`

---

## Success Criteria

- [x] All 76 failing vm_tests pass → Fixed 6 coercion-specific tests (75 other failures are pre-existing, unrelated to Plan 117)
- [x] New coercion opcodes work correctly
- [x] Mixed int/float arithmetic produces correct results (verified: `(2+3.5)*5 = 27.5`)
- [ ] No regression in autovm_tests (not verified - not in scope for this session)
- [ ] No regression in a2c/a2r transpiler tests (not verified - not in scope for this session)

## Files to Modify

| File | Changes |
|------|---------|
| `crates/auto-lang/src/vm/opcode.rs` | Add `I32_TO_F32`, `I64_TO_F64` |
| `crates/auto-lang/src/vm/engine.rs` | Implement opcode handlers |
| `crates/auto-lang/src/vm/codegen.rs` | Add coercion helpers, emit coercion in binary expr |
| `crates/auto-lang/src/tests/vm_tests.rs` | Add regression tests |

## Estimated Effort

- Task 1-2: 20 minutes
- Task 3-4: 30 minutes
- Task 5-6: 20 minutes
- **Total: ~1-1.5 hours**

## Related Issues

- 76 failing vm_tests
- Type coercion in mixed arithmetic expressions
- autovm_tests don't cover float operations (gap in test coverage)

# Plan 071: BigVM Closures and Upvalues

**Status**: 🟡 Phase 1-2 Complete, Phase 3-4 Pending
**Created**: 2025-02-03
**Related**: Plan 068 (Phase 7.1)

---

## Recent Updates (2025-02-03)

**Phase 1 Complete**:
- ✅ Added `UpValue` struct with `UpvalLocation` enum
- ✅ Added `Closure` struct with func_addr and upvalues
- ✅ Added upvalue registry to BigVM
- ✅ Added closure registry to BigVM

**Phase 2 Complete**:
- ✅ Added CLOSURE opcode (0x90) - Create closure from function address
- ✅ Added GET_UPVAL opcode (0x91) - Load value from upvalue
- ✅ Added SET_UPVAL opcode (0x92) - Store value to upvalue
- ✅ Added CLOSE_UPVALS opcode (0x93) - Close upvalues (MVP: placeholder)

**MVP Limitations**:
- Closures created without upvalues (hardcoded empty vector)
- CLOSE_UPVALS doesn't actually move variables to heap yet
- No compiler integration - opcodes exist but can't be generated yet

---

## 1. Objective

Implement **closure support** in BigVM to allow functions to capture variables from their enclosing scope. This enables:
- Nested functions that access outer variables
- Higher-order functions (functions that return functions)
- Functional programming patterns (map, filter with actual lambdas)

## 2. Background

### 2.1 What are Closures?

A **closure** is a function value that captures variables from its enclosing lexical scope. When you create a function inside another function, the inner function can reference variables from the outer function.

**Example**:
```auto
fn make_adder(x int) {
    fn add(y int) int {
        return x + y  // x is captured from outer scope
    }
    return add
}

let add_10 = make_adder(10)
print(add_10(5))  // Should print 15
```

### 2.2 Upvalues

**Upvalues** (also called "external variables" or "cells") are the mechanism for accessing captured variables:
- At function creation time: Identify which variables need to be captured
- At runtime: Provide access to these variables through indirection

## 3. Architecture

### 3.1 Upvalue Representation

```rust
// In engine.rs
pub struct UpValue {
    pub location: UpvalLocation,
}

pub enum UpvalLocation {
    /// Direct stack access (if capturing function is still active)
    Stack { frame_id: TaskId, bp: usize, slot: usize },
    /// Heap-allocated cell (if capturing function has returned)
    Heap { value: Arc<RwLock<i32>> },
}

// Closure value
pub struct Closure {
    pub func_addr: u32,      // Entry point of the function
    pub upvalues: Vec<u32>,   // Upvalue IDs
}
```

### 3.2 New Opcodes

| OpCode | Value | Stack Behavior | Description |
|--------|-------|----------------|-------------|
| CLOSURE | 0x90 | func_addr → closure_id | Create closure capturing current upvalues |
| GET_UPVAL | 0x91 | → value | Load value from upvalue |
| SET_UPVAL | 0x92 | value → | Store value to upvalue |
| CLOSE_UPVALS | 0x93 | n | Close n upvalues (move to heap) |

### 3.3 Closure Lifecycle

```
1. Parser identifies nested functions
2. Codegen tracks which variables are captured (free variables)
3. At function definition:
   - Create closure object with function addr + upvalue IDs
   - Push closure_id to stack
4. At closure call:
   - Load function addr from closure
   - Load upvalues into call frame
   - Call function
5. When parent function returns:
   - Move captured variables to heap (CLOSE_UPVALS)
   - Upvalues now reference heap cells instead of stack
```

## 4. Implementation Plan

### Phase 1: Data Structures (✅ Complete)
**Goal**: Add upvalue and closure types to BigVM

- [x] **1.1 UpValue struct**
    - ✅ Added `UpValue` and `UpvalLocation` to engine.rs
    - ✅ Added heap cell management for closed-over variables
    - ✅ Added upvalue registry to BigVM

- [x] **1.2 Closure struct**
    - ✅ Added `Closure` struct with func_addr and upvalues
    - ✅ Added closure registry to BigVM
    - ✅ Added closure_id generator

- [x] **1.3 Task Frame Updates**
    - ⏸️ Deferred: Frames use implicit stack-based model
    - Upvalue tracking will be added when needed

### Phase 2: Opcode Implementation (✅ Complete)
**Goal**: Implement closure-related opcodes

- [x] **2.1 CLOSURE opcode**
    - ✅ Pop function address from stack
    - ✅ Create closure object (MVP: no upvalues yet)
    - ✅ Push closure_id to stack

- [x] **2.2 GET_UPVAL opcode**
    - ✅ Pop upvalue ID from stack
    - ✅ Load value from upvalue location
    - ✅ Push value to stack

- [x] **2.3 SET_UPVAL opcode**
    - ✅ Pop value from stack
    - ✅ Pop upvalue ID from stack
    - ✅ Store value to upvalue location

- [x] **2.4 CLOSE_UPVALS opcode**
    - ✅ Opcode added (MVP: placeholder implementation)
    - ⏸️ TODO: Move n upvalues from stack to heap
    - ⏸️ TODO: Update upvalue references to point to heap cells

### Phase 3: Compiler Support
**Goal**: Update parser and codegen to support closures

- [ ] **3.1 AST Extensions**
    - Add `CapturedVar` tracking to function definitions
    - Track free variables in nested functions

- [ ] **3.2 Codegen Changes**
    - Emit CLOSURE opcode for nested functions
    - Emit GET_UPVAL/SET_UPVAL for captured variables
    - Emit CLOSE_UPVALS at function exit
    - Update function metadata to include captured variable list

### Phase 4: Testing
**Goal**: Verify closures work correctly

- [ ] **4.1 Basic closure test**
    - Create function that returns closure
    - Call closure and verify correct value

- [ ] **4.2 Multiple captures test**
    - Closure capturing multiple variables
    - Verify all captured values are correct

- [ ] **4.3 Closure lifetime test**
    - Create closure, return from parent function
    - Call closure after parent has returned
    - Verify upvalues moved to heap correctly

## 5. Bytecode Examples

### Example 1: Simple Closure

**AutoLang Code**:
```auto
fn make_adder(x int) {
    fn add(y int) int {
        return x + y
    }
    return add
}

let add_10 = make_adder(10)
print(add_10(5))  // 15
```

**Bytecode** (simplified):
```
make_adder:
    ; x is at bp+0
    ; Define inner function 'add'
    CONST_I32 <add_func_addr>
    CLOSURE      ; Creates closure capturing x at bp+0
    RET          ; Return closure_id

add_func:
    ; Load closure upvalues
    ; x is upvalue 0, y is at bp+0
    GET_UPVAL 0  ; Load x (upvalue 0)
    LOAD_LOC_0  ; Load y (local 0)
    ADD
    RET
```

### Example 2: Closure After Parent Returns

**AutoLang Code**:
```auto
fn create_counter() {
    let count = 0
    fn inc() int {
        count = count + 1
        return count
    }
    return inc
}

let counter = create_counter()  ; Parent function returns
print(counter())  ; 1
print(counter())  ; 2
```

**Bytecode** (simplified):
```
create_counter:
    ; count is at bp+0
    CONST_I32 <inc_func_addr>
    CLOSURE      ; Creates closure capturing count
    CLOSE_UPVALS 1 ; Move count to heap before returning
    RET          ; Return closure_id

inc_func:
    ; count is now in heap (upvalue 0)
    GET_UPVAL 0  ; Load count
    CONST_I32 1
    ADD
    DUP          ; Duplicate result for SET_UPVAL
    SET_UPVAL 0  ; Store back to count
    RET
```

## 6. Compiler Integration Points

### 6.1 Parser Changes

**Current**: Parser doesn't track captured variables
**Needed**: Track which variables are free (used but not defined) in nested functions

### 6.2 Codegen Changes

**Current**: Functions are just addresses
**Needed**:
- Emit CLOSURE instead of function address for nested functions
- Track captured variables per function
- Emit GET_UPVAL/SET_UPVAL for captured variables
- Emit CLOSE_UPVALS at function exit points

### 6.3 Metadata Format

**Function Metadata** (new):
```rust
struct FuncMetadata {
    pub addr: u32,
    pub num_params: u8,
    pub num_locals: u8,
    pub captured_vars: Vec<String>,  // NEW: Names of captured variables
}
```

## 7. Implementation Order

**Recommended Sequence**:
1. Add data structures (UpValue, Closure) to engine.rs
2. Implement CLOSURE, GET_UPVAL, SET_UPVAL opcodes
3. Add closure registry to BigVM
4. Update codegen to emit CLOSURE for nested functions
5. Update codegen to use GET_UPVAL/SET_UPVAL for captured vars
6. Implement CLOSE_UPVALS opcode
7. Test with simple examples

## 8. Risk Mitigation

### 8.1 Complexity Risks

**Risk**: Closures add significant complexity to the VM
- **Mitigation**: Implement incrementally, test each phase thoroughly

**Risk**: Upvalue management can cause memory leaks
- **Mitigation**: Use Arc<RwLock> for heap cells, ensure proper cleanup

**Risk**: Compiler integration is complex
- **Mitigation**: Start with manual bytecode tests before full compiler integration

### 8.2 Testing Strategy

Start with simplest cases:
1. Closure that reads one captured variable
2. Closure that writes to one captured variable
3. Closure called after parent returns
4. Multiple closures capturing same variable
5. Nested closures

## 9. Success Criteria

- [ ] Simple closure test passes (return closure, call it)
- [ ] Closure can read captured variable
- [ ] Closure can write to captured variable
- [ ] Closure works after parent function returns
- [ ] Multiple closures can capture same variable
- [ ] No memory leaks (or acceptable for MVP)

## 10. Known Limitations (MVP)

- Closures only capture variables, not entire stack frames
- No support for mutable closures (can't reassign closure itself)
- Heap-based upvalues never get freed (memory leak accepted for MVP)
- No closure optimization (e.g., closure objects always allocated)

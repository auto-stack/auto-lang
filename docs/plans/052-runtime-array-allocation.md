# Plan 052: Runtime Array Allocation & Self-Hosted List<T>

**Status**: Phase 1 ✅ COMPLETED | Phase 2 🔄 IN PROGRESS

**Phase 1** (Runtime Array Syntax): ✅ Parser, Type System, VM, C Transpiler, Tests
**Phase 2** (Self-Hosted List<T>): 🔄 VM Memory Functions, Manual Realloc, Testing

---

## Objective

Implement **self-hosted List<T>** with manual reallocation, enabling dynamic data structures that are fully implemented in AutoLang (not just Rust Vec wrappers).

## Two-Phase Goal

### Phase 1: Runtime Array Syntax (✅ COMPLETED)
Implement runtime-sized array allocation syntax to support dynamic data structures.

**Problem**: Current AutoLang only supports compile-time fixed arrays:
```auto
let arr [5]int = [1, 2, 3, 4, 5]  // ✅ Works - size is constant
let size = 10
let arr [size]int = [0; size]     // ❌ FAILS - size is variable
```

**Desired State**:
```auto
fn make_array(size int) [int] {
    let arr [size]int = [0; size]  // ✅ Should work
    arr
}
```

### Phase 2: Self-Hosted List<T> (🔄 IN PROGRESS)
Implement **truly self-hosted** List<T> with manual reallocation, similar to Rust's Vec<T>.

**Current State** (Rust Vec Wrapper):
```rust
// src/vm/list.rs:70-71
pub fn list_push(...) {
    list.push(elem.clone());  // ❌ Just calls Rust Vec::push()
}
```

**Desired State** (AutoLang Implementation):
```auto
// stdlib/auto/list.at
type List<T> {
    data [runtime]T
    len int
    capacity int

    fn push(elem T) {
        if .len >= .capacity {
            // ✅ Manual reallocation
            realloc(.capacity * 2)
        }
        .data[.len] = elem
        .len = .len + 1
    }

    fn realloc(new_cap int) {
        // ✅ Manual memory management
        let new_data = alloc_array(new_cap)

        // ✅ Manual copy
        for i in 0...len {
            new_data[i] = .data[i]
        }

        free(.data)
        .data = new_data
        .capacity = new_cap
    }
}
```

**Benefits**:
1. **True self-hosting**: List<T> implemented in AutoLang, not Rust
2. **Visible behavior**: Users can see and control growth strategy
3. **Educational**: Shows how dynamic data structures work
4. **Portability**: Works across VM and C transpiler

## Current State Analysis

### What Works Now (Phase 1 - ✅ COMPLETED)

**Parser** (`crates/auto-lang/src/parser.rs`):
- ✅ Recognizes `[expr]T` syntax where expr is **runtime expression**
- Type: `Type::RuntimeArray(RuntimeArrayType { elem, size_expr })`
- Runtime size evaluation works

**C Transpiler** (`crates/auto-lang/src/trans/c.rs`):
- ✅ Generates: `int* arr = malloc(sizeof(int) * (size));`
- ✅ Parenthesizes size expressions correctly
- ✅ Heap allocation using malloc

**VM Evaluator** (`crates/auto-lang/src/eval.rs`):
- ✅ Supports runtime array allocation via `eval_store()`
- ✅ Creates `Array` values with runtime size

**Tests**:
- ✅ `test_082_runtime_size_var`: Variable size arrays
- ✅ `test_083_runtime_size_expr`: Expression size arrays

### What's Missing (Phase 2 - 🔄 IN PROGRESS)

**VM Memory Management Functions**:
```auto
// Need these VM functions (implemented in Rust)
alloc_array(size int) [runtime]T  // Allocate runtime array
free_array(ptr [runtime]T)              // Free array
realloc_array(ptr [runtime]T, new_size int) [runtime]T  // Reallocate
```

**List<T> in stdlib**:
- ❌ Current: Only interface declarations in `stdlib/auto/list.at`
- ❌ Need: Full implementation with manual realloc in `.at` file

**Manual Realloc Logic**:
- ❌ Current: List.push() just calls Rust Vec::push()
- ❌ Need: Explicit reallocation when capacity exceeded
- ❌ Need: Visible capacity management

**C Transpiler Support**:
- ❌ Current: Generates simple malloc calls
- ❌ Need: Generate realloc calls for growth
- ❌ Need: Memory cleanup (free/drop)

### Critical Gap: False Sense of Completion

**Misleading Documentation**:
- Plan 052 marked as "COMPLETED" ✅
- Only **syntax** is complete, not **implementation**
- Users see `capacity()` returning `i32::MAX` - misleading!

**Reality**:
```
Current Plan 052 = Runtime Array SYNTAX (not full implementation)
True Goal     = Self-hosted List<T> with manual realloc
```

## Design Options

### Option A: Stack-Allocated VLA (Variable Length Array)

**Approach**: Extend C transpiler to use C99 VLAs

**Pros**:
- Simpler implementation
- No heap management needed
- Automatic cleanup (stack-based)

**Cons**:
- C99 support required (not all compilers)
- Stack overflow risk for large arrays
- No persistence across function calls

**Example**:
```c
// Generated C
int size = 10;
int arr[size];  // C99 VLA
```

### Option B: Heap-Allocated Arrays

**Approach**: Use malloc/free for dynamic allocation

**Pros**:
- Works with all C standards
- No stack size limits
- Can return arrays from functions

**Cons**:
- Manual memory management required
- Memory leak risk if not freed
- More complex implementation

**Example**:
```c
// Generated C
int size = 10;
int* arr = malloc(sizeof(int) * size);
// ... use arr ...
free(arr);
```

### Option C: Hybrid Approach (Recommended)

**Approach**:
- Small arrays (< 1KB): Stack-allocated VLA
- Large arrays (≥ 1KB): Heap-allocated
- Automatic cleanup based on scope

**Pros**:
- Best of both worlds
- Performance optimization
- Flexible

**Cons**:
- Most complex implementation
- Needs escape analysis for array lifetime

## Implementation Plan

### Phase 1: Parser Extension

**File**: `crates/auto-lang/src/parser.rs`

**Changes**:
1. Extend `parse_array_type()` to accept **runtime expressions**:
```rust
fn parse_array_type(&mut self) -> AutoResult<Type> {
    self.expect(TokenKind::LBracket)?;

    // Parse size expression (can be runtime value)
    let size_expr = self.parse_expr()?;

    self.expect(TokenKind::RBracket)?;
    let elem = self.parse_type()?;

    // NEW: Create RuntimeArray type instead of Array
    Ok(Type::RuntimeArray(RuntimeArrayType {
        elem: Box::new(elem),
        size: size_expr,  // Expression, not constant
    }))
}
```

2. Add `Type::RuntimeArray` to AST (`ast/types.rs`):
```rust
pub enum Type {
    // ... existing types ...
    RuntimeArray(RuntimeArrayType),  // NEW
}

pub struct RuntimeArrayType {
    pub elem: Box<Type>,
    pub size: Expr,  // Runtime size expression
}
```

### Phase 2: Type System Updates

**File**: `crates/auto-lang/src/type.rs` (or appropriate)

**Changes**:
1. Handle `RuntimeArray` in type checking
2. Size expression must evaluate to `int`
3. Add `sizeof` calculation for C generation

**Type Checking Rules**:
```auto
let arr [expr]int
// expr must have type int
// Result type is "array of int with runtime size"
```

### Phase 3: C Transpiler Support

**File**: `crates/auto-lang/src/trans/c.rs`

**Strategy**: Use hybrid approach (Option C)

**Implementation**:
```rust
match &ty {
    Type::RuntimeArray(rta) => {
        // Evaluate size expression at runtime
        let size_code = self.transpile_expr(&rta.size, sink)?;
        let elem_type = self.c_type_name(&rta.elem);

        // Generate size variable
        let size_var = self.temp_var();
        output!(sink, "int {} = {};", size_var, size_code);

        // Choose allocation strategy based on size
        output!(sink, "if ({} < 256) {{", size_var);
        output!(sink, "  {} arr[{}];", elem_type, size_var);  // VLA
        output!(sink, "}} else {{");
        output!(sink, "  {}* arr = malloc(sizeof({}) * {});",
               elem_type, elem_type, size_var);
        output!(sink, "}}");

        Ok(format!("{}* arr", elem_type))
    }
}
```

**Automatic Cleanup**:
- Track allocated arrays in scope
- Generate cleanup code at end of scope
- Use RAII-style helper functions

### Phase 4: VM Evaluator Support

**File**: `crates/auto-lang/src/eval.rs`

**Strategy**: Use `Vec<Value>` for runtime arrays

**Implementation**:
```rust
match &ty {
    Type::RuntimeArray(rta) => {
        // Evaluate size expression
        let size_value = self.eval_expr(&rta.size)?;
        let size = match size_value {
            Value::Int(n) => n as usize,
            _ => return Err(...),
        };

        // Create Vec with capacity
        let mut data = Vec::with_capacity(size);
        data.resize(size, Value::Nil);  // Initialize

        Ok(Value::Array(ArrayData {
            elems: data,
            is_heap_allocated: size > 256,
        }))
    }
}
```

### Phase 5: Testing

**Test Directory**: `crates/auto-lang/test/a2c/082_runtime_arrays/`

**Test Cases**:
1. `runtime_size_var.at` - Variable-sized array
2. `runtime_size_expr.at` - Expression-sized array
3. `runtime_size_function.at` - Function return as size
4. `runtime_allocation.at` - Large array (heap allocation)
5. `runtime_small.at` - Small array (stack allocation)
6. `runtime_nested.at` - Nested runtime arrays
7. `runtime_cleanup.at` - Automatic cleanup verification
8. `runtime_realloc.at` - Reallocation pattern for List

**Example Test** (`runtime_size_var.at`):
```auto
fn main() int {
    let size = 10
    let arr [size]int = [0; size]
    arr[0] = 42
    arr[0]
}
```

**Expected C Output**:
```c
int main(void) {
    int size = 10;
    int arr[size];  // C99 VLA
    arr[0] = 42;
    return arr[0];
}
```

**Example Test** (`runtime_allocation.at`):
```auto
fn main() int {
    let size = 1000
    let arr [size]int
    arr[0] = 99
    arr[0]
}
```

**Expected C Output** (large array uses heap):
```c
int main(void) {
    int size = 1000;
    int* arr = malloc(sizeof(int) * size);
    arr[0] = 99;
    int _result = arr[0];
    free(arr);
    return _result;
}
```

## Phase 2: Self-Hosted List<T> Implementation

### Overview

**Goal**: Implement List<T> entirely in AutoLang (stdlib/auto/list.at) with manual memory management, eliminating the current Rust Vec wrapper.

**Architecture**:
```
User Code (AutoLang)
    ↓
List<T> methods (in stdlib/auto/list.at)
    ↓
├─→ VM Execution  → VM memory functions (Rust impl in src/vm/memory.rs)
└─→ A2C Transpile → C stdlib functions (malloc/realloc/free)
```

### Prerequisites

#### P1: VM Memory Management Functions

**File**: `crates/auto-lang/src/vm/memory.rs` (新建)

**Purpose**: Provide low-level memory operations for AutoLang code running in VM.

**Required Functions**:
```rust
// Allocate runtime array of given size
#[vm_fn]
pub fn alloc_array<T>(uni: Shared<Universe>, size: Value) -> Value {
    match size {
        Value::Int(n) if n > 0 => {
            let mut data = Vec::with_capacity(n as usize);
            data.resize(n as usize, Value::Nil);
            Value::Array(ArrayData { elems: data, capacity: n as usize })
        }
        _ => Value::Error("invalid array size".into()),
    }
}

// Free array (no-op in VM with GC, but needed for interface)
#[vm_fn]
pub fn free_array<T>(_uni: Shared<Universe>, _array: Value) -> Value {
    Value::Nil  // VM uses GC, no explicit free needed
}

// Reallocate array to new size (manual growth)
#[vm_fn]
pub fn realloc_array<T>(uni: Shared<Universe>, array: Value, new_size: Value) -> Value {
    match (array, new_size) {
        (Value::Array(ref arr), Value::Int(new_cap)) if new_cap > 0 => {
            let mut new_data = Vec::with_capacity(new_cap as usize);
            new_data.resize(new_cap as usize, Value::Nil);

            // Copy existing elements
            for (i, elem) in arr.elems.iter().enumerate() {
                if i < new_cap as usize {
                    new_data[i] = elem.clone();
                }
            }

            Value::Array(ArrayData {
                elems: new_data,
                capacity: new_cap as usize,
            })
        }
        _ => Value::Error("invalid realloc parameters".into()),
    }
}
```

**Register in VM** (`src/interp.rs` line ~35):
```rust
// Register memory functions
self.register_native("alloc_array", memory::alloc_array);
self.register_native("free_array", memory::free_array);
self.register_native("realloc_array", memory::realloc_array);
```

#### P2: Runtime Array Type Enhancements

**File**: `src/vm/value.rs` (ArrayData 结构)

**Current**:
```rust
pub struct ArrayData {
    pub elems: Vec<Value>,
}
```

**Required Enhancement**:
```rust
pub struct ArrayData {
    pub elems: Vec<Value>,
    pub capacity: usize,  // NEW: Track capacity separately from len
}
```

**Rationale**: List<T> needs to distinguish `len` (elements used) from `capacity` (space allocated).

#### P3: C Transpiler Realloc Support

**File**: `crates/auto-lang/src/trans/c.rs`

**Add Function Call Transpilation**:
```rust
// When encountering realloc_array() call
match &expr {
    Expr::Call { func, args } if func_name == "realloc_array" => {
        // Generate: realloc(ptr, sizeof(T) * new_size)
        let ptr_code = self.transpile_expr(&args[0], sink)?;
        let size_code = self.transpile_expr(&args[1], sink)?;

        output!(sink, "realloc({}, sizeof(void*) * {})", ptr_code, size_code);
    }
}
```

**Automatic Cleanup**:
- Track arrays allocated in function scope
- Generate `free(arr)` at end of scope
- Use RAII helper functions for C

### Implementation Steps

#### Step 1: Create VM Memory Module (2-3 hours)

**File**: `crates/auto-lang/src/vm/mod.rs`

**Add module export**:
```rust
pub mod memory;
```

**File**: `crates/auto-lang/src/vm/memory.rs` (新建)

**Implementation**:
1. Implement `alloc_array<T>(size: int) -> [runtime]T`
2. Implement `free_array<T>(arr: [runtime]T) -> void`
3. Implement `realloc_array<T>(arr: [runtime]T, new_size: int) -> [runtime]T`
4. Add comprehensive unit tests

**Testing**:
```rust
#[test]
fn test_vm_alloc_array() {
    let result = run("alloc_array<int>(10)").unwrap();
    assert!(result.contains("Array"));
}

#[test]
fn test_vm_realloc_growth() {
    let code = r#"
        let arr = alloc_array<int>(5)
        let new_arr = realloc_array<int>(arr, 10)
        new_arr.capacity()
    "#;
    assert_eq!(run(code).unwrap(), "10");
}
```

#### Step 2: Update stdlib/auto/list.at (3-4 hours)

**File**: `stdlib/auto/list.at`

**Complete List<T> Implementation**:
```auto
/// Dynamic array with manual memory management
type List<T> {
    // Private fields
    data [runtime]T
    len int
    capacity int

    // ============================================================================
    // Construction
    // ============================================================================

    /// Create new empty list with initial capacity 4
    #[vm, c]
    static fn new() List<T> {
        let list List<T>
        list.data = alloc_array<T>(4)
        list.len = 0
        list.capacity = 4
        list
    }

    /// Create list with capacity
    #[vm, c]
    static fn with_capacity(cap int) List<T> {
        let list List<T>
        list.data = alloc_array<T>(cap)
        list.len = 0
        list.capacity = cap
        list
    }

    // ============================================================================
    // Element Access
    // ============================================================================

    /// Get element at index (panics if out of bounds)
    #[vm, c]
    fn get(index int) T {
        if index < 0 || index >= .len {
            panic("index out of bounds")
        }
        .data[index]
    }

    /// Set element at index (panics if out of bounds)
    #[vm, c]
    fn set(index int, value T) {
        if index < 0 || index >= .len {
            panic("index out of bounds")
        }
        .data[index] = value
    }

    // ============================================================================
    // Capacity Management
    // ============================================================================

    /// Returns number of elements
    #[vm, c]
    fn len() int {
        .len
    }

    /// Returns total capacity
    #[vm, c]
    fn capacity() int {
        .capacity
    }

    /// Returns 1 if empty, 0 otherwise
    #[vm, c]
    fn is_empty() int {
        if .len == 0 { 1 } else { 0 }
    }

    // ============================================================================
    // Modification
    // ============================================================================

    /// Add element to end (grows capacity if needed)
    #[vm, c]
    fn push(elem T) {
        if .len >= .capacity {
            .realloc(.capacity * 2)
        }
        .data[.len] = elem
        .len = .len + 1
    }

    /// Remove and return last element
    #[vm, c]
    fn pop() T {
        if .len == 0 {
            panic("cannot pop from empty list")
        }
        .len = .len - 1
        .data[.len]
    }

    /// Insert element at index
    #[vm, c]
    fn insert(index int, elem T) {
        if index < 0 || index > .len {
            panic("index out of bounds")
        }
        if .len >= .capacity {
            .realloc(.capacity * 2)
        }
        // Shift elements right
        for i in (index ... len).reverse() {
            .data[i + 1] = .data[i]
        }
        .data[index] = elem
        .len = .len + 1
    }

    /// Remove element at index
    #[vm, c]
    fn remove(index int) T {
        if index < 0 || index >= .len {
            panic("index out of bounds")
        }
        let value = .data[index]
        // Shift elements left
        for i in index ... (len - 1) {
            .data[i] = .data[i + 1]
        }
        .len = .len - 1
        value
    }

    /// Clear all elements
    #[vm, c]
    fn clear() {
        .len = 0
    }

    // ============================================================================
    // Internal Memory Management
    // ============================================================================

    /// Reallocate to new capacity (internal)
    #[vm, c]
    fn realloc(new_cap int) {
        if new_cap <= .capacity {
            return  // Never shrink
        }
        let new_data = realloc_array<T>(.data, new_cap)
        .data = new_data
        .capacity = new_cap
    }

    // ============================================================================
    // Destruction
    // ============================================================================

    /// Cleanup (called by VM GC or explicit)
    #[c]
    fn drop() {
        free_array<T>(.data)
    }
}
```

**Key Design Decisions**:
1. **Growth Strategy**: Double capacity when full (standard approach)
2. **No Shrinking**: `realloc()` never shrinks to avoid thrashing
3. **Explicit Drop**: A2C generates `free()` calls, VM uses GC
4. **Panic on Errors**: Out-of-bounds access panics (Rust-style)

#### Step 3: VM Function Registration (1 hour)

**File**: `crates/auto-lang/src/interp.rs`

**In `Interpreter::new()`** (约 line 35):
```rust
// Register memory management functions
self.register_native("alloc_array", vm::memory::alloc_array);
self.register_native("free_array", vm::memory::free_array);
self.register_native("realloc_array", vm::memory::realloc_array);
```

**Testing**:
```bash
cargo test -p auto-lang test_vm_memory_functions
```

#### Step 4: C Transpiler Realloc Support (2-3 hours)

**File**: `crates/auto-lang/src/trans/c.rs`

**Add to `transpile_expr()`** (约 line 1200):
```rust
Expr::Call { func, args } => {
    let func_name = self.expr_name(func);

    match func_name.as_str() {
        "alloc_array" => {
            let size = self.transpile_expr(&args[0], sink)?;
            Ok(format!("malloc(sizeof(void*) * {})", size))
        }
        "realloc_array" => {
            let ptr = self.transpile_expr(&args[0], sink)?;
            let size = self.transpile_expr(&args[1], sink)?;
            Ok(format!("realloc({}, sizeof(void*) *) {})", ptr, size))
        }
        "free_array" => {
            let arr = self.transpile_expr(&args[0], sink)?;
            output!(sink, "free({});", arr);
            Ok("void".to_string())
        }
        _ => {
            // ... existing call handling ...
        }
    }
}
```

**Testing**:
```bash
cargo test -p auto-lang test_a2c_084_list_realloc
```

#### Step 5: Update Prelude (0.5 hours)

**File**: `stdlib/auto/prelude.at`

**Add List import**:
```auto
// ============================================================================
// Collections
// ============================================================================
use auto.list: List
```

**Remove old comment** about List being disabled (lines 35-40).

#### Step 6: Comprehensive Testing (2-3 hours)

**VM Tests** (`src/tests/self_hosted_list_tests.rs` 新建):
```rust
#[test]
fn test_self_hosted_list_push() {
    let code = r#"
        use auto.list: List
        fn main() {
            let list = List.new()
            list.push(1)
            list.push(2)
            list.push(3)
            list.len()
        }
    "#;
    assert_eq!(run(code).unwrap(), "3");
}

#[test]
fn test_self_hosted_list_growth() {
    let code = r#"
        use auto.list: List
        fn main() {
            let list = List.new()  // capacity = 4
            for i in 0...10 {
                list.push(i)  // Force realloc at 4, 8
            }
            list.capacity()  // Should be 16
        }
    "#;
    assert!(run(code).unwrap().contains("16"));
}

#[test]
fn test_self_hosted_list_manipulation() {
    let code = r#"
        use auto.list: List
        fn main() {
            let list = List.new()
            list.push(10)
            list.push(20)
            list.insert(1, 15)
            list.remove(0)  // Remove 10
            list.get(0)  // Should be 15
        }
    "#;
    assert_eq!(run(code).unwrap(), "15");
}
```

**A2C Tests** (`test/a2c/084_self_hosted_list/` 新建):
```auto
// test.at
use auto.list: List

fn main() {
    let list = List.new()
    list.push(1)
    list.push(2)
    list.push(3)
    let len = list.len()
    let cap = list.capacity()
    printf("len=%d cap=%d\n", len, cap)
}
```

**Expected C** (`expected.c`):
```c
#include "self_hosted_list.h"

int main(void) {
    List_int* list = List_int_new();
    List_int_push(list, 1);
    List_int_push(list, 2);
    List_int_push(list, 3);
    int len = List_int_len(list);
    int cap = List_int_capacity(list);
    printf("len=%d cap=%d\n", len, cap);
    free(list->data);
    free(list);
    return 0;
}
```

#### Step 7: Documentation (1 hour)

**File**: `CLAUDE.md`

**Add section**:
```markdown
## Self-Hosted List<T>

AutoLang's List<T> is fully implemented in the standard library (stdlib/auto/list.at),
not in the compiler. This provides:

1. **Transparency**: Users can see and modify List behavior
2. **Portability**: Works identically in VM and transpiled C
3. **Educational**: Shows how dynamic arrays work

**Memory Management**:
- VM: Uses `alloc_array()` / `realloc_array()` (Rust-implemented)
- A2C: Uses `malloc()` / `realloc()` from C stdlib
- Growth: Doubles capacity when full (4→8→16→32...)
```

**Update** `docs/plans/052-runtime-array-allocation.md` status to "Phase 2 ✅ COMPLETED"

### Success Criteria (Phase 2)

1. ✅ VM memory functions implemented and tested
2. ✅ stdlib/auto/list.at contains full List<T> implementation
3. ✅ Manual realloc logic visible in AutoLang code
4. ✅ VM tests pass (push, pop, growth, insert, remove)
5. ✅ A2C tests pass (generates correct malloc/realloc/free)
6. ✅ Prelude exports List<T>
7. ✅ Zero breaking changes (existing code still works)
8. ✅ Documentation updated

### Benefits of Self-Hosted List<T>

**Before** (Rust Vec wrapper):
- ❌ List implementation hidden in Rust
- ❌ Can't see or modify growth strategy
- ❌ `capacity()` returns misleading `i32::MAX`
- ❌ Different behavior in VM vs A2C

**After** (Self-hosted in AutoLang):
- ✅ List implementation in stdlib/auto/list.at (visible)
- ✅ Users can customize growth strategy
- ✅ `capacity()` returns actual capacity
- ✅ Same behavior in VM and A2C
- ✅ Educational: shows how dynamic arrays work
- ✅ True self-hosting: stdlib implemented in AutoLang

## Risks & Mitigations

### R1: C99 VLA Support

**Risk**: Not all C compilers support VLAs (MSVC < 2013)

**Mitigation**:
- Use heap allocation for large arrays regardless
- Document C99 requirement for small arrays
- Provide compile-time fallback option

### R2: Memory Leaks

**Risk**: Heap-allocated arrays may leak if not freed

**Mitigation**:
- Automatic cleanup at scope exit
- RAII helper functions
- Escape analysis to determine lifetime
- Comprehensive testing with valgrind/ASAN

### R3: Stack Overflow

**Risk**: Large VLAs on stack can cause overflow

**Mitigation**:
- Size threshold (e.g., 1KB) forces heap allocation
- Configurable threshold via compiler flag
- Document limitation clearly

### R4: Type System Complexity

**Risk**: Runtime arrays complicate type checking

**Mitigation**:
- Start with `int` size only (simplest case)
- Add type inference gradually
- Clear error messages for invalid sizes

### R5: Backwards Compatibility

**Risk**: Breaking existing code with fixed arrays

**Mitigation**:
- Keep `[constant]T` as `Type::Array` (unchanged)
- Only use `RuntimeArray` for variable sizes
- Zero breaking changes to existing code

## Implementation Order

### Step 1: AST and Parser (2-3 hours)
- Add `Type::RuntimeArray` to AST
- Extend parser to accept expressions for size
- Handle both constant and variable sizes

### Step 2: Type Checking (2-3 hours)
- Add type rules for runtime arrays
- Validate size expression type
- Add sizeof calculations

### Step 3: C Transpiler - Basic (3-4 hours)
- Transpile size expression
- Generate C99 VLA code
- Test with small arrays

### Step 4: C Transpiler - Heap (3-4 hours)
- Add heap allocation for large arrays
- Implement automatic cleanup
- Test with large arrays

### Step 5: VM Evaluator (2-3 hours)
- Support runtime arrays in evaluator
- Use Vec<Value> for storage
- Test with various sizes

### Step 6: Comprehensive Testing (2-3 hours)
- Create 8+ test cases
- Test edge cases (size=0, size=1, size=MAX)
- Test cleanup and memory management
- Verify no leaks

### Step 7: Documentation (1 hour)
- Document usage and limitations
- Update CLAUDE.md
- Add examples

**Total Estimated Time**: 15-23 hours

## Success Criteria

1. ✅ Parser accepts `[expr]int` where expr is variable/function call
2. ✅ Type checking validates size expression
3. ✅ C transpiler generates correct code (VLA or malloc)
4. ✅ VM evaluator supports runtime-sized arrays
5. ✅ Small arrays use stack allocation
6. ✅ Large arrays use heap allocation
7. ✅ Automatic cleanup works (no leaks)
8. ✅ All 8+ tests pass
9. ✅ Zero breaking changes to existing fixed arrays
10. ✅ Documentation complete

## Future Enhancements

Once basic runtime allocation works:

1. **Reallocation Helper**: Helper function for growing arrays
2. **Slice Support**: Slices of runtime arrays
3. **Multi-dimensional**: `[m][n]int` runtime sizes
4. **Initialization**: `[expr]T = [default; expr]` syntax
5. **Generics**: `[expr]T` for any type T

## Dependencies

- **Required**: Parser infrastructure (✅ exists)
- **Required**: Type system (✅ exists)
- **Required**: C transpiler (✅ exists)
- **Required**: VM evaluator (✅ exists)
- **Optional**: Escape analysis system (can be added later)
- **Optional**: Garbage collection (can use manual free for now)

## Next Steps After This Plan

### Immediate Next Steps (Phase 2 Implementation)

1. **Create VM Memory Module** (`src/vm/memory.rs`)
   - Implement `alloc_array<T>()`, `free_array<T>()`, `realloc_array<T>()`
   - Register functions in VM interpreter
   - Add unit tests

2. **Update ArrayData Structure** (`src/vm/value.rs`)
   - Add `capacity: usize` field to distinguish from `len`
   - Update all ArrayData creation sites

3. **Implement Self-Hosted List<T>** (`stdlib/auto/list.at`)
   - Full List<T> implementation with manual realloc
   - Methods: `new()`, `push()`, `pop()`, `get()`, `set()`, `insert()`, `remove()`
   - Capacity management: `len()`, `capacity()`, `is_empty()`, `clear()`

4. **C Transpiler Realloc Support** (`src/trans/c.rs`)
   - Transpile `alloc_array()` → `malloc()`
   - Transpile `realloc_array()` → `realloc()`
   - Transpile `free_array()` → `free()`
   - Add automatic cleanup at scope exit

5. **Testing**
   - VM tests for self-hosted List<T> behavior
   - A2C tests for correct C code generation
   - Memory leak tests (valgrind/ASAN)

6. **Documentation**
   - Update CLAUDE.md with self-hosted List<T> explanation
   - Add examples showing manual realloc logic
   - Document VM vs A2C memory management differences

### Future Enhancements (After Phase 2)

1. **Custom Growth Strategies**: Allow users to specify custom growth factors
2. **Arena Allocator**: Alternative to malloc/free for embedded systems
3. **Memory Pool**: Pre-allocated pool for fixed-size allocations
4. **Generics**: Full generic List<T> support with type inference
5. **Iteration Support**: Integrate with Plan 053 (for loops)
6. **Slice Support**: Borrow slices of List<T> without copying

### Relationship to Other Plans

1. ✅ **Plan 054 Phase 2** (Root Config): Enabled by Phase 1 ✅
2. ✅ **Plan 055 Phase 7** (C Transpiler): Enabled by Phase 1 ✅
3. ⏸️ **Plan 053** (Iteration): Will integrate with self-hosted List<T>
4. ⏸️ **Plan 051** (Fixed-Capacity List): DEPRECATED - superseded by Plan 055

### Dependencies

**Required by Phase 2**:
- ✅ Phase 1 complete (runtime array syntax)
- ✅ RuntimeArray type in AST
- ✅ Parser accepts `[expr]T`
- ✅ VM evaluator allocates runtime arrays
- ✅ C transpiler generates malloc calls

**Enables Future Work**:
- Phase 2 completion → True self-hosted stdlib
- Custom allocators for embedded systems
- Memory-mapped I/O for MCU targets
- Zero-copy data structures

## Current Status

**Status**: Phase 1 ✅ COMPLETED (2025-01-23), Phase 2 🔄 IN PROGRESS

**Phase 1 Completed** (Runtime Array Syntax):
- ✅ Problem analysis
- ✅ Design options evaluated
- ✅ Implementation strategy chosen (heap allocation approach)
- ✅ **Phase 1: AST and Parser** - RuntimeArray type added to AST, parser supports `[expr]T`
- ✅ **Phase 2: Type System** - substitute() and occurs_in() support RuntimeArray
- ✅ **Phase 3: C Transpiler** - Generates `int* arr = malloc(sizeof(int) * (size))`
- ✅ **Phase 4: VM Evaluator** - Runtime array allocation via `eval_store()`
- ✅ **Phase 5: Testing** - 2 a2c tests passing (test_082, test_083)

**Phase 2 In Progress** (Self-Hosted List<T>):
- ⏸️ Prerequisites: VM memory management functions
- ⏸️ Implementation: Manual realloc logic in list.at
- ⏸️ Testing: Self-hosted List verification
- ⏸️ Documentation: Update CLAUDE.md

**Blocked**: Need VM memory functions (alloc_array, free_array, realloc_array)

**Priority**: 🔄 HIGH - Required for true self-hosted stdlib

**Implementation Files**:

### Phase 1 Files (✅ COMPLETED):
- `src/ast/types.rs` - RuntimeArray type definition (line 21, 264-267)
- `src/parser.rs` - parse_array_type() supports runtime expressions (line 4220-4247)
- `src/eval.rs` - eval_store() allocates runtime arrays (line 894-907)
- `src/trans/c.rs` - C generation with malloc (line 1518-1523, 1680-1703)
- `src/infer/unification.rs` - occurs_in() support (line 142)
- `test/a2c/082_runtime_size_var/` - Variable size test
- `test/a2c/083_runtime_size_expr/` - Expression size test

### Phase 2 Files (🔄 TO BE CREATED):
- `src/vm/memory.rs` - VM memory management functions (NEW)
- `src/vm/value.rs` - ArrayData with capacity field (UPDATE)
- `src/interp.rs` - Register memory functions (UPDATE)
- `src/trans/c.rs` - Realloc transpilation (UPDATE)
- `stdlib/auto/list.at` - Self-hosted List<T> implementation (UPDATE)
- `stdlib/auto/prelude.at` - Export List<T> (UPDATE)
- `src/tests/self_hosted_list_tests.rs` - VM tests (NEW)
- `test/a2c/084_self_hosted_list/` - A2C tests (NEW)

**Relationship to Other Plans**:

### Plan 054 (Context Environment)
- **Dependency**: Plan 054 Phase 2 (Root Config) requires runtime allocation
  - User can specify `const HeapSize = 4096` in config
  - This heap needs runtime allocation support
- **Enables**: Full MCU vs PC environment customization
  - MCU: User-defined static heap arrays
  - PC: Dynamic heap allocation with malloc

### Plan 055 (Storage Injection)
- **Current State**: Plan 055 implements basic Storage infrastructure
  - ✅ Storage type markers (Fixed/Dynamic) work
  - ✅ Target detection works
  - ✅ Environment injection works
  - ⏸️ But actual dynamic List growth still requires runtime allocation
- **Enhancement**: Plan 052 enables Plan 055 Phase 7 (C Transpiler Enhancements)
  - Generate static arrays for MCU Fixed storage
  - Generate heap-allocated structures for PC Dynamic storage
  - Support user-specified heap sizes from root config

### Plan 051 (Fixed-Capacity List)
- **Status**: DEPRECATED - superseded by Plan 055
- **Legacy**: Plan 051's array testing findings inform Plan 052 design

### Plan 050 (Prelude System)
- **No Direct Dependency**: Prelude system doesn't require runtime arrays
- **Future**: Could enable dynamic prelude configuration

**Notes**:
- This is a major language feature
- Requires careful testing for memory safety
- Backwards compatibility is critical
- Hybrid approach balances complexity and performance
- **Important**: Plan 054/055 can proceed without Plan 052, but advanced features require it

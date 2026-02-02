# Plan 068: AutoVM (BigVM) Implementation

**Status**: 🟡 Active - Phase 6 Complete, Phase 7-9 Pending
**Owner**: AutoLang Team
**Related**: `docs/design/auto-vm-bigvm.md`, `docs/design/abc.md`

**Recent Updates** (2025-02-03):
- ✅ **Symbol Table Implementation**: Complete symbol table with scope tracking in Codegen
- ✅ **Memory Corruption Fix**: Fixed critical bug where stack would overwrite local variables
- ✅ **List Support**: Full List implementation with 9 native functions
- ✅ **Native Function Registry**: Runtime native function mapping with automatic ID resolution
- ✅ **Entry Point Resolution**: Automatic main/test/ address 0 lookup
- ✅ **Iterator Support (Phase 7.1)**: Implemented `List.iter()` and `Iterator.next()` native functions

## 1. Objective

Implement **AutoVM (BigVM)**, a bytecode-based virtual machine for AutoLang, to replace the current `eval.rs` TreeWalker interpreter.
BigVM is designed to be a "Digital Twin" of the MicroVM (embedded runtime), ensuring that behavior on PC matches the microcontroller environment exactly (stack overflow, memory alignment, wrapping arithmetic, etc.).

## 2. Architecture Recap

- **ISA**: AutoByteCode (ABC) v1.0, a variable-length, stack-based instruction set.
- **Memory Model**:
    - **VirtualFlash**: `Vec<u8>` - Read-only byte array for code and constants (XIP simulation).
    - **VirtualRAM**: `Vec<i32>` - Read-write array for Stack and Heap.
      - **Note**: Originally `Vec<Word>` union, simplified to `Vec<i32>` (2025-02-03) to eliminate memory corruption bugs
      - **Memory Layout**: Local variables at `bp+0, bp+1, ...`, stack grows from `sp` (where `sp >= num_locals`)
- **Execution**: `loop { match op { ... } }` dispatch.

## 3. Implementation Phases

### Phase 1: The Core Framework (ISA & Memory)
**Goal**: Establish the VM scaffolding and execute simple arithmetic (`1 + 2`).

- [x] **1.1 ABC Definitions**: Create `crates/auto-lang/src/vm/opcode.rs`.
    - Define `enum OpCode` matching `abc.md`.
    - Implement `impl From<u8> for OpCode`. or decode logic.
- [x] **1.2 Virtual Memory**: Create `crates/auto-lang/src/vm/memory.rs` (update existing).
    - Define `union Word`.
    - Implement `struct VirtualFlash` with `read_u8`, `read_i32`.
    - Implement `struct VirtualRAM` with `push`, `pop`, `read`, `write`.
- [x] **1.3 Execution Engine**: Create `crates/auto-lang/src/vm/engine.rs`.
    - Define `struct BigVM`.
    - Implement the main decode-dispatch loop.
    - Implement `CONST_I32`, `ADD`, `HALT`.
- [x] **1.4 Minimal Assembler/Codegen**:
    - Create a unit test that manually constructs a `Vec<u8>` bytecode behavior.
    - Verify `1 + 1 = 2` relies on the stack.

### Phase 2: Control Flow & Variables
**Goal**: Execute logic with branches and local variables (`if`, `let`).

- [x] **2.1 Stack Frames**:
    - Add `bp` (Base Pointer) to `BigVM`.
    - Implement `LOAD_LOCAL`, `STORE_LOCAL` relative to `bp`.
    - **Memory Corruption Fix** (2025-02-03): Fixed critical bug where stack would overwrite local variables.
      - **Root Cause**: Stack and locals shared same memory space starting at address 0
      - **Solution**: Reserve stack space for locals at function entry by pushing dummy `CONST_0` values
      - This ensures `sp` starts at `n_locals`, preventing stack operations from overwriting locals
- [x] **2.2 Jumps**:
    - Implement `JMP`, `JMP_IF_Z`, `JMP_IF_NZ`.
    - Handle 16-bit relative offsets.
- [x] **2.3 Compiler Backend (Basic)**:
    - Create `crates/auto-lang/src/vm/codegen.rs`.
    - **Symbol Table Implementation** (2025-02-03):
      - Added `locals: HashMap<String, usize>` to track variables in current scope
      - Added `scope_stack: Vec<HashMap<String, usize>>` for nested scope support
      - Implemented `lookup_var()` - searches all scopes from innermost to outermost
      - Implemented `add_var()` - adds variable to current scope, returns index
      - Implemented `push_scope()` and `pop_scope()` for scope management
      - Updated `Stmt::Store` to use symbol table and emit correct `STORE_LOC_N` opcodes
      - Updated `Expr::Ident` to use symbol table and emit correct `LOAD_LOC_N` opcodes
      - Updated `Stmt::Fn` to push/pop scopes for each function
      - Added `emit_store_loc()` and `emit_load_loc()` with fast-path opcodes for locals 0-2
      - Implemented assignment (`Op::Asn`) with proper symbol table lookup
    - Handle `Expression`, `Block`, `IfStatement`.
    - **Status**: ✅ Symbol table complete, multiple local variables per function working correctly

### Phase 3: Functions & Calls
**Goal**: Support function calls, recursion, and parameter passing.

- [x] **3.1 Call Infrastructure**:
    - Implement `CALL` (push ret_addr, old_bp; bp = sp).
    - Implement `RET` (cleanup stack, restore bp/ip).
- [x] **3.2 Symbol Linking**:
    - Implement `SymbolTable` to map Function Name -> Flash Address.
    - Implement `CALL` patchup (compiler emits placeholder, updates after function offset known).

### Phase 4: Native Interface (FFI)
**Goal**: Call `std::print` and other Rust-hosted functions.

- [x] **4.1 Shim Registry**:
    - Define `type ShimFunc = Fn(&mut VirtualRAM)`.
    - Create `NativeInterface` in VM.
- [x] **4.2 Standard Library Shims**:
    - Implement `print` shim.
    - Implement `CALL_NAT` instruction.

### Phase 5: Integration & Migration
**Goal**: Replace `Evaler` with `BigVM`.

- [x] **5.1 Runner Integration**: Create `crates/auto-vm` to compile-then-run `AT` files.
- [/] **5.2 Test Suite**: Port interpreter tests from `tests/vm_tests.rs` and related files to BigVM.
    - [x] Test infrastructure (`run_bigvm` helper in `vm/tests_bigvm.rs`)
    - [x] Category A: Primitives (arithmetic, unary ops, comparisons)
    - [x] Category A: Control flow (if/else expressions)  
    - [x] Category A: Functions & calls (CALL/RET, locals, recursion)
    - [ ] Category B: Data structures (Lists, Maps - blocked by Phase 6.3)


### Phase 6: Data Structures & Heap (Prerequisite for Tests)
**Goal**: Support Strings, Arrays, and Objects with Linear Memory management (RAII).

- [x] **6.1 Heap Model**: Implement `LinearAllocator` and RAII-style lifetime management (Auto-Free).
- [x] **6.2 Strings**: Implement `String` support (constant pool, `LOAD_STR` opcode, `print_str`).
- [x] **6.3 Collections**: Implement `List` (dynamic array) and `Map` (objects).
    - **List Native Functions** (2025-02-03):
      - Created `BigVMNativeRegistry` for runtime native function mapping
      - Implemented 9 List native shims: `new`, `push`, `pop`, `len`, `is_empty`, `clear`, `get`, `set`, `drop`
      - Added List storage to BigVM using `DashMap<u64, Arc<RwLock<Vec<i32>>>>`
      - Fixed RwLock panic by switching from `tokio::sync::RwLock` to `std::sync::RwLock`
      - Changed from union to struct for `Word`, then to `Vec<i32>` for simpler memory management
    - **Status**: ✅ All List operations working, comprehensive tests passing
- [x] **6.4 Stdlib Hooks**: Connect `List.new`, `push`, `len` to VM native functions.
    - **Native Function Registry** (2025-02-03):
      - Implemented `BIGVM_NATIVES` lazy_static for runtime function name → ID mapping
      - `register_builtin_natives()` registers List methods at startup
      - Codegen emits `CALL_NAT` with resolved native ID (no relocation needed)
      - Supports both static methods (`List.new`) and instance methods (`List.len(list)`)
    - **Entry Point Resolution** (2025-02-03):
      - Implemented automatic entry point lookup: `main()` → `test()` → address 0
      - Fixed type mismatch (u32 vs usize) in task spawning
    - **Status**: ✅ Complete, all List tests passing

### Phase 7: Advanced Features
**Goal**: Support closures and iterators used in `list_tests.rs`.

- [ ] **7.1 Closures**: Implement `CLOSURE` opcode and Upvalues.
- [ ] **7.2 Iterators**: Implement iterator protocol for `for` loops.

### Phase 8: Comprehensive Test Migration  
**Goal**: Port ALL interpreter tests to BigVM.

- [x] **8.1 Test Infrastructure**:
    - Created `crates/auto-lang/src/vm/tests_bigvm.rs` module.
    - Implemented test harness `run_bigvm(code) -> Result<String, String>`.
- [x] **8.2 Port Primitives & Control Flow**:
    - ✅ Arithmetic operators (`+`, `-`, `*`, `/`)
    - ✅ Unary operators (`-`, `!`)
    - ✅ Comparison operators (`<`, `>`, `==`, `!=`, `<=`, `>=`)
    - ✅ If/else expressions
- [x] **8.3 Port Functions & Calls**:
    - ✅ Function definitions and calls
    - ✅ Local variables
    - ✅ Recursion tests
- [ ] **8.4 Port Complex Types**:
    - [ ] `list_tests.rs` (blocked: needs Phase 6.3 List implementation)
    - [ ] `string_tests.rs` (partial: basic strings work, advanced tests pending)
    - [ ] `object_tests.rs` (blocked: needs Phase 6.3 Map implementation)

### Phase 9: Deprecation & Replacement
**Goal**: Replace `Evaler` with `BigVM`.

- [ ] **9.1 Benchmarking**: Compare BigVM vs Evaler performance.
- [ ] **9.2 Feature Parity Check**: Ensure all tests pass.
- [ ] **9.3 Switchover**: Update `auto-shell` and `auto-run` to use BigVM by default.

## 4. Work Breakdown & Task List

### Step 1: Core Structs
Create the basic memory and opcode structures.

### Step 2: CPU Loop
Implement the fetch-decode-execute cycle for arithmetic.

### Step 3: Compiler Basics
Translate simple AST directly to byte buffers.

### Step 4: Branching
Add Jump support and `if/else` compilation.

### Step 5: Function Calls
Implement stack frame management and call instructions.

### Step 6: Heap & Collections
Implement Linear Memory Manager, Strings, Lists, and Maps.

### Step 7: Migration
Systematically port tests and verify parity.

## 5. Critical Bug Fixes & Learnings

### 5.1 Memory Corruption Bug (2025-02-03)

**Problem**: When using multiple local variables in a function, the stack would overwrite the local variables.

**Symptom**:
```auto
fn main() {
    let a = 10
    let b = 20
    print(a)  // Printed 20 instead of 10!
    print(b)  // Printed 20 (correct)
}
```

**Root Cause**:
- Stack and local variables shared the same memory space starting at address 0
- `STORE_LOC_0`: writes to `raw[bp+0] = raw[0] = 10`, `sp` becomes 0
- `CONST_I32 20`: pushes to `raw[sp=0] = 20`, overwriting `raw[0]`
- `STORE_LOC_1`: writes to `raw[bp+1] = raw[1] = 20`, `sp` becomes 0
- `LOAD_LOC_0`: reads `raw[0] = 20` (the value 10 was overwritten)

**Solution**:
At function entry, reserve stack space for local variables by pushing `n_locals` dummy `CONST_0` values. This ensures:
- `sp` starts at `n_locals` (not 0)
- Local variables occupy `raw[0..n_locals-1]`
- Stack operations use `raw[sp..]` where `sp >= n_locals`
- No overlap between locals and stack

**Implementation**:
```rust
// In codegen.rs Stmt::Fn compilation:
let n_locals = self.scope_stack.last().unwrap().len();

// Emit stack reservation at FUNCTION START (right after entry point)
if n_locals > 0 {
    // Insert CONST_0 opcodes at entry_point to reserve stack space
    for _ in 0..n_locals {
        self.code.insert(entry_point as usize, OpCode::CONST_0 as u8);
        self.code.insert(entry_point as usize + 1, 0u8);
        self.code.insert(entry_point as usize + 2, 0u8);
        self.code.insert(entry_point as usize + 3, 0u8);
        self.code.insert(entry_point as usize + 4, 0u8);
    }
}
```

**Status**: ✅ Fixed and tested. Multiple local variables now work correctly.

### 5.2 Word Union Memory Issues (2025-02-03)

**Problem**: `Vec<Word>` with union fields caused mysterious memory corruption.

**Symptoms**:
- Writing to `raw[1]` would also overwrite `raw[0]`
- Debug output showed values already written before actual write operations
- Issue persisted even after changing from union to struct

**Root Cause**:
- Union's `debug_ptr: usize` field (8 bytes in debug builds) caused alignment issues
- Compiler optimizations and unsafe code interactions caused unpredictable behavior

**Solution**:
- Phase 1: Changed from union to struct with single `i` field
- Phase 2: Simplified to `Vec<i32>` directly, eliminating `Word` wrapper entirely

**Status**: ✅ Resolved. VirtualRAM now uses `Vec<i32>` for clarity and correctness.

# Plan 068: AutoVM (BigVM) Implementation

**Status**: In Progress
**Owner**: AutoLang Team
**Related**: `docs/design/auto-vm-bigvm.md`, `docs/design/abc.md`

## 1. Objective

Implement **AutoVM (BigVM)**, a bytecode-based virtual machine for AutoLang, to replace the current `eval.rs` TreeWalker interpreter.
BigVM is designed to be a "Digital Twin" of the MicroVM (embedded runtime), ensuring that behavior on PC matches the microcontroller environment exactly (stack overflow, memory alignment, wrapping arithmetic, etc.).

## 2. Architecture Recap

- **ISA**: AutoByteCode (ABC) v1.0, a variable-length, stack-based instruction set.
- **Memory Model**:
    - **VirtualFlash**: `Vec<u8>` - Read-only byte array for code and constants (XIP simulation).
    - **VirtualRAM**: `Vec<Word>` - Read-write array for Stack and Heap.
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
- [x] **2.2 Jumps**:
    - Implement `JMP`, `JMP_IF_Z`, `JMP_IF_NZ`.
    - Handle 16-bit relative offsets.
- [x] **2.3 Compiler Backend (Basic)**:
    - Create `crates/auto-lang/src/compile/codegen.rs`.
    - Implement visiting `ast::Stmt` and `ast::Expr` to emit bytecode.
    - Handle `Expression`, `Block`, `IfStatement`.

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
- [ ] **6.3 Collections**: Implement `List` (dynamic array) and `Map` (objects).
- [ ] **6.4 Stdlib Hooks**: Connect `List.new`, `push`, `len` to VM native functions.

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

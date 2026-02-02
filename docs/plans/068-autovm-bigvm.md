# Plan 068: AutoVM (BigVM) Implementation

**Status**: Draft
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

- [ ] **1.1 ABC Definitions**: Create `crates/auto-lang/src/vm/opcode.rs`.
    - Define `enum OpCode` matching `abc.md`.
    - Implement `impl From<u8> for OpCode`. or decode logic.
- [ ] **1.2 Virtual Memory**: Create `crates/auto-lang/src/vm/memory.rs` (update existing).
    - Define `union Word`.
    - Implement `struct VirtualFlash` with `read_u8`, `read_i32`.
    - Implement `struct VirtualRAM` with `push`, `pop`, `read`, `write`.
- [ ] **1.3 Execution Engine**: Create `crates/auto-lang/src/vm/engine.rs`.
    - Define `struct BigVM`.
    - Implement the main decode-dispatch loop.
    - Implement `CONST_I32`, `ADD`, `HALT`.
- [ ] **1.4 Minimal Assembler/Codegen**:
    - Create a unit test that manually constructs a `Vec<u8>` bytecode behavior.
    - Verify `1 + 1 = 2` relies on the stack.

### Phase 2: Control Flow & Variables
**Goal**: Execute logic with branches and local variables (`if`, `let`).

- [ ] **2.1 Stack Frames**:
    - Add `bp` (Base Pointer) to `BigVM`.
    - Implement `LOAD_LOCAL`, `STORE_LOCAL` relative to `bp`.
- [ ] **2.2 Jumps**:
    - Implement `JMP`, `JMP_IF_Z`, `JMP_IF_NZ`.
    - Handle 16-bit relative offsets.
- [ ] **2.3 Compiler Backend (Basic)**:
    - Create `crates/auto-lang/src/compile/codegen.rs`.
    - Implement visiting `ast::Stmt` and `ast::Expr` to emit bytecode.
    - Handle `Expression`, `Block`, `IfStatement`.

### Phase 3: Functions & Calls
**Goal**: Support function calls, recursion, and parameter passing.

- [ ] **3.1 Call Infrastructure**:
    - Implement `CALL` (push ret_addr, old_bp; bp = sp).
    - Implement `RET` (cleanup stack, restore bp/ip).
- [ ] **3.2 Symbol Linking**:
    - Implement `SymbolTable` to map Function Name -> Flash Address.
    - Implement `CALL` patchup (compiler emits placeholder, updates after function offset known).

### Phase 4: Native Interface (FFI)
**Goal**: Call `std::print` and other Rust-hosted functions.

- [ ] **4.1 Shim Registry**:
    - Define `type ShimFunc = Fn(&mut VirtualRAM)`.
    - Create `NativeInterface` in VM.
- [ ] **4.2 Standard Library Shims**:
    - Implement `print` shim.
    - Implement `CALL_NAT` instruction.

### Phase 5: Integration & Migration
**Goal**: Replace `Evaler` with `BigVM`.

- [ ] **5.1 REPL Integration**: Switch `auto-shell` to compile-then-run mode.
- [ ] **5.2 Test Suite**: Port `a2c_tests` to run on BigVM.

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

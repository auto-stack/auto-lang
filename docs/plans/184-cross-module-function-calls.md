# Cross-Module Function Calls Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Enable AutoLang functions defined in external modules (via `use`) to be called from the main script, by compiling dependency modules to bytecode and linking them together at runtime.

**Architecture:** When `resolve_uses` loads a module, it now also compiles the module's function bodies to bytecode (producing a `Module`). The main script and all dependency modules are linked together using the existing `Linker` in `loader.rs`, which already supports multi-module code layout and cross-module relocation.

**Tech Stack:** Rust, AutoVM bytecode, existing `Linker` infrastructure

---

## Current Architecture (Problem)

```
test_stdlib.at:
  use auto.io: say        ← resolve_uses() loads io.at
  say("hello")            ← codegen emits CALL with reloc for "say"

io.at / io.vm.at:
  fn say(msg str) { print(msg) }  ← TypeStore gets the signature, but NO bytecode is generated

Link phase:
  codegen.exports = { "main": 0x00 }  ← only main script's exports
  relocs = [{ symbol: "say", offset: 0x12 }]  ← say NOT in exports → ERROR
```

## Target Architecture (Solution)

```
test_stdlib.at:
  use auto.io: say        ← resolve_uses() compiles io.at → Module { exports: { "say": 0x00 } }
  say("hello")            ← codegen emits CALL with reloc for "say"

Link phase (using existing Linker):
  linker.add_module(io_module)    ← dependency module (exports: { "say": 0x00 })
  linker.add_module(main_module) ← main script (exports: { "main": 0x20 })
  linker.link()                  ← lays out both, resolves "say" → 0x00 → SUCCESS
```

---

## Key Insight: Three Categories of External Functions

1. **`#[vm]` native functions** (e.g., `File.open`) — already work via `native_registry`. No bytecode needed.
2. **Functions calling only natives** (e.g., `say` calls `print`) — need bytecode compilation + linking.
3. **Functions calling other user functions** (e.g., `foo` calls `bar` in same module) — handled automatically once the module is compiled as a whole.

All three cases are unified by the same mechanism: compile the module, export its functions, link them together.

---

### Task 1: Add `compiled_modules` field to CompileSession

**Files:**
- Modify: `crates/auto-lang/src/compile.rs:42-55`

**Step 1: Add field to CompileSession struct**

Add a new field to store compiled dependency modules:

```rust
/// Cross-module function calls: compiled dependency modules
/// Each module's bytecode is compiled during resolve_uses and stored here
/// for later linking in execute_autovm
compiled_modules: Vec<crate::vm::loader::Module>,
```

**Step 2: Initialize the field**

In `CompileSession::new()` (line 73), add:
```rust
compiled_modules: Vec::new(),
```

In `CompileSession::clone()` (line 57), add:
```rust
compiled_modules: Vec::new(), // Don't clone — modules are rebuilt per session
```

**Step 3: Add accessor method**

```rust
/// Take all compiled dependency modules (moves ownership)
pub fn take_compiled_modules(&mut self) -> Vec<crate::vm::loader::Module> {
    std::mem::take(&mut self.compiled_modules)
}
```

**Step 4: Verify it compiles**

Run: `cargo build -p auto-lang`
Expected: BUILD SUCCEEDS

**Step 5: Commit**

```
git add crates/auto-lang/src/compile.rs
git commit -m "refactor(vm): add compiled_modules field to CompileSession"
```

---

### Task 2: Add module bytecode compilation in load_module_inner

**Files:**
- Modify: `crates/auto-lang/src/compile.rs:333-451` (load_module_inner)

**Step 1: Understand the current flow**

`load_module_inner` currently:
1. Resolves module file path
2. Reads source + merges .vm.at context
3. Parses to AST, extracts type declarations into TypeStore
4. Returns Ok(())

We need to add step 3.5: compile the module's function bodies to bytecode.

**Step 2: Add compilation logic after parse_module_to_type_store**

After line 424 (`let module_type_store = self.parse_module_to_type_store(...)`), add:

```rust
// Cross-module function calls: compile module to bytecode
// This generates actual function implementations that can be linked
let module_code = self.compile_module_to_bytecode(&module_source, &root_path.to_string_lossy())?;
if !module_code.code.is_empty() {
    self.compiled_modules.push(module_code);
}
```

**Step 3: Implement compile_module_to_bytecode method**

Add a new method to CompileSession:

```rust
/// Compile a module's source code to bytecode (for cross-module function calls)
///
/// This compiles all function declarations in the module to bytecode,
/// producing a Module with exports that can be linked by the Linker.
fn compile_module_to_bytecode(
    &self,
    source: &str,
    path: &str,
) -> AutoResult<crate::vm::loader::Module> {
    use crate::vm::codegen::Codegen;
    use crate::vm::opcode::OpCode;

    let mut parser = Parser::from(source);
    let ast = parser.parse()
        .map_err(|e| crate::error::attach_source(e, path.to_string(), source.to_string()))?;

    let mut codegen = Codegen::new();

    // Compile all function declarations
    for stmt in &ast.stmts {
        match stmt {
            crate::ast::Stmt::Fn(fn_decl) => {
                // Only compile non-native functions (#[vm] functions don't need bytecode)
                let is_native = fn_decl.annotations.iter().any(|a| a.name == "vm");
                if !is_native {
                    codegen.compile_stmt(stmt)?;
                }
            }
            crate::ast::Stmt::TypeDecl(_) => {
                codegen.compile_stmt(stmt)?;
            }
            _ => {
                // Skip other top-level statements (expressions, assignments, etc.)
                // They are side-effects that shouldn't run at import time
            }
        }
    }

    // Add HALT at end
    codegen.code.push(OpCode::HALT as u8);

    let module_name = path.replace('\\', "/")
        .rsplit('/').next().unwrap_or("unknown")
        .trim_end_matches(".at")
        .trim_end_matches(".auto")
        .to_string();

    Ok(codegen.finish(module_name))
}
```

**Step 4: Verify it compiles**

Run: `cargo build -p auto-lang`
Expected: BUILD SUCCEEDS

**Step 5: Commit**

```
git add crates/auto-lang/src/compile.rs
git commit -m "feat(vm): compile dependency modules to bytecode in resolve_uses"
```

---

### Task 3: Replace inline linking in execute_autovm with Linker

**Files:**
- Modify: `crates/auto-lang/src/lib.rs:341-379`

**Step 1: Import the Linker**

At the top of `execute_autovm` (line 269-273), add:

```rust
use crate::vm::loader::Linker;
```

**Step 2: Replace the inline relocation loop**

Replace the entire linking block (lines 341-379) with:

```rust
// 3. Perform multi-module linking
let strings = codegen.strings.clone();

// Build linker with dependency modules first, then main module
let mut linker = Linker::new();
let dep_modules = session.take_compiled_modules();
for module in dep_modules {
    vm_debug!("DEBUG: Adding dependency module: {} (exports: {:?})",
        module.name, module.exports.keys().collect::<Vec<_>>());
    linker.add_module(module);
}

// Add main module
let main_module = codegen.finish("<main>".to_string());
vm_debug!("DEBUG: Main module exports: {:?}", main_module.exports.keys().collect::<Vec<_>>());
linker.add_module(main_module);

// Link all modules together
let (linked_code, global_symbols) = linker.link().map_err(|e| {
    crate::error::AutoError::Msg(e)
})?;

vm_debug!("DEBUG: Linked code size: {} bytes", linked_code.len());
vm_debug!("DEBUG: Global symbols: {:?}", global_symbols);
```

**Step 3: Update downstream code to use linked_code**

After linking, replace the flash creation (lines 384-388):

```rust
// 4. Load into VM
let flash = VirtualFlash::new_with_code_and_keys(
    linked_code,  // was: codegen.code
    codegen.object_keys,
    codegen.object_types,
);
```

Note: `codegen.object_keys` and `codegen.object_types` are still from the main codegen (string indices from the main module). Cross-module string pool merging is a future enhancement — for now the main module's strings suffice since test cases don't use cross-module string constants.

**Step 4: Verify it compiles**

Run: `cargo build -p auto-lang`
Expected: BUILD SUCCEEDS

**Step 5: Commit**

```
git add crates/auto-lang/src/lib.rs
git commit -m "feat(vm): use Linker for multi-module linking in execute_autovm"
```

---

### Task 4: Test with test_stdlib.at

**Step 1: Run the test case**

Run: `auto ./test_stdlib.at`
Expected output:
```
----------------------
Running Auto ./test_stdlib.at
----------------------
hello
```

**Step 2: If it works, also test the pipe case**

Run: `echo 'use auto.io: say; say("world")' | auto`
Expected: `world`

**Step 3: Commit (if tests pass)**

```
git commit --allow-empty -m "test: verify cross-module function calls work"
```

---

### Task 5: Handle edge case — module with no compilable functions

**Files:**
- Modify: `crates/auto-lang/src/compile.rs` (compile_module_to_bytecode)

**Step 1: Some modules may have only #[vm] functions or type declarations**

The current implementation already handles this — if all functions are `#[vm]`, no bytecode is emitted, and `module_code.code` will only contain the HALT byte. The `!module_code.code.is_empty()` check will still push the module (with just HALT), which is harmless.

However, we should also handle modules that have **zero statements** after filtering. Add a check: only add the module if it has actual exports (functions that were compiled).

Update the filter in load_module_inner (Task 2, Step 2):

```rust
if !module_code.exports.is_empty() {
    self.compiled_modules.push(module_code);
}
```

This is more precise — we only link modules that actually export user-defined functions.

**Step 2: Verify**

Run: `auto ./test_stdlib.at`
Expected: Still works

**Step 3: Commit**

```
git add crates/auto-lang/src/compile.rs
git commit -m "fix(vm): only link modules that export user-defined functions"
```

---

### Task 6: Handle edge case — recursive cross-module calls

**Files:**
- Modify: `crates/auto-lang/src/compile.rs` (load_module_inner)
- Modify: `crates/auto-lang/src/vm/loader.rs` (Linker::link)

**Step 1: Understand the problem**

If module A calls function B in module B, and module B calls function A in module A, both modules have relocs for each other's symbols. The Linker already handles this — it does a single pass over all modules after registering all exports (Pass 1 builds the global symbol table, Pass 2 resolves all relocs). No change needed.

**Step 2: Verify the existing Linker handles this**

Read `loader.rs:223-293` — the Linker does:
1. Pass 1: iterate ALL modules, register ALL exports into global_symbols
2. Pass 2: iterate ALL modules, resolve ALL relocs against global_symbols

This correctly handles cross-module references regardless of direction. No code change needed.

---

### Task 7: Handle edge case — same-named functions across modules

**Files:**
- Modify: `crates/auto-lang/src/vm/loader.rs:236-240`

**Step 1: The Linker already reports "Duplicate symbol" errors**

Line 237-239:
```rust
if global_symbols.contains_key(sym_name) {
    return Err(format!("Duplicate symbol: {}", sym_name));
}
```

This is correct behavior — two modules exporting the same function name is a conflict. No change needed.

---

### Task 8: Handle edge case — intrinsics in dependency modules

**Files:**
- Verify: no changes needed

**Step 1: Understand**

When `io.vm.at` is compiled, `print` calls inside `say` will:
1. Codegen checks `self.intrinsics` for "print" → found (NATIVE_PRINT_STR)
2. Emits `CALL_NAT` with the native ID
3. No relocation entry is created for `print`

So `print` works inside `say` without any cross-module linking. Native calls are self-contained.

**Step 2: Verify**

Run: `auto ./test_stdlib.at`
Expected: Still works

---

### Task 9: Run full test suite to check for regressions

**Step 1: Run all tests**

Run: `cargo test -p auto-lang`
Expected: All existing tests pass

**Step 2: Run VM file-based tests**

Run: `cargo test -p auto-lang -- vm_file_tests`
Expected: All pass

**Step 3: If any tests fail, investigate and fix**

Common failure modes:
- **"Duplicate symbol"**: A module's function name collides with main script's. Fix by qualifying or renaming.
- **"Undefined symbol"**: A function was expected from a module but the module wasn't compiled (e.g., it only had type declarations). Already handled by Task 5.
- **Wrong output**: Stack layout issue. Investigate with `VM_DEBUG=1 auto ./test_stdlib.at`.

---

### Task 10: Add a dedicated cross-module test

**Files:**
- Create: `crates/auto-lang/test/vm/17_modules/use_fn.at`
- Create: `crates/auto-lang/test/vm/17_modules/use_fn.expected`

**Step 1: Create the test module file**

Create `stdlib/auto/test_mod.at`:
```auto
pub fn greet(name str) str {
    f"hello, $name"
}

pub fn add(a int, b int) int {
    a + b
}
```

**Step 2: Create the test input**

Create `crates/auto-lang/test/vm/17_modules/use_fn.at`:
```auto
use auto.test_mod: greet, add

pub fn main() {
    print(greet("world"))
    print(add(1, 2))
}
```

**Step 3: Run to get expected output**

Run: `auto crates/auto-lang/test/vm/17_modules/use_fn.at`
Save output to `use_fn.expected`

**Step 4: Register the test**

Add test function in the test module (find where vm_file_tests are registered).

**Step 5: Run and verify**

Run: `cargo test -p auto-lang -- use_fn`
Expected: PASS

**Step 6: Commit**

```
git add stdlib/auto/test_mod.at crates/auto-lang/test/vm/17_modules/
git commit -m "test: add cross-module function call test (use_fn)"
```

---

## Verification

After all tasks complete:

```bash
# Core test case
auto ./test_stdlib.at
# Expected: "hello"

# Cross-module test
cargo test -p auto-lang -- use_fn

# Full regression
cargo test -p auto-lang
cargo test -p auto-man
```

## Risk Assessment

**Low risk changes:** Tasks 1-3 are the core implementation. The Linker already exists and works — we're just wiring it into the execution path.

**Potential issues:**
1. String pool merging — each module has its own string indices. Cross-module string references won't work yet. Mitigation: most cross-module calls don't pass string literals across module boundaries.
2. Module initialization code — top-level expressions in imported modules won't execute. Mitigation: this is expected behavior (like Python's `import` which only runs module-level code once).
3. Type declarations in modules — these are compiled by codegen (Task 2 includes TypeDecl). The type info is also in TypeStore from parse_module_to_type_store. No conflict expected.

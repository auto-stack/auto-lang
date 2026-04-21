# VM Function Integration Implementation Plan

**ID:** 001
**Status:** Planning
**Created:** 2026-01-08
**Requirements:** [docs/requirements/vm_functions.md](../requirements/vm_functions.md)

## Overview

Integrate native Rust VM functions with the AutoLang interpreter to enable standard library APIs backed by Rust implementations. This will make `hello.at` work and provide a foundation for extending the language with high-performance native functions.

## Current State

### ✅ Already Implemented
- **Parser**: Recognizes `fn.vm` syntax, sets `FnKind::VmFunction`
- **AST**: Has `FnKind::VmFunction` variant in `ast/fun.rs`
- **VM Functions**: `open`, `read_text`, `close` implemented in `vm/io.rs`
- **VM Reference Management**: Universe has `add_vmref`, `get_vmref`, `drop_vmref`

### ❌ Missing Components
- **VM Function Registry**: No centralized registry to map function names to implementations
- **Use Statement**: `eval_use()` returns `Value::Void` (not implemented)
- **Function Dispatch**: `eval_vm_fn_call()` returns "Not implemented yet"
- **Method Dispatch**: No handling for instance methods like `file.close()`

## Architecture Design

### Registry Structure

Create a centralized VM module registry with these components:

```rust
// In vm/mod.rs
pub type VmFunction = fn(Shared<Universe>, Value) -> Value;
pub type VmMethod = fn(Shared<Universe>, &mut Value, Vec<Value>) -> Value;

pub struct VmModule {
    pub name: AutoStr,
    pub functions: HashMap<AutoStr, VmFunctionEntry>,
    pub types: HashMap<AutoStr, VmTypeEntry>,
}

pub struct VmRegistry {
    modules: HashMap<AutoStr, VmModule>,
}

// Global registry instance
lazy_static! {
    pub static ref VM_REGISTRY: Mutex<VmRegistry> = VmRegistry::new();
}
```

### Key Design Decisions

1. **Lazy Loading**: Modules register when first loaded via `use` statement
2. **Separate Function/Method Types**: Functions are standalone, methods operate on instances
3. **Module-based Organization**: Groups functions logically (e.g., "auto.io", "auto.math")
4. **Thread-safe**: Uses `Mutex` for concurrent access

## Implementation Phases

### Phase 1: Foundation (Registry Infrastructure)

**Files:**
- `crates/auto-lang/src/vm/mod.rs` - Create registry structures

**Tasks:**
1. Create `VmRegistry`, `VmModule`, `VmFunctionEntry` structs
2. Implement `VmRegistry::new()`, `register_module()`, `get_module()`
3. Add global `VM_REGISTRY` with `lazy_static`
4. Add unit tests

**Success Criteria:** Registry compiles, basic tests pass

---

### Phase 2: IO Module Registration

**Files:**
- `crates/auto-lang/src/vm/io.rs` - Add method wrappers
- `crates/auto-lang/src/vm/mod.rs` - Add `init_io_module()`

**Tasks:**
1. Create wrapper functions for methods:
   ```rust
   pub fn close_method(uni: Shared<Universe>, instance: &mut Value, args: Vec<Value>) -> Value {
       close(uni, instance)
   }
   ```
2. Implement `init_io_module()` to register `open`, `File` type with methods
3. Call `init_io_module()` during initialization

**Code Example:**
```rust
pub fn init_io_module() {
    let mut io_module = VmModule {
        name: "auto.io".into(),
        functions: HashMap::new(),
        types: HashMap::new(),
    };

    // Register 'open' function
    io_module.functions.insert("open".into(), VmFunctionEntry {
        name: "open".into(),
        func: io::open,
        is_method: false,
        param_types: vec![ast::Type::Str(0)],
        return_type: ast::Type::User(...),
    });

    // Register File type with methods
    let mut file_type = VmTypeEntry {
        name: "File".into(),
        methods: HashMap::new(),
    };
    file_type.methods.insert("close".into(), io::close_method as VmMethod);
    file_type.methods.insert("read_text".into(), io::read_text_method as VmMethod);

    io_module.types.insert("File".into(), file_type);
    VM_REGISTRY.lock().unwrap().register_module(io_module);
}
```

**Success Criteria:** IO module registered, lookups work

---

### Phase 3: Use Statement Implementation

**Files:**
- `crates/auto-lang/src/eval.rs` - Implement `eval_use_auto()`

**Tasks:**
1. Implement `eval_use_auto()` to handle `use auto.io: open, File`
2. Parse module path from `use_stmt.paths`
3. Lookup module in VM registry
4. Register imported functions in current scope using `universe.define()`
5. Handle errors (module not found, item not found)

**Implementation:**
```rust
fn eval_use_auto(&mut self, use_stmt: &Use) -> Value {
    let module_path = use_stmt.paths.join(".");

    // Check if module exists
    let registry = VM_REGISTRY.lock().unwrap();
    let module = match registry.get_module(&module_path) {
        Some(m) => m,
        None => return Value::Error(format!("Module '{}' not found", module_path).into()),
    };
    drop(registry);

    // Register each imported item in current scope
    for item_name in &use_stmt.items {
        if let Some(func_entry) = VM_REGISTRY.lock().unwrap()
            .get_function(&module_path, item_name) {
            // Create VmFunction metadata and register in scope
            let fn_decl = ast::Fn::new(
                ast::FnKind::VmFunction,
                item_name.clone(),
                None,
                vec![],
                ast::Body::new(),
                func_entry.return_type.clone(),
            );
            self.universe.borrow_mut().define(item_name, Rc::new(Meta::Fn(fn_decl)));
        }
    }

    Value::Void
}
```

**Success Criteria:** `use auto.io: open` doesn't error, `open` is callable

---

### Phase 4: VM Function Call Dispatch

**Files:**
- `crates/auto-lang/src/eval.rs` - Implement `eval_vm_fn_call()`
- `crates/auto-lang/src/universe.rs` - Add VM function lookup cache

**Tasks:**
1. Implement `eval_vm_fn_call()` to dispatch to Rust functions
2. Add `vm_function_cache` HashMap to Universe for O(1) lookup
3. Lookup function in registry and call implementation
4. Handle argument conversion

**Implementation:**
```rust
pub fn eval_vm_fn_call(&mut self, fn_decl: &Fn, args: &Vec<Value>) -> Value {
    // Check cache first
    if let Some(cached) = self.universe.borrow().vm_function_cache.get(&fn_decl.name) {
        let uni = self.universe.clone();
        return (cached.func)(uni, args[0].clone());
    }

    // Lookup in registry
    let registry = VM_REGISTRY.lock().unwrap();
    let vm_func = self.find_vm_function(&fn_decl.name, &registry);
    drop(registry);

    match vm_func {
        Some(func_entry) => {
            let uni = self.universe.clone();
            let result = (func_entry.func)(uni, args[0].clone());

            // Cache for future lookups
            self.universe.borrow_mut().vm_function_cache.insert(
                fn_decl.name.clone(),
                func_entry.clone()
            );

            result
        },
        None => Value::Error(format!("VM function '{}' not found", fn_decl.name).into()),
    }
}
```

**Success Criteria:** `open("test.at")` calls Rust implementation successfully

---

### Phase 5: VM Method Call Support

**Files:**
- `crates/auto-lang/src/eval.rs` - Add method detection in `eval_call()`

**Tasks:**
1. Detect VM method calls in `eval_call()` (pattern: `instance.method()`)
2. Implement `eval_vm_method_call()` to dispatch methods
3. Pass instance as first argument to method
4. Handle method return values

**Implementation:**
```rust
fn eval_call(&mut self, call: &Call) -> Value {
    let callee = self.eval_expr(&call.name);

    // Check for VM method call pattern
    if let Value::Instance(instance) = &callee {
        // Extract method name from call.name (need to investigate AST structure)
        if let Some(method) = self.find_vm_method(&instance.ty, &method_name) {
            return self.eval_vm_method_call(instance, method, &call.args);
        }
    }

    // ... rest of existing eval_call logic
}

fn eval_vm_method_call(&mut self, instance: &mut Value, method: VmMethod, args: &Args) -> Value {
    let uni = self.universe.clone();
    let mut arg_vals = Vec::new();
    for arg in args.args.iter() {
        match arg {
            Arg::Pos(expr) => arg_vals.push(self.eval_expr(expr)),
            _ => {},
        }
    }
    method(uni, instance, arg_vals)
}
```

**Note:** Need to investigate how method calls are parsed (check if `file.close()` creates `Expr::Dot`)

**Success Criteria:** `file.close()` successfully calls Rust implementation

---

### Phase 6: Testing & Integration

**Files:**
- `hello.at` - Verify full workflow
- Create test suite in `tests/vm_functions/`

**Tasks:**
1. Run `hello.at` to verify complete workflow
2. Create comprehensive tests:
   - `test_basic_import.at` - Test `use auto.io: open`
   - `test_method_call.at` - Test `file.close()`
   - `test_error_handling.at` - Test file not found errors
3. Performance testing (ensure no regressions)
4. Update documentation

**Test Cases:**
```auto
# test_basic.at
use auto.io: open
let file = open("test.at")
assert(file != nil)

# test_methods.at
use auto.io: open, File
let file = open("test.at")
let content = file.read_text()
file.close()
assert(content != "")

# test_errors.at
use auto.io: open
let file = open("nonexistent.txt")
assert(file == Error)
```

**Success Criteria:**
- `hello.at` executes without errors
- All test cases pass
- No regressions in existing tests (221 tests)
- Performance: function dispatch < 1μs

## Critical Files

### Implementation Priority

1. **crates/auto-lang/src/vm/mod.rs** - Create registry (foundation)
2. **crates/auto-lang/src/eval.rs** - Implement dispatch logic (execution)
3. **crates/auto-lang/src/vm/io.rs** - Adapt existing functions (concrete example)
4. **crates/auto-lang/src/universe.rs** - Add lookup cache (optimization)
5. **stdlib/auto/io.at** - User-facing API (test case)

## Error Handling

### Error Types
- `ModuleNotFound` - Module doesn't exist in registry
- `FunctionNotFound` - Function not found in module
- `MethodNotFound` - Method not found on type
- `RuntimeError` - Errors from VM function (file not found, etc.)

### Error Messages
```rust
Value::Error(format!(
    "Module 'auto.nonexistent' not found. Available: auto.io, auto.math"
).into())
```

## Potential Pitfalls

### 1. Method Call Parsing
**Risk:** Parser may not recognize `file.close()` as method call
**Mitigation:** Investigate AST structure, add `Expr::Dot` if needed

### 2. Function Signature Mismatch
**Risk:** Current signatures incompatible with multi-argument dispatch
**Mitigation:** Create wrapper functions for each method

### 3. Borrowing Issues
**Risk:** `&mut Value` in methods causes borrow conflicts
**Mitigation:** Use interior mutability or clone values

## Verification

### End-to-End Test
```bash
# Run the main test case
cargo run --bin auto -- hello.at

# Should execute without errors and display file content
```

### Unit Tests
```bash
# Test registry
cargo test -p auto-lang vm_registry

# Test IO module
cargo test -p auto-lang vm_io_module

# Test dispatch
cargo test -p auto-lang vm_dispatch
```

### Integration Tests
```bash
# Run all VM function tests
cargo test -p auto-lang test_vm_*

# Verify no regressions
cargo test -p auto-lang --lib
```

## Success Criteria

- ✅ `hello.at` executes successfully
- ✅ File operations work (open, read, close)
- ✅ VM functions importable via `use`
- ✅ VM methods work on type instances
- ✅ Errors properly returned
- ✅ No regressions (221 tests pass)
- ✅ Easy to add new VM modules

## Future Enhancements

- Additional VM modules: `auto.math`, `auto.json`, `auto.http`
- Async VM functions
- Hot-reloading of VM modules during development
- Module namespace support (e.g., `auto.io.fs`)

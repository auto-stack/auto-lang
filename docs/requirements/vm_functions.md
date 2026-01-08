# VM Function Integration Requirements

## Overview

This document describes the requirements for integrating native Rust VM functions with the AutoLang interpreter, enabling the standard library to provide high-level APIs backed by Rust implementations.

## Current State

### Existing Components

1. **VM Functions Implementation** (`crates/auto-lang/src/vm/io.rs`)
   - `open(path: Value) -> Value` - Opens a file and returns a File instance
   - `read_text(uni, file: &mut Value) -> Value` - Reads file contents as string
   - `close(uni, file: &mut Value) -> Value` - Closes a file and cleans up resources

2. **Standard Library Structure** (`stdlib/auto/io.at`)
   - Dual sections: `# AUTO` (high-level API) and `# C` (low-level bindings)
   - VM function declarations using `fn.vm` prefix
   - Type definitions with VM methods

3. **Import Syntax** (`hello.at`)
   ```auto
   use auto.io: open, File
   ```

4. **Interpreter Stubs**
   - `eval_vm_fn_call()` returns "Not implemented yet"
   - `eval_use()` returns `Value::Void` (TODO comment)

### Problem Statement

The interpreter lacks the mechanism to:
1. Parse and recognize `fn.vm` declarations
2. Load and register VM functions via `use` statements
3. Dispatch VM function calls to Rust implementations
4. Handle VM methods on type instances (e.g., `file.close()`)

## Functional Requirements

### FR1: VM Function Registry

**Requirement**: Create a centralized registry for VM modules and their functions.

- **Registry Structure**: Map module names to their exported functions and types
- **Function Signature**: Store function pointers with signature `fn(Shared<Universe>, &Args) -> Value`
- **Module Registration**: Provide API for VM modules to register themselves
- **Type Information**: Store which types have VM methods

**Example**:
```rust
// Registry maps "auto.io" -> { open: fn(...), File: { close: fn(...), read_text: fn(...) } }
```

### FR2: Use Statement Evaluation

**Requirement**: Implement `use auto.io: open, File` to load VM modules.

- **Parse Module Name**: Extract "auto.io" from use statement
- **Lookup Module**: Find module in VM registry
- **Register Functions**: Add imported functions to current scope
- **Register Types**: Add imported types with their VM methods to scope

**Behavior**:
```auto
use auto.io: open, File

# After this, 'open' is callable and 'File' type is available with VM methods
let file = open("test.at")  # Calls vm::io::open
file.close()                 # Calls vm::io::close
```

### FR3: VM Function Call Dispatch

**Requirement**: Route `fn.vm` calls to Rust implementations.

- **Detect VM Functions**: Identify `FnKind::VmFunction` in function metadata
- **Lookup Implementation**: Find function pointer in VM registry
- **Convert Arguments**: Transform AutoLang arguments to Rust `Args` structure
- **Call Function**: Invoke Rust function with universe and arguments
- **Return Result**: Convert Rust `Value` return to AutoLang result

**Implementation Point**: Complete `eval_vm_fn_call()` in `eval.rs`

### FR4: VM Method Call Dispatch

**Requirement**: Handle VM methods on type instances (e.g., `file.close()`).

- **Detect VM Methods**: Identify methods marked with `fn.vm` in type definitions
- **Method Lookup**: Find method implementation in type's VM method registry
- **Instance Handling**: Pass instance as first argument to method
- **Dispatch**: Call Rust method with instance, universe, and arguments

**Syntax**:
```auto
type File {
    fn.vm close()           # Instance method, no return
    fn.vm read_text() str   # Instance method, returns string
}

# Usage
file.close()               # Method call on instance
```

### FR5: VM Reference Management

**Requirement**: Properly manage native resources (file handles, etc.).

- **Existing Infrastructure**: Universe already has `add_vmref`, `get_vmref`, `drop_vmref`
- **Lifecycle**: Resources tracked by universe, cleaned up on close or drop
- **Error Handling**: Return `Value::Error` for failures (file not found, etc.)

## Technical Requirements

### TR1: Parse fn.vm Declarations

**Requirement**: Lexer and parser must recognize `fn.vm` as distinct from regular functions.

- **Token Type**: Add `FnVm` token or distinguish in lexer
- **AST Node**: `FnDecl` should have `kind: FnKind` with `VmFunction` variant (already exists)
- **Metadata Storage**: Store which functions are VM functions in function metadata

### TR2: Module Loading System

**Requirement**: Load VM modules from standard library paths.

- **Search Path**: Check `stdlib/auto/` for `.at` files
- **Parse AUTO Section**: Only parse `# AUTO` section (ignore `# C` section)
- **Caching**: Avoid re-loading modules multiple times

### TR3: Type-Method Association

**Requirement**: Associate VM methods with their types.

- **Type Registry**: Universe already tracks types
- **Method Registry**: Add VM methods to type metadata
- **Instance Lookup**: Instance methods lookup via type name

### TR4: Error Handling

**Requirement**: Gracefully handle errors in VM function calls.

- **File Not Found**: Return `Value::Error` with message
- **Type Mismatch**: Validate argument types before calling
- **Missing Module**: Error if `use auto.nonexistent` attempted

## Non-Functional Requirements

### NFR1: Performance

- **Minimal Overhead**: VM function dispatch should be fast (hash map lookup)
- **No Unnecessary Cloning**: Use references where possible

### NFR2: Maintainability

- **Clear Separation**: VM code separate from interpreter core
- **Extensibility**: Easy to add new VM modules (math, json, etc.)

### NFR3: Compatibility

- **Backward Compatible**: Don't break existing AutoLang code
- **C Section**: Don't interfere with `# C` section handling

## User Interface

### Import Syntax
```auto
use auto.io: open, File, read_text
```

### Function Call Syntax
```auto
let file = open("path.txt")
```

### Method Call Syntax
```auto
file.close()
file.read_text()
```

### Type Definition Syntax
```auto
type File {
    fn.vm close()
    fn.vm read_text() str
}
```

## Test Cases

### TC1: Basic VM Function Call
```auto
use auto.io: open
let file = open("test.at")
# Should call vm::io::open
```

### TC2: VM Method Call
```auto
use auto.io: open, File
let file = open("test.at")
file.close()
# Should call vm::io::close
```

### TC3: VM Method with Return Value
```auto
use auto.io: open, File
let file = open("test.at")
let content = file.read_text()
# Should return file contents as string
```

### TC4: Complete Workflow (hello.at)
```auto
use auto.io: open, File

let file = open("pac.at")
file.read_text()

file.close()
# Should execute without errors
```

### TC5: Error Handling
```auto
use auto.io: open
let file = open("nonexistent.txt")
# Should return Value::Error
```

## Implementation Phases

### Phase 1: Foundation
1. Create VM module registry in `vm/mod.rs`
2. Define `VmModule`, `VmFunction` structures
3. Implement module registration API
4. Register `vm::io` module

### Phase 2: Use Statement
1. Implement `eval_use()` for `UseKind::Auto`
2. Parse module name from use statement
3. Lookup module in registry
4. Register functions and types in scope

### Phase 3: Function Dispatch
1. Complete `eval_vm_fn_call()` implementation
2. Lookup VM function in registry
3. Convert arguments to `Args`
4. Call Rust function and return result

### Phase 4: Method Support
1. Add VM method detection in dot expression handler
2. Lookup methods in type metadata
3. Call VM methods with instance reference

### Phase 5: Testing
1. Test with `hello.at`
2. Add comprehensive test cases
3. Error handling tests

## Success Criteria

- ✅ `hello.at` executes successfully
- ✅ File operations work (open, read, close)
- ✅ VM functions can be imported via `use`
- ✅ VM methods work on type instances
- ✅ Errors are properly returned
- ✅ No regressions in existing tests
- ✅ Easy to add new VM modules (e.g., `auto.math`)

## Open Questions

1. **Module Namespace**: Should modules be nested (e.g., `auto.io.fs`)?
2. **Method Storage**: Should VM methods be stored in type metadata or separate registry?
3. **Lazy vs Eager Loading**: Load modules on use or at startup?
4. **C Section Interaction**: How should `# AUTO` and `# C` sections interact?

## Future Enhancements

- Additional VM modules: `auto.math`, `auto.json`, `auto.http`
- Async VM functions
- VM function documentation in stdlib
- Hot-reloading of VM modules during development

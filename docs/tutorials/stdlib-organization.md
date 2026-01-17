# AutoLang Standard Library Organization

This tutorial explains how AutoLang's standard library is organized across multiple files with different purposes.

## File Organization Structure

AutoLang standard library uses a **file-based separation** strategy to organize code for different compilation targets:

| File Extension | Purpose | Loaded By | Architecture Layer |
|----------------|---------|-----------|-------------------|
| `.at` | Pure Auto code | **All targets** | Top (business logic) |
| `.vm.at` | VM-specific code | **Interpreter only** | Bottom (system interface) |
| `.c.at` | C-specific code | **C transpiler only** | Bottom (system interface) |

## Loading Order (Layered Architecture)

**Key Principle**: Load bottom layer (target-specific) first, then top layer (common).

```
Interpreter mode: use io
  1. Load: io.vm.at (bottom - provides fn.vm interfaces)
  2. Load: io.at     (top - calls fn.vm for advanced features)

Transpiler mode: use io
  1. Load: io.c.at   (bottom - provides fn.c interfaces)
  2. Load: io.at     (top - calls fn.c for advanced features)
```

**Why this order matters**:
- `io.at` needs to call `fn.vm` functions defined in `io.vm.at`
- Bottom files provide interfaces, top files implement business logic
- Follows "define bottom first, call from top" dependency pattern

## Directory Structure

```
stdlib/auto/
├── io.at       ← Pure Auto code (all targets)
├── io.vm.at    ← VM-specific code (interpreter)
├── io.c.at     ← C-specific code (transpiler)
├── math.at     ← Pure Auto code (all targets)
├── str.at      ← Pure Auto code with ext statements
├── sys.at      ← Pure Auto code (all targets)
└── sys.c.at    ← C-specific code (transpiler)
```

## File Content Examples

### `.at` Files (Pure Auto - All Targets)

**Purpose**: Business logic, algorithms, type declarations
**Loaded by**: Interpreter and Transpiler

```auto
# stdlib/auto/io.at

// Type declaration (declared only once!)
type File {
    path str
    file *FILE

    // Common method using VM functions
    fn read_all() str {
        let mut result = ""
        loop {
            if .is_eof() {
                break
            }
            let line = .read_line()
            if line == "" {
                break
            }
            if result != "" {
                result = result.append(c"\n")
            }
            result = result.append(line)
        }
        result
    }

    // VM method declarations (implemented in Rust)
    fn.vm read_line() str
    fn.vm is_eof() bool
}

// Common function (Auto implementation)
fn say(msg str) {
    print(msg)
    print("\n")
}
```

### `.vm.at` Files (VM-Specific - Interpreter Only)

**Purpose**: `fn.vm` function declarations (implemented in Rust)
**Loaded by**: Interpreter only

```auto
# stdlib/auto/io.vm.at

// VM function declarations (implemented in Rust)
fn.vm open(path str) File
fn.vm open_read(path str) File
fn.vm open_write(path str) File
fn.vm open_append(path str) File
```

### `.c.at` Files (C-Specific - Transpiler Only)

**Purpose**: `fn.c` external function declarations, C headers
**Loaded by**: C transpiler only

```auto
# stdlib/auto/io.c.at

// Import C headers
use.c <stdio.h>
use.c <stdlib.h>
use.c <string.h>

// External C function declarations
fn.c fopen(path cstr, mode cstr) *FILE
fn.c fclose(stream *FILE) int
fn.c fgets(buf cstr, size int, stream *FILE) cstr
fn.c fputs(s cstr, stream *FILE) int
fn.c printf(fmt cstr, ...)
```

## Design Principles

### 1. Prefer Auto Implementation
Most functionality should be implemented in AutoLang for:
- Better maintainability
- Easier debugging
- Cross-platform compatibility
- Self-hosting capability

**Example**: String manipulation, algorithms, business logic

### 2. Minimize VM/C Dependencies
Only use `fn.vm` or `fn.c` when necessary:
- System calls (file I/O, network)
- Performance-critical operations
- Hardware-specific features
- Legacy C library integration

**Example**: `fopen`, `getpid`, `sqrt`

### 3. Avoid Duplication
- Declare types **only once** in `.at` files
- Don't repeat type definitions in `.vm.at` or `.c.at`
- Share common logic in `.at` files

### 4. Clear Separation
- **Common logic** → `.at` files
- **Platform-specific** → `.vm.at` or `.c.at` files
- **Bottom layer** provides interfaces
- **Top layer** implements features

## Section Markers (Legacy)

**Note**: Section markers like `# AUTO`, `# VM`, `# C` are **legacy syntax**. The new approach uses **separate files** instead.

**Old approach** (still supported):
```auto
# AUTO
fn add(a int, b int) int { a + b }

# VM
fn.vm subtract(a int, b int) int

# C
fn.c subtract(a int, b int) int
```

**New approach** (recommended):
```
io.at      → # AUTO section
io.vm.at   → # VM section
io.c.at    → # C section
```

Both approaches work, but file separation is cleaner for large standard libraries.

## Method Call Syntax

With Plan 038 (VM Method Call Expressions), all methods use consistent dot syntax:

```auto
// VM methods (implemented in Rust)
let words = "hello world".split(" ")
let lines = "line1\nline2".lines()
let file = File.open_read("test.txt")

// Auto methods (implemented in AutoLang)
let content = file.read_all()
let len = "hello".len()

// Method chaining
let first = "hello world".split(" ")[0]
```

## Best Practices

### When to Use Each File Type

**Use `.at` files for**:
- ✅ Type declarations
- ✅ Business logic
- ✅ Algorithms
- ✅ Data structures
- ✅ Helper functions

**Use `.vm.at` files for**:
- ✅ `fn.vm` function declarations
- ✅ System-level operations
- ✅ Performance-critical operations
- ✅ Interpreter-specific features

**Use `.c.at` files for**:
- ✅ `fn.c` external function declarations
- ✅ C header imports (`use.c <stdio.h>`)
- ✅ C-specific type definitions
- ✅ Platform-specific C code

### Naming Conventions

**VM function naming**: `{type}_{method}`
- `str_split` for `str.split()`
- `file_read_all` for `File.read_all()`
- `file_write_lines` for `File.write_lines()`

**File naming**:
- `module.at` - Common code
- `module.vm.at` - VM-specific code
- `module.c.at` - C-specific code

## Migration Guide

### From Single File to Split Files

**Before** (single file with sections):
```auto
# io.at

# AUTO
type File {
    path str
    fn read_all() str { ... }
}

# VM
fn.vm open(path str) File

# C
use.c <stdio.h>
fn.c fopen(path cstr, mode cstr) *FILE
```

**After** (split files):
```auto
# io.at (common)
type File {
    path str
    fn read_all() str { ... }
}

# io.vm.at (VM only)
fn.vm open(path str) File

# io.c.at (C only)
use.c <stdio.h>
fn.c fopen(path cstr, mode cstr) *FILE
```

Both approaches work! File separation is recommended for new code.

## Related Documentation

- [Plan 036: Unified Auto Section](../plans/036-unified-auto-section.md) - Implementation details
- [Plan 035: ext Statement](../plans/035-ext-statement.md) - Method definitions
- [Plan 038: VM Method Call Expressions](../plans/038-vm-method-call-expressions.md) - Method call syntax
- [Method Calls Tutorial](./method-calls.md) - How to call methods

## Examples

### Example 1: String Methods

**File**: `stdlib/auto/str.at` (pure Auto with ext)

```auto
ext str {
    // VM method (implemented in Rust)
    fn.vm split(delimiter str) []str

    // Auto method (implemented in AutoLang)
    fn char_count() int {
        .size
    }

    // Auto method using VM method
    fn word_count() int {
        .words().len()
    }
}
```

### Example 2: File I/O

**File**: `stdlib/auto/io.at` (common)

```auto
type File {
    path str
    file *FILE

    // Auto-implemented method using VM methods
    fn read_all() str {
        let mut result = ""
        loop {
            if .is_eof() {
                break
            }
            let line = .read_line()
            if line == "" {
                break
            }
            if result != "" {
                result = result.append(c"\n")
            }
            result = result.append(line)
        }
        result
    }

    // VM methods
    fn.vm read_line() str
    fn.vm is_eof() bool
}
```

**File**: `stdlib/auto/io.vm.at` (VM only)

```auto
fn.vm open(path str) File
fn.vm open_read(path str) File
fn.vm open_write(path str) File
```

### Example 3: Math Functions

**File**: `stdlib/auto/math.at` (pure Auto - no VM needed)

```auto
fn abs(x int) int {
    if x < 0 {
        -x
    } else {
        x
    }
}

fn min(a int, b int) int {
    if a < b {
        a
    } else {
        b
    }
}

fn max(a int, b int) int {
    if a > b {
        a
    } else {
        b
    }
}
```

Pure Auto implementation - no VM/C code needed!

## Summary

- **`.at` files**: Common code for all targets (business logic)
- **`.vm.at` files**: Interpreter-specific (VM function declarations)
- **`.c.at` files**: Transpiler-specific (C external functions)
- **Loading order**: Bottom first (interfaces), then top (logic)
- **Goal**: AutoLang as the primary implementation language

This organization enables AutoLang to be **self-hosting** while maintaining clean separation of concerns.

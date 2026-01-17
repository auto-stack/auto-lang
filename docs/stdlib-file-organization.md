# AutoLang Standard Library File Organization Convention

## Overview

As of Plan 036 implementation, AutoLang standard library uses **file-based separation** instead of section markers within files. This provides cleaner organization and better support for different compilation targets.

**Status**: ✅ **ACTIVE** (as of 2025-01-17)
**Migration**: Section markers (`# AUTO`, `# VM`, `# C`) have been removed from all split files

---

## File Naming Convention

### File Types

| Extension | Purpose | Loaded By | Example |
|-----------|---------|-----------|---------|
| `.at` | Common Auto code | **All targets** | `io.at` |
| `.vm.at` | VM-specific code | **Interpreter only** | `io.vm.at` |
| `.c.at` | C-specific code | **C transpiler only** | `io.c.at` |

### Naming Pattern

For a module named `module`:
- **`module.at`** - Common Auto code (business logic, types, methods)
- **`module.vm.at`** - VM-specific code (VM function declarations)
- **`module.c.at`** - C-specific code (C external functions, headers)

---

## Loading Order

**Critical**: Files are loaded in a specific order to maintain dependency relationships.

### Interpreter Mode
```auto
use io

// Loading order:
1. io.vm.at   (bottom - VM function declarations)
2. io.at      (top - business logic using VM functions)
```

### C Transpiler Mode
```auto
use io

// Loading order:
1. io.c.at    (bottom - C external function declarations)
2. io.at      (top - business logic using C functions)
```

**Why this order?**
- Bottom files provide interfaces (function declarations)
- Top files implement features using those interfaces
- Follows "define bottom first, call from top" dependency pattern

---

## File Content Guidelines

### `.at` Files (Common Code)

**Purpose**: Business logic, algorithms, type declarations
**Loaded by**: All compilation targets

**What goes here**:
- ✅ Type declarations
- ✅ Method implementations (Auto code)
- ✅ Helper functions
- ✅ Algorithms and business logic
- ✅ `ext` statements (adding methods to types)

**What does NOT go here**:
- ❌ `fn.vm` function declarations (use `.vm.at`)
- ❌ `fn.c` external function declarations (use `.c.at`)
- ❌ `use.c` C header imports (use `.c.at`)
- ❌ Section markers (e.g., `# AUTO`, `# VM`, `# C`)

**Example**:
```auto
/// Common I/O functions and types

fn say(msg str) {
    printf(c"%s\n", msg)
}

type File {
    path str
    file *FILE

    // Auto-implemented method
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

    // VM method declarations only
    fn.vm read_line() str
    fn.vm is_eof() bool
}
```

### `.vm.at` Files (VM-Specific)

**Purpose**: VM function declarations (implemented in Rust)
**Loaded by**: Interpreter only

**What goes here**:
- ✅ `fn.vm` function declarations
- ✅ VM-specific type declarations (if any)

**What does NOT go here**:
- ❌ Business logic implementations (use `.at`)
- ❌ C function declarations (use `.c.at`)
- ❌ Section markers

**Example**:
```auto
// VM function declarations for I/O operations
// These are implemented in Rust in crates/auto-lang/src/libs/file.rs

fn.vm open(path str) File
fn.vm open_read(path str) File
fn.vm open_write(path str) File
fn.vm open_append(path str) File
```

### `.c.at` Files (C-Specific)

**Purpose**: C external function declarations and C headers
**Loaded by**: C transpiler only

**What goes here**:
- ✅ `fn.c` external function declarations
- ✅ `use.c` C header imports
- ✅ C-specific type declarations (if any)

**What does NOT go here**:
- ❌ Business logic implementations (use `.at`)
- ❌ `fn.vm` function declarations (use `.vm.at`)
- ❌ Section markers

**Example**:
```auto
// C external function declarations for I/O operations

use.c <stdlib.h>
fn.c exit(status int) void

use.c <stdio.h>
fn.c printf(fmt cstr, arg cstr)
type.c FILE
fn.c fopen(path cstr, mode cstr) *FILE
fn.c fclose(stream *FILE) int
```

---

## Legacy Section Markers (Deprecated)

### ❌ DO NOT Use Section Markers

**Old approach** (deprecated):
```auto
# io.at

# AUTO
fn add(a int, b int) int { a + b }

# VM
fn.vm subtract(a int, b int) int

# C
fn.c subtract(a int, b int) int
```

**New approach** (recommended):
```auto
// io.at (common code only)
fn add(a int, b int) int { a + b }

// io.vm.at (VM only)
fn.vm subtract(a int, b int) int

// io.c.at (C only)
fn.c subtract(a int, b int) int
```

### Migration Status

✅ **COMPLETED** (2025-01-17):
- All section markers removed from split files
- No `# AUTO`, `# VM`, or `# C` markers in `.at`, `.vm.at`, or `.c.at` files
- Documentation updated to reflect new convention

---

## Current Standard Library Structure

```
stdlib/auto/
├── io.at           # Common I/O types and methods
├── io.vm.at        # VM function declarations for I/O
├── io.c.at         # C external function declarations for I/O
├── math.at         # Pure Auto math functions (no split needed)
├── str.at          # String extension methods (ext statements)
├── sys.at          # Common system functions
└── sys.c.at        # C external function declarations for system
```

### Notes

- **`math.at`** and **`str.at`** don't have `.vm.at` or `.c.at` files because they're pure Auto code
- Pure Auto modules don't need platform-specific implementations
- They use `fn.vm` for methods that need VM implementation, but don't need separate files

---

## Best Practices

### 1. File Separation

**When to split files**:
- ✅ Module has both Auto and VM implementations → Create `.vm.at`
- ✅ Module has both Auto and C declarations → Create `.c.at`
- ✅ Module is pure Auto → Single `.at` file only

**When NOT to split**:
- ❌ Pure Auto modules (like `math.at`, `str.at`)
- ❌ Simple modules with no platform-specific code

### 2. Content Organization

**Keep in `.at` files**:
- Type declarations
- Auto method implementations
- Business logic
- Algorithms

**Move to `.vm.at`**:
- `fn.vm` function declarations
- VM-specific helper types

**Move to `.c.at`**:
- `fn.c` external function declarations
- `use.c` header imports
- C-specific types

### 3. Documentation

- ✅ Document file purpose at the top of each file
- ✅ Use comments to explain non-obvious code
- ✅ Include examples for public APIs
- ❌ Don't use section markers (they're redundant)

---

## Examples

### Example 1: Simple Module (No Split)

**File**: `math.at` (single file)
```auto
/// Pure Auto math functions
/// No platform-specific code needed

fn abs(x int) int {
    if x < 0 {
        -x
    } else {
        x
    }
}

fn min(a int, b int) int {
    if a < b { a } else { b }
}

fn max(a int, b int) int {
    if a > b { a } else { b }
}
```

### Example 2: Module with VM Functions

**Files**: `str.at` (single file with `fn.vm`)
```auto
/// String extension methods

ext str {
    // VM method (implemented in Rust)
    fn.vm split(delimiter str) []str

    // Auto method using VM method
    fn word_count() int {
        .words().len()
    }

    // Auto method with simple logic
    fn is_empty() bool {
        .size == 0
    }
}
```

**Note**: Even though `str.at` has `fn.vm` declarations, it doesn't need a separate `.vm.at` file because all methods are logically grouped in the `ext` block.

### Example 3: Module with Split Files

**File**: `io.at` (common code)
```auto
/// Common I/O functions and File type

fn say(msg str) {
    printf(c"%s\n", msg)
}

type File {
    path str
    file *FILE

    // Auto-implemented method
    fn read_all() str {
        // ... implementation using VM methods
    }

    // VM method declarations
    fn.vm read_line() str
    fn.vm is_eof() bool
}
```

**File**: `io.vm.at` (VM declarations)
```auto
/// VM function declarations for I/O

fn.vm open(path str) File
fn.vm open_read(path str) File
fn.vm open_write(path str) File
fn.vm open_append(path str) File
```

**File**: `io.c.at` (C declarations)
```auto
/// C external function declarations for I/O

use.c <stdlib.h>
fn.c exit(status int) void

use.c <stdio.h>
fn.c printf(fmt cstr, arg cstr)
type.c FILE
fn.c fopen(path cstr, mode cstr) *FILE
```

---

## Migration Guide for New Modules

### Step 1: Determine if Split is Needed

Ask yourself:
- Does this module need VM function declarations?
- Does this module need C external function declarations?

**If both answers are "no"**: Create single `module.at` file
**If either answer is "yes"**: Consider split files

### Step 2: Create Files

If splitting:
1. Create `module.at` for common Auto code
2. Create `module.vm.at` if needed (VM declarations)
3. Create `module.c.at` if needed (C declarations)

### Step 3: Organize Content

**In `module.at`**:
- Add type declarations
- Add Auto method implementations
- Add helper functions

**In `module.vm.at`** (if exists):
- Add `fn.vm` function declarations
- Group related functions together

**In `module.c.at`** (if exists):
- Add `use.c` header imports
- Add `fn.c` external function declarations
- Group related functions together

### Step 4: Document

- Add file-level documentation
- Comment on non-obvious code
- Include usage examples

### Step 5: Test

- Run interpreter: `auto run test.at`
- Run C transpiler: `auto c test.at`
- Verify both modes work correctly

---

## Summary

**Key Points**:
1. ✅ Section markers (`# AUTO`, `# VM`, `# C`) are deprecated
2. ✅ Use file extensions instead: `.at`, `.vm.at`, `.c.at`
3. ✅ Load order: Bottom first (`.vm.at`/`.c.at`), then top (`.at`)
4. ✅ Pure Auto modules don't need split files
5. ✅ Document purpose and organization

**Benefits**:
- Cleaner file organization
- Better separation of concerns
- Easier to maintain
- Clearer compilation target support

**Related Documentation**:
- [Plan 036: Unified Auto Section](../plans/036-unified-auto-section.md) - Implementation details
- [stdlib-organization.md](../tutorials/stdlib-organization.md) - User-facing tutorial

# Standard Library Implementation Plan

**Status**: 🔄 **ACTIVE** - Migrating to AutoLang-first architecture
**Last Updated**: 2025-01-20
**Architecture**: Multi-platform stdlib via ext mechanism

## Executive Summary

Build foundational standard library components for the self-hosting Auto compiler using the **new AutoLang-first architecture**:

1. **Write in AutoLang** - All stdlib components are implemented in AutoLang (not hand-written C)
2. **Transpile to C** - Use a2c transpiler to automatically generate C code from AutoLang source
3. **Multi-platform** - Use ext mechanism for VM and C platform-specific implementations
4. **Clean APIs** - OOP-style methods inside types, no module prefixes in AutoLang source

**Timeline**: 6-8 months (for all components)
**Complexity**: High (requires tag types, generics, ownership system)
**Priority**: BLOCKER - Must complete before self-hosting compiler can begin

## New Architecture (✅ Current Approach)

### Implementation Flow

```
AutoLang Source (.at)
    ↓
a2c Transpiler
    ↓
C Code (.c + .h)  ← Auto-generated, NOT hand-written
    ↓
C Compiler
    ↓
Executable / Library
```

### File Organization

```
stdlib/
├── auto/                # Current: Multi-platform via ext
│   ├── io.at            # Public interface
│   ├── io.vm.at         # VM implementation
│   ├── io.c.at          # C implementation (future)
│   ├── math.at
│   ├── str.at
│   └── sys.at
├── may/                 # Future: May<T> type (using tag syntax)
│   └── may.at           # AutoLang source, transpiled to C
├── collections/         # Future: HashMap/HashSet
│   ├── hashmap.at       # AutoLang source
│   └── hashset.at       # AutoLang source
├── string/              # Future: String utilities
│   ├── builder.at       # StringBuilder
│   └── intern.at        # String interning
└── sys/                 # Future: System utilities
    └── args.at          # Command-line arguments
```

### Key Design Principles

**CRITICAL**: All stdlib components follow these principles:

1. ✅ **AutoLang-first**: Write in AutoLang, NOT hand-written C
2. ✅ **OOP style**: Methods inside types (like Java), NOT module-prefixed functions
3. ✅ **Clean APIs**: No prefixes in AutoLang source; a2c adds prefixes only in generated C code
4. ✅ **Multi-platform**: Use ext with .vm.at/.c.at for platform-specific code
5. ✅ **Auto-generated C**: a2c transpiler generates .c/.h files from .at source
6. ✅ **Proper FFI**: Use `fn.c` ONLY for existing C libraries (stdio.h, stdlib.h, etc.)

## Current Implementation Status

### ✅ Completed Components

#### 1. Basic I/O (Phase 1)
**Status**: ✅ Working for VM platform
**Files**: [io.at](stdlib/auto/io.at), [io.vm.at](stdlib/auto/io.vm.at)
**Tests**: `test_std_file`, `test_std_io_say` - All passing

**Implemented**:
```auto
type File {
    #[pub]
    path str

    #[pub]
    static fn open(path str) File

    #[pub]
    fn read_text() str

    #[pub]
    fn read_line() str

    #[pub]
    fn close()
}
```

**VM Implementation** ([io.vm.at:24-36](stdlib/auto/io.vm.at#L24-L36)):
```auto
ext File {
    #[vm]
    static fn open(path str) File

    #[vm]
    fn read_text() str

    #[vm]
    fn read_line() str

    #[vm]
    fn close()
}
```

#### 2. System Functions (Phase 1)
**Status**: ✅ Working for VM platform
**Files**: [sys.at](stdlib/auto/sys.at), [sys.vm.at](stdlib/auto/sys.vm.at)
**Tests**: `test_std_sys_get_pid` - Passing

**Implemented**:
```auto
#[pub]
fn getpid() int
```

#### 3. Test Module (Phase 1)
**Status**: ✅ Working for VM platform
**Files**: [test.at](stdlib/auto/test.at), [test.vm.at](stdlib/auto/test.vm.at)
**Tests**: `test_std_test` - Passing

**Implemented**:
```auto
#[pub]
fn test() int {
    42
}
```

### 🔄 In Progress Components

#### May<T> Type (Phase 1b)
**Status**: ✅ **COMPLETE** (2025-01-17)
**Design**: Unified three-state type using `tag` syntax
**Tests**: 34 passing tests (exceeded 30+ goal)

**Tag-Based Implementation**:
```auto
tag May<T> {
    nil Nil
    err Err
    val T

    // Static methods
    static fn empty() May<T> { May.nil() }
    static fn value(v T) May<T> { May.val(v) }
    static fn error(e Err) May<T> { May.err(e) }

    // Instance methods
    fn is_some() bool {
        is self {
            val(_) => true,
            _ => false
        }
    }

    fn unwrap() T {
        is self {
            val(v) => v,
            nil => panic("unwrap on nil"),
            err(e) => panic(f"unwrap on error: $e")
        }
    }
}
```

**Key Achievements**:
- ✅ Tag types working correctly in C transpiler
- ✅ Pattern matching with `is` statements
- ✅ Return type inference for pattern matching branches
- ✅ Tag constructor optional arguments support

**Remaining Work** (deferred):
- Pattern-bound variable extraction (complex, needs 2-3 days)
- Generics support for true May<T> (needs 1-2 weeks)
- `?T` syntactic sugar
- `.?` and `??` operators

### ⏸️ Planned Components

#### Phase 2: StringBuilder
**Dependencies**: May<T> (for error handling)
**Timeline**: 6 weeks

**Design** (AutoLang-first):
```auto
type StringBuilder {
    #[pub]
    data *char

    #[pub]
    len int

    #[pub]
    capacity int

    #[pub]
    static fn new() StringBuilder

    #[pub]
    fn append(s str) StringBuilder

    #[pub]
    fn build() str
}
```

#### Phase 3: HashMap/HashSet
**Dependencies**: None (standalone)
**Timeline**: 10-12 weeks

**Design** (AutoLang-first):
```auto
type HashMap<K, V> {
    // Implementation in AutoLang
    // Transpiled to C via a2c
}

type HashSet<T> {
    // Implementation in AutoLang
    // Transpiled to C via a2c
}
```

#### Phase 4: String Interning
**Dependencies**: HashSet
**Timeline**: 6 weeks

#### Phase 5: Args Parser
**Dependencies**: None
**Timeline**: 2 weeks

## Old Architecture (❌ Deprecated)

The following sections describe the OLD approach that is now **deprecated**:

### What Was Wrong

1. **Wrong implementation approach**: May<T> was implemented with hand-written C code instead of AutoLang
   - Current: Manually written `may.c` and `may.h`
   - Correct: May<T> should be written in AutoLang, then transpiled to C via a2c

2. **Non-idiomatic API**: Used C-style functions instead of OOP methods
   - Current: `May_empty()`, `May_is_empty(may)`
   - Correct: `May.empty()`, `may.is_empty()` (Java-style OOP)

3. **Unnecessary module prefixes**: Added `May_` prefix in AutoLang code
   - Current: `May_empty`, `May_is_empty` in AutoLang source
   - Correct: `empty`, `is_empty` in AutoLang; a2c adds `May_` prefix only in generated C code

4. **Incorrect use of `spec extern`**: Should use plain `fn` declarations
   - `spec` is for interface definitions (like Rust traits)
   - `fn.c` is for declaring external C functions
   - Plain `fn` (with or without body) is for normal functions

5. **Wrong type system**: Used plain `type` instead of `tag` (tagged union)
   - Current: Used `type May<T>` with manual tag field
   - Correct: Should use `tag May<T>` for discriminated unions

### Migration Status

The following hand-written C implementations exist but need migration:

1. **May<T>** - Partially migrated to tag-based implementation (Phase 1b complete)
2. **StringBuilder** - Not started (old hand-written C design exists)
3. **HashMap/HashSet** - Not started (old hand-written C design exists)
4. **String Interning** - Not started
5. **Args Parser** - Not started

## Summary

**Current Status**: Basic I/O, system functions, and May<T> type working for VM platform

**Architecture**: Clean separation between interface and implementation
- `.at` files define public interface
- `.vm.at` files provide VM implementations
- Future `.c.at` files will provide transpiled C implementations

**Next Steps**:
1. Complete remaining I/O methods (write, character I/O, positioning, error handling)
2. Migrate hand-written C implementations to AutoLang-first approach
3. Implement StringBuilder using AutoLang
4. Implement HashMap/HashSet using AutoLang
5. Add string interning support
6. Add command-line argument parsing

**Timeline**: 6-8 months for all components

## Related Documentation

- [Plan 020: Standard Library I/O Implementation](020-stdlib-io-expansion.md) - I/O specific implementation
- [Design: stdlib-organization.md](../design/stdlib-organization.md) - Multi-platform stdlib design
- [Tutorial: stdlib-organization.md](../tutorials/stdlib-organization.md) - How to use ext mechanism

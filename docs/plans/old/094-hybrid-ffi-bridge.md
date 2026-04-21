# Plan 094: Hybrid FFI Bridge

> **Status**: ✅ Phase 1-5 Complete (All 43 Shims Implemented, #[rust_fn] macro working)
> **Priority**: High (blocks Plan 095: CTEE)
> **Dependencies**: Plan 092 (Dynamic FFI via use.rust)
> **Consumers**: Plan 095 (Compile-Time Execution Engine)

## Implementation Status

| Phase | Status | Description |
|-------|--------|-------------|
| Phase 1 | ✅ Complete | `VMConvertible` trait + primitive types |
| Phase 2 | ✅ Complete | `#[rust_fn]` macro (using macro shims for all basic native functions) |
| Phase 3 | ✅ Complete | Unified `NativeInterface` lookup |
| Phase 4 | ✅ Complete | All 43 built-in stdlib shims |
| Phase 5 | ✅ Complete | Tests and documentation |
| Phase 6 | ⏳ Future | JIT inline cache support |

### Files Created

```
crates/auto-lang/src/vm/ffi/
├── mod.rs          # Public API, constants
├── convert.rs      # VMConvertible trait + impls
├── error.rs        # FFIError type
└── stdlib.rs       # Built-in FFI functions
```

### ID Space Allocation

| Range | Category | Status |
|-------|----------|--------|
| 0-999 | VM Intrinsics | ✅ Existing (List, HashMap, etc.) |
| 1000-1299 | Built-in stdlib | ✅ Implemented (File, Env, Time) |
| 1300-9999 | Reserved | - |
| 10000+ | Dynamic FFI | ✅ Infrastructure ready |

## Overview

Implement a **hybrid FFI architecture** that combines:
1. **Static FFI** - `#[rust_fn]` macro for built-in stdlib functions
2. **Dynamic FFI** - `use.rust` + sandbox for user crates (Plan 092)

This creates a unified, JIT-friendly FFI system with automatic type conversion.

## Current State

### What Exists (Plan 092)

```
┌─────────────────────────────────────────────────────────────┐
│                    Dynamic FFI (Plan 092)                    │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│   dep serde_json "1.0.100"                                  │
│   use.rust serde_json::{from_str, to_string}               │
│                                                             │
│   ──────────────────────────────────────────────────────    │
│                                                             │
│   1. Sandbox compiles crate to .dll/.so                     │
│   2. libloading loads library at runtime                    │
│   3. Symbol lookup: "auto_export_{func_name}"              │
│   4. Manual type conversion in shim                         │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### What's Missing

1. **Static FFI macro** - No `#[rust_fn]` for built-in functions
2. **Automatic type conversion** - Manual shim boilerplate
3. **Unified ID space** - No clear static vs dynamic ID ranges
4. **JIT optimization path** - No inline caching strategy

## Proposed Architecture

### Unified FFI Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                      AutoVM Hybrid FFI Architecture                  │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│   ┌─────────────────────┐      ┌─────────────────────────────────┐ │
│   │  Static Bindings    │      │  Dynamic FFI (use.rust)         │ │
│   │  #[rust_fn]         │      │  Plan 092                       │ │
│   │                     │      │                                 │ │
│   │  IDs: 0-9999        │      │  IDs: 10000+                    │ │
│   │                     │      │                                 │ │
│   │  Built into VM:     │      │  Loaded from sandbox:           │ │
│   │  • File.read_text   │      │  • serde_json::from_str         │ │
│   │  • File.write_text  │      │  • tokio::net::TcpStream        │ │
│   │  • Http.get         │      │  • user_crate::custom_fn        │ │
│   │  • Json.parse       │      │                                 │ │
│   │                     │      │                                 │ │
│   │  Zero overhead      │      │  ABI verified                   │ │
│   │  Compile-time type  │      │  Runtime loaded                 │ │
│   └─────────────────────┘      └─────────────────────────────────┘ │
│               │                              │                     │
│               ▼                              ▼                     │
│   ┌─────────────────────────────────────────────────────────────┐  │
│   │                    NativeInterface                          │  │
│   │                                                             │  │
│   │   if id < 10000: static_shims[id]     # Array lookup       │  │
│   │   else:          dynamic_shims[id]    # HashMap lookup     │  │
│   │                                                             │  │
│   └─────────────────────────────────────────────────────────────┘  │
│                              │                                     │
│                              ▼                                     │
│   ┌─────────────────────────────────────────────────────────────┐  │
│   │                      CALL_NAT opcode                        │  │
│   │                    (doesn't care which)                     │  │
│   └─────────────────────────────────────────────────────────────┘  │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

## Implementation

### Phase 1: VMConvertible Trait

Create automatic type conversion between Rust and AutoVM types.

```rust
// crates/auto-lang/src/vm/ffi/convert.rs

/// Trait for types that can cross the FFI boundary
pub trait VMConvertible: Sized {
    /// Convert from AutoVM value to Rust type
    fn from_vm(value: &VMValue, vm: &AutoVM) -> Result<Self, FFIError>;

    /// Convert from Rust type to AutoVM value
    fn to_vm(&self, vm: &mut AutoVM) -> Result<VMValue, FFIError>;
}

// Implementations for common types

impl VMConvertible for String {
    fn from_vm(value: &VMValue, vm: &AutoVM) -> Result<Self, FFIError> {
        match value {
            VMValue::String(idx) => {
                let bytes = vm.get_string(*idx)
                    .ok_or(FFIError::InvalidStringIndex(*idx))?;
                Ok(String::from_utf8_lossy(bytes).to_string())
            }
            _ => Err(FFIError::TypeMismatch {
                expected: "String",
                found: value.type_name(),
            }),
        }
    }

    fn to_vm(&self, vm: &mut AutoVM) -> Result<VMValue, FFIError> {
        let idx = vm.add_string(self.as_bytes().to_vec());
        Ok(VMValue::String(idx))
    }
}

impl VMConvertible for i32 {
    fn from_vm(value: &VMValue, _vm: &AutoVM) -> Result<Self, FFIError> {
        match value {
            VMValue::Int(n) => Ok(*n),
            _ => Err(FFIError::TypeMismatch {
                expected: "i32",
                found: value.type_name(),
            }),
        }
    }

    fn to_vm(&self, _vm: &mut AutoVM) -> Result<VMValue, FFIError> {
        Ok(VMValue::Int(*self))
    }
}

impl VMConvertible for bool {
    fn from_vm(value: &VMValue, _vm: &AutoVM) -> Result<Self, FFIError> {
        match value {
            VMValue::Bool(b) => Ok(*b),
            VMValue::Int(n) => Ok(*n != 0),
            _ => Err(FFIError::TypeMismatch {
                expected: "bool",
                found: value.type_name(),
            }),
        }
    }

    fn to_vm(&self, _vm: &mut AutoVM) -> Result<VMValue, FFIError> {
        Ok(VMValue::Bool(*self))
    }
}

// Support for Result types (for fallible FFI functions)
impl<T: VMConvertible, E: std::fmt::Display> VMConvertible for Result<T, E> {
    fn from_vm(value: &VMValue, vm: &AutoVM) -> Result<Self, FFIError> {
        Ok(Ok(T::from_vm(value, vm)?))
    }

    fn to_vm(&self, vm: &mut AutoVM) -> Result<VMValue, FFIError> {
        match self {
            Ok(t) => t.to_vm(vm),
            Err(e) => Err(FFIError::RuntimeError(e.to_string())),
        }
    }
}

// Support for Vec<T> (lists)
impl<T: VMConvertible + Clone> VMConvertible for Vec<T> {
    fn from_vm(value: &VMValue, vm: &AutoVM) -> Result<Self, FFIError> {
        match value {
            VMValue::List(list_id) => {
                let list = vm.get_heap_object(*list_id as u64)
                    .ok_or(FFIError::InvalidListId(*list_id as u64))?;
                let guard = list.read().unwrap();

                // Extract elements (simplified)
                let mut result = Vec::new();
                // ... iterate list and convert each element
                Ok(result)
            }
            _ => Err(FFIError::TypeMismatch {
                expected: "List",
                found: value.type_name(),
            }),
        }
    }

    fn to_vm(&self, vm: &mut AutoVM) -> Result<VMValue, FFIError> {
        // Create new list and populate
        // ... implementation
        Ok(VMValue::List(0)) // placeholder
    }
}
```

### Phase 2: #[rust_fn] Macro

Create procedural macro for static FFI registration.

```rust
// crates/auto-lang/src/vm/ffi/macros.rs

/// Macro to register a Rust function as a static FFI binding
///
/// # Example
///
/// ```rust
/// #[rust_fn("File.read_text")]
/// fn read_text(path: String) -> Result<String, io::Error> {
///     std::fs::read_to_string(&path)
/// }
/// ```
///
/// This generates:
/// 1. A shim function compatible with NativeInterface
/// 2. Static registration in the NATIVE_REGISTRY
/// 3. Type conversion code using VMConvertible
#[proc_macro_attribute]
pub fn rust_fn(attr: TokenStream, item: TokenStream) -> TokenStream {
    let name = attr.to_string().trim_matches('"').to_string();
    let func = parse_macro_input!(item as ItemFn);

    // Generate shim code
    let shim_name = quote::format_ident!("__shim_{}", name.replace(".", "_"));
    let func_name = &func.sig.ident;

    let expanded = quote! {
        #func

        #[allow(non_snake_case)]
        pub fn #shim_name(task: &mut crate::vm::task::AutoTask, vm: &crate::vm::engine::AutoVM)
            -> Result<(), crate::vm::engine::VMError>
        {
            use crate::vm::ffi::VMConvertible;

            // Get arguments from stack (reverse order)
            let args = /* extract args from task.ram */;

            // Call the actual function
            let result = #func_name(/* converted args */);

            // Convert result back to VM value
            let vm_result = result.to_vm(vm)
                .map_err(|e| crate::vm::engine::VMError::RuntimeError(e.to_string()))?;

            // Push result to stack
            vm_result.push_to_stack(task);

            Ok(())
        }

        // Static registration
        crate::inventory::submit! {
            crate::vm::ffi::StaticFFIRegistration {
                name: #name,
                shim: #shim_name,
            }
        }
    };

    expanded.into()
}
```

### Phase 3: Unified NativeInterface

Update NativeInterface to support hybrid lookup.

```rust
// crates/auto-lang/src/vm/native.rs

const STATIC_ID_MAX: u16 = 10000;

pub struct NativeInterface {
    // Static: Fixed-size array for direct lookup (fast)
    static_shims: [Option<ShimFunc>; STATIC_ID_MAX as usize],

    // Dynamic: HashMap for runtime additions (flexible)
    dynamic_shims: HashMap<u16, ShimFunc>,

    // Next available dynamic ID
    next_dynamic_id: u16,
}

impl NativeInterface {
    pub fn new() -> Self {
        Self {
            static_shims: [const { None }; STATIC_ID_MAX as usize],
            dynamic_shims: HashMap::new(),
            next_dynamic_id: STATIC_ID_MAX,
        }
    }

    /// Register a static shim (called by #[rust_fn] macro)
    pub fn register_static(&mut self, id: u16, shim: ShimFunc) {
        assert!(id < STATIC_ID_MAX, "Static ID must be < {}", STATIC_ID_MAX);
        self.static_shims[id as usize] = Some(shim);
    }

    /// Register a dynamic shim (called by use.rust loading)
    pub fn register_dynamic(&mut self, shim: ShimFunc) -> u16 {
        let id = self.next_dynamic_id;
        self.next_dynamic_id += 1;
        self.dynamic_shims.insert(id, shim);
        id
    }

    /// Unified lookup - used by CALL_NAT opcode
    pub fn get(&self, id: u16) -> Option<&ShimFunc> {
        if id < STATIC_ID_MAX {
            self.static_shims.get(id as usize)?.as_ref()
        } else {
            self.dynamic_shims.get(&id)
        }
    }

    /// Check if ID is static or dynamic
    pub fn is_static(&self, id: u16) -> bool {
        id < STATIC_ID_MAX
    }
}
```

### Phase 4: Static FFI Registry

Use `inventory` crate for compile-time registration.

```rust
// crates/auto-lang/src/vm/ffi/registry.rs

use inventory::Collect;

/// Static registration entry (submitted by #[rust_fn] macro)
pub struct StaticFFIRegistration {
    pub name: &'static str,
    pub shim: fn(&mut AutoTask, &AutoVM) -> Result<(), VMError>,
}

impl Collect for StaticFFIRegistration {
    #[inline]
    fn registry() -> &'static registry::Registry<Self> {
        static REGISTRY: registry::Registry<StaticFFIRegistration> = registry::Registry::new();
        &REGISTRY
    }
}

/// Initialize all static FFI registrations
pub fn init_static_ffi(natives: &mut NativeInterface, registry: &mut AutoVMNativeRegistry) {
    for registration in inventory::iter::<StaticFFIRegistration> {
        // Assign ID based on name hash (deterministic)
        let id = hash_name_to_id(registration.name);

        // Register shim
        natives.register_static(id, Arc::new((registration.shim)));

        // Register name mapping
        registry.register_static(registration.name, id);
    }
}

fn hash_name_to_id(name: &str) -> u16 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    name.hash(&mut hasher);
    (hasher.finish() % (STATIC_ID_MAX as u64)) as u16
}
```

### Phase 5: Built-in Stdlib Functions

Implement static FFI for common stdlib functions.

```rust
// crates/auto-lang/src/vm/ffi/stdlib.rs

use std::fs;
use std::io;

#[rust_fn("File.read_text")]
fn file_read_text(path: String) -> Result<String, io::Error> {
    fs::read_to_string(&path)
}

#[rust_fn("File.write_text")]
fn file_write_text(path: String, content: String) -> Result<(), io::Error> {
    fs::write(&path, &content)
}

#[rust_fn("File.exists")]
fn file_exists(path: String) -> bool {
    fs::metadata(&path).is_ok()
}

#[rust_fn("Env.get")]
fn env_get(key: String) -> Option<String> {
    std::env::var(&key).ok()
}

#[rust_fn("Env.set")]
fn env_set(key: String, value: String) {
    std::env::set_var(&key, &value);
}

#[rust_fn("Process.exit")]
fn process_exit(code: i32) {
    std::process::exit(code);
}

#[rust_fn("Time.now_ms")]
fn time_now_ms() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64
}
```

### Phase 6: VM Intrinsics (Keep Low-Level)

VM intrinsics that need direct access to VM internals remain as low-level shims.

```rust
// crates/auto-lang/src/vm/native.rs

// These stay as manual shims because they need VM internals:

pub fn shim_list_new(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    // Direct access to vm.insert_heap_object()
    // Cannot use #[rust_fn] because List is a VM-internal type
}

pub fn shim_list_push(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    // Direct access to vm.get_heap_object()
    // Needs to downcast to ListData<i32>
}

pub fn shim_hashmap_new(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    // Direct access to heap registry
}

// etc.
```

## JIT Optimization Path

```
┌──────────────────────────────────────────────────────────────────────┐
│                        JIT Optimization                               │
├──────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  Static (File.read_text, ID 134):                                    │
│  ┌────────────────────────────────────────────────────────────────┐ │
│  │  Tier 0: CALL_NAT 134                                          │ │
│  │  Tier 1: call [static_shims + 134 * 8]    ; Direct            │ │
│  │  Tier 2: inlined                          ; Full inline        │ │
│  └────────────────────────────────────────────────────────────────┘ │
│                                                                      │
│  Dynamic (serde_json::from_str, ID 10000):                           │
│  ┌────────────────────────────────────────────────────────────────┐ │
│  │  Tier 0: CALL_NAT 10000                                        │ │
│  │  Tier 1: call [dynamic_shims.lookup(10000)]  ; Indirect        │ │
│  │  Tier 2: inline cache + guard                ; If stable       │ │
│  └────────────────────────────────────────────────────────────────┘ │
│                                                                      │
└──────────────────────────────────────────────────────────────────────┘
```

### Inline Cache for Dynamic FFI

```rust
// Future: JIT inline cache implementation

struct InlineCacheEntry {
    expected_func_id: u16,      // Last seen function ID
    func_ptr: *const u8,        // Direct pointer to shim
    hit_count: u32,             // Optimization hint
}

// JIT can "promote" stable dynamic bindings to pseudo-static
impl JITCompiler {
    fn maybe_promote(&mut self, call_site: CallSite) {
        let stats = &self.call_site_stats[call_site];

        if stats.hit_count > 1000 && stats.unique_targets == 1 {
            // This dynamic binding has been stable!
            // Promote to "pseudo-static" for aggressive optimization
            self.promote_to_pseudo_static(call_site, stats.last_target);
        }
    }
}
```

## ID Space Allocation

| Range | Category | Registration | Lookup |
|-------|----------|--------------|--------|
| 0-999 | VM Intrinsics | Manual shim | Array |
| 1000-4999 | Built-in stdlib | `#[rust_fn]` | Array |
| 5000-9999 | Reserved | - | - |
| 10000-65535 | Dynamic FFI | `use.rust` | HashMap |

## Comparison: Static vs Dynamic

| Aspect | Static (`#[rust_fn]`) | Dynamic (`use.rust`) |
|--------|----------------------|---------------------|
| **Registration** | Compile-time | Runtime |
| **ID Range** | 0-9999 | 10000+ |
| **Lookup** | Array O(1) | HashMap O(1) |
| **Type Safety** | Compile-time | Runtime |
| **Performance** | Zero overhead | Indirect call |
| **Inlining** | Full inline possible | Inline cache |
| **Flexibility** | Requires recompile | Load any crate |
| **ABI Check** | Not needed | Sandbox verified |

## Stdlib Shims for Self-Hosting

> **Goal**: Implement ~20 manual shims before designing `#[rust_fn]` macro
> **Purpose**: Support self-hosting compiler (A2R + AutoVM) path

### Self-Hosting Requirements Analysis

A self-hosting AutoLang compiler needs these capabilities:

| Component | Required Operations |
|-----------|---------------------|
| **Lexer** | Character classification, substring, peek/advance |
| **Parser** | String matching, token comparison, error formatting |
| **Codegen** | String building, indentation, line tracking |
| **File I/O** | Read/write source files, output generation |
| **Imports** | Path resolution, module discovery |

### Current Implementation Status

| Category | Implemented | Needed | Gap |
|----------|-------------|--------|-----|
| **File I/O** | 5 | 7 | 2 |
| **Env** | 3 | 3 | 0 ✅ |
| **Time** | 3 | 3 | 0 ✅ |
| **Process** | 1 | 3 | 2 |
| **String** | 0 | 6 | 6 |
| **Path** | 0 | 4 | 4 |
| **Math** | 0 | 2 | 2 |
| **Total** | **12** | **28** | **16** |

### Phase 1: File & Path Operations (Critical for Compiler)

**Current (5 shims)**:
```rust
// Already implemented
NATIVE_FILE_READ_TEXT    = 1000  // ✅ File.read_text(path) -> str
NATIVE_FILE_WRITE_TEXT   = 1001  // ✅ File.write_text(path, content) -> result
NATIVE_FILE_EXISTS       = 1002  // ✅ File.exists(path) -> bool
NATIVE_FILE_DELETE       = 1003  // ✅ File.delete(path) -> result
NATIVE_FILE_CREATE_DIR   = 1004  // ✅ File.create_dir(path) -> result
```

**Needed (2 more shims)**:
```rust
// ID 1005-1006
NATIVE_FILE_READ_BYTES   = 1005  // File.read_bytes(path) -> []byte (for binary files)
NATIVE_FILE_WRITE_BYTES  = 1006  // File.write_bytes(path, []byte) -> result
NATIVE_FILE_COPY         = 1007  // File.copy(src, dst) -> result
NATIVE_FILE_SIZE         = 1008  // File.size(path) -> i64
NATIVE_FILE_IS_DIR       = 1009  // File.is_dir(path) -> bool
```

**Path Operations (4 new shims, ID 1400-1499)**:
```rust
// Path operations - critical for import resolution
NATIVE_PATH_JOIN         = 1400  // Path.join(parts...) -> str
NATIVE_PATH_PARENT       = 1401  // Path.parent(path) -> str?
NATIVE_PATH_EXTENSION    = 1402  // Path.extension(path) -> str?
NATIVE_PATH_FILENAME     = 1403  // Path.filename(path) -> str
NATIVE_PATH_CANONICALIZE = 1404  // Path.canonicalize(path) -> str
```

### Phase 2: String Operations (Critical for Lexer/Parser)

**String Operations (6 new shims, ID 1500-1599)**:

> **Why needed**: A lexer needs to classify characters, extract substrings, and build strings efficiently.

```rust
// Character classification (lexer needs these)
NATIVE_STR_LEN           = 1500  // Str.len(s) -> int
NATIVE_STR_IS_EMPTY      = 1501  // Str.is_empty(s) -> bool
NATIVE_STR_CHAR_AT       = 1502  // Str.char_at(s, index) -> int (unicode codepoint)
NATIVE_STR_SUBSTR        = 1503  // Str.substr(s, start, end) -> str
NATIVE_STR_CONTAINS      = 1504  // Str.contains(s, needle) -> bool
NATIVE_STR_STARTS_WITH   = 1505  // Str.starts_with(s, prefix) -> bool
NATIVE_STR_ENDS_WITH     = 1506  // Str.ends_with(s, suffix) -> bool
NATIVE_STR_TRIM          = 1507  // Str.trim(s) -> str
NATIVE_STR_SPLIT         = 1508  // Str.split(s, delimiter) -> []str
NATIVE_STR_REPEAT        = 1509  // Str.repeat(s, n) -> str
```

**Character Classification (for lexer, ID 1600-1699)**:
```rust
// Character predicates
NATIVE_CHAR_IS_ALPHA     = 1600  // Char.is_alpha(c) -> bool
NATIVE_CHAR_IS_DIGIT     = 1601  // Char.is_digit(c) -> bool
NATIVE_CHAR_IS_ALPHANUM  = 1602  // Char.is_alphanum(c) -> bool
NATIVE_CHAR_IS_WHITESPACE= 1603  // Char.is_whitespace(c) -> bool
NATIVE_CHAR_IS_IDENT     = 1604  // Char.is_ident(c) -> bool (for Auto identifiers)
NATIVE_CHAR_TO_LOWER     = 1605  // Char.to_lower(c) -> int
NATIVE_CHAR_TO_UPPER     = 1606  // Char.to_upper(c) -> int
```

### Phase 3: Process Operations (For Build System)

**Current (1 shim)**:
```rust
NATIVE_PROCESS_EXIT      = 1300  // ✅ Process.exit(code)
```

**Needed (2 more shims)**:
```rust
NATIVE_PROCESS_ARGS      = 1301  // ✅ Process.args() -> []str (command line args)
NATIVE_PROCESS_SPAWN     = 1302  // Process.spawn(cmd, args) -> result (run external tool)
NATIVE_PROCESS_CURRENT_DIR = 1303  // Process.current_dir() -> str
NATIVE_PROCESS_SET_CURRENT_DIR = 1304  // Process.set_current_dir(path) -> result
```

### Phase 4: Math Operations (For Const Evaluation)

**Math Operations (2 new shims, ID 1700-1799)**:
```rust
// For compile-time constant evaluation
NATIVE_MATH_ABS          = 1700  // Math.abs(n) -> n
NATIVE_MATH_MIN          = 1701  // Math.min(a, b) -> n
NATIVE_MATH_MAX          = 1702  // Math.max(a, b) -> n
NATIVE_MATH_POW          = 1703  // Math.pow(base, exp) -> n
NATIVE_MATH_SQRT         = 1704  // Math.sqrt(n) -> float
```

### Complete Shim Roadmap

| Phase | Category | Shims | IDs | Priority |
|-------|----------|-------|-----|----------|
| ✅ Done | File I/O | 5 | 1000-1004 | Critical |
| ✅ Done | Env | 3 | 1100-1102 | High |
| ✅ Done | Time | 3 | 1200-1202 | Medium |
| ✅ Done | Process | 1 | 1300 | High |
| 🔄 Next | **Path** | 5 | 1400-1404 | **Critical** (imports) |
| 🔄 Next | **String** | 10 | 1500-1509 | **Critical** (lexer) |
| ⏳ Soon | **Char** | 7 | 1600-1606 | **Critical** (lexer) |
| ⏳ Soon | Process+ | 4 | 1301-1304 | High (build) |
| ⏳ Later | Math | 5 | 1700-1704 | Medium |
| **Total** | | **43** | | |

### Minimum Viable Set for Self-Hosting

To compile the AutoLang compiler itself, we need at minimum:

| Shim | Why Needed |
|------|------------|
| `File.read_text` | Read source files |
| `File.write_text` | Write generated code |
| `File.exists` | Check import paths |
| `Path.join` | Resolve import paths |
| `Path.parent` | Relative imports |
| `Path.extension` | Detect .at files |
| `Str.len` | Lexer position tracking |
| `Str.char_at` | Lexer peek operation |
| `Str.substr` | Extract tokens |
| `Str.starts_with` | Keyword matching |
| `Char.is_alpha` | Identifier lexing |
| `Char.is_digit` | Number lexing |
| `Char.is_whitespace` | Skip whitespace |
| `Process.args` | CLI argument parsing |

**Minimum: 14 shims** (we have 12, need 2 more)

### Shim Implementation Pattern

Before `#[rust_fn]`, all shims follow this pattern:

```rust
/// Shim: Path.join(parts...) -> str
///
/// Stack: n, part_n, ..., part_0 -> result_str_idx
pub fn shim_path_join(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    // 1. Pop argument count
    let n = task.ram.pop_i32() as usize;

    // 2. Pop arguments in reverse order
    let mut parts = Vec::with_capacity(n);
    for _ in 0..n {
        let part: String = VMConvertible::pop_from_stack(task, vm)
            .map_err(|e| VMError::RuntimeError(e.to_string()))?;
        parts.push(part);
    }
    parts.reverse();

    // 3. Perform operation
    let result = parts.join(std::path::MAIN_SEPARATOR_STR);

    // 4. Push result
    result.push_to_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;

    Ok(())
}
```

### ID Space Allocation (Updated)

| Range | Category | Count | Status |
|-------|----------|-------|--------|
| 0-999 | VM Intrinsics | 1000 | ✅ Existing |
| 1000-1099 | File I/O | 10 | ✅ 5/10 done |
| 1100-1199 | Environment | 10 | ✅ 3/10 done |
| 1200-1299 | Time | 10 | ✅ 3/10 done |
| 1300-1399 | Process | 10 | ⏳ 1/10 done |
| 1400-1499 | Path | 10 | ⏳ 0/10 done |
| 1500-1599 | String | 10 | ⏳ 0/10 done |
| 1600-1699 | Char | 10 | ⏳ 0/10 done |
| 1700-1799 | Math | 10 | ⏳ 0/10 done |
| 1800-9999 | Reserved | 8200 | - |
| 10000+ | Dynamic FFI | 55535 | ✅ Infrastructure ready |

## Implementation Phases

### Phase 1: Core Infrastructure ✅ Complete
- [x] Create `vm/ffi/` module structure
- [x] Implement `VMConvertible` trait
- [x] Add implementations for primitive types
- [x] Add unit tests for type conversion

### Phase 2: Unified Lookup ✅ Complete
- [x] Update `NativeInterface` for hybrid lookup
- [x] Add ID range validation
- [x] Update `CALL_NAT` opcode to use unified `get()`
- [x] Integration tests

### Phase 3: Manual Shim Implementation ✅ Complete
- [x] File I/O shims (10/10) ✅
- [x] Env shims (3/3) ✅
- [x] Time shims (3/3) ✅
- [x] Process shims (5/5) ✅
- [x] **Path shims (5/5)** ✅
  - [x] `Path.join(parts...) -> str`
  - [x] `Path.parent(path) -> str`
  - [x] `Path.extension(path) -> str`
  - [x] `Path.filename(path) -> str`
  - [x] `Path.canonicalize(path) -> str`
- [x] **String shims (10/10)** ✅
  - [x] `Str.len(s) -> int`
  - [x] `Str.is_empty(s) -> bool`
  - [x] `Str.char_at(s, index) -> int`
  - [x] `Str.substr(s, start, end) -> str`
  - [x] `Str.contains(s, needle) -> bool`
  - [x] `Str.starts_with(s, prefix) -> bool`
  - [x] `Str.ends_with(s, suffix) -> bool`
  - [x] `Str.trim(s) -> str`
  - [x] `Str.split(s, delimiter) -> List<str>`
  - [x] `Str.repeat(s, n) -> str`
- [x] **Char shims (7/7)** ✅
  - [x] `Char.is_alpha(c) -> bool`
  - [x] `Char.is_digit(c) -> bool`
  - [x] `Char.is_alphanum(c) -> bool`
  - [x] `Char.is_whitespace(c) -> bool`
  - [x] `Char.is_ident(c) -> bool`
  - [x] `Char.to_lower(c) -> int`
  - [x] `Char.to_upper(c) -> int`
- [x] **Process shims (5/5)** ✅
  - [x] `Process.args() -> []str`
  - [x] `Process.spawn(cmd, args) -> result`
  - [x] `Process.current_dir() -> str`
  - [x] `Process.set_current_dir(path) -> result`
  - [x] `Process.exit(code)`
- [x] **Math shims (4/4)** ✅
  - [x] `Math.abs(n) -> n`
  - [x] `Math.min(a, b) -> n`
  - [x] `Math.max(a, b) -> n`
  - [x] `Math.sqrt(n) -> float`
- [x] **File+ shims (5/5)** ✅
  - [x] `File.read_bytes(path) -> []byte`
  - [x] `File.write_bytes(path, []byte) -> result`
  - [x] `File.copy(src, dst) -> result`
  - [x] `File.size(path) -> i64`
  - [x] `File.is_dir(path) -> bool`

**Total: 43/43 shims implemented (100%)** ✅

### Phase 4: `#[rust_fn]` Macro Design 🔄 Ready for Analysis
> **Unblocked**: We now have 32 shims to analyze patterns

After implementing 32 shims, we can identify:
- **Common patterns**:
  - All shims pop args in reverse order (LIFO)
  - Most use `VMConvertible::pop_from_stack`
  - Return values use `push_to_stack` or direct `task.ram.push_*`
  - Error handling via `map_err(|e| VMError::RuntimeError(e.to_string()))`
- **Type patterns**:
  - `String` for text
  - `i32`/`i64` for numbers
  - `bool` as `i32` (0/1)
  - `Vec<i32>` for int lists
  - `Vec<String>` for string lists (needs VMConvertible impl)

- [ ] Analyze patterns from 32 manual shims
- [ ] Implement `#[rust_fn]` procedural macro
- [ ] Create `StaticFFIRegistration` with `inventory`
- [ ] Integrate with `NativeInterface::register_static()`
- [ ] Migrate existing shims to `#[rust_fn]` (optional)

### Phase 5: Documentation & Tests ⏳ Future
- [ ] Document all shims with examples
- [ ] Document `#[rust_fn]` usage (if implemented)
- [ ] Document `use.rust` + `dep` workflow
- [ ] Add comprehensive test suite for each shim
- [ ] Update CLAUDE.md with stdlib reference

### Phase 6: JIT Preparation ⏳ Future
- [ ] Design inline cache structure
- [ ] Add call site statistics
- [ ] Implement promotion logic
- [ ] Performance benchmarks

## File Structure

```
crates/auto-lang/src/vm/
├── ffi/
│   ├── mod.rs              # Public API
│   ├── convert.rs          # VMConvertible trait + impls
│   ├── macros.rs           # #[rust_fn] procedural macro
│   ├── registry.rs         # Static registration with inventory
│   └── stdlib.rs           # Built-in FFI functions
├── native.rs               # NativeInterface (updated)
└── native_registry.rs      # Name → ID mapping (updated)
```

## Success Criteria

### Phase 1-2 Complete ✅
- [x] `VMConvertible` trait implemented
- [x] Type conversion works for String, i32, bool
- [x] Unified `CALL_NAT` works for static and dynamic
- [x] ID ranges enforced

### Phase 3 Milestones (Shim Implementation)

**Milestone 1: Minimum Self-Hosting (14 shims)**
- [x] 5 File I/O shims
- [x] 3 Env shims
- [ ] 5 Path shims (critical for imports)
- [ ] 1 Process.args shim (CLI parsing)

**Milestone 2: Lexer Support (24 shims)**
- [ ] 10 String shims
- [ ] 7 Char shims (character classification)

**Milestone 3: Full Stdlib (43 shims)**
- [ ] 4 Process+ shims
- [ ] 5 Math shims
- [ ] Remaining File/Path shims

### Phase 4 Complete (`#[rust_fn]` ready)
- [ ] 20+ manual shims implemented
- [ ] Patterns analyzed and documented
- [ ] `#[rust_fn]` macro generates working shims
- [ ] Can optionally migrate existing shims

### Overall
- [ ] Static FFI functions callable from AutoVM
- [ ] Dynamic FFI (Plan 092) still works
- [ ] All tests pass
- [ ] Documentation complete

## Dependencies

- `inventory` crate for compile-time registration (Phase 4)
- Plan 092 for dynamic FFI infrastructure
- `syn` and `quote` for procedural macro (Phase 4)

## Risks

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Macro complexity | Medium | Medium | Start simple, iterate |
| Type conversion bugs | Medium | High | Comprehensive tests |
| Performance regression | Low | Medium | Benchmark before/after |
| ABI incompatibility | Low | High | Sandbox verification |

## Related Plans

- [Plan 092: Dynamic FFI](./092-dynamic-ffi.md) - `use.rust` + sandbox
- [Plan 091: Universe Removal](./091-universe-removal.md) - VM architecture
- [Plan 093: Auto-Man Rust Support](./093-automan-rust-support.md) - Cargo integration
- [Plan 095: Compile-Time Execution Engine](./095-compile-time-execution-engine.md) - Requires FFI for comptime native calls

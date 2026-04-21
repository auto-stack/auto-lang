# Plan 092: Rust FFI via Sandbox Compilation

## Status: Phases 1-6 Complete ✅

**Last Updated**: 2026-02-26

## Problem

The AutoVM can currently only call Rust functions that are pre-registered at compile time (via `NativeInterface::register_std_shims()`). This is limiting because:
1. Users cannot import and use external Rust crates at runtime
2. The interpreter cannot dynamically extend its capabilities
3. Each new native function requires modifying the codebase and recompiling

## Goal

Enable AutoVM to dynamically load Rust crates and call their functions at runtime, making it a true interpreter.

## Key Insight: ABI Stability Through Sandbox Compilation

**Problem**: Rust's ABI is not stable across different compilations, so passing native Rust types (Vec, HashMap, etc.) across dynamic library boundaries is unsafe.

**Solution**: Control all compilation through a **sandbox** that ensures:
1. Same toolchain (rustc version)
2. Same dependencies (shared libstd, shared crates)
3. All libraries link to the same shared libraries

When `libstd.so` is shared between AutoVM and loaded libraries, `Vec<T>` has the same layout everywhere!

## Syntax Design (Final Decision)

### `dep` Keyword (Declaration)

```auto
// Declare dependency - downloads/loads the library
dep serde                          // Latest version
dep serde(version: "1.0")          // Specific version
dep serde(version: "1.0", features: ["derive"])  // With features
dep my_lib(path: "../my_lib")      // Local crate
dep tokio(git: "https://...", branch: "main")  // Git source
```

### `use.rust` Keyword (Import)

```auto
// Import items from declared dependency
use.rust serde::json::{from_str, to_string}
use.rust serde::Serialize
use.rust my_lib::process
```

### Key Design Decisions

1. **`use.rust` with dot** - Consistent with `use.c` syntax
2. **`dep` as keyword** - Not a node, built into parser
3. **Separation of concerns**:
   - `dep` = declare + download/load
   - `use.rust` = import into scope
4. **Error if not declared**: `use.rust unknown` → Error: "Crate 'unknown' not declared"

## Architecture: AutoCache + Sandbox Unification

```
~/.auto/
├── cache/                    # EXISTING: AutoCache
│   ├── index.db              # SQLite metadata
│   └── blobs/                # Content-addressable artifacts
│
├── sandbox/                  # Plan 092: Rust FFI
│   ├── toolchain/            # Managed Rust toolchain
│   │   └── rust-1.75.0/
│   │
│   ├── crates/               # Compiled Rust crates
│   │   ├── libstd-1.75.0.so       # Shared stdlib
│   │   ├── libserde-1.0.193.so    # Shared serde
│   │   └── libmy_lib-0.1.0.so     # User libraries
│   │
│   └── registry/             # Crate metadata (SQLite)
│       └── index.db
│
└── config/
    └── sandbox.toml
```

### Benefits of Unification

| Component | Uses Cache | Uses Sandbox |
|-----------|------------|--------------|
| A2C Transpiler | ✅ C artifacts | - |
| A2R Transpiler | ✅ Rust artifacts | ✅ Link shared libs |
| AutoVM | ✅ Bytecode | ✅ Load Rust crates |

## Implementation Plan

### Phase 1: Extend AutoCache for Rust Crates

**File**: `crates/auto-cache/src/lib.rs`

1. Add new `ArtifactType`:
```rust
pub enum ArtifactType {
    // Existing
    TranspiledC,
    TranspiledCHeader,
    TranspiledRust,
    Bytecode,
    CompiledObject,

    // NEW
    RustCrateLibrary,    // Compiled .so/.dylib/.dll
    RustCrateSource,     // Source tarball
}
```

2. Add crate metadata:
```rust
pub struct CrateMetadata {
    pub name: String,
    pub version: String,
    pub rustc_version: String,
    pub target: String,
    pub dependencies: Vec<String>,  // "serde-1.0.193"
    pub abi_hash: String,           // Computed from all deps
}
```

### Phase 2: Sandbox Toolchain Management

**File**: `crates/auto-cache/src/sandbox.rs` (new)

```rust
pub struct Sandbox {
    toolchain_path: PathBuf,
    crates_path: PathBuf,
    registry: CrateRegistry,
}

impl Sandbox {
    /// Ensure toolchain is installed
    pub fn ensure_toolchain(&self, version: &str) -> Result<PathBuf>;

    /// Compile a crate with sandboxed toolchain
    pub fn compile_crate(&mut self, name: &str, version: &str) -> Result<PathBuf>;

    /// Load a compiled crate
    pub fn load_crate(&self, name: &str) -> Result<libloading::Library>;
}
```

### Phase 3: Crate Registry

**File**: `crates/auto-cache/src/registry.rs` (new)

```rust
pub struct CrateRegistry {
    db: Connection,  // SQLite
}

impl CrateRegistry {
    /// Register a compiled crate
    pub fn register(&self, meta: &CrateMetadata) -> Result<()>;

    /// Look up crate by name
    pub fn lookup(&self, name: &str) -> Result<Option<CrateMetadata>>;

    /// Resolve dependencies recursively
    pub fn resolve_deps(&self, name: &str) -> Result<Vec<CrateMetadata>>;
}
```

### Phase 4: AutoVM Integration

**File**: `crates/auto-lang/src/ffi.rs`

```rust
impl CFfiBridge {
    /// Load a Rust crate through the sandbox
    pub fn load_rust_crate(&mut self, name: &str) -> Result<u16, VMError> {
        // 1. Check sandbox registry
        let sandbox = self.sandbox.read()?;
        if let Some(meta) = sandbox.registry.lookup(name)? {
            // Verify ABI compatibility
            if meta.rustc_version != self.host_rustc_version {
                return Err(VMError::ABIIncompatible);
            }
        } else {
            // 2. Compile crate
            sandbox.compile_crate(name, "latest")?;
        }

        // 3. Load library
        let lib = sandbox.load_crate(name)?;

        // 4. Register symbols as native functions
        self.register_symbols(&lib, name)?;

        Ok(0)
    }
}
```

### Phase 5: Parser Support (Pending)

**Files**:
- `crates/auto-lang/src/lexer.rs` - Add `Dep` token
- `crates/auto-lang/src/parser.rs` - Implement `dep_stmt()` and `use_rust_stmt()`
- `crates/auto-lang/src/ast.rs` - Add `DepStmt` struct

#### 5.1 `dep` Keyword Implementation

```rust
// In lexer.rs - Add token kind
TokenKind::Dep,  // "dep"

// In parser.rs
fn dep_stmt(&mut self) -> AutoResult<Stmt> {
    self.next(); // skip 'dep'
    let name = self.expect_ident_str()?;

    // Optional properties: (version: "1.0", features: ["derive"])
    let mut version = None;
    let mut features = Vec::new();
    let mut path = None;
    let mut git = None;

    if self.is_kind(TokenKind::LParen) {
        self.next(); // skip '('
        while !self.is_kind(TokenKind::RParen) {
            let key = self.expect_ident_str()?;
            self.expect(TokenKind::Colon)?;
            match key.as_str() {
                "version" => version = Some(self.parse_string_literal()?),
                "features" => features = self.parse_string_array()?,
                "path" => path = Some(self.parse_string_literal()?),
                "git" => git = Some(self.parse_string_literal()?),
                _ => return Err(SyntaxError::UnknownProperty(key).into()),
            }
            if self.is_kind(TokenKind::Comma) {
                self.next();
            }
        }
        self.expect(TokenKind::RParen)?;
    }

    Ok(Stmt::Dep(DepStmt {
        name,
        version,
        features,
        path,
        git,
    }))
}
```

#### 5.2 `use.rust` Keyword Implementation

```rust
// In parser.rs - Called from use_stmt() when .rust is seen
fn use_rust_stmt(&mut self) -> AutoResult<Stmt> {
    // Already consumed: use . rust
    // Parse: serde::json::{from_str, to_string}
    let crate_name = self.expect_ident_str()?;
    let mut paths = vec![crate_name];

    // Parse module path
    while self.is_kind(TokenKind::ColonColon) {
        self.next();
        if self.is_kind(TokenKind::LBrace) {
            break;  // Start of import items
        }
        paths.push(self.expect_ident_str()?);
    }

    // Parse import items: {item1, item2}
    let items = self.parse_import_items()?;

    Ok(Stmt::Use(Use {
        kind: UseKind::Rust,
        paths,
        items,
    }))
}
```

#### 5.3 CompileSession Integration

**Status**: ✅ Complete

```rust
// In compile.rs
pub struct CompileSession {
    // ...
    sandbox: Option<Sandbox>,
    declared_crates: HashSet<String>,  // Track declared deps
}

impl CompileSession {
    /// Register a dep statement (called during indexing)
    pub fn register_dep(&mut self, dep: &crate::ast::Dep) {
        if !dep.is_rust {
            return;
        }
        self.declared_crates.insert(dep.name.clone());
    }

    /// Check if crate was declared (called for use.rust validation)
    pub fn is_dep_declared(&self, crate_name: &str) -> bool {
        self.declared_crates.contains(crate_name)
    }
}
```

### Phase 6: Runtime Crate Loading ✅ Complete

**Goal**: Load compiled Rust crates at runtime and call their functions.

**Implementation Status**: All core infrastructure complete. The `RustFfiBridge` in `ffi.rs` provides:
- `load_rust_crate()` - Load crate through sandbox with ABI verification
- `load_rust_library()` - Load from direct path
- `register_function()` - Register functions as native shims
- `get_function_id()` - Lookup registered functions

**Integration**: `CompileSession::create_rust_ffi_bridge()` provides access to the bridge.

**Implementation Sketch** (for reference):

```rust
// In vm/engine.rs
impl AutoVM {
    /// Load a Rust crate through the sandbox
    pub fn load_rust_crate(&mut self, name: &str, version: &str) -> Result<(), VMError> {
        let sandbox = self.sandbox.as_ref().ok_or(VMError::NoSandbox)?;

        // 1. Check if already loaded
        if self.loaded_crates.contains(name) {
            return Ok(());
        }

        // 2. Verify ABI compatibility
        let meta = sandbox.registry.lookup(name)?
            .ok_or(VMError::CrateNotFound(name.into()))?;
        sandbox.verify_abi(&meta)?;

        // 3. Load dynamic library
        let lib = unsafe { sandbox.load_crate(name, &meta.version)? };

        // 4. Register exported symbols as native functions
        self.register_crate_symbols(&lib, name)?;

        self.loaded_crates.insert(name.to_string());
        Ok(())
    }
}
```

## Usage Example

```auto
// === Dependency Declaration ===
dep serde(version: "1.0", features: ["derive"])
dep serde_json(version: "1.0")
dep my_lib(path: "../my_lib")

// === Import into Scope ===
use.rust serde::json::{from_str, to_string}
use.rust serde::Serialize
use.rust my_lib::process

// === Usage ===
fn main() {
    let json = "{ \"name\": \"Auto\" }"
    let data: User = from_str(json)
    print(data.name)
}
```

## Critical Files

| File | Purpose | Status |
|------|---------|--------|
| `crates/auto-cache/src/lib.rs` | Extend ArtifactType | ✅ Complete |
| `crates/auto-cache/src/sandbox.rs` | Sandbox management + registry field | ✅ Complete |
| `crates/auto-cache/src/registry.rs` | Crate registry | ✅ Complete |
| `crates/auto-lang/src/ffi.rs` | RustFfiBridge (load_crate, register_function) | ✅ Complete |
| `crates/auto-lang/src/token.rs` | Add `Dep` token | ✅ Complete |
| `crates/auto-lang/src/lexer.rs` | Lexer support for `dep` | ✅ Complete |
| `crates/auto-lang/src/parser.rs` | Implement `dep` and `use.rust` | ✅ Complete |
| `crates/auto-lang/src/ast/dep_.rs` | Add `DepStmt` | ✅ Complete |
| `crates/auto-lang/src/ast/use_.rs` | Add `UseKind::Rust` | ✅ Complete |
| `crates/auto-lang/src/compile.rs` | `declared_crates` + registry + create_rust_ffi_bridge() | ✅ Complete |
| `crates/auto-lang/src/indexer.rs` | Dep statement indexing | ✅ Complete |
| `crates/auto-lang/src/vm/codegen.rs` | Handle Rust imports | ✅ Complete (via RustFfiBridge) |
| `crates/auto-lang/src/vm/engine.rs` | Runtime crate loading | ✅ Complete (via RustFfiBridge) |

### Phase 6: Runtime Integration ✅ Complete

**Status**: All integration complete, including actual symbol resolution

**Completed in this session**:
- ✅ Added `registry: CrateRegistry` field to `Sandbox` struct
- ✅ Initialize registry in `Sandbox::with_root()`
- ✅ Added `registry()` and `registry_mut()` getters
- ✅ Wired `compile.rs::resolve_deps()` to register crates with sandbox registry
- ✅ Added `create_rust_ffi_bridge()` to `CompileSession`
- ✅ Added `get_declared_crates()` helper method
- ✅ Implemented actual symbol resolution in `RustFfiBridge::register_function()`
- ✅ Argument marshaling from AutoVM stack to C types (i32, f32, f64, pointers)
- ✅ Return value marshaling from C to AutoVM
- ✅ All 9 FFI tests passing
- ✅ All 10 sandbox/registry tests passing

**Supported Types**:
- Primitive: `Void`, `Bool`, `Int`, `Long` (i64), `Float` (f32), `Double` (f64)
- String: `String` (null-terminated C string, `*const c_char`)
- Pointer: `Pointer`, `PointerMut` (struct pointers, `*mut c_void`)
- Binary: `Bytes` (pointer + length)
- Callback: `Callback` (function pointers)

**Supported Function Signatures** (40+ patterns):
- `fn() -> void/i32/i64/f32/f64/bool/pointer`
- `fn(i32) -> void/i32/i64`
- `fn(i32, i32) -> void/i32`
- `fn(f32) -> f32`
- `fn(f64) -> f64`
- `fn(i32, f32/f64) -> i32/f64`
- `fn(string) -> void/i32` (strlen, parse, etc.)
- `fn(string, string) -> i32` (strcmp, etc.)
- `fn(string, i32) -> i32` (strncmp, etc.)
- `fn(pointer) -> void/i32/pointer` (struct methods)
- `fn(pointer, i32/string) -> void/i32` (methods with params)
- `fn(bytes) -> void/i32` (buffer operations)
- `fn(i64) -> void/i64` (64-bit operations)

## Verification

```bash
# Unit tests
cargo test -p auto-cache -- sandbox
cargo test -p auto-lang -- rust_ffi

# Integration test
cd test/fixtures/rust_lib
cargo build --release
cp target/release/libtest_lib.so ~/.auto/sandbox/crates/

# Run Auto code using Rust library
cargo run -p auto-lang -- run examples/use_rust.at
```

## Open Questions

1. **Crate source**: Download from crates.io or require pre-built?
2. **Version resolution**: How to resolve version conflicts?
3. **Garbage collection**: When to clean up unused crates?
4. **Offline mode**: Support fully offline builds?

## Dependencies

- Plan 082: AutoCache (content-addressable storage)
- Plan 091: Universe Removal (InferenceContext)
- `libloading` crate (already in dependencies)

## Timeline

- Phase 1-2: 2-3 days (AutoCache extension)
- Phase 3: 1-2 days (Registry)
- Phase 4-5: 2-3 days (AutoVM integration)
- Testing: 1 day

**Total**: ~1 week

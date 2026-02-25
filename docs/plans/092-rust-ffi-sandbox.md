# Plan 092: Rust FFI via Sandbox Compilation

## Status: Draft

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

## Architecture: AutoCache + Sandbox Unification

```
~/.auto/
├── cache/                    # EXISTING: AutoCache
│   ├── index.db              # SQLite metadata
│   └── blobs/                # Content-addressable artifacts
│
├── sandbox/                  # NEW: Unified compilation sandbox
│   ├── toolchain/            # Managed Rust toolchain
│   │   └── rust-1.75.0/
│   │
│   ├── crates/               # Compiled Rust crates
│   │   ├── libstd-1.75.0.so       # Shared stdlib
│   │   ├── libserde-1.0.193.so    # Shared serde
│   │   └── libmy_lib-0.1.0.so     # User libraries
│   │
│   └── registry/             # Crate metadata
│       ├── serde.json
│       └── my_lib.json
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

### Phase 5: Parser Support

**File**: `crates/auto-lang/src/parser.rs`

```rust
pub fn use_rust_stmt(&mut self) -> AutoResult<Stmt> {
    // Parse: use rust serde::json::{from_str, to_string}
    let crate_name = self.expect_ident_str()?;  // "serde"
    let module = self.parse_module_path()?;     // ["json"]
    let items = self.parse_import_items()?;     // ["from_str", "to_string"]

    Ok(Stmt::Use(Use {
        kind: UseKind::Rust,
        crate_name: Some(crate_name),
        module,
        items,
    }))
}
```

## Usage Example

```auto
// Import Rust crate
use rust serde_json::{from_str, to_string}

fn main() {
    let json = "{ \"name\": \"Auto\" }"
    let data = from_str(json)
    print(data.name)  // "Auto"
}
```

## Critical Files

| File | Purpose |
|------|---------|
| `crates/auto-cache/src/lib.rs` | Extend ArtifactType |
| `crates/auto-cache/src/sandbox.rs` | NEW - Sandbox management |
| `crates/auto-cache/src/registry.rs` | NEW - Crate registry |
| `crates/auto-lang/src/ffi.rs` | Complete library loading |
| `crates/auto-lang/src/parser.rs` | Implement `use_rust_stmt()` |
| `crates/auto-lang/src/vm/codegen.rs` | Handle Rust imports |

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

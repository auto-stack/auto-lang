# Plan 082: AutoCache - Global Build Cache Implementation

## Objective

Implement AutoCache, a global build cache system that stores compiled artifacts across projects, enabling cross-project compilation artifact reuse and dramatically reducing build times.

## Architecture Overview

AutoCache is a **Content-Addressable Store (CAS)** system that:
- Stores compiled artifacts globally (~/.auto/cache/)
- Uses AST-level hashing for format-independent fingerprints
- Integrates with AutoMan, a2c, a2r, and AutoVM
- Supports cross-project and cross-version artifact sharing

## Design Summary

### Storage Architecture
```
~/.auto/cache/
├── index.db            # SQLite metadata
├── blobs/              # Binary artifacts (CAS)
│   ├── a1/             # Sharded by hash prefix
│   │   └── a1b2c3d4...
│   └── f9/
└── locks/              # Process locks
```

### Fingerprint Strategy
```
TargetHash = SHA256(
    ContentHash(AST)    // Format-independent, path-remapped
    + ContextHash(target, flags, toolchain)
    + DependencyHash(merkle_tree_of_deps)
)
```

## Implementation Plan

### Phase 1: Core Cache Infrastructure (Week 1-2)

**Goal**: Implement basic CAS storage layer

#### 1.1 Cache Storage Module

**File**: `crates/auto-cache/src/lib.rs` (new crate)

```rust
// Core structures
pub struct AutoCache {
    db: SqliteConnection,
    cache_dir: PathBuf,
}

pub struct Artifact {
    pub hash_key: String,      // SHA256 hex
    pub blob_path: PathBuf,
    pub artifact_type: ArtifactType,
    pub file_size: u64,
    pub created_at: u64,
    pub last_used_at: u64,
    pub access_count: u64,
}

pub enum ArtifactType {
    ObjectFile,      // .o/.obj
    StaticLib,       // .a/.lib
    Bytecode,        // .abc (AutoVM)
    CSource,         // .c (transpiled)
    RustSource,      // .rs (transpiled)
}
```

**Key Methods**:
- `AutoCache::new()` - Open/create cache
- `AutoCache::get(hash)` - Retrieve artifact
- `AutoCache::put(hash, file)` - Store artifact
- `AutoCache::contains(hash)` - Check existence
- `AutoCache::remove(hash)` - Delete artifact

#### 1.2 SQLite Schema

**Database**: `crates/auto-cache/src/schema.rs`

```sql
CREATE TABLE artifacts (
    hash_key TEXT PRIMARY KEY,
    blob_path TEXT NOT NULL,
    artifact_type INTEGER,
    file_size INTEGER,
    created_at INTEGER,
    last_used_at INTEGER,
    access_count INTEGER DEFAULT 1
);

CREATE INDEX idx_lru ON artifacts(last_used_at);
CREATE INDEX idx_type ON artifacts(artifact_type);
```

#### 1.3 Blob Storage

**File**: `crates/auto-cache/src/storage.rs`

```rust
pub struct BlobStore {
    root_dir: PathBuf,
}

impl BlobStore {
    // Convert hash to blob path: "a1b2..." -> "blobs/a1/a1b2..."
    pub fn hash_to_path(&self, hash: &str) -> PathBuf;

    // Atomic write: write to temp, then rename
    pub fn put(&self, hash: &str, src: PathBuf) -> Result<PathBuf>;

    // Check if blob exists
    pub fn contains(&self, hash: &str) -> bool;

    // Get blob path (for hard linking)
    pub fn get(&self, hash: &str) -> Option<PathBuf>;

    // Delete blob
    pub fn remove(&self, hash: &str) -> Result<()>;
}
```

**Dependencies**:
- `rusqlite` - SQLite bindings
- `blake3` - Fast hashing (already in workspace)

#### 1.4 Tests

- Test cache creation and initialization
- Test blob storage and retrieval
- Test SQLite CRUD operations
- Test concurrent access

---

### Phase 2: Fingerprint Computation (Week 2-3)

**Goal**: Implement AST-based hashing with context awareness

#### 2.1 Content Hash (AST-Level)

**File**: `crates/auto-cache/src/fingerprint.rs`

```rust
pub struct ContentHasher {
    // Path remapping: /User/dev/proj -> {ROOT}
    path_mappings: HashMap<String, String>,
}

impl ContentHasher {
    // Hash AST (format-independent)
    pub fn hash_ast(&self, ast: &Ast) -> [u8; 32] {
        // Serialize AST to canonical form
        // Exclude comments, whitespace
        // Remap absolute paths to relative
        let canonical = self.canonicalize_ast(ast);
        blake3::hash(&canonical).into()
    }

    // Remap paths in AST
    fn canonicalize_ast(&self, ast: &Ast) -> Vec<u8>;
}
```

**Key Features**:
- Path remapping: `/home/dev/project/src/lib.at` → `{ROOT}/src/lib.at`
- Format normalization: ignore whitespace, comments
- Serialization: AST → binary format

#### 2.2 Context Hash

```rust
pub struct ContextHash {
    pub target_triple: String,      // x86_64-linux-gnu
    pub opt_level: String,           // -O2, -O3
    pub debug_symbols: bool,         // -g
    pub compiler_version: String,    // Auto 0.1.0
    pub c_compiler_version: String,  // GCC 12.1
    pub capabilities: HashSet<String>, // fs=false, etc.
}

impl ContextHash {
    pub fn compute(&self) -> [u8; 32] {
        // Serialize context to binary
        // Hash with blake3
    }
}
```

#### 2.3 Dependency Hash (Merkle Tree)

```rust
pub struct DepHasher {
    // Cache of computed hashes
    hash_cache: HashMap<String, [u8; 32]>,
}

impl DepHasher {
    // Compute Merkle root of dependency tree
    pub fn compute_dep_hash(&self, deps: &[Dependency]) -> [u8; 32] {
        // Recursively hash dependencies
        // Build Merkle tree
        // Return root hash
    }
}
```

#### 2.4 TargetHash Integration

```rust
pub struct TargetHash {
    pub content_hash: [u8; 32],
    pub context_hash: [u8; 32],
    pub dep_hash: [u8; 32],
}

impl TargetHash {
    pub fn compute(&self) -> String {
        // Combine all three hashes
        let combined = [
            &self.content_hash[..],
            &self.context_hash[..],
            &self.dep_hash[..],
        ].concat();

        // Return hex string
        hex_encode(blake3::hash(&combined))
    }
}
```

---

### Phase 3: AutoMan Integration (Week 3-4)

**Goal**: Integrate AutoCache into AutoMan build process

#### 3.1 AutoMan Cache Commands

**File**: `crates/auto-man/src/cache.rs` (new module)

```rust
use auto_cache::AutoCache;

pub struct AutoManCache {
    cache: AutoCache,
}

impl AutoManCache {
    // Query cache before compilation
    pub fn query_module(&self, module: &Module) -> Option<PathBuf> {
        // 1. Compute TargetHash
        // 2. Query AutoCache
        // 3. If hit, create hard link to output
        // 4. Update last_used_at
    }

    // Store artifact after compilation
    pub fn store_module(&self, module: &Module, artifact: PathBuf) {
        // 1. Compute TargetHash
        // 2. Move artifact to cache
        // 3. Update SQLite
    }
}
```

#### 3.2 Build Process Integration

**File**: `crates/auto-man/src/build.rs` (modify)

```rust
impl Builder {
    pub fn build_module(&mut self, module: &Module) -> Result<PathBuf> {
        // BEFORE: Direct compilation
        // let output = compile_module(module);

        // AFTER: Check cache first
        if let Some(cached) = self.cache.query_module(module) {
            println!("[Cache Hit] {} (Hash: {})", module.name, hash);
            return Ok(cached);
        }

        // Cache miss - compile
        println!("[Cache Miss] {}", module.name);
        let output = self.compile_module(module)?;

        // Store in cache
        self.cache.store_module(module, output.clone());

        Ok(output)
    }
}
```

#### 3.3 CLI Commands

**File**: `crates/auto-man/src/main.rs` (modify)

```rust
// Cache commands
auto cache inspect <module>     # Show hash details
auto cache prune                 # Run GC
auto cache stats                 # Show cache statistics
auto cache clear                 # Clear all cache
```

---

### Phase 4: Transpiler Integration (Week 4-5)

**Goal**: Integrate with a2c and a2r transpilers

#### 4.1 a2c Integration

**File**: `crates/auto-lang/src/trans/c_cache.rs` (new module)

```rust
pub struct A2CCache {
    cache: AutoCache,
}

impl A2CCache {
    // Check if C transpilation is cached
    pub fn get_transpiled(&self, at_source: &Ast) -> Option<String> {
        let hash = self.compute_hash(at_source);
        self.cache.get(&hash)
    }

    // Store transpiled C code
    pub fn store_transpiled(&self, at_source: &Ast, c_code: &str) {
        let hash = self.compute_hash(at_source);
        self.cache.put(&hash, c_code.as_bytes())
    }
}
```

**Integration**:
```rust
// In trans/c.rs
pub fn trans_c_cached(module: &Module) -> Result<String> {
    let cache = A2CCache::new();

    // Check cache
    if let Some(c_code) = cache.get_transpiled(&module.ast) {
        return Ok(c_code);
    }

    // Transpile
    let c_code = trans_c(&module.ast)?;

    // Store
    cache.store_transpiled(&module.ast, &c_code);

    Ok(c_code)
}
```

#### 4.2 a2r Integration

Similar to a2c, for Rust transpilation.

#### 4.3 AutoVM Integration

**File**: `crates/auto-lang/src/vm/bytecode_cache.rs` (new module)

```rust
pub struct BytecodeCache {
    cache: AutoCache,
}

impl BytecodeCache {
    // Check if bytecode is cached
    pub fn get_bytecode(&self, module: &Module) -> Option<Vec<u8>> {
        let hash = self.compute_hash(module);
        self.cache.get(&hash)
    }

    // Store compiled bytecode
    pub fn store_bytecode(&self, module: &Module, bytecode: &[u8]) {
        let hash = self.compute_hash(module);
        self.cache.put(&hash, bytecode)
    }
}
```

**Integration**:
```rust
// In vm/codegen.rs
pub fn compile_cached(module: &Module) -> Result<Vec<u8>> {
    let cache = BytecodeCache::new();

    if let Some(bytecode) = cache.get_bytecode(module) {
        return Ok(bytecode);
    }

    // Compile
    let bytecode = compile(module)?;

    // Store
    cache.store_bytecode(module, &bytecode);

    Ok(bytecode)
}
```

---

### Phase 5: Garbage Collection (Week 5)

**Goal**: Implement LRU-based cache cleanup

#### 5.1 GC Strategy

**File**: `crates/auto-cache/src/gc.rs` (new module)

```rust
pub struct GarbageCollector {
    cache: AutoCache,
    max_size_gb: u64,      // Default: 10GB
    water_mark_gb: u64,    // Default: 8GB
}

impl GarbageCollector {
    // Check if GC needed
    pub fn should_gc(&self) -> bool {
        self.current_size() > self.max_size_gb
    }

    // Run LRU garbage collection
    pub fn run_gc(&self) -> Result<usize> {
        let target_free = self.current_size() - self.water_mark_gb;

        // Query oldest artifacts
        let victims = self.query_lru_victims(target_free);

        // Delete files and DB records
        let freed = self.delete_artifacts(victims)?;

        // Vacuum database
        self.vacuum_db()?;

        Ok(freed)
    }

    // Get current cache size
    fn current_size(&self) -> u64;

    // Query LRU victims
    fn query_lru_victims(&self, target_bytes: u64) -> Vec<String>;

    // Delete artifacts
    fn delete_artifacts(&self, hashes: Vec<String>) -> Result<usize>;
}
```

#### 5.2 AutoMan Integration

```rust
// Run GC after build if needed
impl Builder {
    pub fn build(&mut self) -> Result<()> {
        // Build project
        self.build_all()?;

        // Check cache size
        if self.cache.gc.should_gc() {
            println!("Running cache GC...");
            let freed = self.cache.gc.run_gc()?;
            println!("Freed {} MB", freed / 1024 / 1024);
        }

        Ok(())
    }
}
```

---

### Phase 6: Hard Link Optimization (Week 5-6)

**Goal**: Use hard links for zero-copy cache hits

#### 6.1 Hard Link Manager

**File**: `crates/auto-cache/src/link.rs` (new module)

```rust
pub struct LinkManager;

impl LinkManager {
    // Create hard link from cache to output
    pub fn link_from_cache(
        &self,
        cache_path: &PathBuf,
        output_path: &PathBuf,
    ) -> Result<()> {
        // std::fs::hard_link(cache_path, output_path)
    }

    // Fall back to copy if hard link fails (cross-device)
    pub fn copy_from_cache(
        &self,
        cache_path: &PathBuf,
        output_path: &PathBuf,
    ) -> Result<()> {
        // std::fs::copy(cache_path, output_path)
    }
}
```

#### 6.2 Integration

```rust
impl AutoManCache {
    pub fn query_module(&self, module: &Module) -> Option<PathBuf> {
        if let Some(blob_path) = self.cache.get(&hash) {
            // Try hard link first
            let output = self.get_output_path(module);

            match LinkManager::link_from_cache(&blob_path, &output) {
                Ok(_) => return Some(output),
                Err(_) => {
                    // Cross-device: fall back to copy
                    LinkManager::copy_from_cache(&blob_path, &output).ok()?;
                    return Some(output);
                }
            }
        }
        None
    }
}
```

---

### Phase 7: Debugging & Diagnostics (Week 6)

**Goal**: Add debugging tools and commands

#### 7.1 Inspection Commands

```rust
// Show hash computation details
auto cache inspect std.io

// Output:
// Content Hash: a1b2c3d4...
//   - AST nodes: 142
//   - Dependencies: 5
// Context Hash: e5f6g7h8...
//   - Target: x86_64-linux-gnu
//   - Opt: -O2
//   - Compiler: Auto 0.1.0
// Dep Hash: i9j0k1l2...
//   - std.core (a1b2c3d4...)
//   - std.io (e5f6g7h8...)
// Target Hash: m3n4o5p6...
// Cache Status: HIT
//   - Path: blobs/m3/m3n4o5p6...
//   - Size: 45.2 KB
//   - Created: 2 days ago
//   - Used: 5 times
```

#### 7.2 Statistics Command

```rust
auto cache stats

// Output:
// Cache Statistics:
//   Total Artifacts: 1,234
//   Total Size: 4.2 GB / 10 GB
//   Hit Rate: 78.5%
//   - Last 7 days: 8,234 hits / 2,156 misses
//   - Last 24 hours: 1,234 hits / 345 misses
// Artifact Types:
//   - Object Files: 856 (3.2 GB)
//   - Bytecode: 234 (800 MB)
//   - C Source: 144 (200 MB)
// Oldest Artifacts:
//   - std.core (45 days old)
//   - hal.gpio (30 days old)
// Newest Artifacts:
//   - myapp.main (just now)
//   - utils.http (2 hours ago)
```

#### 7.3 Environment Variables

```bash
# Disable cache
export AUTO_NO_CACHE=1

# Set custom cache directory
export AUTO_CACHE_DIR=/mnt/ssd/auto-cache

# Set cache size limit
export AUTO_CACHE_MAX_SIZE=20GB

# Enable verbose logging
export AUTO_CACHE_DEBUG=1
```

---

### Phase 8: Cross-Platform Support (Week 6-7)

**Goal**: Ensure cache works across platforms

#### 8.1 Path Handling

```rust
pub struct PathNormalizer;

impl PathNormalizer {
    // Normalize paths for hash computation
    // Windows: C:\Users\dev\project -> {DRIVE}/Users/dev/project
    // Unix: /home/dev/project -> {ROOT}/home/dev/project

    pub fn normalize_path(&self, path: &Path) -> String {
        // Detect platform
        // Replace absolute paths with tokens
        // Use forward slashes consistently
    }
}
```

#### 8.2 Cross-Platform Context

```rust
pub struct Context {
    pub platform: Platform,
    pub target_triple: String,
    // ...
}

pub enum Platform {
    Linux,
    Windows,
    MacOS,
    MCU,
}

// Platform-specific hashing
impl Context {
    pub fn compute_hash(&self) -> [u8; 32] {
        // Include platform in hash
        // Prevents sharing incompatible artifacts
    }
}
```

---

### Phase 9: Testing & Validation (Week 7)

**Goal**: Comprehensive test coverage

#### 9.1 Unit Tests

- Cache storage operations
- Hash computation accuracy
- Path normalization
- Merkle tree construction
- LRU GC logic

#### 9.2 Integration Tests

- AutoMan + AutoCache workflow
- a2c cache hit/miss scenarios
- AutoVM bytecode caching
- Cross-project artifact sharing
- Concurrent access

#### 9.3 Performance Tests

- Cache lookup speed (target: < 10ms)
- Hash computation speed (target: < 100ms for large projects)
- GC performance (target: < 1s for 1GB cleanup)
- Hard link vs copy performance

---

### Phase 10: Documentation (Week 8)

**Goal**: User-facing documentation

#### 10.1 User Guide

**File**: `docs/guides/autocache-guide.md`

- How cache works
- How to use cache (automatic)
- Cache commands
- Troubleshooting

#### 10.2 Architecture Docs

**File**: `docs/architecture/autocache-implementation.md`

- Storage layer design
- Fingerprint algorithm
- Integration points

#### 10.3 API Documentation

- AutoCache API
- Integration guide for transpilers
- Extending the cache

---

## Implementation Details

### New Crate Structure

```
crates/
└── auto-cache/          # NEW CRATE
    ├── Cargo.toml
    └── src/
        ├── lib.rs        # Main AutoCache API
        ├── schema.rs     # SQLite schema
        ├── storage.rs    # Blob storage
        ├── fingerprint.rs # Hash computation
        ├── gc.rs         # Garbage collection
        └── link.rs       # Hard link management
```

### Modified Files

```
crates/
├── auto-man/
│   └── src/
│       ├── cache.rs     # NEW: AutoManCache integration
│       ├── build.rs     # MODIFY: Add cache queries
│       └── main.rs      # MODIFY: Add CLI commands
└── auto-lang/
    └── src/
        ├── trans/
        │   ├── c_cache.rs        # NEW: a2c cache integration
        │   ├── rust_cache.rs     # NEW: a2r cache integration
        │   └── c.rs              # MODIFY: Use cache
        └── vm/
            ├── bytecode_cache.rs # NEW: Bytecode cache
            └── codegen.rs        # MODIFY: Use cache
```

---

## Success Criteria

✅ **Functional**:
- Cache stores and retrieves artifacts correctly
- Hash computation is deterministic
- Cross-project artifact sharing works
- GC prevents disk overflow

✅ **Performance**:
- Cache lookup: < 10ms
- Hash computation: < 100ms (large projects)
- 80%+ cache hit rate for rebuilds
- Build time reduction: > 50% (after warm-up)

✅ **Reliability**:
- Concurrent access safe (SQLite WAL)
- Cross-platform compatible (Windows, Linux, macOS)
- Handles corrupted cache gracefully
- No stale artifacts served

✅ **Usability**:
- Zero configuration (automatic)
- Clear cache statistics
- Easy debugging
- Simple CLI commands

---

## Migration Path

### For Existing Projects

1. **Phase 1-4**: No changes required (cache is opt-in)
2. **Phase 5**: AutoMan automatically uses cache
3. **Users can**:
   - Disable: `export AUTO_NO_CACHE=1`
   - Clear: `auto cache clear`
   - Configure: Set cache size limits

### Backward Compatibility

- Cache miss = same as no cache (slow but works)
- Old projects work without modification
- No breaking changes to existing APIs

---

## Future Enhancements (Post-Plan 082)

### AutoHub: Remote Cache Server

- HTTP API for artifact sharing
- Team/CI server integration
- Distributed compilation

### Cache Prefetching

- Predict likely dependencies
- Background download of artifacts
- Faster cold starts

### Compression

- Compress blobs before storage
- Reduce disk usage
- Trade-off: CPU vs disk space

---

## Risks & Mitigations

### Risk 1: Hash Collisions

**Mitigation**: SHA-256 has 256-bit space, collision probability negligible

### Risk 2: Cache Poisoning

**Mitigation**: Content-addressable design, verified by hash

### Risk 3: Disk Exhaustion

**Mitigation**: LRU GC with configurable limits

### Risk 4: Cross-Platform Incompatibility

**Mitigation**: Context hash includes platform/target triple

---

## Dependencies

### New Dependencies

```toml
[dependencies]
rusqlite = "0.30"      # SQLite
blake3 = "1.4"         # Already in workspace
dirs = "5.0"           # Already in workspace
```

### Already Available

- `tokio` - Async runtime
- `thiserror` - Error handling
- `serde` - Serialization (for AST hashing)

---

## Timeline

| Phase | Duration | Deliverable |
|-------|----------|-------------|
| 1 | Week 1-2 | Core cache infrastructure |
| 2 | Week 2-3 | Fingerprint computation |
| 3 | Week 3-4 | AutoMan integration |
| 4 | Week 4-5 | Transpiler integration |
| 5 | Week 5 | Garbage collection |
| 6 | Week 5-6 | Hard link optimization |
| 7 | Week 6 | Debugging & diagnostics |
| 8 | Week 6-7 | Cross-platform support |
| 9 | Week 7 | Testing & validation |
| 10 | Week 8 | Documentation |

**Total**: 8 weeks

---

## Next Steps

Ready to proceed with implementation?

**Suggested Start**: Phase 1 - Create auto-cache crate with basic storage layer

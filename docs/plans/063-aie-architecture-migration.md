# Plan 063: AIE Architecture Migration

**Status**: 🚧 In Planning (2025-01-31)
**Priority**: High (Critical for AutoLive)
**Complexity**: High (Multi-phase, 3-6 months)
**Dependencies**: None (can start immediately)

---

## Objective (目标)

Migrate AutoLang from the current **file-based full compilation architecture** to the **AIE (Auto Incremental Engine) query-based incremental compilation architecture** as specified in [docs/design/incremental-compilation.md](../design/incremental-compilation.md).

**Primary Goal**: Enable **AutoLive (亚秒级热重载)** by making only recompile modified functions and their direct dependents, instead of recompiling entire files.

---

## Current Architecture Analysis (现状分析)

### Current Architecture (File-Based Full Compilation)

**Entry Points** ([lib.rs:62-264](../../crates/auto-lang/src/lib.rs)):
```rust
pub fn run(code: &str) -> AutoResult<String>
pub fn parse(code: &str) -> AutoResult<ast::Code>
pub fn trans_c(path: &str) -> AutoResult<String>
pub fn trans_rust(path: &str) -> AutoResult<String>
```

**Compilation Flow**:
```
Source (.at file)
    ↓
Lexer (lexer.rs) → Tokens
    ↓
Parser (parser.rs) → AST (ast.rs)
    ├─→ Side-effects: Modifies Rc<RefCell<Universe>>
    └─→ Symbol registration, scope management
    ↓
├─→ Evaluator (eval.rs) → Value
├─→ C Transpiler (trans/c.rs) → C code
└─→ Rust Transpiler (trans/rust.rs) → Rust code
```

**Key Characteristics**:
- ✅ Simple, predictable execution
- ✅ Easy to debug
- ❌ **No caching**: Every compilation starts from scratch
- ❌ **No granularity**: File-level change = full file recompilation
- ❌ **No dependency tracking**: Can't skip unaffected functions
- ❌ **Stateful Parser**: Parser mutates Universe as side effect

### Current Universe Structure ([universe.rs:129-165](../../crates/auto-lang/src/universe.rs))

```rust
pub struct Universe {
    // Scopes and symbols
    pub scopes: HashMap<Sid, Scope>,
    pub asts: HashMap<Sid, ast::Code>,
    pub code_paks: HashMap<Sid, CodePak>,

    // Values (runtime)
    pub values: HashMap<ValueID, Rc<RefCell<ValueData>>>,

    // Types and specs
    pub types: TypeInfoStore,
    pub type_aliases: HashMap<AutoStr, (Vec<AutoStr>, Type)>,
    pub specs: HashMap<AutoStr, Rc<SpecDecl>>,

    // Symbol locations (for LSP)
    pub symbol_locations: HashMap<AutoStr, SymbolLocation>,

    // Builtins and environment
    pub builtins: HashMap<AutoStr, Value>,
    pub env_vals: HashMap<AutoStr, Box<dyn Any>>,
    pub args: Obj,

    // VM references
    pub vm_refs: HashMap<usize, RefCell<VmRefData>>,
}
```

**Problems for Incremental Compilation**:
1. ❌ **No stable IDs**: Functions don't have persistent IDs across compilations
2. ❌ **No versioning**: Can't detect if function signature changed
3. ❌ **No dependency graph**: Don't know which functions call which
4. ❌ **No caching**: Types, bytecode computed every time
5. ❌ **Monolithic**: Entire file parsed and processed together

---

## Target Architecture (目标架构)

### AIE (Auto Incremental Engine)

**Core Philosophy** ([incremental-compilation.md:20-31](../design/incremental-compilation.md)):
- **From "Process" to "Database"**: Compiler is a persistent database, not a one-shot process
- **Pull Model**: Query engine lazily computes and caches results
- **Declaration-Level Granularity**: Functions, structs, consts are independent units
- **Multi-Level Hashing**: Text → AST → Interface (熔断级联重编)

### New Database Structure

```rust
pub struct Database {
    // =========================================================================
    // LAYER 1: STORAGE (written by Indexer only)
    // =========================================================================

    // Source input
    sources: HashMap<FileId, String>,

    // Parsed artifacts (declaration-level)
    frag_asts: HashMap<FragId, Arc<FnDecl>>,
    frag_meta: HashMap<FragId, FragMeta>,  // name, span, file_id

    // Symbols (stable across compilations)
    symbols: HashMap<Sid, SymbolMeta>,  // name, offset, file_id

    // =========================================================================
    // LAYER 2: CACHE (computed by Query Engine)
    // =========================================================================

    // Derived data (lazy computation)
    types: DashMap<Sid, Type>,
    bytecodes: DashMap<FragId, Blob>,

    // Dependency tracking
    dep_graph: DependencyGraph,  // Map<ProviderID, List<ConsumerID>>

    // Hash chains
    text_hashes: HashMap<FragId, u64>,    // L1: source text hash
    ast_hashes: HashMap<FragId, u64>,     // L2: AST structure hash
    iface_hashes: HashMap<FragId, u64>,   // L3: interface hash (signature)
}
```

### ID System (Stable Identifiers)

```rust
// File-level ID
pub struct FileId(u64);

// Fragment ID (declaration-level)
pub struct FragId {
    file: FileId,
    offset: usize,  // byte offset in file
    generation: u32,  // increments on change
}

// Symbol ID (stable across compilations)
pub struct Sid(String);  // e.g., "main", "math::add", "List.len"
```

### Dependency Graph

```rust
pub struct DependencyGraph {
    // Reverse dependency table: who depends on X?
    reverse_deps: HashMap<ProviderId, Vec<ConsumerId>>,

    // Forward dependency table: what does X depend on?
    forward_deps: HashMap<ConsumerId, Vec<ProviderId>>,
}
```

---

## Migration Strategy (迁移策略)

### Phased Approach

The migration will be executed in **3 phases** to minimize disruption and maintain backward compatibility:

1. **Phase 1**: Structural Refactoring (架构重构) - 4-6 weeks
2. **Phase 2**: File-Level Incremental (文件级增量) - 3-4 weeks
3. **Phase 3**: Fine-Grained Incremental + AutoLive (细粒度增量+热重载) - 6-8 weeks

---

## Phase 1: Structural Refactoring (架构重构)

**Goal**: Eliminate `Rc<RefCell<Universe>>`, establish `Database` skeleton, make parser a pure function.

**Duration**: 4-6 weeks
**Risk**: Medium (affects all compilation paths)

### 1.1 Create Database Module

**File**: `crates/auto-lang/src/database.rs` (new, ~800 lines)

**Tasks**:
1. Define `Database` struct
2. Implement stable ID types (`FileId`, `FragId`, `Sid`)
3. Implement storage layer (sources, frag_asts, symbols)
4. Implement cache layer (types, bytecodes, dep_graph)
5. Add concurrent access support (`DashMap` for cache)

**Acceptance Criteria**:
- [ ] `Database::new()` creates empty database
- [ ] `Database::insert_source()` stores source code
- [ ] `Database::get_frag()` retrieves fragment by ID
- [ ] Thread-safe concurrent reads via `DashMap`

**Example**:
```rust
impl Database {
    pub fn new() -> Self {
        Self {
            sources: HashMap::new(),
            frag_asts: HashMap::new(),
            frag_meta: HashMap::new(),
            symbols: HashMap::new(),
            types: DashMap::new(),
            bytecodes: DashMap::new(),
            dep_graph: DependencyGraph::new(),
        }
    }

    pub fn insert_source(&mut self, path: &str, code: String) -> FileId {
        let id = FileId(self.source_counter);
        self.source_counter += 1;
        self.sources.insert(id, code);
        id
    }

    pub fn get_frag(&self, frag_id: &FragId) -> Option<Arc<FnDecl>> {
        self.frag_asts.get(frag_id).cloned()
    }
}
```

### 1.2 Refactor Parser to Pure Function

**File**: `crates/auto-lang/src/parser.rs` (modify, ~3000 lines)

**Current Behavior**:
```rust
pub fn parse(&mut self) -> AutoResult<ast::Code> {
    // Parser mutates self.scope: Rc<RefCell<Universe>>
    self.scope.define(...);
    self.scope.enter_fn(...);
}
```

**Target Behavior**:
```rust
pub fn parse(&mut self) -> AutoResult<ast::Code> {
    // Parser is now pure: returns AST only
    // No side effects on Universe
}

// New entry point
pub fn parse_to_database(
    code: &str,
    db: &mut Database,
    file_id: FileId,
) -> AutoResult<HashMap<FragId, Arc<ast::FnDecl>>> {
    let mut parser = Parser::new(code);
    let ast = parser.parse()?;

    // Indexer: fragment AST into Database
    let indexer = Indexer::new(db);
    indexer.index_ast(ast, file_id)?;

    Ok(/* fragments */)
}
```

**Tasks**:
1. Remove all `self.scope.define()` calls from parser
2. Remove `self.scope.enter_fn()`, `self.scope.exit_fn()` calls
3. Make parser stateless (no `Rc<RefCell<Universe>>` field)
4. Return pure AST only
5. Create separate `Indexer` to handle symbol registration

**Acceptance Criteria**:
- [ ] Parser produces same AST as before
- [ ] No `Rc<RefCell<Universe>>` in Parser struct
- [ ] All existing tests pass (`cargo test -p auto-lang parser`)

### 1.3 Implement Indexer

**File**: `crates/auto-lang/src/indexer.rs` (new, ~600 lines)

**Responsibilities**:
1. **Resilient Parsing**: Scan source code for function/struct boundaries
2. **Fragmenting**: Split code into independent `Frag` units
3. **Registration**: Assign stable IDs, store in Database

**Example**:
```rust
pub struct Indexer<'db> {
    db: &'db mut Database,
}

impl<'db> Indexer<'db> {
    pub fn index_ast(
        &mut self,
        ast: ast::Code,
        file_id: FileId,
    ) -> AutoResult<Vec<FragId>> {
        let mut frag_ids = Vec::new();

        for item in ast.items {
            match item {
                ast::TopLevelFn(fn_decl) => {
                    let frag_id = FragId::new(file_id, fn_decl.span.offset);
                    self.db.frag_asts.insert(frag_id.clone(), Arc::new(fn_decl));
                    frag_ids.push(frag_id);
                }
                ast::TopLevelStruct(struct_decl) => {
                    // Same for structs
                }
                // ...
            }
        }

        Ok(frag_ids)
    }
}
```

**Acceptance Criteria**:
- [ ] Indexer processes all top-level declarations
- [ ] Fragments are stored in Database
- [ ] Stable IDs generated correctly

### 1.4 Migrate Entry Points

**Files**: `crates/auto-lang/src/lib.rs` (modify, ~270 lines)

**Current**:
```rust
pub fn trans_c(path: &str) -> AutoResult<String> {
    let scope = Rc::new(RefCell::new(Universe::new()));
    let mut parser = Parser::new(code.as_str(), scope);
    let ast = parser.parse()?;
    // ...
}
```

**Target**:
```rust
pub fn trans_c(path: &str) -> AutoResult<String> {
    let mut db = Database::new();
    let code = std::fs::read_to_string(path)?;
    let file_id = db.insert_source(path, code);

    // Pure parsing + indexing
    let frag_ids = parse_to_database(&code, &mut db, file_id)?;

    // Query-based transpilation
    let mut sink = Sink::new(...);
    let mut trans = CTrans::new(&db);

    for frag_id in frag_ids {
        let fn_decl = db.get_frag(&frag_id).unwrap();
        trans.trans_fn(&fn_decl, &mut sink)?;
    }

    // ...
}
```

**Tasks**:
1. Update `trans_c()` to use Database
2. Update `trans_rust()` to use Database
3. Update `run()` to use Database
4. Update `parse()` to use Database
5. Keep `Universe` for evaluator runtime (Phase 2)

**Acceptance Criteria**:
- [ ] All transpilation tests pass (`cargo test -p auto-lang -- trans`)
- [ ] All evaluation tests pass (`cargo test -p auto-lang eval`)
- [ ] No regressions in existing functionality

### 1.5 Separate Runtime from Compilation

**File**: `crates/auto-lang/src/runtime.rs` (new, ~400 lines)

**Rationale**: The current `Universe` serves two purposes:
1. **Symbol table** (compile-time): Type definitions, function signatures
2. **Runtime values** (execution-time): Variable values, heap allocation

These should be **separated**:
- **Database**: Compile-time symbols (persistent)
- **Runtime**: Execution values (ephemeral, per-run)

**Design**:
```rust
// Compile-time: Database
pub struct Database {
    pub symbols: HashMap<Sid, SymbolMeta>,
    pub types: HashMap<Sid, Type>,
    // ...
}

// Runtime: ExecutionEngine
pub struct ExecutionEngine {
    pub values: HashMap<ValueID, Rc<RefCell<ValueData>>>,
    pub call_stack: Vec<CallFrame>,
    pub heap: Heap,
}
```

**Tasks**:
1. Extract runtime-related fields from `Universe` to `ExecutionEngine`
2. Keep `Universe` as alias to `ExecutionEngine` for backward compatibility
3. Update evaluator to use `ExecutionEngine`

**Acceptance Criteria**:
- [ ] `ExecutionEngine` contains only runtime state
- [ ] `Database` contains only compile-time state
- [ ] Evaluator uses `ExecutionEngine` correctly
- [ ] No cross-contamination between compile-time and runtime

---

## Phase 2: File-Level Incremental (文件级增量)

**Goal**: When a file is modified, only recompile that file, not its dependencies (unless imports changed).

**Duration**: 3-4 weeks
**Risk**: Low-Medium (new feature, doesn't break existing workflows)

### 2.1 Implement File Hashing

**File**: `crates/auto-lang/src/database.rs` (extend, +200 lines)

**Tasks**:
1. Add `text_hashes: HashMap<FileId, u64>` to Database
2. Implement `Database::hash_file()` using BLAKE3
3. Add `Database::is_file_dirty()` to detect changes

**Example**:
```rust
impl Database {
    pub fn hash_file(&mut self, file_id: FileId) -> u64 {
        let code = self.sources.get(&file_id).unwrap();
        let hash = blake3::hash(code.as_bytes());
        let hash_u64 = u64::from_le_bytes(*hash.as_bytes().get(..8).unwrap());

        self.text_hashes.insert(file_id, hash_u64);
        hash_u64
    }

    pub fn is_file_dirty(&self, file_id: FileId, new_hash: u64) -> bool {
        match self.text_hashes.get(&file_id) {
            Some(old_hash) => old_hash != &new_hash,
            None => true,  // new file
        }
    }
}
```

**Acceptance Criteria**:
- [ ] File hashing implemented with BLAKE3
- [ ] Dirty detection works correctly
- [ ] Added to Database structure

### 2.2 Implement File Dependency Graph

**File**: `crates/auto-lang/src/depgraph.rs` (new, ~400 lines)

**Tasks**:
1. Define `FileDependencyGraph` struct
2. Track `use`/import relationships between files
3. Implement dirty propagation (if A imports B, and B changes, A is dirty)

**Example**:
```rust
pub struct FileDependencyGraph {
    // Which files does this file import?
    imports: HashMap<FileId, Vec<FileId>>,

    // Which files import this file? (reverse)
    imported_by: HashMap<FileId, Vec<FileId>>,
}

impl FileDependencyGraph {
    pub fn mark_imports(&mut self, file: FileId, imports: Vec<FileId>) {
        for imported in &imports {
            self.imported_by
                .entry(*imported)
                .or_insert_with(Vec::new)
                .push(file);
        }
        self.imports.insert(file, imports);
    }

    pub fn get_dependents(&self, file: FileId) -> Vec<FileId> {
        self.imported_by.get(&file).cloned().unwrap_or_default()
    }
}
```

**Acceptance Criteria**:
- [ ] `use` statements tracked during parsing
- [ ] Dependency graph built correctly
- [ ] Can query which files depend on a given file

### 2.3 Incremental Re-Indexing

**File**: `crates/auto-lang/src/indexer.rs` (extend, +300 lines)

**Tasks**:
1. Implement `Indexer::reindex_file()` to re-parse only changed files
2. Clear cache for dirty files
3. Trigger recompilation of dependents

**Example**:
```rust
impl Indexer<'_> {
    pub fn reindex_file(
        &mut self,
        file_id: FileId,
        new_code: String,
    ) -> AutoResult<Vec<FragId>> {
        // Check if file actually changed
        let new_hash = hash_code(&new_code);
        if !self.db.is_file_dirty(file_id, new_hash) {
            return Ok(vec![]);  // No change
        }

        // Update source
        self.db.sources.insert(file_id, new_code.clone());

        // Remove old fragments for this file
        self.db.clear_file_frags(file_id);

        // Re-parse
        let ast = parse_code(&new_code)?;
        let frag_ids = self.index_ast(ast, file_id)?;

        // Mark dependents as dirty
        let dependents = self.db.dep_graph.get_dependents(file_id);
        for dep in dependents {
            self.db.mark_file_dirty(dep);
        }

        Ok(frag_ids)
    }
}
```

**Acceptance Criteria**:
- [ ] Changed files are re-indexed
- [ ] Unchanged files are skipped
- [ ] Dependent files marked dirty

### 2.4 Query Engine Prototype

**File**: `crates/auto-lang/src/query.rs` (new, ~500 lines)

**Tasks**:
1. Define `Query` trait
2. Implement `QueryEngine` with caching
3. Add queries: `get_type()`, `get_bytecode()`, `get_deps()`

**Example**:
```rust
pub trait Query: Send + Sync {
    type Output;

    fn execute(&self, db: &Database) -> AutoResult<Self::Output>;
    fn cache_key(&self) -> String;
}

pub struct QueryEngine {
    db: Arc<Database>,
    cache: DashMap<String, CacheEntry>,
}

impl QueryEngine {
    pub fn execute<Q: Query>(&self, query: &Q) -> AutoResult<Q::Output> {
        let key = query.cache_key();

        // Check cache
        if let Some(entry) = self.cache.get(&key) {
            if !entry.is_dirty() {
                return Ok(entry.value.clone());
            }
        }

        // Execute query
        let result = query.execute(&self.db)?;

        // Cache result
        self.cache.insert(key, CacheEntry::new(result.clone()));

        Ok(result)
    }
}
```

**Acceptance Criteria**:
- [ ] Query engine implemented
- [ ] Caching works correctly
- [ ] Cache invalidation on file changes

### 2.5 Testing Infrastructure

**File**: `crates/auto-lang/test/incremental/` (new directory)

**Tests**:
1. `test_file_no_change()` - No recompilation if file unchanged
2. `test_file_changed()` - Only changed file recompiled
3. `test_import_chain()` - A imports B, B changes → A recompiled
4. `test_import_diamond()` - A,B import C, C changes → A,B recompiled

**Example**:
```rust
#[test]
fn test_file_no_change() {
    let mut db = Database::new();
    let file_id = db.insert_source("test.at", "fn main() int { 42 }");
    let hash1 = db.hash_file(file_id);

    // Re-index same content
    let frags = indexer.reindex_file(file_id, "fn main() int { 42 }".to_string()).unwrap();

    // Should not recompile
    assert!(frags.is_empty());
}
```

**Acceptance Criteria**:
- [ ] All incremental tests pass
- [ ] Performance tests show speedup for unchanged files

---

## Phase 3: Fine-Grained Incremental + AutoLive (细粒度增量+热重载)

**Goal**: Function-level incremental compilation, patch generation, MCU hot reload integration.

**Duration**: 6-8 weeks
**Risk**: High (complex integration with runtime)

### 3.1 Fragment-Level Hashing

**File**: `crates/auto-lang/src/hash.rs` (new, ~600 lines)

**Tasks**:
1. Implement L1 Text Hash (source text hash)
2. Implement L2 AST Hash (structure hash, ignoring comments/formatting)
3. Implement L3 Interface Hash (signature hash only)

**Example**:
```rust
pub struct FragmentHasher;

impl FragmentHasher {
    // L1: Did source text change?
    pub fn hash_text(frag: &FnDecl) -> u64 {
        let source = frag.span.source_text();
        blake3::hash(source.as_bytes()).into()
    }

    // L2: Did AST structure change?
    pub fn hash_ast(frag: &FnDecl) -> u64 {
        let mut hasher = blake3::Hasher::new();
        hash_fn_decl_structure(&mut hasher, frag);
        hasher.finalize().into()
    }

    // L3: Did signature (interface) change?
    pub fn hash_interface(frag: &FnDecl) -> u64 {
        let mut hasher = blake3::Hasher::new();
        // Hash only: name, params, return type
        hasher.update(frag.name.as_bytes());
        for param in &frag.params {
            hasher.update(param.ty.to_string().as_bytes());
        }
        hasher.update(frag.return_type.to_string().as_bytes());
        hasher.finalize().into()
    }
}
```

**Acceptance Criteria**:
- [ ] L1 hash detects any text change
- [ ] L2 hash ignores comment/formatting changes
- [ ] L3 hash detects only signature changes

### 3.2 Interface Hash熔断

**File**: `crates/auto-lang/src/depgraph.rs` (extend, +400 lines)

**Concept**: If a function's body changes but signature doesn't, dependents don't need to recompile.

**Tasks**:
1. Build reverse dependency graph at **fragment level**
2. On fragment change, compute L3 (Interface Hash)
3. If Interface Hash unchanged → **熔断** (stop propagation)
4. If Interface Hash changed → propagate to dependents

**Example**:
```rust
impl DependencyGraph {
    pub fn invalidate_fragment(
        &mut self,
        frag_id: &FragId,
        old_iface_hash: u64,
        new_iface_hash: u64,
    ) -> Vec<FragId> {
        // If signature didn't change,熔断propagation
        if old_iface_hash == new_iface_hash {
            return vec![];  // No dependents need recompilation
        }

        // Signature changed → propagate to all dependents
        self.get_dependents(frag_id)
    }
}
```

**Acceptance Criteria**:
- [ ] Function body change doesn't trigger dependent recompilation
- [ ] Function signature change triggers dependent recompilation
- [ ]熔断works correctly in all test cases

### 3.3 Fragment-Level Dependency Tracking

**File**: `crates/auto-lang/src/depgraph.rs` (extend, +500 lines)

**Tasks**:
1. Track which functions call which other functions
2. Track which functions use which types
3. Build fine-grained dependency graph

**Algorithm**:
```rust
pub fn build_fn_dependencies(
    fn_decl: &FnDecl,
    db: &Database,
) -> Vec<FragId> {
    let mut deps = Vec::new();

    // Scan function body for function calls
    for expr in walk_exprs(&fn_decl.body) {
        if let Expr::Call(call) = expr {
            if let Some(callee) = db.resolve_fn(&call.name) {
                deps.push(callee.frag_id);
            }
        }
    }

    deps
}
```

**Acceptance Criteria**:
- [ ] All function calls tracked
- [ ] All type usages tracked
- [ ] Dependency graph accurate

### 3.4 Incremental Query Engine

**File**: `crates/auto-lang/src/query.rs` (extend, +400 lines)

**Tasks**:
1. Implement dirty flag propagation
2. Implement lazy recomputation
3. Implement query result caching

**Example**:
```rust
pub struct TypeQuery {
    pub frag_id: FragId,
}

impl Query for TypeQuery {
    type Output = Type;

    fn execute(&self, db: &Database) -> AutoResult<Self::Output> {
        let fn_decl = db.get_frag(&self.frag_id).unwrap();
        typeck::infer_fn_type(fn_decl, db)
    }

    fn cache_key(&self) -> String {
        format!("type({})", self.frag_id)
    }
}
```

**Acceptance Criteria**:
- [ ] Type query cached correctly
- [ ] Bytecode query cached correctly
- [ ] Dirty flags trigger recomputation

### 3.5 Patch Generation

**File**: `crates/auto-lang/src/patch.rs` (new, ~800 lines)

**Tasks**:
1. Define `Patch` structure (frag_id, code_size, code[], relocs[])
2. Implement codegen to generate patches instead of full binaries
3. Implement relocation table generation

**Example**:
```rust
#[derive(Debug, Clone)]
pub struct Patch {
    pub frag_id: FragId,
    pub code_size: u32,
    pub code: Vec<u8>,
    pub relocs: Vec<Reloc>,
}

#[derive(Debug, Clone)]
pub struct Reloc {
    pub offset: u32,      // Offset in code
    pub symbol: Sid,      // Symbol to resolve
    pub kind: RelocKind,  // Abs, Rel, GOT, PLT
}

pub fn generate_patch(
    fn_decl: &FnDecl,
    db: &Database,
) -> AutoResult<Patch> {
    // Generate bytecode for this function only
    let mut codegen = Codegen::new(db);
    let (code, relocs) = codegen.compile_fn(fn_decl)?;

    Ok(Patch {
        frag_id: fn_decl.frag_id,
        code_size: code.len() as u32,
        code,
        relocs,
    })
}
```

**Acceptance Criteria**:
- [ ] Patch structure defined
- [ ] Codegen generates patches
- [ ] Relocation tables correct

### 3.6 MCU Runtime Integration (AutoLive)

**File**: `crates/auto-lang/src/autolive.rs` (new, ~600 lines)

**Tasks**:
1. Design **RAM Overlay** system (hot reload zone in MCU memory)
2. Implement **GOT (Global Offset Table)** updates
3. Implement debugger protocol for patch injection

**RAM Overlay Design**:
```
MCU Memory Layout:
┌─────────────────────────────────┐
│  Fixed Code (Flash)             │  ← Bootloader, core runtime
├─────────────────────────────────┤
│  GOT (Global Offset Table)      │  ← Function pointers
├─────────────────────────────────┤
│  Hot Zone (RAM)                 │  ← Reloadable patches
│  - Patch 1: main()              │
│  - Patch 2: calculate()         │
│  - Patch 3: render()            │
└─────────────────────────────────┘
```

**Example**:
```rust
pub struct AutoLiveRuntime {
    got: HashMap<Sid, *const u8>,  // Function pointers
    hot_zone: *mut u8,             // RAM region for patches
    hot_zone_size: usize,
}

impl AutoLiveRuntime {
    pub fn apply_patch(&mut self, patch: &Patch) -> AutoResult<()> {
        // Write patch to hot zone
        let offset = self.allocate_hot_slot(patch.code_size);
        unsafe {
            ptr::copy_nonoverlapping(
                patch.code.as_ptr(),
                self.hot_zone.add(offset),
                patch.code_size as usize,
            );
        }

        // Update GOT entry
        let new_addr = self.hot_zone.add(offset);
        self.got.insert(patch.frag_id.sid(), new_addr);

        // Resolve relocations
        for reloc in &patch.relocs {
            self.resolve_reloc(reloc, offset)?;
        }

        Ok(())
    }
}
```

**Acceptance Criteria**:
- [ ] RAM overlay allocated correctly
- [ ] Patches applied without reboot
- [ ] GOT updated correctly
- [ ] Relocations resolved

### 3.7 Debugger Protocol

**File**: `crates/auto-lang/src/debugger.rs` (new, ~500 lines)

**Tasks**:
1. Define wire protocol for patch transmission
2. Implement CRC checking
3. Implement rollback on error

**Protocol Design**:
```
Message Format:
[Cmd:1][Len:4][Payload:Len][CRC:4]

Commands:
- 0x01: PATCH_BEGIN (frag_id:8, size:4)
- 0x02: PATCH_DATA (chunk_idx:2, data:var)
- 0x03: PATCH_COMMIT (expected_crc:4)
- 0x04: PATCH_ROLLBACK
- 0x05: GET_STATUS

Response:
[Status:1][CRC:4]
```

**Acceptance Criteria**:
- [ ] Protocol defined
- [ ] Transmission reliable
- [ ] Rollback works on errors

### 3.8 End-to-End Testing

**File**: `crates/auto-lang/test/autolive/` (new directory)

**Tests**:
1. `test_fn_body_change()` - Body change → new patch, dependents unchanged
2. `test_fn_sig_change()` - Signature change → patch + dependents recompiled
3. `test_hot_reload()` - Apply patch to running MCU
4. `test_rollback()` - Failed patch rollback

**Example**:
```rust
#[test]
fn test_fn_body_change() {
    let mut db = Database::new();
    // ... setup ...

    // Change function body (not signature)
    let new_frag = parse_fn("fn add(a int, b int) int { a + b + 1 }");
    let patch = generate_patch(&new_frag, &db).unwrap();

    // Old dependents should not recompile
    let dependents = db.dep_graph.get_dependents(&frag_id);
    assert!(dependents.is_empty());
}
```

**Acceptance Criteria**:
- [ ] All AutoLive tests pass
- [ ] Hot reload works on real hardware
- [ ] Sub-second reload achieved

---

## Success Criteria (成功标准)

### Phase 1 Success (Architecture Refactoring)

- ✅ No `Rc<RefCell<Universe>>` in Parser
- ✅ Database module implemented
- ✅ All existing tests pass
- ✅ Parser is pure function
- ✅ Indexer handles symbol registration

### Phase 2 Success (File-Level Incremental)

- ✅ File hashing implemented
- ✅ File dependency graph working
- ✅ Unchanged files not recompiled
- ✅ Import tracking works
- ✅ Query engine prototype working

### Phase 3 Success (Fine-Grained + AutoLive)

- ✅ Fragment-level hashing (L1/L2/L3)
- ✅ Interface hash熔断works
- ✅ Function-level dependency graph
- ✅ Patch generation working
- ✅ MCU hot reload working
- ✅ **亚秒级热重载 achieved (<1s)**

---

## Risk Mitigation (风险控制)

### Technical Risks

| Risk | Impact | Probability | Mitigation |
|------|--------|------------|------------|
| **Parser refactoring breaks tests** | High | Medium | Incremental refactoring, extensive testing |
| **Dependency graph inaccurate** | High | Medium | Formal verification, cross-check with static analysis |
| **MCU memory constraints** | High | Low | Careful memory planning, compression |
| **Hot reload crashes MCU** | Critical | Low | Rollback mechanism, CRC checking |
| **Performance regression** | Medium | Low | Benchmarking, profiling |

### Migration Risks

| Risk | Impact | Probability | Mitigation |
|------|--------|------------|------------|
| **Long migration blocks features** | Medium | Medium | Phased approach, backward compatibility |
| **Team unfamiliar with architecture** | Medium | Medium | Documentation, pair programming |
| **Incomplete feature parity** | High | Low | Comprehensive test coverage |

---

## Performance Targets (性能目标)

### Baseline (Current)

- Full file recompilation: **2-5s** for typical project
- No incremental support

### Phase 1 (Architecture Refactoring)

- Full recompilation: **3-5s** (slower due to new overhead)
- Expected: 10-20% slower initially

### Phase 2 (File-Level Incremental)

- Unchanged file: **<100ms** (hash check only)
- Changed file: **2-3s** (same file)
- Import chain: **3-6s** (propagates)

### Phase 3 (Fine-Grained + AutoLive)

- Unchanged function: **<10ms** (cache hit)
- Function body change: **<100ms** (patch gen)
- Function signature change: **<500ms** (with dependents)
- **Hot reload: <1s** (target achieved)

---

## Resource Estimation (资源估算)

### Human Resources

- **Phase 1**: 1 senior engineer, 4-6 weeks
- **Phase 2**: 1 engineer, 3-4 weeks
- **Phase 3**: 1-2 senior engineers, 6-8 weeks
- **Total**: 13-18 engineer-weeks (~4-5 months calendar time)

### Code Estimates

| Phase | New Code | Modified Code | Tests |
|-------|----------|---------------|-------|
| Phase 1 | ~2,000 lines | ~1,000 lines | ~1,000 lines |
| Phase 2 | ~1,500 lines | ~500 lines | ~800 lines |
| Phase 3 | ~3,000 lines | ~1,500 lines | ~1,500 lines |
| **Total** | **~6,500 lines** | **~3,000 lines** | **~3,300 lines** |

### Dependencies

- `blake3` - Hashing (already in use)
- `dashmap` - Concurrent hashmap (already in use)
- `serde` - Serialization (for patch protocol)
- `tokio` - Async I/O (for debugger protocol)

---

## Timeline (时间线)

```
Week 1-2:  Phase 1.1-1.2 (Database + Parser Refactoring)
Week 3-4:  Phase 1.3-1.4 (Indexer + Entry Points)
Week 5-6:  Phase 1.5 (Runtime Separation) + Testing
            ↓ Phase 1 Complete

Week 7-8:  Phase 2.1-2.2 (File Hashing + Dep Graph)
Week 9-10: Phase 2.3-2.4 (Re-indexing + Query Engine)
Week 11:   Phase 2.5 (Testing) + Validation
            ↓ Phase 2 Complete

Week 12-13: Phase 3.1-3.2 (Fragment Hashing + 熔断)
Week 14-15: Phase 3.3-3.4 (Dep Tracking + Query Engine)
Week 16-17: Phase 3.5-3.6 (Patch Gen + MCU Runtime)
Week 18-19: Phase 3.7-3.8 (Debugger + Testing)
            ↓ Phase 3 Complete
            ↓ AIE Architecture Complete
```

**Total**: ~4.5 months (18 weeks)

---

## Open Questions (待解决的问题)

1. **Symbol Stability**: How to ensure `FragId` is stable across file edits?
   - **Option A**: Use byte offset (fragments on edit invalidate ID)
   - **Option B**: Use UUID (assign once, keep forever)
   - **Recommendation**: Start with offset, migrate to UUID in Phase 3

2. **Caching Strategy**: In-memory or on-disk cache?
   - **In-memory**: Fast, but lost on restart
   - **On-disk**: Persistent, but slower
   - **Recommendation**: In-memory for Phase 2, add on-disk in Phase 3

3. **MCU Memory Layout**: Fixed-size hot zone or dynamic allocation?
   - **Fixed**: Simple, but limited
   - **Dynamic**: Flexible, but requires allocator
   - **Recommendation**: Fixed-size hot zone (64KB) for Phase 3

4. **Thread Safety**: Does Database need to be thread-safe?
   - **Single-threaded**: Simpler, faster
   - **Multi-threaded**: Parallel compilation
   - **Recommendation**: Start single-threaded, add locks in Phase 3 if needed

5. **Backward Compatibility**: Keep old API or breaking change?
   - **Keep**: Slower migration, more complexity
   - **Break**: Clean slate, faster
   - **Recommendation**: Breaking change (this is a major architectural shift)

---

## Next Steps (下一步)

1. **Review this plan** with team (Week 1)
2. **Create detailed task breakdown** for Phase 1
3. **Set up CI/CD** for incremental testing
4. **Start Phase 1.1**: Create Database module

---

## References (参考文档)

1. [Incremental Compilation Design](../design/incremental-compilation.md) - Overall AIE architecture
2. [Plan 019: Spec Trait System](019-spec-trait-system.md) - Related trait system work
3. [Plan 052: Storage-Based Lists](052-storage-based-lists.md) - Storage abstraction
4. [Rust Compiler Query System](https://rustc-dev-guide.rust-lang.org/query.html) - Inspiration for query engine
5. [Elsa: Incremental Rust Compiler](https://github.com/elsa-rs/elsa) - Incremental compilation research

---

**Document Status**: Draft v1.0
**Last Updated**: 2025-01-31
**Author**: Claude (AI) + User
**Reviewers**: TBD

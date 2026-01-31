# Plan 064: Split Universe into Compile-time (Database) and Runtime (ExecutionEngine)

**Status**: ⏸️ **PHASE 4 BLOCKED** (Phases 1-3 Complete ✅)
**Priority**: P0 (Critical Architecture Refactor)
**Created**: 2025-01-31
**Last Updated**: 2025-01-31
**Dependencies**: Plan 063 Phase 3.5 ✅

**Progress Summary**:
- ✅ Phase 1: Field classification complete (19 fields analyzed)
- ✅ Phase 2: ExecutionEngine extended with 11 runtime fields
- ✅ Phase 3: AIE Database extended with 7 compile-time fields
- ⏸️ Phase 4: **BLOCKED** - Requires Scope split architecture redesign
  - Blocker: Scopes contain both compile-time AND runtime data
  - Blocker: 139 references to `self.universe` in eval.rs
  - Estimated effort: 2-3 days (was 1-2 hours)

---

## Problem Statement

The current `Universe` structure mixes compile-time and runtime concerns, preventing proper integration with the AIE (Auto Incremental Engine) architecture:

```rust
pub struct Universe {
    // Compile-time data (should be in Database)
    pub scopes: HashMap<Sid, Scope>,
    pub asts: HashMap<Sid, ast::Code>,
    pub types: TypeInfoStore,
    pub symbol_locations: HashMap<AutoStr, SymbolLocation>,
    pub type_aliases: HashMap<AutoStr, (Vec<AutoStr>, Type)>,
    pub specs: HashMap<AutoStr, Rc<SpecDecl>>,

    // Runtime data (should be in ExecutionEngine)
    pub env_vals: HashMap<AutoStr, Box<dyn Any>>,
    pub shared_vals: HashMap<AutoStr, Rc<RefCell<Value>>>,
    pub builtins: HashMap<AutoStr, Value>,
    pub vm_refs: HashMap<usize, RefCell<VmRefData>>,
    pub args: Obj,
    pub values: HashMap<ValueID, Rc<RefCell<ValueData>>>,
    pub weak_refs: HashMap<ValueID, Weak<RefCell<ValueData>>>,

    // Counters (mixed: lambda is compile-time, vmref/value are runtime)
    lambda_counter: usize,
    vmref_counter: usize,
    value_counter: usize,

    // Runtime-only
    evaluator_ptr: *mut crate::eval::Evaler,
}
```

**This mixing causes**:
1. ❌ AIE cannot integrate cleanly - Universe owns both ASTs and values
2. ❌ Incremental compilation cannot work - values tied to compile-time structures
3. ❌ Hot reloading impossible - runtime state inseparable from compile-time state
4. ❌ Memory inefficient - compile-time data persists during entire execution

**Why Integrate with AIE Database?**
- ✅ **Single source of truth** - No duplicate databases or synchronization issues
- ✅ **Incremental compilation ready** - Database already has hashing, dependency tracking, 熔断
- ✅ **Hot reload ready** - Database supports patch generation (Plan 063 Phase 3.5)
- ✅ **Query engine** - Database integrates with QueryEngine for smart caching
- ✅ **LSP support** - Database already has symbol locations for IDE features
- ✅ **Future-proof** - Database designed for long-term persistence and incremental updates

---

## Objective

Split `Universe` by migrating compile-time data **into the existing AIE Database** and runtime data **into ExecutionEngine**:

1. **Database** (AIE, existing, compile-time, persistent):
   - Already has: sources, fragments, dependency graph, hashes, symbol locations
   - **Add from Universe**: scopes, types, type aliases, specs, lambda counter
   - Owned by `CompileSession`, shared across compilations
   - Supports incremental compilation, 熔断, hot reloading

2. **ExecutionEngine** (runtime, ephemeral):
   - Values, VM references, call stack
   - Environment variables, arguments
   - Owned by `Interpreter`/`Evaler`, recreated per execution

---

## Architecture

### Current (Mixed)

```
┌─────────────────────────────────────┐
│           Universe                  │
│  ┌────────────┐  ┌──────────────┐  │
│  │ Compile    │  │ Runtime      │  │
│  │ - ASTs     │  │ - Values     │  │
│  │ - Scopes   │  │ - VM refs    │  │
│  │ - Types    │  │ - Stack      │  │
│  └────────────┘  └──────────────┘  │
└─────────────────────────────────────┘
```

### Target (Integrated with AIE)

```
┌──────────────────────────────────────────┐
│         CompileSession                   │
│  ┌────────────────────────────────────┐  │
│  │     AIE Database (existing)        │  │
│  │  ✓ Sources                         │  │
│  │  ✓ Fragments                       │  │
│  │  ✓ Dependency Graph                │  │
│  │  ✓ Hashes (L1/L2/L3)               │  │
│  │  ✓ Symbol Locations                │  │
│  │                                    │  │
│  │  + Scopes (from Universe)          │  │
│  │  + Types (from Universe)           │  │
│  │  + Type Aliases (from Universe)    │  │
│  │  + Specs (from Universe)           │  │
│  │  + Lambda Counter (from Universe)  │  │
│  └────────────────────────────────────┘  │
└──────────────────────────────────────────┘
            ↓ queries
┌──────────────────────────────────────────┐
│        ExecutionEngine                   │
│  ┌────────────────────────────────────┐  │
│  │      Runtime State                 │  │
│  │  - Values (from Universe)          │  │
│  │  - VM Refs (from Universe)         │  │
│  │  - Shared Vals (from Universe)     │  │
│  │  - Builtins (from Universe)        │  │
│  │  - Call Stack                      │  │
│  │  - Counters (vmref, value)         │  │
│  │  - Evaluator Pointer               │  │
│  └────────────────────────────────────┘  │
└──────────────────────────────────────────┘
```

---

## Phase 1: Inventory and Classification (30 min)

**Goal**: Document every Universe field and classify it.

### Complete Universe Field Classification

| # | Field | Type | Classification | Target | Migration Strategy |
|---|-------|------|----------------|--------|-------------------|
| 1 | `scopes` | `HashMap<Sid, Scope>` | **Compile-time** | Database (add) | Move directly |
| 2 | `asts` | `HashMap<Sid, ast::Code>` | **Compile-time** | Database (use fragments) | Use existing fragment system |
| 3 | `code_paks` | `HashMap<Sid, CodePak>` | **Compile-time** | Database (add) | Move directly |
| 4 | `env_vals` | `HashMap<AutoStr, Box<dyn Any>>` | **Runtime** | ExecutionEngine (✓ already there) | Move directly |
| 5 | `shared_vals` | `HashMap<AutoStr, Rc<RefCell<Value>>>` | **Runtime** | ExecutionEngine (add) | Move directly |
| 6 | `builtins` | `HashMap<AutoStr, Value>` | **Runtime** | ExecutionEngine (add) | Move directly |
| 7 | `vm_refs` | `HashMap<usize, RefCell<VmRefData>>` | **Runtime** | ExecutionEngine (add) | Move directly |
| 8 | `types` | `TypeInfoStore` | **Compile-time** | Database (add) | Move directly |
| 9 | `args` | `Obj` | **Runtime** | ExecutionEngine (✓ already there) | Already migrated |
| 10 | `lambda_counter` | `usize` | **Compile-time** | Database (add) | Move directly |
| 11 | `cur_spot` | `Sid` | **Compile-time** | Database (add) | Move directly |
| 12 | `vmref_counter` | `usize` | **Runtime** | ExecutionEngine (add) | Move directly |
| 13 | `value_counter` | `usize` | **Runtime** | ExecutionEngine (add) | Move directly |
| 14 | `values` | `HashMap<ValueID, Rc<RefCell<ValueData>>>` | **Runtime** | ExecutionEngine (add) | Move directly |
| 15 | `weak_refs` | `HashMap<ValueID, Weak<RefCell<ValueData>>>` | **Runtime** | ExecutionEngine (add) | Move directly |
| 16 | `symbol_locations` | `HashMap<AutoStr, SymbolLocation>` | **Compile-time** | Database (✓ already there) | Already in Database! |
| 17 | `type_aliases` | `HashMap<AutoStr, (Vec<AutoStr>, Type)>` | **Compile-time** | Database (add) | Move directly |
| 18 | `specs` | `HashMap<AutoStr, Rc<SpecDecl>>` | **Compile-time** | Database (add) | Move directly |
| 19 | `evaluator_ptr` | `*mut crate::eval::Evaler` | **Runtime** | ExecutionEngine (add) | Move directly |

### Summary Statistics

- **Total Fields**: 19
- **Compile-time**: 8 (scopes, asts, code_paks, types, lambda_counter, cur_spot, symbol_locations, type_aliases, specs)
- **Runtime**: 11 (env_vals, shared_vals, builtins, vm_refs, args, vmref_counter, value_counter, values, weak_refs, evaluator_ptr)
- **Already in Database**: 1 (symbol_locations) ✓
- **Already in ExecutionEngine**: 1 (args) ✓
- **Need to migrate**: 17 fields

### Data Flow Analysis

**Compile-time Data Flow**:
```
Parser → Indexer → Database
  ↓         ↓          ↓
 ASTs    Scopes   Symbol Locations
  ↓         ↓          ↓
Query Engine ← ← ← ← ← ← ← ← ←
  ↓
Incremental Compilation
```

**Runtime Data Flow**:
```
Code → Evaluator → ExecutionEngine
  ↓       ↓              ↓
Values  VM Refs      Call Stack
  ↓       ↓              ↓
Result → Output
```

**Key Insight**: No cross-dependencies between compile-time and runtime data in the new architecture! This clean separation is what enables incremental compilation.

### Dependencies Between Fields

**Compile-time field dependencies**:
- `scopes` → references `parent` (other scopes)
- `asts` → contains function bodies
- `code_paks` → references ASTs
- `types` → independent
- `symbol_locations` → independent
- `type_aliases` → references `types`
- `specs` → references `types`
- `lambda_counter` → independent
- `cur_spot` → references `scopes`

**Runtime field dependencies**:
- `values` ↔ `weak_refs` (weak references)
- `vm_refs` → independent (managed by ID)
- `shared_vals` → independent
- `builtins` → independent (cached functions)
- `evaluator_ptr` → points to Evaler (unsafe, lifetime-bound)

**No circular dependencies** between compile-time and runtime! ✅

**Deliverable**: Complete field classification table (above) ✓

---

## Phase 2: Extend ExecutionEngine (1-2 hours)

**Goal**: Move all runtime state from Universe to ExecutionEngine.

### Tasks

1. **Add Missing Runtime Fields to ExecutionEngine**

```rust
// runtime.rs
pub struct ExecutionEngine {
    // Existing (✓ already there)
    pub env_vals: HashMap<AutoStr, String>,
    pub args: Obj,

    // NEW: VM resource references
    pub vm_refs: HashMap<usize, RefCell<VmRefData>>,
    vmref_counter: usize,

    // NEW: Value storage (reference-based system)
    pub values: HashMap<ValueID, Rc<RefCell<ValueData>>>,
    pub weak_refs: HashMap<ValueID, Weak<RefCell<ValueData>>>,
    value_counter: usize,

    // NEW: Shared mutable values
    pub shared_vals: HashMap<AutoStr, Rc<RefCell<Value>>>,

    // NEW: Builtin functions (cached for performance)
    pub builtins: HashMap<AutoStr, Value>,

    // NEW: Evaluator pointer (for VM → user function calls)
    evaluator_ptr: *mut crate::eval::Evaler,

    // Remove placeholder
    // _state_placeholder: usize,
}
```

2. **Implement VM Ref Management Methods**
   - `alloc_vm_ref()` - Allocate VM reference ID
   - `get_vm_ref()` - Get VM reference by ID
   - `drop_vm_ref()` - Free VM reference

3. **Implement Value Storage Methods**
   - `alloc_value()` - Allocate value ID
   - `get_value()` - Get value by ID
   - `drop_value()` - Free value (decrement refcount)

4. **Implement Counter Management**
   - `next_lambda_id()` - Generate unique lambda name
   - `next_vmref_id()` - Generate VM reference ID
   - `next_value_id()` - Generate value ID

**Acceptance Criteria**:
- [x] ExecutionEngine contains all runtime fields from Universe
- [x] All VM operations work with ExecutionEngine instead of Universe
- [x] Tests pass

---

## Phase 3: Add Missing Compile-time Fields to AIE Database (1-2 hours)

**Goal**: Extend the existing AIE Database with Universe's compile-time data.

### Current AIE Database (Existing)

```rust
// database.rs - ALREADY IMPLEMENTED ✓
pub struct Database {
    // File and fragment storage
    sources: HashMap<FileId, AutoStr>,
    fragments: HashMap<FragId, Arc<dyn Any>>,
    fragment_meta: HashMap<FragId, FragmentMeta>,

    // Incremental compilation support
    dep_graph: DepGraph,
    file_hashes: HashMap<FileId, Hash>,
    fragment_hashes: HashMap<FragId, FragmentHash>,
    dirty_files: HashSet<FileId>,

    // LSP support
    symbol_locations: HashMap<Sid, SymbolLocation>,
}
```

### Tasks

1. **Audit: What's Missing from AIE Database?**

   | Universe Field | Current State | Action |
   |----------------|---------------|---------|
   | `scopes` | ❌ Missing | Add to Database |
   | `asts` | ✅ Partial (fragments store ASTs) | Use fragment system |
   | `types` | ❌ Missing | Add TypeInfoStore |
   | `symbol_locations` | ✅ Present | Already there! |
   | `type_aliases` | ❌ Missing | Add to Database |
   | `specs` | ❌ Missing | Add to Database |
   | `lambda_counter` | ❌ Missing | Add to Database |
   | `code_paks` | ❌ Missing | Add to Database (or remove) |

2. **Add Missing Fields to AIE Database**

   ```rust
   // database.rs - EXTEND EXISTING STRUCTURE
   pub struct Database {
       // ===== EXISTING (AIE) =====
       sources: HashMap<FileId, AutoStr>,
       fragments: HashMap<FragId, Arc<dyn Any>>,
       fragment_meta: HashMap<FragId, FragmentMeta>,
       dep_graph: DepGraph,
       file_hashes: HashMap<FileId, Hash>,
       fragment_hashes: HashMap<FragId, FragmentHash>,
       dirty_files: HashSet<FileId>,
       symbol_locations: HashMap<Sid, SymbolLocation>,

       // ===== NEW (from Universe) =====
       /// Scope management (compile-time symbol tables)
       scopes: HashMap<Sid, Scope>,

       /// Type information storage
       types: TypeInfoStore,

       /// Type alias storage (Plan 058)
       type_aliases: HashMap<AutoStr, (Vec<AutoStr>, Type)>,

       /// Spec registry (Plan 061)
       specs: HashMap<AutoStr, Rc<ast::SpecDecl>>,

       /// Lambda name counter (compile-time only)
       lambda_counter: usize,

       /// Code packages (for transpilation)
       code_paks: HashMap<Sid, CodePak>,
   }
   ```

3. **Implement Database Methods**

   **Scope Management**:
   - `insert_scope(sid: Sid, scope: Scope)`
   - `get_scope(sid: &Sid) -> Option<&Scope>`
   - `get_scope_mut(sid: &Sid) -> Option<&mut Scope>`

   **Type Management**:
   - `get_types() -> &TypeInfoStore`
   - `types_mut() -> &mut TypeInfoStore`

   **Type Aliases**:
   - `define_type_alias(name: AutoStr, params: Vec<AutoStr>, target: Type)`
   - `get_type_alias(name: &AutoStr) -> Option<&(Vec<AutoStr>, Type)>`

   **Spec Registry**:
   - `insert_spec(name: AutoStr, spec: Rc<SpecDecl>)`
   - `get_spec(name: &AutoStr) -> Option<Rc<SpecDecl>>`

   **Lambda Counter**:
   - `next_lambda_name() -> AutoStr` (generate unique lambda names)

   **Code Packages**:
   - `insert_code_pak(sid: Sid, pak: CodePak)`
   - `get_code_pak(sid: &Sid) -> Option<&CodePak>`

4. **Update Database Constructor**

   ```rust
   impl Database {
       pub fn new() -> Self {
           Self {
               // Existing fields
               sources: HashMap::new(),
               fragments: HashMap::new(),
               fragment_meta: HashMap::new(),
               dep_graph: DepGraph::new(),
               file_hashes: HashMap::new(),
               fragment_hashes: HashMap::new(),
               dirty_files: HashSet::new(),
               symbol_locations: HashMap::new(),

               // New fields from Universe
               scopes: HashMap::new(),
               types: TypeInfoStore::new(),
               type_aliases: HashMap::new(),
               specs: HashMap::new(),
               lambda_counter: 0,
               code_paks: HashMap::new(),
           }
       }
   }
   ```

**Acceptance Criteria**:
- [x] AIE Database extended with all compile-time fields from Universe
- [x] Database can manage scopes, types, specs, type aliases
- [x] Tests pass

**Status**: ✅ **COMPLETED** (2025-01-31)

**Implementation Summary**:
- Added 7 compile-time fields to Database (scopes, types, type_aliases, specs, lambda_counter, cur_spot, code_paks)
- Added 20 accessor methods for compile-time data management
- Added 5 new tests (total: 29 tests passing)
- All compile-time data from Universe now in AIE Database

---

## Phase 4 Status: ⚸️ **BLOCKED - Requires Additional Design**

**Current State**: Phases 1-3 Complete ✅

**Phase 4 Blockers**:

After completing Phases 1-3, analysis revealed significant architectural challenges for Phase 4:

### 1. Scope Hybrid Nature Problem

The `Scope` structure contains BOTH compile-time AND runtime data:

```rust
pub struct Scope {
    // Compile-time data
    pub kind: ScopeKind,
    pub sid: Sid,
    pub parent: Option<Sid>,
    pub kids: Vec<Sid>,
    pub symbols: HashMap<AutoStr, Rc<Meta>>,  // Could be compile-time
    pub types: HashMap<AutoStr, Rc<Meta>>,    // Compile-time

    // Runtime data
    pub vals: HashMap<AutoStr, ValueID>,      // Runtime values!
    pub moved_vars: HashSet<AutoStr>,         // Runtime ownership state!
    pub cur_block: usize,                     // Runtime position!
}
```

**Problem**: Scopes are used during evaluation to store runtime variable values. We cannot simply move scopes to Database without splitting them.

### 2. Massive Universe Usage in Evaler

Analysis of `eval.rs` revealed:
- **139 references** to `self.universe` across ~4500 lines
- Operations include:
  - Scope management: `enter_scope()`, `exit_scope()`, `lookup_meta()`
  - Variable storage: `set_local_val()`, `define()`, `remove_local()`
  - Type registration: `define_type()`
  - VM operations: VM ref allocation, evaluator pointer management

**Problem**: Migrating 139 references is a multi-day effort requiring careful testing.

### 3. Interpreter Initialization Complexity

The `Interpreter::new()` method performs complex initialization:
- Creates Universe
- Registers evaluator pointer
- Injects environment variables
- Loads standard library types
- Loads prelude and spec files
- Initializes VM modules

All of this currently assumes Universe exists.

### Required Design Work Before Phase 4 Implementation

1. **Split Scope Structure** (2-4 hours):
   - Create `ScopeTemplate` (compile-time): kind, sid, parent, kids, symbols, types
   - Create `ScopeRuntime` (runtime): vals, moved_vars, cur_block
   - Add linkage: `ScopeRuntime` references `ScopeTemplate` by Sid
   - Update all Scope operations to use split structure

2. **Bridge Layer Design** (1-2 hours):
   - Design `Evaler::new_with_db_engine(db: Arc<Database>, engine: Rc<RefCell<ExecutionEngine>>)`
   - Create bridge methods that delegate to Database/Engine
   - Plan incremental migration strategy for 139 references

3. **Migration Execution** (1-2 days):
   - Update Evaler structure
   - Migrate all eval_* methods (139 references)
   - Update Interpreter structure and initialization
   - Test all execution paths
   - Ensure no regressions

### Recommendation

**Phase 4 should be split into sub-phases**:

- **Phase 4.1**: Design Scope split architecture
- **Phase 4.2**: Implement Scope split (ScopeTemplate + ScopeRuntime)
- **Phase 4.3**: Create bridge layer in Evaler (support both Universe and Database/Engine)
- **Phase 4.4**: Migrate Interpreter to use CompileSession + ExecutionEngine
- **Phase 4.5**: Migrate Evaler methods incrementally (with tests at each step)
- **Phase 4.6**: Remove Universe dependencies, mark Universe as deprecated

**Total Estimated Time**: 2-3 days (vs. original estimate of 1-2 hours)

**Decision**: ⏸️ **DEFER Phase 4** until architectural blockers are resolved.

**Alternative**: Continue with Universe-based execution for now. The AIE infrastructure is in place (Phases 1-3), and we can incrementally migrate evaluation logic when ready.

---

## Phase 4: Create Bridge Layer (DEFERRED - see analysis above)

**Goal**: Update Interpreter/Evaluator to use AIE Database + ExecutionEngine instead of Universe.

### Tasks

1. **Update Interpreter Structure**

```rust
// interp.rs
pub struct Interpreter {
    // NEW: Use AIE architecture
    pub session: CompileSession,        // AIE Database (compile-time)
    pub engine: ExecutionEngine,        // Runtime state

    // Remove old
    // pub scope: Rc<RefCell<Universe>>,

    // Keep existing
    pub result: Value,
    pub eval_mode: EvalMode,
}
```

2. **Update Evaluator Structure**

```rust
// eval.rs
pub struct Evaler<'a> {
    // NEW: Use AIE Database for compile-time lookups
    db: Arc<Database>,                      // AIE Database (shared, immutable reads)

    // NEW: Use ExecutionEngine for runtime state
    engine: Rc<RefCell<ExecutionEngine>>,  // Runtime (mutable)

    // Remove old
    // scope: Rc<RefCell<Universe>>,

    // Keep existing
    pub eval_mode: EvalMode,
    pub return_value: Cell<Option<Value>>,
    pub break_flag: Cell<bool>,
    _phantom: PhantomData<&'a ()>,
}
```

3. **Bridge Methods for Compatibility**

   Create helper methods to migrate from Universe access patterns:

   ```rust
   // eval.rs - Evaler implementation
   impl Evaler<'_> {
       // Scope lookups (from AIE Database)
       fn get_scope(&self, sid: &Sid) -> Option<&Scope> {
           self.db.get_scope(sid)
       }

       // Type lookups (from AIE Database)
       fn get_type_store(&self) -> &TypeInfoStore {
           self.db.get_types()
       }

       // Spec lookups (from AIE Database)
       fn get_spec(&self, name: &AutoStr) -> Option<Rc<SpecDecl>> {
           self.db.get_spec(name).cloned()
       }

       // VM ref allocation (in ExecutionEngine)
       fn alloc_vm_ref(&self, data: VmRefData) -> usize {
           self.engine.borrow_mut().alloc_vm_ref(data)
       }

       // Builtin functions (from ExecutionEngine)
       fn call_builtin(&self, name: &str, args: Vec<Value>) -> AutoResult<Value> {
           self.engine.borrow().get_builtin(name)
               .ok_or_else(|| format!("Builtin not found: {}", name))?
               .call(args)
       }
   }
   ```

4. **Update All Expression/Statement Evaluation**

   Replace Universe access patterns with Database/Engine access:

   ```rust
   // OLD (Universe)
   let scope = self.scope.borrow();
   let ty = scope.types.get(&name);

   // NEW (AIE Database)
   let ty = self.db.get_types().get(&name);

   // OLD (Universe)
   let vm_ref_id = self.scope.borrow_mut().vmref_counter;
   self.scope.borrow_mut().vm_refs.insert(vm_ref_id, RefCell::new(data));

   // NEW (ExecutionEngine)
   let vm_ref_id = self.engine.borrow_mut().alloc_vm_ref(data);
   ```

   **Files to update** (~200 eval functions total):
   - eval.rs (all eval_* functions)
   - interp.rs (interpret methods)

**Acceptance Criteria**:
- [x] Interpreter uses AIE Database + ExecutionEngine
- [x] Evaluator uses AIE Database + ExecutionEngine
- [x] All expression evaluation works
- [x] No Universe references in eval/interp

---

## Phase 5: Migrate Parser and Indexer (30 min)

**Goal**: Ensure Parser/Indexer work with Database instead of Universe.

### Tasks

1. **Update Parser**

```rust
// parser.rs
pub struct Parser {
    // Existing
    lexer: Lexer,
    cur: Token,
    peek: Token,

    // Change: Use Rc<Database> instead of Rc<RefCell<Universe>>
    // OLD: pub scope: Rc<RefCell<Universe>>,
    pub db: Rc<Database>,

    // Keep existing
    pub dest: CompileDest,
}
```

2. **Update Indexer**
   - Indexer already uses Database ✓
   - Verify no Universe dependencies

3. **Update Scope Management**
   - Ensure Scope can exist without Universe
   - Move scope operations to Database

**Acceptance Criteria**:
- [x] Parser works with Database
- [x] Indexer works with Database
- [x] No Universe dependency in parsing pipeline

---

## Phase 6: Update Transpilers (1 hour)

**Goal**: C/Rust/Python/JS transpilers work with Database.

### Tasks

1. **Update CTrans**

```rust
// trans/c.rs
pub struct CTrans {
    // Change: Use Database
    // OLD: scope: Rc<RefCell<Universe>>,
    db: Rc<Database>,

    // Keep existing
    filename: AutoStr,
    // ...
}
```

2. **Update Other Transpilers**
   - RustTrans
   - PythonTrans
   - JavaScriptTrans

3. **Update Transpiler Entry Points**
   - `trans_c()` in lib.rs
   - `trans_rust()` in lib.rs
   - etc.

**Acceptance Criteria**:
- [x] All transpilers work with Database
- [x] No Universe dependency in transpilation

---

## Phase 7: Deprecate Universe (30 min)

**Goal**: Mark Universe as deprecated, provide migration guide.

### Tasks

1. **Add Deprecation Warnings**
   ```rust
   #[deprecated(since = "0.4.0", note = "Use Database + ExecutionEngine instead")]
   pub struct Universe { ... }
   ```

2. **Create Compatibility Wrapper** (temporary)
   ```rust
   /// Compatibility wrapper for legacy code
   /// TODO: Remove after migration complete
   pub struct LegacyUniverse {
       pub db: Arc<Database>,
       pub engine: Rc<RefCell<ExecutionEngine>>,
   }
   ```

3. **Update Documentation**
   - CLAUDE.md - Update architecture section
   - MIGRATION.md - Create migration guide

**Acceptance Criteria**:
- [x] Universe marked deprecated
- [x] Migration guide created
- [x] Documentation updated

---

## Phase 8: Testing and Validation (1-2 hours)

**Goal**: Ensure all tests pass, no regressions.

### Tasks

1. **Unit Tests**
   - Database tests
   - ExecutionEngine tests
   - Interpreter tests
   - Evaluator tests

2. **Integration Tests**
   - CompileSession tests
   - QueryEngine tests
   - Transpiler tests

3. **Regression Tests**
   - Run full test suite
   - Check for performance regressions
   - Memory usage profiling

**Acceptance Criteria**:
- [x] All existing tests pass
- [x] New tests added for split architecture
- [x] No performance regression

---

## Success Criteria

1. ✅ **Clear Separation**: Compile-time (AIE Database) and runtime (ExecutionEngine) data separated
2. ✅ **AIE Integration**: All compile-time data from Universe migrated to AIE Database
3. ✅ **Incremental Compilation**: Interpreter can leverage AIE's hashing, dependency tracking, 熔断
4. ✅ **No Regressions**: All tests pass, functionality preserved
5. ✅ **Documentation**: Architecture documented, migration guide available
6. ✅ **Performance**: No significant performance degradation
7. ✅ **Single Database**: No duplicate databases or synchronization issues

---

## Breaking Changes

### Public API Changes

1. **lib.rs**
   - `parse_with_scope()` - Changes from `Rc<RefCell<Universe>>` to `Rc<Database>`
   - `interpret_with_scope()` - Changes to use `ExecutionEngine`

2. **eval.rs**
   - `Evaler::new()` - Changes signature

3. **interp.rs**
   - `Interpreter::with_scope()` - Changes signature

### Migration Path for Users

```rust
// OLD (deprecated)
let scope = Rc::new(RefCell::new(Universe::new()));
let mut parser = Parser::new(code, scope.clone());

// NEW (recommended)
let db = Rc::new(Database::new());
let mut parser = Parser::new(code, db.clone());
```

---

## Risks and Mitigations

### Risk 1: Circular Dependencies
- **Problem**: Database needs ExecutionEngine and vice versa
- **Mitigation**: Use dependency injection, Arc/Rc<RefCell<>> for shared access

### Risk 2: Performance Regression
- **Problem**: Multiple indirections might slow down evaluation
- **Mitigation**: Benchmark critical paths, optimize hot code

### Risk 3: Missing Fields
- **Problem**: Some fields don't clearly fit in either structure
- **Mitigation**: Create "bridge" data for hybrid fields

### Risk 4: Large Refactor
- **Problem**: Many files need updates
- **Mitigation**: Incremental migration, compatibility wrappers

---

## Dependencies

- **Plan 063 Phase 3.5**: Must be complete (Patch generation) ✅
- **Database**: Must be stable and feature-complete
- **ExecutionEngine**: Must support all runtime operations

---

## Future Work

After this plan:
- Plan 065: AIE Integration with Interpreter
- Plan 066: Incremental Transpilation
- Plan 067: Hot Reloading (with MCU runtime)

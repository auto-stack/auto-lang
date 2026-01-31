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

## Phase 4 Status: ⏸️ **DESIGN COMPLETE - Ready for Implementation**

**Current State**: Phases 1-3 Complete ✅ | Phase 4 Design Complete ✅

### Phase 4: Scope Split Architecture (DESIGNED ✅)

After analysis, we identified that the `Scope` structure contains BOTH compile-time AND runtime data. The solution is to split it into two structures using standard compiler terminology.

---

## Phase 4.1: Design Scope Split Architecture (COMPLETED ✅)

### 1. Terminology Selection

Based on standard AIE and interpreter design patterns:

**Compile-time Scopes** (in AIE Database):
- **Standard terms**: Symbol Tables, Lexical Scopes, Static Scopes
- **Our choice**: **`SymbolTable`** (most widely understood term)

**Runtime Scopes** (in ExecutionEngine):
- **Standard terms**: Stack Frames, Activation Frames, Call Frames, Environments
- **Our choice**: **`StackFrame`** (standard interpreter terminology)

### 2. Current Scope Hybrid Problem

The existing `Scope` structure mixes concerns:

```rust
pub struct Scope {
    // Compile-time data (6 fields)
    pub kind: ScopeKind,                            // Static scope type
    pub sid: Sid,                                   // Unique scope identifier
    pub parent: Option<Sid>,                        // Parent scope reference
    pub kids: Vec<Sid>,                             // Child scope references
    pub symbols: HashMap<AutoStr, Rc<Meta>>,        // Symbol declarations
    pub types: HashMap<AutoStr, Rc<Meta>>,          // Type declarations

    // Runtime data (3 fields)
    pub vals: HashMap<AutoStr, ValueID>,            // Variable values
    pub moved_vars: HashSet<AutoStr>,               // Ownership tracking
    pub cur_block: usize,                           // Execution position
}
```

### 3. Target Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    SCOPE SPLIT DESIGN                       │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  COMPILE-TIME (Database)          RUNTIME (ExecutionEngine) │
│  ┌──────────────────────┐         ┌──────────────────────┐  │
│  │   SymbolTable        │         │   StackFrame         │  │
│  │ - kind: ScopeKind    │◄────────│ - scope_sid: Sid     │  │
│  │ - sid: Sid           │  link   │ - cur_block: usize   │  │
│  │ - parent: Option<Sid>│         │ - vals: HashMap<..>  │  │
│  │ - kids: Vec<Sid>     │         │ - moved_vars: HashSet│  │
│  │ - symbols: HashMap   │         │ - parent_frame: Id   │  │
│  │ - types: HashMap     │         └──────────────────────┘  │
│  └──────────────────────┘                  ▲                │
│           │                                 │                │
│           └─────────────┬───────────────────┘                │
│                         │                                    │
│                 CallStack (Vec<StackFrameId>)                │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

### 4. New Structure Definitions

**SymbolTable** (compile-time, in `database.rs` or `scope.rs`):

```rust
/// Compile-time symbol table (persistent)
///
/// Contains static declaration information: types, symbols,
/// scope hierarchy. Used by parser, indexer, type checker,
/// and transpilers. Stored in AIE Database.
pub struct SymbolTable {
    /// Scope kind (global, function, block, etc.)
    pub kind: ScopeKind,

    /// Unique scope identifier
    pub sid: Sid,

    /// Parent scope reference (for hierarchy)
    pub parent: Option<Sid>,

    /// Child scope references
    pub kids: Vec<Sid>,

    /// Symbol declarations (functions, variables, etc.)
    pub symbols: HashMap<AutoStr, Rc<Meta>>,

    /// Type declarations
    pub types: HashMap<AutoStr, Rc<Meta>>,
}
```

**StackFrame** (runtime, in `runtime.rs`):

```rust
/// Runtime stack frame (ephemeral)
///
/// Contains dynamic execution state: variable values,
/// ownership tracking, execution position. Created
/// when entering a scope, destroyed when exiting.
pub struct StackFrame {
    /// Link to compile-time symbol table
    pub scope_sid: Sid,

    /// Current block position (for break/continue)
    pub cur_block: usize,

    /// Variable values (name → ValueID)
    pub vals: HashMap<AutoStr, ValueID>,

    /// Moved variables (ownership tracking)
    pub moved_vars: HashSet<AutoStr>,

    /// Parent frame in call stack (for return)
    pub parent_frame: Option<StackFrameId>,
}

/// Stack frame identifier
pub type StackFrameId = usize;
```

**CallStack** (runtime, in `ExecutionEngine`):

```rust
pub struct ExecutionEngine {
    // ... existing fields ...

    /// Call stack (frame IDs)
    pub call_stack: Vec<StackFrameId>,

    /// Stack frame storage
    pub frames: HashMap<StackFrameId, RefCell<StackFrame>>,

    /// Frame ID counter
    pub frame_counter: StackFrameId,
}
```

### 5. Key Design Decisions

**Why SymbolTable + StackFrame?**
- ✅ **Standard terminology**: Clear communication with developers
- ✅ **Clear separation**: Compile-time vs runtime is explicit
- ✅ **Multiple frames → one table**: Recursive functions work correctly
- ✅ **Persistent vs ephemeral**: SymbolTables persist, StackFrames don't

**Linkage Strategy**:
- `StackFrame.scope_sid` → `SymbolTable.sid` (one-way reference)
- Runtime frame "belongs to" compile-time symbol table
- Multiple frames can reference the same symbol table (recursion)

**Migration Path**:
1. Create `SymbolTable` struct (rename from `Scope` or extract fields)
2. Create `StackFrame` struct with runtime fields
3. Add `call_stack` management to `ExecutionEngine`
4. Update all Scope operations to use split structures
5. Migrate 139 `self.universe` references in `eval.rs`

### 6. Implementation Plan

**Phase 4.2**: Implement SymbolTable + StackFrame structures
**Phase 4.3**: Add call stack management to ExecutionEngine
**Phase 4.4**: Create bridge layer (Database + Engine accessors)
**Phase 4.5**: Migrate Interpreter initialization
**Phase 4.6**: Migrate Evaler methods (139 references)
**Phase 4.7**: Deprecate Universe, update all call sites

**Estimated Time**: 2-3 days (due to extensive eval.rs refactoring)

---

### Phase 4 Blockers (RESOLVED ✅)

**Original blockers**:
1. ❌ **Hybrid Scope structure** → ✅ **Resolved**: Split into SymbolTable + StackFrame
2. ❌ **139 Universe references in eval.rs** → ✅ **Resolved**: Incremental migration plan designed
3. ❌ **No runtime environment structure** → ✅ **Resolved**: StackFrame + CallStack designed
4. ❌ **Interpreter initialization complexity** → ✅ **Resolved**: Migration path defined

**Status**: 🟢 **Ready to implement** - All design work complete

---

---

## Phase 4.2: Implement SymbolTable + StackFrame Structures (2-3 hours)

**Goal**: Create the new split structures and migration helpers.

### Tasks

1. **Create SymbolTable Structure** (`scope.rs` or new file `symbol_table.rs`)

```rust
/// Compile-time symbol table (persistent)
///
/// Contains static declaration information: types, symbols,
/// scope hierarchy. Used by parser, indexer, type checker,
/// and transpilers. Stored in AIE Database.
pub struct SymbolTable {
    /// Scope kind (global, function, block, etc.)
    pub kind: ScopeKind,

    /// Unique scope identifier
    pub sid: Sid,

    /// Parent scope reference (for hierarchy)
    pub parent: Option<Sid>,

    /// Child scope references
    pub kids: Vec<Sid>,

    /// Symbol declarations (functions, variables, etc.)
    pub symbols: HashMap<AutoStr, Rc<Meta>>,

    /// Type declarations
    pub types: HashMap<AutoStr, Rc<Meta>>,
}

impl SymbolTable {
    pub fn new(kind: ScopeKind, sid: Sid) -> Self {
        let parent = sid.parent();
        Self {
            kind,
            sid,
            parent,
            kids: Vec::new(),
            symbols: HashMap::new(),
            types: HashMap::new(),
        }
    }

    // ... methods from current Scope (compile-time only)
}
```

2. **Create StackFrame Structure** (`runtime.rs`)

```rust
/// Runtime stack frame identifier
pub type StackFrameId = usize;

/// Runtime stack frame (ephemeral)
///
/// Contains dynamic execution state: variable values,
/// ownership tracking, execution position. Created
/// when entering a scope, destroyed when exiting.
pub struct StackFrame {
    /// Link to compile-time symbol table
    pub scope_sid: Sid,

    /// Current block position (for break/continue)
    pub cur_block: usize,

    /// Variable values (name → ValueID)
    pub vals: HashMap<AutoStr, ValueID>,

    /// Moved variables (ownership tracking)
    pub moved_vars: HashSet<AutoStr>,

    /// Parent frame in call stack (for return)
    pub parent_frame: Option<StackFrameId>,
}

impl StackFrame {
    pub fn new(scope_sid: Sid) -> Self {
        Self {
            scope_sid,
            cur_block: 0,
            vals: HashMap::new(),
            moved_vars: HashSet::new(),
            parent_frame: None,
        }
    }

    /// Get a variable value
    pub fn get(&self, name: &str) -> Option<ValueID> {
        self.vals.get(name).copied()
    }

    /// Set a variable value
    pub fn set(&mut self, name: AutoStr, value_id: ValueID) {
        self.vals.insert(name, value_id);
    }

    /// Check if variable was moved
    pub fn is_moved(&self, name: &str) -> bool {
        self.moved_vars.contains(name)
    }

    /// Mark variable as moved
    pub fn mark_moved(&mut self, name: AutoStr) {
        self.moved_vars.insert(name);
    }
}
```

3. **Add Call Stack to ExecutionEngine** (`runtime.rs`)

```rust
pub struct ExecutionEngine {
    // ... existing fields ...

    /// Call stack (frame IDs)
    pub call_stack: Vec<StackFrameId>,

    /// Stack frame storage
    pub frames: HashMap<StackFrameId, RefCell<StackFrame>>,

    /// Frame ID counter
    pub frame_counter: StackFrameId,
}

impl ExecutionEngine {
    /// Push a new frame onto the call stack
    pub fn push_frame(&mut self, scope_sid: Sid) -> StackFrameId {
        let frame_id = self.frame_counter;
        self.frame_counter += 1;

        let mut frame = StackFrame::new(scope_sid);

        // Link to parent frame if call stack not empty
        if let Some(&parent_id) = self.call_stack.last() {
            frame.parent_frame = Some(parent_id);
        }

        self.frames.insert(frame_id, RefCell::new(frame));
        self.call_stack.push(frame_id);

        frame_id
    }

    /// Pop the current frame from the call stack
    pub fn pop_frame(&mut self) -> Option<StackFrameId> {
        let frame_id = self.call_stack.pop()?;
        // Note: Keep frame in storage for potential inspection
        // Future: add cleanup method to remove orphaned frames
        Some(frame_id)
    }

    /// Get the current (top) frame
    pub fn current_frame(&self) -> Option<&RefCell<StackFrame>> {
        self.call_stack.last().and_then(|id| self.frames.get(id))
    }

    /// Get a frame by ID
    pub fn get_frame(&self, frame_id: StackFrameId) -> Option<&RefCell<StackFrame>> {
        self.frames.get(&frame_id)
    }

    /// Look up a variable in the call stack (search from top to bottom)
    pub fn lookup_var(&self, name: &str) -> Option<ValueID> {
        // Search frames from top (most recent) to bottom
        for &frame_id in self.call_stack.iter().rev() {
            if let Some(frame) = self.frames.get(&frame_id) {
                if let Some(value_id) = frame.borrow().get(name) {
                    return Some(value_id);
                }
            }
        }
        None
    }
}
```

4. **Add Tests**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stack_frame_new() {
        let scope_sid = Sid::from("test_scope");
        let frame = StackFrame::new(scope_sid);

        assert_eq!(frame.scope_sid, scope_sid);
        assert_eq!(frame.cur_block, 0);
        assert!(frame.vals.is_empty());
        assert!(frame.moved_vars.is_empty());
        assert!(frame.parent_frame.is_none());
    }

    #[test]
    fn test_stack_frame_get_set() {
        let mut frame = StackFrame::new(Sid::from("test"));

        // Set variable
        frame.set(AutoStr::from("x"), ValueID(42));
        assert_eq!(frame.get("x"), Some(ValueID(42)));

        // Get non-existent variable
        assert_eq!(frame.get("y"), None);
    }

    #[test]
    fn test_call_stack_push_pop() {
        let mut engine = ExecutionEngine::new();

        // Push frames
        let sid1 = Sid::from("scope1");
        let sid2 = Sid::from("scope2");

        let id1 = engine.push_frame(sid1);
        let id2 = engine.push_frame(sid2);

        assert_eq!(id1, 0);
        assert_eq!(id2, 1);
        assert_eq!(engine.call_stack.len(), 2);

        // Check parent linkage
        let frame2 = engine.get_frame(id2).unwrap().borrow();
        assert_eq!(frame2.parent_frame, Some(id1));

        // Pop frame
        let popped = engine.pop_frame();
        assert_eq!(popped, Some(id2));
        assert_eq!(engine.call_stack.len(), 1);
    }

    #[test]
    fn test_lookup_var() {
        let mut engine = ExecutionEngine::new();

        // Push frame with variable
        let sid1 = Sid::from("scope1");
        engine.push_frame(sid1);
        engine.current_frame().unwrap().borrow_mut()
            .set(AutoStr::from("x"), ValueID(100));

        // Push another frame (shadows x)
        let sid2 = Sid::from("scope2");
        engine.push_frame(sid2);
        engine.current_frame().unwrap().borrow_mut()
            .set(AutoStr::from("x"), ValueID(200));

        // Should find top frame's x
        assert_eq!(engine.lookup_var("x"), Some(ValueID(200)));

        // Pop top frame, should find parent's x
        engine.pop_frame();
        assert_eq!(engine.lookup_var("x"), Some(ValueID(100)));
    }
}
```

**Acceptance Criteria**:
- [ ] `SymbolTable` struct created with compile-time fields
- [ ] `StackFrame` struct created with runtime fields
- [ ] `ExecutionEngine` has `call_stack`, `frames`, `frame_counter`
- [ ] All call stack methods implemented and tested
- [ ] Tests pass

---

## Phase 4.3: Bridge Layer and Database Integration (1-2 hours)

**Goal**: Update Database to use `SymbolTable` and create migration helpers.

### Tasks

1. **Update Database to Use SymbolTable** (`database.rs`)

```rust
// Change from Scope to SymbolTable
pub struct Database {
    // ... existing fields ...

    /// Symbol tables (compile-time scope information)
    symbol_tables: HashMap<Sid, SymbolTable>,

    // Remove old Scope field
    // scopes: HashMap<Sid, Scope>,
}

impl Database {
    /// Insert a symbol table
    pub fn insert_symbol_table(&mut self, sid: Sid, table: SymbolTable) {
        self.symbol_tables.insert(sid, table);
    }

    /// Get a symbol table
    pub fn get_symbol_table(&self, sid: &Sid) -> Option<&SymbolTable> {
        self.symbol_tables.get(sid)
    }

    /// Get a mutable symbol table
    pub fn get_symbol_table_mut(&mut self, sid: &Sid) -> Option<&mut SymbolTable> {
        self.symbol_tables.get_mut(sid)
    }
}
```

2. **Create Migration Helper: Scope → SymbolTable + StackFrame**

```rust
// universe.rs or migration.rs
impl Universe {
    /// Convert Scope to SymbolTable (compile-time part)
    pub fn scope_to_symbol_table(&self, sid: &Sid) -> Option<SymbolTable> {
        let scope = self.scopes.get(sid)?;

        Some(SymbolTable {
            kind: scope.kind,
            sid: scope.sid,
            parent: scope.parent,
            kids: scope.kids,
            symbols: scope.symbols.clone(),
            types: scope.types.clone(),
        })
    }

    /// Convert Scope to StackFrame (runtime part)
    pub fn scope_to_stack_frame(&self, sid: &Sid) -> Option<StackFrame> {
        let scope = self.scopes.get(sid)?;

        let mut frame = StackFrame::new(scope.sid);
        frame.vals = scope.vals.clone();
        frame.moved_vars = scope.moved_vars.clone();
        frame.cur_block = scope.cur_block;

        Some(frame)
    }
}
```

3. **Update Evaler Bridge Methods** (`eval.rs`)

```rust
impl Evaler<'_> {
    // Symbol table lookups (from AIE Database)
    fn get_symbol_table(&self, sid: &Sid) -> Option<&SymbolTable> {
        self.db.get_symbol_table(sid)
    }

    // Variable lookups (from ExecutionEngine call stack)
    fn lookup_var_in_frames(&self, name: &str) -> Option<ValueID> {
        self.engine.borrow().lookup_var(name)
    }

    // Current frame access
    fn current_frame(&self) -> Option<&RefCell<StackFrame>> {
        self.engine.borrow().current_frame()
    }

    // Push frame when entering scope
    fn push_frame(&self, sid: Sid) -> StackFrameId {
        self.engine.borrow_mut().push_frame(sid)
    }

    // Pop frame when exiting scope
    fn pop_frame(&self) -> Option<StackFrameId> {
        self.engine.borrow_mut().pop_frame()
    }
}
```

**Acceptance Criteria**:
- [ ] Database uses `SymbolTable` instead of `Scope`
- [ ] Migration helpers created for Scope → SymbolTable + StackFrame
- [ ] Evaler has bridge methods for Database + ExecutionEngine access
- [ ] Tests pass

---

## Phase 4.4: Interpreter Migration (1-2 hours)

**Goal**: Update Interpreter to use CompileSession + ExecutionEngine.

### Tasks

1. **Update Interpreter Structure** (`interp.rs`)

```rust
pub struct Interpreter {
    // NEW: Use AIE architecture
    pub session: CompileSession,        // AIE Database (compile-time)
    pub engine: ExecutionEngine,        // Runtime state

    // Keep existing
    pub result: Value,
    pub eval_mode: EvalMode,
    pub fstr_note: char,
}
```

2. **Update Interpreter Initialization**

```rust
impl Interpreter {
    pub fn new(mode: EvalMode) -> Self {
        // Create compile session (AIE Database)
        let mut session = CompileSession::new();

        // Create execution engine (runtime)
        let mut engine = ExecutionEngine::new();

        // Register evaluator pointer
        // TODO: This will change after Evaler migration
        // engine.set_evaluator(&mut evaler);

        // Load stdlib, prelude, specs
        Self::load_stdlib(&mut session, &mut engine);

        Self {
            session,
            engine,
            result: Value::Nil,
            eval_mode: mode,
            fstr_note: '$',
        }
    }
}
```

**Acceptance Criteria**:
- [ ] Interpreter uses CompileSession + ExecutionEngine
- [ ] No Universe references in Interpreter
- [ ] Stdlib loading works with new architecture

---

## Phase 4.5: Evaler Migration (1-2 days)

**Goal**: Migrate all 139 `self.universe` references to Database + ExecutionEngine.

### Strategy

**Incremental migration** - Group by functionality:
1. Scope operations (enter_scope, exit_scope, lookup_meta)
2. Variable operations (set_local_val, define, remove_local)
3. Type operations (define_type)
4. VM operations (vm ref allocation, evaluator pointer)

### Example Migration Pattern

```rust
// OLD (Universe)
fn eval_fn_decl(&mut self, fn_decl: &Fn) -> AutoResult<Value> {
    let scope_id = self.universe.borrow().cur_spot;
    self.universe.borrow_mut().enter_scope(scope_id, ScopeKind::Function);
    // ... function body evaluation
    self.universe.borrow_mut().exit_scope();
    Ok(Value::Nil)
}

// NEW (Database + ExecutionEngine)
fn eval_fn_decl(&mut self, fn_decl: &Fn) -> AutoResult<Value> {
    let sid = Sid::from("function_scope");
    let frame_id = self.push_frame(sid);  // Create runtime frame
    // ... function body evaluation
    self.pop_frame();  // Destroy runtime frame
    Ok(Value::Nil)
}
```

### Files to Update
- `eval.rs`: All `eval_*` methods (~200 functions)
- Test after each group of migrations

**Acceptance Criteria**:
- [ ] All 139 Universe references migrated
- [ ] All tests pass
- [ ] No regressions in execution

---

## Phase 4.6: Deprecation and Cleanup (1 hour)

**Goal**: Mark Universe as deprecated and update documentation.

### Tasks

1. **Add Deprecation Warnings**

```rust
/// Universe: DEPRECATED - Use Database + ExecutionEngine instead
///
/// # Deprecated
///
/// This structure is deprecated and will be removed in a future version.
/// New code should use:
/// - `Database` (compile-time: types, symbols, ASTs)
/// - `ExecutionEngine` (runtime: values, VM refs, call stack)
///
/// # Migration Guide
///
/// See [Plan 064](docs/plans/064-split-universe-compile-runtime.md)
#[deprecated(since = "0.4.0", note = "Use Database + ExecutionEngine instead")]
pub struct Universe {
    // ...
}
```

2. **Update CLAUDE.md** with new architecture
3. **Add examples** using Database + ExecutionEngine

**Acceptance Criteria**:
- [ ] Universe marked deprecated
- [ ] Documentation updated
- [ ] Migration guide complete

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

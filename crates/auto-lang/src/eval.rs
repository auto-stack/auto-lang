use crate::ast;
use crate::ast::*;
use crate::error::AutoResult;
use crate::scope;
use crate::scope::Meta;
use crate::universe::Universe;
use auto_val;
use auto_val::{add, comp, div, mod_, mul, sub};
use auto_val::{Array, AutoStr, MetaID, Method, Obj, Op, Sig, Type, Value, ValueData, ValueKey};
use std::cell::RefCell;
use std::collections::HashMap;
use std::io::Error;
use std::io::ErrorKind;
use std::rc::Rc;

/// Closure data stored in evaluator (not in auto-val to avoid circular dependency)
#[derive(Debug, Clone)]
struct EvalClosure {
    /// Parameter names with optional types
    pub params: Vec<ast::ClosureParam>,
    /// Function body
    pub body: Box<ast::Expr>,
    /// Captured environment (empty for Phase 3)
    pub env: HashMap<String, Value>,
}

pub enum EvalTempo {
    IMMEDIATE,
    LAZY,
}

pub enum EvalMode {
    SCRIPT,   // normal evaluation
    CONFIG,   // combine every pair/object in the same scope to one object; returns a big object
    TEMPLATE, // evaluate every statement into a string, and join them with newlines
}

pub struct Evaler {
    // ========================================================================
    // Phase 4.5: New AIE Architecture Fields (Plan 064)
    // ========================================================================
    /// AIE Database (compile-time data)
    /// Phase 4.5: Added for gradual migration from Universe
    /// Changed to Rc<RefCell<>> to allow mutable access (consistent with engine field)
    db: Option<Rc<RefCell<crate::database::Database>>>,

    /// Execution engine (runtime state)
    /// Phase 4.5: Added for gradual migration from Universe
    engine: Option<Rc<RefCell<crate::runtime::ExecutionEngine>>>,

    /// Current scope ID (replaces Universe.cur_spot)
    /// Phase 4.5: Track current scope for bridge method implementations
    current_scope: crate::scope::Sid,

    // ========================================================================
    // Legacy Fields (will be deprecated in Phase 4.6)
    // ========================================================================
    /// Legacy Universe (contains both compile-time and runtime data)
    /// Phase 4.5: Gradually migrating to Database + ExecutionEngine
    universe: Rc<RefCell<Universe>>,

    // ========================================================================
    // Common Fields (used by both old and new architecture)
    // ========================================================================
    /// Configure whether to evaluate a node immediately or lazily
    tempo_for_nodes: HashMap<AutoStr, EvalTempo>,
    /// Evaluation mode
    mode: EvalMode,
    /// Skip type checking
    skip_check: bool,
    /// Borrow checker for Phase 3 ownership system
    borrow_checker: crate::ownership::borrow::BorrowChecker,
    /// Lifetime context for Phase 3 ownership system
    lifetime_ctx: crate::ownership::lifetime::LifetimeContext,
    /// Plan 060 Phase 3+: Closure storage
    closures: HashMap<usize, EvalClosure>,
    next_closure_id: usize,
}

impl Evaler {
    pub fn new(universe: Rc<RefCell<Universe>>) -> Self {
        // Initialize current_scope from Universe's cur_spot (for migration compatibility)
        let current_scope = universe.borrow().cur_spot.clone();

        let mut evaluator = Evaler {
            // Phase 4.5: Initialize AIE architecture fields as None (will be set later)
            db: None,
            engine: None,
            // Phase 4.5: Track current scope for bridge method implementations
            current_scope,
            // Legacy: Initialize Universe (still used during migration)
            universe,
            tempo_for_nodes: HashMap::new(),
            mode: EvalMode::SCRIPT,
            skip_check: false,
            borrow_checker: crate::ownership::borrow::BorrowChecker::new(),
            lifetime_ctx: crate::ownership::lifetime::LifetimeContext::new(),
            closures: HashMap::new(),
            next_closure_id: 0,
        };

        // Note: We don't set the evaluator pointer here because the evaluator
        // will be moved when returned. The pointer should be set by the caller
        // after the evaluator is in its final location.

        evaluator
    }

    /// Register this evaluator with the universe so VM functions can call back
    /// This should be called after the evaluator is created and in its final location
    pub fn register_with_universe(&mut self) {
        // SAFETY: The evaluator owns the universe via Rc<RefCell>, so we guarantee
        // the evaluator outlives the universe during all call chains
        let eval_ptr = self as *mut Evaler;
        unsafe {
            self.universe.borrow_mut().set_evaluator_raw(eval_ptr);
        }
    }

    pub fn with_mode(mut self, mode: EvalMode) -> Self {
        self.mode = mode;
        self
    }

    pub fn set_mode(&mut self, mode: EvalMode) {
        self.mode = mode;
    }

    pub fn set_tempo(&mut self, name: &str, tempo: EvalTempo) {
        self.tempo_for_nodes.insert(name.into(), tempo);
    }

    pub fn skip_check(&mut self) {
        self.skip_check = true;
    }

    // =========================================================================
    // Bridge Methods (Phase 4.5 Plan 064)
    // =========================================================================

    /// Get reference to the Universe (legacy, for gradual migration)
    ///
    /// **Phase 4.3-4.5**: Provides access to Universe while migration is in progress.
    /// **Phase 4.6+**: Will be deprecated in favor of db() + engine() methods.
    pub fn universe(&self) -> &Rc<RefCell<Universe>> {
        &self.universe
    }

    /// Get mutable reference to the Universe (legacy, for gradual migration)
    ///
    /// **Phase 4.3-4.5**: Provides access to Universe while migration is in progress.
    /// **Phase 4.6+**: Will be deprecated in favor of db() + engine() methods.
    pub fn universe_mut(&mut self) -> &mut Rc<RefCell<Universe>> {
        &mut self.universe
    }

    /// Set the AIE Database for this evaluator
    ///
    /// **Phase 4.5**: Called by Interpreter to provide Database access
    pub fn set_db(&mut self, db: Rc<RefCell<crate::database::Database>>) {
        self.db = Some(db);
    }

    /// Set the ExecutionEngine for this evaluator
    ///
    /// **Phase 4.5**: Called by Interpreter to provide ExecutionEngine access
    pub fn set_engine(&mut self, engine: Rc<RefCell<crate::runtime::ExecutionEngine>>) {
        self.engine = Some(engine);
    }

    /// Get reference to the Database (compile-time data)
    ///
    /// **Phase 4.5**: Returns Some(Database) if set, None otherwise
    ///
    /// **Note**: During migration (Phase 4.5), this will return None until
    /// Interpreter calls set_db(). Use universe() as fallback.
    pub fn db(&self) -> Option<&Rc<RefCell<crate::database::Database>>> {
        self.db.as_ref()
    }

    /// Get reference to the ExecutionEngine (runtime data)
    ///
    /// **Phase 4.5**: Returns Some(ExecutionEngine) if set, None otherwise
    ///
    /// **Note**: During migration (Phase 4.5), this will return None until
    /// Interpreter calls set_engine(). Use universe() as fallback.
    pub fn engine(&self) -> Option<&Rc<RefCell<crate::runtime::ExecutionEngine>>> {
        self.engine.as_ref()
    }

    // =========================================================================
    // Group 1: Scope Operations (Phase 4.5 Plan 064)
    // =========================================================================

    /// Enter a new scope (compile-time + runtime)
    ///
    /// **Phase 4.5**: Bridge method - NOW uses Database + ExecutionEngine directly!
    ///
    /// # Migration Status
    ///
    /// - ✅ **COMPLETED**: Now creates SymbolTable in Database and StackFrame in ExecutionEngine
    /// - Falls back to Universe if db not set (for gradual migration)
    ///
    /// # Implementation
    ///
    /// Creates a new scope as child of current scope:
    /// 1. Generate new scope ID (child of current_scope)
    /// 2. Create SymbolTable in Database (compile-time)
    /// 3. Push StackFrame in ExecutionEngine (runtime)
    /// 4. Update current_scope
    pub fn enter_scope(&mut self) {
        // Phase 4.5: Try to use new AIE architecture
        if let Some(db) = &self.db {
            use crate::scope::{ScopeKind, SymbolTable};

            // Generate new scope ID as child of current scope
            let new_sid = crate::scope::Sid::kid_of(&self.current_scope, "_block");

            // Create SymbolTable in Database (compile-time)
            let symbol_table = SymbolTable::new(ScopeKind::Block, new_sid.clone());
            db.borrow_mut().insert_symbol_table(new_sid.clone(), symbol_table);

            // Push StackFrame in ExecutionEngine if available
            if let Some(engine) = &self.engine {
                engine.borrow_mut().push_frame(new_sid.clone());
            }

            // Update current scope
            self.current_scope = new_sid;
        } else {
            // Fallback: Use legacy Universe during migration
            self.universe.borrow_mut().enter_scope();
            // Sync current_scope from Universe
            self.current_scope = self.universe.borrow().cur_spot.clone();
        }
    }

    /// Exit the current scope
    ///
    /// **Phase 4.5**: Bridge method - NOW uses ExecutionEngine directly!
    ///
    /// # Migration Status
    ///
    /// - ✅ **COMPLETED**: Now pops StackFrame from ExecutionEngine
    /// - Falls back to Universe if engine not set (for gradual migration)
    ///
    /// # Implementation
    ///
    /// Exits current scope:
    /// 1. Pop StackFrame from ExecutionEngine (runtime cleanup)
    /// 2. Update current_scope to parent scope
    /// 3. SymbolTable persists in Database (compile-time data)
    pub fn exit_scope(&mut self) {
        // Phase 4.5: Try to use new AIE architecture
        if let Some(engine) = &self.engine {
            // Pop StackFrame from ExecutionEngine
            if let Some(_frame_id) = engine.borrow_mut().pop_frame() {
                // Update current_scope to parent
                if let Some(parent) = self.current_scope.parent() {
                    self.current_scope = parent;
                } else {
                    // Fallback to global if no parent (shouldn't happen normally)
                    self.current_scope = crate::scope::SID_PATH_GLOBAL.clone();
                }
            }
        } else {
            // Fallback: Use legacy Universe during migration
            self.universe.borrow_mut().exit_scope();
            // Sync current_scope from Universe
            self.current_scope = self.universe.borrow().cur_spot.clone();
        }
    }

    /// Look up a symbol (function, type, variable) by name
    ///
    /// **Phase 4.5**: Bridge method - NOW uses Database directly!
    ///
    /// # Migration Status
    ///
    /// - ✅ **COMPLETED**: Now searches Database's SymbolTables
    /// - Falls back to Universe if db not set (for gradual migration)
    /// - Searches from current scope up through parent scopes
    ///
    /// # Returns
    ///
    /// - `Some(Rc<Meta>)` if symbol is found
    /// - `None` if symbol doesn't exist
    pub fn lookup_meta(&self, name: &str) -> Option<Rc<Meta>> {
        // Phase 4.5: Try to use new AIE architecture
        if let Some(db) = &self.db {
            let auto_name = auto_val::AutoStr::from(name);

            // Search from current scope up through parent scopes
            let mut search_sid = Some(self.current_scope.clone());
            while let Some(sid) = search_sid {
                if let Some(symbol_table) = db.borrow().get_symbol_table(&sid) {
                    // Check symbols
                    if let Some(meta) = symbol_table.symbols.get(&auto_name) {
                        return Some(meta.clone());
                    }
                    // Check types
                    if let Some(meta) = symbol_table.types.get(&auto_name) {
                        return Some(meta.clone());
                    }
                }

                // Move to parent scope
                search_sid = sid.parent();
            }

            // Not found in Database - fall back to Universe during migration
            // (Database is empty until stdlib registration is migrated)
            self.universe.borrow().lookup_meta(name)
        } else {
            // Fallback: Use legacy Universe during migration
            self.universe.borrow().lookup_meta(name)
        }
    }

    /// Look up a variable value by name (Phase 4.5: bridge method)
    ///
    /// **Phase 4.5**: Bridge method - NOW uses ExecutionEngine directly!
    ///
    /// # Migration Status
    ///
    /// - ✅ **COMPLETED**: Now searches ExecutionEngine's StackFrames
    /// - Falls back to Universe if engine not set (for gradual migration)
    /// - Searches from current frame up through parent frames
    ///
    /// # Returns
    ///
    /// - `Some(Value)` if variable is found
    /// - `None` if variable doesn't exist
    pub fn lookup_val(&self, name: &str) -> Option<Value> {
        // Phase 4.5: Try to use new AIE architecture
        if let Some(engine) = &self.engine {
            // Use ExecutionEngine's built-in lookup_var which searches the call stack
            if let Some(value_id) = engine.borrow().lookup_var(name) {
                // Resolve ValueID to actual Value
                // For now, return Value::ValueRef wrapper (caller can resolve)
                // TODO: In full implementation, dereference and return the actual Value
                Some(Value::ValueRef(value_id))
            } else {
                // Not found in ExecutionEngine - fall back to Universe during migration
                self.universe.borrow().lookup_val(name)
            }
        } else {
            // Fallback: Use legacy Universe during migration
            self.universe.borrow().lookup_val(name)
        }
    }

    // =========================================================================
    // Group 2: Variable Operations (Phase 4.5 Plan 064)
    // =========================================================================

    /// Set a local variable value in the current scope
    ///
    /// **Phase 4.5**: Bridge method - Uses Universe during migration.
    ///
    /// # Migration Status
    ///
    /// - ⏸️ **TODO**: Need ValueID allocation system to use ExecutionEngine
    /// - For now, uses Universe (requires Value → ValueID conversion)
    ///
    /// # Implementation
    ///
    /// Stores variable in current scope:
    /// 1. Allocates ValueID from ExecutionEngine (TODO)
    /// 2. Stores ValueID in current StackFrame
    pub fn set_local_val(&mut self, name: &str, value: Value) {
        // Phase 4.5: For now, use Universe (needs ValueID allocation system)
        self.universe.borrow_mut().set_local_val(name, value);
    }

    /// Define a symbol (function, type, variable) in the current scope
    ///
    /// **Phase 4.5**: Bridge method that uses Universe during migration.
    /// **Future**: Will use Database's SymbolTable for compile-time symbol definitions.
    ///
    /// # Migration Path
    ///
    /// - **Current**: Uses `universe.define()` for both compile-time and runtime
    /// - **Target**: Will use Database's SymbolTable for compile-time symbols
    ///
    /// # Parameters
    ///
    /// - `name`: Symbol name (converts to AutoStr)
    /// - `meta`: Symbol metadata (function, type, variable reference)
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Old (Universe)
    /// self.universe.borrow_mut().define("my_func", Rc::new(Meta::Fn(...)));
    ///
    /// // New (Phase 4.5+)
    /// self.define("my_func", Rc::new(Meta::Fn(...)));  // Bridge method
    ///
    /// // Future (Phase 4.6)
    /// self.db.define_symbol(sid, "my_func", Symbol::Fn(...));
    /// ```
    pub fn define(&mut self, name: impl Into<AutoStr>, meta: Rc<Meta>) {
        self.universe.borrow_mut().define(name, meta);
    }

    /// Remove a local variable from the current scope
    ///
    /// **Phase 4.5**: Bridge method that uses Universe during migration.
    /// **Future**: Will use ExecutionEngine's StackFrame for runtime variable removal.
    ///
    /// # Returns
    ///
    /// - `Some(ValueID)` if variable was removed
    /// - `None` if variable didn't exist
    ///
    /// # Migration Path
    ///
    /// - **Current**: Uses `universe.remove_local()` for both compile-time and runtime
    /// - **Target**: Will use `engine.current_frame().remove(name)` for runtime
    pub fn remove_local(&mut self, name: &str) -> Option<auto_val::ValueID> {
        self.universe.borrow_mut().remove_local(name)
    }

    /// Set a global variable value
    ///
    /// **Phase 4.5**: Bridge method that uses Universe during migration.
    /// **Future**: Will use Database for global variable storage.
    ///
    /// # Migration Path
    ///
    /// - **Current**: Uses `universe.set_global()` for global variables
    /// - **Target**: Will use Database's global symbol table
    ///
    /// # Parameters
    ///
    /// - `name`: Global variable name
    /// - `value`: Value to store
    pub fn set_global(&mut self, name: impl Into<String>, value: Value) {
        self.universe.borrow_mut().set_global(name, value);
    }

    // =========================================================================
    // Group 3: Type Operations (Phase 4.5 Plan 064)
    // =========================================================================

    /// Define a type in the current scope
    ///
    /// **Phase 4.5**: Bridge method that uses Universe during migration.
    /// **Future**: Will use Database's SymbolTable for compile-time type definitions.
    ///
    /// # Migration Path
    ///
    /// - **Current**: Uses `universe.define_type()` for both compile-time and runtime
    /// - **Target**: Will use Database's SymbolTable for type storage
    ///
    /// # Parameters
    ///
    /// - `name`: Type name (converts to AutoStr)
    /// - `meta`: Type metadata (TypeDecl, enum variants, etc.)
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Old (Universe)
    /// self.universe.borrow_mut().define_type("MyType", Rc::new(Meta::Type(...)));
    ///
    /// // New (Phase 4.5+)
    /// self.define_type("MyType", Rc::new(Meta::Type(...)));  // Bridge method
    ///
    /// // Future (Phase 4.6)
    /// self.db.define_type(sid, "MyType", type_decl);
    /// ```
    pub fn define_type(&mut self, name: impl Into<AutoStr>, meta: Rc<Meta>) {
        self.universe.borrow_mut().define_type(name, meta);
    }

    // =========================================================================
    // Group 4: VM Operations (Phase 4.5 Plan 064)
    // =========================================================================

    /// Allocate a new VM reference (HashMap, HashSet, File, List, etc.)
    ///
    /// **Phase 4.5**: Bridge method that uses Universe during migration.
    /// **Future**: Will use ExecutionEngine's VM reference management.
    ///
    /// # Migration Path
    ///
    /// - **Current**: Uses `universe.add_vmref()` for VM resource allocation
    /// - **Target**: Will use `engine.alloc_vm_ref()` for runtime VM resources
    ///
    /// # Returns
    ///
    /// The VM reference ID (usize)
    ///
    /// # Parameters
    ///
    /// - `data`: VM reference data (List, HashMap, HashSet, File, etc.)
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Old (Universe)
    /// let list_data = ListData::new();
    /// let ref_id = self.universe.borrow_mut().add_vmref(VmRefData::List(list_data));
    ///
    /// // New (Phase 4.5+)
    /// let list_data = ListData::new();
    /// let ref_id = self.alloc_vmref(VmRefData::List(list_data));  // Bridge method
    ///
    /// // Future (Phase 4.6)
    /// let list_data = ListData::new();
    /// let ref_id = self.engine.alloc_vm_ref(VmRefData::List(list_data));
    /// ```
    pub fn alloc_vmref(&mut self, data: crate::universe::VmRefData) -> usize {
        self.universe.borrow_mut().add_vmref(data)
    }

    // NOTE: get_vmref() is not provided as a bridge method due to lifetime issues.
    // During migration, use `self.universe.borrow().get_vmref_ref(refid)` directly.
    // In Phase 4.6, this will be replaced with `self.engine.get_vm_ref(refid)`.

    // =========================================================================
    // Additional Helper Bridge Methods (Phase 4.5 Plan 064)
    // =========================================================================

    /// Check if a local variable exists in current scope
    ///
    /// **Phase 4.5**: Bridge method that uses Universe during migration.
    pub fn has_local(&self, name: &str) -> bool {
        self.universe.borrow().has_local(name)
    }

    /// Clear moved status for a variable
    ///
    /// **Phase 4.5**: Bridge method that uses Universe during migration.
    pub fn clear_moved(&mut self, name: &str) {
        self.universe.borrow_mut().clear_moved(name);
    }

    /// Check if a variable exists (in any scope)
    ///
    /// **Phase 4.5**: Bridge method that uses Universe during migration.
    pub fn exists(&self, name: &str) -> bool {
        self.universe.borrow().exists(name)
    }

    /// Update a variable's value
    ///
    /// **Phase 4.5**: Bridge method that uses Universe during migration.
    pub fn update_val(&mut self, name: &str, value: Value) {
        self.universe.borrow_mut().update_val(name, value);
    }

    /// Get all defined variable names
    ///
    /// **Phase 4.5**: Bridge method that uses Universe during migration.
    pub fn get_defined_names(&self) -> Vec<String> {
        self.universe.borrow().get_defined_names()
    }

    /// Check if there's an argument with the given name
    ///
    /// **Phase 4.5**: Bridge method that uses Universe during migration.
    pub fn has_arg(&self, name: &str) -> bool {
        self.universe.borrow().has_arg(name)
    }

    /// Get an argument value by name
    ///
    /// **Phase 4.5**: Bridge method that uses Universe during migration.
    pub fn get_arg(&self, name: &str) -> Value {
        self.universe.borrow().get_arg(name)
    }

    /// Allocate a new value ID (Phase 4.5: bridge method)
    ///
    /// **Phase 4.5**: Bridge method that uses Universe during migration.
    pub fn alloc_value(&mut self, data: auto_val::ValueData) -> auto_val::ValueID {
        self.universe.borrow_mut().alloc_value(data)
    }

    /// Dereference a Value to its actual data (Phase 4.5: bridge method)
    ///
    /// **Phase 4.5**: Bridge method that uses Universe during migration.
    pub fn deref_val(&self, val: Value) -> Value {
        self.universe.borrow().deref_val(val)
    }

    /// Look up a type by name (Phase 4.5: bridge method)
    ///
    /// **Phase 4.5**: Bridge method that uses Universe during migration.
    pub fn lookup_type(&self, name: &str) -> ast::Type {
        self.universe.borrow().lookup_type(name)
    }

    /// Mark a variable as moved (Phase 4.5: bridge method)
    ///
    /// **Phase 4.5**: Bridge method that uses Universe during migration.
    pub fn mark_moved(&mut self, name: &str) {
        self.universe.borrow_mut().mark_moved(name);
    }

    /// Enter a function scope (Phase 4.5: bridge method)
    ///
    /// **Phase 4.5**: Bridge method that uses Universe during migration.
    pub fn enter_fn(&mut self, name: impl Into<AutoStr>) {
        self.universe.borrow_mut().enter_fn(name);
    }

    /// Set local object properties (Phase 4.5: bridge method)
    ///
    /// **Phase 4.5**: Bridge method that uses Universe during migration.
    pub fn set_local_obj(&mut self, obj: &Obj) {
        self.universe.borrow_mut().set_local_obj(obj);
    }

    /// Register a spec declaration (Phase 4.5: bridge method)
    ///
    /// **Phase 4.5**: Bridge method that uses Universe during migration.
    pub fn register_spec(&mut self, spec: std::rc::Rc<ast::SpecDecl>) {
        self.universe.borrow_mut().register_spec(spec);
    }

    /// Get a spec declaration by name (Phase 4.5: bridge method)
    ///
    /// **Phase 4.5**: Bridge method that uses Universe during migration.
    pub fn get_spec(&self, name: &AutoStr) -> Option<std::rc::Rc<ast::SpecDecl>> {
        self.universe.borrow().specs.get(name).cloned()
    }

    // =========================================================================
    // Evaluation Methods
    // =========================================================================

    pub fn eval(&mut self, code: &Code) -> AutoResult<Value> {
        match self.mode {
            EvalMode::SCRIPT => {
                let mut value = Value::Nil;
                for stmt in code.stmts.iter() {
                    value = self.eval_stmt(stmt)?;
                    // Don't panic on errors - let them propagate as error values
                    // This allows tests to check for errors using Result::Err
                }

                // Automatically call main() if it's defined
                // This allows test code to define fn main() {...} and have it execute
                let main_fn = self.lookup_meta("main");  // Phase 4.5: Use bridge method
                if let Some(main_meta) = main_fn {
                    if let scope::Meta::Fn(fn_decl) = main_meta.as_ref() {
                        // Call main() with no arguments
                        value = self.eval_fn_call(fn_decl, &ast::Args::new())?;
                    }
                }

                Ok(value)
            }
            EvalMode::CONFIG => {
                if code.stmts.len() == 1 {
                    let first_val = self.eval_stmt(&code.stmts[0])?;
                    // For Array, we need to process it to consolidate nodes
                    if matches!(first_val, Value::Array(_)) {
                        // Process the array using the same logic as multi-statement case
                        match first_val {
                            Value::Array(arr) => {
                                use auto_val::Array;
                                use std::collections::HashMap;

                                // First, check if array contains any config items (nodes, pairs, objs, instances)
                                // If not, return the array as-is (pure value array)
                                let has_config_items = arr.values.iter().any(|item| {
                                    matches!(
                                        item,
                                        Value::Node(_)
                                            | Value::Pair(_, _)
                                            | Value::Obj(_)
                                            | Value::Instance(_)
                                    )
                                });

                                if !has_config_items {
                                    return Ok(Value::Array(arr));
                                }

                                // Has config items, need to consolidate
                                let mut nodes_by_name: HashMap<AutoStr, Vec<auto_val::Node>> =
                                    HashMap::new();
                                let mut other_items: Vec<Value> = Vec::new();

                                // First pass: separate nodes from other items
                                for item in arr.values.into_iter() {
                                    match item {
                                        Value::Node(n) => {
                                            nodes_by_name
                                                .entry(n.name.clone())
                                                .or_default()
                                                .push(n);
                                        }
                                        Value::Pair(key, value) => {
                                            let mut node = auto_val::Node::new("root");
                                            node.set_prop(key, *value);
                                            return Ok(Value::Node(node));
                                        }
                                        Value::Obj(o) => {
                                            let mut node = auto_val::Node::new("root");
                                            node.merge_obj(o);
                                            return Ok(Value::Node(node));
                                        }
                                        Value::Instance(inst) => {
                                            // Convert instance to node with type name as node name
                                            let mut kid_node = auto_val::Node::new(&inst.ty.name());
                                            // Add instance fields as node properties
                                            for (k, v) in inst.fields.iter() {
                                                kid_node.set_prop(k.clone(), v.clone());
                                            }
                                            nodes_by_name
                                                .entry(kid_node.name.clone())
                                                .or_default()
                                                .push(kid_node);
                                        }
                                        _ => {
                                            if !item.is_void() {
                                                other_items.push(item);
                                            }
                                        }
                                    }
                                }

                                // Consolidate into a root node
                                let mut node = auto_val::Node::new("root");

                                // Second pass: add consolidated nodes
                                for (name, nodes) in nodes_by_name.into_iter() {
                                    if nodes.len() == 1 {
                                        // Single node: add as kid
                                        node.add_kid(nodes.into_iter().next().unwrap());
                                    } else {
                                        // Multiple nodes with same name: create plural form property
                                        let plural_name = format!("{}s", name); // dir -> dirs
                                        let node_values: Vec<Value> =
                                            nodes.into_iter().map(|n| Value::Node(n)).collect();
                                        node.set_prop(
                                            plural_name,
                                            Value::Array(Array::from_vec(node_values)),
                                        );
                                    }
                                }

                                // Handle remaining non-node items
                                for item in other_items.into_iter() {
                                    node.set_prop(item.to_astr(), item);
                                }

                                return Ok(Value::Node(node));
                            }
                            _ => unreachable!(),
                        }
                    }
                    match first_val {
                        Value::Obj(_) => {
                            return Ok(first_val);
                        }
                        Value::Node(n) => {
                            let mut node = auto_val::Node::new("root");
                            node.add_kid(n);
                            return Ok(Value::Node(node));
                        }
                        Value::Pair(k, v) => {
                            let mut node = auto_val::Node::new("root");
                            node.set_prop(k, *v);
                            return Ok(Value::Node(node));
                        }
                        _ => {
                            return Err(Error::new(
                                ErrorKind::InvalidInput,
                                "Invalid configuration statement",
                            )
                            .into());
                        }
                    }
                }
                let mut node = auto_val::Node::new("root");
                for stmt in code.stmts.iter() {
                    let val = self.eval_stmt(stmt)?;
                    match val {
                        Value::Pair(key, value) => {
                            // first level pairs are viewed as variable declarations
                            // TODO: this should only happen in a Config scenario
                            let mut value = *value;
                            if let Some(name) = key.name() {
                                // Phase 4.5: Use bridge methods instead of direct Universe access
                                if self.has_arg(name) {
                                    let arg_val = self.get_arg(name);
                                    // println!(
                                    // "replacing value of {} from {} to {}",
                                    // name, value, arg_val
                                    // );
                                    value = arg_val;
                                }
                                self.set_local_val(name, value.clone());
                            }
                            node.set_prop(key, value);
                        }
                        Value::Obj(o) => {
                            node.merge_obj(o);
                        }
                        Value::Node(n) => {
                            node.add_kid(n);
                        }
                        Value::Block(arr) => {
                            for item in arr.values.into_iter() {
                                match item {
                                    Value::Node(n) => {
                                        node.add_kid(n);
                                    }
                                    Value::Pair(key, value) => {
                                        node.set_prop(key, *value);
                                    }
                                    Value::Obj(o) => {
                                        node.merge_obj(o);
                                    }
                                    Value::Instance(inst) => {
                                        // Convert instance to node with type name as node name
                                        let mut kid_node = auto_val::Node::new(&inst.ty.name());
                                        // Add instance fields as node properties
                                        for (k, v) in inst.fields.iter() {
                                            kid_node.set_prop(k.clone(), v.clone());
                                        }
                                        node.add_kid(kid_node);
                                    }
                                    _ => {}
                                }
                            }
                        }
                        Value::Array(arr) => {
                            use auto_val::Array;
                            use std::collections::HashMap;

                            // Group nodes by name for consolidation
                            let mut nodes_by_name: HashMap<AutoStr, Vec<auto_val::Node>> =
                                HashMap::new();
                            let mut other_items: Vec<Value> = Vec::new();

                            // First pass: separate nodes from other items
                            for item in arr.values.into_iter() {
                                match item {
                                    Value::Node(n) => {
                                        nodes_by_name.entry(n.name.clone()).or_default().push(n);
                                    }
                                    Value::Pair(key, value) => {
                                        node.set_prop(key, *value);
                                    }
                                    Value::Obj(o) => {
                                        node.merge_obj(o);
                                    }
                                    Value::Instance(inst) => {
                                        // Convert instance to node with type name as node name
                                        let mut kid_node = auto_val::Node::new(&inst.ty.name());
                                        // Add instance fields as node properties
                                        for (k, v) in inst.fields.iter() {
                                            kid_node.set_prop(k.clone(), v.clone());
                                        }
                                        nodes_by_name
                                            .entry(kid_node.name.clone())
                                            .or_default()
                                            .push(kid_node);
                                    }
                                    _ => {
                                        if !item.is_void() {
                                            other_items.push(item);
                                        }
                                    }
                                }
                            }

                            // Second pass: add consolidated nodes
                            for (name, nodes) in nodes_by_name.into_iter() {
                                if nodes.len() == 1 {
                                    // Single node: add as kid
                                    node.add_kid(nodes.into_iter().next().unwrap());
                                } else {
                                    // Multiple nodes with same name: create plural form property
                                    let plural_name = format!("{}s", name); // dir -> dirs
                                    let node_values: Vec<Value> =
                                        nodes.into_iter().map(|n| Value::Node(n)).collect();
                                    node.set_prop(
                                        plural_name,
                                        Value::Array(Array::from_vec(node_values)),
                                    );
                                }
                            }

                            // Handle remaining non-node items
                            for item in other_items.into_iter() {
                                node.set_prop(item.to_astr(), item);
                            }
                        }
                        _ => {}
                    }
                }
                Ok(Value::Node(node))
            }
            EvalMode::TEMPLATE => {
                let mut result = Vec::new();
                for stmt in code.stmts.iter() {
                    let val = self.eval_stmt(stmt)?;
                    if !val.is_nil() {
                        result.push(val.to_astr());
                    }
                }
                Ok(Value::Str(result.join("\n").into()))
            }
        }
    }

    pub fn dump_scope(&self) {
        self.universe.borrow().dump();
    }

    pub fn eval_stmt(&mut self, stmt: &Stmt) -> AutoResult<Value> {
        match stmt {
            Stmt::Use(use_stmt) => Ok(self.eval_use(use_stmt)),
            Stmt::Expr(expr) => Ok(self.eval_expr(expr)),
            Stmt::If(if_) => self.eval_if(if_),
            Stmt::For(for_stmt) => self.eval_for(for_stmt),
            Stmt::Block(body) => self.eval_body(body),
            Stmt::Store(store) => Ok(self.eval_store(store)),
            Stmt::Fn(_) => Ok(Value::Nil),
            Stmt::TypeDecl(type_decl) => Ok(self.type_decl(type_decl)),
            Stmt::Node(node) => self.eval_node(node),
            Stmt::Is(stmt) => self.eval_is(stmt),
            Stmt::EnumDecl(_) => Ok(Value::Nil),
            Stmt::OnEvents(on) => Ok(self.eval_on_events(on)),
            Stmt::Comment(_) => Ok(Value::Nil),
            Stmt::Alias(_) => Ok(Value::Void),
            Stmt::TypeAlias(_) => Ok(Value::Void), // Type aliases are compile-time only
            Stmt::EmptyLine(_) => Ok(Value::Void),
            Stmt::Union(_) => Ok(Value::Void),
            Stmt::Tag(tag) => Ok(self.eval_tag_decl(tag)),
            Stmt::SpecDecl(spec_decl) => Ok(self.spec_decl(spec_decl)),
            Stmt::Break => Ok(Value::Void),
            Stmt::Return(expr) => Ok(self.eval_expr(expr)),
            Stmt::Ext(ext) => Ok(self.eval_ext(ext)),
        }
    }

    fn eval_use(&mut self, use_: &Use) -> Value {
        match use_.kind {
            ast::UseKind::Auto => self.eval_use_auto(use_),
            ast::UseKind::C => self.eval_use_c(use_),
            ast::UseKind::Rust => self.eval_use_rust(use_),
        }
    }

    fn eval_use_auto(&mut self, use_stmt: &ast::Use) -> Value {
        // Construct module path from paths (e.g., ["auto", "io"] -> "auto.io")
        let module_path = use_stmt.paths.join(".");

        // Check if module exists in VM registry
        let registry = crate::vm::VM_REGISTRY.lock().unwrap();
        let module = match registry.get_module(&module_path) {
            Some(m) => m,
            None => {
                return Value::Error(format!("Module '{}' not found", module_path).into());
            }
        };

        // Register all types from this module in the universe
        // (Types need to be available even if not explicitly imported)
        for (type_name, _type_entry) in module.types.iter() {
            let type_decl = ast::TypeDecl {
                name: type_name.clone(),
                kind: ast::TypeDeclKind::UserType,
                parent: None,
                has: vec![],
                specs: vec![],
                spec_impls: vec![], // Plan 057
                generic_params: vec![],
                members: vec![],
                delegations: vec![],
                methods: vec![],
            };
            self.define_type(
                type_name.clone(),
                std::rc::Rc::new(crate::scope::Meta::Type(ast::Type::User(type_decl))),
            );  // Phase 4.5: Use bridge method
        }
        drop(registry);

        // Register each imported item in current scope
        for item_name in &use_stmt.items {
            // Check if it's a function or type
            // IMPORTANT: Extract data with short-lived lock to avoid deadlock
            // The lock must be released BEFORE calling universe.borrow_mut().define()
            let is_function = {
                let registry = crate::vm::VM_REGISTRY.lock().unwrap();
                registry.get_function(&module_path, item_name).is_some()
            };

            if is_function {
                // Create a VmFunction metadata entry
                let fn_decl = ast::Fn::new(
                    ast::FnKind::VmFunction,
                    item_name.clone(),
                    None,
                    vec![],
                    ast::Body::new(),
                    ast::Type::Unknown,
                );

                // Register in current scope
                self.define(
                    item_name.clone(),
                    std::rc::Rc::new(crate::scope::Meta::Fn(fn_decl)),
                );  // Phase 4.5: Use bridge method
            } else {
                // Check if it's a type (with short-lived lock)
                let has_type = {
                    let registry = crate::vm::VM_REGISTRY.lock().unwrap();
                    registry
                        .get_module(&module_path)
                        .and_then(|module| module.types.contains_key(item_name).then_some(true))
                        .unwrap_or(false)
                };

                if has_type {
                    // Import the type as Value::Type
                    let type_decl = ast::TypeDecl {
                        name: item_name.clone(),
                        kind: ast::TypeDeclKind::UserType,
                        parent: None,
                        has: vec![],
                        specs: vec![],
                        spec_impls: vec![], // Plan 057
                        generic_params: vec![],
                        members: vec![],
                        delegations: vec![],
                        methods: vec![],
                    };

                    self.define(
                        item_name.clone(),
                        std::rc::Rc::new(crate::scope::Meta::Type(ast::Type::User(type_decl))),
                    );  // Phase 4.5: Use bridge method
                }
            }
        }

        Value::Void
    }

    fn eval_use_c(&mut self, _use_stmt: &ast::Use) -> Value {
        // TODO: Implement C library loading
        Value::Void
    }

    fn eval_use_rust(&mut self, _use_stmt: &ast::Use) -> Value {
        // TODO: Implement Rust library loading
        Value::Void
    }

    fn collect_config_body(&mut self, vals: Vec<Value>) -> Vec<Value> {
        vals
    }

    fn eval_body(&mut self, body: &Body) -> AutoResult<Value> {
        self.enter_scope();
        let mut res = Vec::new();
        for stmt in body.stmts.iter() {
            // Handle return statements - evaluate and immediately exit
            if let Stmt::Return(expr) = stmt {
                let value = self.eval_expr(expr);
                self.exit_scope();
                return Ok(value);
            }
            res.push(self.eval_stmt(stmt)?);
        }
        let res = match self.mode {
            EvalMode::SCRIPT => Ok(res.last().unwrap_or(&Value::Nil).clone()),
            EvalMode::CONFIG => Ok(Value::Block(Array::from_vec(self.collect_config_body(res)))),
            EvalMode::TEMPLATE => Ok(Value::Str(
                res.iter()
                    .map(|v| match v {
                        Value::Str(s) => s.clone(),
                        _ => v.to_astr(),
                    })
                    .collect::<Vec<AutoStr>>()
                    .join("\n")
                    .into(),
            )),
        };
        self.exit_scope();
        res
    }

    fn eval_loop_body(
        &mut self,
        body: &Body,
        is_mid: bool,
        is_new_line: bool,
    ) -> AutoResult<Value> {
        // Phase 4.5: Use bridge method
        self.set_local_val("is_mid", Value::Bool(is_mid));
        let mut res = Vec::new();
        let sep = if is_new_line { "\n" } else { "" };
        for stmt in body.stmts.iter() {
            res.push(self.eval_stmt(stmt)?);
        }
        Ok(match self.mode {
            EvalMode::SCRIPT => res.last().unwrap_or(&Value::Nil).clone(),
            EvalMode::CONFIG => Value::Array(Array::from_vec(self.collect_config_body(res))),
            EvalMode::TEMPLATE => Value::Str(
                res.into_iter()
                    .filter(|v| !v.is_nil())
                    .map(|v| match v {
                        Value::Str(s) => s.clone(),
                        _ => v.to_astr(),
                    })
                    .collect::<Vec<AutoStr>>()
                    .join(sep)
                    .into(),
            ),
        })
    }

    fn eval_is(&mut self, stmt: &Is) -> AutoResult<Value> {
        let t = &stmt.target;
        for br in &stmt.branches {
            match br {
                IsBranch::EqBranch(expr, body) => {
                    // Check if this is a tag pattern match
                    if let Expr::Cover(cover) = &expr {
                        // Cover::Tag is the only variant
                        let tag_cover = match cover {
                            ast::Cover::Tag(tc) => tc,
                        };
                        // Tag pattern matching: is atom { Atom.Int(i) => ... }
                        if self.matches_tag_pattern(t, tag_cover)? {
                            // Bind the element variable if present
                            if tag_cover.elem != "" {
                                let target_val = self.eval_expr(t);
                                if let Value::Node(node) = target_val {
                                    let payload = node.get_prop("payload");
                                    self.universe
                                        .borrow_mut()
                                        .set_local_val(tag_cover.elem.as_str(), payload);
                                }
                            }
                            return self.eval_body(&body);
                        } else {
                            continue; // Try next branch
                        }
                    }

                    // Regular equality comparison
                    // Resolve ValueRefs before comparison
                    let target_val = self.eval_expr(t);
                    let expr_val = self.eval_expr(&expr);

                    let target_resolved = self.resolve_or_clone(&target_val);
                    let expr_resolved = self.resolve_or_clone(&expr_val);

                    // Convert back to Value for comparison
                    let target_value = Value::from_data(target_resolved);
                    let expr_value = Value::from_data(expr_resolved);

                    let cond = target_value == expr_value;
                    if cond {
                        return self.eval_body(&body);
                    }
                }
                // TODO: implement other types of is-branch
                _ => {
                    return Ok(Value::Void);
                }
            }
        }
        Ok(Value::Void)
    }

    fn matches_tag_pattern(&mut self, target: &Expr, pattern: &ast::TagCover) -> AutoResult<bool> {
        // Evaluate the target to get the tag value
        let target_val = self.eval_expr(target);

        // Check if target is a Node with tag properties
        if let Value::Node(node) = target_val {
            let tag_name = &node.name;
            let variant = node.get_prop("variant");

            // Match tag kind (e.g., "Atom" in Atom.Int)
            if tag_name != &pattern.kind {
                return Ok(false);
            }

            // Match variant name (e.g., "Int" in Atom.Int)
            let variant_str = variant.to_string();
            if variant_str != pattern.tag {
                return Ok(false);
            }

            // Pattern matches!
            return Ok(true);
        }

        Ok(false)
    }

    fn eval_if(&mut self, if_: &If) -> AutoResult<Value> {
        for branch in if_.branches.iter() {
            let cond = self.eval_expr(&branch.cond);

            // Resolve ValueRef before checking truthiness
            let cond_is_true = match &cond {
                Value::ValueRef(_vid) => {
                    if let Some(data) = self.resolve_value(&cond) {
                        let borrowed_data = data.borrow();
                        match &*borrowed_data {
                            ValueData::Bool(b) => *b,
                            ValueData::Int(i) => *i > 0,
                            ValueData::Uint(u) => *u > 0,
                            ValueData::Float(f) => *f > 0.0,
                            ValueData::Str(s) => s.len() > 0,
                            ValueData::Byte(b) => *b > 0,
                            _ => false,
                        }
                    } else {
                        false
                    }
                }
                _ => cond.is_true(),
            };

            if cond_is_true {
                return self.eval_body(&branch.body);
            }
        }
        if let Some(else_stmt) = &if_.else_ {
            return self.eval_body(else_stmt);
        }
        Ok(Value::Void)
    }

    fn eval_iter(&mut self, iter: &Iter, idx: usize, item: Value) {
        match iter {
            Iter::Indexed(index, iter) => {
                self.set_local_val(&index, Value::Int(idx as i32));  // Phase 4.5: Use bridge method
                // println!("set index {}, iter: {}, item: {}", index.text, iter.text, item.clone());
                self.set_local_val(&iter, item);  // Phase 4.5: Use bridge method
            }
            Iter::Named(iter) => self.set_local_val(&iter, item),  // Phase 4.5: Use bridge method
            Iter::Call(_) => {
                todo!()
            }
            Iter::Ever => {
                // No iteration variable for infinite loops
            }
            Iter::Cond => {
                // No iteration variable for conditional loops
            }
        }
    }

    fn eval_for(&mut self, for_stmt: &For) -> AutoResult<Value> {
        let iter = &for_stmt.iter;
        let body = &for_stmt.body;
        let mut max_loop = 1000;

        // Execute init statement if present
        if let Some(init_stmt) = &for_stmt.init {
            self.eval_stmt(init_stmt)?;
        }

        // Handle conditional for loop: for condition { ... }
        if matches!(iter, Iter::Cond) {
            let mut res = Array::new();

            // Only enter a new scope if there's an initializer
            // For simple conditionals like "for i < max { ... }", use outer scope
            let has_init = for_stmt.init.is_some();
            if has_init {
                self.enter_scope();  // Phase 4.5: Use bridge method
            }

            loop {
                if max_loop <= 0 {
                    if has_init {
                        self.exit_scope();  // Phase 4.5: Use bridge method
                    }
                    return Ok(Value::error("Max loop reached"));
                }
                max_loop -= 1;

                let cond = self.eval_expr(&for_stmt.range);
                let cond_is_true = cond.is_true();

                if !cond_is_true {
                    break;
                }

                match self.eval_loop_body(body, false, for_stmt.new_line) {
                    Ok(val) => {
                        if let Value::Array(arr) = &val {
                            res.extend(arr);
                        } else {
                            res.push(val);
                        }
                    }
                    Err(e) => {
                        if has_init {
                            self.exit_scope();  // Phase 4.5: Use bridge method
                        }
                        return Err(e);
                    }
                }
            }
            if has_init {
                self.exit_scope();  // Phase 4.5: Use bridge method
            }

            return Ok(match self.mode {
                EvalMode::SCRIPT => Value::Void,
                EvalMode::CONFIG => Value::Array(res),
                EvalMode::TEMPLATE => Value::Str(
                    res.iter()
                        .filter(|v| match v {
                            Value::Nil => false,
                            _ => true,
                        })
                        .map(|v| v.to_string())
                        .collect::<Vec<_>>()
                        .join("")
                        .into(),
                ),
            });
        }

        let range = self.eval_expr(&for_stmt.range);

        // Resolve ValueRef for range/array operations
        let range_resolved = match &range {
            Value::ValueRef(_vid) => {
                if let Some(data) = self.resolve_value(&range) {
                    let borrowed_data = data.borrow();
                    let data_clone = borrowed_data.clone();
                    drop(borrowed_data);
                    Some(Value::from_data(data_clone))
                } else {
                    None
                }
            }
            _ => Some(range.clone()),
        };

        let range_final = match range_resolved {
            Some(v) => v,
            None => return Ok(Value::error(format!("Invalid range {}", range))),
        };

        let mut res = Array::new();
        let mut is_mid = true;
        let is_new_line = for_stmt.new_line;
        let sep = if for_stmt.new_line { "\n" } else { "" };
        self.enter_scope();  // Phase 4.5: Use bridge method
        match range_final {
            Value::Range(start, end) => {
                let len = (end - start) as usize;
                for (idx, n) in (start..end).enumerate() {
                    if idx == len - 1 {
                        is_mid = false;
                    }
                    self.eval_iter(iter, idx, Value::Int(n));
                    match self.eval_loop_body(body, is_mid, is_new_line) {
                        Ok(val) => {
                            if let Value::Array(arr) = &val {
                                res.extend(arr);
                            } else {
                                res.push(val);
                            }
                        }
                        Err(e) => {
                            self.exit_scope();  // Phase 4.5: Use bridge method
                            return Err(e);
                        }
                    }
                    max_loop -= 1;
                }
            }
            Value::RangeEq(start, end) => {
                let len = (end - start) as usize;
                for (idx, n) in (start..=end).enumerate() {
                    if idx == len - 1 {
                        is_mid = false;
                    }
                    self.eval_iter(iter, idx, Value::Int(n));
                    match self.eval_loop_body(body, is_mid, is_new_line) {
                        Ok(val) => {
                            if let Value::Array(arr) = &val {
                                res.extend(arr);
                            } else {
                                res.push(val);
                            }
                        }
                        Err(e) => {
                            self.exit_scope();  // Phase 4.5: Use bridge method
                            return Err(e);
                        }
                    }
                    max_loop -= 1;
                }
            }
            Value::Array(values) => {
                let len = values.len();
                for (idx, item) in values.iter().enumerate() {
                    if idx == len - 1 {
                        is_mid = false;
                    }
                    self.eval_iter(iter, idx, item.clone());
                    match self.eval_loop_body(body, is_mid, is_new_line) {
                        Ok(val) => {
                            if let Value::Array(arr) = &val {
                                res.extend(arr);
                            } else {
                                res.push(val);
                            }
                        }
                        Err(e) => {
                            self.exit_scope();  // Phase 4.5: Use bridge method
                            return Err(e);
                        }
                    }
                    max_loop -= 1;
                }
            }
            Value::Instance(ref instance) => {
                // Handle iteration over VM instances (e.g., List)
                if let auto_val::Type::User(ref_name) = &instance.ty {
                    if ref_name.as_str() == "List" {
                        // Get the list ID and retrieve elements
                        if let Some(Value::USize(list_id)) = instance.fields.get("id") {
                            // Clone the list elements to avoid holding the borrow across the loop
                            let list_elems = {
                                let uni = self.universe.borrow();
                                if let Some(vmref) = uni.get_vmref_ref(list_id) {
                                    let ref_box = vmref.borrow();
                                    if let crate::universe::VmRefData::List(list_data) = &*ref_box {
                                        list_data.elems.clone()
                                    } else {
                                        vec![]
                                    }
                                } else {
                                    vec![]
                                }
                            };

                            let len = list_elems.len();
                            for (idx, item) in list_elems.iter().enumerate() {
                                if max_loop <= 0 {
                                    self.exit_scope();  // Phase 4.5: Use bridge method
                                    return Ok(Value::error("Max loop reached"));
                                }

                                if idx == len - 1 {
                                    is_mid = false;
                                }
                                self.eval_iter(iter, idx, item.clone());
                                match self.eval_loop_body(body, is_mid, is_new_line) {
                                    Ok(val) => {
                                        if let Value::Array(arr) = &val {
                                            res.extend(arr);
                                        } else {
                                            res.push(val);
                                        }
                                    }
                                    Err(e) => {
                                        self.exit_scope();  // Phase 4.5: Use bridge method
                                        return Err(e);
                                    }
                                }
                                max_loop -= 1;
                            }
                            self.exit_scope();  // Phase 4.5: Use bridge method
                            return Ok(match self.mode {
                                EvalMode::SCRIPT => Value::Void,
                                EvalMode::CONFIG => Value::Array(res),
                                EvalMode::TEMPLATE => Value::Str(
                                    res.iter()
                                        .filter(|v| match v {
                                            Value::Nil => false,
                                            Value::Str(s) => !s.is_empty(),
                                            _ => true,
                                        })
                                        .map(|v| v.to_astr())
                                        .collect::<Vec<AutoStr>>()
                                        .join(sep)
                                        .into(),
                                ),
                            });
                        }
                    }
                }
                return Ok(Value::error(format!(
                    "Cannot iterate over instance of type {:?}",
                    instance.ty
                )));
            }
            _ => {
                return Ok(Value::error(format!("Invalid range {}", range_final)));
            }
        }
        self.exit_scope();  // Phase 4.5: Use bridge method
        if max_loop <= 0 {
            Ok(Value::error("Max loop reached"))
        } else {
            Ok(match self.mode {
                EvalMode::SCRIPT => Value::Void,
                EvalMode::CONFIG => Value::Array(res),
                EvalMode::TEMPLATE => Value::Str(
                    res.iter()
                        .filter(|v| match v {
                            Value::Nil => false,
                            Value::Str(s) => !s.is_empty(),
                            _ => true,
                        })
                        .map(|v| v.to_astr())
                        .collect::<Vec<AutoStr>>()
                        .join(sep)
                        .into(),
                ),
            })
        }
    }

    fn eval_store(&mut self, store: &Store) -> Value {
        // Plan 052: Handle runtime array allocation and uninitialized arrays
        let value = if matches!(store.ty, ast::Type::RuntimeArray(_)) {
            // For runtime arrays, evaluate the size expression and allocate
            if let ast::Type::RuntimeArray(rta) = &store.ty {
                // Evaluate size expression
                let size_value = self.eval_expr(&rta.size_expr);
                let size = match size_value {
                    Value::Int(n) => n as usize,
                    Value::Uint(n) => n as usize,
                    Value::U8(n) => n as usize,
                    Value::I8(n) => n as usize,
                    _ => {
                        eprintln!(
                            "Runtime array size must be an integer, got: {:?}",
                            size_value
                        );
                        return Value::Error(
                            format!("Runtime array size must be an integer").into(),
                        );
                    }
                };

                // Determine element type for proper initialization
                let default_val = match &*rta.elem {
                    ast::Type::Int => Value::Int(0),
                    ast::Type::Uint => Value::Uint(0),
                    ast::Type::Float => Value::Float(0.0),
                    ast::Type::Double => Value::Double(0.0),
                    ast::Type::Bool => Value::Bool(false),
                    ast::Type::Byte => Value::Byte(0),
                    _ => Value::Int(0), // Default to Int(0) for unknown types
                };

                // Create array with specified size, initialized to default value
                let mut elems = Vec::with_capacity(size);
                elems.resize(size, default_val);
                Value::array(elems)
            } else {
                unreachable!()
            }
        } else if matches!(store.ty, ast::Type::Array(_)) && matches!(store.expr, Expr::Nil) {
            // For uninitialized regular arrays (e.g., `mut arr [3]int` without `=`),
            // create an array initialized to zeros
            if let ast::Type::Array(array_type) = &store.ty {
                // Determine element type for proper initialization
                let default_val = match &*array_type.elem {
                    ast::Type::Int => Value::Int(0),
                    ast::Type::Uint => Value::Uint(0),
                    ast::Type::Float => Value::Float(0.0),
                    ast::Type::Double => Value::Double(0.0),
                    ast::Type::Bool => Value::Bool(false),
                    ast::Type::Byte => Value::Byte(0),
                    _ => Value::Int(0), // Default to Int(0) for unknown types
                };

                // Create array with specified size, initialized to default value
                let mut elems = Vec::with_capacity(array_type.len);
                elems.resize(array_type.len, default_val);
                Value::array(elems)
            } else {
                unreachable!()
            }
        } else if matches!(store.ty, ast::Type::User(_)) {
            // For user-defined types (e.g., `let p Point`), initialize with default values
            // if no expression is provided (store.expr is Nil)
            if matches!(store.expr, Expr::Nil) {
                if let ast::Type::User(type_decl) = &store.ty {
                    self.create_default_instance(&type_decl.name)
                } else {
                    unreachable!()
                }
            } else {
                // Normal assignment: let p Point = other_p
                self.eval_expr(&store.expr)
            }
        } else {
            // Normal value evaluation
            match &store.expr {
                Expr::Ref(target) => Value::Ref(target.clone().into()),
                _ => self.eval_expr(&store.expr),
            }
        };

        let mut value = value;

        // TODO: add general type coercion in assignment
        // int -> byte
        if matches!(store.ty, ast::Type::Byte) && matches!(value, Value::Int(_)) {
            value = Value::Byte(value.as_int() as u8);
        }

        // Move semantics: Mark the right-hand side as moved if it's a variable reference
        // This enforces ownership transfer: `let y = x` moves x to y
        // TODO: Only move linear types when they are implemented (Phase 2)
        // NOTE: Disabled for now - AutoLang copies primitive types by default
        // Only move when explicit linear types are implemented
        // self.mark_expr_as_moved(&store.expr);

        // Move semantics: Check if this is a reassignment
        // If so, the old value is dropped here (its last use)
        if self.has_local(&store.name) {  // Phase 4.5: Use bridge method
            // Remove old value - this will trigger cleanup if implemented
            // TODO: In Phase 2, we'll call drop_linear() here for linear types
            self.remove_local(&store.name);  // Phase 4.5: Use bridge method
            // Clear moved status since we're reassigning
            self.clear_moved(&store.name);  // Phase 4.5: Use bridge method
        }

        self.define(
            store.name.as_str(),
            Rc::new(scope::Meta::Store(store.clone())),
        );  // Phase 4.5: Use bridge method
        self.set_local_val(&store.name, value);  // Phase 4.5: Use bridge method
        Value::Void
    }

    fn eval_range(&mut self, range: &Range) -> Value {
        if range.eq {
            self.range_eq(&range.start, &range.end)
        } else {
            self.range(&range.start, &range.end)
        }
    }

    fn eval_bina(&mut self, left: &Expr, op: &Op, right: &Expr) -> Value {
        // Handle compound assignment operators (+=, -=, *=, /=, %=)
        match op {
            Op::AddEq | Op::SubEq | Op::MulEq | Op::DivEq | Op::ModEq => {
                // Get current value of left side
                let current_value = self.eval_expr(left);

                // Get right side value
                let right_value = self.eval_expr(right);

                // Perform the arithmetic operation
                let result = match op {
                    Op::AddEq => add(current_value.clone(), right_value.clone()),
                    Op::SubEq => sub(current_value.clone(), right_value.clone()),
                    Op::MulEq => mul(current_value.clone(), right_value.clone()),
                    Op::DivEq => div(current_value.clone(), right_value.clone()),
                    Op::ModEq => mod_(current_value.clone(), right_value.clone()),
                    _ => Value::Nil,
                };

                // Assign the result back to the left side
                self.eval_asn(left, result)
            }
            _ => {
                // Handle regular binary operators
                let left_value = self.eval_expr(left);
                let right_value = self.eval_expr(right);

                // Resolve ValueRef for arithmetic operations
                let left_resolved = self.resolve_or_clone(&left_value);
                let right_resolved = self.resolve_or_clone(&right_value);

                match op {
                    Op::Add => {
                        // Convert resolved ValueData back to Value for add()
                        add(
                            Value::from_data(left_resolved.clone()),
                            Value::from_data(right_resolved.clone()),
                        )
                    }
                    Op::Sub => sub(
                        Value::from_data(left_resolved.clone()),
                        Value::from_data(right_resolved.clone()),
                    ),
                    Op::Mul => mul(
                        Value::from_data(left_resolved.clone()),
                        Value::from_data(right_resolved.clone()),
                    ),
                    Op::Div => div(
                        Value::from_data(left_resolved.clone()),
                        Value::from_data(right_resolved.clone()),
                    ),
                    Op::Mod => mod_(
                        Value::from_data(left_resolved.clone()),
                        Value::from_data(right_resolved.clone()),
                    ),
                    Op::Eq | Op::Neq | Op::Lt | Op::Gt | Op::Le | Op::Ge => comp(
                        &Value::from_data(left_resolved),
                        &op,
                        &Value::from_data(right_resolved),
                    ),
                    Op::Asn => self.eval_asn(left, right_value),
                    Op::Range => self.range(left, right),
                    Op::RangeEq => self.range_eq(left, right),
                    Op::Dot => self.dot(left, right),
                    _ => Value::Nil,
                }
            }
        }
    }

    /// Build an AccessPath from a nested dot expression
    /// For example: obj.level1.level2.value -> Nested(Nested(Field("level1"), Field("level2")), Field("value"))
    /// Returns (root_identifier, access_path)
    fn build_dot_path(&self, expr: &Expr) -> Option<(AutoStr, auto_val::AccessPath)> {
        let mut fields = Vec::new();
        let mut current = expr;

        // Traverse the dot chain to collect all field names
        while let Expr::Dot(inner_obj, field) = current {
            fields.push(field.clone());
            current = inner_obj;
        }

        // The root should be an identifier
        if let Expr::Ident(root_name) = current {
            // Reverse the fields since we collected them right-to-left
            fields.reverse();

            // Build nested access path
            let mut path = None;
            for field in fields {
                let field_path = auto_val::AccessPath::Field(field);
                path = Some(match path {
                    None => field_path,
                    Some(inner) => {
                        auto_val::AccessPath::Nested(Box::new(inner), Box::new(field_path))
                    }
                });
            }

            path.map(|p| (root_name.clone(), p))
        } else {
            None
        }
    }

    fn eval_asn(&mut self, left: &Expr, val: Value) -> Value {
        match left {
            // Case 1: Simple identifier: x = value
            Expr::Ident(name) => {
                // check ref
                let left_val = self.lookup(&name);
                match left_val {
                    Value::Ref(target) => {
                        // println!("ref: {}", target); // LSP: disabled
                        if self.exists(&target) {  // Phase 4.5: Use bridge method
                            self.update_val(&target, val);  // Phase 4.5: Use bridge method
                        } else {
                            // Variable not found - return error with suggestion
                            let candidates = self.get_defined_names();  // Phase 4.5: Use bridge method
                            let suggestion = if let Some(s) =
                                crate::error::find_best_match(&target, &candidates)
                            {
                                format!(". Did you mean '{}'?", s)
                            } else {
                                String::new()
                            };
                            return Value::Error(
                                format!(
                                    "Variable (ref {} -> {}) not found{}",
                                    name, target, suggestion
                                )
                                .into(),
                            );
                        }
                    }
                    _ => {
                        if self.exists(&name) {  // Phase 4.5: Use bridge method
                            self.update_val(&name, val);  // Phase 4.5: Use bridge method
                        } else {
                            // Variable not found - return error with suggestion
                            let candidates = self.get_defined_names();  // Phase 4.5: Use bridge method
                            let suggestion = if let Some(s) =
                                crate::error::find_best_match(&name, &candidates)
                            {
                                format!(". Did you mean '{}'?", s)
                            } else {
                                String::new()
                            };
                            return Value::Error(
                                format!("Variable {} not found{}", name, suggestion).into(),
                            );
                        }
                    }
                }
                Value::Void
            }

            // Case 2: Dot expression: obj.field = value (Plan 056: Phase 2)
            Expr::Dot(object, field) => {
                // Handle simple case: obj.field = value where obj is an identifier
                if let Expr::Ident(obj_name) = object.as_ref() {
                    if let Some(obj_vid) = self.lookup_vid(obj_name) {
                        let field_name = field.clone();
                        let path = auto_val::AccessPath::Field(field_name);
                        let right_data = val.into_data();
                        let right_vid = self.alloc_value(right_data);

                        match self
                            .universe
                            .borrow_mut()
                            .update_nested(obj_vid, &path, right_vid)
                        {
                            Ok(()) => Value::Void,
                            Err(e) => Value::error(format!("Failed to assign to field: {:?}", e)),
                        }
                    } else {
                        Value::error(format!("Variable not found: {}", obj_name))
                    }
                } else if let Expr::Index(array, index_expr) = object.as_ref() {
                    // Handle arr[0].field = value case
                    if let Expr::Ident(arr_name) = array.as_ref() {
                        if let Some(arr_vid) = self.lookup_vid(arr_name) {
                            let idx_val = self.eval_expr(index_expr);
                            if let Value::Int(i) = idx_val {
                                let field_name = field.clone();
                                let right_data = val.into_data();
                                let right_vid = self.alloc_value(right_data);

                                let path = auto_val::AccessPath::Nested(
                                    Box::new(auto_val::AccessPath::Index(i as usize)),
                                    Box::new(auto_val::AccessPath::Field(field_name)),
                                );

                                match self
                                    .universe
                                    .borrow_mut()
                                    .update_nested(arr_vid, &path, right_vid)
                                {
                                    Ok(()) => Value::Void,
                                    Err(e) => Value::error(format!(
                                        "Failed to assign to array element field: {:?}",
                                        e
                                    )),
                                }
                            } else {
                                Value::error("Array index must be integer")
                            }
                        } else {
                            Value::error(format!("Array not found: {}", arr_name))
                        }
                    } else {
                        Value::error(
                            "Complex field assignment with non-identifier array not supported",
                        )
                    }
                } else if let Expr::Dot(inner_obj, inner_field) = object.as_ref() {
                    // Handle nested dot case: obj.inner.x = value or data[0].info.age = value
                    // First, check if inner_obj is an identifier: obj.inner.x = value
                    if let Expr::Ident(root_name) = inner_obj.as_ref() {
                        if let Some(root_vid) = self.lookup_vid(root_name) {
                            // Build nested path: obj.inner.x -> Nested(Field("inner"), Field("x"))
                            let inner_path = auto_val::AccessPath::Field(inner_field.clone());
                            let outer_path = auto_val::AccessPath::Field(field.clone());
                            let path = auto_val::AccessPath::Nested(
                                Box::new(inner_path),
                                Box::new(outer_path),
                            );

                            let right_data = val.into_data();
                            let right_vid = self.alloc_value(right_data);

                            match self
                                .universe
                                .borrow_mut()
                                .update_nested(root_vid, &path, right_vid)
                            {
                                Ok(()) => Value::Void,
                                Err(e) => Value::error(format!(
                                    "Failed to assign to nested field: {:?}",
                                    e
                                )),
                            }
                        } else {
                            Value::error(format!("Variable not found: {}", root_name))
                        }
                    } else if let Expr::Index(array, index_expr) = inner_obj.as_ref() {
                        // Handle data[0].info.age = value case
                        if let Expr::Ident(arr_name) = array.as_ref() {
                            if let Some(arr_vid) = self.lookup_vid(arr_name) {
                                let idx_val = self.eval_expr(index_expr);
                                if let Value::Int(i) = idx_val {
                                    // Build path: data[0].info.age -> Nested(Nested(Index(0), Field("info")), Field("age"))
                                    let index_path = auto_val::AccessPath::Index(i as usize);
                                    let info_path = auto_val::AccessPath::Nested(
                                        Box::new(index_path),
                                        Box::new(auto_val::AccessPath::Field(inner_field.clone())),
                                    );
                                    let age_path = auto_val::AccessPath::Nested(
                                        Box::new(info_path),
                                        Box::new(auto_val::AccessPath::Field(field.clone())),
                                    );

                                    let right_data = val.into_data();
                                    let right_vid =
                                        self.alloc_value(right_data);

                                    match self
                                        .universe
                                        .borrow_mut()
                                        .update_nested(arr_vid, &age_path, right_vid)
                                    {
                                        Ok(()) => Value::Void,
                                        Err(e) => Value::error(format!(
                                            "Failed to assign to deeply nested field: {:?}",
                                            e
                                        )),
                                    }
                                } else {
                                    Value::error("Array index must be integer")
                                }
                            } else {
                                Value::error(format!("Array not found: {}", arr_name))
                            }
                        } else {
                            Value::error(
                                "Nested field assignment with non-identifier array not supported",
                            )
                        }
                    } else {
                        // Handle arbitrary nesting of dot expressions: obj.level1.level2.value = value
                        // Build the full path including the outermost field
                        let full_dot = Expr::Dot(object.clone(), field.clone());
                        if let Some((root_name, path)) = self.build_dot_path(&full_dot) {
                            if let Some(root_vid) = self.lookup_vid(&root_name) {
                                let right_data = val.into_data();
                                let right_vid = self.alloc_value(right_data);

                                match self
                                    .universe
                                    .borrow_mut()
                                    .update_nested(root_vid, &path, right_vid)
                                {
                                    Ok(()) => Value::Void,
                                    Err(e) => Value::error(format!(
                                        "Failed to assign to deeply nested field: {:?}",
                                        e
                                    )),
                                }
                            } else {
                                Value::error(format!("Variable not found: {}", root_name))
                            }
                        } else {
                            Value::error("Nested field assignment with complex inner expression not supported")
                        }
                    }
                } else {
                    Value::error("Complex field assignment not yet implemented")
                }
            }

            // Case 3: Nested access: obj.field = value or obj.inner.field = value or obj.arr[0] = value
            Expr::Bina(left_obj, op, right_field) if *op == Op::Dot => {
                // Convert right-hand side to ValueData and allocate (only for nested assignment)
                let right_data = val.into_data();
                let right_vid = self.alloc_value(right_data);

                match left_obj.as_ref() {
                    // Simple case: obj.field = value
                    Expr::Ident(obj_name) => {
                        if let Some(obj_vid) = self.lookup_vid(obj_name) {
                            // Check if right_field is an index expression (obj.arr[0] = value)
                            match &**right_field {
                                Expr::Index(arr_field, index_expr) => {
                                    if let Expr::Ident(arr_name) = &**arr_field {
                                        let idx_val = self.eval_expr(index_expr);
                                        if let Value::Int(i) = idx_val {
                                            let path = auto_val::AccessPath::Nested(
                                                Box::new(auto_val::AccessPath::Field(
                                                    arr_name.clone(),
                                                )),
                                                Box::new(auto_val::AccessPath::Index(i as usize)),
                                            );
                                            match self
                                                .universe
                                                .borrow_mut()
                                                .update_nested(obj_vid, &path, right_vid)
                                            {
                                                Ok(()) => Value::Void,
                                                Err(e) => Value::error(format!(
                                                    "Failed to assign to array element: {:?}",
                                                    e
                                                )),
                                            }
                                        } else {
                                            Value::error("Array index must be integer")
                                        }
                                    } else {
                                        Value::error(format!("Invalid array target"))
                                    }
                                }
                                _ => {
                                    // Regular field access: obj.field = value
                                    let field_name = self.expr_to_astr(right_field);
                                    let path = auto_val::AccessPath::Field(field_name);
                                    match self
                                        .universe
                                        .borrow_mut()
                                        .update_nested(obj_vid, &path, right_vid)
                                    {
                                        Ok(()) => Value::Void,
                                        Err(e) => Value::error(format!(
                                            "Failed to assign to field: {:?}",
                                            e
                                        )),
                                    }
                                }
                            }
                        } else {
                            Value::error(format!("Variable not found: {}", obj_name))
                        }
                    }
                    // Nested case: obj.inner.field = value or arr[0].field = value
                    _ => {
                        // Extract the top-level identifier from the nested path
                        // We need to rebuild the path as Nested(top_level_field, rest_of_path)
                        // Actually, for cases like obj.inner.field, we need to:
                        // 1. Look up obj (top-level identifier)
                        // 2. Build path for inner.field
                        // So we need to extract the first component separately

                        // For now, handle the common case: arr[0].field
                        // The left_obj is arr[0] (Index expression)
                        // We need to get the array name and index
                        if let Expr::Index(array, index) = left_obj.as_ref() {
                            if let Expr::Ident(arr_name) = array.as_ref() {
                                if let Some(arr_vid) = self.lookup_vid(arr_name) {
                                    let idx_val = self.eval_expr(index);
                                    if let Value::Int(i) = idx_val {
                                        let field_name = self.expr_to_astr(right_field);
                                        let path = auto_val::AccessPath::Nested(
                                            Box::new(auto_val::AccessPath::Index(i as usize)),
                                            Box::new(auto_val::AccessPath::Field(field_name)),
                                        );
                                        match self
                                            .universe
                                            .borrow_mut()
                                            .update_nested(arr_vid, &path, right_vid)
                                        {
                                            Ok(()) => Value::Void,
                                            Err(e) => Value::error(format!(
                                                "Failed to assign to nested field: {:?}",
                                                e
                                            )),
                                        }
                                    } else {
                                        Value::error("Array index must be integer")
                                    }
                                } else {
                                    Value::error(format!("Array not found: {}", arr_name))
                                }
                            } else {
                                Value::error(format!("Invalid assignment target: {}", left_obj))
                            }
                        } else {
                            // Handle obj.inner.field case
                            // left_obj is obj.inner (Bina expression)
                            // We need to find the top-level identifier
                            let top_level = self.extract_top_level_identifier(left_obj);
                            if let Some(obj_name) = top_level {
                                if let Some(obj_vid) = self.lookup_vid(&obj_name) {
                                    // Build path for the rest (inner), excluding the top-level identifier
                                    let inner_path = match self
                                        .build_path_excluding_top_level(left_obj, &obj_name)
                                    {
                                        Ok(path) => path,
                                        Err(e) => {
                                            return Value::error(format!(
                                                "Invalid access path: {}",
                                                e
                                            ))
                                        }
                                    };

                                    // Add the rightmost field to complete the path
                                    let right_field_name = self.expr_to_astr(right_field);
                                    let full_path = auto_val::AccessPath::Nested(
                                        Box::new(inner_path),
                                        Box::new(auto_val::AccessPath::Field(right_field_name)),
                                    );

                                    match self
                                        .universe
                                        .borrow_mut()
                                        .update_nested(obj_vid, &full_path, right_vid)
                                    {
                                        Ok(()) => Value::Void,
                                        Err(e) => Value::error(format!(
                                            "Failed to assign to nested field: {:?}",
                                            e
                                        )),
                                    }
                                } else {
                                    Value::error(format!("Variable not found: {}", obj_name))
                                }
                            } else {
                                Value::error(format!("Invalid assignment target"))
                            }
                        }
                    }
                }
            }

            // Case 3: Array index: arr[0] = value or matrix[0][1] = value
            Expr::Index(array, index) => {
                // Convert right-hand side to ValueData and allocate (only for nested assignment)
                let right_data = val.into_data();
                let right_vid = self.alloc_value(right_data);

                match array.as_ref() {
                    // Simple case: arr[0] = value
                    Expr::Ident(arr_name) => {
                        if let Some(arr_vid) = self.lookup_vid(arr_name) {
                            let idx_val = self.eval_expr(index);
                            if let Value::Int(i) = idx_val {
                                let path = auto_val::AccessPath::Index(i as usize);
                                match self
                                    .universe
                                    .borrow_mut()
                                    .update_nested(arr_vid, &path, right_vid)
                                {
                                    Ok(()) => Value::Void,
                                    Err(e) => {
                                        Value::error(format!("Failed to assign to index: {:?}", e))
                                    }
                                }
                            } else {
                                Value::error("Array index must be integer")
                            }
                        } else {
                            Value::error(format!("Array not found: {}", arr_name))
                        }
                    }
                    // Nested case: matrix[0][1] = value
                    Expr::Index(nested_array, nested_index) => {
                        // Extract top-level array name
                        if let Expr::Ident(arr_name) = nested_array.as_ref() {
                            if let Some(arr_vid) = self.lookup_vid(arr_name) {
                                let idx_val = self.eval_expr(index);
                                if let Value::Int(i) = idx_val {
                                    // Build nested path: [nested_index][i]
                                    let nested_idx_val = self.eval_expr(nested_index);
                                    if let Value::Int(nested_i) = nested_idx_val {
                                        let path = auto_val::AccessPath::Nested(
                                            Box::new(auto_val::AccessPath::Index(
                                                nested_i as usize,
                                            )),
                                            Box::new(auto_val::AccessPath::Index(i as usize)),
                                        );
                                        match self
                                            .universe
                                            .borrow_mut()
                                            .update_nested(arr_vid, &path, right_vid)
                                        {
                                            Ok(()) => Value::Void,
                                            Err(e) => Value::error(format!(
                                                "Failed to assign to nested index: {:?}",
                                                e
                                            )),
                                        }
                                    } else {
                                        Value::error("Nested array index must be integer")
                                    }
                                } else {
                                    Value::error("Array index must be integer")
                                }
                            } else {
                                Value::error(format!("Array not found: {}", arr_name))
                            }
                        } else {
                            Value::error(format!("Invalid assignment target"))
                        }
                    }
                    // Case: obj.items[0] = value (Plan 056: Dot expression)
                    Expr::Dot(left_obj, right_field) => {
                        // Handle obj.items[0] where left_obj is an identifier and right_field is the field
                        if let Expr::Ident(obj_name) = left_obj.as_ref() {
                            if let Some(obj_vid) = self.lookup_vid(obj_name) {
                                let field_name = right_field.clone();
                                let idx_val = self.eval_expr(index);
                                if let Value::Int(i) = idx_val {
                                    let path = auto_val::AccessPath::Nested(
                                        Box::new(auto_val::AccessPath::Field(field_name)),
                                        Box::new(auto_val::AccessPath::Index(i as usize)),
                                    );
                                    match self
                                        .universe
                                        .borrow_mut()
                                        .update_nested(obj_vid, &path, right_vid)
                                    {
                                        Ok(()) => Value::Void,
                                        Err(e) => Value::error(format!(
                                            "Failed to assign to nested array element: {:?}",
                                            e
                                        )),
                                    }
                                } else {
                                    Value::error("Array index must be integer")
                                }
                            } else {
                                Value::error(format!("Object not found: {}", obj_name))
                            }
                        } else {
                            // Nested case: obj.inner.items[0] = value
                            let top_level = self.extract_top_level_identifier(array);
                            if let Some(obj_name) = top_level {
                                if let Some(obj_vid) = self.lookup_vid(&obj_name) {
                                    // Build the full path: inner.items[0]

                                    // Build path for the left_obj part (e.g., inner)
                                    let left_path = match self.build_access_path(left_obj) {
                                        Ok(path) => path,
                                        Err(e) => {
                                            return Value::error(format!(
                                                "Invalid access path: {}",
                                                e
                                            ))
                                        }
                                    };

                                    // Build path for right_field + index
                                    let field_name = right_field.clone();
                                    let idx_val = self.eval_expr(index);
                                    if let Value::Int(i) = idx_val {
                                        let field_idx_path = auto_val::AccessPath::Nested(
                                            Box::new(auto_val::AccessPath::Field(field_name)),
                                            Box::new(auto_val::AccessPath::Index(i as usize)),
                                        );

                                        // Combine left_path + field_idx_path
                                        let full_path = auto_val::AccessPath::Nested(
                                            Box::new(left_path),
                                            Box::new(field_idx_path),
                                        );

                                        match self
                                            .universe
                                            .borrow_mut()
                                            .update_nested(obj_vid, &full_path, right_vid)
                                        {
                                            Ok(()) => Value::Void,
                                            Err(e) => Value::error(format!(
                                                "Failed to assign to deeply nested element: {:?}",
                                                e
                                            )),
                                        }
                                    } else {
                                        Value::error("Array index must be integer")
                                    }
                                } else {
                                    Value::error(format!("Object not found: {}", obj_name))
                                }
                            } else {
                                Value::error("Cannot extract top-level identifier")
                            }
                        }
                    }
                    // Case: obj.items[0] = value or obj.inner.arr[0] = value (legacy syntax)
                    Expr::Bina(left_obj, op, right_field) if *op == Op::Dot => {
                        // We need to handle obj.items[0] where:
                        // - left_obj could be an identifier or another Bina expression
                        // - right_field is the field containing the array
                        match left_obj.as_ref() {
                            // Simple case: obj.items[0] = value
                            Expr::Ident(obj_name) => {
                                if let Some(obj_vid) = self.lookup_vid(obj_name) {
                                    let field_name = self.expr_to_astr(right_field);
                                    let idx_val = self.eval_expr(index);
                                    if let Value::Int(i) = idx_val {
                                        let path = auto_val::AccessPath::Nested(
                                            Box::new(auto_val::AccessPath::Field(field_name)),
                                            Box::new(auto_val::AccessPath::Index(i as usize)),
                                        );
                                        match self
                                            .universe
                                            .borrow_mut()
                                            .update_nested(obj_vid, &path, right_vid)
                                        {
                                            Ok(()) => Value::Void,
                                            Err(e) => Value::error(format!(
                                                "Failed to assign to nested array element: {:?}",
                                                e
                                            )),
                                        }
                                    } else {
                                        Value::error("Array index must be integer")
                                    }
                                } else {
                                    Value::error(format!("Object not found: {}", obj_name))
                                }
                            }
                            // Nested case: obj.inner.items[0] = value
                            _ => {
                                let top_level = self.extract_top_level_identifier(array);
                                if let Some(obj_name) = top_level {
                                    if let Some(obj_vid) = self.lookup_vid(&obj_name) {
                                        // Build the full path: inner.items[0]

                                        // Build path for the left_obj part (e.g., inner)
                                        let left_path = match self.build_access_path(left_obj) {
                                            Ok(path) => path,
                                            Err(e) => {
                                                return Value::error(format!(
                                                    "Invalid access path: {}",
                                                    e
                                                ))
                                            }
                                        };

                                        // Build path for right_field + index
                                        let field_name = self.expr_to_astr(right_field);
                                        let idx_val = self.eval_expr(index);
                                        if let Value::Int(i) = idx_val {
                                            let field_idx_path = auto_val::AccessPath::Nested(
                                                Box::new(auto_val::AccessPath::Field(field_name)),
                                                Box::new(auto_val::AccessPath::Index(i as usize)),
                                            );

                                            // Combine left_path with field_idx_path
                                            let full_path = auto_val::AccessPath::Nested(
                                                Box::new(left_path),
                                                Box::new(field_idx_path),
                                            );

                                            match self
                                                .universe
                                                .borrow_mut()
                                                .update_nested(obj_vid, &full_path, right_vid)
                                            {
                                                Ok(()) => Value::Void,
                                                Err(e) => Value::error(format!(
                                                    "Failed to assign to deeply nested array element: {:?}",
                                                    e
                                                )),
                                            }
                                        } else {
                                            Value::error("Array index must be integer")
                                        }
                                    } else {
                                        Value::error(format!("Object not found: {}", obj_name))
                                    }
                                } else {
                                    Value::error(format!("Invalid assignment target"))
                                }
                            }
                        }
                    }
                    _ => Value::error(format!("Invalid assignment target")),
                }
            }

            _ => Value::error(format!("Invalid target of asn {} = {}", left, val)),
        }
    }

    /// Helper: Convert expression to AutoStr (for field names)
    fn expr_to_astr(&self, expr: &Expr) -> AutoStr {
        match expr {
            Expr::Ident(name) => name.clone(),
            Expr::Str(s) => s.clone().into(),
            Expr::Int(i) => i.to_string().into(),
            _ => expr.repr().into(),
        }
    }

    /// Helper: Recursively build AccessPath from expression (without top-level identifier)
    /// Examples:
    /// - `field` → Field("field")
    /// - `inner.field` → Nested(Field("inner"), Field("field"))
    /// - `arr[0]` → Index(0)
    /// - `arr[0].field` → Nested(Index(0), Field("field"))
    /// - `matrix[0][1]` → Nested(Index(0), Index(1))
    fn build_access_path(&mut self, expr: &Expr) -> Result<auto_val::AccessPath, String> {
        match expr {
            // Case 1: Simple field access (base case for recursion)
            Expr::Ident(name) => Ok(auto_val::AccessPath::Field(name.clone())),

            // Case 2: Nested field access: obj.field or arr[0].field
            Expr::Bina(left, op, right) if *op == Op::Dot => {
                // Recursively build path for left side, then add right side
                let left_path = self.build_access_path(left)?;
                let right_field = self.expr_to_astr(right);
                Ok(auto_val::AccessPath::Nested(
                    Box::new(left_path),
                    Box::new(auto_val::AccessPath::Field(right_field)),
                ))
            }

            // Case 3: Array indexing: arr[0] or matrix[0][1]
            Expr::Index(array, index_expr) => {
                // Evaluate the index expression
                let idx_val = self.eval_expr(index_expr);
                if let Value::Int(i) = idx_val {
                    // Check if the array itself is indexed (for matrix[0][1])
                    if matches!(array.as_ref(), Expr::Index(_, _)) {
                        // Nested array indexing
                        let left_path = self.build_access_path(array)?;
                        Ok(auto_val::AccessPath::Nested(
                            Box::new(left_path),
                            Box::new(auto_val::AccessPath::Index(i as usize)),
                        ))
                    } else {
                        // Simple array indexing
                        Ok(auto_val::AccessPath::Index(i as usize))
                    }
                } else {
                    Err(format!("Array index must be integer, got {}", idx_val))
                }
            }

            _ => Err(format!("Invalid access path expression: {}", expr)),
        }
    }

    /// Helper: Extract top-level identifier from a nested expression
    /// Examples:
    /// - `obj` → Some("obj")
    /// - `obj.field` → Some("obj")
    /// - `obj.inner.field` → Some("obj")
    /// - `arr[0]` → Some("arr")
    /// - `arr[0].field` → Some("arr")
    fn extract_top_level_identifier(&self, expr: &Expr) -> Option<AutoStr> {
        match expr {
            Expr::Ident(name) => Some(name.clone()),
            Expr::Bina(left, _, _) => self.extract_top_level_identifier(left),
            Expr::Index(array, _) => self.extract_top_level_identifier(array),
            _ => None,
        }
    }

    /// Helper: Build path from expression, excluding the top-level identifier
    /// Examples:
    /// - `field` → Field("field")
    /// - `obj.inner` → Field("inner")  (excludes "obj")
    /// - `arr[0]` → Index(0)  (excludes "arr")
    /// - `obj.level1.level2` → Nested(Field("level1"), Field("level2"))  (excludes "obj")
    fn build_path_excluding_top_level(
        &mut self,
        expr: &Expr,
        top_level: &str,
    ) -> Result<auto_val::AccessPath, String> {
        match expr {
            Expr::Ident(name) if name == top_level => Err(format!(
                "Expression is just the top-level identifier: {}",
                name
            )),
            Expr::Ident(name) => Ok(auto_val::AccessPath::Field(name.clone())),
            Expr::Bina(left, op, right) if *op == Op::Dot => {
                // Check if left is the top-level identifier
                if let Expr::Ident(name) = left.as_ref() {
                    if name == top_level {
                        // This is where we are: obj.level1 where obj is top-level
                        // But right might be further nested, so we need to check
                        match &**right {
                            // If right is also a Bina (further nesting), recurse
                            Expr::Bina(_inner_left, inner_op, inner_right)
                                if *inner_op == Op::Dot =>
                            {
                                let left_path =
                                    auto_val::AccessPath::Field(self.expr_to_astr(right));
                                let right_field = self.expr_to_astr(inner_right);
                                Ok(auto_val::AccessPath::Nested(
                                    Box::new(left_path),
                                    Box::new(auto_val::AccessPath::Field(right_field)),
                                ))
                            }
                            // Right is a simple identifier
                            _ => {
                                let field_name = self.expr_to_astr(right);
                                Ok(auto_val::AccessPath::Field(field_name))
                            }
                        }
                    } else {
                        // Nested case: obj.inner.field where inner != top_level
                        let left_path = self.build_path_excluding_top_level(left, top_level)?;
                        let right_field = self.expr_to_astr(right);
                        Ok(auto_val::AccessPath::Nested(
                            Box::new(left_path),
                            Box::new(auto_val::AccessPath::Field(right_field)),
                        ))
                    }
                } else {
                    // Recursively handle nested left side
                    let left_path = self.build_path_excluding_top_level(left, top_level)?;
                    let right_field = self.expr_to_astr(right);
                    Ok(auto_val::AccessPath::Nested(
                        Box::new(left_path),
                        Box::new(auto_val::AccessPath::Field(right_field)),
                    ))
                }
            }
            Expr::Index(array, index_expr) => {
                // Check if array is the top-level identifier
                if let Expr::Ident(name) = array.as_ref() {
                    if name == top_level {
                        // Simple case: arr[0] where arr is top-level
                        let idx_val = self.eval_expr(index_expr);
                        if let Value::Int(i) = idx_val {
                            Ok(auto_val::AccessPath::Index(i as usize))
                        } else {
                            Err(format!("Array index must be integer, got {}", idx_val))
                        }
                    } else {
                        // Nested case: shouldn't happen normally
                        Err(format!("Unexpected nested index"))
                    }
                } else {
                    // Nested case: matrix[0][1]
                    let left_path = self.build_path_excluding_top_level(array, top_level)?;
                    let idx_val = self.eval_expr(index_expr);
                    if let Value::Int(i) = idx_val {
                        Ok(auto_val::AccessPath::Nested(
                            Box::new(left_path),
                            Box::new(auto_val::AccessPath::Index(i as usize)),
                        ))
                    } else {
                        Err(format!("Array index must be integer, got {}", idx_val))
                    }
                }
            }
            _ => Err(format!("Invalid expression: {}", expr)),
        }
    }

    /// Update an object's properties (Phase 4.5: bridge method)
    #[allow(dead_code)]
    fn update_obj(&mut self, name: &str, f: impl FnOnce(&mut Obj)) -> Value {
        self.universe.borrow_mut().update_obj(name, f);
        Value::Void
    }

    /// Update an array element (Phase 4.5: bridge method)
    #[allow(dead_code)]
    fn update_array(&mut self, name: &str, idx: Value, val: Value) -> Value {
        self.universe.borrow_mut().update_array(name, idx, val);
        Value::Void
    }

    fn range(&mut self, left: &Expr, right: &Expr) -> Value {
        let left_value = self.eval_expr(left);
        let right_value = self.eval_expr(right);

        // Resolve ValueRef for range operations
        let left_resolved = self.resolve_or_clone(&left_value);
        let right_resolved = self.resolve_or_clone(&right_value);

        match (&left_resolved, &right_resolved) {
            (auto_val::ValueData::Int(left), auto_val::ValueData::Int(right)) => {
                Value::Range(*left, *right)
            }
            _ => Value::error(format!("Invalid range {}..{}", left_value, right_value)),
        }
    }

    fn range_eq(&mut self, left: &Expr, right: &Expr) -> Value {
        let left_value = self.eval_expr(left);
        let right_value = self.eval_expr(right);

        // Resolve ValueRef for range operations
        let left_resolved = self.resolve_or_clone(&left_value);
        let right_resolved = self.resolve_or_clone(&right_value);

        match (&left_resolved, &right_resolved) {
            (auto_val::ValueData::Int(left), auto_val::ValueData::Int(right)) => {
                Value::RangeEq(*left, *right)
            }
            _ => Value::error(format!("Invalid range {}..={}", left_value, right_value)),
        }
    }

    fn eval_una(&mut self, op: &Op, e: &Expr) -> Value {
        let value = self.eval_expr(e);
        match op {
            Op::Add => value, // Unary & (address-of) - just return value for VM
            Op::Sub => value.neg(),
            Op::Not => value.not(),
            Op::Mul => {
                // Plan 052: Unary * (dereference) - not fully supported in VM
                // For C transpiler compatibility only
                Value::error("Pointer dereference (*) not supported in VM evaluator. Use C transpiler instead.")
            }
            _ => Value::Nil,
        }
    }

    fn lookup(&self, name: &str) -> Value {
        // Phase 4.5: Use bridge method
        self.lookup_val(name).unwrap_or(Value::Nil)
    }

    /// Get value ID directly without wrapping (Phase 4.5: bridge method)
    fn lookup_vid(&self, name: &str) -> Option<auto_val::ValueID> {
        self.universe.borrow().lookup_val_id(name)
    }

    /// Resolve Value::Ref to actual data (Phase 4.5: bridge method)
    fn resolve_value(&self, value: &Value) -> Option<Rc<RefCell<auto_val::ValueData>>> {
        match value {
            Value::ValueRef(vid) => self.universe.borrow().get_value(*vid),
            _ => None, // Inline values don't have stored data
        }
    }

    /// Helper: Resolve Ref or clone inline value
    fn resolve_or_clone(&self, val: &Value) -> auto_val::ValueData {
        match val {
            Value::ValueRef(vid) => self
                .universe
                .borrow()
                .get_value(*vid)
                .map(|cell| cell.borrow().clone())
                .unwrap_or(auto_val::ValueData::Nil),
            _ => val.clone().into_data(),
        }
    }

    #[allow(dead_code)]
    /// Mark a variable as moved if the expression is a variable reference
    /// This is used to enforce move semantics when values are passed to functions
    fn mark_expr_as_moved(&mut self, expr: &Expr) {
        match expr {
            // Direct variable reference: `x`
            Expr::Ident(name) => {
                self.mark_moved(name.as_str());
            }
            // Variable reference through `ref`: `ref x`
            Expr::Ref(name) => {
                self.mark_moved(name.as_str());
            }
            // For expressions, we need to check if the base is a variable
            // e.g., `x.field` or `x[index` - x is moved
            Expr::Bina(left, op, right) => {
                if *op == Op::Dot || *op == Op::LSquare {
                    // Mark the base object as moved
                    self.mark_expr_as_moved(left);
                } else {
                    // For other binary ops, mark both operands as potentially moved
                    self.mark_expr_as_moved(left);
                    self.mark_expr_as_moved(right);
                }
            }
            // Plan 056: Dot expression - field access should NOT move the object
            // `say(p.x)` should read the field without moving p
            Expr::Dot(_object, _field) => {
                // Do NOT mark the object as moved for field access
                // Field access is a read operation, not a move
            }
            Expr::Index(base, _index) => {
                // Mark the array as moved, unless it's a VM reference type (List, HashMap, etc.)
                // VM reference types use VmRef which is like a shared pointer, so indexing doesn't move them
                let base_value = self.eval_expr(base);
                if !matches!(base_value, Value::Instance(inst) if matches!(&inst.ty, auto_val::Type::User(name) if matches!(name.as_str(), "List" | "HashMap" | "HashSet" | "StringBuilder")))
                {
                    self.mark_expr_as_moved(base);
                }
            }
            // Nested expressions: recurse to find variable references
            Expr::Unary(_op, inner_expr) => {
                self.mark_expr_as_moved(inner_expr);
            }
            // Other expressions don't involve variable moves
            _ => {
                // Do nothing
            }
        }
    }

    fn eval_array(&mut self, elems: &Vec<Expr>) -> Value {
        let mut values = Array::new();
        for elem in elems.iter() {
            let v = self.eval_expr(elem);
            if !v.is_void() {
                values.push(v);
            }
        }
        Value::array(values)
    }

    fn object(&mut self, pairs: &Vec<ast::Pair>) -> Value {
        let mut obj = Obj::new();
        for pair in pairs.iter() {
            obj.set(self.eval_key(&pair.key), self.eval_expr(&pair.value));
        }
        Value::Obj(obj)
    }

    fn pair(&mut self, pair: &ast::Pair) -> Value {
        let key = self.eval_key(&pair.key);
        let value = self.eval_expr(&pair.value);
        Value::Pair(key, Box::new(value))
    }

    fn eval_key(&self, key: &Key) -> ValueKey {
        match key {
            Key::NamedKey(name) => ValueKey::Str(name.clone().into()),
            Key::IntKey(value) => ValueKey::Int(*value),
            Key::BoolKey(value) => ValueKey::Bool(*value),
            Key::StrKey(value) => ValueKey::Str(value.clone().into()),
        }
    }

    /// Get the type name of a Value (for method lookup)
    fn get_type_name(&self, value: &Value) -> String {
        match value {
            Value::Int(_) => "int".to_string(),
            Value::Uint(_) => "uint".to_string(),
            Value::I8(_) => "i8".to_string(),
            Value::U8(_) => "u8".to_string(),
            Value::Float(_) => "float".to_string(),
            Value::Double(_) => "double".to_string(),
            Value::Bool(_) => "bool".to_string(),
            Value::Str(_) => "str".to_string(),
            Value::OwnedStr(_) => "str".to_string(), // OwnedStr is also a string type
            Value::CStr(_) => "cstr".to_string(),
            Value::Char(_) => "char".to_string(),
            Value::Array(_) => "array".to_string(),
            Value::Instance(ref inst) => inst.ty.name().to_string(),
            Value::Type(ref ty) => ty.name().to_string(),
            Value::ValueRef(_vid) => {
                // Resolve ValueRef to get actual type
                if let Some(data) = self.resolve_value(value) {
                    let borrowed = data.borrow();
                    let value_from_data = Value::from_data(borrowed.clone());
                    self.get_type_name(&value_from_data)
                } else {
                    "unknown".to_string()
                }
            }
            _ => "unknown".to_string(),
        }
    }

    // TODO: 需要整理一下，逻辑比较乱
    fn eval_call(&mut self, call: &Call) -> AutoResult<Value> {
        // Check if this is a method call like `file.close()` or `x.triple()`
        // OR a tag construction like `Atom.Int(5)`
        if let Expr::Dot(object, method) = &*call.name {
            // First, check if this is tag construction: `Tag.Variant(args)`
            if let Expr::Ident(tag_name) = &**object {
                // Check if tag_name is a tag type
                let tag_type = self.lookup_type(tag_name);
                if matches!(tag_type, ast::Type::Tag(_)) {
                    // This is tag construction!
                    return self.eval_tag_construction(tag_name, method, &call.args);
                }
            }

            // This is a dot expression - check if it's a method call
            // Evaluate the left side to get the instance
            let instance = self.eval_expr(object);

            // Resolve ValueRef if needed
            let instance_resolved = match &instance {
                Value::ValueRef(_vid) => {
                    if let Some(data) = self.resolve_value(&instance) {
                        let borrowed_data = data.borrow();
                        let data_clone = borrowed_data.clone();
                        drop(borrowed_data);
                        Some(Value::from_data(data_clone))
                    } else {
                        None
                    }
                }
                _ => Some(instance.clone()),
            };

            if let Some(inst) = instance_resolved {
                // Get the type name of the instance
                let type_name = self.get_type_name(&inst);
                let method_name = method;

                // First, check if it's a VM method (for instances)
                if let Value::Instance(ref inst_data) = &inst {
                    let registry = crate::vm::VM_REGISTRY.lock().unwrap();
                    let method = registry
                        .get_method(&inst_data.ty.name(), method_name.as_str())
                        .cloned();
                    drop(registry);

                    if let Some(method) = method {
                        // Evaluate arguments (Arg::Pos contains Expr, not Value)
                        let mut arg_vals = Vec::new();
                        for arg in call.args.args.iter() {
                            match arg {
                                ast::Arg::Pos(expr) => {
                                    arg_vals.push(self.eval_expr(expr));
                                }
                                _ => {}
                            }
                        }

                        // Check if this is a direct variable access (like iter.next())
                        // We need to update the binding after mutation
                        let binding_name = if let Expr::Ident(var_name) = &**object {
                            Some(var_name.clone())
                        } else {
                            None
                        };

                        // Call the VM method with the instance
                        // Phase 4.6: Pass &mut self instead of uni.clone()

                        // Create a mutable copy for the method call
                        let mut inst_copy = inst.clone();
                        let result = method(self, &mut inst_copy, arg_vals);

                        // If this was a variable binding, update it with the mutated instance
                        if let Some(var_name) = binding_name {
                            // Phase 4.5: Use bridge method instead of direct Universe access
                            self.set_local_val(var_name.as_str(), inst_copy);
                        }

                        return Ok(result);
                    }
                }

                // Plan 019 Stage 8.5: Check spec default methods
                // If method not found on type, look through spec implementations
                if let Value::Instance(ref inst_data) = &inst {
                    if let Some(result) = self.resolve_spec_method(&inst, method_name, &call.args) {
                        return Ok(result);
                    }
                }

                // Next, check if it's an ext method (Plan 035) or type method
                // Look for "TypeName.method_name" in universe (using dot)
                let qualified_method_name: AutoStr =
                    format!("{}.{}", type_name, method_name).into();
                // Phase 4.5: Use bridge method for lookup
                let fn_decl_opt = self.lookup_meta(&qualified_method_name)
                    .and_then(|meta| {
                        if let scope::Meta::Fn(fn_decl) = meta.as_ref() {
                            Some(fn_decl.clone())
                        } else {
                            None
                        }
                    });

                if let Some(fn_decl) = fn_decl_opt {
                    // Plan 035 Phase 4.3: Only bind self for instance methods
                    // Static methods (is_static == true) don't have self
                    if !fn_decl.is_static {
                        // Bind self to the instance value before calling the method
                        // This allows the method body to access the instance via 'self'
                        self.universe
                            .borrow_mut()
                            .set_local_val("self", inst.clone());
                    }

                    // Call the method
                    return self.eval_fn_call(&fn_decl, &call.args);
                }

                // Plan 056: Try to find static function in VM registry
                // For static methods like File.open, List.new, HashMap.new
                // These are registered as "TypeName.method_name" in VM registry
                let vm_function_name: AutoStr = format!("{}.{}", type_name, method_name).into();
                let registry = crate::vm::VM_REGISTRY.lock().unwrap();
                let vm_func_entry = registry
                    .modules()
                    .values()
                    .find_map(|module| module.functions.get(vm_function_name.as_str()))
                    .cloned();
                drop(registry);

                if let Some(vm_func) = vm_func_entry {
                    // This is a VM static function - call it directly
                    // Phase 4.6: Pass &mut self instead of uni.clone()
                    let arg_vals: Vec<Value> = call
                        .args
                        .args
                        .iter()
                        .filter_map(|arg| match arg {
                            ast::Arg::Pos(expr) => Some(self.eval_expr(expr)),
                            _ => None,
                        })
                        .collect();

                    // VM static functions take (&mut Evaler, Value)
                    // For functions with single argument
                    if arg_vals.len() == 1 {
                        let result = (vm_func.func)(self, arg_vals[0].clone());
                        return Ok(result);
                    } else if arg_vals.is_empty() {
                        // For no-argument functions, pass Nil
                        let result = (vm_func.func)(self, Value::Nil);
                        return Ok(result);
                    } else {
                        // For multi-argument functions, wrap them in an Array
                        // This allows functions like List.new(1, 2, 3) to receive
                        // all arguments as a single Value::Array parameter
                        use auto_val::Array;
                        let array_value = Value::Array(Array { values: arg_vals });
                        let result = (vm_func.func)(self, array_value);
                        return Ok(result);
                    }
                }

                // Plan 038: Try to find VM function (e.g., str_split for str.split())
                // VM function naming convention: {type}_{method}
                let vm_function_name: AutoStr = format!("{}_{}", type_name, method_name).into();
                let vm_fn = self.lookup_val(&vm_function_name);  // Phase 4.5: Use bridge method

                if let Some(Value::ExtFn(ext_fn)) = vm_fn {
                    // Call VM function with self as first argument
                    // Build args: prepend self (the instance) to the provided arguments
                    let mut evaluated_args = Vec::new();
                    evaluated_args.push(auto_val::Arg::Pos(inst.clone()));

                    // Evaluate the provided arguments
                    for arg in &call.args.args {
                        match arg {
                            ast::Arg::Pos(expr) => {
                                let val = self.eval_expr(expr);
                                evaluated_args.push(auto_val::Arg::Pos(val));
                            }
                            _ => {}
                        }
                    }

                    let args = auto_val::Args {
                        args: evaluated_args,
                    };
                    let result = (ext_fn.fun)(&args);
                    return Ok(result);
                }
            }
        }

        if let Expr::Bina(left, op, right) = &*call.name {
            if *op == Op::Dot {
                // First, check if this is tag construction: `Tag.Variant(args)`
                if let Expr::Ident(tag_name) = &**left {
                    if let Expr::Ident(variant_name) = &**right {
                        // Check if tag_name is a tag type
                        let tag_type = self.lookup_type(tag_name);
                        if matches!(tag_type, ast::Type::Tag(_)) {
                            // This is tag construction!
                            return self.eval_tag_construction(tag_name, variant_name, &call.args);
                        }
                    }
                }

                // This is a dot expression - check if it's a method call
                // Evaluate the left side to get the instance
                let instance = self.eval_expr(left);

                // Resolve ValueRef if needed
                let instance_resolved = match &instance {
                    Value::ValueRef(_vid) => {
                        if let Some(data) = self.resolve_value(&instance) {
                            let borrowed_data = data.borrow();
                            let data_clone = borrowed_data.clone();
                            drop(borrowed_data);
                            Some(Value::from_data(data_clone))
                        } else {
                            None
                        }
                    }
                    _ => Some(instance.clone()),
                };

                if let Some(inst) = instance_resolved {
                    // Get the type name of the instance
                    let type_name = self.get_type_name(&inst);

                    // Check if right side is an identifier (method name)
                    if let Expr::Ident(method_name) = &**right {
                        // First, check if it's a VM method (for instances)
                        if let Value::Instance(ref inst_data) = &inst {
                            let registry = crate::vm::VM_REGISTRY.lock().unwrap();
                            let method = registry
                                .get_method(&inst_data.ty.name(), method_name.as_str())
                                .cloned();
                            drop(registry);

                            if let Some(method) = method {
                                // Evaluate arguments (Arg::Pos contains Expr, not Value)
                                let mut arg_vals = Vec::new();
                                for arg in call.args.args.iter() {
                                    match arg {
                                        ast::Arg::Pos(expr) => {
                                            arg_vals.push(self.eval_expr(expr));
                                        }
                                        _ => {}
                                    }
                                }

                                // Call the VM method with the instance
                                // Phase 4.6: Pass &mut self instead of uni.clone()
                                return Ok(method(self, &mut inst.clone(), arg_vals));
                            }
                        }

                        // Next, check if it's an ext method (Plan 035) or type method
                        // Look for "TypeName::method_name" in universe (using double colon)
                        let qualified_method_name: AutoStr =
                            format!("{}.{}", type_name, method_name).into();
                        // Phase 4.5: Use bridge method for lookup
                        let fn_decl_opt = self.lookup_meta(&qualified_method_name)
                            .and_then(|meta| {
                                if let scope::Meta::Fn(fn_decl) = meta.as_ref() {
                                    Some(fn_decl.clone())
                                } else {
                                    None
                                }
                            });

                        if let Some(fn_decl) = fn_decl_opt {
                            // Plan 035 Phase 4.3: Only bind self for instance methods
                            // Static methods (is_static == true) don't have self
                            if !fn_decl.is_static {
                                // Bind self to the instance value before calling the method
                                // This allows the method body to access the instance via 'self'
                                self.set_local_val("self", inst.clone());  // Phase 4.5: Use bridge method
                            }

                            // Call the method
                            return self.eval_fn_call(&fn_decl, &call.args);
                        }

                        // Plan 056: Try to find static function in VM registry
                        // For static methods like File.open, List.new, HashMap.new
                        // These are registered as "TypeName.method_name" in VM registry
                        let vm_function_name: AutoStr =
                            format!("{}.{}", type_name, method_name).into();
                        let registry = crate::vm::VM_REGISTRY.lock().unwrap();
                        let vm_func_entry = registry
                            .modules()
                            .values()
                            .find_map(|module| module.functions.get(vm_function_name.as_str()))
                            .cloned();
                        drop(registry);

                        if let Some(vm_func) = vm_func_entry {
                            // This is a VM static function - call it directly
                            // Phase 4.6: Pass &mut self instead of uni.clone()
                            let arg_vals: Vec<Value> = call
                                .args
                                .args
                                .iter()
                                .filter_map(|arg| match arg {
                                    ast::Arg::Pos(expr) => Some(self.eval_expr(expr)),
                                    _ => None,
                                })
                                .collect();

                            // VM static functions take (&mut Evaler, Value)
                            // For functions with single argument
                            if arg_vals.len() == 1 {
                                let result = (vm_func.func)(self, arg_vals[0].clone());
                                return Ok(result);
                            } else if arg_vals.is_empty() {
                                // For no-argument functions, pass Nil
                                let result = (vm_func.func)(self, Value::Nil);
                                return Ok(result);
                            } else {
                                // For multi-argument functions, wrap them in an Array
                                use auto_val::Array;
                                let array_value = Value::Array(Array { values: arg_vals });
                                let result = (vm_func.func)(self, array_value);
                                return Ok(result);
                            }
                        }

                        // Plan 038: Try to find VM function (e.g., str_split for str.split())
                        // VM function naming convention: {type}_{method}
                        let vm_function_name: AutoStr =
                            format!("{}_{}", type_name, method_name).into();
                        let vm_fn = {
                        self.lookup_val(&vm_function_name)  // Phase 4.5: Use bridge method
                    };

                        if let Some(Value::ExtFn(ext_fn)) = vm_fn {
                            // Call VM function with self as first argument
                            // Build args: prepend self (the instance) to the provided arguments
                            let mut evaluated_args = Vec::new();
                            evaluated_args.push(auto_val::Arg::Pos(inst.clone()));

                            // Evaluate the provided arguments
                            for arg in &call.args.args {
                                match arg {
                                    ast::Arg::Pos(expr) => {
                                        let val = self.eval_expr(expr);
                                        evaluated_args.push(auto_val::Arg::Pos(val));
                                    }
                                    _ => {}
                                }
                            }

                            let args = auto_val::Args {
                                args: evaluated_args,
                            };
                            let result = (ext_fn.fun)(&args);
                            return Ok(result);
                        }
                    }
                }
            }
        }

        // Regular function call (non-method)
        // First, try to lookup the function name directly (only for simple identifiers)
        // This enables functions like alloc_array() that are registered globally
        if let Expr::Ident(func_name) = call.name.as_ref() {
            // Check if this is a type instantiation (e.g., Point(x: 1, y: 2))
            let type_lookup = self.lookup_type(func_name);
            if !matches!(type_lookup, ast::Type::Unknown) {
                // This is a type instantiation!
                // Convert AST args to auto_val::Args
                let mut arg_vals = Vec::new();
                for arg in call.args.args.iter() {
                    match arg {
                        ast::Arg::Pos(expr) => {
                            let val = self.eval_expr(expr);
                            arg_vals.push(auto_val::Arg::Pos(val));
                        }
                        ast::Arg::Pair(name, expr) => {
                            let val = self.eval_expr(expr);
                            arg_vals.push(auto_val::Arg::Pair(
                                auto_val::ValueKey::Str(name.clone()),
                                val,
                            ));
                        }
                        ast::Arg::Name(name) => {
                            arg_vals.push(auto_val::Arg::Name(name.clone()));
                        }
                    }
                }
                let args = auto_val::Args { args: arg_vals };
                return Ok(self.eval_type_new(func_name, &args));
            }

            // Try to find global VM function in VM registry (Plan 052 Phase 2)
            // Search for this function in all VM modules
            let registry = crate::vm::VM_REGISTRY.lock().unwrap();
            let mut found_vm_func = None;

            // Search all modules for this function
            for (_module_name, module) in registry.modules().iter() {
                if let Some(func_entry) = module.functions.get(func_name.as_str()) {
                    found_vm_func = Some(func_entry.clone());
                    break;
                }
            }
            drop(registry);

            // If found in VM registry, call it
            if let Some(func_entry) = found_vm_func {
                // Evaluate arguments
                let mut arg_vals = Vec::new();
                for arg in call.args.args.iter() {
                    match arg {
                        ast::Arg::Pos(expr) => {
                            arg_vals.push(self.eval_expr(expr));
                        }
                        _ => {}
                    }
                }

                // Call the VM function (single parameter - the last argument)
                // Phase 4.6: Pass &mut self instead of self.universe.clone()
                // For multi-arg functions, they should be wrapped in Array
                let result = if arg_vals.len() == 1 {
                    (func_entry.func)(self, arg_vals[0].clone())
                } else {
                    // Multiple arguments - wrap in Array
                    let args_array = Value::Array(auto_val::Array { values: arg_vals });
                    (func_entry.func)(self, args_array)
                };

                return Ok(result);
            }
        }

        // Fall back to normal function lookup (handles dot expressions like List.new())
        let name = self.eval_expr(&call.name);
        if name == Value::Nil {
            return Ok(Value::error(format!(
                "Invalid function name to call {}",
                call.name
            )));
        }

        // Resolve ValueRef before matching on function type
        let name_resolved = match &name {
            Value::ValueRef(_vid) => {
                if let Some(data) = self.resolve_value(&name) {
                    let borrowed_data = data.borrow();
                    let data_clone = borrowed_data.clone();
                    drop(borrowed_data);
                    Some(Value::from_data(data_clone))
                } else {
                    None
                }
            }
            _ => Some(name.clone()),
        };

        let name_final = match name_resolved {
            Some(v) => v,
            None => {
                return Ok(Value::error(format!(
                    "Invalid function name to call {}",
                    call.name
                )))
            }
        };

        match name_final {
            // Value::Type(Type::User(u)) => {
            // return self.eval_type_new(&u, &call.args);
            // }
            Value::Meta(meta_id) => match meta_id {
                MetaID::Fn(sig) => {
                    return self.eval_fn_call_with_sig(&sig, &call.args);
                }
                // MetaID::Type(name) => {
                // return self.eval_type_new(&name, &call.args);
                // }
                _ => {
                    // println!("Strange function call {}", meta_id); // LSP: disabled
                }
            },
            Value::ExtFn(extfn) => {
                let args_val = self.eval_args(&call.args);
                return Ok((extfn.fun)(&args_val));
            }
            Value::Lambda(name) => {
                // Try to lookup lambda in SymbolTable
                let meta = self.lookup_meta(&name);  // Phase 4.5: Use bridge method
                if let Some(meta) = meta {
                    match meta.as_ref() {
                        scope::Meta::Fn(fn_decl) => {
                            return self.eval_fn_call(fn_decl, &call.args);
                        }
                        _ => {
                            return Ok(Value::error(format!("Invalid lambda {}", name)));
                        }
                    }
                } else {
                    return Ok(Value::error(format!("Invalid lambda {}", name)));
                }
            }
            Value::Widget(_widget) => {
                let node: Node = call.clone().into();
                return self.eval_node(&node);
            }
            Value::Method(method) => {
                return self.eval_method(&method, &call.args);
            }
            // Plan 060 Phase 3+: Closure calling
            Value::Closure(closure) => {
                return self.call_closure(&closure, &call.args);
            }
            _ => {
                return Ok(Value::error(format!("Invalid function call {}", name)));
            }
        }

        // Lookup Fn meta
        let meta = self.lookup_meta(&call.get_name_text());  // Phase 4.5: Use bridge method
        if let Some(meta) = meta {
            match meta.as_ref() {
                scope::Meta::Fn(fn_decl) => {
                    return self.eval_fn_call(fn_decl, &call.args);
                }
                _ => {
                    return Ok(Value::error(format!(
                        "Invalid lambda {}",
                        call.get_name_text()
                    )))
                }
            }
        } else {
            // convert call to node intance
            // println!("call {} not found, try to eval node", call.get_name_text()); // LSP: disabled
            let node: Node = call.clone().into();
            return self.eval_node(&node);
        }
    }

    pub fn eval_type_new(&mut self, name: &str, args: &auto_val::Args) -> Value {
        let typ = self.lookup_type(name);
        match typ {
            ast::Type::User(type_decl) => {
                let instance = self.eval_instance(&type_decl, args);
                return instance;
            }
            _ => Value::error(format!("Invalid type instance of {}", name)),
        }
    }

    fn eval_instance(&mut self, type_decl: &TypeDecl, args: &auto_val::Args) -> Value {
        let ty = self.eval_type(&type_decl);
        let fields = self.eval_fields(&type_decl, args);
        Value::Instance(auto_val::Instance { ty, fields })
    }

    fn eval_type(&mut self, type_decl: &TypeDecl) -> Type {
        Type::User(type_decl.name.clone())
    }

    fn eval_fields(&mut self, type_decl: &TypeDecl, args: &auto_val::Args) -> Obj {
        let members = &type_decl.members;
        // TODO: remove unnecessary clone
        let mut fields = Obj::new();

        // First, mix in fields from composed types
        for has_type in &type_decl.has {
            if let ast::Type::User(has_decl) = has_type {
                // Add default values for fields from composed type
                for member in &has_decl.members {
                    if !fields.has(member.name.clone()) {
                        match &member.value {
                            Some(default_value) => {
                                let val_data = self.eval_expr(default_value).into_data();
                                let vid = self.alloc_value(val_data);
                                fields.set(member.name.clone(), auto_val::Value::ValueRef(vid));
                            }
                            None => {
                                // No default value, set to nil
                                fields.set(member.name.clone(), Value::Nil);
                            }
                        }
                    }
                }
            }
        }

        // Then, add fields from direct arguments
        for (j, arg) in args.args.iter().enumerate() {
            // let val_arg = self.eval_arg(arg);
            match arg {
                auto_val::Arg::Pair(key, val) => {
                    for member in members.iter() {
                        if key.to_string() == member.name {
                            // If val is a ValueRef, we need to get the actual value from universe
                            let val_to_store = match val {
                                auto_val::Value::ValueRef(_vid) => {
                                    if let Some(data) = self.resolve_value(val) {
                                        let borrowed = data.borrow();
                                        let cloned = borrowed.clone();
                                        drop(borrowed);
                                        Some(Value::from_data(cloned))
                                    } else {
                                        None
                                    }
                                }
                                _ => Some(val.clone()),
                            };
                            if let Some(v) = val_to_store {
                                let val_data = v.into_data();
                                let vid = self.alloc_value(val_data);
                                fields.set(member.name.clone(), auto_val::Value::ValueRef(vid));
                            }
                        }
                    }
                }
                auto_val::Arg::Pos(value) => {
                    if j < members.len() {
                        let member = &members[j];
                        // If value is a ValueRef, we need to get the actual value from universe
                        let val_to_store = match value {
                            auto_val::Value::ValueRef(_vid) => {
                                if let Some(data) = self.resolve_value(value) {
                                    let borrowed = data.borrow();
                                    let cloned = borrowed.clone();
                                    drop(borrowed);
                                    Some(Value::from_data(cloned))
                                } else {
                                    None
                                }
                            }
                            _ => Some(value.clone()),
                        };
                        if let Some(v) = val_to_store {
                            let val_data = v.into_data();
                            let vid = self.alloc_value(val_data);
                            fields.set(member.name.clone(), auto_val::Value::ValueRef(vid));
                        }
                    }
                }
                auto_val::Arg::Name(name) => {
                    for member in members.iter() {
                        if *name == member.name {
                            fields.set(member.name.clone(), Value::Str(name.clone()));
                        }
                    }
                }
            }
        }
        // check default field values
        for member in members.iter() {
            match &member.value {
                Some(value) => {
                    if fields.has(member.name.clone()) {
                        continue;
                    }
                    let val_data = self.eval_expr(value).into_data();
                    let vid = self.alloc_value(val_data);
                    fields.set(member.name.clone(), auto_val::Value::ValueRef(vid));
                }
                None => {}
            }
        }
        fields
    }

    pub fn eval_method(&mut self, method: &Method, args: &Args) -> AutoResult<Value> {
        let target = &method.target;
        let name = &method.name;
        // methods for Any
        match target.as_ref() {
            Value::Str(_s) => {
                // First, check the types system for built-in methods
                let method_fn = self
                    .universe
                    .borrow()
                    .types
                    .lookup_method(Type::Str, name.clone());
                if let Some(method_fn) = method_fn {
                    return Ok(method_fn(&target));
                }

                // Plan 025 String Migration: Check for ext methods in universe
                // name might already be qualified (e.g., "str::contains") or simple (e.g., "contains")
                let ext_method = self.lookup_meta(name);  // Phase 4.5: Use bridge method
                if let Some(meta) = ext_method {
                    if let scope::Meta::Fn(fn_decl) = meta.as_ref() {
                        // Bind self and call the ext method
                        self.set_local_val("self", target.as_ref().clone());  // Phase 4.5: Use bridge method
                        return self.eval_fn_call(fn_decl, args);
                    }
                }
            }
            Value::OwnedStr(_s) => {
                // OwnedStr supports the same methods as Str
                let method_fn = self
                    .universe
                    .borrow()
                    .types
                    .lookup_method(Type::Str, name.clone());
                if let Some(method_fn) = method_fn {
                    return Ok(method_fn(&target));
                }

                // Plan 025 String Migration: Check for ext methods
                let ext_method = self.lookup_meta(name);  // Phase 4.5: Use bridge method
                if let Some(meta) = ext_method {
                    if let scope::Meta::Fn(fn_decl) = meta.as_ref() {
                        self.universe
                            .borrow_mut()
                            .set_local_val("self", target.as_ref().clone());
                        return self.eval_fn_call(fn_decl, args);
                    }
                }
            }
            Value::Instance(inst) => {
                // First, try to find the method directly in the type
                let meth = self.lookup_meta(&method.name);  // Phase 4.5: Use bridge method
                if let Some(meta) = meth {
                    match meta.as_ref() {
                        Meta::Fn(fn_decl) => {
                            // println!("Eval Method: {}", fn_decl.name); // LSP: disabled
                            // println!("Current Scope: {}", self.universe.borrow().cur_spot); // LSP: disabled
                            // self.enter_scope();
                            self.set_local_obj(&inst.fields);
                            // Fields are now available as local variables (x, y, etc.)
                            // No need to add 'self' parameter - methods access fields directly
                            let res = self.eval_fn_call(fn_decl, args)?;
                            // self.exit_scope();
                            return Ok(res);
                        }
                        _ => {
                            return Ok(Value::error(format!("wrong meta for method: {}", meta)));
                        }
                    }
                }

                // Method not found directly, check delegations
                // Get the type declaration to check for delegations
                // Collect delegation info first to avoid borrow issues
                let mut delegation_target: Option<Value> = None;
                let mut delegated_method_name: Option<AutoStr> = None;

                match &inst.ty {
                    auto_val::Type::User(type_name) => {
                        // Lookup the TypeDecl from universe
                        let type_name_clone = type_name.clone();
                        if let Some(meta) = self.lookup_meta(&type_name_clone)  /* Phase 4.5: Use bridge method */ {
                            if let Meta::Type(ast::Type::User(type_decl)) = meta.as_ref() {
                                for delegation in &type_decl.delegations {
                                    // Check if this delegation handles the method
                                    let spec_name = delegation.spec_name.clone();
                                    let member_name = delegation.member_name.clone();
                                    if let Some(spec_meta) =
                                        self.lookup_meta(&spec_name)  /* Phase 4.5: Use bridge method */
                                    {
                                        if let Meta::Spec(spec_decl) = spec_meta.as_ref() {
                                            // Check if the spec has this method
                                            if spec_decl
                                                .methods
                                                .iter()
                                                .any(|m| m.name == method.name)
                                            {
                                                // Found delegation! Get the delegated member value
                                                if let Some(member_value) =
                                                    inst.fields.lookup(&member_name)
                                                {
                                                    // Resolve ValueRef if needed
                                                    let resolved_member = match member_value {
                                                        Value::ValueRef(_vid) => {
                                                            if let Some(data) =
                                                                self.resolve_value(&member_value)
                                                            {
                                                                let borrowed = data.borrow();
                                                                let cloned = borrowed.clone();
                                                                drop(borrowed);
                                                                Some(Value::from_data(cloned))
                                                            } else {
                                                                None
                                                            }
                                                        }
                                                        _ => Some(member_value.clone()),
                                                    };

                                                    if resolved_member.is_some() {
                                                        delegation_target = resolved_member;
                                                        delegated_method_name =
                                                            Some(method.name.clone());
                                                        break;
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }

                // If we found a delegation, call the method on the delegated member
                if let (Some(target), Some(method_name)) =
                    (delegation_target, delegated_method_name)
                {
                    let delegated_method = Method {
                        target: Box::new(target),
                        name: method_name,
                    };
                    return self.eval_method(&delegated_method, args);
                }

                // Check VM_REGISTRY for instance methods (e.g., InlineInt64.capacity())
                if let auto_val::Type::User(type_name) = &inst.ty {
                    let registry = crate::vm::VM_REGISTRY.lock().unwrap();
                    let method_opt = registry
                        .modules()
                        .values()
                        .find_map(|module| {
                            module
                                .types
                                .get(type_name.as_str())
                                .and_then(|type_entry| type_entry.methods.get(name.as_str()))
                        })
                        .cloned();
                    drop(registry);

                    if let Some(method_fn) = method_opt {
                        // Call the VM instance method
                        // Phase 4.6: Pass &mut self instead of uni.clone()
                        let mut target_clone = target.clone();

                        // Convert Args to Vec<Value> by evaluating each Arg
                        let arg_vals: Vec<Value> = args
                            .args
                            .iter()
                            .map(|arg| match arg {
                                ast::Arg::Pos(expr) => self.eval_expr(expr),
                                ast::Arg::Pair(_, expr) => self.eval_expr(expr),
                                ast::Arg::Name(_) => Value::Nil,
                            })
                            .collect();

                        return Ok(method_fn(self, &mut target_clone, arg_vals));
                    }
                }
            }
            _ => {
                let method_fn = self
                    .universe
                    .borrow()
                    .types
                    .lookup_method(Type::Any, name.clone());
                if let Some(method_fn) = method_fn {
                    return Ok(method_fn(&target));
                }
            }
        }
        Ok(Value::error(format!(
            "Invalid method {} on {}",
            name, target
        )))
    }

    fn eval_fn_call_with_sig(&mut self, sig: &Sig, args: &Args) -> AutoResult<Value> {
        // Try to lookup in universe first
        let fn_decl_opt = self
            .universe
            .borrow()
            .lookup_sig(sig)
            .map(|meta| match meta.as_ref() {
                scope::Meta::Fn(fn_decl) => Some(fn_decl.clone()),
                _ => None,
            })
            .flatten();

        if let Some(fn_decl) = fn_decl_opt {
            return self.eval_fn_call(&fn_decl, args);
        }

        // If not found in universe, try VM registry (for static methods like HashMap.new(), List.new())
        let registry = crate::vm::VM_REGISTRY.lock().unwrap();
        let func_entry = registry
            .modules()
            .values()
            .find_map(|module| module.functions.get(sig.name.as_str()))
            .cloned();
        drop(registry);

        if let Some(func_entry) = func_entry {
            // This is a VM static function - call it directly
            // Phase 4.6: Pass &mut self instead of uni.clone()
            let arg_vals: Vec<Value> = args
                .args
                .iter()
                .filter_map(|arg| match arg {
                    ast::Arg::Pos(expr) => Some(self.eval_expr(expr)),
                    _ => None,
                })
                .collect();

            // VM static functions take (&mut Evaler, Value)
            // For varargs support (e.g., List.new(1, 2, 3)), pack multiple args into an Array
            if arg_vals.len() == 0 {
                return Ok((func_entry.func)(self, Value::Nil));
            } else if arg_vals.len() == 1 {
                return Ok((func_entry.func)(self, arg_vals[0].clone()));
            } else {
                // Pack multiple arguments into an Array for varargs functions
                let args_array = Value::Array(auto_val::Array { values: arg_vals });
                return Ok((func_entry.func)(self, args_array));
            }
        }

        let msg: AutoStr = format!("Invalid function call {}", sig.name.as_str()).into();
        Ok(Value::error(msg))
    }

    fn eval_fn_arg(&mut self, arg: &Arg, i: usize, params: &Vec<Param>) -> Value {
        match arg {
            Arg::Pair(name, expr) => {
                let val = self.eval_expr(expr);
                let name = &name;
                self.set_local_val(&name, val.clone());  // Phase 4.5: Use bridge method
                val
            }
            Arg::Pos(expr) => {
                let val = self.eval_expr(expr);
                // Only set local variable if params has this index
                // VM functions have empty params, so we skip setting local vars
                if i < params.len() {
                    let name = &params[i].name;
                    self.set_local_val(&name, val.clone());  // Phase 4.5: Use bridge method
                }
                val
            }
            Arg::Name(name) => {
                self.set_local_val(name.as_str(), Value::Str(name.clone()));  // Phase 4.5: Use bridge method
                Value::Str(name.clone())
            }
        }
    }

    pub fn eval_vm_fn_call(&mut self, fn_decl: &Fn, args: &Vec<Value>) -> Value {
        // First, try to look up the function in the VM registry
        let registry = crate::vm::VM_REGISTRY.lock().unwrap();

        // Search all modules for the function
        let func_entry = registry
            .modules()
            .values()
            .find_map(|module| module.functions.get(fn_decl.name.as_str()))
            .cloned();

        drop(registry);

        match func_entry {
            Some(func_entry) => {
                // Call the Rust function with evaluator and first argument
                // Phase 4.6: Pass &mut self instead of uni.clone()

                // For single-argument functions like open()
                if args.len() == 1 {
                    (func_entry.func)(self, args[0].clone())
                } else {
                    // For multi-argument functions (not yet supported)
                    Value::Error(
                        format!(
                            "VM functions with {} arguments not yet supported",
                            args.len()
                        )
                        .into(),
                    )
                }
            }
            None => {
                // Plan 025 String Migration: Fallback to ExtFn lookup
                // If the function is not in VM_REGISTRY, try to find it as an ExtFn builtin
                // This allows string extension methods to work with existing Rust implementations

                // Convert method name "str::len" to builtin function name "str_len"
                let lookup_name = if fn_decl.name.contains("::") {
                    // This is a method name like "str::len", convert to "str_len"
                    fn_decl.name.replace("::", "_")
                } else {
                    fn_decl.name.clone()
                };

                // Phase 4.5: Use bridge method instead of direct Universe access
                let builtin_opt = self.lookup_val(&lookup_name);

                if let Some(Value::ExtFn(extfn)) = builtin_opt {
                    // Check if this is an instance method (has parent) and needs self as first arg
                    let is_instance_method = fn_decl.parent.is_some() && !fn_decl.is_static;

                    // Build the complete args list for ExtFn
                    let mut final_args = Vec::new();

                    if is_instance_method {
                        // Add self as the first argument
                        // Note: self was bound in eval_method, but we need to get it here
                        // Try to get self from current scope
                        let self_val = self.lookup_val("self");  // Phase 4.5: Use bridge method

                        match self_val {
                            Some(val) => {
                                // If self is a ValueRef, resolve it to get the actual value
                                let resolved_val = if matches!(val, Value::ValueRef(_)) {
                                    if let Some(data) = self.resolve_value(&val) {
                                        let borrowed = data.borrow();
                                        Value::from_data(borrowed.clone())
                                    } else {
                                        val
                                    }
                                } else {
                                    val
                                };

                                final_args.push(auto_val::Arg::Pos(resolved_val));
                            }
                            None => {
                                return Value::Error(
                                    format!(
                                        "Method '{}' requires 'self' but it's not bound",
                                        fn_decl.name
                                    )
                                    .into(),
                                );
                            }
                        }
                    }

                    // Add the rest of the arguments
                    for arg in args.iter() {
                        final_args.push(auto_val::Arg::Pos(arg.clone()));
                    }

                    let args_struct = auto_val::Args { args: final_args };

                    (extfn.fun)(&args_struct)
                } else {
                    Value::Error(
                        format!(
                            "VM function '{}' not found in registry or builtins",
                            fn_decl.name
                        )
                        .into(),
                    )
                }
            }
        }
    }

    pub fn eval_fn_call(&mut self, fn_decl: &Fn, args: &Args) -> AutoResult<Value> {
        // IMPORTANT: Mark arguments as moved in caller's scope BEFORE entering function scope
        // This ensures move semantics are tracked in the correct scope
        // NOTE: Only mark expressions as moved if they are non-copy types
        // For now, we don't mark anything as moved since AutoLang doesn't have true move semantics yet
        // This is a placeholder for when linear types are implemented (Phase 2)
        /*
        for arg in args.args.iter() {
            match arg {
                Arg::Pair(_name, expr) => {
                    self.mark_expr_as_moved(expr);
                }
                Arg::Pos(expr) => {
                    self.mark_expr_as_moved(expr);
                }
                Arg::Name(_name) => {
                    // Name-only args don't move anything
                }
            }
        }
        */

        self.enter_fn(&fn_decl.name);
        // println!("scope after enter: {}", self.universe.borrow().cur_spot); // LSP: disabled
        // println!(
        //     "enter call scope {}",
        //     self.universe.borrow().current_scope().sid
        // );

        // Now evaluate and set arguments as local variables in function scope
        let mut arg_vals = Vec::new();
        for (i, arg) in args.args.iter().enumerate() {
            arg_vals.push(self.eval_fn_arg(arg, i, &fn_decl.params));
        }
        let result = match fn_decl.kind {
            FnKind::Function | FnKind::Lambda => {
                let result = self.eval_body(&fn_decl.body)?;
                self.exit_scope();

                // Plan 050 Step 4: Auto-wrap return value if function has ?T return type
                let ret_type = &fn_decl.ret;
                if is_may_type(ret_type) {
                    Ok(auto_wrap_may(result, ret_type))
                } else {
                    Ok(result)
                }
            }
            FnKind::VmFunction => {
                let result = self.eval_vm_fn_call(fn_decl, &arg_vals);
                self.exit_scope();
                Ok(result)
            }
            _ => Ok(Value::Error(
                format!("Fn {} eval not supported ", fn_decl.name).into(),
            )),
        };
        result
    }

    /// Evaluate a user-defined function by name with provided arguments
    /// This is called by VM functions (like map_iter_next) to invoke user functions
    pub fn eval_user_function(&mut self, fn_name: &AutoStr, args: Vec<Value>) -> Value {
        // Look up the function in the universe's meta registry
        let meta = self.lookup_meta(fn_name)  /* Phase 4.5 */;

        if let Some(ref meta_rc) = meta {
            if let scope::Meta::Fn(fn_decl) = meta_rc.as_ref() {
                // Build Args from the Vec<Value> by creating literal expressions
                let mut expr_args = Vec::new();
                for arg_val in args {
                    let expr = match arg_val {
                        Value::Int(i) => Expr::Int(i),
                        Value::Uint(u) => Expr::Uint(u),
                        Value::Float(f) => Expr::Float(f as f64, "".into()),
                        Value::Double(d) => Expr::Float(d, "".into()),
                        Value::Bool(b) => Expr::Bool(b),
                        Value::Str(ref s) => Expr::Str(s.clone()),
                        Value::OwnedStr(ref s) => Expr::Str(s.to_string().into()),
                        Value::Char(c) => Expr::Char(c),
                        Value::Nil => Expr::Nil,
                        _ => Expr::Nil, // For complex values, we'd need more handling
                    };
                    expr_args.push(ast::Arg::Pos(expr));
                }

                let call_args = Args { args: expr_args };

                // Call the function
                match self.eval_fn_call(fn_decl, &call_args) {
                    Ok(result) => result,
                    Err(e) => Value::Error(format!("Function call error: {}", e).into()),
                }
            } else {
                Value::Error(format!("{} is not a function", fn_name).into())
            }
        } else {
            Value::Error(format!("Function not found: {}", fn_name).into())
        }
    }

    fn index(&mut self, array: &Expr, index: &Expr) -> Value {
        let mut array_value = self.eval_expr(array);

        // Check if index is a range expression for slicing
        if let Expr::Range(ref range) = index {
            return self.slice(&array_value, range);
        }

        let index_value = self.eval_expr(index);
        let mut idx = match index_value {
            Value::Int(index) => index,
            // TODO: support negative index
            _ => return Value::error(format!("Invalid index {}", index_value)),
        };

        // Resolve ValueRef to actual value
        if let Value::ValueRef(_vid) = &array_value {
            if let Some(data) = self.resolve_value(&array_value) {
                let borrowed_data = data.borrow();
                let data_clone = borrowed_data.clone();
                drop(borrowed_data);
                array_value = Value::from_data(data_clone);
            }
        }

        match array_value {
            Value::Array(values) => {
                let len = values.len();
                if idx >= len as i32 {
                    return Value::error(format!("Index out of bounds {}", idx));
                }
                if idx < -(len as i32) {
                    return Value::error(format!("Index out of bounds {}", idx));
                }
                if idx < 0 {
                    idx = len as i32 + idx;
                }
                values[idx as usize].clone()
            }
            Value::Str(s) => {
                let idx = idx as usize;
                if idx >= s.len() {
                    return Value::error(format!("Index out of bounds {}", idx));
                }
                Value::Char(s.chars().nth(idx).unwrap())
            }
            Value::OwnedStr(s) => {
                let idx = idx as usize;
                if idx >= s.len() {
                    return Value::error(format!("Index out of bounds {}", idx));
                }
                Value::Char(s.as_str().chars().nth(idx).unwrap())
            }
            Value::Instance(ref instance) => {
                // Handle index operations on VM instances (e.g., List, HashMap)
                if let auto_val::Type::User(ref_name) = &instance.ty {
                    match ref_name.as_str() {
                        "List" => {
                            // Use the list_get VM method
                            let id = instance.fields.get("id");
                            if let Some(Value::USize(_list_id)) = id {
                                // Phase 4.6: Pass &mut self instead of uni.clone()
                                crate::vm::list::list_get(self, &mut array_value, vec![index_value])
                            } else {
                                Value::error(format!("Invalid List instance"))
                            }
                        }
                        "HashMap" | "HashSet" | "StringBuilder" => {
                            // These types don't support index operations
                            Value::error(format!(
                                "Type {} does not support index operations",
                                ref_name
                            ))
                        }
                        _ => Value::error(format!("Unknown type {}", ref_name)),
                    }
                } else {
                    Value::error(format!("Invalid instance type"))
                }
            }
            _ => Value::error(format!("Invalid array {}", array_value)),
        }
    }

    /// Slice a value using a range expression
    ///
    /// Supports slicing for:
    /// - str (Value::Str, Value::OwnedStr) → returns substring
    /// - Array (Value::Array) → returns subarray
    /// - List (VM instance) → returns new List with slice
    fn slice(&mut self, value: &Value, range: &ast::Range) -> Value {
        // Evaluate range bounds
        let start_val = self.eval_expr(&range.start);
        let end_val = self.eval_expr(&range.end);

        let start = match start_val {
            Value::Int(i) => i as usize,
            Value::Uint(u) => u as usize,
            _ => return Value::error(format!("Invalid range start: {:?}", start_val)),
        };

        let mut end = match end_val {
            Value::Int(i) => i as usize,
            Value::Uint(u) => u as usize,
            _ => return Value::error(format!("Invalid range end: {:?}", end_val)),
        };

        // Adjust end based on whether it's inclusive (..=) or exclusive (..)
        if range.eq {
            end += 1; // Inclusive range, so add 1 to end
        }

        match value {
            Value::Str(s) => {
                let bytes = s.as_bytes();
                if start > bytes.len() || end > bytes.len() || start > end {
                    return Value::error(format!(
                        "Slice out of bounds: [{}..{}], len={}",
                        start,
                        end,
                        bytes.len()
                    ));
                }
                let slice_str = &s[start..end];
                Value::Str(slice_str.into())
            }
            Value::OwnedStr(s) => {
                let s_str = s.as_str();
                if start > s_str.len() || end > s_str.len() || start > end {
                    return Value::error(format!(
                        "Slice out of bounds: [{}..{}], len={}",
                        start,
                        end,
                        s_str.len()
                    ));
                }
                let slice_str = &s_str[start..end];
                Value::OwnedStr(auto_val::Str::from_str(slice_str))
            }
            Value::Array(values) => {
                if start > values.len() || end > values.len() || start > end {
                    return Value::error(format!(
                        "Slice out of bounds: [{}..{}], len={}",
                        start,
                        end,
                        values.len()
                    ));
                }
                let sliced_values: Vec<Value> = values.values[start..end].to_vec();
                Value::Array(auto_val::Array::from_vec(sliced_values))
            }
            Value::Instance(ref instance) => {
                // Handle slicing on VM instances (e.g., List, dstr)
                if let auto_val::Type::User(ref_name) = &instance.ty {
                    match ref_name.as_str() {
                        "List" => {
                            // Get the list data
                            let id = instance.fields.get("id");
                            if let Some(Value::USize(_list_id)) = id {
                                // Phase 4.5: Removed unused _uni variable (was dead code)
                                // Create a new list from the slice
                                // TODO: Implement efficient List slicing
                                Value::error(format!("List slicing not yet implemented"))
                            } else {
                                Value::error(format!("Invalid List instance"))
                            }
                        }
                        "dstr" => {
                            // dstr has a List field called "data"
                            let data_field = instance.fields.get("data");
                            if let Some(Value::Instance(ref _list_inst)) = data_field {
                                // Slice the underlying List
                                // TODO: Implement dstr slicing
                                Value::error(format!("dstr slicing not yet implemented"))
                            } else {
                                Value::error(format!("Invalid dstr instance"))
                            }
                        }
                        _ => Value::error(format!("Type {} does not support slicing", ref_name)),
                    }
                } else {
                    Value::error(format!("Invalid instance type"))
                }
            }
            _ => Value::error(format!("Cannot slice type {:?}", value)),
        }
    }

    pub fn eval_expr(&mut self, expr: &Expr) -> Value {
        match expr {
            Expr::Byte(value) => Value::Byte(*value),
            Expr::Uint(value) => Value::Uint(*value),
            Expr::Int(value) => Value::Int(*value),
            Expr::Dot(object, field) => {
                // First, evaluate the object expression
                let obj_val = self.eval_expr(object);

                // Resolve ValueRef if needed (for variables)
                let obj_resolved = match &obj_val {
                    Value::ValueRef(_vid) => {
                        if let Some(data) = self.resolve_value(&obj_val) {
                            let borrowed_data = data.borrow();
                            let data_clone = borrowed_data.clone();
                            drop(borrowed_data);
                            Value::from_data(data_clone)
                        } else {
                            obj_val.clone()
                        }
                    }
                    _ => obj_val.clone(),
                };

                match obj_resolved {
                    Value::Instance(inst) => {
                        // First, try to get as a field
                        if let Some(val) = inst.fields.get(field.as_str()) {
                            val.clone()
                        } else {
                            // Field not found - create a Method value for method call
                            // This handles cases like f.read_text(), f.close(), etc.
                            // The actual method lookup happens in eval_call when this Method is called
                            Value::Method(Method::new(obj_val.clone(), field.clone()))
                        }
                    }
                    Value::Obj(obj) => {
                        // Support field access on plain objects (not type instances)
                        // Try different key types: Str, Int, Bool
                        // First, try as string key
                        let str_key = ValueKey::Str(field.as_str().into());
                        if let Some(val) = obj.get(str_key) {
                            self.deref_val(val.clone())
                        } else {
                            // Try as integer key (e.g., a.3 for {3: value})
                            if let Ok(int_val) = field.as_str().parse::<i32>() {
                                let int_key = ValueKey::Int(int_val);
                                if let Some(val) = obj.get(int_key) {
                                    self.deref_val(val.clone())
                                } else {
                                    // Try as boolean key
                                    match field.as_str().to_lowercase().as_str() {
                                        "true" => {
                                            let bool_key = ValueKey::Bool(true);
                                            if let Some(val) = obj.get(bool_key) {
                                                self.deref_val(val.clone())
                                            } else {
                                                Value::error(format!(
                                                    "Field '{}' not found in object",
                                                    field
                                                ))
                                            }
                                        }
                                        "false" => {
                                            let bool_key = ValueKey::Bool(false);
                                            if let Some(val) = obj.get(bool_key) {
                                                self.deref_val(val.clone())
                                            } else {
                                                Value::error(format!(
                                                    "Field '{}' not found in object",
                                                    field
                                                ))
                                            }
                                        }
                                        _ => Value::error(format!(
                                            "Field '{}' not found in object",
                                            field
                                        )),
                                    }
                                }
                            } else {
                                // Try as boolean key
                                match field.as_str().to_lowercase().as_str() {
                                    "true" => {
                                        let bool_key = ValueKey::Bool(true);
                                        if let Some(val) = obj.get(bool_key) {
                                            self.deref_val(val.clone())
                                        } else {
                                            Value::error(format!(
                                                "Field '{}' not found in object",
                                                field
                                            ))
                                        }
                                    }
                                    "false" => {
                                        let bool_key = ValueKey::Bool(false);
                                        if let Some(val) = obj.get(bool_key) {
                                            self.deref_val(val.clone())
                                        } else {
                                            Value::error(format!(
                                                "Field '{}' not found in object",
                                                field
                                            ))
                                        }
                                    }
                                    _ => Value::error(format!(
                                        "Field '{}' not found in object",
                                        field
                                    )),
                                }
                            }
                        }
                    }
                    _ => {
                        // For non-instance values (int, str, etc.), try method lookup
                        // This handles cases like 1.str(), "hello".upper(), etc.
                        self.dot(object, &Expr::Ident(field.clone()))
                    }
                }
            }
            Expr::I8(value) => Value::I8(*value),
            Expr::U8(value) => Value::U8(*value),
            Expr::I64(value) => Value::I64(*value),
            Expr::Float(value, _) => Value::Float(*value),
            Expr::Double(value, _) => Value::Double(*value),
            // Why not move here?
            Expr::Char(value) => Value::Char(*value),
            Expr::Str(value) => Value::OwnedStr(auto_val::Str::from_str(value.as_str())),
            Expr::CStr(value) => Value::Str(value.clone().into()),
            Expr::Bool(value) => Value::Bool(*value),
            Expr::Ref(target) => {
                let target_val = self.eval_expr(&Expr::Ident(target.clone()));
                target_val
            }
            Expr::View(e) => {
                // View borrow: immutable borrow (like Rust &T)
                // Evaluate the expression and create an immutable borrow
                let value = self.eval_expr(e);

                // Generate a fresh lifetime for this borrow
                let lifetime = self.lifetime_ctx.fresh_lifetime();

                // Check borrow conflicts
                if let Err(err) = self.borrow_checker.check_borrow(
                    e,
                    crate::ownership::borrow::BorrowKind::View,
                    lifetime,
                ) {
                    // Borrow conflict detected
                    return Value::Error(format!("Borrow error: {}", err).into());
                }

                // Return the value (immutable borrow)
                // TODO: Track that this value is a borrow with the given lifetime
                value
            }
            Expr::Mut(e) => {
                // Mut borrow: mutable borrow (like Rust &mut T)
                // Evaluate the expression and create a mutable borrow
                let value = self.eval_expr(e);

                // Generate a fresh lifetime for this borrow
                let lifetime = self.lifetime_ctx.fresh_lifetime();

                // Check borrow conflicts
                if let Err(err) = self.borrow_checker.check_borrow(
                    e,
                    crate::ownership::borrow::BorrowKind::Mut,
                    lifetime,
                ) {
                    // Borrow conflict detected
                    return Value::Error(format!("Borrow error: {}", err).into());
                }

                // Return the value (mutable borrow)
                // TODO: Track that this value is a mut borrow with the given lifetime
                value
            }
            Expr::Take(e) => {
                // Take: move semantics (like Rust move or std::mem::take)
                // Evaluate the expression and transfer ownership
                let value = self.eval_expr(e);

                // Generate a fresh lifetime for this take
                let lifetime = self.lifetime_ctx.fresh_lifetime();

                // Check borrow conflicts (take conflicts with all borrows)
                if let Err(err) = self.borrow_checker.check_borrow(
                    e,
                    crate::ownership::borrow::BorrowKind::Take,
                    lifetime,
                ) {
                    // Borrow conflict detected
                    return Value::Error(format!("Borrow error: {}", err).into());
                }

                // Return the value (ownership transferred)
                // TODO: Mark the original value as moved/invalidate it
                // The Phase 1 Linear trait should handle this automatically
                value
            }
            Expr::Hold(hold) => {
                // Hold expression: temporary path binding with syntax sugar
                // This is equivalent to:
                // {
                //     let <name> = mut <path>
                //     <body>
                //     // <name>'s lifetime ends here
                // }

                // Evaluate the path expression
                let path_value = self.eval_expr(&hold.path);

                // Create a mutable borrow (like Mut expression)
                let lifetime = self.lifetime_ctx.fresh_lifetime();
                if let Err(err) = self.borrow_checker.check_borrow(
                    &hold.path,
                    crate::ownership::borrow::BorrowKind::Mut,
                    lifetime,
                ) {
                    return Value::Error(format!("Hold borrow error: {}", err).into());
                }

                // Create a new scope for the hold block
                self.enter_scope();

                // Create a Store AST node for the binding (similar to let/var)
                use crate::ast::{Store, StoreKind};
                let store = Store {
                    kind: StoreKind::Var, // Hold creates a mutable binding
                    name: hold.name.clone(),
                    ty: crate::ast::Type::Unknown,
                    expr: *hold.path.clone(),
                };

                // Bind the path value to the name (using Meta::Store)
                self.define(  // Phase 4.5: Use bridge method
                    hold.name.clone(),
                    std::rc::Rc::new(crate::scope::Meta::Store(store)),
                );
                self.set_local_val(&hold.name, path_value);  // Phase 4.5: Use bridge method

                // Evaluate the body
                let result = match self.eval_body(&hold.body) {
                    Ok(v) => v,
                    Err(e) => Value::Error(format!("Error in hold body: {:?}", e).into()),
                };

                // Pop scope (ends the borrow)
                self.exit_scope();

                result
            }
            Expr::Ident(name) => self.eval_ident(name),
            Expr::GenName(name) => Value::Str(name.into()),
            Expr::Unary(op, e) => self.eval_una(op, e),
            Expr::Bina(left, op, right) => self.eval_bina(left, op, right),
            Expr::Range(range) => self.eval_range(range),
            Expr::If(if_) => match self.eval_if(if_) {
                Ok(v) => v,
                Err(e) => Value::Error(format!("Error in if expression: {:?}", e).into()),
            },
            Expr::Array(elems) => self.eval_array(elems),
            Expr::Call(call) => match self.eval_call(call) {
                Ok(v) => v,
                Err(e) => Value::Error(format!("Error in call: {:?}", e).into()),
            },
            Expr::Node(node) => match self.eval_node(node) {
                Ok(v) => v,
                Err(e) => Value::Error(format!("Error in node: {:?}", e).into()),
            },
            Expr::Index(array, index) => self.index(array, index),
            Expr::Pair(pair) => self.pair(pair),
            Expr::Object(pairs) => self.object(pairs),
            Expr::Block(body) => match self.eval_body(body) {
                Ok(v) => v,
                Err(e) => Value::Error(format!("Error in block: {:?}", e).into()),
            },
            Expr::Lambda(lambda) => Value::Lambda(lambda.name.clone().into()),
            Expr::Closure(closure) => {
                // Plan 060 Phase 3: Evaluate closure and capture environment
                self.closure(closure)
            }
            Expr::FStr(fstr) => self.fstr(fstr),
            Expr::Grid(grid) => self.grid(grid),
            Expr::Cover(cover) => self.cover(cover),
            Expr::Uncover(_) => Value::Void,
            Expr::Null => Value::Null,
            Expr::Nil => Value::Nil,
            Expr::NullCoalesce(left, right) => {
                // Null-coalescing operator: left ?? right
                // If left evaluates to some value, return it
                // Otherwise return right
                let left_val = self.eval_expr(left);
                match left_val {
                    Value::Nil => self.eval_expr(right),
                    _ => left_val,
                }
            }
            Expr::ErrorPropagate(expr) => {
                // Error propagation operator: expression.?
                // If expression evaluates to MayInt.Val(v), return v
                // If expression evaluates to MayInt.Nil or MayInt.Err, return early
                // This is like Rust's ? operator
                let expr_val = self.eval_expr(expr);
                match &expr_val {
                    Value::Node(node) => {
                        // Check if this is a May type with Val variant
                        match node.get_prop("is_some") {
                            Value::Bool(true) => {
                                // Return the unwrapped value
                                node.get_prop("value")
                            }
                            _ => {
                                // Early return for Nil or Err
                                // TODO: Implement proper error propagation
                                expr_val
                            }
                        }
                    }
                    _ => expr_val,
                }
            }
        }
    }

    fn cover(&mut self, _cover: &Cover) -> Value {
        Value::Void
    }

    fn eval_ident(&mut self, name: &AutoStr) -> Value {
        // let univ = self.universe.borrow_mut();
        // return Some(RefMut::map(univ, |map| map.get_mut_val(name).unwrap()));

        let res = self.lookup(&name);
        match res {
            Value::Ref(target) => {
                let target_val = self.eval_expr(&Expr::Ident(target));
                target_val
            }
            Value::ValueRef(vid) => {
                // Resolve ValueRef to actual value
                self.deref_val(Value::ValueRef(vid))
            }
            Value::Nil => {
                // Try types FIRST (before meta)
                // This ensures type names are found before local functions
                let typ = self.lookup_type(name);
                if !matches!(typ, ast::Type::Unknown) {
                    let vty: auto_val::Type = typ.into();
                    return Value::Type(vty);
                }
                // Try meta (after types)
                let meta = self.lookup_meta(&name);  // Phase 4.5: Use bridge method
                if let Some(meta) = meta {
                    return Value::Meta(to_meta_id(&meta));
                }
                // Try builtin
                let v = self
                    .universe
                    .borrow()
                    .lookup_builtin(&name)
                    .unwrap_or(Value::Nil);

                if !v.is_nil() {
                    return v;
                }
                if self.skip_check {
                    Value::Str(name.clone())
                } else {
                    Value::Nil
                }
            }
            _ => res,
        }
    }

    fn type_decl(&mut self, type_decl: &TypeDecl) -> Value {
        // Register the type itself
        let type_meta = scope::Meta::Type(ast::Type::User(type_decl.clone()));
        self.universe
            .borrow_mut()
            .define(type_decl.name.clone(), std::rc::Rc::new(type_meta));

        // Mix in methods from composed types (has relationships)
        for has_type in &type_decl.has {
            if let ast::Type::User(has_decl) = has_type {
                // Register each method from the composed type
                for method in &has_decl.methods {
                    // Create fully qualified method name: TypeName::method_name
                    let method_name: AutoStr = format!("{}.{}", type_decl.name, method.name).into();

                    // Clone the method and update its name to reflect the new owner
                    let mut mixed_method = method.clone();
                    mixed_method.name = type_decl.name.clone();

                    // Register in universe with qualified name
                    self.universe
                        .borrow_mut()
                        .define(method_name, std::rc::Rc::new(scope::Meta::Fn(mixed_method)));
                }
            }
        }

        // Also register the type's own methods (but only if they have bodies)
        // Interface declarations without bodies (e.g., in .at files) should not register methods
        // The actual implementations come from ext blocks (e.g., in .vm.at or .c.at files)
        for method in &type_decl.methods {
            // Only register methods that have bodies (interface-only methods are just declarations)
            if !method.body.stmts.is_empty() {
                let method_name: AutoStr = format!("{}.{}", type_decl.name, method.name).into();
                self.define(  /* Phase 4.5 */
                    method_name,
                    std::rc::Rc::new(scope::Meta::Fn(method.clone())),
                );
            }
        }

        Value::Void
    }

    /// Evaluate ext statement (Plan 035)
    ///
    /// Extends a type with additional methods (like Rust's impl block).
    /// This enables adding methods to built-in types (str, int, etc.) and
    /// user-defined types after their initial definition.
    ///
    /// # Arguments
    /// * `ext` - The Ext statement containing target type and methods
    ///
    /// # Example
    ///
    /// ```auto
    /// ext str {
    ///     fn len() int {
    ///         return .size
    ///     }
    /// }
    /// ```
    fn eval_ext(&mut self, ext: &ast::Ext) -> Value {
        // Register each method in the ext block
        for method in &ext.methods {
            // Create fully qualified method name: TypeName::method_name
            // Use double colon (::) to match type_decl's convention
            let method_name: AutoStr = format!("{}.{}", ext.target, method.name).into();

            // Plan 035 Phase 5: Check for duplicate method definitions
            if let Some(existing_meta) = self.lookup_meta(&method_name)  /* Phase 4.5 */ {
                // Method already exists, issue a warning
                if let scope::Meta::Fn(_existing_fn) = existing_meta.as_ref() {
                    eprintln!(
                        "Warning: Method '{}' already defined for type '{}'. Overwriting previous definition.",
                        method.name, ext.target
                    );
                    // Optionally: could check if the definitions are compatible
                }
            }

            // Clone method and ensure parent and name are set correctly
            let mut registered_method = method.clone();
            registered_method.parent = Some(ext.target.clone());
            registered_method.name = method_name.clone(); // Update name to qualified name (e.g., "str::contains")

            // Register in universe with qualified name
            self.define(  /* Phase 4.5 */
                method_name,
                std::rc::Rc::new(scope::Meta::Fn(registered_method)),
            );
        }

        Value::Void
    }

    fn eval_tag_decl(&mut self, tag: &ast::Tag) -> Value {
        // Register each method in the tag definition
        for method in &tag.methods {
            // Create fully qualified method name: TagName::method_name
            // Use double colon (::) to match type_decl's convention
            let method_name: AutoStr = format!("{}.{}", tag.name, method.name).into();

            // Check for duplicate method definitions
            if let Some(existing_meta) = self.lookup_meta(&method_name)  /* Phase 4.5 */ {
                // Method already exists, issue a warning
                if let scope::Meta::Fn(_existing_fn) = existing_meta.as_ref() {
                    eprintln!(
                        "Warning: Method '{}' already defined for tag '{}'. Overwriting previous definition.",
                        method.name, tag.name
                    );
                }
            }

            // Clone method and ensure parent and name are set correctly
            let mut registered_method = method.clone();
            registered_method.parent = Some(tag.name.clone());
            registered_method.name = method_name.clone(); // Update name to qualified name

            // Register in universe with qualified name
            self.define(  /* Phase 4.5 */
                method_name,
                std::rc::Rc::new(scope::Meta::Fn(registered_method)),
            );
        }

        Value::Void
    }

    fn spec_decl(&mut self, spec_decl: &ast::SpecDecl) -> Value {
        // Plan 019 Stage 8.5: Register the spec in the universe's specs HashMap
        // This makes specs available for:
        // - Spec method resolution (default method implementations)
        // - Type checking and constraint validation
        self.register_spec(std::rc::Rc::new(spec_decl.clone()));
        Value::Void
    }

    /// Plan 019 Stage 8.5: Resolve spec methods with default implementations
    /// When a method is not found on a type, look through its spec implementations
    /// Returns Some(result) if found and executed, None if not found
    fn resolve_spec_method(&mut self, instance: &Value, method_name: &AutoStr, args: &ast::Args) -> Option<Value> {
        // Get the type name
        let type_name = if let Value::Instance(ref inst_data) = instance {
            inst_data.ty.name().to_string()
        } else {
            return None;
        };

        // Get the TypeDecl for this type
        let type_decl = self.lookup_type(&type_name);  // Phase 4.5: Use bridge method

        let type_decl = match type_decl {
            ast::Type::User(decl) => decl,
            _ => return None,
        };

        // Iterate through spec implementations
        for spec_impl in type_decl.spec_impls.iter() {
            // Look up the spec declaration from specs HashMap
            let spec_decl = self.get_spec(&spec_impl.spec_name);  // Phase 4.5: Use bridge method

            let spec_decl = match spec_decl {
                Some(decl) => decl,
                None => continue,
            };

            // Check if this spec has the method
            if let Some(spec_method) = spec_decl.get_method(&ast::Name::from(method_name.as_str())) {
                // Check if it has a default implementation
                if let Some(body) = &spec_method.body {
                    return Some(self.eval_spec_method_body(instance, body, args, &spec_method.params));
                }
            }
        }

        None
    }

    /// Evaluate a spec method body with `self` bound to the instance
    fn eval_spec_method_body(&mut self, instance: &Value, body: &ast::Expr, args: &ast::Args, params: &[ast::Param]) -> Value {
        // Create a new scope and bind `self` to the instance
        self.enter_fn(&AutoStr::from("<spec_method>"));

        // Bind `self` to the instance
        self.universe
            .borrow_mut()
            .set_local_val("self", instance.clone());

        // Bind spec method parameters (e.g., `f` in `map(f)`)
        for (i, param) in params.iter().enumerate() {
            if let Some(arg_expr) = args.args.get(i) {
                match arg_expr {
                    ast::Arg::Pos(expr) => {
                        let arg_val = self.eval_expr(expr);
                        self.set_local_val(&param.name.to_string(), arg_val)  // Phase 4.5;
                    }
                    _ => {}
                }
            }
        }

        // Evaluate the method body
        let result = match body {
            ast::Expr::Block(block) => {
                // Evaluate each statement in the block
                let mut value = Value::Void;
                for stmt in &block.stmts {
                    value = self.eval_stmt(stmt).unwrap_or(Value::Void);
                }
                value
            }
            _ => self.eval_expr(body),
        };

        // Exit the scope
        self.exit_scope();

        result
    }

    fn dot_node(&mut self, node: &auto_val::Node, right: &Expr) -> Option<Value> {
        let Expr::Ident(name) = right else {
            return None;
        };
        if name == "name" {
            return Some(Value::Str(node.name.clone()));
        }
        if name == "id" {
            return Some(Value::Str(node.id.clone()));
        }
        let mut name = name.clone();
        // 1. lookup in the props
        let v = node.get_prop(&name);
        if v.is_nil() {
            // 2.1 check if nodes with the name exists
            let nodes = node.get_nodes(&name);
            if nodes.len() > 1 {
                return Some(Value::array_of(nodes.iter().map(|n| n.clone()).collect()));
            } else if nodes.len() == 1 {
                return Some(Value::Node(nodes[0].clone()));
            }
            // 2.2 lookup in sub nodes
            if name.ends_with("s") {
                name = name[..name.len() - 1].into();
            }
            let nodes = node.get_nodes(&name);
            if nodes.len() > 1 {
                Some(Value::array_of(nodes.iter().map(|n| n.clone()).collect()))
            } else if nodes.len() == 1 {
                Some(Value::Node(nodes[0].clone()))
            } else {
                None
            }
        } else {
            Some(v)
        }
    }

    fn enum_val(&mut self, en: &AutoStr, name: &AutoStr) -> Value {
        // find enum's decl
        let typ = self.lookup_type(en);
        match typ {
            ast::Type::Enum(en) => {
                // lookup enum value in Enum's items
                match en.borrow().get_item(name) {
                    Some(item) => Value::Int(item.value),
                    None => Value::Nil,
                }
            }
            _ => Value::Nil,
        }
    }

    fn eval_tag_construction(
        &mut self,
        tag_name: &AutoStr,
        variant_name: &AutoStr,
        args: &ast::Args,
    ) -> AutoResult<Value> {
        // Get the tag type definition
        let tag_type = self.lookup_type(tag_name);
        match tag_type {
            ast::Type::Tag(tag) => {
                let tag = tag.borrow();
                // Find the variant in the tag definition
                let _field = tag
                    .fields
                    .iter()
                    .find(|f| f.name == *variant_name)
                    .ok_or_else(|| format!("Undefined variant: {}.{}", tag_name, variant_name))?;

                // Evaluate payload from arguments
                let payload = if args.args.len() > 0 {
                    match &args.args[0] {
                        ast::Arg::Pos(expr) => self.eval_expr(expr),
                        _ => Value::Nil,
                    }
                } else {
                    Value::Nil
                };

                // Create a Node to represent the tag value
                let mut node = auto_val::Node::new(tag.name.as_str());
                node.set_prop("variant", auto_val::Value::str(variant_name.as_str()));
                node.set_prop("payload", payload);

                Ok(auto_val::Value::Node(node))
            }
            _ => Ok(Value::Nil),
        }
    }

    fn dot(&mut self, left: &Expr, right: &Expr) -> Value {
        let mut left_value = self.eval_expr(left);

        // Resolve ValueRef to actual value
        if let Value::ValueRef(_vid) = &left_value {
            if let Some(data) = self.resolve_value(&left_value) {
                let borrowed_data = data.borrow();
                let data_clone = borrowed_data.clone();
                drop(borrowed_data);
                left_value = Value::from_data(data_clone);
            }
        }

        let res: Option<Value> = match &left_value {
            Value::Type(typ) => {
                match typ {
                    Type::Enum(en) => {
                        // lookup enum value in Enum's items
                        match right {
                            Expr::Ident(name) => Some(self.enum_val(en, name)),
                            _ => None,
                        }
                    }
                    Type::User(type_name) => {
                        // Handle static method calls on User types (e.g., HashMap.new(), List.new())
                        // Build the fully qualified function name (e.g., "HashMap.new", "List.new")
                        if let Expr::Ident(method_name) = right {
                            let qualified_name: AutoStr =
                                format!("{}.{}", type_name.as_str(), method_name).into();

                            // Look up the function in the VM registry
                            let registry = crate::vm::VM_REGISTRY.lock().unwrap();
                            let func_entry = registry
                                .modules()
                                .values()
                                .find_map(|module| module.functions.get(qualified_name.as_str()))
                                .cloned();
                            drop(registry);

                            if let Some(_func_entry) = func_entry {
                                // Return the function as a VmFunction metadata entry
                                let fn_decl = ast::Fn::new(
                                    ast::FnKind::VmFunction,
                                    qualified_name.clone(),
                                    None,
                                    vec![],
                                    ast::Body::new(),
                                    ast::Type::Unknown,
                                );

                                Some(Value::Meta(MetaID::Fn(to_value_sig(&fn_decl))))
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    }
                    _ => None,
                }
            }
            Value::Meta(meta_id) => {
                // lookup meta
                match meta_id {
                    MetaID::Enum(name) => {
                        let right_name = right.repr();
                        Some(self.enum_val(name, &AutoStr::from(right_name)))
                    }
                    MetaID::Type(type_name) => {
                        // Handle static method calls on Type meta (e.g., HashMap.new(), List.new())
                        if let Expr::Ident(method_name) = right {
                            let qualified_name: AutoStr =
                                format!("{}.{}", type_name, method_name).into();

                            // Look up the function in the VM registry
                            let registry = crate::vm::VM_REGISTRY.lock().unwrap();
                            let func_entry = registry
                                .modules()
                                .values()
                                .find_map(|module| module.functions.get(qualified_name.as_str()))
                                .cloned();
                            drop(registry);

                            if let Some(_func_entry) = func_entry {
                                // Return the function as a VmFunction metadata entry
                                let fn_decl = ast::Fn::new(
                                    ast::FnKind::VmFunction,
                                    qualified_name.clone(),
                                    None,
                                    vec![],
                                    ast::Body::new(),
                                    ast::Type::Unknown,
                                );

                                Some(Value::Meta(MetaID::Fn(to_value_sig(&fn_decl))))
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    }
                    _ => None,
                }
            }
            Value::Obj(obj) => match right {
                Expr::Ident(name) => {
                    let field_value = obj.lookup(&name);
                    // Recursively resolve ValueRef from field lookup
                    match &field_value {
                        Some(Value::ValueRef(vid)) => {
                            if let Some(data) = self.resolve_value(&Value::ValueRef(*vid)) {
                                let borrowed_data = data.borrow();
                                let data_clone = borrowed_data.clone();
                                drop(borrowed_data);
                                Some(Value::from_data(data_clone))
                            } else {
                                field_value.clone()
                            }
                        }
                        _ => field_value,
                    }
                }
                Expr::Int(key) => {
                    let field_value = obj.lookup(&key.to_string());
                    match &field_value {
                        Some(Value::ValueRef(vid)) => {
                            if let Some(data) = self.resolve_value(&Value::ValueRef(*vid)) {
                                let borrowed_data = data.borrow();
                                let data_clone = borrowed_data.clone();
                                drop(borrowed_data);
                                Some(Value::from_data(data_clone))
                            } else {
                                field_value.clone()
                            }
                        }
                        _ => field_value,
                    }
                }
                Expr::Bool(key) => {
                    let field_value = obj.lookup(&key.to_string());
                    match &field_value {
                        Some(Value::ValueRef(vid)) => {
                            if let Some(data) = self.resolve_value(&Value::ValueRef(*vid)) {
                                let borrowed_data = data.borrow();
                                let data_clone = borrowed_data.clone();
                                drop(borrowed_data);
                                Some(Value::from_data(data_clone))
                            } else {
                                field_value.clone()
                            }
                        }
                        _ => field_value,
                    }
                }
                _ => None,
            },
            Value::Node(node) => self.dot_node(node, right),
            Value::Widget(widget) => match right {
                Expr::Ident(name) => match name.as_str() {
                    "model" => Some(Value::Model(widget.model.clone())),
                    "view" => Some(Value::Meta(widget.view_id.clone())),
                    _ => None,
                },
                _ => None,
            },
            Value::Model(model) => match right {
                Expr::Ident(name) => model.find(&name),
                _ => None,
            },
            Value::View(view) => match right {
                Expr::Ident(name) => view.find(&name),
                _ => None,
            },
            Value::Instance(instance) => match right {
                Expr::Ident(name) => {
                    let f = instance.fields.lookup(&name);
                    match f {
                        Some(Value::ValueRef(vid)) => {
                            // Dereference the ValueRef to get the actual value
                            if let Some(data) = self.resolve_value(&Value::ValueRef(vid)) {
                                let borrowed_data = data.borrow();
                                let data_clone = borrowed_data.clone();
                                drop(borrowed_data);
                                Some(Value::from_data(data_clone))
                            } else {
                                None
                            }
                        }
                        Some(v) => Some(v),
                        None => {
                            // not a field, try method
                            let typ = instance.ty.name();
                            let combined_name: AutoStr = format!("{}.{}", typ, name).into();
                            // println!("Combined name: {}", combined_name); // LSP: disabled
                            let method = self.lookup_meta(&combined_name)  /* Phase 4.5 */;
                            if let Some(meta) = method {
                                match meta.as_ref() {
                                    scope::Meta::Fn(_) => Some(Value::Method(Method::new(
                                        left_value.clone(),
                                        combined_name,
                                    ))),
                                    _ => None,
                                }
                            } else {
                                None
                            }
                        }
                    }
                }
                _ => None,
            },

            _ => {
                // try to lookup method
                match right {
                    Expr::Ident(name) => {
                        // First, check the types system for built-in methods
                        let found_in_types = self
                            .universe
                            .borrow()
                            .types
                            .lookup_method_for_value(&left_value, name.clone())
                            .is_some();

                        if found_in_types {
                            Some(Value::Method(Method::new(left_value.clone(), name.clone())))
                        } else {
                            // Plan 025 String Migration: Check for ext methods in universe
                            // Build qualified name like "str::contains"
                            let type_name = self.get_type_name(&left_value);
                            let qualified_method_name: AutoStr =
                                format!("{}.{}", type_name, name).into();

                            // Check if this method exists in universe (registered by ext statement)
                            let method_exists = self
                                .universe
                                .borrow()
                                .lookup_meta(&qualified_method_name)
                                .is_some();

                            if method_exists {
                                // Return Method with qualified name
                                Some(Value::Method(Method::new(
                                    left_value.clone(),
                                    qualified_method_name,
                                )))
                            } else {
                                None
                            }
                        }
                    }
                    _ => None,
                }
            }
        };
        res.unwrap_or(Value::error(format!(
            "Invalid dot expression {}.{}",
            left_value.name(),
            right
        )))
    }

    fn eval_mid(&mut self, node: &Node) -> AutoResult<Value> {
        // Resolve ValueRef before converting to bool
        let is_mid_value = self
            .universe
            .borrow()
            .lookup_val("is_mid")
            .unwrap_or(Value::Bool(false));

        let is_mid = match &is_mid_value {
            Value::ValueRef(_vid) => {
                if let Some(data) = self.resolve_value(&is_mid_value) {
                    let borrowed_data = data.borrow();
                    match &*borrowed_data {
                        ValueData::Bool(b) => *b,
                        _ => false,
                    }
                } else {
                    false
                }
            }
            Value::Bool(b) => *b,
            _ => false,
        };

        let args = &node.args.args;
        let mut res = Value::Str("".into());
        if args.len() >= 1 {
            if is_mid {
                // mid
                let mid = self.eval_expr(&args[0].get_expr());
                res = mid;
            }
        }
        if args.len() >= 2 {
            if !is_mid {
                // last
                let last = self.eval_expr(&args[1].get_expr());
                res = last;
            }
        }
        if is_mid && node.body.stmts.len() != 0 {
            for stmt in node.body.stmts.iter() {
                let val = self.eval_stmt(stmt)?;
                res = val;
            }
        }
        Ok(res)
    }

    fn eval_arg(&mut self, arg: &ast::Arg) -> auto_val::Arg {
        match arg {
            ast::Arg::Name(name) => auto_val::Arg::Name(name.clone().into()),
            ast::Arg::Pair(name, expr) => {
                auto_val::Arg::Pair(ValueKey::Str(name.clone().into()), self.eval_expr(expr))
            }
            ast::Arg::Pos(expr) => auto_val::Arg::Pos(self.eval_expr(expr)),
        }
    }

    fn eval_args(&mut self, args: &ast::Args) -> auto_val::Args {
        let mut res = auto_val::Args::new();
        for arg in args.args.iter() {
            let val = self.eval_arg(arg);
            res.args.push(val);
        }
        res
    }

    fn eval_on_events(&mut self, events: &ast::OnEvents) -> Value {
        // TODO: currently only supports for AutoConfig
        let mut nd = auto_val::Node::new("on");
        for branch in events.branches.iter() {
            let mut ev = auto_val::Node::new("ev");
            match branch {
                Event::Arrow(arrow) => {
                    if let Some(src) = &arrow.src {
                        ev.set_prop("src", src.to_code());
                    } else {
                        ev.set_prop("src", "DEFAULT");
                    }
                    if let Some(dest) = &arrow.dest {
                        ev.set_prop("dest", dest.to_code());
                    } else {
                        ev.set_prop("dest", "None");
                    }
                    if let Some(handler) = &arrow.with {
                        ev.set_prop("with", handler.to_code());
                    } else {
                        ev.set_prop("with", "()");
                    }
                    nd.add_kid(ev);
                }
                Event::CondArrow(cond) => {
                    let src = if let Some(src) = &cond.src {
                        src.to_code()
                    } else {
                        "DEFAULT".into()
                    };
                    ev.set_prop("src", src.clone());
                    ev.set_prop("dest", "CONDITION");
                    ev.set_prop("with", cond.cond.to_code());
                    nd.add_kid(ev);
                    for arrow in cond.subs.iter() {
                        // println!("NEWSUB!!!! {}", arrow.with.clone().unwrap().to_code()); // LSP: disabled
                        let mut sub = auto_val::Node::new("ev");
                        sub.set_prop("src", src.clone());
                        if let Some(dest) = &arrow.dest {
                            sub.set_prop("dest", dest.to_code());
                        } else {
                            sub.set_prop("dest", "DEFAULT");
                        }
                        if let Some(handler) = &arrow.with {
                            sub.set_prop("with", handler.to_code());
                        } else {
                            sub.set_prop("with", "()");
                        }
                        nd.add_kid(sub);
                    }
                }
            }
        }
        Value::Node(nd)
    }

    // TODO: should node only be used in config mode?
    pub fn eval_node(&mut self, node: &Node) -> AutoResult<Value> {
        let name = node.name.clone();
        let expr = Expr::Ident(name);
        let name_expr = self.eval_expr(&expr);
        let mut args = self.eval_args(&node.args);

        // Remember original args count before adding body pairs
        let original_args_count = args.args.len();

        // Plan 056: Extract Pair properties from node.body and add them to args
        // This handles cases like `Point { x: 1, y: 2 }` where the properties are defined in body
        for stmt in node.body.stmts.iter() {
            if let Stmt::Expr(Expr::Pair(pair)) = stmt {
                // Convert AST Pair to auto_val Arg::Pair
                // pair.key is Key (NamedKey, IntKey, BoolKey, StrKey)
                // pair.value is Box<Expr> (needs to be evaluated)
                let key_name: AutoStr = match &pair.key {
                    ast::Key::NamedKey(name) => name.clone(),
                    ast::Key::StrKey(s) => s.clone(),
                    ast::Key::IntKey(i) => i.to_string().into(),
                    ast::Key::BoolKey(b) => b.to_string().into(),
                };
                let value_val = self.eval_expr(&pair.value);
                // Convert AutoStr to ValueKey using .into()
                args.args
                    .push(auto_val::Arg::Pair(key_name.into(), value_val));
            }
        }

        if let Value::Type(Type::User(type_decl)) = name_expr {
            // println!("EVAL TYPE _NEWNWNWN"); // LSP: disabled
            return Ok(self.eval_type_new(&type_decl, &args));
        }

        let mut nodes = Vec::new();
        let mut props = Obj::new();
        let mut body = MetaID::Nil;
        let name = &node.name;
        if name == "mid" {
            return self.eval_mid(&node);
        }
        let name: AutoStr = name.into();
        let tempo = self
            .tempo_for_nodes
            .get(&name)
            .unwrap_or(&EvalTempo::IMMEDIATE);

        match tempo {
            EvalTempo::IMMEDIATE => {
                // eval each stmts in body and extract props and sub nodes
                self.enter_scope();
                // put args as local values
                for arg in args.args.iter() {
                    match arg {
                        auto_val::Arg::Pair(name, value) => {
                            self.universe
                                .borrow_mut()
                                .set_local_val(&name.to_string().as_str(), value.clone());
                        }
                        _ => {}
                    }
                }
                for stmt in node.body.stmts.iter() {
                    let val = self.eval_stmt(stmt)?;
                    match val {
                        Value::Str(s) => {
                            let mut n = auto_val::Node::new("text");
                            n.text = s.clone();
                            // NEW: Use kids API with node name as key
                            // Will be added to kids after nd is created
                            nodes.push(n);
                        }
                        Value::Pair(key, value) => {
                            self.universe
                                .borrow_mut()
                                .set_local_val(&key.to_string(), *value.clone());
                            props.set(key, *value);
                        }
                        Value::Node(node) => {
                            nodes.push(node);
                        }
                        Value::Array(arr) | Value::Block(arr) => {
                            for item in arr.values.into_iter() {
                                match item {
                                    Value::Pair(key, value) => {
                                        props.set(key, *value);
                                    }
                                    Value::Obj(o) => {
                                        props.merge(&o);
                                    }
                                    Value::Node(n) => {
                                        nodes.push(n);
                                    }
                                    Value::Instance(inst) => {
                                        // Convert instance to node with type name as node name
                                        let mut kid_node = auto_val::Node::new(&inst.ty.name());
                                        // Add instance fields as node properties
                                        for (k, v) in inst.fields.iter() {
                                            kid_node.set_prop(k.clone(), v.clone());
                                        }
                                        nodes.push(kid_node);
                                    }
                                    _ => {
                                        props.set(item.to_astr(), item);
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
                self.exit_scope();
            }
            EvalTempo::LAZY => {
                // push node body to scope meta
                // TODO: support multiple nodes of same name
                body = MetaID::Body(name.clone().into());
                // println!("define global {}", name); // LSP: disabled
                self.universe
                    .borrow_mut()
                    .define_global(&name, Rc::new(Meta::Body(node.body.clone())));
            }
        }
        let mut nd = auto_val::Node::new(name);
        // id is not specified, try use first argument as id
        if !node.id.is_empty() {
            nd.id = node.id.clone();
        } else {
            let first_arg = node.args.first_arg();
            if let Some(Expr::Ident(ident)) = first_arg {
                let v = self.eval_ident(&ident);
                let v = self.deref_val(v);
                match v {
                    Value::Str(s) => {
                        nd.id = s;
                    }
                    Value::OwnedStr(s) => {
                        nd.id = s.as_str().into();
                    }
                    _ => {}
                }
            }
        }
        let ndid = nd.id.clone();
        nd.args = args.clone(); // Keep for backward compatibility

        // NEW: Populate unified props with args
        // Use original_args_count to distinguish between args and body props
        nd.num_args = original_args_count;
        for arg in args.args.iter() {
            match arg {
                auto_val::Arg::Pos(expr) => {
                    // Positional arg: use empty key
                    nd.set_prop("", expr.clone());
                }
                auto_val::Arg::Name(name) => {
                    // Named arg: use name as key with Str value
                    nd.set_prop(name.as_str(), Value::Str(name.clone()));
                }
                auto_val::Arg::Pair(key, _) => {
                    // Pair arg: extract key and value
                    match key {
                        auto_val::ValueKey::Str(k) => {
                            nd.set_prop(k.as_str(), arg.get_val());
                        }
                        auto_val::ValueKey::Int(i) => {
                            nd.set_prop(i.to_string(), arg.get_val());
                        }
                        auto_val::ValueKey::Bool(b) => {
                            nd.set_prop(b.to_string(), arg.get_val());
                        }
                    }
                }
            }
        }

        nd.merge_obj(props);

        // NEW: Use kids API instead of nodes and body_ref
        // Use integer indices as keys to preserve order and allow duplicates
        for (idx, node) in nodes.iter().enumerate() {
            nd.add_node_kid(idx as i32, node.clone());
        }
        if body != MetaID::Nil {
            nd.set_kids_ref(body.clone());
        }

        let nd = Value::Node(nd);
        // save value to scope
        if !ndid.is_empty() {
            self.set_global(ndid, nd.clone())  // Phase 4.5;
        }

        Ok(nd)
    }

    // fn eval_value_node_body(&mut self, node_val: &mut Value) {
    //     self.universe.borrow_mut().enter_scope();
    //     match node_val {
    //         Value::Node(ref mut node) => {
    //             let props = &mut node.props;
    //             let nodes = &mut node.nodes;
    //             let mut stmts = Vec::new();
    //             {
    //                 let scope = self.universe.borrow();
    //                 let meta = scope.lookup_meta(&node.name);
    //                 stmts = meta.map(|m| {
    //                     match m.as_ref() {
    //                         scope::Meta::Body(body) => body.stmts.clone(),
    //                         _ => Vec::new(),
    //                     }
    //                 }).unwrap();
    //             }
    //             for stmt in stmts.iter() {
    //                 let val = self.eval_stmt(stmt);
    //                 match val {
    //                     Value::Node(node) => {nodes.push(node);},
    //                     Value::Pair(key, value) => {props.set(key, *value);},
    //                     _ => {},
    //                 }
    //             }
    //         },
    //         _ => {},
    //     };
    //     self.universe.borrow_mut().exit_scope();
    // }

    /// Plan 060 Phase 4: Find variables referenced in closure body that should be captured
    /// Excludes: parameters, local variables defined in closure body
    fn find_captured_vars(
        &mut self,
        expr: &ast::Expr,
        params: &[ast::ClosureParam],
    ) -> HashMap<String, Value> {
        let mut captured = HashMap::new();
        let param_names: std::collections::HashSet<String> =
            params.iter().map(|p| p.name.to_string()).collect();

        // First collect all identifier names that need to be captured
        let mut names_to_capture = Vec::new();
        Self::collect_ident_names(expr, &param_names, &mut names_to_capture);

        // Then evaluate each one to get the current value
        for name in names_to_capture {
            if !captured.contains_key(&name) {
                // Use eval_expr to resolve the value like normal identifier evaluation
                let ident_expr = ast::Expr::Ident(name.clone().into());
                let val = self.eval_expr(&ident_expr);
                // Only capture if it's a real value (not an error)
                if !matches!(val, Value::Error(_)) {
                    captured.insert(name, val);
                }
            }
        }

        captured
    }

    /// Recursively collect identifier names from an expression (static analysis)
    fn collect_ident_names(
        expr: &ast::Expr,
        exclude: &std::collections::HashSet<String>,
        names: &mut Vec<String>,
    ) {
        match expr {
            ast::Expr::Ident(name) => {
                let name_str = name.to_string();
                // Skip if it's a parameter or already collected
                if !exclude.contains(&name_str) && !names.contains(&name_str) {
                    names.push(name_str);
                }
            }
            // Bina is a tuple: (Box<Expr>, Op, Box<Expr>)
            ast::Expr::Bina(left, _op, right) => {
                Self::collect_ident_names(left, exclude, names);
                Self::collect_ident_names(right, exclude, names);
            }
            // Unary is a tuple: (Op, Box<Expr>)
            ast::Expr::Unary(_op, operand) => {
                Self::collect_ident_names(operand, exclude, names);
            }
            ast::Expr::Call(call) => {
                // Call.name is the function/method being called
                Self::collect_ident_names(&call.name, exclude, names);
                for arg in &call.args.args {
                    if let ast::Arg::Pos(e) = arg {
                        Self::collect_ident_names(e, exclude, names);
                    }
                }
            }
            // Index is a tuple: (Box<Expr>, Box<Expr>)
            ast::Expr::Index(target, index) => {
                Self::collect_ident_names(target, exclude, names);
                Self::collect_ident_names(index, exclude, names);
            }
            // Dot is a tuple: (Box<Expr>, Name)
            ast::Expr::Dot(object, _field) => {
                Self::collect_ident_names(object, exclude, names);
            }
            ast::Expr::Block(body) => {
                for stmt in &body.stmts {
                    if let ast::Stmt::Expr(e) = stmt {
                        Self::collect_ident_names(e, exclude, names);
                    } else if let ast::Stmt::Return(e) = stmt {
                        Self::collect_ident_names(e, exclude, names);
                    }
                }
            }
            ast::Expr::If(if_expr) => {
                for branch in &if_expr.branches {
                    Self::collect_ident_names(&branch.cond, exclude, names);
                    for stmt in &branch.body.stmts {
                        if let ast::Stmt::Expr(e) = stmt {
                            Self::collect_ident_names(e, exclude, names);
                        }
                    }
                }
            }
            ast::Expr::Closure(inner_closure) => {
                // For nested closures, process inner body with updated excludes
                let mut inner_exclude = exclude.clone();
                for p in &inner_closure.params {
                    inner_exclude.insert(p.name.to_string());
                }
                Self::collect_ident_names(&inner_closure.body, &inner_exclude, names);
            }
            // Primitives - no identifiers to collect
            ast::Expr::Int(_)
            | ast::Expr::Float(_, _)
            | ast::Expr::Str(_)
            | ast::Expr::Bool(_)
            | ast::Expr::Nil
            | ast::Expr::Byte(_) => {}
            // Other expressions - add more cases as needed
            _ => {}
        }
    }

    /// Evaluate closure expression and create closure value (Plan 060 Phase 3+4)
    fn closure(&mut self, closure: &Closure) -> Value {
        

        // Generate unique closure ID
        let closure_id = self.next_closure_id;
        self.next_closure_id += 1;

        // Plan 060 Phase 4: Capture variables from enclosing scope
        let captured_env = self.find_captured_vars(&closure.body, &closure.params);

        // Store closure data in evaluator with captured environment
        let eval_closure = EvalClosure {
            params: closure.params.clone(),
            body: closure.body.clone(),
            env: captured_env, // Phase 4: Now populated with captured vars
        };
        self.closures.insert(closure_id, eval_closure);

        // Create closure value with ID
        let closure_val = auto_val::Closure {
            id: closure_id,
            params: closure.params.iter().map(|p| p.name.to_string()).collect(),
            name: format!("<closure_{}>", closure_id),
        };

        Value::Closure(closure_val)
    }

    /// Call a closure value (Plan 060 Phase 3+)
    fn call_closure(&mut self, closure: &auto_val::Closure, args: &ast::Args) -> AutoResult<Value> {
        // Get closure data from evaluator (clone to avoid borrow checker issues)
        let eval_closure = self
            .closures
            .get(&closure.id)
            .ok_or_else(|| {
                crate::error::AutoError::Msg(format!(
                    "Closure {} not found in evaluator",
                    closure.id
                ))
            })?
            .clone();

        // Check argument count
        let arg_count = args.args.len();
        let param_count = eval_closure.params.len();
        if arg_count != param_count {
            return Ok(Value::error(format!(
                "Closure arity mismatch: expected {} arguments, got {}",
                param_count, arg_count
            )));
        }

        // Evaluate arguments
        let mut arg_values = Vec::new();
        for arg in args.args.iter() {
            match arg {
                ast::Arg::Pos(expr) => {
                    arg_values.push(self.eval_expr(expr));
                }
                _ => {
                    return Ok(Value::error("Unsupported argument type in closure call"));
                }
            }
        }

        // Push new scope for closure execution
        self.enter_scope();  // Phase 4.5: Use bridge method

        // Plan 060 Phase 4: Restore captured environment
        for (name, value) in &eval_closure.env {
            self.set_local_val(name, value.clone());  // Phase 4.5: Use bridge method
        }

        // Bind parameters to arguments (after env, so params can shadow captured vars)
        for (param, arg_value) in eval_closure.params.iter().zip(arg_values.iter()) {
            let param_name = param.name.as_str();
            // Store the argument value in the current scope
            // This creates a ValueRef that can be resolved when the closure body references the parameter
            self.set_local_val(param_name, arg_value.clone());  // Phase 4.5: Use bridge method
        }

        // Execute closure body
        let result = self.eval_expr(&eval_closure.body);

        // Pop scope
        self.exit_scope();  // Phase 4.5: Use bridge method

        Ok(result)
    }

    fn fstr(&mut self, fstr: &FStr) -> Value {
        let parts: Vec<AutoStr> = fstr
            .parts
            .iter()
            .map(|part| {
                let val = self.eval_expr(part);
                match val {
                    Value::Str(s) => s,
                    // Resolve ValueRef before converting to string
                    Value::ValueRef(_vid) => {
                        if let Some(data) = self.resolve_value(&val) {
                            let borrowed_data = data.borrow();
                            let data_clone = borrowed_data.clone();
                            drop(borrowed_data);
                            let resolved_val = Value::from_data(data_clone);
                            resolved_val.to_astr()
                        } else {
                            val.to_astr()
                        }
                    }
                    _ => val.to_astr(),
                }
            })
            .collect();
        Value::Str(parts.join("").into())
    }

    fn grid(&mut self, grid: &Grid) -> Value {
        // head
        let mut head = Vec::new();
        let mut data = Vec::new();
        if grid.head.len() == 1 {
            let expr = &grid.head.args[0].get_expr();
            match expr {
                Expr::Array(array) => {
                    for elem in array.iter() {
                        if let Expr::Object(pairs) = elem {
                            for p in pairs.iter() {
                                match p.key.to_string().as_str() {
                                    "id" => {
                                        let id = self.eval_expr(&p.value);
                                        head.push((ValueKey::Str("id".to_string().into()), id));
                                    }
                                    k => {
                                        head.push((
                                            ValueKey::Str(k.to_string().into()),
                                            self.eval_expr(&p.value),
                                        ));
                                    }
                                }
                            }
                        }
                    }
                }
                Expr::Ident(_) => {
                    let val = self.eval_expr(expr);
                    if let Value::Array(array) = val {
                        for elem in array.into_iter() {
                            if let Value::Obj(obj) = &elem {
                                let id = obj.get_str("id");
                                match id {
                                    Some(id) => {
                                        head.push((ValueKey::Str(id.to_string().into()), elem));
                                    }
                                    None => {}
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        if head.len() == 0 {
            for arg in grid.head.args.iter() {
                match arg {
                    Arg::Pair(name, value) => {
                        head.push((ValueKey::Str(name.clone().into()), self.eval_expr(value)));
                    }
                    Arg::Pos(value) => match value {
                        Expr::Str(value) => {
                            head.push((
                                ValueKey::Str(value.clone().into()),
                                Value::Str(value.clone().into()),
                            ));
                        }
                        _ => {}
                    },
                    Arg::Name(name) => {
                        head.push((
                            ValueKey::Str(name.clone().into()),
                            Value::Str(name.clone().into()),
                        ));
                    }
                }
            }
        }
        for row in grid.data.iter() {
            let row_data = row.iter().map(|elem| self.eval_expr(elem)).collect();
            data.push(row_data);
        }
        Value::Grid(auto_val::Grid { head, data })
    }

    fn create_default_instance(&mut self, type_name: &str) -> Value {
        // Look up the type declaration
        let type_decl_opt = self.lookup_type(type_name);  // Phase 4.5: Use bridge method

        if let ast::Type::User(decl) = type_decl_opt {
            let mut fields = auto_val::Obj::new();

            // Initialize fields with default values
            for member in &decl.members {
                let val = if let Some(default_expr) = &member.value {
                    self.eval_expr(default_expr)
                } else {
                    // Use type-based default
                    match member.ty {
                        ast::Type::Int => Value::Int(0),
                        ast::Type::Uint => Value::Uint(0),
                        ast::Type::Float => Value::Float(0.0),
                        ast::Type::Double => Value::Double(0.0),
                        ast::Type::Bool => Value::Bool(false),
                        ast::Type::Byte => Value::Byte(0),
                        ast::Type::Str(_) => Value::empty_str(),
                        _ => Value::Nil,
                    }
                };

                // Allocate value in universe if needed
                fields.set(member.name.clone(), val);
            }

            Value::Instance(auto_val::Instance {
                ty: auto_val::Type::User(type_name.into()),
                fields,
            })
        } else {
            Value::error(format!("Type {} not found or invalid", type_name))
        }
    }
}

fn to_meta_id(meta: &Rc<scope::Meta>) -> MetaID {
    match meta.as_ref() {
        scope::Meta::Fn(fn_decl) => MetaID::Fn(to_value_sig(&fn_decl)),
        scope::Meta::Type(type_decl) => MetaID::Type(type_decl.unique_name().into()),
        scope::Meta::Enum(enum_decl) => MetaID::Enum(enum_decl.unique_name()),
        scope::Meta::Node(nd) => MetaID::Node(nd.id.clone()),
        _ => MetaID::Nil,
    }
}

fn to_value_sig(fn_decl: &Fn) -> Sig {
    let mut params = Vec::new();
    for param in fn_decl.params.iter() {
        params.push(auto_val::Param {
            name: param.name.clone().into(),
            ty: Box::new(to_value_type(&param.ty)),
        });
    }
    let ret = to_value_type(&fn_decl.ret);
    Sig {
        name: fn_decl.name.clone().into(),
        params,
        ret,
    }
}

// Plan 050 Step 4: Check if a Type is a May<T> type
fn is_may_type(ty: &crate::ast::Type) -> bool {
    use crate::ast::Type;
    match ty {
        Type::Tag(tag) => {
            let tag_ref = tag.borrow();
            let tag_name = tag_ref.name.as_str();
            // Check if this is a May type (either May_int from stdlib or MayInt from fallback)
            tag_name.starts_with("May_")
                || tag_name.starts_with("MayInt")
                || tag_name.starts_with("MayStr")
                || tag_name.starts_with("MayUint")
                || tag_name.starts_with("MayFloat")
                || tag_name.starts_with("MayDouble")
                || tag_name.starts_with("MayChar")
                || tag_name.starts_with("MayBool")
        }
        _ => false,
    }
}

// Plan 050 Step 4: Auto-wrap a value in May.val() if it's not already wrapped
fn auto_wrap_may(value: Value, _may_type: &crate::ast::Type) -> Value {
    // If value is already a Node with is_some property, assume it's already a May value
    if let Value::Node(node) = &value {
        if node.has_prop("is_some") {
            // Already wrapped, return as-is
            return value;
        }
    }

    // Otherwise, wrap in May.val()
    use auto_val::Node;

    let mut may_node = Node::new("May");
    may_node.set_prop("variant", auto_val::Value::str("val"));
    may_node.set_prop("is_some", auto_val::Value::Bool(true));
    may_node.set_prop("is_nil", auto_val::Value::Bool(false));
    may_node.set_prop("is_err", auto_val::Value::Bool(false));

    // Store the actual value in both "value" and "val" properties
    may_node.set_prop("value", value.clone());
    may_node.set_prop("val", value.clone());

    Value::Node(may_node)
}

fn to_value_type(ty: &ast::Type) -> auto_val::Type {
    match ty {
        ast::Type::Byte => auto_val::Type::Byte,
        ast::Type::Int => auto_val::Type::Int,
        ast::Type::Uint => auto_val::Type::Uint,
        ast::Type::USize => auto_val::Type::Uint, // TODO: should be U64?
        ast::Type::Float => auto_val::Type::Float,
        ast::Type::Double => auto_val::Type::Double,
        ast::Type::Bool => auto_val::Type::Bool,
        ast::Type::Char => auto_val::Type::Char,
        ast::Type::Str(_) => auto_val::Type::Str,
        ast::Type::CStr => auto_val::Type::CStr,
        ast::Type::StrSlice => auto_val::Type::StrSlice, // Borrowed string slice (Phase 3)
        ast::Type::Array(_) => auto_val::Type::Array,
        ast::Type::RuntimeArray(_) => auto_val::Type::Array, // Plan 052: Runtime arrays
        ast::Type::List(_) => auto_val::Type::Array,         // TODO: Add List to auto_val::Type
        ast::Type::Slice(_) => auto_val::Type::Array,        // TODO: Add Slice to auto_val::Type
        ast::Type::Ptr(_) => auto_val::Type::Ptr,
        ast::Type::Reference(_) => auto_val::Type::Ptr, // Plan 052: Reference transpiles to Ptr
        ast::Type::User(type_decl) => auto_val::Type::User(type_decl.name.clone()),
        ast::Type::Enum(decl) => auto_val::Type::Enum(decl.borrow().name.clone()),
        ast::Type::Spec(decl) => auto_val::Type::User(decl.borrow().name.clone()),
        ast::Type::Union(u) => auto_val::Type::Union(u.name.clone()),
        ast::Type::Tag(tag) => auto_val::Type::Tag(tag.borrow().name.clone()),
        ast::Type::Linear(inner) => to_value_type(inner), // Linear wraps inner type
        ast::Type::Variadic => auto_val::Type::Any,       // Variadic maps to Any
        ast::Type::Void => auto_val::Type::Void,
        ast::Type::Unknown => auto_val::Type::Any,
        ast::Type::CStruct(_) => auto_val::Type::Void,
        ast::Type::Storage(_) => auto_val::Type::Any, // Storage maps to Any for now
        ast::Type::Fn(_, _) => auto_val::Type::Any,   // Function type maps to Any for now
        ast::Type::GenericInstance(_) => auto_val::Type::Any, // TODO: Handle generic instances properly
    }
}

pub fn eval_basic_expr(expr: &Expr) -> Value {
    match expr {
        Expr::Str(s) => Value::Str(s.clone().into()),
        Expr::Byte(b) => Value::Byte(*b),
        Expr::Int(i) => Value::Int(*i),
        Expr::Float(f, _) => Value::Float(*f),
        Expr::Bool(b) => Value::Bool(*b),
        Expr::Char(c) => Value::Char(*c),
        _ => Value::error(format!("Unsupported basic expression: {:?}", expr)),
    }
}

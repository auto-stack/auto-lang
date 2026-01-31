// =============================================================================
// ExecutionEngine: Runtime execution state (ephemeral, per-run)
//
// The ExecutionEngine contains ONLY runtime state:
// - Variable values
// - Call stack management
// - VM references (file handles, collections, etc.)
//
// This is deliberately separated from compile-time concerns (types, symbols)
// which live in the Database.
//
// Phase 1.5: Basic ExecutionEngine structure (placeholder for future migration)
// Phase 2: Extract runtime logic from Universe (CURRENT - Plan 064)
// Phase 3: Full integration with Database

use crate::eval::Evaler;
use crate::universe::VmRefData;
use auto_val::{AutoStr, Obj, Value, ValueData, ValueID};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::{Rc, Weak};

/// Runtime execution engine (ephemeral, per-run)
///
/// Contains ONLY runtime state needed during execution.
/// Deliberately separated from compile-time concerns (Database).
///
/// # Architecture
///
/// ```text
/// Compile-time (persistent)    Runtime (ephemeral)
/// ┌──────────────────┐         ┌──────────────────┐
/// │   Database        │         │ ExecutionEngine  │
/// │ - Types           │   ←→    │ - Values         │
/// │ - Symbols         │         │ - VM Refs        │
/// │ - Fragments       │         │ - Stack          │
/// └──────────────────┘         └──────────────────┘
/// ```
///
/// # Phase 2: Plan 064 - Extract all runtime state from Universe
///
/// All runtime fields from Universe are now in ExecutionEngine.
pub struct ExecutionEngine {
    /// Environment variables
    pub env_vals: HashMap<AutoStr, String>,

    /// Command-line arguments
    pub args: Obj,

    /// Shared mutable values (across multiple evaluations)
    pub shared_vals: HashMap<AutoStr, Rc<RefCell<Value>>>,

    /// Builtin functions (cached for performance)
    pub builtins: HashMap<AutoStr, Value>,

    /// VM resource references (HashMap, HashSet, File, List, etc.)
    pub vm_refs: HashMap<usize, RefCell<VmRefData>>,

    /// VM reference ID counter
    pub vmref_counter: usize,

    /// Central value storage for reference-based system
    pub values: HashMap<ValueID, Rc<RefCell<ValueData>>>,

    /// Value ID counter
    pub value_counter: usize,

    /// Weak references to values (for cleanup)
    pub weak_refs: HashMap<ValueID, Weak<RefCell<ValueData>>>,

    /// Raw pointer to evaluator for VM functions to call user-defined functions
    /// WARNING: This is only valid during evaluator's lifetime
    /// The evaluator must outlive the ExecutionEngine
    evaluator_ptr: *mut Evaler,
}

impl ExecutionEngine {
    /// Create a new execution engine
    pub fn new() -> Self {
        // Get builtins from libs module
        let builtins = crate::libs::builtin::builtins();

        Self {
            env_vals: HashMap::new(),
            args: Obj::new(),
            shared_vals: HashMap::new(),
            builtins,
            vm_refs: HashMap::new(),
            vmref_counter: 0,
            values: HashMap::new(),
            value_counter: 0,
            weak_refs: HashMap::new(),
            evaluator_ptr: std::ptr::null_mut(),
        }
    }

    /// Set environment variable
    pub fn set_env_val(&mut self, name: &str, value: String) {
        self.env_vals.insert(name.into(), value);
    }

    /// Get environment variable
    pub fn get_env_val(&self, name: &str) -> Option<&str> {
        self.env_vals.get(name).map(|s| s.as_str())
    }

    /// Set arguments
    pub fn set_args(&mut self, args: &Obj) {
        self.args = args.clone();
    }

    /// Get arguments
    pub fn get_args(&self) -> &Obj {
        &self.args
    }

    // ========================================================================
    // VM Reference Management
    // ========================================================================

    /// Allocate a new VM reference ID
    pub fn alloc_vm_ref(&mut self, data: VmRefData) -> usize {
        let id = self.vmref_counter;
        self.vm_refs.insert(id, RefCell::new(data));
        self.vmref_counter += 1;
        id
    }

    /// Get a VM reference by ID
    pub fn get_vm_ref(&self, id: usize) -> Option<&RefCell<VmRefData>> {
        self.vm_refs.get(&id)
    }

    /// Drop a VM reference (cleanup)
    pub fn drop_vm_ref(&mut self, id: usize) {
        self.vm_refs.remove(&id);
    }

    // ========================================================================
    // Value Storage Management
    // ========================================================================

    /// Allocate a new value ID
    pub fn alloc_value(&mut self) -> ValueID {
        let id = ValueID(self.value_counter);
        self.value_counter += 1;
        id
    }

    /// Get a value by ID
    pub fn get_value(&self, id: ValueID) -> Option<&Rc<RefCell<ValueData>>> {
        self.values.get(&id)
    }

    /// Insert a value into storage
    pub fn insert_value(&mut self, id: ValueID, value: Rc<RefCell<ValueData>>) {
        self.values.insert(id, value);
    }

    /// Remove a value from storage
    pub fn remove_value(&mut self, id: ValueID) -> Option<Rc<RefCell<ValueData>>> {
        self.values.remove(&id)
    }

    // ========================================================================
    // Builtin Function Management
    // ========================================================================

    /// Get a builtin function by name
    pub fn get_builtin(&self, name: &str) -> Option<&Value> {
        self.builtins.get(name)
    }

    /// Insert a builtin function
    pub fn insert_builtin(&mut self, name: AutoStr, value: Value) {
        self.builtins.insert(name, value);
    }

    // ========================================================================
    // Evaluator Pointer Management (for VM → user function calls)
    // ========================================================================

    /// Set the evaluator pointer for VM functions to call user-defined functions
    /// # Safety
    /// The evaluator must outlive the ExecutionEngine. This is guaranteed by the
    /// ownership structure where Evaler owns the ExecutionEngine.
    pub fn set_evaluator(&mut self, evaluator: &mut Evaler) {
        self.evaluator_ptr = evaluator;
    }

    /// Set the evaluator pointer from a raw pointer
    /// # Safety
    /// The pointer must be valid and outlive the ExecutionEngine
    pub unsafe fn set_evaluator_raw(&mut self, evaluator: *mut Evaler) {
        self.evaluator_ptr = evaluator;
    }

    /// Evaluate a user-defined function using the stored evaluator pointer
    /// Returns None if no evaluator is set
    /// # Safety
    /// The evaluator pointer must be valid and outlive this call
    pub fn eval_user_fn(&self, fn_name: &AutoStr, args: Vec<Value>) -> Option<Value> {
        if self.evaluator_ptr.is_null() {
            return None;
        }
        // SAFETY: The evaluator outlives the engine during call chains
        // This is guaranteed by the ownership structure (Evaler owns ExecutionEngine)
        unsafe {
            Some((*self.evaluator_ptr).eval_user_function(fn_name, args))
        }
    }

    /// Get the raw evaluator pointer
    /// # Safety
    /// The pointer must only be used while the original borrow is active
    pub unsafe fn get_evaluator_ptr(&self) -> *mut Evaler {
        self.evaluator_ptr
    }
}

impl Default for ExecutionEngine {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use auto_val::Value;

    #[test]
    fn test_execution_engine_new() {
        let engine = ExecutionEngine::new();
        assert_eq!(engine.env_vals.len(), 0);
        assert_eq!(engine.vmref_counter, 0);
        assert_eq!(engine.value_counter, 0);
    }

    #[test]
    fn test_env_vals() {
        let mut engine = ExecutionEngine::new();

        engine.set_env_val("TEST", "value".to_string());
        assert_eq!(engine.get_env_val("TEST"), Some("value"));
        assert_eq!(engine.get_env_val("MISSING"), None);
    }

    #[test]
    fn test_args() {
        let mut engine = ExecutionEngine::new();
        let mut args = Obj::new();
        args.set("key", Value::Int(100));

        engine.set_args(&args);

        let retrieved = engine.get_args();
        assert_eq!(retrieved.get("key"), Some(Value::Int(100)));
    }

    #[test]
    fn test_vm_ref_management() {
        let mut engine = ExecutionEngine::new();

        // Allocate a VM ref
        let data = VmRefData::List(crate::universe::ListData::new());
        let id1 = engine.alloc_vm_ref(data);
        assert_eq!(id1, 0);

        // Allocate another
        let data2 = VmRefData::List(crate::universe::ListData::new());
        let id2 = engine.alloc_vm_ref(data2);
        assert_eq!(id2, 1);

        // Check retrieval
        assert!(engine.get_vm_ref(id1).is_some());
        assert!(engine.get_vm_ref(id2).is_some());

        // Check counter
        assert_eq!(engine.vmref_counter, 2);

        // Drop ref
        engine.drop_vm_ref(id1);
        assert!(engine.get_vm_ref(id1).is_none());
        assert!(engine.get_vm_ref(id2).is_some());
    }

    #[test]
    fn test_builtin_functions() {
        let engine = ExecutionEngine::new();

        // Should have builtins loaded
        assert!(engine.get_builtin("print").is_some());
        assert!(engine.get_builtin("str_new").is_some());

        // Should not have arbitrary functions
        assert!(engine.get_builtin("nonexistent").is_none());
    }

    #[test]
    fn test_value_management() {
        let mut engine = ExecutionEngine::new();

        // Allocate value IDs
        let id1 = engine.alloc_value();
        let id2 = engine.alloc_value();

        assert_eq!(id1.0, 0);
        assert_eq!(id2.0, 1);
        assert_eq!(engine.value_counter, 2);

        // Insert and retrieve values
        let value_data = Rc::new(RefCell::new(ValueData::Int(42)));
        engine.insert_value(id1, value_data.clone());

        let retrieved = engine.get_value(id1);
        assert!(retrieved.is_some());
        // Use pattern matching to check the value
        match &*retrieved.unwrap().borrow() {
            ValueData::Int(v) => assert_eq!(*v, 42),
            _ => panic!("Expected Int value"),
        }

        // Remove value
        let removed = engine.remove_value(id1);
        assert!(removed.is_some());
        assert!(engine.get_value(id1).is_none());
    }
}

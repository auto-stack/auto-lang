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
use crate::scope::Sid;
use crate::universe::VmRefData;
use auto_val::{AutoStr, Obj, Value, ValueData, ValueID};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::{Rc, Weak};

/// Stack frame identifier
pub type StackFrameId = usize;

/// Runtime stack frame (ephemeral)
///
/// Contains dynamic execution state: variable values,
/// ownership tracking, execution position. Created
/// when entering a scope, destroyed when exiting.
///
/// # Architecture (Plan 064 Phase 4)
///
/// ```text
/// Compile-time (Database)       Runtime (ExecutionEngine)
/// ┌──────────────────┐         ┌──────────────────┐
/// │   SymbolTable    │◄────────│   StackFrame      │
/// │ - kind, sid      │  link   │ - scope_sid       │
/// │ - symbols, types │         │ - vals            │
/// └──────────────────┘         │ - moved_vars      │
///                              │ - cur_block       │
///                              └──────────────────┘
/// ```
///
/// # Linkage
///
/// `StackFrame.scope_sid` → `SymbolTable.sid` (one-way reference)
/// - Runtime frame "belongs to" compile-time symbol table
/// - Multiple frames can reference the same symbol table (recursion support)
#[derive(Debug)]
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
    /// Create a new stack frame for a scope
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

    /// Remove a variable (returns old value if present)
    pub fn remove(&mut self, name: &str) -> Option<ValueID> {
        self.vals.remove(name)
    }
}

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

    /// Call stack (frame IDs) - Phase 4 Plan 064
    pub call_stack: Vec<StackFrameId>,

    /// Stack frame storage - Phase 4 Plan 064
    pub frames: HashMap<StackFrameId, RefCell<StackFrame>>,

    /// Frame ID counter - Phase 4 Plan 064
    pub frame_counter: StackFrameId,

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
            call_stack: Vec::new(),
            frames: HashMap::new(),
            frame_counter: 0,
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
    // Call Stack Management (Phase 4 Plan 064)
    // ========================================================================

    /// Push a new frame onto the call stack
    ///
    /// Creates a new StackFrame for the given scope and adds it to the call stack.
    /// The frame is automatically linked to its parent frame (if any).
    ///
    /// # Returns
    ///
    /// The ID of the newly created frame
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
    ///
    /// Removes and returns the ID of the top frame.
    /// Note: The frame data remains in storage for potential inspection.
    /// Future: Add cleanup method to remove orphaned frames.
    pub fn pop_frame(&mut self) -> Option<StackFrameId> {
        self.call_stack.pop()
    }

    /// Get the current (top) frame
    ///
    /// Returns the frame at the top of the call stack, or None if stack is empty.
    pub fn current_frame(&self) -> Option<&RefCell<StackFrame>> {
        self.call_stack.last().and_then(|id| self.frames.get(id))
    }

    /// Get a frame by ID
    pub fn get_frame(&self, frame_id: StackFrameId) -> Option<&RefCell<StackFrame>> {
        self.frames.get(&frame_id)
    }

    /// Look up a variable in the call stack
    ///
    /// Searches frames from top (most recent) to bottom (oldest).
    /// Returns the ValueID if found, None otherwise.
    ///
    /// This implements lexical scoping - inner frames shadow outer frames.
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

    // ========================================================================
    // StackFrame Tests (Phase 4 Plan 064)
    // ========================================================================

    #[test]
    fn test_stack_frame_new() {
        let scope_sid = Sid::from("test_scope");
        let frame = StackFrame::new(scope_sid.clone());

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
    fn test_stack_frame_moved_vars() {
        let mut frame = StackFrame::new(Sid::from("test"));

        // Initially not moved
        assert!(!frame.is_moved("x"));

        // Mark as moved
        frame.mark_moved(AutoStr::from("x"));
        assert!(frame.is_moved("x"));

        // Other variable not moved
        assert!(!frame.is_moved("y"));
    }

    #[test]
    fn test_stack_frame_remove() {
        let mut frame = StackFrame::new(Sid::from("test"));

        frame.set(AutoStr::from("x"), ValueID(42));
        assert_eq!(frame.get("x"), Some(ValueID(42)));

        // Remove variable
        let removed = frame.remove("x");
        assert_eq!(removed, Some(ValueID(42)));
        assert_eq!(frame.get("x"), None);

        // Remove non-existent variable
        assert_eq!(frame.remove("y"), None);
    }

    // ========================================================================
    // Call Stack Tests (Phase 4 Plan 064)
    // ========================================================================

    #[test]
    fn test_call_stack_push_pop() {
        let mut engine = ExecutionEngine::new();

        // Initially empty
        assert_eq!(engine.call_stack.len(), 0);
        assert!(engine.current_frame().is_none());

        // Push first frame
        let sid1 = Sid::from("scope1");
        let id1 = engine.push_frame(sid1.clone());
        assert_eq!(id1, 0);
        assert_eq!(engine.call_stack.len(), 1);
        assert!(engine.current_frame().is_some());

        // Push second frame
        let sid2 = Sid::from("scope2");
        let id2 = engine.push_frame(sid2.clone());
        assert_eq!(id2, 1);
        assert_eq!(engine.call_stack.len(), 2);

        // Check parent linkage
        {
            let frame2 = engine.get_frame(id2).unwrap().borrow();
            assert_eq!(frame2.parent_frame, Some(id1));
        } // Drop borrow before calling pop_frame

        // Pop frame
        let popped = engine.pop_frame();
        assert_eq!(popped, Some(id2));
        assert_eq!(engine.call_stack.len(), 1);

        // Current frame is now the first frame
        let current = engine.current_frame().unwrap().borrow();
        assert_eq!(current.scope_sid, sid1);
    }

    #[test]
    fn test_lookup_var() {
        let mut engine = ExecutionEngine::new();

        // Push frame with variable
        let sid1 = Sid::from("scope1");
        engine.push_frame(sid1);
        engine.current_frame().unwrap().borrow_mut()
            .set(AutoStr::from("x"), ValueID(100));

        // Look up variable
        assert_eq!(engine.lookup_var("x"), Some(ValueID(100)));

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

    #[test]
    fn test_lookup_var_not_found() {
        let mut engine = ExecutionEngine::new();

        // Empty stack
        assert_eq!(engine.lookup_var("x"), None);

        // Push frame without variable
        engine.push_frame(Sid::from("scope1"));
        assert_eq!(engine.lookup_var("x"), None);
    }

    #[test]
    fn test_frame_counter() {
        let mut engine = ExecutionEngine::new();

        // Frame IDs increment
        let id1 = engine.push_frame(Sid::from("scope1"));
        let id2 = engine.push_frame(Sid::from("scope2"));
        let id3 = engine.push_frame(Sid::from("scope3"));

        assert_eq!(id1, 0);
        assert_eq!(id2, 1);
        assert_eq!(id3, 2);
        assert_eq!(engine.frame_counter, 3);
    }

    #[test]
    fn test_get_frame_by_id() {
        let mut engine = ExecutionEngine::new();

        let sid = Sid::from("test_scope");
        let frame_id = engine.push_frame(sid.clone());

        // Get frame by ID
        let frame = engine.get_frame(frame_id);
        assert!(frame.is_some());

        let borrowed = frame.unwrap().borrow();
        assert_eq!(borrowed.scope_sid, sid);

        // Get non-existent frame
        assert!(engine.get_frame(999).is_none());
    }
}

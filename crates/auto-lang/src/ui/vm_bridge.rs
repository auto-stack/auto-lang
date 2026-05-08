//! # VmBridge - Bridge between AutoVM and the UI system
//!
//! This module provides [`VmBridge`], which replaces [`InterpreterBridge`] by using
//! the AutoVM (bytecode VM) instead of the old AST interpreter for dynamic UI rendering.
//!
//! ## Architecture
//!
//! ```text
//! AuraWidget (extracted from .at source)
//!    |
//!    v
//! VmBridge
//!  - Creates AutoVM instance
//!  - Stores widget state as GenericInstanceData on VM heap
//!  - Records handler addresses from LogicPayload::Bytecode
//!    |
//!    v
//! UI Backend (iced, GPUI, headless) reads state via read_state()
//! and triggers handlers via call_handler()
//! ```
//!
//! ## Plan 205 Phase 1
//!
//! Phase 1 focuses on state management:
//! - Initialize VM instance for a widget
//! - Load widget state as a VM heap object (GenericInstanceData)
//! - Read state fields from the VM
//! - Call handlers by name (if bytecode is available)

use std::collections::HashMap;

use crate::vm::engine::AutoVM;
use crate::vm::generic_registry::GenericInstanceData;
use crate::vm::task::AutoTask;
use crate::vm::virt_memory::VirtualFlash;
use crate::aura::{AuraExpr, AuraWidget, AuraStateDef, AuraNode, LogicPayload};
use auto_val::Value;

// ============================================================================
// Error Types
// ============================================================================

/// Errors that can occur during VmBridge operations.
///
/// UI should never crash from VM errors - all errors are graceful.
#[derive(Debug)]
pub enum VmBridgeError {
    /// Field not found in widget state
    FieldNotFound(String),
    /// Handler not found for the given event name
    HandlerNotFound(String),
    /// VM execution error
    VmError(String),
    /// Invalid state (e.g., corrupt heap object)
    InvalidState(String),
}

impl std::fmt::Display for VmBridgeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VmBridgeError::FieldNotFound(name) => {
                write!(f, "field not found: {}", name)
            }
            VmBridgeError::HandlerNotFound(name) => {
                write!(f, "handler not found: {}", name)
            }
            VmBridgeError::VmError(msg) => {
                write!(f, "VM error: {}", msg)
            }
            VmBridgeError::InvalidState(msg) => {
                write!(f, "invalid state: {}", msg)
            }
        }
    }
}

impl std::error::Error for VmBridgeError {}

pub type Result<T> = std::result::Result<T, VmBridgeError>;

// ============================================================================
// VmBridge
// ============================================================================

/// Bridge between AutoVM and the UI system.
///
/// Holds a VM instance with widget state and handler metadata.
/// Each widget gets its own VmBridge with an isolated VM.
///
/// # Lifecycle
///
/// 1. `VmBridge::new(widget)` - Create bridge, initialize state on heap
/// 2. `bridge.read_state("count")` - Read state field values for rendering
/// 3. `bridge.call_handler("Inc", &[])` - Execute handler on user interaction
/// 4. `bridge.read_state(...)` - Read updated state for re-rendering
pub struct VmBridge {
    /// AutoVM instance (owned, isolated per widget)
    vm: AutoVM,

    /// Widget state as a VM heap object ID.
    /// The heap object is a `GenericInstanceData` with field names and values.
    state_obj_id: u64,

    /// Handler mapping: event name (e.g., "Inc") -> closure ID in the VM.
    /// Closures are registered during `new()` from LogicPayload::Bytecode.
    handler_closures: HashMap<String, u32>,

    /// Handler mapping: event name -> function address in flash.
    /// Used when handlers are pre-compiled bytecode.
    handler_addrs: HashMap<String, u32>,

    /// State field names (ordered, matching GenericInstanceData field order)
    state_field_names: Vec<String>,

    /// Widget name for debugging
    widget_name: String,
}

impl VmBridge {
    /// Create a new VmBridge for a given AuraWidget.
    ///
    /// This initializes:
    /// 1. An AutoVM instance
    /// 2. Widget state as a `GenericInstanceData` on the VM heap
    /// 3. Handler mappings from the widget's `handlers` map
    ///
    /// # Arguments
    ///
    /// * `widget` - The AuraWidget to create a bridge for
    ///
    /// # Errors
    ///
    /// Returns an error if state initialization fails.
    pub fn new(widget: &AuraWidget) -> Result<Self> {
        // Ensure BIGVM_NATIVES is populated before AutoVM::new()
        // (normal codegen path calls this via Codegen::new(), but dynamic UI skips codegen)
        crate::vm::native_registry::register_builtin_natives();

        // 1. Create AutoVM instance with empty flash
        let flash = VirtualFlash::new(0);
        let vm = AutoVM::new(flash, 4096);

        Self::new_with_vm(vm, widget)
    }

    /// Create a new VmBridge with a pre-configured AutoVM instance.
    ///
    /// Use this when the VM already has bytecode loaded (e.g., from a compiled module).
    ///
    /// # Arguments
    ///
    /// * `vm` - Pre-configured AutoVM instance
    /// * `widget` - The AuraWidget to create a bridge for
    pub fn new_with_vm(mut vm: AutoVM, widget: &AuraWidget) -> Result<Self> {
        // Ensure BIGVM_NATIVES is populated (idempotent if already called)
        crate::vm::native_registry::register_builtin_natives();

        let widget_name = widget.name.clone();

        // 2. Build state fields and default values
        let mut field_names = Vec::with_capacity(widget.state_vars.len());
        let mut field_values = Vec::with_capacity(widget.state_vars.len());

        for state_var in &widget.state_vars {
            let default_value = eval_aura_expr_to_value(&state_var.initial);
            field_names.push(state_var.name.clone());
            field_values.push(default_value);
        }

        // 3. Create GenericInstanceData on the VM heap
        let mono_name = format!("{}_State", widget_name);
        let instance = GenericInstanceData::new_with_names(
            mono_name,
            field_values,
            field_names.clone(),
        );
        let state_obj_id = vm.insert_heap_object(instance);

        // 4. Process handlers - register bytecode or record addresses
        let mut handler_closures = HashMap::new();
        let mut handler_addrs = HashMap::new();

        for (event_pattern, payload) in &widget.handlers {
            // Extract the handler name from the pattern.
            // Patterns can be ".Inc", "Msg::Inc", or plain "Inc".
            let handler_name = extract_handler_name(event_pattern);

            match payload {
                LogicPayload::Bytecode(_bytes) => {
                    // For Phase 1, record that a handler exists with bytecode.
                    // The actual bytecode loading requires VMLoader integration
                    // which is deferred to Phase 2.
                    // For now, we check if the VM's flash has exports matching
                    // the handler name.
                    if let Some(&addr) = vm.flash.exports_by_name.get(handler_name) {
                        handler_addrs.insert(handler_name.to_string(), addr);
                    }
                    // If not found in exports, handler will fail gracefully at call time
                }
                LogicPayload::AstBlock(_stmts) => {
                    // AST blocks require compilation - deferred to Phase 2.
                    // For now, skip handler registration.
                }
                LogicPayload::AstStmts(_stmts) => {
                    // AutoLang AST statements - deferred to Phase 2.
                }
            }
        }

        Ok(Self {
            vm,
            state_obj_id,
            handler_closures,
            handler_addrs,
            state_field_names: field_names,
            widget_name,
        })
    }

    /// Read a state field value from the VM.
    ///
    /// Accesses the state heap object and returns the field value by name.
    ///
    /// # Arguments
    ///
    /// * `field_name` - Name of the state field (e.g., "count")
    ///
    /// # Returns
    ///
    /// The current value of the field, or an error if the field doesn't exist.
    pub fn read_state(&self, field_name: &str) -> Result<Value> {
        // Find field index by name
        let field_index = self.state_field_names.iter()
            .position(|name| name == field_name)
            .ok_or_else(|| VmBridgeError::FieldNotFound(field_name.to_string()))?;

        // Access the heap object
        let obj = self.vm.get_heap_object(self.state_obj_id)
            .ok_or_else(|| VmBridgeError::InvalidState(
                format!("state heap object {} not found", self.state_obj_id)
            ))?;

        let guard = obj.read().unwrap();
        let instance = guard.as_any().downcast_ref::<GenericInstanceData>()
            .ok_or_else(|| VmBridgeError::InvalidState(
                "state object is not a GenericInstanceData".to_string()
            ))?;

        instance.get_field(field_index)
            .cloned()
            .ok_or_else(|| VmBridgeError::FieldNotFound(field_name.to_string()))
    }

    /// Write a state field value to the VM.
    ///
    /// Updates the state heap object field by name.
    ///
    /// # Arguments
    ///
    /// * `field_name` - Name of the state field
    /// * `value` - New value for the field
    pub fn write_state(&mut self, field_name: &str, value: Value) -> Result<()> {
        // Find field index by name
        let field_index = self.state_field_names.iter()
            .position(|name| name == field_name)
            .ok_or_else(|| VmBridgeError::FieldNotFound(field_name.to_string()))?;

        // Access the heap object with write access
        let obj = self.vm.get_heap_object_mut(self.state_obj_id)
            .ok_or_else(|| VmBridgeError::InvalidState(
                format!("state heap object {} not found", self.state_obj_id)
            ))?;

        let mut guard = obj.write().unwrap();
        let instance = guard.as_any_mut().downcast_mut::<GenericInstanceData>()
            .ok_or_else(|| VmBridgeError::InvalidState(
                "state object is not a GenericInstanceData".to_string()
            ))?;

        instance.set_field(field_index, value)
            .map_err(|e| VmBridgeError::InvalidState(e))
    }

    /// Read all state fields as a name -> value map.
    ///
    /// Useful for bulk state reads during rendering.
    pub fn read_all_state(&self) -> HashMap<String, Value> {
        let mut result = HashMap::new();

        let obj = match self.vm.get_heap_object(self.state_obj_id) {
            Some(o) => o,
            None => return result,
        };

        let guard = obj.read().unwrap();
        let instance = match guard.as_any().downcast_ref::<GenericInstanceData>() {
            Some(i) => i,
            None => return result,
        };

        for (i, name) in self.state_field_names.iter().enumerate() {
            if let Some(value) = instance.get_field(i) {
                result.insert(name.clone(), value.clone());
            }
        }

        result
    }

    /// Call a handler by name with arguments.
    ///
    /// Looks up the handler's bytecode address or closure ID, then executes
    /// it via the VM. If the handler modifies state, the state heap object
    /// is updated in place.
    ///
    /// # Arguments
    ///
    /// * `event_name` - Handler name (e.g., "Inc", "Dec")
    /// * `args` - Arguments to pass to the handler
    ///
    /// # Errors
    ///
    /// Returns an error if the handler is not found or VM execution fails.
    /// UI should handle errors gracefully (log and continue).
    pub fn call_handler(&mut self, event_name: &str, _args: &[Value]) -> Result<()> {
        // Try closure-based handler first
        if let Some(&closure_id) = self.handler_closures.get(event_name) {
            let mut task = AutoTask::new(0, 1024, 0);

            return self.vm.call_closure(&mut task, closure_id, 0)
                .map_err(|e| VmBridgeError::VmError(format!("{:?}", e)));
        }

        // Try address-based handler
        if let Some(&_addr) = self.handler_addrs.get(event_name) {
            // Phase 1: Direct function call via address
            // This requires setting up a task and executing from the address.
            // For now, we create a task and run from the address.
            //
            // Full implementation (spawning task, pushing args, executing,
            // reading back state mutations) is deferred to Phase 2 when
            // VMLoader integration is complete.
            return Err(VmBridgeError::VmError(
                format!(
                    "handler '{}' found at address but direct address-based execution \
                     is not yet implemented (requires VMLoader integration, Phase 2)",
                     event_name
                )
            ));
        }

        Err(VmBridgeError::HandlerNotFound(event_name.to_string()))
    }

    /// Register a handler closure for a given event name.
    ///
    /// This is used when closures are created externally (e.g., from VMLoader)
    /// and need to be associated with a handler name.
    pub fn register_handler_closure(&mut self, event_name: &str, closure_id: u32) {
        self.handler_closures.insert(event_name.to_string(), closure_id);
    }

    /// Register a handler address for a given event name.
    ///
    /// This is used when function addresses are known from compiled bytecode.
    pub fn register_handler_addr(&mut self, event_name: &str, addr: u32) {
        self.handler_addrs.insert(event_name.to_string(), addr);
    }

    /// Get the widget name.
    pub fn widget_name(&self) -> &str {
        &self.widget_name
    }

    /// Get the state field names.
    pub fn state_fields(&self) -> &[String] {
        &self.state_field_names
    }

    /// Get the state heap object ID (for advanced VM integration).
    pub fn state_obj_id(&self) -> u64 {
        self.state_obj_id
    }

    /// Get a reference to the underlying AutoVM (for advanced VM integration).
    pub fn vm(&self) -> &AutoVM {
        &self.vm
    }

    /// Get a mutable reference to the underlying AutoVM.
    pub fn vm_mut(&mut self) -> &mut AutoVM {
        &mut self.vm
    }

    /// Check if a handler exists for the given event name.
    pub fn has_handler(&self, event_name: &str) -> bool {
        self.handler_closures.contains_key(event_name)
            || self.handler_addrs.contains_key(event_name)
    }

    /// List all registered handler names.
    pub fn handler_names(&self) -> Vec<&str> {
        let mut names: Vec<&str> = self.handler_closures.keys()
            .chain(self.handler_addrs.keys())
            .map(|s| s.as_str())
            .collect();
        names.sort();
        names.dedup();
        names
    }
}

// ============================================================================
// Helpers
// ============================================================================

/// Convert an AuraExpr initial value to a runtime Value.
///
/// Handles the common literal types. Complex expressions default to Nil.
fn eval_aura_expr_to_value(expr: &AuraExpr) -> Value {
    match expr {
        AuraExpr::Int(i) => Value::Int(*i as i32),
        AuraExpr::Float(f) => Value::Float(*f as f64),
        AuraExpr::Bool(b) => Value::Bool(*b),
        AuraExpr::Literal(s) => Value::str(s),
        AuraExpr::StateRef(name) => {
            // State references in initial values default to zero/null.
            // This is a placeholder - the actual value should be resolved
            // from the state after all fields are initialized.
            Value::Int(0)
        }
        AuraExpr::Binary { .. } => {
            // Binary expressions in initial values: evaluate simple cases
            Value::Int(0)
        }
        AuraExpr::Unary { .. } => Value::Int(0),
        AuraExpr::Array(elements) => {
            // Array literals: evaluate each element
            let values: Vec<Value> = elements.iter()
                .map(|e| eval_aura_expr_to_value(e))
                .collect();
            Value::Array(auto_val::Array::from(values))
        }
        AuraExpr::Object(fields) => {
            // Object literals: evaluate each field
            let mut obj = auto_val::Obj::new();
            for (key, val_expr) in fields {
                obj.set(key.clone(), eval_aura_expr_to_value(val_expr));
            }
            Value::Obj(obj)
        }
        // Complex expressions default to Nil for safety
        _ => Value::Nil,
    }
}

/// Extract a clean handler name from an event pattern.
///
/// Patterns can be:
/// - ".Inc" -> "Inc"
/// - "Msg::Inc" -> "Inc"
/// - "Inc" -> "Inc"
fn extract_handler_name(pattern: &str) -> &str {
    let name = pattern.trim_start_matches('.');
    if let Some(pos) = name.rfind("::") {
        &name[pos + 2..]
    } else {
        name
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Type;

    /// Helper to create a minimal AuraWidget for testing
    fn make_test_widget(name: &str, state_vars: Vec<AuraStateDef>) -> AuraWidget {
        AuraWidget {
            name: name.to_string(),
            state_vars,
            computed: vec![],
            messages: vec![],
            view_tree: AuraNode::element("col"),
            handlers: HashMap::new(),
            props: vec![],
            routes: None,
            lifecycle: vec![],
            tick_interval: None,
            handler_params: HashMap::new(),
        }
    }

    #[test]
    fn test_vm_bridge_creation_empty_state() {
        let widget = make_test_widget("EmptyWidget", vec![]);
        let bridge = VmBridge::new(&widget).unwrap();

        assert_eq!(bridge.widget_name(), "EmptyWidget");
        assert!(bridge.state_fields().is_empty());
        assert!(bridge.handler_names().is_empty());
    }

    #[test]
    fn test_vm_bridge_creation_with_state() {
        let widget = make_test_widget("Counter", vec![
            AuraStateDef {
                name: "count".to_string(),
                type_info: Type::Int,
                initial: AuraExpr::Int(0),
                decorators: vec![],
            },
            AuraStateDef {
                name: "label".to_string(),
                type_info: Type::StrFixed(0),
                initial: AuraExpr::Literal("Hello".to_string()),
                decorators: vec![],
            },
        ]);

        let bridge = VmBridge::new(&widget).unwrap();

        assert_eq!(bridge.widget_name(), "Counter");
        assert_eq!(bridge.state_fields().len(), 2);
        assert_eq!(bridge.state_fields()[0], "count");
        assert_eq!(bridge.state_fields()[1], "label");
    }

    #[test]
    fn test_read_state_int() {
        let widget = make_test_widget("Counter", vec![
            AuraStateDef {
                name: "count".to_string(),
                type_info: Type::Int,
                initial: AuraExpr::Int(42),
                decorators: vec![],
            },
        ]);

        let bridge = VmBridge::new(&widget).unwrap();

        let value = bridge.read_state("count").unwrap();
        assert_eq!(value, Value::Int(42));
    }

    #[test]
    fn test_read_state_string() {
        let widget = make_test_widget("Greeter", vec![
            AuraStateDef {
                name: "greeting".to_string(),
                type_info: Type::StrFixed(0),
                initial: AuraExpr::Literal("Hello World".to_string()),
                decorators: vec![],
            },
        ]);

        let bridge = VmBridge::new(&widget).unwrap();

        let value = bridge.read_state("greeting").unwrap();
        assert_eq!(value, Value::str("Hello World"));
    }

    #[test]
    fn test_read_state_bool() {
        let widget = make_test_widget("Toggle", vec![
            AuraStateDef {
                name: "active".to_string(),
                type_info: Type::Bool,
                initial: AuraExpr::Bool(true),
                decorators: vec![],
            },
        ]);

        let bridge = VmBridge::new(&widget).unwrap();

        let value = bridge.read_state("active").unwrap();
        assert_eq!(value, Value::Bool(true));
    }

    #[test]
    fn test_read_state_not_found() {
        let widget = make_test_widget("Counter", vec![
            AuraStateDef {
                name: "count".to_string(),
                type_info: Type::Int,
                initial: AuraExpr::Int(0),
                decorators: vec![],
            },
        ]);

        let bridge = VmBridge::new(&widget).unwrap();

        let result = bridge.read_state("nonexistent");
        assert!(result.is_err());
        match result.unwrap_err() {
            VmBridgeError::FieldNotFound(name) => assert_eq!(name, "nonexistent"),
            other => panic!("Expected FieldNotFound, got {:?}", other),
        }
    }

    #[test]
    fn test_write_state() {
        let widget = make_test_widget("Counter", vec![
            AuraStateDef {
                name: "count".to_string(),
                type_info: Type::Int,
                initial: AuraExpr::Int(0),
                decorators: vec![],
            },
        ]);

        let mut bridge = VmBridge::new(&widget).unwrap();

        // Write new value
        bridge.write_state("count", Value::Int(10)).unwrap();

        // Read back
        let value = bridge.read_state("count").unwrap();
        assert_eq!(value, Value::Int(10));
    }

    #[test]
    fn test_write_state_not_found() {
        let widget = make_test_widget("Counter", vec![]);
        let mut bridge = VmBridge::new(&widget).unwrap();

        let result = bridge.write_state("nope", Value::Int(1));
        assert!(result.is_err());
    }

    #[test]
    fn test_read_all_state() {
        let widget = make_test_widget("Multi", vec![
            AuraStateDef {
                name: "x".to_string(),
                type_info: Type::Int,
                initial: AuraExpr::Int(1),
                decorators: vec![],
            },
            AuraStateDef {
                name: "y".to_string(),
                type_info: Type::Int,
                initial: AuraExpr::Int(2),
                decorators: vec![],
            },
            AuraStateDef {
                name: "name".to_string(),
                type_info: Type::StrFixed(0),
                initial: AuraExpr::Literal("test".to_string()),
                decorators: vec![],
            },
        ]);

        let bridge = VmBridge::new(&widget).unwrap();

        let state = bridge.read_all_state();
        assert_eq!(state.len(), 3);
        assert_eq!(state.get("x"), Some(&Value::Int(1)));
        assert_eq!(state.get("y"), Some(&Value::Int(2)));
        assert_eq!(state.get("name"), Some(&Value::str("test")));
    }

    #[test]
    fn test_read_all_state_after_write() {
        let widget = make_test_widget("Counter", vec![
            AuraStateDef {
                name: "count".to_string(),
                type_info: Type::Int,
                initial: AuraExpr::Int(0),
                decorators: vec![],
            },
        ]);

        let mut bridge = VmBridge::new(&widget).unwrap();

        bridge.write_state("count", Value::Int(99)).unwrap();

        let state = bridge.read_all_state();
        assert_eq!(state.get("count"), Some(&Value::Int(99)));
    }

    #[test]
    fn test_call_handler_not_found() {
        let widget = make_test_widget("Counter", vec![]);
        let mut bridge = VmBridge::new(&widget).unwrap();

        let result = bridge.call_handler("NonExistent", &[]);
        assert!(result.is_err());
        match result.unwrap_err() {
            VmBridgeError::HandlerNotFound(name) => assert_eq!(name, "NonExistent"),
            other => panic!("Expected HandlerNotFound, got {:?}", other),
        }
    }

    #[test]
    fn test_has_handler() {
        let widget = make_test_widget("Counter", vec![]);
        let mut bridge = VmBridge::new(&widget).unwrap();

        assert!(!bridge.has_handler("Inc"));

        bridge.register_handler_addr("Inc", 42);
        assert!(bridge.has_handler("Inc"));
    }

    #[test]
    fn test_handler_names() {
        let widget = make_test_widget("Counter", vec![]);
        let mut bridge = VmBridge::new(&widget).unwrap();

        bridge.register_handler_addr("Inc", 10);
        bridge.register_handler_addr("Dec", 20);
        bridge.register_handler_closure("Reset", 5);

        let names = bridge.handler_names();
        assert_eq!(names.len(), 3);
        assert!(names.contains(&"Dec"));
        assert!(names.contains(&"Inc"));
        assert!(names.contains(&"Reset"));
    }

    #[test]
    fn test_vm_access() {
        let widget = make_test_widget("Test", vec![]);
        let bridge = VmBridge::new(&widget).unwrap();

        // Verify VM is accessible
        assert_eq!(bridge.vm().heap_object_count(), 1); // state object
    }

    #[test]
    fn test_state_obj_id() {
        let widget = make_test_widget("Test", vec![]);
        let bridge = VmBridge::new(&widget).unwrap();

        // State object ID should be a valid heap object ID
        let id = bridge.state_obj_id();
        assert!(id >= 4000000); // Heap object IDs start at 4000000
    }

    #[test]
    fn test_extract_handler_name() {
        assert_eq!(extract_handler_name(".Inc"), "Inc");
        assert_eq!(extract_handler_name("Msg::Inc"), "Inc");
        assert_eq!(extract_handler_name("Inc"), "Inc");
        assert_eq!(extract_handler_name(".AddItem"), "AddItem");
        assert_eq!(extract_handler_name("Event::Click::Press"), "Press");
    }

    #[test]
    fn test_eval_aura_expr() {
        assert_eq!(eval_aura_expr_to_value(&AuraExpr::Int(42)), Value::Int(42));
        assert_eq!(eval_aura_expr_to_value(&AuraExpr::Float(3.14)), Value::Float(3.14f64));
        assert_eq!(eval_aura_expr_to_value(&AuraExpr::Bool(true)), Value::Bool(true));
        assert_eq!(eval_aura_expr_to_value(&AuraExpr::Literal("hi".into())), Value::str("hi"));
    }

    #[test]
    fn test_eval_aura_expr_array() {
        let expr = AuraExpr::Array(vec![AuraExpr::Int(1), AuraExpr::Int(2)]);
        let val = eval_aura_expr_to_value(&expr);
        match val {
            Value::Array(arr) => {
                assert_eq!(arr.len(), 2);
                assert_eq!(arr[0], Value::Int(1));
                assert_eq!(arr[1], Value::Int(2));
            }
            other => panic!("Expected Array, got {:?}", other),
        }
    }

    #[test]
    fn test_multiple_writes() {
        let widget = make_test_widget("Counter", vec![
            AuraStateDef {
                name: "count".to_string(),
                type_info: Type::Int,
                initial: AuraExpr::Int(0),
                decorators: vec![],
            },
        ]);

        let mut bridge = VmBridge::new(&widget).unwrap();

        // Simulate incrementing counter
        for i in 1..=5 {
            bridge.write_state("count", Value::Int(i)).unwrap();
        }

        assert_eq!(bridge.read_state("count").unwrap(), Value::Int(5));
    }
}

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

use crate::vm::engine::{AutoVM, Closure};
use crate::vm::generic_registry::GenericInstanceData;
use crate::vm::opcode::OpCode;
use crate::vm::task::AutoTask;
use crate::vm::virt_memory::VirtualFlash;
use crate::aura::{AuraExpr, AuraWidget, AuraStateDef, AuraNode, LogicPayload};
use auto_val::{Op, Value};

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
        crate::vm::native_registry::register_builtin_natives();

        // 1. Pre-compile all handler AST statements into bytecode
        let mut handler_bytecode: Vec<(String, Vec<u8>)> = Vec::new();
        let mut all_strings: Vec<String> = Vec::new();
        let field_names: Vec<String> = widget.state_vars.iter().map(|v| v.name.clone()).collect();
        let temp_state_id = 4000000u64;

        // Sort handlers by name for deterministic compilation order
        let mut sorted_handlers: Vec<_> = widget.handlers.iter().collect();
        sorted_handlers.sort_by_key(|(pattern, _)| extract_handler_name(pattern).to_string());

        for (event_pattern, payload) in &sorted_handlers {
            if let LogicPayload::AstStmts(stmts) = payload {
                let handler_name = extract_handler_name(event_pattern);
                let string_base = all_strings.len();
                if let Ok((bytecode, strings)) = compile_handler_stmts(stmts, temp_state_id, &field_names, string_base) {
                    all_strings.extend(strings);
                    handler_bytecode.push((handler_name.to_string(), bytecode));
                }
            }
        }

        // 2. Build flash with all handler bytecode
        let mut flash = VirtualFlash::new(0);
        let mut handler_addrs: HashMap<String, u32> = HashMap::new();
        for (name, bytecode) in &handler_bytecode {
            let addr = flash.memory.len() as u32;
            flash.memory.extend_from_slice(bytecode);
            handler_addrs.insert(name.clone(), addr);
        }

        // 3. Create VM
        let mut vm = AutoVM::new(flash, 4096);

        // 4. Add string constants to VM's string pool
        {
            let mut strings_pool = vm.strings.write().unwrap();
            for s in &all_strings {
                strings_pool.push(s.clone().into_bytes());
            }
        }

        Self::new_with_vm_and_handlers(vm, widget, handler_addrs)
    }

    /// Create a new VmBridge with a pre-configured AutoVM instance.
    ///
    /// Use this when the VM already has bytecode loaded (e.g., from a compiled module).
    ///
    /// # Arguments
    ///
    /// * `vm` - Pre-configured AutoVM instance
    /// * `widget` - The AuraWidget to create a bridge for
    pub fn new_with_vm(vm: AutoVM, widget: &AuraWidget) -> Result<Self> {
        Self::new_with_vm_and_handlers(vm, widget, HashMap::new())
    }

    /// Create a new VmBridge with pre-compiled handler addresses.
    ///
    /// This is the core constructor used by both `new()` and `new_with_vm()`.
    /// Handler bytecode should already be loaded into flash before calling this.
    ///
    /// # Arguments
    ///
    /// * `vm` - Pre-configured AutoVM instance with handler bytecode in flash
    /// * `widget` - The AuraWidget to create a bridge for
    /// * `pre_compiled_addrs` - Handler name -> flash address mappings from pre-compilation
    pub fn new_with_vm_and_handlers(
        mut vm: AutoVM,
        widget: &AuraWidget,
        pre_compiled_addrs: HashMap<String, u32>,
    ) -> Result<Self> {
        // Ensure BIGVM_NATIVES is populated (idempotent if already called)
        crate::vm::native_registry::register_builtin_natives();

        let widget_name = widget.name.clone();

        // Build state fields and default values
        let mut field_names = Vec::with_capacity(widget.state_vars.len());
        let mut field_values = Vec::with_capacity(widget.state_vars.len());

        for state_var in &widget.state_vars {
            let default_value = eval_aura_expr_to_value(&state_var.initial);
            field_names.push(state_var.name.clone());
            field_values.push(default_value);
        }

        // Create GenericInstanceData on the VM heap
        let mono_name = format!("{}_State", widget_name);
        let instance = GenericInstanceData::new_with_names(
            mono_name,
            field_values,
            field_names.clone(),
        );
        let state_obj_id = vm.insert_heap_object(instance);

        // Register pre-compiled handlers as closures
        let mut handler_closures = HashMap::new();
        let mut handler_addrs = HashMap::new();

        // First: register pre-compiled handlers (from new() path)
        for (name, func_addr) in pre_compiled_addrs {
            let closure_id = vm.closure_id_gen.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            vm.closures.insert(closure_id, Closure {
                func_addr,
                env: HashMap::new(),
                n_args: 0,
            });
            handler_closures.insert(name, closure_id);
        }

        // Then: process any remaining handlers from widget definition
        for (event_pattern, payload) in &widget.handlers {
            let handler_name = extract_handler_name(event_pattern);

            // Skip if already registered via pre-compilation
            if handler_closures.contains_key(handler_name) {
                continue;
            }

            match payload {
                LogicPayload::Bytecode(_bytes) => {
                    if let Some(&addr) = vm.flash.exports_by_name.get(handler_name) {
                        handler_addrs.insert(handler_name.to_string(), addr);
                    }
                }
                LogicPayload::AstBlock(_stmts) => {
                    // AST blocks require compilation - not yet supported
                }
                LogicPayload::AstStmts(_stmts) => {
                    // Already handled via pre-compilation in new()
                    // If we reach here, pre-compilation was skipped (new_with_vm path)
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
            return Err(VmBridgeError::VmError(
                format!("handler '{}' found at address but direct address-based execution is not yet implemented", event_name)
            ));
        }

        Err(VmBridgeError::HandlerNotFound(event_name.to_string()))
    }

    /// Call a handler by interpreting its AST statements directly against state.
    ///
    /// This is the primary execution path for dynamic UI handlers. It parses the
    /// handler's AST body and evaluates assignments/expressions against the
    /// state heap object without requiring bytecode compilation.
    pub fn call_handler_ast(&mut self, event_name: &str, stmts: &[crate::ast::Stmt]) -> Result<()> {
        for stmt in stmts {
            self.exec_stmt(stmt)?;
        }
        Ok(())
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

    // ========================================================================
    // AST interpreter for handler bodies
    // ========================================================================

    /// Execute a single AST statement against the widget state.
    fn exec_stmt(&mut self, stmt: &crate::ast::Stmt) -> Result<()> {
        match stmt {
            crate::ast::Stmt::Expr(expr) => {
                self.eval_assign(expr)?;
                Ok(())
            }
            crate::ast::Stmt::Store(store) => {
                let value = self.eval_expr(&store.expr)?;
                self.write_state(store.name.as_str(), value)
            }
            crate::ast::Stmt::If(if_stmt) => {
                // Check first branch condition
                if let Some(branch) = if_stmt.branches.first() {
                    let cond = self.eval_expr(&branch.cond)?;
                    if cond.as_bool() {
                        for s in &branch.body.stmts {
                            self.exec_stmt(s)?;
                        }
                    } else if let Some(else_body) = &if_stmt.else_ {
                        for s in &else_body.stmts {
                            self.exec_stmt(s)?;
                        }
                    }
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }

    /// Evaluate an assignment expression (e.g., `count = count + 1`).
    fn eval_assign(&mut self, expr: &crate::ast::Expr) -> Result<()> {
        match expr {
            crate::ast::Expr::Bina(lhs, op, rhs) if matches!(op, Op::Asn) => {
                let value = self.eval_expr(rhs)?;
                let target = self.resolve_assign_target(lhs)?;
                self.write_state(&target, value)
            }
            crate::ast::Expr::Bina(lhs, op, rhs) if matches!(op, Op::AddEq | Op::SubEq | Op::MulEq | Op::DivEq) => {
                let target = self.resolve_assign_target(lhs)?;
                let current = self.read_state(&target)?;
                let rhs_val = self.eval_expr(rhs)?;
                let new_val = self.apply_compound_op(&current, op, &rhs_val);
                self.write_state(&target, new_val)
            }
            _ => Ok(()),
        }
    }

    /// Resolve the target name for an assignment (e.g., `count` or `.count` -> "count").
    fn resolve_assign_target(&self, expr: &crate::ast::Expr) -> Result<String> {
        match expr {
            crate::ast::Expr::Ident(name) => Ok(name.as_str().to_string()),
            crate::ast::Expr::Dot(obj, field) => {
                if let crate::ast::Expr::Ident(obj_name) = obj.as_ref() {
                    if obj_name.as_str() == "self" || obj_name.as_str() == "." {
                        return Ok(field.as_str().to_string());
                    }
                }
                Ok(field.as_str().to_string())
            }
            _ => Err(VmBridgeError::InvalidState(
                format!("cannot assign to expression: {:?}", expr)
            )),
        }
    }

    /// Evaluate an expression against the current state.
    fn eval_expr(&self, expr: &crate::ast::Expr) -> Result<Value> {
        match expr {
            crate::ast::Expr::Int(i) => Ok(Value::Int(*i)),
            crate::ast::Expr::Float(f, _) => Ok(Value::Float(*f)),
            crate::ast::Expr::Double(f, _) => Ok(Value::Double(*f)),
            crate::ast::Expr::Bool(b) => Ok(Value::Bool(*b)),
            crate::ast::Expr::Str(s) => Ok(Value::str(s.as_str())),
            crate::ast::Expr::Ident(name) => self.read_state(name.as_str()),
            crate::ast::Expr::Dot(obj, field) => {
                if let crate::ast::Expr::Ident(obj_name) = obj.as_ref() {
                    if obj_name.as_str() == "self" || obj_name.as_str() == "." {
                        return self.read_state(field.as_str());
                    }
                }
                Ok(Value::Nil)
            }
            crate::ast::Expr::Unary(op, operand) => {
                let val = self.eval_expr(operand)?;
                match op {
                    Op::Not => Ok(Value::Bool(!val.as_bool())),
                    Op::Sub => match val {
                        Value::Int(i) => Ok(Value::Int(-i)),
                        Value::Float(f) => Ok(Value::Float(-f)),
                        _ => Ok(Value::Nil),
                    },
                    _ => Ok(val),
                }
            }
            crate::ast::Expr::Bina(lhs, op, rhs) => {
                let l = self.eval_expr(lhs)?;
                let r = self.eval_expr(rhs)?;
                Ok(self.apply_binop(&l, op, &r))
            }
            _ => Ok(Value::Nil),
        }
    }

    /// Apply a binary operation to two values.
    fn apply_binop(&self, lhs: &Value, op: &Op, rhs: &Value) -> Value {
        match (lhs, rhs) {
            (Value::Int(a), Value::Int(b)) => match op {
                Op::Add => Value::Int(a + b),
                Op::Sub => Value::Int(a - b),
                Op::Mul => Value::Int(a * b),
                Op::Div if *b != 0 => Value::Int(a / b),
                Op::Mod if *b != 0 => Value::Int(a % b),
                Op::Eq => Value::Bool(a == b),
                Op::Neq => Value::Bool(a != b),
                Op::Lt => Value::Bool(a < b),
                Op::Gt => Value::Bool(a > b),
                Op::Le => Value::Bool(a <= b),
                Op::Ge => Value::Bool(a >= b),
                _ => Value::Nil,
            },
            (Value::Float(a), Value::Float(b)) => match op {
                Op::Add => Value::Float(a + b),
                Op::Sub => Value::Float(a - b),
                Op::Mul => Value::Float(a * b),
                Op::Div => Value::Float(a / b),
                Op::Eq => Value::Bool((a - b).abs() < f64::EPSILON),
                Op::Neq => Value::Bool((a - b).abs() >= f64::EPSILON),
                Op::Lt => Value::Bool(a < b),
                Op::Gt => Value::Bool(a > b),
                Op::Le => Value::Bool(a <= b),
                Op::Ge => Value::Bool(a >= b),
                _ => Value::Nil,
            },
            (Value::Int(a), Value::Float(b)) => match op {
                Op::Add => Value::Float(*a as f64 + b),
                Op::Sub => Value::Float(*a as f64 - b),
                Op::Mul => Value::Float(*a as f64 * b),
                Op::Div => Value::Float(*a as f64 / b),
                _ => Value::Nil,
            },
            (Value::Float(a), Value::Int(b)) => match op {
                Op::Add => Value::Float(a + *b as f64),
                Op::Sub => Value::Float(a - *b as f64),
                Op::Mul => Value::Float(a * *b as f64),
                Op::Div => Value::Float(a / *b as f64),
                _ => Value::Nil,
            },
            (Value::Bool(a), Value::Bool(b)) => match op {
                Op::Eq => Value::Bool(a == b),
                Op::Neq => Value::Bool(a != b),
                Op::And => Value::Bool(*a && *b),
                Op::Or => Value::Bool(*a || *b),
                _ => Value::Nil,
            },
            (Value::Str(a), Value::Str(b)) => match op {
                Op::Eq => Value::Bool(a.as_str() == b.as_str()),
                Op::Neq => Value::Bool(a.as_str() != b.as_str()),
                _ => Value::Nil,
            },
            _ => Value::Nil,
        }
    }

    /// Apply a compound assignment operator (+=, -=, etc.).
    fn apply_compound_op(&self, current: &Value, op: &Op, rhs: &Value) -> Value {
        let synthetic_op = match op {
            Op::AddEq => &Op::Add,
            Op::SubEq => &Op::Sub,
            Op::MulEq => &Op::Mul,
            Op::DivEq => &Op::Div,
            _ => return Value::Nil,
        };
        self.apply_binop(current, synthetic_op, rhs)
    }
}

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
// Handler Bytecode Compilation
// ============================================================================

/// Compile handler AST statements into VM bytecode.
///
/// Produces bytecode that reads/writes widget state via GET_GENERIC_FIELD /
/// SET_GENERIC_FIELD opcodes on the state heap object.
///
/// Returns (bytecode, string_constants) where string_constants need to be
/// added to the VM's string pool before execution.
///
/// `string_base_idx` is the starting index in the VM string pool for this handler's strings.
fn compile_handler_stmts(
    stmts: &[crate::ast::Stmt],
    state_obj_id: u64,
    state_field_names: &[String],
    string_base_idx: usize,
) -> std::result::Result<(Vec<u8>, Vec<String>), String> {
    let mut ctx = CompileContext {
        code: Vec::new(),
        strings: Vec::new(),
        state_obj_id,
        state_field_names,
        string_base_idx,
    };
    for stmt in stmts {
        compile_stmt(&mut ctx, stmt)?;
    }
    // Emit RET 0 (void return)
    ctx.code.push(OpCode::RET as u8);
    ctx.code.push(0);
    Ok((ctx.code, ctx.strings))
}

/// Compilation context tracking bytecode and string constants.
struct CompileContext<'a> {
    code: Vec<u8>,
    strings: Vec<String>,
    state_obj_id: u64,
    state_field_names: &'a [String],
    string_base_idx: usize,
}

impl<'a> CompileContext<'a> {
    /// Register a string constant, return its global pool index.
    fn add_string(&mut self, s: &str) -> usize {
        let local_idx = self.strings.len();
        self.strings.push(s.to_string());
        self.string_base_idx + local_idx
    }
}

fn compile_stmt<'a>(
    ctx: &mut CompileContext<'a>,
    stmt: &crate::ast::Stmt,
) -> std::result::Result<(), String> {
    match stmt {
        crate::ast::Stmt::Expr(expr) => {
            compile_expr_stmt(ctx, expr)
        }
        crate::ast::Stmt::Store(store) => {
            let field_idx = ctx.state_field_names.iter().position(|n| n == &store.name)
                .ok_or_else(|| format!("field '{}' not found", store.name))?;
            compile_expr(ctx, &store.expr)?;
            emit_set_field(&mut ctx.code, ctx.state_obj_id, field_idx);
            Ok(())
        }
        crate::ast::Stmt::If(if_stmt) => {
            // Compile: cond → JMP_IF_Z to else_end → then_body → JMP past else → else_body
            if let Some(branch) = if_stmt.branches.first() {
                compile_expr(ctx, &branch.cond)?;
                // JMP_IF_Z with placeholder offset (2 bytes)
                ctx.code.push(OpCode::JMP_IF_Z as u8);
                let jmp_else_pos = ctx.code.len();
                ctx.code.push(0); // placeholder low byte
                ctx.code.push(0); // placeholder high byte
                // Compile then-body
                for s in &branch.body.stmts {
                    compile_stmt(ctx, s)?;
                }
                if let Some(else_body) = &if_stmt.else_ {
                    // JMP over else body
                    ctx.code.push(OpCode::JMP as u8);
                    let jmp_end_pos = ctx.code.len();
                    ctx.code.push(0); // placeholder
                    ctx.code.push(0); // placeholder
                    // Patch JMP_IF_Z to jump here (start of else)
                    let else_start = ctx.code.len();
                    let jmp_else_offset = (else_start as isize) - (jmp_else_pos as isize + 2) as isize;
                    ctx.code[jmp_else_pos] = (jmp_else_offset & 0xFF) as u8;
                    ctx.code[jmp_else_pos + 1] = ((jmp_else_offset >> 8) & 0xFF) as u8;
                    // Compile else-body
                    for s in &else_body.stmts {
                        compile_stmt(ctx, s)?;
                    }
                    // Patch JMP to jump here (after else)
                    let after_else = ctx.code.len();
                    let jmp_end_offset = (after_else as isize) - (jmp_end_pos as isize + 2) as isize;
                    ctx.code[jmp_end_pos] = (jmp_end_offset & 0xFF) as u8;
                    ctx.code[jmp_end_pos + 1] = ((jmp_end_offset >> 8) & 0xFF) as u8;
                } else {
                    // No else — patch JMP_IF_Z to jump to end
                    let after_then = ctx.code.len();
                    let offset = (after_then as isize) - (jmp_else_pos as isize + 2) as isize;
                    ctx.code[jmp_else_pos] = (offset & 0xFF) as u8;
                    ctx.code[jmp_else_pos + 1] = ((offset >> 8) & 0xFF) as u8;
                }
            }
            Ok(())
        }
        crate::ast::Stmt::EmptyLine(_) | crate::ast::Stmt::Comment(_) => {
            // Skip comments and empty lines in handler bytecode
            Ok(())
        }
        _ => Err(format!("unsupported stmt type in handler: {:?}", stmt)),
    }
}

/// Compile an expression statement (assignment or compound assignment).
fn compile_expr_stmt<'a>(
    ctx: &mut CompileContext<'a>,
    expr: &crate::ast::Expr,
) -> std::result::Result<(), String> {
    match expr {
        // Simple assignment: .field = expr
        crate::ast::Expr::Bina(lhs, op, rhs) if matches!(op, Op::Asn) => {
            let target = resolve_state_target(lhs)?;
            let field_idx = ctx.state_field_names.iter().position(|n| n == &target)
                .ok_or_else(|| format!("field '{}' not found", target))?;
            compile_expr(ctx, rhs)?;
            emit_set_field(&mut ctx.code, ctx.state_obj_id, field_idx);
            Ok(())
        }
        // Compound assignment: .field += expr, .field -= expr
        crate::ast::Expr::Bina(lhs, op, rhs)
            if matches!(op, Op::AddEq | Op::SubEq | Op::MulEq | Op::DivEq) =>
        {
            let target = resolve_state_target(lhs)?;
            let field_idx = ctx.state_field_names.iter().position(|n| n == &target)
                .ok_or_else(|| format!("field '{}' not found", target))?;
            emit_get_field(&mut ctx.code, ctx.state_obj_id, field_idx);
            compile_expr(ctx, rhs)?;
            match op {
                Op::AddEq => ctx.code.push(OpCode::ADD as u8),
                Op::SubEq => ctx.code.push(OpCode::SUB as u8),
                Op::MulEq => ctx.code.push(OpCode::MUL as u8),
                Op::DivEq => ctx.code.push(OpCode::DIV as u8),
                _ => unreachable!(),
            }
            emit_set_field(&mut ctx.code, ctx.state_obj_id, field_idx);
            Ok(())
        }
        _ => Ok(()), // Ignore other expression statements
    }
}

/// Compile an expression, pushing its result onto the VM stack.
fn compile_expr<'a>(
    ctx: &mut CompileContext<'a>,
    expr: &crate::ast::Expr,
) -> std::result::Result<(), String> {
    match expr {
        crate::ast::Expr::Int(n) => {
            emit_const_i32(&mut ctx.code, *n);
        }
        crate::ast::Expr::Bool(b) => {
            emit_const_i32(&mut ctx.code, if *b { 1 } else { 0 });
        }
        crate::ast::Expr::Str(s) => {
            // Use LOAD_STR to push string — works in both nanbox and non-nanbox modes
            let idx = ctx.add_string(s);
            ctx.code.push(OpCode::LOAD_STR as u8);
            ctx.code.extend_from_slice(&(idx as u16).to_le_bytes());
        }
        crate::ast::Expr::Float(f, _) | crate::ast::Expr::Double(f, _) => {
            // Push as f64 via CONST_F64
            ctx.code.push(OpCode::CONST_F64 as u8);
            ctx.code.extend_from_slice(&f.to_le_bytes());
        }
        crate::ast::Expr::Ident(name) => {
            let field_idx = ctx.state_field_names.iter().position(|n| n == name.as_ref())
                .ok_or_else(|| format!("field '{}' not found", name))?;
            emit_get_field(&mut ctx.code, ctx.state_obj_id, field_idx);
        }
        crate::ast::Expr::Dot(obj, field) => {
            if is_self_ref(obj) {
                let field_idx = ctx.state_field_names.iter().position(|n| n == field.as_ref())
                    .ok_or_else(|| format!("field '{}' not found", field))?;
                emit_get_field(&mut ctx.code, ctx.state_obj_id, field_idx);
            } else {
                return Err(format!("nested dot access not supported in handlers"));
            }
        }
        crate::ast::Expr::Bina(lhs, op, rhs) => {
            // For string concatenation with +, use STR_CAT
            if matches!(op, Op::Add) && is_likely_string_expr(lhs) {
                compile_expr(ctx, lhs)?;
                compile_expr(ctx, rhs)?;
                ctx.code.push(OpCode::STR_CAT as u8);
            } else {
                compile_expr(ctx, lhs)?;
                compile_expr(ctx, rhs)?;
                match op {
                    Op::Add => ctx.code.push(OpCode::ADD as u8),
                    Op::Sub => ctx.code.push(OpCode::SUB as u8),
                    Op::Mul => ctx.code.push(OpCode::MUL as u8),
                    Op::Div => ctx.code.push(OpCode::DIV as u8),
                    Op::Mod => ctx.code.push(OpCode::MOD as u8),
                    Op::Eq => ctx.code.push(OpCode::EQ as u8),
                    Op::Neq => ctx.code.push(OpCode::NE as u8),
                    Op::Lt => ctx.code.push(OpCode::LT as u8),
                    Op::Gt => ctx.code.push(OpCode::GT as u8),
                    Op::Le => ctx.code.push(OpCode::LE as u8),
                    Op::Ge => ctx.code.push(OpCode::GE as u8),
                    _ => return Err(format!("binary op {:?} not supported in handlers", op)),
                }
            }
        }
        crate::ast::Expr::Unary(op, operand) => {
            compile_expr(ctx, operand)?;
            match op {
                Op::Not => ctx.code.push(OpCode::NOT as u8),
                Op::Sub => ctx.code.push(OpCode::NEG as u8),
                _ => return Err(format!("unary op {:?} not supported in handlers", op)),
            }
        }
        _ => return Err(format!("expr type not supported in handler: {:?}", expr)),
    }
    Ok(())
}

/// Check if an expression likely produces a string value.
/// Used to decide between ADD (arithmetic) and STR_CAT (string concatenation).
fn is_likely_string_expr(expr: &crate::ast::Expr) -> bool {
    match expr {
        crate::ast::Expr::Str(_) => true,
        crate::ast::Expr::Dot(_, name) => {
            let n = name.as_ref();
            n.contains("display") || n.contains("operator") || n.contains("input")
                || n.contains("label") || n.contains("text") || n.contains("name")
                || n.contains("str") || n.contains("msg")
        }
        crate::ast::Expr::Ident(name) => {
            let n = name.as_ref();
            n.contains("display") || n.contains("operator") || n.contains("input")
                || n.contains("label") || n.contains("text") || n.contains("name")
                || n.contains("str") || n.contains("msg")
        }
        crate::ast::Expr::Bina(_, op, _) if matches!(op, Op::Add) => {
            // Nested concatenation — check left side
            false
        }
        _ => false,
    }
}

fn resolve_state_target(expr: &crate::ast::Expr) -> std::result::Result<String, String> {
    match expr {
        crate::ast::Expr::Ident(name) => Ok(name.as_ref().to_string()),
        crate::ast::Expr::Dot(obj, field) if is_self_ref(obj) => {
            Ok(field.as_ref().to_string())
        }
        _ => Err(format!("cannot resolve assignment target: {:?}", expr)),
    }
}

fn is_self_ref(expr: &crate::ast::Expr) -> bool {
    matches!(expr, crate::ast::Expr::Ident(name) if name.as_ref() == "." || name.as_ref() == "self")
}

fn emit_const_i32(code: &mut Vec<u8>, val: i32) {
    code.push(OpCode::CONST_I32 as u8);
    code.extend_from_slice(&val.to_le_bytes());
}

fn emit_get_field(code: &mut Vec<u8>, state_obj_id: u64, field_idx: usize) {
    emit_const_i32(code, state_obj_id as i32);
    code.push(OpCode::GET_GENERIC_FIELD as u8);
    code.extend_from_slice(&(field_idx as u32).to_le_bytes());
}

fn emit_set_field(code: &mut Vec<u8>, state_obj_id: u64, field_idx: usize) {
    emit_const_i32(code, state_obj_id as i32);
    code.push(OpCode::SET_GENERIC_FIELD as u8);
    code.extend_from_slice(&(field_idx as u32).to_le_bytes());
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

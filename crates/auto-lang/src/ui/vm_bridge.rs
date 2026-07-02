//! # VmBridge - Bridge between AutoVM and the UI system
//!
//! This module provides [`VmBridge`], which links widget handlers compiled by the
//! genuine VM `Codegen` (the same compiler the non-UI `run()` path uses) to the
//! UI rendering backend.
//!
//! ## Architecture (Plan 323 / Option B)
//!
//! ```text
//! AuraWidget (extracted from .at source)
//!    |
//! |  handler_codegen::synthesize_widget_module
//! |  (imports + type AppState + handler fns -> ONE Codegen pass -> Module)
//!    v
//! VmBridge
//!  - Links the module into a VirtualFlash
//!  - Stores widget state as GenericInstanceData on the VM heap
//!  - Dispatches handlers via call_fn_by_name
//!    |
//!    v
//! UI Backend (iced, GPUI, headless) reads state via read_state()
//! and triggers handlers via call_handler()
//! ```
//!
//! Each handler is a real VM function `fn handler_<Name>(__state AppState, ...)`.
//! State references (`.field`) are AST-rewritten to `__state.field` by
//! `handler_codegen`, which Codegen lowers to `LOAD_LOCAL + GET_FIELD/SET_FIELD`
//! against the state heap object. `call_handler` pushes the state heap id as the
//! first argument and dispatches via `call_fn_by_name`.
//!
//! This replaces the bespoke mini-compiler + AST tree-walker that stalled during
//! the Plan 205 migration: handlers can now use the full language (loops, arrays,
//! objects, cross-module `CALL` like `build_month_grid`).

use std::collections::HashMap;

use crate::ast::Stmt;
use crate::vm::engine::AutoVM;
use crate::vm::generic_registry::GenericInstanceData;
use crate::vm::loader::Linker;
use crate::vm::task::AutoTask;
use crate::vm::virt_memory::VirtualFlash;
use crate::aura::{AuraExpr, AuraWidget, AuraStateDef, AuraNode, AuraUnaryOp};
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
/// Holds a VM instance with widget state and handler bytecode.
/// Each widget gets its own VmBridge with an isolated VM.
///
/// # Lifecycle
///
/// 1. `VmBridge::new(widget)` - Create bridge, synthesize handlers, init state
/// 2. `bridge.read_state("count")` - Read state field values for rendering
/// 3. `bridge.call_handler("Inc", &[])` - Execute handler on user interaction
/// 4. `bridge.read_state(...)` - Read updated state for re-rendering
pub struct VmBridge {
    /// AutoVM instance (owned, isolated per widget)
    vm: AutoVM,

    /// Widget state as a VM heap object ID.
    /// The heap object is a `GenericInstanceData` with field names and values.
    state_obj_id: u64,

    /// State field names (ordered, matching GenericInstanceData field order)
    state_field_names: Vec<String>,

    /// Widget name for debugging
    widget_name: String,

    /// Plan 337: child widget state heap object IDs, keyed by widget name.
    /// Each child widget instance gets its own GenericInstanceData on the same
    /// VM heap (single VM, multiple state objects). Uses RefCell for interior
    /// mutability so AuraViewBuilder (which holds &VmBridge) can create/update
    /// child states during rendering.
    child_state_map: std::cell::RefCell<std::collections::HashMap<String, u64>>,
}

impl VmBridge {
    /// Create a new VmBridge for a given AuraWidget, with no imported symbols.
    ///
    /// Delegates to [`VmBridge::new_with_imports`] with an empty import list.
    /// Use `new_with_imports` when the widget's `use`-imported functions/types
    /// must be available to its handlers (e.g. `build_month_grid`).
    pub fn new(widget: &AuraWidget) -> Result<Self> {
        Self::new_with_imports(widget, Vec::new())
    }

    /// Create a new VmBridge, compiling imports + state type + handlers in one
    /// `Codegen` pass so cross-references resolve within a single module.
    ///
    /// # Arguments
    ///
    /// * `widget` - The AuraWidget to create a bridge for
    /// * `import_stmts` - `Stmt::Fn` / `Stmt::TypeDecl` / `Stmt::EnumDecl` /
    ///   `Stmt::Ext` collected from the widget's `use`-imported modules (the
    ///   helpers it calls, like `build_month_grid`)
    ///
    /// # Errors
    ///
    /// Returns an error if handler synthesis or VM initialization fails.
    pub fn new_with_imports(widget: &AuraWidget, import_stmts: Vec<Stmt>) -> Result<Self> {
        let empty = std::collections::HashMap::new();
        Self::new_with_children(widget, &[], import_stmts, &empty, false)
    }

    /// Plan 337: create a VmBridge compiling root widget + child widgets into
    /// ONE VM module (single VM widget tree). Child handlers get namespaced fn
    /// names (handler_<Widget>_<Event>) so they coexist in one module.
    pub fn new_with_children(
        widget: &AuraWidget,
        child_widgets: &[crate::aura::AuraWidget],
        import_stmts: Vec<Stmt>,
        import_aliases: &std::collections::HashMap<String, String>,
        api_over_http: bool,
    ) -> Result<Self> {
        // Ensure BIGVM_NATIVES is populated before AutoVM::new()
        crate::vm::native_registry::register_builtin_natives();

        let widget_name = widget.name.clone();

        // 1. Synthesize imports + ALL widgets' state types + handlers into ONE
        //    Module via the genuine VM Codegen. Plan 337: single VM.
        let (module, registry) = crate::ui::handler_codegen::synthesize_widget_module(widget, child_widgets, import_stmts, import_aliases, api_over_http)
            .map_err(|e| VmBridgeError::InvalidState(format!(
                "handler synthesis failed for '{}': {}", widget_name, e
            )))?;

        // Metadata we still need after handing the module to the linker.
        let object_keys = module.object_keys.clone();
        let object_types = module.object_types.clone();
        let strings = module.strings.clone();

        // 2. Link (single module → no cross-module relocation).
        let mut linker = Linker::new();
        linker.add_module(module);
        let (code, exports) = linker.link().map_err(|e| VmBridgeError::InvalidState(
            format!("link failed for '{}': {}", widget_name, e)
        ))?;

        // 3. Build flash + VM with unified metadata tables.
        let flash = VirtualFlash::from_vec_with_metadata(code, exports, object_keys, object_types);
        let mut vm = AutoVM::new(flash, 4096);
        // Plan 336: load the codegen's generic_registry so CONSTRUCT_INSTANCE can
        // resolve struct field names (Note.title). Without this, field_names fall
        // back to "_unknown" and struct field access in handler bodies / for-loop
        // bindings fails.
        vm.load_generic_registry(registry);
        vm.load_strings(strings);

        // 4. Build state fields and default values, then create the state
        //    GenericInstanceData on the VM heap. This is the same object the
        //    handler's `__state.field` GET_FIELD/SET_FIELD opcodes touch.
        let mut field_names = Vec::with_capacity(widget.state_vars.len());
        let mut field_values = Vec::with_capacity(widget.state_vars.len());
        for state_var in &widget.state_vars {
            field_names.push(state_var.name.clone());
            field_values.push(eval_aura_expr_to_value(&state_var.initial));
        }

        let mono_name = format!("{}_State", widget_name);
        let instance = GenericInstanceData::new_with_names(
            mono_name,
            field_values,
            field_names.clone(),
        );
        let state_obj_id = vm.insert_heap_object(instance);

        Ok(Self {
            vm,
            state_obj_id,
            state_field_names: field_names,
            widget_name,
            child_state_map: std::cell::RefCell::new(std::collections::HashMap::new()),
        })
    }

    /// Create a new VmBridge with a pre-configured AutoVM instance (legacy path).
    ///
    /// Plan 323: the VM is always rebuilt from the synthesized module, so the
    /// passed `vm` is intentionally ignored. This entry point is retained for
    /// API compatibility; prefer [`VmBridge::new`] / [`VmBridge::new_with_imports`].
    pub fn new_with_vm(_vm: AutoVM, widget: &AuraWidget) -> Result<Self> {
        Self::new_with_imports(widget, Vec::new())
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

    /// Read a state field that holds an array_id and return the actual Vec<Value> from arrays DashMap.
    ///
    /// When the state field is `Value::Array`, returns its inner Vec directly.
    /// When it's `Value::Int(id)` (array_id from `[...]` literal), reads from vm.arrays.
    /// Plan 335: `Value::VmRef` (heap id 4000000+ from `List<T>.new` for struct
    /// element lists) is dereferenced: try heap_objects (ListData<Value>/ListData<i32>)
    /// first, then vm.arrays. This unblocks 015-notes vm rendering where
    /// `list_notes()` → `notes.to_array()` returns a VmRef to the struct list.
    pub fn read_state_as_vec(&self, field_name: &str) -> Result<Vec<Value>> {
        let val = self.read_state(field_name)?;
        match val {
            Value::Array(arr) => Ok(arr.values),
            Value::Int(id) if id >= 2000000 => {
                let arr_id = id as u64;
                self.vm.arrays.get(&arr_id)
                    .map(|r| r.read().unwrap().clone())
                    .ok_or_else(|| VmBridgeError::InvalidState(
                        format!("array_id {} not found in arrays DashMap", arr_id)
                    ))
            }
            Value::VmRef(r) => self.vmref_to_vec(r.id),
            other => Err(VmBridgeError::InvalidState(
                format!("Expected array for field '{}', got {:?}", field_name, other)
            )),
        }
    }

    /// Plan 335: dereference a `VmRef` holding a list into `Vec<Value>`. Tries
    /// heap_objects first (ListData<Value> from `List<T>.new` of structs, or
    /// ListData<i32>), then vm.arrays. Order avoids depending on id-range
    /// conventions (4000000 heap / 2000000 arrays) so it stays correct if the
    /// generators' start values change.
    /// Plan 336: index into a list value held as a `VmRef` (heap id) or array_id,
    /// returning the element at `i`. Used by the view builder's Index expr
    /// (e.g. `.notes[.active_id]`) to dereference a List<Note> element.
    pub fn index_list(&self, id: usize, i: i32) -> Option<Value> {
        if let Ok(elems) = self.vmref_to_vec(id) {
            let idx = i as usize;
            if idx < elems.len() {
                return Some(elems[idx].clone());
            }
        }
        None
    }

    fn vmref_to_vec(&self, id: usize) -> Result<Vec<Value>> {        // Path 1: heap_objects (4000000+) — ListData<Value> or ListData<i32>.
        if let Some(obj) = self.vm.get_heap_object(id as u64) {
            let guard = obj.read().unwrap();
            use crate::vm::types::ListData;
            if let Some(list) = guard.as_any().downcast_ref::<ListData<Value>>() {
                return Ok(list.elems.clone());
            }
            if let Some(list) = guard.as_any().downcast_ref::<ListData<i32>>() {
                return Ok(list.elems.iter().map(|i| Value::Int(*i)).collect());
            }
        }
        // Path 2: vm.arrays (2000000+) — Vec<Value>.
        if let Some(arr_ref) = self.vm.arrays.get(&(id as u64)) {
            return Ok(arr_ref.read().unwrap().clone());
        }
        Err(VmBridgeError::InvalidState(format!(
            "VmRef {} is not a readable list (not in heap_objects as ListData nor in arrays)", id
        )))
    }

    /// Write a Vec<Value> back to a state field that holds an array reference.
    ///
    /// When the state field is `Value::Array`, writes back as Value::Array.
    /// When it's `Value::Int(id)` (array_id from `[...]` literal), writes to vm.arrays directly.
    pub fn write_state_vec(&mut self, field_name: &str, values: Vec<Value>) -> Result<()> {
        let val = self.read_state(field_name)?;
        match val {
            Value::Array(_) => {
                self.write_state(field_name, Value::Array(auto_val::Array { values }))
            }
            Value::Int(id) if id >= 2000000 => {
                let arr_id = id as u64;
                if let Some(arr_ref) = self.vm.arrays.get(&arr_id) {
                    *arr_ref.write().unwrap() = values;
                    Ok(())
                } else {
                    Err(VmBridgeError::InvalidState(
                        format!("array_id {} not found in arrays DashMap", arr_id)
                    ))
                }
            }
            other => Err(VmBridgeError::InvalidState(
                format!("Expected array for field '{}', got {:?}", field_name, other)
            )),
        }
    }

    /// Materialize a heap object reference into an inline `Value::Obj` so the
    /// view builder's `Value::Obj`-based field resolvers can read its fields.
    ///
    /// An Auto Obj literal (`{ label: ..., ... }`) is compiled to `CREATE_OBJ`,
    /// which stores an `ObjectData` in the VM `objects` registry and leaves its
    /// id on the stack as a plain `Value::Int`. When such a value is iterated as
    /// a loop item (e.g. `for cell in .days`), the binding is a bare
    /// `Value::Int(obj_id)` and `cell.label` cannot resolve — the view builder's
    /// resolvers only handle `Value::Obj`. This derefs the id (via `vm.objects`,
    /// keyed by `object_id_gen` starting at 1_000_000) into a `Value::Obj` whose
    /// string-keyed fields mirror the `ObjectData`. Non-object values (real ints,
    /// already-inline `Value::Obj`, etc.) pass through unchanged.
    pub fn materialize_obj_ref(&self, v: &Value) -> Value {
        match v {
            Value::Int(id) => {
                // 1M segment: CREATE_OBJ → ObjectData in vm.objects
                if let Some(arc) = self.vm.objects.get(&(*id as u64)) {
                    let obj = arc.read().unwrap();
                    let mut out = auto_val::Obj::new();
                    for (key, val) in obj.fields.iter() {
                        if let auto_val::ValueKey::Str(s) = key {
                            out.set(s.clone(), val.clone());
                        }
                    }
                    return Value::Obj(out);
                }
                // Plan 336: 4M segment: CONSTRUCT_INSTANCE → GenericInstanceData in
                // heap_objects. List<Note>.new([Note{...}]) stores Note instances
                // as bare Int(heap_id) elements; without this arm, note.title in a
                // for-loop body can't resolve.
                if *id >= 4_000_000 {
                    if let Some(obj) = self.vm.get_heap_object(*id as u64) {
                        let guard = obj.read().unwrap();
                        if let Some(inst) = guard.as_any().downcast_ref::<crate::vm::generic_registry::GenericInstanceData>() {
                            let mut out = auto_val::Obj::new();
                            for (val, name) in inst.fields.iter().zip(inst.field_names.iter()) {
                                if name != "_unknown" {
                                    out.set(name.clone(), val.clone());
                                }
                            }
                            return Value::Obj(out);
                        }
                    }
                }
                v.clone()
            }
            // Plan 335: VmRef (heap id from other paths) — same GenericInstanceData deref.
            Value::VmRef(r) => {
                if let Some(obj) = self.vm.get_heap_object(r.id as u64) {
                    let guard = obj.read().unwrap();
                    if let Some(inst) = guard.as_any().downcast_ref::<crate::vm::generic_registry::GenericInstanceData>() {
                        let mut out = auto_val::Obj::new();
                        for (val, name) in inst.fields.iter().zip(inst.field_names.iter()) {
                            if name != "_unknown" {
                                out.set(name.clone(), val.clone());
                            }
                        }
                        return Value::Obj(out);
                    }
                }
                v.clone()
            }
            _ => v.clone(),
        }
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

    /// Plan 333: run the synthesized `__module_init` fn, which initializes
    /// imported module-level globals (`var notes = ...` etc.). Must be called
    /// once before `Init` (and before any handler that reads those globals).
    /// No-op (returns Ok) if the fn isn't present (no module-level stores).
    pub fn run_module_init(&mut self) -> Result<()> {
        let fn_name = crate::ui::handler_codegen::MODULE_INIT_FN;
        if !self.vm.flash.exports_by_name.contains_key(fn_name) {
            return Ok(());
        }
        let mut task = AutoTask::new(0, 4096, 0);
        self.vm
            .call_fn_by_name(&mut task, fn_name, 0)
            .map_err(|e| VmBridgeError::VmError(format!("{:?}", e)))
    }

    /// Call a handler by name with arguments.
    ///
    /// Looks up the synthesized `handler_<Name>` function in the module exports,
    /// pushes the state heap id as the first argument (`__state`) followed by the
    /// caller-supplied args, then dispatches via `call_fn_by_name`. Any state
    /// mutation the handler performs is written through to the state heap object
    /// in place, so a subsequent `read_state` reflects it.
    ///
    /// # Arguments
    ///
    /// * `event_name` - Handler name (e.g., "Inc", "Init", "PrevMonth")
    /// * `args` - Arguments to pass to the handler (after `__state`)
    ///
    /// # Errors
    ///
    /// Returns an error if the handler is not found or VM execution fails.
    /// UI should handle errors gracefully (log and continue).
    /// Get the state object heap ID (for event routing).
    pub fn state_obj_id(&self) -> u64 {
        self.state_obj_id
    }

    /// Plan 337: get a child widget's state object heap ID (if it exists).
    pub fn get_child_state_id(&self, widget_name: &str) -> Option<u64> {
        self.child_state_map.borrow().get(widget_name).copied()
    }

    /// Plan 337: ensure a child widget's state object exists on the VM heap,
    /// and update its prop fields. Returns the child state heap id.
    /// Called by render_child_widget (which no longer creates a new VM).
    /// Plan 337: write child widget's prop values directly into the ROOT state
    /// object. Since there is only ONE VM with ONE unified state, all widget
    /// fields (App's model + child props like `note`) live in the same
    /// GenericInstanceData. This returns the ROOT state_obj_id so child views
    /// also read from root state.
    pub fn ensure_child_state(
        &self,
        _widget_name: &str,
        _state_field_names: &[String],
        props: &std::collections::HashMap<String, auto_val::Value>,
    ) -> u64 {
        let root_id = self.state_obj_id;

        // Write prop values into the root state object (adding fields if missing).
        if let Some(obj) = self.vm.get_heap_object(root_id) {
            let mut guard = obj.write().unwrap();
            if let Some(inst) = guard.as_any_mut().downcast_mut::<GenericInstanceData>() {
                for (name, val) in props {
                    if let Some(idx) = inst.field_names.iter().position(|n| n == name) {
                        let _ = inst.set_field(idx, val.clone());
                    } else {
                        // Add new field (prop not yet in root state).
                        inst.field_names.push(name.clone());
                        inst.fields.push(val.clone());
                    }
                }
            }
        }

        // Return root state id — all reads/writes go to the unified state.
        root_id
    }

    /// Plan 337: read a state field from a SPECIFIC child widget's state object
    /// (by heap id), not the root widget's state.
    pub fn read_child_state(&self, child_state_id: u64, field_name: &str) -> Result<auto_val::Value> {
        use crate::vm::generic_registry::GenericInstanceData;
        let obj = self.vm.get_heap_object(child_state_id)
            .ok_or_else(|| VmBridgeError::InvalidState(
                format!("child state heap object {} not found", child_state_id)
            ))?;
        let guard = obj.read().unwrap();
        let inst = guard.as_any().downcast_ref::<GenericInstanceData>()
            .ok_or_else(|| VmBridgeError::InvalidState("not GenericInstanceData".into()))?;
        let idx = inst.field_names.iter().position(|n| n == field_name)
            .ok_or_else(|| VmBridgeError::FieldNotFound(field_name.to_string()))?;
        inst.get_field(idx)
            .cloned()
            .ok_or_else(|| VmBridgeError::FieldNotFound(field_name.to_string()))
    }

    /// Plan 337: write child widget's prop values directly into the ROOT state
    /// object. Since all handlers run against root state (single VM, unified
    /// state), props must live in the same GenericInstanceData as the parent's
    /// model fields. This avoids "Field 'note' not found" when a child handler
    /// reads .note.title. Idempotent — if the field exists, updates it.
    pub fn sync_child_props_to_root(&self, _child_state_id: u64) {
        // No-op placeholder — actual syncing happens in ensure_child_state which
        // writes directly to root state. Kept for API compat.
    }
    pub fn read_child_state_as_vec(&self, child_state_id: u64, field_name: &str) -> Result<Vec<auto_val::Value>> {
        let val = self.read_child_state(child_state_id, field_name)?;
        match val {
            auto_val::Value::Array(arr) => Ok(arr.values),
            auto_val::Value::Int(id) if id >= 2000000 => {
                self.vm.arrays.get(&(id as u64))
                    .map(|r| r.read().unwrap().clone())
                    .ok_or_else(|| VmBridgeError::InvalidState(
                        format!("array_id {} not found", id)
                    ))
            }
            auto_val::Value::VmRef(r) => self.vmref_to_vec(r.id),
            other => Err(VmBridgeError::InvalidState(
                format!("Expected array for '{}', got {:?}", field_name, other)
            )),
        }
    }

    /// Call a handler by name with arguments.
    ///
    /// Looks up the synthesized `handler_<WidgetName>_<EventName>` function in
    /// the module exports (Plan 337: single-VM namespaced handlers). Pushes the
    /// state heap id as the first argument (`__state`) followed by the
    /// caller-supplied args, then dispatches via `call_fn_by_name`.
    ///
    /// # Arguments
    ///
    /// * `widget_name` - Widget that owns the handler (e.g. "App", "EditorPanel")
    /// * `event_name` - Handler name (e.g., "Inc", "Init", "Edit")
    /// * `args` - Arguments to pass to the handler (after `__state`)
    pub fn call_handler_for(&mut self, widget_name: &str, event_name: &str, state_obj_id: u64, args: &[Value]) -> Result<()> {
        let fn_name = crate::ui::handler_codegen::namespaced_handler_fn_name(widget_name, event_name);

        // Verify the handler is exported before setting up a call frame.
        if !self.vm.flash.exports_by_name.contains_key(&fn_name) {
            return Err(VmBridgeError::HandlerNotFound(format!("{}.{}", widget_name, event_name)));
        }

        let mut task = AutoTask::new(0, 4096, 0);
        task.ram.push_i32(state_obj_id as i32);
        for a in args {
            if let Value::Str(s) = a {
                let idx = {
                    let mut strings = self.vm.strings.write().unwrap();
                    let idx = strings.len();
                    strings.push(s.as_bytes().to_vec());
                    idx
                };
                task.ram.push_str_idx(idx as u32);
            } else {
                push_value(&mut task.ram, a);
            }
        }

        self.vm
            .call_fn_by_name(&mut task, &fn_name, 1 + args.len())
            .map_err(|e| VmBridgeError::VmError(format!("{:?}", e)))
    }

    /// Legacy call_handler — calls handler on the ROOT widget (widget_name
    /// defaults to this bridge's widget name). Uses self.state_obj_id.
    pub fn call_handler(&mut self, event_name: &str, args: &[Value]) -> Result<()> {
        let fn_name = format!("handler_{}", extract_handler_name(event_name));

        // Plan 337: try namespaced first (handler_<WidgetName>_<Event>),
        // then fall back to legacy (handler_<Event>) for backward compat.
        let namespaced = crate::ui::handler_codegen::namespaced_handler_fn_name(&self.widget_name, event_name);
        let fn_name = if self.vm.flash.exports_by_name.contains_key(&namespaced) {
            namespaced
        } else if self.vm.flash.exports_by_name.contains_key(&fn_name) {
            fn_name
        } else {
            return Err(VmBridgeError::HandlerNotFound(event_name.to_string()));
        };

        let mut task = AutoTask::new(0, 4096, 0);

        // Push arguments left-to-right: __state (the state heap id) first, then
        // the handler's declared params. `call_fn_by_name` then sets up the call
        // frame; params are accessed as bp-(n_args+1) .. bp-2 (see LOAD_LOCAL).
        task.ram.push_i32(self.state_obj_id as i32);
        for a in args {
            // Strings must be interned into the VM strings pool and pushed as
            // their tagged index (the same encoding LOAD_STR / GET_FIELD use),
            // otherwise a payload like `.SelectDay(cell.date)` arrives as 0.
            if let Value::Str(s) = a {
                let idx = {
                    let mut strings = self.vm.strings.write().unwrap();
                    let idx = strings.len();
                    strings.push(s.as_bytes().to_vec());
                    idx
                };
                task.ram.push_str_idx(idx as u32);
            } else {
                push_value(&mut task.ram, a);
            }
        }

        self.vm
            .call_fn_by_name(&mut task, &fn_name, 1 + args.len())
            .map_err(|e| VmBridgeError::VmError(format!("{:?}", e)))
    }

    /// Get the widget name.
    pub fn widget_name(&self) -> &str {
        &self.widget_name
    }

    /// Get the state field names.
    pub fn state_fields(&self) -> &[String] {
        &self.state_field_names
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
    ///
    /// A handler exists iff the synthesized `handler_<Name>` function is present
    /// in the module exports.
    pub fn has_handler(&self, event_name: &str) -> bool {
        let fn_name = format!("handler_{}", extract_handler_name(event_name));
        self.vm.flash.exports_by_name.contains_key(&fn_name)
    }

    /// List all registered handler names (bare names, without the `handler_` prefix).
    pub fn handler_names(&self) -> Vec<&str> {
        let mut names: Vec<&str> = self
            .vm
            .flash
            .exports_by_name
            .keys()
            .filter_map(|k| k.strip_prefix("handler_"))
            .collect();
        names.sort();
        names
    }
}

/// Push a runtime [`Value`] onto a task's RAM stack using the appropriate
/// encoding for its type (mirrors how GET_FIELD/CREATE_OBJ push values).
fn push_value(ram: &mut crate::vm::virt_memory::VirtualRAM, value: &Value) {
    match value {
        Value::Int(i) => ram.push_i32(*i),
        Value::Uint(u) => ram.push_i32(*u as i32),
        Value::Bool(b) => ram.push_i32(if *b { 1 } else { 0 }),
        Value::Char(c) => ram.push_i32(*c as i32),
        Value::Float(f) => ram.push_f32(*f as f32),
        Value::Double(d) => ram.push_f64(*d),
        Value::Nil => ram.push_i32(0),
        // Heap-referenced / complex values are not passed as scalar args in the
        // current handler surface; push a placeholder so arg arity stays correct.
        _ => ram.push_i32(0),
    }
}

/// Convert an AuraExpr initial value to a runtime Value.
///
/// Handles the common literal types. Complex expressions default to Nil.
fn eval_aura_expr_to_value(expr: &AuraExpr) -> Value {
    match expr {
        AuraExpr::Int(i) => Value::Int(*i as i32),
        AuraExpr::Float(f) => Value::Double(*f as f64),
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
        AuraExpr::Unary { op, operand } => {
            let val = eval_aura_expr_to_value(operand);
            match op {
                AuraUnaryOp::Neg => match val {
                    Value::Int(i) => Value::Int(-i),
                    Value::Double(f) => Value::Double(-f),
                    Value::Float(f) => Value::Float(-f),
                    _ => Value::Int(0),
                },
                AuraUnaryOp::Not => match val {
                    Value::Bool(b) => Value::Bool(!b),
                    _ => Value::Bool(true),
                },
            }
        }
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

/// Convert a serde_json::Value to an auto_val::Value.
///
/// Retained for Plan 323 Task 5 (extracting HTTP shims into natives); currently
/// unused after the AST interpreter removal.
#[cfg(feature = "ui-interpreter")]
#[allow(dead_code)]
fn json_to_value(json: &serde_json::Value) -> Value {
    match json {
        serde_json::Value::Null => Value::Nil,
        serde_json::Value::Bool(b) => Value::Bool(*b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Value::Int(i as i32)
            } else if let Some(f) = n.as_f64() {
                Value::Float(f)
            } else {
                Value::Nil
            }
        }
        serde_json::Value::String(s) => Value::str(s.as_str()),
        serde_json::Value::Array(arr) => {
            let items: Vec<Value> = arr.iter().map(json_to_value).collect();
            Value::Array(auto_val::Array::from(items))
        }
        serde_json::Value::Object(map) => {
            let mut obj = auto_val::Obj::new();
            for (key, val) in map {
                obj.set(key.as_str(), json_to_value(val));
            }
            Value::Obj(obj)
        }
    }
}

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
            span_map: HashMap::new(),
            key_bindings: HashMap::new(),
            api_imports: vec![],
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
    fn test_read_state_unary_neg() {
        // This mirrors how `var editing_id int = -1` is parsed:
        // Expr::Unary(Sub, Int(1)) → AuraExpr::Unary { Neg, Int(1) }
        // eval_aura_expr_to_value should produce Value::Int(-1), NOT Value::Int(0)
        let widget = make_test_widget("Todo", vec![
            AuraStateDef {
                name: "editing_id".to_string(),
                type_info: Type::Int,
                initial: AuraExpr::Unary {
                    op: AuraUnaryOp::Neg,
                    operand: Box::new(AuraExpr::Int(1)),
                },
                decorators: vec![],
            },
        ]);

        let bridge = VmBridge::new(&widget).unwrap();
        let value = bridge.read_state("editing_id").unwrap();
        assert_eq!(value, Value::Int(-1), "expected -1, got {:?}", value);
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
    fn test_has_handler_absent() {
        // No handlers synthesized → has_handler is false for any name.
        let widget = make_test_widget("Counter", vec![]);
        let bridge = VmBridge::new(&widget).unwrap();

        assert!(!bridge.has_handler("Inc"));
        assert!(bridge.handler_names().is_empty());
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

    /// Plan 323 (Option B) end-to-end unit test: a handler synthesized as a REAL
    /// VM function and dispatched via `call_handler` actually mutates widget
    /// state through the heap object. Exercises state-ref rewrite (`.count` →
    /// `__state.count`), Codegen GET_FIELD/SET_FIELD on the state instance, and
    /// `call_fn_by_name` dispatch.
    #[test]
    fn test_handler_counter_increment() {
        use crate::ast::{Expr, Name, Stmt};
        use crate::aura::LogicPayload;
        use auto_val::Op;

        let mut widget = make_test_widget("Counter", vec![AuraStateDef {
            name: "count".to_string(),
            type_info: Type::Int,
            initial: AuraExpr::Int(0),
            decorators: vec![],
        }]);

        // `.Inc -> { .count = .count + 1 }` parses as a single Asn expression stmt.
        let inc_body = vec![Stmt::Expr(Expr::Bina(
            Box::new(Expr::Ident(Name::from("count"))),
            Op::Asn,
            Box::new(Expr::Bina(
                Box::new(Expr::Ident(Name::from("count"))),
                Op::Add,
                Box::new(Expr::Int(1)),
            )),
        ))];
        widget
            .handlers
            .insert(".Inc".to_string(), LogicPayload::AstStmts(inc_body));

        let mut bridge = VmBridge::new(&widget).unwrap();
        assert!(bridge.has_handler("Inc"));

        // Initial state is 0.
        assert_eq!(bridge.read_state("count").unwrap(), Value::Int(0));

        // Dispatch the synthesized handler three times via the real VM.
        for _ in 0..3 {
            bridge.call_handler("Inc", &[]).unwrap();
        }
        assert_eq!(bridge.read_state("count").unwrap(), Value::Int(3));
    }

    /// Plan 323 (Option B) full-pipeline proof against the REAL 016-calendar
    /// source: parse app.at + calendar_util.at → collect imported `Fn`s →
    /// `VmBridge::new_with_imports` → `call_handler("Init")` → the imported
    /// `build_month_grid` (loops, arrays, Obj literals, cross-fn CALL) executes
    /// and `.days` ends up as a 42-cell grid. This is the exact bug Option B
    /// fixes: before it, imported-module functions never entered the VM and the
    /// grid rendered empty.
    #[test]
    fn test_calendar_init_builds_42_cells() {
        use crate::ast::Stmt;
        use crate::parser::Parser;
        use crate::session::CompilerSession;

        let front = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("examples/ui/016-calendar/src/front");
        let app_src = match std::fs::read_to_string(front.join("app.at")) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("skipping calendar e2e (app.at unreadable): {}", e);
                return;
            }
        };
        let util_src = std::fs::read_to_string(front.join("calendar_util.at"))
            .expect("calendar_util.at must exist in-repo");

        // 1. Parse app.at → first AuraWidget.
        let session = CompilerSession::ui();
        let mut parser = Parser::from(app_src.as_str()).with_session(session);
        let app_ast = parser.parse().expect("app.at should parse");
        let widget = app_ast
            .stmts
            .iter()
            .find_map(|s| match s {
                crate::ast::Stmt::WidgetDecl(d) => {
                    crate::aura::extract_widget_from_decl(d).ok()
                }
                _ => None,
            })
            .expect("app.at must declare a widget");

        // 2. Parse calendar_util.at → collect all Fn/TypeDecl/EnumDecl/Ext
        //    (wildcard: build_month_grid + its pure-Auto callees).
        let util_session = CompilerSession::ui();
        let mut util_parser =
            Parser::from(util_src.as_str()).with_session(util_session);
        let util_ast = util_parser.parse().expect("calendar_util.at should parse");
        let import_stmts: Vec<Stmt> = util_ast
            .stmts
            .iter()
            .filter(|s| {
                matches!(
                    s,
                    Stmt::Fn(_) | Stmt::TypeDecl(_) | Stmt::EnumDecl(_) | Stmt::Ext(_)
                )
            })
            .cloned()
            .collect();
        assert!(
            import_stmts
                .iter()
                .any(|s| matches!(s, Stmt::Fn(f) if f.name.as_str() == "build_month_grid")),
            "build_month_grid must be among the imports"
        );

        // 3. Build the bridge + fire Init.
        let mut bridge =
            VmBridge::new_with_imports(&widget, import_stmts).expect("bridge builds");
        assert!(bridge.has_handler("Init"), "Init handler must be synthesized");

        bridge
            .call_handler("Init", &[])
            .expect("Init handler should execute");

        // 4. `.days` must now hold 42 cells (6 weeks), not be empty.
        let days = bridge
            .read_state_as_vec("days")
            .expect(".days should be an array after Init");
        assert_eq!(days.len(), 42, "build_month_grid must produce 42 cells");

        // Each cell is an Obj literal { label, date, is_other_month }.
        for (i, cell) in days.iter().enumerate() {
            match cell {
                Value::Obj(_) | Value::Int(_) => {}
                other => panic!("cell {} is not an object: {:?}", i, other),
            }
        }

        // Cells are heap ObjectData refs (Value::Int(obj_id)); the view builder
        // materializes them via materialize_obj_ref. Verify that deref yields a
        // Value::Obj whose "label" field is a non-empty string, so `cell.label`
        // renders the day number instead of an empty cell.
        let first = bridge.materialize_obj_ref(&days[0]);
        match first {
            Value::Obj(obj) => {
                let label = obj.get_str("label").expect("cell.label must resolve");
                assert!(!label.is_empty(), "cell.label should be a day number");
                // The cell carries a computed `style` (day_style output); confirm
                // it materialized as a non-empty string so `class: cell.style`
                // applies the per-cell highlight.
                let style = obj.get_str("style").expect("cell.style must resolve");
                assert!(!style.is_empty(), "cell.style should be a Tailwind class string");
            }
            other => panic!(
                "materialize_obj_ref(cell) should yield Value::Obj, got {:?}",
                other
            ),
        }
    }

    /// Plan 327 SelectDay: a handler with a DECLARED param (`.SetX(n) ->`)
    /// receives the value dispatched via call_handler's args — both int and
    /// string payloads. This is the contract `onclick: .SelectDay(cell.date)`
    /// depends on (view resolves cell.date → args; on() forwards args).
    #[test]
    fn test_handler_receives_declared_param() {
        use crate::ast::{Expr, Name, Stmt};
        use crate::aura::LogicPayload;
        use auto_val::Op;

        // widget with an int field and a str field
        let mut widget = make_test_widget("App", vec![
            AuraStateDef {
                name: "count".to_string(),
                type_info: Type::Int,
                initial: AuraExpr::Int(0),
                decorators: vec![],
            },
            AuraStateDef {
                name: "label".to_string(),
                type_info: Type::StrOwned,
                initial: AuraExpr::Literal("".to_string()),
                decorators: vec![],
            },
        ]);

        // `.SetCount(n) -> { .count = n }` — `n` is a handler param (not a state
        // field), so the rewriter must leave it alone and Codegen resolves it as
        // the declared param.
        widget.handlers.insert(
            ".SetCount".to_string(),
            LogicPayload::AstStmts(vec![Stmt::Expr(Expr::Bina(
                Box::new(Expr::Ident(Name::from("count"))),
                Op::Asn,
                Box::new(Expr::Ident(Name::from("n"))),
            ))]),
        );
        widget
            .handler_params
            .insert(".SetCount".to_string(), vec!["n".to_string()]);

        // `.SetLabel(s) -> { .label = s }`
        widget.handlers.insert(
            ".SetLabel".to_string(),
            LogicPayload::AstStmts(vec![Stmt::Expr(Expr::Bina(
                Box::new(Expr::Ident(Name::from("label"))),
                Op::Asn,
                Box::new(Expr::Ident(Name::from("s"))),
            ))]),
        );
        widget
            .handler_params
            .insert(".SetLabel".to_string(), vec!["s".to_string()]);

        let mut bridge = VmBridge::new(&widget).unwrap();
        assert!(bridge.has_handler("SetCount"));
        assert!(bridge.has_handler("SetLabel"));

        // Int payload: n = 42 → count = 42
        bridge
            .call_handler("SetCount", &[Value::Int(42)])
            .expect("SetCount runs");
        assert_eq!(bridge.read_state("count").unwrap(), Value::Int(42));

        // String payload: s = "2026-06-17" → label = "2026-06-17"
        bridge
            .call_handler("SetLabel", &[Value::str("2026-06-17")])
            .expect("SetLabel runs");
        match bridge.read_state("label").unwrap() {
            Value::Str(s) => assert_eq!(s.as_str(), "2026-06-17"),
            other => panic!("label should be the dispatched string, got {:?}", other),
        }
    }

    /// Plan 323 (Option B) regression smoke against the REAL 002-counter
    /// source: parse → extract widget → VmBridge → dispatch Inc/Dec/Reset via
    /// the real VM → read `.count`. Confirms the canonical handler-mutation
    /// example works end-to-end through genuine Codegen/AutoVM dispatch.
    #[test]
    fn test_counter_002_handlers_mutate_state() {
        use crate::parser::Parser;
        use crate::session::CompilerSession;

        let app_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("examples/ui/002-counter/src/front/app.at");
        let app_src = match std::fs::read_to_string(&app_path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("skipping 002-counter smoke (app.at unreadable): {}", e);
                return;
            }
        };

        let session = CompilerSession::ui();
        let mut parser = Parser::from(app_src.as_str()).with_session(session);
        let ast = parser.parse().expect("002-counter app.at should parse");
        let widget = ast
            .stmts
            .iter()
            .find_map(|s| match s {
                crate::ast::Stmt::WidgetDecl(d) => {
                    crate::aura::extract_widget_from_decl(d).ok()
                }
                _ => None,
            })
            .expect("002-counter must declare a widget");

        let mut bridge = VmBridge::new(&widget).expect("bridge builds");
        assert!(bridge.has_handler("Inc"));
        assert!(bridge.has_handler("Dec"));
        assert!(bridge.has_handler("Reset"));

        assert_eq!(bridge.read_state("count").unwrap(), Value::Int(0));

        bridge.call_handler("Inc", &[]).unwrap();
        bridge.call_handler("Inc", &[]).unwrap();
        bridge.call_handler("Dec", &[]).unwrap();
        assert_eq!(bridge.read_state("count").unwrap(), Value::Int(1));

        bridge.call_handler("Reset", &[]).unwrap();
        assert_eq!(bridge.read_state("count").unwrap(), Value::Int(0));
    }

    /// Plan 327 (recursive import loading): drive import collection from the
    /// REAL 016-calendar `use` clause — `use calendar_util: build_month_grid,
    /// month_name, add_months_year, add_months_month` — which does NOT name
    /// `weekday_of`/`days_in_month`/`format_date`/`is_leap`, yet `build_month_grid`
    /// calls them. The recursive collector must pull the whole module (intra-
    /// module callees) so Init links. This is the exact bug that made 016-calendar
    /// fail to start in VM render mode ("Undefined symbol: weekday_of").
    #[test]
    fn test_calendar_imports_resolve_intra_module_callees() {
        use crate::parser::Parser;
        use crate::session::CompilerSession;
        use crate::use_scanner::scan_use_statements;

        let front = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("examples/ui/016-calendar/src/front");
        let app_src = match std::fs::read_to_string(front.join("app.at")) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("skipping (app.at unreadable): {}", e);
                return;
            }
        };

        // Resolve the calendar_util module exactly as run_file_dynamic_ui does.
        let calendar_util_path = crate::resolve_module_path(&front, "calendar_util")
            .expect("calendar_util.at must resolve");

        // Recursive collection from calendar_util.at (mirrors production).
        let mut visited = std::collections::HashSet::new();
        let mut seen = std::collections::HashSet::new();
        let mut import_stmts: Vec<crate::ast::Stmt> = Vec::new();
        let mut session = crate::compile::CompileSession::new();
        crate::collect_module_imports(
            &calendar_util_path,
            &mut visited,
            &mut import_stmts,
            &mut seen,
            &mut session,
            None, // PR-6: no scenario override in test
        );

        // The non-imported callees MUST be present (the bug was their absence).
        let names: std::collections::HashSet<String> = import_stmts
            .iter()
            .filter_map(|s| crate::stmt_symbol_name(s))
            .collect();
        for required in ["build_month_grid", "weekday_of", "days_in_month", "format_date", "is_leap"] {
            assert!(
                names.contains(required),
                "recursive import collection must include `{}` (callee of build_month_grid not named in the use clause)",
                required
            );
        }

        // And driving Init from app.at's widget with these imports yields 42 cells.
        let session = CompilerSession::ui();
        let mut parser = Parser::from(app_src.as_str()).with_session(session);
        let app_ast = parser.parse().expect("app.at should parse");
        let widget = app_ast
            .stmts
            .iter()
            .find_map(|s| match s {
                crate::ast::Stmt::WidgetDecl(d) => {
                    crate::aura::extract_widget_from_decl(d).ok()
                }
                _ => None,
            })
            .expect("app.at must declare a widget");

        let mut bridge = VmBridge::new_with_imports(&widget, import_stmts).expect("bridge builds");
        bridge.call_handler("Init", &[]).expect("Init runs");
        let days = bridge
            .read_state_as_vec("days")
            .expect(".days is an array after Init");
        assert_eq!(days.len(), 42);

        // Silence unused warning for scan_use_statements when the body returns early.
        let _ = scan_use_statements;
    }
}

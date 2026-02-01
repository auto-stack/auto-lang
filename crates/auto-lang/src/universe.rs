use super::scope::*;
use crate::ast::FnKind;
use crate::ast::{self, SpecDecl, Type};
use crate::atom::Atom;
use crate::libs;
use crate::vm::collections::{HashMapData, HashSetData};
use crate::vm::builder::StringBuilderData;
use auto_val::{
    shared, AccessError, AccessPath, Args, AutoStr, ExtFn, Obj, PathComponent, Sig, TypeInfoStore,
    Value, ValueData, ValueID,
};
use std::any::Any; // Still needed for env_vals
use std::cell::RefCell;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::rc::Rc;
use std::rc::Weak;

/// Location information for a symbol definition
#[derive(Debug, Clone)]
pub struct SymbolLocation {
    pub line: usize,
    pub character: usize,
    pub pos: usize,
}

impl SymbolLocation {
    pub fn new(line: usize, character: usize, pos: usize) -> Self {
        Self {
            line,
            character,
            pos,
        }
    }
}

/// Enum-based storage for VM references, avoiding TypeId/downcasting issues
#[derive(Debug)]
pub enum VmRefData {
    HashMap(HashMapData),
    HashSet(HashSetData),
    StringBuilder(StringBuilderData),
    File(BufReader<File>),
    List(ListData),
}

/// Data for dynamic lists (similar to Rust's Vec<T>)
#[derive(Debug)]
pub struct ListData {
    pub elems: Vec<Value>,
}

impl ListData {
    pub fn new() -> Self {
        Self {
            elems: Vec::new(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            elems: Vec::with_capacity(capacity),
        }
    }

    pub fn len(&self) -> usize {
        self.elems.len()
    }

    pub fn is_empty(&self) -> bool {
        self.elems.is_empty()
    }

    pub fn push(&mut self, elem: Value) {
        self.elems.push(elem);
    }

    pub fn pop(&mut self) -> Option<Value> {
        self.elems.pop()
    }

    pub fn clear(&mut self) {
        self.elems.clear();
    }

    pub fn reserve(&mut self, additional: usize) {
        self.elems.reserve(additional);
    }

    pub fn get(&self, index: usize) -> Option<&Value> {
        self.elems.get(index)
    }

    pub fn set(&mut self, index: usize, elem: Value) -> bool {
        if index < self.elems.len() {
            self.elems[index] = elem;
            true
        } else {
            false
        }
    }

    pub fn insert(&mut self, index: usize, elem: Value) {
        if index <= self.elems.len() {
            self.elems.insert(index, elem);
        }
    }

    pub fn remove(&mut self, index: usize) -> Option<Value> {
        if index < self.elems.len() {
            Some(self.elems.remove(index))
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
pub struct CodePak {
    pub sid: Sid,
    pub text: AutoStr,
    pub ast: ast::Code,
    pub file: AutoStr,
    pub cfile: AutoStr,
    pub header: AutoStr,
}

/// Universe: **DEPRECATED** - Use Database + ExecutionEngine instead
///
/// # Deprecation Notice
///
/// **This structure is deprecated** and will be removed in a future version.
/// New code should use the split architecture:
/// - **`Database`** (compile-time): Types, symbols, scopes, ASTs
/// - **`ExecutionEngine`** (runtime): Values, VM refs, call stack
///
/// # Migration Guide
///
/// See [Plan 064](docs/plans/064-split-universe-compile-runtime.md) for details.
///
/// ## Quick Reference
///
/// | Old (Universe) | New (Split Architecture) |
/// |----------------|--------------------------|
/// | `universe.scopes` | `database.symbol_tables()` |
/// | `universe.types` | `database.get_types()` |
/// | `universe.values` | `engine.get_value()` |
/// | `universe.vm_refs` | `engine.get_vm_ref()` |
/// | `universe.args` | `engine.args` |
///
/// ## Current Status
///
/// The compiler is in a **hybrid migration state** (Plan 064 Phase 4.5):
/// - ✅ Database and ExecutionEngine implemented
/// - ✅ Bridge methods active (try Database/Engine first, fallback to Universe)
/// - ⏸️ VM reference migration deferred (Phase 4.7 blocked on lifetime issues)
/// - ⏸️ Parser/Indexer migration deferred (requires breaking API changes)
/// - ⏸️ Transpiler migration deferred (requires breaking API changes)
///
/// ## When to Use Universe
///
/// During migration, you may still need to use `universe()` and `universe_mut()`
/// getter methods from Evaler for:
/// - VM reference management (until Phase 4.7)
/// - Bridge method fallbacks when db/engine are None
/// - Diagnostic/debug code
///
/// ## Timeline
///
/// - **2025-01-31**: Phases 1-4 complete (60% migrated)
/// - **2025-02-01**: Phase 4.5 practically complete (hybrid architecture accepted)
/// - **Future**: Phases 4.7, 5-6 to complete migration
#[deprecated(since = "0.4.0", note = "Use Database + ExecutionEngine instead (see Plan 064)")]
pub struct Universe {
    pub scopes: HashMap<Sid, Scope>,   // sid -> scope
    pub asts: HashMap<Sid, ast::Code>, // sid -> ast
    pub code_paks: HashMap<Sid, CodePak>,
    // pub stack: Vec<StackedScope>,
    pub env_vals: HashMap<AutoStr, Box<dyn Any>>,
    pub shared_vals: HashMap<AutoStr, Rc<RefCell<Value>>>,
    pub builtins: HashMap<AutoStr, Value>, // Value of builtin functions
    pub vm_refs: HashMap<usize, RefCell<VmRefData>>,
    pub types: TypeInfoStore,
    pub args: Obj,
    lambda_counter: usize,
    pub cur_spot: Sid,
    vmref_counter: usize,

    // NEW: Central value storage for reference-based system
    value_counter: usize,
    pub values: HashMap<ValueID, Rc<RefCell<ValueData>>>,
    weak_refs: HashMap<ValueID, Weak<RefCell<ValueData>>>,

    // NEW: Symbol location table for LSP support
    // Maps symbol name -> definition location
    pub symbol_locations: HashMap<AutoStr, SymbolLocation>,

    // NEW: Type alias storage for Plan 058
    // Maps alias name -> (params, target_type)
    pub type_aliases: HashMap<AutoStr, (Vec<AutoStr>, Type)>,

    // Plan 061 Phase 2: Spec registry for constraint validation
    // Maps spec name -> spec declaration
    pub specs: HashMap<AutoStr, Rc<SpecDecl>>,

    // Raw pointer to evaluator for VM functions to call user-defined functions
    // WARNING: This is only valid during evaluator's lifetime
    // The evaluator must outlive the universe (guaranteed by ownership: Evaler owns Universe)
    evaluator_ptr: *mut crate::eval::Evaler,
}

impl Default for Universe {
    fn default() -> Self {
        Self::new()
    }
}

impl Universe {
    pub fn new() -> Self {
        let builtins = libs::builtin::builtins();
        let mut scopes = HashMap::new();
        scopes.insert(
            SID_PATH_GLOBAL.clone(),
            Scope::new(ScopeKind::Global, SID_PATH_GLOBAL.clone()),
        );
        let mut uni = Self {
            scopes,
            asts: HashMap::new(),
            code_paks: HashMap::new(),
            // stack: vec![StackedScope::new()],
            env_vals: HashMap::new(),
            shared_vals: HashMap::new(),
            builtins,
            vm_refs: HashMap::new(),
            types: TypeInfoStore::new(),
            lambda_counter: 0,
            vmref_counter: 0,
            cur_spot: SID_PATH_GLOBAL.clone(),
            args: Obj::new(),
            // NEW: Initialize value storage
            value_counter: 0,
            values: HashMap::new(),
            weak_refs: HashMap::new(),
            // NEW: Initialize symbol location table
            symbol_locations: HashMap::new(),
            // NEW: Initialize type alias storage
            type_aliases: HashMap::new(),
            // Plan 061: Initialize spec registry
            specs: HashMap::new(),
            // Initialize evaluator pointer (will be set by evaluator)
            evaluator_ptr: std::ptr::null_mut(),
        };
        uni.define_sys_types();
        uni.define_builtin_funcs();
        uni
    }

    pub fn set_args(&mut self, args: &Obj) {
        self.args = args.clone();
    }

    pub fn has_arg(&self, name: &str) -> bool {
        self.args.has(name)
    }

    pub fn get_arg(&self, name: &str) -> Value {
        self.args.get_or_nil(name)
    }

    /// Set the evaluator pointer for VM functions to call user-defined functions
    /// This allows VM operations like map/filter to call arbitrary user functions
    /// # Safety
    /// The evaluator must outlive the universe. This is guaranteed by the ownership
    /// structure where Evaler owns the Universe via Rc<RefCell<Universe>>
    pub fn set_evaluator(&mut self, evaluator: &mut crate::eval::Evaler) {
        self.evaluator_ptr = evaluator;
    }

    /// Set the evaluator pointer from a raw pointer
    /// # Safety
    /// The pointer must be valid and outlive the universe
    pub unsafe fn set_evaluator_raw(&mut self, evaluator: *mut crate::eval::Evaler) {
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
        // SAFETY: The evaluator outlives the universe during call chains
        // This is guaranteed by the ownership structure (Evaler owns Universe)
        unsafe {
            Some((*self.evaluator_ptr).eval_user_function(fn_name, args))
        }
    }

    /// Get the raw evaluator pointer
    /// # Safety
    /// The pointer must only be used while the original borrow is active
    pub unsafe fn get_evaluator_ptr(&self) -> *mut crate::eval::Evaler {
        self.evaluator_ptr
    }

    pub fn dump(&self) {
        for (name, meta) in self.builtins.iter() {
            println!("Builtin: {} = {}", name, meta);
        }
        for (name, meta) in self.scopes.iter() {
            println!("Scope: {} ->", name);
            meta.dump();
        }
    }

    pub fn chart(&self) -> AutoStr {
        let mut chart = String::new();
        for (sid, scope) in self.scopes.iter() {
            if let Some(parent) = &scope.parent {
                chart.push_str(&format!("{} -> {}\n", sid, parent));
            } else {
                chart.push_str(&format!("{} -> {}\n", sid, "Global"));
            }
        }
        // for (i, scope) in self.stack.iter().enumerate() {
        //     chart.push_str(&format!("{}: {}\n", i, scope.dump()));
        // }
        chart.into()
    }

    pub fn gen_lambda_id(&mut self) -> AutoStr {
        self.lambda_counter += 1;
        format!("lambda_{}", self.lambda_counter).into()
    }

    pub fn define_builtin_funcs(&mut self) {
        self.define(
            "print",
            Rc::new(Meta::Fn(ast::Fn::new(
                FnKind::Function,
                "print".into(),
                None,
                vec![],
                ast::Body::new(),
                ast::Type::Void,
            ))),
        );
    }

    pub fn define_sys_types(&mut self) {
        self.define("int", Rc::new(Meta::Type(ast::Type::Int)));
        self.define("uint", Rc::new(Meta::Type(ast::Type::Uint)));
        self.define("float", Rc::new(Meta::Type(ast::Type::Float)));
        self.define("double", Rc::new(Meta::Type(ast::Type::Double)));
        self.define("bool", Rc::new(Meta::Type(ast::Type::Bool)));
        self.define("str", Rc::new(Meta::Type(ast::Type::Str(0))));
        self.define("cstr", Rc::new(Meta::Type(ast::Type::CStr)));
        self.define("str_slice", Rc::new(Meta::Type(ast::Type::StrSlice)));  // Borrowed string slice (Phase 3)
        self.define("byte", Rc::new(Meta::Type(ast::Type::Byte)));
        self.define("char", Rc::new(Meta::Type(ast::Type::Char)));
        self.define("void", Rc::new(Meta::Type(ast::Type::Void)));
    }

    fn enter_named_scope(&mut self, name: impl Into<AutoStr>, kind: ScopeKind) {
        // Create a new scope under Global
        let new_sid = Sid::kid_of(&self.cur_spot, name.into());
        // if new_sid exists, return it
        if self.scopes.contains_key(&new_sid) {
            self.cur_spot = new_sid;
            self.cur_scope_mut().cur_block = 0;
            return;
        }
        let new_scope = Scope::new(kind, new_sid.clone());
        self.cur_scope_mut().kids.push(new_sid.clone());
        self.scopes.insert(new_sid.clone(), new_scope);
        self.cur_spot = new_sid;
    }

    pub fn enter_mod(&mut self, name: impl Into<AutoStr>) {
        self.enter_named_scope(name.into(), ScopeKind::Mod);
    }

    pub fn enter_fn(&mut self, name: impl Into<AutoStr>) {
        self.enter_named_scope(name.into(), ScopeKind::Fn);
    }

    pub fn enter_type(&mut self, name: impl Into<AutoStr>) {
        self.enter_named_scope(name.into(), ScopeKind::Type);
    }

    pub fn cur_scope(&self) -> &Scope {
        self.scopes.get(&self.cur_spot).unwrap()
    }

    pub fn cur_scope_mut(&mut self) -> &mut Scope {
        self.scopes.get_mut(&self.cur_spot).unwrap()
    }

    pub fn enter_scope(&mut self) {
        let name = format!("block_{}", self.cur_scope().cur_block);
        self.cur_scope_mut().cur_block += 1;
        self.enter_named_scope(name, ScopeKind::Block);
    }

    pub fn exit_mod(&mut self) {
        self.exit_scope();
    }

    pub fn exit_fn(&mut self) {
        self.exit_scope();
    }

    pub fn exit_type(&mut self) {
        self.exit_scope();
    }

    pub fn exit_scope(&mut self) {
        // Automatic cleanup: Drop all local variables in current scope
        // Get the current scope before we exit it
        let current_sid = self.cur_spot.clone();

        // Collect all ValueIDs from the current scope
        let _scope_vals_to_drop = if let Some(scope) = self.scopes.get(&current_sid) {
            scope.vals.values().copied().collect::<Vec<_>>()
        } else {
            Vec::new()
        };

        // NOTE: We cannot safely delete values here because ValueRefs don't
        // increase Rc reference counts. A ValueRef in a return value would become
        // a dangling reference if we delete the value here.
        //
        // Current strategy:
        // - Scope.vals is cleared when the scope is dropped (normal Rust behavior)
        // - Values in Universe.values persist until explicitly removed via remove_local()
        // - In Phase 2, we'll implement proper reference counting or epoch-based reclamation

        // Now move to parent scope
        let parent_sid = self.cur_spot.parent();
        if let Some(parent) = parent_sid {
            self.cur_spot = parent;
        } else {
            // println!("No parent scope to exit!"); // LSP: disabled
        }
    }

    pub fn reset_spot(&mut self) {
        self.cur_spot = SID_PATH_GLOBAL.clone();
    }

    pub fn set_spot(&mut self, spot: Sid) {
        self.cur_spot = spot;
    }

    pub fn current_scope(&self) -> &Scope {
        self.scopes.get(&self.cur_spot).unwrap_or_else(|| {
            // Defensive: if cur_spot is invalid, we should have been reset to global scope
            // This can happen if the universe state gets corrupted (e.g., during test cleanup)
            eprintln!("Warning: Current scope {:?} does not exist, resetting to global scope", self.cur_spot);
            eprintln!("Available scopes: {:?}", self.scopes.keys().collect::<Vec<_>>());
            // Return global scope as fallback
            self.scopes.get(&SID_PATH_GLOBAL).expect("Global scope must exist")
        })
    }

    pub fn current_scope_mut(&mut self) -> &mut Scope {
        // Defensive: if cur_spot is invalid, reset to global and continue
        // This can happen if the universe state gets corrupted (e.g., during test cleanup)
        if !self.scopes.contains_key(&self.cur_spot) {
            eprintln!("Warning: Current scope {:?} does not exist, resetting to global scope", self.cur_spot);
            self.cur_spot = SID_PATH_GLOBAL.clone();
        }
        self.scopes.get_mut(&self.cur_spot).expect("Global scope must exist")
    }

    pub fn global_scope(&self) -> &Scope {
        self.scopes
            .get(&SID_PATH_GLOBAL)
            .expect("No global scope left")
    }

    pub fn global_scope_mut(&mut self) -> &mut Scope {
        self.scopes
            .get_mut(&SID_PATH_GLOBAL)
            .expect("No global scope left")
    }

    pub fn set_local_val(&mut self, name: &str, value: Value) {
        // Allocate value with proper nested allocation
        let vid = self.alloc_value_from_value(value);
        self.current_scope_mut().set_val(name, vid);
    }

    /// Check if a local variable exists in the current scope
    pub fn has_local(&self, name: &str) -> bool {
        self.current_scope().has_val(name)
    }

    /// Remove a local variable from the current scope
    /// Returns the ValueID if found, None otherwise
    ///
    /// NOTE: This only removes the variable name from the scope, not the value data.
    /// The value data in Universe.values persists because ValueRefs don't increase
    /// Rc reference counts, and deleting values here could cause dangling references.
    pub fn remove_local(&mut self, name: &str) -> Option<ValueID> {
        self.current_scope_mut().remove_val(name)
    }

    /// Mark a variable as moved (for ownership semantics)
    pub fn mark_moved(&mut self, name: &str) {
        self.current_scope_mut().mark_moved(name);
    }

    /// Check if a variable has been moved
    pub fn is_moved(&self, name: &str) -> bool {
        self.current_scope().is_moved(name)
    }

    /// Clear moved status (used when variable is reassigned)
    pub fn clear_moved(&mut self, name: &str) {
        self.current_scope_mut().clear_moved(name);
    }

    pub fn set_local_obj(&mut self, obj: &Obj) {
        // TODO: too much clone
        for key in obj.keys() {
            let val = obj.get(key.clone());
            if let Some(v) = val {
                // Allocate value with proper nested allocation
                let vid = self.alloc_value_from_value(v.clone());
                self.current_scope_mut()
                    .set_val(key.to_string().as_str(), vid);
            }
        }
    }

    pub fn set_shared(&mut self, name: &str, value: Rc<RefCell<Value>>) {
        self.shared_vals.insert(name.into(), value);
    }

    pub fn get_shared(&self, name: &str) -> Option<Rc<RefCell<Value>>> {
        self.shared_vals.get(name).cloned()
    }

    pub fn has_global(&self, name: &str) -> bool {
        self.global_scope().exists(name)
    }

    pub fn set_global(&mut self, name: impl Into<String>, value: Value) {
        // Allocate value with proper nested allocation
        let vid = self.alloc_value_from_value(value);
        self.global_scope_mut().set_val(name.into(), vid);
    }

    pub fn add_global_fn(&mut self, name: &str, f: fn(&Args) -> Value) {
        // Allocate function value with proper nested allocation
        let value = Value::ExtFn(ExtFn {
            fun: f,
            name: name.into(),
        });
        let vid = self.alloc_value_from_value(value);
        self.global_scope_mut().set_val(name, vid);
    }

    pub fn get_global(&self, name: &str) -> Value {
        // TODO: Update to use ValueID resolution
        // For now, this is a compatibility shim
        self.global_scope()
            .get_val_id(name)
            .and_then(|vid| self.get_value(vid))
            .map(|cell| {
                let data = cell.borrow();
                // Convert ValueData back to Value (simplified)
                match &*data {
                    ValueData::Int(i) => Value::Int(*i),
                    ValueData::Str(s) => Value::Str(s.clone()),
                    ValueData::Bool(b) => Value::Bool(*b),
                    ValueData::Nil => Value::Nil,
                    _ => Value::Nil, // TODO: handle other cases
                }
            })
            .unwrap_or(Value::Nil)
    }

    pub fn define(&mut self, name: impl Into<AutoStr>, meta: Rc<Meta>) {
        let name = name.into();
        match meta.as_ref() {
            Meta::Enum(decl) => {
                let type_meta = Meta::Type(ast::Type::Enum(shared(decl.clone())));
                self.current_scope_mut()
                    .define_type(name.clone(), Rc::new(type_meta));
            }
            Meta::Type(_) => {
                // println!("Defining type {} in scope {}", name, self.cur_spot);
                self.current_scope_mut()
                    .define_type(name.clone(), meta.clone());
                // also put the Type name as a symbol into the scope
                // used for static method calls
                self.current_scope_mut().put_symbol(name.as_str(), meta);
            }
            Meta::Spec(decl) => {
                // Spec 也是一种类型，需要同时注册到 types 和 symbols
                let type_meta = Meta::Type(ast::Type::Spec(shared(decl.clone())));
                self.current_scope_mut()
                    .define_type(name.clone(), Rc::new(type_meta));
                self.current_scope_mut().put_symbol(name.as_str(), meta);
            }
            _ => {
                self.current_scope_mut().put_symbol(name.as_str(), meta);
            }
        }
    }

    pub fn define_type(&mut self, name: impl Into<AutoStr>, meta: Rc<Meta>) {
        self.current_scope_mut().define_type(name, meta);
    }

    pub fn define_env(&mut self, name: &str, val: Box<dyn Any>) {
        self.env_vals.insert(name.into(), val);
    }

    pub fn get_env(&self, name: &str) -> Option<&Box<dyn Any>> {
        self.env_vals.get(name)
    }

    pub fn define_global(&mut self, name: &str, meta: Rc<Meta>) {
        self.global_scope_mut().put_symbol(name, meta);
    }

    /// Register a symbol's definition location for LSP support
    pub fn define_symbol_location(&mut self, name: impl Into<AutoStr>, location: SymbolLocation) {
        self.symbol_locations.insert(name.into(), location);
    }

    /// Lookup a symbol's definition location
    pub fn get_symbol_location(&self, name: &str) -> Option<&SymbolLocation> {
        self.symbol_locations.get(name)
    }

    /// Get all symbol locations (for LSP workspace symbols)
    pub fn get_all_symbol_locations(&self) -> &HashMap<AutoStr, SymbolLocation> {
        &self.symbol_locations
    }

    /// Clear all symbol locations (when re-parsing a file)
    pub fn clear_symbol_locations(&mut self) {
        self.symbol_locations.clear();
    }

    pub fn is_fn(&self, name: &str) -> bool {
        // TODO: check meta if fn
        self.exists(name)
    }

    fn exists_recurse(&self, name: &str, sid: &Sid) -> bool {
        if let Some(scope) = self.scopes.get(sid) {
            if scope.exists(name) {
                return true;
            }
        }
        if let Some(parent) = sid.parent() {
            return self.exists_recurse(name, &parent);
        }
        false
    }

    pub fn exists(&self, name: &str) -> bool {
        if self.exists_recurse(name, &self.cur_spot) {
            return true;
        }
        // check for builtins
        let is_builtin = self.builtins.contains_key(name);
        is_builtin
    }

    #[allow(dead_code)]
    fn find_scope_for(&mut self, name: &str) -> Option<&mut Scope> {
        let mut sid = self.cur_spot.clone();
        loop {
            {
                let scope = self.scopes.get(&sid)?;
                if scope.exists(name) {
                    break;
                }
            }
            if let Some(parent) = sid.parent() {
                sid = parent;
            } else {
                return None;
            }
        }
        self.scopes.get_mut(&sid)
    }

    pub fn get_mut_val(&mut self, _name: &str) -> Option<&mut Value> {
        // DEPRECATED: Use the new value storage system instead
        // This method is kept for backward compatibility during migration
        None
    }

    fn lookup_val_recurse(&self, name: &str, sid: &Sid) -> Option<Value> {
        // First try to get ValueID from scopes
        if let Some(scope) = self.scopes.get(sid) {
            // Check if variable has been moved (use-after-move prevention)
            if scope.is_moved(name) {
                // Return error value indicating use-after-move
                return Some(Value::Error(format!("Use after move: variable '{}' has been moved", name).into()));
            }
            if let Some(vid) = scope.get_val_id(name) {
                // Resolve ValueID to Value (using ValueRef wrapper)
                return Some(Value::ValueRef(vid));
            }
        }
        if let Some(parent) = sid.parent() {
            return self.lookup_val_recurse(name, &parent);
        }
        None
    }

    pub fn lookup_val(&self, name: &str) -> Option<Value> {
        // Try scopes first (returns Value::ValueRef)
        if let Some(val) = self.lookup_val_recurse(name, &self.cur_spot) {
            return Some(val);
        }
        // Fallback to shared_vals (legacy)
        let shared = self.shared_vals.get(name);
        if let Some(shared) = shared {
            return Some(shared.borrow().clone());
        }
        // Fallback to builtins
        self.builtins.get(name).cloned()
    }

    fn lookup_val_mut_recurse(&mut self, name: &str, sid: &Sid) -> Option<&mut Value> {
        // DEPRECATED: Use get_value_mut with ValueID instead
        if !self.scopes.contains_key(sid) {
            if let Some(parent) = sid.parent() {
                return self.lookup_val_mut_recurse(name, &parent);
            }
        }
        // This method is deprecated - return None
        None
    }

    pub fn lookup_val_mut(&mut self, name: &str) -> Option<&mut Value> {
        // DEPRECATED: Use get_value_mut with ValueID instead
        let sid = self.cur_spot.clone();
        self.lookup_val_mut_recurse(name, &sid)
    }

    fn update_val_recurse(&mut self, name: &str, value: Value, sid: &Sid) {
        let exists = if let Some(scope) = self.scopes.get(sid) {
            scope.exists(name)
        } else {
            false
        };

        if exists {
            // Convert Value to ValueData with proper nested allocation
            let data = self.value_to_data_allocated(value);
            let vid = self.alloc_value(data);
            // Now get scope again after alloc_value
            if let Some(scope) = self.scopes.get_mut(sid) {
                scope.set_val(name, vid);
            }
            return;
        }

        if let Some(parent) = sid.parent() {
            self.update_val_recurse(name, value, &parent);
        }
    }

    /// Helper: Convert Value to ValueData, allocating nested values
    fn value_to_data_allocated(&mut self, value: Value) -> auto_val::ValueData {
        use auto_val::Value;
        match value {
            Value::Byte(v) => auto_val::ValueData::Byte(v),
            Value::Int(v) => auto_val::ValueData::Int(v),
            Value::Uint(v) => auto_val::ValueData::Uint(v),
            Value::USize(v) => auto_val::ValueData::USize(v),
            Value::I8(v) => auto_val::ValueData::I8(v),
            Value::U8(v) => auto_val::ValueData::U8(v),
            Value::I64(v) => auto_val::ValueData::I64(v),
            Value::Float(v) => auto_val::ValueData::Float(v),
            Value::Double(v) => auto_val::ValueData::Double(v),
            Value::Bool(v) => auto_val::ValueData::Bool(v),
            Value::Char(v) => auto_val::ValueData::Char(v),
            Value::Nil => auto_val::ValueData::Nil,
            Value::Str(v) => auto_val::ValueData::Str(v),
            Value::Array(v) => {
                // Allocate each element
                let vids: Vec<auto_val::ValueID> = v
                    .iter()
                    .map(|val| {
                        let data = self.value_to_data_allocated(val.clone());
                        self.alloc_value(data)
                    })
                    .collect();
                auto_val::ValueData::Array(vids)
            }
            Value::Obj(obj) => {
                // Allocate each field value
                let fields: Vec<(auto_val::ValueKey, auto_val::ValueID)> = obj
                    .iter()
                    .map(|(k, val)| {
                        let data = self.value_to_data_allocated(val.clone());
                        let vid = self.alloc_value(data);
                        (k.clone(), vid)
                    })
                    .collect();
                auto_val::ValueData::Obj(fields)
            }
            Value::Range(l, r) => auto_val::ValueData::Range(l, r),
            Value::RangeEq(l, r) => auto_val::ValueData::RangeEq(l, r),
            // Other variants - simplified
            _ => auto_val::ValueData::Nil,
        }
    }

    pub fn update_val(&mut self, name: &str, value: Value) {
        let sid = self.cur_spot.clone();
        self.update_val_recurse(name, value, &sid);
    }

    fn lookup_meta_recurse(&self, name: &str, sid: &Sid) -> Option<Rc<Meta>> {
        if let Some(scope) = self.scopes.get(sid) {
            if let Some(meta) = scope.get_symbol(name) {
                return Some(meta);
            }
        }
        if let Some(parent) = sid.parent() {
            return self.lookup_meta_recurse(name, &parent);
        }
        None
    }

    pub fn lookup_meta(&self, name: &str) -> Option<Rc<Meta>> {
        let sid = self.cur_spot.clone();
        self.lookup_meta_recurse(name, &sid)
    }

    pub fn find_type_for_name(&self, name: &str) -> Option<Type> {
        let meta = self.lookup_meta(name);
        if let Some(meta) = meta {
            match meta.as_ref() {
                Meta::Store(s) => {
                    return Some(s.ty.clone());
                }
                Meta::Type(s) => {
                    return Some(s.clone());
                }
                _ => return None,
            }
        }
        None
    }

    pub fn lookup_ident_type(&self, name: &str) -> Option<Type> {
        let meta = self.lookup_meta(name);
        if let Some(meta) = meta {
            if let Meta::Type(ty) = meta.as_ref() {
                return Some(ty.clone());
            }
        }
        None
    }

    fn lookup_type_recurse(&self, name: impl Into<AutoStr>, sid: &Sid) -> Option<Rc<Meta>> {
        let name = name.into();
        if let Some(scope) = self.scopes.get(sid) {
            if let Some(meta) = scope.lookup_type(name.clone()) {
                return Some(meta.clone());
            }
        }
        if let Some(parent) = sid.parent() {
            return self.lookup_type_recurse(name, &parent);
        }
        None
    }

    pub fn lookup_type_meta(&self, name: impl Into<AutoStr>) -> Option<Rc<Meta>> {
        let sid = self.cur_spot.clone();
        self.lookup_type_recurse(name, &sid)
    }

    pub fn lookup_type(&self, name: &str) -> ast::Type {
        match self.lookup_type_meta(name) {
            Some(meta) => match meta.as_ref() {
                Meta::Type(ty) => ty.clone(),
                Meta::Spec(spec_decl) => ast::Type::Spec(auto_val::shared(spec_decl.clone())),
                _ => ast::Type::Unknown,
            },
            None => ast::Type::Unknown,
        }
    }

    pub fn lookup(&self, name: &str, path: AutoStr) -> Option<Rc<Meta>> {
        let sid = Sid::new(path);
        self.lookup_meta_recurse(name, &sid)
    }

    pub fn lookup_sig(&self, sig: &Sig) -> Option<Rc<Meta>> {
        self.lookup_meta(&sig.name)
    }

    pub fn lookup_builtin(&self, name: &str) -> Option<Value> {
        self.builtins.get(name).cloned()
    }

    // Plan 061 Phase 2: Spec registry methods for constraint validation
    /// Register a spec declaration for constraint validation
    pub fn register_spec(&mut self, spec: Rc<SpecDecl>) {
        self.specs.insert(spec.name.clone(), spec);
    }

    /// Get a spec declaration by name
    pub fn get_spec(&self, name: &str) -> Option<Rc<SpecDecl>> {
        self.specs.get(name).cloned()
    }

    /// Get function declaration by name (for constraint checking)
    pub fn get_fn_decl(&self, name: &str) -> Option<ast::Fn> {
        // Lookup function metadata
        match self.lookup_meta(name) {
            Some(meta) => {
                match meta.as_ref() {
                    Meta::Fn(fn_decl) => Some(fn_decl.clone()),
                    _ => None,
                }
            }
            None => None,
        }
    }

    /// Get all defined variable/function names in current scope for suggestions
    ///
    /// Returns a list of names that could be used for "did you mean?" suggestions
    pub fn get_defined_names(&self) -> Vec<String> {
        let mut names = Vec::new();

        // Collect names from current scope and all parent scopes
        let mut current_sid = Some(self.cur_spot.clone());

        while let Some(sid) = current_sid {
            if let Some(scope) = self.scopes.get(&sid) {
                // Add all symbols (variables, functions, etc.)
                for name in scope.symbols.keys() {
                    names.push(name.to_string());
                }

                // Add all types
                for name in scope.types.keys() {
                    names.push(name.to_string());
                }

                // Move to parent scope
                current_sid = scope.parent.clone();
            } else {
                break;
            }
        }

        // Also add builtin functions
        for name in self.builtins.keys() {
            names.push(name.to_string());
        }

        names.sort();
        names.dedup();
        names
    }

    pub fn define_alias(&mut self, alias: AutoStr, target: AutoStr) {
        self.cur_scope_mut().define_alias(alias, target);
    }

    pub fn define_var(&mut self, name: &str, expr: ast::Expr) {
        // Add meta to current scope
        let ast_name = name.into();
        let store = ast::Store {
            kind: ast::StoreKind::Var,
            name: ast_name,
            ty: ast::Type::Int,
            expr,
        };
        self.define(name, Rc::new(Meta::Store(store)));
    }

    /// Update the type of an existing store in the current scope
    /// Used by the C transpiler when it infers types from expressions
    pub fn update_store_type(&mut self, name: &str, new_ty: ast::Type) {
        if let Some(meta) = self.lookup_meta(name) {
            if let Meta::Store(store) = meta.as_ref() {
                let updated_store = ast::Store {
                    kind: store.kind.clone(),
                    name: store.name.clone(),
                    ty: new_ty,
                    expr: store.expr.clone(),
                };
                self.define(name, Rc::new(Meta::Store(updated_store)));
            }
        }
    }

    pub fn import(&mut self, path: AutoStr, ast: ast::Code, file: AutoStr, text: AutoStr) {
        let sid = Sid::new(path.as_str());
        self.code_paks.insert(
            sid.clone(),
            CodePak {
                sid: sid.clone(),
                ast: ast.clone(),
                file: file.clone(),
                cfile: file.replace(".at", ".c"),
                header: file.replace(".at", ".h"),
                text: text.clone(),
            },
        );
        self.asts.insert(sid, ast);
    }

    // TODO: support nested nodes
    pub fn merge_atom(&mut self, atom: &Atom) {
        match atom {
            Atom::Node(node) => {
                // Extract properties from the node
                let name = node.get_prop_of("name");
                if !name.is_nil() {
                    self.set_global("name", name);
                }
                // Set properties from the node
                for (key, value) in node.props_iter() {
                    self.set_global(key.to_string(), value.clone());
                }
                // Set kids from the node
                let kids_groups = node.group_kids();
                for (name, kids) in kids_groups.iter() {
                    let plural_key = format!("{}s", name);
                    let key = plural_key.as_str();
                    // for each kid, set its main arg as `id`, and all props as is
                    let mut kids_vec: Vec<Value> = Vec::new();
                    for kid in kids.into_iter() {
                        kids_vec.push(Value::Node((*kid).clone()));
                    }
                    if !self.has_global(key) {
                        self.set_global(key, kids_vec.into());
                    } else {
                        let existing = self.get_global(key);
                        if let Value::Array(mut existing) = existing {
                            for kid in kids_vec.iter() {
                                existing.push(kid.clone());
                            }
                            self.set_global(key, Value::Array(existing));
                        }
                    }
                    // if len is 1, also set key with single form
                    if kids.len() == 1 {
                        let single_key = name.as_str();
                        let kid = kids[0].clone();
                        self.set_global(single_key, kid.into());
                    }
                }
            }
            Atom::Array(array) => {
                for (i, val) in array.iter().enumerate() {
                    self.set_global(format!("item_{}", i).as_str(), val.clone());
                }
            }
            _ => {}
        }
    }

    pub fn add_vmref(&mut self, data: VmRefData) -> usize {
        self.vmref_counter += 1;
        let refid = self.vmref_counter;
        self.vm_refs.insert(refid, RefCell::new(data));
        refid
    }

    /// DEPRECATED: Use get_vmref_ref() instead
    /// This method is kept for backward compatibility but returns None
    pub fn get_vmref(&mut self, _refid: usize) -> Option<&mut VmRefData> {
        // Cannot return mutable reference through RefCell
        // Use get_vmref_ref() to get &RefCell<Box<dyn Any>>, then borrow_mut()
        None
    }

    /// Get a reference to the RefCell containing the VM data
    /// This allows mutable access through interior mutability
    pub fn get_vmref_ref(&self, refid: usize) -> Option<&RefCell<VmRefData>> {
        self.vm_refs.get(&refid)
    }

    pub fn drop_vmref(&mut self, refid: usize) {
        self.vm_refs.remove(&refid);
    }

    // =========================================================================
    // NEW: Value Storage Methods (Reference-based system)
    // =========================================================================

    /// Allocate a new value and return its ID
    pub fn alloc_value(&mut self, data: ValueData) -> ValueID {
        self.value_counter += 1;
        let vid = ValueID(self.value_counter);
        let rc = Rc::new(RefCell::new(data));
        self.values.insert(vid, rc);
        vid
    }

    /// Allocate a value with parent tracking (for cycle detection)
    pub fn alloc_value_with_parent(&mut self, data: ValueData, parent: ValueID) -> ValueID {
        self.value_counter += 1;
        let vid = ValueID(self.value_counter);
        let rc = Rc::new(RefCell::new(data));

        // Store weak reference to parent for cycle detection
        if let Some(parent_rc) = self.values.get(&parent) {
            self.weak_refs.insert(vid, Rc::downgrade(parent_rc));
        }

        self.values.insert(vid, rc);
        vid
    }

    /// Allocate a Value, properly handling nested arrays/objects
    /// This replaces into_data() for Values that contain nested structures
    pub fn alloc_value_from_value(&mut self, value: Value) -> ValueID {
        match value {
            // Primitives - simple allocation
            Value::Byte(v) => self.alloc_value(ValueData::Byte(v)),
            Value::Int(v) => self.alloc_value(ValueData::Int(v)),
            Value::Uint(v) => self.alloc_value(ValueData::Uint(v)),
            Value::USize(v) => self.alloc_value(ValueData::USize(v)),
            Value::I8(v) => self.alloc_value(ValueData::I8(v)),
            Value::U8(v) => self.alloc_value(ValueData::U8(v)),
            Value::I64(v) => self.alloc_value(ValueData::I64(v)),
            Value::Float(v) => self.alloc_value(ValueData::Float(v)),
            Value::Double(v) => self.alloc_value(ValueData::Double(v)),
            Value::Bool(v) => self.alloc_value(ValueData::Bool(v)),
            Value::Char(v) => self.alloc_value(ValueData::Char(v)),
            Value::Nil => self.alloc_value(ValueData::Nil),
            Value::Str(v) => self.alloc_value(ValueData::Str(v)),
            Value::Range(l, r) => self.alloc_value(ValueData::Range(l, r)),
            Value::RangeEq(l, r) => self.alloc_value(ValueData::RangeEq(l, r)),

            // Array - allocate each element
            Value::Array(arr) => {
                let vids: Vec<ValueID> = arr
                    .iter()
                    .map(|v| self.alloc_value_from_value(v.clone()))
                    .collect();
                self.alloc_value(ValueData::Array(vids))
            }

            // Object - allocate each field value
            Value::Obj(obj) => {
                let mut fields = Vec::new();
                for (k, v) in obj.iter() {
                    let vid = self.alloc_value_from_value(v.clone());
                    fields.push((k.clone(), vid));
                }
                self.alloc_value(ValueData::Obj(fields))
            }

            // Pair - allocate both key and value
            Value::Pair(k, v) => {
                // Convert ValueKey to Value for allocation
                let k_value = match k {
                    auto_val::ValueKey::Str(s) => Value::Str(s.clone()),
                    auto_val::ValueKey::Int(i) => Value::Int(i),
                    auto_val::ValueKey::Bool(b) => Value::Bool(b),
                };
                let k_vid = self.alloc_value_from_value(k_value);
                let v_vid = self.alloc_value_from_value(*v.clone());
                self.alloc_value(ValueData::Pair(Box::new(k_vid), Box::new(v_vid)))
            }

            // For ValueRef, just return the ID (already allocated)
            Value::ValueRef(vid) => vid,

            // Other types not yet supported - store as Opaque
            // This preserves the full Value for functions, types, nodes, etc.
            _ => self.alloc_value(ValueData::Opaque(Box::new(value))),
        }
    }

    /// Get immutable reference to value data by ID
    pub fn get_value(&self, vid: ValueID) -> Option<Rc<RefCell<ValueData>>> {
        self.values.get(&vid).cloned()
    }

    /// Recursively dereference all VIDs in a value, replacing them with actual values
    pub fn deref_val(&self, val: Value) -> Value {
        match val {
            // Case 1: ValueRef - dereference and recursively process
            Value::ValueRef(vid) => {
                if let Some(d) = self.clone_value(vid) {
                    self.deref_val(Value::from_data(d))
                } else {
                    Value::Nil
                }
            }

            // Case 2: Instance - recursively dereference all fields
            Value::Instance(instance) => {
                let mut dereferenced_fields = auto_val::Obj::new();
                for (key, field_val) in instance.fields.iter() {
                    let deref_field_val = self.deref_val(field_val.clone());
                    dereferenced_fields.set(key.clone(), deref_field_val);
                }
                Value::Instance(auto_val::Instance {
                    ty: instance.ty,
                    fields: dereferenced_fields,
                })
            }

            // Case 3: Array - recursively dereference all elements
            Value::Array(arr) => {
                let dereferenced_elems: Vec<Value> = arr
                    .iter()
                    .map(|elem| self.deref_val(elem.clone()))
                    .collect();
                Value::Array(dereferenced_elems.into())
            }

            // Case 4: Obj (plain object) - recursively dereference all fields
            Value::Obj(obj) => {
                let mut dereferenced_obj = auto_val::Obj::new();
                for (key, field_val) in obj.iter() {
                    let deref_field_val = self.deref_val(field_val.clone());
                    dereferenced_obj.set(key.clone(), deref_field_val);
                }
                Value::Obj(dereferenced_obj)
            }

            // Case 5: Pair - recursively dereference both elements
            Value::Pair(key, val) => {
                let deref_val = self.deref_val(*val);
                Value::Pair(key, Box::new(deref_val))
            }

            // Case 6: Node - recursively dereference args, props, and kids
            Value::Node(node) => {
                // Clone fields we need before creating new node
                let name = node.name.clone();
                let id = node.id.clone();
                let num_args = node.num_args;
                let text = node.text.clone();

                // Create new node with same name and id
                let mut dereferenced_node = auto_val::Node::new(name);
                dereferenced_node.id = id;
                dereferenced_node.num_args = num_args;
                dereferenced_node.text = text;

                // Dereference all args (may contain ValueRef)
                for arg in &node.args.args {
                    let deref_arg = match arg {
                        auto_val::Arg::Pos(v) => {
                            auto_val::Arg::Pos(self.deref_val(v.clone()))
                        }
                        auto_val::Arg::Pair(k, v) => {
                            auto_val::Arg::Pair(k.clone(), self.deref_val(v.clone()))
                        }
                        auto_val::Arg::Name(n) => {
                            auto_val::Arg::Name(n.clone())
                        }
                    };
                    dereferenced_node.args.args.push(deref_arg);
                }

                // Dereference all props (args are already in props)
                for (key, prop_val) in node.props_iter() {
                    let deref_prop_val = self.deref_val(prop_val.clone());
                    dereferenced_node.set_prop(key.clone(), deref_prop_val);
                }

                // Dereference all kids
                for (key, kid) in node.kids_iter() {
                    let dereferenced_kid = match kid {
                        auto_val::Kid::Node(child_node) => {
                            let deref_node = self.deref_val(Value::Node(child_node.clone()));
                            auto_val::Kid::Node(deref_node.to_node().clone())
                        }
                        auto_val::Kid::Lazy(meta_id) => {
                            // Keep lazy references as-is
                            auto_val::Kid::Lazy(meta_id.clone())
                        }
                    };
                    // Add to kids (need to use add_node_kid or add_lazy_kid)
                    match dereferenced_kid {
                        auto_val::Kid::Node(n) => {
                            dereferenced_node.add_node_kid(key.clone(), n);
                        }
                        auto_val::Kid::Lazy(mid) => {
                            dereferenced_node.add_lazy_kid(key.clone(), mid);
                        }
                    }
                }

                Value::Node(dereferenced_node)
            }

            // Case 7: All other value types - return as-is (no nested VIDs)
            _ => val,
        }
    }

    /// Clone value data (for when you actually need a copy)
    pub fn clone_value(&self, vid: ValueID) -> Option<ValueData> {
        self.values.get(&vid).map(|v| v.borrow().clone())
    }

    /// Get mutable access to value data
    pub fn get_value_mut(&mut self, vid: ValueID) -> Option<std::cell::RefMut<'_, ValueData>> {
        self.values.get(&vid).map(|v| v.borrow_mut())
    }

    /// Update value data directly
    pub fn update_value(&mut self, vid: ValueID, new_data: ValueData) {
        if let Some(cell) = self.values.get(&vid) {
            *cell.borrow_mut() = new_data;
        }
    }

    /// Update nested field: obj.field = value
    pub fn update_nested(
        &mut self,
        vid: ValueID,
        path: &AccessPath,
        new_vid: ValueID,
    ) -> Result<(), AccessError> {
        // Flatten nested paths and process step by step
        let path_components = self.flatten_path(path);
        self.update_nested_iterative(vid, &path_components, 0, new_vid)
    }

    /// Flatten a potentially nested AccessPath into a vector of path components
    fn flatten_path(&self, path: &AccessPath) -> Vec<PathComponent> {
        let mut components = Vec::new();
        self.collect_path_components(path, &mut components);
        components
    }

    /// Recursively collect path components from an AccessPath
    fn collect_path_components(&self, path: &AccessPath, components: &mut Vec<PathComponent>) {
        match path {
            AccessPath::Field(field) => {
                components.push(PathComponent::Field(field.clone()));
            }
            AccessPath::Index(idx) => {
                components.push(PathComponent::Index(*idx));
            }
            AccessPath::Nested(parent, child) => {
                // Collect parent first, then child
                self.collect_path_components(parent, components);
                self.collect_path_components(child, components);
            }
        }
    }

    /// Iteratively update nested value following path components
    fn update_nested_iterative(
        &mut self,
        vid: ValueID,
        components: &[PathComponent],
        depth: usize,
        new_vid: ValueID,
    ) -> Result<(), AccessError> {
        // If we're at the last component, perform the update
        if depth == components.len() - 1 {
            return self.update_nested_single(vid, &components[depth], new_vid);
        }

        // Process current component to get the next vid
        let next_vid = match &components[depth] {
            PathComponent::Field(field) => {
                let cell = self.values.get(&vid).ok_or(AccessError::FieldNotFound)?;
                let data = cell.borrow();

                // First, extract what we need from the borrow
                let next_vid_result: Result<Value, AccessError> = match &*data {
                    ValueData::Obj(fields) => fields
                        .iter()
                        .find(|(k, _)| k == &auto_val::ValueKey::Str(field.clone()))
                        .map(|(_, v)| Value::ValueRef(*v))
                        .ok_or(AccessError::FieldNotFound),
                    ValueData::Opaque(ref opaque_val) => {
                        if let auto_val::Value::Instance(ref instance) = &**opaque_val {
                            // Use the lookup method which handles different ValueKey types
                            instance
                                .fields
                                .lookup(field)
                                .ok_or(AccessError::FieldNotFound)
                        } else {
                            Err(AccessError::NotAnObject)
                        }
                    }
                    _ => Err(AccessError::NotAnObject),
                };

                // Release the borrow before potentially allocating new values
                drop(data);

                // Now handle the result, allocating if needed
                match next_vid_result {
                    Ok(Value::ValueRef(inner_vid)) => inner_vid,
                    Ok(field_value) => {
                        // Allocate the value and get its VID
                        let field_data = field_value.into_data();
                        self.alloc_value(field_data)
                    }
                    Err(e) => return Err(e),
                }
            }
            PathComponent::Index(idx) => {
                let cell = self.values.get(&vid).ok_or(AccessError::FieldNotFound)?;
                let data = cell.borrow();
                match &*data {
                    ValueData::Array(elems) => {
                        if *idx < elems.len() {
                            elems[*idx]
                        } else {
                            return Err(AccessError::IndexOutOfBounds);
                        }
                    }
                    _ => return Err(AccessError::NotAnArray),
                }
            }
        };

        // Recurse to next level
        self.update_nested_iterative(next_vid, components, depth + 1, new_vid)
    }

    /// Update a single component (not nested)
    fn update_nested_single(
        &mut self,
        vid: ValueID,
        component: &PathComponent,
        new_vid: ValueID,
    ) -> Result<(), AccessError> {
        let cell = self.values.get(&vid).ok_or(AccessError::FieldNotFound)?;
        let mut data = cell.borrow_mut();

        match component {
            PathComponent::Field(field) => {
                if let ValueData::Obj(ref mut fields) = &mut *data {
                    // Check if field exists before mutating
                    let field_key = auto_val::ValueKey::Str(field.clone());
                    let field_exists = fields.iter().any(|(k, _)| k == &field_key);
                    if !field_exists {
                        return Err(AccessError::FieldNotFound);
                    }
                    // Find and remove existing field with this name, then add the new one
                    fields.retain(|(k, _)| k != &field_key);
                    fields.push((field_key, new_vid));
                    return Ok(());
                }

                // Check if it's an Opaque Instance
                if let ValueData::Opaque(_) = &*data {
                    // Need to use a different approach - get mutable access to the opaque value
                    drop(data); // Release the borrow
                    let cell = self.values.get(&vid).ok_or(AccessError::FieldNotFound)?;
                    let mut data = cell.borrow_mut();
                    if let ValueData::Opaque(ref mut opaque_val) = &mut *data {
                        if let auto_val::Value::Instance(ref mut instance) = &mut **opaque_val {
                            // Update the field in the instance (will create if doesn't exist)
                            instance.fields.set(
                                auto_val::ValueKey::Str(field.clone()),
                                auto_val::Value::ValueRef(new_vid),
                            );
                            return Ok(());
                        }
                    }
                }

                Err(AccessError::NotAnObject)
            }
            PathComponent::Index(idx) => {
                if let ValueData::Array(ref mut elems) = &mut *data {
                    if *idx < elems.len() {
                        elems[*idx] = new_vid;
                        Ok(())
                    } else {
                        Err(AccessError::IndexOutOfBounds)
                    }
                } else {
                    Err(AccessError::NotAnArray)
                }
            }
        }
    }

    /// Legacy update_nested method (now a wrapper that calls flatten_path)
    #[allow(dead_code)]
    fn update_nested_legacy(
        &mut self,
        vid: ValueID,
        path: &AccessPath,
        new_vid: ValueID,
    ) -> Result<(), AccessError> {
        let cell = self.values.get(&vid).ok_or(AccessError::FieldNotFound)?;
        let mut data = cell.borrow_mut();

        match path {
            AccessPath::Field(field) => {
                // Check if it's an Obj
                if let ValueData::Obj(ref mut fields) = &mut *data {
                    // Find and remove existing field with this name, then add the new one
                    let field_key = auto_val::ValueKey::Str(field.clone());
                    fields.retain(|(k, _)| k != &field_key);
                    fields.push((field_key, new_vid));
                    return Ok(());
                }

                // Check if it's an Opaque Instance
                if let ValueData::Opaque(_) = &*data {
                    // Need to use a different approach - get mutable access to the opaque value
                    drop(data); // Release the borrow
                    let cell = self.values.get(&vid).ok_or(AccessError::FieldNotFound)?;
                    let mut data = cell.borrow_mut();
                    if let ValueData::Opaque(ref mut opaque_val) = &mut *data {
                        if let auto_val::Value::Instance(ref mut instance) = &mut **opaque_val {
                            // Update the field in the instance
                            instance.fields.set(
                                auto_val::ValueKey::Str(field.clone()),
                                auto_val::Value::ValueRef(new_vid),
                            );
                            return Ok(());
                        }
                    }
                }

                Err(AccessError::NotAnObject)
            }
            AccessPath::Index(idx) => {
                if let ValueData::Array(ref mut elems) = &mut *data {
                    if *idx < elems.len() {
                        elems[*idx] = new_vid;
                        Ok(())
                    } else {
                        Err(AccessError::IndexOutOfBounds)
                    }
                } else {
                    Err(AccessError::NotAnArray)
                }
            }
            AccessPath::Nested(parent_path, child_path) => {
                // First resolve parent, then recurse
                let parent_vid = match &*data {
                    ValueData::Obj(fields) => {
                        let key = match &**parent_path {
                            AccessPath::Field(f) => f.clone(),
                            _ => return Err(AccessError::NotAnObject),
                        };
                        fields
                            .iter()
                            .find(|(k, _)| k == &auto_val::ValueKey::Str(key.clone()))
                            .map(|(_, vid)| *vid)
                            .ok_or(AccessError::FieldNotFound)?
                    }
                    ValueData::Array(elems) => {
                        let idx = match &**parent_path {
                            AccessPath::Index(i) => *i,
                            _ => return Err(AccessError::NotAnArray),
                        };
                        *elems.get(idx).ok_or(AccessError::IndexOutOfBounds)?
                    }
                    _ => return Err(AccessError::NotAnObject),
                };
                drop(data); // Release borrow before recursion
                self.update_nested(parent_vid, child_path, new_vid)
            }
        }
    }

    /// Check if creating an edge would create a cycle
    pub fn would_create_cycle(&self, parent: ValueID, child: ValueID) -> bool {
        self.has_path(child, parent)
    }

    fn has_path(&self, from: ValueID, to: ValueID) -> bool {
        if from == to {
            return true;
        }
        if let Some(cell) = self.get_value(from) {
            let data = cell.borrow();
            match &*data {
                ValueData::Array(elems) => elems.iter().any(|&vid| self.has_path(vid, to)),
                ValueData::Obj(fields) => fields.iter().any(|(_, vid)| self.has_path(*vid, to)),
                ValueData::Pair(left, right) => {
                    self.has_path(**left, to) || self.has_path(**right, to)
                }
                _ => false,
            }
        } else {
            false
        }
    }

    /// Lookup value ID by name (NEW)
    pub fn lookup_val_id(&self, name: &str) -> Option<ValueID> {
        self.lookup_val_id_recurse(name, &self.cur_spot)
    }

    fn lookup_val_id_recurse(&self, name: &str, sid: &Sid) -> Option<ValueID> {
        if let Some(scope) = self.scopes.get(sid) {
            if let Some(vid) = scope.get_val_id(name) {
                return Some(vid);
            }
        }
        if let Some(parent) = sid.parent() {
            return self.lookup_val_id_recurse(name, &parent);
        }
        None
    }

    // ============================================================================
    // Environment Injection (Plan 055: Storage-based Environment Injection)
    // ============================================================================

    /// 注入环境变量到 Universe
    ///
    /// 此方法根据目标平台（MCU/PC）注入相应的环境变量，
    /// 用于支持基于 Storage 的环境注入机制。
    ///
    /// # 参数
    ///
    /// * `target` - 编译目标（Mcu 或 Pc）
    ///
    /// # 注入的环境变量
    ///
    /// - `TARGET`: "mcu" 或 "pc"
    /// - `DEFAULT_STORAGE`: "Fixed<64>" (MCU) 或 "Dynamic" (PC)
    /// - `HAS_HEAP`: "1" (PC) 或 "0" (MCU)
    ///
    /// # 示例
    ///
    /// ```rust
    /// use auto_lang::Universe;
    /// use auto_lang::target::Target;
    ///
    /// let mut uni = Universe::new();
    /// uni.inject_environment(Target::Pc);
    ///
    /// assert_eq!(uni.get_env_val("TARGET"), Some("pc".into()));
    /// assert_eq!(uni.get_env_val("DEFAULT_STORAGE"), Some("Dynamic".into()));
    /// ```
    pub fn inject_environment(&mut self, target: crate::target::Target) {
        self.set_env_val("TARGET", target.to_string());
        self.set_env_val("DEFAULT_STORAGE", target.default_storage_str().to_string());
        self.set_env_val("HAS_HEAP", if target.has_heap() { "1".to_string() } else { "0".to_string() });
    }

    /// 设置环境变量
    ///
    /// # 参数
    ///
    /// * `name` - 环境变量名称
    /// * `value` - 环境变量值
    ///
    /// # 示例
    ///
    /// ```rust
    /// use auto_lang::Universe;
    ///
    /// let mut uni = Universe::new();
    /// uni.set_env_val("MY_VAR", "my_value");
    /// ```
    pub fn set_env_val(&mut self, name: &str, value: String) {
        self.env_vals.insert(name.into(), Box::new(value));
    }

    /// 获取环境变量
    ///
    /// # 参数
    ///
    /// * `name` - 环境变量名称
    ///
    /// # 返回
    ///
    /// 如果环境变量存在，返回 Some(value)，否则返回 None
    ///
    /// # 示例
    ///
    /// ```rust
    /// use auto_lang::Universe;
    ///
    /// let mut uni = Universe::new();
    /// uni.set_env_val("MY_VAR", "my_value".to_string());
    ///
    /// assert_eq!(uni.get_env_val("MY_VAR"), Some("my_value".into()));
    /// assert_eq!(uni.get_env_val("NONEXISTENT"), None);
    /// ```
    pub fn get_env_val(&self, name: &str) -> Option<AutoStr> {
        self.env_vals.get(name).and_then(|boxed| {
            boxed.downcast_ref::<String>().map(|s| s.as_str().into())
        })
    }

    // ============================================================================
    // Type Alias Management (Plan 058)
    // ============================================================================

    /// Define a type alias in the current scope
    pub fn define_type_alias(&mut self, name: AutoStr, params: Vec<AutoStr>, target: Type) {
        self.type_aliases.insert(name, (params, target));
    }

    /// Look up a type alias by name
    pub fn lookup_type_alias(&self, name: &str) -> Option<&(Vec<AutoStr>, Type)> {
        self.type_aliases.get(name)
    }

    /// Check if a name is a type alias
    pub fn is_type_alias(&self, name: &str) -> bool {
        self.type_aliases.contains_key(name)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_global_define_and_lookup_type() {
        let uni = Rc::new(RefCell::new(Universe::new()));
        let uni_clone = uni.clone();
        uni_clone
            .borrow_mut()
            .define_type("int", Rc::new(Meta::Type(ast::Type::Int)));

        let typ = uni.borrow().lookup_type("int");
        assert!(matches!(typ, ast::Type::Int));
    }

    // ============================================================================
    // Environment Injection Tests (Plan 055)
    // ============================================================================

    #[test]
    fn test_set_and_get_env_val() {
        let mut uni = Universe::new();

        // 测试设置和获取环境变量
        uni.set_env_val("TEST_VAR", "test_value".to_string());
        assert_eq!(uni.get_env_val("TEST_VAR"), Some("test_value".into()));

        // 测试不存在的环境变量
        assert_eq!(uni.get_env_val("NONEXISTENT"), None);
    }

    #[test]
    fn test_inject_environment_mcu() {
        let mut uni = Universe::new();
        uni.inject_environment(crate::target::Target::Mcu);

        assert_eq!(uni.get_env_val("TARGET"), Some("mcu".into()));
        assert_eq!(uni.get_env_val("DEFAULT_STORAGE"), Some("Fixed<64>".into()));
        assert_eq!(uni.get_env_val("HAS_HEAP"), Some("0".into()));
    }

    #[test]
    fn test_inject_environment_pc() {
        let mut uni = Universe::new();
        uni.inject_environment(crate::target::Target::Pc);

        assert_eq!(uni.get_env_val("TARGET"), Some("pc".into()));
        assert_eq!(uni.get_env_val("DEFAULT_STORAGE"), Some("Dynamic".into()));
        assert_eq!(uni.get_env_val("HAS_HEAP"), Some("1".into()));
    }

    #[test]
    fn test_env_val_overwrite() {
        let mut uni = Universe::new();

        // 设置初始值
        uni.set_env_val("VAR", "value1".to_string());
        assert_eq!(uni.get_env_val("VAR"), Some("value1".into()));

        // 覆盖值
        uni.set_env_val("VAR", "value2".to_string());
        assert_eq!(uni.get_env_val("VAR"), Some("value2".into()));
    }
}

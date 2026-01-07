use super::scope::*;
use crate::ast::FnKind;
use crate::ast::{self, Type};
use crate::libs;
use auto_atom::Atom;
use auto_val::{Args, AutoStr, ExtFn, NodeItem, Obj, Sig, TypeInfoStore, Value, ValueID, ValueData, AccessPath, AccessError, shared};
use std::any::Any;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::rc::Weak;

#[derive(Debug, Clone)]
pub struct CodePak {
    pub sid: Sid,
    pub text: AutoStr,
    pub ast: ast::Code,
    pub file: AutoStr,
    pub cfile: AutoStr,
    pub header: AutoStr,
}

pub struct Universe {
    pub scopes: HashMap<Sid, Scope>,   // sid -> scope
    pub asts: HashMap<Sid, ast::Code>, // sid -> ast
    pub code_paks: HashMap<Sid, CodePak>,
    // pub stack: Vec<StackedScope>,
    pub env_vals: HashMap<AutoStr, Box<dyn Any>>,
    pub shared_vals: HashMap<AutoStr, Rc<RefCell<Value>>>,
    pub builtins: HashMap<AutoStr, Value>, // Value of builtin functions
    pub vm_refs: HashMap<usize, Box<dyn Any>>,
    pub types: TypeInfoStore,
    pub args: Obj,
    lambda_counter: usize,
    pub cur_spot: Sid,
    vmref_counter: usize,

    // NEW: Central value storage for reference-based system
    value_counter: usize,
    pub values: HashMap<ValueID, Rc<RefCell<ValueData>>>,
    weak_refs: HashMap<ValueID, Weak<RefCell<ValueData>>>,
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
        let parent_sid = self.cur_spot.parent();
        if let Some(parent) = parent_sid {
            self.cur_spot = parent;
        } else {
            println!("No parent scope to exit!");
        }
    }

    pub fn reset_spot(&mut self) {
        self.cur_spot = SID_PATH_GLOBAL.clone();
    }

    pub fn set_spot(&mut self, spot: Sid) {
        self.cur_spot = spot;
    }

    pub fn current_scope(&self) -> &Scope {
        self.scopes.get(&self.cur_spot).expect("No scope left")
    }

    pub fn current_scope_mut(&mut self) -> &mut Scope {
        self.scopes.get_mut(&self.cur_spot).expect("No scope left")
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
        // Convert Value to ValueData and allocate
        let data = value.into_data();
        let vid = self.alloc_value(data);
        self.current_scope_mut().set_val(name, vid);
    }

    pub fn set_local_obj(&mut self, obj: &Obj) {
        // TODO: too much clone
        for key in obj.keys() {
            let val = obj.get(key.clone());
            if let Some(v) = val {
                // Convert Value to ValueData and allocate
                let data = v.into_data();
                let vid = self.alloc_value(data);
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
        // Convert Value to ValueData and allocate
        let data = value.into_data();
        let vid = self.alloc_value(data);
        self.global_scope_mut().set_val(name.into(), vid);
    }

    pub fn add_global_fn(&mut self, name: &str, f: fn(&Args) -> Value) {
        // Convert Value to ValueData and allocate
        let value = Value::ExtFn(ExtFn {
            fun: f,
            name: name.into(),
        });
        let data = value.into_data();
        let vid = self.alloc_value(data);
        self.global_scope_mut().set_val(name, vid);
    }

    pub fn get_global(&self, name: &str) -> Value {
        // TODO: Update to use ValueID resolution
        // For now, this is a compatibility shim
        self.global_scope().get_val_id(name)
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

    pub fn get_mut_val(&mut self, name: &str) -> Option<&mut Value> {
        // DEPRECATED: Use the new value storage system instead
        // This method is kept for backward compatibility during migration
        None
    }

    fn lookup_val_recurse(&self, name: &str, sid: &Sid) -> Option<Value> {
        // First try to get ValueID from scopes
        if let Some(scope) = self.scopes.get(sid) {
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

    fn update_obj_recurse(&mut self, name: &str, f: impl FnOnce(&mut Obj)) {
        // DEPRECATED: Use update_nested instead
        // This is a no-op during migration
    }

    pub fn update_obj(&mut self, name: &str, f: impl FnOnce(&mut Obj)) {
        // DEPRECATED: Use update_nested instead
        eprintln!("Warning: update_obj is deprecated. Use update_nested instead.");
    }

    fn update_array_recurse(&mut self, name: &str, idx: Value, val: Value) {
        // DEPRECATED: Use update_nested instead
        eprintln!("Warning: update_array_recurse is deprecated. Use update_nested instead.");
    }

    pub fn update_array(&mut self, name: &str, idx: Value, val: Value) {
        self.update_array_recurse(name, idx, val);
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
                let vids: Vec<auto_val::ValueID> = v.iter()
                    .map(|val| {
                        let data = self.value_to_data_allocated(val.clone());
                        self.alloc_value(data)
                    })
                    .collect();
                auto_val::ValueData::Array(vids)
            }
            Value::Obj(obj) => {
                // Allocate each field value
                let fields: Vec<(auto_val::ValueKey, auto_val::ValueID)> = obj.iter()
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
        match &atom.root {
            auto_atom::Root::Node(node) => {
                // let main_arg = node.main_arg();
                // self.set_global("name", main_arg);
                let name = node.get_prop_of("name");
                if !name.is_nil() {
                    self.set_global("name", name);
                }
                for (_key, item) in node.props_iter() {
                    self.set_global(_key.to_string(), item.clone());
                }
                // set kids
                let kids_groups = node.group_kids();
                for (name, kids) in kids_groups.iter() {
                    let plural_key = format!("{}s", name);
                    let key = plural_key.as_str();
                    // for each kid, set its main arg as `id`, and all props as is
                    let mut kids_vec: Vec<Value> = Vec::new();
                    for kid in kids.into_iter() {
                        // let mut props = kid.props.clone();
                        // props.set("name", kid.main_arg());
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
            auto_atom::Root::NodeBody(node) => {
                // let main_arg = node.main_arg();
                // self.set_global("name", main_arg);
                let name = node.get_prop_of("name");
                if !name.is_nil() {
                    self.set_global("name", name);
                }
                for (_key, item) in node.map.iter() {
                    match item {
                        NodeItem::Prop(p) => {
                            self.set_global(p.key.to_string(), p.value.clone());
                        }
                        _ => {
                            //
                        }
                    }
                }
                // set kids
                let kids_groups = node.group_kids();
                for (name, kids) in kids_groups.iter() {
                    let plural_key = format!("{}s", name);
                    let key = plural_key.as_str();
                    // for each kid, set its main arg as `id`, and all props as is
                    let mut kids_vec: Vec<Value> = Vec::new();
                    for kid in kids.into_iter() {
                        // let mut props = kid.props.clone();
                        // props.set("name", kid.main_arg());
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
            auto_atom::Root::Array(array) => {
                for (i, val) in array.iter().enumerate() {
                    self.set_global(format!("item_{}", i).as_str(), val.clone());
                }
            }
            _ => {}
        }
    }

    pub fn add_vmref(&mut self, data: Box<dyn Any>) -> usize{
        self.vmref_counter += 1;
        let refid = self.vmref_counter;
        self.vm_refs.insert(refid, data);
        refid
    }

    pub fn get_vmref(&mut self, refid: usize) -> Option<&mut Box<dyn Any>> {
        self.vm_refs.get_mut(&refid)
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

    /// Get immutable reference to value data by ID
    pub fn get_value(&self, vid: ValueID) -> Option<Rc<RefCell<ValueData>>> {
        self.values.get(&vid).cloned()
    }

    /// Clone value data (for when you actually need a copy)
    pub fn clone_value(&self, vid: ValueID) -> Option<ValueData> {
        self.values.get(&vid).map(|v| v.borrow().clone())
    }

    /// Get mutable access to value data
    pub fn get_value_mut(&mut self, vid: ValueID) -> Option<std::cell::RefMut<ValueData>> {
        self.values.get(&vid).map(|v| v.borrow_mut())
    }

    /// Update value data directly
    pub fn update_value(&mut self, vid: ValueID, new_data: ValueData) {
        if let Some(cell) = self.values.get(&vid) {
            *cell.borrow_mut() = new_data;
        }
    }

    /// Update nested field: obj.field = value
    pub fn update_nested(&mut self, vid: ValueID, path: &AccessPath, new_vid: ValueID) -> Result<(), AccessError> {
        let cell = self.values.get(&vid).ok_or(AccessError::FieldNotFound)?;
        let mut data = cell.borrow_mut();

        match path {
            AccessPath::Field(field) => {
                if let ValueData::Obj(ref mut fields) = &mut *data {
                    fields.push((auto_val::ValueKey::Str(field.clone()), new_vid));
                    Ok(())
                } else {
                    Err(AccessError::NotAnObject)
                }
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
                        fields.iter()
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
                ValueData::Array(elems) => {
                    elems.iter().any(|&vid| self.has_path(vid, to))
                }
                ValueData::Obj(fields) => {
                    fields.iter().any(|(_, vid)| self.has_path(*vid, to))
                }
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
}

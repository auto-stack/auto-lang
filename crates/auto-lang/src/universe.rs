use super::scope::*;
use crate::ast::FnKind;
use crate::ast::{self, Type};
use crate::libs;
use auto_atom::Atom;
use auto_val::{
    shared, AccessError, AccessPath, Args, AutoStr, ExtFn, NodeItem, Obj, PathComponent, Sig,
    TypeInfoStore, Value, ValueData, ValueID,
};
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
        // Allocate value with proper nested allocation
        let vid = self.alloc_value_from_value(value);
        self.current_scope_mut().set_val(name, vid);
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

    #[allow(dead_code)]
    fn update_obj_recurse(&mut self, _name: &str, _f: impl FnOnce(&mut Obj)) {
        // DEPRECATED: Use update_nested instead
        // This is a no-op during migration
    }

    pub fn update_obj(&mut self, _name: &str, _f: impl FnOnce(&mut Obj)) {
        // DEPRECATED: Use update_nested instead
        eprintln!("Warning: update_obj is deprecated. Use update_nested instead.");
    }

    fn update_array_recurse(&mut self, _name: &str, _idx: Value, _val: Value) {
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

    pub fn add_vmref(&mut self, data: Box<dyn Any>) -> usize {
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

            // Case 6: Node - recursively dereference args, props, nodes, and body
            Value::Node(node) => {
                // Clone fields we need before creating new node
                let name = node.name.clone();
                let id = node.id.clone();
                let text = node.text.clone();
                let body_ref = node.body_ref.clone();
                let args = &node.args;
                let nodes = &node.nodes;
                let body = &node.body;

                // Create new node with same name and id
                let mut dereferenced_node = auto_val::Node::new(name);
                dereferenced_node.id = id;
                dereferenced_node.text = text;
                dereferenced_node.body_ref = body_ref;

                // Dereference all args
                for arg in args.args.iter() {
                    match arg {
                        auto_val::Arg::Pos(val) => {
                            let deref_val = self.deref_val(val.clone());
                            dereferenced_node
                                .args
                                .args
                                .push(auto_val::Arg::Pos(deref_val));
                        }
                        auto_val::Arg::Name(name) => {
                            dereferenced_node
                                .args
                                .args
                                .push(auto_val::Arg::Name(name.clone()));
                        }
                        auto_val::Arg::Pair(key, val) => {
                            let deref_val = self.deref_val(val.clone());
                            dereferenced_node
                                .args
                                .args
                                .push(auto_val::Arg::Pair(key.clone(), deref_val));
                        }
                    }
                }

                // Dereference all props
                for (key, prop_val) in node.props_iter() {
                    let deref_prop_val = self.deref_val(prop_val.clone());
                    dereferenced_node.set_prop(key.clone(), deref_prop_val);
                }

                // Dereference all child nodes
                for child_node in nodes.iter() {
                    let deref_child = self.deref_val(Value::Node(child_node.clone()));
                    dereferenced_node.nodes.push(deref_child.to_node().clone());
                }

                // Dereference all items in body
                for (key, body_item) in body.map.iter() {
                    let dereferenced_item = match body_item {
                        auto_val::NodeItem::Prop(pair) => {
                            let deref_val = self.deref_val(pair.value.clone());
                            auto_val::NodeItem::Prop(auto_val::Pair::new(
                                pair.key.clone(),
                                deref_val,
                            ))
                        }
                        auto_val::NodeItem::Node(child_node) => {
                            let deref_node = self.deref_val(Value::Node(child_node.clone()));
                            auto_val::NodeItem::Node(deref_node.to_node().clone())
                        }
                    };
                    dereferenced_node
                        .body
                        .map
                        .insert(key.clone(), dereferenced_item);
                }
                dereferenced_node.body.index = body.index.clone();

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

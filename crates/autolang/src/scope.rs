use std::collections::HashMap;
use autoval::{Value, Args};
use autoval::Sig;
use autoval::MetaID;
use crate::ast;
use crate::libs;
use std::rc::Rc;
use autoval::{TypeInfoStore, ExtFn, Obj};
use std::any::Any;
use std::fmt;

const SID_PATH_GLOBAL: &str = "::";

pub enum ScopeKind {
    Global,
    Mod,
    Type,
    Fn,
}

// TODO: Sid should be a Sharable object, with cheap cloning, like SharedString
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Sid {
    path: String,
}

impl From<String> for Sid {
    fn from(value: String) -> Self {
        Self {
            path: value
        }
    }
}

impl From<&str> for Sid {
    fn from(value: &str) -> Self {
        Self {
            path: value.to_string()
        }
    }
}

impl Sid {
    pub fn new(path: impl Into<String>) -> Self {
        Self {
            path: path.into()
        }
    }

    pub fn kid_of(parent: &Sid, name: impl Into<String>) -> Self {
        Self {
            path: format!("{}.{}", parent.path, name.into())
        }
    }

    pub fn global(name: impl Into<String>) -> Self {
        Self {
            path: format!("{}.{}", SID_PATH_GLOBAL, name.into())
        }
    }

    pub fn parent(&self) -> Self {
        let parent_path = if let Some(pos) = self.path.rfind('.') {
            &self.path[0..pos]
        } else {
            SID_PATH_GLOBAL
        };
        Self {
            path: parent_path.to_string()
        }
    }

    pub fn name(&self) -> &str {
        if let Some(pos) = self.path.rfind('.') {
            &self.path[pos+1..]
        } else {
            self.path.as_str()
        }
    }
}

pub struct Scope {
    pub kind: ScopeKind,
    pub sid: Sid, // TODO: should use SharedString?
    pub parent: Sid, // sid to parent
    pub kids: Vec<Sid>,
    pub symbols: HashMap<String, Rc<Meta>>,
    pub vals: HashMap<String, Value>,
}

impl Scope {
    pub fn new(kind: ScopeKind, sid: Sid) -> Self {
        let parent = sid.parent();
        Self {
            kind,
            sid,
            parent,
            kids: Vec::new(),
            symbols: HashMap::new(),
            vals: HashMap::new(),
        }
    }

    pub fn is_global(&self) -> bool {
        return matches!(self.kind, ScopeKind::Global)
    }

    pub fn dump(&self) {
        println!("Vals: {:?}", self.vals);
        println!("Symbols: {:?}", self.symbols);
    }

    pub fn set_val(&mut self, name: impl Into<String>, value: Value) {
        self.vals.insert(name.into(), value);
    }

    pub fn get_val(&self, name: &str) -> Option<Value> {
        self.vals.get(name).cloned()
    }

    pub fn get_val_mut(&mut self, name: &str) -> Option<&mut Value> {
        self.vals.get_mut(name)
    }

    pub fn put_symbol(&mut self, name: &str, meta: Rc<Meta>) {
        self.symbols.insert(name.to_string(), meta);
    }

    pub fn get_symbol(&self, name: &str) -> Option<&Rc<Meta>> {
        self.symbols.get(name)
    }

    pub fn exists(&self, name: &str) -> bool {
        self.symbols.contains_key(name) || self.vals.contains_key(name)
    }
}

pub enum ScopeSpot {
    Global,  // Global Scope
    Mid(Sid), // One of the named scopes: mod/type/fn
    Local, // Unnamed local scopes: e.g. in a if/else block
}

pub struct Universe {
    pub scopes: HashMap<Sid, Scope>, // sid -> scope
    pub stack: Vec<StackedScope>,
    pub env_vals: HashMap<String, Box<dyn Any>>,
    pub builtins: HashMap<String, Value>, // Value of builtin functions
    pub types: TypeInfoStore,
    lambda_counter: usize,
    cur_spot: ScopeSpot,
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
        let global_sid = Sid::new(SID_PATH_GLOBAL);
        scopes.insert(global_sid.clone(), Scope::new(ScopeKind::Global, global_sid.clone()));
        let mut uni = Self {
            scopes,
            stack: vec![StackedScope::new()],
            env_vals: HashMap::new(),
            builtins, 
            types: TypeInfoStore::new(), 
            lambda_counter: 0,
            cur_spot: ScopeSpot::Global,
        };
        uni.define_sys_types();
        uni
    }

    pub fn dump(&self) {
        for scope in self.stack.iter() {
            scope.dump();
        }
        for (name, meta) in self.builtins.iter() {
            println!("Builtin: {} = {}", name, meta);
        }
    }

    pub fn gen_lambda_id(&mut self) -> String {
        self.lambda_counter += 1;
        format!("lambda_{}", self.lambda_counter)
    }

    pub fn define_sys_types(&mut self) {
        self.define("int", Rc::new(Meta::Type(ast::Type::Int)));
        self.define("float", Rc::new(Meta::Type(ast::Type::Float)));
        self.define("bool", Rc::new(Meta::Type(ast::Type::Bool)));
        self.define("str", Rc::new(Meta::Type(ast::Type::Str)));
        self.define("byte", Rc::new(Meta::Type(ast::Type::Byte)));
    }

    pub fn enter_mod(&mut self, name: impl Into<String>) {
        match &self.cur_spot {
            ScopeSpot::Global => {
                // Create a new scope under global
                let sid = Sid::global(name);
                let new_scope = Scope::new(ScopeKind::Mod, sid.clone());
                self.scopes.insert(sid.clone(), new_scope);
                self.cur_spot = ScopeSpot::Mid(sid);
            }
            ScopeSpot::Mid(cur_sid) => {
                let sid = Sid::kid_of(cur_sid, name);
                let new_scope = Scope::new(ScopeKind::Mod, sid.clone());
                self.scopes.insert(sid.clone(), new_scope);
                self.cur_spot = ScopeSpot::Mid(sid);
            }
            ScopeSpot::Local => {
                panic!("No mod in local scope spot!");
            }
        }
    }

    pub fn enter_fn(&mut self, name: impl Into<String>) {
        match &self.cur_spot {
            ScopeSpot::Global => {
                // Create a new scope under global
                let sid = Sid::global(name);
                let new_scope = Scope::new(ScopeKind::Fn, sid.clone());
                self.scopes.insert(sid.clone(), new_scope);
                self.cur_spot = ScopeSpot::Mid(sid);
            }
            ScopeSpot::Mid(cur_sid) => {
                let sid = Sid::kid_of(cur_sid, name);
                let new_scope = Scope::new(ScopeKind::Fn, sid.clone());
                self.scopes.insert(sid.clone(), new_scope);
                self.cur_spot = ScopeSpot::Mid(sid);
            }
            ScopeSpot::Local => {
                panic!("No fn in local scope spot!");
            }
        }
    }

    pub fn enter_type(&mut self, name: impl Into<String>) {
        match &self.cur_spot {
            ScopeSpot::Global => {
                // Create a new scope under global
                let sid = Sid::global(name);
                let new_scope = Scope::new(ScopeKind::Type, sid.clone());
                self.scopes.insert(sid.clone(), new_scope);
                self.cur_spot = ScopeSpot::Mid(sid);
            }
            ScopeSpot::Mid(cur_sid) => {
                let sid = Sid::kid_of(cur_sid, name);
                let new_scope = Scope::new(ScopeKind::Type, sid.clone());
                self.scopes.insert(sid.clone(), new_scope);
                self.cur_spot = ScopeSpot::Mid(sid);
            }
            ScopeSpot::Local => {
                panic!("No type in local scope spot!");
            }
        }
    }

    pub fn enter_scope(&mut self) {
        self.stack.push(StackedScope::new());
    }

    pub fn exit_mod(&mut self) {

    }

    pub fn exit_fn(&mut self) {

    }

    pub fn exit_type(&mut self) {

    }

    pub fn exit_scope(&mut self) {
        if self.stack.len() > 1 {
            self.stack.pop();
        }
    }

    pub fn current_scope(&self) -> &StackedScope {
        self.stack.last().expect("No scope left")
    }

    pub fn current_scope_mut(&mut self) -> &mut StackedScope {
        self.stack.last_mut().expect("No scope left")
    }

    pub fn global_scope(&self) -> &StackedScope {
        self.stack.first().expect("No global scope left")
    }

    pub fn global_scope_mut(&mut self) -> &mut StackedScope {
        self.stack.first_mut().expect("No global scope left")
    }

    pub fn set_local_val(&mut self, name: &str, value: Value) {
        self.current_scope_mut().set_val(name, value);
    }

    pub fn set_local_obj(&mut self, obj: &Obj) {
        // TODO: too much clone
        for key in obj.keys() {
            let val = obj.get(key.clone());
            if let Some(v) = val {
                self.current_scope_mut().set_val(key.to_string().as_str(), v);
            }
        }
    }

    pub fn set_global(&mut self, name: impl Into<String>, value: Value) {
        self.global_scope_mut().set_val(name, value);
    }

    pub fn add_global_fn(&mut self, name: &str, f: fn(&Args) -> Value) {
        self.global_scope_mut().set_val(name, Value::ExtFn(ExtFn { fun: f, name: name.to_string() }));
    }

    pub fn get_global(&self, name: &str) -> Value {
        self.global_scope().get_val(name).unwrap_or(Value::Nil)
    }

    pub fn define(&mut self, name: &str, meta: Rc<Meta>) {
        self.current_scope_mut().put_symbol(name, meta);
    }

    pub fn define_env(&mut self, name: &str, val: Box<dyn Any>) {
        self.env_vals.insert(name.to_string(), val);
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

    pub fn exists(&self, name: &str) -> bool {
        // check for symbols
        for scope in self.stack.iter().rev() {
            if scope.exists(name) {
                return true;
            }
        }

        // check for builtins
        let is_builtin = self.builtins.contains_key(name);
        is_builtin
    }


    pub fn lookup_val(&self, name: &str) -> Option<Value> {
        for scope in self.stack.iter().rev() {
            if let Some(value) = scope.get_val(name) {
                return Some(value);
            }
        }
        self.builtins.get(name).cloned()
    }

    pub fn update_obj(&mut self, name: &str, f: impl FnOnce(&mut Obj)) {
        for scope in self.stack.iter_mut().rev() {
            if let Some(value) = scope.get_val_mut(name) {
                if let Value::Obj(o) = value {
                    f(o);
                    return;
                }
            }
        }
    }

    pub fn update_array(&mut self, name: &str, idx: Value, val: Value) {
        for scope in self.stack.iter_mut().rev() {
            if let Some(value) = scope.get_val_mut(name) {
                if let Value::Array(a) = value {
                    match idx {
                        Value::Int(i) => a[i as usize] = val,
                        Value::Uint(i) => a[i as usize] = val,
                        _ => {}
                    }
                    return;
                }
            }
        }
    }

    pub fn lookup_val_mut(&mut self, name: &str) -> Option<&mut Value> {
        for scope in self.stack.iter_mut().rev() {
            if let Some(value) = scope.get_val_mut(name) {
                return Some(value);
            }
        }
        None
    }

    pub fn update_val(&mut self, name: &str, value: Value) {
        for scope in self.stack.iter_mut().rev() {
            if scope.exists(name) {
                scope.set_val(name, value);
                return;
            }
        }
    }

    pub fn lookup_meta(&self, name: &str) -> Option<Rc<Meta>> {
        for scope in self.stack.iter().rev() {
            if let Some(meta) = scope.get_symbol(name) {
                return Some(meta.clone());
            }
        }
        None
    }

    pub fn lookup_sig(&self, sig: &Sig) -> Option<Rc<Meta>> {
        self.lookup_meta(&sig.name)
    }

    pub fn lookup_builtin(&self, name: &str) -> Option<Value> {
        self.builtins.get(name).cloned()
    }

    // TODO: return a RC of view instead of clone
    pub fn lookup_view(&self, id: &MetaID) -> Option<ast::View> {
        match id {
            MetaID::View(viewid) => {
                let meta = self.lookup_meta(viewid);
                match meta {
                    Some(meta) => match meta.as_ref() {
                        Meta::View(view) => Some(view.clone()),
                        _ => None,
                    }
                    None => None,
                }
            }
            MetaID::Body(bodyid) => {
                let meta = self.lookup_meta(bodyid);
                match meta {
                    Some(meta) => match meta.as_ref() {
                        Meta::Body(body) => Some(Self::body_to_view(body)),
                        _ => None,
                    }
                    None => None,
                }
            }
            _ => None,
        }
    }

    pub fn body_to_view(body: &ast::Body) -> ast::View {
        let mut view = ast::View::default();
        // view.body = body.clone();
        for stmt in body.stmts.iter() {
            match stmt {
                ast::Stmt::Node(node) => {
                    view.nodes.push((node.name.clone(), node.clone()));
                }
                ast::Stmt::Expr(ast::Expr::Call(call)) => {
                    let call = call.clone();
                    let node: ast::Node = call.into();
                    view.nodes.push((node.name.clone(), node));
                }
                _ => (),
            }
        }
        view
    }
}

#[derive(Debug)]
pub enum Meta {
    Store(ast::Store),
    Var(ast::Var),
    Ref(ast::Name),
    Fn(ast::Fn),
    Type(ast::Type),
    Widget(ast::Widget),
    View(ast::View),
    Body(ast::Body),
}

impl fmt::Display for Meta {

    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Meta::Store(_) => write!(f, "STORE"),
            Meta::Var(_) => write!(f, "VAR"),
            Meta::Ref(_) => write!(f, "REF"),
            Meta::Fn(_) => write!(f, "FN"),
            Meta::Type(_) => write!(f, "TYPE"),
            Meta::Widget(_) => write!(f, "Widget"),
            Meta::View(_) => write!(f, "VIEW"),
            Meta::Body(_) => write!(f, "BoDY"),
        }
    }
}

pub struct StackedScope {
    pub vals: HashMap<String, Value>,
    pub symbols: HashMap<String, Rc<Meta>>,
}

impl StackedScope {
    pub fn new() -> StackedScope {
        StackedScope { vals: HashMap::new(), symbols: HashMap::new() }
    }

    pub fn dump(&self) {
        println!("Vals: {:?}", self.vals);
        println!("Symbols: {:?}", self.symbols);
    }

    pub fn set_val(&mut self, name: impl Into<String>, value: Value) {
        self.vals.insert(name.into(), value);
    }

    pub fn get_val(&self, name: &str) -> Option<Value> {
        self.vals.get(name).cloned()
    }

    pub fn get_val_mut(&mut self, name: &str) -> Option<&mut Value> {
        self.vals.get_mut(name)
    }

    pub fn put_symbol(&mut self, name: &str, meta: Rc<Meta>) {
        self.symbols.insert(name.to_string(), meta);
    }

    pub fn get_symbol(&self, name: &str) -> Option<&Rc<Meta>> {
        self.symbols.get(name)
    }

    pub fn exists(&self, name: &str) -> bool {
        self.symbols.contains_key(name) || self.vals.contains_key(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sid() {
        let sid = Sid::new("std.math");
        assert_eq!(sid.parent(), Sid::new("std"));

        assert_eq!(sid.name(), "math");
    }

}

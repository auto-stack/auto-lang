use std::collections::HashMap;
use auto_val::{Value, Args};
use auto_val::Sig;
use auto_val::MetaID;
use auto_atom::Atom;
use crate::ast;
use crate::libs;
use std::rc::Rc;
use std::cell::RefCell;
use auto_val::{TypeInfoStore, ExtFn, Obj};
use std::any::Any;
use std::fmt;
use std::sync::LazyLock;
use ecow::EcoString as AutoStr;

static SID_PATH_GLOBAL: LazyLock<Sid> = LazyLock::new(|| Sid::new(""));

pub enum ScopeKind {
    Global,
    Mod,
    Type,
    Fn,
    Block,
}

impl fmt::Display for ScopeKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ScopeKind::Global => write!(f, "GlobalScope"),
            ScopeKind::Mod => write!(f, "ModScope"),
            ScopeKind::Type => write!(f, "TypeScope"),
            ScopeKind::Fn => write!(f, "FnScope"),
            ScopeKind::Block => write!(f, "BlockScope"),
        }
    }
}

// TODO: Sid should be a Sharable object, with cheap cloning, like SharedString
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Sid {
    path: AutoStr,
}

impl fmt::Display for Sid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.path)
    }
}

impl From<String> for Sid {
    fn from(value: String) -> Self {
        Self {
            path: AutoStr::from(value)
        }
    }
}

impl From<AutoStr> for Sid {
    fn from(value: AutoStr) -> Self {
        Self {
            path: value
        }
    }
}

impl From<&str> for Sid {
    fn from(value: &str) -> Self {
        Self {
            path: AutoStr::from(value)
        }
    }
}

impl Sid {
    pub fn new(path: impl Into<String>) -> Self {
        Self {
            path: AutoStr::from(path.into())
        }
    }

    pub fn kid_of(parent: &Sid, name: impl Into<String>) -> Self {
        Self {
            path: if parent.is_global() {
                AutoStr::from(name.into())
            } else {
                AutoStr::from(format!("{}.{}", parent.path, name.into()))
            }
        }
    }

    pub fn top(name: impl Into<String>) -> Self {
        Self {
            path: AutoStr::from(name.into())
        }
    }

    pub fn parent(&self) -> Option<Self> {
        if let Some(pos) = self.path.rfind('.') {
            Some(Self { path: AutoStr::from(self.path[0..pos].to_string()) })
        } else if self.path == SID_PATH_GLOBAL.path {
            None
        } else {
            Some(SID_PATH_GLOBAL.clone())
        }
    }

    pub fn name(&self) -> AutoStr {
        if let Some(pos) = self.path.rfind('.') {
            self.path[pos+1..].into()
        } else {
            self.path.clone()
        }
    }

    pub fn is_global(&self) -> bool {
        self.path == ""
    }
}

pub struct Scope {
    pub kind: ScopeKind,
    pub sid: Sid, // TODO: should use SharedString?
    pub parent: Option<Sid>, // sid to parent
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

    pub fn get_symbol(&self, name: &str) -> Option<Rc<Meta>> {
        self.symbols.get(name).cloned()
    }

    pub fn exists(&self, name: &str) -> bool {
        self.symbols.contains_key(name) || self.vals.contains_key(name)
    }
}

pub struct Universe {
    pub scopes: HashMap<Sid, Scope>, // sid -> scope
    pub asts: HashMap<Sid, ast::Code>, // sid -> ast
    // pub stack: Vec<StackedScope>,
    pub env_vals: HashMap<String, Box<dyn Any>>,
    pub shared_vals: HashMap<String, Rc<RefCell<Value>>>,
    pub builtins: HashMap<String, Value>, // Value of builtin functions
    pub types: TypeInfoStore,
    pub args: Obj,
    lambda_counter: usize,
    cur_spot: Sid,
    pub widget: Value,
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
        scopes.insert(SID_PATH_GLOBAL.clone(), Scope::new(ScopeKind::Global, SID_PATH_GLOBAL.clone()));
        let mut uni = Self {
            scopes,
            asts: HashMap::new(),
            // stack: vec![StackedScope::new()],
            env_vals: HashMap::new(),
            shared_vals: HashMap::new(),
            builtins, 
            types: TypeInfoStore::new(), 
            lambda_counter: 0,
            cur_spot: SID_PATH_GLOBAL.clone(),
            widget: Value::Nil,
            args: Obj::new(),
        };
        uni.define_sys_types();
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
        // for scope in self.stack.iter() {
        //     scope.dump();
        // }
        for (name, meta) in self.builtins.iter() {
            println!("Builtin: {} = {}", name, meta);
        }
    }

    pub fn chart(&self) -> String {
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
        chart
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

    fn enter_named_scope(&mut self, name: impl Into<String>, kind: ScopeKind) {
        // Create a new scope under Global
        let new_sid = Sid::kid_of(&self.cur_spot, name);
        let new_scope = Scope::new(kind, new_sid.clone());
        self.cur_scope_mut().kids.push(new_sid.clone());
        self.scopes.insert(new_sid.clone(), new_scope);
        self.cur_spot = new_sid;
    }

    pub fn enter_mod(&mut self, name: impl Into<String>) {
        self.enter_named_scope(name, ScopeKind::Mod);
    }

    pub fn enter_fn(&mut self, name: impl Into<String>) {
        self.enter_named_scope(name, ScopeKind::Fn);
    }

    pub fn enter_type(&mut self, name: impl Into<String>) {
        self.enter_named_scope(name, ScopeKind::Type);
    }
    
    pub fn cur_scope(&self) -> &Scope {
        self.scopes.get(&self.cur_spot).unwrap()
    }

    pub fn cur_scope_mut(&mut self) -> &mut Scope {
        self.scopes.get_mut(&self.cur_spot).unwrap()
    }

    pub fn enter_scope(&mut self) {
        let name = format!("block_{}", self.cur_scope().kids.len());
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

    pub fn current_scope(&self) -> &Scope {
        self.scopes.get(&self.cur_spot).expect("No scope left")
    }

    pub fn current_scope_mut(&mut self) -> &mut Scope {
        self.scopes.get_mut(&self.cur_spot).expect("No scope left")
    }

    pub fn global_scope(&self) -> &Scope {
        self.scopes.get(&SID_PATH_GLOBAL).expect("No global scope left")
    }

    pub fn global_scope_mut(&mut self) -> &mut Scope {
        self.scopes.get_mut(&SID_PATH_GLOBAL).expect("No global scope left")
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

    pub fn set_shared(&mut self, name: &str, value: Rc<RefCell<Value>>) {
        self.shared_vals.insert(name.to_string(), value);
    }

    pub fn get_shared(&self, name: &str) -> Option<Rc<RefCell<Value>>> {
        self.shared_vals.get(name).cloned()
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

    fn lookup_val_recurse(&self, name: &str, sid: &Sid) -> Option<Value> {
        if let Some(scope) = self.scopes.get(sid) {
            let val = scope.get_val(name);
            if let Some(val) = val {
                return Some(val);
            }
        }
        if let Some(parent) = sid.parent() {
            return self.lookup_val_recurse(name, &parent);
        }
        None
    }

    pub fn lookup_val(&self, name: &str) -> Option<Value> {
        if let Some(val) = self.lookup_val_recurse(name, &self.cur_spot) {
            return Some(val);
        }
        let shared = self.shared_vals.get(name);
        if let Some(shared) = shared {
            return Some(shared.borrow().clone());
        }
        self.builtins.get(name).cloned()
    }

    fn update_obj_recurse(&mut self, name: &str, f: impl FnOnce(&mut Obj)) {
        if let Some(value) = self.lookup_val_mut(name) {
            if let Value::Obj(o) = value {
                f(o);
                return;
            }
        }
    }

    pub fn update_obj(&mut self, name: &str, f: impl FnOnce(&mut Obj)) {
        self.update_obj_recurse(name, f);
    }

    fn update_array_recurse(&mut self, name: &str, idx: Value, val: Value) {
        if let Some(value) = self.lookup_val_mut(name) {
            if let Value::Array(a) = value {
                match idx {
                    Value::Int(i) => a[i as usize] = val,
                    Value::Uint(i) => a[i as usize] = val,
                    _ => {}
                }
            }
        }
    }

    pub fn update_array(&mut self, name: &str, idx: Value, val: Value) {
        self.update_array_recurse(name, idx, val);
    }

    fn lookup_val_mut_recurse(&mut self, name: &str, sid: &Sid) -> Option<&mut Value> {
        if !self.scopes.contains_key(sid) {
            if let Some(parent) = sid.parent() {
                return self.lookup_val_mut_recurse(name, &parent);
            }
        }
        if let Some(scope) = self.scopes.get_mut(sid) {
            return scope.get_val_mut(name);
        }
        None
    }

    pub fn lookup_val_mut(&mut self, name: &str) -> Option<&mut Value> {
        let sid = self.cur_spot.clone();
        self.lookup_val_mut_recurse(name, &sid)
    }

    fn update_val_recurse(&mut self, name: &str, value: Value, sid: &Sid) {
        if let Some(scope) = self.scopes.get_mut(sid) {
            if scope.exists(name) {
                scope.set_val(name, value);
                return;
            }
        }
        if let Some(parent) = sid.parent() {
            self.update_val_recurse(name, value, &parent);
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

    pub fn define_var(&mut self, name: &str, expr: ast::Expr) {
        // Add meta to current scope
        let ast_name = ast::Name::new(name.to_string());
        let store = ast::Store {
            kind: ast::StoreKind::Var,
            name: ast_name,
            ty: ast::Type::Int,
            expr: expr,
        };
        self.define(name, Rc::new(Meta::Store(store)));
    }

    pub fn import(&mut self, path: AutoStr, ast: ast::Code) {
        let sid = Sid::new(path.as_str());
        self.asts.insert(sid, ast);
    }

    pub fn widget(&self) -> Value {
        self.widget.clone()
    }

    // TODO: support nested nodes
    pub fn merge_atom(&mut self, atom: &Atom) {
        match &atom.root {
            auto_atom::Root::Node(node) => {
                let main_arg = node.main_arg();
                self.set_global("name", main_arg);
                for (key, val) in node.props.iter() {
                    self.set_global(key.to_string(), val.clone());
                }
                // set kids
                let kids_groups = node.group_kids();
                for (name, kids) in kids_groups.iter() {
                    let plural = format!("{}s", name);
                    // for each kid, set its main arg as `id`, and all props as is
                    let mut kids_vec: Vec<Value> = Vec::new();
                    for kid in kids.iter() {
                        let mut props = kid.props.clone();
                        props.set("name", kid.main_arg());
                        kids_vec.push(props.into());
                    }
                    self.set_global(plural.as_str(), kids_vec.into());
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
}

#[derive(Debug)]
pub enum Meta {
    Store(ast::Store),
    Pair(ast::Pair),
    Var(ast::Var),
    Ref(ast::Name),
    Fn(ast::Fn),
    Type(ast::Type),
    Widget(ast::Widget),
    View(ast::View),
    Body(ast::Body),
    Use(String),
    Node(ast::Node),
}

impl fmt::Display for Meta {

    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Meta::Store(_) => write!(f, "STORE"),
            Meta::Pair(_) => write!(f, "PAIR"),
            Meta::Var(_) => write!(f, "VAR"),
            Meta::Ref(_) => write!(f, "REF"),
            Meta::Fn(_) => write!(f, "FN"),
            Meta::Type(_) => write!(f, "TYPE"),
            Meta::Widget(_) => write!(f, "Widget"),
            Meta::View(_) => write!(f, "VIEW"),
            Meta::Body(_) => write!(f, "BoDY"),
            Meta::Node(_) => write!(f, "NODE"),
            Meta::Use(name) => write!(f, "USE {}", name),
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

    pub fn dump(&self) -> String {
        let mut chart = String::new();
        chart.push_str(&format!("Vals: {:?}\n", self.vals));
        chart.push_str(&format!("Symbols: {:?}\n", self.symbols));
        chart
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
        assert_eq!(sid.parent().unwrap(), Sid::new("std"));

        assert_eq!(sid.name(), "math");
    }

    #[test]
    fn test_scope_enter_and_exit() {
        let mut uni = Universe::new();
        uni.enter_mod("std");
        assert_eq!(uni.cur_spot, Sid::new("std"));
        uni.enter_fn("math");
        assert_eq!(uni.cur_spot, Sid::new("std.math"));
        uni.enter_type("Matrix");
        assert_eq!(uni.cur_spot, Sid::new("std.math.Matrix"));
        uni.enter_scope();
        assert_eq!(uni.cur_spot, Sid::new("std.math.Matrix.block_0"));
        uni.exit_scope();
        assert_eq!(uni.cur_spot, Sid::new("std.math.Matrix"));
        uni.exit_scope();
        assert_eq!(uni.cur_spot, Sid::new("std.math"));
        uni.exit_scope();
        assert_eq!(uni.cur_spot, Sid::new("std"));
        uni.exit_scope();
        assert_eq!(uni.cur_spot, *SID_PATH_GLOBAL);
    }

    #[test]
    fn test_scope_define_and_lookup() {
        let mut uni = Universe::new();
        uni.enter_mod("std");
        uni.enter_mod("math");
        let val_expr = ast::Expr::Int(32);
        uni.define_var("a", val_expr);
        uni.enter_fn("add");
        let meta = uni.lookup_meta("a");
        // TODO: Meta destructureing is a mess
        let mut succ = false;
        if let Some(meta) = meta {
            if let Meta::Store(store) = meta.as_ref() {
                if let ast::Expr::Int(32) = store.expr {
                    succ = true;
                }
            }
        }
        assert!(succ);

        uni.set_local_val("x", Value::Int(12));
        let val = uni.lookup_val("x");
        assert_eq!(val, Some(Value::Int(12)));
    }
}

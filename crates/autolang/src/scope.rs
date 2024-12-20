use std::collections::HashMap;
use autoval::{Value, Args};
use autoval::Sig;
use autoval::MetaID;
use crate::ast;
use crate::ast::Call;
use crate::libs;
use std::rc::Rc;
use autoval::{TypeInfoStore, ExtFn, Obj};
use std::any::Any;
use std::fmt;

pub struct Universe {
    pub scopes: Vec<Scope>,
    pub builtins: HashMap<String, Value>,
    pub evn_vals: HashMap<String, Box<dyn Any>>,
    pub types: TypeInfoStore,
    pub widget: Value,
    lambda_counter: usize,
}

impl Default for Universe {
    fn default() -> Self {
        Self::new()
    }
}

impl Universe {
    pub fn new() -> Universe {
        let builtins = libs::builtin::builtins();
        let mut uni = Universe { scopes: vec![Scope::new()], builtins, evn_vals: HashMap::new(), types: TypeInfoStore::new(), widget: Value::Nil, lambda_counter: 0 };
        uni.define_sys_types();
        uni
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

    pub fn dump(&self) {
        for scope in self.scopes.iter() {
            scope.dump();
        }
        for (name, meta) in self.builtins.iter() {
            println!("Builtin: {} = {}", name, meta);
        }
    }

    pub fn enter_scope(&mut self) {
        self.scopes.push(Scope::new());
    }

    pub fn exit_scope(&mut self) {
        if self.scopes.len() > 1 {
            self.scopes.pop();
        }
    }

    pub fn current_scope(&self) -> &Scope {
        self.scopes.last().expect("No scope left")
    }

    pub fn current_scope_mut(&mut self) -> &mut Scope {
        self.scopes.last_mut().expect("No scope left")
    }

    pub fn global_scope(&self) -> &Scope {
        self.scopes.first().expect("No global scope left")
    }

    pub fn global_scope_mut(&mut self) -> &mut Scope {
        self.scopes.first_mut().expect("No global scope left")
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
        self.evn_vals.insert(name.to_string(), val);
    }

    pub fn get_env(&self, name: &str) -> Option<&Box<dyn Any>> {
        self.evn_vals.get(name)
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
        for scope in self.scopes.iter().rev() {
            if scope.exists(name) {
                return true;
            }
        }

        // check for builtins
        let is_builtin = self.builtins.contains_key(name);
        is_builtin
    }


    pub fn lookup_val(&self, name: &str) -> Option<Value> {
        for scope in self.scopes.iter().rev() {
            if let Some(value) = scope.get_val(name) {
                return Some(value);
            }
        }
        self.builtins.get(name).cloned()
    }

    pub fn update_obj(&mut self, name: &str, f: impl FnOnce(&mut Obj)) {
        for scope in self.scopes.iter_mut().rev() {
            if let Some(value) = scope.get_val_mut(name) {
                if let Value::Obj(o) = value {
                    f(o);
                    return;
                }
            }
        }
    }

    pub fn update_array(&mut self, name: &str, idx: Value, val: Value) {
        for scope in self.scopes.iter_mut().rev() {
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
        for scope in self.scopes.iter_mut().rev() {
            if let Some(value) = scope.get_val_mut(name) {
                return Some(value);
            }
        }
        None
    }

    pub fn update_val(&mut self, name: &str, value: Value) {
        for scope in self.scopes.iter_mut().rev() {
            if scope.exists(name) {
                scope.set_val(name, value);
                return;
            }
        }
    }

    pub fn lookup_meta(&self, name: &str) -> Option<Rc<Meta>> {
        for scope in self.scopes.iter().rev() {
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

pub struct Scope {
    pub vals: HashMap<String, Value>,
    pub symbols: HashMap<String, Rc<Meta>>,
}

impl Scope {
    pub fn new() -> Scope {
        Scope { vals: HashMap::new(), symbols: HashMap::new() }
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

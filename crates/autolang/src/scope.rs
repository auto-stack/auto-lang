use std::collections::HashMap;
use autoval::value::Value;
use crate::ast;
use crate::libs;
use std::rc::Rc;
pub struct Universe {
    pub scopes: Vec<Scope>,
    pub builtins: HashMap<String, Value>,
}

impl Universe {
    pub fn new() -> Universe {
        let builtins = libs::builtin::builtins();
        let mut uni = Universe { scopes: vec![Scope::new()], builtins };
        uni.define_sys_types();
        uni
    }

    pub fn define_sys_types(&mut self) {
        self.define("int".to_string(), Rc::new(Meta::Type(ast::Type::Int)));
        self.define("float".to_string(), Rc::new(Meta::Type(ast::Type::Float)));
        self.define("bool".to_string(), Rc::new(Meta::Type(ast::Type::Bool)));
        self.define("str".to_string(), Rc::new(Meta::Type(ast::Type::Str)));
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

    pub fn set_local(&mut self, name: &str, value: Value) {
        self.current_scope_mut().set_val(name, value);
    }

    pub fn get_local(&self, name: &str) -> Value {
        self.current_scope().get_val(name).unwrap_or(Value::Nil)
    }

    pub fn set_global(&mut self, name: &str, value: Value) {
        self.global_scope_mut().set_val(name, value);
    }

    pub fn get_global(&self, name: &str) -> Value {
        self.global_scope().get_val(name).unwrap_or(Value::Nil)
    }

    pub fn put_symbol(&mut self, name: &str, meta: Rc<Meta>) {
        self.current_scope_mut().put_symbol(name, meta);
    }

    pub fn get_symbol(&self, name: &str) -> Option<Rc<Meta>> {
        self.current_scope().get_symbol(name).cloned()
    }

    pub fn is_fn(&self, name: &str) -> bool {
        // TODO: check meta if fn
        self.exists(name)
    }

    pub fn exists(&self, name: &str) -> bool {
        // check for vars
        for scope in self.scopes.iter().rev() {
            if scope.exists(name) {
                return true;
            }
        }
        // check for builtins
        let is_builtin = self.builtins.contains_key(name);
        is_builtin
    }

    pub fn define(&mut self, name: String, meta: Rc<Meta>) {
        self.current_scope_mut().put_symbol(name.as_str(), meta);
    }

    pub fn lookup_val(&self, name: &str) -> Option<Value> {
        for scope in self.scopes.iter().rev() {
            if let Some(value) = scope.get_val(name) {
                return Some(value);
            }
        }
        self.builtins.get(name).cloned()
    }

    pub fn lookup_meta(&self, name: &str) -> Option<Rc<Meta>> {
        for scope in self.scopes.iter().rev() {
            if let Some(meta) = scope.get_symbol(name) {
                return Some(meta.clone());
            }
        }
        None
    }

    pub fn lookup_builtin(&self, name: &str) -> Option<Value> {
        self.builtins.get(name).cloned()
    }
}

#[derive(Debug)]
pub enum Meta {
    Var(ast::Var),
    Fn(ast::Fn),
    Type(ast::Type),
    Widget(ast::Widget),
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

    pub fn set_val(&mut self, name: &str, value: Value) {
        self.vals.insert(name.to_string(), value);
    }

    pub fn get_val(&self, name: &str) -> Option<Value> {
        self.vals.get(name).cloned()
    }

    pub fn put_symbol(&mut self, name: &str, meta: Rc<Meta>) {
        self.symbols.insert(name.to_string(), meta);
    }

    pub fn get_symbol(&self, name: &str) -> Option<&Rc<Meta>> {
        self.symbols.get(name)
    }

    pub fn exists(&self, name: &str) -> bool {
        self.symbols.contains_key(name)
    }
}
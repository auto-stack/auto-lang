use crate::ast;
use auto_val::Value;
use ecow::EcoString as AutoStr;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;
use std::sync::LazyLock;

pub static SID_PATH_GLOBAL: LazyLock<Sid> = LazyLock::new(|| Sid::new(""));

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
            path: AutoStr::from(value),
        }
    }
}

impl From<AutoStr> for Sid {
    fn from(value: AutoStr) -> Self {
        Self { path: value }
    }
}

impl From<&str> for Sid {
    fn from(value: &str) -> Self {
        Self {
            path: AutoStr::from(value),
        }
    }
}

impl Sid {
    pub fn new(path: impl Into<String>) -> Self {
        Self {
            path: AutoStr::from(path.into()),
        }
    }

    pub fn kid_of(parent: &Sid, name: impl Into<String>) -> Self {
        Self {
            path: if parent.is_global() {
                AutoStr::from(name.into())
            } else {
                AutoStr::from(format!("{}.{}", parent.path, name.into()))
            },
        }
    }

    pub fn top(name: impl Into<String>) -> Self {
        Self {
            path: AutoStr::from(name.into()),
        }
    }

    pub fn parent(&self) -> Option<Self> {
        if let Some(pos) = self.path.rfind('.') {
            Some(Self {
                path: AutoStr::from(self.path[0..pos].to_string()),
            })
        } else if self.path == SID_PATH_GLOBAL.path {
            None
        } else {
            Some(SID_PATH_GLOBAL.clone())
        }
    }

    pub fn name(&self) -> AutoStr {
        if let Some(pos) = self.path.rfind('.') {
            self.path[pos + 1..].into()
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
    pub sid: Sid,            // TODO: should use SharedString?
    pub parent: Option<Sid>, // sid to parent
    pub kids: Vec<Sid>,
    pub symbols: HashMap<AutoStr, Rc<Meta>>,
    pub types: HashMap<AutoStr, Rc<Meta>>,
    pub vals: HashMap<AutoStr, Value>,
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
            types: HashMap::new(),
            vals: HashMap::new(),
        }
    }

    pub fn is_global(&self) -> bool {
        return matches!(self.kind, ScopeKind::Global);
    }

    pub fn dump(&self) {
        println!("Vals: {:?}", self.vals);
        println!("Symbols: {:?}", self.symbols);
    }

    pub fn set_val(&mut self, name: impl Into<AutoStr>, value: Value) {
        self.vals.insert(name.into(), value);
    }

    pub fn get_val(&self, name: impl Into<AutoStr>) -> Option<Value> {
        self.vals.get(&name.into()).cloned()
    }

    pub fn get_val_mut(&mut self, name: &str) -> Option<&mut Value> {
        self.vals.get_mut(name)
    }

    pub fn put_symbol(&mut self, name: impl Into<AutoStr>, meta: Rc<Meta>) {
        self.symbols.insert(name.into(), meta);
    }

    pub fn get_symbol(&self, name: impl Into<AutoStr>) -> Option<Rc<Meta>> {
        self.symbols.get(&name.into()).cloned()
    }

    pub fn define_type(&mut self, name: impl Into<AutoStr>, meta: Rc<Meta>) {
        let name = name.into();
        self.types.insert(name, meta);
    }

    pub fn lookup_type(&self, name: impl Into<AutoStr>) -> Option<Rc<Meta>> {
        let name = name.into();
        self.types.get(&name).cloned()
    }

    pub fn exists(&self, name: impl Into<AutoStr>) -> bool {
        let name = name.into();
        self.symbols.contains_key(&name) || self.vals.contains_key(&name)
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
    Enum(ast::EnumDecl),
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
            Meta::Enum(_) => write!(f, "ENUM"),
            Meta::Widget(_) => write!(f, "Widget"),
            Meta::View(_) => write!(f, "VIEW"),
            Meta::Body(_) => write!(f, "BoDY"),
            Meta::Node(nd) => write!(f, "{}", nd),
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
        StackedScope {
            vals: HashMap::new(),
            symbols: HashMap::new(),
        }
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
        let mut uni = crate::Universe::new();
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
        let mut uni = crate::Universe::new();
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

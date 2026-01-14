use crate::ast;
use auto_val::{Value, ValueID};
use ecow::EcoString as AutoStr;
use std::collections::{HashMap, HashSet};
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
        if self.path.is_empty() {
            write!(f, "🌳")
        } else {
            write!(f, "{}", self.path)
        }
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
    pub cur_block: usize,
    pub symbols: HashMap<AutoStr, Rc<Meta>>,
    pub types: HashMap<AutoStr, Rc<Meta>>,
    pub vals: HashMap<AutoStr, ValueID>,  // CHANGED: Now stores ValueID instead of Value
    pub moved_vars: HashSet<AutoStr>,     // Track moved variables for ownership semantics
}

impl Scope {
    pub fn new(kind: ScopeKind, sid: Sid) -> Self {
        let parent = sid.parent();
        Self {
            kind,
            sid,
            parent,
            kids: Vec::new(),
            cur_block: 0,
            symbols: HashMap::new(),
            types: HashMap::new(),
            vals: HashMap::new(),
            moved_vars: HashSet::new(),
        }
    }

    pub fn is_global(&self) -> bool {
        return matches!(self.kind, ScopeKind::Global);
    }

    pub fn dump(&self) {
        // println!("Vals: {:?}", self.vals); // LSP: disabled
        // println!("Symbols: {:?}", self.symbols); // LSP: disabled
    }

    pub fn set_val(&mut self, name: impl Into<AutoStr>, vid: ValueID) {
        self.vals.insert(name.into(), vid);
    }

    pub fn get_val(&self, _name: impl Into<AutoStr>) -> Option<Value> {
        // TODO: This needs to resolve the ValueID to actual Value
        // For now, return None - this will be updated when Universe is connected
        None
    }

    /// Get value ID (NEW)
    pub fn get_val_id(&self, name: impl Into<AutoStr>) -> Option<ValueID> {
        self.vals.get(&name.into()).copied()
    }

    /// Check if a value exists in this scope
    pub fn has_val(&self, name: impl Into<AutoStr>) -> bool {
        self.vals.contains_key(&name.into())
    }

    /// Remove a value from this scope, returning its ValueID
    /// Returns None if the value doesn't exist
    pub fn remove_val(&mut self, name: impl Into<AutoStr>) -> Option<ValueID> {
        self.vals.remove(&name.into())
    }

    /// Mark a variable as moved (use-after-move prevention)
    pub fn mark_moved(&mut self, name: impl Into<AutoStr>) {
        self.moved_vars.insert(name.into());
    }

    /// Check if a variable has been moved
    pub fn is_moved(&self, name: impl Into<AutoStr>) -> bool {
        self.moved_vars.contains(&name.into())
    }

    /// Clear moved status (used when variable is reassigned)
    pub fn clear_moved(&mut self, name: impl Into<AutoStr>) {
        self.moved_vars.remove(&name.into());
    }

    // REMOVED: get_val_mut - no longer needed with reference-based system

    pub fn put_symbol(&mut self, name: impl Into<AutoStr>, meta: Rc<Meta>) {
        self.symbols.insert(name.into(), meta);
    }

    pub fn get_symbol(&self, name: impl Into<AutoStr>) -> Option<Rc<Meta>> {
        self.symbols.get(&name.into()).cloned()
    }

    pub fn define_alias(&mut self, alias: AutoStr, target: AutoStr) {
        self.symbols
            .insert(alias.into(), Rc::new(Meta::Alias(target.into())));
    }

    pub fn define_type(&mut self, name: impl Into<AutoStr>, meta: Rc<Meta>) {
        let name = name.into();
        self.types.insert(name, meta);
        // println!("types: {:?}", self.types);
    }

    pub fn lookup_type(&self, name: impl Into<AutoStr>) -> Option<Rc<Meta>> {
        let name = name.into();
        // println!("Checking type {}", name);
        // println!("from: {:?}", self.types);
        // println!("found: {}", self.types.contains_key(&name));
        self.types.get(&name).cloned()
    }

    pub fn exists(&self, name: impl Into<AutoStr>) -> bool {
        let name = name.into();
        self.symbols.contains_key(&name)
            || self.vals.contains_key(&name)
            || self.types.contains_key(&name)
    }
}

#[derive(Debug)]
pub enum Meta {
    Store(ast::Store),
    Pair(ast::Pair),
    // Var(ast::Var),
    Ref(ast::Name),
    Fn(ast::Fn),
    Type(ast::Type),
    Enum(ast::EnumDecl),
    Spec(ast::SpecDecl),
    Body(ast::Body),
    Use(String),
    Node(ast::Node),
    Alias(AutoStr),
}

impl fmt::Display for Meta {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Meta::Store(_) => write!(f, "STORE"),
            Meta::Pair(_) => write!(f, "PAIR"),
            // Meta::Var(_) => write!(f, "VAR"),
            Meta::Ref(_) => write!(f, "REF"),
            Meta::Fn(_) => write!(f, "FN"),
            Meta::Type(_) => write!(f, "TYPE"),
            Meta::Enum(_) => write!(f, "ENUM"),
            Meta::Spec(spec) => write!(f, "SPEC {}", spec.name),
            Meta::Body(_) => write!(f, "BoDY"),
            Meta::Node(nd) => write!(f, "{}", nd),
            Meta::Use(name) => write!(f, "USE {}", name),
            Meta::Alias(alias) => write!(f, "ALIAS {}", alias),
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
    use auto_val::ValueData;

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

        // Test 1: Define and lookup metadata
        let val_expr = ast::Expr::Int(32);
        uni.define_var("a", val_expr);
        uni.enter_fn("add");

        let meta = uni.lookup_meta("a");
        assert!(matches!(meta.as_deref(), Some(Meta::Store(s)) if matches!(s.expr, ast::Expr::Int(32))));

        // Test 2: Set and lookup value with ValueRef resolution
        uni.set_local_val("x", Value::Int(12));
        let val = uni.lookup_val("x");

        // Resolve ValueRef and check the actual value data
        match val {
            Some(Value::ValueRef(vid)) => {
                let data = uni.get_value(vid).unwrap();
                assert!(matches!(*data.borrow(), ValueData::Int(12)));
            }
            other => panic!("Expected ValueRef, got {:?}", other),
        }
    }
}

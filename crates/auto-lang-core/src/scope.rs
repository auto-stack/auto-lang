use crate::ast;
use auto_val::{Value, ValueID};
use ecow::EcoString as AutoStr;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::rc::Rc;
use std::sync::LazyLock;

pub static SID_PATH_GLOBAL: LazyLock<Sid> = LazyLock::new(|| Sid::new(""));

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

// =============================================================================
// SymbolTable: Compile-time symbol table (Plan 064 Phase 4)
// =============================================================================

/// Compile-time symbol table (persistent)
///
/// Contains static declaration information: types, symbols,
/// scope hierarchy. Used by parser, indexer, type checker,
/// and transpilers. Stored in AIE Database.
///
/// # Architecture (Plan 064 Phase 4)
///
/// ```text
/// Compile-time (Database)       Runtime (ExecutionEngine)
/// ┌──────────────────┐         ┌──────────────────┐
/// │   SymbolTable    │◄────────│   StackFrame      │
/// │ - kind, sid      │  link   │ - scope_sid       │
/// │ - symbols, types │         │ - vals            │
/// └──────────────────┘         │ - moved_vars      │
///                              └──────────────────┘
/// ```
///
/// # Purpose
///
/// - **Parser**: Records symbol and type declarations
/// - **Indexer**: Builds scope hierarchy
/// - **Type checker**: Resolves type references
/// - **Transpilers**: Generates backend code
///
/// # Key Difference from Scope
///
/// `SymbolTable` is **compile-time only** - it contains NO runtime values.
/// Runtime values live in `StackFrame` (in ExecutionEngine).
///
/// This separation enables:
/// - Incremental compilation (SymbolTables persist)
/// - Hot reloading (runtime state is separate)
/// - Recursive functions (multiple StackFrames → one SymbolTable)
#[derive(Debug, Clone)]
pub struct SymbolTable {
    /// Scope kind (global, function, block, etc.)
    pub kind: ScopeKind,

    /// Unique scope identifier
    pub sid: Sid,

    /// Parent scope reference (for hierarchy)
    pub parent: Option<Sid>,

    /// Child scope references
    pub kids: Vec<Sid>,

    /// Symbol declarations (functions, variables, etc.)
    pub symbols: HashMap<AutoStr, Rc<Meta>>,

    /// Type declarations
    pub types: HashMap<AutoStr, Rc<Meta>>,
}

impl SymbolTable {
    /// Create a new symbol table
    pub fn new(kind: ScopeKind, sid: Sid) -> Self {
        let parent = sid.parent();
        Self {
            kind,
            sid,
            parent,
            kids: Vec::new(),
            symbols: HashMap::new(),
            types: HashMap::new(),
        }
    }

    /// Check if this is the global scope
    pub fn is_global(&self) -> bool {
        matches!(self.kind, ScopeKind::Global)
    }

    /// Get the scope name
    pub fn name(&self) -> AutoStr {
        self.sid.name()
    }

    /// Add a child scope
    pub fn add_kid(&mut self, kid_sid: Sid) {
        self.kids.push(kid_sid);
    }

    /// Insert a symbol declaration
    pub fn insert_symbol(&mut self, name: AutoStr, meta: Rc<Meta>) {
        self.symbols.insert(name, meta);
    }

    /// Get a symbol declaration
    pub fn get_symbol(&self, name: &str) -> Option<&Rc<Meta>> {
        self.symbols.get(name)
    }

    /// Insert a type declaration
    pub fn insert_type(&mut self, name: AutoStr, meta: Rc<Meta>) {
        self.types.insert(name, meta);
    }

    /// Get a type declaration
    pub fn get_type(&self, name: &str) -> Option<&Rc<Meta>> {
        self.types.get(name)
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
    // Plan 121: Task/Msg system
    Task(ast::TaskDef),
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
            Meta::Task(task) => write!(f, "TASK {}", task.name),
        }
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
    fn test_symbol_table_name() {
        let sid = Sid::from("std.math");
        let table = SymbolTable::new(ScopeKind::Mod, sid);
        assert_eq!(table.name(), "math");
    }

    #[test]
    fn test_symbol_table_parent() {
        let sid = Sid::from("std.math");
        let table = SymbolTable::new(ScopeKind::Mod, sid);

        // Parent should be automatically set from Sid
        assert_eq!(table.parent, Some(Sid::from("std")));
    }

    #[test]
    fn test_symbol_table_symbols() {
        let mut table = SymbolTable::new(ScopeKind::Fn, Sid::from("test_fn"));

        // Insert and retrieve symbol
        let meta = Rc::new(Meta::Store(ast::Store {
            kind: ast::StoreKind::Let,
            attrs: vec![],
            is_pub: false,
            name: AutoStr::from("x"),
            ty: ast::Type::Int,
            expr: ast::Expr::Int(42),
        }));
        table.insert_symbol(AutoStr::from("x"), meta.clone());

        let retrieved = table.get_symbol("x");
        assert!(retrieved.is_some());
        assert!(Rc::ptr_eq(retrieved.unwrap(), &meta));

        // Non-existent symbol
        assert!(table.get_symbol("y").is_none());
    }

    #[test]
    fn test_symbol_table_types() {
        let mut table = SymbolTable::new(ScopeKind::Type, Sid::from("MyType"));

        // Insert and retrieve type
        let meta = Rc::new(Meta::Type(ast::Type::Int));
        table.insert_type(AutoStr::from("Int"), meta.clone());

        let retrieved = table.get_type("Int");
        assert!(retrieved.is_some());
        assert!(Rc::ptr_eq(retrieved.unwrap(), &meta));

        // Non-existent type
        assert!(table.get_type("Float").is_none());
    }

    #[test]
    fn test_symbol_table_kids() {
        let mut table = SymbolTable::new(ScopeKind::Mod, Sid::from("std"));

        // Add child scopes
        table.add_kid(Sid::from("std.math"));
        table.add_kid(Sid::from("std.io"));

        assert_eq!(table.kids.len(), 2);
        assert_eq!(table.kids[0], Sid::from("std.math"));
        assert_eq!(table.kids[1], Sid::from("std.io"));
    }

}

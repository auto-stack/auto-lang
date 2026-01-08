mod alias;
pub use alias::*;
mod body;
pub use body::*;
mod branch;
pub use branch::*;
mod call;
pub use call::*;
mod cover;
pub use cover::*;
mod enums;
pub use enums::*;
mod fun;
pub use fun::*;
mod fstr;
pub use fstr::*;
mod grid;
pub use grid::*;
mod if_;
pub use if_::*;
mod is;
pub use is::*;
mod for_;
pub use for_::*;
mod node;
pub use node::*;
mod on;
pub use on::*;
mod store;
pub use store::*;
mod tag;
pub use tag::*;
mod types;
pub use types::*;
mod union;
pub use union::*;
mod use_;
pub use use_::*;
mod range;
pub use range::*;

mod atom_helpers;
pub use atom_helpers::*;

mod parsers;

use auto_val::{AutoStr, Node as AutoNode, Op, Value};
use std::fmt;

pub type Name = AutoStr;

/// Converts AST node to ATOM format Value (for primitive/atomic types)
///
/// # When to Implement ToAtom vs ToNode
///
/// - **ToNode**: For AST types that are naturally represented as nodes
///   with children, properties, and arguments (If, For, Fn, Store, etc.)
///
/// - **ToAtom**: For primitive/atomic types that map to simple values
///   (Type → Value::Str, Key → Value::Int/Bool/Str, Pair → Value::Pair)
///
/// # Example
///
/// ```rust
/// use auto_lang::ast::*;
///
/// let ty = Type::Int;
/// let value = ty.to_atom();  // Returns Value::Str("int")
/// ```
pub trait ToAtom {
    fn to_atom(&self) -> Value;
}

/// Converts AST node to ATOM format Node directly (for complex structures)
///
/// # When to Implement ToNode vs ToAtom
///
/// - **ToNode**: For AST types that are naturally represented as nodes
///   with children, properties, and arguments (If, For, Fn, Store, etc.)
///
/// - **ToAtom**: For primitive/atomic types that map to simple values
///   (Type → Value::Str, Key → Value::Int/Bool/Str, Pair → Value::Pair)
///
/// ```
pub trait ToNode {
    fn to_node(&self) -> AutoNode;
}

#[derive(Debug, Clone)]
pub struct Code {
    pub stmts: Vec<Stmt>,
}

impl Code {
    pub fn new() -> Self {
        Self { stmts: Vec::new() }
    }
}

impl Default for Code {
    fn default() -> Self {
        Self {
            stmts: Vec::default(),
        }
    }
}

impl fmt::Display for Code {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(code ")?;
        let last = self.stmts.len();
        let last = if last > 0 { last - 1 } else { 0 };
        for (i, stmt) in self.stmts.iter().enumerate() {
            write!(f, "{}", stmt)?;
            if i < last {
                write!(f, " ")?;
            }
        }
        write!(f, ")")
    }
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Expr(Expr),
    If(If),
    For(For),
    Is(Is),
    Store(Store),
    Block(Body),
    Fn(Fn),
    EnumDecl(EnumDecl),
    TypeDecl(TypeDecl),
    Union(Union),
    Tag(Tag),
    Node(Node),
    Use(Use),
    OnEvents(OnEvents),
    Comment(AutoStr),
    Alias(Alias),
    EmptyLine(usize),
    Break,
}

impl Stmt {
    pub fn as_fn(&self) -> Option<&Fn> {
        match self {
            Stmt::Fn(fn_decl) => Some(fn_decl),
            _ => None,
        }
    }

    pub fn is_decl(&self) -> bool {
        match self {
            Stmt::Fn(_)
            | Stmt::TypeDecl(_)
            | Stmt::EnumDecl(_)
            | Stmt::Store(_)
            | Stmt::Union(_)
            | Stmt::Tag(_)
            | Stmt::Alias(_) => true,
            _ => false,
        }
    }

    pub fn is_new_block(&self) -> bool {
        match self {
            Stmt::Block(_) | Stmt::Fn(_) | Stmt::TypeDecl(_) | Stmt::EnumDecl(_) => true,
            _ => false,
        }
    }
}

impl fmt::Display for Stmt {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Stmt::Use(use_stmt) => write!(f, "{}", use_stmt),
            Stmt::Expr(expr) => write!(f, "{}", expr),
            Stmt::If(if_stmt) => write!(f, "{}", if_stmt),
            Stmt::For(for_stmt) => write!(f, "{}", for_stmt),
            Stmt::Is(is_stmt) => write!(f, "{}", is_stmt),
            Stmt::Block(body) => write!(f, "{}", body),
            Stmt::Store(store) => write!(f, "{}", store),
            Stmt::Fn(fn_decl) => write!(f, "{}", fn_decl),
            Stmt::TypeDecl(type_decl) => write!(f, "{}", type_decl),
            Stmt::EnumDecl(enum_decl) => write!(f, "{}", enum_decl),
            Stmt::Node(node) => write!(f, "{}", node),
            Stmt::OnEvents(on_events) => write!(f, "{}", on_events),
            Stmt::Comment(cmt) => write!(f, "{}", cmt),
            Stmt::Alias(alias) => write!(f, "{}", alias),
            Stmt::EmptyLine(n) => write!(f, "(nl*{})", n),
            Stmt::Union(u) => write!(f, "{}", u),
            Stmt::Tag(tag) => write!(f, "{}", tag),
            Stmt::Break => write!(f, "(break)"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Expr {
    // value exprs
    Int(i32),
    Uint(u32),
    I8(i8),
    U8(u8),
    I64(i64),
    Byte(u8),
    Float(f64, AutoStr),
    Double(f64, AutoStr),
    Bool(bool),
    Char(char),
    Str(AutoStr),
    CStr(AutoStr),
    Ident(Name),
    GenName(Name), // names that is generated during parsing or gen that need not to be stored in SymbolTable
    // composite exprs
    Ref(Name),
    Unary(Op, Box<Expr>),
    Bina(Box<Expr>, Op, Box<Expr>),
    Range(Range),
    Array(Vec<Expr>),
    Pair(Pair),
    Block(Body),
    Object(Vec<Pair>),
    Call(Call),
    Node(Node),
    Index(/*array*/ Box<Expr>, /*index*/ Box<Expr>),
    Lambda(Fn),
    FStr(FStr),
    Grid(Grid),
    Cover(Cover),
    Uncover(TagUncover),
    // stmt exprs
    If(If),
    Nil,
    Null,
}

fn fmt_array(f: &mut fmt::Formatter, elems: &Vec<Expr>) -> fmt::Result {
    write!(f, "(array ")?;
    for (i, elem) in elems.iter().enumerate() {
        write!(f, "{}", elem)?;
        if i < elems.len() - 1 {
            write!(f, " ")?;
        }
    }
    write!(f, ")")
}

fn fmt_object(f: &mut fmt::Formatter, pairs: &Vec<Pair>) -> fmt::Result {
    write!(f, "(object ")?;
    for (i, pair) in pairs.iter().enumerate() {
        write!(f, "{}", pair)?;
        if i < pairs.len() - 1 {
            write!(f, " ")?;
        }
    }
    write!(f, ")")
}

fn fmt_block(f: &mut fmt::Formatter, body: &Body) -> fmt::Result {
    write!(f, "{}", body)
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Expr::Byte(b) => write!(f, "(byte {})", b),
            Expr::Int(i) => write!(f, "(int {})", i),
            Expr::Uint(u) => write!(f, "(uint {})", u),
            Expr::I8(i) => write!(f, "(i8 {})", i),
            Expr::U8(u) => write!(f, "(u8 {})", u),
            Expr::I64(i) => write!(f, "(i64 {})", i),
            Expr::Float(v, _) => write!(f, "(float {})", v),
            Expr::Double(v, _) => write!(f, "(double {})", v),
            Expr::Bool(b) => write!(f, "({})", b),
            Expr::Char(c) => write!(f, "(char '{}')", c),
            Expr::Str(s) => write!(f, "(str \"{}\")", s),
            Expr::CStr(s) => write!(f, "(cstr \"{}\")", s),
            Expr::Ident(n) => write!(f, "(name {})", n),
            Expr::Ref(n) => write!(f, "(ref {})", n),
            Expr::Bina(l, op, r) => write!(f, "(bina {} {} {})", l, op, r),
            Expr::Unary(op, e) => write!(f, "(una {} {})", op, e),
            Expr::Array(elems) => fmt_array(f, elems),
            Expr::Pair(pair) => write!(f, "{}", pair),
            Expr::Object(pairs) => fmt_object(f, pairs),
            Expr::Block(body) => fmt_block(f, body),
            Expr::If(if_) => write!(f, "{}", if_),
            Expr::Call(call) => fmt_call(f, &call),
            Expr::Node(node) => write!(f, "{}", node),
            Expr::Index(array, index) => write!(f, "(index {} {})", array, index),
            Expr::Lambda(lambda) => write!(f, "{}", lambda),
            Expr::FStr(fstr) => write!(f, "{}", fstr),
            Expr::Grid(grid) => write!(f, "{}", grid),
            Expr::Cover(cover) => write!(f, "{}", cover),
            Expr::Uncover(uncover) => write!(f, "{}", uncover),
            Expr::GenName(name) => write!(f, "(gen-name {})", name),
            Expr::Nil => write!(f, "(nil)"),
            Expr::Null => write!(f, "(null)"),
            Expr::Range(r) => write!(f, "{}", r),
        }
    }
}

impl Expr {
    pub fn repr(&self) -> AutoStr {
        match self {
            Expr::Int(i) => i.to_string().into(),
            Expr::Uint(u) => u.to_string().into(),
            Expr::Float(f, _) => f.to_string().into(),
            Expr::Bool(b) => b.to_string().into(),
            Expr::Char(c) => c.to_string().into(),
            Expr::Str(s) => s.clone(),
            Expr::Ident(n) => n.clone(),
            Expr::Ref(n) => n.clone(),
            Expr::Bina(l, op, r) => format!("{}{}{}", l.repr(), op.repr(), r.repr()).into(),
            Expr::Unary(op, e) => format!("{}{}", op.repr(), e.repr()).into(),
            Expr::Array(elems) => format!(
                "[{}]",
                elems
                    .iter()
                    .map(|e| e.repr())
                    .collect::<Vec<AutoStr>>()
                    .join(", ")
            )
            .into(),
            Expr::Pair(pair) => format!("{}:{}", pair.key.to_string(), pair.value.repr()).into(),
            Expr::Object(pairs) => format!(
                "{{{}}}",
                pairs
                    .iter()
                    .map(|p| p.repr().to_string())
                    .collect::<Vec<String>>()
                    .join(", ".into())
            )
            .into(),
            _ => self.to_string().into(),
        }
    }

    pub fn to_code(&self) -> AutoStr {
        match self {
            Expr::Int(i) => i.to_string().into(),
            Expr::Uint(u) => u.to_string().into(),
            Expr::Float(f, _) => f.to_string().into(),
            Expr::Bool(b) => b.to_string().into(),
            Expr::Char(c) => c.to_string().into(),
            Expr::Str(s) => format!("\"{}\"", s).into(),
            Expr::Ident(n) => n.clone(),
            Expr::Ref(n) => n.clone(),
            Expr::Bina(l, op, r) => format!("{}{}{}", l.to_code(), op.repr(), r.to_code()).into(),
            Expr::Unary(op, e) => format!("{}{}", op.repr(), e.to_code()).into(),
            Expr::Array(elems) => format!(
                "[{}]",
                elems
                    .iter()
                    .map(|e| e.to_code())
                    .collect::<Vec<AutoStr>>()
                    .join(", ")
            )
            .into(),
            Expr::Pair(pair) => format!("{}:{}", pair.key.to_string(), pair.value.to_code()).into(),
            Expr::Object(pairs) => format!(
                "{{{}}}",
                pairs
                    .iter()
                    .map(|p| p.repr().to_string())
                    .collect::<Vec<String>>()
                    .join(", ")
            )
            .into(),
            _ => self.to_string().into(),
        }
    }
}

// ToAtom implementations

impl ToAtom for Expr {
    fn to_atom(&self) -> Value {
        match self {
            // Literals - simple wrapper nodes
            Expr::Int(i) => {
                let mut node = auto_val::Node::new("int");
                node.add_arg(auto_val::Arg::Pos(Value::Int(*i)));
                Value::Node(node)
            }
            Expr::Uint(u) => {
                let mut node = auto_val::Node::new("uint");
                node.add_arg(auto_val::Arg::Pos(Value::Uint(*u)));
                Value::Node(node)
            }
            Expr::I8(i) => {
                let mut node = auto_val::Node::new("i8");
                node.add_arg(auto_val::Arg::Pos(Value::Int(*i as i32)));
                Value::Node(node)
            }
            Expr::U8(u) => {
                let mut node = auto_val::Node::new("u8");
                node.add_arg(auto_val::Arg::Pos(Value::Uint(*u as u32)));
                Value::Node(node)
            }
            Expr::I64(i) => {
                let mut node = auto_val::Node::new("i64");
                node.add_arg(auto_val::Arg::Pos(Value::Int(*i as i32)));
                Value::Node(node)
            }
            Expr::Byte(b) => {
                let mut node = auto_val::Node::new("byte");
                node.add_arg(auto_val::Arg::Pos(Value::Uint(*b as u32)));
                Value::Node(node)
            }
            Expr::Float(f, _) => {
                let mut node = auto_val::Node::new("float");
                node.add_arg(auto_val::Arg::Pos(Value::Float(*f)));
                Value::Node(node)
            }
            Expr::Double(d, _) => {
                let mut node = auto_val::Node::new("double");
                node.add_arg(auto_val::Arg::Pos(Value::Float(*d)));
                Value::Node(node)
            }
            Expr::Bool(b) => {
                let mut node = auto_val::Node::new("bool");
                node.add_arg(auto_val::Arg::Pos(Value::Bool(*b)));
                Value::Node(node)
            }
            Expr::Char(c) => {
                let mut node = auto_val::Node::new("char");
                node.add_arg(auto_val::Arg::Pos(Value::Char(*c)));
                Value::Node(node)
            }
            Expr::Str(s) => {
                let mut node = auto_val::Node::new("str");
                node.add_arg(auto_val::Arg::Pos(Value::str(s)));
                Value::Node(node)
            }
            Expr::CStr(s) => {
                let mut node = auto_val::Node::new("cstr");
                node.add_arg(auto_val::Arg::Pos(Value::str(s)));
                Value::Node(node)
            }
            Expr::Nil => {
                let node = auto_val::Node::new("nil");
                Value::Node(node)
            }
            Expr::Null => {
                let node = auto_val::Node::new("null");
                Value::Node(node)
            }

            // Identifiers
            Expr::Ident(name) => {
                let mut node = auto_val::Node::new("name");
                node.add_arg(auto_val::Arg::Pos(Value::str(name)));
                Value::Node(node)
            }
            Expr::Ref(name) => {
                let mut node = auto_val::Node::new("ref");
                node.add_arg(auto_val::Arg::Pos(Value::str(name)));
                Value::Node(node)
            }
            Expr::GenName(name) => {
                let mut node = auto_val::Node::new("gen-name");
                node.add_arg(auto_val::Arg::Pos(Value::str(name)));
                Value::Node(node)
            }

            // Operators
            Expr::Unary(op, expr) => {
                let mut node = auto_val::Node::new("una");
                node.set_prop("op", Value::str(op.to_string().as_str()));
                node.add_kid(expr.to_atom().to_node());
                Value::Node(node)
            }
            Expr::Bina(left, op, right) => {
                let mut node = auto_val::Node::new("bina");
                node.set_prop("op", Value::str(op.to_string().as_str()));
                node.add_kid(left.to_atom().to_node());
                node.add_kid(right.to_atom().to_node());
                Value::Node(node)
            }

            // Containers
            Expr::Array(elems) => {
                let items: Vec<Value> = elems.iter().map(|e| e.to_atom()).collect();
                let mut node = auto_val::Node::new("array");
                node.add_arg(auto_val::Arg::Pos(Value::array(auto_val::Array::from_vec(items))));
                Value::Node(node)
            }
            Expr::Object(pairs) => {
                let mut obj = auto_val::Obj::new();
                for pair in pairs {
                    let value_key = match &pair.key {
                        Key::NamedKey(k) => auto_val::ValueKey::Str(k.clone()),
                        Key::IntKey(i) => auto_val::ValueKey::Int(*i),
                        Key::BoolKey(b) => auto_val::ValueKey::Bool(*b),
                        Key::StrKey(s) => auto_val::ValueKey::Str(s.clone()),
                    };
                    obj.set(value_key, pair.value.to_atom());
                }
                let mut node = auto_val::Node::new("object");
                node.add_arg(auto_val::Arg::Pos(Value::Obj(obj)));
                Value::Node(node)
            }

            // Control flow and calls
            Expr::Call(call) => {
                let mut node = auto_val::Node::new("call");
                node.add_kid(call.name.to_atom().to_node());
                node.add_kid(call.args.to_atom().to_node());
                Value::Node(node)
            }
            Expr::Index(array, index) => {
                let mut node = auto_val::Node::new("index");
                node.add_kid(array.to_atom().to_node());
                node.add_kid(index.to_atom().to_node());
                Value::Node(node)
            }
            Expr::If(if_) => {
                // Delegate to If::to_atom() (will be implemented in Step 5)
                if_.to_atom()
            }
            Expr::Range(range) => range.to_atom(),
            Expr::Block(body) => body.to_atom(),
            Expr::Pair(pair) => pair.to_atom(),
            Expr::Node(node) => {
                // Delegate to Node::to_atom() (will be implemented later)
                Value::str(node.to_string().as_str())
            }

            // For complex expressions, use stub for now (will be fully implemented later)
            _ => Value::str(self.to_string().as_str()),
        }
    }
}

impl ToNode for Stmt {
    fn to_node(&self) -> AutoNode {
        match self {
            Stmt::Expr(expr) => expr.to_atom().to_node(),
            Stmt::If(if_) => if_.to_node(),
            Stmt::For(for_) => for_.to_node(),
            Stmt::Is(is) => is.to_node(),
            Stmt::Store(store) => store.to_node(),
            Stmt::Block(body) => body.to_node(),
            Stmt::Fn(fn_) => fn_.to_node(),
            Stmt::EnumDecl(enum_decl) => enum_decl.to_node(),
            Stmt::TypeDecl(type_decl) => type_decl.to_node(),
            Stmt::Union(union) => union.to_node(),
            Stmt::Tag(tag) => tag.to_node(),
            Stmt::Node(node) => node.to_node(),
            Stmt::Use(use_) => use_.to_node(),
            Stmt::OnEvents(on_events) => on_events.to_node(),
            Stmt::Comment(comment) => {
                let mut node = AutoNode::new("comment");
                node.add_arg(auto_val::Arg::Pos(Value::str(comment.as_str())));
                node
            }
            Stmt::Alias(alias) => alias.to_node(),
            Stmt::EmptyLine(n) => {
                let mut node = AutoNode::new("nl");
                node.set_prop("count", Value::Int(*n as i32));
                node
            }
            Stmt::Break => AutoNode::new("break"),
        }
    }
}

impl ToAtom for Stmt {
    fn to_atom(&self) -> Value {
        match self {
            Stmt::Expr(expr) => expr.to_atom(),
            Stmt::If(if_) => if_.to_atom(),
            Stmt::For(for_) => for_.to_atom(),
            Stmt::Is(is) => is.to_atom(),
            Stmt::Store(store) => store.to_atom(),
            Stmt::Block(body) => body.to_atom(),
            Stmt::Fn(fn_) => fn_.to_atom(),
            Stmt::EnumDecl(enum_decl) => enum_decl.to_atom(),
            Stmt::TypeDecl(type_decl) => type_decl.to_atom(),
            Stmt::Union(union) => union.to_atom(),
            Stmt::Tag(tag) => tag.to_atom(),
            Stmt::Node(node) => node.to_atom(),
            Stmt::Use(use_) => use_.to_atom(),
            Stmt::OnEvents(on_events) => on_events.to_atom(),
            Stmt::Comment(comment) => Value::str(comment.as_str()),
            Stmt::Alias(alias) => alias.to_atom(),
            Stmt::EmptyLine(n) => {
                let mut node = auto_val::Node::new("nl");
                node.set_prop("count", Value::Int(*n as i32));
                Value::Node(node)
            }
            Stmt::Break => {
                let node = auto_val::Node::new("break");
                Value::Node(node)
            }
        }
    }
}

impl ToNode for Code {
    fn to_node(&self) -> AutoNode {
        let mut node = AutoNode::new("code");
        for stmt in &self.stmts {
            let stmt_node = stmt.to_atom().to_node();
            node.add_kid(stmt_node);
        }
        node
    }
}

impl ToAtom for Code {
    fn to_atom(&self) -> Value {
        Value::Node(self.to_node())
    }
}

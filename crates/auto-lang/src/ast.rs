mod alias;
pub use alias::*;
mod body;
pub use body::*;
mod branch;
pub use branch::*;
mod call;
pub use call::*;
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
mod types;
pub use types::*;
mod use_;
pub use use_::*;

mod parsers;

use auto_val::{AutoStr, Op};
use std::fmt;

pub type Name = AutoStr;

#[derive(Debug)]
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
    Node(Node),
    Use(Use),
    OnEvents(OnEvents),
    Comment(AutoStr),
    Alias(Alias),
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
    Byte(u8),
    Float(f64, AutoStr),
    Double(f64, AutoStr),
    Bool(bool),
    Char(char),
    Str(AutoStr),
    CStr(AutoStr),
    Ident(Name),
    // composite exprs
    Ref(Name),
    Unary(Op, Box<Expr>),
    Bina(Box<Expr>, Op, Box<Expr>),
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
    // stmt exprs
    If(If),
    Nil,
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
            Expr::Nil => write!(f, "(nil)"),
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

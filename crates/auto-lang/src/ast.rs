mod types;
pub use types::*;
mod enums;
pub use enums::*;
mod is;
pub use is::*;
mod on;
pub use on::*;
mod node;
pub use node::*;

mod parsers;

use auto_val::{AutoStr, Op};
use serde::Serialize;
use std::fmt::{self, write};

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
pub enum UseKind {
    Auto,
    C,
    Rust,
}

#[derive(Debug, Clone)]
pub struct Use {
    pub kind: UseKind,
    pub paths: Vec<AutoStr>,
    pub items: Vec<AutoStr>,
}

impl fmt::Display for Use {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(use")?;
        match self.kind {
            UseKind::C => write!(f, " (kind c)")?,
            UseKind::Rust => write!(f, " (kind rust)")?,
            _ => (),
        }
        if !self.paths.is_empty() {
            write!(f, " (path {})", self.paths.join("."))?;
        }
        if !self.items.is_empty() {
            write!(f, " (items {})", self.items.join(","))?;
        }
        write!(f, ")")
    }
}

#[derive(Debug, Clone)]
pub struct Branch {
    pub cond: Expr,
    pub body: Body,
}

impl fmt::Display for Branch {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(branch {} {})", self.cond, self.body)
    }
}

#[derive(Debug, Clone)]
pub enum StoreKind {
    Let,
    Mut,
    Var,
    Field, // field of struct
}

#[derive(Debug, Clone)]
pub struct Store {
    pub kind: StoreKind,
    pub name: Name,
    pub ty: Type,
    pub expr: Expr,
}

impl fmt::Display for Store {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let ty_str = if matches!(self.ty, Type::Unknown) {
            " ".to_string()
        } else {
            format!(" (type {}) ", self.ty)
        };
        match self.kind {
            StoreKind::Let => write!(f, "(let (name {}){}{})", self.name, ty_str, self.expr),
            StoreKind::Mut => write!(f, "(mut (name {}){}{})", self.name, ty_str, self.expr),
            StoreKind::Var => write!(f, "(var (name {}) {})", self.name, self.expr),
            StoreKind::Field => write!(f, "(field (name {}) {})", self.name, self.expr),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Var {
    pub name: Name,
    pub expr: Expr,
}

impl fmt::Display for Var {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(var {} {})", self.name, self.expr)
    }
}

impl Var {
    pub fn new(name: Name, expr: Expr) -> Self {
        Self { name, expr }
    }
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Expr(Expr),
    If(
        /*multiple branches with condition/body*/ Vec<Branch>,
        /*else*/ Option<Body>,
    ),
    For(For),
    Is(Is),
    Store(Store),
    Block(Body),
    Fn(Fn),
    EnumDecl(EnumDecl),
    TypeDecl(TypeDecl),
    Widget(Widget),
    Node(Node),
    Use(Use),
    OnEvents(OnEvents),
    Comment(AutoStr),
    Alias(Alias),
}

#[derive(Debug, Clone)]
pub struct Alias {
    pub alias: Name,
    pub target: Name,
}

#[derive(Debug, Clone)]
pub struct For {
    pub iter: Iter,
    pub range: Expr,
    pub body: Body,
    pub new_line: bool,
    // TODO: maybe we could put mid block here
}

impl fmt::Display for For {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(for {} {} {})", self.iter, self.range, self.body)
    }
}

#[derive(Debug, Clone)]
pub enum Iter {
    Indexed(/*index*/ Name, /*iter*/ Name),
    Named(/*iter*/ Name),
}

#[derive(Debug, Clone)]
pub struct Body {
    pub stmts: Vec<Stmt>,
    pub has_new_line: bool,
}

impl Body {
    pub fn new() -> Self {
        Self {
            stmts: Vec::new(),
            has_new_line: false,
        }
    }

    pub fn single_expr(expr: Expr) -> Self {
        Self {
            stmts: vec![Stmt::Expr(expr)],
            has_new_line: false,
        }
    }
}

impl fmt::Display for Iter {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Iter::Indexed(index, iter) => write!(f, "((name {}) (name {}))", index, iter),
            Iter::Named(iter) => write!(f, "(name {})", iter),
        }
    }
}

impl fmt::Display for Body {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(body ")?;
        for (i, stmt) in self.stmts.iter().enumerate() {
            write!(f, "{}", stmt)?;
            if i < self.stmts.len() - 1 {
                write!(f, " ")?;
            }
        }
        write!(f, ")")
    }
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
            Stmt::If(branches, else_stmt) => {
                write!(f, "(if ")?;
                for branch in branches.iter() {
                    write!(f, "{}", branch)?;
                }
                if let Some(else_stmt) = else_stmt {
                    write!(f, " (else {})", else_stmt)?;
                }
                write!(f, ")")
            }
            Stmt::For(for_stmt) => write!(f, "{}", for_stmt),
            Stmt::Is(is_stmt) => write!(f, "{}", is_stmt),
            Stmt::Block(body) => write!(f, "{}", body),
            Stmt::Store(store) => write!(f, "{}", store),
            Stmt::Fn(fn_decl) => write!(f, "{}", fn_decl),
            Stmt::TypeDecl(type_decl) => write!(f, "{}", type_decl),
            Stmt::EnumDecl(enum_decl) => write!(f, "{}", enum_decl),
            Stmt::Widget(widget) => write!(f, "{}", widget),
            Stmt::Node(node) => write!(f, "{}", node),
            Stmt::OnEvents(on_events) => write!(f, "{}", on_events),
            Stmt::Comment(cmt) => write!(f, "{}", cmt),
            Stmt::Alias(alias) => write!(f, "{}", alias),
        }
    }
}

impl fmt::Display for Alias {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(alias (name {}) (target {}))", self.alias, self.target)
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
    If(Vec<Branch>, Option<Body>),
    Nil,
}

#[derive(Debug, Clone)]
pub struct Call {
    pub name: Box<Expr>,
    pub args: Args,
}

impl Call {
    pub fn get_name_text(&self) -> AutoStr {
        match &self.name.as_ref() {
            Expr::Ident(name) => name.clone(),
            _ => panic!("Expected identifier, got {:?}", self.name),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Args {
    // pub array: Vec<Expr>,
    // pub map: Vec<(Name, Expr)>,
    pub args: Vec<Arg>,
}

#[derive(Debug, Clone)]
pub enum Arg {
    Pos(Expr),
    Name(Name),
    Pair(Name, Expr),
}

impl Arg {
    pub fn get_expr(&self) -> Expr {
        match self {
            Arg::Pos(expr) => expr.clone(),
            Arg::Name(name) => Expr::Str(name.clone()),
            Arg::Pair(_, expr) => expr.clone(),
        }
    }

    pub fn repr(&self) -> AutoStr {
        match self {
            Arg::Pos(expr) => expr.repr(),
            Arg::Name(name) => name.clone(),
            Arg::Pair(key, expr) => format!("{}:{}", key, expr.repr()).into(),
        }
    }

    pub fn to_code(&self) -> AutoStr {
        match self {
            Arg::Pos(expr) => expr.to_code(),
            Arg::Name(name) => name.clone(),
            Arg::Pair(key, expr) => format!("{}:{}", key, expr.to_code()).into(),
        }
    }
}

impl Args {
    pub fn new() -> Self {
        Self { args: Vec::new() }
    }

    pub fn len(&self) -> usize {
        self.args.len()
    }
    pub fn get(&self, idx: usize) -> Option<Arg> {
        self.args.get(idx).cloned()
    }

    pub fn lookup(&self, name: &str) -> Option<Arg> {
        for arg in self.args.iter() {
            match arg {
                Arg::Name(n) => {
                    if n == name {
                        return Some(arg.clone());
                    }
                }
                Arg::Pair(n, _) => {
                    if n == name {
                        return Some(arg.clone());
                    }
                }
                _ => {}
            }
        }
        None
    }

    pub fn is_empty(&self) -> bool {
        self.args.is_empty()
    }

    pub fn id(&self) -> AutoStr {
        let empty = "".into();
        let id = match self.args.first() {
            Some(Arg::Name(name)) => name.clone(),
            Some(Arg::Pair(k, v)) => {
                if k == "id" {
                    v.repr().clone()
                } else {
                    empty
                }
            }
            Some(Arg::Pos(p)) => match p {
                Expr::Str(s) => s.clone(),
                Expr::Ident(n) => n.clone(),
                _ => empty,
            },
            _ => empty,
        };
        let id = if id.is_empty() {
            // try all args
            let arg = self.args.iter().find_map(|arg| match arg {
                Arg::Pair(k, v) => {
                    if k == "id" {
                        Some(v.repr().clone())
                    } else {
                        None
                    }
                }
                _ => None,
            });
            if let Some(arg) = arg {
                arg
            } else {
                id
            }
        } else {
            id
        };
        id
    }

    pub fn major(&self) -> Option<&Arg> {
        self.args.first()
    }

    pub fn first_arg(&self) -> Option<Expr> {
        let Some(arg) = self.args.first() else {
            return None;
        };
        match arg {
            Arg::Pos(expr) => Some(expr.clone()),
            Arg::Name(n) => Some(Expr::Ident(n.clone())),
            _ => None,
        }
    }
}

impl fmt::Display for Args {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(args")?;
        if !self.args.is_empty() {
            for arg in self.args.iter() {
                write!(f, " {}", arg)?;
            }
        }
        write!(f, ")")
    }
}

impl fmt::Display for Arg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Arg::Pos(expr) => write!(f, "{}", expr),
            Arg::Name(name) => write!(f, "(name {})", name),
            Arg::Pair(name, expr) => write!(f, "(pair (name {}) {})", name, expr),
        }
    }
}

fn fmt_call(f: &mut fmt::Formatter, call: &Call) -> fmt::Result {
    write!(f, "(call ")?;
    write!(f, "{}", call.name)?;
    if !call.args.is_empty() {
        write!(f, " {}", call.args)?;
    }
    write!(f, ")")?;
    Ok(())
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
            Expr::If(branches, else_stmt) => write!(f, "(if {:?} {:?})", branches, else_stmt),
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

#[derive(Debug, Clone)]
pub struct Param {
    pub name: Name,
    pub ty: Type,
    pub default: Option<Expr>,
}

impl PartialEq for Param {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl fmt::Display for Param {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(param (name {}) (type {})", self.name, self.ty)?;
        if let Some(default) = &self.default {
            write!(f, " (default {})", default)?;
        }
        write!(f, ")")
    }
}

#[derive(Debug, Clone)]
pub struct Fn {
    // TODO: add FnKind to differ Fn/Lambda/Method?
    pub name: Name,
    pub parent: Option<Name>, // for method
    pub params: Vec<Param>,
    pub body: Body,
    pub ret: Type,
}

impl Serialize for Fn {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        return serializer.serialize_str("fn");
    }
}

impl PartialEq for Fn {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.params == other.params
    }
}

impl fmt::Display for Fn {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(fn (name {})", self.name)?;
        if !self.params.is_empty() {
            write!(f, " (params ")?;
            for (i, param) in self.params.iter().enumerate() {
                write!(f, "{}", param)?;
                if i < self.params.len() - 1 {
                    write!(f, " ")?;
                }
            }
            write!(f, ")")?;
        }
        if !matches!(self.ret, Type::Unknown) {
            write!(f, " (ret {})", self.ret)?;
        }
        write!(f, " {}", self.body)?;
        write!(f, ")")
    }
}

impl Fn {
    pub fn new(name: Name, parent: Option<Name>, params: Vec<Param>, body: Body, ret: Type) -> Fn {
        Fn {
            name,
            parent,
            params,
            body,
            ret,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Widget {
    pub name: Name,
    pub model: Model,
    pub view: View,
}

impl Widget {
    pub fn new(name: Name) -> Self {
        Self {
            name,
            model: Model::default(),
            view: View::default(),
        }
    }
}

impl fmt::Display for Widget {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(widget {} {} {})", self.name, self.model, self.view)
    }
}

#[derive(Debug, Clone)]
pub struct Model {
    pub vars: Vec<Store>,
}

impl Default for Model {
    fn default() -> Self {
        Self {
            vars: Vec::default(),
        }
    }
}

impl fmt::Display for Model {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(model")?;
        for (i, state) in self.vars.iter().enumerate() {
            write!(f, " {}", state)?;
            if i < self.vars.len() - 1 {
                write!(f, " ")?;
            }
        }
        write!(f, ")")
    }
}

#[derive(Debug, Clone)]
pub struct View {
    pub nodes: Vec<(Name, Node)>,
    pub body: Body,
}

impl Default for View {
    fn default() -> Self {
        Self {
            nodes: Vec::new(),
            body: Body::new(),
        }
    }
}

impl fmt::Display for View {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(view")?;
        for (_name, node) in self.nodes.iter() {
            write!(f, " {}", node)?;
        }
        write!(f, ")")
    }
}

// fn fmt_type_inst(f: &mut fmt::Formatter, name: &Box<Expr>, entries: &Vec<Pair>) -> fmt::Result {
//     write!(f, "(type-inst {} ", name.as_ref())?;
//     for (i, pair) in entries.iter().enumerate() {
//         write!(f, "{}", pair)?;
//         if i < entries.len() - 1 {
//             write!(f, " ")?;
//         }
//     }
//     write!(f, ")")
// }

#[derive(Debug, Clone)]
pub struct FStr {
    pub parts: Vec<Expr>,
}

impl FStr {
    pub fn new(parts: Vec<Expr>) -> Self {
        Self { parts }
    }
}

impl fmt::Display for FStr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(fstr")?;
        for part in self.parts.iter() {
            write!(f, " {}", part)?;
        }
        write!(f, ")")
    }
}

#[derive(Debug, Clone)]
pub struct Grid {
    pub head: Args,
    pub data: Vec<Vec<Expr>>,
}

impl fmt::Display for Grid {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(grid")?;
        if !self.head.is_empty() {
            write!(f, " (head")?;
            for arg in self.head.args.iter() {
                write!(f, " {}", arg)?;
            }
            write!(f, ")")?;
        }
        if !self.data.is_empty() {
            write!(f, " (data")?;
            for row in self.data.iter() {
                write!(f, " (row ")?;
                for (j, cell) in row.iter().enumerate() {
                    write!(f, "{}", cell)?;
                    if j < row.len() - 1 {
                        write!(f, " ")?;
                    }
                }
                write!(f, ")")?;
            }
            write!(f, ")")?;
        }
        write!(f, ")")
    }
}

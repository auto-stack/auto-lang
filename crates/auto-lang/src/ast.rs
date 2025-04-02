mod types;
pub use types::*;

mod enums;
pub use enums::*;

use auto_val::{AutoStr, Op};
use serde::Serialize;
use std::fmt;

#[derive(Debug)]
pub struct Code {
    pub stmts: Vec<Stmt>,
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
pub struct Use {
    pub paths: Vec<String>,
    pub items: Vec<String>,
}

impl fmt::Display for Use {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(use ")?;
        write!(f, "(path {})", self.paths.join("."))?;
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

#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct Name {
    pub text: String,
}

impl Name {
    pub fn new(text: impl Into<String>) -> Name {
        Name { text: text.into() }
    }
}

impl Default for Name {
    fn default() -> Self {
        Self {
            text: "".to_string(),
        }
    }
}

impl fmt::Display for Name {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(name {})", self.text)
    }
}

impl From<Name> for AutoStr {
    fn from(name: Name) -> Self {
        AutoStr::from(name.text)
    }
}

impl From<&str> for Name {
    fn from(text: &str) -> Self {
        Name {
            text: text.to_string(),
        }
    }
}

impl From<String> for Name {
    fn from(text: String) -> Self {
        Name { text }
    }
}

impl From<AutoStr> for Name {
    fn from(text: AutoStr) -> Self {
        Name {
            text: text.to_string(),
        }
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
            StoreKind::Let => write!(f, "(let {}{}{})", self.name, ty_str, self.expr),
            StoreKind::Mut => write!(f, "(mut {}{}{})", self.name, ty_str, self.expr),
            StoreKind::Var => write!(f, "(var {} {})", self.name, self.expr),
            StoreKind::Field => write!(f, "(field {} {})", self.name, self.expr),
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
    Store(Store),
    Block(Body),
    Fn(Fn),
    EnumDecl(EnumDecl),
    TypeDecl(TypeDecl),
    Widget(Widget),
    Node(Node),
    Use(Use),
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
            Iter::Indexed(index, iter) => write!(f, "({} {})", index, iter),
            Iter::Named(iter) => write!(f, "{}", iter),
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
            Stmt::Block(body) => write!(f, "{}", body),
            Stmt::Store(store) => write!(f, "{}", store),
            Stmt::Fn(fn_decl) => write!(f, "{}", fn_decl),
            Stmt::TypeDecl(type_decl) => write!(f, "{}", type_decl),
            Stmt::EnumDecl(enum_decl) => write!(f, "{}", enum_decl),
            Stmt::Widget(widget) => write!(f, "{}", widget),
            Stmt::Node(node) => write!(f, "{}", node),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Expr {
    // value exprs
    Int(i32),
    Uint(u32),
    Byte(u8),
    Float(f64),
    Bool(bool),
    Char(char),
    Str(String),
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
    pub fn get_name_text(&self) -> String {
        match &self.name.as_ref() {
            Expr::Ident(name) => name.text.clone(),
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
            Arg::Name(name) => Expr::Str(name.text.clone()),
            Arg::Pair(_, expr) => expr.clone(),
        }
    }

    pub fn repr(&self) -> String {
        match self {
            Arg::Pos(expr) => expr.repr(),
            Arg::Name(name) => name.text.clone(),
            Arg::Pair(key, expr) => format!("{}:{}", key.text, expr.repr()),
        }
    }

    pub fn to_code(&self) -> String {
        match self {
            Arg::Pos(expr) => expr.to_code(),
            Arg::Name(name) => name.text.clone(),
            Arg::Pair(key, expr) => format!("{}:{}", key.text, expr.to_code()),
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
                    if n.text == name {
                        return Some(arg.clone());
                    }
                }
                Arg::Pair(n, _) => {
                    if n.text == name {
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
            Arg::Name(name) => write!(f, "{}", name),
            Arg::Pair(name, expr) => write!(f, "(pair {} {})", name, expr),
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
            Expr::Float(v) => write!(f, "(float {})", v),
            Expr::Bool(b) => write!(f, "({})", b),
            Expr::Char(c) => write!(f, "(char '{}')", c),
            Expr::Str(s) => write!(f, "(str \"{}\")", s),
            Expr::Ident(n) => write!(f, "(name {})", n.text),
            Expr::Ref(n) => write!(f, "(ref {})", n.text),
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
    pub fn repr(&self) -> String {
        match self {
            Expr::Int(i) => i.to_string(),
            Expr::Uint(u) => u.to_string(),
            Expr::Float(f) => f.to_string(),
            Expr::Bool(b) => b.to_string(),
            Expr::Char(c) => c.to_string(),
            Expr::Str(s) => s.clone(),
            Expr::Ident(n) => n.text.clone(),
            Expr::Ref(n) => n.text.clone(),
            Expr::Bina(l, op, r) => format!("{}{}{}", l.repr(), op.repr(), r.repr()),
            Expr::Unary(op, e) => format!("{}{}", op.repr(), e.repr()),
            Expr::Array(elems) => format!(
                "[{}]",
                elems
                    .iter()
                    .map(|e| e.repr())
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            Expr::Pair(pair) => format!("{}:{}", pair.key.to_string(), pair.value.repr()),
            Expr::Object(pairs) => format!(
                "{{{}}}",
                pairs
                    .iter()
                    .map(|p| p.repr())
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            _ => self.to_string(),
        }
    }

    pub fn to_code(&self) -> String {
        match self {
            Expr::Int(i) => i.to_string(),
            Expr::Uint(u) => u.to_string(),
            Expr::Float(f) => f.to_string(),
            Expr::Bool(b) => b.to_string(),
            Expr::Char(c) => c.to_string(),
            Expr::Str(s) => format!("\"{}\"", s),
            Expr::Ident(n) => n.text.clone(),
            Expr::Ref(n) => n.text.clone(),
            Expr::Bina(l, op, r) => format!("{}{}{}", l.to_code(), op.repr(), r.to_code()),
            Expr::Unary(op, e) => format!("{}{}", op.repr(), e.to_code()),
            Expr::Array(elems) => format!(
                "[{}]",
                elems
                    .iter()
                    .map(|e| e.to_code())
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            Expr::Pair(pair) => format!("{}:{}", pair.key.to_string(), pair.value.to_code()),
            Expr::Object(pairs) => format!(
                "{{{}}}",
                pairs
                    .iter()
                    .map(|p| p.repr())
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            _ => self.to_string(),
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
        write!(f, "(param {} (type {})", self.name, self.ty)?;
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
        write!(f, "(fn {}", self.name)?;
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
pub struct Node {
    pub name: Name,
    pub args: Args,
    // pub props: BTreeMap<Key, Expr>,
    pub body: Body,
}

impl Node {
    pub fn new(name: impl Into<Name>) -> Self {
        Self {
            name: name.into(),
            args: Args::new(),
            body: Body::new(),
        }
    }
}

impl From<Call> for Node {
    fn from(call: Call) -> Self {
        let name = Name::new(call.get_name_text());
        let mut node = Node::new(name);
        node.args = call.args;
        node
    }
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(node {}", self.name)?;
        if !self.args.is_empty() {
            write!(f, " {}", self.args)?;
        }

        if !self.body.stmts.is_empty() {
            write!(f, " {}", self.body)?;
        }

        write!(f, ")")
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

use std::{collections::BTreeMap, fmt};

#[derive(Debug)]
pub struct Code {
    pub stmts: Vec<Stmt>,
}

impl Default for Code {
    fn default() -> Self {
        Self { stmts: Vec::default() }
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
    pub fn new(text: String) -> Name {
        Name { text }
    }
}

impl fmt::Display for Name {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(name {})", self.text)
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

#[derive(Debug, Clone)]
pub enum Stmt {
    Expr(Expr),
    If(/*multiple branches with condition/body*/Vec<Branch>, /*else*/Option<Body>),
    For(Name, Expr, Body),
    Var(Var),
    Fn(Fn),
    TypeDecl(TypeDecl),
    Widget(Widget),
}

#[derive(Debug, Clone)]
pub struct Body {
    pub stmts: Vec<Stmt>,
}

impl fmt::Display for Body {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(body ")?;
        for stmt in self.stmts.iter() {
            write!(f, "{}", stmt)?;
        }
        Ok(())
    }
}

impl fmt::Display for Stmt {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Stmt::Expr(expr) => write!(f, "(stmt {})", expr),
            Stmt::If(branches, else_stmt) => {
                write!(f, "(if ")?;
                for branch in branches.iter() {
                    write!(f, "{}", branch)?;
                }
                if let Some(else_stmt) = else_stmt {
                    write!(f, " (else {})", else_stmt)?;
                }
                Ok(())
            },
            Stmt::For(name, expr, body) => write!(f, "(for {} {} {})", name, expr, body),
            Stmt::Var(var) => write!(f, "(var {} {})", var.name, var.expr),
            Stmt::Fn(fn_decl) => write!(f, "{}", fn_decl),
            Stmt::TypeDecl(type_decl) => write!(f, "{}", type_decl),    
            Stmt::Widget(widget) => write!(f, "{}", widget),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Op {
    Add,
    Sub,
    Mul,
    Div,
    Not,
    LSquare,
    LParen,
    LBrace,
    Asn,
    Eq,
    Neq,
    Lt,
    Gt,
    Le,
    Ge,
    Range,
    RangeEq,
    Dot,
}

impl fmt::Display for Op {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Op::Add => write!(f, "(op +)"),
            Op::Sub => write!(f, "(op -)"),
            Op::Mul => write!(f, "(op *)"),
            Op::Div => write!(f, "(op /)"),
            Op::Not => write!(f, "(op !)"),
            Op::LSquare => write!(f, "(op [)"),
            Op::Asn => write!(f, "(op =)"),
            Op::Eq => write!(f, "(op ==)"),
            Op::Neq => write!(f, "(op !=)"),
            Op::Lt => write!(f, "(op <)"),
            Op::Gt => write!(f, "(op >)"),
            Op::Le => write!(f, "(op <=)"),
            Op::Ge => write!(f, "(op >=)"),
            Op::Range => write!(f, "(op ..)"),
            Op::RangeEq => write!(f, "(op ..=)"),
            Op::Dot => write!(f, "(op .)"),
            Op::LParen => write!(f, "(op ()"),
            Op::LBrace => write!(f, "(op {{)"),
        }
    }
}

impl Op {
    pub fn op(&self) -> &str {
        match self {
            Op::Add => "+",
            Op::Sub => "-",
            Op::Mul => "*",
            Op::Div => "/",
            Op::Not => "!",
            Op::LSquare => "[",
            Op::LParen => "(",
            Op::LBrace => "{",
            Op::Asn => "=",
            Op::Eq => "==",
            Op::Neq => "!=",
            Op::Lt => "<",
            Op::Gt => ">",
            Op::Le => "<=",
            Op::Ge => ">=",
            Op::Range => "..",
            Op::RangeEq => "..=",
            Op::Dot => ".",
        }
    }
}

#[derive(Debug, Clone)]
pub enum Expr {
    // value exprs
    Int(i32),
    Float(f64),
    Bool(bool),
    Str(String),
    Ident(Name),
    // composite exprs
    Unary(Op, Box<Expr>),
    Bina(Box<Expr>, Op, Box<Expr>),
    Array(Vec<Expr>),
    Object(Vec<(Key, Expr)>),
    Call(/*name*/Box<Expr>, /*args*/Vec<Expr>),
    Index(/*array*/Box<Expr>, /*index*/Box<Expr>),
    TypeInst(/*name*/Box<Expr>, /*entries*/Vec<(Key, Expr)>),
    Lambda(/*params*/Vec<Param>, /*body*/Box<Stmt>),
    // stmt exprs
    If(Vec<Branch>, Option<Body>),
    Nil,
}

fn fmt_call(f: &mut fmt::Formatter, name: &Expr, args: &Vec<Expr>) -> fmt::Result {
    write!(f, "(call ")?;
    write!(f, "{}", name)?;
    write!(f, " (args")?;
    for arg in args.iter() {
        write!(f, " {}", arg)?;
    }
    write!(f, ")")?;
    Ok(())
}

fn fmt_object(f: &mut fmt::Formatter, pairs: &Vec<(Key, Expr)>) -> fmt::Result {
    write!(f, "(object ")?;
    for (i, (k, v)) in pairs.iter().enumerate() {
        write!(f, "(pair {} {})", k, v)?;
        if i < pairs.len() - 1 {
            write!(f, " ")?;
        }
    }
    write!(f, ")")
}

fn fmt_lambda(f: &mut fmt::Formatter, params: &Vec<Param>, body: &Box<Stmt>) -> fmt::Result {
    write!(f, "(lambda")?;
    if !params.is_empty() {
        write!(f, " (params")?;
        for (i, param) in params.iter().enumerate() {
            write!(f, "{}", param)?;
            if i < params.len() - 1 {
                write!(f, " ")?;
            }
        }
        write!(f, ")")?;
    }
    write!(f, " {}", body)?;
    Ok(())
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Expr::Int(i) => write!(f, "(int {})", i),
            Expr::Float(v) => write!(f, "(float {})", v),
            Expr::Bool(b) => write!(f, "({})", b),
            Expr::Str(s) => write!(f, "(\"{}\")", s),
            Expr::Ident(n) => write!(f, "(name {})", n.text),
            Expr::Bina(l, op, r) => write!(f, "(bina {} {} {})", l, op, r),
            Expr::Unary(op, e) => write!(f, "(una {} {})", op, e),
            Expr::Array(elems) => write!(f, "(array {:?})", elems),
            Expr::Object(pairs) => fmt_object(f, pairs),
            Expr::If(branches, else_stmt) => write!(f, "(if {:?} {:?})", branches, else_stmt),
            Expr::Call(name, args) => fmt_call(f, name, args),
            Expr::Index(array, index) => write!(f, "(index {} {})", array, index),
            Expr::TypeInst(name, entries) => write!(f, "(type-inst {} {:?})", name, entries),
            Expr::Lambda(params, body) => fmt_lambda(f, params, body),
            Expr::Nil => write!(f, "(nil)"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Param {
    pub name: Name,
    pub default: Option<Expr>,
}

impl PartialEq for Param {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl fmt::Display for Param {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(param {}", self.name.text)?;
        if let Some(default) = &self.default {
            write!(f, "={}", default)?;
        }
        write!(f, ")")?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum Type {
    Int,
    Float,
    Bool,
    Str,
    User(TypeDecl),
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Type::Int => write!(f, "int"),
            Type::Float => write!(f, "float"),
            Type::Bool => write!(f, "bool"),
            Type::Str => write!(f, "str"),
            Type::User(type_decl) => write!(f, "{}", type_decl),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Fn {
    pub name: Name,
    pub params: Vec<Param>,
    pub body: Body,
    pub ret: Option<Type>,
}

impl PartialEq for Fn {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.params == other.params
    }
}

impl fmt::Display for Fn {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(fn {} (params ", self.name)?;
        for (i, param) in self.params.iter().enumerate() {
            write!(f, "{}", param)?;
            if i < self.params.len() - 1 {
                write!(f, " ")?;
            }
        }
        write!(f, ") {}", self.body)
    }
}

impl Fn {
    pub fn new(name: Name, params: Vec<Param>, body: Body, ret: Option<Type>) -> Fn {
        Fn { name, params, body, ret}
    }

}

#[derive(Debug, Clone)]
pub struct TypeDecl {
    pub name: Name,
    pub members: Vec<Member>,
    pub methods: Vec<Fn>,
}

impl fmt::Display for TypeDecl {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(type-decl {} (members ", self.name)?;
        for (i, member) in self.members.iter().enumerate() {
            write!(f, "{}", member)?;
            if i < self.members.len() - 1 {
                write!(f, " ")?;
            }
        }
        for (i, method) in self.methods.iter().enumerate() {
            write!(f, "{}", method)?;
            if i < self.methods.len() - 1 {
                write!(f, " ")?;
            }
        }
        write!(f, "))")
    }
}

#[derive(Debug, Clone)]
pub struct Member {
    pub name: Name,
    pub ty: Type,
}

impl fmt::Display for Member {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(member {} (type {}))", self.name, self.ty)
    }
}

#[derive(Debug, Clone)]
pub struct TypeInst {
    pub name: Name,
    pub entries: Vec<(Key, Expr)>,
}

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord)]
pub enum Key {
    NamedKey(Name),
    IntKey(i32),
    BoolKey(bool),
}

impl fmt::Display for Key {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Key::NamedKey(name) => write!(f, "{}", name),
            Key::IntKey(i) => write!(f, "{}", i),
            Key::BoolKey(b) => write!(f, "{}", b),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Node {
    pub name: Name,
    pub args: Vec<Expr>,
    pub props: BTreeMap<Key, Expr>,
}

impl Node {
    pub fn new(name: Name) -> Self {
        Self { name, args: Vec::new(), props: BTreeMap::new() }
    }
}   

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(node {} (args", self.name)?;
        for (i, arg) in self.args.iter().enumerate() {
            write!(f, " {}", arg)?;
            if i < self.args.len() - 1 {
                write!(f, " ")?;
            }
        }
        if !self.props.is_empty() {
            write!(f, ") (props")?;
            for (key, expr) in self.props.iter() {
                write!(f, " (pair {} {})", key, expr)?;
            }
        }
        write!(f, "))")
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
        Self { name, model: Model::default(), view: View::default() }
    }
}

impl fmt::Display for Widget {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(widget {} {} {})", self.name, self.model, self.view)
    }
}

#[derive(Debug, Clone)]
pub struct Model {
    pub vars: Vec<Var>,
}

impl Default for Model {
    fn default() -> Self {
        Self { vars: Vec::default() }
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
    pub nodes: BTreeMap<Name, Node>,
}

impl Default for View {
    fn default() -> Self {
        Self { nodes: BTreeMap::new() }
    }
}

impl fmt::Display for View {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(view")?;
        for (name, node) in self.nodes.iter() {
            write!(f, " {}", node)?;
        }
        write!(f, ")")
    }
}

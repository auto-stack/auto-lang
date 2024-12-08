use std::fmt;
use serde::Serialize;
use autoval::Op;

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

impl Default for Name {
    fn default() -> Self {
        Self { text: "".to_string() }
    }
}

impl fmt::Display for Name {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(name {})", self.text)
    }
}

#[derive(Debug, Clone)]
pub enum StoreKind {
    Let,
    Mut,
    Var,
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

#[derive(Debug, Clone)]
pub enum Stmt {
    Expr(Expr),
    If(/*multiple branches with condition/body*/Vec<Branch>, /*else*/Option<Body>),
    For(For),
    Store(Store),
    Block(Body),
    Fn(Fn),
    TypeDecl(TypeDecl),
    Widget(Widget),
    Node(Node),
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
    Indexed(/*index*/Name, /*iter*/Name),
    Named(/*iter*/Name),
}

#[derive(Debug, Clone)]
pub struct Body {
    pub stmts: Vec<Stmt>,
    pub has_new_line: bool,
}

impl Body {
    pub fn new() -> Self {
        Self { stmts: Vec::new(), has_new_line: false }
    }

    pub fn single_expr(expr: Expr) -> Self {
        Self { stmts: vec![Stmt::Expr(expr)], has_new_line: false }
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
        Ok(())
    }
}

impl fmt::Display for Stmt {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Stmt::Expr(expr) => write!(f, "{}", expr),
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
            Stmt::For(for_stmt) => write!(f, "{}", for_stmt),
            Stmt::Block(body) => write!(f, "{}", body),
            Stmt::Store(store) => write!(f, "{}", store),
            Stmt::Fn(fn_decl) => write!(f, "{}", fn_decl),
            Stmt::TypeDecl(type_decl) => write!(f, "{}", type_decl),    
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
    Index(/*array*/Box<Expr>, /*index*/Box<Expr>),
    Lambda(Fn),
    FStr(FStr),
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
    pub fn get_name(&self) -> String {
        match &self.name.as_ref() {
            Expr::Ident(name) => name.text.clone(),
            _ => panic!("Expected identifier, got {:?}", self.name),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Args {
    pub array: Vec<Expr>,
    pub map: Vec<(Name, Expr)>,
}

impl Args {
    pub fn new() -> Self {
        Self { array: Vec::new(), map: Vec::new() }
    }

    pub fn get(&self, idx: usize) -> Option<Expr> {
        self.array.get(idx).cloned()
    }

    pub fn lookup(&self, name: &str) -> Option<Expr> {
        self.map.iter().find(|(n, _)| n.text == name).map(|(_, v)| v.clone())
    }

    pub fn is_empty(&self) -> bool {
        self.array.is_empty() && self.map.is_empty()
    }
}

impl fmt::Display for Args {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(args")?;
        fmt_array(f, &self.array)?;
        for (name, expr) in self.map.iter() {
            write!(f, " (pair {} {})", name, expr)?;
        }
        write!(f, ")")
    }
}

fn fmt_call(f: &mut fmt::Formatter, call: &Call) -> fmt::Result {
    write!(f, "(call ")?;
    write!(f, "{}", call.name)?;
    write!(f, " (args")?;
    for arg in call.args.array.iter() {
        write!(f, " {}", arg)?;
    }
    for (name, expr) in call.args.map.iter() {
        write!(f, " (pair {} {})", name, expr)?;
    }
    write!(f, ")")?;
    Ok(())
}

fn fmt_list(f: &mut fmt::Formatter, elems: &Vec<Expr>) -> fmt::Result {
    write!(f, "(list")?;
    for elem in elems.iter() {
        write!(f, " {}", elem)?;
    }
    write!(f, ")")
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
            Expr::Index(array, index) => write!(f, "(index {} {})", array, index),
            Expr::Lambda(lambda) => write!(f, "{}", lambda),
            Expr::FStr(fstr) => write!(f, "{}", fstr),
            Expr::Nil => write!(f, "(nil)"),
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
pub enum Type {
    Byte,
    Int,
    Float,
    Bool,
    Str,
    Array(ArrayType),
    User(TypeDecl),
    Unknown,
}

impl Type {
    pub fn unique_name(&self) -> String {
        match self {
            Type::User(type_decl) => type_decl.name.text.clone(),
            _ => "".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ArrayType {
    pub elem: Box<Type>,
    pub len: usize,
}

impl fmt::Display for ArrayType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(array-type (elem {}) (len {}))", &self.elem, self.len)
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Type::Byte => write!(f, "byte"),
            Type::Int => write!(f, "int"),
            Type::Float => write!(f, "float"),
            Type::Bool => write!(f, "bool"),
            Type::Str => write!(f, "str"),
            Type::Array(array_type) => write!(f, "{}", array_type),
            Type::User(type_decl) => write!(f, "{}", type_decl),
            Type::Unknown => write!(f, "unknown"),
        }
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
        write!(f, " {}", self.body)
    }
}

impl Fn {
    pub fn new(name: Name, parent: Option<Name>, params: Vec<Param>, body: Body, ret: Type) -> Fn {
        Fn { name, parent, params, body, ret }
    }

}

#[derive(Debug, Clone)]
pub struct TypeDecl {
    pub name: Name,
    pub has: Vec<Type>,
    pub members: Vec<Member>,
    pub methods: Vec<Fn>,
}

impl fmt::Display for TypeDecl {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(type-decl {}", self.name)?;
        if !self.has.is_empty() {
            write!(f, " (has ")?;
            for h in self.has.iter() {
                write!(f, "(type {})", h.unique_name())?;
            }
            write!(f, ")")?;
        }
        if !self.members.is_empty() {
            write!(f, " (members ")?;
            for (i, member) in self.members.iter().enumerate() {
                write!(f, "{}", member)?;
                if i < self.members.len() - 1 {
                    write!(f, " ")?;
                }
            }
            write!(f, ")")?;
        }
        if !self.methods.is_empty() {
            write!(f, " (methods ")?;
        }
        for (i, method) in self.methods.iter().enumerate() {
            write!(f, "{}", method)?;
            if i < self.methods.len() - 1 {
                write!(f, " ")?;
            }
        }
        if !self.methods.is_empty() {
            write!(f, ")")?;
        }
        write!(f, ")")
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
    pub entries: Vec<Pair>,
}

#[derive(Debug, Clone)]
pub struct Pair {
    pub key: Key,
    pub value: Box<Expr>,
}

impl fmt::Display for Pair {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(pair {} {})", self.key, self.value)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord)]
pub enum Key {
    NamedKey(Name),
    IntKey(i32),
    BoolKey(bool),
    StrKey(String),
}

impl fmt::Display for Key {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Key::NamedKey(name) => write!(f, "{}", name),
            Key::IntKey(i) => write!(f, "{}", i),
            Key::BoolKey(b) => write!(f, "{}", b),
            Key::StrKey(s) => write!(f, "\"{}\"", s),
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
    pub fn new(name: Name) -> Self {
        Self { name, args: Args::new(), body: Body::new() }
    }
}   

impl Into<Node> for Call {
    fn into(self) -> Node {
        let name = Name::new(self.get_name());
        let mut node = Node::new(name);
        node.args = self.args;
        node
    }
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(node {}", self.name)?;
        if !self.args.array.is_empty() {
            write!(f, " (args")?;
            for (i, arg) in self.args.array.iter().enumerate() {
                write!(f, " {}", arg)?;
                if i < self.args.array.len() - 1 {
                    write!(f, " ")?;
                }
            }
            if !self.args.map.is_empty() {
                write!(f, " ")?;
                for (name, expr) in self.args.map.iter() {
                    write!(f, " (pair {} {})", name, expr)?;
                }
            }
            write!(f, ")")?;
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
    pub vars: Vec<Store>,
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
    pub nodes: Vec<(Name, Node)>,
    pub body: Body,
}

impl Default for View {
    fn default() -> Self {
        Self { nodes: Vec::new(), body: Body::new() }
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

fn fmt_type_inst(f: &mut fmt::Formatter, name: &Box<Expr>, entries: &Vec<Pair>) -> fmt::Result {
    write!(f, "(type-inst {} ", name.as_ref())?;
    for (i, pair) in entries.iter().enumerate() {
        write!(f, "{}", pair)?;
        if i < entries.len() - 1 {
            write!(f, " ")?;
        }
    }
    write!(f, ")")
}

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
        for (i, part) in self.parts.iter().enumerate() {
            write!(f, " {}", part)?;
        }
        write!(f, ")")
    }
}

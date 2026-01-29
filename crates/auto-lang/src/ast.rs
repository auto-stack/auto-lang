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
mod ext;
pub use ext::*;
mod fun;
pub use fun::*;
mod fstr;
pub use fstr::*;
mod grid;
pub use grid::*;
mod hold;
pub use hold::*;
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
mod spec;
pub use spec::*;
mod types;
pub use types::*;
mod type_alias;
pub use type_alias::*;
mod union;
pub use union::*;
mod use_;
pub use use_::*;
mod range;
pub use range::*;

mod atom_helpers;
pub use atom_helpers::*;

mod parsers;

#[allow(hidden_glob_reexports)]
use auto_val::{AutoResult, AutoStr, Node as AutoNode, Op, Value};
use std::{fmt, io};

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
    fn to_atom(&self) -> AutoStr;
}

pub trait AtomWriter {
    fn write_atom(&self, f: &mut impl io::Write) -> AutoResult<()>;
}

/// Helper trait to convert any AtomWriter implementer to AutoStr
///
/// This trait provides a convenience method `to_atom_str()` that converts
/// any type implementing `AtomWriter` directly to `AutoStr` without needing
/// to manually manage the buffer.
pub trait ToAtomStr {
    fn to_atom_str(&self) -> AutoStr;
}

// Blanket implementation for all AtomWriter types
impl<T: AtomWriter> ToAtomStr for T {
    fn to_atom_str(&self) -> AutoStr {
        let mut buf = Vec::new();
        let _ = self.write_atom(&mut buf);
        String::from_utf8(buf).unwrap_or_default().into()
    }
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
    SpecDecl(SpecDecl),
    Node(Node),
    Use(Use),
    OnEvents(OnEvents),
    Comment(AutoStr),
    Alias(Alias),
    TypeAlias(TypeAlias),  // Type alias: type List<T> = List<T, DefaultStorage>
    EmptyLine(usize),
    Break,
    Return(Box<Expr>),  // Return statement with value
    Ext(Ext),  // Type extension (like Rust's impl)
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
            | Stmt::SpecDecl(_)
            | Stmt::Alias(_)
            | Stmt::TypeAlias(_)
            | Stmt::Ext(_) => true,  // Plan 035 Phase 5.2: Ext statement is a declaration
            _ => false,
        }
    }

    pub fn is_new_block(&self) -> bool {
        match self {
            Stmt::Block(_) | Stmt::Fn(_) | Stmt::TypeDecl(_) | Stmt::EnumDecl(_) | Stmt::SpecDecl(_) => true,
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
            Stmt::TypeAlias(type_alias) => write!(f, "{}", type_alias),
            Stmt::EmptyLine(n) => write!(f, "(nl*{})", n),
            Stmt::Union(u) => write!(f, "{}", u),
            Stmt::Tag(tag) => write!(f, "{}", tag),
            Stmt::SpecDecl(spec_decl) => write!(f, "{}", spec_decl),
            Stmt::Break => write!(f, "(break)"),
            Stmt::Return(expr) => write!(f, "(return {})", expr),
            Stmt::Ext(ext) => write!(f, "{}", ext),
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
    // Borrow expressions (Phase 3)
    View(Box<Expr>),    // Immutable borrow (like Rust &T)
    Mut(Box<Expr>),     // Mutable borrow (like Rust &mut T)
    Take(Box<Expr>),    // Move semantics (like Rust move or std::mem::take)
    Hold(Hold),         // Hold path binding (temporary borrow with syntax sugar)
    Unary(Op, Box<Expr>),
    Bina(Box<Expr>, Op, Box<Expr>),
    /// Dot expression: object.field or Type.method
    /// Used for both field access and method calls
    /// Examples:
    ///   - Field access: obj.field
    ///   - Static method: List.new()
    ///   - Instance method: list.push(1)
    Dot(Box<Expr>, Name),
    Range(Range),
    Array(Vec<Expr>),
    Pair(Pair),
    Block(Body),
    Object(Vec<Pair>),
    Call(Call),
    Node(Node),
    Index(/*array*/ Box<Expr>, /*index*/ Box<Expr>),
    Lambda(Fn),        // Named lambda function
    Closure(Closure),  // Plan 060: Closure:  x => expr or (a, b) => expr
    FStr(FStr),
    Grid(Grid),
    Cover(Cover),
    Uncover(TagUncover),
    // stmt exprs
    If(If),
    Nil,
    Null,
    // May type operators (Phase 1b.3)
    NullCoalesce(Box<Expr>, Box<Expr>),  // left ?? right
    ErrorPropagate(Box<Expr>),            // expression.?
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
            Expr::View(e) => write!(f, "({}.view)", e),
            Expr::Mut(e) => write!(f, "({}.mut)", e),
            Expr::Take(e) => write!(f, "({}.take)", e),
            Expr::Hold(hold) => write!(f, "{}", hold),
            Expr::Bina(l, op, r) => write!(f, "(bina {} {} {})", l, op, r),
            Expr::Dot(object, field) => write!(f, "(dot {}.{})", object, field),
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
            Expr::Closure(closure) => write!(f, "{}", closure),
            Expr::FStr(fstr) => write!(f, "{}", fstr),
            Expr::Grid(grid) => write!(f, "{}", grid),
            Expr::Cover(cover) => write!(f, "{}", cover),
            Expr::Uncover(uncover) => write!(f, "{}", uncover),
            Expr::GenName(name) => write!(f, "(gen-name {})", name),
            Expr::Nil => write!(f, "(nil)"),
            Expr::Null => write!(f, "(null)"),
            Expr::Range(r) => write!(f, "{}", r),
            Expr::NullCoalesce(l, r) => write!(f, "(?? {} {})", l, r),
            Expr::ErrorPropagate(e) => write!(f, "(?. {})", e),
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

impl<T: AtomWriter> AtomWriter for Vec<T> {
    fn write_atom(&self, f: &mut impl io::Write) -> AutoResult<()> {
        write!(f, "array(")?;
        for (i, elem) in self.iter().enumerate() {
            elem.write_atom(f)?;
            if i < self.len() - 1 {
                write!(f, ", ")?;
            }
        }
        write!(f, ")")?;
        Ok(())
    }
}

impl AtomWriter for Pair {
    fn write_atom(&self, f: &mut impl io::Write) -> AutoResult<()> {
        write!(
            f,
            "pair({}, {})",
            self.key.to_atom_str(),
            self.value.to_atom_str()
        )?;
        Ok(())
    }
}

impl AtomWriter for Expr {
    fn write_atom(&self, f: &mut impl io::Write) -> AutoResult<()> {
        match self {
            Expr::Int(i) => write!(f, "{}", i)?,
            Expr::Uint(u) => write!(f, "{}", u)?,
            Expr::Float(fl, _) => write!(f, "{}", fl)?,
            Expr::Bool(b) => write!(f, "{}", b)?,
            Expr::Char(c) => write!(f, "{}", c)?,
            Expr::Str(s) => write!(f, "\"{}\"", s)?,
            Expr::Ident(n) => write!(f, "{}", n)?,
            Expr::Ref(n) => write!(f, "ref({})", n)?,
            Expr::Bina(l, op, r) => {
                // Special case for dot operator (field access): output as "bina(left, right)"
                if *op == auto_val::Op::Dot {
                    write!(f, "bina({}, {})", l.to_atom_str(), r.to_atom_str())?;
                } else if *op == auto_val::Op::Asn {
                    // Assignment: output as "asn left right" not "bina(asn, left, right)"
                    write!(f, "asn {} {}", l.to_atom_str(), r.to_atom_str())?;
                } else {
                    let op_str = match op {
                        auto_val::Op::Add => "+",
                        auto_val::Op::Sub => "-",
                        auto_val::Op::Mul => "*",
                        auto_val::Op::Div => "/",
                        auto_val::Op::Eq => "==",
                        auto_val::Op::Neq => "!=",
                        auto_val::Op::Lt => "<",
                        auto_val::Op::Le => "<=",
                        auto_val::Op::Gt => ">",
                        auto_val::Op::Ge => ">=",
                        auto_val::Op::AddEq => "+=",
                        auto_val::Op::SubEq => "-=",
                        auto_val::Op::MulEq => "*=",
                        auto_val::Op::DivEq => "/=",
                        auto_val::Op::Range => "..",
                        auto_val::Op::RangeEq => "..=",
                        _ => "?",
                    };
                    write!(
                        f,
                        "bina({}, {}, {})",
                        op_str,
                        l.to_atom_str(),
                        r.to_atom_str()
                    )?;
                }
            }
            Expr::Unary(op, e) => {
                let op_str = match op {
                    auto_val::Op::Not => "!",
                    auto_val::Op::Sub => "-",
                    auto_val::Op::Mul => "*",
                    auto_val::Op::Add => "&",
                    _ => "?",
                };
                write!(f, "una({}, {})", op_str, e.to_atom_str())?;
            }
            Expr::Array(elems) => {
                // Delegate to Vec<Expr> AtomWriter which outputs [elem1, elem2, ...]
                write!(f, "{}", elems.to_atom_str())?;
            }
            Expr::Pair(pair) => {
                write!(f, "pair({}, ", pair.key.to_atom_str())?;
                pair.value.write_atom(f)?;
                write!(f, ")")?;
            }
            Expr::Object(pairs) => {
                write!(f, "obj {{")?;
                for (i, pair) in pairs.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    } else {
                        write!(f, " ")?;
                    }
                    write!(
                        f,
                        "pair({}, {})",
                        pair.key.to_atom_str(),
                        pair.value.to_atom_str()
                    )?;
                }
                write!(f, " }}")?;
            }
            Expr::FStr(fstr) => fstr.write_atom(f)?,
            Expr::Index(array, index) => {
                // Check if this is a slice expression (index with range or range with step)
                match &**index {
                    Expr::Range(range) => {
                        // Check if start is itself a Range (which means this is a slice with step)
                        if let Expr::Range(inner_range) = &*range.start {
                            // Slice with step: arr[0..10..2] parses as Range(Range(0, 10), 2)
                            let start_str = match &*inner_range.start {
                                Expr::Int(i) => i.to_string(),
                                _ => inner_range.start.to_atom_str().to_string(),
                            };
                            let end_str = match &*inner_range.end {
                                Expr::Int(i) => i.to_string(),
                                _ => inner_range.end.to_atom_str().to_string(),
                            };
                            let step_str = match &*range.end {
                                Expr::Int(i) => i.to_string(),
                                _ => range.end.to_atom_str().to_string(),
                            };
                            write!(
                                f,
                                "slice({}, {}, {}, {})",
                                array.to_atom_str(),
                                start_str,
                                end_str,
                                step_str
                            )?;
                        } else {
                            // Simple slice: arr[0..10]
                            let start_str = match &*range.start {
                                Expr::Int(i) => i.to_string(),
                                _ => range.start.to_atom_str().to_string(),
                            };
                            let end_str = match &*range.end {
                                Expr::Int(i) => i.to_string(),
                                _ => range.end.to_atom_str().to_string(),
                            };
                            write!(
                                f,
                                "slice({}, {}, {})",
                                array.to_atom_str(),
                                start_str,
                                end_str
                            )?;
                        }
                    }
                    _ => {
                        write!(f, "index({}, {})", array.to_atom_str(), index.to_atom_str())?;
                    }
                }
            }
            Expr::Lambda(lambda) => lambda.write_atom(f)?,
            Expr::Closure(closure) => closure.write_atom(f)?,
            Expr::Call(call) => call.write_atom(f)?,
            Expr::Node(node) => node.write_atom(f)?,
            Expr::Block(body) => body.write_atom(f)?,
            Expr::Range(range) => range.write_atom(f)?,
            _ => write!(f, "{}", self)?,
        }
        Ok(())
    }
}

// ToAtom implementations

impl ToAtom for Expr {
    fn to_atom(&self) -> AutoStr {
        // Use the ToAtomStr helper trait for cleaner implementation
        self.to_atom_str()
    }
}

impl ToNode for Expr {
    fn to_node(&self) -> AutoNode {
        match self {
            Expr::Byte(b) => {
                let mut node = AutoNode::new("byte");
                node.add_arg(auto_val::Arg::Pos(Value::Int(*b as i32)));
                node
            }
            Expr::Int(i) => {
                let mut node = AutoNode::new("int");
                node.add_arg(auto_val::Arg::Pos(Value::Int(*i)));
                node
            }
            Expr::Uint(u) => {
                let mut node = AutoNode::new("uint");
                node.add_arg(auto_val::Arg::Pos(Value::Uint(*u)));
                node
            }
            Expr::I8(i) => {
                let mut node = AutoNode::new("i8");
                node.add_arg(auto_val::Arg::Pos(Value::Int(*i as i32)));
                node
            }
            Expr::U8(u) => {
                let mut node = AutoNode::new("u8");
                node.add_arg(auto_val::Arg::Pos(Value::Int(*u as i32)));
                node
            }
            Expr::I64(i) => {
                let mut node = AutoNode::new("i64");
                node.add_arg(auto_val::Arg::Pos(Value::Int(*i as i32)));
                node
            }
            Expr::Float(v, _) => {
                let mut node = AutoNode::new("float");
                node.add_arg(auto_val::Arg::Pos(Value::Double(*v)));
                node
            }
            Expr::Double(v, _) => {
                let mut node = AutoNode::new("double");
                node.add_arg(auto_val::Arg::Pos(Value::Double(*v)));
                node
            }
            Expr::Bool(b) => {
                let mut node = AutoNode::new("bool");
                node.add_arg(auto_val::Arg::Pos(Value::Bool(*b)));
                node
            }
            Expr::Char(c) => {
                let mut node = AutoNode::new("char");
                node.add_arg(auto_val::Arg::Pos(Value::Int(*c as i32)));
                node
            }
            Expr::Str(s) => {
                let mut node = AutoNode::new("str");
                node.add_arg(auto_val::Arg::Pos(Value::Str(s.clone())));
                node
            }
            Expr::CStr(s) => {
                let mut node = AutoNode::new("cstr");
                node.add_arg(auto_val::Arg::Pos(Value::Str(s.clone())));
                node
            }
            Expr::Ident(n) => {
                let mut node = AutoNode::new("name");
                node.add_arg(auto_val::Arg::Pos(Value::Str(n.clone())));
                node
            }
            Expr::Ref(n) => {
                let mut node = AutoNode::new("ref");
                node.add_arg(auto_val::Arg::Pos(Value::Str(n.clone())));
                node
            }
            Expr::View(e) => {
                let mut node = AutoNode::new("view");
                node.add_kid(e.to_node());
                node
            }
            Expr::Mut(e) => {
                let mut node = AutoNode::new("mut");
                node.add_kid(e.to_node());
                node
            }
            Expr::Take(e) => {
                let mut node = AutoNode::new("take");
                node.add_kid(e.to_node());
                node
            }
            Expr::Bina(l, op, r) => {
                let mut node = AutoNode::new("bina");
                node.add_kid(l.to_node());
                node.add_arg(auto_val::Arg::Pos(Value::str(&*op.repr())));
                node.add_kid(r.to_node());
                node
            }
            Expr::Unary(op, e) => {
                let mut node = AutoNode::new("una");
                node.add_arg(auto_val::Arg::Pos(Value::str(&*op.repr())));
                node.add_kid(e.to_node());
                node
            }
            Expr::Array(elems) => {
                let mut node = AutoNode::new("array");
                for elem in elems {
                    node.add_kid(elem.to_node());
                }
                node
            }
            Expr::Pair(pair) => pair.to_node(),
            Expr::Object(pairs) => {
                let mut node = AutoNode::new("object");
                for pair in pairs {
                    node.add_kid(pair.to_node());
                }
                node
            }
            Expr::Block(body) => body.to_node(),
            Expr::If(if_) => if_.to_node(),
            Expr::Call(call) => call.to_node(),
            Expr::Node(node) => node.to_node(),
            Expr::Index(array, index) => {
                let mut node = AutoNode::new("index");
                node.add_kid(array.to_node());
                node.add_kid(index.to_node());
                node
            }
            Expr::Lambda(lambda) => lambda.to_node(),
            Expr::Closure(closure) => closure.to_node(),
            Expr::FStr(fstr) => fstr.to_node(),
            Expr::Grid(grid) => grid.to_node(),
            Expr::Cover(cover) => cover.to_node(),
            Expr::Uncover(uncover) => uncover.to_node(),
            Expr::GenName(name) => {
                let mut node = AutoNode::new("gen-name");
                node.add_arg(auto_val::Arg::Pos(Value::Str(name.clone())));
                node
            }
            Expr::Hold(hold) => {
                let mut node = AutoNode::new("hold");
                node.add_kid(hold.path.to_node());
                node.add_arg(auto_val::Arg::Pos(Value::Str(hold.name.clone())));
                node.add_kid(hold.body.to_node());
                node
            }
            Expr::Nil => AutoNode::new("nil"),
            Expr::Null => AutoNode::new("null"),
            Expr::Range(r) => r.to_node(),
            Expr::NullCoalesce(l, r) => {
                let mut node = AutoNode::new("??");
                node.add_kid(l.to_node());
                node.add_kid(r.to_node());
                node
            }
            Expr::ErrorPropagate(e) => {
                let mut node = AutoNode::new("?.");
                node.add_kid(e.to_node());
                node
            }
            Expr::Dot(object, field) => {
                let mut node = AutoNode::new("dot");
                node.add_kid(object.to_node());
                node.add_arg(auto_val::Arg::Pos(Value::str(field.as_str())));
                node
            }
        }
    }
}

impl ToNode for Stmt {
    fn to_node(&self) -> AutoNode {
        match self {
            Stmt::Expr(expr) => expr.to_node(), // Changed from expr.to_atom().to_node()
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
            Stmt::SpecDecl(spec_decl) => spec_decl.to_node(),
            Stmt::Node(node) => node.to_node(),
            Stmt::Use(use_) => use_.to_node(),
            Stmt::OnEvents(on_events) => on_events.to_node(),
            Stmt::Comment(comment) => {
                let mut node = AutoNode::new("comment");
                node.add_arg(auto_val::Arg::Pos(Value::str(comment.as_str())));
                node
            }
            Stmt::Alias(alias) => alias.to_node(),
            Stmt::TypeAlias(type_alias) => type_alias.to_node(),
            Stmt::EmptyLine(n) => {
                let mut node = AutoNode::new("nl");
                node.set_prop("count", Value::Int(*n as i32));
                node
            }
            Stmt::Break => AutoNode::new("break"),
            Stmt::Return(_) => AutoNode::new("return"),

            Stmt::Ext(ext) => ext.to_node(),
        }
    }
}

impl AtomWriter for Stmt {
    fn write_atom(&self, f: &mut impl io::Write) -> AutoResult<()> {
        // Delegate to each variant's to_atom_str
        write!(f, "{}", self.to_atom())?;
        Ok(())
    }
}

impl ToAtom for Stmt {
    fn to_atom(&self) -> AutoStr {
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
            Stmt::SpecDecl(spec_decl) => spec_decl.to_atom(),
            Stmt::Node(node) => node.to_atom(),
            Stmt::Use(use_) => use_.to_atom(),
            Stmt::OnEvents(on_events) => on_events.to_atom(),
            Stmt::Comment(comment) => comment.clone(),
            Stmt::Alias(alias) => alias.to_atom(),
            Stmt::TypeAlias(type_alias) => type_alias.to_atom(),
            Stmt::EmptyLine(n) => format!("(nl (count {}))", n).into(),
            Stmt::Break => "(break)".into(),
            Stmt::Return(expr) => format!("(return {})", expr.to_atom()).into(),
            Stmt::Ext(ext) => ext.to_atom(),
        }
    }
}

impl ToNode for Code {
    fn to_node(&self) -> AutoNode {
        let mut node = AutoNode::new("code");
        for stmt in &self.stmts {
            node.add_kid(stmt.to_node()); // Changed from stmt.to_atom().to_node()
        }
        node
    }
}

impl AtomWriter for Code {
    fn write_atom(&self, f: &mut impl io::Write) -> AutoResult<()> {
        write!(f, "(code")?;
        for stmt in &self.stmts {
            write!(f, " {}", stmt.to_atom_str())?;
        }
        write!(f, ")")?;
        Ok(())
    }
}

impl ToAtom for Code {
    fn to_atom(&self) -> AutoStr {
        self.to_atom_str()
    }
}

// ============================================================
// Markdown Test Infrastructure for to_atom() tests
// ============================================================

#[cfg(test)]
mod markdown_tests {
    use super::*;
    use crate::parser::Parser;
    use crate::Universe;
    use std::cell::RefCell;
    use std::fs;
    use std::path::Path;
    use std::rc::Rc;

    #[derive(Debug)]
    struct TestCase {
        name: String,
        input: String,
        expected: String,
    }

    fn parse_markdown_tests(content: &str) -> Vec<TestCase> {
        let mut cases = Vec::new();
        let lines: Vec<&str> = content.lines().collect();
        let mut i = 0;

        while i < lines.len() {
            // Look for "##" to start a test case
            if lines[i].starts_with("## ") {
                let name = lines[i][3..].trim().to_string();
                i += 1;

                // Skip empty lines
                while i < lines.len() && lines[i].trim().is_empty() {
                    i += 1;
                }

                // Read input code until we hit "---"
                let mut input = String::new();
                while i < lines.len() && !lines[i].starts_with("---") {
                    if !input.is_empty() {
                        input.push('\n');
                    }
                    input.push_str(lines[i]);
                    i += 1;
                }

                // Skip "---" line
                if i < lines.len() && lines[i].starts_with("---") {
                    i += 1;
                }

                // Skip empty lines
                while i < lines.len() && lines[i].trim().is_empty() {
                    i += 1;
                }

                // Read expected output until next "##" or end
                let mut expected = String::new();
                while i < lines.len() && !lines[i].starts_with("##") {
                    if !expected.is_empty() {
                        expected.push('\n');
                    }
                    expected.push_str(lines[i]);
                    i += 1;
                }

                cases.push(TestCase {
                    name,
                    input: input.trim().to_string(),
                    expected: expected.trim().to_string(),
                });
            } else {
                i += 1;
            }
        }

        cases
    }

    /// Format ATOM output with proper indentation and newlines for readability
    #[allow(dead_code)]
    fn pretty_atom(atom: &str) -> String {
        let mut result = String::new();
        let mut indent = 0;
        let mut in_braces = false;
        let mut paren_depth = 0; // Track parenthesis depth
        let mut in_string = false; // Track if we're inside a string literal
        let mut chars = atom.chars().peekable();

        while let Some(c) = chars.next() {
            match c {
                '\n' => {
                    if !in_string {
                        // Check if next word is "else" and add proper indentation
                        let mut lookahead = chars.clone();
                        while let Some(next) = lookahead.next() {
                            if next == ' ' {
                                continue;
                            } else if next.is_alphabetic() {
                                // Check if this spells "else"
                                let mut word = String::new();
                                word.push(next);
                                let mut more_lookahead = lookahead.clone();
                                let mut found_else = true;
                                for expected in "else".chars().skip(1) {
                                    if let Some(next_char) = more_lookahead.next() {
                                        if next_char == expected {
                                            word.push(next_char);
                                        } else {
                                            found_else = false;
                                            break;
                                        }
                                    } else {
                                        found_else = false;
                                        break;
                                    }
                                }
                                if found_else {
                                    result.push('\n');
                                    for _ in 0..indent {
                                        result.push(' ');
                                    }
                                } else {
                                    result.push(c);
                                }
                                break;
                            } else {
                                result.push(c);
                                break;
                            }
                        }
                    } else {
                        result.push(c);
                    }
                }
                '"' => {
                    result.push(c);
                    in_string = !in_string;
                }
                '{' => {
                    if !in_string {
                        in_braces = true;
                        result.push(c);
                        // Add newline and indentation after opening brace
                        result.push('\n');
                        indent += 4;
                        for _ in 0..indent {
                            result.push(' ');
                        }
                        // Consume any spaces that immediately follow { to avoid double spacing
                        while let Some(&next) = chars.peek() {
                            if next == ' ' {
                                chars.next();
                            } else {
                                break;
                            }
                        }
                    } else {
                        result.push(c);
                    }
                }
                '}' => {
                    if in_string {
                        result.push(c);
                    } else if in_braces {
                        // Only trim trailing whitespace on the current line
                        while result.ends_with(' ') || result.ends_with('\n') {
                            result.pop();
                        }
                        result.push('\n');
                        indent -= 4;
                        for _ in 0..indent {
                            result.push(' ');
                        }
                        result.push(c);
                        in_braces = false;
                    } else {
                        result.push(c);
                    }
                }
                '(' => {
                    result.push(c);
                    if !in_string {
                        paren_depth += 1;
                    }
                }
                ')' => {
                    result.push(c);
                    if !in_string {
                        paren_depth -= 1;
                        // Add newline after ) if we're in braces and not inside another function call
                        // Check if next chars are space followed by alphabetic (start of new statement)
                        if in_braces && paren_depth == 0 {
                            if let Some(&next) = chars.peek() {
                                if next.is_alphabetic() {
                                    // Case: `)call` - direct alphabetic after )
                                    result.push('\n');
                                    for _ in 0..indent {
                                        result.push(' ');
                                    }
                                } else if next == ' ' {
                                    // Case: `) call` - space after ), check if it's followed by alphabetic
                                    // Peek ahead to see what comes after the space
                                    let mut chars_copy = chars.clone();
                                    chars_copy.next(); // Skip the space
                                    if let Some(&after_space) = chars_copy.peek() {
                                        if after_space.is_alphabetic() {
                                            // This is `) <space> <alphabetic>` - skip the space and add newline
                                            chars.next(); // Consume the space
                                            result.push('\n');
                                            for _ in 0..indent {
                                                result.push(' ');
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                ',' => {
                    result.push(c);
                    // Add newline after comma if we're in braces AND not inside parens or strings
                    // But if we're inside parens (like in function calls), add a space instead
                    if in_braces && !in_string && paren_depth == 0 {
                        result.push('\n');
                        for _ in 0..indent {
                            result.push(' ');
                        }
                    } else if !in_string {
                        // Only add space if next char is not already a space
                        if let Some(&next) = chars.peek() {
                            if next != ' ' && next != '\n' {
                                result.push(' ');
                            }
                        } else {
                            result.push(' ');
                        }
                    }
                }
                ' ' => {
                    // Preserve spaces inside strings
                    if in_string {
                        result.push(c);
                    } else if !result.ends_with('\n') && !result.is_empty() {
                        if let Some(&next) = chars.peek() {
                            // Skip space if next char is certain punctuation
                            if next != ' ' && next != '}' && next != ',' {
                                result.push(c);
                            }
                        }
                    }
                }
                _ => {
                    result.push(c);
                }
            }
        }

        result.trim().to_string()
    }

    fn run_markdown_test_file(path: &str) {
        let full_path = Path::new("test/ast").join(path);
        let content = fs::read_to_string(&full_path)
            .unwrap_or_else(|_| panic!("Failed to read test file: {:?}", full_path));

        let cases = parse_markdown_tests(&content);

        for tc in cases {
            let scope = Rc::new(RefCell::new(Universe::new()));
            let mut parser = Parser::new(&tc.input, scope.clone());
            let code = parser
                .parse()
                .unwrap_or_else(|e| panic!("{}: Parse failed: {:?}", tc.name, e));

            // Extract the first statement's atom representation, or join multiple statements with "; "
            let actual = if code.stmts.len() == 1 {
                code.stmts[0].to_atom()
            } else {
                // Join multiple statements with "; " instead of wrapping in (code ...)
                let stmts: Vec<AutoStr> =
                    code.stmts.iter().map(|stmt| stmt.to_atom_str()).collect();
                AutoStr::from(stmts.join("; "))
            };

            // let actual_normalized = pretty_atom(&actual.replace("\r\n", "\n"));
            let actual_normalized = actual.replace("\r\n", "\n");
            let expected_normalized = tc.expected.replace("\r\n", "\n");

            if actual_normalized != expected_normalized {
                // Debug: show the difference
                eprintln!("Comparing:");
                eprintln!(
                    "  Expected len: {}, repr: {:?}",
                    expected_normalized.len(),
                    expected_normalized
                );
                eprintln!(
                    "  Actual len: {}, repr: {:?}",
                    actual_normalized.len(),
                    actual_normalized
                );
                panic!(
                    "\nTest '{}' failed:\nInput:\n{}\n\nExpected:\n{}\n\nActual:\n{}\n",
                    tc.name, tc.input, expected_normalized, actual_normalized
                );
            }
        }
    }

    #[test]
    fn test_01_literals() {
        run_markdown_test_file("01_literals.test.md");
    }

    #[test]
    fn test_02_exprs() {
        run_markdown_test_file("02_exprs.test.md");
    }

    #[test]
    fn test_03_functions() {
        run_markdown_test_file("03_functions.test.md");
    }

    #[test]
    fn test_04_controls() {
        run_markdown_test_file("04_controls.test.md");
    }

    #[test]
    fn test_05_types() {
        run_markdown_test_file("05_types.test.md");
    }

    // TODO: Uncomment when test files are ready
    // #[test]
    // fn test_06_declarations() {
    //     run_markdown_test_file("06_declarations.test.md");
    // }

    // #[test]
    // fn test_07_advanced_control() {
    //     run_markdown_test_file("07_advanced_control.test.md");
    // }

    // #[test]
    // fn test_08_statements() {
    //     run_markdown_test_file("08_statements.test.md");
    // }

    // #[test]
    // fn test_09_events() {
    //     run_markdown_test_file("09_events.test.md");
    // }

    // #[test]
    // fn test_10_complex_cases() {
    //     run_markdown_test_file("10_complex_cases.test.md");
    // }

    // #[test]
    // fn test_11_more_expressions() {
    //     run_markdown_test_file("11_more_expressions.test.md");
    // }

    /// Remove ANSI color codes from string for testing
    fn strip_ansi_codes(s: &str) -> String {
        // This regex matches ANSI escape sequences like \x1b[...m or \033[...m
        let ansi_regex = regex::Regex::new(r"\x1b\[[0-9;]*m").unwrap();
        ansi_regex.replace_all(s, "").to_string()
    }

    /// Run markdown tests that expect errors
    fn run_markdown_error_test_file(path: &str) {
        let full_path = Path::new("test/ast").join(path);
        let content = fs::read_to_string(&full_path)
            .unwrap_or_else(|_| panic!("Failed to read test file: {:?}", full_path));

        let cases = parse_markdown_tests(&content);

        for tc in cases {
            let scope = Rc::new(RefCell::new(Universe::new()));
            let mut parser = Parser::new(&tc.input, scope.clone());

            match parser.parse() {
                Ok(_) => {
                    panic!(
                        "Test '{}' expected an error but parsing succeeded.\nInput:\n{}",
                        tc.name, tc.input
                    );
                }
                Err(err) => {
                    // Convert error to miette report and format it
                    let report = miette::Report::new(err);
                    let error_output = format!("{:?}", report);

                    // Strip ANSI color codes for reliable string comparison
                    let error_clean = strip_ansi_codes(&error_output);

                    // Check if the error output contains expected text
                    // Normalize both for comparison (handle line endings)
                    let error_normalized = error_clean.replace("\r\n", "\n");
                    let expected_normalized = tc.expected.replace("\r\n", "\n");

                    // Check if all expected lines are present in the error output
                    for expected_line in expected_normalized.lines() {
                        let trimmed = expected_line.trim();
                        if !trimmed.is_empty() && !error_normalized.contains(trimmed) {
                            eprintln!("Error output:\n{}", error_normalized);
                            panic!(
                                "\nTest '{}' failed:\nInput:\n{}\n\nExpected to find:\n{}\n\nIn error output:\n{}\n",
                                tc.name, tc.input, trimmed, error_normalized
                            );
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn test_06_errors() {
        run_markdown_error_test_file("06_errors.test.md");
    }
}

use super::{Expr, Fn, Name};
use auto_val::{AutoStr, Shared};
use std::{fmt, str::Bytes};

#[derive(Debug, Clone)]
pub enum Type {
    Byte,
    Int,
    Float,
    Double,
    Bool,
    Char,
    Str,
    CStr,
    Array(ArrayType),
    Ptr(PtrType),
    User(TypeDecl),
    Void,
    Unknown,
}

impl Type {
    pub fn unique_name(&self) -> AutoStr {
        match self {
            Type::Int => "int".into(),
            Type::Float => "float".into(),
            Type::Bool => "bool".into(),
            Type::Byte => "byte".into(),
            Type::Char => "char".into(),
            Type::Str => "str".into(),
            Type::CStr => "cstr".into(),
            Type::Array(array_type) => {
                format!("[{}]{}", array_type.elem.unique_name(), array_type.len).into()
            }
            Type::Ptr(ptr_type) => format!("*{}", ptr_type.of.borrow().unique_name()).into(),
            Type::User(type_decl) => type_decl.name.clone(),
            _ => "undefined_name".into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PtrType {
    pub of: Shared<Type>,
}

impl fmt::Display for PtrType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(ptr-type (of {}))", &self.of.borrow())
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
            Type::Double => write!(f, "double"),
            Type::Bool => write!(f, "bool"),
            Type::Char => write!(f, "char"),
            Type::Str => write!(f, "str"),
            Type::CStr => write!(f, "cstr"),
            Type::Array(array_type) => write!(f, "{}", array_type),
            Type::Ptr(ptr_type) => write!(f, "{}", ptr_type),
            Type::User(type_decl) => write!(f, "{}", type_decl),
            Type::Void => write!(f, "void"),
            Type::Unknown => write!(f, "unknown"),
        }
    }
}

impl From<Type> for auto_val::Type {
    fn from(ty: Type) -> Self {
        match ty {
            Type::Byte => auto_val::Type::Byte,
            Type::Int => auto_val::Type::Int,
            Type::Float => auto_val::Type::Float,
            Type::Double => auto_val::Type::Double,
            Type::Bool => auto_val::Type::Bool,
            Type::Char => auto_val::Type::Char,
            Type::Str => auto_val::Type::Str,
            Type::CStr => auto_val::Type::CStr,
            Type::Array(_) => auto_val::Type::Array,
            Type::Ptr(_) => auto_val::Type::Ptr,
            Type::User(decl) => auto_val::Type::User(decl.name),
            Type::Void => auto_val::Type::Void,
            Type::Unknown => auto_val::Type::Void, // TODO: is this correct?
        }
    }
}

// currently, spec is just a name
pub type Spec = AutoStr;

#[derive(Debug, Clone)]
pub struct TypeDecl {
    pub name: Name,
    pub has: Vec<Type>,
    pub specs: Vec<Spec>,
    pub members: Vec<Member>,
    pub methods: Vec<Fn>,
}

impl fmt::Display for TypeDecl {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(type-decl (name {})", self.name)?;
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
    pub value: Option<Expr>,
}

impl fmt::Display for Member {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(member (name {}) (type {})", self.name, self.ty)?;
        if let Some(value) = &self.value {
            write!(f, " (value {})", value)?;
        }
        write!(f, ")")
    }
}

impl Member {
    pub fn new(name: Name, ty: Type, value: Option<Expr>) -> Self {
        Self { name, ty, value }
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

impl Pair {
    pub fn repr(&self) -> String {
        format!("{}:{}", self.key.to_string(), self.value.repr())
    }
}

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord)]
pub enum Key {
    NamedKey(Name),
    IntKey(i32),
    BoolKey(bool),
    StrKey(AutoStr),
}

impl From<Key> for Expr {
    fn from(key: Key) -> Self {
        match key {
            Key::NamedKey(name) => Expr::Ident(name),
            Key::IntKey(i) => Expr::Int(i),
            Key::BoolKey(b) => Expr::Bool(b),
            Key::StrKey(s) => Expr::Str(s),
        }
    }
}

impl fmt::Display for Key {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Key::NamedKey(name) => write!(f, "(name {})", name),
            Key::IntKey(i) => write!(f, "{}", i),
            Key::BoolKey(b) => write!(f, "{}", b),
            Key::StrKey(s) => write!(f, "\"{}\"", s),
        }
    }
}

impl Key {
    pub fn name(&self) -> Option<&str> {
        match self {
            Key::NamedKey(name) => Some(&name),
            Key::StrKey(s) => Some(s),
            _ => None,
        }
    }
}

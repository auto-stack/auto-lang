use super::{Expr, Fn, Name};
use auto_val::AutoStr;
use std::fmt;

#[derive(Debug, Clone)]
pub enum Type {
    Byte,
    Int,
    Float,
    Bool,
    Char,
    Str,
    Array(ArrayType),
    User(TypeDecl),
    Void,
    Unknown,
}

impl Type {
    pub fn unique_name(&self) -> AutoStr {
        match self {
            Type::User(type_decl) => type_decl.name.text.clone(),
            _ => "".into(),
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
            Type::Char => write!(f, "char"),
            Type::Str => write!(f, "str"),
            Type::Array(array_type) => write!(f, "{}", array_type),
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
            Type::Bool => auto_val::Type::Bool,
            Type::Char => auto_val::Type::Char,
            Type::Str => auto_val::Type::Str,
            Type::Array(_) => auto_val::Type::Array,
            Type::User(decl) => auto_val::Type::User(decl.name.text),
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
    pub value: Option<Expr>,
}

impl fmt::Display for Member {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(member {} (type {})", self.name, self.ty)?;
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

impl Key {
    pub fn name(&self) -> Option<&str> {
        match self {
            Key::NamedKey(name) => Some(&name.text),
            Key::StrKey(s) => Some(s),
            _ => None,
        }
    }
}

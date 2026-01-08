use crate::ast::{AtomWriter, EnumDecl, ToAtomStr};

use super::{Expr, Fn, Name, Tag, Union};
use auto_val::{AutoStr, Shared};
use std::{fmt, io as stdio};

#[derive(Debug, Clone)]
pub enum Type {
    Byte,
    Int,
    Uint,
    USize,
    Float,
    Double,
    Bool,
    Char, // char is actually u8/ubyte
    Str(usize),
    CStr,
    Array(ArrayType),
    Ptr(PtrType),
    User(TypeDecl),
    Union(Union),
    Tag(Shared<Tag>),
    Enum(Shared<EnumDecl>),
    Void,
    Unknown,
    CStruct(TypeDecl),
}

impl Type {
    pub fn unique_name(&self) -> AutoStr {
        match self {
            Type::Int => "int".into(),
            Type::Uint => "uint".into(),
            Type::USize => "usize".into(),
            Type::Float => "float".into(),
            Type::Bool => "bool".into(),
            Type::Byte => "byte".into(),
            Type::Char => "char".into(),
            Type::Str(_) => "str".into(),
            Type::CStr => "cstr".into(),
            Type::Array(array_type) => {
                format!("[{}]{}", array_type.elem.unique_name(), array_type.len).into()
            }
            Type::Ptr(ptr_type) => format!("*{}", ptr_type.of.borrow().unique_name()).into(),
            Type::User(type_decl) => type_decl.name.clone(),
            Type::Enum(enum_decl) => enum_decl.borrow().name.clone(),
            Type::CStruct(type_decl) => format!("struct {}", type_decl.name).into(),
            Type::Void => "void".into(),
            Type::Unknown => "<unknown>".into(),
            _ => format!("undefined_type_name<{}>", self).into(),
        }
    }

    pub fn default_value(&self) -> AutoStr {
        match self {
            Type::Int => "0".into(),
            Type::Uint => "0".into(),
            Type::USize => "0".into(),
            Type::Float => "0.0".into(),
            Type::Bool => "false".into(),
            Type::Byte => "0".into(),
            Type::Char => "0".into(),
            Type::Str(_) => "\"\"".into(),
            Type::CStr => "\"\"".into(),
            Type::Array(_) => "[]".into(),
            Type::Ptr(ptr_type) => format!("*{}", ptr_type.of.borrow().default_value()).into(),
            Type::User(_) => "{}".into(),
            Type::Enum(enum_decl) => enum_decl.borrow().default_value().to_string().into(),
            Type::CStruct(_) => "{}".into(),
            Type::Unknown => "<unknown>".into(),
            _ => "<unknown_type>".into(),
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
            Type::Uint => write!(f, "uint"),
            Type::USize => write!(f, "usize"),
            Type::Float => write!(f, "float"),
            Type::Double => write!(f, "double"),
            Type::Bool => write!(f, "bool"),
            Type::Char => write!(f, "char"),
            Type::Str(_) => write!(f, "str"),
            Type::CStr => write!(f, "cstr"),
            Type::Array(array_type) => write!(f, "{}", array_type),
            Type::Ptr(ptr_type) => write!(f, "{}", ptr_type),
            Type::User(type_decl) => write!(f, "{}", type_decl),
            Type::Enum(enum_decl) => write!(f, "{}", enum_decl.borrow()),
            Type::Union(u) => write!(f, "{}", u),
            Type::Tag(t) => write!(f, "{}", t.borrow()),
            Type::Void => write!(f, "void"),
            Type::Unknown => write!(f, "unknown"),
            Type::CStruct(type_decl) => write!(f, "struct {}", type_decl.name),
        }
    }
}

impl From<Type> for auto_val::Type {
    fn from(ty: Type) -> Self {
        match ty {
            Type::Byte => auto_val::Type::Byte,
            Type::Int => auto_val::Type::Int,
            Type::Uint => auto_val::Type::Uint,
            Type::USize => auto_val::Type::Uint,
            Type::Float => auto_val::Type::Float,
            Type::Double => auto_val::Type::Double,
            Type::Bool => auto_val::Type::Bool,
            Type::Char => auto_val::Type::Char,
            Type::Str(_) => auto_val::Type::Str,
            Type::CStr => auto_val::Type::CStr,
            Type::Array(_) => auto_val::Type::Array,
            Type::Ptr(_) => auto_val::Type::Ptr,
            Type::User(decl) => auto_val::Type::User(decl.name),
            Type::Enum(decl) => auto_val::Type::Enum(decl.borrow().name.clone()),
            Type::Union(u) => auto_val::Type::Union(u.name),
            Type::Tag(t) => auto_val::Type::Tag(t.borrow().name.clone()),
            Type::Void => auto_val::Type::Void,
            Type::Unknown => auto_val::Type::Void, // TODO: is this correct?
            Type::CStruct(_) => auto_val::Type::Void,
        }
    }
}

// currently, spec is just a name
pub type Spec = AutoStr;

#[derive(Debug, Clone)]
pub enum TypeDeclKind {
    UserType,
    CType,
}

#[derive(Debug, Clone)]
pub struct TypeDecl {
    pub name: Name,
    pub kind: TypeDeclKind,
    pub has: Vec<Type>,
    pub specs: Vec<Spec>,
    pub members: Vec<Member>,
    pub methods: Vec<Fn>,
}

impl TypeDecl {
    pub fn find_member(&self, name: &str) -> Option<&Member> {
        self.members.iter().find(|m| m.name == name)
    }

    pub fn has_method(&self, name: &str) -> bool {
        self.methods.iter().find(|m| m.name == name).is_some()
    }
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

    pub fn to_astr(&self) -> AutoStr {
        match self {
            Key::StrKey(s) => s.clone(),
            Key::NamedKey(name) => name.clone(),
            Key::BoolKey(b) => {
                if *b {
                    "true".into()
                } else {
                    "false".into()
                }
            }
            Key::IntKey(i) => i.to_string().into(),
        }
    }
}

// ToAtom and ToNode implementations

use crate::ast::{ToAtom, ToNode};
use auto_val::{Node, Value, ValueKey};

impl ToNode for Type {
    fn to_node(&self) -> Node {
        let mut node = Node::new("type");
        node.set_prop("name", Value::str(self.unique_name().as_str()));
        node
    }
}

impl AtomWriter for Type {
    fn write_atom(&self, f: &mut impl stdio::Write) -> auto_val::AutoResult<()> {
        match self {
            Type::Byte => write!(f, "byte")?,
            Type::Int => write!(f, "int")?,
            Type::Uint => write!(f, "uint")?,
            Type::USize => write!(f, "usize")?,
            Type::Float => write!(f, "float")?,
            Type::Double => write!(f, "double")?,
            Type::Bool => write!(f, "bool")?,
            Type::Char => write!(f, "char")?,
            Type::Str(_) => write!(f, "str")?,
            Type::CStr => write!(f, "cstr")?,
            Type::Array(array_type) => {
                write!(
                    f,
                    "array({}, {})",
                    array_type.elem.to_atom_str(),
                    array_type.len
                )?;
            }
            Type::Ptr(ptr_type) => {
                write!(f, "ptr({})", ptr_type.of.borrow().to_atom_str())?;
            }
            Type::User(type_decl) => write!(f, "{}", type_decl.name)?,
            Type::Enum(enum_decl) => write!(f, "{}", enum_decl.borrow().name)?,
            Type::Union(u) => write!(f, "{}", u.name)?,
            Type::Tag(t) => write!(f, "{}", t.borrow().name)?,
            Type::Void => write!(f, "void")?,
            Type::Unknown => write!(f, "unknown")?,
            Type::CStruct(type_decl) => write!(f, "struct {}", type_decl.name)?,
        }
        Ok(())
    }
}

impl ToAtom for Type {
    fn to_atom(&self) -> AutoStr {
        self.to_atom_str()
    }
}

impl ToAtom for Key {
    fn to_atom(&self) -> AutoStr {
        self.to_atom_str()
    }
}

impl AtomWriter for Key {
    fn write_atom(&self, f: &mut impl stdio::Write) -> auto_val::AutoResult<()> {
        match self {
            Key::NamedKey(name) => write!(f, "{}", name)?,
            Key::IntKey(i) => write!(f, "{}", i)?,
            Key::BoolKey(b) => write!(f, "{}", b)?,
            Key::StrKey(s) => write!(f, "\"{}\"", s)?,
        }
        Ok(())
    }
}

impl ToNode for Key {
    fn to_node(&self) -> Node {
        match self {
            Key::NamedKey(name) => {
                let mut node = Node::new("name");
                node.add_arg(auto_val::Arg::Pos(Value::Str(name.clone())));
                node
            }
            Key::IntKey(i) => {
                let mut node = Node::new("int");
                node.add_arg(auto_val::Arg::Pos(Value::Int(*i)));
                node
            }
            Key::BoolKey(b) => {
                let mut node = Node::new("bool");
                node.add_arg(auto_val::Arg::Pos(Value::Bool(*b)));
                node
            }
            Key::StrKey(s) => {
                let mut node = Node::new("str");
                node.add_arg(auto_val::Arg::Pos(Value::Str(s.clone())));
                node
            }
        }
    }
}

impl ToAtom for Pair {
    fn to_atom(&self) -> AutoStr {
        self.to_atom_str()
    }
}

// Note: AtomWriter for Pair is already implemented in ast.rs with format "key:value"

impl ToNode for Pair {
    fn to_node(&self) -> Node {
        let mut node = Node::new("pair");
        node.add_kid(self.key.to_node());
        node.add_kid(self.value.to_node());
        node
    }
}

impl AtomWriter for Member {
    fn write_atom(&self, f: &mut impl stdio::Write) -> auto_val::AutoResult<()> {
        write!(
            f,
            "member(name(\"{}\"), type({}))",
            self.name,
            self.ty.to_atom_str()
        )?;
        if let Some(value) = &self.value {
            write!(f, ", value({})", value.to_atom_str())?;
        }
        write!(f, ")")?;
        Ok(())
    }
}

impl ToNode for Member {
    fn to_node(&self) -> Node {
        let mut node = Node::new("member");
        node.set_prop("name", Value::str(self.name.as_str()));
        node.set_prop("type", Value::str(&*self.ty.to_atom()));
        if let Some(value) = &self.value {
            node.set_prop("value", Value::str(&*value.to_atom()));
        }
        node
    }
}

impl ToAtom for Member {
    fn to_atom(&self) -> AutoStr {
        self.to_atom_str()
    }
}

impl AtomWriter for TypeDecl {
    fn write_atom(&self, f: &mut impl stdio::Write) -> auto_val::AutoResult<()> {
        write!(f, "type-decl(name(\"{}\"))", self.name)?;

        for member in &self.members {
            write!(f, " {}", member.to_atom_str())?;
        }

        write!(f, ")")?;
        Ok(())
    }
}

impl ToNode for TypeDecl {
    fn to_node(&self) -> Node {
        let mut node = Node::new("type-decl");
        node.set_prop("name", Value::str(self.name.as_str()));
        node.set_prop(
            "kind",
            Value::str(match &self.kind {
                TypeDeclKind::UserType => "user",
                TypeDeclKind::CType => "c",
            }),
        );

        if !self.has.is_empty() {
            let has_types: Vec<Value> =
                self.has.iter().map(|t| Value::str(&*t.to_atom())).collect();
            node.set_prop("has", Value::array(auto_val::Array::from_vec(has_types)));
        }

        for member in &self.members {
            node.add_kid(member.to_node());
        }

        for method in &self.methods {
            node.add_kid(method.to_node());
        }

        node
    }
}

impl ToAtom for TypeDecl {
    fn to_atom(&self) -> AutoStr {
        self.to_atom_str()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_to_atom_int() {
        let ty = Type::Int;
        let atom = ty.to_atom();
        assert_eq!(atom, "int");
    }

    #[test]
    fn test_type_to_atom_bool() {
        let ty = Type::Bool;
        let atom = ty.to_atom();
        assert_eq!(atom, "bool");
    }

    #[test]
    fn test_key_to_atom_named() {
        let key = Key::NamedKey("x".into());
        let atom = key.to_atom();
        assert!(
            atom.contains("x"),
            "Expected atom to contain 'x', got: {}",
            atom
        );
    }

    #[test]
    fn test_key_to_atom_int() {
        let key = Key::IntKey(42);
        let atom = key.to_atom();
        assert!(
            atom.contains("42"),
            "Expected atom to contain '42', got: {}",
            atom
        );
    }

    #[test]
    fn test_pair_to_atom() {
        let pair = Pair {
            key: Key::NamedKey("x".into()),
            value: Box::new(Expr::Int(42)),
        };
        let atom = pair.to_atom();
        // Should be in format "(name x):(int 42)"
        assert!(
            atom.contains("x"),
            "Expected atom to contain 'x', got: {}",
            atom
        );
        assert!(
            atom.contains("42"),
            "Expected atom to contain '42', got: {}",
            atom
        );
    }

    #[test]
    fn test_member_to_atom() {
        let member = Member {
            name: "x".into(),
            ty: Type::Int,
            value: Some(Expr::Int(42)),
        };
        let atom = member.to_atom();
        // Should be in format "(member (name x) (type int) (value int(42)))"
        assert!(
            atom.contains("member"),
            "Expected atom to contain 'member', got: {}",
            atom
        );
        assert!(
            atom.contains("x"),
            "Expected atom to contain 'x', got: {}",
            atom
        );
        assert!(
            atom.contains("int"),
            "Expected atom to contain 'int', got: {}",
            atom
        );
    }

    #[test]
    fn test_type_decl_to_atom() {
        let type_decl = TypeDecl {
            name: "Point".into(),
            kind: TypeDeclKind::UserType,
            has: vec![],
            specs: vec![],
            members: vec![Member::new("x".into(), Type::Int, None)],
            methods: vec![],
        };
        let atom = type_decl.to_atom();
        // Should be in format "(type-decl (name Point) (kind user) ...)"
        assert!(
            atom.contains("type-decl"),
            "Expected atom to contain 'type-decl', got: {}",
            atom
        );
        assert!(
            atom.contains("Point"),
            "Expected atom to contain 'Point', got: {}",
            atom
        );
    }
}

use crate::ast::{AtomWriter, EnumDecl, SpecDecl, ToAtomStr};

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
    StrSlice,  // Borrowed string slice (Phase 3)
    Array(ArrayType),         // [N]T - static array (compile-time size)
    RuntimeArray(RuntimeArrayType),  // [expr]T - runtime-sized array (Plan 052)
    List(Box<Type>),          // List<T> - dynamic list
    Slice(SliceType),         // []T - slice type
    Ptr(PtrType),             // *T - raw pointer (Plan 052)
    Reference(Box<Type>),     // &T - reference (Plan 052, for Rust transpiler)
    User(TypeDecl),
    Union(Union),
    Tag(Shared<Tag>),
    Enum(Shared<EnumDecl>),
    Spec(Shared<SpecDecl>),  // Spec 类型（多态接口）
    // May(Box<Type>) removed - use generic tag May<T> from stdlib instead
    GenericInstance(GenericInstance),  // User-defined generic instance (e.g., MyType<int>)
    Storage(StorageType),  // Storage strategy type (Plan 055)
    Void,
    Unknown,
    CStruct(TypeDecl),
    Linear(Box<Type>),  // Linear type (move-only semantics)
    Variadic,  // C variadic functions (...)
}

impl Type {
    pub fn unique_name(&self) -> AutoStr {
        match self {
            Type::Int => "int".into(),
            Type::Uint => "uint".into(),
            Type::USize => "usize".into(),
            Type::Float => "float".into(),
            Type::Double => "double".into(),
            Type::Bool => "bool".into(),
            Type::Byte => "byte".into(),
            Type::Char => "char".into(),
            Type::Str(_) => "str".into(),
            Type::CStr => "cstr".into(),
            Type::StrSlice => "str_slice".into(),
            Type::Array(array_type) => {
                format!("[{}]{}", array_type.elem.unique_name(), array_type.len).into()
            }
            Type::RuntimeArray(rta) => {
                format!("[runtime:{}]{}", rta.elem.unique_name(), rta.size_expr.repr()).into()
            }
            Type::List(elem) => format!("List<{}>", elem.unique_name()).into(),
            Type::Slice(slice_type) => format!("[]{}", slice_type.elem.unique_name()).into(),
            Type::Storage(storage) => storage.to_string().into(),
            Type::Ptr(ptr_type) => format!("*{}", ptr_type.of.borrow().unique_name()).into(),
            Type::Reference(inner) => format!("&{}", inner.unique_name()).into(),  // Plan 052
            Type::User(type_decl) => type_decl.name.clone(),
            Type::Enum(enum_decl) => enum_decl.borrow().name.clone(),
            Type::Spec(spec_decl) => spec_decl.borrow().name.clone(),
            Type::GenericInstance(inst) => {
                let args: Vec<String> = inst.args.iter()
                    .map(|t| t.unique_name().to_string())
                    .collect();
                format!("{}<{}>", inst.base_name, args.join(", ")).into()
            }
            Type::CStruct(type_decl) => format!("struct {}", type_decl.name).into(),
            Type::Linear(inner) => format!("linear<{}>", inner.unique_name()).into(),
            Type::Variadic => "...".into(),
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
            Type::StrSlice => "\"\"".into(),  // Default empty slice
            Type::Array(_) => "[]".into(),
            Type::RuntimeArray(_) => "[runtime]".into(),  // Runtime array placeholder
            Type::List(_) => "List.new()".into(),  // Empty list constructor
            Type::Slice(_) => "[]".into(),  // Empty slice literal
            Type::Ptr(ptr_type) => format!("*{}", ptr_type.of.borrow().default_value()).into(),
            Type::Reference(inner) => format!("&{}", inner.default_value()).into(),  // Plan 052
            Type::User(_) => "{}".into(),
            Type::Enum(enum_decl) => enum_decl.borrow().default_value().to_string().into(),
            Type::Spec(_) => "{}".into(),  // Spec 默认值为空对象
            Type::GenericInstance(_) => "{}".into(),  // Generic instances default to empty object
            Type::Storage(_) => "Storage".into(),  // Storage type default
            Type::Linear(inner) => inner.default_value(),  // Linear type wraps inner type
            Type::Variadic => "...".into(),  // Variadic has no default value
            Type::CStruct(_) => "{}".into(),
            Type::Unknown => "<unknown>".into(),
            _ => "<unknown_type>".into(),
        }
    }

    /// Substitute type parameters with concrete types
    ///
    /// # Examples
    /// - `T` replace with `int` → `int`
    /// - `List<T>` replace `T` with `int` → `List<int>`
    /// - `May<T>` replace `T` with `string` → `May<string>`
    ///
    /// # Arguments
    /// * `params` - Slice of type parameter names to replace (e.g., ["T", "K"])
    /// * `args` - Slice of concrete types to substitute (e.g., [int, str])
    ///
    /// # Returns
    /// A new Type with all type parameters replaced
    pub fn substitute(&self, params: &[Name], args: &[Type]) -> Type {
        match self {
            // Basic types: return directly (no substitution needed)
            Type::Byte | Type::Int | Type::Uint | Type::USize | Type::Float | Type::Double |
            Type::Bool | Type::Char | Type::Void | Type::CStr | Type::StrSlice |
            Type::Unknown | Type::Variadic => self.clone(),

            Type::Str(_) => Type::Str(0), // Str with size 0 is generic

            // Type parameters: lookup and replace
            Type::User(type_decl) => {
                if let Some(idx) = params.iter().position(|p| p == &type_decl.name) {
                    args[idx].clone()
                } else {
                    self.clone()
                }
            }

            // Compound types: recursive substitution
            Type::List(elem) => {
                Type::List(Box::new(elem.substitute(params, args)))
            }
            Type::Array(array_type) => {
                Type::Array(ArrayType {
                    elem: Box::new(array_type.elem.substitute(params, args)),
                    len: array_type.len,
                })
            }
            Type::RuntimeArray(rta) => {
                // Runtime arrays keep their size expression as-is (no substitution in expressions)
                Type::RuntimeArray(RuntimeArrayType {
                    elem: Box::new(rta.elem.substitute(params, args)),
                    size_expr: rta.size_expr.clone(),
                })
            }
            Type::Slice(slice_type) => {
                Type::Slice(SliceType {
                    elem: Box::new(slice_type.elem.substitute(params, args)),
                })
            }
            Type::Ptr(ptr_type) => {
                Type::Ptr(PtrType {
                    of: auto_val::shared(Type::from(ptr_type.of.borrow().clone()).substitute(params, args)),
                })
            }
            Type::Reference(inner) => {  // Plan 052
                Type::Reference(Box::new(inner.substitute(params, args)))
            }
            Type::Linear(inner) => {
                Type::Linear(Box::new(inner.substitute(params, args)))
            }

            // Generic instances: recursive substitution
            Type::GenericInstance(inst) => {
                Type::GenericInstance(GenericInstance {
                    base_name: inst.base_name.clone(),
                    args: inst.args.iter().map(|t| t.substitute(params, args)).collect(),
                })
            }

            // Complex types: clone as-is (no substitution in metadata)
            Type::Enum(_) | Type::Spec(_) | Type::Tag(_) | Type::Union(_) |
            Type::CStruct(_) | Type::Storage(_) => self.clone(),  // Storage types are not generic
        }
    }
}

/// Generic parameter - can be either a type parameter or const parameter (Plan 052)
/// Examples:
/// - Type parameter: `T` in `List<T>` (no type annotation)
/// - Const parameter: `N u32` in `Inline<T, N u32>` (with type annotation)
#[derive(Debug, Clone)]
pub enum GenericParam {
    Type(TypeParam),
    Const(ConstParam),
}

/// Type parameter (e.g., `T` in `List<T>`)
#[derive(Debug, Clone)]
pub struct TypeParam {
    pub name: Name,
    pub constraint: Option<Box<Type>>,
}

/// Const parameter (e.g., `N u32` in `Inline<T, N u32>`)
#[derive(Debug, Clone)]
pub struct ConstParam {
    pub name: Name,           // Parameter name (e.g., "N", "CAPACITY")
    pub typ: Type,            // Parameter type (e.g., u32, usize)
    pub default: Option<Expr>, // Default value (optional, future extension)
}

impl fmt::Display for GenericParam {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            GenericParam::Type(tp) => write!(f, "{}", tp),
            GenericParam::Const(cp) => write!(f, "{} {}", cp.name, cp.typ),
        }
    }
}

impl fmt::Display for TypeParam {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)?;
        if let Some(ref constraint) = self.constraint {
            write!(f, ": {}", constraint)?;
        }
        Ok(())
    }
}

impl fmt::Display for ConstParam {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {}", self.name, self.typ)
    }
}

/// Generic type instance - represents instantiation of a generic type
/// Example: `List<int>`, `May<string>`, `Map<str, int>`
#[derive(Debug, Clone)]
pub struct GenericInstance {
    pub base_name: Name,       // Base type name (e.g., "List", "May", "Map")
    pub args: Vec<Type>,        // Type arguments (e.g., [int], [str, int])
}

impl fmt::Display for GenericInstance {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.base_name)?;
        if !self.args.is_empty() {
            write!(f, "<")?;
            for (i, arg) in self.args.iter().enumerate() {
                if i > 0 {
                    write!(f, ", ")?;
                }
                write!(f, "{}", arg.unique_name())?;
            }
            write!(f, ">")?;
        }
        Ok(())
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

#[derive(Debug, Clone)]
pub struct SliceType {
    pub elem: Box<Type>,
}

/// Runtime-sized array type (Plan 052)
/// Represents arrays where the size is determined at runtime
/// Example: [size]int where size is a variable or function call
#[derive(Debug, Clone)]
pub struct RuntimeArrayType {
    pub elem: Box<Type>,
    pub size_expr: Box<Expr>,  // Size expression evaluated at runtime (boxed to avoid recursive type)
}

impl fmt::Display for RuntimeArrayType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(runtime-array-type (elem {}) (size-expr {}))", &self.elem, &self.size_expr)
    }
}

/// Storage strategy type (Plan 055: Environment-based Storage Injection)
/// Represents storage strategies for collections like List<T>
#[derive(Debug, Clone)]
pub struct StorageType {
    pub kind: StorageKind,
}

/// Storage strategy kind
#[derive(Debug, Clone, PartialEq)]
pub enum StorageKind {
    /// Fixed-capacity storage (stack/static allocation)
    /// Used for MCU environments - no heap allocation required
    Fixed { capacity: usize },
    /// Dynamic-capacity storage (heap allocation)
    /// Used for PC environments - grows as needed
    Dynamic,
}

impl fmt::Display for StorageType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.kind {
            StorageKind::Fixed { capacity } => write!(f, "Fixed<{}>", capacity),
            StorageKind::Dynamic => write!(f, "Dynamic"),
        }
    }
}

impl fmt::Display for SliceType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(slice-type (elem {}))", &self.elem)
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
            Type::StrSlice => write!(f, "str_slice"),
            Type::Array(array_type) => write!(f, "{}", array_type),
            Type::RuntimeArray(rta) => write!(f, "{}", rta),
            Type::List(elem) => write!(f, "List<{}>", elem),
            Type::Slice(slice_type) => write!(f, "{}", slice_type),
            Type::Ptr(ptr_type) => write!(f, "{}", ptr_type),
            Type::Reference(inner) => write!(f, "&{}", inner),  // Plan 052
            Type::User(type_decl) => write!(f, "{}", type_decl),
            Type::Enum(enum_decl) => write!(f, "{}", enum_decl.borrow()),
            Type::Spec(spec_decl) => write!(f, "spec {}", spec_decl.borrow().name),
            Type::Union(u) => write!(f, "{}", u),
            Type::Tag(t) => write!(f, "{}", t.borrow()),
            Type::GenericInstance(inst) => write!(f, "{}", inst),
            Type::Linear(inner) => write!(f, "linear<{}>", inner),
            Type::Variadic => write!(f, "..."),
            Type::Void => write!(f, "void"),
            Type::Unknown => write!(f, "unknown"),
            Type::CStruct(type_decl) => write!(f, "struct {}", type_decl.name),
            Type::Storage(storage) => write!(f, "{}", storage),
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
            Type::StrSlice => auto_val::Type::StrSlice,
            Type::Array(_) => auto_val::Type::Array,
            Type::RuntimeArray(_) => auto_val::Type::Array,  // Runtime arrays transpile to Array type
            Type::List(_) => auto_val::Type::Array,  // TODO: Add List to auto_val::Type
            Type::Slice(_) => auto_val::Type::Array,  // TODO: Add Slice to auto_val::Type
            Type::Ptr(_) => auto_val::Type::Ptr,
            Type::Reference(_) => auto_val::Type::Ptr,  // Plan 052: Reference transpiles to Ptr in auto_val
            Type::User(decl) => auto_val::Type::User(decl.name),
            Type::Enum(decl) => auto_val::Type::Enum(decl.borrow().name.clone()),
            Type::Spec(decl) => auto_val::Type::User(decl.borrow().name.clone()),
            Type::Union(u) => auto_val::Type::Union(u.name),
            Type::Tag(t) => auto_val::Type::Tag(t.borrow().name.clone()),
            Type::Linear(inner) => (*inner).into(),  // Linear wraps inner type
            Type::Variadic => auto_val::Type::Void,  // Variadic transpiles to void for now
            Type::Void => auto_val::Type::Void,
            Type::Unknown => auto_val::Type::Void, // TODO: is this correct?
            Type::CStruct(_) => auto_val::Type::Void,
            Type::GenericInstance(_) => auto_val::Type::Void,  // TODO: Handle generic instances properly
            Type::Storage(_) => auto_val::Type::Void,  // Storage types are marker types
        }
    }
}

// currently, spec is just a name
pub type Spec = AutoStr;

#[derive(Debug, Clone, PartialEq)]
pub enum TypeDeclKind {
    UserType,
    CType,
}

/// 成员级委托声明
/// 表示 `has member Type for Spec` 语法
#[derive(Debug, Clone)]
pub struct Delegation {
    pub member_name: AutoStr,  // 成员名
    pub member_type: Type,     // 成员类型
    pub spec_name: AutoStr,    // 委托的 spec
}

impl fmt::Display for Delegation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "(delegation (member {}) (type {}) (for spec {}))",
            self.member_name, self.member_type.unique_name(), self.spec_name
        )
    }
}

#[derive(Debug, Clone)]
pub struct TypeDecl {
    pub name: Name,
    pub kind: TypeDeclKind,
    pub parent: Option<Box<Type>>,  // 单继承：父类型（使用 Box 避免递归类型）
    pub has: Vec<Type>,            // 组合：多个组合类型
    pub specs: Vec<Spec>,          // Spec 声明：实现的 specs (names only for compatibility)
    pub spec_impls: Vec<super::spec::SpecImpl>,  // Plan 057: Generic spec implementations with type arguments
    pub generic_params: Vec<GenericParam>,  // Generic parameters (Plan 052: type + const)
    pub members: Vec<Member>,
    pub delegations: Vec<Delegation>,  // 新增：委托成员
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
        write!(f, "(type-decl (name {}", self.name)?;
        if !self.generic_params.is_empty() {
            write!(f, "<")?;
            for (i, param) in self.generic_params.iter().enumerate() {
                if i > 0 {
                    write!(f, ", ")?;
                }
                write!(f, "{}", param)?;
            }
            write!(f, ">")?;
        }
        write!(f, ")")?;
        if let Some(ref parent) = self.parent {
            write!(f, " (is {})", parent.unique_name())?;
        }
        if !self.has.is_empty() {
            write!(f, " (has ")?;
            for h in self.has.iter() {
                write!(f, "(type {})", h.unique_name())?;
            }
            write!(f, ")")?;
        }
        if !self.delegations.is_empty() {
            write!(f, " (delegations ")?;
            for (i, del) in self.delegations.iter().enumerate() {
                write!(f, "{}", del)?;
                if i < self.delegations.len() - 1 {
                    write!(f, " ")?;
                }
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
use auto_val::{Node, Value};

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
            Type::StrSlice => write!(f, "str_slice")?,
            Type::Array(array_type) => {
                write!(
                    f,
                    "array({}, {})",
                    array_type.elem.to_atom_str(),
                    array_type.len
                )?;
            }
            Type::RuntimeArray(rta) => {
                write!(
                    f,
                    "runtime_array({}, {})",
                    rta.elem.to_atom_str(),
                    rta.size_expr.to_atom_str()
                )?;
            }
            Type::List(elem) => {
                write!(f, "list({})", elem.to_atom_str())?;
            }
            Type::Slice(slice_type) => {
                write!(f, "slice({})", slice_type.elem.to_atom_str())?;
            }
            Type::Ptr(ptr_type) => {
                write!(f, "ptr({})", ptr_type.of.borrow().to_atom_str())?;
            }
            Type::Reference(inner) => {  // Plan 052
                write!(f, "ref({})", inner.to_atom_str())?;
            }
            Type::User(type_decl) => write!(f, "{}", type_decl.name)?,
            Type::Enum(enum_decl) => write!(f, "{}", enum_decl.borrow().name)?,
            Type::Spec(spec_decl) => write!(f, "spec {}", spec_decl.borrow().name)?,
            Type::Union(u) => write!(f, "{}", u.name)?,
            Type::Tag(t) => write!(f, "{}", t.borrow().name)?,
            Type::Linear(inner) => {
                write!(f, "linear({})", inner.to_atom_str())?;
            }
            Type::Variadic => write!(f, "...")?,
            Type::Void => write!(f, "void")?,
            Type::Unknown => write!(f, "unknown")?,
            Type::CStruct(type_decl) => write!(f, "struct {}", type_decl.name)?,
            Type::Storage(storage) => {
                match &storage.kind {
                    StorageKind::Fixed { capacity } => {
                        write!(f, "Fixed<{}>", capacity)?;
                    }
                    StorageKind::Dynamic => write!(f, "Dynamic")?,
                }
            }
            Type::GenericInstance(inst) => {
                write!(f, "{}", inst.base_name)?;
                if !inst.args.is_empty() {
                    write!(f, "<")?;
                    for (i, arg) in inst.args.iter().enumerate() {
                        if i > 0 {
                            write!(f, ", ")?;
                        }
                        write!(f, "{}", arg.to_atom_str())?;
                    }
                    write!(f, ">")?;
                }
            }
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
        write!(f, "member({}, {}", self.name, self.ty.to_atom_str())?;
        if let Some(value) = &self.value {
            write!(f, ", {}", value.to_atom_str())?;
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
        write!(f, "type {} {{", self.name)?;

        for (i, member) in self.members.iter().enumerate() {
            write!(f, " {}", member.to_atom_str())?;
            if i < self.members.len() - 1 || !self.methods.is_empty() {
                write!(f, ";")?;
            }
        }

        // Add methods if present
        for (i, method) in self.methods.iter().enumerate() {
            write!(f, " {}", method.to_atom_str())?;
            if i < self.methods.len() - 1 {
                write!(f, ";")?;
            }
        }

        write!(f, " }}")?;
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

use crate::pair::ValueKey;
use crate::string::AutoStr;
use crate::types::Type;
use crate::value::Value;
use std::fmt;

/// Represents an argument in a function call.
#[derive(Debug, Clone, PartialEq)]
pub enum Arg {
    /// Positional argument.
    Pos(Value),
    /// Pair argument: key is the arg's name, value is the arg's value.
    Pair(ValueKey, Value),
    /// Name argument, where the name and value are the same.
    Name(AutoStr),
}

/// Methods for the `Arg` enum.
impl Arg {
    /// Returns the value of the argument.
    pub fn get_val(&self) -> Value {
        match self {
            Arg::Pos(value) => value.clone(),
            Arg::Pair(_, value) => value.clone(),
            Arg::Name(name) => Value::Str(name.clone()),
        }
    }

    /// Returns the argument as an AutoStr.
    pub fn to_astr(&self) -> AutoStr {
        match self {
            Arg::Pos(value) => value.to_astr(),
            Arg::Pair(_, value) => value.to_astr(),
            Arg::Name(name) => name.clone(),
        }
    }
}

/// Container for arguments.
#[derive(Debug, Clone, PartialEq)]
pub struct Args {
    pub args: Vec<Arg>,
}

impl Args {
    pub const fn new() -> Self {
        Self { args: Vec::new() }
    }

    pub const EMPTY: Self = Self { args: vec![] };

    pub fn get_val(&self, index: usize) -> Value {
        self.args
            .get(index)
            .map(|arg| arg.get_val())
            .unwrap_or(Value::Nil)
    }

    pub fn array(values: Vec<impl Into<Value>>) -> Self {
        Self {
            args: values.into_iter().map(|v| Arg::Pos(v.into())).collect(),
        }
    }

    pub fn add_name(&mut self, name: impl Into<AutoStr>) {
        self.args.push(Arg::Name(name.into()));
    }

    pub fn add_pos(&mut self, value: impl Into<Value>) {
        self.args.push(Arg::Pos(value.into()));
    }

    pub fn add_pair(&mut self, name: &str, value: Value) {
        self.args.push(Arg::Pair(name.into(), value));
    }

    pub fn is_empty(&self) -> bool {
        self.args.is_empty()
    }
}

/// Print related

impl fmt::Display for Args {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.args.is_empty() {
            return Ok(());
        }
        write!(f, "(")?;
        for (i, arg) in self.args.iter().enumerate() {
            write!(f, "{}", arg)?;
            if i < self.args.len() - 1 {
                write!(f, ", ")?;
            }
        }
        write!(f, ")")
    }
}

impl fmt::Display for Arg {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Arg::Pos(value) => write!(f, "{}", value),
            Arg::Pair(key, value) => write!(f, "{}:{}", key, value),
            Arg::Name(name) => write!(f, "{}", name),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Fn {
    pub sig: Sig,
    pub fun: fn(&Vec<Value>) -> Value,
}

impl PartialEq for Fn {
    fn eq(&self, other: &Self) -> bool {
        self.sig == other.sig && std::ptr::fn_addr_eq(self.fun, other.fun)
    }
}

/// Function signature
#[derive(Debug, Clone, PartialEq)]
pub struct Sig {
    pub name: AutoStr,
    pub params: Vec<Param>,
    pub ret: Type,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub name: AutoStr,
    pub ty: Box<Type>,
}

#[derive(Debug, Clone)]
pub struct ExtFn {
    pub name: AutoStr,
    pub fun: fn(&Args) -> Value,
}

impl PartialEq for ExtFn {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && std::ptr::fn_addr_eq(self.fun, other.fun)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum MetaID {
    Fn(Sig),
    Lambda(Sig),
    Type(AutoStr),
    Enum(AutoStr),
    Node(AutoStr),
    Body(AutoStr),
    Method(MethodMeta),
    Nil,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MethodMeta {
    pub name: AutoStr,
    pub ty: Type,
}

impl fmt::Display for MethodMeta {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}::{}", self.ty, self.name)
    }
}

impl fmt::Display for MetaID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MetaID::Fn(sig) => write!(f, "<fn:{}>", sig),
            MetaID::Lambda(sig) => write!(f, "<lambda:{}>", sig),
            MetaID::Body(id) => write!(f, "<body:{}>", id),
            MetaID::Nil => write!(f, "<meta-nil>"),
            MetaID::Method(method) => write!(f, "<method:{}>", method),
            MetaID::Type(id) => write!(f, "<type:{}>", id),
            MetaID::Enum(name) => write!(f, "<enum:{}>", name),
            MetaID::Node(id) => write!(f, "<node:{}>", id),
        }
    }
}

impl fmt::Display for Sig {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)?;
        for param in &self.params {
            write!(f, " {}", param)?;
        }
        write!(f, " -> {}", self.ret)
    }
}

impl fmt::Display for Param {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: {}", self.name, self.ty)
    }
}

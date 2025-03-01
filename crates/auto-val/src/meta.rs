use std::fmt;
use crate::types::Type;
use crate::value::Value;
use crate::pair::ValueKey;
use crate::string::AutoStr;


#[derive(Debug, Clone, PartialEq)]
pub enum Arg {
    Pos(Value),
    Pair(ValueKey, Value),
    Name(AutoStr),
}

impl Arg {
    pub fn get_val(&self) -> Value {
        match self {
            Arg::Pos(value) => value.clone(),
            Arg::Pair(_, value) => value.clone(),
            Arg::Name(name) => Value::Str(name.clone()),
        }
    }

    pub fn to_astr(&self) -> AutoStr {
        match self {
            Arg::Pos(value) => value.to_astr(),
            Arg::Pair(_, value) => value.to_astr(),
            Arg::Name(name) => name.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Args {
    pub args: Vec<Arg>,
}

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

impl Args {
    pub fn new() -> Self {
        Self { args: Vec::new() }
    }

    pub const EMPTY: Self = Self { args: vec![] };

    pub fn get_val(&self, index: usize) -> Value {
        self.args.get(index).map(|arg| arg.get_val()).unwrap_or(Value::Nil)
    }

    pub fn array(values: Vec<impl Into<Value>>) -> Self {
        Self { args: values.into_iter().map(|v| Arg::Pos(v.into())).collect() }
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

#[derive(Debug, Clone, PartialEq)]
pub struct Fn {
    pub sig: Sig,
    pub fun: fn(&Vec<Value>) -> Value,
}

/// Function signature
#[derive(Debug, Clone, PartialEq)]
pub struct Sig {
    pub name: String,
    pub params: Vec<Param>,
    pub ret: Type,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub name: String,
    pub ty: Box<Type>,
}

#[derive(Debug, Clone)]
pub struct ExtFn {
    pub name: String,
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
    Type(String),
    View(String),
    Body(String),
    Method(MethodMeta),
    Nil,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MethodMeta {
    pub name: String,
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
            MetaID::View(id) => write!(f, "<view:{}>", id),
            MetaID::Body(id) => write!(f, "<body:{}>", id),
            MetaID::Nil => write!(f, "<meta-nil>"),
            MetaID::Method(method) => write!(f, "<method:{}>", method),
            MetaID::Type(id) => write!(f, "<type:{}>", id),
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
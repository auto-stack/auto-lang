use super::{Body, Expr, Name, Type};
use serde::Serialize;
use std::fmt;

#[derive(Debug, Clone)]
pub enum FnKind {
    Function,
    Lambda,
    Method,
    CFunction, // C function declaration
}

#[derive(Debug, Clone)]
pub struct Fn {
    // TODO: add FnKind to differ Fn/Lambda/Method?
    pub kind: FnKind,
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
        write!(f, "(fn (name {})", self.name)?;
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
    pub fn new(
        kind: FnKind,
        name: Name,
        parent: Option<Name>,
        params: Vec<Param>,
        body: Body,
        ret: Type,
    ) -> Fn {
        Fn {
            kind,
            name,
            parent,
            params,
            body,
            ret,
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
        write!(f, "(param (name {}) (type {})", self.name, self.ty)?;
        if let Some(default) = &self.default {
            write!(f, " (default {})", default)?;
        }
        write!(f, ")")
    }
}

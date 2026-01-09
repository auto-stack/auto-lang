use super::{Body, Expr, Name, Type};
use crate::ast::call::Arg;
use crate::ast::Stmt;
use crate::ast::{AtomWriter, ToAtomStr};
use serde::Serialize;
use std::{fmt, io as stdio};

#[derive(Debug, Clone)]
pub enum FnKind {
    Function,
    Lambda,
    Method,
    CFunction,  // C function declaration
    VmFunction, // VM implemented function declaration
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
    pub ret_name: Option<Name>, // Original return type name (for unresolved types)
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
            ret_name: None,
        }
    }

    pub fn with_ret_name(
        kind: FnKind,
        name: Name,
        parent: Option<Name>,
        params: Vec<Param>,
        body: Body,
        ret: Type,
        ret_name: Name,
    ) -> Fn {
        Fn {
            kind,
            name,
            parent,
            params,
            body,
            ret,
            ret_name: Some(ret_name),
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

impl Param {
    pub fn new(name: Name, ty: Type, default: Option<Expr>) -> Self {
        Self { name, ty, default }
    }
}

// ToAtom and ToNode implementations

use crate::ast::{ToAtom, ToNode};
use auto_val::{AutoStr, Node as AutoNode, Value};

impl AtomWriter for Param {
    fn write_atom(&self, f: &mut impl stdio::Write) -> auto_val::AutoResult<()> {
        write!(f, "({}, {})", self.name, self.ty.to_atom_str())?;
        if let Some(default) = &self.default {
            write!(f, " = {}", default.to_atom_str())?;
        }
        Ok(())
    }
}

impl ToNode for Param {
    fn to_node(&self) -> AutoNode {
        let mut node = AutoNode::new("param");
        node.set_prop("name", Value::str(self.name.as_str()));
        node.set_prop("type", Value::str(&*self.ty.to_atom()));
        if let Some(default) = &self.default {
            node.set_prop("default", Value::str(&*default.to_atom()));
        }
        node
    }
}

impl ToAtom for Param {
    fn to_atom(&self) -> AutoStr {
        self.to_atom_str()
    }
}

impl AtomWriter for Fn {
    fn write_atom(&self, f: &mut impl stdio::Write) -> auto_val::AutoResult<()> {
        match self.kind {
            FnKind::Lambda => {
                // Lambda format: lambda(x, y) { body }
                write!(f, "lambda(")?;
                for (i, param) in self.params.iter().enumerate() {
                    write!(f, "{}", param.name)?;
                    if i < self.params.len() - 1 {
                        write!(f, ", ")?;
                    }
                }
                write!(f, ") {{")?;
                if !self.body.stmts.is_empty() {
                    write!(f, " ")?;
                    self.body.write_statements(f)?;
                    write!(f, " }}")?;
                } else {
                    write!(f, "}}")?;
                }
            }
            FnKind::CFunction => {
                // C Function format: fn.c name (n, double) double
                write!(f, "fn.c {}", self.name)?;
                write!(f, " (")?;
                for (i, param) in self.params.iter().enumerate() {
                    write!(f, "{}", param.name)?;
                    if i < self.params.len() - 1 || !matches!(self.ret, Type::Unknown) {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", param.ty)?;
                }
                write!(f, ")")?;
                if !matches!(self.ret, Type::Unknown) {
                    write!(f, " {}", self.ret.to_atom_str())?;
                }
            }
            _ => {
                // Function format: fn name ((a, int), (b, int)) int { body }
                write!(f, "fn {}", self.name)?;
                write!(f, " (")?;
                for (i, param) in self.params.iter().enumerate() {
                    write!(f, "({}, {})", param.name, param.ty)?;
                    if i < self.params.len() - 1 {
                        write!(f, ", ")?;
                    }
                }
                write!(f, ")")?;
                // Output return type: use ret_name if ret is Unknown, otherwise use ret.to_atom_str()
                if matches!(self.ret, Type::Unknown) {
                    if let Some(ret_name) = &self.ret_name {
                        write!(f, " {}", ret_name)?;
                    }
                } else if !matches!(self.ret, Type::Unknown) {
                    write!(f, " {}", self.ret.to_atom_str())?;
                }
                write!(f, " {{")?;
                if !self.body.stmts.is_empty() {
                    write!(f, " ")?;
                    self.body.write_statements(f)?;
                    write!(f, " }}")?;
                } else {
                    write!(f, "}}")?;
                }
            }
        }
        Ok(())
    }
}

impl ToNode for Fn {
    fn to_node(&self) -> AutoNode {
        let mut node = AutoNode::new("fn");
        node.set_prop("name", Value::str(self.name.as_str()));
        node.set_prop("kind", Value::str(format!("{:?}", self.kind).as_str()));

        if let Some(parent) = &self.parent {
            node.set_prop("parent", Value::str(parent.as_str()));
        }

        if !matches!(self.ret, Type::Unknown) {
            node.set_prop("return", Value::str(&*self.ret.to_atom()));
        }

        // Add params as children
        for param in &self.params {
            node.add_kid(param.to_node());
        }

        // Add body
        node.add_kid(self.body.to_node());

        node
    }
}

impl ToAtom for Fn {
    fn to_atom(&self) -> AutoStr {
        self.to_atom_str()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_param_to_atom() {
        let param = Param::new("x".into(), Type::Int, None);
        let atom = param.to_atom();
        assert_eq!(atom, "(x, int)");
    }

    #[test]
    fn test_fn_to_atom() {
        let fn_decl = Fn::new(
            FnKind::Function,
            "add".into(),
            None,
            vec![Param::new("a".into(), Type::Int, None)],
            Body::new(),
            Type::Int,
        );
        let atom = fn_decl.to_atom();
        assert_eq!(atom, "fn add ((a, int)) int {}");
    }
}

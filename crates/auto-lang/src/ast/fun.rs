use super::{Body, Expr, Name, Type};
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
        write!(
            f,
            "param(name(\"{}\"), type({}))",
            self.name,
            self.ty.to_atom_str()
        )?;
        if let Some(default) = &self.default {
            write!(f, ", default({})", default.to_atom_str())?;
        }
        write!(f, ")")?;
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
        write!(f, "fn(name(\"{}\")) {{", self.name)?;
        for param in &self.params {
            write!(f, " {}", param.to_atom_str())?;
        }
        if !matches!(self.ret, Type::Unknown) {
            write!(f, " return(type({}))", self.ret.to_atom_str())?;
        }
        if !matches!(self.body.stmts.len(), 0) {
            write!(f, " body {{{}}}", self.body.to_atom_str())?;
        }
        write!(f, " }}")?;
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
        // Should be in format "(param (name x) (type int))"
        assert!(
            atom.contains("param"),
            "Expected atom to contain 'param', got: {}",
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
        // Should be in format "(fn (name add) ...)"
        assert!(
            atom.contains("fn"),
            "Expected atom to contain 'fn', got: {}",
            atom
        );
        assert!(
            atom.contains("add"),
            "Expected atom to contain 'add', got: {}",
            atom
        );
        assert!(
            atom.contains("return"),
            "Expected atom to contain 'return', got: {}",
            atom
        );
        assert!(
            atom.contains("int"),
            "Expected atom to contain 'int', got: {}",
            atom
        );
    }
}

use super::{Body, Expr, Name, Type};
use serde::Serialize;
use std::fmt;

#[derive(Debug, Clone)]
pub enum FnKind {
    Function,
    Lambda,
    Method,
    CFunction, // C function declaration
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

// ToAtom implementation

use crate::ast::ToAtom;
use auto_val::{Node, Value};

impl ToAtom for Param {
    fn to_atom(&self) -> Value {
        let mut node = Node::new("param");
        node.set_prop("name", Value::str(self.name.as_str()));
        node.set_prop("type", self.ty.to_atom());
        if let Some(default) = &self.default {
            node.set_prop("default", default.to_atom());
        }
        Value::Node(node)
    }
}

impl ToAtom for Fn {
    fn to_atom(&self) -> Value {
        let mut node = Node::new("fn");
        node.set_prop("name", Value::str(self.name.as_str()));
        node.set_prop("kind", Value::str(format!("{:?}", self.kind).as_str()));

        if let Some(parent) = &self.parent {
            node.set_prop("parent", Value::str(parent.as_str()));
        }

        if !matches!(self.ret, Type::Unknown) {
            node.set_prop("return", self.ret.to_atom());
        }

        // Add params as children
        for param in &self.params {
            node.add_kid(param.to_atom().to_node());
        }

        // Add body
        node.add_kid(self.body.to_atom().to_node());

        Value::Node(node)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_param_to_atom() {
        let param = Param::new("x".into(), Type::Int, None);
        let atom = param.to_atom();

        match atom {
            Value::Node(node) => {
                assert_eq!(node.name, "param");
                assert_eq!(node.get_prop("name"), Value::str("x"));
                assert_eq!(node.get_prop("type"), Value::str("int"));
            }
            _ => panic!("Expected Node, got {:?}", atom),
        }
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

        match atom {
            Value::Node(node) => {
                assert_eq!(node.name, "fn");
                assert_eq!(node.get_prop("name"), Value::str("add"));
                assert_eq!(node.get_prop("return"), Value::str("int"));
            }
            _ => panic!("Expected Node, got {:?}", atom),
        }
    }
}

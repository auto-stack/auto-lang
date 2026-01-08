use super::{Name, Type};
use std::fmt;

#[derive(Debug, Clone)]
pub struct Union {
    pub name: Name,
    pub fields: Vec<UnionField>,
}

#[derive(Debug, Clone)]
pub struct UnionField {
    pub name: Name,
    pub ty: Type,
}

impl fmt::Display for UnionField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.name, self.ty)
    }
}

impl fmt::Display for Union {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "union {} {{", self.name)?;
        for field in &self.fields {
            write!(f, "\n    {}", field)?;
        }
        write!(f, "\n}}")
    }
}

// ToAtom and ToNode implementations

use crate::ast::{ToAtom, ToNode};
use auto_val::{Node as AutoNode, Value};

impl ToNode for Union {
    fn to_node(&self) -> AutoNode {
        let mut node = AutoNode::new("union");
        node.set_prop("name", Value::str(self.name.as_str()));

        for field in &self.fields {
            node.add_kid(field.to_node());
        }

        node
    }
}

impl ToAtom for Union {
    fn to_atom(&self) -> Value {
        Value::Node(self.to_node())
    }
}

impl ToNode for UnionField {
    fn to_node(&self) -> AutoNode {
        let mut node = AutoNode::new("field");
        node.set_prop("name", Value::str(self.name.as_str()));
        node.set_prop("type", self.ty.to_atom());
        node
    }
}

impl ToAtom for UnionField {
    fn to_atom(&self) -> Value {
        Value::Node(self.to_node())
    }
}

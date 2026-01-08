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

// ToAtom implementation

use crate::ast::ToAtom;
use auto_val::{Node, Value};

impl ToAtom for Union {
    fn to_atom(&self) -> Value {
        let mut node = Node::new("union");
        node.set_prop("name", Value::str(self.name.as_str()));

        for field in &self.fields {
            node.add_kid(field.to_atom().to_node());
        }

        Value::Node(node)
    }
}

impl ToAtom for UnionField {
    fn to_atom(&self) -> Value {
        let mut node = Node::new("field");
        node.set_prop("name", Value::str(self.name.as_str()));
        node.set_prop("type", self.ty.to_atom());
        Value::Node(node)
    }
}

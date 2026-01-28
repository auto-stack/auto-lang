use super::{Name, Type};
use crate::ast::AtomWriter;
use auto_val::{AutoStr, Node as AutoNode, Value};
use std::{fmt, io as stdio};

/// Type alias declaration: `type List<T> = List<T, DefaultStorage>`
#[derive(Debug, Clone)]
pub struct TypeAlias {
    /// Alias name (e.g., "List")
    pub name: Name,
    /// Generic parameters (e.g., ["T"])
    pub params: Vec<Name>,
    /// Target type (e.g., List<T, DefaultStorage>)
    pub target: Type,
}

impl fmt::Display for TypeAlias {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "type {}", self.name)?;
        if !self.params.is_empty() {
            write!(f, "<{}>", self.params.iter().map(|p| p.as_str()).collect::<Vec<_>>().join(", "))?;
        }
        write!(f, " = {}", self.target)
    }
}

impl AtomWriter for TypeAlias {
    fn write_atom(&self, f: &mut impl stdio::Write) -> auto_val::AutoResult<()> {
        write!(f, "type_alias(name(\"{}\"), params([", self.name)?;
        for (i, p) in self.params.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "\"{}\"", p)?;
        }
        write!(f, "]), target({}))", self.target)?;
        Ok(())
    }
}

use crate::ast::{ToAtom, ToAtomStr, ToNode};

impl ToNode for TypeAlias {
    fn to_node(&self) -> AutoNode {
        let mut node = AutoNode::new("type_alias");
        node.set_prop("name", Value::str(self.name.as_str()));

        let params: Vec<Value> = self.params.iter().map(|p| Value::str(p.as_str())).collect();
        node.set_prop("params", Value::array_of(params));

        node.set_prop("target", self.target.to_node());
        node
    }
}

impl ToAtom for TypeAlias {
    fn to_atom(&self) -> AutoStr {
        self.to_atom_str()
    }
}

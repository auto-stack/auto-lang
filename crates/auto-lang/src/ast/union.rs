use super::{Name, Type};
use crate::ast::{AtomWriter, ToAtomStr};
use std::{fmt, io as stdio};

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
use auto_val::{AutoStr, Node as AutoNode, Value};

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

impl AtomWriter for Union {
    fn write_atom(&self, f: &mut impl stdio::Write) -> auto_val::AutoResult<()> {
        write!(f, "union(name(\"{}\")) {{", self.name)?;
        for field in &self.fields {
            write!(f, " {}", field.to_atom_str())?;
        }
        write!(f, " }}")?;
        Ok(())
    }
}

impl ToAtom for Union {
    fn to_atom(&self) -> AutoStr {
        self.to_atom_str()
    }
}

impl AtomWriter for UnionField {
    fn write_atom(&self, f: &mut impl stdio::Write) -> auto_val::AutoResult<()> {
        write!(
            f,
            "field(name(\"{}\"), type({}))",
            self.name,
            self.ty.to_atom_str()
        )?;
        Ok(())
    }
}

impl ToNode for UnionField {
    fn to_node(&self) -> AutoNode {
        let mut node = AutoNode::new("field");
        node.set_prop("name", Value::str(self.name.as_str()));
        node.set_prop("type", Value::str(&*self.ty.to_atom()));
        node
    }
}

impl ToAtom for UnionField {
    fn to_atom(&self) -> AutoStr {
        self.to_atom_str()
    }
}

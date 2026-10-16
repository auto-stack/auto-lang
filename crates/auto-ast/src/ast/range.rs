use core::fmt;
use std::io as stdio;

use super::Expr;
use crate::ast::{AtomWriter, ToAtomStr};

#[derive(Debug, Clone)]
pub struct Range {
    pub start: Box<Expr>,
    pub end: Box<Expr>,
    pub eq: bool,
}

impl fmt::Display for Range {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "(range (start {}) (end {}) (eq {}))",
            self.start, self.end, self.eq
        )
    }
}

// ToAtom and ToNode implementations

use crate::ast::{ToAtom, ToNode};
use auto_val::{AutoStr, Node as AutoNode, Value};

impl AtomWriter for Range {
    fn write_atom(&self, f: &mut impl stdio::Write) -> auto_val::AutoResult<()> {
        write!(
            f,
            "range(start({}), end({})",
            self.start.to_atom_str(),
            self.end.to_atom_str()
        )?;
        if self.eq {
            write!(f, ", eq(true)")?;
        } else {
            write!(f, ", eq(false)")?;
        }
        write!(f, ")")?;
        Ok(())
    }
}

impl ToNode for Range {
    fn to_node(&self) -> AutoNode {
        let mut node = AutoNode::new("range");
        node.set_prop("eq", Value::Bool(self.eq));
        node.add_kid(self.start.to_node()); // Changed from start.to_atom().to_node()
        node.add_kid(self.end.to_node()); // Changed from end.to_atom().to_node()
        node
    }
}

impl ToAtom for Range {
    fn to_atom(&self) -> AutoStr {
        self.to_atom_str()
    }
}

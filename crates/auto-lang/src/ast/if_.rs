use super::{Body, Branch};
use std::fmt;

#[derive(Debug, Clone)]
pub struct If {
    pub branches: Vec<Branch>,
    pub else_: Option<Body>,
}

impl fmt::Display for If {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(if ")?;
        for branch in self.branches.iter() {
            write!(f, "{}", branch)?;
        }
        if let Some(else_stmt) = &self.else_ {
            write!(f, " (else {})", else_stmt)?;
        }
        write!(f, ")")
    }
}

// ToAtom and ToNode implementations

use crate::ast::{ToAtom, ToNode};
use auto_val::{Node as AutoNode, Value};

impl ToNode for If {
    fn to_node(&self) -> AutoNode {
        let mut node = AutoNode::new("if");
        for branch in &self.branches {
            node.add_kid(branch.to_node());
        }
        if let Some(else_body) = &self.else_ {
            let mut else_node = AutoNode::new("else");
            else_node.add_kid(else_body.to_node());
            node.add_kid(else_node);
        }
        node
    }
}

impl ToAtom for If {
    fn to_atom(&self) -> Value {
        Value::Node(self.to_node())
    }
}

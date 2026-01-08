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

// ToAtom implementation

use crate::ast::ToAtom;
use auto_val::{Node, Value};

impl ToAtom for If {
    fn to_atom(&self) -> Value {
        let mut node = Node::new("if");
        for branch in &self.branches {
            node.add_kid(branch.to_atom().to_node());
        }
        if let Some(else_body) = &self.else_ {
            let mut else_node = Node::new("else");
            else_node.add_kid(else_body.to_atom().to_node());
            node.add_kid(else_node);
        }
        Value::Node(node)
    }
}

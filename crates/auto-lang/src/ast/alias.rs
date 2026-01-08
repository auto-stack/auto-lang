use super::Name;
use std::fmt;

#[derive(Debug, Clone)]
pub struct Alias {
    pub alias: Name,
    pub target: Name,
}

impl fmt::Display for Alias {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(alias (name {}) (target {}))", self.alias, self.target)
    }
}

// ToAtom implementation

use crate::ast::ToAtom;
use auto_val::{Node, Value};

impl ToAtom for Alias {
    fn to_atom(&self) -> Value {
        let mut node = Node::new("alias");
        node.set_prop("name", Value::str(self.alias.as_str()));
        node.set_prop("target", Value::str(self.target.as_str()));
        Value::Node(node)
    }
}

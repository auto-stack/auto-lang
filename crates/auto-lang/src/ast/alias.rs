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

// ToAtom and ToNode implementations

use crate::ast::{ToAtom, ToNode};
use auto_val::{Node as AutoNode, Value};

impl ToNode for Alias {
    fn to_node(&self) -> AutoNode {
        let mut node = AutoNode::new("alias");
        node.set_prop("name", Value::str(self.alias.as_str()));
        node.set_prop("target", Value::str(self.target.as_str()));
        node
    }
}

impl ToAtom for Alias {
    fn to_atom(&self) -> Value {
        Value::Node(self.to_node())
    }
}

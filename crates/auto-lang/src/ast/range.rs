use core::fmt;

use super::Expr;

#[derive(Debug, Clone)]
pub struct Range {
    pub start: Box<Expr>,
    pub end: Box<Expr>,
    pub eq: bool,
}

impl fmt::Display for Range {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(range (start {}) (end {}) (eq {}))", self.start, self.end, self.eq)
    }
}

// ToAtom and ToNode implementations

use crate::ast::{ToAtom, ToNode};
use auto_val::{Node as AutoNode, Value};

impl ToNode for Range {
    fn to_node(&self) -> AutoNode {
        let mut node = AutoNode::new("range");
        node.set_prop("eq", Value::Bool(self.eq));
        node.add_kid(self.start.to_atom().to_node());
        node.add_kid(self.end.to_atom().to_node());
        node
    }
}

impl ToAtom for Range {
    fn to_atom(&self) -> Value {
        Value::Node(self.to_node())
    }
}
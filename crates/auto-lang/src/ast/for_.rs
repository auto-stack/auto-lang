use super::{Body, Call, Expr, Name};
use std::fmt;

#[derive(Debug, Clone)]
pub struct For {
    pub iter: Iter,
    pub range: Expr,
    pub body: Body,
    pub new_line: bool,
    // TODO: maybe we could put mid block here
}

#[derive(Debug, Clone)]
pub enum Iter {
    Indexed(/*index*/ Name, /*iter*/ Name),
    Named(/*iter*/ Name),
    Call(Call),
    Ever,
}

#[derive(Debug, Clone)]
pub enum Break {
    // TODO: maybe we could put mid block here
}

impl fmt::Display for For {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(for {} {} {})", self.iter, self.range, self.body)
    }
}

impl fmt::Display for Iter {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Iter::Indexed(index, iter) => write!(f, "((name {}) (name {}))", index, iter),
            Iter::Named(iter) => write!(f, "(name {})", iter),
            Iter::Call(call) => write!(f, "(call {})", call),
            Iter::Ever => write!(f, "(ever)"),
        }
    }
}

impl fmt::Display for Break {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(break)")
    }
}

// ToAtom and ToNode implementations

use crate::ast::{ToAtom, ToNode};
use auto_val::{Node as AutoNode, Value};

impl ToNode for For {
    fn to_node(&self) -> AutoNode {
        let mut node = AutoNode::new("for");

        // Add iterator based on type
        match &self.iter {
            Iter::Indexed(index, iter_name) => {
                let mut iter_node = AutoNode::new("iter");
                iter_node.set_prop("index", Value::str(index.as_str()));
                iter_node.set_prop("name", Value::str(iter_name.as_str()));
                node.add_kid(iter_node);
            }
            Iter::Named(name) => {
                let mut iter_node = AutoNode::new("iter");
                iter_node.set_prop("name", Value::str(name.as_str()));
                node.add_kid(iter_node);
            }
            Iter::Call(call) => {
                node.add_kid(call.to_node());
            }
            Iter::Ever => {
                let iter_node = AutoNode::new("ever");
                node.add_kid(iter_node);
            }
        }

        node.add_kid(self.range.to_atom().to_node());
        node.add_kid(self.body.to_node());
        node
    }
}

impl ToAtom for For {
    fn to_atom(&self) -> Value {
        Value::Node(self.to_node())
    }
}

impl ToNode for Iter {
    fn to_node(&self) -> AutoNode {
        match self {
            Iter::Indexed(index, iter_name) => {
                let mut node = AutoNode::new("iter");
                node.set_prop("index", Value::str(index.as_str()));
                node.set_prop("name", Value::str(iter_name.as_str()));
                node
            }
            Iter::Named(name) => {
                let mut node = AutoNode::new("iter");
                node.set_prop("name", Value::str(name.as_str()));
                node
            }
            Iter::Call(call) => call.to_node(),
            Iter::Ever => AutoNode::new("ever"),
        }
    }
}

impl ToAtom for Iter {
    fn to_atom(&self) -> Value {
        Value::Node(self.to_node())
    }
}

impl ToNode for Break {
    fn to_node(&self) -> AutoNode {
        AutoNode::new("break")
    }
}

impl ToAtom for Break {
    fn to_atom(&self) -> Value {
        Value::Node(self.to_node())
    }
}

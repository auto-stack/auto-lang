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

// ToAtom implementation

use crate::ast::ToAtom;
use auto_val::{Node, Value};

impl ToAtom for For {
    fn to_atom(&self) -> Value {
        let mut node = Node::new("for");

        // Add iterator based on type
        match &self.iter {
            Iter::Indexed(index, iter_name) => {
                let mut iter_node = Node::new("iter");
                iter_node.set_prop("index", Value::str(index.as_str()));
                iter_node.set_prop("name", Value::str(iter_name.as_str()));
                node.add_kid(iter_node);
            }
            Iter::Named(name) => {
                let mut iter_node = Node::new("iter");
                iter_node.set_prop("name", Value::str(name.as_str()));
                node.add_kid(iter_node);
            }
            Iter::Call(call) => {
                node.add_kid(call.to_atom().to_node());
            }
            Iter::Ever => {
                let iter_node = Node::new("ever");
                node.add_kid(iter_node);
            }
        }

        node.add_kid(self.range.to_atom().to_node());
        node.add_kid(self.body.to_atom().to_node());
        Value::Node(node)
    }
}

impl ToAtom for Iter {
    fn to_atom(&self) -> Value {
        match self {
            Iter::Indexed(index, iter_name) => {
                let mut node = Node::new("iter");
                node.set_prop("index", Value::str(index.as_str()));
                node.set_prop("name", Value::str(iter_name.as_str()));
                Value::Node(node)
            }
            Iter::Named(name) => {
                let mut node = Node::new("iter");
                node.set_prop("name", Value::str(name.as_str()));
                Value::Node(node)
            }
            Iter::Call(call) => call.to_atom(),
            Iter::Ever => {
                let node = Node::new("ever");
                Value::Node(node)
            }
        }
    }
}

impl ToAtom for Break {
    fn to_atom(&self) -> Value {
        let node = Node::new("break");
        Value::Node(node)
    }
}

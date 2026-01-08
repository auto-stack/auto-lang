use crate::ast::Expr;
use std::fmt;

#[derive(Debug, Clone)]
pub struct Arrow {
    pub src: Option<Expr>,
    pub dest: Option<Expr>,
    pub with: Option<Expr>,
}

#[derive(Debug, Clone)]
pub struct CondArrow {
    pub src: Option<Expr>,
    pub cond: Expr,
    pub subs: Vec<Arrow>,
}

#[derive(Debug, Clone)]
pub enum Event {
    Arrow(Arrow),
    CondArrow(CondArrow),
}

#[derive(Debug, Clone)]
pub struct OnEvents {
    pub branches: Vec<Event>,
}

impl Arrow {
    pub fn new(src: Option<Expr>, dest: Option<Expr>, with: Option<Expr>) -> Self {
        Self { src, dest, with }
    }
}

impl CondArrow {
    pub fn new(src: Option<Expr>, cond: Expr, subs: Vec<Arrow>) -> Self {
        Self { src, cond, subs }
    }
}

impl OnEvents {
    pub fn new(branches: Vec<Event>) -> Self {
        Self { branches }
    }
}

impl fmt::Display for Arrow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(arrow")?;
        if let Some(src) = &self.src {
            write!(f, " (from {})", src)?;
        }
        if let Some(dest) = &self.dest {
            write!(f, " (to {})", dest)?;
        }
        if let Some(with) = &self.with {
            write!(f, " (with {})", with)?;
        }
        write!(f, ")")
    }
}

impl fmt::Display for CondArrow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(cond-arrow")?;
        if let Some(src) = &self.src {
            write!(f, " (from {})", src)?;
        }
        write!(f, " (cond {})", self.cond)?;
        for sub in &self.subs {
            write!(f, " {}", sub)?;
        }
        write!(f, ")")
    }
}

impl fmt::Display for Event {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Event::Arrow(a) => write!(f, "{}", a),
            Event::CondArrow(c) => write!(f, "{}", c),
        }
    }
}

impl fmt::Display for OnEvents {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(on")?;
        for branch in &self.branches {
            write!(f, " {}", branch)?;
        }
        write!(f, ")")
    }
}

// ToAtom implementation

use crate::ast::ToAtom;
use auto_val::{Node, Value};

impl ToAtom for OnEvents {
    fn to_atom(&self) -> Value {
        let mut node = Node::new("on");

        for branch in &self.branches {
            node.add_kid(branch.to_atom().to_node());
        }

        Value::Node(node)
    }
}

impl ToAtom for Event {
    fn to_atom(&self) -> Value {
        match self {
            Event::Arrow(arrow) => arrow.to_atom(),
            Event::CondArrow(cond_arrow) => cond_arrow.to_atom(),
        }
    }
}

impl ToAtom for Arrow {
    fn to_atom(&self) -> Value {
        let mut node = Node::new("arrow");

        if let Some(src) = &self.src {
            node.set_prop("from", src.to_atom());
        }
        if let Some(dest) = &self.dest {
            node.set_prop("to", dest.to_atom());
        }
        if let Some(with) = &self.with {
            node.set_prop("with", with.to_atom());
        }

        Value::Node(node)
    }
}

impl ToAtom for CondArrow {
    fn to_atom(&self) -> Value {
        let mut node = Node::new("cond-arrow");

        if let Some(src) = &self.src {
            node.set_prop("from", src.to_atom());
        }

        node.set_prop("cond", self.cond.to_atom());

        for sub in &self.subs {
            node.add_kid(sub.to_atom().to_node());
        }

        Value::Node(node)
    }
}

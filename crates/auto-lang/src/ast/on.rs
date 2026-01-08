use crate::ast::Expr;
use crate::ast::{AtomWriter, ToAtomStr};
use std::{fmt, io as stdio};

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

// ToAtom and ToNode implementations

use crate::ast::{ToAtom, ToNode};
use auto_val::{AutoStr, Node as AutoNode, Value};

impl ToNode for OnEvents {
    fn to_node(&self) -> AutoNode {
        let mut node = AutoNode::new("on");

        for branch in &self.branches {
            node.add_kid(branch.to_node());
        }

        node
    }
}

impl AtomWriter for OnEvents {
    fn write_atom(&self, f: &mut impl stdio::Write) -> auto_val::AutoResult<()> {
        write!(f, "(on")?;
        for branch in &self.branches {
            write!(f, " {}", branch.to_atom_str())?;
        }
        write!(f, ")")?;
        Ok(())
    }
}

impl ToAtom for OnEvents {
    fn to_atom(&self) -> AutoStr {
        self.to_atom_str()
    }
}

impl AtomWriter for Event {
    fn write_atom(&self, f: &mut impl stdio::Write) -> auto_val::AutoResult<()> {
        match self {
            Event::Arrow(arrow) => arrow.write_atom(f),
            Event::CondArrow(cond_arrow) => cond_arrow.write_atom(f),
        }
    }
}

impl ToNode for Event {
    fn to_node(&self) -> AutoNode {
        match self {
            Event::Arrow(arrow) => arrow.to_node(),
            Event::CondArrow(cond_arrow) => cond_arrow.to_node(),
        }
    }
}

impl ToAtom for Event {
    fn to_atom(&self) -> AutoStr {
        self.to_atom_str()
    }
}

impl ToNode for Arrow {
    fn to_node(&self) -> AutoNode {
        let mut node = AutoNode::new("arrow");

        if let Some(src) = &self.src {
            node.set_prop("from", Value::str(&*src.to_atom()));
        }
        if let Some(dest) = &self.dest {
            node.set_prop("to", Value::str(&*dest.to_atom()));
        }
        if let Some(with) = &self.with {
            node.set_prop("with", Value::str(&*with.to_atom()));
        }

        node
    }
}

impl AtomWriter for Arrow {
    fn write_atom(&self, f: &mut impl stdio::Write) -> auto_val::AutoResult<()> {
        write!(f, "(arrow")?;
        if let Some(src) = &self.src {
            write!(f, " (from {})", src.to_atom_str())?;
        }
        if let Some(dest) = &self.dest {
            write!(f, " (to {})", dest.to_atom_str())?;
        }
        if let Some(with) = &self.with {
            write!(f, " (with {})", with.to_atom_str())?;
        }
        write!(f, ")")?;
        Ok(())
    }
}

impl ToAtom for Arrow {
    fn to_atom(&self) -> AutoStr {
        self.to_atom_str()
    }
}

impl ToNode for CondArrow {
    fn to_node(&self) -> AutoNode {
        let mut node = AutoNode::new("cond-arrow");

        if let Some(src) = &self.src {
            node.set_prop("from", Value::str(&*src.to_atom()));
        }

        node.set_prop("cond", Value::str(&*self.cond.to_atom()));

        for sub in &self.subs {
            node.add_kid(sub.to_node());
        }

        node
    }
}

impl AtomWriter for CondArrow {
    fn write_atom(&self, f: &mut impl stdio::Write) -> auto_val::AutoResult<()> {
        write!(f, "(cond-arrow")?;
        if let Some(src) = &self.src {
            write!(f, " (from {})", src.to_atom_str())?;
        }
        write!(f, " (cond {})", self.cond.to_atom_str())?;
        for sub in &self.subs {
            write!(f, " {}", sub.to_atom_str())?;
        }
        write!(f, ")")?;
        Ok(())
    }
}

impl ToAtom for CondArrow {
    fn to_atom(&self) -> AutoStr {
        self.to_atom_str()
    }
}

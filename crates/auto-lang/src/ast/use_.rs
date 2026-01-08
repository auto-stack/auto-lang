use crate::ast::AtomWriter;
use auto_val::AutoStr;
use std::{fmt, io as stdio};

#[derive(Debug, Clone)]
pub enum UseKind {
    Auto,
    C,
    Rust,
}

#[derive(Debug, Clone)]
pub struct Use {
    pub kind: UseKind,
    pub paths: Vec<AutoStr>,
    pub items: Vec<AutoStr>,
}

impl fmt::Display for Use {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(use")?;
        match self.kind {
            UseKind::C => write!(f, " (kind c)")?,
            UseKind::Rust => write!(f, " (kind rust)")?,
            _ => (),
        }
        if !self.paths.is_empty() {
            write!(f, " (path {})", self.paths.join("."))?;
        }
        if !self.items.is_empty() {
            write!(f, " (items {})", self.items.join(","))?;
        }
        write!(f, ")")
    }
}

// ToAtom and ToNode implementations

use crate::ast::{ToAtom, ToAtomStr, ToNode};
use auto_val::{Arg as AutoValArg, Array, Node as AutoNode, Value};

impl ToNode for Use {
    fn to_node(&self) -> AutoNode {
        let mut node = AutoNode::new("use");

        // Only set kind property if not Auto (default)
        match self.kind {
            UseKind::C => node.set_prop("kind", Value::str("c")),
            UseKind::Rust => node.set_prop("kind", Value::str("rust")),
            UseKind::Auto => {} // Default, omit
        }

        if !self.paths.is_empty() {
            let path_str = self.paths.join(".");
            node.set_prop("path", Value::str(path_str.as_str()));
        }

        if !self.items.is_empty() {
            let items: Vec<Value> = self.items.iter().map(|s| Value::str(s.as_str())).collect();
            node.add_arg(AutoValArg::Pos(Value::array(Array::from_vec(items))));
        }

        node
    }
}

impl AtomWriter for Use {
    fn write_atom(&self, f: &mut impl stdio::Write) -> auto_val::AutoResult<()> {
        write!(f, "(use")?;
        match self.kind {
            UseKind::C => write!(f, " (kind c)")?,
            UseKind::Rust => write!(f, " (kind rust)")?,
            UseKind::Auto => {}
        }
        if !self.paths.is_empty() {
            write!(f, " (path {})", self.paths.join("."))?;
        }
        if !self.items.is_empty() {
            write!(f, " (items {})", self.items.join(","))?;
        }
        write!(f, ")")?;
        Ok(())
    }
}

impl ToAtom for Use {
    fn to_atom(&self) -> AutoStr {
        self.to_atom_str()
    }
}

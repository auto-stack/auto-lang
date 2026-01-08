use auto_val::AutoStr;
use std::fmt;

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

// ToAtom implementation

use crate::ast::ToAtom;
use auto_val::{Array, Arg as AutoValArg, Node, Value};

impl ToAtom for Use {
    fn to_atom(&self) -> Value {
        let mut node = Node::new("use");

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

        Value::Node(node)
    }
}

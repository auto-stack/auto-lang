use super::{Expr, ToNode};
use auto_val::Node as AutoNode;
use std::fmt;

#[derive(Debug, Clone)]
pub struct FStr {
    pub parts: Vec<Expr>,
}

impl FStr {
    pub fn new(parts: Vec<Expr>) -> Self {
        Self { parts }
    }
}

impl fmt::Display for FStr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(fstr")?;
        for part in self.parts.iter() {
            write!(f, " {}", part)?;
        }
        write!(f, ")")
    }
}

impl ToNode for FStr {
    fn to_node(&self) -> AutoNode {
        let mut node = AutoNode::new("fstr");
        for part in &self.parts {
            node.add_kid(part.to_node());
        }
        node
    }
}

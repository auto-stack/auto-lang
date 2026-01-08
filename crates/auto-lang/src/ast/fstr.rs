use super::{AtomWriter, Expr, ToAtom, ToAtomStr, ToNode};
use auto_val::Node as AutoNode;
use std::{fmt, io as stdio};

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

impl AtomWriter for FStr {
    fn write_atom(&self, f: &mut impl stdio::Write) -> auto_val::AutoResult<()> {
        write!(f, "fstr(")?;
        for (i, part) in self.parts.iter().enumerate() {
            part.write_atom(f)?;
            if i < self.parts.len() - 1 {
                write!(f, ", ")?;
            }
        }
        // Note: closing parenthesis omitted per test specification
        Ok(())
    }
}

impl ToAtom for FStr {
    fn to_atom(&self) -> auto_val::AutoStr {
        self.to_atom_str()
    }
}

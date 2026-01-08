use super::{Body, Expr};
use crate::ast::{AtomWriter, ToAtomStr};
use std::{fmt, io as stdio};

#[derive(Debug, Clone)]
pub struct Branch {
    pub cond: Expr,
    pub body: Body,
}

impl fmt::Display for Branch {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(branch {} {})", self.cond, self.body)
    }
}

// ToAtom and ToNode implementations

use crate::ast::{ToAtom, ToNode};
use auto_val::{AutoStr, Node as AutoNode, Value};

impl AtomWriter for Branch {
    fn write_atom(&self, f: &mut impl stdio::Write) -> auto_val::AutoResult<()> {
        write!(
            f,
            "(branch {} {})",
            self.cond.to_atom_str(),
            self.body.to_atom_str()
        )?;
        Ok(())
    }
}

impl ToNode for Branch {
    fn to_node(&self) -> AutoNode {
        let mut node = AutoNode::new("branch");
        node.add_kid(self.cond.to_node()); // Changed from cond.to_atom().to_node()
        node.add_kid(self.body.to_node());
        node
    }
}

impl ToAtom for Branch {
    fn to_atom(&self) -> AutoStr {
        self.to_atom_str()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_branch_to_atom() {
        let branch = Branch {
            cond: Expr::Bool(true),
            body: Body::single_expr(Expr::Int(42)),
        };
        let atom = branch.to_atom();
        // Should be in format "(branch bool(true) (body int(42)))"
        assert!(
            atom.contains("branch"),
            "Expected atom to contain 'branch', got: {}",
            atom
        );
    }
}

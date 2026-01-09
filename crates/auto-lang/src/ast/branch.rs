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
        // Output condition, opening brace, body statements, and closing brace
        write!(f, " {} {{", self.cond.to_atom_str())?;
        for stmt in &self.body.stmts {
            write!(f, " {}", stmt.to_atom_str())?;
        }
        if !self.body.stmts.is_empty() {
            write!(f, " ")?;
        }
        write!(f, "}}")?;
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
        assert_eq!(atom, " true { 42 }");
    }
}

use super::{Body, Branch};
use crate::ast::{AtomWriter, ToAtomStr};
use std::{fmt, io as stdio};

#[derive(Debug, Clone)]
pub struct If {
    pub branches: Vec<Branch>,
    pub else_: Option<Body>,
}

impl fmt::Display for If {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(if ")?;
        for branch in self.branches.iter() {
            write!(f, "{}", branch)?;
        }
        if let Some(else_stmt) = &self.else_ {
            write!(f, " (else {})", else_stmt)?;
        }
        write!(f, ")")
    }
}

// ToAtom and ToNode implementations

use crate::ast::{ToAtom, ToNode};
use auto_val::{AutoStr, Node as AutoNode};

impl AtomWriter for If {
    fn write_atom(&self, f: &mut impl stdio::Write) -> auto_val::AutoResult<()> {
        write!(f, "if {{")?;
        for (i, branch) in self.branches.iter().enumerate() {
            // Output branch without leading space for first branch
            let branch_str = branch.to_atom_str();
            // Branch format: " condition { body }"
            // We need to remove the leading space from branch.to_atom_str()
            let branch_str = branch_str.trim_start();
            write!(f, " {}", branch_str)?;
            // Add semicolon between branches
            if i < self.branches.len() - 1 || self.else_.is_some() {
                write!(f, ";")?;
            }
        }
        if let Some(else_body) = &self.else_ {
            // else is inside the if braces, output body without wrapping
            write!(f, " else {{")?;
            if !else_body.stmts.is_empty() {
                write!(f, " ")?;
                else_body.write_statements(f)?;
                write!(f, " ")?;
            }
            write!(f, "}}")?;
        }
        write!(f, " }}")?;
        Ok(())
    }
}

impl ToNode for If {
    fn to_node(&self) -> AutoNode {
        let mut node = AutoNode::new("if");
        for branch in &self.branches {
            node.add_kid(branch.to_node());
        }
        if let Some(else_body) = &self.else_ {
            let mut else_node = AutoNode::new("else");
            else_node.add_kid(else_body.to_node());
            node.add_kid(else_node);
        }
        node
    }
}

impl ToAtom for If {
    fn to_atom(&self) -> AutoStr {
        self.to_atom_str()
    }
}

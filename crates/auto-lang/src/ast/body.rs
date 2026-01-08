use super::{Expr, Stmt};
use crate::ast::{AtomWriter, ToAtomStr};
use std::{fmt, io as stdio};

#[derive(Debug, Clone)]
pub struct Body {
    pub stmts: Vec<Stmt>,
    pub has_new_line: bool,
}

impl Body {
    pub fn new() -> Self {
        Self {
            stmts: Vec::new(),
            has_new_line: false,
        }
    }

    pub fn single_expr(expr: Expr) -> Self {
        Self {
            stmts: vec![Stmt::Expr(expr)],
            has_new_line: false,
        }
    }
}

impl fmt::Display for Body {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(body ")?;
        for (i, stmt) in self.stmts.iter().enumerate() {
            write!(f, "{}", stmt)?;
            if i < self.stmts.len() - 1 {
                write!(f, " ")?;
            }
        }
        write!(f, ")")
    }
}

// ToAtom and ToNode implementations

use crate::ast::{ToAtom, ToNode};
use auto_val::{Arg as AutoValArg, Array, AutoStr, Node as AutoNode, Value};

impl AtomWriter for Body {
    fn write_atom(&self, f: &mut impl stdio::Write) -> auto_val::AutoResult<()> {
        write!(f, "body {{")?;
        for stmt in &self.stmts {
            write!(f, " {}", stmt.to_atom_str())?;
        }
        write!(f, " }}")?;
        Ok(())
    }
}

impl ToNode for Body {
    fn to_node(&self) -> AutoNode {
        let mut node = AutoNode::new("body");
        // Convert statements to an array
        let stmts: Vec<Value> = self
            .stmts
            .iter()
            .map(|stmt| Value::str(&*stmt.to_atom()))
            .collect();
        node.add_arg(AutoValArg::Pos(Value::array(Array::from_vec(stmts))));
        node
    }
}

impl ToAtom for Body {
    fn to_atom(&self) -> AutoStr {
        self.to_atom_str()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_body_to_atom_empty() {
        let body = Body::new();
        let atom = body.to_atom();
        // Should be in format "(body)"
        assert!(
            atom.contains("body"),
            "Expected atom to contain 'body', got: {}",
            atom
        );
    }

    #[test]
    fn test_body_to_atom_single_expr() {
        let body = Body::single_expr(Expr::Int(42));
        let atom = body.to_atom();
        // Should be in format "(body int(42))"
        assert!(
            atom.contains("body"),
            "Expected atom to contain 'body', got: {}",
            atom
        );
        assert!(
            atom.contains("int(42)"),
            "Expected atom to contain 'int(42)', got: {}",
            atom
        );
    }
}

use std::fmt::{self};

use super::{Body, Expr};

#[derive(Debug, Clone)]
pub struct Is {
    pub target: Expr,
    pub branches: Vec<IsBranch>,
}

#[derive(Debug, Clone)]
pub enum IsBranch {
    EqBranch(Expr, Body),
    IfBranch(Expr, Body),
    ElseBranch(Body),
}

impl fmt::Display for Is {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(is {} ", self.target)?;
        for (i, b) in self.branches.iter().enumerate() {
            match b {
                IsBranch::EqBranch(expr, body) => {
                    write!(f, "(eq ")?;
                    write!(f, "{} ", expr)?;
                    write!(f, "{}", body)?;
                    write!(f, ")")?;
                }
                IsBranch::IfBranch(expr, body) => {
                    write!(f, "(if ")?;
                    write!(f, "{} ", expr)?;
                    write!(f, "{}", body)?;
                    write!(f, ")")?;
                }
                IsBranch::ElseBranch(body) => {
                    write!(f, "(else ")?;
                    write!(f, "{}", body)?;
                    write!(f, ")")?;
                }
            }
            if i < self.branches.len() - 1 {
                write!(f, " ")?;
            }
        }
        write!(f, ")")
    }
}

// ToAtom implementation

use crate::ast::ToAtom;
use auto_val::{Node, Value};

impl ToAtom for Is {
    fn to_atom(&self) -> Value {
        let mut node = Node::new("is");
        node.add_kid(self.target.to_atom().to_node());

        for branch in &self.branches {
            match branch {
                IsBranch::EqBranch(expr, body) => {
                    let mut eq_node = Node::new("eq");
                    eq_node.add_kid(expr.to_atom().to_node());
                    eq_node.add_kid(body.to_atom().to_node());
                    node.add_kid(eq_node);
                }
                IsBranch::IfBranch(expr, body) => {
                    let mut if_node = Node::new("if");
                    if_node.add_kid(expr.to_atom().to_node());
                    if_node.add_kid(body.to_atom().to_node());
                    node.add_kid(if_node);
                }
                IsBranch::ElseBranch(body) => {
                    let mut else_node = Node::new("else");
                    else_node.add_kid(body.to_atom().to_node());
                    node.add_kid(else_node);
                }
            }
        }

        Value::Node(node)
    }
}

impl ToAtom for IsBranch {
    fn to_atom(&self) -> Value {
        match self {
            IsBranch::EqBranch(expr, body) => {
                let mut node = Node::new("eq");
                node.add_kid(expr.to_atom().to_node());
                node.add_kid(body.to_atom().to_node());
                Value::Node(node)
            }
            IsBranch::IfBranch(expr, body) => {
                let mut node = Node::new("if");
                node.add_kid(expr.to_atom().to_node());
                node.add_kid(body.to_atom().to_node());
                Value::Node(node)
            }
            IsBranch::ElseBranch(body) => {
                let mut node = Node::new("else");
                node.add_kid(body.to_atom().to_node());
                Value::Node(node)
            }
        }
    }
}

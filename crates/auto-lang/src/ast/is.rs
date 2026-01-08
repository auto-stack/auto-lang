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

// ToAtom and ToNode implementations

use crate::ast::{ToAtom, ToNode};
use auto_val::{Node as AutoNode, Value};

impl ToNode for Is {
    fn to_node(&self) -> AutoNode {
        let mut node = AutoNode::new("is");
        node.add_kid(self.target.to_atom().to_node());

        for branch in &self.branches {
            node.add_kid(branch.to_node());
        }

        node
    }
}

impl ToAtom for Is {
    fn to_atom(&self) -> Value {
        Value::Node(self.to_node())
    }
}

impl ToNode for IsBranch {
    fn to_node(&self) -> AutoNode {
        match self {
            IsBranch::EqBranch(expr, body) => {
                let mut node = AutoNode::new("eq");
                node.add_kid(expr.to_atom().to_node());
                node.add_kid(body.to_node());
                node
            }
            IsBranch::IfBranch(expr, body) => {
                let mut node = AutoNode::new("if");
                node.add_kid(expr.to_atom().to_node());
                node.add_kid(body.to_node());
                node
            }
            IsBranch::ElseBranch(body) => {
                let mut node = AutoNode::new("else");
                node.add_kid(body.to_node());
                node
            }
        }
    }
}

impl ToAtom for IsBranch {
    fn to_atom(&self) -> Value {
        Value::Node(self.to_node())
    }
}

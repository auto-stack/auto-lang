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

use std::fmt::{self, Write};

use super::{Body, Expr};

#[derive(Debug, Clone)]
pub struct When {
    pub target: Expr,
    pub branches: Vec<WhenBranch>,
}

#[derive(Debug, Clone)]
pub enum WhenBranch {
    IsBranch(Expr, Body),
    IfBranch(Expr, Body),
    ElseBranch(Body),
}

impl fmt::Display for When {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(when {} ", self.target)?;
        for b in &self.branches {
            match b {
                WhenBranch::IsBranch(expr, body) => {
                    write!(f, "(is ")?;
                    write!(f, "{} ", expr)?;
                    write!(f, "{}", body)?;
                    write!(f, ")")?;
                }
                WhenBranch::IfBranch(expr, body) => {
                    write!(f, "(if ")?;
                    write!(f, "{} ", expr)?;
                    write!(f, "{}", body)?;
                    write!(f, ")")?;
                }
                WhenBranch::ElseBranch(body) => {
                    write!(f, "(else ")?;
                    write!(f, "{}", body)?;
                    write!(f, ")")?;
                }
            }
        }
        write!(f, ")")
    }
}

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
        for (i, b) in self.branches.iter().enumerate() {
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
            if i < self.branches.len() - 1 {
                write!(f, " ")?;
            }
        }
        write!(f, ")")
    }
}

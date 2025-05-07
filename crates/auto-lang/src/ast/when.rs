use std::fmt::{self, Write};

use super::{Body, Expr};

#[derive(Debug, Clone)]
pub struct When {
    pub branches: Vec<WhenBranch>,
}

#[derive(Debug, Clone)]
pub enum WhenBranch {
    IsBranch(Expr, Body),
    IfBranch(Expr, Body),
    ElseBranch,
}

impl fmt::Display for When {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for b in &self.branches {
            match b {
                WhenBranch::IsBranch(expr, body) => todo!(),
                WhenBranch::IfBranch(expr, body) => todo!(),
                WhenBranch::ElseBranch => todo!(),
            }
        }
        Ok(())
    }
}

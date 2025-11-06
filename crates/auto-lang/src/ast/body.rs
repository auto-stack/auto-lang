use super::{Expr, Stmt};
use std::fmt;

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

use crate::ast::Expr;
use std::fmt;

#[derive(Debug, Clone)]
pub struct Goto {
    pub src: Option<Expr>,
    pub dest: Expr,
    pub with: Option<Expr>,
}

impl Goto {
    pub fn new(src: Option<Expr>, dest: Expr, with: Option<Expr>) -> Self {
        Self { src, dest, with }
    }
}

impl fmt::Display for Goto {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(goto")?;
        if let Some(src) = &self.src {
            write!(f, " (from {})", src)?;
        }
        write!(f, " (to {})", self.dest)?;
        if let Some(with) = &self.with {
            write!(f, " (with {})", with)?;
        }
        write!(f, ")")
    }
}

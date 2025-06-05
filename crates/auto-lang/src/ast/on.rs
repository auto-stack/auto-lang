use crate::ast::Expr;
use std::fmt;

#[derive(Debug, Clone)]
pub struct Arrow {
    pub src: Option<Expr>,
    pub dest: Option<Expr>,
    pub with: Option<Expr>,
}

#[derive(Debug, Clone)]
pub struct OnEvents {
    pub branches: Vec<Arrow>,
}

impl Arrow {
    pub fn new(src: Option<Expr>, dest: Option<Expr>, with: Option<Expr>) -> Self {
        Self { src, dest, with }
    }
}

impl OnEvents {
    pub fn new(branches: Vec<Arrow>) -> Self {
        Self { branches }
    }
}

impl fmt::Display for Arrow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(arrow")?;
        if let Some(src) = &self.src {
            write!(f, " (from {})", src)?;
        }
        if let Some(dest) = &self.dest {
            write!(f, " (to {})", dest)?;
        }
        if let Some(with) = &self.with {
            write!(f, " (with {})", with)?;
        }
        write!(f, ")")
    }
}

impl fmt::Display for OnEvents {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(on")?;
        for branch in &self.branches {
            write!(f, " {}", branch)?;
        }
        write!(f, ")")
    }
}

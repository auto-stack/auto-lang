use core::fmt;

use super::Expr;

#[derive(Debug, Clone)]
pub struct Range {
    pub start: Box<Expr>,
    pub end: Box<Expr>,
    pub eq: bool,
}

impl fmt::Display for Range {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(range (start {}) (end {}) (eq {}))", self.start, self.end, self.eq)
    }
}
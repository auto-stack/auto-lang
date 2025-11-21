use super::{Body, Call, Expr, Name};
use std::fmt;

#[derive(Debug, Clone)]
pub struct For {
    pub iter: Iter,
    pub range: Expr,
    pub body: Body,
    pub new_line: bool,
    // TODO: maybe we could put mid block here
}

#[derive(Debug, Clone)]
pub enum Iter {
    Indexed(/*index*/ Name, /*iter*/ Name),
    Named(/*iter*/ Name),
    Call(Call),
}

impl fmt::Display for For {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(for {} {} {})", self.iter, self.range, self.body)
    }
}

impl fmt::Display for Iter {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Iter::Indexed(index, iter) => write!(f, "((name {}) (name {}))", index, iter),
            Iter::Named(iter) => write!(f, "(name {})", iter),
            Iter::Call(call) => write!(f, "(call {})", call),
        }
    }
}

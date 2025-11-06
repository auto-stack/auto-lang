use super::{Body, Branch};
use std::fmt;

#[derive(Debug, Clone)]
pub struct If {
    pub branches: Vec<Branch>,
    pub else_: Option<Body>,
}

impl fmt::Display for If {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(if ")?;
        for branch in self.branches.iter() {
            write!(f, "{}", branch)?;
        }
        if let Some(else_stmt) = &self.else_ {
            write!(f, " (else {})", else_stmt)?;
        }
        write!(f, ")")
    }
}

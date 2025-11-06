use super::Expr;
use std::fmt;

#[derive(Debug, Clone)]
pub struct FStr {
    pub parts: Vec<Expr>,
}

impl FStr {
    pub fn new(parts: Vec<Expr>) -> Self {
        Self { parts }
    }
}

impl fmt::Display for FStr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(fstr")?;
        for part in self.parts.iter() {
            write!(f, " {}", part)?;
        }
        write!(f, ")")
    }
}

use super::{Args, Expr};
use std::fmt;

#[derive(Debug, Clone)]
pub struct Grid {
    pub head: Args,
    pub data: Vec<Vec<Expr>>,
}

impl fmt::Display for Grid {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(grid")?;
        if !self.head.is_empty() {
            write!(f, " (head")?;
            for arg in self.head.args.iter() {
                write!(f, " {}", arg)?;
            }
            write!(f, ")")?;
        }
        if !self.data.is_empty() {
            write!(f, " (data")?;
            for row in self.data.iter() {
                write!(f, " (row ")?;
                for (j, cell) in row.iter().enumerate() {
                    write!(f, "{}", cell)?;
                    if j < row.len() - 1 {
                        write!(f, " ")?;
                    }
                }
                write!(f, ")")?;
            }
            write!(f, ")")?;
        }
        write!(f, ")")
    }
}

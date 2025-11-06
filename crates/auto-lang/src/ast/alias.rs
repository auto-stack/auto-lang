use super::Name;
use std::fmt;

#[derive(Debug, Clone)]
pub struct Alias {
    pub alias: Name,
    pub target: Name,
}

impl fmt::Display for Alias {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(alias (name {}) (target {}))", self.alias, self.target)
    }
}

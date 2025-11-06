use auto_val::AutoStr;
use std::fmt;

#[derive(Debug, Clone)]
pub enum UseKind {
    Auto,
    C,
    Rust,
}

#[derive(Debug, Clone)]
pub struct Use {
    pub kind: UseKind,
    pub paths: Vec<AutoStr>,
    pub items: Vec<AutoStr>,
}

impl fmt::Display for Use {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(use")?;
        match self.kind {
            UseKind::C => write!(f, " (kind c)")?,
            UseKind::Rust => write!(f, " (kind rust)")?,
            _ => (),
        }
        if !self.paths.is_empty() {
            write!(f, " (path {})", self.paths.join("."))?;
        }
        if !self.items.is_empty() {
            write!(f, " (items {})", self.items.join(","))?;
        }
        write!(f, ")")
    }
}

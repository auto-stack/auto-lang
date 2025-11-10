use super::{Name, Type};
use std::fmt;

#[derive(Debug, Clone)]
pub struct Tag {
    pub name: Name,
    pub fields: Vec<TagField>,
}

#[derive(Debug, Clone)]
pub struct TagField {
    pub name: Name,
    pub ty: Type,
}

impl Tag {
    pub fn new(name: Name, fields: Vec<TagField>) -> Self {
        Self { name, fields }
    }
}

impl fmt::Display for Tag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "tag {} {{", self.name)?;
        for field in &self.fields {
            write!(f, "\n    {}", field)?;
        }
        write!(f, "\n}}")
    }
}

impl fmt::Display for TagField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.name, self.ty)
    }
}

use super::{Name, Type};
use std::fmt;

#[derive(Debug, Clone)]
pub struct Union {
    pub name: Name,
    pub fields: Vec<UnionField>,
}

#[derive(Debug, Clone)]
pub struct UnionField {
    pub name: Name,
    pub ty: Type,
}

impl fmt::Display for UnionField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.name, self.ty)
    }
}

impl fmt::Display for Union {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "union {} {{", self.name)?;
        for field in &self.fields {
            write!(f, "\n    {}", field)?;
        }
        write!(f, "\n}}")
    }
}

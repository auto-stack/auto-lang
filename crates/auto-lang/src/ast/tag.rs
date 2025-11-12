use super::{Name, Type};
use auto_val::AutoStr;
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

    pub fn enum_name(&self, field_name: &str) -> AutoStr {
        format!("{}_{}", self.name.to_uppercase(), field_name.to_uppercase()).into()
    }

    pub fn has_field(&self, name: &Name) -> bool {
        self.fields.iter().any(|f| f.name == *name)
    }

    pub fn get_field_type(&self, name: &Name) -> Type {
        self.fields
            .iter()
            .find(|f| f.name == *name)
            .map(|f| f.ty.clone())
            .unwrap_or(Type::Unknown)
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

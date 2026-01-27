use super::{Name, Type};
use crate::ast::GenericParam;
use crate::ast::{AtomWriter, ToAtomStr};
use auto_val::AutoStr;
use std::{fmt, io as stdio};

#[derive(Debug, Clone)]
pub struct Tag {
    pub name: Name,
    pub generic_params: Vec<GenericParam>,  // Generic parameters (Plan 052: type + const)
    pub fields: Vec<TagField>,
    pub methods: Vec<super::Fn>,
}

#[derive(Debug, Clone)]
pub struct TagField {
    pub name: Name,
    pub ty: Type,
}

impl Tag {
    pub fn new(name: Name, fields: Vec<TagField>) -> Self {
        Self {
            name,
            generic_params: Vec::new(),
            fields,
            methods: Vec::new(),
        }
    }

    pub fn with_methods(name: Name, fields: Vec<TagField>, methods: Vec<super::Fn>) -> Self {
        Self {
            name,
            generic_params: Vec::new(),
            fields,
            methods,
        }
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
        write!(f, "tag {}", self.name)?;
        if !self.generic_params.is_empty() {
            write!(f, "<")?;
            for (i, param) in self.generic_params.iter().enumerate() {
                if i > 0 {
                    write!(f, ", ")?;
                }
                write!(f, "{}", param)?;
            }
            write!(f, ">")?;
        }
        write!(f, " {{")?;
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

// ToAtom and ToNode implementations

use crate::ast::{ToAtom, ToNode};
use auto_val::{Node as AutoNode, Value};

impl ToNode for Tag {
    fn to_node(&self) -> AutoNode {
        let mut node = AutoNode::new("tag");
        node.set_prop("name", Value::str(self.name.as_str()));

        // Add generic parameters if present
        if !self.generic_params.is_empty() {
            let params: Vec<String> = self.generic_params.iter()
                .map(|p| format!("{}", p))
                .collect();
            node.set_prop("generic_params", Value::str(params.join(", ").as_str()));
        }

        for field in &self.fields {
            node.add_kid(field.to_node());
        }

        for method in &self.methods {
            node.add_kid(method.to_node());
        }

        node
    }
}

impl AtomWriter for Tag {
    fn write_atom(&self, f: &mut impl stdio::Write) -> auto_val::AutoResult<()> {
        write!(f, "tag(name(\"{}\")", self.name)?;
        if !self.generic_params.is_empty() {
            write!(f, "<")?;
            for (i, param) in self.generic_params.iter().enumerate() {
                if i > 0 {
                    write!(f, ", ")?;
                }
                write!(f, "{}", param)?;
            }
            write!(f, ">")?;
        }
        write!(f, ") {{")?;
        for field in &self.fields {
            write!(f, " {}", field.to_atom_str())?;
        }
        write!(f, " }}")?;
        Ok(())
    }
}

impl ToAtom for Tag {
    fn to_atom(&self) -> AutoStr {
        self.to_atom_str()
    }
}

impl AtomWriter for TagField {
    fn write_atom(&self, f: &mut impl stdio::Write) -> auto_val::AutoResult<()> {
        write!(
            f,
            "field(name(\"{}\"), type({}))",
            self.name,
            self.ty.to_atom_str()
        )?;
        Ok(())
    }
}

impl ToNode for TagField {
    fn to_node(&self) -> AutoNode {
        let mut node = AutoNode::new("field");
        node.set_prop("name", Value::str(self.name.as_str()));
        node.set_prop("type", Value::str(&*self.ty.to_atom()));
        node
    }
}

impl ToAtom for TagField {
    fn to_atom(&self) -> AutoStr {
        self.to_atom_str()
    }
}

use std::{fmt, io as stdio};

use super::{Fn, Name, Type};
use crate::ast::{AtomWriter, GenericParam, ToAtomStr};
use auto_val::AutoStr;

/// The kind of enum, determining its semantics.
///
/// - **Scalar**: C-style enumerations with optional integer values and optional repr type.
///   Examples: `enum Color { Red, Green }`, `enum HttpCode u16 { OK = 200 }`
///
/// - **Homogeneous**: All variants share a single payload type.
///   Example: `enum Vertex Point { LeftTop, RightTop }`
///
/// - **Heterogeneous**: Each variant may have a different payload type (algebraic data type).
///   Example: `enum Msg { Quit, Move Point, Write string }`
#[derive(Debug, Clone, PartialEq)]
pub enum EnumKind {
    /// C-style scalar enumeration with optional explicit representation type.
    /// `enum Color { Red, Green }` or `enum HttpCode u16 { OK = 200 }`
    Scalar {
        /// Optional explicit representation type (e.g., `u16` in `enum HttpCode u16`).
        repr_type: Option<Type>,
    },

    /// All variants share the same payload type.
    /// `enum Vertex Point { LeftTop, RightTop }` — every variant is a `Point`.
    Homogeneous {
        /// The shared payload type for all variants.
        payload_type: Type,
    },

    /// Each variant may carry a different payload type (sum type / ADT).
    /// `enum Msg { Quit, Move Point, Write string }`
    Heterogeneous {
        /// Generic parameters for the enum (e.g., `T` in `enum Option<T>`).
        generic_params: Vec<GenericParam>,
        /// Methods defined on the enum.
        methods: Vec<Fn>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnumDecl {
    pub name: AutoStr,
    pub items: Vec<EnumItem>,
    pub kind: EnumKind,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnumItem {
    pub name: AutoStr,
    /// Scalar form: optional explicit integer value (e.g., `OK = 200`).
    pub scalar_value: Option<i32>,
    /// Heterogeneous form: the payload type for this variant (e.g., `Point` in `Move Point`).
    pub payload_type: Option<Type>,
}

impl EnumItem {
    /// Backward-compatible helper: returns the scalar value or 0 if not set.
    pub fn value(&self) -> i32 {
        self.scalar_value.unwrap_or(0)
    }
}

impl fmt::Display for EnumItem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(item (name {}) (value {}))", self.name, self.value())
    }
}

impl fmt::Display for EnumDecl {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(enum (name {}) ", self.name)?;
        for (i, item) in self.items.iter().enumerate() {
            if i > 0 {
                write!(f, " ")?;
            }
            write!(f, "{}", item)?;
        }
        write!(f, ")")
    }
}

impl EnumDecl {
    pub fn new(name: AutoStr, items: Vec<EnumItem>) -> Self {
        Self {
            name,
            items,
            kind: EnumKind::Scalar {
                repr_type: None,
            },
        }
    }

    pub fn unique_name(&self) -> AutoStr {
        format!("{}", self.name).into()
    }

    pub fn get_item(&self, name: &str) -> Option<&EnumItem> {
        self.items.iter().find(|item| item.name == name)
    }

    pub fn default_value(&self) -> i32 {
        self.items.first().map_or(0, |item| item.value())
    }
}

// ToAtom and ToNode implementations

use crate::ast::{ToAtom, ToNode};
use auto_val::{Node as AutoNode, Value};

impl ToNode for EnumDecl {
    fn to_node(&self) -> AutoNode {
        let mut node = AutoNode::new("enum");
        node.set_prop("name", Value::str(self.name.as_str()));

        for item in &self.items {
            node.add_kid(item.to_node());
        }

        node
    }
}

impl AtomWriter for EnumDecl {
    fn write_atom(&self, f: &mut impl stdio::Write) -> auto_val::AutoResult<()> {
        write!(f, "enum(name(\"{}\")) {{", self.name)?;
        for item in &self.items {
            write!(f, " {}", item.to_atom_str())?;
        }
        write!(f, " }}")?;
        Ok(())
    }
}

impl ToAtom for EnumDecl {
    fn to_atom(&self) -> AutoStr {
        self.to_atom_str()
    }
}

impl AtomWriter for EnumItem {
    fn write_atom(&self, f: &mut impl stdio::Write) -> auto_val::AutoResult<()> {
        write!(
            f,
            "item(name(\"{}\"), value(int({})))",
            self.name,
            self.value()
        )?;
        Ok(())
    }
}

impl ToNode for EnumItem {
    fn to_node(&self) -> AutoNode {
        let mut node = AutoNode::new("item");
        node.set_prop("name", Value::str(self.name.as_str()));
        node.set_prop("value", Value::Int(self.value()));
        node
    }
}

impl ToAtom for EnumItem {
    fn to_atom(&self) -> AutoStr {
        self.to_atom_str()
    }
}

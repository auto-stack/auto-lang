use std::fmt;

use auto_val::AutoStr;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EnumDecl {
    pub name: AutoStr,
    pub items: Vec<EnumItem>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EnumItem {
    pub name: AutoStr,
    pub value: i32,
}

impl fmt::Display for EnumItem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(item (name {}) (value {}))", self.name, self.value)
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
        Self { name, items }
    }

    pub fn unique_name(&self) -> AutoStr {
        format!("{}", self.name).into()
    }

    pub fn get_item(&self, name: &str) -> Option<&EnumItem> {
        self.items.iter().find(|item| item.name == name)
    }

    pub fn default_value(&self) -> i32 {
        self.items.first().map_or(0, |item| item.value)
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

impl ToAtom for EnumDecl {
    fn to_atom(&self) -> Value {
        Value::Node(self.to_node())
    }
}

impl ToNode for EnumItem {
    fn to_node(&self) -> AutoNode {
        let mut node = AutoNode::new("item");
        node.set_prop("name", Value::str(self.name.as_str()));
        node.set_prop("value", Value::Int(self.value));
        node
    }
}

impl ToAtom for EnumItem {
    fn to_atom(&self) -> Value {
        Value::Node(self.to_node())
    }
}

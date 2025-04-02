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
}

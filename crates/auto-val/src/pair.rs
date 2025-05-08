use crate::AutoStr;
use std::fmt::{self, Display, Formatter};

#[derive(Debug, Clone, PartialEq, Hash, Ord, Eq, PartialOrd)]
pub enum ValueKey {
    Str(AutoStr),
    Int(i32),
    Bool(bool),
}

impl ValueKey {
    pub fn name(&self) -> Option<&str> {
        match self {
            ValueKey::Str(s) => Some(s.as_str()),
            _ => None,
        }
    }
}

impl Display for ValueKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ValueKey::Str(s) => write!(f, "{}", s),
            ValueKey::Int(i) => write!(f, "{}", i),
            ValueKey::Bool(b) => write!(f, "{}", b),
        }
    }
}

impl Into<ValueKey> for i32 {
    fn into(self) -> ValueKey {
        ValueKey::Int(self)
    }
}

impl Into<ValueKey> for bool {
    fn into(self) -> ValueKey {
        ValueKey::Bool(self)
    }
}

impl Into<ValueKey> for i64 {
    fn into(self) -> ValueKey {
        ValueKey::Int(self as i32)
    }
}

impl Into<ValueKey> for String {
    fn into(self) -> ValueKey {
        ValueKey::Str(self.into())
    }
}

impl Into<ValueKey> for &str {
    fn into(self) -> ValueKey {
        ValueKey::Str(self.into())
    }
}

impl From<AutoStr> for ValueKey {
    fn from(s: AutoStr) -> ValueKey {
        ValueKey::Str(s)
    }
}

impl ValueKey {
    pub fn to_astr(&self) -> AutoStr {
        self.to_string().into()
    }
}

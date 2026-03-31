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

impl From<i32> for ValueKey {
    fn from(val: i32) -> Self {
        ValueKey::Int(val)
    }
}

impl From<bool> for ValueKey {
    fn from(val: bool) -> Self {
        ValueKey::Bool(val)
    }
}

impl From<i64> for ValueKey {
    fn from(val: i64) -> Self {
        ValueKey::Int(val as i32)
    }
}

impl From<String> for ValueKey {
    fn from(val: String) -> Self {
        ValueKey::Str(val.into())
    }
}

impl From<&str> for ValueKey {
    fn from(val: &str) -> Self {
        ValueKey::Str(val.into())
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

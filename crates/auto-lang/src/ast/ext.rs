use super::{Fn, Name};
use crate::ast::{AtomWriter, ToNode};
use auto_val::AutoStr;
use std::{fmt, io as stdio};

// Re-export AutoNode from auto_val for convenience
pub use auto_val::Node as AutoNode;

/// Type extension statement (like Rust's impl)
///
/// Allows adding methods to a type AFTER its initial definition.
/// This enables extending built-in types (str, cstr, int, etc.) that
/// are defined in the Rust compiler implementation.
///
/// # Example
///
/// ```auto
/// ext str {
///     fn len() int {
///         return .size  // .prop accesses self.prop
///     }
///
///     static fn new(data *char, size int) str {
///         return str_new(data, size)
///     }
/// }
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Ext {
    /// Type being extended (e.g., "str", "Point")
    pub target: Name,

    /// Methods to add to the type
    pub methods: Vec<Fn>,
}

impl Ext {
    /// Create a new type extension
    pub fn new(target: Name, methods: Vec<Fn>) -> Self {
        Self { target, methods }
    }
}

impl fmt::Display for Ext {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(ext (target {}) (methods", self.target)?;
        for method in &self.methods {
            write!(f, " {}", method)?;
        }
        write!(f, "))")
    }
}

impl AtomWriter for Ext {
    fn write_atom(&self, f: &mut impl stdio::Write) -> auto_val::AutoResult<()> {
        write!(f, "ext({}, ", self.target)?;
        for (i, method) in self.methods.iter().enumerate() {
            method.write_atom(f)?;
            if i < self.methods.len() - 1 {
                write!(f, ", ")?;
            }
        }
        write!(f, ")")?;
        Ok(())
    }
}

impl ToNode for Ext {
    fn to_node(&self) -> AutoNode {
        let mut node = AutoNode::new("ext");
        node.set_prop("target", auto_val::Value::Str(self.target.clone()));
        for method in &self.methods {
            node.add_kid(method.to_node());
        }
        node
    }
}

impl crate::ast::ToAtom for Ext {
    fn to_atom(&self) -> AutoStr {
        // Use the Display representation as atom format
        self.to_string().into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ext_creation() {
        let ext = Ext::new("str".into(), vec![]);
        assert_eq!(ext.target, "str");
        assert!(ext.methods.is_empty());
    }

    #[test]
    fn test_ext_display() {
        let ext = Ext::new("str".into(), vec![]);
        let display = format!("{}", ext);
        assert!(display.contains("ext"));
        assert!(display.contains("str"));
    }

    #[test]
    fn test_ext_equality() {
        let ext1 = Ext::new("str".into(), vec![]);
        let ext2 = Ext::new("str".into(), vec![]);
        assert_eq!(ext1, ext2);

        let ext3 = Ext::new("int".into(), vec![]);
        assert_ne!(ext1, ext3);
    }
}

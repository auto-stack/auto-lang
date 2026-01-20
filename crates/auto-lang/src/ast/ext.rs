use super::{Fn, Member, Name};
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
/// With the enhanced ext mechanism, same-module extensions can also
/// add private fields to enable platform-specific memory layouts.
///
/// # Example
///
/// ```auto
/// // Method extension (always allowed)
/// ext str {
///     fn len() int {
///         return .size  // .prop accesses self.prop
///     }
///
///     static fn new(data *char, size int) str {
///         return str_new(data, size)
///     }
/// }
///
/// // Field extension (only in same module)
/// // In io.c.at:
/// ext File {
///     _fp *FILE  // Private field for C platform
///
///     fn read() str {
///         // Use ._fp here
///     }
/// }
/// ```
#[derive(Debug, Clone)]
pub struct Ext {
    /// Type being extended (e.g., "str", "Point", "File")
    pub target: Name,

    /// Private fields to add to the type (same-module only)
    pub fields: Vec<Member>,

    /// Methods to add to the type
    pub methods: Vec<Fn>,

    /// Module path where this ext is defined (e.g., "auto.io")
    pub module_path: AutoStr,

    /// Whether this ext is defined in the same module as the target type
    pub is_same_module: bool,
}

impl Ext {
    /// Create a new type extension
    pub fn new(target: Name, methods: Vec<Fn>) -> Self {
        Self {
            target,
            fields: Vec::new(),
            methods,
            module_path: AutoStr::from(""),
            is_same_module: false,
        }
    }

    /// Create a new type extension with fields
    pub fn with_fields(
        target: Name,
        fields: Vec<Member>,
        methods: Vec<Fn>,
        module_path: AutoStr,
        is_same_module: bool,
    ) -> Self {
        Self {
            target,
            fields,
            methods,
            module_path,
            is_same_module,
        }
    }
}

impl PartialEq for Ext {
    fn eq(&self, other: &Self) -> bool {
        self.target == other.target
            && self.fields.len() == other.fields.len()
            && self.methods == other.methods
            && self.module_path == other.module_path
            && self.is_same_module == other.is_same_module
    }
}

impl fmt::Display for Ext {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(ext (target {})", self.target)?;

        // Add fields if present
        if !self.fields.is_empty() {
            write!(f, " (fields")?;
            for field in &self.fields {
                write!(f, " {}", field)?;
            }
            write!(f, ")")?;
        }

        // Add methods
        write!(f, " (methods")?;
        for method in &self.methods {
            write!(f, " {}", method)?;
        }
        write!(f, "))")
    }
}

impl AtomWriter for Ext {
    fn write_atom(&self, f: &mut impl stdio::Write) -> auto_val::AutoResult<()> {
        write!(f, "ext({}, ", self.target)?;

        // Write fields if present
        if !self.fields.is_empty() {
            write!(f, "fields: [")?;
            for (i, field) in self.fields.iter().enumerate() {
                write!(f, "({}:{})", field.name, field.ty)?;
                if i < self.fields.len() - 1 {
                    write!(f, ", ")?;
                }
            }
            write!(f, "], ")?;
        }

        // Write methods
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

        // Add fields if present
        if !self.fields.is_empty() {
            let mut fields_node = AutoNode::new("fields");
            for field in &self.fields {
                fields_node.add_kid(field.to_node());
            }
            node.add_kid(fields_node);
        }

        // Add methods
        for method in &self.methods {
            node.add_kid(method.to_node());
        }

        // Add metadata
        node.set_prop("module_path", auto_val::Value::Str(self.module_path.clone()));
        node.set_prop(
            "is_same_module",
            auto_val::Value::Bool(self.is_same_module),
        );

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
    use crate::ast::Type;

    #[test]
    fn test_ext_creation() {
        let ext = Ext::new("str".into(), vec![]);
        assert_eq!(ext.target, "str");
        assert!(ext.methods.is_empty());
        assert!(ext.fields.is_empty());
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

    #[test]
    fn test_ext_with_fields() {
        let field = Member::new("_fp".into(), Type::Unknown, None);
        let ext = Ext::with_fields(
            "File".into(),
            vec![field],
            vec![],
            "auto.io".into(),
            true,
        );

        assert_eq!(ext.target, "File");
        assert_eq!(ext.fields.len(), 1);
        assert_eq!(ext.fields[0].name, "_fp");
        assert_eq!(ext.module_path, "auto.io");
        assert!(ext.is_same_module);
    }

    #[test]
    fn test_ext_display_with_fields() {
        let field = Member::new("_fp".into(), Type::Unknown, None);
        let ext = Ext::with_fields(
            "File".into(),
            vec![field],
            vec![],
            "auto.io".into(),
            true,
        );

        let display = format!("{}", ext);
        assert!(display.contains("ext"));
        assert!(display.contains("File"));
        assert!(display.contains("fields"));
    }

    #[test]
    fn test_ext_to_node_with_fields() {
        let field = Member::new("_fp".into(), Type::Unknown, None);
        let ext = Ext::with_fields(
            "File".into(),
            vec![field],
            vec![],
            "auto.io".into(),
            true,
        );

        let node = ext.to_node();
        let target_value = node.get_prop("target");
        let module_path_value = node.get_prop("module_path");
        let is_same_module_value = node.get_prop("is_same_module");

        assert_eq!(target_value.as_str(), "File");
        assert_eq!(module_path_value.as_str(), "auto.io");
        assert_eq!(is_same_module_value.as_bool(), true);
    }
}

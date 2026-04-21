//! Qualified Name type for canonical native function identification.
//!
//! Provides a structured way to reference native functions using a
//! dot-separated module path (e.g., "auto.fs.read_text", "auto.list.push").
//!
//! Plan 203 Phase 1: Introduces the type alongside existing short names.
//! No breaking changes — existing code paths work unchanged.

use std::fmt;

/// A canonical qualified name for native functions.
/// Format: "module.path.function_name" (e.g., "auto.fs.read_text", "auto.list.push")
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct QualifiedName {
    /// Full qualified path (e.g., "auto.fs.read_text")
    path: String,
}

impl QualifiedName {
    /// Create a new qualified name from a dot-separated path string.
    pub fn new(path: &str) -> Self {
        Self {
            path: path.to_string(),
        }
    }

    /// Get the full qualified path.
    pub fn path(&self) -> &str {
        &self.path
    }

    /// Get the last component (function name).
    ///
    /// ```
    /// # use auto_lang::vm::qualified_name::QualifiedName;
    /// let qn = QualifiedName::new("auto.fs.read_text");
    /// assert_eq!(qn.name(), "read_text");
    /// ```
    pub fn name(&self) -> &str {
        self.path.rsplit('.').next().unwrap_or(&self.path)
    }

    /// Get the module path (everything before the last dot).
    ///
    /// ```
    /// # use auto_lang::vm::qualified_name::QualifiedName;
    /// let qn = QualifiedName::new("auto.fs.read_text");
    /// assert_eq!(qn.module(), Some("auto.fs"));
    /// ```
    pub fn module(&self) -> Option<&str> {
        self.path.rsplitn(2, '.').nth(1)
    }
}

impl fmt::Display for QualifiedName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.path)
    }
}

impl From<&str> for QualifiedName {
    fn from(s: &str) -> Self {
        QualifiedName::new(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_path() {
        let qn = QualifiedName::new("auto.fs.read_text");
        assert_eq!(qn.path(), "auto.fs.read_text");
        assert_eq!(qn.name(), "read_text");
        assert_eq!(qn.module(), Some("auto.fs"));
    }

    #[test]
    fn test_simple_name() {
        let qn = QualifiedName::new("print");
        assert_eq!(qn.path(), "print");
        assert_eq!(qn.name(), "print");
        assert_eq!(qn.module(), None);
    }

    #[test]
    fn test_two_part_path() {
        let qn = QualifiedName::new("List.push");
        assert_eq!(qn.path(), "List.push");
        assert_eq!(qn.name(), "push");
        assert_eq!(qn.module(), Some("List"));
    }

    #[test]
    fn test_display() {
        let qn = QualifiedName::new("auto.list.push");
        assert_eq!(format!("{}", qn), "auto.list.push");
    }

    #[test]
    fn test_from_str() {
        let qn: QualifiedName = "auto.math.abs".into();
        assert_eq!(qn.path(), "auto.math.abs");
        assert_eq!(qn.name(), "abs");
        assert_eq!(qn.module(), Some("auto.math"));
    }

    #[test]
    fn test_equality() {
        let a = QualifiedName::new("auto.list.push");
        let b = QualifiedName::new("auto.list.push");
        let c = QualifiedName::new("auto.list.pop");
        assert_eq!(a, b);
        assert_ne!(a, c);
    }
}

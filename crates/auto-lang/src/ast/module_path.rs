//! Plan 131: Module Path Syntax
//!
//! Represents the different ways to reference a module:
//! - `db` → same directory
//! - `super.db` → parent directory
//! - `pac.db` → package root
//! - `pac.api.handlers` → deep path from root
//! - `database.connection` → from dependency

use auto_val::AutoStr;

/// The prefix of a module path
#[derive(Debug, Clone, PartialEq)]
pub enum PathPrefix {
    /// No prefix - same directory: `use db`
    None,
    /// `super.` prefix - parent directory: `use super.db`
    Super,
    /// `pac.` prefix - package root: `use pac.db`
    Pac,
    /// Dependency name - from declared dep: `use database.connection`
    Dep(AutoStr),
}

/// A fully parsed module path
#[derive(Debug, Clone, PartialEq)]
pub struct ModulePath {
    /// The prefix (None, Super, Pac, or Dep name)
    pub prefix: PathPrefix,
    /// The path segments (e.g., ["api", "handlers"] for "pac.api.handlers")
    pub segments: Vec<AutoStr>,
    /// Symbols to import (after `:`)
    pub items: Vec<AutoStr>,
}

impl ModulePath {
    /// Create a new module path
    pub fn new(prefix: PathPrefix, segments: Vec<AutoStr>, items: Vec<AutoStr>) -> Self {
        Self { prefix, segments, items }
    }

    /// Create a simple path (same directory)
    pub fn local(segments: Vec<AutoStr>) -> Self {
        Self::new(PathPrefix::None, segments, Vec::new())
    }

    /// Create a super path (parent directory)
    pub fn super_path(segments: Vec<AutoStr>) -> Self {
        Self::new(PathPrefix::Super, segments, Vec::new())
    }

    /// Create a package path (from root)
    pub fn pac(segments: Vec<AutoStr>) -> Self {
        Self::new(PathPrefix::Pac, segments, Vec::new())
    }

    /// Create a dependency path
    pub fn dep(dep_name: AutoStr, segments: Vec<AutoStr>) -> Self {
        Self::new(PathPrefix::Dep(dep_name), segments, Vec::new())
    }

    /// Add import items
    pub fn with_items(mut self, items: Vec<AutoStr>) -> Self {
        self.items = items;
        self
    }

    /// Get the full path as a string (for display)
    pub fn display(&self) -> String {
        let mut result = String::new();
        match &self.prefix {
            PathPrefix::None => {}
            PathPrefix::Super => result.push_str("super."),
            PathPrefix::Pac => result.push_str("pac."),
            PathPrefix::Dep(name) => {
                result.push_str(name.as_str());
                result.push('.');
            }
        }
        result.push_str(&self.segments.join("."));
        result
    }
}

impl std::fmt::Display for ModulePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_local_path() {
        let path = ModulePath::local(vec!["db".into()]);
        assert_eq!(path.display(), "db");
        assert_eq!(path.prefix, PathPrefix::None);
    }

    #[test]
    fn test_super_path() {
        let path = ModulePath::super_path(vec!["db".into()]);
        assert_eq!(path.display(), "super.db");
        assert_eq!(path.prefix, PathPrefix::Super);
    }

    #[test]
    fn test_pac_path() {
        let path = ModulePath::pac(vec!["api".into(), "handlers".into()]);
        assert_eq!(path.display(), "pac.api.handlers");
        assert_eq!(path.prefix, PathPrefix::Pac);
    }

    #[test]
    fn test_dep_path() {
        let path = ModulePath::dep("database".into(), vec!["connection".into()]);
        assert_eq!(path.display(), "database.connection");
        assert_eq!(path.prefix, PathPrefix::Dep("database".into()));
    }

    #[test]
    fn test_with_items() {
        let path = ModulePath::local(vec!["db".into()])
            .with_items(vec!["load".into(), "save".into()]);
        assert_eq!(path.items, vec!["load", "save"]);
    }
}

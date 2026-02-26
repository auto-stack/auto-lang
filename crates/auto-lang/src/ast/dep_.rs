// DepStmt: Dependency declaration statement
//
// Plan 092: Rust FFI - `dep` keyword for declaring Rust crate dependencies

use crate::ast::{AtomWriter, ToAtom, ToAtomStr, ToNode};
use auto_val::AutoStr;
use auto_val::{Array, Node as AutoNode, Value};
use std::{fmt, io as stdio};

/// Dependency declaration statement
///
/// Syntax:
/// ```auto
/// dep serde                          // Latest version
/// dep serde(version: "1.0")          // Specific version
/// dep serde(version: "1.0", features: ["derive"])  // With features
/// dep my_lib(path: "../my_lib")      // Local crate
/// dep tokio(git: "https://...", branch: "main")  // Git source
/// ```
#[derive(Debug, Clone)]
pub struct DepStmt {
    /// Crate name (e.g., "serde", "serde_json")
    pub name: AutoStr,

    /// Version specification (optional)
    /// If None, use latest compatible version
    pub version: Option<AutoStr>,

    /// Feature flags to enable
    pub features: Vec<AutoStr>,

    /// Local path for local crates
    pub path: Option<AutoStr>,

    /// Git repository URL
    pub git: Option<AutoStr>,

    /// Git branch/tag/commit
    pub git_ref: Option<AutoStr>,
}

impl DepStmt {
    pub fn new(name: AutoStr) -> Self {
        Self {
            name,
            version: None,
            features: Vec::new(),
            path: None,
            git: None,
            git_ref: None,
        }
    }

    /// Check if this is a local dependency
    pub fn is_local(&self) -> bool {
        self.path.is_some()
    }

    /// Check if this is a git dependency
    pub fn is_git(&self) -> bool {
        self.git.is_some()
    }

    /// Check if this is a crates.io dependency
    pub fn is_crates_io(&self) -> bool {
        !self.is_local() && !self.is_git()
    }
}

impl fmt::Display for DepStmt {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(dep {}", self.name)?;

        if let Some(ref v) = self.version {
            write!(f, " (version {})", v)?;
        }

        if !self.features.is_empty() {
            write!(f, " (features [{}])", self.features.join(", "))?;
        }

        if let Some(ref p) = self.path {
            write!(f, " (path {})", p)?;
        }

        if let Some(ref g) = self.git {
            write!(f, " (git {})", g)?;
        }

        write!(f, ")")
    }
}

impl ToNode for DepStmt {
    fn to_node(&self) -> AutoNode {
        let mut node = AutoNode::new("dep");

        node.set_prop("name", Value::str(self.name.as_str()));

        if let Some(ref v) = self.version {
            node.set_prop("version", Value::str(v.as_str()));
        }

        if !self.features.is_empty() {
            let features: Vec<Value> = self
                .features
                .iter()
                .map(|s| Value::str(s.as_str()))
                .collect();
            node.set_prop("features", Value::array(Array::from_vec(features)));
        }

        if let Some(ref p) = self.path {
            node.set_prop("path", Value::str(p.as_str()));
        }

        if let Some(ref g) = self.git {
            node.set_prop("git", Value::str(g.as_str()));
        }

        if let Some(ref r) = self.git_ref {
            node.set_prop("git_ref", Value::str(r.as_str()));
        }

        node
    }
}

impl AtomWriter for DepStmt {
    fn write_atom(&self, f: &mut impl stdio::Write) -> auto_val::AutoResult<()> {
        write!(f, "dep({}", self.name)?;

        if let Some(ref v) = self.version {
            write!(f, ", version(\"{}\")", v)?;
        }

        if !self.features.is_empty() {
            write!(f, ", features([{}])", self.features.join(", "))?;
        }

        if let Some(ref p) = self.path {
            write!(f, ", path(\"{}\")", p)?;
        }

        if let Some(ref g) = self.git {
            write!(f, ", git(\"{}\")", g)?;
        }

        write!(f, ")")?;
        Ok(())
    }
}

impl ToAtom for DepStmt {
    fn to_atom(&self) -> AutoStr {
        self.to_atom_str()
    }
}

use crate::ast::module_path::ModulePath;
use crate::ast::AtomWriter;
use auto_val::AutoStr;
use std::{fmt, io as stdio};

#[derive(Debug, Clone)]
pub enum UseKind {
    Auto,
    C,
    Rust,
}

#[derive(Debug, Clone)]
pub struct Use {
    pub kind: UseKind,
    /// Plan 131: Structured module path (new syntax)
    pub module_path: Option<ModulePath>,
    /// Legacy: dotted path segments (for backward compat)
    pub paths: Vec<AutoStr>,
    /// Symbols to import (after `:`)
    pub items: Vec<AutoStr>,
}

impl fmt::Display for Use {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(use")?;
        match self.kind {
            UseKind::C => write!(f, " (kind c)")?,
            UseKind::Rust => write!(f, " (kind rust)")?,
            _ => (),
        }
        // Plan 131: Display module_path if present
        if let Some(ref mp) = self.module_path {
            write!(f, " (module_path {})", mp.display())?;
        } else if !self.paths.is_empty() {
            write!(f, " (path {})", self.paths.join("."))?;
        }
        if !self.items.is_empty() {
            write!(f, " (items {})", self.items.join(","))?;
        }
        write!(f, ")")
    }
}

// ToAtom and ToNode implementations

use crate::ast::{ToAtom, ToAtomStr, ToNode};
use auto_val::{Arg as AutoValArg, Array, Node as AutoNode, Value};

impl ToNode for Use {
    fn to_node(&self) -> AutoNode {
        let mut node = AutoNode::new("use");

        // Only set kind property if not Auto (default)
        match self.kind {
            UseKind::C => node.set_prop("kind", Value::str("c")),
            UseKind::Rust => node.set_prop("kind", Value::str("rust")),
            UseKind::Auto => {} // Default, omit
        }

        // Plan 131: Include module_path if present
        if let Some(ref mp) = self.module_path {
            node.set_prop("module_path", Value::str(mp.display().as_str()));
        } else if !self.paths.is_empty() {
            let path_str = self.paths.join(".");
            node.set_prop("path", Value::str(path_str.as_str()));
        }

        if !self.items.is_empty() {
            let items: Vec<Value> = self.items.iter().map(|s| Value::str(s.as_str())).collect();
            node.add_arg(AutoValArg::Pos(Value::array(Array::from_vec(items))));
        }

        node
    }
}

impl AtomWriter for Use {
    fn write_atom(&self, f: &mut impl stdio::Write) -> auto_val::AutoResult<()> {
        write!(f, "use(")?;
        match self.kind {
            UseKind::C => write!(f, "kind(\"c\"), ")?,
            UseKind::Rust => write!(f, "kind(\"rust\"), ")?,
            UseKind::Auto => {}
        }
        // Plan 131: Include module_path if present
        if let Some(ref mp) = self.module_path {
            write!(f, "module_path(\"{}\")", mp.display())?;
        } else if !self.paths.is_empty() {
            write!(f, "path(\"{}\")", self.paths.join("."))?;
        }
        if !self.items.is_empty() {
            if self.module_path.is_some() || !self.paths.is_empty() {
                write!(f, ", ")?;
            }
            write!(f, "items([{}])", self.items.join(", "))?;
        }
        write!(f, ")")?;
        Ok(())
    }
}

impl ToAtom for Use {
    fn to_atom(&self) -> AutoStr {
        self.to_atom_str()
    }
}

#[cfg(test)]
mod plan131_tests {
    use super::*;

    #[test]
    fn test_use_with_pac_prefix() {
        let use_stmt = Use {
            kind: UseKind::Auto,
            module_path: Some(ModulePath::pac(vec!["db".into()])),
            paths: vec![],
            items: vec![],
        };
        assert_eq!(
            use_stmt.module_path.as_ref().unwrap().display(),
            "pac.db"
        );
        // Test Display trait
        assert_eq!(format!("{}", use_stmt), "(use (module_path pac.db))");
    }

    #[test]
    fn test_use_with_super_prefix() {
        let use_stmt = Use {
            kind: UseKind::Auto,
            module_path: Some(ModulePath::super_path(vec!["utils".into()])),
            paths: vec![],
            items: vec![],
        };
        assert_eq!(
            use_stmt.module_path.as_ref().unwrap().display(),
            "super.utils"
        );
        // Test Display trait
        assert_eq!(format!("{}", use_stmt), "(use (module_path super.utils))");
    }

    #[test]
    fn test_use_with_items() {
        let use_stmt = Use {
            kind: UseKind::Auto,
            module_path: Some(
                ModulePath::pac(vec!["io".into()]).with_items(vec!["say".into(), "ask".into()]),
            ),
            paths: vec![],
            items: vec!["say".into(), "ask".into()],
        };
        assert_eq!(use_stmt.items, vec!["say", "ask"]);
        // Test Display trait
        assert_eq!(
            format!("{}", use_stmt),
            "(use (module_path pac.io) (items say,ask))"
        );
    }

    #[test]
    fn test_use_legacy_paths_backward_compat() {
        // Legacy paths still work
        let use_stmt = Use {
            kind: UseKind::Auto,
            module_path: None,
            paths: vec!["std".into(), "io".into()],
            items: vec!["say".into()],
        };
        assert!(use_stmt.module_path.is_none());
        assert_eq!(format!("{}", use_stmt), "(use (path std.io) (items say))");
    }

    #[test]
    fn test_use_to_node_with_module_path() {
        let use_stmt = Use {
            kind: UseKind::Auto,
            module_path: Some(ModulePath::pac(vec!["db".into()])),
            paths: vec![],
            items: vec![],
        };
        let node = use_stmt.to_node();
        assert_eq!(node.name.as_str(), "use");
        let module_path_val = node.get_prop("module_path");
        assert!(matches!(module_path_val, Value::Str(_)));
        if let Value::Str(s) = module_path_val {
            assert_eq!(s.as_str(), "pac.db");
        }
    }

    #[test]
    fn test_use_write_atom_with_module_path() {
        let use_stmt = Use {
            kind: UseKind::Auto,
            module_path: Some(ModulePath::pac(vec!["db".into()])),
            paths: vec![],
            items: vec!["load".into(), "save".into()],
        };
        let mut output = Vec::new();
        use_stmt.write_atom(&mut output).unwrap();
        let result = String::from_utf8(output).unwrap();
        assert_eq!(result, "use(module_path(\"pac.db\"), items([load, save]))");
    }
}

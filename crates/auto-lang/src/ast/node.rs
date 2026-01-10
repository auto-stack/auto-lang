use super::*;
use auto_val::{shared, Shared};

#[derive(Debug, Clone)]
pub struct Node {
    pub name: Name,
    pub id: Name,
    pub num_args: usize, // NEW: Number of args (for unified API)
    pub args: Args,      // TODO: Will be removed in Phase 5
    // pub props: BTreeMap<Key, Expr>,
    pub body: Body,
    pub typ: Shared<Type>,
}

impl Node {
    pub fn new(name: impl Into<Name>) -> Self {
        Self {
            name: name.into(),
            id: Name::new(),
            num_args: 0,
            args: Args::new(),
            body: Body::new(),
            typ: shared(Type::Unknown),
        }
    }

    // ========== UNIFIED API: Args (Phase 2 migration) ==========
    // These methods parallel the auto_val::Node API for consistency
    // During migration, args are stored in BOTH args (old) and tracked by num_args (new)

    /// Add an arg and increment num_args counter
    /// NOTE: For AST nodes, we don't have a unified props store yet, so we just track
    /// the count and populate the old args field. The full unification happens later
    /// when ast::Node is converted to auto_val::Node.
    pub fn add_arg_unified(&mut self, key: impl Into<Name>, value: Expr) {
        let key = key.into();
        self.args.args.push(crate::ast::call::Arg::Pair(key, value));
        self.num_args += 1;
    }

    /// Add a positional arg (empty key)
    pub fn add_pos_arg_unified(&mut self, value: Expr) {
        self.args.args.push(crate::ast::call::Arg::Pos(value));
        self.num_args += 1;
    }

    /// Add a named arg
    pub fn add_name_arg_unified(&mut self, name: Name) {
        self.args.args.push(crate::ast::call::Arg::Name(name));
        self.num_args += 1;
    }
}

impl From<Call> for Node {
    fn from(call: Call) -> Self {
        let name = call.get_name_text();
        let mut node = Node::new(name);
        node.args = call.args;
        node
    }
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(node")?;
        write!(f, " (name {})", self.name)?;
        if !self.id.is_empty() {
            write!(f, " (id {})", self.id)?;
        }
        if !self.args.is_empty() {
            write!(f, " {}", self.args)?;
        }

        if !self.body.stmts.is_empty() {
            write!(f, " {}", self.body)?;
        }

        write!(f, ")")
    }
}

// ToAtom and ToNode implementations

use crate::ast::{AtomWriter, ToAtom, ToAtomStr, ToNode};
use auto_val::{Node as AutoNode, Value};
use std::{fmt, io as stdio};

impl AtomWriter for Node {
    fn write_atom(&self, f: &mut impl stdio::Write) -> auto_val::AutoResult<()> {
        write!(f, "node {}", self.name)?;
        if !self.args.args.is_empty() {
            // Constructor call format: node name (arg1, arg2)
            write!(f, " (")?;
            for (i, arg) in self.args.args.iter().enumerate() {
                match arg {
                    crate::ast::call::Arg::Pos(expr) => write!(f, "{}", expr.to_atom_str())?,
                    crate::ast::call::Arg::Name(name) => write!(f, "{}", name)?,
                    crate::ast::call::Arg::Pair(key, value) => {
                        write!(f, "{}: {}", key, value.to_atom_str())?
                    }
                }
                if i < self.args.args.len() - 1 {
                    write!(f, ", ")?;
                }
            }
            write!(f, ")")?;
        }
        if !self.id.is_empty() {
            write!(f, " id(\"{}\")", self.id)?;
        }
        // Only add braces if there's a body with statements or additional props
        if !self.body.stmts.is_empty()
            || self
                .args
                .args
                .iter()
                .any(|a| matches!(a, crate::ast::call::Arg::Pair(_, _)))
        {
            write!(f, " {{")?;
            if !self.body.stmts.is_empty() {
                write!(f, " {}", self.body.to_atom_str())?;
            }
            write!(f, " }}")?;
        }
        Ok(())
    }
}

impl ToNode for Node {
    fn to_node(&self) -> AutoNode {
        let mut node = AutoNode::new("node");
        node.set_prop("name", Value::str(self.name.as_str()));

        if !self.id.is_empty() {
            node.set_prop("id", Value::str(self.id.as_str()));
        }

        if !self.args.is_empty() {
            node.add_kid(self.args.to_node());
        }

        if !self.body.stmts.is_empty() {
            node.add_kid(self.body.to_node());
        }

        node
    }
}

impl ToAtom for Node {
    fn to_atom(&self) -> AutoStr {
        self.to_atom_str()
    }
}

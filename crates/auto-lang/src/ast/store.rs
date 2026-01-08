use super::{Expr, Name, Type};
use crate::ast::{AtomWriter, ToAtomStr};
use std::{fmt, io as stdio};

#[derive(Debug, Clone)]
pub enum StoreKind {
    Let,
    Mut,
    Var,
    CVar,  // C variable declaration
    Field, // field of struct
}

#[derive(Debug, Clone)]
pub struct Store {
    pub kind: StoreKind,
    pub name: Name,
    pub ty: Type,
    pub expr: Expr,
}

impl fmt::Display for Store {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let ty_str = if matches!(self.ty, Type::Unknown) {
            " ".to_string()
        } else {
            format!(" (type {}) ", self.ty)
        };
        match self.kind {
            StoreKind::Let => write!(f, "(let (name {}){}{})", self.name, ty_str, self.expr),
            StoreKind::Mut => write!(f, "(mut (name {}){}{})", self.name, ty_str, self.expr),
            StoreKind::Var => write!(f, "(var (name {}) {})", self.name, self.expr),
            StoreKind::Field => write!(f, "(field (name {}) {})", self.name, self.expr),
            StoreKind::CVar => write!(f, "(cvar (name {}))", self.name),
        }
    }
}

impl fmt::Display for StoreKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            StoreKind::Let => write!(f, "let"),
            StoreKind::Mut => write!(f, "mut"),
            StoreKind::Var => write!(f, "var"),
            StoreKind::Field => write!(f, "field"),
            StoreKind::CVar => write!(f, "cvar"),
        }
    }
}

// ToAtom and ToNode implementations

use crate::ast::{ToAtom, ToNode};
use auto_val::{AutoStr, Node as AutoNode, Value};

impl AtomWriter for Store {
    fn write_atom(&self, f: &mut impl stdio::Write) -> auto_val::AutoResult<()> {
        let kind_name = match self.kind {
            StoreKind::Let => "let",
            StoreKind::Mut => "mut",
            StoreKind::Var => "var",
            StoreKind::Field => "field",
            StoreKind::CVar => "cvar",
        };

        write!(
            f,
            "{}(name(\"{}\"), type({}), expr({}))",
            kind_name,
            self.name,
            self.ty.to_atom_str(),
            self.expr.to_atom_str()
        )?;
        Ok(())
    }
}

impl ToNode for Store {
    fn to_node(&self) -> AutoNode {
        let node_name = match &self.kind {
            StoreKind::Let => "let",
            StoreKind::Mut => "mut",
            StoreKind::Var => "var",
            StoreKind::CVar => "cvar",
            StoreKind::Field => "field",
        };

        let mut node = AutoNode::new(node_name);
        node.set_prop("name", Value::str(self.name.as_str()));

        if !matches!(self.ty, Type::Unknown) {
            node.set_prop("type", Value::str(&*self.ty.to_atom()));
        }

        node.add_kid(self.expr.to_node()); // Changed from expr.to_atom().to_node()
        node
    }
}

impl ToAtom for Store {
    fn to_atom(&self) -> AutoStr {
        self.to_atom_str()
    }
}

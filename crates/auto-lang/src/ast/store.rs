use super::{Expr, Name, Type};
use std::fmt;

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

// ToAtom implementation

use crate::ast::ToAtom;
use auto_val::{Node, Value};

impl ToAtom for Store {
    fn to_atom(&self) -> Value {
        let node_name = match &self.kind {
            StoreKind::Let => "let",
            StoreKind::Mut => "mut",
            StoreKind::Var => "var",
            StoreKind::CVar => "cvar",
            StoreKind::Field => "field",
        };

        let mut node = Node::new(node_name);
        node.set_prop("name", Value::str(self.name.as_str()));

        if !matches!(self.ty, Type::Unknown) {
            node.set_prop("type", self.ty.to_atom());
        }

        node.add_kid(self.expr.to_atom().to_node());
        Value::Node(node)
    }
}

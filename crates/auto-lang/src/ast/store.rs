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

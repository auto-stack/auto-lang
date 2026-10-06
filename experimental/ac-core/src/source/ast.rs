//! Source-subset AST for the `auto.source.core-i32.draft` profile
//! (PLAN-745 T-02).
//!
//! Every node carries a byte [`Span`] into the original `.at` source.
//! This is a statement-oriented surface: expressions only appear where a
//! value is consumed (initializers, assignments, returns, conditions, call
//! arguments); there are no expression statements.

use crate::atom_text::Span;

#[derive(Clone, Debug)]
pub struct Ident {
    pub name: String,
    pub span: Span,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TypeWord {
    Int,
    I32,
    Bool,
}

/// A type annotation. `int` and `i32` are synonyms inside this profile
/// (both are the exact signed 32-bit type); `bool` is the boolean type.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct TypeAnn {
    pub word: TypeWord,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct Param {
    pub name: Ident,
    pub ty: TypeAnn,
    /// Name token through the type token: the span used by duplicate-param
    /// diagnostics.
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct FnItem {
    pub name: Ident,
    pub params: Vec<Param>,
    pub ret: TypeAnn,
    pub body: Block,
    /// `fn` keyword through the closing brace of the body.
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct Program {
    pub items: Vec<FnItem>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BinOp {
    Add,
    Mul,
    Lt,
}

#[derive(Clone, Debug)]
pub enum Arg {
    Pos(Expr),
    Named {
        name: Ident,
        value: Expr,
        span: Span,
    },
}

#[derive(Clone, Debug)]
pub enum ExprKind {
    /// Range-checked signed 32-bit literal (`-2147483648` and
    /// `- 2147483648` are one literal; see the parser).
    IntLit(i32),
    BoolLit(bool),
    Var(Ident),
    Call {
        callee: Ident,
        args: Vec<Arg>,
    },
    Binary {
        op: BinOp,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
}

#[derive(Clone, Debug)]
pub struct Expr {
    pub kind: ExprKind,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub enum Stmt {
    Let {
        kw: Span,
        name: Ident,
        ann: Option<TypeAnn>,
        mutable: bool,
        value: Expr,
        /// Keyword through the end of the initializer.
        span: Span,
    },
    Assign {
        name: Ident,
        value: Expr,
        /// Name through the end of the value.
        span: Span,
    },
    Return {
        value: Expr,
        /// `return` through the end of the value.
        span: Span,
    },
    If {
        cond: Expr,
        then: Block,
        els: Option<Block>,
        span: Span,
    },
    While {
        cond: Expr,
        body: Block,
        span: Span,
    },
    Loop {
        body: Block,
        span: Span,
    },
    Break {
        span: Span,
    },
    Continue {
        span: Span,
    },
}

#[derive(Clone, Debug)]
pub struct Block {
    pub stmts: Vec<Stmt>,
    /// Opening brace through closing brace.
    pub span: Span,
}

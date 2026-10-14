//! Source-subset AST for the `auto.source.core-i32.draft` and
//! `auto.source.scalar-i32.draft` profiles (PLAN-745 T-02, PLAN-767 T-03).
//!
//! Every node carries a byte [`Span`] into the original `.at` source.
//! This is a statement-oriented surface: expressions only appear where a
//! value is consumed (initializers, assignments, returns, conditions, call
//! arguments); there are no expression statements.
//!
//! The operator set is the union of both profiles; which operators a given
//! parse accepts is decided by the parser's profile (the core surface
//! rejects every scalar-only operator and node at the offending token).
//! `&&`/`||` parse into [`BinOp::And`]/[`BinOp::Or`] surface nodes — the
//! eager/lazy distinction materializes in the HIR adapter
//! (`ExprKind::Logical`), never by making the RHS an unconditional operand
//! of an eager binary op.

use crate::atom_text::Span;

#[derive(Clone, PartialEq, Eq, Debug)]
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
/// The record profile (PLAN-771) adds nominal record types by name —
/// builtin-vs-record is decided syntactically here, existence and closure
/// in resolve/typecheck.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct TypeAnn {
    pub kind: TypeRef,
    pub span: Span,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum TypeRef {
    Builtin(TypeWord),
    /// Nominal record type by declared name.
    Record(Ident),
}

/// Parameter passing mode (PLAN-771 §5.3, 774 D-04). `View` is the
/// default (and the only mode for i32/bool params); `Mut` is the explicit
/// exclusive borrow for record params, spelled `mut name Type` (the real
/// parser's mode-prefix order).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ParamMode {
    View,
    Mut,
}

#[derive(Clone, Debug)]
pub struct Param {
    pub name: Ident,
    pub ty: TypeAnn,
    /// Passing mode; `View` unless `mut` was written (record profile).
    pub mode: ParamMode,
    /// Name token through the type token: the span used by duplicate-param
    /// diagnostics.
    pub span: Span,
}

/// Method receiver shape inside a `type` block (PLAN-771). `Free` marks
/// top-level functions.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Receiver {
    Free,
    Static,
    /// Readonly instance method: implicit `self` view param.
    View,
}

#[derive(Clone, Debug)]
pub struct FnItem {
    pub name: Ident,
    pub params: Vec<Param>,
    pub ret: TypeAnn,
    pub body: Block,
    /// `fn` keyword through the closing brace of the body.
    pub span: Span,
    /// Receiver (record profile): free function / static method /
    /// readonly instance method.
    pub receiver: Receiver,
    /// Owning record name for methods.
    pub owner: Option<Ident>,
}

/// One record field, in declaration order (= layout order, 774 D-05).
#[derive(Clone, Debug)]
pub struct FieldDecl {
    pub name: Ident,
    pub ty: TypeAnn,
    /// Field name token through the type token.
    pub span: Span,
}

/// A `type Name { fields; methods }` item (record profile only).
#[derive(Clone, Debug)]
pub struct TypeItem {
    pub name: Ident,
    pub fields: Vec<FieldDecl>,
    pub methods: Vec<FnItem>,
    /// `type` keyword through the closing brace.
    pub span: Span,
}

#[derive(Clone, Debug)]
pub enum Item {
    Fn(FnItem),
    Type(TypeItem),
}

#[derive(Clone, Debug)]
pub struct Program {
    pub items: Vec<Item>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Lt,
    Le,
    Gt,
    Ge,
    Eq,
    Ne,
    And,
    Or,
}

impl BinOp {
    pub fn as_str(&self) -> &'static str {
        match self {
            BinOp::Add => "+",
            BinOp::Sub => "-",
            BinOp::Mul => "*",
            BinOp::Div => "/",
            BinOp::Rem => "%",
            BinOp::Lt => "<",
            BinOp::Le => "<=",
            BinOp::Gt => ">",
            BinOp::Ge => ">=",
            BinOp::Eq => "==",
            BinOp::Ne => "!=",
            BinOp::And => "&&",
            BinOp::Or => "||",
        }
    }
}

/// Unary operators (PLAN-767): `-` on i32, `!` on bool.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum UnaryOp {
    Neg,
    Not,
}

impl UnaryOp {
    pub fn as_str(&self) -> &'static str {
        match self {
            UnaryOp::Neg => "-",
            UnaryOp::Not => "!",
        }
    }
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
    Unary {
        op: UnaryOp,
        operand: Box<Expr>,
    },
    // -- record profile (PLAN-771) --
    /// Positional construction `Pair(1, true)`; `Pair` must resolve to a
    /// record type (resolve/typecheck decide — the parser keeps the
    /// call-looking form).
    CtorPos {
        ty: Ident,
        args: Vec<Arg>,
    },
    /// Named construction `Pair { left: 1, right: true }` — arguments
    /// evaluate in SOURCE order, then map to declaration fields.
    CtorNamed {
        ty: Ident,
        fields: Vec<NamedCtorField>,
    },
    /// Field read `base.name` (rvalue; a copy for record fields).
    Field {
        base: Box<Expr>,
        name: Ident,
    },
    /// `recv.name(args)` — instance method when `recv` is a value, static
    /// method when `recv` is a bare record type name (resolve decides).
    MethodCall {
        recv: Box<Expr>,
        name: Ident,
        args: Vec<Arg>,
    },
    /// `p.mut` — the explicit mut-borrow argument form (record profile).
    MutArg {
        name: Ident,
    },
}

/// One `name: value` entry of a named constructor.
#[derive(Clone, Debug)]
pub struct NamedCtorField {
    pub name: Ident,
    pub value: Expr,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct Expr {
    pub kind: ExprKind,
    pub span: Span,
}

/// Assignment target: a local, or a field path rooted at a local
/// (`p.inner.left = e` — PLAN-771).
#[derive(Clone, Debug)]
pub struct AssignTarget {
    pub base: Ident,
    pub path: Vec<Ident>,
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
        target: AssignTarget,
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

/// Operator chains parse iteratively, so a legal program may hold an
/// arbitrarily deep `Binary` spine; the derived recursive drop would
/// overflow on it (R3-QA-02). Drop the tree with an explicit work stack:
/// children are moved out of their boxes onto the stack (a cheap dummy
/// stays behind), so every actual drop is shallow.
impl Drop for Expr {
    fn drop(&mut self) {
        fn dummy() -> Expr {
            Expr {
                kind: ExprKind::IntLit(0),
                span: Span { start: 0, end: 0 },
            }
        }
        let detach = |node: &mut Expr, stack: &mut Vec<Expr>| {
            match &mut node.kind {
                ExprKind::Binary { lhs, rhs, .. } => {
                    let l = std::mem::replace(lhs.as_mut(), dummy());
                    let r = std::mem::replace(rhs.as_mut(), dummy());
                    stack.push(l);
                    stack.push(r);
                }
                ExprKind::Unary { operand, .. } => {
                    let o = std::mem::replace(operand.as_mut(), dummy());
                    stack.push(o);
                }
                // nested calls carry inline argument expressions; drain
                // them so no level drops recursively
                ExprKind::Call { args, .. } | ExprKind::CtorPos { args, .. } => {
                    for a in args.iter_mut() {
                        let e = match a {
                            Arg::Pos(e) => std::mem::replace(e, dummy()),
                            Arg::Named { value, .. } => std::mem::replace(value, dummy()),
                        };
                        stack.push(e);
                    }
                }
                ExprKind::MethodCall { args, recv, .. } => {
                    let r = std::mem::replace(recv.as_mut(), dummy());
                    stack.push(r);
                    for a in args.iter_mut() {
                        let e = match a {
                            Arg::Pos(e) => std::mem::replace(e, dummy()),
                            Arg::Named { value, .. } => std::mem::replace(value, dummy()),
                        };
                        stack.push(e);
                    }
                }
                ExprKind::Field { base, .. } => {
                    let b = std::mem::replace(base.as_mut(), dummy());
                    stack.push(b);
                }
                ExprKind::CtorNamed { fields, .. } => {
                    for f in fields.iter_mut() {
                        let e = std::mem::replace(&mut f.value, dummy());
                        stack.push(e);
                    }
                }
                _ => {}
            }
        };
        let mut stack: Vec<Expr> = Vec::new();
        detach(self, &mut stack);
        while let Some(mut node) = stack.pop() {
            detach(&mut node, &mut stack);
            // node's boxes hold dummies now; its drop is shallow
        }
    }
}

/// Statement blocks nest arbitrarily deep (R6: unlimited block nesting), so
/// the derived recursive drop of a nested block tree would overflow — drop
/// with an explicit work stack (same pattern as `Expr`): every nested block
/// is moved out onto the stack (a cheap empty dummy stays behind), so each
/// actual drop is shallow.
impl Drop for Block {
    fn drop(&mut self) {
        fn dummy() -> Block {
            Block {
                stmts: Vec::new(),
                span: Span { start: 0, end: 0 },
            }
        }
        let detach = |b: &mut Block, stack: &mut Vec<Block>| {
            for s in b.stmts.iter_mut() {
                match s {
                    Stmt::If { then, els, .. } => {
                        stack.push(std::mem::replace(then, dummy()));
                        if let Some(e) = els {
                            stack.push(std::mem::replace(e, dummy()));
                        }
                    }
                    Stmt::While { body, .. } | Stmt::Loop { body, .. } => {
                        stack.push(std::mem::replace(body, dummy()));
                    }
                    _ => {}
                }
            }
        };
        let mut stack: Vec<Block> = Vec::new();
        detach(self, &mut stack);
        while let Some(mut b) = stack.pop() {
            detach(&mut b, &mut stack);
        }
    }
}

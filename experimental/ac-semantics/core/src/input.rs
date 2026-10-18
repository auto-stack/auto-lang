//! Neutral semantic input model (PLAN-796 §5.2).
//!
//! The inference profile reasons over THIS model, not over any frontend
//! crate's AST — the (feature-gated) adapter projects the existing
//! ac-core public AST into it. The model carries the limited profile
//! surface: i32/bool literals, locals with let/var + shadowing, explicit
//! function params/returns, direct calls, the operator set, condition/
//! loop constraints and assignment consistency.

/// Byte span into the original source (start inclusive, end exclusive).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Span {
    pub start: u32,
    pub end: u32,
}

impl Span {
    pub fn new(start: u32, end: u32) -> Self {
        Span { start, end }
    }
}

/// A type annotation in the input. `Inferred` marks an un-annotated
/// binding — the one place the limited profile actually solves.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Ann {
    I32,
    Bool,
    /// Nominal record reference by declared name (resolved against the
    /// module's records).
    Record(&'static str),
    /// No annotation: the binding's type is the initializer's type.
    Inferred,
    /// An annotation outside the limited i32/bool profile (text/char
    /// surfaces) — surfaces as unsupported-construct, never widened.
    Unsupported(&'static str),
}

/// One function of the input module.
#[derive(Clone, Debug)]
pub struct InputFn {
    pub name: String,
    /// (name, annotation) in declaration order.
    pub params: Vec<(String, Ann)>,
    pub ret: Ann,
    pub body: InputBlock,
}

/// One record declaration (fields in declaration order; methods kept
/// separately).
#[derive(Clone, Debug)]
pub struct InputRecord {
    pub name: String,
    pub fields: Vec<(String, Ann)>,
    pub methods: Vec<InputFn>,
}

/// A whole input module.
#[derive(Clone, Debug, Default)]
pub struct SemaInput {
    pub package: String,
    pub module: String,
    pub fns: Vec<InputFn>,
    pub records: Vec<InputRecord>,
    /// Exact-source fingerprint at projection time (F-796-R4-05):
    /// binds position-bearing facts to the source bytes they were
    /// derived from. Semantic CONTENT keys stay span-free (position-only
    /// edits still reuse the value); serving CURRENT positions requires
    /// the fingerprint to match, else a projection refresh.
    pub source_fp: String,
}

impl SemaInput {
    pub fn lookup_fn(&self, name: &str) -> Option<&InputFn> {
        self.fns.iter().find(|f| f.name == name)
    }
    pub fn lookup_record(&self, name: &str) -> Option<&InputRecord> {
        self.records.iter().find(|r| r.name == name)
    }
    /// The function-domain candidate set (sorted: the negative-dependency
    /// fingerprint input).
    pub fn fn_candidates(&self) -> Vec<String> {
        let mut v: Vec<String> = self.fns.iter().map(|f| f.name.clone()).collect();
        v.sort();
        v
    }
    /// The type-domain candidate set.
    pub fn record_candidates(&self) -> Vec<String> {
        let mut v: Vec<String> = self.records.iter().map(|r| r.name.clone()).collect();
        v.sort();
        v
    }
}

#[derive(Clone, Debug)]
pub struct InputBlock {
    pub stmts: Vec<InputStmt>,
}

#[derive(Clone, Debug)]
pub enum InputStmt {
    Let {
        name: String,
        ann: Ann,
        mutable: bool,
        value: InputExpr,
        span: Span,
    },
    Assign {
        target: String,
        /// Field path after the base local (`p.inner.left = e`); a
        /// non-empty path is outside the limited profile.
        path: Vec<String>,
        value: InputExpr,
        span: Span,
    },
    Return {
        value: InputExpr,
        span: Span,
    },
    If {
        cond: InputExpr,
        then: InputBlock,
        els: Option<InputBlock>,
        span: Span,
    },
    While {
        cond: InputExpr,
        body: InputBlock,
        span: Span,
    },
    Loop {
        body: InputBlock,
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
pub struct InputExpr {
    pub kind: InputExprKind,
    pub span: Span,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BinOpKind {
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

impl BinOpKind {
    pub fn as_str(self) -> &'static str {
        match self {
            BinOpKind::Add => "+",
            BinOpKind::Sub => "-",
            BinOpKind::Mul => "*",
            BinOpKind::Div => "/",
            BinOpKind::Rem => "%",
            BinOpKind::Lt => "<",
            BinOpKind::Le => "<=",
            BinOpKind::Gt => ">",
            BinOpKind::Ge => ">=",
            BinOpKind::Eq => "==",
            BinOpKind::Ne => "!=",
            BinOpKind::And => "&&",
            BinOpKind::Or => "||",
        }
    }
}

#[derive(Clone, Debug)]
pub enum InputExprKind {
    IntLit(i32),
    BoolLit(bool),
    Var(String),
    Call { callee: String, args: Vec<InputExpr> },
    Binary { op: BinOpKind, lhs: Box<InputExpr>, rhs: Box<InputExpr> },
    Unary { neg: bool, operand: Box<InputExpr> },
    /// A construction outside the limited surface: reported as
    /// unsupported-construct, never silently typed.
    Unsupported { what: &'static str },
}

impl InputExpr {
    pub fn int(v: i32) -> Self {
        InputExpr { kind: InputExprKind::IntLit(v), span: Span::new(0, 0) }
    }
    pub fn boolean(v: bool) -> Self {
        InputExpr { kind: InputExprKind::BoolLit(v), span: Span::new(0, 0) }
    }
    pub fn var(name: &str) -> Self {
        InputExpr { kind: InputExprKind::Var(name.into()), span: Span::new(0, 0) }
    }
}

/// Legal operator chains and call nestings are unbounded in depth, so
/// the derived recursive drop would overflow on a long spine (same
/// discipline as the ac-core AST): drop with an explicit work stack,
/// replacing children with cheap dummies so every actual drop is
/// shallow.
impl Drop for InputExpr {
    fn drop(&mut self) {
        fn dummy() -> InputExpr {
            InputExpr { kind: InputExprKind::IntLit(0), span: Span::new(0, 0) }
        }
        let detach = |node: &mut InputExpr, stack: &mut Vec<InputExpr>| match &mut node.kind {
            InputExprKind::Binary { lhs, rhs, .. } => {
                let l = std::mem::replace(lhs.as_mut(), dummy());
                let r = std::mem::replace(rhs.as_mut(), dummy());
                stack.push(l);
                stack.push(r);
            }
            InputExprKind::Unary { operand, .. } => {
                let o = std::mem::replace(operand.as_mut(), dummy());
                stack.push(o);
            }
            InputExprKind::Call { args, .. } => {
                for a in args.iter_mut() {
                    stack.push(std::mem::replace(a, dummy()));
                }
            }
            _ => {}
        };
        let mut stack: Vec<InputExpr> = Vec::new();
        detach(self, &mut stack);
        while let Some(mut node) = stack.pop() {
            detach(&mut node, &mut stack);
        }
    }
}

/// Nested statement blocks nest arbitrarily deep — same iterative-drop
/// discipline as `InputExpr`.
impl Drop for InputBlock {
    fn drop(&mut self) {
        fn dummy() -> InputBlock {
            InputBlock { stmts: Vec::new() }
        }
        let detach = |b: &mut InputBlock, stack: &mut Vec<InputBlock>| {
            for s in b.stmts.iter_mut() {
                match s {
                    InputStmt::If { then, els, .. } => {
                        stack.push(std::mem::replace(then, dummy()));
                        if let Some(e) = els {
                            stack.push(std::mem::replace(e, dummy()));
                        }
                    }
                    InputStmt::While { body, .. } | InputStmt::Loop { body, .. } => {
                        stack.push(std::mem::replace(body, dummy()));
                    }
                    _ => {}
                }
            }
        };
        let mut stack: Vec<InputBlock> = Vec::new();
        detach(self, &mut stack);
        while let Some(mut b) = stack.pop() {
            detach(&mut b, &mut stack);
        }
    }
}

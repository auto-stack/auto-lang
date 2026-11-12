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
    /// PLAN-963（T-02）：文本值（`str`/`Str`）——记录域的既有标量面类型，
    /// 作为泛型实参时与 i32/bool 同等参与求解。
    Text,
    /// No annotation: the binding's type is the initializer's type.
    Inferred,
    /// An annotation outside the limited i32/bool profile (text/char
    /// surfaces) — surfaces as unsupported-construct, never widened.
    Unsupported(&'static str),
    /// PLAN-935 (T-04): reference to the fn's i-th DECLARED type parameter.
    /// Only produced for template signatures (stubs) inside the instances
    /// discovery pass; the solver's call hook mints one shared var per
    /// type parameter per CALL SITE and substitutes before unification —
    /// plain `ann_ty` never sees this variant on a solved path.
    TyParam(u16),
}

/// One function of the input module.
#[derive(Clone, Debug)]
pub struct InputFn {
    pub name: String,
    /// (name, annotation) in declaration order.
    pub params: Vec<(String, Ann)>,
    pub ret: Ann,
    /// PLAN-935 (T-04): declared type parameter names (empty outside the
    /// instances template face). A fn carrying them is a TEMPLATE: the
    /// discovery pass admits its SIGNATURE only (body stays a stub — type
    /// errors of a template body are checked per concrete instance).
    pub type_params: Vec<String>,
    pub body: InputBlock,
}

/// PLAN-938: an external callable has a signature and nominal contract identity,
/// never a fabricated local body. It is selectable by call inference only.
#[derive(Clone, Debug)]
pub struct ExternalSignature {
    pub name: String,
    pub params: Vec<(String, Ann)>,
    pub ret: Ann,
    pub contract: String,
}

#[derive(Clone, Copy, Debug)]
pub struct SignatureView<'a> {
    pub name: &'a str,
    pub type_params: &'a [String],
    pub params: &'a [(String, Ann)],
    pub ret: Ann,
    pub external_contract: Option<&'a str>,
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
    pub external_signatures: Vec<ExternalSignature>,
    pub records: Vec<InputRecord>,
    /// Exact-source fingerprint at projection time (F-796-R4-05):
    /// binds position-bearing facts to the source bytes they were
    /// derived from. Semantic CONTENT keys stay span-free (position-only
    /// edits still reuse the value); serving CURRENT positions requires
    /// the fingerprint to match, else a projection refresh.
    pub source_fp: String,
}

impl SemaInput {
    pub fn signatures(&self) -> impl Iterator<Item = SignatureView<'_>> {
        self.fns
            .iter()
            .map(|f| SignatureView {
                name: &f.name,
                type_params: &f.type_params,
                params: &f.params,
                ret: f.ret,
                external_contract: None,
            })
            .chain(self.external_signatures.iter().map(|f| SignatureView {
                name: &f.name,
                type_params: &[],
                params: &f.params,
                ret: f.ret,
                external_contract: Some(&f.contract),
            }))
    }

    pub fn lookup_signature(&self, name: &str) -> Option<SignatureView<'_>> {
        self.signatures().find(|s| s.name == name)
    }
    pub fn lookup_fn(&self, name: &str) -> Option<&InputFn> {
        self.fns.iter().find(|f| f.name == name)
    }
    pub fn lookup_record(&self, name: &str) -> Option<&InputRecord> {
        self.records.iter().find(|r| r.name == name)
    }
    /// The function-domain candidate set (sorted: the negative-dependency
    /// fingerprint input).
    pub fn fn_candidates(&self) -> Vec<String> {
        let mut v: Vec<String> = self.signatures().map(|f| f.name.to_string()).collect();
        v.sort();
        v
    }
    /// The type-domain candidate set.
    pub fn record_candidates(&self) -> Vec<String> {
        let mut v: Vec<String> = self.records.iter().map(|r| r.name.clone()).collect();
        v.sort();
        v
    }
    /// PLAN-963（T-02）：字段名全域候选（负依赖指纹输入；跨记录去重排序）。
    pub fn field_candidates(&self) -> Vec<String> {
        let mut v: Vec<String> = Vec::new();
        for r in &self.records {
            for (n, _) in &r.fields {
                v.push(n.clone());
            }
        }
        v.sort();
        v.dedup();
        v
    }
    /// PLAN-963（T-02）：方法名全域候选（负依赖指纹输入）。
    pub fn method_candidates(&self) -> Vec<String> {
        let mut v: Vec<String> = Vec::new();
        for r in &self.records {
            for m in &r.methods {
                v.push(m.name.clone());
            }
        }
        v.sort();
        v.dedup();
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

/// One call argument (PLAN-934 T-04 named-argument input).
///
/// The vector order IS the source evaluation order; `name` carries the
/// parameter the argument binds to. Binding happens by NAME while the
/// operand is still evaluated in written order — `f(b: e1, a: e2)` reads
/// `e1` first and then binds it to `b`. A `None` name keeps the legacy
/// positional rule (argument i → parameter i).
#[derive(Clone, Debug)]
pub struct InputArg {
    pub name: Option<String>,
    pub value: InputExpr,
}

impl InputArg {
    pub fn pos(value: InputExpr) -> Self {
        InputArg { name: None, value }
    }

    pub fn named(name: impl Into<String>, value: InputExpr) -> Self {
        InputArg {
            name: Some(name.into()),
            value,
        }
    }
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
    /// PLAN-963（T-02）：文本字面量（记录域 `str` 面）。
    StrLit(String),
    Var(String),
    /// PLAN-963（T-02）：记录构造 `Name { field: e, ... }`——结果类型是
    /// 声明记录名，字段值按声明字段注解核对。
    Ctor {
        /// Interned declared record name (the solver's `Ty::Record` key).
        name: &'static str,
        /// Source-ordered field values (each bound to a field by name when
        /// written in named form).
        fields: Vec<InputArg>,
    },
    /// PLAN-963（T-02）：字段读取 `base.field`。
    Field {
        base: Box<InputExpr>,
        name: String,
    },
    /// PLAN-963（T-02）：方法调用 `recv.name(args)`（接收位是记录值）。
    MethodCall {
        recv: Box<InputExpr>,
        name: String,
        args: Vec<InputArg>,
    },
    Call {
        callee: String,
        /// Source-ordered arguments (evaluation order), each optionally
        /// bound to a parameter by name.
        args: Vec<InputArg>,
    },
    Binary {
        op: BinOpKind,
        lhs: Box<InputExpr>,
        rhs: Box<InputExpr>,
    },
    Unary {
        neg: bool,
        operand: Box<InputExpr>,
    },
    /// A construction outside the limited surface: reported as
    /// unsupported-construct, never silently typed.
    Unsupported {
        what: &'static str,
    },
}

impl InputExpr {
    pub fn int(v: i32) -> Self {
        InputExpr {
            kind: InputExprKind::IntLit(v),
            span: Span::new(0, 0),
        }
    }
    pub fn boolean(v: bool) -> Self {
        InputExpr {
            kind: InputExprKind::BoolLit(v),
            span: Span::new(0, 0),
        }
    }
    pub fn var(name: &str) -> Self {
        InputExpr {
            kind: InputExprKind::Var(name.into()),
            span: Span::new(0, 0),
        }
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
            InputExpr {
                kind: InputExprKind::IntLit(0),
                span: Span::new(0, 0),
            }
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
            InputExprKind::Field { base, .. } => {
                let b = std::mem::replace(base.as_mut(), dummy());
                stack.push(b);
            }
            InputExprKind::MethodCall { recv, args, .. } => {
                let r = std::mem::replace(recv.as_mut(), dummy());
                stack.push(r);
                for a in args.iter_mut() {
                    stack.push(std::mem::replace(&mut a.value, dummy()));
                }
            }
            InputExprKind::Ctor { fields, .. } => {
                for a in fields.iter_mut() {
                    stack.push(std::mem::replace(&mut a.value, dummy()));
                }
            }
            InputExprKind::Call { args, .. } => {
                for a in args.iter_mut() {
                    stack.push(std::mem::replace(&mut a.value, dummy()));
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

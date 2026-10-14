p = 'src/source/resolve.rs'
s = open(p, encoding='utf-8').read()
orig = s

# ---- VT gains Rec ----
s = s.replace("""#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum VT {
    I32,
    Bool,
}

impl VT {
    pub fn as_str(&self) -> &'static str {
        match self {
            VT::I32 => "i32",
            VT::Bool => "bool",
        }
    }
}""",
"""#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum VT {
    I32,
    Bool,
    /// Nominal record type by index into [`ResolvedProgram::records`]
    /// (PLAN-771). Two records with identical shapes are distinct types.
    Rec(usize),
}

impl VT {
    pub fn as_str(&self) -> &'static str {
        match self {
            VT::I32 => "i32",
            VT::Bool => "bool",
            VT::Rec(_) => "record",
        }
    }
}""")

# ---- record tables ----
s = s.replace("""pub struct ResolvedProgram {
    pub fns: Vec<ResolvedFn>,
}""",
"""pub struct ResolvedProgram {
    pub fns: Vec<ResolvedFn>,
    /// Nominal record types in declaration order (PLAN-771). `VT::Rec(i)`
    /// indexes here.
    pub records: Vec<ResolvedRecord>,
}

/// One resolved record type: fields in declaration order with their value
/// types; methods with their resolved fn indexes.
#[derive(Clone, Debug)]
pub struct ResolvedRecord {
    pub name: String,
    pub name_span: Span,
    pub fields: Vec<ResolvedField>,
    pub methods: Vec<ResolvedMethod>,
}

#[derive(Clone, Debug)]
pub struct ResolvedField {
    pub name: String,
    pub vt: VT,
    pub span: Span,
}

/// A type-bound method (PLAN-771): the resolved fn carries the body; this
/// entry is the type-side view (receiver shape + param modes).
#[derive(Clone, Debug)]
pub struct ResolvedMethod {
    pub name: String,
    /// static (no receiver) or readonly view (receiver = params[0]).
    pub is_static: bool,
    /// Index into `ResolvedProgram::fns`.
    pub fn_index: usize,
    pub span: Span,
}""")

# ---- LocalDecl param mode ----
s = s.replace("""#[derive(Clone, Debug)]
pub struct LocalDecl {
    pub name: String,
    pub vt: VT,
    pub mutable: bool,
    pub is_param: bool,
    pub param_index: Option<u32>,
    /// Name through the initializer (let/var) or the whole param phrase.
    pub span: Span,
}""",
"""#[derive(Clone, Debug)]
pub struct LocalDecl {
    pub name: String,
    pub vt: VT,
    pub mutable: bool,
    pub is_param: bool,
    pub param_index: Option<u32>,
    /// PLAN-771: explicit `mut` param mode (exclusive borrow); `false` for
    /// every non-record param and every binding.
    pub param_mut: bool,
    /// Name through the initializer (let/var) or the whole param phrase.
    pub span: Span,
}""")

# ---- RExprKind record variants ----
s = s.replace("""    Unary {
        op: UnaryOp,
        operand: Box<ResolvedExpr>,
    },
}""",
"""    Unary {
        op: UnaryOp,
        operand: Box<ResolvedExpr>,
    },
    // -- record profile (PLAN-771) --
    /// Positional construction `Pair(1, true)` — args in source order,
    /// bound to declaration fields by position.
    CtorPos {
        rec: usize,
        args: Vec<ResolvedArg>,
    },
    /// Named construction — field values in SOURCE order with their field
    /// names (the adapter maps to declaration indices; evaluation order is
    /// the written order, never declaration order).
    CtorNamed {
        rec: usize,
        fields: Vec<(String, ResolvedExpr)>,
    },
    /// Field read through a resolved path: each segment is
    /// (owner record index, field declaration index).
    FieldGet {
        base: Box<ResolvedExpr>,
        path: Vec<(usize, usize)>,
    },
    /// Method call: `recv` is `None` for static methods (the receiver was
    /// a bare type name). Args exclude the receiver for instance methods —
    /// the adapter re-attaches it as argument 0 (borrowed view).
    MethodCall {
        fn_index: usize,
        recv: Option<Box<ResolvedExpr>>,
        args: Vec<ResolvedArg>,
    },
    /// `p.mut` — exclusive borrow argument (record profile).
    BorrowMut {
        local: LocalRef,
    },
}""")

# ---- ResolvedFn gains receiver info + modes ----
s = s.replace("""pub struct ResolvedFn {
    pub name: String,
    pub name_span: Span,
    /// `fn` keyword through the body's closing brace (duplicate-fn span).
    pub item_span: Span,
    pub param_names: Vec<String>,
    pub params: Vec<VT>,
    pub result: VT,
    pub locals: Vec<LocalDecl>,
    pub body: RBlock,
    /// Number of registered loops (While + Loop statements) in walk order.
    pub loop_count: usize,
    /// Trace intrinsic symbols referenced by this function's body.
    pub used_intrinsics: BTreeSet<&'static str>,
}""",
"""pub struct ResolvedFn {
    pub name: String,
    pub name_span: Span,
    /// `fn` keyword through the body's closing brace (duplicate-fn span).
    pub item_span: Span,
    pub param_names: Vec<String>,
    pub params: Vec<VT>,
    /// PLAN-771: per-param explicit `mut` mode (false everywhere else).
    pub param_modes: Vec<bool>,
    pub result: VT,
    pub locals: Vec<LocalDecl>,
    pub body: RBlock,
    /// Number of registered loops (While + Loop statements) in walk order.
    pub loop_count: usize,
    /// Trace intrinsic symbols referenced by this function's body.
    pub used_intrinsics: BTreeSet<&'static str>,
    /// PLAN-771: receiver shape — Free for top-level functions, Static /
    /// View for methods (View's params[0] is the implicit `self` view).
    pub receiver: ast::Receiver,
    /// Owning record index for methods.
    pub owner_rec: Option<usize>,
}""")

assert s != orig
open(p, 'w', encoding='utf-8', newline='\n').write(s)
print('resolve patch A ok')

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
    /// `str` — borrowed UTF-8 slice (PLAN-770, text profile only).
    Str,
    /// `Str` — owned UTF-8 buffer (PLAN-770).
    StrOwned,
    /// `char` — Unicode scalar (PLAN-770).
    Char,
    /// `void` — the Unit result type (PLAN-862, entry-unit profile only).
    /// Legal in RESULT position of a function declaration; never a value
    /// type (no locals, params, fields or expression types).
    Void,
    /// PLAN-961 numeric surface type words (`byte`≡U8, `uint`≡U32,
    /// `float`≡F32, `double`≡`f64`≡F64; `int`/`i32` stay the adjudicated
    /// 32-bit integer above).
    I8,
    U8,
    Byte,
    Uint,
    U32,
    I64,
    U64,
    Float,
    Double,
    F64,
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
    /// PLAN-1080 (D-04): the annotation was OMITTED (Auto face untyped
    /// parameter / return). Never produced by the frozen narrow faces
    /// (their grammars require the annotation); the inference pre-pass
    /// replaces every `Inferred` with a concrete type or reports
    /// `bind/source.inference` — resolve never sees one.
    Inferred,
    /// PLAN-935: a declared type parameter reference (`T`) — only produced
    /// under the `ac.source.instances.scalar/v1` capability face, and only
    /// valid inside the template fn that declares it. Never reaches a
    /// Checked unit: the instances layer concretizes every `TyVar` into a
    /// concrete builtin before resolve/emit; a `TyVar` seen by resolve or
    /// the semantics adapter is a definitive named rejection (template not
    /// instantiated), never a silent fallback.
    TyVar(Ident),
    /// Nominal record type by declared name.
    Record(Ident),
    /// PLAN-864: `?T` — the optional-type sugar for `Option<T>` (collections
    /// surface). `inner` is the payload type.
    Optional(Box<TypeAnn>),
    /// PLAN-864: a parameterized nominal type — `List<T>` / `Result<T, E>`
    /// (collections surface). `args` are the type arguments in written
    /// order; arity is checked in resolve.
    Generic {
        name: Ident,
        args: Vec<TypeAnn>,
    },
    /// PLAN-962 (L-15): a tuple type `(T1, T2, ...)`.
    Tuple(Vec<TypeAnn>),
    /// PLAN-865: closure function type `fn(T0, T1) -> R`.
    Function {
        params: Vec<TypeAnn>,
        result: Box<TypeAnn>,
    },
    /// PLAN-965 (M2-A): `!T` result-with-str-error sugar (`Result<T, Str>`).
    ResultBang(Box<TypeAnn>),
    /// PLAN-965 (M2-A): `~Iter<T>` generator stream type.
    Stream(Box<TypeAnn>),
}

/// Parameter ownership mode word (PLAN-770; old-language convention: the
/// mode word precedes the name — `fn f(mut s str)` / `fn f(take move Str)`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ParamMode {
    /// No mode word: text params default to a read-only view.
    Default,
    /// `mut` — exclusive borrow (`mut str` borrows mutable, `mut Str`
    /// exclusively borrows an owned value).
    Mut,
    /// `move` — ownership transfer in (`move Str` only; call sites must
    /// mark `.move`).
    Move,
}

#[derive(Clone, Debug)]
pub struct Param {
    pub name: Ident,
    pub ty: TypeAnn,
    pub mode: ParamMode,
    /// PLAN-963 (T-08, L-24): the declared default value (`n int = 42`).
    /// Defaults are resolved at each CALL SITE in the caller's scope (never
    /// in the callee's), so an expression that names a parameter or local is
    /// a bind-level error rather than a silent rebinding; the value is
    /// evaluated exactly once, at the call, and only when the argument is
    /// omitted.
    pub default: Option<Box<Expr>>,
    /// Mode word (when present) through the type token: the span used by
    /// duplicate-param diagnostics.
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
    /// Mutable instance method: implicit `mut self` param.
    Mut,
}

impl Receiver {
    pub(crate) fn is_instance(self) -> bool {
        matches!(self, Self::View | Self::Mut)
    }
}

#[derive(Clone, Debug)]
pub struct FnItem {
    pub name: Ident,
    /// PLAN-935: declared type parameters (`<T>`) in written order. PLAN-963
    /// （T-02）把形状统一到 [`TypeParamDecl`]（可携带 `<T has S>` 约束）。
    /// 仅非空于实例能力面；带类型参数的 fn 是**模板**——语义描述，永不是
    /// 独立检查单元（实例层把模板从脱糖程序移除，换成按需的具体实例）。
    pub type_params: Vec<TypeParamDecl>,
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
    /// PLAN-862: this function is the SYNTHETIC entry built from a
    /// single-file script (top-level statement sequence). The parser owns
    /// the naming (`main`); resolve owns the result type (a trailing i32
    /// expression statement types the script as `int`, otherwise Unit).
    /// Never true for a declared `fn`.
    pub script: bool,
    /// PLAN-1080 (D-05): this function is a signature-only PROTOTYPE
    /// declaration (`fn test() int;` — the legacy 009 forward-decl shape,
    /// Auto face only). The projection leaves the body empty and the
    /// inference pre-pass unifies it with the real body item before
    /// resolve; a proto without a matching body is a named bind error.
    pub proto: bool,
    /// Original name span of an explicit test declaration attribute.
    pub test_attribute: Option<Span>,
}

/// One record field, in declaration order (= layout order, 774 D-05).
#[derive(Clone, Debug)]
pub struct FieldDecl {
    pub name: Ident,
    pub ty: TypeAnn,
    /// Field name token through the type token.
    pub span: Span,
}

/// PLAN-963 (T-03): one declared type parameter of a user generic type.
/// A bound (`<T has S>`) is checked at monomorphization: the substituted
/// type must be a record whose conformance list names `S`.
#[derive(Clone, Debug)]
pub struct TypeParamDecl {
    pub name: Ident,
    pub bound: Option<Ident>,
    /// PLAN-963 (T-04): value parameter (`<const N>` / `<const N u32>`).
    pub kind: TypeParamKind,
}

/// PLAN-963 (T-04): the two declaration kinds of a generic parameter.
///
/// The only accepted value spelling is `<const N>` (declared type defaults to
/// `u32`) and its explicit twin `<const N u32>`; any other declared type is a
/// named projection rejection. A value parameter is substituted by a compile
/// time integer at instantiation and is therefore usable inside the template
/// body (it is never a runtime function argument).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TypeParamKind {
    Type,
    Const,
}

/// Explicit method instance arguments in source order within each kind.
#[derive(Clone, Debug)]
pub struct InstanceArguments {
    pub types: Vec<TypeAnn>,
    pub consts: Vec<ConstArgUse>,
}

/// PLAN-963 (T-04): one explicit value argument at a call site
/// (`rep<2>(21)` — the `2`). The value domain is the declared type of the
/// parameter (`u32`, so `0..=4294967295`); projection already rejected
/// anything outside it, and the instance lane re-checks against the
/// declaration before minting an identity.
#[derive(Clone, Debug)]
pub struct ConstArgUse {
    pub value: u64,
    pub span: Span,
}

/// A `type Name { fields; methods }` item (record profile only).
#[derive(Clone, Debug)]
pub struct TypeItem {
    pub name: Ident,
    /// PLAN-963 (T-02): declaration-order type parameters of a **generic user
    /// record** (`type Point<T>`). Empty for ordinary records. Templates are
    /// monomorphized at projection time (explicit type arguments at every
    /// use), so no synthesized `TypeItem` reaching resolve carries these.
    pub type_params: Vec<TypeParamDecl>,
    pub instance: Option<RecordInstance>,
    /// PLAN-963 (T-03): specs claimed by `type X as A[, B]`, in written order.
    /// Consumed at projection time (conformance validation + default method
    /// injection); resolve and the four backends see only concrete methods.
    pub conforms: Vec<ConformRef>,
    /// Verified direct and inherited spec memberships.
    pub spec_memberships: Vec<String>,
    pub fields: Vec<FieldDecl>,
    pub methods: Vec<FnItem>,
    /// `type` keyword through the closing brace.
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct RecordInstance {
    pub template: String,
    pub arguments: Vec<TypeAnn>,
}

/// One conformance claim of `type X as S` / `as S<int>` (PLAN-963 T-03).
/// `args` carries the written type arguments of a **generic spec**; they are
/// consumed at projection time (instantiated into the injected methods), so no
/// downstream stage ever sees an unsubstituted spec parameter.
///
/// PLAN-963 (T-05): `assoc` carries the named associated-type bindings of
/// `as S<Item=int>` (written order, name left of `=`). Also consumed at
/// projection time — specs are erased before resolve.
#[derive(Clone, Debug)]
pub struct ConformRef {
    pub name: Ident,
    pub args: Vec<TypeAnn>,
    pub assoc: Vec<(Ident, TypeAnn)>,
}

#[derive(Clone, Debug)]
pub enum Item {
    Fn(FnItem),
    Type(TypeItem),
    /// PLAN-864: a finite `enum` declaration (collections surface) — one
    /// nominal variant type whose arms carry at most one payload each.
    Enum(EnumItem),
    /// PLAN-960: a module-level constant `const NAME: T = expr` (packages
    /// module surface). Immutable global; the initializer is evaluated at
    /// module initialization time, not folded at parse time.
    Const(ConstItem),
    /// PLAN-960: a module-level mutable store `var NAME: T = expr`
    /// (packages module surface, module-private; no `pub` form).
    Store(StoreItem),
    /// PLAN-967 F0: the public Rust import frontdoor `use.rs <provider>`.
    /// Exactly one per module; the provider id must match the session's
    /// admitted foreign manifest (validated in resolve, never parsed away).
    UseRs(UseRsItem),
    /// PLAN-1026: the Python import frontdoor `use.py <module>: <members>`
    /// (dotted module path, explicit member list; multiple items per
    /// module are legal, each member binds once in source order).
    UsePy(UsePyItem),
    /// PLAN-968: a `task` actor declaration (scheduler surface only).
    Task(TaskItem),
    /// PLAN-1095: an AGE node declaration admitted on the ordinary Auto
    /// face. The declaration body is consumed at parse time (CPU-only: the
    /// node view text is observed via the 979 producer; no renderer, no
    /// event/scene lanes). Resolve/typecheck/lower never see the body.
    Node(NodeItem),
}

/// PLAN-1095: a `node <Name> { ... }` declaration (ordinary Auto face).
#[derive(Clone, Debug)]
pub struct NodeItem {
    pub name: Ident,
    pub span: Span,
}

impl Item {
    pub(crate) fn span(&self) -> Span {
        match self {
            Self::Fn(item) => item.span,
            Self::Type(item) => item.span,
            Self::Enum(item) => item.span,
            Self::Const(item) => item.span,
            Self::Store(item) => item.span,
            Self::UseRs(item) => item.span,
            Self::UsePy(item) => item.span,
            Self::Task(item) => item.span,
            Self::Node(item) => item.span,
        }
    }
}

/// PLAN-968: a `task Name { ... }` actor declaration (scheduler surface).
/// Members: state fields with defaults (`count = 0`), optional lifecycle
/// hooks `fn start() ! {}` / `fn stop() ! {}`, and one `on { ... }` message
/// handler block whose arms pattern-match the (monomorphic) message type.
#[derive(Clone, Debug)]
pub struct TaskItem {
    pub name: Ident,
    pub single: bool,
    pub fields: Vec<TaskFieldDecl>,
    pub start: Option<FnItem>,
    pub stop: Option<FnItem>,
    pub arms: Vec<OnArm>,
    /// `task` keyword through the closing brace.
    pub span: Span,
}

/// PLAN-968: one task state field `name = <default expr>` (type inferred
/// from the default, spawn-time positional initializers override it in
/// declaration order — the Plan 390 §5 Phase B public contract).
#[derive(Clone, Debug)]
pub struct TaskFieldDecl {
    pub name: Ident,
    pub default: Expr,
    /// Field name token through the default expression.
    pub span: Span,
}

/// PLAN-968: one `on` message arm `pattern -> { body }`.
#[derive(Clone, Debug)]
pub struct OnArm {
    pub pattern: TaskPattern,
    pub guard: Option<Expr>,
    pub body: Block,
    /// Pattern start through the closing brace.
    pub span: Span,
}

/// PLAN-968: an arm pattern — a literal (`1 ->`), a typed binding
/// (`n int ->`) or the wildcard binding (`_ ->`, binds nothing).
#[derive(Clone, Debug)]
pub enum TaskPattern {
    Lit(Expr),
    Bind { name: Ident, ty: TypeAnn },
    Named { name: Ident, bindings: Vec<Ident> },
    Wildcard(Span),
}

/// A `use.rs <provider>` / `use.c <provider>` module item (PLAN-967
/// F0/T-10). `lang` is the frozen spelling marker ("rs" | "c"); the
/// resolve pass checks it against the admitted manifest family.
#[derive(Clone, Debug)]
pub struct UseRsItem {
    pub lang: String,
    pub provider: Ident,
    /// `use` keyword through the provider identifier.
    pub span: Span,
}

/// A `use.py <module.path>: <member, ...>` module item (PLAN-1026). The
/// dotted module path and member names are runtime import facts — the
/// resolve pass binds each member to a py-bridge token local and lowers
/// uses through the frozen Python op set; nothing is imported at compile
/// time (R-01).
#[derive(Clone, Debug)]
pub struct UsePyItem {
    pub module: Vec<Ident>,
    pub members: Vec<Ident>,
    /// `use` keyword through the last member.
    pub span: Span,
}

/// A `const NAME: T = expr` module item (PLAN-960).
#[derive(Clone, Debug)]
pub struct ConstItem {
    pub name: Ident,
    pub ty: TypeAnn,
    pub value: Expr,
    /// `const` keyword through the end of the initializer expression.
    pub span: Span,
}

/// A `var NAME: T = expr` module store item (PLAN-960).
#[derive(Clone, Debug)]
pub struct StoreItem {
    pub name: Ident,
    pub ty: TypeAnn,
    pub value: Expr,
    /// `var` keyword through the end of the initializer expression.
    pub span: Span,
}

/// A `enum Name { Arm(T)? ... }` item (PLAN-864, collections surface).
/// Arms are whitespace separated (optional `,` tolerated); declaration
/// order is the runtime discriminant order. PLAN-1083 (auto face): arms
/// may carry named-field payloads (`Arm { w int, h int }`, `fields`) or
/// an explicit scalar discriminant (`Arm = 3`, `disc`), and the enum body
/// may carry `fn` members (`methods`) — the instance-method surface with
/// the implicit variant receiver.
#[derive(Clone, Debug)]
pub struct EnumItem {
    pub name: Ident,
    pub type_params: Vec<TypeParamDecl>,
    pub instance: Option<EnumInstance>,
    /// Variants synthesized from verified spec conformance witnesses.
    pub spec_witnesses: bool,
    pub arms: Vec<EnumArm>,
    pub methods: Vec<FnItem>,
    /// `enum` keyword through the closing brace.
    pub span: Span,
}

/// Source family and unavailable payloads of an inferred enum instance.
#[derive(Clone, Debug)]
pub struct EnumInstance {
    pub template: String,
    pub arguments: Vec<Option<TypeAnn>>,
    pub unbound_arms: Vec<String>,
    pub unbound_methods: Vec<String>,
}

/// One variant arm: `Name` or `Name(PayloadType)`. PLAN-1083 (auto face):
/// `Name(T1, T2, ...)` projects a multi-element tuple payload into
/// `payload = TypeRef::Tuple`; `Name { f int, ... }` carries `fields`
/// (resolve interns the synthesized record payload); `Name = N` carries
/// the explicit scalar `disc`.
#[derive(Clone, Debug)]
pub struct EnumArm {
    pub name: Ident,
    pub payload: Option<TypeAnn>,
    pub fields: Vec<(Ident, TypeAnn)>,
    pub disc: Option<i32>,
    /// Arm name token through the payload type token.
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct Program {
    pub items: Vec<Item>,
}

impl Program {
    pub fn has_instance_templates(&self) -> bool {
        fn generic(f: &FnItem) -> bool {
            !f.type_params.is_empty()
        }
        self.items.iter().any(|item| match item {
            Item::Fn(f) => generic(f),
            Item::Type(t) => !t.type_params.is_empty() || t.methods.iter().any(generic),
            Item::Enum(e) => !e.type_params.is_empty() || e.methods.iter().any(generic),
            _ => false,
        })
    }
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
    /// PLAN-961: int-family bit ops (numeric profile; the projection
    /// maps amp/vbar/caret leaves — `&&`/`||` stay the logical And/Or).
    BitAnd,
    BitOr,
    BitXor,
    /// PLAN-965 (M2-A): lazy null-coalescing `lhs ?? rhs`.
    NullCoalesce,
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
            // PLAN-961: bit-op spellings.
            BinOp::BitAnd => "&",
            BinOp::BitOr => "|",
            BinOp::BitXor => "^",
            BinOp::NullCoalesce => "??",
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
    /// `x.move` — a positional argument marked for ownership transfer
    /// (PLAN-770; only legal as a direct call argument).
    MovePos(Expr),
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
    /// PLAN-962 (L-15): a tuple literal `(e1, e2, ...)` (>= 2 elements).
    TupleLit(Vec<Expr>),
    /// PLAN-962 (T-08, L-14): a grid literal `grid (c1, c2) { [r..], ... }`
    /// — column names plus rows; the resolver desugars at the statement
    /// level into one List per column (column-tuple representation) with
    /// static row-length dimension checks.
    GridLit {
        cols: Vec<Ident>,
        rows: Vec<Vec<Expr>>,
    },
    /// PLAN-962 (L-16): a non-empty array/List literal `[e1, e2, ...]`.
    /// The resolver desugars at the statement level (`List<T>.new()` +
    /// pushes); other positions reject with a named diagnostic.
    ArrayLit(Vec<Expr>),
    /// A list element whose value is evaluated and appended only if its condition holds.
    ArrayConditionalElem {
        cond: Box<Expr>,
        value: Box<Expr>,
    },
    /// PLAN-962 (L-48): an index suffix `base[index]` — reads desugar to
    /// the element-get path (view for owned elements); index-assign
    /// targets desugar to the list-set effect.
    Index {
        base: Box<Expr>,
        index: Box<Expr>,
    },
    /// PLAN-964 (L-16/L-20): a bare object literal `{ k: v, ... }` —
    /// keys are idents or string literals, values homogeneous. The
    /// resolver types it `Object{elem}` and lowers to HIR ObjectCtor.
    ObjectLit {
        fields: Vec<ObjectField>,
    },
    /// PLAN-964 (L-21): the nil literal (`nil` / `null`, PLAN-053 unified).
    NilLit,
    /// PLAN-964 (L-22/FStr): display an I32 as a decimal owned Str.
    TextDisplay(Box<Expr>),
    /// PLAN-968: `await <expr>` — park the current task chain until the
    /// future is published (scheduler surface only). The operand is the
    /// future-bearing expression (a handle `ask` call).
    Await(Box<Expr>),
    /// PLAN-968: `go <expr>` — fire-and-forget concurrent evaluation of a
    /// call expression on a fresh anonymous task chain (scheduler surface
    /// only). Yields no value.
    Go(Box<Expr>),
    /// PLAN-962 (L-19): a range `a..b` / `a..=b` — legal ONLY in the
    /// for-in iterable position (the resolver desugars to a counted
    /// i32 loop; a range in any other position is a named rejection).
    Range {
        start: Box<Expr>,
        end: Box<Expr>,
        inclusive: bool,
    },
    /// PLAN-961: a typed numeric literal (`1i64`, `0x1Fu32`, `1.5f32`,
    /// `2.5d`, `3b`). Decoded at the projection: `bits` carries the
    /// payload in the type's own width (i8/u8/u32 low bits, i64/u64 full,
    /// f32/f64 `to_bits()`); negative literals fold into the signed
    /// payload there; a negative unsigned literal is a projection-time
    /// `source.literal-range` rejection.
    TypedLit {
        ty: TypeWord,
        bits: u64,
    },
    BoolLit(bool),
    Var(Ident),
    /// Decoded string literal (text profile, PLAN-770); cross-line breaks
    /// are part of the value.
    StrLit(String),
    /// Decoded char literal: one Unicode scalar.
    CharLit(u32),
    /// Method call `recv.name(args)` (text profile). Builtin text methods
    /// bind by receiver type in the resolver; `char.from_codepoint` is the
    /// type-level form with a `Var("char")` receiver that must not be
    /// shadowed. `.to(str|Str)` carries a single [`ExprKind::TypeWordArg`]
    /// argument; `.move` parses as a bare method (only legal as a direct
    /// call argument).
    Method {
        recv: Box<Expr>,
        name: Ident,
        args: Vec<Arg>,
    },
    /// A bare type word in `.to(...)` argument position (never a value).
    TypeWordArg(TypeWord),
    Call {
        callee: Ident,
        args: Vec<Arg>,
    },
    /// PLAN-963 (T-04): explicit value arguments — `rep<2>(21)`.
    ///
    /// Only the `generic-call` atom of the instance faces produces this shape.
    /// The instance lane rewrites it into a plain [`ExprKind::Call`] naming the
    /// concrete instance (the values are consumed by the instance identity and
    /// the body substitution), so resolve / HIR / the four backends never see
    /// it — a `ConstCall` reaching resolve is a named rejection.
    ConstCall {
        callee: Ident,
        /// Explicit type arguments on the ordinary surface. Source spans
        /// retain their order relative to the explicit value arguments.
        types: Vec<TypeAnn>,
        consts: Vec<ConstArgUse>,
        args: Vec<Arg>,
    },
    /// An indirect call through a function-typed expression.
    CallValue {
        callee: Box<Expr>,
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
    /// Named construction `Pair { left: 1, right: true }` — arguments
    /// evaluate in SOURCE order, then map to declaration fields.
    /// (Positional `Pair(1, true)` parses as a plain call; the resolver
    /// routes it to construction when the name is a record — F-771-R1-02.)
    CtorNamed {
        ty: Ident,
        /// PLAN-963（T-02）：显式类型实参形 `Point<int> { .. }` 的实参表
        /// （空 = 旧写法的普通具名构造）。投影期单态化把它解析为具体记录
        /// 并清空，resolve 之后的一切只见具体 nominal 构造。
        args: Vec<TypeAnn>,
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
        instance: Option<Box<InstanceArguments>>,
    },
    /// `p.mut` — the explicit mut-borrow argument form (record profile).
    /// PLAN-962 (T-08, L-06): `x.view` — immutable borrow argument
    /// (the readonly twin of [`ExprKind::MutArg`]).
    ViewArg {
        name: Ident,
    },
    MutArg {
        name: Ident,
    },
    /// PLAN-864: a parameterized type written in receiver position —
    /// `List<int>.new()`. Only legal as the receiver of a constructor
    /// method call; resolve owns the routing.
    GenericType {
        name: Ident,
        args: Vec<TypeAnn>,
    },
    /// PLAN-936（T-04）：编译期表达式 `#{ expr }` —— 单表达式 CT 需求
    /// 原子。仅 scalar-ct 面（`ac.source.cte.scalar/v1` 能力）解析产出；
    /// inner 由普通 resolver 绑定/类型，HashExpr 自身类型 = inner 类型。
    /// HIR 发射前必须已由 CT 服务回灌为常量——未服务的 HashExpr 在发射
    /// 处受控拒（不得静默当运行时表达式下跑，§5.2）。
    HashExpr {
        expr: Box<Expr>,
    },
    /// PLAN-865 lambda expression. Capture analysis runs in resolve and
    /// records the ordered environment before HIR emission.
    Closure {
        move_capture: bool,
        params: Vec<Param>,
        result: Option<TypeAnn>,
        body: Box<ClosureBody>,
    },
    /// PLAN-965 (M2-A): `expr.?` propagation postfix step.
    Propagate {
        inner: Box<Expr>,
    },
    /// PLAN-965 (M2-A): `expr.?(fallback)` lazy unwrap-or postfix step.
    UnwrapOr {
        inner: Box<Expr>,
        fallback: Box<Expr>,
    },
    /// PLAN-965 (M2-A): `is expr { arms... }` expression form.
    IsExpr {
        scrutinee: Box<Expr>,
        arms: Vec<AggMatchArm>,
    },
}

#[derive(Clone, Debug)]
pub enum ClosureBody {
    /// `(x int) => x + 1` synthesizes the expression's value as its result.
    Expr(Expr),
    /// `(x int) => { return x }` follows ordinary function return rules.
    Block(Block),
}

/// One `name: value` entry of a named constructor.
#[derive(Clone, Debug)]
pub struct NamedCtorField {
    pub name: Ident,
    pub value: Expr,
    pub span: Span,
}

/// PLAN-964: one `key: value` entry of an object literal — the key is a
/// static identifier or string literal (dynamic keys go through
/// index-assign).
#[derive(Clone, Debug)]
pub struct ObjectField {
    pub key: crate::hir::ObjectKey,
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

/// PLAN-962 (L-48): `xs[i] = v` — element replacement through the index
/// place (list-set semantics; `.set(i, v)` spelled positionally).
#[derive(Clone, Debug)]
pub struct IndexAssign {
    pub base: Ident,
    pub index: Expr,
    pub value: Expr,
    pub span: Span,
}

/// PLAN-1082: one step of a place chain — a record field or an index.
#[derive(Clone, Debug)]
pub enum PlaceStep {
    Field(Ident),
    Index(Expr),
}

/// PLAN-1082: the assignment operator of a place-chain statement; the
/// compound spellings are the two-token `+ =`-style sequences.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PlaceOp {
    Set,
    Add,
    Sub,
    Mul,
    Div,
}

/// PLAN-1082 (L-19/L-48): `base (.f | [e])* (op)= v` — the unified
/// place-chain write. The grammar orders this after path/index assign so
/// it only captures shapes those frozen rules do not match.
#[derive(Clone, Debug)]
pub struct PlaceAssign {
    pub base: Ident,
    pub steps: Vec<PlaceStep>,
    pub op: PlaceOp,
    pub value: Expr,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub enum Stmt {
    /// PLAN-862: an expression evaluated for its value only where the
    /// surface consumes it — the SINGLE legal position is the script's
    /// trailing statement (the tail value). Resolve rejects every other
    /// occurrence (`source.entry-tail`), so no discarded-value semantics
    /// exists in this profile.
    ExprStmt {
        value: Expr,
        span: Span,
    },
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
    /// PLAN-962 (L-48): `xs[i] = v`.
    IndexAssign(IndexAssign),
    /// PLAN-1082 (L-19/L-48): `base (.f | [e])* (op)= v` — the unified
    /// place-chain assignment/compound-assignment statement (mixed
    /// field×index chains, chained indices, and compound `+= -= *= /=`).
    PlaceAssign(PlaceAssign),

    /// PLAN-962 (L-15): `let (a, b, ...) = expr` — flat tuple
    /// destructuring (binding names map positionally to 0..n-1).
    LetDestruct {
        names: Vec<Ident>,
        value: Expr,
        span: Span,
    },
    /// PLAN-962 (T-08, L-07): `let hold x as v { }` — whole-local hold.
    /// The binder aliases the held local for the block; the binding dies
    /// with the block (>= the old engine's MVP semantics — its own
    /// source notes Hold carried no borrow discipline).
    Hold {
        name: Ident,
        bind: Ident,
        body: Block,
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
    /// PLAN-864: `is <local> { Arm(v)? -> { .. } ... }` — exhaustive
    /// variant dispatch (collections surface). Resolve checks
    /// exhaustiveness/duplicates and lowers to guard/branch chains with
    /// payload projections; there is no fall-through default.
    IsMatch {
        scrutinee: Ident,
        arms: Vec<MatchArm>,
        span: Span,
    },
    /// PLAN-864: `for x in <list-local> { .. }` — bounded List iteration
    /// (collections surface). Resolve lowers to an index loop; the
    /// binding is a copy for Copy elements and a checked read-only view
    /// context for owned elements.
    ForIn {
        var: Ident,
        iter: Expr,
        body: Block,
        span: Span,
    },
    /// PLAN-965 (M2-A): aggregates `is <expr> { arms... }` statement form.
    AggIsMatch {
        scrutinee: Expr,
        arms: Vec<AggMatchArm>,
        span: Span,
    },
    /// PLAN-965 (M2-A): `try { .. } [catch [(e)|e] { .. }] [finally { .. }]`.
    Try {
        body: Block,
        catch: Option<CatchClause>,
        finally: Option<Block>,
        span: Span,
    },
    /// PLAN-1094: `with <expr> [as <name>] { body }` — a context-manager
    /// statement (the manager's `__enter__` binds `name`, `__exit__` runs
    /// exactly once on every exit).
    With {
        expr: Expr,
        bind: Option<Ident>,
        body: Block,
        span: Span,
    },
    /// PLAN-965 (M2-A): `yield expr` inside a `~Iter<T>` generator body.
    Yield {
        value: Expr,
        span: Span,
    },
    /// PLAN-966（T-02）：语句级编译期条件 `#if <ct> { .. } [else { .. }]`。
    /// 绑定期按显式目标事实折叠为选中分支语句；未选分支不绑定/不发射。
    HashIf {
        cond: CtExpr,
        then: Block,
        els: Option<Block>,
        span: Span,
    },
    /// PLAN-966（T-02）：语句级编译期循环展开 `#for i in a..b { .. }`。
    /// 循环变量只在 CT 谓词域可见（运行时引用按普通未定义名处理，同旧实现）。
    HashFor {
        var: Ident,
        start: CtExpr,
        end: CtExpr,
        body: Block,
        span: Span,
    },
    /// PLAN-966（T-02）：语句级编译期匹配 `#is <ct> { pat -> { .. } }`。
    HashIs {
        scrutinee: CtExpr,
        arms: Vec<HashIsArm>,
        span: Span,
    },
    /// PLAN-1081 (T-03): a nested `fn` ITEM declared inside a function
    /// body (Auto face). Items are hoisted to the module function table
    /// under an owner-mangled name; they are call-only (never values) and
    /// follow Rust nested-fn semantics — no capture of enclosing locals.
    NestedFn {
        item: FnItem,
    },
}

/// PLAN-966（T-02）：`#is` 一个臂；`pattern = None` 是 `else` 臂。
#[derive(Clone, Debug)]
pub struct HashIsArm {
    pub pattern: Option<CtExpr>,
    pub body: Block,
    pub span: Span,
}

/// PLAN-966（T-02）：编译期标量表达式（项位 `#in`/`#if`/`#is`/`#for` 与语句级
/// 构造共用的谓词域：i32/bool 字面量、显式目标事实引用（`sys.current` 等）、
/// 比较/逻辑算子。无调用、算术与字符串；越域由绑定具名拒）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CtExpr {
    Int(i32),
    Bool(bool),
    /// `current` / `sys.Windows`（head[.member]；绑定期按事实/CT 循环变量解析）。
    Ref {
        head: String,
        member: Option<String>,
        span: Span,
    },
    Bin {
        op: CtBinOp,
        lhs: Box<CtExpr>,
        rhs: Box<CtExpr>,
        span: Span,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CtBinOp {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
}

impl CtExpr {
    pub fn span(&self) -> Span {
        match self {
            CtExpr::Int(_) | CtExpr::Bool(_) => Span { start: 0, end: 0 },
            CtExpr::Ref { span, .. } | CtExpr::Bin { span, .. } => *span,
        }
    }
}

#[derive(Clone, Debug)]
pub struct CatchClause {
    pub bind: Option<Ident>,
    pub body: Block,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub enum MatchPattern {
    Variant {
        qual: Option<Ident>,
        name: Ident,
        /// Payload bindings in source order — one for the PLAN-864 single
        /// payload, more for the PLAN-1083 multi-field/tuple destructuring
        /// (`Arm(a, b)`); empty for the bare arm.
        bindings: Vec<Ident>,
    },
    IntLit(i32),
    BoolLit(bool),
    Else,
}

#[derive(Clone, Debug)]
pub enum AggArmBody {
    Block(Block),
    Expr(Expr),
}

#[derive(Clone, Debug)]
pub struct AggMatchArm {
    pub pat: MatchPattern,
    pub body: AggArmBody,
    /// PLAN-1083 (auto face): `pat if cond -> body` — the guard evaluates
    /// only after the pattern matches (bindings visible); false continues
    /// to the next arm.
    pub guard: Option<Expr>,
    pub span: Span,
}

/// One `is` arm: `Name -> { .. }`, `Enum.Name -> { .. }` or with a
/// `(v)` payload binding (PLAN-864).
#[derive(Clone, Debug)]
pub struct MatchArm {
    /// Qualified form `Enum.Arm` — the enum name; `None` for bare arms.
    pub qual: Option<Ident>,
    pub pattern: Ident,
    /// `Some(v)` payload binding — a fresh local owned by the arm block.
    pub binding: Option<Ident>,
    pub body: Block,
    /// Pattern token through the block's closing brace.
    pub span: Span,
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
                ExprKind::Method { recv, args, .. } => {
                    let r = std::mem::replace(recv.as_mut(), dummy());
                    stack.push(r);
                    for a in args.iter_mut() {
                        let e = match a {
                            Arg::Pos(e) | Arg::MovePos(e) => std::mem::replace(e, dummy()),
                            Arg::Named { value, .. } => std::mem::replace(value, dummy()),
                        };
                        stack.push(e);
                    }
                }
                ExprKind::Call { args, .. } => {
                    for a in args.iter_mut() {
                        let e = match a {
                            Arg::Pos(e) | Arg::MovePos(e) => std::mem::replace(e, dummy()),
                            Arg::Named { value, .. } => std::mem::replace(value, dummy()),
                        };
                        stack.push(e);
                    }
                }
                ExprKind::ConstCall { args, .. } => {
                    for a in args.iter_mut() {
                        let e = match a {
                            Arg::Pos(e) | Arg::MovePos(e) => std::mem::replace(e, dummy()),
                            Arg::Named { value, .. } => std::mem::replace(value, dummy()),
                        };
                        stack.push(e);
                    }
                }
                ExprKind::CallValue { callee, args } => {
                    let e = std::mem::replace(callee.as_mut(), dummy());
                    stack.push(e);
                    for a in args.iter_mut() {
                        let e = match a {
                            Arg::Pos(e) | Arg::MovePos(e) => std::mem::replace(e, dummy()),
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
                            Arg::Pos(e) | Arg::MovePos(e) => std::mem::replace(e, dummy()),
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
                ExprKind::Closure { body, .. } => {
                    if let ClosureBody::Expr(expr) = body.as_mut() {
                        let e = std::mem::replace(expr, dummy());
                        stack.push(e);
                    }
                }
                ExprKind::Propagate { inner } => {
                    let e = std::mem::replace(inner.as_mut(), dummy());
                    stack.push(e);
                }
                ExprKind::UnwrapOr { inner, fallback } => {
                    let i = std::mem::replace(inner.as_mut(), dummy());
                    let f = std::mem::replace(fallback.as_mut(), dummy());
                    stack.push(i);
                    stack.push(f);
                }
                ExprKind::IsExpr { scrutinee, arms } => {
                    let s = std::mem::replace(scrutinee.as_mut(), dummy());
                    stack.push(s);
                    for a in arms.iter_mut() {
                        if let AggArmBody::Expr(expr) = &mut a.body {
                            let e = std::mem::replace(expr, dummy());
                            stack.push(e);
                        }
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
                    // PLAN-864: both new statements carry nested blocks.
                    Stmt::IsMatch { arms, .. } => {
                        for a in arms.iter_mut() {
                            stack.push(std::mem::replace(&mut a.body, dummy()));
                        }
                    }
                    Stmt::ForIn { body, .. } => {
                        stack.push(std::mem::replace(body, dummy()));
                    }
                    Stmt::AggIsMatch { arms, .. } => {
                        for a in arms.iter_mut() {
                            if let AggArmBody::Block(block) = &mut a.body {
                                stack.push(std::mem::replace(block, dummy()));
                            }
                        }
                    }
                    Stmt::Try {
                        body,
                        catch,
                        finally,
                        ..
                    } => {
                        stack.push(std::mem::replace(body, dummy()));
                        if let Some(c) = catch {
                            stack.push(std::mem::replace(&mut c.body, dummy()));
                        }
                        if let Some(f) = finally {
                            stack.push(std::mem::replace(f, dummy()));
                        }
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

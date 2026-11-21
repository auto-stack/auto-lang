//! Signature pre-registration, lexical name resolution and argument mapping
//! for the `auto.source.core-i32.draft` profile (PLAN-745 T-03).
//!
//! This is the source frontend's bind layer: names resolve against block
//! scopes (inner shadows outer; initializer right-hand sides see the
//! pre-declaration environment), function signatures are pre-registered so
//! forward and recursive calls resolve, and call arguments map to formals
//! either all-positionally or all by name. Trace intrinsics (`mark_a` /
//! `mark_b`) resolve only when the caller provided the trace capability,
//! and never shadow or hijack user functions or locals.
//!
//! Diagnostics use stage `bind` with `source.*` codes. Values get exact
//! types here ([`VT`]); mismatch *checking* is [`super::typecheck`].

use super::ast::{self, Arg, BinOp, Expr, ExprKind, FnItem, Stmt, TypeWord, UnaryOp};
use super::{meta_intrinsic, trace_intrinsic, SourceProfile, META_CAPABILITY, TRACE_CAPABILITY};
use crate::atom_text::{Diagnostic, Span, Stage};
use crate::py_ops::{PyLiteral, PyOpFact, PyOpKind, PyRole, PyValClass};
use std::collections::{BTreeMap, BTreeSet};

#[path = "agg_lower.rs"]
mod agg_lower;

#[path = "resolve_instances.rs"]
mod instances;
pub use instances::{discover_auto_instance_calls, AutoInstanceDemand, AutoInstanceDemands, AutoEnumDemand};

/// Value type of the profile: `int`/`i32` are the same exact signed 32-bit
/// type; `bool` is distinct with no implicit conversions.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum VT {
    I32,
    Bool,
    /// Borrowed text slice `str` (PLAN-770).
    Str,
    /// Owned text buffer `Str` (PLAN-770).
    StrOwned,
    /// Unicode scalar `char` (PLAN-770).
    Char,
    /// Nominal record type by index into [`ResolvedProgram::records`]
    /// (PLAN-771). Two records with identical shapes are distinct types.
    Rec(usize),
    /// Unit (PLAN-862): the `void` RESULT type. Never a value type — no
    /// `let`, param, field or expression may carry it; it exists so a
    /// function (and therefore the entry) can declare "no value".
    Unit,
    /// PLAN-864: owned typed list by element type (structural identity —
    /// index into [`CollTables::lists`], deduplicated).
    List(usize),
    /// PLAN-864: nominal variant (`?T` / `Result<T,E>` / declared `enum`)
    /// by index into [`CollTables::variants`]. Arm declaration order is
    /// the runtime discriminant order.
    Variant(usize),
    /// PLAN-865: invariant function signature, interned in `CollTables`.
    Function(usize),
    /// PLAN-962 (L-15): a structural tuple type by index into
    /// [`CollTables::tuples`] (interned, deduplicated by shape; identity
    /// = the element list).
    Tuple(usize),
    /// PLAN-964 (L-16/L-20): a shared mutable string-keyed object map by
    /// element type (structural — index into [`CollTables::objects`]).
    Object(usize),
    /// PLAN-964 (L-21): the null value type.
    Nil,
    /// Type-erased owned language value.
    Dyn,
    /// PLAN-968: an opaque task-actor handle, monomorphic per declared
    /// task — `VT::Task(i)` indexes into [`CollTables::sched_tasks`]. At
    /// runtime a handle is the i32 scheduler task id.
    Task(usize),
    /// Ordinary TaskRef<T>, interned by mailbox payload type.
    TaskRef(usize),
    /// PLAN-968: an unresolved `ask` reply future, monomorphic per task —
    /// `VT::Future(i)` indexes into [`CollTables::sched_tasks`]. At
    /// runtime a future is the i32 scheduler reply-port id.
    Future(usize),
    /// Ordinary Future<T>, indexed by the zero-argument completion signature.
    Deferred(usize),
    /// PLAN-961 numeric surface (`byte`≡U8, `uint`≡U32, `float`≡F32,
    /// `double`≡`f64`≡F64).
    I8,
    U8,
    U32,
    I64,
    U64,
    F32,
    F64,
}

impl VT {
    pub fn as_str(&self) -> &'static str {
        match self {
            VT::Tuple(_) => "tuple",
            VT::I32 => "i32",
            VT::Bool => "bool",
            VT::Str => "str",
            VT::StrOwned => "Str",
            VT::Char => "char",
            VT::Rec(_) => "record",
            VT::Unit => "unit",
            VT::List(_) => "list",
            // PLAN-961 numeric names.
            VT::I8 => "i8",
            VT::U8 => "u8",
            VT::U32 => "u32",
            VT::I64 => "i64",
            VT::U64 => "u64",
            VT::F32 => "f32",
            VT::F64 => "f64",
            VT::Variant(_) => "variant",
            VT::Function(_) => "function",
            VT::Object(_) => "object",
            VT::Nil => "nil",
            VT::Dyn => "Dyn",
            // PLAN-968 scheduler handle types (opaque i32 ids at runtime).
            VT::Task(_) | VT::TaskRef(_) => "task",
            VT::Future(_) | VT::Deferred(_) => "future",
        }
    }

    /// PLAN-864: whether this type owns resources under the collections
    /// contract (List/Variant containers, owned Str, records reaching a
    /// resource leaf). Copyable element types copy out; owned ones move.
    pub fn is_owned_coll(self, prog: &ResolvedProgram) -> bool {
        match self {
            VT::StrOwned | VT::List(_) | VT::Variant(_) | VT::Function(_) | VT::Object(_) => true,
            VT::Rec(ri) => record_reaches_resource(ri, &prog.records, &prog.tuples),
            VT::Tuple(ti) => prog
                .tuples
                .get(ti)
                .is_some_and(|elems| elems.iter().any(|&v| v.is_owned_coll(prog))),
            _ => false,
        }
    }

    pub fn is_text(self) -> bool {
        matches!(self, VT::Str | VT::StrOwned)
    }
}

pub fn vt_of_word(w: TypeWord) -> VT {
    match w {
        TypeWord::Int | TypeWord::I32 => VT::I32,
        TypeWord::Bool => VT::Bool,
        TypeWord::Str => VT::Str,
        TypeWord::StrOwned => VT::StrOwned,
        TypeWord::Char => VT::Char,
        TypeWord::Void => VT::Unit,
        // PLAN-961 numeric surface words.
        TypeWord::I8 => VT::I8,
        TypeWord::U8 | TypeWord::Byte => VT::U8,
        TypeWord::U32 | TypeWord::Uint => VT::U32,
        TypeWord::I64 => VT::I64,
        TypeWord::U64 => VT::U64,
        TypeWord::Float => VT::F32,
        TypeWord::Double | TypeWord::F64 => VT::F64,
    }
}

/// The text-profile builtin method set (PLAN-770 §5.1), bound by receiver
/// type at resolve time. User functions never shadow these (distinct
/// namespaces: methods bind on (receiver VT, name); functions on bare
/// names) and these never shadow user names.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BuiltinMethod {
    Len,
    LenBytes,
    CharAt,
    Slice,
    Clone,
    ClosureClone,
    To,
    ToCodepoint,
    FromCodepoint,
    /// PLAN-961: numeric shift `.shl(n)`/`.shr(n)` — the amount must have
    /// the receiver's exact type (checked in `resolve_method`).
    Shl,
    Shr,
    /// PLAN-961: numeric conversion `.as(T)`/`.to(T)` — target in the
    /// Method node's `to` field.
    NumTo,
}

/// (receiver VT, method name) -> (builtin, param VTs, result VT).
pub fn builtin_method(recv: VT, name: &str) -> Option<(BuiltinMethod, &'static [VT], VT)> {
    use VT::*;
    match (recv, name) {
        (Str, "len") | (StrOwned, "len") => Some((BuiltinMethod::Len, &[], I32)),
        (Str, "len_bytes") | (StrOwned, "len_bytes") => Some((BuiltinMethod::LenBytes, &[], I32)),
        (Str, "char_at") | (StrOwned, "char_at") => Some((BuiltinMethod::CharAt, &[I32], Char)),
        (Str, "slice") | (StrOwned, "slice") => Some((BuiltinMethod::Slice, &[I32, I32], Str)),
        (Str, "clone") | (StrOwned, "clone") => Some((BuiltinMethod::Clone, &[], StrOwned)),
        (Str, "to") | (StrOwned, "to") => Some((BuiltinMethod::To, &[], Str)),
        // PLAN-975 B1 (decode/parse core decision): `c.to(Str)` — a char
        // materialized as a fresh owned 1-scalar Str. The To arm in
        // `resolve_method` carries the target word and enforces the
        // owned-only rule (a computed scalar has no buffer to borrow).
        (Char, "to") => Some((BuiltinMethod::To, &[], Str)),
        (Char, "to_codepoint") => Some((BuiltinMethod::ToCodepoint, &[], I32)),
        (Char, "from_codepoint") => Some((BuiltinMethod::FromCodepoint, &[I32], Char)),
        _ => None,
    }
}

/// Contextual literal typing (PLAN-770): a string literal defaults to the
/// borrowing view `str`; an owning context (`let x Str = ...`, an `Str`
/// formal, an `Str` return) retags it in place — the promotion is a
/// TypeUse fact the adapter emits, never a value conversion at this layer.
fn retag_literal(v: &mut ResolvedExpr, want: VT) {
    if matches!(v.kind, RExprKind::StrLit(_)) && matches!(want, VT::Str | VT::StrOwned) {
        v.vt = want;
    }
}

/// Contextual integer literals fit their annotated type without changing
/// the type of explicitly suffixed literals or ordinary arithmetic.
fn retag_integer_literal(expression: &mut ResolvedExpr, expected: VT) {
    let RExprKind::IntLit(value) = expression.kind else {
        return;
    };
    let bits = match expected {
        VT::I8 => i8::try_from(value).ok().map(|value| value as u8 as u64),
        VT::U8 => u8::try_from(value).ok().map(u64::from),
        VT::U32 | VT::U64 => u64::try_from(value).ok(),
        VT::I64 => Some(value as i64 as u64),
        _ => None,
    };
    if let Some(bits) = bits {
        expression.vt = expected;
        expression.kind = RExprKind::TypedLit { vt: expected, bits };
    }
}

/// Result value type of a surface binary operator (PLAN-767 §5.2):
/// arithmetic produces i32; comparisons, equality and the logical
/// operators produce bool.
pub fn bin_op_vt(op: BinOp) -> VT {
    match op {
        BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Rem => VT::I32,
        // PLAN-961: the bit ops' REAL result type is the (same-type)
        // operand type — Combine recomputes it; this placeholder only
        // covers the impossible numeric-free path.
        BinOp::BitAnd | BinOp::BitOr | BinOp::BitXor => VT::I32,
        BinOp::Lt
        | BinOp::Le
        | BinOp::Gt
        | BinOp::Ge
        | BinOp::Eq
        | BinOp::Ne
        | BinOp::And
        | BinOp::Or => VT::Bool,
        BinOp::NullCoalesce => unreachable!("?? lowers to control flow in resolve"),
    }
}

/// PLAN-961: the source spelling of a type word (diagnostics).
fn word_text(w: TypeWord) -> &'static str {
    match w {
        TypeWord::Int => "int",
        TypeWord::I32 => "i32",
        TypeWord::Bool => "bool",
        TypeWord::Str => "str",
        TypeWord::StrOwned => "Str",
        TypeWord::Char => "char",
        TypeWord::Void => "void",
        TypeWord::I8 => "i8",
        TypeWord::U8 => "u8",
        TypeWord::Byte => "byte",
        TypeWord::Uint => "uint",
        TypeWord::U32 => "u32",
        TypeWord::I64 => "i64",
        TypeWord::U64 => "u64",
        TypeWord::Float => "float",
        TypeWord::Double => "double",
        TypeWord::F64 => "f64",
    }
}

/// PLAN-961: whether a VT is a numeric value type (the operator domain).
pub fn vt_is_numeric(v: VT) -> bool {
    matches!(
        v,
        VT::I32 | VT::I8 | VT::U8 | VT::U32 | VT::I64 | VT::U64 | VT::F32 | VT::F64
    )
}

/// PLAN-961: whether a VT is a FLOAT numeric (no bit/shift ops; negation
/// is a sign flip).
pub fn vt_is_float(v: VT) -> bool {
    matches!(v, VT::F32 | VT::F64)
}

/// PLAN-961: the numeric-alias builtin type of a VT (I32 included; used
/// for the typed op axis).
pub fn vt_numeric_hir(v: VT) -> Option<crate::hir::BuiltinType> {
    Some(match v {
        VT::I32 => crate::hir::BuiltinType::I32,
        VT::I8 => crate::hir::BuiltinType::I8,
        VT::U8 => crate::hir::BuiltinType::U8,
        VT::U32 => crate::hir::BuiltinType::U32,
        VT::I64 => crate::hir::BuiltinType::I64,
        VT::U64 => crate::hir::BuiltinType::U64,
        VT::F32 => crate::hir::BuiltinType::F32,
        VT::F64 => crate::hir::BuiltinType::F64,
        _ => return None,
    })
}

/// Index into [`ResolvedFn::locals`].
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct LocalRef(pub usize);

/// Registration index of a loop within its function (walk order); break and
/// continue carry the innermost enclosing loop's id, or `None` when they sit
/// outside any loop (rejected by the verifier through a phantom target).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct LoopRef(pub usize);

#[derive(Clone, Debug)]
pub struct LocalDecl {
    pub name: String,
    pub vt: VT,
    pub mutable: bool,
    pub is_param: bool,
    pub param_index: Option<u32>,
    /// Capture environment index for a synthesized closure body.
    pub capture_index: Option<u32>,
    pub is_capture: bool,
    /// PLAN-909: this local is CELL-represented (a shared write cell for a
    /// `var` binding some non-move closure writes, or a closure body's
    /// private/shared writable capture). The HIR type stays scalar; the
    /// lowering materializes the single-element List handle and routes
    /// every read/write through it.
    pub cell: bool,
    /// Parameter ownership mode (PLAN-770): Default = view, Mut = exclusive
    /// borrow, Move = transfer in. Bindings always carry Default.
    pub mode: super::ast::ParamMode,
    /// Name through the initializer (let/var) or the whole param phrase.
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct ResolvedArg {
    /// Source argument order (evaluation order, left to right).
    pub value: ResolvedExpr,
    /// Present for named arguments.
    pub name: Option<ast::Ident>,
    /// `x.move` marker (PLAN-770): ownership transfers to the callee.
    pub moved: bool,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub enum RExprKind {
    IntLit(i32),
    /// PLAN-961: typed numeric literal, payload bits in the type's width.
    TypedLit {
        vt: VT,
        bits: u64,
    },
    BoolLit(bool),
    StrLit(String),
    CharLit(u32),
    /// Builtin text/char method bound by receiver type (PLAN-770). The
    /// `to` conversions carry their target in `to` (None for every other
    /// method); arguments were checked against the builtin table at bind.
    Method {
        recv: Box<ResolvedExpr>,
        method: BuiltinMethod,
        /// `Some(target)` only for `BuiltinMethod::To`.
        to: Option<VT>,
        args: Vec<ResolvedArg>,
    },
    /// `x.move` outside/inside argument wiring: wraps the (single) inner
    /// read; only legal as a direct call argument.
    Move {
        inner: Box<ResolvedExpr>,
    },
    /// A bare type-word marker (`.to(str)` argument, `char.from_codepoint`
    /// receiver). Never a value; the typecheck walk rejects strays.
    TypeWord(TypeWord),
    ReadLocal(LocalRef),
    /// PLAN-939 S06: a NAMED top-level function referenced by name as a
    /// foreign call's CALLBACK lane argument — a callback TARGET, never a
    /// first-class function value (the text-record domain has none). Its
    /// type is `VT::Function` of the (params, result)-matching interned
    /// closure type, so structurally it can only satisfy a callback lane
    /// parameter; every other position is a type mismatch.
    CallbackRef {
        fn_index: usize,
    },
    /// PLAN-862: the unit value — the operand of the synthesized return at
    /// the end of a Unit function (and of a fall-off script). It carries no
    /// payload and is never produced by user syntax.
    Unit,
    CallFn {
        fn_index: usize,
        args: Vec<ResolvedArg>,
    },
    CallClosure {
        callee: Box<ResolvedExpr>,
        args: Vec<ResolvedArg>,
    },
    CreateClosure {
        /// PLAN-909 stable lambda identity: the callee is referenced by its
        /// owner-stable name (`__closure_<owner>_<k>`), never by a
        /// module-wide index — inserting/removing an unrelated lambda in
        /// another owner shifts no DefIds here.
        callee: String,
        captures: Vec<(LocalRef, crate::hir::CaptureMode)>,
    },
    CallIntrinsic {
        symbol: &'static str,
        args: Vec<ResolvedExpr>,
    },
    /// PLAN-960: read of a module global (`module.global.get:<sym>`); the
    /// packages binder rewrites bare/qualified const & store reads to this
    /// call form. `vt` of the node is the global's declared type.
    CallGlobalGet {
        global: String,
        take: bool,
    },
    /// PLAN-960: write of a module store (`module.global.set:<sym>(v)`);
    /// a statement-position effect (result Unit).
    CallGlobalSet {
        global: String,
        value: Box<ResolvedExpr>,
    },
    /// PLAN-936（T-04）：`#{ expr }` CT 需求的已绑定形式。inner 是完整
    /// 绑定后的表达式（含 MetaQuery 节点）；HashExpr 的类型 = inner 的
    /// 类型。合法消费：CT 车道以冻结常量结构性替换本节点（T-06/T-07）；
    /// typecheck 走到未服务 HashExpr = 受控拒（`source.cte.unserved`）。
    HashExpr {
        inner: Box<ResolvedExpr>,
    },
    /// PLAN-936（T-04）：meta 查询内在（ct_field_count/ct_has_field）。
    /// type_name/field 是 query selector（数据，不是运行时值）；vt 为
    /// I32（field_count）/Bool（has_field）。只在 HashExpr 子树内可绑定。
    MetaQuery {
        op: &'static str,
        type_name: String,
        field: Option<String>,
    },
    Binary {
        op: BinOp,
        lhs: Box<ResolvedExpr>,
        rhs: Box<ResolvedExpr>,
    },
    Unary {
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
    /// `r.clone()` — builtin deep clone of a record value (PLAN-782;
    /// one allocation per non-empty Str field, copyable fields by value).
    /// Binds by (record receiver, method name); a user-declared `clone`
    /// method on a record is rejected at declaration registration.
    RecordClone {
        base: Box<ResolvedExpr>,
    },
    OwnedClone {
        operand: Box<ResolvedExpr>,
    },
    // -- PLAN-864 collections surface ------------------------------------
    /// `List<T>.new()` — an empty list of the interned list type.
    /// PLAN-962 (L-15): a tuple literal (positional elements; the
    /// adapter materializes the anonymous positional record).
    TupleLit {
        elems: Vec<ResolvedExpr>,
    },
    /// PLAN-962 (L-15): positional element access (destructured reads).
    TupleGet {
        base: Box<ResolvedExpr>,
        index: usize,
    },
    ListNew {
        list: usize,
    },
    /// `xs.len()` — element count (i32).
    ListLen {
        list: Box<ResolvedExpr>,
    },
    /// `xs.push(v)` — append (unit result). Owned elements require a
    /// move-source (checked here).
    ListPush {
        list: Box<ResolvedExpr>,
        value: Box<ResolvedExpr>,
    },
    /// `xs.insert(i, v)` — insert (unit result).
    ListInsert {
        list: Box<ResolvedExpr>,
        index: Box<ResolvedExpr>,
        value: Box<ResolvedExpr>,
    },
    /// `xs.get(i)` — copy-out for Copy element types; owned elements go
    /// through field views (FieldGet over this node) or `remove`.
    ListGet {
        list: Box<ResolvedExpr>,
        index: Box<ResolvedExpr>,
    },
    /// `xs.set(i, v)` — replace (unit result; owned elements require a
    /// move-source).
    ListSet {
        list: Box<ResolvedExpr>,
        index: Box<ResolvedExpr>,
        value: Box<ResolvedExpr>,
    },
    /// `xs.remove(i)` — element transfer/copy out.
    ListRemove {
        list: Box<ResolvedExpr>,
        index: Box<ResolvedExpr>,
    },
    /// `xs.clear()` — unit result; lowered to a remove-from-tail loop.
    ListClear {
        list: Box<ResolvedExpr>,
    },
    /// `xs.get(i).f.g` — a scalar-terminal field projection on an owned
    /// element (ListFieldGet in MIR). Str-terminal projections must ride
    /// [`RExprKind::ElemView`] (under `.len()`).
    ListFieldGet {
        list: Box<ResolvedExpr>,
        index: Box<ResolvedExpr>,
        path: Vec<(usize, usize)>,
    },
    /// A checked read-only element/field view (PLAN-864 contract C4):
    /// `xs.get(i)` (whole Str element) or `xs.get(i).f` (Str-terminal
    /// field path), consumed only by `.len()`. The lowering expands the
    /// view to remove/view/read/reinsert.
    ElemView {
        list: Box<ResolvedExpr>,
        index: Box<ResolvedExpr>,
        path: Vec<(usize, usize)>,
    },
    // -- PLAN-964 object/nil/FStr surface --------------------------------
    /// Object literal `{ k: v, ... }` — keys static, values homogeneous
    /// (`vt` = `VT::Object(oi)`; the elem derives from the interned
    /// object table).
    ObjectCtor {
        keys: Vec<crate::hir::ObjectKey>,
        args: Vec<ResolvedExpr>,
    },
    /// Dynamic index read `obj[k]` — missing key traps at runtime.
    ObjectGet {
        base: Box<ResolvedExpr>,
        key: Box<ResolvedExpr>,
    },
    /// Dynamic index write `obj[k] = v` — upsert.
    ObjectSet {
        base: Box<ResolvedExpr>,
        key: Box<ResolvedExpr>,
        value: Box<ResolvedExpr>,
    },
    /// The nil literal (`nil`/`null`).
    NilLit,
    DynCast {
        operand: Box<ResolvedExpr>,
        test_nil: bool,
    },
    /// I32 → decimal owned Str (FStr interpolation display).
    TextDisplay {
        operand: Box<ResolvedExpr>,
    },
    /// FStr interpolation chain: parts in source order, each expression
    /// evaluated exactly once (the lowering flattens into TextConcat).
    FStr {
        parts: Vec<FStrPart>,
    },
    /// `Some(e)/None/Ok(e)/Err(e)/E.A/E.A(e)` — variant construction.
    /// A transient [`RExprKind::CtorUnknown`] resolves here once the
    /// expected type (or the payload type, for `Some`) is known.
    VariantCtor {
        variant: usize,
        disc: u32,
        payload: Option<Box<ResolvedExpr>>,
    },
    /// The transient bare-constructor form — never escapes resolution
    /// (resolve with an expected type or it is `source.type-ambiguous`).
    CtorUnknown {
        name: String,
        payload: Option<Box<ResolvedExpr>>,
    },
    /// `is` dispatch test (bool): true iff the variant local read holds
    /// arm `disc`. Root must be a whole-local read.
    VariantGuardTest {
        local: LocalRef,
        variant: usize,
        disc: u32,
    },
    /// `is` payload projection — move-out for owned payloads, copy for
    /// Copy payloads.
    VariantProject {
        local: LocalRef,
        variant: usize,
        disc: u32,
    },
}

/// PLAN-964 (L-22): one part of an FStr chain — a literal text segment
/// (already an owned Str expression) or an interpolated expression
/// (already lowered to its display/concat form).
#[derive(Clone, Debug)]
pub enum FStrPart {
    Text(ResolvedExpr),
    Expr(ResolvedExpr),
}

/// PLAN-1026: one piece of a runtime py frame (static text or an i32
/// display — decimal renderings for token/value/length fields).
pub enum PyFramePart {
    Txt(String),
    Disp(ResolvedExpr),
    /// PLAN-1107: dynamic TEXT (a chunk-helper result) embedded verbatim.
    TextExpr(ResolvedExpr),
}

impl FStrPart {
    pub fn vt(&self) -> VT {
        match self {
            FStrPart::Text(e) | FStrPart::Expr(e) => e.vt,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ResolvedExpr {
    pub kind: RExprKind,
    pub span: Span,
    pub vt: VT,
}

/// Operator chains are unbounded (they resolve iteratively), so the
/// derived recursive drop would overflow on deep spines — drop with an
/// explicit work stack (R3-QA-02; same pattern as `ast::Expr`).
impl Drop for ResolvedExpr {
    fn drop(&mut self) {
        fn dummy() -> ResolvedExpr {
            ResolvedExpr {
                span: Span { start: 0, end: 0 },
                vt: VT::I32,
                kind: RExprKind::IntLit(0),
            }
        }
        let detach = |node: &mut ResolvedExpr, stack: &mut Vec<ResolvedExpr>| match &mut node.kind {
            // PLAN-962: tuple literal elements / tuple-get base detach.
            RExprKind::TupleLit { elems } => {
                for e in elems {
                    stack.push(std::mem::replace(e, dummy()));
                }
            }
            RExprKind::TupleGet { base, .. } => {
                stack.push(std::mem::replace(base.as_mut(), dummy()));
            }
            // PLAN-862: leaf, nothing to detach.
            RExprKind::Unit => {}
            // PLAN-936（T-04）：CT 需求/meta 查询在 closures 面不可达（语法
            // 面门控）；叶处理。
            RExprKind::HashExpr { .. } | RExprKind::MetaQuery { .. } => {}
            // PLAN-961: a typed literal is a leaf.
            RExprKind::TypedLit { .. } => {}
            // PLAN-939 S06: a named callback target is a leaf.
            RExprKind::CallbackRef { .. } => {}
            RExprKind::Binary { lhs, rhs, .. } => {
                let l = std::mem::replace(lhs.as_mut(), dummy());
                let r = std::mem::replace(rhs.as_mut(), dummy());
                stack.push(l);
                stack.push(r);
            }
            RExprKind::Unary { operand, .. } => {
                let o = std::mem::replace(operand.as_mut(), dummy());
                stack.push(o);
            }
            // F-771-R1/V-09: the record profile adds DEEP non-operator
            // spines (nested `T { r: T { r: .. } }` ctors, field paths,
            // method/call args). Every boxed or Vec-held child must be
            // detached here or that level falls back to recursive drop
            // glue and a rejected malformed partial overflows the stack.
            RExprKind::FieldGet { base, .. }
            | RExprKind::RecordClone { base }
            | RExprKind::OwnedClone { operand: base } => {
                let b = std::mem::replace(base.as_mut(), dummy());
                stack.push(b);
            }
            RExprKind::CallIntrinsic { args, .. } => {
                stack.extend(args.drain(..));
            }
            RExprKind::CallGlobalGet { .. } => {}
            RExprKind::CallGlobalSet { value, .. } => {
                let v = std::mem::replace(value.as_mut(), dummy());
                stack.push(v);
            }
            RExprKind::CallFn { args, .. } | RExprKind::CtorPos { args, .. } => {
                for a in args.iter_mut() {
                    let v = std::mem::replace(&mut a.value, dummy());
                    stack.push(v);
                }
            }
            RExprKind::CallClosure { callee, args } => {
                let e = std::mem::replace(callee.as_mut(), dummy());
                stack.push(e);
                for a in args.iter_mut() {
                    let v = std::mem::replace(&mut a.value, dummy());
                    stack.push(v);
                }
            }
            RExprKind::CreateClosure { .. } => {}
            RExprKind::CtorNamed { fields, .. } => {
                for (_, v) in fields.iter_mut() {
                    let e = std::mem::replace(v, dummy());
                    stack.push(e);
                }
            }
            RExprKind::MethodCall { recv, args, .. } => {
                if let Some(r) = recv.as_mut() {
                    let e = std::mem::replace(r.as_mut(), dummy());
                    stack.push(e);
                }
                for a in args.iter_mut() {
                    let v = std::mem::replace(&mut a.value, dummy());
                    stack.push(v);
                }
            }
            // PLAN-864: collection/variant nodes hold boxed children.
            RExprKind::ListNew { .. }
            | RExprKind::VariantGuardTest { .. }
            | RExprKind::VariantProject { .. } => {}
            RExprKind::ListLen { list, .. } | RExprKind::ListClear { list } => {
                let e = std::mem::replace(list.as_mut(), dummy());
                stack.push(e);
            }
            RExprKind::ListPush { list, value } => {
                for slot in [list.as_mut(), value.as_mut()] {
                    let e = std::mem::replace(slot, dummy());
                    stack.push(e);
                }
            }
            RExprKind::ListGet { list, index } | RExprKind::ListRemove { list, index } => {
                for slot in [list.as_mut(), index.as_mut()] {
                    let e = std::mem::replace(slot, dummy());
                    stack.push(e);
                }
            }
            RExprKind::ListInsert { list, index, value }
            | RExprKind::ListSet { list, index, value } => {
                for slot in [list.as_mut(), index.as_mut(), value.as_mut()] {
                    let e = std::mem::replace(slot, dummy());
                    stack.push(e);
                }
            }
            RExprKind::ListFieldGet { list, index, .. }
            | RExprKind::ElemView { list, index, .. } => {
                for slot in [list.as_mut(), index.as_mut()] {
                    let e = std::mem::replace(slot, dummy());
                    stack.push(e);
                }
            }
            RExprKind::VariantCtor { payload, .. } | RExprKind::CtorUnknown { payload, .. } => {
                if let Some(pl) = payload.as_mut() {
                    let e = std::mem::replace(pl.as_mut(), dummy());
                    stack.push(e);
                }
            }
            RExprKind::IntLit(_)
            | RExprKind::BoolLit(_)
            | RExprKind::StrLit(_)
            | RExprKind::CharLit(_)
            | RExprKind::TypeWord(_)
            | RExprKind::Method { .. }
            | RExprKind::Move { .. }
            | RExprKind::ReadLocal(_)
            | RExprKind::BorrowMut { .. }
            // PLAN-964: NilLit is a leaf.
            | RExprKind::NilLit => {}
            // PLAN-964: object/dynamic children detach like the collection
            // nodes (boxed children + arg vectors).
            RExprKind::ObjectCtor { args, .. } => {
                for a in args.iter_mut() {
                    let e = std::mem::replace(a, dummy());
                    stack.push(e);
                }
            }
            RExprKind::ObjectGet { base, key } => {
                for slot in [base.as_mut(), key.as_mut()] {
                    let e = std::mem::replace(slot, dummy());
                    stack.push(e);
                }
            }
            RExprKind::ObjectSet { base, key, value } => {
                for slot in [base.as_mut(), key.as_mut(), value.as_mut()] {
                    let e = std::mem::replace(slot, dummy());
                    stack.push(e);
                }
            }
            RExprKind::TextDisplay { operand } | RExprKind::DynCast { operand, .. } => {
                let e = std::mem::replace(operand.as_mut(), dummy());
                stack.push(e);
            }
            RExprKind::FStr { parts } => {
                for p in parts.iter_mut() {
                    match p {
                        FStrPart::Text(e) | FStrPart::Expr(e) => {
                            let e = std::mem::replace(e, dummy());
                            stack.push(e);
                        }
                    }
                }
            }
        };
        let mut stack: Vec<ResolvedExpr> = Vec::new();
        detach(self, &mut stack);
        while let Some(mut node) = stack.pop() {
            detach(&mut node, &mut stack);
        }
    }
}

#[derive(Clone, Debug)]
pub enum RStmt {
    Let {
        local: LocalRef,
        value: ResolvedExpr,
        span: Span,
    },
    Assign {
        local: LocalRef,
        /// PLAN-771: field path rooted at the local — (record index, field
        /// declaration index) per segment; empty = whole-local assign.
        path: Vec<(usize, usize)>,
        value: ResolvedExpr,
        span: Span,
    },
    Return {
        value: ResolvedExpr,
        span: Span,
    },
    If {
        cond: ResolvedExpr,
        then: RBlock,
        els: Option<RBlock>,
        span: Span,
    },
    While {
        cond: ResolvedExpr,
        body: RBlock,
        span: Span,
        target: LoopRef,
    },
    Loop {
        body: RBlock,
        span: Span,
        target: LoopRef,
        unwind: Option<(LocalRef, LocalRef)>,
    },
    Break {
        target: Option<LoopRef>,
        span: Span,
    },
    Continue {
        target: Option<LoopRef>,
        span: Span,
    },
    /// PLAN-862: an expression statement. Only legal as the SCRIPT's
    /// trailing statement (the tail value); every other occurrence is
    /// rejected by [`check_entry_matrix`] with `source.entry-tail`.
    ExprStmt {
        value: ResolvedExpr,
        span: Span,
    },
    /// End a named text view when execution leaves its lexical scope.
    EndView {
        local: LocalRef,
        span: Span,
    },
    /// PLAN-965: language-level error unwind (`fail(msg)` / uncaught `.?`).
    Fail {
        value: ResolvedExpr,
        span: Span,
    },
}

fn rblock_always_returns(block: &RBlock) -> bool {
    block.stmts.iter().any(|stmt| match stmt {
        RStmt::Return { .. } | RStmt::Fail { .. } => true,
        RStmt::If {
            then,
            els: Some(els),
            ..
        } => rblock_always_returns(then) && rblock_always_returns(els),
        _ => false,
    })
}

/// PLAN-1036 (F28/F29): may control reach the END of `block`? The counted
/// for-in appends its latch there; when the body's last statement already
/// terminates (`break`/`continue`/`return`/`fail`) the appended latch
/// would be unreachable code, so it is omitted and the `continue` sites
/// carry the latch instead. Conservative: only a definitively terminating
/// last statement reports `false`.
fn rblock_can_fall_through(block: &RBlock) -> bool {
    match block.stmts.last() {
        None => true,
        Some(RStmt::Break { .. })
        | Some(RStmt::Continue { .. })
        | Some(RStmt::Return { .. })
        | Some(RStmt::Fail { .. }) => false,
        Some(RStmt::If {
            then,
            els: Some(els),
            ..
        }) => rblock_can_fall_through(then) || rblock_can_fall_through(els),
        Some(_) => true,
    }
}

#[derive(Clone, Debug)]
pub struct RBlock {
    pub stmts: Vec<RStmt>,
    pub span: Span,
}

/// Resolved block trees nest arbitrarily deep (R6: unlimited block
/// nesting), so the derived recursive drop would overflow — drop with an
/// explicit work stack (same pattern as `ResolvedExpr`/`ast::Block`).
impl Drop for RBlock {
    fn drop(&mut self) {
        fn dummy() -> RBlock {
            RBlock {
                stmts: Vec::new(),
                span: Span { start: 0, end: 0 },
            }
        }
        let detach = |b: &mut RBlock, stack: &mut Vec<RBlock>| {
            for s in b.stmts.iter_mut() {
                match s {
                    RStmt::If { then, els, .. } => {
                        stack.push(std::mem::replace(then, dummy()));
                        if let Some(e) = els {
                            stack.push(std::mem::replace(e, dummy()));
                        }
                    }
                    RStmt::While { body, .. } | RStmt::Loop { body, .. } => {
                        stack.push(std::mem::replace(body, dummy()));
                    }
                    _ => {}
                }
            }
        };
        let mut stack: Vec<RBlock> = Vec::new();
        detach(self, &mut stack);
        while let Some(mut b) = stack.pop() {
            detach(&mut b, &mut stack);
        }
    }
}

#[derive(Clone)]
pub struct ResolvedFn {
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
    /// PLAN-1083: owning variant index for ENUM instance methods (at most
    /// one of owner_rec/owner_var is Some).
    pub owner_var: Option<usize>,
}

impl ResolvedFn {
    /// PLAN-881: the body-work meter key — owner-qualified for methods so
    /// a bare method name can never alias a same-named free function in
    /// the observation window (the structural gate keys on this).
    pub fn work_key(&self, records: &[ResolvedRecord]) -> String {
        match self.owner_rec {
            None => self.name.clone(),
            Some(ri) => format!(
                "{}::{}",
                records.get(ri).map(|r| r.name.as_str()).unwrap_or("?"),
                self.name
            ),
        }
    }

    /// PLAN-1083: the enum owner name (instance methods on variants).
    pub fn owner_var_name(&self, variants: &[ResolvedVariant]) -> Option<String> {
        self.owner_var
            .map(|vi| variants.get(vi).map(|v| v.name.clone()).unwrap_or_default())
    }
}

impl ResolvedProgram {
    pub fn foreign_by_slot(
        &self,
        slot: usize,
    ) -> Option<&ace_foreign_contract::manual::SourceDecl> {
        if self.foreign_slots.is_empty() {
            self.foreign.get(slot)
        } else {
            self.foreign_slots
                .get(slot)
                .and_then(|name| self.foreign.iter().find(|d| &d.name == name))
        }
    }
}

pub struct ResolvedProgram {
    /// Stable process-local foreign call slots. Empty means the cold declaration
    /// order. Removed names remain tombstones; additions append without touching
    /// any non-reader body. Restore has no resolved bodies and starts fresh slots.
    pub foreign_slots: Vec<String>,
    pub foreign: Vec<ace_foreign_contract::manual::SourceDecl>,
    pub fns: Vec<ResolvedFn>,
    /// Nominal record types in declaration order (PLAN-771). `VT::Rec(i)`
    /// indexes here.
    pub records: Vec<ResolvedRecord>,
    /// PLAN-864: List element types (structural, deduplicated).
    /// `VT::List(i)` indexes here.
    pub lists: Vec<VT>,
    /// PLAN-962 (L-15): structural tuple shapes (deduplicated);
    /// `VT::Tuple(i)` indexes here.
    pub tuples: Vec<Vec<VT>>,
    /// PLAN-864: nominal variant types (`?T`/`Result<T,E>`/declared
    /// `enum`s). `VT::Variant(i)` indexes here.
    pub variants: Vec<ResolvedVariant>,
    /// PLAN-865: closure function signatures referenced by `VT::Function`.
    pub function_types: Vec<ClosureFnType>,
    /// PLAN-964 (L-16/L-20): object element types (structural,
    /// deduplicated). `VT::Object(i)` indexes here.
    pub objects: Vec<VT>,
    /// PLAN-968: the scheduler task type table (scheduler face only;
    /// `VT::Task(i)`/`VT::Future(i)` index here).
    pub sched_tasks: Vec<SchedTaskType>,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ClosureFnType {
    pub params: Vec<VT>,
    pub result: VT,
}

/// One resolved variant type (PLAN-864): name plus arms in declaration
/// order; each arm carries at most one payload type.
#[derive(Clone, Debug)]
pub struct ResolvedVariant {
    pub name: String,
    pub name_span: Span,
    pub arms: Vec<(String, Option<VT>)>,
    /// PLAN-1083: instance methods declared inside the enum body (the
    /// implicit receiver is the variant value; `is self` dispatches on it).
    pub methods: Vec<ResolvedMethod>,
}

/// The mutable type tables grown while bodies resolve (PLAN-864): `List<T>`
/// annotations and variant constructors intern entries on first use.
/// PLAN-968: one scheduler task's resolved type facts (indexed by
/// `VT::Task(i)` / `VT::Future(i)`).
#[derive(Clone, Debug)]
pub struct SchedTaskType {
    /// User-facing task name.
    pub name: String,
    /// Synthesized driver function name (`__task_<Name>`).
    pub driver: String,
    /// State field value types in declaration order (spawn initializers).
    pub fields_vt: Vec<VT>,
    /// The unified message value type.
    pub msg: VT,
    /// The reply value type (i32 when the task never replies).
    pub reply: VT,
}

#[derive(Default)]
pub struct CollTables {
    ordinary_auto: bool,
    legacy_text_records: BTreeSet<usize>,
    age_package: bool,
    legacy_age_widgets: Vec<Span>,
    age_widget_numeric_casts: std::cell::Cell<bool>,
    script_errors: bool,
    /// PLAN-968: the scheduler task type table (scheduler face only).
    pub sched_tasks: Vec<SchedTaskType>,
    task_messages: Vec<VT>,
    pub lists: Vec<VT>,
    pub variants: Vec<ResolvedVariant>,
    pub function_types: Vec<ClosureFnType>,
    variant_names: BTreeMap<String, usize>,
    spec_variants: BTreeSet<usize>,
    enum_instances: BTreeMap<usize, ast::EnumInstance>,
    record_instances: BTreeMap<String, ast::RecordInstance>,
    enum_type_templates: BTreeMap<String, Vec<String>>,
    enum_type_requests: Vec<instances::AutoEnumDemand>,
    /// PLAN-962: aggregates-profile admission — nested List, composite
    /// record/variant field and payload positions, and the wide numeric
    /// element kinds are legal. The frozen 822 one-layer element domain
    /// stays in force for every older profile.
    pub aggregates_admission: bool,
    /// PLAN-105 T-02（U03 真语料 013）：AGE 组件面子集开关——元素视图
    /// （`xs[i].f` / `xs.get(i).f`）放行；其余域保持冻结语义。
    pub elem_view_admission: bool,
    /// PLAN-105 T-02（U03 真语料 013）：AGE 组件面的**整局部隐式转移**——组件
    /// 状态列表（`var todos []Todo = []`）由组件自身拥有，`push(t)` 视为所有者
    /// 转移（作者面不写 `.move`；其它 profile 仍要求显式 `.move`）。
    pub implicit_push_transfer: bool,
    /// PLAN-962 (L-15): structural tuple shapes (element VT lists,
    /// interned/deduplicated; `VT::Tuple(i)` indexes here).
    pub tuples: Vec<Vec<VT>>,
    /// PLAN-964 (L-16/L-20): object element types (structural,
    /// deduplicated; `VT::Object(i)` indexes here).
    pub objects: Vec<VT>,

    /// PLAN-1083: scalar `= N` enums — enum name to (arm, value) pairs;
    /// `Color.Red` resolves as the plain int constant.
    pub scalar_enums: BTreeMap<String, Vec<(String, i32)>>,
    /// PLAN-1083: (variant, disc) arms whose payload is a NAMED-FIELD
    /// record (`Arm { f T, .. }`) — a single binding destructures the one
    /// field (an anonymous record-typed payload keeps the whole-value
    /// binding, e.g. `Msg.Move(p)`).
    pub named_field_arms: BTreeSet<(usize, u32)>,
    scoped_iter_fns: BTreeSet<usize>,
}

impl CollTables {
    fn legacy_age_widget_at(&self, span: Span) -> bool {
        let index = self
            .legacy_age_widgets
            .partition_point(|widget| widget.start <= span.start);
        index > 0 && span.end <= self.legacy_age_widgets[index - 1].end
    }

    fn age_widget_numeric_cast(&self, value: ResolvedExpr, to: VT) -> ResolvedExpr {
        self.age_widget_numeric_casts.set(true);
        ResolvedExpr {
            span: value.span,
            vt: to,
            kind: RExprKind::Method {
                recv: Box::new(value),
                method: BuiltinMethod::NumTo,
                to: Some(to),
                args: Vec::new(),
            },
        }
    }

    fn intern_task_message(&mut self, message: VT) -> usize {
        if let Some(index) = self.task_messages.iter().position(|ty| *ty == message) {
            return index;
        }
        let index = self.task_messages.len();
        self.task_messages.push(message);
        index
    }

    fn intern_function(&mut self, signature: ClosureFnType) -> usize {
        if let Some(i) = self.function_types.iter().position(|s| *s == signature) {
            return i;
        }
        self.function_types.push(signature);
        self.function_types.len() - 1
    }

    /// PLAN-962 (L-15): intern one tuple shape (deduplicated).
    fn intern_tuple(&mut self, vts: Vec<VT>) -> usize {
        if let Some(i) = self.tuples.iter().position(|t| *t == vts) {
            return i;
        }
        self.tuples.push(vts);
        self.tuples.len() - 1
    }

    fn intern_list(&mut self, elem: VT) -> usize {
        if let Some(i) = self.lists.iter().position(|e| *e == elem) {
            return i;
        }
        self.lists.push(elem);
        self.lists.len() - 1
    }

    /// PLAN-964: intern one object element type (deduplicated).
    fn intern_object(&mut self, elem: VT) -> usize {
        if let Some(i) = self.objects.iter().position(|e| *e == elem) {
            return i;
        }
        self.objects.push(elem);
        self.objects.len() - 1
    }

    fn intern_variant(
        &mut self,
        name: &str,
        name_span: Span,
        arms: Vec<(String, Option<VT>)>,
    ) -> usize {
        if matches!(name, "Option" | "Result") {
            if let Some(i) = self
                .variants
                .iter()
                .position(|v| v.name == name && v.arms == arms)
            {
                return i;
            }
            let i = self.variants.len();
            self.variants.push(ResolvedVariant {
                name: name.to_string(),
                name_span,
                methods: Vec::new(),
                arms,
            });
            return i;
        }
        if let Some(&i) = self.variant_names.get(name) {
            return i;
        }
        let i = self.variants.len();
        self.variant_names.insert(name.to_string(), i);
        self.variants.push(ResolvedVariant {
            name: name.to_string(),
            name_span,
            arms,
            methods: Vec::new(),
        });
        i
    }

    fn variant_by_name(&self, name: &str) -> Option<usize> {
        self.variant_names.get(name).copied()
    }

    /// Rebuild the name index after restoring a saved table (recheck path).
    pub(crate) fn reindex_variant(&mut self, name: &str, idx: usize) {
        self.variant_names.insert(name.to_string(), idx);
    }
}

/// PLAN-1035 (F34): resource reachability over the record/tuple tables —
/// a record "owns" when its field closure reaches an owned `Str`, a `List`
/// buffer, a `Variant` payload, a runtime closure (`Function`), an object
/// cell or another (tuple) aggregate carrying one. This replaces the
/// PLAN-782 Str-only walk for the collections owned-classification
/// (PLAN-864): a List-only record (`type Bag { items List<int> }`) is a
/// resource record, exactly like one holding `Str`.
pub(crate) fn record_reaches_resource(
    ri: usize,
    records: &[ResolvedRecord],
    tuples: &[Vec<VT>],
) -> bool {
    let mut stack: Vec<usize> = vec![ri];
    let mut seen: BTreeSet<usize> = BTreeSet::new();
    while let Some(r) = stack.pop() {
        if !seen.insert(r) {
            continue;
        }
        if records[r]
            .fields
            .iter()
            .any(|f| vt_reaches_resource(f.vt, records, tuples, &mut seen))
        {
            return true;
        }
    }
    false
}

/// One value type's resource-edge test shared by the record/tuple closure
/// walk: leaf resources return true, aggregates recurse.
fn vt_reaches_resource(
    vt: VT,
    records: &[ResolvedRecord],
    tuples: &[Vec<VT>],
    seen: &mut BTreeSet<usize>,
) -> bool {
    match vt {
        VT::StrOwned | VT::List(_) | VT::Variant(_) | VT::Function(_) | VT::Object(_) => true,
        VT::Rec(inner) => record_reaches_resource_inner(inner, records, tuples, seen),
        VT::Tuple(ti) => tuples.get(ti).is_some_and(|elems| {
            elems
                .iter()
                .any(|&e| vt_reaches_resource(e, records, tuples, seen))
        }),
        _ => false,
    }
}

/// The record half of the closure walk, sharing the caller's `seen` set so
/// nested aggregates do not re-walk (the cycle gate already rejected direct
/// record recursion; this guard keeps the walk bounded regardless).
fn record_reaches_resource_inner(
    ri: usize,
    records: &[ResolvedRecord],
    tuples: &[Vec<VT>],
    seen: &mut BTreeSet<usize>,
) -> bool {
    if !seen.insert(ri) {
        return false;
    }
    records[ri]
        .fields
        .iter()
        .any(|f| vt_reaches_resource(f.vt, records, tuples, seen))
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
    /// A static method has no receiver; instance methods use params[0].
    pub is_static: bool,
    pub is_mut: bool,
    /// Index into `ResolvedProgram::fns`.
    pub fn_index: usize,
    pub span: Span,
}

/// Parameter position the argument at `idx` binds to: positional arguments
/// bind by source order, named arguments by formal name. Shared by the type
/// checker and the Atom adapter's binding map.
pub fn param_position(idx: usize, arg: &ResolvedArg, callee: &ResolvedFn) -> Option<usize> {
    match &arg.name {
        None => Some(idx),
        Some(name) => callee.param_names.iter().position(|p| p == &name.name),
    }
}

pub(crate) struct FnSig {
    param_names: Vec<String>,
    params: Vec<VT>,
    /// PLAN-963（T-08，L-24）：与 `param_names` 逐位对齐的声明位默认值
    /// （`self` 位与外来回调位恒 `None`）。只存 AST 形状；调用位缺实参时
    /// 才在**调用者**作用域解析一次，故不进入任何语义身份/缓存键。
    defaults: Vec<Option<Box<ast::Expr>>>,
    result: VT,
    index: usize,
}

impl Clone for FnSig {
    fn clone(&self) -> Self {
        FnSig {
            param_names: self.param_names.clone(),
            params: self.params.clone(),
            defaults: self.defaults.clone(),
            result: self.result,
            index: self.index,
        }
    }
}

impl FnSig {
    /// PLAN-913 T-03: the portable declaration view of this signature.
    pub(crate) fn dump(&self) -> (Vec<String>, Vec<VT>, VT, usize) {
        (
            self.param_names.clone(),
            self.params.clone(),
            self.result,
            self.index,
        )
    }
}

/// PLAN-913 T-03: re-form a signature from untrusted portable data.
pub(crate) fn fn_sig_from_parts(
    param_names: Vec<String>,
    params: Vec<VT>,
    result: VT,
    index: usize,
) -> FnSig {
    let default_slots = param_names.len();
    FnSig {
        param_names,
        params,
        defaults: vec![None; default_slots],
        result,
        index,
    }
}

pub fn resolve(
    program: &ast::Program,
    capabilities: &[String],
) -> Result<ResolvedProgram, Vec<Diagnostic>> {
    resolve_with(program, capabilities, SourceProfile::Core)
}

/// Resolve with an explicit profile surface (PLAN-767): identical name
/// resolution on both surfaces — the profile only gated which operators
/// the parser accepted; value types are computed per operator here.
/// PLAN-909 §5.5: the owner-granular frontend cache of one closures
/// module — per top-level function, its item-text fingerprint plus the
/// resolved entry and its lambdas. A later apply whose DECLARATION CONTEXT
/// fingerprint matches and whose owner text is byte-identical reuses the
/// entry wholesale (no resolve walk); anything else re-resolves that owner.
#[derive(Clone)]
pub struct ClosuresOwnerCache {
    /// Fingerprint of every item's declaration region (fn signatures,
    /// type/other items) — the module-level fact a cached body's resolve
    /// depends on outside its own text.
    pub context_fp: u64,
    pub owners: BTreeMap<String, CachedOwner>,
    /// The module's closure-signature table as of this cache: body-driven
    /// (lambdas intern during their owner's resolve) and referenced BY
    /// INDEX from cached entries (`VT::Function(i)`). A warm apply SEEDS
    /// the fresh table with this snapshot before resolving, so interning
    /// (value-dedup, append-only) can never shift a cached index.
    pub function_types: Vec<ClosureFnType>,
}

#[derive(Clone)]
pub struct CachedOwner {
    pub text_fp: u64,
    pub resolved: ResolvedFn,
    /// The owner's lambdas in creation order (stable owner-bound names).
    pub lambdas: Vec<ResolvedFn>,
    /// PLAN-912 §5.2: the creation-site facts of each lambda in `lambdas`
    /// (same order) — the checked interface context a lambda body can be
    /// re-resolved against WITHOUT walking its owner or its siblings.
    pub sites: Vec<LambdaSiteFacts>,
}

/// PLAN-912 §5.2: everything a lambda's CREATION SITE fixes for its body.
/// Minted by the resolver while it resolves the owner; retained by the
/// closures function frontend; consumed by [`resolve_lambda_in_context`]
/// to re-resolve ONE lambda body alone. The capture declaration here is the
/// fast-path oracle: a re-resolve whose raw capture uses / params / move
/// flag differ from these facts is the explicit non-fast path (the owner's
/// creation site changed), never a silent widen of the body's authority.
#[derive(Clone, Debug, PartialEq)]
pub struct LambdaSiteFacts {
    /// Enclosing top-level function (the owner).
    pub owner: String,
    /// Per-owner creation ordinal `k` of `__closure_<owner>_<k>` (depth-first).
    pub ordinal: usize,
    /// The synthetic display name (wire identity `d_<name>`).
    pub name: String,
    /// The whole lambda expression span in the source the owner was
    /// resolved from (coordinate catalog; remapped by the frontend's M axis).
    pub expr_span: Span,
    pub move_capture: bool,
    /// Declared parameters `(name, type)` in order.
    pub params: Vec<(String, VT)>,
    /// Raw capture uses `(name, written)` in first-use order — the pre-mode
    /// fact the creation site derived modes from.
    pub capture_uses: Vec<(String, bool)>,
    /// Derived capture declarations `(name, body type, mode, writable, cell)`
    /// in environment order — the body's Capture locals.
    pub capture_decls: Vec<(String, VT, crate::hir::CaptureMode, bool, bool)>,
    /// The owner locals the creation site captures (CreateClosure operands),
    /// same order as `capture_decls`.
    pub capture_sources: Vec<LocalRef>,
    /// Every local name visible at the creation point (the set
    /// `analyze_capture_uses` ran against).
    pub outer_names: BTreeSet<String>,
    /// Inferred result type and the interned closure signature index.
    pub result: VT,
    pub type_index: usize,
    /// How many lambdas were created INSIDE this body (ordinals
    /// `ordinal+1 ..= ordinal+nested`).
    pub nested: usize,
}

/// PLAN-912: why an isolated lambda re-resolve declined the fast path.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LambdaNonFast {
    /// Raw capture uses (names / order / written) differ from the site.
    CaptureUsesChanged,
    /// Parameter list or `move` flag differ (closure signature changed).
    SignatureChanged,
    /// The new body creates nested lambdas, or the retained one did.
    NestedLambdas,
    /// The inferred result type differs (closure signature changed).
    ResultChanged,
}

/// The error side of [`resolve_lambda_in_context`]: either a NAMED
/// non-fast decline or the body's own source diagnostics. Success is the
/// returned [`BodyState`].
pub enum LambdaReResolve {
    /// Declined: the owner must re-resolve (bounded non-fast path).
    NonFast(LambdaNonFast),
    /// Source diagnostics from the body itself.
    Diagnostics(Vec<Diagnostic>),
}

pub(crate) struct WarmIo<'a> {
    pub src: &'a str,
    pub prev: Option<&'a ClosuresOwnerCache>,
    pub context_fp: u64,
    pub out: ClosuresOwnerCache,
    pub reused: BTreeSet<String>,
}

fn fnv64(bytes: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in bytes {
        h ^= *b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_001b_3);
    }
    h
}

/// The module's declaration-context fingerprint: every item's signature
/// region (`fn name(params) -> ret`, everything before the body brace) or,
/// for non-fn items, the whole item text. Length-framed pieces — no
/// concatenation ambiguity.
pub(crate) fn decl_context_fp(program: &ast::Program, src: &str) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    let mix = |piece: &str, h: &mut u64| {
        *h ^= piece.len() as u64;
        *h = h.wrapping_mul(0x0000_0100_0000_001b_3);
        for b in piece.bytes() {
            *h ^= b as u64;
            *h = h.wrapping_mul(0x0000_0100_0000_001b_3);
        }
    };
    for item in &program.items {
        match item {
            ast::Item::Fn(f) => mix(&src[f.span.start..f.body.span.start], &mut h),
            ast::Item::Type(t) => mix(&src[t.span.start..t.span.end], &mut h),
            ast::Item::Enum(e) => mix(&src[e.span.start..e.span.end], &mut h),
            // PLAN-968: task declarations hash by their full declaration
            // region (fields/hooks/arms all live inside the span).
            ast::Item::Task(t) => mix(&src[t.span.start..t.span.end], &mut h),
            // PLAN-960: module globals never reach the single-module closures
            // cache (the packages binder lowers them to init functions); their
            // declaration text is still part of the context identity.
            ast::Item::Const(c) => mix(&src[c.span.start..c.span.end], &mut h),
            ast::Item::Store(v) => mix(&src[v.span.start..v.span.end], &mut h),
            // PLAN-967 F0: the frontdoor spelling participates in the
            // module content identity (provider identity is load-bearing).
            ast::Item::UseRs(u) => mix(&src[u.span.start..u.span.end], &mut h),
            // PLAN-1026: the py frontdoor spelling participates in the
            // module content identity (module/member lists are load-bearing).
            ast::Item::UsePy(u) => mix(&src[u.span.start..u.span.end], &mut h),
        }
    }
    h
}

/// PLAN-909 §5.5 warm closures resolve: the whole module resolves cold
/// unless a previous cache is supplied; with one, owners whose text and
/// the declaration context are unchanged are reused without a walk.
/// Returns the resolved program, the NEW cache (for the next apply) and
/// the set of reused owner names (the incremental report).
pub fn resolve_closures_cached(
    program: &ast::Program,
    src: &str,
    capabilities: &[String],
    prev: Option<&ClosuresOwnerCache>,
) -> Result<(ResolvedProgram, ClosuresOwnerCache, BTreeSet<String>), Vec<Diagnostic>> {
    let context_fp = decl_context_fp(program, src);
    let mut warm = WarmIo {
        src,
        prev,
        context_fp,
        out: ClosuresOwnerCache {
            context_fp,
            owners: BTreeMap::new(),
            function_types: Vec::new(),
        },
        reused: BTreeSet::new(),
    };
    let (fns, records, coll, _) = resolve_detailed(
        program,
        capabilities,
        SourceProfile::Closures,
        Some(&mut warm),
        None,
        None,
    )?;
    Ok((
        ResolvedProgram {
            foreign: Vec::new(),
            foreign_slots: Vec::new(),
            fns,
            records,
            lists: coll.lists,
            tuples: Vec::new(),
            variants: coll.variants,
            function_types: coll.function_types,
            objects: coll.objects,
            sched_tasks: coll.sched_tasks,
        },
        warm.out,
        warm.reused,
    ))
}

/// PLAN-912 §5.2: cold closures resolve that RETAINS the checked environment
/// (the function frontend's resident program: owners then lambdas, the
/// signature/record indexes and the closure-signature table) AND returns the
/// owner cache whose entries carry each lambda's creation-site facts.
/// [`resolve_closures_cached`] stays byte-identical for the PLAN-909 path.
pub(crate) fn resolve_closures_env(
    program: &ast::Program,
    src: &str,
    capabilities: &[String],
) -> Result<(CheckedEnv, ClosuresOwnerCache), Vec<Diagnostic>> {
    let context_fp = decl_context_fp(program, src);
    let mut warm = WarmIo {
        src,
        prev: None,
        context_fp,
        out: ClosuresOwnerCache {
            context_fp,
            owners: BTreeMap::new(),
            function_types: Vec::new(),
        },
        reused: BTreeSet::new(),
    };
    let (fns, records, coll, ix) = resolve_detailed(
        program,
        capabilities,
        SourceProfile::Closures,
        Some(&mut warm),
        None,
        None,
    )?;
    Ok((
        CheckedEnv {
            program: ResolvedProgram {
                foreign: Vec::new(),
                foreign_slots: Vec::new(),
                fns,
                records,
                lists: coll.lists,
                tuples: Vec::new(),
                variants: coll.variants,
                function_types: coll.function_types,
                objects: coll.objects,
                sched_tasks: coll.sched_tasks,
            },
            rec_index: ix.rec_index,
            sigs: ix.sigs,
            trace_enabled: ix.trace_enabled,
            profile: SourceProfile::Closures,
            asm_lane: false,
            resolve_walks: 0,
            check_walks: 0,
            id: next_env_id(),
            generation: 0,
            closed: false,
        },
        warm.out,
    ))
}

pub fn resolve_with(
    program: &ast::Program,
    capabilities: &[String],
    surface: SourceProfile,
) -> Result<ResolvedProgram, Vec<Diagnostic>> {
    resolve_with_source_mode(program, capabilities, surface, false)
}

pub(crate) fn resolve_with_source_mode(
    program: &ast::Program,
    capabilities: &[String],
    surface: SourceProfile,
    script_errors: bool,
) -> Result<ResolvedProgram, Vec<Diagnostic>> {
    resolve_source_program(program, capabilities, surface, script_errors, None)
        .map(|(program, _)| program)
}

pub(crate) fn resolve_age_package(
    program: &ast::Program,
    capabilities: &[String],
    surface: SourceProfile,
    legacy_widgets: &[Span],
) -> Result<(ResolvedProgram, bool), Vec<Diagnostic>> {
    resolve_source_program(program, capabilities, surface, false, Some(legacy_widgets))
}

fn resolve_source_program(
    program: &ast::Program,
    capabilities: &[String],
    surface: SourceProfile,
    script_errors: bool,
    legacy_widgets: Option<&[Span]>,
) -> Result<(ResolvedProgram, bool), Vec<Diagnostic>> {
    let (fns, records, coll, _) = resolve_detailed_in_mode(
        program,
        capabilities,
        surface,
        None,
        None,
        None,
        script_errors,
        None,
        false,
        legacy_widgets,
    )?;
    let numeric_casts = coll.age_widget_numeric_casts.get();
    Ok((
        ResolvedProgram {
            foreign: Vec::new(),
            foreign_slots: Vec::new(),
            fns,
            records,
            lists: coll.lists,
            tuples: coll.tuples,
            variants: coll.variants,
            function_types: coll.function_types,
            objects: coll.objects,
            sched_tasks: coll.sched_tasks,
        },
        numeric_casts,
    ))
}

/// PLAN-105 T-02（U03 真语料 013）：带**元素视图**开关的解析入口。
pub fn resolve_with_view(
    program: &ast::Program,
    capabilities: &[String],
    surface: SourceProfile,
    elem_view: bool,
) -> Result<ResolvedProgram, Vec<Diagnostic>> {
    resolve_with_view_mode(program, capabilities, surface, elem_view, false)
}

pub(crate) fn resolve_with_view_mode(
    program: &ast::Program,
    capabilities: &[String],
    surface: SourceProfile,
    elem_view: bool,
    script_errors: bool,
) -> Result<ResolvedProgram, Vec<Diagnostic>> {
    let (fns, records, coll, _) = resolve_detailed_in_mode(
        program,
        capabilities,
        surface,
        None,
        None,
        None,
        script_errors,
        None,
        elem_view,
        None,
    )?;
    Ok(ResolvedProgram {
        foreign: Vec::new(),
        foreign_slots: Vec::new(),
        fns,
        records,
        lists: coll.lists,
        tuples: coll.tuples,
        variants: coll.variants,
        function_types: coll.function_types,
        objects: coll.objects,
        sched_tasks: coll.sched_tasks,
    })
}
/// PLAN-968: resolve a desugared scheduler-surface program with its task
/// side-table (`VT::Task(i)`/`VT::Future(i)` interning, `__sched_*`
/// intrinsic typing). Every other face keeps [`resolve_with`].
pub fn resolve_with_sched(
    program: &ast::Program,
    capabilities: &[String],
    surface: SourceProfile,
    sched: &super::sched_desugar::SchedSurface,
) -> Result<ResolvedProgram, Vec<Diagnostic>> {
    let (fns, records, coll, _) =
        resolve_detailed(program, capabilities, surface, None, None, Some(sched))?;
    Ok(ResolvedProgram {
        foreign: Vec::new(),
        foreign_slots: Vec::new(),
        fns,
        records,
        lists: coll.lists,
        tuples: coll.tuples,
        variants: coll.variants,
        function_types: coll.function_types,
        objects: coll.objects,
        sched_tasks: coll.sched_tasks,
    })
}

/// PLAN-934 T-04: the type facts a fact-driven bind installs.
///
/// The supply is keyed by the CURRENT source map (span), but spans are not
/// the fact's identity: the query layer derives this view from a page that
/// was admitted against the body's STRUCTURAL walk first (body version,
/// node ids, kinds, coverage). Core treats every returned type as
/// authoritative and its own rule as the expectation to CHECK — a missing
/// or mismatching fact is a bind-stage rejection.
pub trait TypeSupply {
    fn foreign(&self) -> Option<&ace_foreign_contract::manual::AdmittedForeign> {
        None
    }
    /// PLAN-939: a supply may carry ONLY the admitted foreign environment
    /// (the text-record resource lane has no solver facts — the 934
    /// inference surface is scalar-only). Such a supply grafts foreign
    /// declarations but leaves body typing to the resolver's own rules.
    fn facts_enabled(&self) -> bool {
        true
    }
    /// Type of the expression node at this source span.
    fn expr_ty(&self, span: Span) -> Option<VT>;
    /// Type of the `let`/`var` declaration whose statement span is given.
    fn binding_ty(&self, span: Span) -> Option<VT>;
    /// Every admitted fact carried by this span (PLAN-934 T-08).
    ///
    /// The core parser gives a unary expression the span of its operand, so
    /// one span can carry a parent and a child fact. Core still treats the
    /// facts as the authority: it installs the fact that equals this node's
    /// rule and reports `source.fact-mismatch` when NO fact of the span
    /// matches. The default keeps single-fact suppliers unchanged.
    fn expr_tys(&self, span: Span) -> Vec<VT> {
        self.expr_ty(span).into_iter().collect()
    }
}

/// PLAN-934 T-04: bind with the query's facts as the type authority.
///
/// Every expression and `let`/`var` type of the finite scalar surface is
/// taken from `supply`; the resolver's own typing rules become checks
/// (mismatch/missing fact = `source.fact-mismatch` / `source.fact-missing`
/// at stage `bind`). No typecheck walk and no post-hoc overwrite runs.
/// PLAN-939: the TEXT-RECORD full-resolve lane for resource foreign
/// sources. The supply contributes the admitted foreign environment ONLY
/// — no solver facts exist for this surface (the 934 inference surface is
/// scalar-only), so every body resolves with its own full walk. The
/// scalar lane's fact-supplied shortcut is untouched.

/// PLAN-1026: a foreign-only supply (the py-bridge lane grafts the
/// adapter's declared ops into the AGGREGATES surface; body typing stays
/// the resolver's own full walk — `facts_enabled() == false`).
pub struct ForeignOnlySupply<'a> {
    foreign: &'a ace_foreign_contract::manual::AdmittedForeign,
}
impl<'a> ForeignOnlySupply<'a> {
    pub fn new(foreign: &'a ace_foreign_contract::manual::AdmittedForeign) -> Self {
        Self { foreign }
    }
}
impl TypeSupply for ForeignOnlySupply<'_> {
    fn foreign(&self) -> Option<&ace_foreign_contract::manual::AdmittedForeign> {
        Some(self.foreign)
    }
    fn facts_enabled(&self) -> bool {
        false
    }
    fn expr_ty(&self, _span: Span) -> Option<VT> {
        None
    }
    fn binding_ty(&self, _span: Span) -> Option<VT> {
        None
    }
}

/// PLAN-1026: the AGGREGATES foreign lane — the py-bridge training
/// surface (aggregates numerics/text/collections + the adapter's
/// declared frame ops, grafted through the same supply lane the
/// text-record F0 path uses). Body typing is the resolver's own walk.
pub fn resolve_aggregates_with_foreign(
    program: &ast::Program,
    capabilities: &[String],
    foreign: &ace_foreign_contract::manual::AdmittedForeign,
) -> Result<ResolvedProgram, Vec<Diagnostic>> {
    resolve_with_foreign_source(
        program,
        capabilities,
        foreign,
        SourceProfile::Aggregates,
        false,
    )
}

pub(crate) fn resolve_with_foreign_source(
    program: &ast::Program,
    capabilities: &[String],
    foreign: &ace_foreign_contract::manual::AdmittedForeign,
    profile: SourceProfile,
    script_errors: bool,
) -> Result<ResolvedProgram, Vec<Diagnostic>> {
    let supply = ForeignOnlySupply::new(foreign);
    let (fns, records, coll, _) = resolve_detailed_in_mode(
        program,
        capabilities,
        profile,
        None,
        Some(&supply),
        None,
        script_errors,
        None,
        false,
        None,
    )?;
    Ok(ResolvedProgram {
        foreign_slots: Vec::new(),
        foreign: foreign.source_declarations(),
        fns,
        records,
        lists: coll.lists,
        tuples: coll.tuples,
        variants: coll.variants,
        function_types: coll.function_types,
        objects: coll.objects,
        sched_tasks: coll.sched_tasks,
    })
}

pub fn resolve_text_record_with_foreign(
    program: &ast::Program,
    capabilities: &[String],
    surface: SourceProfile,
    supply: &dyn TypeSupply,
) -> Result<ResolvedProgram, Vec<Diagnostic>> {
    if surface != SourceProfile::TextRecord {
        return Err(vec![Diagnostic::new(
            Stage::Bind,
            "source.fact-profile",
            format!(
                "the resource foreign lane lowers the text-record surface only, not `{}`",
                surface.as_str()
            ),
            Span { start: 0, end: 0 },
        )]);
    }
    let (fns, records, coll, _) =
        resolve_detailed(program, capabilities, surface, None, Some(supply), None)?;
    Ok(ResolvedProgram {
        foreign_slots: Vec::new(),
        foreign: supply
            .foreign()
            .map(|f| f.source_declarations())
            .unwrap_or_default(),
        fns,
        records,
        lists: coll.lists,
        tuples: Vec::new(),
        variants: coll.variants,
        function_types: coll.function_types,
        objects: coll.objects,
        sched_tasks: coll.sched_tasks,
    })
}

pub fn resolve_with_supply(
    program: &ast::Program,
    capabilities: &[String],
    surface: SourceProfile,
    supply: &dyn TypeSupply,
) -> Result<ResolvedProgram, Vec<Diagnostic>> {
    // PLAN-999 (T-03): the fact-supplied binding admits the CORE surface
    // alongside the scalar family — the same fact producer/walk covers its
    // statement set (`while`/`loop`), and the product entry verifies the
    // emitted bundle through the Core-domain verifier. Every other
    // surface (record/text domain binding) still rejects here.
    if !(surface.is_scalar() || matches!(surface, SourceProfile::Core)) {
        return Err(vec![Diagnostic::new(
            Stage::Bind,
            "source.fact-profile",
            format!(
                "fact-supplied binding supports the core and scalar surfaces only, not `{}`",
                surface.as_str()
            ),
            Span { start: 0, end: 0 },
        )]);
    }
    let (fns, records, coll, _) =
        resolve_detailed(program, capabilities, surface, None, Some(supply), None)?;
    Ok(ResolvedProgram {
        foreign_slots: Vec::new(),
        foreign: supply
            .foreign()
            .map(|f| f.source_declarations())
            .unwrap_or_default(),
        fns,
        records,
        lists: coll.lists,
        tuples: Vec::new(),
        variants: coll.variants,
        function_types: coll.function_types,
        objects: coll.objects,
        sched_tasks: coll.sched_tasks,
    })
}

/// PLAN-828 D-01: the retained checked-environment indexes of one cold
/// resolve — what a later single-body re-check consults.
struct DeclIndexes {
    rec_index: BTreeMap<String, usize>,
    sigs: BTreeMap<String, FnSig>,
    trace_enabled: bool,
}

/// Reject cyclic payload graphs without recursing through user type depth.
/// Edges include synthesized Option/Result payloads, not only declared enums.
fn reject_variant_cycles(coll: &CollTables, diags: &mut Vec<Diagnostic>) {
    let count = coll.variants.len();
    let mut pending = vec![0usize; count];
    let mut parents = vec![Vec::new(); count];
    for (index, variant) in coll.variants.iter().enumerate() {
        for (_, payload) in &variant.arms {
            if let Some(VT::Variant(child)) = payload {
                pending[index] += 1;
                parents[*child].push(index);
            }
        }
    }
    let mut ready: Vec<_> = pending
        .iter()
        .enumerate()
        .filter_map(|(i, degree)| (*degree == 0).then_some(i))
        .collect();
    while let Some(child) = ready.pop() {
        for &parent in &parents[child] {
            pending[parent] -= 1;
            if pending[parent] == 0 {
                ready.push(parent);
            }
        }
    }
    if let Some(index) = pending.iter().position(|degree| *degree != 0) {
        let variant = &coll.variants[index];
        diags.push(Diagnostic::new(
            Stage::Bind, "source.type-mismatch",
            format!("variant `{}` has a recursive payload graph; this profile requires acyclic variants", variant.name),
            variant.name_span,
        ));
    }
}

/// The declaration passes + per-function resolve shared by the
/// whole-module entry ([`resolve_with`]) and the retained-environment
/// cold entry ([`resolve_env`]).
/// Declaration-only foreign admission, shared by the fact-query entry and
/// core binding. It never walks a body and keeps the source name's location.
pub fn validate_foreign_context(
    program: &ast::Program,
    foreign: &ace_foreign_contract::manual::AdmittedForeign,
    capabilities: &[String],
) -> Result<(), Vec<Diagnostic>> {
    let names: Vec<_> = program
        .items
        .iter()
        .map(|item| match item {
            ast::Item::Fn(f) => (f.name.name.as_str(), f.name.span, true),
            ast::Item::Type(t) => (t.name.name.as_str(), t.name.span, false),
            ast::Item::Enum(e) => (e.name.name.as_str(), e.name.span, false),
            // PLAN-968: task names participate in duplicate-name detection
            // (not callable directly — spawned by string name).
            ast::Item::Task(t) => (t.name.name.as_str(), t.name.span, false),
            // PLAN-960: module globals are callable-surface declarations for
            // the foreign directory (name/spans only; accessors are not fns).
            ast::Item::Const(c) => (c.name.name.as_str(), c.name.span, false),
            ast::Item::Store(v) => (v.name.name.as_str(), v.name.span, false),
            // PLAN-967 F0: the import frontdoor contributes no callable
            // name; it is validated against the admitted provider below.
            ast::Item::UseRs(_) => ("", Span { start: 0, end: 0 }, false),
            // PLAN-1026: py imports contribute no callable name here; the
            // member bindings are hoisted script locals (validated below).
            ast::Item::UsePy(_) => ("", Span { start: 0, end: 0 }, false),
        })
        .collect();
    // The use.rs frontdoor: at most one, and its provider id must be the
    // session's admitted manifest provider (the import cannot invent or
    // widen a provider the host did not admit).
    let mut use_rs_seen: Option<Span> = None;
    for item in &program.items {
        if let ast::Item::UseRs(u) = item {
            if let Some(first) = use_rs_seen {
                return Err(vec![Diagnostic::new(
                    Stage::Bind,
                    "source.use-rs.duplicate",
                    "at most one `use.rs` import is allowed per module",
                    first.join(u.span),
                )]);
            }
            let family_prefix = if u.lang == "c" { "c/" } else { "rust/" };
            if u.provider.name != foreign.manifest().provider
                || !foreign.manifest().facts.logical.starts_with(family_prefix)
            {
                return Err(vec![Diagnostic::new(
                    Stage::Bind,
                    "source.use-rs.provider",
                    format!(
                        "`use.{} {}` does not match the admitted provider `{}` (family `{}`)",
                        u.lang,
                        u.provider.name,
                        foreign.manifest().provider,
                        foreign.manifest().facts.logical
                    ),
                    u.provider.span,
                )]);
            }
            use_rs_seen = Some(u.span);
        }
    }
    // PLAN-1026: the use.py frontdoor — the admitted environment must be
    // the py adapter itself (provider id `py`, the frozen front-door
    // identity); members bind as script locals, so duplicates and name
    // conflicts with source declarations are named rejections.
    let mut py_members: std::collections::BTreeMap<&str, Span> = std::collections::BTreeMap::new();
    for item in &program.items {
        if let ast::Item::UsePy(u) = item {
            if foreign.manifest().provider != "py" {
                return Err(vec![Diagnostic::new(
                    Stage::Bind,
                    "source.use-py.provider",
                    format!(
                        "`use.py` requires the admitted py adapter (provider `py`), not `{}`",
                        foreign.manifest().provider
                    ),
                    u.span,
                )]);
            }
            for m in &u.members {
                if let Some(first) = py_members.get(m.name.as_str()) {
                    return Err(vec![Diagnostic::new(
                        Stage::Bind,
                        "source.use-py.duplicate",
                        format!("duplicate py member import `{}`", m.name),
                        first.join(m.span),
                    )]);
                }
                if let Some((_, span, _)) = names.iter().find(|(n, _, _)| *n == m.name.as_str()) {
                    return Err(vec![Diagnostic::new(
                        Stage::Bind,
                        "source.use-py.conflict",
                        format!("py member `{}` conflicts with a source declaration", m.name),
                        *span,
                    )]);
                }
                py_members.insert(m.name.as_str(), m.span);
            }
        }
    }
    validate_foreign_declarations(&names, foreign, capabilities)
}
/// Same admission for a restored declaration directory: name/span metadata is
/// sufficient; no placeholder local function or body projection is needed.
pub fn validate_foreign_declarations(
    names: &[(&str, Span, bool)],
    foreign: &ace_foreign_contract::manual::AdmittedForeign,
    capabilities: &[String],
) -> Result<(), Vec<Diagnostic>> {
    if !capabilities
        .iter()
        .any(|c| c == ace_foreign_contract::manual::SOURCE_CAP)
        || foreign
            .manifest()
            .facts
            .capabilities
            .iter()
            .any(|c| !capabilities.contains(c))
    {
        let span = names
            .first()
            .map(|(_, span, _)| *span)
            .unwrap_or(Span { start: 0, end: 0 });
        return Err(vec![Diagnostic::new(
            Stage::Bind,
            "source.foreign-capability",
            "foreign.scalar capability is required",
            span,
        )]);
    }
    let declarations = foreign.source_declarations_ref();
    let mut ds = Vec::new();
    for (name, span, function) in names {
        if *function && declarations.iter().any(|d| d.name == *name) {
            ds.push(Diagnostic::new(
                Stage::Bind,
                "source.foreign-conflict",
                format!("foreign declaration `{name}` conflicts with a source function"),
                *span,
            ));
        }
    }
    if ds.is_empty() {
        Ok(())
    } else {
        Err(ds)
    }
}

/// PLAN-981：该块里是否出现 AGE 资源服务调用（`material2d.color(...)` 等）。
///
/// 判据与 resolver 的分类**同一条件**（`MethodCall{recv: Var(ns), name}` 且
/// `ns.op` 命中声明表），不另立规则。
fn uses_age_service(b: &ast::Block) -> bool {
    fn is_service(e: &ast::Expr) -> bool {
        if let ast::ExprKind::MethodCall { recv, name, .. } = &e.kind {
            if let ast::ExprKind::Var(id) = &recv.kind {
                let dotted = format!("{}.{}", id.name, name.name);
                return crate::source::age_resources::lookup(&dotted).is_some();
            }
        }
        false
    }
    fn block(b: &ast::Block) -> bool {
        b.stmts.iter().any(|s| match s {
            ast::Stmt::Let { value, .. } => is_service(value),
            ast::Stmt::Assign { value, .. } => is_service(value),
            ast::Stmt::Return { value, .. } => is_service(value),
            ast::Stmt::ExprStmt { value, .. } => is_service(value),
            ast::Stmt::If {
                cond, then, els, ..
            } => is_service(cond) || block(then) || els.as_ref().is_some_and(|e| block(e)),
            ast::Stmt::Loop { body, .. } => block(body),
            _ => false,
        })
    }
    block(b)
}

/// PLAN-981：该类型注解是否点名了 AGE typed 端口（`Material2D`/`Mesh2D`）。
///
/// 端口可能**只**出现在注解里（g01 `shapes.at` 的 `tint: Material2D` 就没有任何
/// 服务调用），因此登记判据必须包含注解位，不能只看服务使用。
fn ann_mentions_age_port(ann: &ast::TypeAnn) -> bool {
    match &ann.kind {
        ast::TypeRef::Record(id) => crate::source::age_resources::is_port_type(&id.name),
        _ => false,
    }
}

/// PLAN-1082: rebuild the READ-side AST of a place prefix (`base` plus the
/// first `n` steps) so the ordinary expression resolver produces the chain
/// (field reads / nested index reads) with all of its existing rules.
fn place_read_ast(base: &ast::Ident, steps: &[ast::PlaceStep]) -> Expr {
    let span = base.span;
    let mut e = Expr {
        span,
        kind: ExprKind::Var(base.clone()),
    };
    for step in steps {
        e = match step {
            ast::PlaceStep::Field(name) => Expr {
                span: name.span,
                kind: ExprKind::Field {
                    base: Box::new(e),
                    name: name.clone(),
                },
            },
            ast::PlaceStep::Index(idx) => Expr {
                span: idx.span,
                kind: ExprKind::Index {
                    base: Box::new(e),
                    index: Box::new(idx.clone()),
                },
            },
        };
    }
    e
}

/// PLAN-1082: the arithmetic of a compound place assignment.
fn place_bin_op(op: ast::PlaceOp) -> BinOp {
    match op {
        ast::PlaceOp::Add => BinOp::Add,
        ast::PlaceOp::Sub => BinOp::Sub,
        ast::PlaceOp::Mul => BinOp::Mul,
        ast::PlaceOp::Div => BinOp::Div,
        ast::PlaceOp::Set => unreachable!("Set is not a compound operator"),
    }

}

fn resolve_detailed(
    program: &ast::Program,
    capabilities: &[String],
    surface: SourceProfile,
    warm: Option<&mut WarmIo<'_>>,
    supply: Option<&dyn TypeSupply>,
    sched: Option<&super::sched_desugar::SchedSurface>,
) -> Result<
    (
        Vec<ResolvedFn>,
        Vec<ResolvedRecord>,
        CollTables,
        DeclIndexes,
    ),
    Vec<Diagnostic>,
> {
    resolve_detailed_in_mode(program, capabilities, surface, warm, supply, sched, false, None, false, None)
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn resolve_detailed_in_mode(

    program: &ast::Program,
    capabilities: &[String],
    surface: SourceProfile,
    mut warm: Option<&mut WarmIo<'_>>,
    supply: Option<&dyn TypeSupply>,
    sched: Option<&super::sched_desugar::SchedSurface>,
    script_errors: bool,
    instance_probe: Option<&instances::InstanceProbe<'_>>,
    elem_view: bool,
    legacy_widgets: Option<&[Span]>,
) -> Result<
    (
        Vec<ResolvedFn>,
        Vec<ResolvedRecord>,
        CollTables,
        DeclIndexes,
    ),
    Vec<Diagnostic>,
> {
    if let Some(foreign) = supply.and_then(|s| s.foreign()) {
        validate_foreign_context(program, foreign, capabilities)?;
    }
    let empty_schedule = super::sched_desugar::SchedSurface::default();
    let desugared = if surface.is_auto_language()
        && program
            .items
            .iter()
            .any(|item| matches!(item, ast::Item::Task(_)))
    {
        Some(super::sched_desugar::desugar_tasks(program, true)?)
    } else {
        None
    };
    let (program, sched) = match &desugared {
        Some((program, schedule)) => (program, Some(schedule)),
        None => (
            program,
            sched.or_else(|| surface.is_auto_language().then_some(&empty_schedule)),
        ),
    };
    let trace_enabled = capabilities.iter().any(|c| c == TRACE_CAPABILITY);
    // PLAN-936（T-04）：meta 查询与 CT 表达式能力（各自独立门；meta 默认关）。
    let meta_enabled = capabilities.iter().any(|c| c == META_CAPABILITY);
    let ct_enabled = capabilities.iter().any(|c| c == super::cst::CT_CAPABILITY);
    // PLAN-972（T-02）：装配 aggregates 面（packages-aggregates-ct lane）——
    // `av.aggregates/1` 准入 + `ac.source.cte.in/v1` 装配能力同时在手时，
    // 闭包字面量按 972 域放行（HOF 公共算法）。
    let asm_lane = surface.is_aggregates()
        && capabilities
            .iter()
            .any(|c| c == super::packages::IN_CAPABILITY);
    let mut diags = Vec::new();
    let mut coll = CollTables::default();
    coll.script_errors = script_errors;
    coll.age_package = legacy_widgets.is_some();
    coll.legacy_age_widgets = legacy_widgets.unwrap_or_default().to_vec();
    if let Some(probe) = instance_probe {
        probe.register_enum_types(&mut coll);
    }
    // PLAN-962: the aggregates surface widens the collections admission.
    coll.aggregates_admission = surface.is_aggregates();
    // AGE 组件面（`compile_program_parts_age`）声明的能力：元素视图放行。
    coll.elem_view_admission = elem_view;
    coll.implicit_push_transfer = elem_view;
    coll.ordinary_auto = surface.is_auto_language();
    let is_coll = surface.is_collections();

    // Pass 0 (PLAN-771): record types pre-register FIRST — field types may
    // reference records declared later (forward references are legal; the
    // containment graph closes across the whole module).
    let mut records: Vec<ResolvedRecord> = Vec::new();
    let mut rec_index: BTreeMap<String, usize> = BTreeMap::new();
    for item in &program.items {
        if let ast::Item::Type(t) = item {
            if rec_index.contains_key(&t.name.name) {
                diags.push(Diagnostic::new(
                    Stage::Bind,
                    "source.duplicate",
                    format!("type `{}` is declared more than once", t.name.name),
                    t.name.span,
                ));
                continue;
            }
            if coll.ordinary_auto
                && t.fields
                    .iter()
                    .any(|field| matches!(field.ty.kind, ast::TypeRef::Builtin(TypeWord::Str)))
            {
                coll.legacy_text_records.insert(records.len());
            }
            rec_index.insert(t.name.name.clone(), records.len());
            records.push(ResolvedRecord {
                name: t.name.name.clone(),
                name_span: t.name.span,
                fields: Vec::new(),
                methods: Vec::new(),
            });
        }
    }
    // PLAN-981（IF-981-ORIGINAL）：AGE typed 端口（`Material2D`/`Mesh2D`）登记为
    // **无字段名义记录**——身份即端口（两种端口互不相等），资源语义仍归领域
    // owner 解释；这里不把资源压平成带颜色字段的普通 record。
    // 包前门用 TextRecord 做领域解析（面在 profile 之上），所以 profile 判据在这里
    // 永远不成立——按**是否使用 AGE 资源服务**登记。
    // 端口类型只要**可能被提到**就必须在场：AGE 作者面的 node 参数不落在
    // `program.items` 的 fn 上（它们在领域行里），故仅凭"程序是否用到服务/注解"
    // 会漏掉 `node Disc(…, tint: Material2D)` 这种合法源。AGE 前门本来就把
    // 整份文档交给本函数，所以对它一律登记——这也是与发射器同一判据。
    let uses_age_ports = surface == SourceProfile::AgePackage
        || program.items.iter().any(|it| match it {
            ast::Item::Fn(f) => {
                uses_age_service(&f.body)
                    || f.params.iter().any(|p| ann_mentions_age_port(&p.ty))
                    || ann_mentions_age_port(&f.ret)
            }
            ast::Item::Type(t) => t.fields.iter().any(|f| ann_mentions_age_port(&f.ty)),
            _ => false,
        });
    if surface == SourceProfile::AgePackage || uses_age_ports {
        for port in crate::source::age_resources::AGE_PORT_TYPES {
            if !rec_index.contains_key(*port) {
                rec_index.insert(port.to_string(), records.len());
                let fields = crate::source::age_resources::AGE_PORT_FIELDS
                    .iter()
                    .find(|(p, _)| *p == *port)
                    .map(|(_, fs)| {
                        fs.iter()
                            .map(|(n, ty)| ResolvedField {
                                name: n.to_string(),
                                // 未知拼写**不得**静默回落 i32：那样 `Str` 会被
                                // 当成整数，端口字段类型就悄悄错了（实测发生过）。
                                vt: match *ty {
                                    "f32" => VT::F32,
                                    "i32" => VT::I32,
                                    "bool" => VT::Bool,
                                    "Str" => VT::StrOwned,
                                    "str" => VT::Str,
                                    // PLAN-981（IF-981-GFX-AV-SEAM）：端口引用字段
                                    //（Frame2D.image: Image2D）——被引端口先注册
                                    // （AGE_PORT_TYPES 声明序保证），按名义记录 id。
                                    other => match rec_index.get(other) {
                                        Some(&ri) => VT::Rec(ri),
                                        None => panic!(
                                            "AGE port field `{n}` declares unsupported type `{other}`"
                                        ),
                                    },
                                },
                                span: Span { start: 0, end: 0 },
                            })
                            .collect()
                    })
                    .unwrap_or_default();
                records.push(ResolvedRecord {
                    name: port.to_string(),
                    name_span: Span { start: 0, end: 0 },
                    fields,
                    methods: Vec::new(),
                });
            }
        }
    }
    // PLAN-889: resolve nominal variant references through the same type
    // table in every annotation position. Register names before payloads,
    // so acyclic forward references do not depend on declaration order.
    // The completed graph is checked below before any body is resolved.
    let mut enum_indices = BTreeMap::new();
    // PLAN-1083: enum name → interned variant index (instance-method
    // registration); scalar `= N` enums never intern.
    let mut enum_variant_index: BTreeMap<String, usize> = BTreeMap::new();
    // PLAN-1083: scalar `= N` enums resolve as i32 constants (the enum
    // name maps each arm to its declared value; `Color.Red` IS the int).
    // Bare-only enums keep the PLAN-864 variant identity unchanged.
    if is_coll {
        for (item_index, item) in program.items.iter().enumerate() {
            if let ast::Item::Enum(e) = item {
                if e.arms.iter().any(|a| a.disc.is_some()) {
                    let mut next = 0i32;
                    let mut arms = Vec::new();
                    let mut mixed = false;
                    for a in &e.arms {
                        if a.disc.is_some() {
                            next = a.disc.unwrap_or(0);
                        } else if a.payload.is_some() || !a.fields.is_empty() {
                            mixed = true;
                        }
                        arms.push((a.name.name.clone(), next));
                        next = next.wrapping_add(1);
                    }
                    if mixed {
                        diags.push(Diagnostic::new(
                            Stage::Bind,
                            "source.enum-mixed-arms",
                            format!(
                                "enum `{}` mixes `= N` scalar arms with payload arms",
                                e.name.name
                            ),
                            e.name.span,
                        ));
                        continue;
                    }
                    coll.scalar_enums.insert(e.name.name.clone(), arms);
                    continue;
                }
                if rec_index.contains_key(&e.name.name)
                    || coll.variant_by_name(&e.name.name).is_some()
                {
                    diags.push(Diagnostic::new(
                        Stage::Bind,
                        "source.duplicate",
                        format!("type `{}` is declared more than once", e.name.name),
                        e.name.span,
                    ));
                    continue;
                }
                let index = coll.intern_variant(&e.name.name, e.name.span, Vec::new());
                enum_indices.insert(item_index, index);
                enum_variant_index.insert(e.name.name.clone(), index);
                if e.spec_witnesses {
                    coll.spec_variants.insert(index);
                }
                if let Some(instance) = &e.instance {
                    coll.enum_instances.insert(index, instance.clone());
                }
            }
        }
    }
    // Fill field shapes (all record names are known now).
    for item in &program.items {
        if let ast::Item::Type(t) = item {
            let Some(&ri) = rec_index.get(&t.name.name) else {
                continue;
            };
            if let Some(instance) = &t.instance {
                coll.record_instances.insert(t.name.name.clone(), instance.clone());
            }
            let mut seen: BTreeSet<&str> = BTreeSet::new();
            for f in &t.fields {
                if !seen.insert(f.name.name.as_str()) {
                    diags.push(Diagnostic::new(
                        Stage::Bind,
                        "source.duplicate",
                        format!(
                            "field `{}` is declared more than once in type `{}`",
                            f.name.name, t.name.name
                        ),
                        f.name.span,
                    ));
                }
                let vt = if is_coll {
                    vt_of_ann_coll(&f.ty, &rec_index, &mut coll, &mut diags, TypePos::Field)
                } else {
                    vt_of_ann(&f.ty, &rec_index, &mut diags)
                };
                records[ri].fields.push(ResolvedField {
                    name: f.name.name.clone(),
                    vt,
                    span: f.name.span,
                });
            }
        }
    }

    // Fill the already registered nominal variant payloads.
    // Collections `type` blocks declare fields only (no methods).
    if is_coll {
        for (item_index, item) in program.items.iter().enumerate() {
            match item {
                ast::Item::Enum(e) => {
                    let Some(&enum_index) = enum_indices.get(&item_index) else {
                        continue;
                    };
                    let mut seen: BTreeSet<&str> = BTreeSet::new();
                    let mut arms: Vec<(String, Option<VT>)> = Vec::new();
                    let mut ok = true;
                    for a in &e.arms {
                        if !seen.insert(a.name.name.as_str()) {
                            diags.push(Diagnostic::new(
                                Stage::Bind,
                                "source.pattern-duplicate",
                                format!(
                                    "arm `{}` is declared more than once in enum `{}`",
                                    a.name.name, e.name.name
                                ),
                                a.name.span,
                            ));
                            ok = false;
                            continue;
                        }
                        // PLAN-1083: a named-field payload `Arm { f T, .. }`
                        // interns a synthesized record (no user ident can
                        // spell `::`), so the payload rides the ordinary
                        // record construction/field-read machinery.
                        if !a.fields.is_empty() {
                            let synth = format!("__enum_{}_{}", e.name.name, a.name.name);
                            let ri = records.len();
                            records.push(ResolvedRecord {
                                name: synth.clone(),
                                name_span: a.name.span,
                                fields: Vec::new(),
                                methods: Vec::new(),
                            });
                            rec_index.insert(synth, ri);
                            let mut fseen: BTreeSet<&str> = BTreeSet::new();
                            for (fname, fty) in &a.fields {
                                if !fseen.insert(fname.name.as_str()) {
                                    diags.push(Diagnostic::new(
                                        Stage::Bind,
                                        "source.duplicate",
                                        format!(
                                            "field `{}` is declared more than once in enum arm `{}.{}`",
                                            fname.name, e.name.name, a.name.name
                                        ),
                                        fname.span,
                                    ));
                                    ok = false;
                                }
                                let vt = vt_of_ann_coll(
                                    fty,
                                    &rec_index,
                                    &mut coll,
                                    &mut diags,
                                    TypePos::Field,
                                );
                                records[ri].fields.push(ResolvedField {
                                    name: fname.name.clone(),
                                    vt,
                                    span: fname.span,
                                });
                            }
                            coll.named_field_arms.insert((enum_index, arms.len() as u32));
                            arms.push((a.name.name.clone(), Some(VT::Rec(ri))));
                            continue;
                        }
                        let payload = a.payload.as_ref().map(|pt| {
                            vt_of_ann_coll(
                                pt,
                                &rec_index,
                                &mut coll,
                                &mut diags,
                                TypePos::ContainerPayload,
                            )
                        });
                        if let Some(VT::List(_)) = payload {
                            if !coll.aggregates_admission {
                                diags.push(Diagnostic::new(
                                    Stage::Bind,
                                    "source.list-value-context",
                                    "List-typed payloads are not part of the 822 element domain"
                                        .to_string(),
                                    a.span,
                                ));
                                ok = false;
                                continue;
                            }
                        }
                        if let Some(pt) = &a.payload {
                            if matches!(pt.kind, ast::TypeRef::Builtin(TypeWord::Void)) {
                                diags.push(Diagnostic::new(
                                    Stage::Bind,
                                    "source.type-mismatch",
                                    "an enum arm payload cannot be void".to_string(),
                                    pt.span,
                                ));
                                ok = false;
                                continue;
                            }
                        }
                        arms.push((a.name.name.clone(), payload));
                    }
                    let _ = ok;
                    coll.variants[enum_index].arms = arms;
                }
                ast::Item::Type(t) => {
                    // PLAN-1080 (D-02): the Auto language face admits record
                    // methods through the same shared method machinery the
                    // record domain uses; the frozen narrow collections-
                    // family profiles keep the historical rejection.
                    if !t.methods.is_empty() && !surface.is_auto_language() {
                        for m in &t.methods {
                            diags.push(Diagnostic::new(
                                Stage::Bind,
                                "source.unsupported",
                                format!(
                                    "record methods are not part of this source profile (method `{}` on `{}`)",
                                    m.name.name, t.name.name
                                ),
                                m.name.span,
                            ));
                        }
                    }
                }
                _ => {}
            }
        }
        reject_variant_cycles(&coll, &mut diags);
        // A cycle must not reach ownership/type construction or emission.
        if !diags.is_empty() {
            if let Some(probe) = instance_probe {
                probe.collect_enum_annotations(&mut coll);
            }
            return Err(diags);
        }
    }

    // Pass 1: signature pre-registration with duplicate detection.
    let mut sigs: BTreeMap<String, FnSig> = BTreeMap::new();
    let mut fn_receiver: Vec<ast::Receiver> = Vec::new();
    let mut fn_owner: Vec<Option<usize>> = Vec::new();
    // PLAN-1083: the variant owner (enum instance methods) — parallel to
    // fn_owner (record index); at most one of the two is Some.
    let mut fn_vowner: Vec<Option<usize>> = Vec::new();
    let mut flat: Vec<&ast::FnItem> = Vec::new();
    for item in &program.items {
        match item {
            ast::Item::Fn(f) => {
                fn_receiver.push(f.receiver);
                fn_owner.push(None);
                fn_vowner.push(None);
                flat.push(f);
            }
            // PLAN-967 F0: the frontdoor declares no function.
            ast::Item::UseRs(_) => {}
            // PLAN-1026: the py frontdoor declares no function (members
            // lower to hoisted py_import calls in the script body).
            ast::Item::UsePy(_) => {}
            ast::Item::Type(t) => {
                for m in &t.methods {
                    fn_receiver.push(m.receiver);
                    fn_owner.push(rec_index.get(&t.name.name).copied());
                    fn_vowner.push(None);
                    flat.push(m);
                }
            }
            // PLAN-1083: auto-face enums carry instance methods (the
            // implicit receiver is the variant value). The frozen profiles
            // keep the PLAN-864 rejection below through the surface gate.
            ast::Item::Enum(e) => {
                if !e.methods.is_empty() {
                    if !surface.is_auto_language() {
                        for m in &e.methods {
                            diags.push(Diagnostic::new(
                                Stage::Bind,
                                "source.unsupported",
                                format!(
                                    "enum methods are not part of this source profile (method `{}` on `{}`)",
                                    m.name.name, e.name.name
                                ),
                                m.name.span,
                            ));
                        }
                    } else if let Some(&vi) = enum_variant_index.get(&e.name.name) {
                        for m in &e.methods {
                            fn_receiver.push(m.receiver);
                            fn_owner.push(None);
                            fn_vowner.push(Some(vi));
                            flat.push(m);
                        }
                    } else {
                        for m in &e.methods {
                            diags.push(Diagnostic::new(
                                Stage::Bind,
                                "source.unsupported",
                                format!(
                                    "enum `{}` declares no variant arms here; methods need the variant surface",
                                    e.name.name
                                ),
                                m.name.span,
                            ));
                        }
                    }
                }
            }
            // PLAN-960: const/store items are consumed by the packages binder
            // (globals + synthesized init functions) before resolve; a raw
            // package unit handed to the single-module resolver is a caller
            // error, never a silent skip.
            // PLAN-960: module globals (const/store) register their accessor
            // pseudo-signatures under the `module.global.get/set:<sym>` names
            // the packages binder emits (names no identifier can spell, so
            // they never shadow user functions). Their initializers are NOT
            // resolved here — the binder moved them into the per-module init
            // function. `index == usize::MAX` marks a non-function entry.
            ast::Item::Const(ast::ConstItem { name, ty, .. })
            | ast::Item::Store(ast::StoreItem { name, ty, .. }) => {
                let vt = if surface.is_auto_language() {
                    vt_of_ann_coll(ty, &rec_index, &mut coll, &mut diags, TypePos::Local)
                } else {
                    vt_of_ann(ty, &rec_index, &mut diags)
                };
                if surface.is_auto_language() && matches!(item, ast::Item::Store(_)) {
                    sigs.insert(
                        format!("{}{}", super::packages::GLOBAL_TAKE_PREFIX, name.name),
                        FnSig {
                            param_names: Vec::new(),
                            params: Vec::new(),
                            defaults: Vec::new(),
                            result: vt,
                            index: usize::MAX,
                        },
                    );
                }
                sigs.insert(
                    format!("module.global.get:{}", name.name),
                    FnSig {
                        param_names: Vec::new(),
                        params: Vec::new(),
                        defaults: Vec::new(),
                        result: vt,
                        index: usize::MAX,
                    },
                );
                sigs.insert(
                    format!("module.global.set:{}", name.name),
                    FnSig {
                        param_names: vec!["value".into()],
                        params: vec![vt],
                        defaults: vec![None],
                        // the write yields 0 so a `let` binds it in every domain
                        result: VT::I32,
                        index: usize::MAX,
                    },
                );
            }
            // PLAN-968: task declarations register their own synthesized
            // functions during resolve; the hook fns are not free functions
            // and never flatten here.
            ast::Item::Task(_) => {}
        }
    }
    // PLAN-1081 (T-03) Pass 1a: collect NESTED fn items declared inside
    // free-function bodies (Auto face). Every item is hoisted into the flat
    // table under an owner-mangled name (`__nested_<owner>_<name>`, one path
    // segment per level); visibility is fn-level and hoisted (Rust item
    // semantics), captured by `nested_children` below. Statement blocks are
    // walked recursively; lambda interiors are NOT collected (a nested item
    // reaching resolve uncollected is a named rejection — never silent).
    let mut fn_nested: Vec<Option<NestedFnInfo>> = vec![None; flat.len()];
    let mut nested_children: BTreeMap<String, Vec<(String, String)>> = BTreeMap::new();
    let mut nested_decl_starts: BTreeSet<usize> = BTreeSet::new();
    if surface == SourceProfile::Auto {
        let mut cursor = 0usize;
        while cursor < flat.len() {
            let chain: Vec<String> = match fn_nested.get(cursor) {
                Some(Some(info)) => info.chain.clone(),
                _ => vec![flat[cursor].name.name.clone()],
            };
            collect_nested_fns(
                &flat[cursor].body,
                &chain,
                &mut flat,
                &mut fn_receiver,
                &mut fn_owner,
                &mut fn_vowner,
                &mut fn_nested,
                &mut nested_children,
                &mut nested_decl_starts,
            );
            cursor += 1;
        }
    }

    // Pass 1b (PLAN-965 M2-A): Pre-register synthesized generator iterator records
    // before function signatures resolve so `~Iter<T>` / `Iter<T>` map to `VT::Rec(ri)`.
    let mut agg_iter_reg = if surface.is_aggregates() {
        agg_lower::collect_agg_iter_records(program, &flat, &mut records, &mut rec_index, &mut coll)
    } else {
        agg_lower::AggIterRegistry::default()
    };

    for (i, f) in flat.iter().enumerate() {
        let _ = &f;
        let owner_ty = fn_owner[i]
            .map(|oi| records[oi].name.clone())
            .or_else(|| {
                fn_vowner[i].map(|vi| coll.variants[vi].name.clone())
            })
            .unwrap_or_default();
        let sig_key = if let Some(info) = fn_nested.get(i).and_then(|n| n.as_ref()) {
            // PLAN-1081: a nested fn item registers under its mangled def
            // name; its BARE name is only visible through the per-fn
            // nested scope (call-only).
            info.mangled.clone()
        } else if fn_receiver[i] == ast::Receiver::Free {
            f.name.name.clone()
        } else {
            format!("{}::{}", owner_ty, f.name.name)
        };
        // PLAN-936（T-04）：meta 能力在时，两个 meta 查询名是受保护 def——
        // 用户同名声明具名冲突（不静默遮蔽查询，§5.2）。
        if meta_enabled && meta_intrinsic(&f.name.name).is_some() {
            diags.push(Diagnostic::new(
                Stage::Bind,
                "source.cte.meta-name-conflict",
                format!(
                    "`{}` is a protected CT meta query name (cte.meta.fields/v1) and cannot be declared as a function here",
                    f.name.name
                ),
                f.span,
            ));
            continue;
        }
        if sigs.contains_key(&sig_key) {
            // PLAN-862 (entry-contract §2.2): in the entry-unit profile the
            // ENTRY MATRIX owns the duplicate-`main` verdict — it reports
            // `source.entry-duplicate`, and that is the diagnostic a caller
            // must see. Emitting the generic duplicate-fn code here would
            // surface FIRST and hide the entry-specific one; every other
            // name (and every other profile) keeps the generic rule.
            let entry_main_duplicate = matches!(
                surface,
                SourceProfile::EntryUnit | SourceProfile::HostConsole
            ) && fn_receiver[i] == ast::Receiver::Free
                && f.name.name == "main";
            if !entry_main_duplicate {
                diags.push(Diagnostic::new(
                    Stage::Bind,
                    "source.duplicate",
                    format!("function `{}` is declared more than once", f.name.name),
                    f.span,
                ));
            }
            continue;
        }
        let sig = fn_sig_of_coll(
            f,
            fn_receiver[i],
            fn_owner[i],
            fn_vowner[i],
            &rec_index,
            &mut coll,
            &mut diags,
            i,
            is_coll,
        );
        if coll.ordinary_auto
            && fn_receiver[i] == ast::Receiver::Free
            && agg_iter_reg.gen_fns.contains_key(&i)
            && f.params
                .iter()
                .zip(&sig.params)
                .any(|(param, &vt)| agg_lower::generator_borrow_param(param.mode, vt))
        {
            coll.scoped_iter_fns.insert(i);
        }
        sigs.insert(sig_key.clone(), sig);
        // PLAN-1083: method-side registry on the owning enum variant.
        if let Some(vi) = fn_vowner[i] {
            coll.variants[vi].methods.push(ResolvedMethod {
                name: f.name.name.clone(),
                is_static: false,
                is_mut: false,
                fn_index: i,
                span: f.span,
            });
        }
        // method-side registry on the owning record
        if let Some(oi) = fn_owner[i] {
            // PLAN-782 §6.2: on the combination surface the builtin
            // `r.clone()` owns the name — a user-declared record method
            // `clone` (any receiver shape) is a controlled duplicate.
            // The plain record profile keeps 771 semantics (any method
            // name is legal; there is no builtin to collide with).
            if f.name.name == "clone" && surface == SourceProfile::TextRecord {
                diags.push(Diagnostic::new(
                    Stage::Bind,
                    "source.duplicate",
                    format!(
                        "method `clone` on record `{}` collides with the builtin deep-clone; user-declared clone methods are not supported",
                        records[oi].name
                    ),
                    f.name.span,
                ));
                continue;
            }
            records[oi].methods.push(ResolvedMethod {
                name: f.name.name.clone(),
                is_static: fn_receiver[i] == ast::Receiver::Static,
                is_mut: fn_receiver[i] == ast::Receiver::Mut,
                fn_index: i,
                span: f.span,
            });
        }
    }

    if surface.is_aggregates() && !agg_iter_reg.by_elem.is_empty() {
        agg_lower::register_agg_iter_sigs(
            &mut agg_iter_reg,
            &mut sigs,
            &records,
            flat.len(),
            coll.ordinary_auto,
        );
    }
    let iter_info: BTreeMap<usize, (VT, VT, usize, usize, Option<usize>)> = agg_iter_reg
        .by_elem
        .iter()
        .map(|m| {
            (
                m.rec_idx,
                (
                    m.elem_vt,
                    m.opt_vt,
                    m.next_fn_index,
                    m.close_fn_index,
                    m.retired_fn_index,
                ),
            )
        })
        .collect();
    // Ordinary Auto can append synthesized async/closure bodies during binding.
    // Keep foreign offsets provisional until those bodies have stable indices.
    let patch_foreign = !agg_iter_reg.by_elem.is_empty() || surface.is_auto_language();

    // PLAN-909 §5.5 warm seeding: cached owners reference the module's
    // closure-signature table by INDEX; seeding the fresh table with the
    // previous snapshot keeps every cached index valid (interning is
    // value-dedup and append-only past the seed).
    if let Some(w) = warm.as_ref() {
        if let Some(prev) = w.prev {
            coll.function_types = prev.function_types.clone();
        }
    }
    // PLAN-939: foreign signatures are inserted AFTER warm seeding so the
    // interned callback closure index is taken on the seeded table (no
    // index drift between the signature pass and body resolution).
    if let Some(foreign) = supply.and_then(|s| s.foreign()) {
        for (index, declaration) in foreign.source_declarations_ref().iter().enumerate() {
            if sigs.contains_key(&declaration.name) {
                diags.push(Diagnostic::new(
                    Stage::Bind,
                    "source.foreign-conflict",
                    format!(
                        "foreign declaration `{}` conflicts with a source function",
                        declaration.name
                    ),
                    Span { start: 0, end: 0 },
                ));
                continue;
            }
            // PLAN-939: ONE resource-aware mapping (scalar decls are its
            // special case) — records must exist with the exact nominal
            // closure; the callback closure type is interned here.
            let sig = match crate::foreign_resources::source_sig(
                declaration,
                &records,
                &mut coll.function_types,
            ) {
                Ok(s) => s,
                Err((code, message)) => {
                    // PLAN-939 subset callability: a resource declaration
                    // whose record view names no type in THIS module is
                    // simply not callable here (the call site resolves as
                    // an unknown function) — a whole-surface manifest must
                    // not require every record type to exist in every
                    // module that admits it. Any other mapping failure is
                    // still a hard diagnostic.
                    if code == crate::foreign_resources::CODE_RECORD {
                        continue;
                    }
                    diags.push(Diagnostic::new(
                        Stage::Bind,
                        code,
                        message,
                        Span { start: 0, end: 0 },
                    ));
                    continue;
                }
            };
            let foreign_slots = sig.params.len();
            sigs.insert(
                declaration.name.clone(),
                FnSig {
                    param_names: (0..sig.params.len()).map(|i| format!("arg{i}")).collect(),
                    params: sig.params,
                    defaults: vec![None; foreign_slots],
                    result: sig.result,
                    index: if patch_foreign {
                        agg_lower::FOREIGN_INDEX_SENTINEL + index
                    } else {
                        flat.len() + index
                    },
                },
            );
        }
        if !diags.is_empty() {
            if let Some(probe) = instance_probe {
                probe.collect_enum_annotations(&mut coll);
            }
            return Err(diags);
        }
    }
    // PLAN-968 Pass 1.5: intern the scheduler task types (VT::Task(i) /
    // VT::Future(i)) from the desugar side-table so every body — driver
    // or caller — sees stable indices.
    if let Some(surface_tbl) = sched {
        for t in &surface_tbl.tasks {
            let msg = vt_of_ann_coll(&t.msg, &rec_index, &mut coll, &mut diags, TypePos::Local);
            let reply = vt_of_word(t.reply);
            coll.sched_tasks.push(SchedTaskType {
                name: t.name.clone(),
                driver: t.driver_fn.clone(),
                fields_vt: t.fields.iter().map(|(_, w, _)| vt_of_word(*w)).collect(),
                msg,
                reply,
            });
        }
    }

    // Pass 2: per-function body resolution. PLAN-909 §5.5: with a warm
    // cache present, an owner whose ITEM TEXT and the module's DECLARATION
    // CONTEXT are unchanged reuses its resolved entry (and its lambdas)
    // without a resolve walk — the reuse is a real work skip, observed as
    // zero `resolve` body-work for that owner.
    let mut fns = Vec::new();
    let mut closure_fns = Vec::new();
    // PLAN-1081 (T-03): effective fn name -> its local NAMES (the no-capture
    // universe for nested bodies; filled as each fn resolves — flat order
    // keeps every ancestor ahead of its nested items).
    let mut locals_of: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let synth_iter_count = agg_iter_reg.synth_fn_count();
    let closure_base = flat.len() + synth_iter_count;

    for (i, f) in flat.iter().enumerate() {
        let owner_name = f.name.name.clone();
        let text_fp = warm.as_ref().map(|w| {
            let span = f.span;
            fnv64(w.src[span.start..span.end].as_bytes())
        });
        let cached_owner: Option<CachedOwner> = {
            let mut hit = None;
            if let (Some(w), Some(fp)) = (warm.as_ref(), text_fp) {
                if let Some(prev) = w.prev {
                    if prev.context_fp == w.context_fp {
                        if let Some(o) = prev.owners.get(&owner_name) {
                            if o.text_fp == fp {
                                hit = Some(o.clone());
                            }
                        }
                    }
                }
            }
            hit
        };
        if let Some(owner) = cached_owner {
            fns.push(owner.resolved.clone());
            closure_fns.extend(owner.lambdas.iter().cloned());
            if let Some(w) = warm.as_mut() {
                w.out.owners.insert(owner_name.clone(), owner);
                w.reused.insert(owner_name.clone());
            }
            continue;
        }
        crate::body_work::record_resolve(&f.name.name);
        if let Some(probe) = instance_probe {
            let owner = fn_owner[i]
                .map(|record| records[record].name.as_str())
                .or_else(|| fn_vowner[i].map(|variant| coll.variants[variant].name.as_str()));
            probe.begin_body(&f.name.name, owner, diags.len());
        }
        let lambdas_before = closure_fns.len();
        let mut sites = Vec::new();
        let gen_meta = agg_iter_reg.gen_fns.get(&i).copied();
        let mut gen_try_finals = Vec::new();
        // PLAN-1081 (T-03): this fn's nested-item context — visible nested
        // fns (children of the enclosing chain, closer shadows farther)
        // and the ancestors' local names a nested body may never capture.
        let nested_ctx = if surface == SourceProfile::Auto {
            let (chain, own_effective): (Vec<String>, String) =
                match fn_nested.get(i).and_then(|n| n.as_ref()) {
                    Some(info) => (info.chain.clone(), info.mangled.clone()),
                    None => (vec![f.name.name.clone()], f.name.name.clone()),
                };
            let mut scope: BTreeMap<String, FnSig> = BTreeMap::new();
            for anc in chain.iter() {
                for (bare, mangled) in nested_children
                    .get(anc)
                    .map(|v| v.as_slice())
                    .unwrap_or(&[])
                {
                    if let Some(sig) = sigs.get(mangled) {
                        scope.insert(bare.clone(), sig.clone());
                    }
                }
            }
            for (bare, mangled) in nested_children
                .get(&own_effective)
                .map(|v| v.as_slice())
                .unwrap_or(&[])
            {
                if let Some(sig) = sigs.get(mangled) {
                    scope.insert(bare.clone(), sig.clone());
                }
            }
            let mut no_capture: BTreeSet<String> = BTreeSet::new();
            for anc in chain.iter() {
                if let Some(names) = locals_of.get(anc) {
                    no_capture.extend(names.iter().cloned());
                }
            }
            NestedCtx {
                scope,
                no_capture,
                decl_starts: nested_decl_starts.clone(),
                sig_key: match fn_nested.get(i).and_then(|n| n.as_ref()) {
                    Some(info) => Some(info.mangled.clone()),
                    None => None,
                },
            }
        } else {
            NestedCtx::default()
        };
        let mut resolved = resolve_fn_agg(
            f,
            &sigs,
            trace_enabled,
            meta_enabled,
            ct_enabled,
            surface,
            asm_lane,
            &mut diags,
            fn_receiver[i],
            fn_owner[i],
            fn_vowner[i],
            &records,
            &rec_index,
            &mut coll,
            &mut closure_fns,
            &mut sites,
            closure_base,
            supply,
            gen_meta.is_some(),
            &iter_info,
            if gen_meta.is_some() {
                Some(&mut gen_try_finals)
            } else {
                None
            },
            sched,
            nested_ctx,
            instance_probe,
        );
        // A scoped constructor may execute the pre-suspension prefix
        // for a direct for-in consumer. No borrowed formal may enter the
        // persistent iterator record. Frozen profiles keep their rejection.
        let mut prefix = None;
        if gen_meta.is_some() {
            let scoped = coll.scoped_iter_fns.contains(&i);
            if scoped {
                prefix = agg_lower::extract_scoped_prefix(&mut resolved, &mut diags);
            } else {
                for decl in resolved.locals.iter().take(resolved.params.len()) {
                    if agg_lower::generator_borrow_param(decl.mode, decl.vt)
                        || (!coll.ordinary_auto && matches!(decl.vt, VT::Rec(_)))
                    {
                        diags.push(Diagnostic::new(
                            Stage::Bind,
                            "source.generator-borrow-param",
                            format!(
                                "borrowed parameter {} cannot enter a suspended iterator",
                                decl.name
                            ),
                            decl.span,
                        ));
                    }
                }
            }
        }
        if let Some((meta_idx, gen_id)) = gen_meta {
            agg_lower::finalize_one_generator(
                &mut resolved,
                meta_idx,
                gen_id,
                &gen_try_finals,
                prefix,
                &mut agg_iter_reg,
                &mut records,
                &mut coll,
            );
        }
        // PLAN-1081 (T-03): a nested item's RESOLVED name is its mangled
        // def name — the atom ids (t_f_/d_/b_) stay unique when different
        // owners declare same-name nested items.
        let mut resolved = resolved;
        if let Some(info) = fn_nested.get(i).and_then(|n| n.as_ref()) {
            resolved.name = info.mangled.clone();
        }
        fns.push(resolved);
        // PLAN-1081 (T-03): publish this fn's local names for its nested
        // items' no-capture discipline.
        {
            let effective = match fn_nested.get(i).and_then(|n| n.as_ref()) {
                Some(info) => info.mangled.clone(),
                None => f.name.name.clone(),
            };
            let last = fns.last().expect("just pushed");
            locals_of.insert(
                effective,
                last.locals.iter().map(|l| l.name.clone()).collect(),
            );
        }
        if let (Some(w), Some(fp)) = (warm.as_mut(), text_fp) {
            let lambdas: Vec<ResolvedFn> = closure_fns[lambdas_before..].to_vec();
            let resolved = fns.last().expect("just pushed").clone();
            w.out.owners.insert(
                owner_name.clone(),
                CachedOwner {
                    text_fp: fp,
                    resolved,
                    lambdas,
                    sites,
                },
            );
        }
    }
    if surface.is_aggregates() && !agg_iter_reg.by_elem.is_empty() {
        agg_lower::build_all_generator_fns(&mut fns, &agg_iter_reg, &records, &mut coll);
    }
    fns.extend(closure_fns);
    if patch_foreign && supply.and_then(|s| s.foreign()).is_some() {
        patch_foreign_sentinels(&mut fns, &mut sigs);
    }
    if let Some(w) = warm.as_mut() {
        w.out.function_types = coll.function_types.clone();
    }

    // PLAN-862 entry matrix: the entry-unit profile has exactly one entry
    // and a frozen accept/reject set (`entry-contract.md` §2). Runs after
    // Pass 2 so a script's result type can be derived from its tail
    // statement's RESOLVED type.
    if let Some(probe) = instance_probe {
        // Discovery may have unfinished call sites. Its result is consumed
        // only by the instance worklist, before any body normalization.
        probe.collect_enum_annotations(&mut coll);
        return Err(diags);
    }
    if surface.is_entry_unit() {
        // PLAN-982（983 组合入口，用户裁定）：**host-console 面**的入口可以收
        // **借用 `str`** 形参（`execute_entry_text_platform` 的文本实参通道）；
        // EntryUnit/Closures/ConfigDocument 的冻结合同不变（argv 式多参仍拒）。
        let text_entry_params = matches!(surface, SourceProfile::HostConsole);
        check_entry_matrix(&mut fns, &flat, &mut diags, text_entry_params);
    } else if surface.is_aggregates() {
        normalize_aggregates_bodies(
            &mut fns,
            &flat,
            &agg_iter_reg,
            &mut sigs,
            surface == SourceProfile::Auto,
        );
    }

    if diags.is_empty() {
        Ok((
            fns,
            records,
            coll,
            DeclIndexes {
                rec_index,
                sigs,
                trace_enabled,
            },
        ))
    } else {
        Err(diags)
    }
}

/// PLAN-862 T-03: the entry matrix of the entry-unit profile
/// (`attachments/862/work/entry-contract.md` §2). Frozen accept/reject set:
///
/// - exactly one entry candidate: `fn main() int`, `fn main() void`, or the
///   synthetic script `main` built from a top-level statement sequence;
/// - a declared `fn main` plus a script is `source.entry-ambiguous`; two
///   `fn main`s are `source.entry-duplicate`; none is
///   `source.entry-missing`;
/// - the entry takes no parameters (`source.entry-params`);
/// - a Unit entry may not `return <value>` (`source.unit-return-value`) —
///   it ends by falling off the body;
/// - an expression statement is legal ONLY as a script's trailing
///   statement; everywhere else it is `source.entry-tail`, and a
///   non-i32 tail is `source.entry-tail-type` (an `int` entry never gets an
///   implicit tail return);
/// - Unit is a RESULT-only type: a param/local of type `unit` is
///   `source.unit-placement`.
///
/// The script's result type is WRITTEN BACK here (i32 tail → int entry, no
/// tail → Unit entry); the downstream passes (typecheck/HIR/lowering) see
/// the resolved result, never a placeholder.
fn check_entry_matrix(
    fns: &mut [ResolvedFn],
    flat: &[&ast::FnItem],
    diags: &mut Vec<Diagnostic>,
    text_entry_params: bool,
) {
    fn push(diags: &mut Vec<Diagnostic>, code: &str, msg: String, span: Span) {
        diags.push(Diagnostic::new(Stage::Bind, code, msg, span));
    }
    let mains: Vec<usize> = (0..flat.len())
        .filter(|&i| {
            flat[i].receiver == ast::Receiver::Free
                && !flat[i].script
                && flat[i].name.name == "main"
        })
        .collect();
    let scripts: Vec<usize> = (0..flat.len()).filter(|&i| flat[i].script).collect();

    if !scripts.is_empty() && !mains.is_empty() {
        push(
            diags,
            "source.entry-ambiguous",
            "a top-level statement sequence (script) and a declared `fn main` cannot coexist in one module; keep exactly one entry form".to_string(),
            flat[scripts[0]].name.span,
        );
    }
    if mains.len() > 1 {
        push(
            diags,
            "source.entry-duplicate",
            format!(
                "`fn main` is declared {} times; a module has exactly one entry",
                mains.len()
            ),
            flat[mains[1]].name.span,
        );
    }
    if mains.is_empty() && scripts.is_empty() {
        push(
            diags,
            "source.entry-missing",
            "the entry-unit profile requires exactly one entry: `fn main() int`, `fn main() void`, or a top-level statement sequence".to_string(),
            Span { start: 0, end: 0 },
        );
    }

    // -- script entry: result type derived from the tail statement --------
    for &si in &scripts {
        let f = &mut fns[si];
        let n = f.body.stmts.len();
        let mut tail_is_i32 = false;
        for (i, stmt) in f.body.stmts.iter().enumerate() {
            let RStmt::ExprStmt { value, span } = stmt else {
                continue;
            };
            if is_console_effect(value) {
                continue;
            }
            // PLAN-964: a unit-typed expression statement is a side-effect
            // call, never a value — it discards nothing (collections/closures
            // statement rule), so it is legal anywhere in a body.
            if value.vt == VT::Unit {
                continue;
            }
            if i + 1 == n {
                if value.vt == VT::I32 {
                    tail_is_i32 = true;
                } else {
                    push(
                        diags,
                        "source.entry-tail-type",
                        format!(
                            "a script's trailing expression IS its result and must be `i32`, found `{}`",
                            value.vt.as_str()
                        ),
                        *span,
                    );
                }
            } else {
                push(
                    diags,
                    "source.entry-tail",
                    "only the script's TRAILING statement may be an expression; an earlier expression statement has no value consumer".to_string(),
                    *span,
                );
            }
        }
        // an expression statement inside a nested arm is never the tail
        for sp in nested_expr_stmts(&f.body) {
            push(
                diags,
                "source.entry-tail",
                "only the script's TRAILING statement may be an expression; a nested expression statement has no value consumer".to_string(),
                sp,
            );
        }
        f.result = if tail_is_i32 { VT::I32 } else { VT::Unit };
    }

    // -- declared entries: params and the result-kind channel -------------
    for &mi in &mains {
        let f = &fns[mi];
        if !f.params.is_empty() {
            // PLAN-982（983 组合入口）：host-console 面的入口可以收**全部为借用
            // `str`** 的形参（文本实参通道，`execute_entry_text_platform`）；
            // 其它任何参数形状保持冻结的 argv 拒绝。
            let all_borrowed_str =
                text_entry_params && f.params.iter().all(|vt| matches!(vt, VT::Str));
            if !all_borrowed_str {
                push(
                    diags,
                    "source.entry-params",
                    format!(
                        "the entry takes no parameters (found {}); argv-style entry arguments are not part of this profile",
                        f.params.len()
                    ),
                    f.name_span,
                );
            }
        }
        match f.result {
            VT::I32 => {
                // an `int` entry never gets an implicit tail return
                if let Some(sp) = first_expr_stmt(&f.body) {
                    push(
                        diags,
                        "source.entry-tail",
                        "an `int` entry returns through explicit `return`; a trailing expression is not an implicit return".to_string(),
                        sp,
                    );
                }
            }
            VT::Unit => {}
            other => push(
                diags,
                "source.entry-result",
                format!(
                    "the entry result must be `int`/`i32` or `void`, found `{}`",
                    other.as_str()
                ),
                f.name_span,
            ),
        }
    }

    // -- the same rules for every other function --------------------------
    for (i, f) in fns.iter().take(flat.len()).enumerate() {
        // expression statements are a script-tail form only (declared
        // entries were already reported above — do not double-report)
        if !flat[i].script && !mains.contains(&i) {
            if let Some(sp) = first_expr_stmt(&f.body) {
                push(
                    diags,
                    "source.entry-tail",
                    "an expression statement is only legal as a script's trailing statement"
                        .to_string(),
                    sp,
                );
            }
        }
        // Unit functions (entry or helper) have no value to return
        if f.result == VT::Unit {
            if let Some(sp) = first_return(&f.body) {
                push(
                    diags,
                    "source.unit-return-value",
                    format!(
                        "`{}` declares no result (`void`); it has no value to return — end the body without `return`",
                        f.name
                    ),
                    sp,
                );
            }
        }
        // Unit is a result-only type
        for (pi, p) in f.params.iter().enumerate() {
            if *p == VT::Unit {
                push(
                    diags,
                    "source.unit-placement",
                    format!(
                        "parameter #{} of `{}` is `unit`; Unit is a RESULT type only",
                        pi + 1,
                        f.name
                    ),
                    f.name_span,
                );
            }
        }
        for l in &f.locals {
            if l.vt == VT::Unit {
                push(
                    diags,
                    "source.unit-placement",
                    format!(
                        "local `{}` of `{}` is `unit`; Unit is a RESULT type only",
                        l.name, f.name
                    ),
                    l.span,
                );
            }
        }
    }

    // -- normalization (only when the matrix is clean) --------------------
    // Every Unit function ends with an explicit `return unit` (the source
    // form has no bare `return`), and a script whose tail IS its result gets
    // that tail rewritten into the return it denotes. Downstream layers
    // therefore see ONE shape: a body whose fall-off is a typed return.
    if diags.is_empty() {
        for (i, f) in fns.iter_mut().enumerate() {
            let end_span = f.item_span;
            if f.result == VT::Unit {
                // PLAN-964: a closure block body already carries its typed
                // fall-off return (lambda_body appends it) — appending a
                // second one would strand a dead block.
                let already_returns = matches!(f.body.stmts.last(), Some(RStmt::Return { .. }));
                if !already_returns {
                    f.body.stmts.push(RStmt::Return {
                        value: ResolvedExpr {
                            kind: RExprKind::Unit,
                            span: end_span,
                            vt: VT::Unit,
                        },
                        span: end_span,
                    });
                }
            } else if i < flat.len() && flat[i].script {
                if let Some(last) = f.body.stmts.last_mut() {
                    let taken = std::mem::replace(
                        last,
                        RStmt::Break {
                            target: None,
                            span: end_span,
                        },
                    );
                    *last = match taken {
                        RStmt::ExprStmt { value, span } if !is_console_effect(&value) => {
                            RStmt::Return { value, span }
                        }
                        other => other,
                    };
                }
            }
        }
    }
}

fn patch_foreign_sentinels(fns: &mut [ResolvedFn], sigs: &mut BTreeMap<String, FnSig>) {
    let base = fns.len();
    let patch_idx = |idx: &mut usize| {
        if *idx >= agg_lower::FOREIGN_INDEX_SENTINEL {
            *idx = base + (*idx - agg_lower::FOREIGN_INDEX_SENTINEL);
        }
    };
    for sig in sigs.values_mut() {
        patch_idx(&mut sig.index);
    }
    fn patch_expr(e: &mut ResolvedExpr, patch_idx: &impl Fn(&mut usize)) {
        match &mut e.kind {
            RExprKind::CallFn { fn_index, args } => {
                patch_idx(fn_index);
                for a in args {
                    patch_expr(&mut a.value, patch_idx);
                }
            }
            RExprKind::MethodCall {
                fn_index,
                recv,
                args,
            } => {
                patch_idx(fn_index);
                if let Some(r) = recv {
                    patch_expr(r, patch_idx);
                }
                for a in args {
                    patch_expr(&mut a.value, patch_idx);
                }
            }
            RExprKind::Binary { lhs, rhs, .. } => {
                patch_expr(lhs, patch_idx);
                patch_expr(rhs, patch_idx);
            }
            RExprKind::Unary { operand, .. }
            | RExprKind::Move { inner: operand }
            | RExprKind::RecordClone { base: operand }
            | RExprKind::OwnedClone { operand }
            | RExprKind::ListLen { list: operand }
            | RExprKind::ListClear { list: operand }
            | RExprKind::TextDisplay { operand }
            | RExprKind::DynCast { operand, .. }
            | RExprKind::TupleGet { base: operand, .. }
            | RExprKind::FieldGet { base: operand, .. } => patch_expr(operand, patch_idx),
            RExprKind::CtorPos { args, .. } => {
                for a in args {
                    patch_expr(&mut a.value, patch_idx);
                }
            }
            RExprKind::Method { recv, args, .. } => {
                patch_expr(recv, patch_idx);
                for a in args {
                    patch_expr(&mut a.value, patch_idx);
                }
            }
            RExprKind::CtorNamed { fields, .. } => {
                for (_, v) in fields {
                    patch_expr(v, patch_idx);
                }
            }
            RExprKind::TupleLit { elems }
            | RExprKind::CallIntrinsic { args: elems, .. }
            | RExprKind::ObjectCtor { args: elems, .. } => {
                for v in elems {
                    patch_expr(v, patch_idx);
                }
            }
            RExprKind::CallGlobalSet { value, .. } => patch_expr(value, patch_idx),
            RExprKind::CallClosure { callee, args } => {
                patch_expr(callee, patch_idx);
                for a in args {
                    patch_expr(&mut a.value, patch_idx);
                }
            }
            RExprKind::ListGet { list, index }
            | RExprKind::ListRemove { list, index }
            | RExprKind::ListFieldGet { list, index, .. }
            | RExprKind::ElemView { list, index, .. }
            | RExprKind::ObjectGet {
                base: list,
                key: index,
            } => {
                patch_expr(list, patch_idx);
                patch_expr(index, patch_idx);
            }
            RExprKind::ListPush { list, value } => {
                patch_expr(list, patch_idx);
                patch_expr(value, patch_idx);
            }
            RExprKind::ListInsert { list, index, value }
            | RExprKind::ListSet { list, index, value } => {
                patch_expr(list, patch_idx);
                patch_expr(index, patch_idx);
                patch_expr(value, patch_idx);
            }
            RExprKind::VariantCtor { payload, .. } => {
                if let Some(p) = payload {
                    patch_expr(p, patch_idx);
                }
            }
            RExprKind::FStr { parts } => {
                for part in parts {
                    match part {
                        FStrPart::Text(x) | FStrPart::Expr(x) => patch_expr(x, patch_idx),
                    }
                }
            }
            _ => {}
        }
    }
    fn patch_block(block: &mut RBlock, patch_idx: &impl Fn(&mut usize)) {
        for s in &mut block.stmts {
            patch_stmt(s, patch_idx);
        }
    }
    fn patch_stmt(s: &mut RStmt, patch_idx: &impl Fn(&mut usize)) {
        match s {
            RStmt::Let { value, .. }
            | RStmt::Assign { value, .. }
            | RStmt::Return { value, .. }
            | RStmt::ExprStmt { value, .. }
            | RStmt::Fail { value, .. } => patch_expr(value, patch_idx),
            RStmt::If {
                cond, then, els, ..
            } => {
                patch_expr(cond, patch_idx);
                patch_block(then, patch_idx);
                if let Some(eb) = els {
                    patch_block(eb, patch_idx);
                }
            }
            RStmt::While { cond, body, .. } => {
                patch_expr(cond, patch_idx);
                patch_block(body, patch_idx);
            }
            RStmt::Loop { body, .. } => patch_block(body, patch_idx),
            RStmt::Break { .. } | RStmt::Continue { .. } | RStmt::EndView { .. } => {}
        }
    }
    for f in fns.iter_mut() {
        patch_block(&mut f.body, &patch_idx);
    }
}

fn normalize_aggregates_bodies(
    fns: &mut [ResolvedFn],
    flat: &[&ast::FnItem],
    agg_iter_reg: &agg_lower::AggIterRegistry,
    sigs: &mut BTreeMap<String, FnSig>,
    ordinary_auto: bool,
) {
    for (i, item) in flat.iter().enumerate() {
        if agg_iter_reg.gen_fns.contains_key(&i) {
            continue;
        }
        let f = &mut fns[i];
        let end_span = f.item_span;
        if item.script {
            let tail_val_vt = match f.body.stmts.last() {
                Some(RStmt::ExprStmt { value, .. })
                    if !is_console_effect(value) && value.vt != VT::Unit =>
                {
                    Some(value.vt)
                }
                _ => None,
            };
            if let Some(vt) = tail_val_vt {
                let vt = if ordinary_auto
                    && matches!(vt, VT::Object(_) | VT::List(_) | VT::Str | VT::StrOwned)
                {
                    VT::Dyn
                } else {
                    vt
                };
                f.result = vt;
                if let Some(sig) = sigs.get_mut(&f.name) {
                    sig.result = vt;
                }
                if let Some(last) = f.body.stmts.last_mut() {
                    let taken = std::mem::replace(
                        last,
                        RStmt::Break {
                            target: None,
                            span: end_span,
                        },
                    );
                    *last = match taken {
                        RStmt::ExprStmt { value, span } => {
                            let value = if vt == VT::Dyn {
                                FnResolver::coerce_dynamic(value, vt)
                            } else {
                                value
                            };
                            RStmt::Return { value, span }
                        }
                        other => other,
                    };
                }
            } else if matches!(f.body.stmts.last(), Some(RStmt::Fail { .. })) {
                // PLAN-965: diverging standalone script (e.g. top-level uncaught `.?`
                // ending in `fail(...)`) never returns normally; keep scalar `I32`
                // result so `mir.entry-sig` accepts the script entry function.
                f.result = VT::I32;
                if let Some(sig) = sigs.get_mut(&f.name) {
                    sig.result = VT::I32;
                }
            } else {
                f.result = VT::Unit;
                if let Some(sig) = sigs.get_mut(&f.name) {
                    sig.result = VT::Unit;
                }
                if !matches!(
                    f.body.stmts.last(),
                    Some(RStmt::Return { .. } | RStmt::Fail { .. })
                ) {
                    f.body.stmts.push(RStmt::Return {
                        value: ResolvedExpr {
                            kind: RExprKind::Unit,
                            span: end_span,
                            vt: VT::Unit,
                        },
                        span: end_span,
                    });
                }
            }
        } else if f.result == VT::Unit {
            if !matches!(
                f.body.stmts.last(),
                Some(RStmt::Return { .. } | RStmt::Fail { .. })
            ) {
                f.body.stmts.push(RStmt::Return {
                    value: ResolvedExpr {
                        kind: RExprKind::Unit,
                        span: end_span,
                        vt: VT::Unit,
                    },
                    span: end_span,
                });
            }
        } else if let Some(RStmt::ExprStmt { value, .. }) = f.body.stmts.last() {
            if !is_console_effect(value) && value.vt != VT::Unit {
                let target_vt = f.result;
                if let Some(last) = f.body.stmts.last_mut() {
                    let taken = std::mem::replace(
                        last,
                        RStmt::Break {
                            target: None,
                            span: end_span,
                        },
                    );
                    *last = match taken {
                        RStmt::ExprStmt { mut value, span } => {
                            retag_literal(&mut value, target_vt);
                            RStmt::Return { value, span }
                        }
                        other => other,
                    };
                }
            }
        }
    }
}

/// Iterative statement walk of one body in source order (nested arms
/// expanded depth-first) — block nesting is unlimited, so no recursion.
/// The index carried with each statement is its position within its OWN
/// block; the caller that needs ROOT positions walks `root.stmts`
/// directly instead of relying on this index.
fn walk_stmts(root: &RBlock) -> Vec<&RStmt> {
    let mut out: Vec<&RStmt> = Vec::new();
    let mut stack: Vec<&RBlock> = vec![root];
    while let Some(b) = stack.pop() {
        for s in b.stmts.iter() {
            out.push(s);
            match s {
                RStmt::If { els, then, .. } => {
                    if let Some(e) = els {
                        stack.push(e);
                    }
                    stack.push(then);
                }
                RStmt::While { body, .. } | RStmt::Loop { body, .. } => stack.push(body),
                _ => {}
            }
        }
    }
    out
}

/// Expression statements in NESTED blocks only (the script tail can only
/// be a root statement).
fn nested_expr_stmts(root: &RBlock) -> Vec<Span> {
    let mut out: Vec<Span> = Vec::new();
    let mut stack: Vec<&RBlock> = Vec::new();
    for s in root.stmts.iter() {
        match s {
            RStmt::If { els, then, .. } => {
                if let Some(e) = els {
                    stack.push(e);
                }
                stack.push(then);
            }
            RStmt::While { body, .. } | RStmt::Loop { body, .. } => stack.push(body),
            _ => {}
        }
    }
    while let Some(b) = stack.pop() {
        for s in b.stmts.iter() {
            if let RStmt::ExprStmt { span, value } = s {
                if !is_console_effect(value) && value.vt != VT::Unit {
                    out.push(*span);
                }
            }
            match s {
                RStmt::If { els, then, .. } => {
                    if let Some(e) = els {
                        stack.push(e);
                    }
                    stack.push(then);
                }
                RStmt::While { body, .. } | RStmt::Loop { body, .. } => stack.push(body),
                _ => {}
            }
        }
    }
    out
}

/// First `return` in the body (any depth), for the Unit-return rejection.
fn first_return(root: &RBlock) -> Option<Span> {
    walk_stmts(root).into_iter().find_map(|s| match s {
        RStmt::Return { span, .. } => Some(*span),
        _ => None,
    })
}

/// Console effects are statements, never script-tail values.
fn is_console_effect(value: &ResolvedExpr) -> bool {
    matches!(&value.kind, RExprKind::CallIntrinsic { symbol, .. } if symbol.starts_with("auto.console."))
}

/// First non-console, non-unit expression statement, including nested blocks.
fn first_expr_stmt(root: &RBlock) -> Option<Span> {
    walk_stmts(root).into_iter().find_map(|s| match s {
        RStmt::ExprStmt { span, value } if !is_console_effect(value) && value.vt != VT::Unit => {
            Some(*span)
        }
        _ => None,
    })
}

/// PLAN-828 D-01: the RETAINED checked environment of one cold resolve — every function's resolved entry, the signature/record indexes, and
/// the profile/trace facts the resolver consulted. A later single-body
/// re-check ([`recheck_function`]) grafts its result into `program.fns`
/// by O(1) slot swap; unchanged entries are never re-walked. The fields
/// are crate-visible: consumers drive the environment through
/// [`recheck_function`] and the emit seam, never by rebuilding it.
pub struct CheckedEnv {
    pub(crate) program: ResolvedProgram,
    pub(crate) rec_index: BTreeMap<String, usize>,
    pub(crate) sigs: BTreeMap<String, FnSig>,
    pub(crate) trace_enabled: bool,
    pub(crate) profile: SourceProfile,
    /// PLAN-972（T-02）：装配 aggregates 面（packages-aggregates-ct）——
    /// 闭包字面量按 972 域放行（见 FnResolver.asm_lane）。
    pub(crate) asm_lane: bool,
    /// Producer walk counters (D-01 observation; see [`env_walk_stats`]).
    pub(crate) resolve_walks: u64,
    pub(crate) check_walks: u64,
    /// D-03: unique environment identity (process-wide monotonic counter).
    /// A prepared candidate records the environment that prepared it; a
    /// disposition against any other environment is a controlled foreign
    /// rejection — a matching function name and slot index is NOT
    /// environment identity.
    pub(crate) id: u64,
    /// D-03: publication generation — advanced by every SUCCESSFUL
    /// external publication of current semantics: a candidate commit and
    /// an in-place [`recheck_function`](fn recheck_function). A candidate
    /// records the generation it was prepared against; committing onto a
    /// newer base is a controlled stale rejection (an old candidate never
    /// overwrites a newer published result).
    pub(crate) generation: u64,
    /// D-03 (T01-C05): set by the explicit [`CheckedEnv::close`] — after it
    /// every recheck/emit/prepare is a controlled `source.closed`
    /// rejection. Release is never implied by "the field lives in the
    /// session".
    pub(crate) closed: bool,
}

/// Declaration-only candidate. Preparing it never visits or copies a retained
/// body. Body candidates are checked under this context and published together.
pub struct ForeignContext {
    pub(crate) env_id: u64,
    pub(crate) base_generation: u64,
    pub(crate) declarations: Vec<ace_foreign_contract::manual::SourceDecl>,
    pub(crate) slots: Vec<String>,
    pub(crate) sigs: BTreeMap<String, FnSig>,
    /// PLAN-939: the closure-signature table with the foreign callback
    /// type interned (append-only over the environment's table) — grafted
    /// with the sigs and committed with them.
    pub(crate) function_types: Vec<ClosureFnType>,
}
impl ForeignContext {
    pub fn prepare(
        env: &CheckedEnv,
        foreign: &ace_foreign_contract::manual::AdmittedForeign,
    ) -> Result<Self, Vec<Diagnostic>> {
        let declarations = foreign.source_declarations();
        if env.closed
            || declarations
                .iter()
                .any(|d| env.program.fns.iter().any(|f| f.name == d.name))
        {
            return Err(vec![Diagnostic::new(
                Stage::Bind,
                "source.foreign-conflict",
                "foreign context is closed or conflicts with a source declaration",
                Span { start: 0, end: 0 },
            )]);
        }
        let mut slots = if env.program.foreign_slots.is_empty() {
            env.program
                .foreign
                .iter()
                .map(|d| d.name.clone())
                .collect::<Vec<_>>()
        } else {
            env.program.foreign_slots.clone()
        };
        for declaration in &declarations {
            if !slots.contains(&declaration.name) {
                slots.push(declaration.name.clone());
            }
        }
        if slots.len() > 8192 {
            return Err(vec![Diagnostic::new(
                Stage::Bind,
                "source.foreign-slot-budget",
                "foreign declaration slot budget exceeded",
                Span { start: 0, end: 0 },
            )]);
        }
        let mut sigs = env.sigs.clone();
        for old in &env.program.foreign {
            sigs.remove(&old.name);
        }
        let mut function_types = env.program.function_types.clone();
        for declaration in &declarations {
            let index = slots
                .iter()
                .position(|n| n == &declaration.name)
                .expect("reserved slot");
            // PLAN-939: the same resource-aware mapping as the cold pass —
            // INCLUDING subset callability: a record view naming no type in
            // THIS module is not callable here (the call site resolves as
            // an unknown function); a whole-surface manifest must not
            // require every record type in every module that admits it.
            let sig = match crate::foreign_resources::source_sig(
                declaration,
                &env.program.records,
                &mut function_types,
            ) {
                Ok(s) => s,
                Err((code, message)) => {
                    if code == crate::foreign_resources::CODE_RECORD {
                        continue;
                    }
                    return Err(vec![Diagnostic::new(
                        Stage::Bind,
                        code,
                        message,
                        Span { start: 0, end: 0 },
                    )]);
                }
            };
            let foreign_slots = sig.params.len();
            sigs.insert(
                declaration.name.clone(),
                FnSig {
                    param_names: (0..sig.params.len()).map(|i| format!("arg{i}")).collect(),
                    params: sig.params,
                    defaults: vec![None; foreign_slots],
                    result: sig.result,
                    index: env.program.fns.len() + index,
                },
            );
        }
        Ok(Self {
            env_id: env.id,
            base_generation: env.generation,
            declarations,
            slots,
            sigs,
            function_types,
        })
    }
    pub fn validate(&self, env: &CheckedEnv) -> Result<(), Vec<Diagnostic>> {
        if env.closed || self.env_id != env.id || self.base_generation != env.generation {
            return Err(vec![Diagnostic::new(
                Stage::Bind,
                "source.stale",
                "foreign context candidate is closed, foreign or stale",
                Span { start: 0, end: 0 },
            )]);
        }
        Ok(())
    }
    /// Exclusive temporary declaration graft, restored even if the caller
    /// unwinds. The body port still owns its own temporary one-slot graft.
    pub(crate) fn with_env<T>(
        &mut self,
        env: &mut CheckedEnv,
        f: impl FnOnce(&mut CheckedEnv) -> T,
    ) -> Result<T, Vec<Diagnostic>> {
        self.validate(env)?;
        struct Graft<'a> {
            env: &'a mut CheckedEnv,
            context: &'a mut ForeignContext,
        }
        impl Drop for Graft<'_> {
            fn drop(&mut self) {
                std::mem::swap(
                    &mut self.env.program.foreign,
                    &mut self.context.declarations,
                );
                std::mem::swap(&mut self.env.program.foreign_slots, &mut self.context.slots);
                std::mem::swap(&mut self.env.sigs, &mut self.context.sigs);
                std::mem::swap(
                    &mut self.env.program.function_types,
                    &mut self.context.function_types,
                );
            }
        }
        std::mem::swap(&mut env.program.foreign, &mut self.declarations);
        std::mem::swap(&mut env.program.foreign_slots, &mut self.slots);
        std::mem::swap(&mut env.sigs, &mut self.sigs);
        std::mem::swap(&mut env.program.function_types, &mut self.function_types);
        let graft = Graft { env, context: self };
        Ok(f(graft.env))
    }
}

impl CheckedEnv {
    /// Cold retained-environment consumer: run the same whole typecheck once,
    /// without first rendering/binding a second whole-module Atom document.
    pub fn typecheck_all(&mut self) -> Result<(), Vec<Diagnostic>> {
        if self.closed {
            return Err(vec![Diagnostic::new(
                Stage::Text,
                "source.closed",
                "the checked environment is closed",
                Span { start: 0, end: 0 },
            )]);
        }
        super::typecheck::typecheck_with(&self.program, self.profile)?;
        env_note_cold_walks(self);
        Ok(())
    }
    /// PLAN-828 D-03 (T01-C05): EXPLICIT release of the environment's
    /// owned body entries (the resolved `fns` and the record/method
    /// registries). The session's close calls this — dropping the session
    /// struct is not the mechanism. Retained for post-close diagnostics:
    /// the signature/record indexes, profile and walk counters. Idempotent.
    pub fn close(&mut self) {
        self.program.fns = Vec::new();
        self.program.records = Vec::new();
        self.program.lists = Vec::new();
        self.program.variants = Vec::new();
        self.program.function_types = Vec::new();
        self.closed = true;
    }

    /// Whether [`CheckedEnv::close`] already ran (D-03).
    pub fn closed(&self) -> bool {
        self.closed
    }

    /// D-03: this environment's unique identity (diagnostics/tests).
    pub fn env_id(&self) -> u64 {
        self.id
    }

    /// D-03: how many successful external publications (candidate commits
    /// plus in-place rechecks) this environment has served.
    pub fn generation(&self) -> u64 {
        self.generation
    }
}

/// PLAN-913 T-03: assemble a retained environment from portable declaration
/// parts. Only the declaration tables + per-function signatures are supplied;
/// bodies are the caller's responsibility (a restored context is
/// signature-only by construction).
pub(crate) fn checked_env_from_parts(
    program: ResolvedProgram,
    rec_index: BTreeMap<String, usize>,
    sigs: BTreeMap<String, FnSig>,
    trace_enabled: bool,
) -> CheckedEnv {
    CheckedEnv {
        program,
        rec_index,
        sigs,
        trace_enabled,
        profile: SourceProfile::Closures,
        asm_lane: false,
        resolve_walks: 0,
        check_walks: 0,
        id: next_env_id(),
        generation: 0,
        closed: false,
    }
}

/// PLAN-929（T-08）：带 profile 的声明面环境装配（stub-body；恢复 R 轴——
/// 与 913 的 checked_env_from_parts 同纪律，跳过全量 typecheck）。
pub(crate) fn checked_env_from_parts_profile(
    program: ResolvedProgram,
    rec_index: BTreeMap<String, usize>,
    sigs: BTreeMap<String, FnSig>,
    trace_enabled: bool,
    profile: SourceProfile,
) -> CheckedEnv {
    CheckedEnv {
        program,
        rec_index,
        sigs,
        trace_enabled,
        profile,
        asm_lane: false,
        resolve_walks: 0,
        check_walks: 0,
        id: next_env_id(),
        generation: 0,
        closed: false,
    }
}

fn next_env_id() -> u64 {
    static SEQ: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
    SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}

/// PLAN-929（T-08）：签名面公共访问（声明面恢复等价性的对账面——
/// 参数 VT 序 + 结果 VT + 槽位）。
pub fn env_sig_of(env: &CheckedEnv, name: &str) -> Option<(usize, Vec<VT>, VT)> {
    env.sigs
        .get(name)
        .map(|s| (s.index, s.params.clone(), s.result.clone()))
}

/// PLAN-828 D-01: cold resolve that ALSO retains the checked environment
/// for later single-body re-checks. [`resolve_with`] stays
/// allocation-identical for whole-module consumers.
pub fn resolve_env(
    program: &ast::Program,
    capabilities: &[String],
    surface: SourceProfile,
) -> Result<CheckedEnv, Vec<Diagnostic>> {
    resolve_env_with_supply(program, capabilities, surface, None)
}

/// PLAN-968: the retained checked environment for a DESUGARED scheduler
/// program with its task side-table (the warm lane's env shape).
pub fn resolve_env_with_sched(
    program: &ast::Program,
    capabilities: &[String],
    surface: SourceProfile,
    sched: &super::sched_desugar::SchedSurface,
) -> Result<CheckedEnv, Vec<Diagnostic>> {
    let (fns, records, coll, ix) =
        resolve_detailed(program, capabilities, surface, None, None, Some(sched))?;
    Ok(CheckedEnv {
        program: ResolvedProgram {
            foreign_slots: Vec::new(),
            foreign: Vec::new(),
            fns,
            records,
            lists: coll.lists,
            tuples: coll.tuples,
            variants: coll.variants,
            function_types: coll.function_types,
            objects: coll.objects,
            sched_tasks: coll.sched_tasks,
        },
        rec_index: ix.rec_index,
        sigs: ix.sigs,
        // PLAN-968 wrapper constructs outside the assembled lane.
        asm_lane: false,
        trace_enabled: ix.trace_enabled,
        profile: surface,
        resolve_walks: 0,
        check_walks: 0,
        id: next_env_id(),
        generation: 0,
        closed: false,
    })
}

/// PLAN-934 T-08: the retained checked environment built from a TYPE FACT
/// SUPPLY — the same cold resolve as [`resolve_env`], with the query fact
/// layer as the authority for every expression/declaration type instead of
/// the resolver's own synthesis.
///
/// A consumer that already holds the admitted fact pages (the SQ-A product
/// entry) passes their supply here, so the warm single-body machinery is
/// seeded from the same facts as the emitted HIR rather than re-deriving
/// types with the legacy rules.
pub fn resolve_env_with_supply(
    program: &ast::Program,
    capabilities: &[String],
    surface: SourceProfile,
    supply: Option<&dyn TypeSupply>,
) -> Result<CheckedEnv, Vec<Diagnostic>> {
    if supply.is_some() && !surface.is_scalar() {
        return Err(vec![Diagnostic::new(
            Stage::Bind,
            "source.fact-profile",
            format!(
                "fact-supplied binding supports the scalar surface only, not `{}`",
                surface.as_str()
            ),
            Span { start: 0, end: 0 },
        )]);
    }
    let (fns, records, coll, ix) =
        resolve_detailed(program, capabilities, surface, None, supply, None)?;
    Ok(CheckedEnv {
        program: ResolvedProgram {
            foreign_slots: Vec::new(),
            foreign: supply
                .and_then(|s| s.foreign())
                .map(|f| f.source_declarations())
                .unwrap_or_default(),
            fns,
            records,
            lists: coll.lists,
            tuples: Vec::new(),
            variants: coll.variants,
            function_types: coll.function_types,
            objects: coll.objects,
            sched_tasks: coll.sched_tasks,
        },
        rec_index: ix.rec_index,
        sigs: ix.sigs,
        trace_enabled: ix.trace_enabled,
        profile: surface,
        asm_lane: surface.is_aggregates()
            && capabilities
                .iter()
                .any(|c| c == super::packages::IN_CAPABILITY),
        resolve_walks: 0,
        check_walks: 0,
        id: next_env_id(),
        generation: 0,
        closed: false,
    })
}

/// The number of resolved function entries in the environment (D-01
/// observation: unchanged entries are retained, not rebuilt).
pub fn env_fn_count(env: &CheckedEnv) -> usize {
    env.program.fns.len()
}

/// The resolved function names in environment order (D-01/D-03): a
/// consumer binds its declaration indices to the environment's slots and
/// verifies the mapping once at construction.
pub fn env_fn_names(env: &CheckedEnv) -> Vec<&str> {
    env.program.fns.iter().map(|f| f.name.as_str()).collect()
}

/// PLAN-925: the slot of ONE FREE function — the `program.fns` entry with
/// that name and NO owner record (a same-named method of some type never
/// cross-wires into the free function's slot; PLAN-881 keeps them in one
/// table under distinct identities).
pub fn env_free_fn_index(env: &CheckedEnv, name: &str) -> Option<usize> {
    env.program
        .fns
        .iter()
        .position(|f| f.name == name && f.owner_rec.is_none())
}

/// PLAN-925 (§5.4): the slot of ONE method — the entry in `program.fns`
/// whose bare name matches AND whose owner record is `owner` (methods and
/// same-named free functions never cross-wire; the receiver shape is
/// revalidated by the graft itself). `None` when the owner record or the
/// method does not exist.
pub fn env_method_index(env: &CheckedEnv, owner: &str, name: &str) -> Option<usize> {
    let ri = env.rec_index.get(owner).copied()?;
    env.program
        .fns
        .iter()
        .position(|f| f.name == name && f.owner_rec == Some(ri))
}

/// Cold directory for a retained source consumer. Later body edits update the
/// consumer's byte ranges without walking unchanged resolved bodies.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FunctionRegion {
    pub index: usize,
    pub name: String,
    pub item: Span,
    pub body: Span,
}
pub fn env_fn_regions(env: &CheckedEnv) -> Vec<FunctionRegion> {
    env.program
        .fns
        .iter()
        .enumerate()
        .map(|(index, f)| FunctionRegion {
            index,
            name: f.name.clone(),
            item: f.item_span,
            body: f.body.span,
        })
        .collect()
}
/// Declaration source-map metadata only; no body is visited or copied.
pub fn env_fn_declarations(env: &CheckedEnv) -> impl Iterator<Item = (&str, Span, Span)> + '_ {
    env.program
        .fns
        .iter()
        .map(|f| (f.name.as_str(), f.name_span, f.item_span))
}

/// Diagnostic producer identity of the actual retained statement allocation.
/// A same-content freshly resolved body has a different live allocation; this
/// number is never used as a cache key or a verification credential.
pub fn env_fn_body_identity(env: &CheckedEnv, index: usize) -> Option<usize> {
    env.program
        .fns
        .get(index)
        .map(|f| f.body.stmts.as_ptr() as usize)
}

/// PLAN-828 D-01: record the COLD baseline walks (the whole-module
/// resolve + typecheck each walked every function once). Called by the
/// consumer that ran the cold typecheck pass.
pub fn env_note_cold_walks(env: &mut CheckedEnv) {
    let n = env.program.fns.len() as u64;
    env.resolve_walks = n;
    env.check_walks = n;
}

/// PLAN-828 D-01: re-check ONE function body against the retained
/// environment. The item's SIGNATURE must be unchanged — a drift is a
/// controlled rejection (signature edits are the session's explicit
/// non-fast path, never silently re-checked). The body runs the REAL
/// [`resolve_fn`] with the environment's signatures/records, then
/// typechecks just this body against the grafted environment (the same
/// `check_block` walk the whole-module pass runs per function). On
/// success the entry is swapped into `env.program.fns[fn_index]` (the
/// environment stays current for later edits and the emit seam reads it
/// from there); on failure the previous entry is restored untouched.
///
/// D-03 unified base invalidation (T01-C05 residual): a SUCCESSFUL
/// re-check is an external PUBLICATION of the current semantics, so it
/// advances the candidate base generation — candidates prepared against
/// the previous base become stale and can no longer overwrite it. A
/// failed re-check publishes nothing.
pub fn recheck_function(
    env: &mut CheckedEnv,
    item: &ast::FnItem,
    fn_index: usize,
) -> Result<(), Vec<Diagnostic>> {
    let published = recheck_swap(env, item, fn_index);
    if published.is_ok() {
        env.generation += 1;
    }
    published.map(|_| ())
}

/// Core re-check: the same guarded walk as [`recheck_function`], returning
/// the DISPLACED previous entry (moved — nothing is cloned) so a
/// transactional caller can hold "current" across the candidate's
/// post-recheck stages (D-03 single-slot transaction). Deliberately does
/// NOT advance the publication generation: prepare uses this as its
/// bounded TEMPORARY graft and restores current before returning, so it
/// publishes nothing — only the external publication entries
/// ([`recheck_function`], `FnCandidate::commit`) advance the base.
/// The body's interned types are part of its transaction. A new List/Option
/// may refer to a newly appended pool index; publishing or cancelling only the
/// resolved function would leave that reference bound to the wrong table.
pub(crate) struct BodyState {
    pub entry: ResolvedFn,
    pools: Option<BodyTypePools>,
}
struct BodyTypePools {
    lists: Vec<VT>,
    variants: Vec<ResolvedVariant>,
    function_types: Vec<ClosureFnType>,
}
impl BodyState {
    /// PLAN-912: a body entry WITHOUT its own interned-table transaction (a
    /// lambda re-resolved inside its owner's transaction — the owner's
    /// pools carry the table growth).
    pub(crate) fn entry_only(entry: ResolvedFn) -> Self {
        BodyState { entry, pools: None }
    }
    /// Move the complete state into one slot and return the displaced state.
    /// Unchanged function entries and their body allocations are never visited.
    pub(crate) fn swap_into(mut self, env: &mut CheckedEnv, index: usize) -> Self {
        std::mem::swap(&mut self.entry, &mut env.program.fns[index]);
        if let Some(pools) = &mut self.pools {
            std::mem::swap(&mut pools.lists, &mut env.program.lists);
            std::mem::swap(&mut pools.variants, &mut env.program.variants);
            std::mem::swap(&mut pools.function_types, &mut env.program.function_types);
        }
        self
    }
}
pub(crate) fn recheck_swap(
    env: &mut CheckedEnv,
    item: &ast::FnItem,
    fn_index: usize,
) -> Result<BodyState, Vec<Diagnostic>> {
    recheck_swap_with_supply(env, item, fn_index, None)
}

pub(crate) fn recheck_swap_with_supply(
    env: &mut CheckedEnv,
    item: &ast::FnItem,
    fn_index: usize,
    supply: Option<&dyn TypeSupply>,
) -> Result<BodyState, Vec<Diagnostic>> {
    if supply.is_some() && !env.profile.is_scalar() {
        return Err(vec![Diagnostic::new(
            Stage::Bind,
            "source.fact-profile",
            "fact-supplied recheck supports the scalar surface only",
            item.span,
        )]);
    }
    if env.closed {
        return Err(vec![Diagnostic::new(
            Stage::Bind,
            "source.closed",
            "the checked environment is closed; rechecks are rejected",
            item.name.span,
        )]);
    }
    if fn_index >= env.program.fns.len() || env.program.fns[fn_index].name != item.name.name {
        return Err(vec![Diagnostic::new(
            Stage::Bind,
            "source.incremental.slot",
            format!(
                "function `{}` is not the retained entry at slot {fn_index}",
                item.name.name
            ),
            item.name.span,
        )]);
    }
    // PLAN-881: a method's graft keeps its RECEIVER/OWNER shape — the
    // retained entry at this slot is the authority; a bare name match
    // must never let a same-named free function and method cross-wire
    // into each other's slot. Methods key the cold sigs registry as
    // `<Owner>::<name>` (the cold pass's exact rule).
    let retained_recv = env.program.fns[fn_index].receiver;
    let (recv, owner_rec) = if retained_recv == ast::Receiver::Free {
        (ast::Receiver::Free, None)
    } else {
        let owner_name = item
            .owner
            .as_ref()
            .map(|o| o.name.clone())
            .unwrap_or_default();
        match env.rec_index.get(&owner_name) {
            Some(&ri) => (item.receiver, Some(ri)),
            None => {
                return Err(vec![Diagnostic::new(
                    Stage::Bind,
                    "source.incremental.owner",
                    format!(
                        "method `{}` names owner record `{}` which does not exist",
                        item.name.name, owner_name
                    ),
                    item.name.span,
                )])
            }
        }
    };
    let sig_key = if recv == ast::Receiver::Free {
        item.name.name.clone()
    } else {
        format!(
            "{}::{}",
            owner_rec
                .and_then(|ri| env.program.records.get(ri).map(|r| r.name.clone()))
                .unwrap_or_default(),
            item.name.name
        )
    };
    // Signature drift guard: rebuild the item's signature with the cold
    // pass's exact rules; any difference is a controlled rejection.
    let mut guard_diags = Vec::new();
    let rebuilt = if env.profile.is_collections() {
        let mut c2 = CollTables::default();
        c2.lists = env.program.lists.clone();
        c2.aggregates_admission = env.profile.is_aggregates();
        c2.ordinary_auto = env.profile.is_auto_language();
        c2.elem_view_admission = false;
        let names2: Vec<(String, usize)> = env
            .program
            .variants
            .iter()
            .enumerate()
            .map(|(k, v)| (v.name.clone(), k))
            .collect();
        c2.variants = env.program.variants.clone();
        for (nm, k) in names2 {
            c2.reindex_variant(&nm, k);
        }
        fn_sig_of_coll(
            item,
            recv,
            owner_rec,
            None,
            &env.rec_index,
            &mut c2,
            &mut guard_diags,
            fn_index,
            true,
        )
    } else {
        fn_sig_of(
            item,
            recv,
            owner_rec,
            &env.rec_index,
            &mut guard_diags,
            fn_index,
            env.profile.is_aggregates(),
        )
    };
    let expected = &env.sigs[&sig_key];
    // PLAN-932: the signature rebuild's OWN diagnostics are real bind-level
    // language facts (duplicate parameters, unknown record annotations) —
    // they pass through verbatim, never collapsed into the "changed its
    // signature" policy code. Only a clean rebuild that DIFFERS from the
    // retained signature is the true signature-drift non-fast path.
    if !guard_diags.is_empty() {
        return Err(guard_diags);
    }
    if rebuilt.param_names != expected.param_names
        || rebuilt.params != expected.params
        || rebuilt.result != expected.result
    {
        return Err(vec![Diagnostic::new(
            Stage::Bind,
            "source.incremental",
            format!(
                "function `{}` changed its signature; that is the non-fast path",
                item.name.name
            ),
            item.name.span,
        )]);
    }
    let mut diags = Vec::new();
    env.resolve_walks += 1;
    // PLAN-881: body-work meters under the OWNER-QUALIFIED key — a bare
    // method name is ambiguous against a same-named free function.
    crate::body_work::record_resolve(&sig_key);
    // PLAN-864: rebuild the collections tables from the retained program
    // (the same lists/variants the cold pass interned). New body-local types
    // join the same transaction as the body; declaration indices stay stable.
    let mut coll = CollTables::default();
    coll.lists = env.program.lists.clone();
    coll.function_types = env.program.function_types.clone();
    coll.aggregates_admission = env.profile.is_aggregates();
    coll.ordinary_auto = env.profile.is_auto_language();
    coll.tuples = env.program.tuples.clone();
    let names: Vec<(String, usize)> = env
        .program
        .variants
        .iter()
        .enumerate()
        .map(|(i, v)| (v.name.clone(), i))
        .collect();
    coll.variants = env.program.variants.clone();
    for (name, i) in names {
        coll.reindex_variant(&name, i);
    }
    let resolved = resolve_fn(
        item,
        &env.sigs,
        env.trace_enabled,
        // PLAN-936（T-04）：closures/暖环境再检面无 CT 语法（语法面门控），
        // meta/CT 能力旗标恒 false。
        false,
        false,
        env.profile,
        env.asm_lane,
        &mut diags,
        recv,
        owner_rec,
        None,
        &env.program.records,
        &env.rec_index,
        &mut coll,
        &mut Vec::new(),
        &mut Vec::new(),
        env.program.fns.len(),
        supply,
    );
    if !diags.is_empty() {
        return Err(diags);
    }
    // Graft, then typecheck the ONE body against the grafted environment
    // (self-recursion sees the new entry); restore the old entry on
    // failure so a rejected body never pollutes the environment.
    let mut previous = BodyState {
        entry: resolved,
        pools: None,
    };
    crate::body_work::record_produced("resolve", &sig_key);
    if env.profile == SourceProfile::Collections {
        previous.pools = Some(BodyTypePools {
            lists: coll.lists,
            variants: coll.variants,
            function_types: coll.function_types,
        });
    }
    let previous = previous.swap_into(env, fn_index);
    let check_diags = if supply.is_some() {
        Vec::new()
    } else {
        env.check_walks += 1;
        super::typecheck::check_body_in(&env.program, fn_index)
    };
    if !check_diags.is_empty() {
        previous.swap_into(env, fn_index);
        return Err(check_diags);
    }
    Ok(previous)
}

// D-03 (T01-C05): the candidate/transaction types live in `adapter.rs`
// next to `prepare_function_candidate` — the pipeline that produces them.

/// PLAN-912 §5.2: re-resolve ONE lambda body alone against its retained
/// creation site — no owner walk, no sibling walk. The site's raw capture
/// uses, parameter list and `move` flag are the fast-path oracle: any
/// difference is returned as the bounded [`LambdaReResolve::NonFast`]
/// (the owner's creation site changed and must re-resolve), never a silent
/// re-derivation of modes/cells from a partial view. On success the body's
/// interned tables travel with it as a [`BodyState`] so the frontend commits
/// pool growth in the same transaction (PLAN-864 rule).
pub(crate) fn resolve_lambda_in_context(
    env: &CheckedEnv,
    site: &LambdaSiteFacts,
    expr: &ast::Expr,
) -> Result<BodyState, LambdaReResolve> {
    let ast::ExprKind::Closure {
        move_capture,
        params,
        result,
        body,
    } = &expr.kind
    else {
        return Err(LambdaReResolve::Diagnostics(vec![Diagnostic::new(
            Stage::Bind,
            "source.incremental-fragment",
            "a lambda fragment must be a lambda expression",
            expr.span,
        )]));
    };
    if *move_capture != site.move_capture {
        return Err(LambdaReResolve::NonFast(LambdaNonFast::SignatureChanged));
    }
    if site.nested != 0 {
        return Err(LambdaReResolve::NonFast(LambdaNonFast::NestedLambdas));
    }
    let uses = super::closure_capture::analyze_capture_uses(params, body, &site.outer_names);
    let uses: Vec<(String, bool)> = uses.into_iter().map(|c| (c.name, c.written)).collect();
    if uses != site.capture_uses {
        return Err(LambdaReResolve::NonFast(LambdaNonFast::CaptureUsesChanged));
    }
    let mut coll = CollTables::default();
    coll.lists = env.program.lists.clone();
    coll.function_types = env.program.function_types.clone();
    coll.aggregates_admission = env.profile.is_aggregates();
    coll.ordinary_auto = env.profile.is_auto_language();
    coll.tuples = env.program.tuples.clone();
    let names: Vec<(String, usize)> = env
        .program
        .variants
        .iter()
        .enumerate()
        .map(|(i, v)| (v.name.clone(), i))
        .collect();
    coll.variants = env.program.variants.clone();
    for (name, i) in names {
        coll.reindex_variant(&name, i);
    }
    let mut diags = Vec::new();
    let mut nested_defs = Vec::new();
    let mut nested_sites = Vec::new();
    let shared_cell_spans = match body.as_ref() {
        ast::ClosureBody::Block(block) => {
            super::closure_capture::scan_shared_capture_cells(params, block)
        }
        ast::ClosureBody::Expr(_) => BTreeSet::new(),
    };
    // The ONE resolve walk this edit performs — the changed lambda's.
    crate::body_work::record_resolve(&site.name);
    let lb = {
        let mut r = FnResolver {
            instance_probe: None,
            sigs: &env.sigs,
            trace_enabled: env.trace_enabled,
            meta_enabled: false,
            ct_enabled: false,
            ct_depth: 0,
            // PLAN-968: lambdas never carry the scheduler surface.
            sched: None,
            profile: SourceProfile::Closures,
            asm_lane: false,
            nested: NestedCtx::default(),
            // callback-by-name is a text-record-only surface; the closures
            // face never reads the boundary.
            source_fn_count: 0,
            diags: &mut diags,
            locals: Vec::new(),
            scopes: vec![Scope {
                names: BTreeMap::new(),
                views: BTreeMap::new(),
                text_mut_aliases: BTreeMap::new(),
            }],
            declared_names: BTreeSet::new(),
            implicit_record_fields: false,
            owned_spec_result: false,
            fn_result: site.result,
            legacy_text_result: false,
            legacy_text_lists: BTreeSet::new(),
            legacy_text_find_results: BTreeMap::new(),
            loop_stack: Vec::new(),
            loop_count: 0,
            used_intrinsics: BTreeSet::new(),
            closure_defs: &mut nested_defs,
            moved_locals: BTreeSet::new(),
            py_locals: BTreeSet::new(),
            py_context: env.sigs.contains_key("py_import"),
            py_lit_locals: BTreeMap::new(),
            borrowed_closure_locals: BTreeSet::new(),
            records: &env.program.records,
            rec_index: &env.rec_index,
            coll: &mut coll,
            iterate_targets: Vec::new(),
            for_latches: Vec::new(),
            shared_cell_spans,
            owner_name: site.owner.clone(),
            closure_count: site.ordinal,
            lambda_sites: &mut nested_sites,
            reuse: None,
            supply: None,
            stmt_sinks: vec![Vec::new()],
            ctrl_frames: Vec::new(),
            ctrl_state: None,
            catch_err_locals: Vec::new(),
            is_generator: false,
            is_script: false,
            iter_info: &EMPTY_ITER_INFO,
            scoped_iter_site: None,
            agg_tmp_counter: 0,
            expr_diverged: false,
            gen_try_finals: Vec::new(),
        };
        r.lambda_body(params, result.as_ref(), body, &site.capture_decls, expr.span)
    };
    if !nested_defs.is_empty() {
        return Err(LambdaReResolve::NonFast(LambdaNonFast::NestedLambdas));
    }
    if !diags.is_empty() {
        return Err(LambdaReResolve::Diagnostics(diags));
    }
    let params_now: Vec<(String, VT)> = lb
        .param_names
        .iter()
        .cloned()
        .zip(lb.param_types.iter().copied())
        .collect();
    if params_now != site.params {
        return Err(LambdaReResolve::NonFast(LambdaNonFast::SignatureChanged));
    }
    if lb.result != site.result {
        return Err(LambdaReResolve::NonFast(LambdaNonFast::ResultChanged));
    }
    let type_index = coll.intern_function(ClosureFnType {
        params: lb.param_types.clone(),
        result: lb.result,
    });
    if type_index != site.type_index {
        return Err(LambdaReResolve::NonFast(LambdaNonFast::SignatureChanged));
    }
    let entry = ResolvedFn {
        name: site.name.clone(),
        name_span: expr.span,
        item_span: expr.span,
        param_names: lb.param_names,
        params: lb.param_types,
        param_modes: vec![false; params.len()],
        result: lb.result,
        locals: lb.locals,
        body: lb.body,
        loop_count: 0,
        used_intrinsics: lb.used_intrinsics,
        receiver: ast::Receiver::Free,
        owner_rec: None,
        owner_var: None,
    };
    Ok(BodyState {
        entry,
        pools: Some(BodyTypePools {
            lists: coll.lists,
            variants: coll.variants,
            function_types: coll.function_types,
        }),
    })
}

/// PLAN-912 T-03: the result of re-resolving ONE owner body against the
/// retained environment with retained-lambda reuse.
pub(crate) struct OwnerRecheck {
    /// The owner's new entry with its interned-table transaction.
    pub state: BodyState,
    /// The owner's lambdas after the walk, in `closure_defs` (post-order)
    /// order — reused entries verbatim, re-resolved ones fresh.
    pub lambdas: Vec<ResolvedFn>,
    pub sites: Vec<LambdaSiteFacts>,
    /// Retained ordinals that were reused WITHOUT a walk.
    pub reused: Vec<usize>,
}

/// PLAN-912 T-03: re-resolve ONE top-level owner body against the retained
/// environment. Same signature-drift guard as [`recheck_swap`] (a signature
/// edit is the explicit non-fast path); the owner's lambdas are resolved
/// through the retained-lambda reuse oracle so unchanged siblings are not
/// walked. Nothing is grafted here — the caller owns the transaction.
pub(crate) fn resolve_owner_in_env(
    env: &CheckedEnv,
    item: &ast::FnItem,
    fn_index: usize,
    reuse_entries: Vec<ReuseEntry>,
    src: &str,
) -> Result<OwnerRecheck, Vec<Diagnostic>> {
    if env.closed {
        return Err(vec![Diagnostic::new(
            Stage::Bind,
            "source.closed",
            "the checked environment is closed; rechecks are rejected",
            item.name.span,
        )]);
    }
    if fn_index >= env.program.fns.len() || env.program.fns[fn_index].name != item.name.name {
        return Err(vec![Diagnostic::new(
            Stage::Bind,
            "source.incremental",
            format!(
                "function `{}` is not the retained entry at slot {fn_index}",
                item.name.name
            ),
            item.name.span,
        )]);
    }
    if item.receiver != ast::Receiver::Free
        || env.program.fns[fn_index].receiver != ast::Receiver::Free
    {
        return Err(vec![Diagnostic::new(
            Stage::Bind,
            "source.incremental",
            "closures owners are free functions",
            item.name.span,
        )]);
    }
    // Signature drift guard with the cold pass's exact rules.
    let mut guard_diags = Vec::new();
    let mut c2 = CollTables::default();
    c2.lists = env.program.lists.clone();
    let names2: Vec<(String, usize)> = env
        .program
        .variants
        .iter()
        .enumerate()
        .map(|(k, v)| (v.name.clone(), k))
        .collect();
    c2.variants = env.program.variants.clone();
    for (nm, k) in names2 {
        c2.reindex_variant(&nm, k);
    }
    let rebuilt = fn_sig_of_coll(
        item,
        ast::Receiver::Free,
        None,
        None,
        &env.rec_index,
        &mut c2,
        &mut guard_diags,
        fn_index,
        true,
    );
    let expected = &env.sigs[&item.name.name];
    if !guard_diags.is_empty()
        || rebuilt.param_names != expected.param_names
        || rebuilt.params != expected.params
        || rebuilt.result != expected.result
    {
        return Err(vec![Diagnostic::new(
            Stage::Bind,
            "source.incremental",
            format!(
                "function `{}` changed its signature; that is the non-fast path",
                item.name.name
            ),
            item.name.span,
        )]);
    }
    let mut diags = Vec::new();
    crate::body_work::record_resolve(&item.name.name);
    let mut coll = CollTables::default();
    coll.lists = env.program.lists.clone();
    coll.function_types = env.program.function_types.clone();
    coll.aggregates_admission = env.profile.is_aggregates();
    coll.ordinary_auto = env.profile.is_auto_language();
    coll.tuples = env.program.tuples.clone();
    let names: Vec<(String, usize)> = env
        .program
        .variants
        .iter()
        .enumerate()
        .map(|(i, v)| (v.name.clone(), i))
        .collect();
    coll.variants = env.program.variants.clone();
    for (name, i) in names {
        coll.reindex_variant(&name, i);
    }
    let mut closure_defs = Vec::new();
    let mut sites = Vec::new();
    let mut reuse = LambdaReuse {
        src,
        entries: reuse_entries,
        reused: Vec::new(),
    };
    let resolved = resolve_fn_reuse(
        item,
        &env.sigs,
        env.trace_enabled,
        false,
        false,
        SourceProfile::Closures,
        false,
        &mut diags,
        ast::Receiver::Free,
        None,
        None,
        &env.program.records,
        &env.rec_index,
        &mut coll,
        &mut closure_defs,
        &mut sites,
        env.program.fns.len(),
        Some(&mut reuse),
        None,
        false,
        &EMPTY_ITER_INFO,
        None,
        None,
        NestedCtx::default(),
        None,
    );
    if !diags.is_empty() {
        return Err(diags);
    }
    Ok(OwnerRecheck {
        state: BodyState {
            entry: resolved,
            pools: Some(BodyTypePools {
                lists: coll.lists,
                variants: coll.variants,
                function_types: coll.function_types,
            }),
        },
        lambdas: closure_defs,
        sites,
        reused: reuse.reused,
    })
}

/// PLAN-912: the ONE capture-mode decision table (PLAN-865/909 rules),
/// shared by the owner walk and the retained-lambda reuse oracle so both
/// derive identical declarations from identical source facts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CaptureDecision {
    Decl {
        mode: crate::hir::CaptureMode,
        body_vt: VT,
        moved: bool,
        writable: bool,
        cell: bool,
    },
    /// A written capture of a non-scalar (`source.closure-shared-mutable`).
    SharedMutableRejected,
    /// Neither Copy, nor supported owned move, nor synchronous Str borrow.
    Unsupported,
}

pub(crate) fn decide_capture(
    source_vt: VT,
    source_mutable: bool,
    source_cell: bool,
    owned: bool,
    move_capture: bool,
    written: bool,
) -> CaptureDecision {
    let copy = matches!(
        source_vt,
        VT::I32 | VT::Bool | VT::Char | VT::TaskRef(_) | VT::Deferred(_)
    ) || (move_capture && vt_is_numeric(source_vt));
    // PLAN-964: an OBJECT captures as a Move Rc share (the cell's nature
    // is shared-mutable — aliases observe each other's writes; a written
    // object capture keeps the share, never a private snapshot). Nil is
    // zero-width and captures trivially. Both are admitted BEFORE the
    // shared-mutable cell gate (the object IS the shared cell); a `move`
    // closure keeps the same share — moving the handle would still alias
    // the cell, and a sibling closure must be able to capture it too.
    if matches!(source_vt, VT::Object(_) | VT::Nil) {
        return CaptureDecision::Decl {
            mode: crate::hir::CaptureMode::Move,
            body_vt: source_vt,
            moved: false,
            writable: matches!(source_vt, VT::Object(_)) && source_mutable,
            cell: false,
        };
    }
    // PLAN-909 SEM-04 extension: a written capture is admitted only in the
    // scalar `var` domain. Everything else (owned Str / record / List /
    // Variant / closure / Dyn) keeps the NAMED rejection — the cell contract
    // does not silently clone, share an owned aggregate or promote a borrow.
    if written && !copy {
        return CaptureDecision::SharedMutableRejected;
    }
    // (mode, body type, kills source, body local writable, body local cell)
    if copy {
        if !move_capture && source_cell {
            // Shared cell: any non-move capture of a binding in the
            // function's shared-write group — reader OR writer — sees the
            // same cell (a reader created first still observes later
            // writes). Writability mirrors the source `var`.
            CaptureDecision::Decl {
                mode: crate::hir::CaptureMode::Shared,
                body_vt: source_vt,
                moved: false,
                writable: source_mutable,
                cell: true,
            }
        } else if move_capture && written {
            // K06 private writable env: snapshot the current value into a
            // cell the env alone owns; the source stays live and is never
            // written back. A `let` source keeps the immutable-write
            // rejection on the body assignment.
            CaptureDecision::Decl {
                mode: crate::hir::CaptureMode::Move,
                body_vt: source_vt,
                moved: false,
                writable: source_mutable,
                cell: source_mutable,
            }
        } else {
            // 865 read-only Copy snapshot (no sibling writer).
            CaptureDecision::Decl {
                mode: crate::hir::CaptureMode::Move,
                body_vt: source_vt,
                moved: false,
                writable: false,
                cell: false,
            }
        }
    } else if owned && move_capture {
        CaptureDecision::Decl {
            mode: crate::hir::CaptureMode::Move,
            body_vt: source_vt,
            moved: true,
            writable: false,
            cell: false,
        }
    } else if source_vt == VT::StrOwned && !move_capture {
        CaptureDecision::Decl {
            mode: crate::hir::CaptureMode::Borrow,
            body_vt: VT::Str,
            moved: false,
            writable: false,
            cell: false,
        }
    } else {
        CaptureDecision::Unsupported
    }
}

/// PLAN-912 T-03: one retained lambda an owner re-resolve may reuse.
#[derive(Clone)]
pub(crate) struct ReuseEntry {
    /// fnv64 of the lambda expression text in the source it was resolved from.
    pub text_fp: u64,
    pub site: LambdaSiteFacts,
    pub body: ResolvedFn,
}

/// PLAN-912 T-03: the retained-lambda reuse oracle handed to an owner
/// re-resolve. `entries` are the owner's retained lambdas in `closure_defs`
/// (post-order) order; `reused` collects the ordinals the walk reused.
pub(crate) struct LambdaReuse<'a> {
    pub src: &'a str,
    pub entries: Vec<ReuseEntry>,
    pub reused: Vec<usize>,
}

impl<'a, 's> FnResolver<'a, 's> {
    /// Try to push lambda `k = closure_count` (and its nested lambdas) from
    /// the retained entries instead of walking it. Succeeds only when the
    /// expression text is byte-identical AND every retained capture still
    /// derives the identical declaration from the CURRENT creation-point
    /// locals; otherwise the caller performs the real walk.
    fn try_reuse_lambda(&mut self, move_capture: bool, span: Span) -> Option<ResolvedExpr> {
        let k = self.closure_count;
        let (entry_idx, nested, text_fp, retained_move, uses, decls) = {
            let reuse = self.reuse.as_deref()?;
            let idx = reuse
                .entries
                .iter()
                .position(|e| e.site.ordinal == k + e.site.nested)?;
            let e = &reuse.entries[idx];
            let text_now = reuse.src.get(span.start..span.end)?;
            (
                idx,
                e.site.nested,
                fnv64(text_now.as_bytes()),
                e.site.move_capture,
                e.site.capture_uses.clone(),
                e.site.capture_decls.clone(),
            )
        };
        {
            let reuse = self.reuse.as_deref()?;
            if reuse.entries[entry_idx].text_fp != text_fp || retained_move != move_capture {
                return None;
            }
        }
        if uses.len() != decls.len() {
            return None;
        }
        // Re-derive every retained capture from the CURRENT locals.
        let mut bindings = Vec::with_capacity(uses.len());
        let mut moved_now = Vec::new();
        for ((name, written), decl) in uses.iter().zip(decls.iter()) {
            let source_local = self.lookup(name)?;
            let l = &self.locals[source_local.0];
            let owned = self.vt_is_coll_owned(l.vt);
            match decide_capture(l.vt, l.mutable, l.cell, owned, move_capture, *written) {
                CaptureDecision::Decl {
                    mode,
                    body_vt,
                    moved,
                    writable,
                    cell,
                } if (name.as_str(), body_vt, mode, writable, cell)
                    == (decl.0.as_str(), decl.1, decl.2, decl.3, decl.4) =>
                {
                    bindings.push((source_local, mode));
                    if moved {
                        moved_now.push(source_local);
                    }
                }
                _ => return None,
            }
        }
        // Commit the reuse: nested entries first (post-order), then this one.
        let (name, type_index) = {
            let reuse = self.reuse.as_deref_mut()?;
            let first = entry_idx - nested;
            // The outer lambda's text is byte-identical, so every nested
            // lambda sits at the same offset relative to it: shift nested
            // spans by the outer's displacement (M-axis coordinate remap,
            // no body walk).
            let outer_shift =
                span.start as i64 - reuse.entries[entry_idx].site.expr_span.start as i64;
            let mut pushed = Vec::with_capacity(nested + 1);
            for i in first..=entry_idx {
                let e = &reuse.entries[i];
                let mut site = e.site.clone();
                let mut body = e.body.clone();
                if i == entry_idx {
                    site.expr_span = span;
                    site.capture_sources = bindings.iter().map(|(l, _)| *l).collect();
                    body.item_span = span;
                    body.name_span = span;
                } else {
                    let shifted = Span {
                        start: (site.expr_span.start as i64 + outer_shift) as usize,
                        end: (site.expr_span.end as i64 + outer_shift) as usize,
                    };
                    site.expr_span = shifted;
                    body.item_span = shifted;
                    body.name_span = shifted;
                }
                pushed.push((site, body, e.site.ordinal));
            }
            for (site, body, ordinal) in pushed {
                self.lambda_sites.push(site);
                self.closure_defs.push(body);
                reuse.reused.push(ordinal);
                self.closure_count += 1;
            }
            let e = &reuse.entries[entry_idx];
            (e.site.name.clone(), e.site.type_index)
        };
        for l in moved_now {
            self.moved_locals.insert(l);
        }
        Some(ResolvedExpr {
            span,
            vt: VT::Function(type_index),
            kind: RExprKind::CreateClosure {
                callee: name,
                captures: bindings,
            },
        })
    }
}

/// Output of one lambda body walk (see [`FnResolver::lambda_body`]).
struct LambdaBodyOut {
    param_names: Vec<String>,
    param_types: Vec<VT>,
    result: VT,
    locals: Vec<LocalDecl>,
    body: RBlock,
    used_intrinsics: BTreeSet<&'static str>,
}

/// PLAN-828 D-01 observation, producer-reported: cumulative body walks
/// this environment has performed — the cold resolve walks every
/// function once; each later [`recheck_function`] adds exactly one
/// resolve walk and one typecheck walk. (resolved_walks, checked_walks)
pub fn env_walk_stats(env: &CheckedEnv) -> (u64, u64) {
    (env.resolve_walks, env.check_walks)
}

/// PLAN-828 D-01: one function's signature value under the cold pass's
/// exact rules (duplicate-param diagnostics included) — shared by the
/// signature registry and the single-body drift guard.
fn fn_sig_of(
    f: &ast::FnItem,
    recv: ast::Receiver,
    owner_rec: Option<usize>,
    rec_index: &BTreeMap<String, usize>,
    diags: &mut Vec<Diagnostic>,
    index: usize,
    aggregates: bool,
) -> FnSig {
    fn_sig_of_coll(
        f,
        recv,
        owner_rec,
        None,
        rec_index,
        &mut {
            let mut t = CollTables::default();
            t.aggregates_admission = aggregates;
            t
        },
        diags,
        index,
        false,
    )
}

/// PLAN-864: signature construction with the collections type tables;
/// `is_coll` selects the table-aware annotation grammar.
/// PLAN-1081 (T-03): the flatten-table sidecar of one nested fn item.
#[derive(Clone)]
pub(crate) struct NestedFnInfo {
    /// The bare source name (`inner`).
    pub bare: String,
    /// The owner-mangled def name (`__nested_main_inner`).
    pub mangled: String,
    /// Enclosing fn effective names, OUTERMOST first, ending with the
    /// direct enclosing fn (`["main"]`, `["main", "__nested_main_inner"]`).
    pub chain: Vec<String>,
}

/// PLAN-1081 (T-03): the per-fn nested-item context the body resolver
/// carries (Auto face): the visible nested fns by bare name (call-only),
/// the enclosing locals a nested body may NEVER capture, and the collected
/// declaration spans (an uncollected item is a named rejection).
pub(crate) struct NestedCtx {
    pub scope: BTreeMap<String, FnSig>,
    pub no_capture: BTreeSet<String>,
    pub decl_starts: BTreeSet<usize>,
    /// The nested item's MANGLED sig key (None for ordinary fns) — the
    /// ResolvedFn construction reads its signature through this key.
    pub sig_key: Option<String>,
}

impl Default for NestedCtx {
    fn default() -> Self {
        NestedCtx {
            scope: BTreeMap::new(),
            no_capture: BTreeSet::new(),
            decl_starts: BTreeSet::new(),
            sig_key: None,
        }
    }
}

/// PLAN-1081 (T-03): depth-first pre-order collection of nested fn items
/// from a function body's statement blocks (expression/lambda interiors
/// are not collected — an uncollected item reaching resolve is a named
/// rejection, never a silent drop). Each item is appended to the flat
/// table with a `__nested_`-prefixed mangled name and a Free receiver.
fn collect_nested_fns<'a>(
    block: &'a ast::Block,
    chain: &[String],
    flat: &mut Vec<&'a ast::FnItem>,
    fn_receiver: &mut Vec<ast::Receiver>,
    fn_owner: &mut Vec<Option<usize>>,
    fn_vowner: &mut Vec<Option<usize>>,
    fn_nested: &mut Vec<Option<NestedFnInfo>>,
    nested_children: &mut BTreeMap<String, Vec<(String, String)>>,
    nested_decl_starts: &mut BTreeSet<usize>,
) {
    for stmt in &block.stmts {
        match stmt {
            ast::Stmt::NestedFn { item } => {
                let bare = item.name.name.clone();
                let mangled = format!("__nested_{}_{}", chain.join("_"), bare);
                let owner_key = chain.last().expect("non-empty chain").clone();
                nested_children
                    .entry(owner_key)
                    .or_default()
                    .push((bare.clone(), mangled.clone()));
                nested_decl_starts.insert(item.span.start);
                fn_nested.push(Some(NestedFnInfo {
                    bare,
                    mangled,
                    chain: chain.to_vec(),
                }));
                fn_receiver.push(ast::Receiver::Free);
                fn_owner.push(None);
                fn_vowner.push(None);
                flat.push(item);
                // Recurse into the nested item's own body with the grown
                // chain (its children key on this item's mangled name).
                let mut child_chain = chain.to_vec();
                child_chain.push(item.name.name.clone());
                collect_nested_fns(
                    &item.body,
                    &child_chain,
                    flat,
                    fn_receiver,
                    fn_owner,
                    fn_vowner,
                    fn_nested,
                    nested_children,
                    nested_decl_starts,
                );
            }
            ast::Stmt::If { then, els, .. } => {
                collect_nested_fns(
                    then,
                    chain,
                    flat,
                    fn_receiver,
                    fn_owner,
                    fn_vowner,
                    fn_nested,
                    nested_children,
                    nested_decl_starts,
                );
                if let Some(eb) = els {
                    collect_nested_fns(
                        eb,
                        chain,
                        flat,
                        fn_receiver,
                        fn_owner,
                        fn_vowner,
                        fn_nested,
                        nested_children,
                        nested_decl_starts,
                    );
                }
            }
            ast::Stmt::While { body, .. }
            | ast::Stmt::Loop { body, .. }
            | ast::Stmt::ForIn { body, .. }
            | ast::Stmt::Hold { body, .. } => {
                collect_nested_fns(
                    body,
                    chain,
                    flat,
                    fn_receiver,
                    fn_owner,
                    fn_vowner,
                    fn_nested,
                    nested_children,
                    nested_decl_starts,
                );
            }
            ast::Stmt::Try {
                body,
                catch,
                finally,
                ..
            } => {
                collect_nested_fns(
                    body,
                    chain,
                    flat,
                    fn_receiver,
                    fn_owner,
                    fn_vowner,
                    fn_nested,
                    nested_children,
                    nested_decl_starts,
                );
                if let Some(cc) = catch {
                    collect_nested_fns(
                        &cc.body,
                        chain,
                        flat,
                        fn_receiver,
                        fn_owner,
                        fn_vowner,
                        fn_nested,
                        nested_children,
                        nested_decl_starts,
                    );
                }
                if let Some(fb) = finally {
                    collect_nested_fns(
                        fb,
                        chain,
                        flat,
                        fn_receiver,
                        fn_owner,
                        fn_vowner,
                        fn_nested,
                        nested_children,
                        nested_decl_starts,
                    );
                }
            }
            _ => {}
        }
    }
}

fn fn_sig_of_coll(
    f: &ast::FnItem,
    recv: ast::Receiver,
    owner_rec: Option<usize>,
    owner_var: Option<usize>,
    rec_index: &BTreeMap<String, usize>,
    coll: &mut CollTables,
    diags: &mut Vec<Diagnostic>,
    index: usize,
    is_coll: bool,
) -> FnSig {
    let mut seen: BTreeSet<&str> = BTreeSet::new();
    for p in &f.params {
        if !seen.insert(p.name.name.as_str()) {
            // N33: span joins both same-named param phrases.
            let first = f
                .params
                .iter()
                .find(|q| q.name.name == p.name.name)
                .unwrap();
            diags.push(Diagnostic::new(
                Stage::Bind,
                "source.duplicate",
                format!("parameter `{}` is declared more than once", p.name.name),
                first.span.join(p.span),
            ));
        }
    }
    // PLAN-771: a view receiver occupies param 0; the visible param
    // names/types below are the written ones.
    let mut param_names: Vec<String> = Vec::new();
    let mut params: Vec<VT> = Vec::new();
    if recv.is_instance() {
        param_names.push("self".to_string());
        // PLAN-1083: an enum instance method's receiver is the VARIANT
        // value (`is self` dispatches on it).
        params.push(match owner_var {
            Some(vi) => VT::Variant(vi),
            None => VT::Rec(owner_rec.unwrap_or(usize::MAX)),
        });
    }
    for p in &f.params {
        param_names.push(p.name.name.clone());
        params.push(if is_coll {
            vt_of_ann_coll(&p.ty, rec_index, coll, diags, TypePos::Param)
        } else {
            vt_of_ann(&p.ty, rec_index, diags)
        });
    }
    // PLAN-963（T-08，L-24）：默认值与 param_names 逐位对齐（隐含 self 位恒 None）。
    let mut defaults: Vec<Option<Box<ast::Expr>>> = vec![None; usize::from(recv.is_instance())];
    for p in &f.params {
        defaults.push(p.default.clone());
    }
    let result = if is_coll {
        vt_of_ann_coll(&f.ret, rec_index, coll, diags, TypePos::Result)
    } else {
        vt_of_ann(&f.ret, rec_index, diags)
    };
    debug_assert_eq!(defaults.len(), param_names.len());
    FnSig {
        param_names,
        params,
        defaults,
        result,
        index,
    }
}

/// Value type of a type annotation (PLAN-771): builtin words map directly;
/// record names resolve through the pre-registered record index (unknown
/// names are a bind-level error pointing at the annotation).
fn vt_of_ann(
    ann: &ast::TypeAnn,
    rec_index: &BTreeMap<String, usize>,
    diags: &mut Vec<Diagnostic>,
) -> VT {
    match &ann.kind {
        ast::TypeRef::Builtin(w) => vt_of_word(*w),
        // PLAN-1080（D-04）：Inferred 到达 resolve = 补全遍没有覆盖到该
        // 注解位（fail-closed 具名拒，不静默默认类型）。
        ast::TypeRef::Inferred => {
            diags.push(Diagnostic::new(
                Stage::Bind,
                "source.inference",
                "an omitted type annotation could not be inferred here".to_string(),
                ann.span,
            ));
            VT::I32
        }
        // PLAN-962：tuple 类型只在聚合面的表感知路径解析；到达本（无表）
        // 路径 = 具名拒。
        ast::TypeRef::Tuple(_) => {
            diags.push(Diagnostic::new(
                Stage::Bind,
                "source.unsupported",
                "tuple types are part of the aggregates source profile".to_string(),
                ann.span,
            ));
            VT::I32
        }
        // PLAN-935：类型变量只能存在于模板内；到达 resolve = 模板未被实例
        // 化（防御面——实例层在 resolve 前已把模板替换为具体实例）。
        ast::TypeRef::TyVar(id) => {
            diags.push(Diagnostic::new(
                Stage::Bind,
                "source.instances-uninstantiated",
                format!(
                    "type parameter `{}` reached type binding — a template must be concretized into instances before resolve",
                    id.name
                ),
                id.span,
            ));
            VT::I32
        }
        ast::TypeRef::Record(id) => match rec_index.get(&id.name) {
            Some(&i) => VT::Rec(i),
            None => {
                diags.push(Diagnostic::new(
                    Stage::Bind,
                    "source.undefined",
                    format!("no record type named `{}` in this module", id.name),
                    id.span,
                ));
                VT::I32
            }
        },
        ast::TypeRef::Optional(_) => {
            diags.push(Diagnostic::new(
                Stage::Bind,
                "source.unsupported",
                "optional types (`?T`) are not part of this source profile".to_string(),
                ann.span,
            ));
            VT::I32
        }
        ast::TypeRef::Generic { name, .. } => {
            diags.push(Diagnostic::new(
                Stage::Bind,
                "source.unsupported",
                format!(
                    "type `{}<..>` is not part of this source profile",
                    name.name
                ),
                ann.span,
            ));
            VT::I32
        }
        ast::TypeRef::Function { .. } => {
            diags.push(Diagnostic::new(
                Stage::Bind,
                "source.unsupported",
                "function types are not part of this source profile".to_string(),
                ann.span,
            ));
            VT::I32
        }
        ast::TypeRef::ResultBang(_) => {
            diags.push(Diagnostic::new(
                Stage::Bind,
                "source.unsupported",
                "error types (`!T`) are part of the aggregates source profile".to_string(),
                ann.span,
            ));
            VT::I32
        }
        ast::TypeRef::Stream(_) => {
            diags.push(Diagnostic::new(
                Stage::Bind,
                "source.unsupported",
                "generator stream types (`~Iter<T>`) are part of the aggregates source profile"
                    .to_string(),
                ann.span,
            ));
            VT::I32
        }
    }
}

/// Where a type annotation appears — PLAN-864 value-context rules.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum TypePos {
    /// Record field: composites are rejected (no container fields).
    Field,
    /// Variant payload: closure values are not admitted into containers.
    ContainerPayload,
    /// Parameter: List/Variant rejected in v1 (function-local lists).
    Param,
    /// Function result: Variant allowed, List rejected.
    Result,
    /// Local annotation: everything allowed.
    Local,
}

/// PLAN-864: table-aware annotation resolution — `?T` sugar, `List<T>`,
/// `Result<T,E>` and record names, with the value-context rejections of
/// [`TypePos`].
pub(crate) const INFERRED_OBJECT_TYPE: &str = "<inferred-object>";

fn is_legacy_text_list(ann: &ast::TypeAnn) -> bool {
    matches!(&ann.kind, ast::TypeRef::Generic { name, args }
        if name.name == "List"
            && matches!(args.as_slice(), [ast::TypeAnn { kind: ast::TypeRef::Builtin(TypeWord::Str), .. }]))
}

fn vt_of_ann_coll(
    ann: &ast::TypeAnn,
    rec_index: &BTreeMap<String, usize>,
    coll: &mut CollTables,
    diags: &mut Vec<Diagnostic>,
    pos: TypePos,
) -> VT {
    let reject = |diags: &mut Vec<Diagnostic>, vt: VT, msg: String, span: Span| {
        diags.push(Diagnostic::new(
            Stage::Bind,
            "source.list-value-context",
            msg,
            span,
        ));
        vt
    };
    if let ast::TypeRef::Generic { name, args } = &ann.kind {
        if let Some(ty) = instances::enum_type_annotation(name, args, rec_index, coll, diags, ann.span) {
            return ty;
        }
    }
    match &ann.kind {
        // PLAN-1080（D-04）：同上——补全遍后仍 Inferred = 具名拒。
        ast::TypeRef::Inferred => reject(
            diags,
            VT::I32,
            "an omitted type annotation could not be inferred here".to_string(),
            ann.span,
        ),
        // PLAN-962 (L-15): tuple type `(T1, T2, ...)`. Structural VT;
        // aggregates surface only (the grammar crops tuple-ty to it).
        ast::TypeRef::Tuple(args) if coll.aggregates_admission => {
            if args.len() < 2 {
                reject(
                    diags,
                    VT::I32,
                    "a tuple type needs at least two elements".to_string(),
                    ann.span,
                )
            } else {
                let element_pos = if coll.ordinary_auto
                    && matches!(pos, TypePos::Field | TypePos::ContainerPayload)
                {
                    TypePos::ContainerPayload
                } else {
                    TypePos::Local
                };
                let vts: Vec<VT> = args
                    .iter()
                    .map(|a| vt_of_ann_coll(a, rec_index, coll, diags, element_pos))
                    .collect();
                VT::Tuple(coll.intern_tuple(vts))
            }
        }
        // PLAN-962: older profiles never parse the shape (grammar crop);
        // a programmatic Tuple reaching this path is a named reject.
        ast::TypeRef::Tuple(_) => reject(
            diags,
            VT::I32,
            "tuple types are part of the aggregates source profile".to_string(),
            ann.span,
        ),
        ast::TypeRef::Builtin(TypeWord::Str)
            if coll.ordinary_auto && matches!(pos, TypePos::Field | TypePos::ContainerPayload) =>
        {
            VT::StrOwned
        }
        ast::TypeRef::Builtin(w) => vt_of_word(*w),
        // PLAN-935：同 vt_of_ann——类型变量到达 bind = 未实例化模板的防御拒。
        ast::TypeRef::TyVar(id) => {
            diags.push(Diagnostic::new(
                Stage::Bind,
                "source.instances-uninstantiated",
                format!(
                    "type parameter `{}` reached type binding — a template must be concretized into instances before resolve",
                    id.name
                ),
                id.span,
            ));
            VT::I32
        }
        ast::TypeRef::Record(id) => match rec_index.get(&id.name) {
            Some(&i) => VT::Rec(i),
            None if coll.variant_by_name(&id.name).is_some() => {
                if matches!(pos, TypePos::Field | TypePos::Param) && !coll.aggregates_admission {
                    return reject(
                        diags,
                        VT::I32,
                        "variant-typed fields/parameters are not part of this source profile"
                            .to_string(),
                        ann.span,
                    );
                }
                VT::Variant(coll.variant_by_name(&id.name).expect("checked name"))
            }
            None => {
                diags.push(Diagnostic::new(
                    Stage::Bind,
                    "source.undefined",
                    format!("no type named `{}` in this module", id.name),
                    id.span,
                ));
                VT::I32
            }
        },
        ast::TypeRef::Optional(inner) => {
            if matches!(pos, TypePos::Field | TypePos::Param) && !coll.aggregates_admission {
                return reject(
                    diags,
                    VT::I32,
                    "variant-typed fields/parameters are not part of this source profile (v1 variants live in locals, returns and the entry channel)".to_string(),
                    ann.span,
                );
            }
            let inner_vt = vt_of_ann_coll(inner, rec_index, coll, diags, TypePos::ContainerPayload);
            if matches!(inner_vt, VT::List(_)) && !coll.aggregates_admission {
                return reject(
                    diags,
                    VT::I32,
                    "List-typed payloads are not part of the 822 element domain".to_string(),
                    inner.span,
                );
            }
            let vi = coll.intern_variant(
                "Option",
                ann.span,
                vec![
                    ("Some".to_string(), Some(inner_vt)),
                    ("None".to_string(), None),
                ],
            );
            VT::Variant(vi)
        }
        ast::TypeRef::Generic { name, args } => match (name.name.as_str(), args.len()) {
            (super::sched_desugar::ACTOR_PAYLOAD_TYPE, 0) if coll.ordinary_auto => VT::Dyn,
            (INFERRED_OBJECT_TYPE, 0) if coll.aggregates_admission => {
                VT::Object(coll.intern_object(VT::Dyn))
            }
            // PLAN-962 (T-08, L-08): `Box<T>` — explicit single-value
            // indirection. Representation: the single-element List cell
            // (t01-freeze D9 — the closure-cell precedent); a Box-typed
            // field is therefore a LIST-typed field: fixed-size handle,
            // and the record containment cycle walk does not follow it
            // (legal recursion: `type Node { next Box<Node> }`).
            ("Future", 1) if coll.ordinary_auto => {
                let result = vt_of_ann_coll(&args[0], rec_index, coll, diags, TypePos::Result);
                VT::Deferred(coll.intern_function(ClosureFnType {
                    params: Vec::new(),
                    result,
                }))
            }
            ("TaskRef", 1) if coll.ordinary_auto => {
                let message =
                    vt_of_ann_coll(&args[0], rec_index, coll, diags, TypePos::ContainerPayload);
                VT::TaskRef(coll.intern_task_message(message))
            }
            ("Box", 1) if coll.aggregates_admission => {
                let elem = vt_of_ann_coll(&args[0], rec_index, coll, diags, TypePos::Local);
                match elem {
                    VT::StrOwned | VT::I32 | VT::Bool | VT::Char | VT::I8 | VT::U8 | VT::U32
                    | VT::I64 | VT::U64 | VT::F32 | VT::F64 | VT::Rec(_) | VT::Variant(_) => {
                        VT::List(coll.intern_list(elem))
                    }
                    VT::List(_) => reject(
                        diags,
                        VT::I32,
                        "Box does not wrap a List (wrap the element type; a Box is already one cell)"
                            .to_string(),
                        args[0].span,
                    ),
                    _ => reject(
                        diags,
                        VT::I32,
                        format!("Box element type `{}` is not admitted", elem.as_str()),
                        args[0].span,
                    ),
                }
            }
            ("List", 1) => {
                if matches!(pos, TypePos::Field | TypePos::Param | TypePos::Result)
                    && !coll.aggregates_admission
                {
                    return reject(
                            diags,
                            VT::I32,
                            "List-typed fields/parameters/results are not part of this source profile (v1 lists are function-local values)".to_string(),
                            ann.span,
                        );
                }
                let elem = vt_of_ann_coll(&args[0], rec_index, coll, diags, TypePos::Local);
                let elem = if coll.ordinary_auto && elem == VT::Str {
                    VT::StrOwned
                } else {
                    elem
                };
                match elem {
                    // PLAN-962: the aggregates surface admits nested List
                    // (the arena node graph is depth-free); the frozen 822
                    // one-layer domain keeps rejecting for older profiles.
                    VT::List(_) if coll.aggregates_admission => VT::List(coll.intern_list(elem)),
                    VT::List(_) => reject(
                        diags,
                        VT::I32,
                        "List does not nest List (822 element domain)".to_string(),
                        args[0].span,
                    ),
                    VT::StrOwned
                    | VT::I32
                    | VT::Bool
                    | VT::Rec(_)
                    | VT::Variant(_)
                    | VT::TaskRef(_)
                    | VT::Deferred(_) => VT::List(coll.intern_list(elem)),
                    // PLAN-962: wide numerics and char ride the numeric
                    // element closure the MIR verifier already admits.
                    VT::Char
                    | VT::I8
                    | VT::U8
                    | VT::U32
                    | VT::I64
                    | VT::U64
                    | VT::F32
                    | VT::F64
                        if coll.aggregates_admission =>
                    {
                        VT::List(coll.intern_list(elem))
                    }
                    _ => reject(
                        diags,
                        VT::I32,
                        format!(
                            "element type `{}` is not in the 822 element domain",
                            elem.as_str()
                        ),
                        args[0].span,
                    ),
                }
            }
            ("Result", 2) => {
                if matches!(pos, TypePos::Field | TypePos::Param) && !coll.aggregates_admission {
                    return reject(
                        diags,
                        VT::I32,
                        "variant-typed fields/parameters are not part of this source profile"
                            .to_string(),
                        ann.span,
                    );
                }
                let ok_vt =
                    vt_of_ann_coll(&args[0], rec_index, coll, diags, TypePos::ContainerPayload);
                let err_vt =
                    vt_of_ann_coll(&args[1], rec_index, coll, diags, TypePos::ContainerPayload);
                for (a, label) in [(&args[0], "Ok"), (&args[1], "Err")] {
                    if matches!(a.kind, ast::TypeRef::Builtin(TypeWord::Void))
                        || matches!(
                            vt_of_ann_coll(
                                a,
                                rec_index,
                                coll,
                                &mut Vec::new(),
                                TypePos::ContainerPayload,
                            ),
                            VT::Unit
                        )
                    {
                        diags.push(Diagnostic::new(
                            Stage::Bind,
                            "source.type-mismatch",
                            format!("`Result` payload `{label}` cannot be void"),
                            a.span,
                        ));
                        return VT::I32;
                    }
                }
                if (matches!(ok_vt, VT::List(_)) || matches!(err_vt, VT::List(_)))
                    && !coll.aggregates_admission
                {
                    return reject(
                        diags,
                        VT::I32,
                        "List-typed payloads are not part of the 822 element domain".to_string(),
                        ann.span,
                    );
                }
                let vi = coll.intern_variant(
                    "Result",
                    ann.span,
                    vec![
                        ("Ok".to_string(), Some(ok_vt)),
                        ("Err".to_string(), Some(err_vt)),
                    ],
                );
                VT::Variant(vi)
            }
            ("Option", _) => {
                diags.push(Diagnostic::new(
                    Stage::Bind,
                    "source.syntax",
                    "`Option` is written `?T` in this source profile".to_string(),
                    name.span,
                ));
                VT::I32
            }
            ("Iter", 1) if coll.aggregates_admission => {
                let elem =
                    vt_of_ann_coll(&args[0], rec_index, coll, diags, TypePos::ContainerPayload);
                let key = iter_record_name(elem);
                rec_index
                    .get(&key)
                    .map(|&ri| VT::Rec(ri))
                    .unwrap_or_else(|| {
                        reject(
                            diags,
                            VT::I32,
                            format!("internal: missing iterator state record `{}`", key),
                            ann.span,
                        )
                    })
            }
            ("Iter", _) => reject(
                diags,
                VT::I32,
                "iterator types (`Iter<T>`) are part of the aggregates source profile".to_string(),
                ann.span,
            ),
            (other, n) => {
                diags.push(Diagnostic::new(
                    Stage::Bind,
                    "source.undefined",
                    format!(
                        "no parameterized type named `{}` taking {} type argument(s)",
                        other, n
                    ),
                    name.span,
                ));
                VT::I32
            }
        },
        ast::TypeRef::Function { params, result } => {
            if matches!(pos, TypePos::Field | TypePos::ContainerPayload) {
                diags.push(Diagnostic::new(
                    Stage::Bind,
                    "source.unsupported",
                    "function values are not admitted in record fields or container payloads"
                        .to_string(),
                    ann.span,
                ));
                return VT::I32;
            }
            let resolved_params: Vec<VT> = params
                .iter()
                .map(|param| vt_of_ann_coll(param, rec_index, coll, diags, TypePos::Param))
                .collect();
            for (index, (param, vt)) in params.iter().zip(&resolved_params).enumerate() {
                if *vt == VT::Unit {
                    diags.push(Diagnostic::new(
                        Stage::Bind,
                        "source.unit-placement",
                        format!(
                            "function type parameter #{} is `unit`; Unit is a RESULT type only",
                            index + 1
                        ),
                        param.span,
                    ));
                }
            }
            let result = vt_of_ann_coll(result, rec_index, coll, diags, TypePos::Result);
            VT::Function(coll.intern_function(ClosureFnType {
                params: resolved_params,
                result,
            }))
        }
        ast::TypeRef::ResultBang(inner) => {
            if !coll.aggregates_admission {
                return reject(
                    diags,
                    VT::I32,
                    "error types (`!T`) are part of the aggregates source profile".to_string(),
                    ann.span,
                );
            }
            let ok_vt = vt_of_ann_coll(inner, rec_index, coll, diags, TypePos::ContainerPayload);
            let vi = coll.intern_variant(
                "Result",
                ann.span,
                vec![
                    ("Ok".to_string(), Some(ok_vt)),
                    ("Err".to_string(), Some(VT::StrOwned)),
                ],
            );
            VT::Variant(vi)
        }
        ast::TypeRef::Stream(inner) => {
            if !coll.aggregates_admission {
                return reject(
                    diags,
                    VT::I32,
                    "generator stream types (`~Iter<T>`) are part of the aggregates source profile"
                        .to_string(),
                    ann.span,
                );
            }
            let elem = vt_of_ann_coll(inner, rec_index, coll, diags, TypePos::ContainerPayload);
            let key = iter_record_name(elem);
            rec_index
                .get(&key)
                .map(|&ri| VT::Rec(ri))
                .unwrap_or_else(|| {
                    reject(
                        diags,
                        VT::I32,
                        format!("internal: missing iterator state record `{}`", key),
                        ann.span,
                    )
                })
        }
    }
}

pub(crate) fn iter_record_name(elem: VT) -> String {
    let suffix = match elem {
        VT::I32 => "i32".to_string(),
        // PLAN-968: handles never iterate; the spelling only feeds the
        // synthesized record name so it stays stable and unique.
        VT::Task(i) => format!("task{i}"),
        VT::TaskRef(i) => format!("taskref{i}"),
        VT::Future(i) => format!("future{i}"),
        VT::Deferred(i) => format!("deferred{i}"),
        VT::Bool => "bool".to_string(),
        VT::Str => "str".to_string(),
        VT::StrOwned => "Str".to_string(),
        VT::Char => "char".to_string(),
        VT::Rec(r) => format!("r{r}"),
        VT::Unit => "unit".to_string(),
        VT::List(l) => format!("l{l}"),
        VT::Variant(v) => format!("v{v}"),
        VT::Function(f) => format!("f{f}"),
        VT::Tuple(t) => format!("t{t}"),
        VT::Object(o) => format!("o{o}"),
        VT::Nil => "nil".to_string(),
        VT::Dyn => "dyn".to_string(),
        VT::I8 => "i8".to_string(),
        VT::U8 => "u8".to_string(),
        VT::U32 => "u32".to_string(),
        VT::I64 => "i64".to_string(),
        VT::U64 => "u64".to_string(),
        VT::F32 => "f32".to_string(),
        VT::F64 => "f64".to_string(),
    };
    format!("__ace_iter_{suffix}")
}

struct Scope {
    names: BTreeMap<String, LocalRef>,
    /// PLAN-864: owned-element `for-in` bindings — a read-only projection
    /// context, never a materialized local. Lookup falls here when `names`
    /// misses; any use other than a field path / `.len()` is
    /// `source.view-escape`.
    views: BTreeMap<String, ViewBinding>,
    /// A text writeback target, resolved lexically and never stored as a value.
    text_mut_aliases: BTreeMap<String, LocalRef>,
}

/// One owned-element iteration binding (PLAN-864 contract C7).
#[derive(Clone, Copy, Debug)]
struct ViewBinding {
    list: LocalRef,
    index: LocalRef,
    /// The element type (a record or `Str`).
    elem: VT,
}

/// PLAN-864: how a method-call receiver was classified before its
/// arguments resolve.
#[derive(Clone, Copy, Debug)]
enum RecvClass<'a> {
    /// A value receiver (record instance / list / variant-free text).
    Value,
    /// A bare record type name selecting a static method — carries the
    /// OWNING record index (PLAN-881: two records declaring same-named
    /// static methods must dispatch to their own owner, never the first
    /// declaring record).
    Static(usize),
    /// `List<T>.new()` — the interned list type.
    ListCtor(usize),
    /// `E.A` / `E.A(e)` — qualified variant construction.
    VariantArm { vi: usize, disc: u32 },
    GenericEnum { template: &'a ast::Ident, types: &'a [ast::TypeAnn] },
    /// PLAN-981（IF-981-ORIGINAL）：AGE 资源服务命名空间接收者
    /// （`material2d.color(...)`）。携带声明表里那一条 op。
    AgeService(&'static super::age_resources::AgeResourceOp),
    /// PLAN-981（IF-981-GFX-AV-SEAM）：AGE 图形服务（`query2d.segment` 等）——
    /// 同 AgeService：接收者是命名空间，按 gfx 声明表分类。
    AgeGfx(&'static super::age_resources::AgeGfxServiceOp),
    /// PLAN-864: `<list<Str>-local>.get(i).len()` — a whole-Str element
    /// view consumed by len (contract C4).
    ElemViewLen {
        xl: LocalRef,
        xvt: VT,
        idx_slot: usize,
    },
}

struct FnResolver<'a, 's> {
    instance_probe: Option<&'a instances::InstanceProbe<'a>>,
    sigs: &'a BTreeMap<String, FnSig>,
    trace_enabled: bool,
    /// PLAN-936（T-04）：meta 查询能力（cte.meta.fields/v1）已授予。
    meta_enabled: bool,
    /// PLAN-936（T-04）：CT 表达式能力（ac.source.cte.scalar/v1）已授予。
    ct_enabled: bool,
    /// PLAN-936（T-04）：当前绑定位置是否在 `#{ ... }` 子树内（嵌套深度）。
    ct_depth: u32,

    /// PLAN-782: the combination profile keeps `.move` wrappers as
    /// first-class argument values (text profile unwraps them into
    /// CallBinding.moved markers instead).
    profile: SourceProfile,
    /// PLAN-972（T-02）：装配 aggregates 面（packages-aggregates-ct lane——
    /// `av.aggregates/1` 准入 + `ac.source.cte.in/v1`）——闭包字面量按 972
    /// 域放行；其余面保持 `resolve_closure` 的 PLAN-964 具名拒绝。
    asm_lane: bool,
    /// PLAN-1081 (T-03): nested fn item context (Auto face).
    nested: NestedCtx,
    diags: &'a mut Vec<Diagnostic>,
    locals: Vec<LocalDecl>,
    scopes: Vec<Scope>,
    /// PLAN-771: the module's record table (read-only here).
    records: &'a [ResolvedRecord],
    rec_index: &'a BTreeMap<String, usize>,
    /// PLAN-864: the collections type tables (lists/variants; grown as
    /// bodies intern new types).
    coll: &'a mut CollTables,
    /// Every local name declared anywhere in this function (any scope), to
    /// distinguish "never declared" from "declared in a scope that ended".
    declared_names: BTreeSet<String>,
    implicit_record_fields: bool,
    owned_spec_result: bool,
    /// Result type of the function being resolved (retag context).
    fn_result: VT,
    legacy_text_result: bool,
    legacy_text_lists: BTreeSet<LocalRef>,
    legacy_text_find_results: BTreeMap<LocalRef, LocalRef>,
    /// PLAN-968: the scheduler surface side-table (scheduler face only).
    sched: Option<&'a super::sched_desugar::SchedSurface>,
    loop_stack: Vec<LoopRef>,
    loop_count: usize,
    used_intrinsics: BTreeSet<&'static str>,
    closure_defs: &'a mut Vec<ResolvedFn>,
    moved_locals: BTreeSet<LocalRef>,
    /// Borrow-capturing closures are restricted to direct calls through a
    /// local binding; any value read is an escape.
    borrowed_closure_locals: BTreeSet<LocalRef>,
    /// PLAN-864: active `for-in` iterations — (iterated local, element
    /// type). Method resolution rejects structural modification of the
    /// iterated list inside the body (`source.iterate-mutate`).
    iterate_targets: Vec<(LocalRef, VT, Span)>,
    /// PLAN-1036 (AUDIT-1024 F28/F29): the counted `for-in` LATCH — the
    /// hidden index advance that closes one iteration. The normal
    /// fall-through runs it at the body's tail; a `continue` into the same
    /// loop must run the very same sequence before the header condition is
    /// re-tested, otherwise the iteration repeats the current element
    /// (a lowering jump cannot target a mid-loop label, so the latch is
    /// re-emitted at each continue site instead of restructured).
    for_latches: Vec<(LoopRef, Vec<RStmt>)>,
    /// PLAN-909: start offsets of cell-eligible `var` bindings this
    /// function's non-move closures write (function-wide static grouping;
    /// `scan_shared_capture_cells` owns the scope model).
    shared_cell_spans: BTreeSet<usize>,
    /// PLAN-909 §5.5: the enclosing top-level function's name and the
    /// per-owner lambda ordinal — together they mint the OWNER-STABLE
    /// synthetic identity `__closure_<owner>_<k>` (an unrelated owner's
    /// lambda insertion/removal never renames this owner's units).
    owner_name: String,
    closure_count: usize,
    /// PLAN-912 §5.2: creation-site facts sink (one entry per lambda this
    /// walk mints, in `closure_defs` order).
    lambda_sites: &'a mut Vec<LambdaSiteFacts>,
    /// PLAN-912 T-03: retained lambdas an owner re-resolve may reuse.
    reuse: Option<&'a mut LambdaReuse<'s>>,
    /// PLAN-934 T-04: the fact supply of a fact-driven bind (`None` = the
    /// legacy synthetic walk).
    supply: Option<&'a dyn TypeSupply>,
    /// PLAN-939 S06: the source fn count — the index boundary below which
    /// a `FnSig.index` addresses a SOURCE function (foreign declarations
    /// index above it); callback-by-name targets are source fns only.
    source_fn_count: usize,
    /// PLAN-1026: py-bridge token locals (hoisted `use.py` member imports
    /// and Python-op results bound by `let`/`var`) — sanctioned uses
    /// lower through the frozen py op set; anything else consuming the
    /// i32 token is a named `source.py-domain` rejection (R-04).
    py_locals: BTreeSet<LocalRef>,
    /// PLAN-1026: this program entered through the use.py front door (the
    /// py lane admits the console print intrinsics under Aggregates).
    py_context: bool,
    /// PLAN-1026: float locals whose initializer is an exact typed literal
    /// (folded into the frozen `F:` frame encoding at the call site).
    py_lit_locals: BTreeMap<LocalRef, (VT, u64)>,
    stmt_sinks: Vec<Vec<RStmt>>,
    ctrl_frames: Vec<CtrlFrame>,
    ctrl_state: Option<AggCtrlLocals>,
    catch_err_locals: Vec<LocalRef>,
    is_generator: bool,
    is_script: bool,
    iter_info: &'a BTreeMap<usize, (VT, VT, usize, usize, Option<usize>)>,
    scoped_iter_site: Option<Span>,
    agg_tmp_counter: usize,
    expr_diverged: bool,
    gen_try_finals: Vec<(LoopRef, LoopRef)>,
}

static EMPTY_ITER_INFO: BTreeMap<usize, (VT, VT, usize, usize, Option<usize>)> = BTreeMap::new();

#[derive(Clone, Copy, Debug)]
struct AggCtrlLocals {
    kind: LocalRef,
    ret_val: LocalRef,
    ret_opt_vi: Option<usize>,
    target_loop: LocalRef,
    err: LocalRef,
}

#[derive(Clone, Debug)]
enum CtrlFrame {
    Loop {
        target: LoopRef,
    },
    ForIter {
        target: LoopRef,
        iter_local: LocalRef,
        close_fn_index: usize,
    },
    Try {
        try_loop: LoopRef,
        has_catch: bool,
    },
}

#[allow(clippy::too_many_arguments)]
fn resolve_fn_agg(
    f: &FnItem,
    sigs: &BTreeMap<String, FnSig>,
    trace_enabled: bool,
    meta_enabled: bool,
    ct_enabled: bool,
    surface: SourceProfile,
    asm_lane: bool,
    diags: &mut Vec<Diagnostic>,
    receiver: ast::Receiver,
    owner_rec: Option<usize>,
    owner_var: Option<usize>,
    records: &[ResolvedRecord],
    rec_index: &BTreeMap<String, usize>,
    coll: &mut CollTables,
    closure_defs: &mut Vec<ResolvedFn>,
    lambda_sites: &mut Vec<LambdaSiteFacts>,
    closure_base: usize,
    supply: Option<&dyn TypeSupply>,
    is_generator: bool,
    iter_info: &BTreeMap<usize, (VT, VT, usize, usize, Option<usize>)>,
    gen_try_finals: Option<&mut Vec<(LoopRef, LoopRef)>>,
    sched: Option<&super::sched_desugar::SchedSurface>,
    nested: NestedCtx,
    instance_probe: Option<&instances::InstanceProbe<'_>>,
) -> ResolvedFn {
    resolve_fn_reuse(
        f,
        sigs,
        trace_enabled,
        meta_enabled,
        ct_enabled,
        surface,
        asm_lane,
        diags,
        receiver,
        owner_rec,
        owner_var,
        records,
        rec_index,
        coll,
        closure_defs,
        lambda_sites,
        closure_base,
        None,
        supply,
        is_generator,
        iter_info,
        gen_try_finals,
        sched,
        nested,
        instance_probe,
    )
}

fn resolve_fn(
    f: &FnItem,
    sigs: &BTreeMap<String, FnSig>,
    trace_enabled: bool,
    meta_enabled: bool,
    ct_enabled: bool,
    surface: SourceProfile,
    asm_lane: bool,
    diags: &mut Vec<Diagnostic>,
    receiver: ast::Receiver,
    owner_rec: Option<usize>,
    owner_var: Option<usize>,
    records: &[ResolvedRecord],
    rec_index: &BTreeMap<String, usize>,
    coll: &mut CollTables,
    closure_defs: &mut Vec<ResolvedFn>,
    lambda_sites: &mut Vec<LambdaSiteFacts>,
    closure_base: usize,
    supply: Option<&dyn TypeSupply>,
) -> ResolvedFn {
    resolve_fn_reuse(
        f,
        sigs,
        trace_enabled,
        meta_enabled,
        ct_enabled,
        surface,
        asm_lane,
        diags,
        receiver,
        owner_rec,
        owner_var,
        records,
        rec_index,
        coll,
        closure_defs,
        lambda_sites,
        closure_base,
        None,
        supply,
        false,
        &EMPTY_ITER_INFO,
        None,
        None,
        NestedCtx::default(),
        None,
    )
}

/// [`resolve_fn`] with an optional retained-lambda reuse oracle (PLAN-912
/// T-03): the owner's lambdas whose text and creation-point facts are
/// unchanged are pushed from the oracle instead of being walked.
#[allow(clippy::too_many_arguments)]
fn resolve_fn_reuse<'a, 's>(
    f: &FnItem,
    sigs: &'a BTreeMap<String, FnSig>,
    trace_enabled: bool,
    meta_enabled: bool,
    ct_enabled: bool,
    surface: SourceProfile,
    asm_lane: bool,
    diags: &'a mut Vec<Diagnostic>,
    receiver: ast::Receiver,
    owner_rec: Option<usize>,
    owner_var: Option<usize>,
    records: &'a [ResolvedRecord],
    rec_index: &'a BTreeMap<String, usize>,
    coll: &'a mut CollTables,
    closure_defs: &'a mut Vec<ResolvedFn>,
    lambda_sites: &'a mut Vec<LambdaSiteFacts>,
    _closure_base: usize,
    reuse: Option<&'a mut LambdaReuse<'s>>,
    supply: Option<&'a dyn TypeSupply>,
    is_generator: bool,
    iter_info: &'a BTreeMap<usize, (VT, VT, usize, usize, Option<usize>)>,
    gen_try_finals: Option<&mut Vec<(LoopRef, LoopRef)>>,
    sched: Option<&'a super::sched_desugar::SchedSurface>,
    nested: NestedCtx,
    instance_probe: Option<&'a instances::InstanceProbe<'a>>,
) -> ResolvedFn {
    let is_coll = surface.is_collections();
    // PLAN-1083: an enum instance method's owner name (precomputed —
    // coll moves into the FnResolver below).
    let var_owner_name: Option<String> =
        owner_var.map(|vi| coll.variants[vi].name.clone());
    let owned_spec_result = owner_var.is_some_and(|index| coll.spec_variants.contains(&index));
    let fn_result = if is_coll {
        vt_of_ann_coll(&f.ret, rec_index, coll, diags, TypePos::Result)
    } else {
        vt_of_ann(&f.ret, rec_index, diags)
    };
    let nested_owner_name = nested
        .sig_key
        .clone()
        .unwrap_or_else(|| f.name.name.clone());
    let mut r = FnResolver {
        instance_probe,
        sigs,
        trace_enabled,
        meta_enabled,
        ct_enabled,
        ct_depth: 0,
        profile: surface,
        asm_lane,
        nested,
        diags,
        locals: Vec::new(),
        scopes: vec![Scope {
            names: BTreeMap::new(),
            views: BTreeMap::new(),
            text_mut_aliases: BTreeMap::new(),
        }],
        declared_names: BTreeSet::new(),
        implicit_record_fields: surface.is_auto_language() && receiver.is_instance() && owner_rec.is_some(),
        owned_spec_result,
        fn_result,
        legacy_text_result: surface.is_auto_language() && is_legacy_text_list(&f.ret),
        legacy_text_lists: BTreeSet::new(),
        legacy_text_find_results: BTreeMap::new(),
        loop_stack: Vec::new(),
        loop_count: 0,
        used_intrinsics: BTreeSet::new(),
        closure_defs,
        moved_locals: BTreeSet::new(),
        py_locals: BTreeSet::new(),
        py_context: supply
            .map(|s| {
                s.foreign()
                    .map(|f| f.manifest().provider == "py")
                    .unwrap_or(false)
            })
            .unwrap_or(false),
        py_lit_locals: BTreeMap::new(),
        borrowed_closure_locals: BTreeSet::new(),
        records,
        rec_index,
        coll,
        iterate_targets: Vec::new(),
        for_latches: Vec::new(),
        shared_cell_spans: super::closure_capture::scan_shared_capture_cells(&f.params, &f.body),
        // PLAN-1081 (T-07 F-1): a nested item's lambdas qualify their wire
        // names with the MANGLED owner — same-name nested items under
        // different owners would otherwise mint colliding
        // `__closure_<bare>_<k>` defs (atom duplicate-id).
        owner_name: nested_owner_name,
        closure_count: 0,
        lambda_sites,
        reuse,
        supply,
        source_fn_count: _closure_base,
        stmt_sinks: vec![Vec::new()],
        ctrl_frames: Vec::new(),
        ctrl_state: None,
        catch_err_locals: Vec::new(),
        is_generator,
        is_script: f.script,
        // PLAN-968: the scheduler surface side-table (scheduler face only).
        sched,
        iter_info,
        scoped_iter_site: None,
        agg_tmp_counter: 0,
        expr_diverged: false,
        gen_try_finals: Vec::new(),
    };
    let mut param_names: Vec<String> = Vec::new();
    let mut params: Vec<VT> = Vec::new();
    let mut param_modes: Vec<bool> = Vec::new();
    // Params live in the function's outermost scope. A view receiver is
    // the implicit `self` param 0 (PLAN-771 §5.3).
    if receiver.is_instance() {
        // PLAN-1083: an enum instance method's receiver is the VARIANT.
        let self_vt = match owner_var {
            Some(vi) => VT::Variant(vi),
            None => VT::Rec(owner_rec.unwrap_or(usize::MAX)),
        };
        let lref = LocalRef(r.locals.len());
        r.locals.push(LocalDecl {
            name: "self".to_string(),
            vt: self_vt,
            mutable: receiver == ast::Receiver::Mut,
            is_param: true,
            param_index: Some(0),
            capture_index: None,
            cell: false,
            is_capture: false,
            mode: if receiver == ast::Receiver::Mut {
                ast::ParamMode::Mut
            } else {
                ast::ParamMode::Default
            },
            span: f.name.span,
        });
        r.scopes[0].names.insert("self".to_string(), lref);
        r.declared_names.insert("self".to_string());
        param_names.push("self".to_string());
        params.push(self_vt);
        param_modes.push(receiver == ast::Receiver::Mut);
    }
    for (i, p) in f.params.iter().enumerate() {
        if rec_index.contains_key(&p.name.name) {
            r.bind_err(
                "source.duplicate",
                format!(
                    "parameter `{}` shadows the record type of the same name; rename the parameter",
                    p.name.name
                ),
                p.name.span,
            );
        }
        let vt = if is_coll {
            vt_of_ann_coll(&p.ty, rec_index, r.coll, &mut Vec::new(), TypePos::Param)
        } else {
            vt_of_ann(&p.ty, rec_index, &mut Vec::new())
        };
        // mut mode is meaningful for record params (774 D-04) AND text
        // `mut str`/`mut Str` exclusive borrows (PLAN-770); scalar params
        // are by value on every profile
        if p.mode == ast::ParamMode::Mut && !matches!(vt, VT::Rec(_) | VT::Str | VT::StrOwned) {
            r.bind_err(
                "source.type-mismatch",
                format!(
                    "`mut` parameter `{}` requires a record or text type; i32/bool params are passed by value",
                    p.name.name
                ),
                p.span,
            );
        }
        let lref = LocalRef(r.locals.len());
        r.locals.push(LocalDecl {
            name: p.name.name.clone(),
            vt,
            mutable: p.mode == ast::ParamMode::Mut,
            is_param: true,
            mode: p.mode,
            param_index: Some((i + if receiver.is_instance() { 1 } else { 0 }) as u32),
            capture_index: None,
            cell: false,
            is_capture: false,
            span: p.span,
        });
        r.scopes[0].names.insert(p.name.name.clone(), lref);
        r.declared_names.insert(p.name.name.clone());
        param_names.push(p.name.name.clone());
        params.push(vt);
        param_modes.push(p.mode == ast::ParamMode::Mut);
    }
    let mut body = r.resolve_body(&f.body, true);
    r.prepend_ctrl_inits(&mut body, f.span);
    if let Some(out) = gen_try_finals {
        *out = r.gen_try_finals;
    }

    let sig_key = if receiver == ast::Receiver::Free {
        f.name.name.clone()
    } else {
        format!(
            "{}::{}",
            owner_rec
                .map(|oi| records[oi].name.clone())
                .or(var_owner_name)
                .unwrap_or_default(),
            f.name.name
        )
    };
    // PLAN-1081: a nested fn item's signature lives under its MANGLED
    // key — the bare name would miss and silently default the result.
    let sig = match r.nested.sig_key.as_deref() {
        Some(mangled) => sigs.get(mangled),
        None => sigs.get(&sig_key),
    };
    ResolvedFn {
        name: f.name.name.clone(),
        name_span: f.name.span,
        item_span: f.span,
        param_names: if receiver.is_instance() {
            param_names
        } else {
            sig.map(|g| g.param_names.clone()).unwrap_or(param_names)
        },
        params: if receiver.is_instance() {
            params
        } else {
            sig.map(|g| g.params.clone()).unwrap_or(params)
        },
        param_modes,
        result: sig.map(|g| g.result).unwrap_or(VT::I32),
        locals: r.locals,
        body,
        loop_count: r.loop_count,
        used_intrinsics: r.used_intrinsics,
        receiver,
        owner_rec,
        owner_var,
    }
}

impl<'a, 's> FnResolver<'a, 's> {
    fn bind_err(&mut self, code: &str, msg: impl Into<String>, span: Span) {
        self.diags
            .push(Diagnostic::new(Stage::Bind, code, msg, span));
    }

    /// PLAN-934 T-04: the installed type of one expression node.
    ///
    /// Legacy bind: the resolver's own rule produces the type (metered as
    /// synthesis). Fact-driven bind: the query's fact is authoritative and
    /// `rule` is only the expectation — a mismatch or a missing fact is a
    /// BIND-STAGE rejection (before any HIR exists).
    fn node_ty(&mut self, span: Span, rule: VT) -> VT {
        match self.supply {
            None => {
                crate::type_synth::record_synth(&self.owner_name);
                rule
            }
            Some(s) if !s.facts_enabled() => {
                crate::type_synth::record_synth(&self.owner_name);
                rule
            }
            Some(s) => {
                let facts = s.expr_tys(span);
                let Some(fact) = facts
                    .iter()
                    .copied()
                    .find(|f| *f == rule)
                    .or_else(|| facts.first().copied())
                else {
                    crate::type_synth::record_missing(&self.owner_name);
                    self.bind_err(
                        "source.fact-missing",
                        "no type fact for this expression node",
                        span,
                    );
                    return rule;
                };
                crate::type_synth::record_fact(&self.owner_name);
                if fact != rule {
                    self.bind_err(
                        "source.fact-mismatch",
                        format!(
                            "type fact says {} but the binding rule for this node is {}",
                            fact.as_str(),
                            rule.as_str()
                        ),
                        span,
                    );
                }
                fact
            }
        }
    }

    /// PLAN-934 T-04: the installed type of one `let`/`var` declaration.
    fn binding_ty(&mut self, span: Span, rule: VT) -> VT {
        match self.supply {
            None => {
                crate::type_synth::record_synth(&self.owner_name);
                rule
            }
            Some(s) if !s.facts_enabled() => {
                crate::type_synth::record_synth(&self.owner_name);
                rule
            }
            Some(s) => match s.binding_ty(span) {
                Some(vt) => {
                    crate::type_synth::record_fact(&self.owner_name);
                    if vt != rule {
                        self.bind_err(
                            "source.fact-mismatch",
                            format!(
                                "binding fact says {} but the declaration's rule is {}",
                                vt.as_str(),
                                rule.as_str()
                            ),
                            span,
                        );
                    }
                    vt
                }
                None => {
                    crate::type_synth::record_missing(&self.owner_name);
                    self.bind_err(
                        "source.fact-missing",
                        "no binding fact for this declaration",
                        span,
                    );
                    rule
                }
            },
        }
    }

    // ---- PLAN-1026: the Python op sugar ----------------------------------

    /// The foreign declaration name a `CallFn` index addresses (None for
    /// source fns / out-of-range).
    fn py_foreign_name(&self, fn_index: usize) -> Option<&str> {
        let i = fn_index
            .checked_sub(agg_lower::FOREIGN_INDEX_SENTINEL)
            .or_else(|| fn_index.checked_sub(self.source_fn_count))?;
        self.supply
            .and_then(|s| s.foreign())
            .and_then(|f| f.source_declarations_ref().get(i))
            .map(|d| d.name.as_str())
    }

    /// A py OBJECT value: a read of a marked local or an inline result of
    /// an object-producing Python op.
    fn py_is_object(&self, e: &ResolvedExpr) -> bool {
        match &e.kind {
            RExprKind::ReadLocal(l) => self.py_locals.contains(l),
            RExprKind::CallFn { fn_index, .. } => matches!(
                self.py_foreign_name(*fn_index),
                Some("py_import" | "py_call" | "py_getattr" | "py_getitem" | "py_binop")
            ),
            _ => false,
        }
    }

    /// PLAN-1107 T-02: the admitted value class of one operand — the same
    /// classification the frame encoder applies (`py_frame_val`), so the
    /// recorded fact and the wire frame can never disagree about an operand.
    fn py_val_class(&self, v: &ResolvedExpr) -> PyValClass {
        if self.py_is_object(v) {
            return PyValClass::Object;
        }
        match &v.kind {
            RExprKind::StrLit(_) => PyValClass::Text,
            RExprKind::BoolLit(_) => PyValClass::Bool,
            RExprKind::TypedLit {
                vt: VT::F64 | VT::F32,
                ..
            } => PyValClass::Float,
            RExprKind::ReadLocal(l)
                if matches!(v.vt, VT::F64 | VT::F32) && self.py_lit_locals.contains_key(l) =>
            {
                PyValClass::Float
            }
            _ if v.vt == VT::I32 => PyValClass::Int,
            _ if v.vt == VT::Bool => PyValClass::Bool,
            _ => PyValClass::Other,
        }
    }

    /// PLAN-1107 T-02: the exact literal payload an operand bakes into the AV
    /// frame text (`i`/`b`/`t`/`F:` encodings), carried in the fact so a
    /// source-generating target never has to decode frame text.
    fn py_operand_literal(&self, v: &ResolvedExpr) -> Option<PyLiteral> {
        match &v.kind {
            RExprKind::IntLit(n) => Some(PyLiteral::Int(*n)),
            RExprKind::BoolLit(b) => Some(PyLiteral::Bool(*b)),
            RExprKind::StrLit(t) => Some(PyLiteral::Text(t.clone())),
            RExprKind::TypedLit { vt: VT::F64, bits } => {
                Some(PyLiteral::Float(f64::from_bits(*bits)))
            }
            RExprKind::TypedLit { vt: VT::F32, bits } => {
                Some(PyLiteral::Float(f32::from_bits(*bits as u32) as f64))
            }
            RExprKind::ReadLocal(l) if matches!(v.vt, VT::F64 | VT::F32) => {
                self.py_lit_locals.get(l).map(|(vt, bits)| {
                    PyLiteral::Float(if *vt == VT::F64 {
                        f64::from_bits(*bits)
                    } else {
                        f32::from_bits(*bits as u32) as f64
                    })
                })
            }
            _ => None,
        }
    }

    /// Call a foreign py op (or a helper fn) by name with ONE positional
    /// argument, typed by the callee's declared result.
    fn py_op_call(&mut self, op: &str, span: Span, value: ResolvedExpr) -> ResolvedExpr {
        let fn_index = self.sigs.get(op).map(|sig| sig.index).unwrap_or(usize::MAX);
        let result = self.sigs.get(op).map(|sig| sig.result).unwrap_or(VT::I32);
        ResolvedExpr {
            span,
            vt: self.node_ty(span, result),
            kind: RExprKind::CallFn {
                fn_index,
                args: vec![ResolvedArg {
                    value,
                    name: None,
                    moved: false,
                    span,
                }],
            },
        }
    }

    /// PLAN-1107: the ONE-evaluation operand chunk. `__py_chunk_<tag>(v)`
    /// renders `"<tag> <dlen>\n<v>\n"` from a single evaluation of `v`, so an
    /// inline py operation (which mints a fresh token per evaluation) can
    /// never desync the declared payload length from the payload.
    fn py_chunk_call(&mut self, tag: &str, v: &ResolvedExpr) -> ResolvedExpr {
        let name = match tag {
            "o" => "__py_chunk_o",
            "b" => "__py_chunk_b",
            // PLAN-1094: dynamic text crosses via the byte-length chunk.
            "t" => "__py_chunk_t",
            _ => "__py_chunk_i",
        };
        let span = v.span;
        self.py_op_call(name, span, v.clone())
    }

    /// The runtime frame: an FStr of static text parts + I32 displays,
    /// then `.to(str)` (the view param form).
    fn py_frame(&mut self, span: Span, parts: Vec<PyFramePart>) -> ResolvedExpr {
        let mut fparts: Vec<FStrPart> = Vec::with_capacity(parts.len() * 2);
        for p in parts {
            match p {
                PyFramePart::Txt(t) => fparts.push(FStrPart::Text(ResolvedExpr {
                    span,
                    vt: VT::Str,
                    kind: RExprKind::StrLit(t),
                })),
                PyFramePart::Disp(e) => fparts.push(FStrPart::Expr(ResolvedExpr {
                    span,
                    vt: VT::StrOwned,
                    kind: RExprKind::TextDisplay {
                        operand: Box::new(e),
                    },
                })),
                PyFramePart::TextExpr(e) => fparts.push(FStrPart::Expr(e)),
            }
        }
        let fstr = ResolvedExpr {
            span,
            vt: VT::StrOwned,
            kind: RExprKind::FStr { parts: fparts },
        };
        ResolvedExpr {
            span,
            vt: VT::Str,
            kind: RExprKind::Method {
                recv: Box::new(fstr),
                method: BuiltinMethod::To,
                to: Some(VT::Str),
                args: Vec::new(),
            },
        }
    }

    /// The wire encoding of one frame VALUE appended to `parts`.
    fn py_frame_val(&mut self, parts: &mut Vec<PyFramePart>, v: &ResolvedExpr) {
        match &v.kind {
            // exact int literal — static "i <len>\n<payload>\n"
            RExprKind::IntLit(n) => {
                let d = n.to_string();
                parts.push(PyFramePart::Txt(format!("i {}\n{d}\n", d.len())));
            }
            // exact bool literal
            RExprKind::BoolLit(b) => {
                let d = if *b { "1" } else { "0" };
                parts.push(PyFramePart::Txt(format!(
                    "b 1
{d}
"
                )));
            }
            // exact text literal — raw payload with its byte length
            RExprKind::StrLit(t) => {
                parts.push(PyFramePart::Txt(format!("t {}\n{t}\n", t.len())));
            }
            // PLAN-1026: an exact float literal — the frozen `F:<decimal>`
            // text encoding (compile-time; the provider parses it back to
            // an exact Python float; dynamic float args stay v1-rejected).
            RExprKind::TypedLit { vt: VT::F64, bits } => {
                let d = f64::from_bits(*bits).to_string();
                parts.push(PyFramePart::Txt(format!(
                    "t {}
F:{d}
",
                    d.len() + 2
                )));
            }
            RExprKind::TypedLit { vt: VT::F32, bits } => {
                let d = (f32::from_bits(*bits as u32) as f64).to_string();
                parts.push(PyFramePart::Txt(format!(
                    "t {}
F:{d}
",
                    d.len() + 2
                )));
            }
            // PLAN-1026: a float local bound to an exact literal — the
            // frozen `F:` compile-time encoding (the ONLY dynamic-float
            // form v1 admits).
            RExprKind::ReadLocal(l)
                if matches!(v.vt, VT::F64 | VT::F32) && self.py_lit_locals.contains_key(l) =>
            {
                let (vt, bits) = self.py_lit_locals[l];
                let d = if vt == VT::F64 {
                    f64::from_bits(bits).to_string()
                } else {
                    (f32::from_bits(bits as u32) as f64).to_string()
                };
                let payload = format!("F:{d}");
                parts.push(PyFramePart::Txt(format!(
                    "t {}
{}
",
                    payload.len(),
                    payload
                )));
            }
            // a py object — dynamic "o <dlen>\n<token>\n". PLAN-1107: ONE
            // evaluation through the chunk helper (the operand may be an
            // inline py operation; evaluating it twice minted two tokens and
            // desynced the declared length from the payload).
            _ if self.py_is_object(v) => {
                let chunk = self.py_chunk_call("o", v);
                parts.push(PyFramePart::TextExpr(chunk));
            }
            // any other i32 dynamic value — dynamic "i <dlen>\n<value>\n"
            _ if v.vt == VT::I32 => {
                let chunk = self.py_chunk_call("i", v);
                parts.push(PyFramePart::TextExpr(chunk));
            }
            // PLAN-1094: nil crosses as Python None (the `with` exit args).
            RExprKind::NilLit => {
                parts.push(PyFramePart::Txt("n 0\n\n".to_string()));
            }
            // PLAN-1094: dynamic text (the `with` error args) crosses via
            // the byte-length text chunk.
            _ if v.vt.is_text() => {
                let chunk = self.py_chunk_call("t", v);
                parts.push(PyFramePart::TextExpr(chunk));
            }
            // v1 boundary: dynamic floats/text/other types do not cross
            _ => {
                self.bind_err(
                    "source.py-domain",
                    "this value kind cannot cross into a Python call (v1: int/bool/text literals and Python objects)",
                    v.span,
                );
                parts.push(PyFramePart::Txt("n 0\n\n".to_string()));
            }
        }
    }

    /// The py_call sugar: receiver + optional member label + source-order
    /// args + named kwargs, one frame, one foreign call.
    fn py_call_sugar(
        &mut self,
        recv: &ResolvedExpr,
        member: &str,
        args: Vec<ResolvedArg>,
        span: Span,
    ) -> ResolvedExpr {
        let mut parts: Vec<PyFramePart> = Vec::new();
        parts.push(PyFramePart::Txt("p967.frame/1\nr ".to_string()));
        if self.py_is_object(recv) {
            let chunk = self.py_chunk_call("o", &recv);
            parts.push(PyFramePart::TextExpr(chunk));
        } else {
            self.bind_err(
                "source.py-domain",
                "the call receiver must be a Python object",
                recv.span,
            );
            parts.push(PyFramePart::Txt("n 0\n\n".to_string()));
        }
        // positionals[0] is ALWAYS the member label ("" for a direct call
        // of the receiver itself — the provider's frozen ABI).
        let pos = args.iter().filter(|a| a.name.is_none()).count();
        parts.push(PyFramePart::Txt(format!(
            "p {}
",
            pos + 1
        )));
        parts.push(PyFramePart::Txt(format!(
            "t {}
{member}
",
            member.len()
        )));
        for a in &args {
            if a.name.is_none() {
                self.py_frame_val(&mut parts, &a.value);
            }
        }
        let kw = args.iter().filter(|a| a.name.is_some()).count();
        parts.push(PyFramePart::Txt(format!("k {kw}\n")));
        for a in &args {
            if let Some(n) = &a.name {
                // kwarg: namelen LF + name ABUTTING the value's header
                parts.push(PyFramePart::Txt(format!("{}\n{}", n.name.len(), n.name)));
                self.py_frame_val(&mut parts, &a.value);
            }
        }
        // PLAN-1107 T-02: record the operation where it is admitted.
        if crate::py_ops::recording_active() {
            let mut fact = PyOpFact::new(PyOpKind::Call, span).with_member(member);
            fact = fact.operand_lit(
                PyRole::Receiver,
                recv.span,
                self.py_val_class(recv),
                self.py_operand_literal(recv),
            );
            let mut slot = 0u32;
            for a in &args {
                if a.name.is_none() {
                    fact = fact.operand_lit(
                        PyRole::Positional(slot),
                        a.value.span,
                        self.py_val_class(&a.value),
                        self.py_operand_literal(&a.value),
                    );
                    slot += 1;
                }
            }
            for a in &args {
                if let Some(n) = &a.name {
                    fact = fact.operand_lit(
                        PyRole::Kwarg(n.name.clone()),
                        a.value.span,
                        self.py_val_class(&a.value),
                        self.py_operand_literal(&a.value),
                    );
                }
            }
            crate::py_ops::record(fact);
        }
        let frame = self.py_frame(span, parts);
        self.py_op_call("py_call", span, frame)
    }

    /// The `.to(...)` extraction sugar on a py object.
    fn py_to_sugar(&mut self, recv: &ResolvedExpr, target: VT, span: Span) -> ResolvedExpr {
        // PLAN-1107 T-02: the extraction operation (the `py_f64_lo` low-word
        // lane is the continuation of the same `.to(float)` extraction and is
        // not a second operation).
        if crate::py_ops::recording_active() {
            let word = match target {
                VT::F64 => Some((PyOpKind::ToFloat, "double")),
                VT::F32 => Some((PyOpKind::ToFloat, "float")),
                VT::I32 => Some((PyOpKind::ToInt, "int")),
                VT::Bool => Some((PyOpKind::ToBool, "bool")),
                VT::Str | VT::StrOwned => Some((PyOpKind::ToText, "str")),
                _ => None,
            };
            if let Some((kind, word)) = word {
                crate::py_ops::record(PyOpFact::new(kind, span).with_member(word).operand_lit(
                    PyRole::Receiver,
                    recv.span,
                    self.py_val_class(recv),
                    self.py_operand_literal(recv),
                ));
            }
        }
        match target {
            VT::F64 | VT::F32 => {
                let frame = self.py_recv_frame(recv, span);
                let hi = self.py_op_call("py_to_f64", span, frame);
                let lo_frame = self.py_frame(
                    span,
                    vec![PyFramePart::Txt(
                        "p967.frame/1\nr n 0\n\np 0\nk 0\n".to_string(),
                    )],
                );
                let lo = self.py_op_call("py_f64_lo", span, lo_frame);
                self.used_intrinsics.insert("__py_f64_bits");
                let v = ResolvedExpr {
                    span,
                    vt: self.node_ty(span, VT::F64),
                    kind: RExprKind::CallIntrinsic {
                        symbol: "__py_f64_bits",
                        args: vec![hi, lo],
                    },
                };
                if target == VT::F32 {
                    ResolvedExpr {
                        span,
                        vt: self.node_ty(span, VT::F32),
                        kind: RExprKind::Method {
                            recv: Box::new(v),
                            method: BuiltinMethod::NumTo,
                            to: Some(VT::F32),
                            args: Vec::new(),
                        },
                    }
                } else {
                    v
                }
            }
            VT::I32 => {
                let frame = self.py_recv_frame(recv, span);
                self.py_op_call("py_to_i32", span, frame)
            }
            VT::Bool => {
                let frame = self.py_recv_frame(recv, span);
                // the bridge returns a 0/1 i32 — coerce to the Bool face.
                let raw = self.py_op_call("py_to_bool", span, frame);
                ResolvedExpr {
                    span,
                    vt: self.node_ty(span, VT::Bool),
                    kind: RExprKind::Binary {
                        op: BinOp::Ne,
                        lhs: Box::new(raw),
                        rhs: Box::new(ResolvedExpr {
                            span,
                            vt: VT::I32,
                            kind: RExprKind::IntLit(0),
                        }),
                    },
                }
            }
            VT::Str | VT::StrOwned => {
                let len = self.py_stage_len(recv, span);
                self.used_intrinsics.insert("__py_staged_text");
                ResolvedExpr {
                    span,
                    vt: self.node_ty(span, VT::StrOwned),
                    kind: RExprKind::CallIntrinsic {
                        symbol: "__py_staged_text",
                        args: vec![len],
                    },
                }
            }
            other => {
                self.bind_err(
                    "source.py-domain",
                    format!(
                        "`.to({})` is not a supported Python extraction (v1: float, double, int, str)",
                        other.as_str()
                    ),
                    span,
                );
                ResolvedExpr {
                    span,
                    vt: VT::Unit,
                    kind: RExprKind::Unit,
                }
            }
        }
    }

    /// The receiver frame prefix + this object's token (shared by the
    /// extraction ops).
    fn py_recv_frame(&mut self, recv: &ResolvedExpr, span: Span) -> ResolvedExpr {
        let mut parts: Vec<PyFramePart> = Vec::new();
        parts.push(PyFramePart::Txt("p967.frame/1\nr ".to_string()));
        if self.py_is_object(recv) {
            let chunk = self.py_chunk_call("o", &recv);
            parts.push(PyFramePart::TextExpr(chunk));
            parts.push(PyFramePart::Txt("p 0\nk 0\n".to_string()));
        } else {
            self.bind_err(
                "source.py-domain",
                "the operation receiver must be a Python object",
                recv.span,
            );
            parts.push(PyFramePart::Txt("n 0\n\np 0\nk 0\n".to_string()));
        }
        self.py_frame(span, parts)
    }

    /// py_stage_str on an object — returns the staged length (i32).
    fn py_stage_len(&mut self, recv: &ResolvedExpr, span: Span) -> ResolvedExpr {
        let frame = self.py_recv_frame(recv, span);
        self.py_op_call("py_stage_str", span, frame)
    }

    fn lookup(&self, name: &str) -> Option<LocalRef> {
        for scope in self.scopes.iter().rev() {
            if let Some(l) = scope.names.get(name) {
                return Some(*l);
            }
            if scope.text_mut_aliases.contains_key(name) {
                return None;
            }
        }
        None
    }

    fn lookup_text_mut_alias(&self, name: &str) -> Option<LocalRef> {
        for scope in self.scopes.iter().rev() {
            if scope.names.contains_key(name) || scope.views.contains_key(name) {
                return None;
            }
            if let Some(root) = scope.text_mut_aliases.get(name) {
                return Some(*root);
            }
        }
        None
    }

    fn bind_text_mut_alias(&mut self, alias: &ast::Ident, root: LocalRef, span: Span) {
        if !self.locals[root.0].mutable {
            self.bind_err("source.immutable", "text .mut needs a mutable owner", span);
            return;
        }
        let scope = self.scopes.last_mut().expect("binding scope");
        if scope.names.contains_key(&alias.name) || scope.text_mut_aliases.contains_key(&alias.name)
        {
            self.bind_err(
                "source.duplicate",
                format!(
                    "local {} is declared more than once in this block",
                    alias.name
                ),
                alias.span,
            );
            return;
        }
        scope.text_mut_aliases.insert(alias.name.clone(), root);
        self.declared_names.insert(alias.name.clone());
    }

    fn resolve_text_append(
        &mut self,
        root: LocalRef,
        recv: &Expr,
        args: &[Arg],
        span: Span,
    ) -> ResolvedExpr {
        let [Arg::Pos(arg)] = args else {
            self.bind_err(
                "source.type-mismatch",
                "text append expects one positional text argument",
                span,
            );
            return ResolvedExpr {
                span,
                vt: VT::Unit,
                kind: RExprKind::Unit,
            };
        };
        let value = self.expr_iterative(arg);
        if !value.vt.is_text() {
            self.bind_err(
                "source.type-mismatch",
                "text append expects a text argument",
                value.span,
            );
            return ResolvedExpr {
                span,
                vt: VT::Unit,
                kind: RExprKind::Unit,
            };
        }
        self.emit_stmt(RStmt::Assign {
            local: root,
            path: Vec::new(),
            value: ResolvedExpr {
                span,
                vt: VT::StrOwned,
                kind: RExprKind::Binary {
                    op: BinOp::Add,
                    lhs: Box::new(ResolvedExpr {
                        span: recv.span,
                        vt: VT::StrOwned,
                        kind: RExprKind::ReadLocal(root),
                    }),
                    rhs: Box::new(value),
                },
            },
            span,
        });
        ResolvedExpr {
            span,
            vt: VT::Unit,
            kind: RExprKind::Unit,
        }
    }

    /// PLAN-864: view-binding lookup (owned-element for-in bindings).
    fn lookup_view(&self, name: &str) -> Option<ViewBinding> {
        for scope in self.scopes.iter().rev() {
            if let Some(v) = scope.views.get(name) {
                return Some(*v);
            }
        }
        None
    }

    fn infer_closure_expr(&self, expr: &Expr, locals: &BTreeMap<String, VT>) -> VT {
        match &expr.kind {
            // PLAN-936（T-04）：HashExpr 的类型 = inner 的类型（closures 面
            // 不产 HashExpr；本臂仅为穷尽性——inner 递归即可）。
            ExprKind::HashExpr { expr: inner } => self.infer_closure_expr(inner, locals),
            // PLAN-968: await preserves the future's value type; go yields no
            // value (Unit) but its operand still infers for diagnostics.
            ExprKind::Await(inner) => self.infer_closure_expr(inner, locals),
            ExprKind::Go(inner) => {
                let _ = self.infer_closure_expr(inner, locals);
                VT::Unit
            }
            // PLAN-962: a range is only a for-in iterable, never a value.
            ExprKind::Range { .. } => VT::I32,
            // PLAN-962: literals/indexes never reach closure inference
            // (statement-level desugar owns them); totals for
            // exhaustiveness.
            ExprKind::ArrayLit(_) | ExprKind::ArrayConditionalElem { .. } => VT::I32,
            // PLAN-963（T-04）：值实参调用由实例层重绑；closures 面不产
            // ConstCall，本臂仅为穷尽性。
            ExprKind::ConstCall { .. } => VT::I32,
            ExprKind::Index { .. } => VT::I32,
            // PLAN-962 (T-08): a grid never reaches closure inference.
            ExprKind::GridLit { .. } => VT::I32,
            // PLAN-962: tuples resolve through the statement walker.
            ExprKind::TupleLit(_) => VT::I32,
            ExprKind::IntLit(_) => VT::I32,
            // PLAN-961: the typed literal carries its own type.
            ExprKind::TypedLit { ty, .. } => vt_of_word(*ty),
            ExprKind::BoolLit(_) => VT::Bool,
            ExprKind::StrLit(_) => VT::Str,
            ExprKind::CharLit(_) => VT::Char,
            ExprKind::Var(name) => locals.get(&name.name).copied().unwrap_or(VT::I32),
            ExprKind::Binary { op, lhs, rhs } => match op {
                BinOp::Add
                    if self.infer_closure_expr(lhs, locals).is_text()
                        && self.infer_closure_expr(rhs, locals).is_text() =>
                {
                    VT::StrOwned
                }
                _ => super::resolve::bin_op_vt(*op),
            },
            ExprKind::Unary { op, .. } => match op {
                ast::UnaryOp::Neg => VT::I32,
                ast::UnaryOp::Not => VT::Bool,
            },
            ExprKind::Call { callee, .. } => self
                .sigs
                .get(&callee.name)
                .map(|sig| sig.result)
                .or_else(|| self.rec_index.get(&callee.name).map(|ri| VT::Rec(*ri)))
                .unwrap_or(VT::I32),
            ExprKind::CallValue { callee, .. } => match self.infer_closure_expr(callee, locals) {
                VT::Function(index) => self
                    .coll
                    .function_types
                    .get(index)
                    .map(|sig| sig.result)
                    .unwrap_or(VT::I32),
                _ => VT::I32,
            },
            ExprKind::Method { recv, name, .. } | ExprKind::MethodCall { recv, name, .. } => {
                let recv = self.infer_closure_expr(recv, locals);
                super::resolve::builtin_method(recv, &name.name)
                    .map(|(_, _, result)| result)
                    .unwrap_or(VT::I32)
            }
            ExprKind::Field { base, name } => {
                let VT::Rec(ri) = self.infer_closure_expr(base, locals) else {
                    return VT::I32;
                };
                self.records
                    .get(ri)
                    .and_then(|record| record.fields.iter().find(|f| f.name == name.name))
                    .map(|field| field.vt)
                    .unwrap_or(VT::I32)
            }
            ExprKind::CtorNamed { ty, .. } => self
                .rec_index
                .get(&ty.name)
                .map(|ri| VT::Rec(*ri))
                .unwrap_or(VT::I32),
            ExprKind::TypeWordArg(word) => vt_of_word(*word),
            // PLAN-964: nil is its own type; object/display/FStr fall back
            // to the exact-resolution walk (inference is a light oracle).
            ExprKind::NilLit => VT::Nil,
            ExprKind::Closure { .. }
            | ExprKind::MutArg { .. }
            | ExprKind::ViewArg { .. }
            | ExprKind::GenericType { .. }
            | ExprKind::ObjectLit { .. }
            | ExprKind::TextDisplay(_) => VT::I32,
            ExprKind::Propagate { inner } | ExprKind::UnwrapOr { inner, .. } => {
                let ivt = self.infer_closure_expr(inner, locals);
                agg_lower::variant_unwrap_info(ivt, self.coll)
                    .map(|u| u.success_vt)
                    .unwrap_or(VT::I32)
            }
            ExprKind::IsExpr { arms, .. } => {
                for a in arms {
                    if let ast::AggArmBody::Expr(e) = &a.body {
                        return self.infer_closure_expr(e, locals);
                    }
                }
                VT::I32
            }
        }
    }

    fn infer_closure_block_result(
        &mut self,
        block: &ast::Block,
        locals: &BTreeMap<String, VT>,
    ) -> VT {
        let mut blocks = vec![(block, locals.clone())];
        while let Some((block, mut visible)) = blocks.pop() {
            for stmt in &block.stmts {
                match stmt {
                    Stmt::Return { value, .. } => return self.infer_closure_expr(value, &visible),
                    // PLAN-1081: a nested fn item binds no value here.
                    Stmt::NestedFn { .. } => {}
                    // PLAN-962: an index assign is an effect statement.
                    Stmt::IndexAssign(_) => {}
                    // PLAN-1082: a place assign is an effect statement.
                    Stmt::PlaceAssign(_) => {}
                    // PLAN-962 (T-08): hold — alias names the held local
                    // inside the body scope only.
                    Stmt::Hold {
                        name, bind, body, ..
                    } => {
                        let held = visible.get(&name.name).copied().unwrap_or(VT::I32);
                        visible.insert(bind.name.clone(), held);
                        blocks.push((body, visible.clone()));
                    }
                    // PLAN-962: tuple destructuring binds names.
                    Stmt::LetDestruct { names, value, .. } => {
                        let vt = self.infer_closure_expr(value, &visible);
                        for (i, n) in names.iter().enumerate() {
                            let evt = match vt {
                                VT::Tuple(ti) => self
                                    .coll
                                    .tuples
                                    .get(ti)
                                    .and_then(|t| t.get(i).copied())
                                    .unwrap_or(VT::I32),
                                _ => VT::I32,
                            };
                            visible.insert(n.name.clone(), evt);
                        }
                    }
                    Stmt::Let {
                        name, ann, value, ..
                    } => {
                        let vt = ann
                            .as_ref()
                            .map(|ann| {
                                vt_of_ann_coll(
                                    ann,
                                    self.rec_index,
                                    self.coll,
                                    &mut Vec::new(),
                                    TypePos::Local,
                                )
                            })
                            .unwrap_or_else(|| self.infer_closure_expr(value, &visible));
                        visible.insert(name.name.clone(), vt);
                    }
                    Stmt::If { then, els, .. } => {
                        if let Some(else_block) = els {
                            blocks.push((else_block, visible.clone()));
                        }
                        blocks.push((then, visible.clone()));
                    }
                    Stmt::While { body, .. } | Stmt::Loop { body, .. } => {
                        blocks.push((body, visible.clone()));
                    }
                    // PLAN-966（T-02）：语句级 CT 在绑定期已折叠；若仍可见则按
                    // 源块继续推断（防御：不静默丢名字/类型）。
                    Stmt::HashIf { then, els, .. } => {
                        if let Some(e) = els {
                            blocks.push((e, visible.clone()));
                        }
                        blocks.push((then, visible.clone()));
                    }
                    Stmt::HashFor { body, .. } => blocks.push((body, visible.clone())),
                    Stmt::HashIs { arms, .. } => {
                        for a in arms {
                            blocks.push((&a.body, visible.clone()));
                        }
                    }
                    Stmt::ForIn {
                        var, iter, body, ..
                    } => {
                        let mut nested = visible.clone();
                        let VT::List(index) = self.infer_closure_expr(iter, &visible) else {
                            blocks.push((body, nested));
                            continue;
                        };
                        nested.insert(
                            var.name.clone(),
                            self.coll.lists.get(index).copied().unwrap_or(VT::I32),
                        );
                        blocks.push((body, nested));
                    }
                    Stmt::IsMatch { arms, .. } => {
                        for arm in arms.iter().rev() {
                            let mut nested = visible.clone();
                            if let Some(binding) = &arm.binding {
                                nested.insert(binding.name.clone(), VT::I32);
                            }
                            blocks.push((&arm.body, nested));
                        }
                    }
                    Stmt::ExprStmt { .. }
                    | Stmt::Assign { .. }
                    | Stmt::Break { .. }
                    | Stmt::Continue { .. }
                    | Stmt::Yield { .. } => {}
                    Stmt::AggIsMatch { arms, .. } => {
                        for arm in arms.iter().rev() {
                            if let ast::AggArmBody::Block(b) = &arm.body {
                                blocks.push((b, visible.clone()));
                            }
                        }
                    }
                    Stmt::Try {
                        body,
                        catch,
                        finally,
                        ..
                    } => {
                        if let Some(f) = finally {
                            blocks.push((f, visible.clone()));
                        }
                        if let Some(c) = catch {
                            let mut nested = visible.clone();
                            if let Some(b) = &c.bind {
                                nested.insert(b.name.clone(), VT::StrOwned);
                            }
                            blocks.push((&c.body, nested));
                        }
                        blocks.push((body, visible.clone()));
                    }
                    // PLAN-1094: `with <expr> [as <name>] { body }` — the
                    // bind is a block-scoped name.
                    Stmt::With {
                        bind,
                        body,
                        ..
                    } => {
                        let mut nested = visible.clone();
                        if let Some(b) = bind {
                            nested.insert(b.name.clone(), VT::I32);
                        }
                        blocks.push((body, nested));
                    }
                }
            }
        }
        VT::Unit
    }

    fn resolve_closure(
        &mut self,
        move_capture: bool,
        params_ast: &[ast::Param],
        result_ast: Option<&ast::TypeAnn>,
        body_ast: &ast::ClosureBody,
        span: Span,
    ) -> ResolvedExpr {
        if self.profile != SourceProfile::Closures
            && self.profile != SourceProfile::Auto
            && !self.asm_lane
        {
            self.bind_err(
                "source.unsupported",
                "lambda expressions are available only in the closures and auto source profiles",
                span,
            );
            return ResolvedExpr {
                span,
                vt: VT::I32,
                kind: RExprKind::IntLit(0),
            };
        }

        // PLAN-912 T-03: retained-lambda REUSE. When an owner re-resolves
        // with a reuse oracle, a lambda whose expression TEXT is byte-identical
        // to the retained one and whose capture sources still derive the same
        // declaration (same mode/type/writability/cell from the CURRENT
        // creation-point locals) is pushed from the retained entry — its AST
        // body and its nested lambdas are never walked. Any difference falls
        // through to the real walk below (a true reader re-resolves).
        if let Some(reused) = self.try_reuse_lambda(move_capture, span) {
            return reused;
        }

        let outer_names: BTreeSet<String> = self
            .scopes
            .iter()
            .flat_map(|scope| scope.names.keys().cloned())
            .collect();
        let captures =
            super::closure_capture::analyze_capture_uses(params_ast, body_ast, &outer_names);
        // PLAN-912 §5.2: the raw pre-mode capture fact the site derives from.
        let capture_uses: Vec<(String, bool)> = captures
            .iter()
            .map(|c| (c.name.clone(), c.written))
            .collect();
        let mut capture_bindings = Vec::new();
        let mut capture_decls = Vec::new();
        for capture in captures {
            let Some(source_local) = self.lookup(&capture.name) else {
                continue;
            };
            let source_vt = self.locals[source_local.0].vt;
            let source_mutable = self.locals[source_local.0].mutable;
            let source_cell = self.locals[source_local.0].cell;
            let owned = self.vt_is_coll_owned(source_vt);
            let (mode, body_vt, moved, writable, cell) = match decide_capture(
                source_vt,
                source_mutable,
                source_cell,
                owned,
                move_capture,
                capture.written,
            ) {
                CaptureDecision::Decl {
                    mode,
                    body_vt,
                    moved,
                    writable,
                    cell,
                } => (mode, body_vt, moved, writable, cell),
                CaptureDecision::SharedMutableRejected => {
                    self.bind_err(
                        "source.closure-shared-mutable",
                        format!(
                            "closure writes captured local `{}` of type `{}`; shared mutable captures support scalar `int`/`bool`/`char` vars only",
                            capture.name,
                            source_vt.as_str()
                        ),
                        capture.span,
                    );
                    continue;
                }
                CaptureDecision::Unsupported => {
                    self.bind_err("source.closure-capture-unsupported", format!("capture `{}` of type `{}` requires a supported move or synchronous Str borrow", capture.name, source_vt.as_str()), capture.span);
                    continue;
                }
            };
            if moved {
                self.moved_locals.insert(source_local);
            }
            capture_bindings.push((source_local, mode));
            capture_decls.push((capture.name, body_vt, mode, writable, cell));
        }

        // Body walk FIRST, ordinal AFTER (post-order): nested lambdas mint
        // their ordinals during this walk and therefore sit BEFORE their
        // parent in `closure_defs` — the PLAN-909 identity order every
        // existing `__closure_<owner>_<k>` wire name depends on. PLAN-912
        // keeps that order byte-for-byte and only records the site facts
        // alongside.
        let count_before = self.closure_count;
        let lb = self.lambda_body(params_ast, result_ast, body_ast, &capture_decls, span);
        let nested = self.closure_count - count_before;
        let type_index = self.coll.intern_function(ClosureFnType {
            params: lb.param_types.clone(),
            result: lb.result,
        });
        // PLAN-909 §5.5: the owner-stable identity. The previous
        // module-wide ordinal made every later lambda rename when an
        // unrelated function or earlier lambda was inserted or reordered.
        let ordinal = self.closure_count;
        let name = format!("__closure_{}_{}", self.owner_name, ordinal);
        self.closure_count += 1;
        // PLAN-912 §5.4: a lambda body that was REALLY walked is a resolve
        // attempt of its own unit (a reused one records nothing) — the
        // PLAN-909 sibling rewalk becomes visible per unit.
        crate::body_work::record_resolve(&name);
        self.lambda_sites.push(LambdaSiteFacts {
            owner: self.owner_name.clone(),
            ordinal,
            name: name.clone(),
            expr_span: span,
            move_capture,
            params: lb
                .param_names
                .iter()
                .cloned()
                .zip(lb.param_types.iter().copied())
                .collect(),
            capture_uses,
            capture_decls: capture_decls.clone(),
            capture_sources: capture_bindings.iter().map(|(l, _)| *l).collect(),
            outer_names,
            result: lb.result,
            type_index,
            nested,
        });
        self.closure_defs.push(ResolvedFn {
            name: name.clone(),
            name_span: span,
            item_span: span,
            param_names: lb.param_names,
            params: lb.param_types,
            param_modes: vec![false; params_ast.len()],
            result: lb.result,
            locals: lb.locals,
            body: lb.body,
            loop_count: 0,
            used_intrinsics: lb.used_intrinsics,
            receiver: ast::Receiver::Free,
            owner_rec: None,
            owner_var: None,
        });
        ResolvedExpr {
            span,
            vt: VT::Function(type_index),
            kind: RExprKind::CreateClosure {
                callee: name.clone(),
                captures: capture_bindings,
            },
        }
    }

    /// PLAN-912: resolve ONE lambda body under an already-decided capture
    /// declaration. Shared by the owner walk ([`FnResolver::resolve_closure`])
    /// and the isolated re-resolve ([`resolve_lambda_in_context`]); the
    /// resolver's owner-level state is saved around the walk exactly as the
    /// inline path always did (the lambda sees only its params and captures).
    fn lambda_body(
        &mut self,
        params_ast: &[ast::Param],
        result_ast: Option<&ast::TypeAnn>,
        body_ast: &ast::ClosureBody,
        capture_decls: &[(String, VT, crate::hir::CaptureMode, bool, bool)],
        span: Span,
    ) -> LambdaBodyOut {
        let old_locals = std::mem::replace(&mut self.locals, Vec::new());
        let old_scopes = std::mem::replace(
            &mut self.scopes,
            vec![Scope {
                names: BTreeMap::new(),
                views: BTreeMap::new(),
                text_mut_aliases: BTreeMap::new(),
            }],
        );
        let old_declared = std::mem::replace(&mut self.declared_names, BTreeSet::new());
        let old_result = self.fn_result;
        let old_loops = std::mem::replace(&mut self.loop_stack, Vec::new());
        let old_loop_count = std::mem::replace(&mut self.loop_count, 0);
        let old_intrinsics = std::mem::replace(&mut self.used_intrinsics, BTreeSet::new());
        let old_moved = std::mem::replace(&mut self.moved_locals, BTreeSet::new());
        let old_borrowed = std::mem::replace(&mut self.borrowed_closure_locals, BTreeSet::new());
        let old_iterate = std::mem::replace(&mut self.iterate_targets, Vec::new());
        let old_ctrl_frames = std::mem::replace(&mut self.ctrl_frames, Vec::new());
        let old_ctrl_state = self.ctrl_state.take();
        let old_catch_errs = std::mem::replace(&mut self.catch_err_locals, Vec::new());
        let old_is_gen = std::mem::replace(&mut self.is_generator, false);
        let old_is_script = std::mem::replace(&mut self.is_script, false);
        let old_diverged = std::mem::replace(&mut self.expr_diverged, false);

        let mut param_names = Vec::new();
        let mut param_types = Vec::new();
        let mut visible = BTreeMap::new();
        for param in params_ast {
            let vt = vt_of_ann_coll(
                &param.ty,
                self.rec_index,
                self.coll,
                self.diags,
                TypePos::Param,
            );
            if vt == VT::Unit {
                self.bind_err(
                    "source.unit-placement",
                    "closure parameter cannot have type unit",
                    param.span,
                );
            }
            if self.scopes[0].names.contains_key(&param.name.name) {
                self.bind_err(
                    "source.duplicate",
                    format!("duplicate closure parameter `{}`", param.name.name),
                    param.name.span,
                );
            }
            let local = LocalRef(self.locals.len());
            self.locals.push(LocalDecl {
                name: param.name.name.clone(),
                vt,
                mutable: false,
                is_param: true,
                param_index: Some(param_types.len() as u32),
                capture_index: None,
                cell: false,
                is_capture: false,
                mode: ast::ParamMode::Default,
                span: param.span,
            });
            self.scopes[0].names.insert(param.name.name.clone(), local);
            self.declared_names.insert(param.name.name.clone());
            visible.insert(param.name.name.clone(), vt);
            param_names.push(param.name.name.clone());
            param_types.push(vt);
        }
        for (index, (name, vt, _mode, writable, capture_cell)) in capture_decls.iter().enumerate() {
            let local = LocalRef(self.locals.len());
            self.locals.push(LocalDecl {
                name: name.clone(),
                vt: *vt,
                mutable: *writable,
                is_param: false,
                param_index: None,
                capture_index: Some(index as u32),
                cell: *capture_cell,
                is_capture: true,
                mode: ast::ParamMode::Default,
                span,
            });
            self.scopes[0].names.insert(name.clone(), local);
            self.declared_names.insert(name.clone());
            visible.insert(name.clone(), *vt);
        }
        let inferred_result = if let Some(result) = result_ast {
            vt_of_ann_coll(result, self.rec_index, self.coll, self.diags, TypePos::Result)
        } else {
            match body_ast {
                ast::ClosureBody::Expr(expr) => self.infer_closure_expr(expr, &visible),
                ast::ClosureBody::Block(block) => self.infer_closure_block_result(block, &visible),
            }
        };
        self.fn_result = inferred_result;
        let (resolved_body, result) = match body_ast {
            ast::ClosureBody::Expr(expr) => {
                self.push_sink();
                let value = self.expr_iterative(expr);
                let result = value.vt;
                let mut stmts = self.pop_sink();
                stmts.push(RStmt::Return {
                    value,
                    span: expr.span,
                });
                let mut b = RBlock { stmts, span };
                self.prepend_ctrl_inits(&mut b, span);
                (b, result)
            }
            ast::ClosureBody::Block(block) => {
                let mut resolved = self.resolve_body(block, true);
                if inferred_result == VT::Unit {
                    resolved.stmts.push(RStmt::Return {
                        value: ResolvedExpr {
                            kind: RExprKind::Unit,
                            span,
                            vt: VT::Unit,
                        },
                        span,
                    });
                }
                self.prepend_ctrl_inits(&mut resolved, span);
                (resolved, inferred_result)
            }
        };
        let synthetic_locals = std::mem::replace(&mut self.locals, old_locals);
        self.scopes = old_scopes;
        self.declared_names = old_declared;
        let used_intrinsics = std::mem::replace(&mut self.used_intrinsics, old_intrinsics);
        self.fn_result = old_result;
        self.loop_stack = old_loops;
        self.loop_count = old_loop_count;
        self.moved_locals = old_moved;
        self.borrowed_closure_locals = old_borrowed;
        self.iterate_targets = old_iterate;
        self.ctrl_frames = old_ctrl_frames;
        self.ctrl_state = old_ctrl_state;
        self.catch_err_locals = old_catch_errs;
        self.is_generator = old_is_gen;
        self.is_script = old_is_script;
        self.expr_diverged = old_diverged;
        LambdaBodyOut {
            param_names,
            param_types,
            result,
            locals: synthetic_locals,
            body: resolved_body,
            used_intrinsics,
        }
    }

    /// PLAN-864: the synthetic unreachable-path return filler of an
    /// exhaustive `is` chain — typed from the function result (int 0 /
    /// unit / empty Str). Exotic result types reject by name: a filler
    /// value would need a payload we refuse to fabricate.
    fn default_return_filler(&mut self, span: Span) -> ResolvedExpr {
        match self.fn_result {
            VT::I32 => ResolvedExpr {
                span,
                vt: VT::I32,
                kind: RExprKind::IntLit(0),
            },
            VT::Unit => ResolvedExpr {
                span,
                vt: VT::Unit,
                kind: RExprKind::Unit,
            },
            VT::Bool => ResolvedExpr {
                span,
                vt: VT::Bool,
                kind: RExprKind::BoolLit(false),
            },
            VT::StrOwned => ResolvedExpr {
                span,
                vt: VT::StrOwned,
                kind: RExprKind::StrLit(String::new()),
            },
            VT::List(li) => ResolvedExpr {
                span,
                vt: VT::List(li),
                kind: RExprKind::ListNew { list: li },
            },
            VT::Variant(vi)
                if self
                    .coll
                    .variants
                    .get(vi)
                    .is_some_and(|v| v.arms.iter().any(|a| a.1.is_none())) =>
            {
                let disc = self.coll.variants[vi]
                    .arms
                    .iter()
                    .position(|a| a.1.is_none())
                    .unwrap() as u32;
                ResolvedExpr {
                    span,
                    vt: VT::Variant(vi),
                    kind: RExprKind::VariantCtor {
                        variant: vi,
                        disc,
                        payload: None,
                    },
                }
            }
            other => {
                self.bind_err(
                    "source.is-tail-return",
                    format!(
                        "an exhaustive `is` whose arms all return needs a fabricable filler for result type `{}` (supported: int/unit/Str)",
                        other.as_str()
                    ),
                    span,
                );
                ResolvedExpr {
                    span,
                    vt: VT::I32,
                    kind: RExprKind::IntLit(0),
                }
            }
        }
    }

    // =========================================================================
    // PLAN-965 (M2-A): Aggregates control-flow, error propagation & generators
    // =========================================================================

    fn push_sink(&mut self) {
        self.stmt_sinks.push(Vec::new());
    }

    fn pop_sink(&mut self) -> Vec<RStmt> {
        self.stmt_sinks.pop().unwrap_or_default()
    }

    fn emit_stmt(&mut self, stmt: RStmt) {
        if let Some(sink) = self.stmt_sinks.last_mut() {
            sink.push(stmt);
        }
    }

    fn drain_sink(&mut self) -> Vec<RStmt> {
        self.stmt_sinks
            .last_mut()
            .map(std::mem::take)
            .unwrap_or_default()
    }

    fn agg_temp(&mut self, prefix: &str, vt: VT, mutable: bool, span: Span) -> LocalRef {
        let id = self.agg_tmp_counter;
        self.agg_tmp_counter += 1;
        let local = LocalRef(self.locals.len());
        self.locals.push(LocalDecl {
            name: format!("__agg_{prefix}_{id}"),
            vt,
            mutable,
            is_param: false,
            param_index: None,
            capture_index: None,
            cell: false,
            is_capture: false,
            mode: ast::ParamMode::Default,
            span,
        });
        local
    }

    fn dummy_of(&mut self, vt: VT, span: Span) -> ResolvedExpr {
        agg_lower::dummy_expr_for_vt(vt, self.records, self.coll, span, 0)
    }

    fn ensure_ctrl_state(&mut self, span: Span) -> AggCtrlLocals {
        if let Some(cs) = self.ctrl_state {
            return cs;
        }
        let kind = self.agg_temp("ck", VT::I32, true, span);
        let target_loop = self.agg_temp("ctl", VT::I32, true, span);
        let err = self.agg_temp("ce", VT::StrOwned, true, span);
        let (ret_val, ret_opt_vi) = if self.fn_result == VT::Unit || self.is_generator {
            (self.agg_temp("crv", VT::I32, true, span), None)
        } else {
            let opt_vi = self.coll.intern_variant(
                "Option",
                span,
                vec![
                    ("Some".to_string(), Some(self.fn_result)),
                    ("None".to_string(), None),
                ],
            );
            (
                self.agg_temp("crv", VT::Variant(opt_vi), true, span),
                Some(opt_vi),
            )
        };
        let cs = AggCtrlLocals {
            kind,
            ret_val,
            ret_opt_vi,
            target_loop,
            err,
        };
        self.ctrl_state = Some(cs);
        cs
    }

    fn prepend_ctrl_inits(&mut self, block: &mut RBlock, span: Span) {
        let Some(cs) = self.ctrl_state else {
            return;
        };
        let ret_init = match cs.ret_opt_vi {
            Some(vi) => ResolvedExpr {
                span,
                vt: VT::Variant(vi),
                kind: RExprKind::VariantCtor {
                    variant: vi,
                    disc: 1,
                    payload: None,
                },
            },
            None => ResolvedExpr {
                span,
                vt: VT::I32,
                kind: RExprKind::IntLit(0),
            },
        };
        let mut init = vec![
            RStmt::Let {
                local: cs.kind,
                value: ResolvedExpr {
                    span,
                    vt: VT::I32,
                    kind: RExprKind::IntLit(0),
                },
                span,
            },
            RStmt::Let {
                local: cs.target_loop,
                value: ResolvedExpr {
                    span,
                    vt: VT::I32,
                    kind: RExprKind::IntLit(-1),
                },
                span,
            },
            RStmt::Let {
                local: cs.err,
                value: ResolvedExpr {
                    span,
                    vt: VT::StrOwned,
                    kind: RExprKind::StrLit(String::new()),
                },
                span,
            },
            RStmt::Let {
                local: cs.ret_val,
                value: ret_init,
                span,
            },
        ];
        init.append(&mut block.stmts);
        block.stmts = init;
    }

    fn nearest_enclosing_loop(&self) -> Option<LoopRef> {
        self.ctrl_frames.iter().rev().find_map(|f| match f {
            CtrlFrame::Loop { target }
            | CtrlFrame::ForIter { target, .. }
            | CtrlFrame::Try {
                try_loop: target, ..
            } => Some(*target),
        })
    }

    /// PLAN-1036 (F28/F29): the latch of the counted for-in loop `target`,
    /// if any. Iterating `for` loops (`ForIter`) re-enter through `next()`
    /// at the header and carry no latch; `while`/`loop` re-test their own
    /// condition, so only the counted range/List desugar registers one.
    fn for_latch_of(&self, target: LoopRef) -> Option<Vec<RStmt>> {
        self.for_latches
            .iter()
            .rev()
            .find(|(t, _)| *t == target)
            .map(|(_, latch)| latch.clone())
    }

    /// Emit the latch sequence a `continue` into `target` must run before
    /// the loop header. A no-op for loops without one.
    fn emit_for_latch(&self, target: Option<LoopRef>, out: &mut Vec<RStmt>) {
        if let Some(target) = target {
            if let Some(latch) = self.for_latch_of(target) {
                out.extend(latch);
            }
        }
    }

    fn has_enclosing_try(&self) -> bool {
        self.ctrl_frames
            .iter()
            .any(|f| matches!(f, CtrlFrame::Try { .. }))
    }

    fn has_enclosing_catch(&self) -> bool {
        self.ctrl_frames.iter().any(|f| {
            matches!(
                f,
                CtrlFrame::Try {
                    has_catch: true,
                    ..
                }
            )
        })
    }

    fn has_outer_ctrl_barrier(&self) -> bool {
        self.ctrl_state.is_some() && !self.ctrl_frames.is_empty()
    }

    /// PLAN-965 (V-DETAIL-02 / F-965-R1-07): a `break`/`continue` needs a real
    /// enclosing user loop. A synthetic try/catch/finally or for-in frame does
    /// NOT count — without this check a stray jump compiled and silently
    /// continued (the adapter registered a phantom loop, defeating the
    /// verifier's `verify.loop-target-scope`).
    fn has_enclosing_user_loop(&self) -> bool {
        self.ctrl_frames
            .iter()
            .any(|f| matches!(f, CtrlFrame::Loop { .. } | CtrlFrame::ForIter { .. }))
            || !self.loop_stack.is_empty()
    }

    fn close_for_iter_stmts(
        &mut self,
        close_fn_index: usize,
        iter_local: LocalRef,
        span: Span,
    ) -> Vec<RStmt> {
        let mut stmts = Vec::new();
        let iter_vt = self.locals[iter_local.0].vt;
        let iter_local = self.prepare_iterator_call(iter_local, span, &mut stmts);
        let local = self.agg_temp("it_close", VT::I32, false, span);
        stmts.push(RStmt::Let {
            local,
            value: ResolvedExpr {
                span,
                vt: VT::I32,
                kind: RExprKind::CallFn {
                    fn_index: close_fn_index,
                    args: vec![ResolvedArg {
                        span,
                        name: None,
                        moved: false,
                        value: ResolvedExpr {
                            span,
                            vt: iter_vt,
                            kind: RExprKind::BorrowMut { local: iter_local },
                        },
                    }],
                },
            },
            span,
        });
        stmts
    }

    /// An owned iterator passed to a mut-record call returns through the
    /// successful write-back edge. Preserve a terminal iterator in the
    /// source slot when a language error can be caught by this frame.
    fn prepare_iterator_call(
        &mut self,
        source: LocalRef,
        span: Span,
        stmts: &mut Vec<RStmt>,
    ) -> LocalRef {
        let vt = self.locals[source.0].vt;
        let VT::Rec(record) = vt else {
            return source;
        };
        let Some(retired) = self.iter_info[&record]
            .4
            .filter(|_| self.has_enclosing_try())
        else {
            return source;
        };
        let terminal = self.agg_temp("it_terminal", vt, false, span);
        let displaced = self.agg_temp("it_active", vt, true, span);
        stmts.push(RStmt::Let {
            local: terminal,
            value: ResolvedExpr {
                span,
                vt,
                kind: RExprKind::CallFn {
                    fn_index: retired,
                    args: Vec::new(),
                },
            },
            span,
        });
        stmts.push(RStmt::Let {
            local: displaced,
            value: ResolvedExpr {
                span,
                vt,
                kind: RExprKind::Move {
                    inner: Box::new(ResolvedExpr {
                        span,
                        vt,
                        kind: RExprKind::ReadLocal(source),
                    }),
                },
            },
            span,
        });
        stmts.push(RStmt::Assign {
            local: source,
            path: Vec::new(),
            value: ResolvedExpr {
                span,
                vt,
                kind: RExprKind::ReadLocal(terminal),
            },
            span,
        });
        displaced
    }

    fn emit_close_active_iters_to_out(&mut self, out: &mut Vec<RStmt>, span: Span) {
        let to_close: Vec<(usize, LocalRef)> = self
            .ctrl_frames
            .iter()
            .rev()
            .filter_map(|f| match f {
                CtrlFrame::ForIter {
                    iter_local,
                    close_fn_index,
                    ..
                } => Some((*close_fn_index, *iter_local)),
                _ => None,
            })
            .collect();
        for (close_fn_index, iter_local) in to_close {
            out.extend(self.close_for_iter_stmts(close_fn_index, iter_local, span));
        }
    }

    fn coerce_err_to_str_owned(&mut self, mut e: ResolvedExpr, span: Span) -> ResolvedExpr {
        if e.vt == VT::StrOwned {
            return e;
        }
        if e.vt == VT::Str && matches!(e.kind, RExprKind::StrLit(_)) {
            e.vt = VT::StrOwned;
            return e;
        }
        if matches!(
            e.vt,
            VT::I32 | VT::I8 | VT::U8 | VT::U32 | VT::I64 | VT::U64 | VT::Bool | VT::Char | VT::Str
        ) {
            return ResolvedExpr {
                span,
                vt: VT::StrOwned,
                kind: RExprKind::TextDisplay {
                    operand: Box::new(e),
                },
            };
        }
        ResolvedExpr {
            span,
            vt: VT::StrOwned,
            kind: RExprKind::StrLit("error".to_string()),
        }
    }

    fn emit_return_to_out(&mut self, val: Option<ResolvedExpr>, span: Span, out: &mut Vec<RStmt>) {
        if self.has_enclosing_try() {
            let cs = self.ensure_ctrl_state(span);
            out.push(RStmt::Assign {
                local: cs.kind,
                path: Vec::new(),
                value: ResolvedExpr {
                    span,
                    vt: VT::I32,
                    kind: RExprKind::IntLit(1),
                },
                span,
            });
            if let (Some(v), Some(opt_vi)) = (val, cs.ret_opt_vi) {
                out.push(RStmt::Assign {
                    local: cs.ret_val,
                    path: Vec::new(),
                    value: ResolvedExpr {
                        span,
                        vt: VT::Variant(opt_vi),
                        kind: RExprKind::VariantCtor {
                            variant: opt_vi,
                            disc: 0,
                            payload: Some(Box::new(v)),
                        },
                    },
                    span,
                });
            }
            out.push(RStmt::Break {
                target: self.nearest_enclosing_loop(),
                span,
            });
        } else {
            self.emit_close_active_iters_to_out(out, span);
            let value = val.unwrap_or(ResolvedExpr {
                span,
                vt: VT::Unit,
                kind: RExprKind::Unit,
            });
            out.push(RStmt::Return { value, span });
        }
    }

    fn emit_fail_to_out(&mut self, err: ResolvedExpr, span: Span, out: &mut Vec<RStmt>) {
        let err_owned = self.coerce_err_to_str_owned(err, span);
        let inside_try = self.has_enclosing_try();
        if inside_try && (!self.profile.is_auto_language() || self.is_generator) {
            let cs = self.ensure_ctrl_state(span);
            out.push(RStmt::Assign {
                local: cs.kind,
                path: Vec::new(),
                value: ResolvedExpr {
                    span,
                    vt: VT::I32,
                    kind: RExprKind::IntLit(4),
                },
                span,
            });
            out.push(RStmt::Assign {
                local: cs.err,
                path: Vec::new(),
                value: err_owned,
                span,
            });
            out.push(RStmt::Break {
                target: self.nearest_enclosing_loop(),
                span,
            });
        } else {
            if !inside_try {
                self.emit_close_active_iters_to_out(out, span);
            }
            out.push(RStmt::Fail {
                value: err_owned,
                span,
            });
        }
    }

    fn emit_break_or_continue_to_out(&mut self, is_break: bool, span: Span, out: &mut Vec<RStmt>) {
        let target = self.loop_stack.last().copied();
        let top_loop = match self.ctrl_frames.last() {
            Some(CtrlFrame::Loop { target }) | Some(CtrlFrame::ForIter { target, .. }) => {
                Some(*target)
            }
            _ => None,
        };
        if target.is_some() && target == top_loop {
            if is_break {
                out.push(RStmt::Break { target, span });
            } else {
                // PLAN-1036 (F28): `continue` into a counted for-in runs its
                // latch before the header re-tests the bounds.
                self.emit_for_latch(target, out);
                out.push(RStmt::Continue { target, span });
            }
            return;
        }
        if self.has_enclosing_try() || (self.ctrl_frames.len() > 1 && target.is_some()) {
            let cs = self.ensure_ctrl_state(span);
            let code = if is_break { 2 } else { 3 };
            let t_id = target.map(|t| t.0 as i32).unwrap_or(-1);
            out.push(RStmt::Assign {
                local: cs.kind,
                path: Vec::new(),
                value: ResolvedExpr {
                    span,
                    vt: VT::I32,
                    kind: RExprKind::IntLit(code),
                },
                span,
            });
            out.push(RStmt::Assign {
                local: cs.target_loop,
                path: Vec::new(),
                value: ResolvedExpr {
                    span,
                    vt: VT::I32,
                    kind: RExprKind::IntLit(t_id),
                },
                span,
            });
            out.push(RStmt::Break {
                target: self.nearest_enclosing_loop(),
                span,
            });
        } else if is_break {
            out.push(RStmt::Break { target, span });
        } else {
            self.emit_for_latch(target, out);
            out.push(RStmt::Continue { target, span });
        }
    }

    fn emit_post_frame_dispatch(&mut self, span: Span, out: &mut Vec<RStmt>) {
        let Some(cs) = self.ctrl_state else {
            return;
        };
        let mut handle_stmts = Vec::new();
        match self.ctrl_frames.last().cloned() {
            Some(CtrlFrame::Loop { target }) | Some(CtrlFrame::ForIter { target, .. }) => {
                let clear_kind = RStmt::Assign {
                    local: cs.kind,
                    path: Vec::new(),
                    value: ResolvedExpr {
                        span,
                        vt: VT::I32,
                        kind: RExprKind::IntLit(0),
                    },
                    span,
                };
                let target_eq = ResolvedExpr {
                    span,
                    vt: VT::Bool,
                    kind: RExprKind::Binary {
                        op: BinOp::Eq,
                        lhs: Box::new(ResolvedExpr {
                            span,
                            vt: VT::I32,
                            kind: RExprKind::ReadLocal(cs.target_loop),
                        }),
                        rhs: Box::new(ResolvedExpr {
                            span,
                            vt: VT::I32,
                            kind: RExprKind::IntLit(target.0 as i32),
                        }),
                    },
                };
                let is_brk_here = ResolvedExpr {
                    span,
                    vt: VT::Bool,
                    kind: RExprKind::Binary {
                        op: BinOp::And,
                        lhs: Box::new(ResolvedExpr {
                            span,
                            vt: VT::Bool,
                            kind: RExprKind::Binary {
                                op: BinOp::Eq,
                                lhs: Box::new(ResolvedExpr {
                                    span,
                                    vt: VT::I32,
                                    kind: RExprKind::ReadLocal(cs.kind),
                                }),
                                rhs: Box::new(ResolvedExpr {
                                    span,
                                    vt: VT::I32,
                                    kind: RExprKind::IntLit(2),
                                }),
                            },
                        }),
                        rhs: Box::new(target_eq.clone()),
                    },
                };
                let is_cont_here = ResolvedExpr {
                    span,
                    vt: VT::Bool,
                    kind: RExprKind::Binary {
                        op: BinOp::And,
                        lhs: Box::new(ResolvedExpr {
                            span,
                            vt: VT::Bool,
                            kind: RExprKind::Binary {
                                op: BinOp::Eq,
                                lhs: Box::new(ResolvedExpr {
                                    span,
                                    vt: VT::I32,
                                    kind: RExprKind::ReadLocal(cs.kind),
                                }),
                                rhs: Box::new(ResolvedExpr {
                                    span,
                                    vt: VT::I32,
                                    kind: RExprKind::IntLit(3),
                                }),
                            },
                        }),
                        rhs: Box::new(target_eq),
                    },
                };
                handle_stmts.push(RStmt::If {
                    cond: is_cont_here,
                    then: RBlock {
                        stmts: {
                            // PLAN-1036 (F28): a routed `continue` into a
                            // counted for-in runs the latch here, exactly as
                            // the direct jump does at its own site.
                            let mut cont_stmts = vec![clear_kind.clone()];
                            self.emit_for_latch(Some(target), &mut cont_stmts);
                            cont_stmts.push(RStmt::Continue {
                                target: Some(target),
                                span,
                            });
                            cont_stmts
                        },
                        span,
                    },
                    els: Some(RBlock {
                        stmts: vec![RStmt::If {
                            cond: is_brk_here,
                            then: RBlock {
                                stmts: vec![
                                    clear_kind,
                                    RStmt::Break {
                                        target: Some(target),
                                        span,
                                    },
                                ],
                                span,
                            },
                            els: Some(RBlock {
                                stmts: vec![RStmt::If {
                                    cond: ResolvedExpr {
                                        span,
                                        vt: VT::Bool,
                                        kind: RExprKind::Binary {
                                            op: BinOp::Ne,
                                            lhs: Box::new(ResolvedExpr {
                                                span,
                                                vt: VT::I32,
                                                kind: RExprKind::ReadLocal(cs.kind),
                                            }),
                                            rhs: Box::new(ResolvedExpr {
                                                span,
                                                vt: VT::I32,
                                                kind: RExprKind::IntLit(0),
                                            }),
                                        },
                                    },
                                    then: RBlock {
                                        stmts: vec![RStmt::Break {
                                            target: Some(target),
                                            span,
                                        }],
                                        span,
                                    },
                                    els: None,
                                    span,
                                }],
                                span,
                            }),
                            span,
                        }],
                        span,
                    }),
                    span,
                });
            }
            Some(CtrlFrame::Try { try_loop, .. }) => {
                handle_stmts.push(RStmt::Break {
                    target: Some(try_loop),
                    span,
                });
            }
            None => {
                // Reached function root after all try/loop frames unwound
                let ret_stmt = if let Some(opt_vi) = cs.ret_opt_vi {
                    let filler = self.dummy_of(self.fn_result, span);
                    RStmt::If {
                        cond: ResolvedExpr {
                            span,
                            vt: VT::Bool,
                            kind: RExprKind::VariantGuardTest {
                                local: cs.ret_val,
                                variant: opt_vi,
                                disc: 0,
                            },
                        },
                        then: RBlock {
                            stmts: vec![RStmt::Return {
                                value: ResolvedExpr {
                                    span,
                                    vt: self.fn_result,
                                    kind: RExprKind::VariantProject {
                                        local: cs.ret_val,
                                        variant: opt_vi,
                                        disc: 0,
                                    },
                                },
                                span,
                            }],
                            span,
                        },
                        els: Some(RBlock {
                            stmts: vec![RStmt::Return {
                                value: filler,
                                span,
                            }],
                            span,
                        }),
                        span,
                    }
                } else {
                    RStmt::Return {
                        value: ResolvedExpr {
                            span,
                            vt: VT::Unit,
                            kind: RExprKind::Unit,
                        },
                        span,
                    }
                };
                let is_ret = ResolvedExpr {
                    span,
                    vt: VT::Bool,
                    kind: RExprKind::Binary {
                        op: BinOp::Eq,
                        lhs: Box::new(ResolvedExpr {
                            span,
                            vt: VT::I32,
                            kind: RExprKind::ReadLocal(cs.kind),
                        }),
                        rhs: Box::new(ResolvedExpr {
                            span,
                            vt: VT::I32,
                            kind: RExprKind::IntLit(1),
                        }),
                    },
                };
                let is_fail = ResolvedExpr {
                    span,
                    vt: VT::Bool,
                    kind: RExprKind::Binary {
                        op: BinOp::Eq,
                        lhs: Box::new(ResolvedExpr {
                            span,
                            vt: VT::I32,
                            kind: RExprKind::ReadLocal(cs.kind),
                        }),
                        rhs: Box::new(ResolvedExpr {
                            span,
                            vt: VT::I32,
                            kind: RExprKind::IntLit(4),
                        }),
                    },
                };
                handle_stmts.push(RStmt::If {
                    cond: is_ret,
                    then: RBlock {
                        stmts: vec![ret_stmt],
                        span,
                    },
                    els: Some(RBlock {
                        stmts: vec![RStmt::If {
                            cond: is_fail,
                            then: RBlock {
                                stmts: vec![RStmt::Fail {
                                    value: ResolvedExpr {
                                        span,
                                        vt: VT::StrOwned,
                                        kind: RExprKind::ReadLocal(cs.err),
                                    },
                                    span,
                                }],
                                span,
                            },
                            els: None,
                            span,
                        }],
                        span,
                    }),
                    span,
                });
            }
        }
        let cond_nonzero = ResolvedExpr {
            span,
            vt: VT::Bool,
            kind: RExprKind::Binary {
                op: BinOp::Ne,
                lhs: Box::new(ResolvedExpr {
                    span,
                    vt: VT::I32,
                    kind: RExprKind::ReadLocal(cs.kind),
                }),
                rhs: Box::new(ResolvedExpr {
                    span,
                    vt: VT::I32,
                    kind: RExprKind::IntLit(0),
                }),
            },
        };
        out.push(RStmt::If {
            cond: cond_nonzero,
            then: RBlock {
                stmts: handle_stmts,
                span,
            },
            els: None,
            span,
        });
    }

    fn resolve_expr_in_branch(
        &mut self,
        e: &ast::Expr,
        expect: Option<VT>,
    ) -> (Vec<RStmt>, ResolvedExpr, bool) {
        self.resolve_in_branch(|resolver| resolver.expr_with_expect(e, expect))
    }

    fn resolve_in_branch(
        &mut self,
        resolve: impl FnOnce(&mut Self) -> ResolvedExpr,
    ) -> (Vec<RStmt>, ResolvedExpr, bool) {
        self.push_sink();
        self.expr_diverged = false;
        let val = resolve(self);
        let div = self.expr_diverged;
        self.expr_diverged = false;
        let stmts = self.pop_sink();
        (stmts, val, div)
    }

    /// Comparing a local Option with its empty constructor reads the tag
    /// without transferring or materializing its payload.
    fn compare_empty_option(
        &self,
        value: &ResolvedExpr,
        empty: &ResolvedExpr,
        op: BinOp,
        span: Span,
    ) -> Option<ResolvedExpr> {
        let RExprKind::ReadLocal(local) = value.kind else {
            return None;
        };
        let RExprKind::VariantCtor {
            variant: empty_variant,
            disc,
            payload: None,
        } = empty.kind
        else {
            return None;
        };
        let VT::Variant(variant) = value.vt else {
            return None;
        };
        let declaration = &self.coll.variants[empty_variant];
        if declaration.name != "Option"
            || self.coll.variants[variant].name != "Option"
            || declaration
                .arms
                .get(disc as usize)
                .map(|arm| arm.0.as_str())
                != Some("None")
        {
            return None;
        }
        let guard = ResolvedExpr {
            span,
            vt: VT::Bool,
            kind: RExprKind::VariantGuardTest {
                local,
                variant,
                disc,
            },
        };
        Some(if op == BinOp::Eq {
            guard
        } else {
            ResolvedExpr {
                span,
                vt: VT::Bool,
                kind: RExprKind::Unary {
                    op: UnaryOp::Not,
                    operand: Box::new(guard),
                },
            }
        })
    }

    fn resolve_condition(&mut self, expression: &ast::Expr) -> ResolvedExpr {
        let expect = (!self.profile.is_auto_language()).then_some(VT::Bool);
        let value = self.expr_with_expect(expression, expect);
        self.condition_value(value)
    }

    fn condition_value(&mut self, value: ResolvedExpr) -> ResolvedExpr {
        if !self.profile.is_auto_language() {
            return value;
        }
        let span = value.span;
        let false_value = ResolvedExpr {
            span,
            vt: VT::Bool,
            kind: RExprKind::BoolLit(false),
        };
        match value.vt {
            VT::Nil => {
                self.eval_for_effect(value);
                false_value
            }
            VT::Dyn => self.resolve_dyn_coalesce(value, Vec::new(), false_value, false, span),
            _ => value,
        }
    }

    fn eval_for_effect(&mut self, value: ResolvedExpr) {
        let span = value.span;
        if matches!(value.kind, RExprKind::NilLit | RExprKind::ReadLocal(_)) {
            return;
        }
        if value.vt == VT::Unit {
            self.emit_stmt(RStmt::ExprStmt { value, span });
        } else {
            let value = Self::own_dynamic_text(value);
            let local = self.agg_temp("effect", value.vt, false, span);
            self.emit_stmt(RStmt::Let { local, value, span });
        }
    }

    fn script_type_error(&mut self, message: String, vt: VT, span: Span) -> ResolvedExpr {
        self.script_error_value(
            ResolvedExpr {
                span,
                vt: VT::StrOwned,
                kind: RExprKind::StrLit(message),
            },
            vt,
            span,
        )
    }

    fn script_error_value(&mut self, message: ResolvedExpr, vt: VT, span: Span) -> ResolvedExpr {
        let mut stmts = Vec::new();
        self.emit_fail_to_out(message, span, &mut stmts);
        for stmt in stmts {
            self.emit_stmt(stmt);
        }
        self.expr_diverged = true;
        self.dummy_of(vt, span)
    }

    fn script_type_name(vt: VT) -> &'static str {
        match vt {
            VT::Nil => "NoneType",
            VT::Bool => "bool",
            VT::F32 | VT::F64 => "float",
            vt if vt.is_text() => "str",
            vt if vt_is_numeric(vt) => "int",
            vt => vt.as_str(),
        }
    }

    fn eval_once(&mut self, value: ResolvedExpr) -> ResolvedExpr {
        if matches!(value.kind, RExprKind::ReadLocal(_)) {
            return value;
        }
        let value = Self::own_dynamic_text(value);
        let span = value.span;
        let vt = value.vt;
        let local = self.agg_temp("script_operand", vt, false, span);
        self.emit_stmt(RStmt::Let { local, value, span });
        ResolvedExpr {
            span,
            vt,
            kind: RExprKind::ReadLocal(local),
        }
    }

    fn guard_boxed_nil(&mut self, value: ResolvedExpr, message: String, span: Span) {
        debug_assert_eq!(value.vt, VT::Dyn);
        let (failure, _, _) =
            self.resolve_in_branch(|resolver| resolver.script_type_error(message, VT::Unit, span));
        self.emit_stmt(RStmt::If {
            cond: Self::dynamic_nil_test(value, span),
            then: RBlock {
                stmts: failure,
                span,
            },
            els: None,
            span,
        });
    }

    fn dynamic_nil_test(value: ResolvedExpr, span: Span) -> ResolvedExpr {
        ResolvedExpr {
            span,
            vt: VT::Bool,
            kind: RExprKind::DynCast {
                operand: Box::new(value),
                test_nil: true,
            },
        }
    }

    fn resolve_dyn_coalesce(
        &mut self,
        lhs: ResolvedExpr,
        mut fallback_stmts: Vec<RStmt>,
        fallback: ResolvedExpr,
        fallback_diverges: bool,
        span: Span,
    ) -> ResolvedExpr {
        let scrutinee = self.agg_temp("dyn_coalesce", VT::Dyn, false, span);
        self.emit_stmt(RStmt::Let {
            local: scrutinee,
            value: lhs,
            span,
        });
        let fallback = Self::own_dynamic_text(fallback);
        let vt = fallback.vt;
        let result = self.agg_temp("dyn_coalesce_result", vt, true, span);
        let initial = self.dummy_of(vt, span);
        self.emit_stmt(RStmt::Let {
            local: result,
            value: initial,
            span,
        });
        if !fallback_diverges {
            fallback_stmts.push(RStmt::Assign {
                local: result,
                path: Vec::new(),
                span: fallback.span,
                value: fallback,
            });
        }
        let read = || ResolvedExpr {
            span,
            vt: VT::Dyn,
            kind: RExprKind::ReadLocal(scrutinee),
        };
        self.emit_stmt(RStmt::If {
            cond: Self::dynamic_nil_test(read(), span),
            then: RBlock {
                stmts: fallback_stmts,
                span,
            },
            els: Some(RBlock {
                stmts: vec![RStmt::Assign {
                    local: result,
                    path: Vec::new(),
                    value: Self::coerce_dynamic(read(), vt),
                    span,
                }],
                span,
            }),
            span,
        });
        ResolvedExpr {
            span,
            vt,
            kind: RExprKind::ReadLocal(result),
        }
    }

    fn compare_null(
        &mut self,
        lhs: ResolvedExpr,
        rhs: ResolvedExpr,
        op: BinOp,
        span: Span,
    ) -> ResolvedExpr {
        let query = if lhs.vt == VT::Dyn {
            let tmp = self.agg_temp("null_cmp", VT::Dyn, false, lhs.span);
            self.emit_stmt(RStmt::Let {
                local: tmp,
                span: lhs.span,
                value: lhs,
            });
            self.eval_for_effect(rhs);
            Self::dynamic_nil_test(
                ResolvedExpr {
                    span,
                    vt: VT::Dyn,
                    kind: RExprKind::ReadLocal(tmp),
                },
                span,
            )
        } else if rhs.vt == VT::Dyn {
            self.eval_for_effect(lhs);
            Self::dynamic_nil_test(rhs, span)
        } else {
            let both_nil = lhs.vt == VT::Nil && rhs.vt == VT::Nil;
            self.eval_for_effect(lhs);
            self.eval_for_effect(rhs);
            return ResolvedExpr {
                span,
                vt: VT::Bool,
                kind: RExprKind::BoolLit(both_nil == (op == BinOp::Eq)),
            };
        };
        if op == BinOp::Eq {
            query
        } else {
            ResolvedExpr {
                span,
                vt: VT::Bool,
                kind: RExprKind::Unary {
                    op: UnaryOp::Not,
                    operand: Box::new(query),
                },
            }
        }
    }

    /// PLAN-1082 (T-03, D-1082-05): the legacy `List.new(e1, ..., en)`
    /// checked compat sugar on the ordinary (auto) face. Element VT comes
    /// from the first argument with the array-literal homogeneous rule and
    /// the same push admission; desugars to ListNew + push (hoisted via
    /// the statement sink). Zero args is a NAMED rejection (no element
    /// type is inferable; the explicit `List<T>.new()` carries the empty
    /// list). NOT a general varargs surface: only the bare `List` receiver
    /// with the `new` member; a shadowing local/record/variant wins.
    fn try_resolve_legacy_list_new(&mut self, e: &ast::Expr, _expect: Option<VT>) -> Option<ResolvedExpr> {
        if !self.profile.is_auto_language() {
            return None;
        }
        let ExprKind::MethodCall {
            recv,
            name,
            args,
            instance: None,
        } = &e.kind
        else {
            return None;
        };
        if name.name != "new" {
            return None;
        }
        let ExprKind::Var(id) = &recv.kind else {
            return None;
        };
        if id.name != "List" {
            return None;
        }
        if self.lookup("List").is_some()
            || self.rec_index.contains_key("List")
            || self.coll.variant_by_name("List").is_some()
        {
            return None;
        }
        if args.is_empty() {
            self.bind_err(
                "source.list-new-empty",
                "`List.new()` cannot infer an element type; write `List<T>.new()` for an empty list"
                    .to_string(),
                e.span,
            );
            return Some(ResolvedExpr {
                span: e.span,
                vt: VT::I32,
                kind: RExprKind::IntLit(0),
            });
        }
        let mut rs = Vec::with_capacity(args.len());
        for a in args {
            let ast::Arg::Pos(x) = a else {
                self.bind_err(
                    "source.unsupported",
                    "`List.new(...)` takes positional elements only".to_string(),
                    e.span,
                );
                return None;
            };
            rs.push(self.expr_iterative(x));
        }
        let elem_vt = rs[0].vt;
        for (r, a) in rs.iter().zip(args.iter()) {
            let ast::Arg::Pos(x) = a else {
                unreachable!("positional checked above");
            };
            if r.vt != elem_vt {
                self.bind_err(
                    "source.type-mismatch",
                    format!(
                        "List.new elements must share one type; first is `{}`, found `{}`",
                        elem_vt.as_str(),
                        r.vt.as_str()
                    ),
                    x.span,
                );
            }
            self.reject_elem_view_use(r, "the literal element");
        }
        let li = self.coll.intern_list(elem_vt);
        let temp = self.agg_temp("list_new", VT::List(li), false, e.span);
        self.emit_stmt(RStmt::Let {
            local: temp,
            value: ResolvedExpr {
                span: e.span,
                vt: VT::List(li),
                kind: RExprKind::ListNew { list: li },
            },
            span: e.span,
        });
        for (mut r, a) in rs.into_iter().zip(args.iter()) {
            let ast::Arg::Pos(x) = a else {
                unreachable!("positional checked above");
            };
            if matches!(x.kind, ExprKind::ArrayLit(_))
                && matches!(r.kind, RExprKind::ReadLocal(_))
                && self.vt_is_coll_owned(r.vt)
            {
                let inner_vt = r.vt;
                let span0 = r.span;
                let taken = std::mem::replace(&mut r.kind, RExprKind::IntLit(0));
                r = ResolvedExpr {
                    span: span0,
                    vt: inner_vt,
                    kind: RExprKind::Move {
                        inner: Box::new(ResolvedExpr {
                            span: span0,
                            vt: inner_vt,
                            kind: taken,
                        }),
                    },
                };
            }
            self.check_push_value(&elem_vt, &mut r, x.span);
            self.emit_stmt(RStmt::ExprStmt {
                value: ResolvedExpr {
                    span: x.span,
                    vt: VT::Unit,
                    kind: RExprKind::ListPush {
                        list: Box::new(ResolvedExpr {
                            span: x.span,
                            vt: VT::List(li),
                            kind: RExprKind::ReadLocal(temp),
                        }),
                        value: Box::new(r),
                    },
                },
                span: x.span,
            });
        }
        Some(ResolvedExpr {
            span: e.span,
            vt: VT::List(li),
            kind: RExprKind::ReadLocal(temp),
        })
    }

    /// PLAN-1083: `E.A` (no call) — a payload-less qualified variant arm
    /// constructs the variant value directly; a payload arm is a named
    /// argument-count error (never silently default).
    fn contextual_enum(&self, template: &ast::Ident, explicit: &[ast::TypeAnn], expect: Option<VT>) -> Option<Vec<ast::TypeAnn>> {
        let Some(VT::Variant(vi)) = expect else { return None };
        let info = self.coll.enum_instances.get(&vi)?;
        if explicit.is_empty() && info.template == template.name {
            info.arguments.iter().cloned().collect()
        } else {
            None
        }
    }

    fn try_resolve_variant_arm(&mut self, e: &ast::Expr, expect: Option<VT>) -> Option<ResolvedExpr> {
        if !self.profile.is_collections() {
            return None;
        }
        let ExprKind::Field { base, name } = &e.kind else {
            return None;
        };
        if let Some((template, types)) = self.generic_enum_receiver(base) {
            let hint = self.contextual_enum(template, types, expect);
            return Some(self.probe_enum_constructor(
                template, hint.as_deref().unwrap_or(types), name, &[], e.span,
            ));
        }
        let ExprKind::Var(id) = &base.kind else { return None };
        if self.lookup(&id.name).is_some() {
            return None;
        }
        let vi = self.coll.variant_by_name(&id.name)?;
        let Some(disc) = self.coll.variants[vi]
            .arms
            .iter()
            .position(|(n, _)| *n == name.name)
        else {
            return None;
        };
        if self.reject_unbound_variant_arm(vi, disc as u32, name.span) {
            return Some(instances::pending_expression(e.span));
        }
        if self.coll.variants[vi].arms[disc].1.is_some() {
            self.bind_err(
                "source.argument",
                format!(
                    "arm `{}.{}` carries a payload; construct it with arguments",
                    id.name, name.name
                ),
                name.span,
            );
            return None;
        }
        Some(ResolvedExpr {
            span: e.span,
            vt: VT::Variant(vi),
            kind: RExprKind::VariantCtor {
                variant: vi,
                disc: disc as u32,
                payload: None,
            },
        })
    }

    /// PLAN-1083: `Color.Red` where `Color` is a scalar `= N` enum — the
    /// plain int constant (never a variant construction).
    fn try_resolve_scalar_enum(&mut self, e: &ast::Expr, expect: Option<VT>) -> Option<ResolvedExpr> {
        let ExprKind::Field { base, name } = &e.kind else {
            return None;
        };
        let ExprKind::Var(id) = &base.kind else {
            return None;
        };
        let Some(arms) = self.coll.scalar_enums.get(&id.name) else {
            return None;
        };
        let found = arms
            .iter()
            .find(|(n, _)| n == &name.name)
            .map(|(_, v)| *v);
        let v = match found {
            Some(v) => v,
            None => {
                self.bind_err(
                    "source.pattern-unknown-arm",
                    format!("scalar enum `{}` has no arm named `{}`", id.name, name.name),
                    name.span,
                );
                return None;
            }
        };
        let _ = expect;
        Some(ResolvedExpr {
            span: e.span,
            vt: VT::I32,
            kind: RExprKind::IntLit(v),
        })
    }

    fn try_resolve_may_ctor(&mut self, e: &ast::Expr, expect: Option<VT>) -> Option<ResolvedExpr> {
        if !self.profile.is_aggregates()
            || self.lookup("May").is_some()
            || self.rec_index.contains_key("May")
            || self.coll.variant_by_name("May").is_some()
        {
            return None;
        }
        match &e.kind {
            ExprKind::Field { base, name }
                if matches!(&base.kind, ExprKind::Var(id) if id.name == "May")
                    && name.name == "nil" =>
            {
                Some(self.resolve_bare_ctor("None", None, expect, e.span))
            }
            ExprKind::MethodCall {
                recv,
                name,
                args,
                instance: None,
            } if matches!(&recv.kind, ExprKind::Var(id) if id.name == "May") => {
                match (name.name.as_str(), args.len()) {
                    ("nil", 0) => Some(self.resolve_bare_ctor("None", None, expect, e.span)),
                    ("val", 1) => {
                        let arg_e = match &args[0] {
                            ast::Arg::Pos(x) | ast::Arg::MovePos(x) => x,
                            _ => return None,
                        };
                        let pl = self.expr_iterative(arg_e);
                        let is_result_expect = matches!(
                            expect,
                            Some(VT::Variant(vi)) if self.coll.variants[vi].name == "Result"
                        );
                        let cname = if is_result_expect { "Ok" } else { "Some" };
                        Some(self.resolve_bare_ctor(cname, Some(pl), expect, e.span))
                    }
                    ("err", 1) => {
                        let arg_e = match &args[0] {
                            ast::Arg::Pos(x) | ast::Arg::MovePos(x) => x,
                            _ => return None,
                        };
                        let pl = self.expr_iterative(arg_e);
                        Some(self.resolve_bare_ctor("Err", Some(pl), expect, e.span))
                    }
                    _ => None,
                }
            }
            _ => None,
        }
    }

    fn own_dynamic_text(mut value: ResolvedExpr) -> ResolvedExpr {
        if value.vt != VT::Str {
            return value;
        }
        if matches!(value.kind, RExprKind::StrLit(_)) {
            value.vt = VT::StrOwned;
            return value;
        }
        ResolvedExpr {
            span: value.span,
            vt: VT::StrOwned,
            kind: RExprKind::Method {
                recv: Box::new(value),
                method: BuiltinMethod::To,
                to: Some(VT::StrOwned),
                args: Vec::new(),
            },
        }
    }

    fn coerce_dynamic(value: ResolvedExpr, expected: VT) -> ResolvedExpr {
        if value.vt == expected || (value.vt != VT::Dyn && expected != VT::Dyn) {
            return value;
        }
        let value = Self::own_dynamic_text(value);
        ResolvedExpr {
            span: value.span,
            vt: if expected == VT::Str {
                VT::StrOwned
            } else {
                expected
            },
            kind: RExprKind::DynCast {
                operand: Box::new(value),
                test_nil: false,
            },
        }
    }

    fn check_object_key(&mut self, key: &ResolvedExpr) {
        let ordinary = self.profile == SourceProfile::Auto;
        if !key.vt.is_text() && !(ordinary && matches!(key.vt, VT::I32 | VT::Bool)) {
            let domain = if ordinary {
                "`Str`, `int` or `bool`"
            } else {
                "`Str` in this profile"
            };
            self.bind_err(
                "source.object-key",
                format!("an object key must be {domain}"),
                key.span,
            );
        }
    }

    fn object_field_key(field: &ast::Ident) -> ResolvedExpr {
        let (vt, kind) = match field.name.as_str() {
            "true" => (VT::Bool, RExprKind::BoolLit(true)),
            "false" => (VT::Bool, RExprKind::BoolLit(false)),
            name => match name.parse::<i32>() {
                Ok(value) => (VT::I32, RExprKind::IntLit(value)),
                Err(_) => (VT::Str, RExprKind::StrLit(name.to_owned())),
            },
        };
        ResolvedExpr {
            span: field.span,
            vt,
            kind,
        }
    }

    fn object_get(base: ResolvedExpr, key: ResolvedExpr, elem: VT, span: Span) -> ResolvedExpr {
        ResolvedExpr {
            span,
            vt: elem,
            kind: RExprKind::ObjectGet {
                base: Box::new(base),
                key: Box::new(key),
            },
        }
    }

    fn resolve_object_field_assign(
        &mut self,
        target: &ast::AssignTarget,
        value: &ast::Expr,
        span: Span,
    ) -> Option<RStmt> {
        if self.profile != SourceProfile::Auto {
            return None;
        }
        let (last, parents) = target.path.split_last()?;
        let local = self.lookup(&target.base.name)?;
        let vt = self.locals[local.0].vt;
        if !matches!(vt, VT::Object(_) | VT::Dyn) {
            return None;
        }
        let mut base = ResolvedExpr {
            span: target.base.span,
            vt,
            kind: RExprKind::ReadLocal(local),
        };
        for field in parents {
            if base.vt == VT::Dyn {
                let object = self.coll.intern_object(VT::Dyn);
                base = Self::coerce_dynamic(base, VT::Object(object));
            }
            let VT::Object(object) = base.vt else {
                return None;
            };
            let field_span = base.span.join(field.span);
            base = Self::object_get(
                base,
                Self::object_field_key(field),
                self.coll.objects[object],
                field_span,
            );
        }
        if base.vt == VT::Dyn {
            let object = self.coll.intern_object(VT::Dyn);
            base = Self::coerce_dynamic(base, VT::Object(object));
        }
        let VT::Object(object) = base.vt else {
            return None;
        };
        let elem = self.coll.objects[object];
        let mut value = self.expr_with_expect(value, Some(elem));
        if value.vt != elem {
            self.bind_err(
                "source.type-mismatch",
                format!(
                    "object element type `{}` expected, found `{}`",
                    elem.as_str(),
                    value.vt.as_str()
                ),
                value.span,
            );
        }
        self.check_push_value(&elem, &mut value, last.span);
        let value = ResolvedExpr {
            span,
            vt: VT::Unit,
            kind: RExprKind::ObjectSet {
                base: Box::new(base),
                key: Box::new(Self::object_field_key(last)),
                value: Box::new(value),
            },
        };
        Some(RStmt::ExprStmt { value, span })
    }

    fn dynamic_place(&self, mut ty: VT, steps: &[ast::PlaceStep]) -> bool {
        for step in steps {
            ty = match (ty, step) {
                (VT::Dyn | VT::Object(_), _) => return true,
                (VT::List(list), ast::PlaceStep::Index(_)) => self.coll.lists[list],
                (VT::Rec(record), ast::PlaceStep::Field(field)) => {
                    let Some(field) = self.records[record]
                        .fields
                        .iter()
                        .find(|f| f.name == field.name)
                    else {
                        return false;
                    };
                    field.vt
                }
                _ => return false,
            };
        }
        false
    }

    fn materialize_place_operand(&mut self, value: ResolvedExpr) -> ResolvedExpr {
        let span = value.span;
        let vt = value.vt;
        let local = self.agg_temp("object_operand", vt, false, span);
        self.emit_stmt(RStmt::Let { local, value, span });
        ResolvedExpr {
            span,
            vt,
            kind: RExprKind::ReadLocal(local),
        }
    }

    fn resolve_dynamic_place_assign(&mut self, place: &ast::PlaceAssign) {
        let Some((last, prefix)) = place.steps.split_last() else {
            return;
        };
        let receiver = place_read_ast(&place.base, prefix);
        let mut base = self.expr_iterative(&receiver);
        let key = match last {
            ast::PlaceStep::Field(field) => {
                if base.vt == VT::Dyn {
                    let object = self.coll.intern_object(VT::Dyn);
                    base = Self::coerce_dynamic(base, VT::Object(object));
                }
                Self::object_field_key(field)
            }
            ast::PlaceStep::Index(index) => {
                base = self.materialize_place_operand(base);
                self.expr_iterative(index)
            }
        };
        self.check_object_key(&key);
        let base = if matches!(last, ast::PlaceStep::Field(_)) {
            self.materialize_place_operand(base)
        } else {
            base
        };
        let key = if matches!(last, ast::PlaceStep::Index(_)) {
            let key = Self::own_dynamic_text(key);
            self.materialize_place_operand(key)
        } else {
            key
        };
        let mut value = self.expr_with_expect(&place.value, Some(VT::Dyn));
        if place.op != ast::PlaceOp::Set {
            let old = Self::object_get(base.clone(), key.clone(), VT::Dyn, place.span);
            value = Self::coerce_dynamic(
                ResolvedExpr {
                    span: place.span,
                    vt: VT::I32,
                    kind: RExprKind::Binary {
                        op: place_bin_op(place.op),
                        lhs: Box::new(Self::coerce_dynamic(old, VT::I32)),
                        rhs: Box::new(Self::coerce_dynamic(value, VT::I32)),
                    },
                },
                VT::Dyn,
            );
        }
        self.emit_stmt(RStmt::ExprStmt {
            span: place.span,
            value: ResolvedExpr {
                span: place.span,
                vt: VT::Unit,
                kind: RExprKind::ObjectSet {
                    base: Box::new(base),
                    key: Box::new(key),
                    value: Box::new(value),
                },
            },
        });
    }

    /// PLAN-1082 (L-19/L-48): resolve the unified place-chain assignment
    /// `base (.f | [e])* (op)= v`. The grammar orders this rule after
    /// path-assign/index-assign, so the shapes arriving here are mixed
    /// field×index chains, chained indices, and every compound
    /// `+= -= *= /=` form (including the bare-name compound). Every
    /// source subexpression (base prefix chain, index, rhs) hoists into a
    /// synthetic temp exactly once in source order through the statement
    /// sink (the pending_splice protocol keeps them ahead of the write);
    /// the write itself rides the existing ListSet / field-path assign
    /// machinery (no new HIR/MIR nodes).
    fn resolve_place_assign(&mut self, pa: &ast::PlaceAssign) {
        let span = pa.span;
        let Some(xl) = self.lookup(&pa.base.name) else {
            self.bind_err(
                "source.undefined",
                format!("no local named `{}` in scope", pa.base.name),
                pa.base.span,
            );
            let _ = self.expr_iterative(&pa.value);
            return;
        };
        let base_vt = self.locals[xl.0].vt;
        if self.profile == SourceProfile::Auto && self.dynamic_place(base_vt, &pa.steps) {
            self.resolve_dynamic_place_assign(pa);
            return;
        }
        let last_idx = pa
            .steps
            .iter()
            .rposition(|s| matches!(s, ast::PlaceStep::Index(_)));

        let Some(li) = last_idx else {
            // ---- field-only chain (all compound field forms; a plain `=`
            // field chain normally matched path-assign first) ----
            let mut path: Vec<(usize, usize)> = Vec::new();
            let mut cur = base_vt;
            for step in &pa.steps {
                let ast::PlaceStep::Field(seg) = step else {
                    unreachable!("rposition guarantees fields only");
                };
                let VT::Rec(ri) = cur else {
                    self.bind_err(
                        "source.type-mismatch",
                        format!(
                            "`{}` has type `{}`; place paths continue through records only",
                            pa.base.name,
                            cur.as_str()
                        ),
                        seg.span,
                    );
                    return;
                };
                match self.records[ri].fields.iter().position(|fl| fl.name == seg.name) {
                    Some(fi) => {
                        cur = self.records[ri].fields[fi].vt;
                        path.push((ri, fi));
                    }
                    None => {
                        self.bind_err(
                            "source.undefined",
                            format!(
                                "record `{}` has no field named `{}`",
                                self.records[ri].name, seg.name
                            ),
                            seg.span,
                        );
                        return;
                    }
                }
            }
            let mut value = self.expr_with_expect(&pa.value, Some(cur));
            self.reject_elem_view_use(&value, "the assigned value");
            match pa.op {
                ast::PlaceOp::Set => {
                    if cur == VT::StrOwned && matches!(value.kind, RExprKind::StrLit(_)) {
                        value.vt = VT::StrOwned;
                    }
                    self.emit_stmt(RStmt::Assign {
                        local: xl,
                        path,
                        value,
                        span,
                    });
                }
                op => {
                    if cur != VT::I32 {
                        self.bind_err(
                            "source.compound-elem-type",
                            "compound assignment on this place requires an i32 field in this profile"
                                .to_string(),
                            span,
                        );
                    }
                    // rhs hoists FIRST (source order: rhs is the only
                    // side-effecting subexpression on a field chain).
                    let vl = self.agg_temp("pl_v", value.vt, false, span);
                    self.emit_stmt(RStmt::Let {
                        local: vl,
                        value,
                        span,
                    });
                    let read_ast = place_read_ast(&pa.base, &pa.steps);
                    let read = self.expr_iterative(&read_ast);
                    self.emit_stmt(RStmt::Assign {
                        local: xl,
                        path,
                        value: ResolvedExpr {
                            span,
                            vt: VT::I32,
                            kind: RExprKind::Binary {
                                op: place_bin_op(op),
                                lhs: Box::new(read),
                                rhs: Box::new(ResolvedExpr {
                                    span,
                                    vt: VT::I32,
                                    kind: RExprKind::ReadLocal(vl),
                                }),
                            },
                        },
                        span,
                    });
                }
            }
            return;
        };

        // ---- index place: `.. prefix .. [index]` (+ rejected field tail) ----
        if pa.steps[li + 1..]
            .iter()
            .any(|s| matches!(s, ast::PlaceStep::Index(_)))
        {
            unreachable!("rposition guarantees no index after the last");
        }
        if !pa.steps[li + 1..].is_empty() {
            // `xs[i].f = v` — element FIELD writes ride LIST_FIELD_SET,
            // whose source seam is not on this face yet (PLAN-1082 T-01
            // decision D-1082-10: the records batch owns that seam).
            self.bind_err(
                "source.place-elem-field",
                "writing a field of an indexed element (`xs[i].f = v`) is not part of this profile yet; replace the element (`xs[i] = v`) or project the field on the records face"
                    .to_string(),
                span,
            );
            let _ = self.expr_iterative(&pa.value);
            return;
        }

        // base prefix: resolve the read chain (fields and/or earlier
        // indices) and hoist once when it has any real subexpression.
        let prefix = &pa.steps[..li];
        let base_read = if prefix.is_empty() {
            ResolvedExpr {
                span: pa.base.span,
                vt: base_vt,
                kind: RExprKind::ReadLocal(xl),
            }
        } else {
            let prefix_ast = place_read_ast(&pa.base, prefix);
            let b = self.expr_iterative(&prefix_ast);
            let b_vt = b.vt;
            let bl = self.agg_temp("pl_b", b_vt, false, span);
            self.emit_stmt(RStmt::Let {
                local: bl,
                value: b,
                span,
            });
            ResolvedExpr {
                span,
                vt: b_vt,
                kind: RExprKind::ReadLocal(bl),
            }
        };
        let VT::List(lix) = base_read.vt else {
            self.bind_err(
                "source.index-base",
                format!(
                    "index assignment requires a List place, found `{}`",
                    base_read.vt.as_str()
                ),
                pa.base.span,
            );
            let _ = self.expr_iterative(&pa.value);
            return;
        };
        let elem = self.coll.lists[lix];

        // index (hoisted once, in source order after the base chain)
        let ast::PlaceStep::Index(idx_ast) = &pa.steps[li] else {
            unreachable!("rposition guarantees an index here");
        };
        let index = self.expr_iterative(idx_ast);
        if index.vt != VT::I32 {
            self.bind_err(
                "source.index-type",
                "an index must be i32 in this profile".to_string(),
                idx_ast.span,
            );
        }
        let il = self.agg_temp("pl_i", VT::I32, false, span);
        self.emit_stmt(RStmt::Let {
            local: il,
            value: index,
            span,
        });

        // structural-modification-during-iteration guard (same contract as
        // `xs[i] = v` / `.set`): keyed on the ROOT local of the chain.
        if let Some(hit) = self
            .iterate_targets
            .iter()
            .find(|(t, _, _)| *t == xl)
            .map(|(_, _, sp)| *sp)
        {
            self.bind_err(
                "source.iterate-mutate",
                "`xs[i] = v` structurally modifies the list being iterated (legal: element field writes)"
                    .to_string(),
                span.join(hit),
            );
        }

        let read_local = |l| ResolvedExpr {
            span,
            vt: VT::I32,
            kind: RExprKind::ReadLocal(l),
        };
        match pa.op {
            ast::PlaceOp::Set => {
                let mut value = self.expr_with_expect(&pa.value, Some(elem));
                self.reject_elem_view_use(&value, "the stored value");
                if value.vt != elem {
                    self.bind_err(
                        "source.type-mismatch",
                        format!(
                            "element type `{}` expected, found `{}`",
                            elem.as_str(),
                            value.vt.as_str()
                        ),
                        pa.value.span,
                    );
                }
                // an owned element keeps the direct expression (move-source
                // admission); value elements hoist once for uniformity.
                let value_expr = if self.vt_is_coll_owned(elem) {
                    self.check_push_value(&elem, &mut value, pa.base.span);
                    value
                } else {
                    let vl = self.agg_temp("pl_v", value.vt, false, span);
                    self.emit_stmt(RStmt::Let {
                        local: vl,
                        value,
                        span,
                    });
                    ResolvedExpr {
                        span,
                        vt: elem,
                        kind: RExprKind::ReadLocal(vl),
                    }
                };
                self.emit_stmt(RStmt::ExprStmt {
                    value: ResolvedExpr {
                        span,
                        vt: VT::Unit,
                        kind: RExprKind::ListSet {
                            list: Box::new(base_read),
                            index: Box::new(read_local(il)),
                            value: Box::new(value_expr),
                        },
                    },
                    span,
                });
            }
            op => {
                if elem != VT::I32 {
                    self.bind_err(
                        "source.compound-elem-type",
                        "compound assignment through an index place requires i32 elements in this profile"
                            .to_string(),
                        span,
                    );
                }
                let value = self.expr_with_expect(&pa.value, Some(VT::I32));
                if value.vt != VT::I32 {
                    self.bind_err(
                        "source.type-mismatch",
                        format!(
                            "i32 expected for compound assignment, found `{}`",
                            value.vt.as_str()
                        ),
                        pa.value.span,
                    );
                }
                let vl = self.agg_temp("pl_v", value.vt, false, span);
                self.emit_stmt(RStmt::Let {
                    local: vl,
                    value,
                    span,
                });
                let current = ResolvedExpr {
                    span,
                    vt: elem,
                    kind: RExprKind::ListGet {
                        list: Box::new(ResolvedExpr {
                            span,
                            vt: VT::List(lix),
                            kind: RExprKind::ReadLocal(match base_read.kind {
                                RExprKind::ReadLocal(l) => l,
                                _ => unreachable!("hoisted base is a temp read"),
                            }),
                        }),
                        index: Box::new(read_local(il)),
                    },
                };
                self.emit_stmt(RStmt::ExprStmt {
                    value: ResolvedExpr {
                        span,
                        vt: VT::Unit,
                        kind: RExprKind::ListSet {
                            list: Box::new(base_read),
                            index: Box::new(read_local(il)),
                            value: Box::new(ResolvedExpr {
                                span,
                                vt: VT::I32,
                                kind: RExprKind::Binary {
                                    op: place_bin_op(op),
                                    lhs: Box::new(current),
                                    rhs: Box::new(read_local(vl)),
                                },
                            }),
                        },
                    },
                    span,
                });
            }
        }
    }

    fn resolve_unwrap_or(
        &mut self,
        lhs_ast: &ast::Expr,
        rhs_ast: &ast::Expr,
        expect: Option<VT>,
        span: Span,
    ) -> ResolvedExpr {
        if !self.profile.is_aggregates() {
            self.bind_err(
                "source.unsupported",
                "`??` / `.?(default)` requires full aggregates lowering".to_string(),
                span,
            );
            return ResolvedExpr {
                span,
                vt: VT::I32,
                kind: RExprKind::IntLit(0),
            };
        }
        let opt_expect = expect.map(|ev| {
            let vi = self.coll.intern_variant(
                "Option",
                span,
                vec![("Some".to_string(), Some(ev)), ("None".to_string(), None)],
            );
            VT::Variant(vi)
        });
        let lhs = self.expr_with_context(lhs_ast, opt_expect, false);
        if self.profile.is_auto_language() {
            if lhs.vt == VT::Nil {
                self.eval_for_effect(lhs);
                return self.expr_with_expect(rhs_ast, expect);
            }
            if lhs.vt == VT::Dyn {
                let (stmts, fallback, diverges) = self.resolve_expr_in_branch(rhs_ast, expect);
                return self.resolve_dyn_coalesce(lhs, stmts, fallback, diverges, span);
            }
        }
        let Some(info) = agg_lower::variant_unwrap_info(lhs.vt, self.coll) else {
            self.bind_err(
                "source.type-mismatch",
                format!(
                    "`??` / `.?(default)` requires an Option or Result operand, found `{}`",
                    lhs.vt.as_str()
                ),
                lhs.span,
            );
            let _ = self.resolve_expr_in_branch(rhs_ast, expect);
            return ResolvedExpr {
                span,
                vt: VT::I32,
                kind: RExprKind::IntLit(0),
            };
        };
        let scr_l = self.agg_temp("nc_scr", lhs.vt, false, span);
        self.emit_stmt(RStmt::Let {
            local: scr_l,
            value: lhs,
            span,
        });
        let res_vt = info.success_vt;
        let res_l = self.agg_temp("nc_res", res_vt, true, span);
        let init_dummy = self.dummy_of(res_vt, span);
        self.emit_stmt(RStmt::Let {
            local: res_l,
            value: init_dummy,
            span,
        });

        let (mut r_stmts, mut rhs, r_div) = self.resolve_expr_in_branch(rhs_ast, Some(res_vt));
        if !r_div {
            retag_literal(&mut rhs, res_vt);
            if rhs.vt != res_vt {
                self.bind_err(
                    "source.type-mismatch",
                    format!(
                        "`??` fallback type mismatch: expected `{}`, found `{}`",
                        res_vt.as_str(),
                        rhs.vt.as_str()
                    ),
                    rhs.span,
                );
            }
            r_stmts.push(RStmt::Assign {
                local: res_l,
                path: Vec::new(),
                value: rhs,
                span: rhs_ast.span,
            });
        }
        self.emit_stmt(RStmt::If {
            cond: ResolvedExpr {
                span,
                vt: VT::Bool,
                kind: RExprKind::VariantGuardTest {
                    local: scr_l,
                    variant: info.vi,
                    disc: info.success_disc,
                },
            },
            then: RBlock {
                stmts: vec![RStmt::Assign {
                    local: res_l,
                    path: Vec::new(),
                    value: ResolvedExpr {
                        span,
                        vt: res_vt,
                        kind: RExprKind::VariantProject {
                            local: scr_l,
                            variant: info.vi,
                            disc: info.success_disc,
                        },
                    },
                    span,
                }],
                span,
            },
            els: Some(RBlock {
                stmts: r_stmts,
                span: rhs_ast.span,
            }),
            span,
        });
        ResolvedExpr {
            span,
            vt: res_vt,
            kind: RExprKind::ReadLocal(res_l),
        }
    }

    fn resolve_propagate(&mut self, inner_ast: &ast::Expr, span: Span) -> ResolvedExpr {
        if !self.profile.is_aggregates() {
            self.bind_err(
                "source.unsupported",
                "`.?` requires full aggregates lowering".to_string(),
                span,
            );
            return ResolvedExpr {
                span,
                vt: VT::I32,
                kind: RExprKind::IntLit(0),
            };
        }
        let val = self.expr_iterative(inner_ast);
        let Some(info) = agg_lower::variant_unwrap_info(val.vt, self.coll) else {
            self.bind_err(
                "source.propagate-incompatible",
                format!(
                    "`.?` requires an Option or Result operand, found `{}`",
                    val.vt.as_str()
                ),
                span,
            );
            return ResolvedExpr {
                span,
                vt: VT::I32,
                kind: RExprKind::IntLit(0),
            };
        };
        let q_l = self.agg_temp("q_scr", val.vt, false, span);
        self.emit_stmt(RStmt::Let {
            local: q_l,
            value: val,
            span,
        });
        let res_vt = info.success_vt;
        let res_l = self.agg_temp("q_res", res_vt, true, span);
        let init_dummy = self.dummy_of(res_vt, span);
        self.emit_stmt(RStmt::Let {
            local: res_l,
            value: init_dummy,
            span,
        });

        let mut err_stmts = Vec::new();
        let ret_info = agg_lower::variant_unwrap_info(self.fn_result, self.coll);
        if info.is_option {
            match ret_info {
                Some(ri) if ri.is_option => {
                    let none_ret = ResolvedExpr {
                        span,
                        vt: self.fn_result,
                        kind: RExprKind::VariantCtor {
                            variant: ri.vi,
                            disc: 1,
                            payload: None,
                        },
                    };
                    self.emit_return_to_out(Some(none_ret), span, &mut err_stmts);
                }
                _ => {
                    self.bind_err(
                        "source.propagate-incompatible",
                        "cannot propagate `Option` with `.?` in a function that does not return `?T`".to_string(),
                        span,
                    );
                }
            }
        } else {
            // Result propagation:
            // 1. Inside try..catch -> throw/unwind to catch
            // 2. Else inside `!U` fn -> return Err(e) (running enclosing finally)
            // 3. Else top-level script -> runtime LangFail (ExecOutcome::Error)
            // 4. Else non-Result function -> source.propagate-incompatible
            let has_catch = self.has_enclosing_catch();
            let ret_is_result = matches!(ret_info, Some(ri) if !ri.is_option);
            let err_l = self.agg_temp("q_err", info.err_vt.unwrap_or(VT::StrOwned), false, span);
            let mut action_stmts = Vec::new();
            action_stmts.push(RStmt::Let {
                local: err_l,
                value: ResolvedExpr {
                    span,
                    vt: info.err_vt.unwrap_or(VT::StrOwned),
                    kind: RExprKind::VariantProject {
                        local: q_l,
                        variant: info.vi,
                        disc: 1,
                    },
                },
                span,
            });
            if has_catch {
                let err_vt = info.err_vt.unwrap_or(VT::StrOwned);
                let mut err_read = ResolvedExpr {
                    span,
                    vt: err_vt,
                    kind: RExprKind::ReadLocal(err_l),
                };
                if self.vt_is_coll_owned(err_vt) {
                    err_read = ResolvedExpr {
                        span,
                        vt: err_vt,
                        kind: RExprKind::Move {
                            inner: Box::new(err_read),
                        },
                    };
                }
                self.emit_fail_to_out(err_read, span, &mut action_stmts);
            } else if ret_is_result {
                let ri = ret_info.unwrap();
                let err_vt = info.err_vt.unwrap_or(VT::StrOwned);
                let mut err_val = ResolvedExpr {
                    span,
                    vt: err_vt,
                    kind: RExprKind::ReadLocal(err_l),
                };
                if self.vt_is_coll_owned(err_vt) {
                    err_val = ResolvedExpr {
                        span,
                        vt: err_vt,
                        kind: RExprKind::Move {
                            inner: Box::new(err_val),
                        },
                    };
                }
                if let Some(want_err_vt) = ri.err_vt {
                    if err_val.vt != want_err_vt && want_err_vt == VT::StrOwned {
                        err_val = self.coerce_err_to_str_owned(err_val, span);
                    }
                }
                let ret_variant = ResolvedExpr {
                    span,
                    vt: self.fn_result,
                    kind: RExprKind::VariantCtor {
                        variant: ri.vi,
                        disc: 1,
                        payload: Some(Box::new(err_val)),
                    },
                };
                self.emit_return_to_out(Some(ret_variant), span, &mut action_stmts);
            } else if self.is_script {
                let err_read = ResolvedExpr {
                    span,
                    vt: info.err_vt.unwrap_or(VT::StrOwned),
                    kind: RExprKind::ReadLocal(err_l),
                };
                self.emit_fail_to_out(err_read, span, &mut action_stmts);
            } else {
                self.bind_err(
                    "source.propagate-incompatible",
                    "cannot propagate `Result` with `.?` in a function that does not return `!T` outside `catch`".to_string(),
                    span,
                );
            }
            err_stmts.push(RStmt::If {
                cond: ResolvedExpr {
                    span,
                    vt: VT::Bool,
                    kind: RExprKind::VariantGuardTest {
                        local: q_l,
                        variant: info.vi,
                        disc: 1,
                    },
                },
                then: RBlock {
                    stmts: action_stmts,
                    span,
                },
                els: None,
                span,
            });
        }

        self.emit_stmt(RStmt::If {
            cond: ResolvedExpr {
                span,
                vt: VT::Bool,
                kind: RExprKind::VariantGuardTest {
                    local: q_l,
                    variant: info.vi,
                    disc: info.success_disc,
                },
            },
            then: RBlock {
                stmts: vec![RStmt::Assign {
                    local: res_l,
                    path: Vec::new(),
                    value: ResolvedExpr {
                        span,
                        vt: res_vt,
                        kind: RExprKind::VariantProject {
                            local: q_l,
                            variant: info.vi,
                            disc: info.success_disc,
                        },
                    },
                    span,
                }],
                span,
            },
            els: Some(RBlock {
                stmts: err_stmts,
                span,
            }),
            span,
        });
        ResolvedExpr {
            span,
            vt: res_vt,
            kind: RExprKind::ReadLocal(res_l),
        }
    }

    fn resolve_short_circuit_bin(
        &mut self,
        op: BinOp,
        lhs_ast: &ast::Expr,
        rhs_ast: &ast::Expr,
        span: Span,
    ) -> ResolvedExpr {
        let lhs = self.expr_iterative(lhs_ast);
        let lhs = self.condition_value(lhs);
        let sc_l = self.agg_temp("sc", VT::Bool, true, span);
        self.emit_stmt(RStmt::Let {
            local: sc_l,
            value: lhs,
            span,
        });
        let (mut r_stmts, rhs, _) =
            self.resolve_in_branch(|resolver| resolver.resolve_condition(rhs_ast));
        r_stmts.push(RStmt::Assign {
            local: sc_l,
            path: Vec::new(),
            value: rhs,
            span: rhs_ast.span,
        });
        let cond = if op == BinOp::And {
            ResolvedExpr {
                span,
                vt: VT::Bool,
                kind: RExprKind::ReadLocal(sc_l),
            }
        } else {
            ResolvedExpr {
                span,
                vt: VT::Bool,
                kind: RExprKind::Unary {
                    op: UnaryOp::Not,
                    operand: Box::new(ResolvedExpr {
                        span,
                        vt: VT::Bool,
                        kind: RExprKind::ReadLocal(sc_l),
                    }),
                },
            }
        };
        self.emit_stmt(RStmt::If {
            cond,
            then: RBlock {
                stmts: r_stmts,
                span: rhs_ast.span,
            },
            els: None,
            span,
        });
        ResolvedExpr {
            span,
            vt: VT::Bool,
            kind: RExprKind::ReadLocal(sc_l),
        }
    }

    fn resolve_agg_is(
        &mut self,
        scrutinee_ast: &ast::Expr,
        arms: &[ast::AggMatchArm],
        expect: Option<VT>,
        is_expr_mode: bool,
        span: Span,
        has_following: bool,
    ) -> (Vec<RStmt>, Option<ResolvedExpr>) {
        let mut out_stmts = Vec::new();
        if !self.profile.is_aggregates() {
            self.bind_err(
                "source.unsupported",
                "`is` expression / pattern arm requires full aggregates lowering".to_string(),
                span,
            );
            return (
                out_stmts,
                if is_expr_mode {
                    Some(ResolvedExpr {
                        span,
                        vt: VT::I32,
                        kind: RExprKind::IntLit(0),
                    })
                } else {
                    None
                },
            );
        }
        let scr = self.expr_iterative(scrutinee_ast);
        let scr_vt = scr.vt;
        let scr_l = self.agg_temp("is_scr", scr_vt, false, scrutinee_ast.span);
        out_stmts.push(RStmt::Let {
            local: scr_l,
            value: scr,
            span: scrutinee_ast.span,
        });
        out_stmts.extend(self.drain_sink());

        let mut seen_else = false;
        let mut seen_discs: BTreeSet<u32> = BTreeSet::new();
        let mut seen_ints: BTreeSet<i32> = BTreeSet::new();
        let mut seen_bools: BTreeSet<bool> = BTreeSet::new();

        enum PatCond {
            Else,
            Variant {
                vi: usize,
                disc: u32,
                /// PLAN-1083: payload bindings in source order — one for the
                /// PLAN-864 single payload, more for multi-field/tuple
                /// destructuring.
                binds: Vec<ast::Ident>,
                payload_vt: Option<VT>,
            },
            Int(i32),
            Bool(bool),
        }

        struct ArmResolved {
            pat: PatCond,
            /// PLAN-1083: the payload-binding Lets, prepended at chain
            /// build (a guard evaluates AFTER them, before the body).
            prefix: Vec<RStmt>,
            block: RBlock,
            expr_val: Option<ResolvedExpr>,
            /// PLAN-1083: the resolved guard — Some only when the arm
            /// carried `pat if cond -> body`.
            guard: Option<ResolvedExpr>,
            span: Span,
        }

        let mut compiled: Vec<ArmResolved> = Vec::with_capacity(arms.len());
        let mut inferred_vt: Option<VT> = expect;

        for arm in arms {
            if seen_else {
                self.bind_err(
                    "source.pattern-unreachable",
                    "`else` arm must be last in `is` match".to_string(),
                    arm.span,
                );
            }
            let pat = match &arm.pat {
                ast::MatchPattern::Else => {
                    seen_else = true;
                    PatCond::Else
                }
                ast::MatchPattern::IntLit(n) => {
                    if scr_vt != VT::I32 {
                        self.bind_err(
                            "source.type-mismatch",
                            format!("integer pattern `{n}` cannot match `{}`", scr_vt.as_str()),
                            arm.span,
                        );
                    }
                    if !seen_ints.insert(*n) {
                        self.bind_err(
                            "source.pattern-duplicate",
                            format!("duplicate `is` pattern `{n}`"),
                            arm.span,
                        );
                    }
                    PatCond::Int(*n)
                }
                ast::MatchPattern::BoolLit(b) => {
                    if scr_vt != VT::Bool {
                        self.bind_err(
                            "source.type-mismatch",
                            format!("boolean pattern `{b}` cannot match `{}`", scr_vt.as_str()),
                            arm.span,
                        );
                    }
                    if !seen_bools.insert(*b) {
                        self.bind_err(
                            "source.pattern-duplicate",
                            format!("duplicate `is` pattern `{b}`"),
                            arm.span,
                        );
                    }
                    PatCond::Bool(*b)
                }
                ast::MatchPattern::Variant {
                    qual,
                    name,
                    bindings,
                } => {
                    // PLAN-1083: a qualified scalar-enum arm on an int
                    // scrutinee matches BY VALUE (Color.Red == 1).
                    if scr_vt == VT::I32 {
                        if let Some(q) = qual.as_ref() {
                            let found = self
                                .coll
                                .scalar_enums
                                .get(&q.name)
                                .and_then(|arms| {
                                    arms.iter().find(|(n, _)| *n == name.name).map(|(_, v)| *v)
                                });
                            if let Some(v) = found {
                                match Some(v) {
                                    Some(v) => {
                                        if !seen_ints.insert(v) {
                                            self.bind_err(
                                                "source.pattern-duplicate",
                                                format!(
                                                    "duplicate `is` pattern `{}.{}`",
                                                    q.name, name.name
                                                ),
                                                arm.span,
                                            );
                                        }
                                        PatCond::Int(v)
                                    }
                                    None => {
                                        self.bind_err(
                                            "source.pattern-unknown-arm",
                                            format!(
                                                "scalar enum `{}` has no arm named `{}`",
                                                q.name, name.name
                                            ),
                                            arm.span,
                                        );
                                        continue;
                                    }
                                }
                            } else {
                                self.bind_err(
                                    "source.type-mismatch",
                                    format!(
                                        "`{}` does not name a scalar enum; `{}` cannot match `{}`",
                                        q.name,
                                        name.name,
                                        scr_vt.as_str()
                                    ),
                                    arm.span,
                                );
                                continue;
                            }
                        } else {
                            self.bind_err(
                                "source.type-mismatch",
                                format!(
                                    "pattern `{}` cannot match `{}`",
                                    name.name, scr_vt.as_str()
                                ),
                                arm.span,
                            );
                            continue;
                        }
                    } else {
                    let Some((vi, disc)) = self.is_arm_disc(scr_vt, qual.as_ref(), name) else {
                        continue;
                    };
                    if !seen_discs.insert(disc) {
                        // PLAN-1083: a disc may repeat while every EARLIER
                        // arm for it is guarded (guard false falls through
                        // to the repeat); a repeat after a plain arm is
                        // unreachable and stays the frozen duplicate.
                        let dup_ok = compiled.iter().all(|c| {
                            !matches!(&c.pat, PatCond::Variant { disc: d, .. } if *d == disc)
                                || c.guard.is_some()
                        });
                        if !dup_ok {
                            self.bind_err(
                                "source.pattern-duplicate",
                                format!("duplicate `is` arm `{}`", name.name),
                                arm.span,
                            );
                        }
                    }
                    let payload_vt = self.coll.variants[vi].arms[disc as usize].1;
                    if let Some(pvt) = payload_vt {
                        let named_arm = self.coll.named_field_arms.contains(&(vi, disc));
                        let want = match pvt {
                            VT::Tuple(ti) => self.coll.tuples.get(ti).map(|t| t.len()),
                            // a NAMED-FIELD payload binds field-by-field; an
                            // anonymous record-typed payload is ONE value.
                            VT::Rec(_) if named_arm => match pvt {
                                VT::Rec(ri) => self.records.get(ri).map(|r| r.fields.len()),
                                _ => None,
                            },
                            _ => None,
                        };
                        if let Some(want) = want {
                            if !bindings.is_empty() && bindings.len() != want {
                                self.bind_err(
                                    "source.pattern-bindings",
                                    format!(
                                        "arm `{}` payload carries {} value(s); the pattern binds {}",
                                        name.name,
                                        want,
                                        bindings.len()
                                    ),
                                    arm.span,
                                );
                            }
                        } else if bindings.len() > 1 {
                            self.bind_err(
                                "source.pattern-bindings",
                                format!(
                                    "arm `{}` carries a single-value payload; multi-binding destructuring needs a tuple/record payload",
                                    name.name
                                ),
                                arm.span,
                            );
                        }
                    } else if !bindings.is_empty() {
                        self.bind_err(
                            "source.type-mismatch",
                            format!("arm `{}` carries no payload; remove the binding", name.name),
                            arm.span,
                        );
                    }
                    PatCond::Variant {
                        vi,
                        disc,
                        binds: bindings.clone(),
                        payload_vt,
                    }
                    }
                }
            };

            self.scopes.push(Scope {
                names: BTreeMap::new(),
                views: BTreeMap::new(),
                text_mut_aliases: BTreeMap::new(),
            });
            let mut prefix_stmts = Vec::new();
            // PLAN-1083: multi-binding destructuring — the payload projects
            // ONCE into a step-local temp; each binding then reads its tuple
            // element / record field. A single binding keeps the PLAN-864
            // direct-projection shape (byte-identical lowering).
            if let PatCond::Variant {
                vi,
                disc,
                binds,
                payload_vt: Some(pvt),
            } = &pat
            {
                let project = |span: Span| ResolvedExpr {
                    span,
                    vt: *pvt,
                    kind: RExprKind::VariantProject {
                        local: scr_l,
                        variant: *vi,
                        disc: *disc,
                    },
                };
                let named_arm = self.coll.named_field_arms.contains(&(*vi, *disc));
                if binds.len() == 1 && named_arm {
                    // PLAN-1083: `Arm(f) -> ..` on a NAMED-FIELD payload
                    // binds the single declared field (never the whole
                    // synthesized record).
                    if let Some(bident) = binds.first() {
                        let hidden = self.agg_temp("__is_payload", *pvt, false, bident.span);
                        prefix_stmts.push(RStmt::Let {
                            local: hidden,
                            value: project(bident.span),
                            span: bident.span,
                        });
                        let (evt, kind) = match *pvt {
                            VT::Rec(ri) => {
                                let evt = self
                                    .records
                                    .get(ri)
                                    .and_then(|r| r.fields.first().map(|f| f.vt))
                                    .unwrap_or(VT::I32);
                                (
                                    evt,
                                    RExprKind::FieldGet {
                                        base: Box::new(ResolvedExpr {
                                            span: bident.span,
                                            vt: *pvt,
                                            kind: RExprKind::ReadLocal(hidden),
                                        }),
                                        path: vec![(ri, 0)],
                                    },
                                )
                            }
                            _ => (VT::I32, RExprKind::ReadLocal(hidden)),
                        };
                        let bl = LocalRef(self.locals.len());
                        self.locals.push(LocalDecl {
                            name: bident.name.clone(),
                            vt: evt,
                            mutable: false,
                            is_param: false,
                            param_index: None,
                            capture_index: None,
                            cell: false,
                            is_capture: false,
                            mode: ast::ParamMode::Default,
                            span: bident.span,
                        });
                        self.scopes
                            .last_mut()
                            .unwrap()
                            .names
                            .insert(bident.name.clone(), bl);
                        self.declared_names.insert(bident.name.clone());
                        prefix_stmts.push(RStmt::Let {
                            local: bl,
                            value: ResolvedExpr {
                                span: bident.span,
                                vt: evt,
                                kind,
                            },
                            span: bident.span,
                        });
                    }
                } else if binds.len() <= 1 {
                    if let Some(bident) = binds.first() {
                        let bl = LocalRef(self.locals.len());
                        self.locals.push(LocalDecl {
                            name: bident.name.clone(),
                            vt: *pvt,
                            mutable: false,
                            is_param: false,
                            param_index: None,
                            capture_index: None,
                            cell: false,
                            is_capture: false,
                            mode: ast::ParamMode::Default,
                            span: bident.span,
                        });
                        self.scopes
                            .last_mut()
                            .unwrap()
                            .names
                            .insert(bident.name.clone(), bl);
                        self.declared_names.insert(bident.name.clone());
                        prefix_stmts.push(RStmt::Let {
                            local: bl,
                            value: project(bident.span),
                            span: bident.span,
                        });
                    }
                } else {
                    let hidden = self.agg_temp("__is_payload", *pvt, false, binds[0].span);
                    prefix_stmts.push(RStmt::Let {
                        local: hidden,
                        value: project(binds[0].span),
                        span: binds[0].span,
                    });
                    for (i, bident) in binds.iter().enumerate() {
                        let (evt, kind) = match *pvt {
                            VT::Tuple(ti) => {
                                let evt = self
                                    .coll
                                    .tuples
                                    .get(ti)
                                    .and_then(|t| t.get(i).copied())
                                    .unwrap_or(VT::I32);
                                (
                                    evt,
                                    RExprKind::TupleGet {
                                        base: Box::new(ResolvedExpr {
                                            span: bident.span,
                                            vt: *pvt,
                                            kind: RExprKind::ReadLocal(hidden),
                                        }),
                                        index: i,
                                    },
                                )
                            }
                            VT::Rec(ri) => {
                                let evt = self
                                    .records
                                    .get(ri)
                                    .and_then(|r| r.fields.get(i).map(|f| f.vt))
                                    .unwrap_or(VT::I32);
                                (
                                    evt,
                                    RExprKind::FieldGet {
                                        base: Box::new(ResolvedExpr {
                                            span: bident.span,
                                            vt: *pvt,
                                            kind: RExprKind::ReadLocal(hidden),
                                        }),
                                        path: vec![(ri, i)],
                                    },
                                )
                            }
                            _ => (VT::I32, RExprKind::ReadLocal(hidden)),
                        };
                        let bl = LocalRef(self.locals.len());
                        self.locals.push(LocalDecl {
                            name: bident.name.clone(),
                            vt: evt,
                            mutable: false,
                            is_param: false,
                            param_index: None,
                            capture_index: None,
                            cell: false,
                            is_capture: false,
                            mode: ast::ParamMode::Default,
                            span: bident.span,
                        });
                        self.scopes
                            .last_mut()
                            .unwrap()
                            .names
                            .insert(bident.name.clone(), bl);
                        self.declared_names.insert(bident.name.clone());
                        prefix_stmts.push(RStmt::Let {
                            local: bl,
                            value: ResolvedExpr {
                                span: bident.span,
                                vt: evt,
                                kind,
                            },
                            span: bident.span,
                        });
                    }
                }
            }
            // PLAN-1083: the guard resolves INSIDE the arm scope (the
            // pattern bindings are visible) and must be bool-typed.
            let guard = arm.guard.as_ref().map(|g| {
                let cond = self.resolve_condition(g);
                if cond.vt != VT::Bool {
                    self.bind_err(
                        "source.type-mismatch",
                        format!("`is` guard must be bool, found `{}`", cond.vt.as_str()),
                        g.span,
                    );
                }
                cond
            });

            let (ablock, expr_val) = match &arm.body {
                ast::AggArmBody::Block(b) => {
                    let mut rb = self.resolve_body(b, true);
                    let mut val = None;
                    if is_expr_mode {
                        if let Some(RStmt::ExprStmt { value, .. }) = rb.stmts.last().cloned() {
                            if !self.expr_diverged {
                                rb.stmts.pop();
                                if inferred_vt.is_none() {
                                    inferred_vt = Some(value.vt);
                                }
                                val = Some(value);
                            }
                        }
                    }
                    (rb, val)
                }
                ast::AggArmBody::Expr(e) => {
                    if is_expr_mode {
                        let (stmts, mut val, div) = self.resolve_expr_in_branch(e, inferred_vt);
                        if !div {
                            if let Some(ivt) = inferred_vt {
                                retag_literal(&mut val, ivt);
                            } else {
                                inferred_vt = Some(val.vt);
                            }
                        }
                        (
                            RBlock {
                                stmts,
                                span: e.span,
                            },
                            if div { None } else { Some(val) },
                        )
                    } else {
                        let (mut stmts, val, div) = self.resolve_expr_in_branch(e, None);
                        if !div {
                            stmts.push(RStmt::ExprStmt {
                                value: val,
                                span: e.span,
                            });
                        }
                        (
                            RBlock {
                                stmts,
                                span: e.span,
                            },
                            None,
                        )
                    }
                }
            };
            self.scopes.pop();
            compiled.push(ArmResolved {
                pat,
                prefix: prefix_stmts,
                block: ablock,
                expr_val,
                guard,
                span: arm.span,
            });
        }

        // Exhaustiveness check. PLAN-1083: the ordinary auto face admits
        // a fall-through STATEMENT `is` (no arm matched → execution
        // continues after the statement; the legacy enum-method family
        // spells exactly this). Value-producing positions — the is
        // EXPRESSION and every frozen profile's statement form — keep the
        // exhaustive contract.
        if !seen_else && (is_expr_mode || !self.profile.is_auto_language()) {
            let ok = match scr_vt {
                VT::Variant(vi) => seen_discs.len() == self.coll.variants[vi].arms.len(),
                VT::Bool => seen_bools.len() == 2,
                _ => false,
            };
            if !ok {
                self.bind_err(
                    "source.pattern-nonexhaustive",
                    "`is` match is non-exhaustive (add remaining arms or an `else` arm)"
                        .to_string(),
                    span,
                );
            }
        }

        // An effect-valued match executes the selected arm but has no
        // storage slot: Unit is a result, never a local value in AV MIR.
        let unit_result = self.profile.is_auto_language()
            && is_expr_mode
            && inferred_vt == Some(VT::Unit);
        let res_local = if is_expr_mode && !unit_result {
            let rv = inferred_vt.unwrap_or(VT::Unit);
            let rl = self.agg_temp("is_res", rv, true, span);
            let dummy = self.dummy_of(rv, span);
            out_stmts.push(RStmt::Let {
                local: rl,
                value: dummy,
                span,
            });
            Some((rl, rv))
        } else {
            None
        };

        fn rblock_all_return(stmts: &[RStmt]) -> bool {
            let Some(last) = stmts.last() else {
                return false;
            };
            match last {
                RStmt::Return { .. } | RStmt::Fail { .. } => true,
                RStmt::If {
                    then,
                    els: Some(els),
                    ..
                } => rblock_all_return(&then.stmts) && rblock_all_return(&els.stmts),
                _ => false,
            }
        }
        let has_else = compiled.iter().any(|ca| matches!(ca.pat, PatCond::Else));
        let mut chain: Option<RBlock> = if !is_expr_mode
            && !has_else
            && !compiled.is_empty()
            && compiled.iter().all(|ca| rblock_all_return(&ca.block.stmts))
            // PLAN-1083: the auto face's fall-through statement `is` adds
            // the synthetic return filler ONLY when nothing follows the
            // statement (a trailing filler would strand later code
            // unreachable; without it an all-arms-return tail has its
            // Returned spine).
            && !(self.profile.is_auto_language() && has_following)
        {
            let mut filler_stmts = Vec::new();
            if self.is_generator {
                self.emit_return_to_out(None, span, &mut filler_stmts);
            } else {
                let f = self.dummy_of(self.fn_result, span);
                self.emit_return_to_out(Some(f), span, &mut filler_stmts);
            }
            Some(RBlock {
                stmts: filler_stmts,
                span,
            })
        } else {
            None
        };
        for mut ca in compiled.into_iter().rev() {
            if let Some((rl, rv)) = res_local {
                if let Some(mut ev) = ca.expr_val.take() {
                    retag_literal(&mut ev, rv);
                    if expect == Some(VT::F32)
                        && ev.vt == VT::I32
                        && self.coll.legacy_age_widget_at(ev.span)
                    {
                        // An authored float destination widens each selected
                        // numeric row field before the branches join.
                        ev = self.coll.age_widget_numeric_cast(ev, VT::F32);
                    }
                    if ev.vt != rv {
                        self.bind_err(
                            "source.type-mismatch",
                            format!(
                                "`is` arm type mismatch: expected `{}`, found `{}`",
                                rv.as_str(),
                                ev.vt.as_str()
                            ),
                            ev.span,
                        );
                    }
                    // PLAN-1083: the value assign lands at the END of the
                    // arm body — inside the guard's then-block below.
                    ca.block.stmts.push(RStmt::Assign {
                        local: rl,
                        path: Vec::new(),
                        value: ev,
                        span: ca.span,
                    });
                }
            }
            if unit_result {
                if let Some(value) = ca.expr_val.take() {
                    if value.vt != VT::Unit {
                        self.bind_err(
                            "source.type-mismatch",
                            format!(
                                "is arm type mismatch: expected unit, found {}",
                                value.vt.as_str()
                            ),
                            value.span,
                        );
                    }
                    ca.block.stmts.push(RStmt::ExprStmt {
                        value,
                        span: ca.span,
                    });
                }
            }
            // PLAN-1083: assemble [prefix Lets, guarded?] body — a guard
            // false falls through to the REMAINING chain (els), never to
            // the end of this arm's block.
            let guard = ca.guard.take();
            let block_span = ca.block.span;
            let mut then_stmts = ca.prefix;
            match guard {
                Some(g) => {
                    // The guard's else and the pattern test's else BOTH
                    // continue to the next arm — clone for the guard (the
                    // pattern wrap below consumes the original).
                    let els = chain.clone();
                    then_stmts.push(RStmt::If {
                        cond: g,
                        then: ca.block,
                        els,
                        span: ca.span,
                    });
                }
                None => {
                    let mut block = ca.block;
                    then_stmts.append(&mut block.stmts);
                }
            }
            ca.block = RBlock {
                stmts: then_stmts,
                span: block_span,
            };
            match ca.pat {
                PatCond::Else => {
                    chain = Some(ca.block);
                }
                PatCond::Variant { vi, disc, .. } => {
                    let cond = ResolvedExpr {
                        span: ca.span,
                        vt: VT::Bool,
                        kind: RExprKind::VariantGuardTest {
                            local: scr_l,
                            variant: vi,
                            disc,
                        },
                    };
                    chain = Some(RBlock {
                        stmts: vec![RStmt::If {
                            cond,
                            then: ca.block,
                            els: chain.take(),
                            span: ca.span,
                        }],
                        span: ca.span,
                    });
                }
                PatCond::Int(n) => {
                    let cond = ResolvedExpr {
                        span: ca.span,
                        vt: VT::Bool,
                        kind: RExprKind::Binary {
                            op: BinOp::Eq,
                            lhs: Box::new(ResolvedExpr {
                                span: ca.span,
                                vt: VT::I32,
                                kind: RExprKind::ReadLocal(scr_l),
                            }),
                            rhs: Box::new(ResolvedExpr {
                                span: ca.span,
                                vt: VT::I32,
                                kind: RExprKind::IntLit(n),
                            }),
                        },
                    };
                    chain = Some(RBlock {
                        stmts: vec![RStmt::If {
                            cond,
                            then: ca.block,
                            els: chain.take(),
                            span: ca.span,
                        }],
                        span: ca.span,
                    });
                }
                PatCond::Bool(b) => {
                    let cond = ResolvedExpr {
                        span: ca.span,
                        vt: VT::Bool,
                        kind: RExprKind::Binary {
                            op: BinOp::Eq,
                            lhs: Box::new(ResolvedExpr {
                                span: ca.span,
                                vt: VT::Bool,
                                kind: RExprKind::ReadLocal(scr_l),
                            }),
                            rhs: Box::new(ResolvedExpr {
                                span: ca.span,
                                vt: VT::Bool,
                                kind: RExprKind::BoolLit(b),
                            }),
                        },
                    };
                    chain = Some(RBlock {
                        stmts: vec![RStmt::If {
                            cond,
                            then: ca.block,
                            els: chain.take(),
                            span: ca.span,
                        }],
                        span: ca.span,
                    });
                }
            }
        }
        if let Some(mut c) = chain {
            out_stmts.extend(std::mem::take(&mut c.stmts));
        }
        let expr_out = res_local.map(|(rl, rv)| ResolvedExpr {
            span,
            vt: rv,
            kind: RExprKind::ReadLocal(rl),
        });
        let expr_out = if unit_result {
            Some(ResolvedExpr {
                span,
                vt: VT::Unit,
                kind: RExprKind::Unit,
            })
        } else {
            expr_out
        };
        (out_stmts, expr_out)
    }

    fn resolve_try_stmt(
        &mut self,
        body: &ast::Block,
        catch: Option<&ast::CatchClause>,
        finally: Option<&ast::Block>,
        span: Span,
        has_following: bool,
    ) -> Vec<RStmt> {
        let mut out = Vec::new();
        if !self.profile.is_aggregates() {
            self.bind_err(
                "source.unsupported",
                "`try`/`catch`/`finally` requires full aggregates lowering".to_string(),
                span,
            );
            return out;
        }
        let cs = self.ensure_ctrl_state(span);
        let try_loop = LoopRef(self.loop_count);
        self.loop_count += 1;
        self.ctrl_frames.push(CtrlFrame::Try {
            try_loop,
            has_catch: catch.is_some(),
        });
        fn rstmts_diverge(stmts: &[RStmt]) -> bool {
            let Some(last) = stmts.last() else {
                return false;
            };
            match last {
                RStmt::Return { .. }
                | RStmt::Fail { .. }
                | RStmt::Break { .. }
                | RStmt::Continue { .. } => true,
                RStmt::If {
                    then,
                    els: Some(els),
                    ..
                } => rstmts_diverge(&then.stmts) && rstmts_diverge(&els.stmts),
                _ => false,
            }
        }
        let mut r_try = self.resolve_body(body, false);
        let try_diverges = rstmts_diverge(&r_try.stmts);
        if !try_diverges {
            r_try.stmts.push(RStmt::Break {
                target: Some(try_loop),
                span,
            });
        }
        self.ctrl_frames.pop();
        out.push(RStmt::Loop {
            body: r_try,
            span,
            target: try_loop,
            unwind: (self.profile.is_auto_language() && !self.is_generator)
                .then_some((cs.kind, cs.err)),
        });

        let mut catch_diverges = true;
        if let Some(cc) = catch {
            let err_l = self.agg_temp("catch_e", VT::StrOwned, false, cc.span);
            let catch_loop = LoopRef(self.loop_count);
            self.loop_count += 1;
            let mut catch_wrapper = vec![
                RStmt::Assign {
                    local: cs.kind,
                    path: Vec::new(),
                    value: ResolvedExpr {
                        span: cc.span,
                        vt: VT::I32,
                        kind: RExprKind::IntLit(0),
                    },
                    span: cc.span,
                },
                RStmt::Let {
                    local: err_l,
                    value: ResolvedExpr {
                        span: cc.span,
                        vt: VT::StrOwned,
                        kind: RExprKind::Move {
                            inner: Box::new(ResolvedExpr {
                                span: cc.span,
                                vt: VT::StrOwned,
                                kind: RExprKind::ReadLocal(cs.err),
                            }),
                        },
                    },
                    span: cc.span,
                },
                RStmt::Assign {
                    local: cs.err,
                    path: Vec::new(),
                    value: ResolvedExpr {
                        span: cc.span,
                        vt: VT::StrOwned,
                        kind: RExprKind::StrLit(String::new()),
                    },
                    span: cc.span,
                },
            ];
            self.scopes.push(Scope {
                names: BTreeMap::new(),
                views: BTreeMap::new(),
                text_mut_aliases: BTreeMap::new(),
            });
            let mut rethrow_local = err_l;
            if let Some(bind_ident) = &cc.bind {
                let user_err = LocalRef(self.locals.len());
                self.locals.push(LocalDecl {
                    name: bind_ident.name.clone(),
                    vt: VT::StrOwned,
                    mutable: false,
                    is_param: false,
                    param_index: None,
                    capture_index: None,
                    cell: false,
                    is_capture: false,
                    mode: ast::ParamMode::Default,
                    span: bind_ident.span,
                });
                self.scopes
                    .last_mut()
                    .unwrap()
                    .names
                    .insert(bind_ident.name.clone(), user_err);
                self.declared_names.insert(bind_ident.name.clone());
                catch_wrapper.push(RStmt::Let {
                    local: user_err,
                    value: ResolvedExpr {
                        span: bind_ident.span,
                        vt: VT::StrOwned,
                        kind: RExprKind::Move {
                            inner: Box::new(ResolvedExpr {
                                span: bind_ident.span,
                                vt: VT::StrOwned,
                                kind: RExprKind::ReadLocal(err_l),
                            }),
                        },
                    },
                    span: bind_ident.span,
                });
                rethrow_local = user_err;
            }
            self.catch_err_locals.push(rethrow_local);
            self.ctrl_frames.push(CtrlFrame::Try {
                try_loop: catch_loop,
                has_catch: false,
            });
            let mut r_catch = self.resolve_body(&cc.body, false);
            catch_diverges = rstmts_diverge(&r_catch.stmts);
            if !catch_diverges {
                r_catch.stmts.push(RStmt::Break {
                    target: Some(catch_loop),
                    span: cc.span,
                });
            }
            self.ctrl_frames.pop();
            self.catch_err_locals.pop();
            self.scopes.pop();
            catch_wrapper.push(RStmt::Loop {
                body: r_catch,
                span: cc.span,
                target: catch_loop,
                unwind: (self.profile.is_auto_language() && !self.is_generator)
                    .then_some((cs.kind, cs.err)),
            });
            out.push(RStmt::If {
                cond: ResolvedExpr {
                    span: cc.span,
                    vt: VT::Bool,
                    kind: RExprKind::Binary {
                        op: BinOp::Eq,
                        lhs: Box::new(ResolvedExpr {
                            span: cc.span,
                            vt: VT::I32,
                            kind: RExprKind::ReadLocal(cs.kind),
                        }),
                        rhs: Box::new(ResolvedExpr {
                            span: cc.span,
                            vt: VT::I32,
                            kind: RExprKind::IntLit(4),
                        }),
                    },
                },
                then: RBlock {
                    stmts: catch_wrapper,
                    span: cc.span,
                },
                els: None,
                span: cc.span,
            });
        }

        if let Some(fb) = finally {
            let saved_k = self.agg_temp("fk", VT::I32, false, fb.span);
            let saved_tl = self.agg_temp("ftl", VT::I32, false, fb.span);
            let saved_e = self.agg_temp("fe", VT::StrOwned, false, fb.span);
            let saved_rv = cs
                .ret_opt_vi
                .map(|vi| self.agg_temp("frv", VT::Variant(vi), false, fb.span));
            out.push(RStmt::Let {
                local: saved_k,
                value: ResolvedExpr {
                    span: fb.span,
                    vt: VT::I32,
                    kind: RExprKind::ReadLocal(cs.kind),
                },
                span: fb.span,
            });
            out.push(RStmt::Let {
                local: saved_tl,
                value: ResolvedExpr {
                    span: fb.span,
                    vt: VT::I32,
                    kind: RExprKind::ReadLocal(cs.target_loop),
                },
                span: fb.span,
            });
            out.push(RStmt::Let {
                local: saved_e,
                value: ResolvedExpr {
                    span: fb.span,
                    vt: VT::StrOwned,
                    kind: RExprKind::Move {
                        inner: Box::new(ResolvedExpr {
                            span: fb.span,
                            vt: VT::StrOwned,
                            kind: RExprKind::ReadLocal(cs.err),
                        }),
                    },
                },
                span: fb.span,
            });
            out.push(RStmt::Assign {
                local: cs.err,
                path: Vec::new(),
                value: ResolvedExpr {
                    span: fb.span,
                    vt: VT::StrOwned,
                    kind: RExprKind::StrLit(String::new()),
                },
                span: fb.span,
            });
            if let (Some(sr), Some(opt_vi)) = (saved_rv, cs.ret_opt_vi) {
                let rv_read = ResolvedExpr {
                    span: fb.span,
                    vt: VT::Variant(opt_vi),
                    kind: RExprKind::ReadLocal(cs.ret_val),
                };
                let rv_val = if self.vt_is_coll_owned(VT::Variant(opt_vi)) {
                    ResolvedExpr {
                        span: fb.span,
                        vt: VT::Variant(opt_vi),
                        kind: RExprKind::Move {
                            inner: Box::new(rv_read),
                        },
                    }
                } else {
                    rv_read
                };
                out.push(RStmt::Let {
                    local: sr,
                    value: rv_val,
                    span: fb.span,
                });
                out.push(RStmt::Assign {
                    local: cs.ret_val,
                    path: Vec::new(),
                    value: ResolvedExpr {
                        span: fb.span,
                        vt: VT::Variant(opt_vi),
                        kind: RExprKind::VariantCtor {
                            variant: opt_vi,
                            disc: 1,
                            payload: None,
                        },
                    },
                    span: fb.span,
                });
            }
            out.push(RStmt::Assign {
                local: cs.kind,
                path: Vec::new(),
                value: ResolvedExpr {
                    span: fb.span,
                    vt: VT::I32,
                    kind: RExprKind::IntLit(0),
                },
                span: fb.span,
            });
            let fin_loop = LoopRef(self.loop_count);
            self.loop_count += 1;
            if self.is_generator {
                self.gen_try_finals.push((try_loop, fin_loop));
            }
            self.ctrl_frames.push(CtrlFrame::Try {
                try_loop: fin_loop,
                has_catch: false,
            });
            let mut r_fin = self.resolve_body(fb, false);
            if !rstmts_diverge(&r_fin.stmts) {
                r_fin.stmts.push(RStmt::Break {
                    target: Some(fin_loop),
                    span: fb.span,
                });
            }
            self.ctrl_frames.pop();
            out.push(RStmt::Loop {
                body: r_fin,
                span: fb.span,
                target: fin_loop,
                unwind: (self.profile.is_auto_language() && !self.is_generator)
                    .then_some((cs.kind, cs.err)),
            });
            let mut restore_stmts = vec![
                RStmt::Assign {
                    local: cs.kind,
                    path: Vec::new(),
                    value: ResolvedExpr {
                        span: fb.span,
                        vt: VT::I32,
                        kind: RExprKind::ReadLocal(saved_k),
                    },
                    span: fb.span,
                },
                RStmt::Assign {
                    local: cs.target_loop,
                    path: Vec::new(),
                    value: ResolvedExpr {
                        span: fb.span,
                        vt: VT::I32,
                        kind: RExprKind::ReadLocal(saved_tl),
                    },
                    span: fb.span,
                },
                RStmt::Assign {
                    local: cs.err,
                    path: Vec::new(),
                    value: ResolvedExpr {
                        span: fb.span,
                        vt: VT::StrOwned,
                        kind: RExprKind::Move {
                            inner: Box::new(ResolvedExpr {
                                span: fb.span,
                                vt: VT::StrOwned,
                                kind: RExprKind::ReadLocal(saved_e),
                            }),
                        },
                    },
                    span: fb.span,
                },
            ];
            if let (Some(sr), Some(opt_vi)) = (saved_rv, cs.ret_opt_vi) {
                let sr_read = ResolvedExpr {
                    span: fb.span,
                    vt: VT::Variant(opt_vi),
                    kind: RExprKind::ReadLocal(sr),
                };
                let sr_val = if self.vt_is_coll_owned(VT::Variant(opt_vi)) {
                    ResolvedExpr {
                        span: fb.span,
                        vt: VT::Variant(opt_vi),
                        kind: RExprKind::Move {
                            inner: Box::new(sr_read),
                        },
                    }
                } else {
                    sr_read
                };
                restore_stmts.push(RStmt::Assign {
                    local: cs.ret_val,
                    path: Vec::new(),
                    value: sr_val,
                    span: fb.span,
                });
            }
            out.push(RStmt::If {
                cond: ResolvedExpr {
                    span: fb.span,
                    vt: VT::Bool,
                    kind: RExprKind::Binary {
                        op: BinOp::Eq,
                        lhs: Box::new(ResolvedExpr {
                            span: fb.span,
                            vt: VT::I32,
                            kind: RExprKind::ReadLocal(cs.kind),
                        }),
                        rhs: Box::new(ResolvedExpr {
                            span: fb.span,
                            vt: VT::I32,
                            kind: RExprKind::IntLit(0),
                        }),
                    },
                },
                then: RBlock {
                    stmts: restore_stmts,
                    span: fb.span,
                },
                els: Some(RBlock {
                    stmts: Vec::new(),
                    span: fb.span,
                }),
                span: fb.span,
            });
        }

        self.emit_post_frame_dispatch(span, &mut out);
        // PLAN-965 (F-965-R1-05): a try/catch whose BOTH arms diverge leaves no
        // reachable code after it, but the dispatch's structural fall-through
        // still needs a terminator. Inject the dummy only when this try really
        // ends the block — injecting it unconditionally made every following
        // statement an unreachable block (`verify.block-structure`).
        if self.ctrl_frames.is_empty() && try_diverges && catch_diverges && !has_following {
            let dummy = if self.fn_result == VT::Unit {
                ResolvedExpr {
                    span,
                    vt: VT::Unit,
                    kind: RExprKind::Unit,
                }
            } else {
                self.dummy_of(self.fn_result, span)
            };
            out.push(RStmt::Return { value: dummy, span });
        }
        out
    }

    /// PLAN-1094: `with <expr> [as <name>] { body }` — a Python context
    /// manager statement. The manager evaluates once; `__enter__` binds
    /// `name`; the body runs inside a protected loop (like try); every exit
    /// (normal / return / break / continue / language error) runs
    /// `__exit__` EXACTLY ONCE — normal exits with None args, language
    /// errors with the error text, and a truthy `__exit__` result SUPPRESSES
    /// the error (the Python protocol) so control continues after the with.
    fn resolve_with_stmt(
        &mut self,
        expr: &ast::Expr,
        bind: Option<&ast::Ident>,
        body: &ast::Block,
        span: Span,
        has_following: bool,
    ) -> Vec<RStmt> {
        let mut out = Vec::new();
        if !self.profile.is_aggregates() {
            self.bind_err(
                "source.unsupported",
                "`with` requires full aggregates lowering".to_string(),
                span,
            );
            return out;
        }
        let cs = self.ensure_ctrl_state(span);
        // 1) the manager: one evaluation, held in a py-token local.
        let cm_value = self.expr_iterative(expr);
        let manager_is_py = self.py_is_object(&cm_value);
        if !manager_is_py {
            self.bind_err(
                "source.with-manager",
                "`with` requires a Python context manager expression (a `use.py` object)"
                    .to_string(),
                expr.span,
            );
        }
        let cm = self.agg_temp("__with_cm", VT::I32, false, span);
        if manager_is_py {
            self.py_locals.insert(cm);
        }
        out.push(RStmt::Let {
            local: cm,
            value: cm_value,
            span,
        });
        // 2) enter: `__enter__` → the bound name (Python: name = enter()).
        if manager_is_py {
            let cm_read = ResolvedExpr {
                span,
                vt: VT::I32,
                kind: RExprKind::ReadLocal(cm),
            };
            let enter = self.py_call_sugar(&cm_read, "__enter__", Vec::new(), span);
            let enter_l = self.agg_temp("__with_enter", VT::I32, false, span);
            self.py_locals.insert(enter_l);
            out.push(RStmt::Let {
                local: enter_l,
                value: enter,
                span,
            });
            if let Some(b) = bind {
                let user = LocalRef(self.locals.len());
                self.locals.push(LocalDecl {
                    name: b.name.clone(),
                    vt: VT::I32,
                    mutable: false,
                    is_param: false,
                    param_index: None,
                    capture_index: None,
                    cell: false,
                    is_capture: false,
                    mode: ast::ParamMode::Default,
                    span: b.span,
                });
                self.py_locals.insert(user);
                self.scopes.push(Scope {
                    names: BTreeMap::from([(b.name.clone(), user)]),
                    views: BTreeMap::new(),
                    text_mut_aliases: BTreeMap::new(),
                });
                out.push(RStmt::Let {
                    local: user,
                    value: ResolvedExpr {
                        span: b.span,
                        vt: VT::I32,
                        kind: RExprKind::ReadLocal(enter_l),
                    },
                    span: b.span,
                });
            }
        }
        // 3) the protected body loop (same shape as try).
        let with_loop = LoopRef(self.loop_count);
        self.loop_count += 1;
        self.ctrl_frames.push(CtrlFrame::Try {
            try_loop: with_loop,
            has_catch: false,
        });
        fn rstmts_diverge(stmts: &[RStmt]) -> bool {
            let Some(last) = stmts.last() else {
                return false;
            };
            match last {
                RStmt::Return { .. }
                | RStmt::Fail { .. }
                | RStmt::Break { .. }
                | RStmt::Continue { .. } => true,
                RStmt::If {
                    then,
                    els: Some(els),
                    ..
                } => rstmts_diverge(&then.stmts) && rstmts_diverge(&els.stmts),
                _ => false,
            }
        }
        let mut r_body = self.resolve_body(body, false);
        if !rstmts_diverge(&r_body.stmts) {
            r_body.stmts.push(RStmt::Break {
                target: Some(with_loop),
                span,
            });
        }
        self.ctrl_frames.pop();
        out.push(RStmt::Loop {
            body: r_body,
            span,
            target: with_loop,
            unwind: (self.profile.is_auto_language() && !self.is_generator)
                .then_some((cs.kind, cs.err)),
        });

        // 4) the exit handler (the loop's normal exit AND the unwind edge
        // both land here — the runtime set cs.kind=4 + cs.err on unwind):
        // save the ctrl state, run __exit__ once per the saved kind,
        // restore, then dispatch.
        if manager_is_py {
            let saved_k = self.agg_temp("wk", VT::I32, false, span);
            let saved_tl = self.agg_temp("wtl", VT::I32, false, span);
            let saved_e = self.agg_temp("we", VT::StrOwned, false, span);
            let read = |local: LocalRef, vt: VT, sp: Span| ResolvedExpr {
                span: sp,
                vt,
                kind: RExprKind::ReadLocal(local),
            };
            out.push(RStmt::Let {
                local: saved_k,
                value: read(cs.kind, VT::I32, span),
                span,
            });
            out.push(RStmt::Let {
                local: saved_tl,
                value: read(cs.target_loop, VT::I32, span),
                span,
            });
            out.push(RStmt::Let {
                local: saved_e,
                value: ResolvedExpr {
                    span,
                    vt: VT::StrOwned,
                    kind: RExprKind::Move {
                        inner: Box::new(read(cs.err, VT::StrOwned, span)),
                    },
                },
                span,
            });
            out.push(RStmt::Assign {
                local: cs.err,
                path: Vec::new(),
                value: ResolvedExpr {
                    span,
                    vt: VT::StrOwned,
                    kind: RExprKind::StrLit(String::new()),
                },
                span,
            });
            // __exit__ per the saved kind: 4 = language error (error text
            // args + suppression check), everything else = None args.
            let cm_read = read(cm, VT::I32, span);
            let nil = ResolvedArg {
                value: ResolvedExpr {
                    span,
                    vt: VT::Nil,
                    kind: RExprKind::NilLit,
                },
                name: None,
                moved: false,
                span,
            };
            let normal_exit = self.py_call_sugar(
                &cm_read,
                "__exit__",
                vec![
                    ResolvedArg {
                        value: ResolvedExpr {
                            span,
                            vt: VT::Nil,
                            kind: RExprKind::NilLit,
                        },
                        name: None,
                        moved: false,
                        span,
                    },
                    nil.clone(),
                    nil,
                ],
                span,
            );
            let err_exit = self.py_call_sugar(
                &cm_read,
                "__exit__",
                vec![ResolvedArg {
                    // a CLONE of the text — the original stays for the
                    // restore/re-raise (the frame arg must not move it).
                    value: Self::clone_owned_text(read(saved_e, VT::StrOwned, span)),
                    name: None,
                    moved: false,
                    span,
                }],
                span,
            );
            let supb = self.agg_temp("__with_sup", VT::Bool, false, span);
            let sup_frame = self.py_recv_frame(&err_exit, span);
            let sup_raw = self.py_op_call("py_to_bool", span, sup_frame);
            out.push(RStmt::Let {
                local: supb,
                value: ResolvedExpr {
                    span,
                    vt: VT::Bool,
                    kind: RExprKind::Binary {
                        op: BinOp::Ne,
                        lhs: Box::new(sup_raw),
                        rhs: Box::new(ResolvedExpr {
                            span,
                            vt: VT::I32,
                            kind: RExprKind::IntLit(0),
                        }),
                    },
                },
                span,
            });
            let suppressed = ResolvedExpr {
                span,
                vt: VT::Bool,
                kind: RExprKind::ReadLocal(supb),
            };
            let is_error = ResolvedExpr {
                span,
                vt: VT::Bool,
                kind: RExprKind::Binary {
                    op: BinOp::Eq,
                    lhs: Box::new(read(saved_k, VT::I32, span)),
                    rhs: Box::new(ResolvedExpr {
                        span,
                        vt: VT::I32,
                        kind: RExprKind::IntLit(4),
                    }),
                },
            };
            // error path: run __exit__(err); if it returns truthy, the
            // error is suppressed (kind→0 + the suppressed flag) so the
            // dispatch continues after the with.
            let suppressed_flag = self.agg_temp("__with_suppressed", VT::Bool, true, span);
            out.push(RStmt::Let {
                local: suppressed_flag,
                value: ResolvedExpr {
                    span,
                    vt: VT::Bool,
                    kind: RExprKind::BoolLit(false),
                },
                span,
            });
            let mut err_then = vec![RStmt::Let {
                local: self.agg_temp("__with_exit_err", VT::I32, false, span),
                value: err_exit,
                span,
            }];
            err_then.push(RStmt::If {
                cond: suppressed,
                then: RBlock {
                    stmts: vec![
                        RStmt::Assign {
                            local: cs.kind,
                            path: Vec::new(),
                            value: ResolvedExpr {
                                span,
                                vt: VT::I32,
                                kind: RExprKind::IntLit(0),
                            },
                            span,
                        },
                        RStmt::Assign {
                            local: cs.err,
                            path: Vec::new(),
                            value: ResolvedExpr {
                                span,
                                vt: VT::StrOwned,
                                kind: RExprKind::StrLit(String::new()),
                            },
                            span,
                        },
                        RStmt::Assign {
                            local: suppressed_flag,
                            path: Vec::new(),
                            value: ResolvedExpr {
                                span,
                                vt: VT::Bool,
                                kind: RExprKind::BoolLit(true),
                            },
                            span,
                        },
                    ],
                    span,
                },
                els: None,
                span,
            });
            out.push(RStmt::If {
                cond: is_error,
                then: RBlock {
                    stmts: err_then,
                    span,
                },
                els: Some(RBlock {
                    stmts: vec![RStmt::Let {
                        local: self.agg_temp("__with_exit_normal", VT::I32, false, span),
                        value: normal_exit,
                        span,
                    }],
                    span,
                }),
                span,
            });
            // restore the saved state for the dispatch (the with's own
            // unwind wrote cs.kind/cs.err; the outer context's state must
            // survive). A SUPPRESSED error keeps kind 0 + empty err (the
            // suppressed flag already zeroed them) — the restore of those
            // two is gated on it.
            let not_suppressed = ResolvedExpr {
                span,
                vt: VT::Bool,
                kind: RExprKind::Unary {
                    op: UnaryOp::Not,
                    operand: Box::new(read(suppressed_flag, VT::Bool, span)),
                },
            };
            out.push(RStmt::If {
                cond: not_suppressed,
                then: RBlock {
                    stmts: vec![
                        RStmt::Assign {
                            local: cs.kind,
                            path: Vec::new(),
                            value: read(saved_k, VT::I32, span),
                            span,
                        },
                        RStmt::Assign {
                            local: cs.err,
                            path: Vec::new(),
                            value: ResolvedExpr {
                                span,
                                vt: VT::StrOwned,
                                kind: RExprKind::Move {
                                    inner: Box::new(read(saved_e, VT::StrOwned, span)),
                                },
                            },
                            span,
                        },
                    ],
                    span,
                },
                els: None,
                span,
            });
            out.push(RStmt::Assign {
                local: cs.target_loop,
                path: Vec::new(),
                value: read(saved_tl, VT::I32, span),
                span,
            });
        }
        self.emit_post_frame_dispatch(span, &mut out);
        if self.ctrl_frames.is_empty() && !has_following {
            let dummy = if self.fn_result == VT::Unit {
                ResolvedExpr {
                    span,
                    vt: VT::Unit,
                    kind: RExprKind::Unit,
                }
            } else {
                self.dummy_of(self.fn_result, span)
            };
            out.push(RStmt::Return { value: dummy, span });
        }
        out
    }

    fn resolve_iter_for_in(
        &mut self,
        var: &ast::Ident,
        iter_val: ResolvedExpr,
        iter_rec_idx: usize,
        body: &ast::Block,
        span: Span,
    ) -> Vec<RStmt> {
        let mut out = Vec::new();
        let (elem_vt, _opt_vt, next_fn_index, close_fn_index, _) = self.iter_info[&iter_rec_idx];
        let iter_vt = VT::Rec(iter_rec_idx);
        let opt_vi = self.coll.intern_variant(
            "Option",
            span,
            vec![
                ("Some".to_string(), Some(elem_vt)),
                ("None".to_string(), None),
            ],
        );
        let opt_vt = VT::Variant(opt_vi);
        let it_local = match iter_val.kind {
            RExprKind::ReadLocal(existing) if self.locals[existing.0].mutable => existing,
            _ => {
                let fresh = self.agg_temp("for_it", iter_vt, true, span);
                out.push(RStmt::Let {
                    local: fresh,
                    value: iter_val,
                    span,
                });
                fresh
            }
        };
        let target = LoopRef(self.loop_count);
        self.loop_count += 1;
        self.loop_stack.push(target);
        self.ctrl_frames.push(CtrlFrame::ForIter {
            target,
            iter_local: it_local,
            close_fn_index,
        });
        let step_l = self.agg_temp("it_step", opt_vt, false, span);
        self.scopes.push(Scope {
            names: BTreeMap::new(),
            views: BTreeMap::new(),
            text_mut_aliases: BTreeMap::new(),
        });
        let elem_l = LocalRef(self.locals.len());
        self.locals.push(LocalDecl {
            name: var.name.clone(),
            vt: elem_vt,
            mutable: false,
            is_param: false,
            param_index: None,
            capture_index: None,
            cell: false,
            is_capture: false,
            mode: ast::ParamMode::Default,
            span: var.span,
        });
        self.scopes
            .last_mut()
            .unwrap()
            .names
            .insert(var.name.clone(), elem_l);
        self.declared_names.insert(var.name.clone());
        let mut b = self.resolve_body(body, false);
        self.scopes.pop();
        self.ctrl_frames.pop();
        self.loop_stack.pop();

        let mut then_stmts = vec![RStmt::Let {
            local: elem_l,
            value: ResolvedExpr {
                span: var.span,
                vt: elem_vt,
                kind: RExprKind::VariantProject {
                    local: step_l,
                    variant: opt_vi,
                    disc: 0,
                },
            },
            span: var.span,
        }];
        then_stmts.append(&mut b.stmts);

        let loop_body = RBlock {
            stmts: vec![
                RStmt::Let {
                    local: step_l,
                    value: ResolvedExpr {
                        span,
                        vt: opt_vt,
                        kind: RExprKind::CallFn {
                            fn_index: next_fn_index,
                            args: vec![ResolvedArg {
                                span,
                                name: None,
                                moved: false,
                                value: ResolvedExpr {
                                    span,
                                    vt: iter_vt,
                                    kind: RExprKind::BorrowMut { local: it_local },
                                },
                            }],
                        },
                    },
                    span,
                },
                RStmt::If {
                    cond: ResolvedExpr {
                        span,
                        vt: VT::Bool,
                        kind: RExprKind::VariantGuardTest {
                            local: step_l,
                            variant: opt_vi,
                            disc: 0,
                        },
                    },
                    then: RBlock {
                        stmts: then_stmts,
                        span: body.span,
                    },
                    els: Some(RBlock {
                        stmts: vec![RStmt::Break {
                            target: Some(target),
                            span,
                        }],
                        span,
                    }),
                    span,
                },
            ],
            span: body.span,
        };
        let unwind =
            if self.profile.is_auto_language() && !self.is_generator && self.has_enclosing_try() {
                // Route failures through the iterator's normal close and
                // post-frame exit dispatch before entering catch/finally.
                let control = self.ensure_ctrl_state(span);
                Some((control.kind, control.err))
            } else {
                None
            };
        out.push(RStmt::Loop {
            body: loop_body,
            span,
            target,
            unwind,
        });
        out.extend(self.close_for_iter_stmts(close_fn_index, it_local, span));
        if self.has_outer_ctrl_barrier() {
            self.emit_post_frame_dispatch(span, &mut out);
        }
        out
    }

    fn resolve_iter_method(
        &mut self,
        rrecv: ResolvedExpr,
        ri: usize,
        method_name: &str,
        arg_count: usize,
        span: Span,
    ) -> Option<ResolvedExpr> {
        let &(_elem_vt, opt_vt, next_fn_index, close_fn_index, _) = self.iter_info.get(&ri)?;
        if arg_count != 0 {
            self.bind_err(
                "source.argument",
                format!("iterator method `{method_name}` takes no arguments"),
                span,
            );
        }
        let (fn_index, ret_vt) = match method_name {
            "next" => (next_fn_index, opt_vt),
            "close" => (close_fn_index, VT::I32),
            _ => return None,
        };
        let it_vt = VT::Rec(ri);
        let it_local = match rrecv.kind {
            RExprKind::ReadLocal(l) if self.locals[l.0].mutable => l,
            _ => {
                let tmp = self.agg_temp("it_recv", it_vt, true, span);
                self.emit_stmt(RStmt::Let {
                    local: tmp,
                    value: rrecv,
                    span,
                });
                tmp
            }
        };
        let it_local = if fn_index == close_fn_index {
            let mut stmts = Vec::new();
            let local = self.prepare_iterator_call(it_local, span, &mut stmts);
            for stmt in stmts {
                self.emit_stmt(stmt);
            }
            local
        } else {
            it_local
        };
        Some(ResolvedExpr {
            span,
            vt: ret_vt,
            kind: RExprKind::CallFn {
                fn_index,
                args: vec![ResolvedArg {
                    span,
                    name: None,
                    moved: false,
                    value: ResolvedExpr {
                        span,
                        vt: it_vt,
                        kind: RExprKind::BorrowMut { local: it_local },
                    },
                }],
            },
        })
    }

    /// PLAN-864: resolve a projection spine (outermost-first segment
    /// names) through the record table starting at `start_vt`; returns
    /// (declaration-index path, terminal type). Errors on non-record
    /// segments and unknown fields.
    fn resolve_projection_path(
        &mut self,
        segs: &[&ast::Ident],
        start_vt: VT,
    ) -> Option<(Vec<(usize, usize)>, VT)> {
        let mut path: Vec<(usize, usize)> = Vec::new();
        let mut vt = start_vt;
        for seg in segs.iter().rev() {
            let VT::Rec(ri) = vt else {
                self.bind_err(
                    "source.view-escape",
                    format!(
                        "`{}` projects through records only (element type `{}`)",
                        seg.name,
                        vt.as_str()
                    ),
                    seg.span,
                );
                return None;
            };
            match self.records[ri]
                .fields
                .iter()
                .position(|fl| fl.name == seg.name)
            {
                Some(fi) => {
                    vt = self.records[ri].fields[fi].vt;
                    path.push((ri, fi));
                }
                None => {
                    self.bind_err(
                        "source.undefined",
                        format!(
                            "record `{}` has no field named `{}`",
                            self.records[ri].name, seg.name
                        ),
                        seg.span,
                    );
                    return None;
                }
            }
        }
        Some((path, vt))
    }

    /// PLAN-864: whether `name` is shadowed by a local or a user function
    /// (bare constructor names never hijack user names).
    fn ctor_name_is_free(&self, name: &str) -> bool {
        self.lookup(name).is_none() && !self.sigs.contains_key(name)
    }

    /// PLAN-962: a whole-element VIEW (ElemView with an empty projection
    /// path) — a borrow of an owned element read out of its list. Legal as
    /// a method receiver (read-through); every materializing use (let
    /// binding, argument, return, store) is a view escape.
    fn is_whole_elem_view(e: &ResolvedExpr) -> bool {
        matches!(&e.kind, RExprKind::ElemView { path, .. } if path.is_empty())
    }

    fn clone_text_element(value: ResolvedExpr) -> ResolvedExpr {
        let view = match &value.kind {
            RExprKind::ElemView { .. } | RExprKind::ListFieldGet { .. } => true,
            RExprKind::FieldGet { base, .. } => Self::is_whole_elem_view(base),
            _ => false,
        };
        if value.vt == VT::StrOwned && view {
            return Self::clone_owned_text(value);
        }
        value
    }

    fn clone_owned_text(value: ResolvedExpr) -> ResolvedExpr {
        ResolvedExpr {
            span: value.span,
            vt: VT::StrOwned,
            kind: RExprKind::Method {
                recv: Box::new(value),
                method: BuiltinMethod::Clone,
                to: None,
                args: Vec::new(),
            },
        }
    }

    fn copy_legacy_text_local(value: ResolvedExpr, expected: Option<VT>) -> ResolvedExpr {
        if value.vt == VT::StrOwned
            && matches!(value.kind, RExprKind::ReadLocal(_) | RExprKind::FieldGet { .. })
        {
            return Self::clone_owned_text(value);
        }
        if expected == Some(VT::StrOwned) && value.vt == VT::Str {
            return Self::own_dynamic_text(value);
        }
        Self::clone_text_element(value)
    }

    fn reject_elem_view_use(&mut self, e: &ResolvedExpr, what: &str) {
        if Self::is_whole_elem_view(e) {
            self.bind_err(
                "source.view-escape",
                format!(
                    "a whole owned element is a view, not a value; read through it (`{what}.get(i)`, field projection) or move it out (`remove`)"
                ),
                e.span,
            );
        }
    }

    fn vt_is_coll_owned(&self, vt: VT) -> bool {
        match vt {
            VT::StrOwned | VT::List(_) | VT::Variant(_) | VT::Function(_) => true,
            // PLAN-1035 (F34): a record owns iff its field closure reaches a
            // resource leaf — Str OR List OR Variant OR closure env OR a
            // tuple aggregate carrying one (the Str-only walk was the hole).
            VT::Rec(ri) => record_reaches_resource(ri, self.records, &self.coll.tuples),
            // PLAN-962: a tuple owns iff any element owns.
            VT::Tuple(ti) => self.coll.tuples[ti]
                .iter()
                .any(|&v| self.vt_is_coll_owned(v)),
            _ => false,
        }
    }

    fn fresh_local(&mut self, name: &str, vt: VT, mutable: bool) -> LocalRef {
        let lref = LocalRef(self.locals.len());
        self.locals.push(LocalDecl {
            name: name.to_string(),
            vt,
            mutable,
            is_param: false,
            param_index: None,
            capture_index: None,
            cell: false,
            is_capture: false,
            mode: ast::ParamMode::Default,
            span: Span { start: 0, end: 0 },
        });
        lref
    }

    /// PLAN-864: classify a `List<T>`-style generic receiver. Only
    /// `List<one elem>` with a legal element type commits;
    /// everything else is a named rejection.
    fn classify_generic_recv(&mut self, name: &ast::Ident, args: &[ast::TypeAnn]) -> RecvClass<'static> {
        if !self.profile.is_collections() {
            self.bind_err(
                "source.unsupported",
                "parameterized type receivers are not part of this source profile".to_string(),
                name.span,
            );
            return RecvClass::Value;
        }
        if name.name != "List" || args.len() != 1 {
            self.bind_err(
                "source.undefined",
                format!(
                    "no parameterized type named `{}` taking {} type argument(s) in this module",
                    name.name,
                    args.len()
                ),
                name.span,
            );
            return RecvClass::Value;
        }
        let elem = vt_of_ann_coll(
            &args[0],
            self.rec_index,
            self.coll,
            self.diags,
            TypePos::Local,
        );
        let elem = if self.coll.ordinary_auto && elem == VT::Str { VT::StrOwned } else { elem };
        match elem {
            // PLAN-962: the aggregates admission widens the receiver
            // element domain too (nested List constructors are legal).
            VT::List(_) if self.coll.aggregates_admission => {
                RecvClass::ListCtor(self.coll.intern_list(elem))
            }
            VT::List(_) => {
                self.bind_err(
                    "source.list-value-context",
                    "List does not nest List (822 element domain)".to_string(),
                    args[0].span,
                );
                RecvClass::Value
            }
            VT::Char | VT::I8 | VT::U8 | VT::U32 | VT::I64 | VT::U64 | VT::F32 | VT::F64
                if self.coll.aggregates_admission =>
            {
                RecvClass::ListCtor(self.coll.intern_list(elem))
            }
            VT::I32 | VT::Bool | VT::StrOwned | VT::Rec(_) | VT::Variant(_) => {
                RecvClass::ListCtor(self.coll.intern_list(elem))
            }
            other => {
                self.bind_err(
                    "source.list-value-context",
                    format!(
                        "element type `{}` is not in the 822 element domain",
                        other.as_str()
                    ),
                    args[0].span,
                );
                RecvClass::Value
            }
        }
    }

    /// PLAN-864: resolve `EnumName.arm_name` to (variant, disc); `None`
    /// when the receiver name is not a declared variant.
    fn qualified_variant_arm(&self, enum_name: &str, arm_name: &str) -> Option<(usize, u32)> {
        let vi = self.coll.variant_by_name(enum_name)?;
        let arms = &self.coll.variants[vi].arms;
        let disc = arms.iter().position(|(n, _)| n == arm_name)?;
        Some((vi, disc as u32))
    }

    /// PLAN-864: a qualified ctor takes exactly the arm's payload arity.
    fn reject_unbound_variant_arm(&mut self, vi: usize, disc: u32, span: Span) -> bool {
        let label = &self.coll.variants[vi].arms[disc as usize].0;
        if self.coll.enum_instances.get(&vi)
            .is_some_and(|instance| instance.unbound_arms.contains(label))
        {
            self.bind_err(
                "unsolved-variable",
                format!("constructing `{label}` requires the complete enum type arguments"),
                span,
            );
            return true;
        }
        false
    }

    fn check_qualified_ctor_args(&mut self, vi: usize, disc: u32, args: &[ast::Arg], span: Span) {
        if self.reject_unbound_variant_arm(vi, disc, span) {
            return;
        }
        // PLAN-1083: tuple/record payloads take one value per element/
        // field (positional or named); a single-value payload keeps the
        // PLAN-864 one-argument contract.
        let payload_vt = self.coll.variants[vi].arms[disc as usize].1;
        let n = args.len();
        let named_arm = self.coll.named_field_arms.contains(&(vi, disc));
        let want = match payload_vt {
            Some(VT::Tuple(ti)) => self.coll.tuples.get(ti).map(|t| t.len()),
            // a NAMED-FIELD payload constructs field-by-field; an anonymous
            // record-typed payload (`Msg.Move(Pt)`) is ONE whole value.
            Some(VT::Rec(_)) if named_arm => {
                payload_vt.and_then(|vt| match vt {
                    VT::Rec(ri) => self.records.get(ri).map(|r| r.fields.len()),
                    _ => None,
                })
            }
            _ => None,
        };
        match (payload_vt, want, n) {
            (None, _, 0) => {}
            (None, _, _) => self.bind_err(
                "source.argument",
                "this variant arm carries no payload".to_string(),
                span,
            ),
            (Some(_), None, 1) => {}
            (Some(_), None, _) => self.bind_err(
                "source.argument",
                "this variant arm takes exactly one payload value".to_string(),
                span,
            ),
            (Some(_), Some(w), _) if n == w => {}
            (Some(_), Some(w), _) => self.bind_err(
                "source.argument",
                format!(
                    "this variant arm takes {w} payload value(s) (positional or named), found {n}"
                ),
                span,
            ),
        }
    }

    /// PLAN-864: resolve a bare constructor (`Some/None/Ok/Err`) against
    /// the expected type. `Some` self-infers from its payload; `None`/
    /// `Ok`/`Err` require the expected variant (`source.type-ambiguous`
    /// otherwise).
    fn resolve_bare_ctor(
        &mut self,
        cname: &str,
        payload: Option<ResolvedExpr>,
        expect: Option<VT>,
        span: Span,
    ) -> ResolvedExpr {
        match (cname, payload, expect) {
            ("Some", Some(mut pl), expect) => {
                if let Some(VT::Variant(vi)) = expect {
                    if self.coll.variants[vi].name == "Option" {
                        if let Some(Some(pvt)) = self.coll.variants[vi].arms.get(0).map(|a| &a.1) {
                            retag_literal(&mut pl, *pvt);
                        }
                    }
                }
                let t = pl.vt;
                let vi = match expect {
                    Some(VT::Variant(vi)) if self.coll.variants[vi].name == "Option" => vi,
                    _ => {
                        if matches!(t, VT::List(_)) {
                            self.bind_err(
                                "source.list-value-context",
                                "List-typed payloads are not part of the 822 element domain"
                                    .to_string(),
                                span,
                            );
                        }
                        self.coll.intern_variant(
                            "Option",
                            span,
                            vec![("Some".to_string(), Some(t)), ("None".to_string(), None)],
                        )
                    }
                };
                ResolvedExpr {
                    span,
                    vt: VT::Variant(vi),
                    kind: RExprKind::VariantCtor {
                        variant: vi,
                        disc: 0,
                        payload: Some(Box::new(pl)),
                    },
                }
            }
            ("Some", None, _) => {
                self.bind_err(
                    "source.argument",
                    "`Some` takes exactly one payload".to_string(),
                    span,
                );
                ResolvedExpr {
                    span,
                    vt: VT::I32,
                    kind: RExprKind::IntLit(0),
                }
            }
            ("None", None, Some(VT::Variant(vi))) if self.coll.variants[vi].name == "Option" => {
                ResolvedExpr {
                    span,
                    vt: VT::Variant(vi),
                    kind: RExprKind::VariantCtor {
                        variant: vi,
                        disc: 1,
                        payload: None,
                    },
                }
            }
            ("Ok" | "Err", Some(mut pl), Some(VT::Variant(vi)))
                if self.coll.variants[vi].name == "Result" =>
            {
                let disc = if cname == "Ok" { 0 } else { 1 };
                if let Some(Some(pvt)) =
                    self.coll.variants[vi].arms.get(disc as usize).map(|a| &a.1)
                {
                    retag_literal(&mut pl, *pvt);
                }
                if matches!(pl.vt, VT::List(_)) {
                    self.bind_err(
                        "source.list-value-context",
                        "List-typed payloads are not part of the 822 element domain".to_string(),
                        span,
                    );
                }
                ResolvedExpr {
                    span,
                    vt: VT::Variant(vi),
                    kind: RExprKind::VariantCtor {
                        variant: vi,
                        disc,
                        payload: Some(Box::new(pl)),
                    },
                }
            }
            ("None", None, None) if self.profile.is_aggregates() => {
                let vi = self.coll.intern_variant(
                    "Option",
                    span,
                    vec![
                        ("Some".to_string(), Some(VT::I32)),
                        ("None".to_string(), None),
                    ],
                );
                ResolvedExpr {
                    span,
                    vt: VT::Variant(vi),
                    kind: RExprKind::VariantCtor {
                        variant: vi,
                        disc: 1,
                        payload: None,
                    },
                }
            }
            ("Ok", Some(pl), None) if self.profile.is_aggregates() => {
                let ok_vt = pl.vt;
                let vi = self.coll.intern_variant(
                    "Result",
                    span,
                    vec![
                        ("Ok".to_string(), Some(ok_vt)),
                        ("Err".to_string(), Some(VT::StrOwned)),
                    ],
                );
                ResolvedExpr {
                    span,
                    vt: VT::Variant(vi),
                    kind: RExprKind::VariantCtor {
                        variant: vi,
                        disc: 0,
                        payload: Some(Box::new(pl)),
                    },
                }
            }
            ("Err", Some(mut pl), None) if self.profile.is_aggregates() => {
                if pl.vt == VT::Str && matches!(pl.kind, RExprKind::StrLit(_)) {
                    pl.vt = VT::StrOwned;
                }
                let err_vt = pl.vt;
                let vi = self.coll.intern_variant(
                    "Result",
                    span,
                    vec![
                        ("Ok".to_string(), Some(VT::I32)),
                        ("Err".to_string(), Some(err_vt)),
                    ],
                );
                ResolvedExpr {
                    span,
                    vt: VT::Variant(vi),
                    kind: RExprKind::VariantCtor {
                        variant: vi,
                        disc: 1,
                        payload: Some(Box::new(pl)),
                    },
                }
            }
            (other, _, _) => {
                self.bind_err(
                    "source.type-ambiguous",
                    format!(
                        "constructor `{}` needs an expected type (annotate the local, or return it from a typed function)",
                        other
                    ),
                    span,
                );
                ResolvedExpr {
                    span,
                    vt: VT::I32,
                    kind: RExprKind::IntLit(0),
                }
            }
        }
    }

    /// PLAN-864: resolve an expression whose ROOT may be a bare
    /// constructor, with the context's expected type. Everything else
    /// resolves bottom-up unchanged.
    fn expr_with_expect(&mut self, v: &ast::Expr, expect: Option<VT>) -> ResolvedExpr {
        self.expr_with_context(v, expect, true)
    }

    fn expr_with_context(
        &mut self,
        v: &ast::Expr,
        expect: Option<VT>,
        recover_dynamic: bool,
    ) -> ResolvedExpr {
        self.expr_diverged = false;
        if let Some(se) = self.try_resolve_scalar_enum(v, expect) {
            return se;
        }
        if let Some(va) = self.try_resolve_variant_arm(v, expect) {
            return va;
        }
        if let Some(may) = self.try_resolve_may_ctor(v, expect) {
            return may;
        }
        // PLAN-1082 (T-03): the legacy List.new varargs sugar rides the
        // same bare-ctor interception shape as May.
        if let Some(list) = self.try_resolve_legacy_list_new(v, expect) {
            return list;
        }
        if self.profile.is_aggregates() {
            match &v.kind {
                ExprKind::Binary {
                    op: BinOp::NullCoalesce,
                    lhs,
                    rhs,
                } => return self.resolve_unwrap_or(lhs, rhs, expect, v.span),
                ExprKind::UnwrapOr { inner, fallback } => {
                    return self.resolve_unwrap_or(inner, fallback, expect, v.span);
                }
                ExprKind::Propagate { inner } => {
                    return self.resolve_propagate(inner, v.span);
                }
                ExprKind::IsExpr { scrutinee, arms } => {
                    let (stmts, expr_opt) =
                        self.resolve_agg_is(scrutinee, arms, expect, true, v.span, false);
                    for s in stmts {
                        self.emit_stmt(s);
                    }
                    return expr_opt.unwrap_or(ResolvedExpr {
                        span: v.span,
                        vt: VT::Unit,
                        kind: RExprKind::Unit,
                    });
                }
                _ => {}
            }
        }
        let ctor: Option<(&str, Option<&ast::Expr>)> = match &v.kind {
            ExprKind::Var(id) if id.name == "None" && self.ctor_name_is_free("None") => {
                Some(("None", None))
            }
            ExprKind::Call { callee, args }
                if args.len() == 1
                    && matches!(callee.name.as_str(), "Some" | "Ok" | "Err")
                    && matches!(&args[0], ast::Arg::Pos(_) | ast::Arg::MovePos(_))
                    && self.ctor_name_is_free(&callee.name) =>
            {
                match &args[0] {
                    ast::Arg::Pos(e) | ast::Arg::MovePos(e) => {
                        Some((callee.name.as_str(), Some(e)))
                    }
                    _ => None,
                }
            }
            _ => None,
        };
        match ctor {
            Some((cname, payload_ast)) => {
                let payload = payload_ast.map(|p| self.expr_iterative(p));
                self.resolve_bare_ctor(cname, payload, expect, v.span)
            }
            None => {
                let mut value = self.expr_iterative_with_hint(v, expect);
                if let Some(expected) = expect {
                    self.adapt_spec_value(&mut value, expected);
                }
                match expect {
                    Some(VT::F32)
                        if value.vt == VT::I32 && self.coll.legacy_age_widget_at(v.span) =>
                    {
                        self.coll.age_widget_numeric_cast(value, VT::F32)
                    }
                    Some(expected)
                        if recover_dynamic
                            && self.profile == SourceProfile::Auto
                            && expected != VT::Unit =>
                    {
                        Self::coerce_dynamic(value, expected)
                    }
                    _ => value,
                }
            }
        }
    }

    /// PLAN-864: an `is` arm pattern against the scrutinee's variant —
    /// bare `Arm` or qualified `Enum.Arm`; returns the discriminant.
    fn is_arm_disc(
        &mut self,
        scrutinee_vt: VT,
        qual: Option<&ast::Ident>,
        pat: &ast::Ident,
    ) -> Option<(usize, u32)> {
        let VT::Variant(vi) = scrutinee_vt else {
            return None;
        };
        if let Some(q) = qual {
            let source_family = self.coll.enum_instances.get(&vi).map(|instance| instance.template.as_str());
            if self.coll.variants[vi].name != q.name && source_family != Some(q.name.as_str()) {
                self.bind_err(
                    "source.pattern-unknown-arm",
                    format!(
                        "arm `{}.{}` does not belong to the scrutinee's variant `{}`",
                        q.name, pat.name, self.coll.variants[vi].name
                    ),
                    q.span.join(pat.span),
                );
                return None;
            }
        }
        match self.coll.variants[vi]
            .arms
            .iter()
            .position(|(n, _)| *n == pat.name)
        {
            Some(d) => Some((vi, d as u32)),
            None => {
                self.bind_err(
                    "source.pattern-unknown-arm",
                    format!(
                        "variant `{}` has no arm named `{}`",
                        self.coll.variants[vi].name, pat.name
                    ),
                    pat.span,
                );
                None
            }
        }
    }

    /// PLAN-864: a method call on a List-typed receiver (contract C3).
    /// PLAN-105 T-02（U03 真语料 013）：AGE 域允许**元素视图**（`xs[i].f` /
    /// `xs.get(i).f`）——verify 层已有 ElemView 的字段投影/`len()` 消费规则，
    /// bind 层不再一刀切拒；聚合域（aggregates_admission）保持原语义。
    fn elem_view_admission(&self) -> bool {
        // AGE 组件面（第二趟 collections 编译）由适配器打开该开关；其余域保持
        // 原语义（closures/text_record 的冻结拒绝不受影响）。
        self.coll.aggregates_admission
            || self.coll.elem_view_admission
            || matches!(
                self.profile,
                SourceProfile::AgeComponent | SourceProfile::AgePackage | SourceProfile::AgeH0
            )
    }
    fn resolve_list_method(
        &mut self,
        li: usize,
        name: &ast::Ident,
        recv: ResolvedExpr,
        mut rargs: Vec<ResolvedArg>,
        span: Span,
    ) -> ResolvedExpr {
        let elem = self.coll.lists[li];
        let record_pop =
            self.profile.is_auto_language() && name.name == "pop" && matches!(elem, VT::Rec(_));
        // Structural modification of an iterated list is a compile reject.
        if record_pop || matches!(name.name.as_str(), "push" | "insert" | "remove" | "clear") {
            if let RExprKind::ReadLocal(l) = recv.kind {
                let hit: Option<Span> = self
                    .iterate_targets
                    .iter()
                    .find(|(target, _, _)| *target == l)
                    .map(|(_, _, sp)| *sp);
                if let Some(sp) = hit {
                    self.bind_err(
                        "source.iterate-mutate",
                        format!(
                            "`{}` structurally modifies the list being iterated (legal: `set` and field writes)",
                            name.name
                        ),
                        name.span.join(sp),
                    );
                }
            }
        }
        let list = Box::new(recv);
        let mk = |vt: VT, kind: RExprKind| ResolvedExpr { span, vt, kind };
        match name.name.as_str() {
            "push" if rargs.len() == 1 => {
                let arg = rargs.remove(0);
                let mut value = self.move_marked_value(arg);
                // A legacy text storage list copies the source string at
                // this store. The explicit List<Str> path retains its move rule.
                if elem == VT::StrOwned
                    && matches!(list.kind, RExprKind::ReadLocal(local) if self.legacy_text_lists.contains(&local))
                {
                    value = match value.vt {
                        VT::Str => Self::own_dynamic_text(value),
                        VT::StrOwned if matches!(value.kind, RExprKind::ReadLocal(_)) => {
                            Self::clone_owned_text(value)
                        }
                        _ => Self::clone_text_element(value),
                    };
                }
                // AGE list iteration borrows owned rows. Appending that row to
                // another list constructs a fresh value and keeps the source intact.
                if self.coll.age_package && value.vt == elem && Self::is_whole_elem_view(&value) {
                    value = match elem {
                        VT::StrOwned => Self::clone_owned_text(value),
                        VT::Rec(_) => ResolvedExpr {
                            span: value.span,
                            vt: value.vt,
                            kind: RExprKind::RecordClone { base: Box::new(value) },
                        },
                        _ => value,
                    };
                }
                self.check_push_value(&elem, &mut value, name.span);
                mk(
                    VT::Unit,
                    RExprKind::ListPush {
                        list,
                        value: Box::new(value),
                    },
                )
            }
            "insert" if rargs.len() == 2 => {
                let index = rargs.remove(0).value;
                let arg = rargs.remove(0);
                let mut value = self.move_marked_value(arg);
                self.check_push_value(&elem, &mut value, name.span);
                mk(
                    VT::Unit,
                    RExprKind::ListInsert {
                        list,
                        index: Box::new(index),
                        value: Box::new(value),
                    },
                )
            }
            "get" if rargs.len() == 1 => {
                let index = rargs.remove(0).value;
                if self.vt_is_coll_owned(elem) {
                    // PLAN-962: the aggregates surface reads a whole owned
                    // element as a VIEW (borrow; materializing uses are
                    // rejected at their sites by `reject_elem_view_use`).
                    // Frozen profiles keep the 822 rejection.
                    if !self.elem_view_admission() {
                        self.bind_err(
                            "source.view-escape",
                            "a whole owned element is not a value; project a field (`xs.get(i).f`) or use `xs.remove(i)`".to_string(),
                            span,
                        );
                    }
                    return mk(
                        elem,
                        RExprKind::ElemView {
                            list,
                            index: Box::new(index),
                            path: Vec::new(),
                        },
                    );
                }
                mk(
                    elem,
                    RExprKind::ListGet {
                        list,
                        index: Box::new(index),
                    },
                )
            }
            "set" if rargs.len() == 2 => {
                let index = rargs.remove(0).value;
                let arg = rargs.remove(0);
                let mut value = self.move_marked_value(arg);
                self.check_push_value(&elem, &mut value, name.span);
                mk(
                    VT::Unit,
                    RExprKind::ListSet {
                        list,
                        index: Box::new(index),
                        value: Box::new(value),
                    },
                )
            }
            "remove" if rargs.len() == 1 => {
                let index = rargs.remove(0).value;
                mk(
                    elem,
                    RExprKind::ListRemove {
                        list,
                        index: Box::new(index),
                    },
                )
            }
            // PLAN-1082 (T-03): the 972 HOF method surface — dispatch to
            // the grafted frozen stdlib public functions (same source, same
            // compiler; never a second algorithm). List<int> only: the
            // stdlib family is int-instantiated. `pop` rides the same seam
            // for the legacy push/pop obligation rows (the frozen stdlib
            // pop keeps the None-not-trap empty contract).
            "to_array" if self.profile.is_auto_language() && rargs.is_empty() => {
                self.clone_collection_value(*list)
            }
            "filter" if self.profile.is_auto_language() && elem != VT::I32 && rargs.len() == 1 => {
                self.filter_typed_list(li, *list, rargs.remove(0), span)
            }
            "map" | "filter" | "find" | "any" | "all" if rargs.len() == 1 => {
                return self.std_list_hof(name, *list, rargs, span);
            }
            "reduce" if rargs.len() == 2 => {
                return self.std_list_hof(name, *list, rargs, span);
            }
            "pop" if record_pop && rargs.is_empty() => {
                // LIST_REMOVE verifies the index and moves the sole owner
                // out of the list. Empty is the ordinary -1 bounds error.
                let list = self.eval_once(*list);
                let length = mk(
                    VT::I32,
                    RExprKind::ListLen {
                        list: Box::new(list.clone()),
                    },
                );
                let last = mk(
                    VT::I32,
                    RExprKind::Binary {
                        op: BinOp::Sub,
                        lhs: Box::new(length),
                        rhs: Box::new(mk(VT::I32, RExprKind::IntLit(1))),
                    },
                );
                mk(
                    elem,
                    RExprKind::ListRemove {
                        list: Box::new(list),
                        index: Box::new(last),
                    },
                )
            }
            "pop" if rargs.is_empty() => {
                return self.std_list_hof(name, *list, rargs, span);
            }
            "len" if rargs.is_empty() => mk(VT::I32, RExprKind::ListLen { list }),
            "clear" if rargs.is_empty() => mk(VT::Unit, RExprKind::ListClear { list }),
            "clone" if rargs.is_empty() => {
                self.bind_err(
                    "source.unsupported",
                    "list clone is not part of this source profile (elements clone explicitly)"
                        .to_string(),
                    span,
                );
                mk(VT::I32, RExprKind::IntLit(0))
            }
            _ => {
                self.bind_err(
                    "source.undefined",
                    format!(
                        "no list method `{}` with {} argument(s) in this profile (legal: new/push/insert/get/set/remove/len/clear)",
                        name.name,
                        rargs.len()
                    ),
                    span,
                );
                mk(VT::I32, RExprKind::IntLit(0))
            }
        }
    }

    fn clone_collection_value(&self, value: ResolvedExpr) -> ResolvedExpr {
        ResolvedExpr {
            span: value.span,
            vt: value.vt,
            kind: RExprKind::OwnedClone {
                operand: Box::new(value),
            },
        }
    }

    fn filter_typed_list(
        &mut self,
        list_type: usize,
        source: ResolvedExpr,
        predicate: ResolvedArg,
        span: Span,
    ) -> ResolvedExpr {
        let elem = self.coll.lists[list_type];
        if predicate.name.is_some() || predicate.moved {
            self.bind_err(
                "source.argument",
                "filter takes one positional predicate",
                predicate.span,
            );
            return instances::pending_expression(span);
        }
        let signature = match predicate.value.vt {
            VT::Function(index) => self.coll.function_types.get(index).filter(|signature| {
                signature.result == VT::Bool
                    && (signature.params == [elem]
                        || (elem == VT::StrOwned && signature.params == [VT::Str]))
            }),
            _ => None,
        };
        let Some(signature) = signature else {
            self.bind_err(
                "source.type-mismatch",
                format!(
                    "`filter` expects a predicate taking one {} element and returning bool",
                    elem.as_str()
                ),
                predicate.span,
            );
            return instances::pending_expression(span);
        };
        let borrowed_element =
            matches!(elem, VT::Rec(_)) || (elem == VT::StrOwned && signature.params == [VT::Str]);

        let expr = |vt, kind| ResolvedExpr { span, vt, kind };
        let source = self.eval_once(source);
        let predicate = self.eval_once(predicate.value);
        // The predicate visits the input as it stood at call time. Copies
        // of owned candidates can escape in the output without borrowing
        // the source list, and each rejected candidate dies in its iteration.
        let snapshot = self.eval_once(self.clone_collection_value(source));
        let result = self.eval_once(expr(
            VT::List(list_type),
            RExprKind::ListNew { list: list_type },
        ));
        let length = self.eval_once(expr(
            VT::I32,
            RExprKind::ListLen {
                list: Box::new(snapshot.clone()),
            },
        ));
        let index = self.agg_temp("filter_index", VT::I32, true, span);
        self.emit_stmt(RStmt::Let {
            local: index,
            value: expr(VT::I32, RExprKind::IntLit(0)),
            span,
        });
        let read_index = || expr(VT::I32, RExprKind::ReadLocal(index));
        let owned = self.vt_is_coll_owned(elem);
        let value = if owned {
            self.clone_collection_value(expr(
                elem,
                RExprKind::ElemView {
                    list: Box::new(snapshot),
                    index: Box::new(read_index()),
                    path: Vec::new(),
                },
            ))
        } else {
            expr(
                elem,
                RExprKind::ListGet {
                    list: Box::new(snapshot),
                    index: Box::new(read_index()),
                },
            )
        };
        let item = self.agg_temp("filter_item", elem, false, span);
        let read_item = || expr(elem, RExprKind::ReadLocal(item));
        let argument = if owned && !borrowed_element {
            self.clone_collection_value(read_item())
        } else {
            read_item()
        };
        let selected = self.resolve_closure_call(
            predicate,
            vec![ResolvedArg {
                value: argument,
                name: None,
                moved: false,
                span,
            }],
            span,
        );
        let retained = if owned {
            expr(
                elem,
                RExprKind::Move {
                    inner: Box::new(read_item()),
                },
            )
        } else {
            read_item()
        };
        let push = RStmt::ExprStmt {
            value: expr(
                VT::Unit,
                RExprKind::ListPush {
                    list: Box::new(result.clone()),
                    value: Box::new(retained),
                },
            ),
            span,
        };
        let target = LoopRef(self.loop_count);
        self.loop_count += 1;
        self.emit_stmt(RStmt::While {
            target,
            cond: expr(
                VT::Bool,
                RExprKind::Binary {
                    op: BinOp::Lt,
                    lhs: Box::new(read_index()),
                    rhs: Box::new(length),
                },
            ),
            body: RBlock {
                stmts: vec![
                    RStmt::Let {
                        local: item,
                        value,
                        span,
                    },
                    RStmt::If {
                        cond: selected,
                        then: RBlock {
                            stmts: vec![push],
                            span,
                        },
                        els: None,
                        span,
                    },
                    RStmt::Assign {
                        local: index,
                        path: Vec::new(),
                        value: expr(
                            VT::I32,
                            RExprKind::Binary {
                                op: BinOp::Add,
                                lhs: Box::new(read_index()),
                                rhs: Box::new(expr(VT::I32, RExprKind::IntLit(1))),
                            },
                        ),
                        span,
                    },
                ],
                span,
            },
            span,
        });
        expr(
            result.vt,
            RExprKind::Move {
                inner: Box::new(result),
            },
        )
    }

    /// PLAN-1082 (T-03): build the CallFn for a stdlib HOF method — the
    /// receiver rides as argument 0 of the grafted frozen public function.
    fn std_list_hof(
        &mut self,
        name: &ast::Ident,
        list: ResolvedExpr,
        mut rargs: Vec<ResolvedArg>,
        span: Span,
    ) -> ResolvedExpr {
        if let VT::List(li) = list.vt {
            let elem = self.coll.lists[li];
            if elem != VT::I32 {
                self.bind_err(
                    "source.hof-elem-domain",
                    format!(
                        "the stdlib HOF family runs over List<int>; `{}` elements stay with the stdlib generics owner",
                        elem.as_str()
                    ),
                    span,
                );
            }
        }
        let mangled = format!("{}{}", crate::source::adapter::STDLIB_LIST_MANGLE, name.name);
        let Some(sig) = self.sigs.get(&mangled) else {
            self.bind_err(
                "source.unsupported",
                "the stdlib HOF functions are not linked into this module (graft missed)".to_string(),
                span,
            );
            return ResolvedExpr {
                span,
                vt: VT::I32,
                kind: RExprKind::IntLit(0),
            };
        };
        let fn_index = sig.index;
        let result = sig.result;
        let mut args = vec![ResolvedArg {
            value: list,
            name: None,
            moved: false,
            span,
        }];
        args.append(&mut rargs);
        ResolvedExpr {
            span,
            vt: result,
            kind: RExprKind::CallFn { fn_index, args },
        }
    }

    fn resolve_array_literal(
        &mut self,
        elems: &[Expr],
        span: Span,
        // Text views become independent owners only in the list being
        // directly published as the interactive script result.
        publish_script_result: bool,
        explicit_list: Option<usize>,
    ) -> ResolvedExpr {
        // PLAN-1082: rvalue admission is the ORDINARY (auto) face
        // only; the frozen narrow profiles keep the 962 init-only
        // contract (`source.array-lit-position`).
        if !self.profile.is_auto_language() {
            self.bind_err(
                "source.array-lit-position",
                "an array literal is only a let/var initializer in this profile (bind it to a name, then use the list)"
                    .to_string(),
                span,
            );
            return ResolvedExpr {
                span,
                vt: VT::I32,
                kind: RExprKind::IntLit(0),
            };
        }
        if elems.is_empty() && explicit_list.is_none() {
            self.bind_err(
                "source.array-lit-empty",
                "an empty array literal has no element type; write List<T>.new() instead"
                    .to_string(),
                span,
            );
            return ResolvedExpr {
                span,
                vt: VT::I32,
                kind: RExprKind::IntLit(0),
            };
        }
        if elems
            .iter()
            .any(|elem| matches!(elem.kind, ExprKind::ArrayConditionalElem { .. }))
        {
            return self.resolve_conditional_array(elems, span, publish_script_result, explicit_list);
        }
        let mut rs = Vec::with_capacity(elems.len());
        for el in elems {
            let value = self.expr_iterative(el);
            let value = if publish_script_result {
                Self::own_dynamic_text(value)
            } else {
                value
            };
            rs.push(value);
        }
        let li = explicit_list.unwrap_or_else(|| {
            let elem_vt = self.array_element_type(&mut rs);
            self.coll.intern_list(elem_vt)
        });
        let temp = self.agg_temp("arr_lit", VT::List(li), false, span);
        self.emit_stmt(RStmt::Let {
            local: temp,
            value: ResolvedExpr {
                span,
                vt: VT::List(li),
                kind: RExprKind::ListNew { list: li },
            },
            span,
        });
        for (value, source) in rs.into_iter().zip(elems) {
            if let Some(push) = self.array_push(temp, li, source, value, explicit_list.is_some()) {
                self.emit_stmt(push);
            }
        }
        ResolvedExpr {
            span,
            vt: VT::List(li),
            kind: RExprKind::ReadLocal(temp),
        }
    }

    fn resolve_conditional_array(
        &mut self,
        elems: &[Expr],
        span: Span,
        publish_script_result: bool,
        explicit_list: Option<usize>,
    ) -> ResolvedExpr {
        let mut items = Vec::with_capacity(elems.len());
        let mut values = Vec::with_capacity(elems.len());
        for elem in elems {
            let (guard, source_value) = match &elem.kind {
                ExprKind::ArrayConditionalElem { cond, value } => {
                    let (prelude, test, _) =
                        self.resolve_in_branch(|resolver| resolver.resolve_condition(cond));
                    (Some((prelude, test)), value.as_ref())
                }
                _ => (None, elem),
            };
            let (prelude, value, diverges) =
                self.resolve_in_branch(|resolver| resolver.expr_iterative(source_value));
            values.push(if publish_script_result {
                Self::own_dynamic_text(value)
            } else {
                value
            });
            items.push((guard, prelude, source_value, diverges));
        }
        let list = explicit_list.unwrap_or_else(|| {
            let elem_vt = self.array_element_type(&mut values);
            self.coll.intern_list(elem_vt)
        });
        let temp = self.agg_temp("arr_lit", VT::List(list), false, span);
        self.emit_stmt(RStmt::Let {
            local: temp,
            value: ResolvedExpr {
                span,
                vt: VT::List(list),
                kind: RExprKind::ListNew { list },
            },
            span,
        });
        for ((guard, mut prelude, source_value, diverges), value) in items.into_iter().zip(values) {
            if !diverges {
                prelude.extend(self.array_push(temp, list, source_value, value, explicit_list.is_some()));
            }
            if let Some((condition_prelude, cond)) = guard {
                for stmt in condition_prelude {
                    self.emit_stmt(stmt);
                }
                self.emit_stmt(RStmt::If {
                    cond,
                    then: RBlock {
                        stmts: prelude,
                        span: source_value.span,
                    },
                    els: None,
                    span: source_value.span,
                });
            } else {
                for stmt in prelude {
                    self.emit_stmt(stmt);
                }
            }
        }
        ResolvedExpr {
            span,
            vt: VT::List(list),
            kind: RExprKind::Move {
                inner: Box::new(ResolvedExpr {
                    span,
                    vt: VT::List(list),
                    kind: RExprKind::ReadLocal(temp),
                }),
            },
        }
    }

    fn array_push(
        &mut self,
        local: LocalRef,
        list: usize,
        source: &Expr,
        mut value: ResolvedExpr,
        explicit_element_type: bool,
    ) -> Option<RStmt> {
        let span = source.span;
        let element_type = self.coll.lists[list];
        if explicit_element_type {
            retag_literal(&mut value, element_type);
            if value.vt != element_type {
                self.bind_err(
                    "source.type-mismatch",
                    format!("expected list element `{}`, found `{}`", element_type.as_str(), value.vt.as_str()),
                    span,
                );
                return None;
            }
        }
        if self.profile.is_auto_language()
            && matches!(source.kind, ExprKind::ArrayLit(_))
            && matches!(value.kind, RExprKind::ReadLocal(_))
            && self.vt_is_coll_owned(value.vt)
        {
            value = ResolvedExpr {
                span: value.span,
                vt: value.vt,
                kind: RExprKind::Move {
                    inner: Box::new(value),
                },
            };
        }
        self.check_push_value(&element_type, &mut value, span);
        Some(RStmt::ExprStmt {
            value: ResolvedExpr {
                span,
                vt: VT::Unit,
                kind: RExprKind::ListPush {
                    list: Box::new(ResolvedExpr {
                        span,
                        vt: VT::List(list),
                        kind: RExprKind::ReadLocal(local),
                    }),
                    value: Box::new(value),
                },
            },
            span,
        })
    }

    /// PLAN-864: push/insert/set values of owned element type must be a
    /// move-source (`.move` local or a fresh rvalue); Copy elements pass
    /// by value.
    fn array_element_type(&mut self, elements: &mut [ResolvedExpr]) -> VT {
        if self.profile.is_auto_language() {
            // String literals in constructed lists allocate owned elements.
            // Computed views retain their borrowing type.
            for element in elements.iter_mut() {
                retag_literal(element, VT::StrOwned);
            }
        }
        let first = elements[0].vt;
        for element in elements.iter() {
            self.reject_elem_view_use(element, "the literal element");
        }
        if self.profile.is_auto_language() && elements.iter().any(|element| element.vt != first) {
            return VT::Dyn;
        }
        for element in elements.iter().filter(|element| element.vt != first) {
            self.bind_err(
                "source.type-mismatch",
                format!(
                    "array literal elements must share one type; first is `{}`, found `{}`",
                    first.as_str(),
                    element.vt.as_str()
                ),
                element.span,
            );
        }
        first
    }

    fn check_push_value(&mut self, elem: &VT, value: &mut ResolvedExpr, span: Span) {
        let _ = span;
        // PLAN-962: storing a whole-element view would alias an owned
        // element into another container.
        self.reject_elem_view_use(value, "the stored value");
        if *elem == VT::Dyn && value.vt != VT::Dyn {
            let placeholder = ResolvedExpr {
                span,
                vt: VT::Nil,
                kind: RExprKind::NilLit,
            };
            *value = Self::coerce_dynamic(std::mem::replace(value, placeholder), VT::Dyn);
        }
        if !self.vt_is_coll_owned(*elem) {
            return;
        }
        match &value.kind {
            RExprKind::Move { .. } => {}
            RExprKind::RecordClone { .. } if self.coll.age_package => {}
            RExprKind::CtorPos { .. } | RExprKind::CtorNamed { .. } => {
                // a fresh rvalue transfers at commit; its Str-typed literal
                // fields retag to the record's owned field types (AP-4 —
                // the owning context constructs)
                value.vt = *elem;
                if let (RExprKind::CtorPos { args, .. }, VT::Rec(ri)) = (&mut value.kind, *elem) {
                    for (ai, a) in args.iter_mut().enumerate() {
                        if let Some(fvt) = self.records.get(ri).and_then(|r| r.fields.get(ai)) {
                            retag_literal(&mut a.value, fvt.vt);
                        }
                    }
                }
            }
            RExprKind::Method {
                method: BuiltinMethod::Clone,
                ..
            }
            | RExprKind::Method {
                method: BuiltinMethod::To,
                to: Some(VT::StrOwned),
                ..
            } if value.vt == VT::StrOwned => {}
            RExprKind::FStr { .. }
                if value.vt == VT::StrOwned && self.coll.legacy_age_widget_at(value.span) => {}
            // PLAN-105 T-02（U03 真语料 012）：**文本拼接/计算出的 owned 文本**
            // 也是新值（`push(a + " " + b)` 不该要求 `.move`）。
            RExprKind::Binary { .. }
            | RExprKind::StrLit(_)
            | RExprKind::CallFn { .. }
            | RExprKind::VariantCtor { .. } => {
                // a fresh rvalue transfers at commit
                value.vt = *elem;
            }
            // PLAN-1081: a closure/fn-value rvalue is a FRESH construction
            // — a lambda's env is created at this site, and a named fn
            // item's zero-env value (or its explicit `.clone()`) is a new
            // owned handle. It transfers at commit like any fresh rvalue.
            RExprKind::CreateClosure { .. }
            | RExprKind::Method {
                method: BuiltinMethod::ClosureClone,
                ..
            } => {
                value.vt = *elem;
            }
            RExprKind::ReadLocal(_) => {
                if self.coll.implicit_push_transfer {
                    // PLAN-105 T-02：AGE 组件面——整局部入组件状态列表按所有者
                    // 转移接受（与「组件自持状态」口径一致）。
                    value.vt = *elem;
                } else {
                    self.bind_err(
                        "source.move-required",
                        "pushing an owned element requires `.move` (or a fresh value); an implicit transfer is rejected".to_string(),
                        span,
                    );
                }
            }
            _ => {
                self.bind_err(
                    "source.move-required",
                    "an owned element value must be a whole-local `.move` or a fresh construction"
                        .to_string(),
                    span,
                );
            }
        }
    }

    fn end_block_views(&self, statements: &[RStmt], span: Span) -> Vec<RStmt> {
        let mut ended = BTreeSet::new();
        let mut ends = Vec::new();
        for statement in statements.iter().rev() {
            match statement {
                RStmt::EndView { local, .. } => {
                    ended.insert(*local);
                }
                RStmt::Let { local, .. }
                    if self.locals[local.0].vt == VT::Str && !ended.contains(local) =>
                {
                    ends.push(RStmt::EndView {
                        local: *local,
                        span,
                    });
                }
                _ => {}
            }
        }
        ends
    }

    /// Resolve one function body with an explicit block-frame stack (R6:
    /// block nesting is unlimited, so no recursion). Each frame owns a
    /// lexical scope (pushed on open, popped on close); while/loop frames
    /// additionally hold their loop target on the shared loop stack for the
    /// duration of the body. Statement order — and therefore diagnostic
    /// order, loop-id registration order and shadowing — is identical to
    /// the previous recursive walk.
    fn resolve_body(&mut self, root: &ast::Block, value_context: bool) -> RBlock {
        struct RFrame<'a> {
            src: &'a ast::Block,
            idx: usize,
            stmts: Vec<RStmt>,
            cont: RCont<'a>,
        }
        enum RCont<'a> {
            FnBody,
            /// The then-arm closed; `els` is the pending else source block.
            ThenArm {
                cond: ResolvedExpr,
                els: Option<&'a ast::Block>,
                span: Span,
            },
            ElseArm {
                cond: ResolvedExpr,
                then: RBlock,
                span: Span,
            },
            WhileArm {
                cond: ResolvedExpr,
                span: Span,
                target: LoopRef,
            },
            LoopArm {
                span: Span,
                target: LoopRef,
            },
        }
        self.scopes.push(Scope {
            names: BTreeMap::new(),
            views: BTreeMap::new(),
            text_mut_aliases: BTreeMap::new(),
        });
        let mut stack: Vec<RFrame> = vec![RFrame {
            src: root,
            idx: 0,
            stmts: Vec::new(),
            cont: RCont::FnBody,
        }];
        let mut pending_splice: Option<(usize, usize)> = None;
        loop {
            if let Some((exp_depth, pre_len)) = pending_splice.take() {
                let pre = self.pop_sink();
                if !pre.is_empty() && exp_depth > 0 && exp_depth <= stack.len() {
                    let stmts = &mut stack[exp_depth - 1].stmts;
                    let tail = stmts.split_off(pre_len);
                    stmts.extend(pre);
                    stmts.extend(tail);
                }
            }
            let action = {
                let frame = stack.last_mut().expect("resolve frame");
                let src: &ast::Block = frame.src;
                let next = src.stmts.get(frame.idx);
                if next.is_some() {
                    frame.idx += 1;
                }
                next
            };
            let Some(s) = action else {
                let frame = stack.pop().expect("resolve frame");
                self.scopes.pop();
                let mut block = RBlock {
                    stmts: frame.stmts,
                    span: frame.src.span,
                };
                if self.profile.is_auto_language()
                    && (!value_context || !matches!(frame.cont, RCont::FnBody))
                    && rblock_can_fall_through(&block)
                {
                    let ends = self.end_block_views(&block.stmts, block.span);
                    block.stmts.extend(ends);
                }
                match frame.cont {
                    RCont::FnBody => return block,
                    RCont::ThenArm { cond, els, span } => match els {
                        Some(eb) => {
                            self.scopes.push(Scope {
                                names: BTreeMap::new(),
                                views: BTreeMap::new(),
                                text_mut_aliases: BTreeMap::new(),
                            });
                            stack.push(RFrame {
                                src: eb,
                                idx: 0,
                                stmts: Vec::new(),
                                cont: RCont::ElseArm {
                                    cond,
                                    then: block,
                                    span,
                                },
                            });
                        }
                        None => {
                            stack
                                .last_mut()
                                .expect("parent frame")
                                .stmts
                                .push(RStmt::If {
                                    cond,
                                    then: block,
                                    els: None,
                                    span,
                                });
                        }
                    },
                    RCont::ElseArm { cond, then, span } => {
                        stack
                            .last_mut()
                            .expect("parent frame")
                            .stmts
                            .push(RStmt::If {
                                cond,
                                then,
                                els: Some(block),
                                span,
                            });
                    }
                    RCont::WhileArm { cond, span, target } => {
                        self.loop_stack.pop();
                        if self.profile.is_aggregates() {
                            self.ctrl_frames.pop();
                        }
                        let parent = &mut stack.last_mut().expect("parent frame").stmts;
                        parent.push(RStmt::While {
                            cond,
                            body: block,
                            span,
                            target,
                        });
                        if self.profile.is_aggregates() && self.has_outer_ctrl_barrier() {
                            self.emit_post_frame_dispatch(span, parent);
                        }
                    }
                    RCont::LoopArm { span, target } => {
                        self.loop_stack.pop();
                        if self.profile.is_aggregates() {
                            self.ctrl_frames.pop();
                        }
                        let parent = &mut stack.last_mut().expect("parent frame").stmts;
                        parent.push(RStmt::Loop {
                            body: block,
                            span,
                            target,
                            unwind: None,
                        });
                        if self.profile.is_aggregates() && self.has_outer_ctrl_barrier() {
                            self.emit_post_frame_dispatch(span, parent);
                        }
                    }
                }
                continue;
            };
            let pre_len = stack.last().expect("resolve frame").stmts.len();
            pending_splice = Some((stack.len(), pre_len));
            self.push_sink();
            match s {
                Stmt::AggIsMatch {
                    scrutinee,
                    arms,
                    span,
                } => {
                    if self.profile.is_aggregates() {
                        let has_following = stack
                            .last()
                            .is_some_and(|f| f.idx < f.src.stmts.len());
                        let (stmts, _) = self.resolve_agg_is(
                            scrutinee,
                            arms,
                            None,
                            false,
                            *span,
                            has_following,
                        );
                        stack.last_mut().expect("resolve frame").stmts.extend(stmts);
                    } else {
                        self.bind_err(
                            "source.unsupported",
                            "control/generator statements require full aggregates lowering"
                                .to_string(),
                            *span,
                        );
                    }
                }
                Stmt::Try {
                    body,
                    catch,
                    finally,
                    span,
                } => {
                    if self.profile.is_aggregates() {
                        let has_following = stack.last().is_some_and(|f| f.idx < f.src.stmts.len());
                        let stmts = self.resolve_try_stmt(
                            body,
                            catch.as_ref(),
                            finally.as_ref(),
                            *span,
                            has_following,
                        );
                        stack.last_mut().expect("resolve frame").stmts.extend(stmts);
                    } else {
                        self.bind_err(
                            "source.unsupported",
                            "control/generator statements require full aggregates lowering"
                                .to_string(),
                            *span,
                        );
                    }
                }
                Stmt::With {
                    expr,
                    bind,
                    body,
                    span,
                } => {
                    if self.profile.is_aggregates() {
                        let has_following = stack.last().is_some_and(|f| f.idx < f.src.stmts.len());
                        let stmts = self.resolve_with_stmt(expr, bind.as_ref(), body, *span, has_following);
                        stack.last_mut().expect("resolve frame").stmts.extend(stmts);
                    } else {
                        self.bind_err(
                            "source.unsupported",
                            "`with` requires full aggregates lowering".to_string(),
                            *span,
                        );
                    }
                }
                Stmt::Yield { value, span } => {
                    if !self.profile.is_aggregates() || !self.is_generator {
                        self.bind_err(
                            "source.yield-outside-generator",
                            "`yield` can only be used inside a `~Iter<T>` generator function"
                                .to_string(),
                            *span,
                        );
                    } else {
                        let elem_vt = if let VT::Rec(ri) = self.fn_result {
                            self.iter_info.get(&ri).map(|info| info.0)
                        } else {
                            None
                        };
                        let mut val = self.expr_with_expect(value, elem_vt);
                        if self.profile.is_auto_language() {
                            if elem_vt == Some(VT::StrOwned) {
                                val = Self::own_dynamic_text(val);
                            } else if let Some(elem) = elem_vt {
                                retag_literal(&mut val, elem);
                            }
                        }
                        let yield_expr = ResolvedExpr {
                            span: *span,
                            vt: VT::Unit,
                            kind: RExprKind::CallIntrinsic {
                                symbol: "__ace_yield",
                                args: vec![val],
                            },
                        };
                        stack
                            .last_mut()
                            .expect("resolve frame")
                            .stmts
                            .push(RStmt::ExprStmt {
                                value: yield_expr,
                                span: *span,
                            });
                    }
                }
                // PLAN-966（T-02）：语句级 CT 必须已在装配/绑定前折叠；到达
                // resolve 即实现错误——具名拒，不静默当普通语句发射。
                Stmt::HashIf { span, .. }
                | Stmt::HashFor { span, .. }
                | Stmt::HashIs { span, .. } => {
                    self.bind_err(
                        "source.cte.unfolded",
                        "a compile-time statement reached resolution without being folded"
                            .to_string(),
                        *span,
                    );
                }
                // PLAN-1081 (T-03): a collected nested fn item resolves as
                // its own flat unit (Pass 1a) — no statement here. An
                // UNcollected item (inside a lambda body) is a named
                // rejection: the legacy silent-drop bug must not return.
                Stmt::NestedFn { item } => {
                    if self.nested.decl_starts.contains(&item.span.start) {
                        // collected elsewhere — nothing to lower here
                    } else {
                        self.bind_err(
                            "source.unsupported",
                            "nested fn items inside lambda bodies are not admitted in this profile"
                                .to_string(),
                            item.span,
                        );
                    }
                }
                // PLAN-962 (T-08, L-07): `let hold x as v { }` — the
                // whole-local hold. v is a NAME ALIAS of the held local
                // inside the block (>= the old engine's MVP semantics —
                // its own source notes Hold carried no borrow discipline);
                // the binding dies with the block. Path forms and
                // non-record bases are named rejections.
                Stmt::Hold {
                    name,
                    bind,
                    body,
                    span,
                } => {
                    let Some(xl) = self.lookup(&name.name) else {
                        self.bind_err(
                            "source.undefined",
                            format!("no local named `{}` in scope", name.name),
                            name.span,
                        );
                        continue;
                    };
                    let vt = self.locals[xl.0].vt;
                    if !matches!(vt, VT::Rec(_)) {
                        self.bind_err(
                            "source.hold-base",
                            format!(
                                "hold borrows a record local in this profile, found `{}`",
                                vt.as_str()
                            ),
                            name.span,
                        );
                        continue;
                    }
                    self.declared_names.insert(bind.name.clone());
                    self.scopes.push(Scope {
                        names: BTreeMap::new(),
                        views: BTreeMap::new(),
                        text_mut_aliases: BTreeMap::new(),
                    });
                    self.scopes
                        .last_mut()
                        .unwrap()
                        .names
                        .insert(bind.name.clone(), xl);
                    let mut b = self.resolve_body(body, false);
                    self.scopes.pop();
                    stack
                        .last_mut()
                        .expect("resolve frame")
                        .stmts
                        .extend(std::mem::take(&mut b.stmts));
                    let _ = span;
                }
                // PLAN-962 (L-15): `let (a, b, ...) = expr` — flat tuple
                // destructuring: bind the value to a synthetic temp, then
                // one positional read per binding name.
                Stmt::LetDestruct { names, value, span } => {
                    let val = self.expr_iterative(value);
                    let VT::Tuple(ti) = val.vt else {
                        self.bind_err(
                            "source.type-mismatch",
                            format!(
                                "tuple destructuring requires a tuple value, found `{}`",
                                val.vt.as_str()
                            ),
                            value.span,
                        );
                        continue;
                    };
                    let shape = self.coll.tuples[ti].clone();
                    if names.len() != shape.len() {
                        self.bind_err(
                            "source.type-mismatch",
                            format!(
                                "tuple destructuring binds {} name(s) for a {}-element tuple",
                                names.len(),
                                shape.len()
                            ),
                            *span,
                        );
                        continue;
                    }
                    // PLAN-962: when the value is already a local read
                    // (the common `let (x, y) = t` and param cases) the
                    // positional reads project DIRECTLY off that local —
                    // no synthetic temp. Record params arrive as views
                    // (771 frozen semantics); materializing one into an
                    // owned temp would cross view->owned.
                    let base_local = match &val.kind {
                        RExprKind::ReadLocal(l) => Some(*l),
                        _ => None,
                    };
                    let temp = match base_local {
                        Some(l) => l,
                        None => {
                            self.declared_names.insert("__tup".to_string());
                            let t = self.fresh_local("__tup", VT::Tuple(ti), false);
                            stack
                                .last_mut()
                                .expect("resolve frame")
                                .stmts
                                .push(RStmt::Let {
                                    local: t,
                                    value: val,
                                    span: *span,
                                });
                            t
                        }
                    };
                    for (i, n) in names.iter().enumerate() {
                        if self.scopes.last().unwrap().names.contains_key(&n.name) {
                            self.bind_err(
                                "source.duplicate",
                                format!(
                                    "local `{}` is declared more than once in this block",
                                    n.name
                                ),
                                n.span,
                            );
                        }
                        let lref = self.fresh_local(&n.name, shape[i], false);
                        self.scopes
                            .last_mut()
                            .unwrap()
                            .names
                            .insert(n.name.clone(), lref);
                        self.declared_names.insert(n.name.clone());
                        let get = ResolvedExpr {
                            span: n.span,
                            vt: shape[i],
                            kind: RExprKind::TupleGet {
                                base: Box::new(ResolvedExpr {
                                    span: n.span,
                                    vt: VT::Tuple(ti),
                                    kind: RExprKind::ReadLocal(temp),
                                }),
                                index: i,
                            },
                        };
                        stack
                            .last_mut()
                            .expect("resolve frame")
                            .stmts
                            .push(RStmt::Let {
                                local: lref,
                                value: get,
                                span: n.span,
                            });
                    }
                }
                // PLAN-962 (L-48): `xs[i] = v` — element replacement via the
                // index place (list-set semantics; the same op `.set(i, v)`
                // lowers to). Base must be a List local.
                Stmt::IndexAssign(ia) => {
                    let Some(xl) = self.lookup(&ia.base.name) else {
                        self.bind_err(
                            "source.undefined",
                            format!("no local named `{}` in scope", ia.base.name),
                            ia.base.span,
                        );
                        continue;
                    };
                    let base_vt = self.locals[xl.0].vt;
                    // PLAN-964 (L-20): an OBJECT local accepts dynamic-key
                    // writes `obj[k] = v` (Str key, elem-typed value).
                    if matches!(base_vt, VT::Object(_) | VT::Dyn) {
                        let elem = match base_vt {
                            VT::Object(object) => self.coll.objects[object],
                            _ => VT::Dyn,
                        };
                        let index = self.expr_iterative(&ia.index);
                        self.check_object_key(&index);
                        let mut value = self.expr_with_expect(&ia.value, Some(elem));
                        self.reject_elem_view_use(&value, "the stored value");
                        if value.vt != elem {
                            self.bind_err(
                                "source.type-mismatch",
                                format!(
                                    "object element type `{}` expected, found `{}`",
                                    elem.as_str(),
                                    value.vt.as_str()
                                ),
                                ia.value.span,
                            );
                        }
                        self.check_push_value(&elem, &mut value, ia.base.span);
                        let set = ResolvedExpr {
                            span: ia.span,
                            vt: VT::Unit,
                            kind: RExprKind::ObjectSet {
                                base: Box::new(ResolvedExpr {
                                    span: ia.base.span,
                                    vt: base_vt,
                                    kind: RExprKind::ReadLocal(xl),
                                }),
                                key: Box::new(index),
                                value: Box::new(value),
                            },
                        };
                        stack
                            .last_mut()
                            .expect("resolve frame")
                            .stmts
                            .push(RStmt::ExprStmt {
                                value: set,
                                span: ia.span,
                            });
                        continue;
                    }
                    let VT::List(li) = base_vt else {
                        self.bind_err(
                            "source.index-base",
                            format!(
                                "index assignment requires a List local, found `{}`",
                                base_vt.as_str()
                            ),
                            ia.base.span,
                        );
                        continue;
                    };
                    let elem = self.coll.lists[li];
                    let index = self.expr_iterative(&ia.index);
                    let mut value = self.expr_with_expect(&ia.value, Some(elem));
                    if index.vt != VT::I32 {
                        self.bind_err(
                            "source.index-type",
                            "an index must be i32 in this profile".to_string(),
                            ia.index.span,
                        );
                    }
                    // PLAN-962: the stored value may not materialize a
                    // whole-element view (an alias into another list).
                    self.reject_elem_view_use(&value, "the stored value");
                    if value.vt != elem {
                        self.bind_err(
                            "source.type-mismatch",
                            format!(
                                "element type `{}` expected, found `{}`",
                                elem.as_str(),
                                value.vt.as_str()
                            ),
                            ia.value.span,
                        );
                    }
                    // a fresh owned rvalue transfers at commit (same rule
                    // as push); an owned local must move explicitly.
                    self.check_push_value(&elem, &mut value, ia.base.span);
                    // structural modification of an iterated list rejects
                    // (same contract as `.set`).
                    if let Some(hit) = self
                        .iterate_targets
                        .iter()
                        .find(|(t, _, _)| *t == xl)
                        .map(|(_, _, sp)| *sp)
                    {
                        self.bind_err(
                            "source.iterate-mutate",
                            "`xs[i] = v` structurally modifies the list being iterated (legal: element field writes)"
                                .to_string(),
                            ia.span.join(hit),
                        );
                    }
                    let set = ResolvedExpr {
                        span: ia.span,
                        vt: VT::Unit,
                        kind: RExprKind::ListSet {
                            list: Box::new(ResolvedExpr {
                                span: ia.base.span,
                                vt: VT::List(li),
                                kind: RExprKind::ReadLocal(xl),
                            }),
                            index: Box::new(index),
                            value: Box::new(value),
                        },
                    };
                    stack
                        .last_mut()
                        .expect("resolve frame")
                        .stmts
                        .push(RStmt::ExprStmt {
                            value: set,
                            span: ia.span,
                        });
                }
                // PLAN-1082 (L-19/L-48): `base (.f | [e])* (op)= v` — the
                // unified place-chain write. Ordered after path/index assign
                // in the grammar, so the shapes arriving here are mixed
                // field×index chains, chained indices, and every compound
                // `+= -= *= /=` (including the bare-name compound form).
                // Source order base→index→rhs holds by hoisting each into a
                // temp exactly once (PLACE-SIDE-EFFECT contract).
                Stmt::PlaceAssign(pa) => {
                    self.resolve_place_assign(pa);
                }
                Stmt::Let {
                    name,
                    ann,
                    mutable,
                    value,
                    span,
                    ..
                } => {
                    if let ExprKind::MutArg { name: owner } = &value.kind {
                        if self.profile.is_auto_language() && ann.is_none() {
                            if let Some(root) = self
                                .lookup(&owner.name)
                                .filter(|root| self.locals[root.0].vt == VT::StrOwned)
                            {
                                self.bind_text_mut_alias(name, root, value.span);
                                continue;
                            }
                        }
                    }
                    // PLAN-962 (L-16): a NON-EMPTY array literal in the
                    // let/var initializer position desugars at the
                    // statement level — bind the target to List<T>.new()
                    // then push each element in source order. Literals in
                    // other expression positions reject at `expr()` with
                    // `source.array-lit-position`.
                    // PLAN-962 (T-08, L-08): `let b = Box(v)` — the single
                    // cell constructs as List<T>.new() + one push (element
                    // type from the operand; Box in non-let positions is a
                    // named rejection at resolve_call).
                    if let ExprKind::Call { callee, args } = &value.kind {
                        if callee.name == "Box"
                            && !self.sigs.contains_key("Box")
                            && args.len() == 1
                            && matches!(&args[0], ast::Arg::Pos(_) | ast::Arg::MovePos(_))
                        {
                            let inner_ast = match &args[0] {
                                ast::Arg::Pos(e) | ast::Arg::MovePos(e) => e,
                                _ => unreachable!("checked shape"),
                            };
                            let mut inner = self.expr_iterative(inner_ast);
                            let elem_vt = inner.vt;
                            let li = self.coll.intern_list(elem_vt);
                            if self.scopes.last().unwrap().names.contains_key(&name.name) {
                                self.bind_err(
                                    "source.duplicate",
                                    format!(
                                        "local `{}` is declared more than once in this block",
                                        name.name
                                    ),
                                    name.span.join(value.span),
                                );
                            }
                            self.declared_names.insert(name.name.clone());
                            let lref = self.fresh_local(&name.name, VT::List(li), *mutable);
                            self.scopes
                                .last_mut()
                                .unwrap()
                                .names
                                .insert(name.name.clone(), lref);
                            stack
                                .last_mut()
                                .expect("resolve frame")
                                .stmts
                                .push(RStmt::Let {
                                    local: lref,
                                    value: ResolvedExpr {
                                        span: value.span,
                                        vt: VT::List(li),
                                        kind: RExprKind::ListNew { list: li },
                                    },
                                    span: *span,
                                });
                            self.check_push_value(&elem_vt, &mut inner, inner_ast.span);
                            let push = ResolvedExpr {
                                span: inner_ast.span,
                                vt: VT::Unit,
                                kind: RExprKind::ListPush {
                                    list: Box::new(ResolvedExpr {
                                        span: inner_ast.span,
                                        vt: VT::List(li),
                                        kind: RExprKind::ReadLocal(lref),
                                    }),
                                    value: Box::new(inner),
                                },
                            };
                            stack
                                .last_mut()
                                .expect("resolve frame")
                                .stmts
                                .push(RStmt::ExprStmt {
                                    value: push,
                                    span: inner_ast.span,
                                });
                            continue;
                        }
                    }
                    // PLAN-962 (T-08, L-14): a grid literal in the
                    // let/var initializer position desugars to ONE LIST
                    // PER COLUMN (column-tuple representation): static
                    // dimension checks at bind — every row's cell count
                    // must equal the column count, column names unique.
                    if let ExprKind::GridLit { cols, rows } = &value.kind {
                        if cols.is_empty() {
                            self.bind_err(
                                "source.grid-cols",
                                "a grid needs at least one column".to_string(),
                                value.span,
                            );
                            continue;
                        }
                        let mut seen_cols = std::collections::BTreeSet::new();
                        for c in cols {
                            if !seen_cols.insert(c.name.clone()) {
                                self.bind_err(
                                    "source.grid-cols",
                                    format!("grid column `{}` is declared more than once", c.name),
                                    c.span,
                                );
                            }
                        }
                        for (ri, row) in rows.iter().enumerate() {
                            if row.len() != cols.len() {
                                self.bind_err(
                                    "source.grid-dims",
                                    format!(
                                        "grid row {} has {} cell(s) for {} column(s)",
                                        ri + 1,
                                        row.len(),
                                        cols.len()
                                    ),
                                    value.span,
                                );
                            }
                        }
                        // resolve cells; unify per column
                        let ncols = cols.len();
                        let mut cell_exprs: Vec<Vec<ResolvedExpr>> = Vec::with_capacity(rows.len());
                        for row in rows {
                            let mut cells = Vec::with_capacity(ncols);
                            for cell in row {
                                cells.push(self.expr_iterative(cell));
                            }
                            cell_exprs.push(cells);
                        }
                        let mut col_vts = Vec::with_capacity(ncols);
                        for j in 0..ncols {
                            let first = cell_exprs
                                .iter()
                                .filter(|r| r.len() == ncols)
                                .find_map(|r| r.get(j).map(|e| e.vt));
                            let Some(elem_vt) = first else {
                                self.bind_err(
                                    "source.grid-dims",
                                    "a grid needs at least one row of the declared width"
                                        .to_string(),
                                    value.span,
                                );
                                continue;
                            };
                            for (ri, r) in cell_exprs.iter().enumerate() {
                                if let Some(c) = r.get(j) {
                                    if c.vt != elem_vt {
                                        self.bind_err(
                                            "source.type-mismatch",
                                            format!(
                                                "grid column `{}` cells must share one type; first is `{}`, row {} has `{}`",
                                                cols[j].name,
                                                elem_vt.as_str(),
                                                ri + 1,
                                                c.vt.as_str()
                                            ),
                                            cols[j].span,
                                        );
                                    }
                                }
                            }
                            col_vts.push(elem_vt);
                        }
                        // per-column list temps + pushes
                        let mut col_temps = Vec::with_capacity(ncols);
                        let mut col_list_vts = Vec::with_capacity(ncols);
                        for (j, &elem_vt) in col_vts.iter().enumerate() {
                            let li = self.coll.intern_list(elem_vt);
                            col_list_vts.push(VT::List(li));
                            let tref = self.fresh_local(
                                &format!("__gridc_{}", cols[j].name),
                                VT::List(li),
                                false,
                            );
                            self.declared_names
                                .insert(format!("__gridc_{}", cols[j].name));
                            col_temps.push(tref);
                            stack
                                .last_mut()
                                .expect("resolve frame")
                                .stmts
                                .push(RStmt::Let {
                                    local: tref,
                                    value: ResolvedExpr {
                                        span: value.span,
                                        vt: VT::List(li),
                                        kind: RExprKind::ListNew { list: li },
                                    },
                                    span: *span,
                                });
                            for (ri, r) in cell_exprs.iter_mut().enumerate() {
                                if r.len() == ncols {
                                    let mut cell = std::mem::replace(
                                        &mut r[j],
                                        ResolvedExpr {
                                            span: *span,
                                            vt: VT::I32,
                                            kind: RExprKind::IntLit(0),
                                        },
                                    );
                                    let cell_span = cell.span;
                                    self.check_push_value(&elem_vt, &mut cell, cell_span);
                                    let push = ResolvedExpr {
                                        span: cell_span,
                                        vt: VT::Unit,
                                        kind: RExprKind::ListPush {
                                            list: Box::new(ResolvedExpr {
                                                span: cell_span,
                                                vt: VT::List(li),
                                                kind: RExprKind::ReadLocal(tref),
                                            }),
                                            value: Box::new(cell),
                                        },
                                    };
                                    let _ = ri;
                                    stack.last_mut().expect("resolve frame").stmts.push(
                                        RStmt::ExprStmt {
                                            value: push,
                                            span: cell_span,
                                        },
                                    );
                                }
                            }
                        }
                        // the bound local: tuple of column lists
                        let ti = self.coll.intern_tuple(col_list_vts.clone());
                        if self.scopes.last().unwrap().names.contains_key(&name.name) {
                            self.bind_err(
                                "source.duplicate",
                                format!(
                                    "local `{}` is declared more than once in this block",
                                    name.name
                                ),
                                name.span.join(value.span),
                            );
                        }
                        self.declared_names.insert(name.name.clone());
                        let lref = self.fresh_local(&name.name, VT::Tuple(ti), *mutable);
                        self.scopes
                            .last_mut()
                            .unwrap()
                            .names
                            .insert(name.name.clone(), lref);
                        let elems: Vec<ResolvedExpr> = col_temps
                            .iter()
                            .zip(col_list_vts.iter())
                            .map(|(&t, &vt)| ResolvedExpr {
                                span: *span,
                                vt,
                                kind: RExprKind::Move {
                                    inner: Box::new(ResolvedExpr {
                                        span: *span,
                                        vt,
                                        kind: RExprKind::ReadLocal(t),
                                    }),
                                },
                            })
                            .collect();
                        stack
                            .last_mut()
                            .expect("resolve frame")
                            .stmts
                            .push(RStmt::Let {
                                local: lref,
                                value: ResolvedExpr {
                                    span: value.span,
                                    vt: VT::Tuple(ti),
                                    kind: RExprKind::TupleLit { elems },
                                },
                                span: *span,
                            });
                        continue;
                    }
                    let plain_array = match &value.kind {
                        ExprKind::ArrayLit(elements)
                            if !elements.iter().any(|elem| {
                                matches!(elem.kind, ExprKind::ArrayConditionalElem { .. })
                            }) =>
                        {
                            Some(elements)
                        }
                        _ => None,
                    };
                    if let Some(elems_ast) = plain_array {
                        let mut elems = Vec::with_capacity(elems_ast.len());
                        for e in elems_ast {
                            elems.push(self.expr_iterative(e));
                        }
                        let mut legacy_text = ann.as_ref().is_some_and(is_legacy_text_list);
                        let li = if elems.is_empty() {
                            let explicit = ann.as_ref().map(|ann| {
                                vt_of_ann_coll(
                                    ann,
                                    self.rec_index,
                                    self.coll,
                                    self.diags,
                                    TypePos::Local,
                                )
                            });
                            let returns_binding = matches!(
                                root.stmts.last(),
                                Some(Stmt::Return { value, .. } | Stmt::ExprStmt { value, .. })
                                    if matches!(&value.kind, ExprKind::Var(id) if id.name == name.name)
                            ) && stack.len() == 1;
                            legacy_text |=
                                explicit.is_none() && returns_binding && self.legacy_text_result;
                            let expected =
                                explicit.or_else(|| returns_binding.then_some(self.fn_result));
                            if let (true, Some(VT::List(li))) =
                                (self.profile.is_auto_language(), expected)
                            {
                                li
                            } else {
                                self.bind_err(
                                    "source.array-lit-empty",
                                    "an empty array literal needs an expected List type; write List<T>.new() instead",
                                    value.span,
                                );
                                continue;
                            }
                        } else {
                            let elem_vt = self.array_element_type(&mut elems);
                            self.coll.intern_list(elem_vt)
                        };
                        let elem_vt = self.coll.lists[li];
                        if self.scopes.last().unwrap().names.contains_key(&name.name) {
                            self.bind_err(
                                "source.duplicate",
                                format!(
                                    "local `{}` is declared more than once in this block",
                                    name.name
                                ),
                                name.span.join(value.span),
                            );
                        }
                        self.declared_names.insert(name.name.clone());
                        let lref = self.fresh_local(&name.name, VT::List(li), *mutable);
                        if self.profile.is_auto_language() && legacy_text {
                            self.legacy_text_lists.insert(lref);
                        }
                        self.scopes
                            .last_mut()
                            .unwrap()
                            .names
                            .insert(name.name.clone(), lref);
                        let new_expr = ResolvedExpr {
                            span: value.span,
                            vt: VT::List(li),
                            kind: RExprKind::ListNew { list: li },
                        };
                        stack
                            .last_mut()
                            .expect("resolve frame")
                            .stmts
                            .push(RStmt::Let {
                                local: lref,
                                value: new_expr,
                                span: *span,
                            });
                        for (e, ast_e) in elems.into_iter().zip(elems_ast.iter()) {
                            let mut e = e;
                            // PLAN-1082: a nested array-literal element rides
                            // an explicit `Move` of its synthetic fresh temp
                            // (same rule as the rvalue expansion above).
                            if self.profile.is_auto_language()
                                && matches!(ast_e.kind, ast::ExprKind::ArrayLit(_))
                                && matches!(e.kind, RExprKind::ReadLocal(_))
                                && self.vt_is_coll_owned(e.vt)
                            {
                                let inner_vt = e.vt;
                                let span0 = e.span;
                                let taken = std::mem::replace(&mut e.kind, RExprKind::IntLit(0));
                                e = ResolvedExpr {
                                    span: span0,
                                    vt: inner_vt,
                                    kind: RExprKind::Move {
                                        inner: Box::new(ResolvedExpr {
                                            span: span0,
                                            vt: inner_vt,
                                            kind: taken,
                                        }),
                                    },
                                };
                            }
                            self.check_push_value(&elem_vt, &mut e, ast_e.span);
                            let push = ResolvedExpr {
                                span: ast_e.span,
                                vt: VT::Unit,
                                kind: RExprKind::ListPush {
                                    list: Box::new(ResolvedExpr {
                                        span: ast_e.span,
                                        vt: VT::List(li),
                                        kind: RExprKind::ReadLocal(lref),
                                    }),
                                    value: Box::new(e),
                                },
                            };
                            stack
                                .last_mut()
                                .expect("resolve frame")
                                .stmts
                                .push(RStmt::ExprStmt {
                                    value: push,
                                    span: ast_e.span,
                                });
                        }
                        continue;
                    }
                    // RHS resolves in the pre-declaration environment. On
                    // the collections surface an annotation is also the
                    // bare-constructor context (`var r Result<int, Str> =
                    // Err("old")`).
                    let legacy_widget = self.coll.legacy_age_widget_at(value.span);
                    let mut ann_vt: Option<VT> = match ann {
                        Some(a) => {
                            if self.profile.is_collections() {
                                let mut discarded = Vec::new();
                                let diagnostics = if self.profile.is_auto_language()
                                    && matches!(a.kind, ast::TypeRef::Generic { .. })
                                {
                                    &mut *self.diags
                                } else {
                                    &mut discarded
                                };
                                Some(vt_of_ann_coll(
                                    a,
                                    self.rec_index,
                                    self.coll,
                                    diagnostics,
                                    TypePos::Local,
                                ))
                            } else {
                                Some(vt_of_ann(a, self.rec_index, &mut Vec::new()))
                            }
                        }
                        None => None,
                    };
                    if legacy_widget && *mutable && ann_vt == Some(VT::Str) {
                        ann_vt = Some(VT::StrOwned);
                    }
                    let mut value = self.expr_with_expect(value, ann_vt);
                    if self.coll.script_errors && self.expr_diverged {
                        let frame = stack.last_mut().expect("resolve frame");
                        frame.idx = frame.src.stmts.len();
                        continue;
                    }
                    if legacy_widget {
                        value = Self::copy_legacy_text_local(value, ann_vt);
                    }
                    if ann.is_none()
                        && Self::is_whole_elem_view(&value)
                        && matches!(value.vt, VT::Rec(ri) if
                            (!*mutable && self.coll.legacy_text_records.contains(&ri))
                            || (legacy_widget && self.records[ri].fields.iter().all(|field|
                                field.vt.is_text() || field.vt == VT::Bool || vt_is_numeric(field.vt))))
                    {
                        value = ResolvedExpr {
                            span: value.span,
                            vt: value.vt,
                            kind: RExprKind::RecordClone {
                                base: Box::new(value),
                            },
                        };
                    }
                    self.reject_elem_view_use(&value, "the bound value");
                    let mut rule = match ann_vt {
                        Some(v) => v,
                        None => value.vt,
                    };
                    // PLAN-964: on the collections-family surfaces a bare
                    // string literal binds as an OWNED Str — the borrowed
                    // `str` type has no local form there (the verifier
                    // rejects `str` locals; the owned promotion is the
                    // collections-domain AP-4 rule made static at bind).
                    if (self.profile.is_collections()
                        || self.profile.is_closures()
                        || self.profile.is_aggregates()
                        || self.profile.is_numeric())
                        && matches!(value.kind, RExprKind::StrLit(_))
                        && value.vt == VT::Str
                    {
                        value.vt = VT::StrOwned;
                        rule = VT::StrOwned;
                    }
                    retag_literal(&mut value, rule);
                    let vt = self.binding_ty(*span, rule);
                    if !self.profile.is_auto_language()
                        && self.scopes.last().unwrap().names.contains_key(&name.name)
                    {
                        self.bind_err(
                            "source.duplicate",
                            format!(
                                "local `{}` is declared more than once in this block",
                                name.name
                            ),
                            name.span.join(value.span),
                        );
                    }
                    // PLAN-771 V-08: a local must not shadow a record type
                    // name (type positions stay unambiguous)
                    if self.rec_index.contains_key(&name.name) {
                        self.bind_err(
                            "source.duplicate",
                            format!(
                                "local `{}` shadows the record type of the same name; rename the local",
                                name.name
                            ),
                            name.span.join(value.span),
                        );
                    }
                    let cached_find = (ann.is_none() && !*mutable)
                        .then(|| self.cache_text_find_index(&mut value))
                        .flatten();
                    let lref = LocalRef(self.locals.len());
                    if let Some(index) = cached_find {
                        self.legacy_text_find_results.insert(lref, index);
                    }
                    // PLAN-1026: a binding whose initializer is a Python
                    // OBJECT (hoisted member import or an object-returning
                    // op) carries the py-op domain discipline.
                    if self.py_is_object(&value) {
                        self.py_locals.insert(lref);
                    }
                    // PLAN-1026: an exact float literal binding folds into
                    // the frozen `F:` frame encoding at py call sites.
                    if let RExprKind::TypedLit {
                        vt: vt @ (VT::F64 | VT::F32),
                        bits,
                    } = &value.kind
                    {
                        self.py_lit_locals.insert(lref, (*vt, *bits));
                    }
                    if matches!(
                        &value.kind,
                        RExprKind::CreateClosure { captures, .. }
                            if captures.iter().any(|(_, mode)| *mode == crate::hir::CaptureMode::Borrow)
                    ) {
                        self.borrowed_closure_locals.insert(lref);
                    }
                    // PLAN-909: a scalar `var` some non-move closure of this
                    // function writes is a shared cell from its declaration
                    // on (static grouping by the declaring name token's
                    // offset, computed by `scan_shared_capture_cells`
                    // before the body resolved).
                    let cell_binding = *mutable
                        && matches!(vt, VT::I32 | VT::Bool | VT::Char)
                        && self.shared_cell_spans.contains(&name.span.start);
                    self.locals.push(LocalDecl {
                        name: name.name.clone(),
                        vt,
                        mutable: *mutable,
                        mode: ast::ParamMode::Default,
                        is_param: false,
                        param_index: None,
                        capture_index: None,
                        cell: cell_binding,
                        is_capture: false,
                        span: name.span.join(value.span),
                    });
                    self.scopes
                        .last_mut()
                        .unwrap()
                        .names
                        .insert(name.name.clone(), lref);
                    self.declared_names.insert(name.name.clone());
                    stack
                        .last_mut()
                        .expect("resolve frame")
                        .stmts
                        .push(RStmt::Let {
                            local: lref,
                            value,
                            span: *span,
                        });
                }
                Stmt::Assign {
                    target,
                    value,
                    span,
                } => {
                    // PLAN-864: an assigned VIEW binding is a view escape.
                    if self.lookup_view(&target.base.name).is_some() {
                        self.bind_err(
                            "source.view-escape",
                            format!(
                                "`{}` is an owned-element view; views are read-only projections and cannot be assigned",
                                target.base.name
                            ),
                            target.base.span,
                        );
                    }
                    if self
                        .lookup(&target.base.name)
                        .is_some_and(|local| self.moved_locals.contains(&local))
                    {
                        self.bind_err(
                            "source.closure-use-after-move",
                            format!(
                                "local `{}` was moved into a closure and cannot be assigned",
                                target.base.name
                            ),
                            target.base.span,
                        );
                    }
                    if let Some(stmt) = self.resolve_object_field_assign(target, value, *span) {
                        self.emit_stmt(stmt);
                        continue;
                    }
                    let expect = self.lookup(&target.base.name).map(|l| self.locals[l.0].vt);
                    let mut value = self.expr_with_expect(value, expect);
                    if self.coll.legacy_age_widget_at(value.span) {
                        value = Self::copy_legacy_text_local(value, expect);
                    }
                    // PLAN-962: storing a whole-element view into a local
                    // is a view escape (the alias must not materialize).
                    self.reject_elem_view_use(&value, "the assigned value");
                    if matches!(
                        &value.kind,
                        RExprKind::CreateClosure { captures, .. }
                            if captures.iter().any(|(_, mode)| *mode == crate::hir::CaptureMode::Borrow)
                    ) {
                        self.bind_err(
                            "source.closure-borrow-escape",
                            "a borrow-capturing closure must be initialized in a local and called directly",
                            value.span,
                        );
                    }
                    // Immutability itself is re-checked by the semantic verifier
                    // on the emitted place/local pair (verify.immutable-write);
                    // here only resolution failures are reported.
                    let rstmt = match self.lookup(&target.base.name) {
                        Some(l) => {
                            // resolve the field path segment by segment
                            // against the actual owner records
                            let mut path: Vec<(usize, usize)> = Vec::new();
                            let mut cur = self.locals[l.0].vt;
                            let mut ok = true;
                            for seg in &target.path {
                                let VT::Rec(ri) = cur else {
                                    self.bind_err(
                                        "source.type-mismatch",
                                        format!(
                                            "`{}` has type `{}`; field paths continue through records only",
                                            seg.name,
                                            cur.as_str()
                                        ),
                                        seg.span,
                                    );
                                    ok = false;
                                    break;
                                };
                                match self.records[ri]
                                    .fields
                                    .iter()
                                    .position(|fl| fl.name == seg.name)
                                {
                                    Some(fi) => {
                                        cur = self.records[ri].fields[fi].vt;
                                        path.push((ri, fi));
                                    }
                                    None => {
                                        self.bind_err(
                                            "source.undefined",
                                            format!(
                                                "record `{}` has no field named `{}`",
                                                self.records[ri].name, seg.name
                                            ),
                                            seg.span,
                                        );
                                        ok = false;
                                        break;
                                    }
                                }
                            }
                            let _ = ok;
                            // PLAN-782 AP-4: an owning Str-field
                            // replacement promotes a borrowing literal
                            // exactly like a constructor field (the owning
                            // context constructs)
                            if cur == VT::StrOwned && matches!(value.kind, RExprKind::StrLit(_)) {
                                value.vt = VT::StrOwned;
                            }
                            RStmt::Assign {
                                local: l,
                                path,
                                value,
                                span: *span,
                            }
                        }
                        None => {
                            if self.sigs.contains_key(&target.base.name) {
                                self.bind_err(
                                    "source.undefined",
                                    format!(
                                        "cannot assign to `{}`: functions are not assignable locals",
                                        target.base.name
                                    ),
                                    target.base.span,
                                );
                            } else {
                                self.bind_err(
                                    "source.undefined",
                                    format!("no local named `{}` in scope", target.base.name),
                                    target.base.span,
                                );
                            }
                            RStmt::Assign {
                                local: LocalRef(usize::MAX),
                                path: Vec::new(),
                                value,
                                span: *span,
                            }
                        }
                    };
                    stack.last_mut().expect("resolve frame").stmts.push(rstmt);
                }
                Stmt::Return { value, span } => {
                    if self.profile.is_aggregates() {
                        if self.is_generator {
                            let is_empty_ret = matches!(&value.kind, ExprKind::TupleLit(xs) if xs.is_empty())
                                || matches!(&value.kind, ExprKind::Var(id) if id.name == "None");
                            if !is_empty_ret {
                                let _ = self.expr_iterative(value);
                            }
                            let mut ret_stmts = Vec::new();
                            self.emit_return_to_out(None, *span, &mut ret_stmts);
                            stack
                                .last_mut()
                                .expect("resolve frame")
                                .stmts
                                .extend(ret_stmts);
                            continue;
                        }
                        self.expr_diverged = false;
                        let mut value = self.expr_with_expect(value, Some(self.fn_result));
                        if self.expr_diverged {
                            if self.coll.script_errors {
                                let frame = stack.last_mut().expect("resolve frame");
                                frame.idx = frame.src.stmts.len();
                            }
                            continue;
                        }
                        self.reject_elem_view_use(&value, "the returned value");
                        if self.owned_spec_result && self.fn_result == VT::StrOwned {
                            value = Self::own_dynamic_text(value);
                        }
                        if value.vt == self.fn_result {
                            self.copy_record_parameter(&mut value);
                        }
                        retag_literal(&mut value, self.fn_result);
                        if self.profile.is_auto_language() {
                            retag_integer_literal(&mut value, self.fn_result);
                        }
                        if matches!(
                            &value.kind,
                            RExprKind::CreateClosure { captures, .. }
                                if captures.iter().any(|(_, mode)| *mode == crate::hir::CaptureMode::Borrow)
                        ) {
                            self.bind_err(
                                "source.closure-borrow-escape",
                                "a borrow-capturing closure cannot be returned",
                                value.span,
                            );
                        }
                        let mut ret_stmts = Vec::new();
                        self.emit_return_to_out(Some(value), *span, &mut ret_stmts);
                        stack
                            .last_mut()
                            .expect("resolve frame")
                            .stmts
                            .extend(ret_stmts);
                    } else {
                        let mut value = self.expr_with_expect(value, Some(self.fn_result));
                        self.reject_elem_view_use(&value, "the returned value");
                        retag_literal(&mut value, self.fn_result);
                        if matches!(
                            &value.kind,
                            RExprKind::CreateClosure { captures, .. }
                                if captures.iter().any(|(_, mode)| *mode == crate::hir::CaptureMode::Borrow)
                        ) {
                            self.bind_err(
                                "source.closure-borrow-escape",
                                "a borrow-capturing closure cannot be returned",
                                value.span,
                            );
                        }
                        stack
                            .last_mut()
                            .expect("resolve frame")
                            .stmts
                            .push(RStmt::Return { value, span: *span });
                    }
                }
                Stmt::ExprStmt { value, span } => {
                    self.expr_diverged = false;
                    let is_root_tail = {
                        let frame = stack.last().expect("resolve frame");
                        value_context
                            && matches!(frame.cont, RCont::FnBody)
                            && frame.idx == frame.src.stmts.len()
                    };
                    let value = match &value.kind {
                        ExprKind::ArrayLit(elems)
                            if self.profile.is_auto_language()
                                && self.is_script
                                && is_root_tail =>
                        {
                            self.resolve_array_literal(elems, value.span, true, None)
                        }
                        _ => self.expr_iterative(value),
                    };
                    if self.profile == SourceProfile::Auto && matches!(value.kind, RExprKind::Unit)
                    {
                        continue;
                    }
                    if self.profile.is_aggregates() && self.expr_diverged {
                        if self.coll.script_errors {
                            let frame = stack.last_mut().expect("resolve frame");
                            frame.idx = frame.src.stmts.len();
                        }
                        continue;
                    }
                    let is_iter_close = self.profile.is_aggregates()
                        && matches!(
                            &value.kind,
                            RExprKind::CallFn { fn_index, .. }
                                if self.iter_info.values().any(|info| info.3 == *fn_index)
                        );
                    // A function body's ROOT tail expression IS its result
                    // (`fn pick() ?int { ... May.val(x) }`); it is rewritten
                    // to a Return after resolution. Every other non-unit
                    // expression statement is evaluated for effect and its
                    // value bound to a discarded temp — PLAN-965
                    // (F-965-R1-01): the old silent drop lost calls (and a
                    // callee's language error) entirely.
                    let sched_effect = self.profile.is_auto_language()
                        && self.fn_result == VT::Unit
                        && matches!(&value.kind, RExprKind::CallIntrinsic { symbol, .. }
                            if symbol.starts_with("__sched_send_")
                                || symbol.starts_with("__sched_go_")
                                || matches!(*symbol, "__sched_stop" | "__sched_sleep" | "__sched_detach"));
                    let stmt = if is_iter_close || sched_effect {
                        let local = self.agg_temp("it_close", VT::I32, false, *span);
                        RStmt::Let {
                            local,
                            value,
                            span: *span,
                        }
                    } else if self.profile.is_aggregates()
                        && !is_root_tail
                        && value.vt != VT::Unit
                        && !is_console_effect(&value)
                    {
                        let local = self.agg_temp("ev", value.vt, false, *span);
                        RStmt::Let {
                            local,
                            value,
                            span: *span,
                        }
                    } else {
                        RStmt::ExprStmt { value, span: *span }
                    };
                    stack.last_mut().expect("resolve frame").stmts.push(stmt);
                }
                Stmt::If {
                    cond,
                    then,
                    els,
                    span,
                } => {
                    let cond = self.resolve_condition(cond);
                    self.scopes.push(Scope {
                        names: BTreeMap::new(),
                        views: BTreeMap::new(),
                        text_mut_aliases: BTreeMap::new(),
                    });
                    stack.push(RFrame {
                        src: then,
                        idx: 0,
                        stmts: Vec::new(),
                        cont: RCont::ThenArm {
                            cond,
                            els: els.as_ref(),
                            span: *span,
                        },
                    });
                }
                Stmt::While {
                    cond, body, span, ..
                } => {
                    let target = LoopRef(self.loop_count);
                    self.loop_count += 1;
                    self.loop_stack.push(target);
                    if self.profile.is_aggregates() {
                        let (mut cond_stmts, cond_val, _) =
                            self.resolve_in_branch(|resolver| resolver.resolve_condition(cond));
                        let negated = ResolvedExpr {
                            span: cond.span,
                            vt: VT::Bool,
                            kind: RExprKind::Unary {
                                op: ast::UnaryOp::Not,
                                operand: Box::new(cond_val),
                            },
                        };
                        cond_stmts.push(RStmt::If {
                            cond: negated,
                            then: RBlock {
                                stmts: vec![RStmt::Break {
                                    target: Some(target),
                                    span: *span,
                                }],
                                span: *span,
                            },
                            els: None,
                            span: *span,
                        });
                        self.ctrl_frames.push(CtrlFrame::Loop { target });
                        self.scopes.push(Scope {
                            names: BTreeMap::new(),
                            views: BTreeMap::new(),
                            text_mut_aliases: BTreeMap::new(),
                        });
                        stack.push(RFrame {
                            src: body,
                            idx: 0,
                            stmts: cond_stmts,
                            cont: RCont::LoopArm {
                                span: *span,
                                target,
                            },
                        });
                    } else {
                        let cond = self.resolve_condition(cond);
                        self.scopes.push(Scope {
                            names: BTreeMap::new(),
                            views: BTreeMap::new(),
                            text_mut_aliases: BTreeMap::new(),
                        });
                        stack.push(RFrame {
                            src: body,
                            idx: 0,
                            stmts: Vec::new(),
                            cont: RCont::WhileArm {
                                cond,
                                span: *span,
                                target,
                            },
                        });
                    }
                }
                Stmt::Loop { body, span } => {
                    let target = LoopRef(self.loop_count);
                    self.loop_count += 1;
                    self.loop_stack.push(target);
                    if self.profile.is_aggregates() {
                        self.ctrl_frames.push(CtrlFrame::Loop { target });
                    }
                    self.scopes.push(Scope {
                        names: BTreeMap::new(),
                        views: BTreeMap::new(),
                        text_mut_aliases: BTreeMap::new(),
                    });
                    stack.push(RFrame {
                        src: body,
                        idx: 0,
                        stmts: Vec::new(),
                        cont: RCont::LoopArm {
                            span: *span,
                            target,
                        },
                    });
                }
                Stmt::Break { span } => {
                    if self.profile.is_aggregates() {
                        if !self.has_enclosing_user_loop() {
                            self.bind_err(
                                "source.break-outside-loop",
                                "`break` requires an enclosing loop".to_string(),
                                *span,
                            );
                            continue;
                        }
                        let mut stmts = Vec::new();
                        if self.profile.is_auto_language() {
                            for frame in stack.iter().rev() {
                                stmts.extend(self.end_block_views(&frame.stmts, *span));
                                if matches!(
                                    frame.cont,
                                    RCont::WhileArm { .. } | RCont::LoopArm { .. } | RCont::FnBody
                                ) {
                                    break;
                                }
                            }
                        }
                        self.emit_break_or_continue_to_out(true, *span, &mut stmts);
                        stack.last_mut().expect("resolve frame").stmts.extend(stmts);
                    } else {
                        let target = self.loop_stack.last().copied();
                        stack
                            .last_mut()
                            .expect("resolve frame")
                            .stmts
                            .push(RStmt::Break {
                                target,
                                span: *span,
                            });
                    }
                }
                Stmt::Continue { span } => {
                    if self.profile.is_aggregates() {
                        if !self.has_enclosing_user_loop() {
                            self.bind_err(
                                "source.continue-outside-loop",
                                "`continue` requires an enclosing loop".to_string(),
                                *span,
                            );
                            continue;
                        }
                        let mut stmts = Vec::new();
                        if self.profile.is_auto_language() {
                            for frame in stack.iter().rev() {
                                stmts.extend(self.end_block_views(&frame.stmts, *span));
                                if matches!(
                                    frame.cont,
                                    RCont::WhileArm { .. } | RCont::LoopArm { .. } | RCont::FnBody
                                ) {
                                    break;
                                }
                            }
                        }
                        self.emit_break_or_continue_to_out(false, *span, &mut stmts);
                        stack.last_mut().expect("resolve frame").stmts.extend(stmts);
                    } else {
                        let target = self.loop_stack.last().copied();
                        let mut stmts = Vec::new();
                        // PLAN-1036 (F28): same latch obligation on the
                        // non-aggregates (collections) face.
                        self.emit_for_latch(target, &mut stmts);
                        stmts.push(RStmt::Continue {
                            target,
                            span: *span,
                        });
                        stack.last_mut().expect("resolve frame").stmts.extend(stmts);
                    }
                }
                // PLAN-864: exhaustive `is` dispatch — resolve the arm
                // bodies, check exhaustiveness/duplicates, then synthesize
                // the guard/branch chain (guard test → let-projection →
                // arm body), nested as If/else with NO fall-through
                // default (the last arm's else is empty: exhaustiveness
                // makes it unreachable).
                Stmt::IsMatch {
                    scrutinee,
                    arms,
                    span,
                } => {
                    let Some(sl) = self.lookup(&scrutinee.name) else {
                        self.bind_err(
                            "source.undefined",
                            format!("no local named `{}` in scope", scrutinee.name),
                            scrutinee.span,
                        );
                        continue;
                    };
                    let VT::Variant(vi) = self.locals[sl.0].vt else {
                        self.bind_err(
                            "source.pattern-scrutinee",
                            format!(
                                "`is` scrutinee must be a variant-typed local, found `{}`",
                                self.locals[sl.0].vt.as_str()
                            ),
                            scrutinee.span,
                        );
                        continue;
                    };
                    let mut resolved: Vec<(u32, Option<(LocalRef, VT)>, RBlock)> = Vec::new();
                    let mut seen: BTreeMap<String, u32> = BTreeMap::new();
                    let mut ok = true;
                    for arm in arms {
                        let Some((_, disc)) =
                            self.is_arm_disc(self.locals[sl.0].vt, arm.qual.as_ref(), &arm.pattern)
                        else {
                            ok = false;
                            continue;
                        };
                        match seen.get(arm.pattern.name.as_str()) {
                            Some(_) => {
                                self.bind_err(
                                    "source.pattern-duplicate",
                                    format!("arm `{}` appears more than once", arm.pattern.name),
                                    arm.pattern.span,
                                );
                                ok = false;
                                continue;
                            }
                            None => {
                                seen.insert(arm.pattern.name.clone(), disc);
                            }
                        }
                        // the binding local (if any) types from the arm's
                        // payload; scope-isolated to the arm body
                        let payload_vt = self.coll.variants[vi].arms[disc as usize].1;
                        let pat_name = arm.pattern.name.clone();
                        let bound: Option<(LocalRef, VT)> = match (&arm.binding, payload_vt) {
                            (Some(b), Some(pvt)) => {
                                self.scopes.push(Scope {
                                    names: BTreeMap::new(),
                                    views: BTreeMap::new(),
                                    text_mut_aliases: BTreeMap::new(),
                                });
                                let lref = self.fresh_local(&b.name, pvt, false);
                                self.scopes
                                    .last_mut()
                                    .unwrap()
                                    .names
                                    .insert(b.name.clone(), lref);
                                Some((lref, pvt))
                            }
                            (Some(b), None) => {
                                self.bind_err(
                                    "source.type-mismatch",
                                    format!(
                                        "arm `{}` carries no payload; remove the binding `{}`",
                                        pat_name, b.name
                                    ),
                                    b.span,
                                );
                                None
                            }
                            (None, _) => None,
                        };
                        let body = self.resolve_body(&arm.body, false);
                        if bound.is_some() {
                            self.scopes.pop();
                        }
                        resolved.push((disc, bound, body));
                    }
                    if !ok {
                        continue;
                    }
                    // exhaustiveness: every arm of the variant covered
                    if resolved.len() != self.coll.variants[vi].arms.len() {
                        self.bind_err(
                            "source.pattern-non-exhaustive",
                            format!(
                                "`is` on `{}` covers {} of {} arms",
                                self.coll.variants[vi].name,
                                resolved.len(),
                                self.coll.variants[vi].arms.len()
                            ),
                            *span,
                        );
                        continue;
                    }
                    // Build the chain with EVERY arm guarded (the last
                    // arm too — a projection needs its live guard fact);
                    // the innermost else is a synthetic `return <default>`
                    // filler: statically unreachable (exhaustiveness is
                    // compile-checked above), present so the shared
                    // no-return-path walk sees a Returned spine. The filler
                    // value is typed from the function result.
                    let all_return = resolved.iter().all(|(_, _, b)| rblock_always_returns(b));
                    let mut current = if all_return {
                        let filler = self.default_return_filler(*span);
                        Some(RBlock {
                            stmts: vec![RStmt::Return {
                                value: filler,
                                span: *span,
                            }],
                            span: *span,
                        })
                    } else {
                        None
                    };
                    let resolved_rev: Vec<(u32, Option<(LocalRef, VT)>, RBlock)> =
                        resolved.into_iter().rev().collect();
                    for (disc, bind, mut body) in resolved_rev {
                        let mut then_stmts: Vec<RStmt> = Vec::new();
                        if let Some((lref, pvt)) = bind {
                            then_stmts.push(RStmt::Let {
                                local: lref,
                                value: ResolvedExpr {
                                    span: body.span,
                                    vt: pvt,
                                    kind: RExprKind::VariantProject {
                                        local: sl,
                                        variant: vi,
                                        disc,
                                    },
                                },
                                span: body.span,
                            });
                        }
                        then_stmts.extend(std::mem::take(&mut body.stmts));
                        let then_block = RBlock {
                            stmts: then_stmts,
                            span: body.span,
                        };
                        let cond = ResolvedExpr {
                            span: *span,
                            vt: VT::Bool,
                            kind: RExprKind::VariantGuardTest {
                                local: sl,
                                variant: vi,
                                disc,
                            },
                        };
                        let if_stmt = RStmt::If {
                            cond,
                            then: then_block,
                            els: current.take(),
                            span: *span,
                        };
                        current = Some(RBlock {
                            stmts: vec![if_stmt],
                            span: *span,
                        });
                    }
                    if let Some(mut block) = current {
                        stack
                            .last_mut()
                            .expect("resolve frame")
                            .stmts
                            .extend(std::mem::take(&mut block.stmts));
                    }
                }
                // PLAN-864: `for x in <list> { .. }` — lower to an index
                // loop with the length read ONCE (contract C7). Copyable
                // elements materialize a per-iteration copy binding; owned
                // elements bind a checked read-only projection context.
                Stmt::ForIn {
                    var,
                    iter,
                    body,
                    span,
                } => {
                    // PLAN-962 (L-19): `for i in a..b` / `a..=b` — desugar
                    // to a counted i32 loop (step 1): lo/hi locals, then
                    // the same Loop{If(i</<=hi){body; i=i+1}} shape the
                    // list iteration builds.
                    if let ast::ExprKind::Range {
                        start,
                        end,
                        inclusive,
                    } = &iter.kind
                    {
                        let lo = self.expr_iterative(start);
                        let hi = self.expr_iterative(end);
                        if lo.vt != VT::I32 || hi.vt != VT::I32 {
                            self.bind_err(
                                "source.range-bounds",
                                "range bounds must be i32 in this profile".to_string(),
                                iter.span,
                            );
                        }
                        let lo_l = self.fresh_local("__range_lo", VT::I32, false);
                        let hi_l = self.fresh_local("__range_hi", VT::I32, false);
                        let i_l = self.fresh_local(&var.name, VT::I32, true);
                        let target = LoopRef(self.loop_count);
                        self.loop_count += 1;
                        self.loop_stack.push(target);
                        let prologue = vec![
                            RStmt::Let {
                                local: lo_l,
                                value: lo,
                                span: *span,
                            },
                            RStmt::Let {
                                local: hi_l,
                                value: hi,
                                span: *span,
                            },
                            RStmt::Let {
                                local: i_l,
                                value: ResolvedExpr {
                                    span: *span,
                                    vt: VT::I32,
                                    kind: RExprKind::ReadLocal(lo_l),
                                },
                                span: *span,
                            },
                        ];
                        self.scopes.push(Scope {
                            names: BTreeMap::new(),
                            views: BTreeMap::new(),
                            text_mut_aliases: BTreeMap::new(),
                        });
                        self.scopes
                            .last_mut()
                            .unwrap()
                            .names
                            .insert(var.name.clone(), i_l);
                        self.declared_names.insert(var.name.clone());
                        if self.profile.is_aggregates() {
                            self.ctrl_frames.push(CtrlFrame::Loop { target });
                        }
                        let read_i = |sp: Span| ResolvedExpr {
                            span: sp,
                            vt: VT::I32,
                            kind: RExprKind::ReadLocal(i_l),
                        };
                        let incr = RStmt::Assign {
                            local: i_l,
                            path: Vec::new(),
                            value: ResolvedExpr {
                                span: *span,
                                vt: VT::I32,
                                kind: RExprKind::Binary {
                                    op: ast::BinOp::Add,
                                    lhs: Box::new(read_i(*span)),
                                    rhs: Box::new(ResolvedExpr {
                                        span: *span,
                                        vt: VT::I32,
                                        kind: RExprKind::IntLit(1),
                                    }),
                                },
                            },
                            span: *span,
                        };
                        // PLAN-1036 (F29): the half-open step can never
                        // overflow (i < hi <= i32::MAX => i + 1 <= MAX); the
                        // inclusive step is guarded by the endpoint so the
                        // LAST member ends the loop instead of synthesizing
                        // `hi + 1` (a body-independent overflow).
                        let latch: Vec<RStmt> = if *inclusive {
                            vec![RStmt::If {
                                cond: ResolvedExpr {
                                    span: *span,
                                    vt: VT::Bool,
                                    kind: RExprKind::Binary {
                                        op: ast::BinOp::Eq,
                                        lhs: Box::new(read_i(*span)),
                                        rhs: Box::new(ResolvedExpr {
                                            span: *span,
                                            vt: VT::I32,
                                            kind: RExprKind::ReadLocal(hi_l),
                                        }),
                                    },
                                },
                                then: RBlock {
                                    stmts: vec![RStmt::Break {
                                        target: Some(target),
                                        span: *span,
                                    }],
                                    span: *span,
                                },
                                els: Some(RBlock {
                                    stmts: vec![incr],
                                    span: *span,
                                }),
                                span: *span,
                            }]
                        } else {
                            vec![incr]
                        };
                        // PLAN-1036 (F28): a `continue` inside the body must
                        // run this latch before the header re-tests `i <= hi`.
                        self.for_latches.push((target, latch.clone()));
                        let mut b = self.resolve_body(body, false);
                        self.for_latches.pop();
                        if self.profile.is_aggregates() {
                            self.ctrl_frames.pop();
                        }
                        self.scopes.pop();
                        self.loop_stack.pop();
                        // The fall-through back edge runs the same latch.
                        if rblock_can_fall_through(&b) {
                            b.stmts.extend(latch);
                        }
                        let cond = ResolvedExpr {
                            span: *span,
                            vt: VT::Bool,
                            kind: RExprKind::Binary {
                                op: if *inclusive {
                                    ast::BinOp::Le
                                } else {
                                    ast::BinOp::Lt
                                },
                                lhs: Box::new(read_i(*span)),
                                rhs: Box::new(ResolvedExpr {
                                    span: *span,
                                    vt: VT::I32,
                                    kind: RExprKind::ReadLocal(hi_l),
                                }),
                            },
                        };
                        let if_stmt = RStmt::If {
                            cond,
                            then: RBlock {
                                stmts: std::mem::take(&mut b.stmts),
                                span: body.span,
                            },
                            els: Some(RBlock {
                                stmts: vec![RStmt::Break {
                                    target: Some(target),
                                    span: *span,
                                }],
                                span: *span,
                            }),
                            span: *span,
                        };
                        let mut prologue = prologue;
                        prologue.push(RStmt::Loop {
                            target,
                            unwind: None,
                            body: RBlock {
                                stmts: vec![if_stmt],
                                span: body.span,
                            },
                            span: *span,
                        });
                        if self.profile.is_aggregates() && self.has_outer_ctrl_barrier() {
                            self.emit_post_frame_dispatch(*span, &mut prologue);
                        }
                        stack
                            .last_mut()
                            .expect("resolve frame")
                            .stmts
                            .extend(prologue);
                        continue;
                    }
                    let is_list_var = matches!(&iter.kind, ast::ExprKind::Var(id) if matches!(self.lookup(&id.name).map(|l| self.locals[l.0].vt), Some(VT::List(_))));
                    if self.profile.is_aggregates() && !is_list_var {
                        let old_site = self.scoped_iter_site;
                        self.scoped_iter_site =
                            matches!(&iter.kind, ast::ExprKind::Call { .. }).then_some(iter.span);
                        let iter_val = self.expr_iterative(iter);
                        self.scoped_iter_site = old_site;
                        if self.coll.script_errors && iter_val.vt == VT::Nil {
                            self.eval_for_effect(iter_val);
                            self.script_type_error(
                                "TypeError: 'NoneType' object is not iterable".to_string(),
                                VT::Unit,
                                *span,
                            );
                            let frame = stack.last_mut().expect("resolve frame");
                            frame.idx = frame.src.stmts.len();
                            continue;
                        }
                        if let VT::Rec(ri) = iter_val.vt {
                            if self.iter_info.contains_key(&ri) {
                                let stmts =
                                    self.resolve_iter_for_in(var, iter_val, ri, body, *span);
                                stack.last_mut().expect("resolve frame").stmts.extend(stmts);
                                continue;
                            }
                        }
                        self.bind_err(
                            "source.iterate-scrutinee",
                            format!(
                                "`for` requires a Range, List, or Iter<T>, found `{}`",
                                iter_val.vt.as_str()
                            ),
                            iter.span,
                        );
                        continue;
                    }
                    let ast::ExprKind::Var(id) = &iter.kind else {
                        self.bind_err(
                            "source.iterate-scrutinee",
                            "for-in iterates a List local in this profile".to_string(),
                            iter.span,
                        );
                        continue;
                    };
                    let Some(xsl) = self.lookup(&id.name) else {
                        self.bind_err(
                            "source.undefined",
                            format!("no local named `{}` in scope", id.name),
                            id.span,
                        );
                        continue;
                    };
                    let VT::List(li) = self.locals[xsl.0].vt else {
                        self.bind_err(
                            "source.iterate-scrutinee",
                            format!(
                                "`for x in {}` requires a List-typed local, found `{}`",
                                id.name,
                                self.locals[xsl.0].vt.as_str()
                            ),
                            iter.span,
                        );
                        continue;
                    };
                    let elem = self.coll.lists[li];
                    let owned = self.vt_is_coll_owned(elem);
                    let i_lref = self.fresh_local("__for_i", VT::I32, true);
                    let n_lref = self.fresh_local("__for_n", VT::I32, false);
                    let target = LoopRef(self.loop_count);
                    self.loop_count += 1;
                    self.loop_stack.push(target);
                    if self.profile.is_aggregates() {
                        self.ctrl_frames.push(CtrlFrame::Loop { target });
                    }
                    self.iterate_targets.push((xsl, elem, *span));
                    let zero = ResolvedExpr {
                        span: *span,
                        vt: VT::I32,
                        kind: RExprKind::IntLit(0),
                    };
                    let xs_read = ResolvedExpr {
                        span: iter.span,
                        vt: VT::List(li),
                        kind: RExprKind::ReadLocal(xsl),
                    };
                    let n_read = ResolvedExpr {
                        span: *span,
                        vt: VT::I32,
                        kind: RExprKind::ReadLocal(n_lref),
                    };
                    let len_expr = ResolvedExpr {
                        span: iter.span,
                        vt: VT::I32,
                        kind: RExprKind::ListLen {
                            list: Box::new(xs_read),
                        },
                    };
                    let mut prologue: Vec<RStmt> = vec![
                        RStmt::Let {
                            local: i_lref,
                            value: zero,
                            span: *span,
                        },
                        RStmt::Let {
                            local: n_lref,
                            value: len_expr,
                            span: *span,
                        },
                    ];
                    // PLAN-1036 (F28): the List latch is the hidden index
                    // advance; `i < n` bounds it, so the step cannot
                    // overflow (i <= n - 1 <= MAX).
                    let one = ResolvedExpr {
                        span: *span,
                        vt: VT::I32,
                        kind: RExprKind::IntLit(1),
                    };
                    let incr = RStmt::Assign {
                        local: i_lref,
                        path: Vec::new(),
                        value: ResolvedExpr {
                            span: *span,
                            vt: VT::I32,
                            kind: RExprKind::Binary {
                                op: ast::BinOp::Add,
                                lhs: Box::new(ResolvedExpr {
                                    span: *span,
                                    vt: VT::I32,
                                    kind: RExprKind::ReadLocal(i_lref),
                                }),
                                rhs: Box::new(one),
                            },
                        },
                        span: *span,
                    };
                    let latch: Vec<RStmt> = vec![incr];
                    self.for_latches.push((target, latch.clone()));
                    let body_res = if owned {
                        self.scopes.push(Scope {
                            names: BTreeMap::new(),
                            views: BTreeMap::new(),
                            text_mut_aliases: BTreeMap::new(),
                        });
                        self.scopes.last_mut().unwrap().views.insert(
                            var.name.clone(),
                            ViewBinding {
                                list: xsl,
                                index: i_lref,
                                elem,
                            },
                        );
                        self.declared_names.insert(var.name.clone());
                        let b = self.resolve_body(body, false);
                        self.scopes.pop();
                        b
                    } else {
                        self.scopes.push(Scope {
                            names: BTreeMap::new(),
                            views: BTreeMap::new(),
                            text_mut_aliases: BTreeMap::new(),
                        });
                        let blref = self.fresh_local(&var.name, elem, false);
                        self.scopes
                            .last_mut()
                            .unwrap()
                            .names
                            .insert(var.name.clone(), blref);
                        self.declared_names.insert(var.name.clone());
                        let mut b = self.resolve_body(body, false);
                        self.scopes.pop();
                        let get = ResolvedExpr {
                            span: iter.span,
                            vt: elem,
                            kind: RExprKind::ListGet {
                                list: Box::new(ResolvedExpr {
                                    span: iter.span,
                                    vt: VT::List(li),
                                    kind: RExprKind::ReadLocal(xsl),
                                }),
                                index: Box::new(ResolvedExpr {
                                    span: *span,
                                    vt: VT::I32,
                                    kind: RExprKind::ReadLocal(i_lref),
                                }),
                            },
                        };
                        b.stmts.insert(
                            0,
                            RStmt::Let {
                                local: blref,
                                value: get,
                                span: *span,
                            },
                        );
                        b
                    };
                    self.iterate_targets.pop();
                    self.loop_stack.pop();
                    self.for_latches.pop();
                    if self.profile.is_aggregates() {
                        self.ctrl_frames.pop();
                    }
                    let mut body_res = body_res;
                    let falls = rblock_can_fall_through(&body_res);
                    let mut then_stmts = std::mem::take(&mut body_res.stmts);
                    // The fall-through back edge runs the latch (a body that
                    // already terminates carries it on its own `continue`).
                    if falls {
                        then_stmts.extend(latch);
                    }
                    let cond = ResolvedExpr {
                        span: *span,
                        vt: VT::Bool,
                        kind: RExprKind::Binary {
                            op: ast::BinOp::Lt,
                            lhs: Box::new(ResolvedExpr {
                                span: *span,
                                vt: VT::I32,
                                kind: RExprKind::ReadLocal(i_lref),
                            }),
                            rhs: Box::new(n_read),
                        },
                    };
                    let else_block = RBlock {
                        stmts: vec![RStmt::Break {
                            target: Some(target),
                            span: *span,
                        }],
                        span: *span,
                    };
                    let if_stmt = RStmt::If {
                        cond,
                        then: RBlock {
                            stmts: then_stmts,
                            span: body.span,
                        },
                        els: Some(else_block),
                        span: *span,
                    };
                    prologue.push(RStmt::Loop {
                        body: RBlock {
                            stmts: vec![if_stmt],
                            span: body.span,
                        },
                        span: *span,
                        target,
                        unwind: None,
                    });
                    if self.profile.is_aggregates() && self.has_outer_ctrl_barrier() {
                        self.emit_post_frame_dispatch(*span, &mut prologue);
                    }
                    stack
                        .last_mut()
                        .expect("resolve frame")
                        .stmts
                        .extend(prologue);
                }
            }
        }
    }

    fn expr(&mut self, e: &Expr) -> ResolvedExpr {
        match &e.kind {
            // PLAN-968: `await`/`go` are desugared into `__sched_*`
            // intrinsic calls BEFORE resolve (sched_desugar); a stray node
            // here means a non-scheduler face carried one through — named
            // rejection, never a silent drop.
            ExprKind::Await(inner) => {
                if self.profile.is_auto_language() {
                    let value = self.expr_iterative(inner);
                    return self.scheduler_operation(
                        "__sched_await",
                        vec![ResolvedArg {
                            span: inner.span,
                            value,
                            moved: false,
                            name: None,
                        }],
                        e.span,
                    );
                }
                let _ = self.expr(inner);
                self.bind_err(
                    "source.sched-unsupported",
                    "`await` is not part of this source profile",
                    e.span,
                );
                ResolvedExpr {
                    span: e.span,
                    vt: VT::I32,
                    kind: RExprKind::Unit,
                }
            }
            ExprKind::Go(inner) => {
                if self.profile.is_auto_language() {
                    let direct = match &inner.kind {
                        ExprKind::Call { callee, args }
                            if self
                                .sigs
                                .get(&callee.name)
                                .is_some_and(|sig| !matches!(sig.result, VT::Deferred(_))) =>
                        {
                            Some((callee, args))
                        }
                        _ => None,
                    };
                    if let Some((callee, args)) = direct {
                        let mut operands = vec![Arg::Pos(Expr {
                            kind: ExprKind::StrLit(callee.name.clone()),
                            span: callee.span,
                        })];
                        operands.extend_from_slice(args);
                        return self.call(
                            &ast::Ident {
                                name: "__sched_go".into(),
                                span: e.span,
                            },
                            &operands,
                            e.span,
                        );
                    }
                    let value = self.expr_iterative(inner);
                    return self.scheduler_operation(
                        "__sched_detach",
                        vec![ResolvedArg {
                            value,
                            name: None,
                            moved: false,
                            span: inner.span,
                        }],
                        e.span,
                    );
                }
                let _ = self.expr(inner);
                self.bind_err(
                    "source.sched-unsupported",
                    "`go` is not part of this source profile",
                    e.span,
                );
                ResolvedExpr {
                    span: e.span,
                    vt: VT::Unit,
                    kind: RExprKind::Unit,
                }
            }
            ExprKind::Propagate { inner } => {
                if self.profile.is_aggregates() {
                    self.resolve_propagate(inner, e.span)
                } else {
                    self.bind_err(
                        "source.unsupported",
                        "error propagation / is-expr requires full aggregates lowering".to_string(),
                        e.span,
                    );
                    ResolvedExpr {
                        span: e.span,
                        vt: VT::I32,
                        kind: RExprKind::IntLit(0),
                    }
                }
            }
            ExprKind::UnwrapOr { inner, fallback } => {
                if self.profile.is_aggregates() {
                    self.resolve_unwrap_or(inner, fallback, None, e.span)
                } else {
                    self.bind_err(
                        "source.unsupported",
                        "error propagation / is-expr requires full aggregates lowering".to_string(),
                        e.span,
                    );
                    ResolvedExpr {
                        span: e.span,
                        vt: VT::I32,
                        kind: RExprKind::IntLit(0),
                    }
                }
            }
            ExprKind::IsExpr { scrutinee, arms } => {
                if self.profile.is_aggregates() {
                    let (stmts, expr_opt) =
                        self.resolve_agg_is(scrutinee, arms, None, true, e.span, false);
                    for s in stmts {
                        self.emit_stmt(s);
                    }
                    expr_opt.unwrap_or(ResolvedExpr {
                        span: e.span,
                        vt: VT::Unit,
                        kind: RExprKind::Unit,
                    })
                } else {
                    self.bind_err(
                        "source.unsupported",
                        "error propagation / is-expr requires full aggregates lowering".to_string(),
                        e.span,
                    );
                    ResolvedExpr {
                        span: e.span,
                        vt: VT::I32,
                        kind: RExprKind::IntLit(0),
                    }
                }
            }
            // PLAN-962 (L-15): a tuple LITERAL resolves positionally —
            // the elements' VTs intern the structural shape; materializ-
            // ation (anonymous positional record) is the adapter's.
            ExprKind::TupleLit(elems_ast) => {
                let mut elems = Vec::with_capacity(elems_ast.len());
                for e in elems_ast {
                    // elements resolve through the iterative walker (they
                    // may be operator chains).
                    elems.push(self.expr_iterative(e));
                }
                let vts: Vec<VT> = elems.iter().map(|e| e.vt).collect();
                let span = e.span;
                ResolvedExpr {
                    span,
                    vt: VT::Tuple(self.coll.intern_tuple(vts)),
                    kind: RExprKind::TupleLit { elems },
                }
            }
            // PLAN-962 (T-08, L-14): a grid literal desugars ONLY in
            // the let/var initializer position (the Let arm intercepts).
            ExprKind::GridLit { .. } => {
                self.bind_err(
                    "source.grid-position",
                    "a grid literal is only a let/var initializer in this profile (bind it to a name, then read the columns)"
                        .to_string(),
                    e.span,
                );
                ResolvedExpr {
                    span: e.span,
                    vt: VT::I32,
                    kind: RExprKind::IntLit(0),
                }
            }
            // PLAN-962 (L-16) → PLAN-1082 (D-1082-04): array literals are
            // admitted in ANY rvalue position — the ListNew+push chain
            // hoists into the statement sink (pending_splice keeps source
            // order) and the value is the temp read. The let/var initializer
            // fast path (the Let arm intercepts first) is unchanged. An
            // EMPTY literal keeps its named rejection: no element type is
            // inferable on this face (write `List<T>.new()`).
            ExprKind::ArrayLit(elems) => self.resolve_array_literal(elems, e.span, false, None),
            ExprKind::ArrayConditionalElem { .. } => {
                self.bind_err(
                    "source.array-element-position",
                    "a conditional element needs an enclosing list literal",
                    e.span,
                );
                self.dummy_of(VT::I32, e.span)
            }
            // PLAN-963（T-04）：`rep<2>(21)` 必经实例层重绑成具体实例的普通
            // 调用；到达 resolve = 模板未被实例化（具名拒，不猜）。
            ExprKind::ConstCall { callee, .. } => {
                self.bind_err(
                    "source.instances-unsupported",
                    format!(
                        "value-argument call `{}<...>(..)` must be instantiated before resolve",
                        callee.name
                    ),
                    e.span,
                );
                ResolvedExpr {
                    span: e.span,
                    vt: VT::I32,
                    kind: RExprKind::IntLit(0),
                }
            }
            // PLAN-962 (L-48): an index READ `xs[i]` — the same element
            // path `.get(i)` resolves through (value element = ListGet;
            // owned element = the tainted whole-element view). PLAN-964:
            // an object receiver `obj[k]` reads by Str key (ObjectGet).
            ExprKind::Index { base, index } => {
                // A literal key on a named record uses the same path and
                // borrow rules as its dot spelling, including loop views.
                if self.coll.age_package {
                    if let (ExprKind::Var(receiver), ExprKind::StrLit(key)) =
                        (&base.kind, &index.kind)
                    {
                        let receiver_type = self.lookup_view(&receiver.name).map(|view| view.elem)
                            .or_else(|| self.lookup(&receiver.name).map(|local| self.locals[local.0].vt));
                        if let Some(VT::Rec(ri)) = receiver_type {
                            let ty = &self.records[ri];
                            if self.coll.legacy_age_widget_at(e.span)
                                && super::age_package::widget_lists::is_sparse_record(&ty.name)
                                && !ty.fields.iter().any(|field| field.name == *key)
                            {
                                // The old widget map read supplies zero for
                                // a key absent from the entire literal shape.
                                return ResolvedExpr {
                                    span: e.span,
                                    vt: VT::I32,
                                    kind: RExprKind::IntLit(0),
                                };
                            }
                            return self.expr_iterative(&Expr {
                                span: e.span,
                                kind: ExprKind::Field {
                                    base: base.clone(),
                                    name: ast::Ident { name: key.clone(), span: index.span },
                                },
                            });
                        }
                    }
                }
                let recv = self.expr_iterative(base);
                let idx = self.expr_iterative(index);
                if self.coll.script_errors && recv.vt == VT::Nil {
                    self.eval_for_effect(recv);
                    self.eval_for_effect(idx);
                    return self.script_type_error(
                        "TypeError: 'NoneType' object is not subscriptable".to_string(),
                        VT::I32,
                        e.span,
                    );
                }
                // PLAN-1026: integer indexing on a Python object —
                // py_getitem(receiver, index); the result is a Python
                // object.
                if self.py_is_object(&recv) {
                    let mut parts: Vec<PyFramePart> =
                        vec![PyFramePart::Txt("p967.frame/1\nr ".to_string())];
                    let chunk = self.py_chunk_call("o", &recv);
                    parts.push(PyFramePart::TextExpr(chunk));
                    parts.push(PyFramePart::Txt("p 1\n".to_string()));
                    self.py_frame_val(&mut parts, &idx);
                    parts.push(PyFramePart::Txt("k 0\n".to_string()));
                    let frame = self.py_frame(e.span, parts);
                    // PLAN-1107 T-02: index operation fact (receiver + slot 0).
                    if crate::py_ops::recording_active() {
                        crate::py_ops::record(
                            PyOpFact::new(PyOpKind::GetItem, e.span)
                                .operand_lit(
                                    PyRole::Receiver,
                                    recv.span,
                                    self.py_val_class(&recv),
                                    self.py_operand_literal(&recv),
                                )
                                .operand_lit(
                                    PyRole::Positional(0),
                                    idx.span,
                                    self.py_val_class(&idx),
                                    self.py_operand_literal(&idx),
                                ),
                        );
                    }
                    return self.py_op_call("py_getitem", e.span, frame);
                }
                match recv.vt {
                    VT::List(li) => {
                        if idx.vt != VT::I32 {
                            self.bind_err(
                                "source.index-type",
                                "an index must be i32 in this profile".to_string(),
                                index.span,
                            );
                        }
                        let elem = self.coll.lists[li];
                        if self.vt_is_coll_owned(elem) {
                            if !self.elem_view_admission() {
                                self.bind_err(
                                    "source.view-escape",
                                    "a whole owned element is not a value; project a field (`xs[i].f`) or use `xs.remove(i)`".to_string(),
                                    e.span,
                                );
                            }
                            ResolvedExpr {
                                span: e.span,
                                vt: elem,
                                kind: RExprKind::ElemView {
                                    list: Box::new(recv),
                                    index: Box::new(idx),
                                    path: Vec::new(),
                                },
                            }
                        } else {
                            ResolvedExpr {
                                span: e.span,
                                vt: elem,
                                kind: RExprKind::ListGet {
                                    list: Box::new(recv),
                                    index: Box::new(idx),
                                },
                            }
                        }
                    }
                    VT::Dyn => {
                        self.check_object_key(&idx);
                        Self::object_get(recv, idx, VT::Dyn, e.span)
                    }
                    // PLAN-964 (L-20): object dynamic-key read.
                    VT::Object(oi) => {
                        self.check_object_key(&idx);
                        Self::object_get(recv, idx, self.coll.objects[oi], e.span)
                    }
                    other => {
                        self.bind_err(
                            "source.index-base",
                            format!(
                                "indexing requires a List receiver, found `{}`",
                                other.as_str()
                            ),
                            base.span,
                        );
                        ResolvedExpr {
                            span: e.span,
                            vt: VT::I32,
                            kind: RExprKind::IntLit(0),
                        }
                    }
                }
            }
            // PLAN-964 (L-16/L-20): object literal `{ k: v, ... }` —
            // keys static, values homogeneous (elem = the common VT;
            // mixed fields are a named reject). Empty `{}` is an empty
            // object (elem must still be pinned — freeze: `{}` requires a
            // `nil` elem and is admitted as an empty map).
            ExprKind::ObjectLit { fields } => {
                if fields.is_empty() {
                    self.bind_err(
                        "source.object-empty",
                        "an empty object literal `{}` is not in this profile (bind an empty map from a typed source)".to_string(),
                        e.span,
                    );
                    return ResolvedExpr {
                        span: e.span,
                        vt: VT::I32,
                        kind: RExprKind::IntLit(0),
                    };
                }
                let mut args = Vec::with_capacity(fields.len());
                let mut keys = Vec::with_capacity(fields.len());
                let ordinary = self.profile == SourceProfile::Auto;
                let mut elem = ordinary.then_some(VT::Dyn);
                let mut dup = std::collections::HashSet::new();
                for f in fields {
                    let value = self.expr_iterative(&f.value);
                    if !dup.insert(&f.key) {
                        self.bind_err(
                            "source.object-duplicate",
                            format!("duplicate object key {:?}", f.key),
                            f.span,
                        );
                    }
                    let value = if ordinary {
                        Self::coerce_dynamic(value, VT::Dyn)
                    } else {
                        value
                    };
                    match elem {
                        None => elem = Some(value.vt),
                        Some(e0) if e0 == value.vt => {}
                        Some(e0) => {
                            self.bind_err(
                                "source.object-elem",
                                format!(
                                    "object fields must share one element type (`{}` vs `{}`)",
                                    e0.as_str(),
                                    value.vt.as_str()
                                ),
                                f.span,
                            );
                        }
                    }
                    keys.push(f.key.clone());
                    args.push(value);
                }
                let elem = elem.unwrap_or(VT::Nil);
                let oi = self.coll.intern_object(elem);
                ResolvedExpr {
                    span: e.span,
                    vt: VT::Object(oi),
                    kind: RExprKind::ObjectCtor { keys, args },
                }
            }
            // PLAN-964 (L-21): the nil literal.
            ExprKind::NilLit => ResolvedExpr {
                span: e.span,
                vt: VT::Nil,
                kind: RExprKind::NilLit,
            },
            // PLAN-964 (L-22/FStr): i32 display — the transform lowers
            // `f"... ${x} ..."` into `__fstr(...)` calls; this node is
            // produced by the `__fstr` builtin resolver (see below).
            ExprKind::TextDisplay(inner) => {
                let op = self.expr_iterative(inner);
                if op.vt != VT::I32 {
                    self.bind_err(
                        "source.fstr-display",
                        "FStr interpolation displays `int` values in this profile (other display kinds belong to the format surface)"
                            .to_string(),
                        e.span,
                    );
                }
                ResolvedExpr {
                    span: e.span,
                    vt: VT::StrOwned,
                    kind: RExprKind::TextDisplay {
                        operand: Box::new(op),
                    },
                }
            }
            // PLAN-962 (L-19): a range is legal ONLY as the for-in
            // iterable (the ForIn arm consumes it before this walk); a
            // range value anywhere else is a named rejection.
            ExprKind::Range { .. } => {
                self.bind_err(
                    "source.range-position",
                    "a range (`a..b`) is only the iterable of a `for` loop in this profile"
                        .to_string(),
                    e.span,
                );
                ResolvedExpr {
                    span: e.span,
                    vt: VT::I32,
                    kind: RExprKind::IntLit(0),
                }
            }
            // PLAN-936（T-04）：`#{ expr }` 是编译期需求，不是运行时表达式。
            // 绑定：CT 能力门（前门已按能力面解析；resolve 复核使携带
            // HashExpr 的 AST 不能在无能力车道绑定）→ inner 在 CT 上下文
            // 内完整绑定（含 MetaQuery 节点）→ HashExpr 类型 = inner 类型。
            // 未服务 HashExpr 由 typecheck 走处受控拒（source.cte.unserved）；
            // 合法消费：CT 车道以冻结常量结构性替换本节点（T-06/T-07）。
            ExprKind::HashExpr { expr: inner } => {
                if !self.ct_enabled {
                    self.bind_err(
                        "source.cte.capability",
                        "a `#{ ... }` compile-time demand requires the ac.source.cte.scalar/v1 capability (this bind lane was not granted it)",
                        e.span,
                    );
                    return ResolvedExpr {
                        span: e.span,
                        vt: VT::I32,
                        kind: RExprKind::IntLit(0),
                    };
                }
                self.ct_depth += 1;
                let r = self.expr_iterative(inner);
                self.ct_depth -= 1;
                let vt = r.vt;
                ResolvedExpr {
                    span: e.span,
                    vt,
                    kind: RExprKind::HashExpr { inner: Box::new(r) },
                }
            }
            // PLAN-864: generic receivers ride the iterative machine only.
            ExprKind::GenericType { name, .. } => {
                self.bind_err(
                    "source.unsupported",
                    format!("receiver `{}` is not a value in this position", name.name),
                    name.span,
                );
                ResolvedExpr {
                    span: e.span,
                    vt: VT::I32,
                    kind: RExprKind::IntLit(0),
                }
            }
            ExprKind::Closure {
                move_capture,
                params,
                result,
                body,
            } => self.resolve_closure(*move_capture, params, result.as_ref(), body, e.span),
            ExprKind::CallValue { .. } => {
                self.bind_err(
                    "source.unsupported",
                    "indirect closure calls have not passed closure lowering in this source profile",
                    e.span,
                );
                ResolvedExpr {
                    span: e.span,
                    vt: VT::I32,
                    kind: RExprKind::IntLit(0),
                }
            }
            ExprKind::IntLit(v) => {
                let vt = self.node_ty(e.span, VT::I32);
                ResolvedExpr {
                    span: e.span,
                    vt,
                    kind: RExprKind::IntLit(*v),
                }
            }
            // PLAN-961: the typed numeric literal binds with its payload
            // type (bits already folded/validated at the projection).
            ExprKind::TypedLit { ty, bits } => {
                let vt = self.node_ty(e.span, vt_of_word(*ty));
                ResolvedExpr {
                    span: e.span,
                    vt,
                    kind: RExprKind::TypedLit { vt, bits: *bits },
                }
            }
            ExprKind::BoolLit(v) => {
                let vt = self.node_ty(e.span, VT::Bool);
                ResolvedExpr {
                    span: e.span,
                    vt,
                    kind: RExprKind::BoolLit(*v),
                }
            }
            ExprKind::Var(id) => match self.lookup(&id.name) {
                Some(l) => {
                    if self.moved_locals.contains(&l) {
                        self.bind_err(
                            "source.closure-use-after-move",
                            format!(
                                "local `{}` was moved into a closure and cannot be used",
                                id.name
                            ),
                            id.span,
                        );
                    }
                    if self.borrowed_closure_locals.contains(&l) {
                        self.bind_err(
                            "source.closure-borrow-escape",
                            format!(
                                "borrow-capturing closure `{}` may only be called directly",
                                id.name
                            ),
                            id.span,
                        );
                    }
                    let rule = self.locals[l.0].vt;
                    let vt = self.node_ty(e.span, rule);
                    ResolvedExpr {
                        span: e.span,
                        vt,
                        kind: RExprKind::ReadLocal(l),
                    }
                }
                None => {
                    if self.lookup_text_mut_alias(&id.name).is_some() {
                        self.bind_err(
                            "source.view-escape",
                            "a mutable text binding cannot be used as a value",
                            id.span,
                        );
                        return ResolvedExpr {
                            span: e.span,
                            vt: VT::Unit,
                            kind: RExprKind::Unit,
                        };
                    }
                    // PLAN-939 S06 callback-by-name: on the text-record
                    // resource lane a bare SOURCE fn name whose (params,
                    // result) exactly match an interned closure type is a
                    // named callback TARGET (never a function value — the
                    // typecheck confines it to callback lane arguments by
                    // type; a manifest without a callback lane interns no
                    // matching closure type and names stay non-values).
                    if self.profile == SourceProfile::TextRecord {
                        if let Some(sig) = self
                            .sigs
                            .get(&id.name)
                            .filter(|s| s.index < self.source_fn_count)
                        {
                            let want = ClosureFnType {
                                params: sig.params.clone(),
                                result: sig.result,
                            };
                            if let Some(idx) =
                                self.coll.function_types.iter().position(|t| *t == want)
                            {
                                return ResolvedExpr {
                                    span: e.span,
                                    vt: self.node_ty(e.span, VT::Function(idx)),
                                    kind: RExprKind::CallbackRef {
                                        fn_index: sig.index,
                                    },
                                };
                            }
                        }
                    }
                    if self.profile.is_aggregates()
                        && id.name == "None"
                        && self.ctor_name_is_free("None")
                    {
                        return self.resolve_bare_ctor("None", None, None, e.span);
                    }
                    if self.nested.scope.contains_key(&id.name) {
                        // PLAN-1081 (T-03): nested fn items are call-only
                        // this batch (no value lane, frozen rule).
                        self.bind_err(
                            "source.fn-as-value",
                            format!(
                                "nested fn `{}` cannot be used as a value in this profile",
                                id.name
                            ),
                            id.span,
                        );
                    } else if let Some(sig) = self.sigs.get(&id.name).cloned() {
                        // PLAN-1081 (T-02): on the Auto face a FREE source
                        // fn item in value position is a checked FUNCTION
                        // VALUE — a zero-capture closure whose callee is the
                        // named item itself. Signature/ABI/target identity
                        // come from the item's own params/result (interned
                        // as its ClosureFnType); no capture analysis runs,
                        // so recursion and mutual recursion carry no
                        // locals. Bare-name sigs are free items (methods
                        // key as `Type::method`); `index < source_fn_count`
                        // excludes foreign/py slots and global pseudo
                        // entries. Old narrow profiles keep the frozen
                        // `source.fn-as-value` rejection below.
                        if self.profile == SourceProfile::Auto
                            && sig.index < self.source_fn_count
                            && id.name != "main"
                        {
                            let type_index = self.coll.intern_function(ClosureFnType {
                                params: sig.params.clone(),
                                result: sig.result,
                            });
                            return ResolvedExpr {
                                span: e.span,
                                vt: self.node_ty(e.span, VT::Function(type_index)),
                                kind: RExprKind::CreateClosure {
                                    callee: id.name.clone(),
                                    captures: Vec::new(),
                                },
                            };
                        }
                        if id.name == "main" {
                            // PLAN-1081: the entry owns the module's root
                            // frame; it is callable as the entry only.
                            self.bind_err(
                                "source.fn-as-value",
                                "the entry function `main` cannot be used as a value",
                                id.span,
                            );
                        } else {
                            self.bind_err(
                                "source.fn-as-value",
                                format!(
                                    "function `{}` cannot be used as a value in this profile",
                                    id.name
                                ),
                                id.span,
                            );
                        }
                    } else if let Some(vb) = self.lookup_view(&id.name) {
                        // PLAN-962: reading a for-in element VIEW binding
                        // yields the whole-element view (borrow) — legal as
                        // a method receiver (read-through); materializing
                        // uses reject at their sites.
                        let xs_vt = self.locals[vb.list.0].vt;
                        let vt = self.node_ty(e.span, vb.elem);
                        return ResolvedExpr {
                            span: e.span,
                            vt,
                            kind: RExprKind::ElemView {
                                list: Box::new(ResolvedExpr {
                                    span: e.span,
                                    vt: xs_vt,
                                    kind: RExprKind::ReadLocal(vb.list),
                                }),
                                index: Box::new(ResolvedExpr {
                                    span: e.span,
                                    vt: VT::I32,
                                    kind: RExprKind::ReadLocal(vb.index),
                                }),
                                path: Vec::new(),
                            },
                        };
                    } else if let Some(field) = self.implicit_record_field(id) {
                        return field;
                    } else if self.nested.no_capture.contains(&id.name) {
                        // PLAN-1081 (T-03): a nested fn body referencing an
                        // ENCLOSING local — Rust nested-fn semantics: the
                        // item is not a closure; the name cannot be
                        // captured. Named rejection, never a silent 0.
                        self.bind_err(
                            "source.nested-fn-no-capture",
                            format!(
                                "nested fn `{}` cannot capture outer local `{}`; nested fn items are not closures",
                                self.owner_name, id.name
                            ),
                            id.span,
                        );
                    } else if self.declared_names.contains(&id.name) {
                        // Declared in a scope that ended before this use
                        // (N09/C10): point at the name itself — the token
                        // that failed to resolve, per the §5.3 precision
                        // rule.
                        self.bind_err(
                            "source.undefined",
                            format!("local `{}` does not escape its declaring block", id.name),
                            id.span,
                        );
                    } else {
                        self.bind_err(
                            "source.undefined",
                            format!("no local named `{}` in scope", id.name),
                            id.span,
                        );
                    }
                    ResolvedExpr {
                        span: e.span,
                        vt: VT::I32,
                        kind: RExprKind::ReadLocal(LocalRef(usize::MAX)),
                    }
                }
            },
            ExprKind::StrLit(v) => {
                // PLAN-936（T-04）：非 text 面上字符串字面量的唯一合法位置是
                // ct_has_field 的字段选择子（meta 拦截在值解析前）——
                // 走到这里的都是散落使用，受控拒。
                if !self.profile.admits_text_literals() {
                    self.bind_err(
                        "source.unsupported",
                        "a string literal on this face is only legal as the field selector of `ct_has_field` inside `#{ ... }`",
                        e.span,
                    );
                    return ResolvedExpr {
                        span: e.span,
                        vt: VT::Str,
                        kind: RExprKind::StrLit(String::new()),
                    };
                }
                ResolvedExpr {
                    span: e.span,
                    // default context: a literal is a borrowing view (D-03);
                    // owning contexts retag it via `retag_literal`
                    vt: VT::Str,
                    kind: RExprKind::StrLit(v.clone()),
                }
            }
            ExprKind::CharLit(v) => ResolvedExpr {
                span: e.span,
                vt: VT::Char,
                kind: RExprKind::CharLit(*v),
            },
            ExprKind::TypeWordArg(w) => {
                // marker node: consumed by `.to(...)` and the type-level
                // `char.from_codepoint` receiver; a stray one is rejected by
                // the typecheck walk
                ResolvedExpr {
                    span: e.span,
                    vt: vt_of_word(*w),
                    kind: RExprKind::TypeWord(*w),
                }
            }
            // Method and Move nodes resolve through the iterative walk
            // (argument subtrees must not recurse); binary/unary likewise.
            ExprKind::Method { .. } => unreachable!("handled by expr_iterative"),
            ExprKind::Call { callee, args } => self.call(callee, args, e.span),
            // Binary nodes are resolved by the ITERATIVE walk below —
            // operator chains are unbounded in length and must not recurse
            // (R3-QA-02).
            ExprKind::Binary { .. } => unreachable!("handled by expr_iterative"),
            ExprKind::Unary { .. } => unreachable!("handled by expr_iterative"),
            // -- record profile (PLAN-771): Field/MethodCall carry a child
            // expression and are handled by the iterative walk like Unary;
            // the remaining kinds are true leaves here. --
            ExprKind::Field { .. } => unreachable!("handled by expr_iterative"),
            ExprKind::MethodCall { .. } => unreachable!("handled by expr_iterative"),
            ExprKind::CtorNamed { .. } => {
                unreachable!("named ctors resolve through the iterative walk")
            }
            // PLAN-962 (T-08, L-06): `x.view` — the readonly borrow
            // argument marker. A plain record parameter ALREADY borrows
            // (771 frozen semantics; the adapter wraps BorrowView by the
            // callee's param mode), so the marker resolves as the local
            // read with a record-type check — the intent is documented,
            // the semantics are the default view pass.
            ExprKind::ViewArg { name } => match self.lookup(&name.name) {
                Some(l) => {
                    let vt = self.locals[l.0].vt;
                    if self.profile.is_auto_language() && vt == VT::StrOwned {
                        return ResolvedExpr {
                            span: e.span,
                            vt: VT::Str,
                            kind: RExprKind::Method {
                                recv: Box::new(ResolvedExpr {
                                    span: name.span,
                                    vt,
                                    kind: RExprKind::ReadLocal(l),
                                }),
                                method: BuiltinMethod::To,
                                to: Some(VT::Str),
                                args: Vec::new(),
                            },
                        };
                    }
                    let readonly_value = self.profile.is_auto_language()
                        && matches!(vt, VT::I32 | VT::Bool | VT::Char | VT::Str);
                    if !matches!(vt, VT::Rec(_)) && !readonly_value {
                        self.bind_err(
                            "source.type-mismatch",
                            format!(
                                "`.view` is not available for local `{}` of type `{}`",
                                name.name,
                                vt.as_str()
                            ),
                            e.span,
                        );
                    }
                    ResolvedExpr {
                        span: e.span,
                        vt,
                        kind: RExprKind::ReadLocal(l),
                    }
                }
                None => {
                    self.bind_err(
                        "source.undefined",
                        format!("no local named `{}` in scope", name.name),
                        name.span,
                    );
                    ResolvedExpr {
                        span: e.span,
                        vt: VT::I32,
                        kind: RExprKind::ReadLocal(LocalRef(usize::MAX)),
                    }
                }
            },
            ExprKind::MutArg { name } => match self.lookup(&name.name) {
                Some(l) => {
                    let vt = self.locals[l.0].vt;
                    if !matches!(vt, VT::Rec(_)) {
                        self.bind_err(
                            "source.type-mismatch",
                            format!(
                                "`.mut` requires a record-typed local, `{}` is `{}`",
                                name.name,
                                vt.as_str()
                            ),
                            e.span,
                        );
                    }
                    ResolvedExpr {
                        span: e.span,
                        vt,
                        kind: RExprKind::BorrowMut { local: l },
                    }
                }
                None => {
                    self.bind_err(
                        "source.undefined",
                        format!("no local named `{}` in scope", name.name),
                        name.span,
                    );
                    ResolvedExpr {
                        span: e.span,
                        vt: VT::I32,
                        kind: RExprKind::ReadLocal(LocalRef(usize::MAX)),
                    }
                }
            },
        }
    }

    /// Positional construction `Pair(1, true)` (PLAN-771 §5.2; reached from
    /// `resolve_call` once the callee name resolves to a record — the
    /// parser cannot tell a ctor from a call without the symbol table).
    /// Argument order IS declaration order here, so eval order matches the
    /// written order trivially; arity and nominal field types are checked
    /// inline (mirroring the named form).
    fn ctor_positional(
        &mut self,
        ri: usize,
        ty: &ast::Ident,
        mut rargs: Vec<ResolvedArg>,
        span: Span,
    ) -> ResolvedExpr {
        if self.profile.is_auto_language()
            && !rargs.is_empty()
            && rargs.iter().all(|arg| arg.name.is_some())
        {
            let values = rargs
                .into_iter()
                .map(|arg| (arg.name.expect("named arguments"), arg.span, arg.value))
                .collect();
            return self.ctor_named_out(ty, values, span);
        }
        let desc = |t: VT, records: &[ResolvedRecord]| match t {
            VT::Rec(i) => match records.get(i) {
                Some(r) => format!("record `{}`", r.name),
                None => "record".to_string(),
            },
            other => other.as_str().to_string(),
        };
        if rargs.iter().any(|a| a.name.is_some()) {
            self.bind_err(
                "source.argument",
                "positional construction takes positional arguments only; named construction uses `Type { field: value }`".to_string(),
                span,
            );
        }
        let fields = &self.records[ri].fields;
        if rargs.len() != fields.len() {
            self.bind_err(
                "source.argument",
                format!(
                    "`{}` has {} fields; positional construction takes exactly that many arguments",
                    ty.name,
                    fields.len()
                ),
                span,
            );
        }
        for (i, a) in rargs.iter_mut().enumerate() {
            let Some(want) = fields.get(i).map(|f| f.vt) else {
                continue;
            };
            // PLAN-782: positional construction promotes a borrowing string
            // literal into an owning Str field exactly like the named form
            // (the owning context constructs, AP-4).
            if want == VT::StrOwned
                && a.value.vt == VT::Str
                && matches!(a.value.kind, RExprKind::StrLit(_))
            {
                a.value.vt = VT::StrOwned;
            }
            if a.value.vt != want {
                // nominal records name both sides (identical shapes still
                // mismatch — PLAN-771 §5.1)
                self.bind_err(
                    "source.type-mismatch",
                    format!(
                        "argument {} initializes field `{}` of `{}`, which has type {}, found {}",
                        i + 1,
                        fields[i].name,
                        ty.name,
                        desc(want, self.records),
                        desc(a.value.vt, self.records)
                    ),
                    a.value.span,
                );
            } else {
                self.copy_record_parameter(&mut a.value);
            }
        }
        ResolvedExpr {
            span,
            vt: VT::Rec(ri),
            kind: RExprKind::CtorPos {
                rec: ri,
                args: rargs,
            },
        }
    }

    /// Named construction `Pair { left: 1, right: true }` (PLAN-771):
    /// field values keep SOURCE order; names map to declaration fields
    /// (completeness/duplicates checked here, types in typecheck).
    /// F-771-R1/V-09: takes PRE-RESOLVED field values — the iterative walk
    /// resolves every field value as its own subtask, so a nested
    /// `T { r: T { r: ... } }` chain never recurses through
    /// ctor_named/expr_iterative frames.
    fn ctor_named_out(
        &mut self,
        ty: &ast::Ident,
        values: Vec<(ast::Ident, Span, ResolvedExpr)>,
        span: Span,
    ) -> ResolvedExpr {
        let Some(&ri) = self.rec_index.get(&ty.name) else {
            self.bind_err(
                "source.undefined",
                format!("no record type named `{}` in this module", ty.name),
                ty.span,
            );
            return ResolvedExpr {
                span,
                vt: VT::I32,
                kind: RExprKind::ReadLocal(LocalRef(usize::MAX)),
            };
        };
        let mut seen: BTreeSet<String> = BTreeSet::new();
        let mut resolved: Vec<(String, ResolvedExpr)> = Vec::new();
        for (name, fspan, mut v) in values {
            if !seen.insert(name.name.clone()) {
                self.bind_err(
                    "source.duplicate",
                    format!("field `{}` is initialized more than once", name.name),
                    name.span,
                );
            }
            match self.records[ri]
                .fields
                .iter()
                .position(|fl| fl.name == name.name)
            {
                Some(fi) => {
                    let want = self.records[ri].fields[fi].vt;
                    // PLAN-782: an owning Str field promotes a borrowing
                    // string literal exactly like `let s Str = "..."` does
                    // (the owning context constructs, AP-4).
                    if want == VT::StrOwned
                        && v.vt == VT::Str
                        && matches!(v.kind, RExprKind::StrLit(_))
                    {
                        v.vt = VT::StrOwned;
                    }
                    if want == VT::StrOwned && self.coll.legacy_age_widget_at(v.span) {
                        v = Self::clone_text_element(v);
                    }
                    if v.vt != want {
                        // nominal records name both sides (identical shapes
                        // still mismatch — PLAN-771 §5.1)
                        let desc = |t: VT| match t {
                            VT::Rec(i) => match self.records.get(i) {
                                Some(r) => format!("record `{}`", r.name),
                                None => "record".to_string(),
                            },
                            other => other.as_str().to_string(),
                        };
                        self.bind_err(
                            "source.type-mismatch",
                            format!(
                                "field `{}` of `{}` has type {}, found {}",
                                name.name,
                                ty.name,
                                desc(want),
                                desc(v.vt)
                            ),
                            fspan,
                        );
                    } else {
                        self.copy_record_parameter(&mut v);
                    }
                    resolved.push((name.name.clone(), v));
                }
                None => {
                    self.bind_err(
                        "source.undefined",
                        format!("record `{}` has no field named `{}`", ty.name, name.name),
                        name.span,
                    );
                }
            }
        }
        if seen.len() != self.records[ri].fields.len() {
            let missing: Vec<String> = self.records[ri]
                .fields
                .iter()
                .filter(|fl| !seen.contains(&fl.name))
                .map(|fl| fl.name.clone())
                .collect();
            self.bind_err(
                "source.argument",
                format!(
                    "construction of `{}` must initialize every field exactly once; missing: {}",
                    ty.name,
                    missing.join(", ")
                ),
                span,
            );
        }
        ResolvedExpr {
            span,
            vt: VT::Rec(ri),
            kind: RExprKind::CtorNamed {
                rec: ri,
                fields: resolved,
            },
        }
    }

    /// Method call `recv.name(args)` (PLAN-771): a bare record type name as
    /// the receiver selects a static method; a record-typed value selects
    /// an instance method (readonly receiver view).
    /// Method resolution from an already-resolved receiver (`None` = a
    /// pre-classified static call whose receiver was a bare type name).
    fn method_call_out(
        &mut self,
        recv: Option<Box<ResolvedExpr>>,
        static_rec: Option<usize>,
        name: &ast::Ident,
        mut rargs: Vec<ResolvedArg>,
        span: Span,
        explicit: Option<&ast::InstanceArguments>,
        expected: Option<VT>,
    ) -> ResolvedExpr {
        if let Some(pending) = self.probe_template_method(
            static_rec,
            recv.as_ref().map(|value| value.vt),
            name,
            &mut rargs,
            explicit,
            span,
            expected,
        ) {
            return pending;
        }
        if explicit.is_some() {
            self.bind_err(
                "source.instances-unsupported",
                format!(
                    "`{}` is not a visible method template on this receiver",
                    name.name
                ),
                span,
            );
            return instances::pending_expression(span);
        }
        if let Some(rrecv) = recv {
            if self.profile.is_auto_language() && matches!(rrecv.vt, VT::TaskRef(_)) {
                let operation = match name.name.as_str() {
                    "send" => "__sched_send",
                    "ask" => "__sched_ask",
                    "stop" => "__sched_stop",
                    _ => "",
                };
                if !operation.is_empty() {
                    rargs.insert(
                        0,
                        ResolvedArg {
                            span: rrecv.span,
                            value: *rrecv,
                            moved: false,
                            name: None,
                        },
                    );
                    return self.scheduler_operation(operation, rargs, span);
                }
            }
            // PLAN-782: on the combination surface the record dot postfix
            // claims method chains first, so a TEXT receiver's builtin
            // call (`t.text.len()`) arrives here — route it into the text
            // builtin table instead of rejecting.
            // PLAN-1083: enum instance methods — the receiver is a
            // variant-typed value (`Light.Red.is_red()`).
            if let VT::Variant(vi) = rrecv.vt {
                if self.coll.enum_instances.get(&vi).is_some_and(|instance| {
                    instance.unbound_methods.contains(&name.name)
                }) {
                    self.bind_err(
                        "unsolved-variable",
                        format!("method `{}` requires the complete enum type arguments", name.name),
                        name.span,
                    );
                    return instances::pending_expression(span);
                }
                if let Some(m) = self.coll.variants[vi]
                    .methods
                    .iter()
                    .find(|m| m.name == name.name)
                {
                    let fn_index = m.fn_index;
                    let sig_params = self.method_sig_params(fn_index, 1);
                    self.retag_method_args(&mut rargs, fn_index, 1);
                    self.check_args(
                        &name.name,
                        &self.method_sig_names(fn_index, 1),
                        &sig_params,
                        &rargs,
                        span,
                    );
                    let _ = &sig_params;
                    return ResolvedExpr {
                        span,
                        vt: self.method_sig_result(fn_index),
                        kind: RExprKind::MethodCall {
                            fn_index,
                            recv: Some(Box::new(*rrecv)),
                            args: rargs,
                        },
                    };
                }
            }
            if !matches!(rrecv.vt, VT::Rec(_)) {
                return self.resolve_method(name, *rrecv, rargs, None, span);
            }
            let VT::Rec(ri) = rrecv.vt else {
                self.bind_err(
                    "source.type-mismatch",
                    format!(
                        "method calls require a record-typed receiver, found `{}`",
                        rrecv.vt.as_str()
                    ),
                    rrecv.span,
                );
                return ResolvedExpr {
                    span,
                    vt: VT::I32,
                    kind: RExprKind::ReadLocal(LocalRef(usize::MAX)),
                };
            };
            if self.profile.is_aggregates()
                && self.iter_info.contains_key(&ri)
                && matches!(name.name.as_str(), "next" | "close")
            {
                if let Some(res) =
                    self.resolve_iter_method(*rrecv, ri, &name.name, rargs.len(), span)
                {
                    return res;
                }
                unreachable!("iter method checked above");
            }
            // PLAN-782: builtin deep clone binds by (record receiver,
            // "clone") on the COMBINATION surface only — the plain record
            // profile keeps its 771 face (`type X has no method named
            // clone`, N06). User `clone` methods on records are rejected
            // at declaration registration, so the builtin owns the name.
            if name.name == "clone" && rargs.is_empty() && self.profile == SourceProfile::TextRecord
            {
                return ResolvedExpr {
                    span,
                    vt: VT::Rec(ri),
                    kind: RExprKind::RecordClone {
                        base: Box::new(*rrecv),
                    },
                };
            }
            return self.instance_or_static(Some(rrecv), ri, name, rargs, span);
        }
        // static: no receiver expression was evaluated — the type name was
        // consumed at classification time and the classification carries
        // the OWNING record (PLAN-881: same-named statics on two records
        // dispatch to their own owner; the old across-all-records scan
        // cross-wired to the first declarer).
        let recs: Vec<usize> = match static_rec {
            Some(ri) => vec![ri],
            None => (0..self.records.len()).collect(),
        };
        for ri in recs {
            let rec = &self.records[ri];
            if let Some(m) = rec
                .methods
                .iter()
                .find(|m| m.name == name.name && m.is_static)
            {
                let fn_index = m.fn_index;
                let sig_params = self.method_sig_params(fn_index, 0);
                let sig_names = self.method_sig_names(fn_index, 0);
                let sig_defaults = self.method_sig_defaults(fn_index, 0);
                self.fill_default_args(&sig_names, &sig_params, &sig_defaults, &mut rargs, span);
                self.retag_method_args(&mut rargs, fn_index, 0);
                self.check_args(
                    &name.name,
                    &self.method_sig_names(fn_index, 0),
                    &sig_params,
                    &rargs,
                    span,
                );
                return ResolvedExpr {
                    span,
                    vt: self.method_sig_result(fn_index),
                    kind: RExprKind::MethodCall {
                        fn_index,
                        recv: None,
                        args: rargs,
                    },
                };
            }
        }
        if let Some(ri) = static_rec {
            if self.profile.is_auto_language()
                && name.name == "new"
                && !self.records[ri].methods.iter().any(|m| m.name == name.name)
            {
                let ty = ast::Ident {
                    name: self.records[ri].name.clone(),
                    span: name.span,
                };
                return self.ctor_positional(ri, &ty, rargs, span);
            }
        }
        // distinguish "no such method at all" from "exists but is an
        // instance method" (V-08: exact resolution diagnostics)
        if self.records.iter().any(|rec| {
            rec.methods
                .iter()
                .any(|m| m.name == name.name && !m.is_static)
        }) {
            self.bind_err(
                "source.not-callable",
                format!(
                    "`{}` is an instance method; call it on a record value, not the type name",
                    name.name
                ),
                name.span,
            );
        } else {
            self.bind_err(
                "source.undefined",
                format!("no static method named `{}` in this module", name.name),
                name.span,
            );
        }
        ResolvedExpr {
            span,
            vt: VT::I32,
            kind: RExprKind::ReadLocal(LocalRef(usize::MAX)),
        }
    }

    fn instance_or_static(
        &mut self,
        recv: Option<Box<ResolvedExpr>>,
        ri: usize,
        name: &ast::Ident,
        mut rargs: Vec<ResolvedArg>,
        span: Span,
    ) -> ResolvedExpr {
        match self.records[ri]
            .methods
            .iter()
            .find(|m| m.name == name.name)
        {
            Some(m) if !m.is_static => {
                let recv = recv.map(|mut recv| {
                    if m.is_mut {
                        if let RExprKind::ReadLocal(local) = recv.kind {
                            recv.kind = RExprKind::BorrowMut { local };
                        } else {
                            self.bind_err(
                                "source.mut-receiver-place",
                                "a mutable method requires a named receiver local",
                                recv.span,
                            );
                        }
                    }
                    recv
                });
                let sig_params = self.method_sig_params(m.fn_index, 1);
                let sig_names = self.method_sig_names(m.fn_index, 1);
                let sig_defaults = self.method_sig_defaults(m.fn_index, 1);
                self.fill_default_args(&sig_names, &sig_params, &sig_defaults, &mut rargs, span);
                self.retag_method_args(&mut rargs, m.fn_index, 1);
                self.check_args(
                    &name.name,
                    &self.method_sig_names(m.fn_index, 1),
                    &sig_params,
                    &rargs,
                    span,
                );
                ResolvedExpr {
                    span,
                    vt: self.method_sig_result(m.fn_index),
                    kind: RExprKind::MethodCall {
                        fn_index: m.fn_index,
                        recv,
                        args: rargs,
                    },
                }
            }
            Some(_) => {
                self.bind_err(
                    "source.not-callable",
                    format!(
                        "`{}` is a static method of `{}`; call it as `{}.{}(...)`",
                        name.name, self.records[ri].name, self.records[ri].name, name.name
                    ),
                    name.span,
                );
                ResolvedExpr {
                    span,
                    vt: VT::I32,
                    kind: RExprKind::ReadLocal(LocalRef(usize::MAX)),
                }
            }
            None => {
                self.bind_err(
                    "source.undefined",
                    format!(
                        "type `{}` has no method named `{}`",
                        self.records[ri].name, name.name
                    ),
                    name.span,
                );
                ResolvedExpr {
                    span,
                    vt: VT::I32,
                    kind: RExprKind::ReadLocal(LocalRef(usize::MAX)),
                }
            }
        }
    }

    #[allow(dead_code)]
    fn method_call(
        &mut self,
        recv: &ast::Expr,
        name: &ast::Ident,
        args: &[ast::Arg],
        span: Span,
    ) -> ResolvedExpr {
        // static: `Pair.new(...)`
        if let ast::ExprKind::Var(id) = &recv.kind {
            if self.lookup(&id.name).is_none() {
                if let Some(&ri) = self.rec_index.get(&id.name) {
                    let rargs = self.resolve_args(args);
                    match self.records[ri]
                        .methods
                        .iter()
                        .find(|m| m.name == name.name)
                    {
                        Some(m) if m.is_static => {
                            let sig_params = self.method_sig_params(m.fn_index, 0);
                            self.check_args(
                                &name.name,
                                &self.method_sig_names(m.fn_index, 0),
                                &sig_params,
                                &rargs,
                                span,
                            );
                            return ResolvedExpr {
                                span,
                                vt: self.method_sig_result(m.fn_index),
                                kind: RExprKind::MethodCall {
                                    fn_index: m.fn_index,
                                    recv: None,
                                    args: rargs,
                                },
                            };
                        }
                        Some(_) => {
                            self.bind_err(
                                "source.not-callable",
                                format!(
                                    "`{}` is an instance method of `{}`; call it on a value (`{}.{}(...)` via a local)",
                                    name.name, self.records[ri].name, self.records[ri].name, name.name
                                ),
                                name.span,
                            );
                            return ResolvedExpr {
                                span,
                                vt: VT::I32,
                                kind: RExprKind::ReadLocal(LocalRef(usize::MAX)),
                            };
                        }
                        None => {
                            self.bind_err(
                                "source.undefined",
                                format!("type `{}` has no method named `{}`", id.name, name.name),
                                name.span,
                            );
                            return ResolvedExpr {
                                span,
                                vt: VT::I32,
                                kind: RExprKind::ReadLocal(LocalRef(usize::MAX)),
                            };
                        }
                    }
                }
            }
        }
        // instance: resolve the receiver expression exactly once
        let rrecv = self.expr_iterative(recv);
        let VT::Rec(ri) = rrecv.vt else {
            self.bind_err(
                "source.type-mismatch",
                format!(
                    "method calls require a record-typed receiver, found `{}`",
                    rrecv.vt.as_str()
                ),
                recv.span,
            );
            return ResolvedExpr {
                span,
                vt: VT::I32,
                kind: RExprKind::ReadLocal(LocalRef(usize::MAX)),
            };
        };
        let args = self.resolve_args(args);
        self.instance_or_static(Some(Box::new(rrecv)), ri, name, args, span)
    }

    /// Resolve an argument list (positional and named).
    fn resolve_args(&mut self, args: &[ast::Arg]) -> Vec<ResolvedArg> {
        let mut out = Vec::new();
        for a in args {
            match a {
                ast::Arg::Pos(e) => {
                    let v = self.expr_iterative(e);
                    out.push(ResolvedArg {
                        span: e.span,
                        name: None,
                        moved: false,
                        value: v,
                    });
                }
                ast::Arg::MovePos(e) => {
                    let v = self.expr_iterative(e);
                    out.push(ResolvedArg {
                        span: e.span,
                        name: None,
                        moved: true,
                        value: v,
                    });
                }
                ast::Arg::Named { name, value, span } => {
                    let v = self.expr_iterative(value);
                    out.push(ResolvedArg {
                        span: *span,
                        name: Some(name.clone()),
                        moved: false,
                        value: v,
                    });
                }
            }
        }
        out
    }

    /// PLAN-105 T-02（U03 真语料 013）：调用实参的 `.move` 标记 → 值上的
    /// `RExprKind::Move`（整局部所有权转移的 sanctioned 形态）。
    fn move_marked_value(&self, arg: ResolvedArg) -> ResolvedExpr {
        if arg.moved && !matches!(arg.value.kind, RExprKind::Move { .. }) {
            ResolvedExpr {
                span: arg.value.span,
                vt: arg.value.vt,
                kind: RExprKind::Move {
                    inner: Box::new(arg.value),
                },
            }
        } else {
            arg.value
        }
    }
    fn method_sig_params(&self, fn_index: usize, from: usize) -> Vec<VT> {
        // fn sigs live behind key names; params are recoverable from the
        // sig table only by key — the FnResolver carries sigs; for methods
        // the receiver-agnostic view is reconstructed from the fn index by
        // scanning the sig table.
        for sig in self.sigs.values() {
            if sig.index == fn_index {
                return sig.params.iter().skip(from).copied().collect();
            }
        }
        Vec::new()
    }

    fn method_sig_names(&self, fn_index: usize, from: usize) -> Vec<String> {
        for sig in self.sigs.values() {
            if sig.index == fn_index {
                return sig.param_names.iter().skip(from).cloned().collect();
            }
        }
        Vec::new()
    }

    fn method_sig_result(&self, fn_index: usize) -> VT {
        for sig in self.sigs.values() {
            if sig.index == fn_index {
                return sig.result;
            }
        }
        VT::I32
    }

    /// PLAN-963（T-08，L-24）：该方法的默认值切片（`from` 之后，与
    /// `method_sig_names/params` 逐位对齐）。
    fn method_sig_defaults(&self, fn_index: usize, from: usize) -> Vec<Option<Box<Expr>>> {
        for sig in self.sigs.values() {
            if sig.index == fn_index {
                return sig.defaults.iter().skip(from).cloned().collect();
            }
        }
        Vec::new()
    }

    /// PLAN-963（T-08，L-24）：按声明位把缺席实参补成默认值。
    ///
    /// 语义（V-08）：默认值只在**实参缺席**时求值，每个调用恰好求值一次，
    /// 且一律在**调用者**作用域解析——用一个空局部作用域解析，任何参数名/
    /// 局部名都按"未定义"拒绝（fail-closed），不会静默绑到调用者的同名局部。
    /// 位置式调用只能补齐末尾缺席位（中段缺席仍由 `check_args` 报 arity）；
    /// 命名式调用补齐任意缺席位并以参数名标注。
    fn fill_default_args(
        &mut self,
        names: &[String],
        params: &[VT],
        defaults: &[Option<Box<Expr>>],
        rargs: &mut Vec<ResolvedArg>,
        call_span: Span,
    ) {
        if defaults.iter().all(|d| d.is_none()) || rargs.len() >= names.len() {
            return;
        }
        let all_named = !rargs.is_empty() && rargs.iter().all(|a| a.name.is_some());
        if !all_named && rargs.iter().any(|a| a.name.is_some()) {
            // 混合形式：由 check_args 报同一诊断，不在此处再报一次。
            return;
        }
        let mut covered = vec![false; names.len()];
        for (i, a) in rargs.iter().enumerate() {
            let pos = match &a.name {
                Some(n) => names.iter().position(|p| p == &n.name),
                None => Some(i),
            };
            if let Some(slot) = pos.and_then(|p| covered.get_mut(p)) {
                *slot = true;
            }
        }
        for idx in 0..names.len() {
            if covered.get(idx).copied().unwrap_or(true) {
                continue;
            }
            let Some(Some(default)) = defaults.get(idx) else {
                continue;
            };
            let saved = std::mem::replace(
                &mut self.scopes,
                vec![Scope {
                    names: BTreeMap::new(),
                    views: BTreeMap::new(),
                    text_mut_aliases: BTreeMap::new(),
                }],
            );
            let mut value = self.expr(default);
            self.scopes = saved;
            if let Some(vt) = params.get(idx) {
                retag_literal(&mut value, *vt);
            }
            let span = default.span;
            rargs.push(ResolvedArg {
                value,
                name: if all_named {
                    Some(ast::Ident {
                        name: names[idx].clone(),
                        span,
                    })
                } else {
                    None
                },
                moved: false,
                span: if all_named { span } else { call_span },
            });
        }
    }

    /// Resolve an expression tree with an explicit work stack (R3-QA-02):
    /// legal operator chains are arbitrarily deep, so no recursion. Leaves
    /// resolve through [`Self::expr`]; binary nodes are combined from
    /// pre-allocated result slots after both children complete (children
    /// first: lhs slot allocated before rhs, combining after both).
    fn expr_iterative(&mut self, root: &Expr) -> ResolvedExpr {
        self.expr_iterative_with_hint(root, None)
    }

    fn expr_iterative_with_hint(&mut self, root: &Expr, root_hint: Option<VT>) -> ResolvedExpr {
        fn dummy_resolved() -> ResolvedExpr {
            ResolvedExpr {
                span: Span { start: 0, end: 0 },
                vt: VT::I32,
                kind: RExprKind::IntLit(0),
            }
        }
        enum Task<'a> {
            /// Resolve this node into `slot`.
            Visit { node: &'a Expr, slot: usize },
            MailboxArg {
                receiver: usize,
                node: &'a Expr,
                slot: usize,
            },
            /// All argument slots are filled; resolve callee/bindings into
            /// `dest` (R5: nested calls must not recurse through call()).
            CallOut {
                callee: &'a ast::Ident,
                explicit: Option<(&'a [ast::TypeAnn], &'a [ast::ConstArgUse])>,
                expected: Option<VT>,
                dest: usize,
                arg_slots: Vec<usize>,
                names: Vec<Option<ast::Ident>>,
                spans: Vec<Span>,
                node_span: Span,
            },
            CallValueOut {
                dest: usize,
                callee_slot: usize,
                arg_slots: Vec<usize>,
                names: Vec<Option<ast::Ident>>,
                spans: Vec<Span>,
                node_span: Span,
            },
            /// Receiver and argument slots are filled; bind the TEXT
            /// builtin method into `dest` (PLAN-770).
            MethodOut {
                name: ast::Ident,
                dest: usize,
                recv_slot: usize,
                arg_slots: Vec<usize>,
                names: Vec<Option<ast::Ident>>,
                spans: Vec<Span>,
                /// PLAN-105 T-02：每个实参是否带 `.move` 标记（AST 的
                /// `Arg::MovePos`——机器的槽位不承载该标记，需显式携带）。
                moved_flags: Vec<bool>,
                node_span: Span,
                to_word: Option<TypeWord>,
            },
            /// Receiver + arg slots are filled (receiver slot `None` for
            /// pre-classified static calls); resolve the RECORD method
            /// into `dest` (PLAN-771).
            RecordMethodOut {
                name: ast::Ident,
                explicit: Option<&'a ast::InstanceArguments>,
                dest: usize,
                recv_slot: Option<usize>,
                recv_kind: RecvClass<'a>,
                expected: Option<VT>,
                arg_slots: Vec<usize>,
                names: Vec<Option<ast::Ident>>,
                spans: Vec<Span>,
                /// PLAN-105 T-02：每个实参是否带 `.move` 标记。
                moved_flags: Vec<bool>,
                node_span: Span,
            },
            /// Both operand slots are filled; combine into `dest`.
            Combine {
                op: BinOp,
                span: Span,
                lslot: usize,
                rslot: usize,
                dest: usize,
            },
            /// Operand slot is filled; apply the unary op into `dest`.
            UnaryOut {
                op: UnaryOp,
                span: Span,
                oslot: usize,
                dest: usize,
            },
            /// Base slot is filled; resolve the field segment into `dest`
            /// (PLAN-771; fields are iterative so 4096-segment chains never
            /// recurse).
            FieldOut {
                name: ast::Ident,
                span: Span,
                bslot: usize,
                dest: usize,
            },

            /// All field-value slots are filled; resolve the named
            /// construction into `dest` (F-771-R1/V-09: nested ctors stay
            /// iterative instead of recursing through `ctor_named`).
            CtorOut {
                ty: ast::Ident,
                dest: usize,
                arg_slots: Vec<usize>,
                names: Vec<ast::Ident>,
                fspans: Vec<Span>,
                node_span: Span,
            },
        }
        let mut out: Vec<ResolvedExpr> = vec![dummy_resolved()];
        let mut stack: Vec<Task> = vec![Task::Visit {
            node: root,
            slot: 0,
        }];
        while let Some(task) = stack.pop() {
            match task {
                Task::Visit { node, slot } => {
                    if let ExprKind::Closure {
                        move_capture,
                        params,
                        result,
                        body,
                    } = &node.kind
                    {
                        out[slot] =
                            self.resolve_closure(*move_capture, params, result.as_ref(), body, node.span);
                        continue;
                    }
                    if let ExprKind::CallValue { callee, args } = &node.kind {
                        let callee_slot = out.len();
                        out.push(dummy_resolved());
                        let mut arg_slots = Vec::with_capacity(args.len());
                        let mut names = Vec::with_capacity(args.len());
                        let mut spans = Vec::with_capacity(args.len());
                        let mut subtasks = Vec::with_capacity(args.len() + 2);
                        subtasks.push(Task::Visit {
                            node: callee,
                            slot: callee_slot,
                        });
                        for arg in args {
                            let (value, name, arg_span) = match arg {
                                Arg::Pos(value) | Arg::MovePos(value) => (value, None, value.span),
                                Arg::Named { name, value, span } => (
                                    value,
                                    Some(ast::Ident {
                                        name: name.name.clone(),
                                        span: name.span,
                                    }),
                                    *span,
                                ),
                            };
                            let arg_slot = out.len();
                            out.push(dummy_resolved());
                            arg_slots.push(arg_slot);
                            names.push(name);
                            spans.push(arg_span);
                            subtasks.push(Task::Visit {
                                node: value,
                                slot: arg_slot,
                            });
                        }
                        subtasks.push(Task::CallValueOut {
                            dest: slot,
                            callee_slot,
                            arg_slots,
                            names,
                            spans,
                            node_span: node.span,
                        });
                        for task in subtasks.into_iter().rev() {
                            stack.push(task);
                        }
                        continue;
                    }
                    let expected = if std::ptr::eq(root, node) { root_hint } else { None };
                    let call = match &node.kind {
                        ExprKind::Call { callee, args } => Some((callee, args, None)),
                        ExprKind::ConstCall {
                            callee,
                            args,
                            types,
                            consts,
                        } if self.instance_probe.is_some() => {
                            Some((callee, args, Some((types.as_slice(), consts.as_slice()))))
                        }
                        _ => None,
                    };
                    if let Some((callee, args, explicit)) = call {
                        // PLAN-936（T-04）：meta 查询内在——selector 实参在
                        // 调度前拦截（不作为子表达式任务解析）。
                        // With no meta grant an actual user declaration keeps
                        // its ordinary meaning, including inside a CT demand.
                        if explicit.is_none()
                            && (self.meta_enabled
                                || (self.ct_depth > 0 && !self.sigs.contains_key(&callee.name)))
                        {
                            if let Some(op) = meta_intrinsic(&callee.name) {
                                out[slot] = self.meta_query(op, callee, args, node.span);
                                continue;
                            }
                        }
                        if args.is_empty() && explicit.is_none() && !self.is_template_call(callee) {
                            out[slot] = self.call(callee, &[], node.span);
                            continue;
                        }
                        let mut arg_slots = Vec::with_capacity(args.len());
                        let mut names = Vec::with_capacity(args.len());
                        let mut spans = Vec::with_capacity(args.len());
                        let mut subtasks = Vec::with_capacity(args.len() * 2 + 1);
                        for a in args {
                            let s = out.len();
                            out.push(dummy_resolved());
                            arg_slots.push(s);
                            match a {
                                Arg::Pos(e) | Arg::MovePos(e) => {
                                    names.push(None);
                                    spans.push(e.span);
                                    subtasks.push(Task::Visit { node: e, slot: s });
                                }
                                Arg::Named { name, value, span } => {
                                    names.push(Some(ast::Ident {
                                        name: name.name.clone(),
                                        span: name.span,
                                    }));
                                    spans.push(*span);
                                    subtasks.push(Task::Visit {
                                        node: value,
                                        slot: s,
                                    });
                                }
                            }
                        }
                        subtasks.push(Task::CallOut {
                            callee,
                            explicit,
                            expected,
                            dest: slot,
                            arg_slots,
                            names,
                            spans,
                            node_span: node.span,
                        });
                        // LIFO: push in reverse so args resolve in source order
                        for t in subtasks.into_iter().rev() {
                            stack.push(t);
                        }
                        continue;
                    }
                    if let ExprKind::Method { recv, name, args } = &node.kind {
                        let recv_slot = out.len();
                        out.push(dummy_resolved());
                        let mut arg_slots = Vec::with_capacity(args.len());
                        let mut names = Vec::with_capacity(args.len());
                        let mut spans = Vec::with_capacity(args.len());
                        let mut moved_flags: Vec<bool> = Vec::with_capacity(args.len());
                        let mut subtasks = Vec::with_capacity(args.len() + 2);
                        // `.to(...)` takes a single bare type word, not a
                        // value expression; keep it for MethodOut
                        let mut to_word = None;
                        for a in args {
                            let sl = out.len();
                            out.push(dummy_resolved());
                            arg_slots.push(sl);
                            match a {
                                Arg::Pos(e) => {
                                    names.push(None);
                                    moved_flags.push(false);
                                    spans.push(e.span);
                                    if let ExprKind::TypeWordArg(w) = &e.kind {
                                        to_word = Some(*w);
                                    } else {
                                        subtasks.push(Task::Visit { node: e, slot: sl });
                                    }
                                }
                                Arg::MovePos(e) => {
                                    names.push(None);
                                    moved_flags.push(true);
                                    spans.push(e.span);
                                    subtasks.push(Task::Visit { node: e, slot: sl });
                                }
                                Arg::Named { name, value, span } => {
                                    names.push(Some(ast::Ident {
                                        name: name.name.clone(),
                                        span: name.span,
                                    }));
                                    moved_flags.push(false);
                                    spans.push(*span);
                                    subtasks.push(Task::Visit {
                                        node: value,
                                        slot: sl,
                                    });
                                }
                            }
                        }
                        subtasks.push(Task::MethodOut {
                            name: ast::Ident {
                                name: name.name.clone(),
                                span: name.span,
                            },
                            dest: slot,
                            recv_slot,
                            arg_slots,
                            names,
                            moved_flags,
                            spans,
                            node_span: node.span,
                            to_word,
                        });
                        // LIFO: receiver first, then args, then the binding
                        let mut ordered = vec![Task::Visit {
                            node: recv,
                            slot: recv_slot,
                        }];
                        ordered.extend(subtasks);
                        for t in ordered.into_iter().rev() {
                            stack.push(t);
                        }
                        continue;
                    }
                    if let ExprKind::Field { base, name } = &node.kind {
                        if self.profile.is_aggregates() {
                            if let Some(se) = self.try_resolve_scalar_enum(node, None) {
                                out[slot] = se;
                                continue;
                            } else if let Some(va) = self.try_resolve_variant_arm(node, expected) {
                                out[slot] = va;
                                continue;
                            } else if let Some(may) = self.try_resolve_may_ctor(node, None) {
                                out[slot] = may;
                                continue;
                            }
                        }
                        // PLAN-864: an owned-element field projection
                        // (`xs.get(i).f[.g]`, or a for-in view binding) fuses
                        // into one ListFieldGet/ElemView node — never a
                        // materialized ListGet. Only the OUTERMOST Field of
                        // the spine builds the fused node.
                        if self.profile.is_collections() {
                            let mut segs: Vec<&ast::Ident> = vec![name];
                            let mut cur = base.as_ref();
                            while let ExprKind::Field { base: b, name: n } = &cur.kind {
                                segs.push(n);
                                cur = b.as_ref();
                            }
                            let mut fused: Option<ResolvedExpr> = None;
                            let mut view_hit = false;
                            if let ExprKind::Var(id) = &cur.kind {
                                view_hit = self.lookup_view(&id.name).is_some();
                                if let Some(vb) = self.lookup_view(&id.name) {
                                    if let Some((path, vt)) =
                                        self.resolve_projection_path(&segs, vb.elem)
                                    {
                                        let xs_vt = self.locals[vb.list.0].vt;
                                        let owned_term = matches!(vt, VT::StrOwned);
                                        let mk = |kind| ResolvedExpr {
                                            span: node.span,
                                            vt,
                                            kind,
                                        };
                                        fused = Some(if owned_term {
                                            mk(RExprKind::ElemView {
                                                list: Box::new(ResolvedExpr {
                                                    span: node.span,
                                                    vt: xs_vt,
                                                    kind: RExprKind::ReadLocal(vb.list),
                                                }),
                                                index: Box::new(ResolvedExpr {
                                                    span: node.span,
                                                    vt: VT::I32,
                                                    kind: RExprKind::ReadLocal(vb.index),
                                                }),
                                                path,
                                            })
                                        } else {
                                            mk(RExprKind::ListFieldGet {
                                                list: Box::new(ResolvedExpr {
                                                    span: node.span,
                                                    vt: xs_vt,
                                                    kind: RExprKind::ReadLocal(vb.list),
                                                }),
                                                index: Box::new(ResolvedExpr {
                                                    span: node.span,
                                                    vt: VT::I32,
                                                    kind: RExprKind::ReadLocal(vb.index),
                                                }),
                                                path,
                                            })
                                        });
                                    }
                                }
                            }
                            if fused.is_none() && !view_hit {
                                if let ExprKind::MethodCall {
                                    recv: r,
                                    name: mn,
                                    args: ma,
                                    instance: None,
                                } = &cur.kind
                                {
                                    if mn.name == "get" && ma.len() == 1 {
                                        if let ExprKind::Var(id) = &r.kind {
                                            if let Some(xl) = self.lookup(&id.name) {
                                                let xvt = self.locals[xl.0].vt;
                                                if let VT::List(li) = xvt {
                                                    let elem = self.coll.lists[li];
                                                    if self.vt_is_coll_owned(elem) {
                                                        if let Some((path, vt)) = self
                                                            .resolve_projection_path(&segs, elem)
                                                        {
                                                            let idx_ast = match &ma[0] {
                                                                ast::Arg::Pos(e)
                                                                | ast::Arg::MovePos(e) => e,
                                                                // PLAN-1035 (F33): the ordinary
                                                                // `get` builtin admits a Named
                                                                // index (single argument by
                                                                // value, label not validated) —
                                                                // the fused projection must match
                                                                // it instead of panicking on a
                                                                // user program.
                                                                ast::Arg::Named {
                                                                    value, ..
                                                                } => value,
                                                            };
                                                            let idx_read =
                                                                self.expr_iterative(idx_ast);
                                                            let xs_read = ResolvedExpr {
                                                                span: node.span,
                                                                vt: xvt,
                                                                kind: RExprKind::ReadLocal(xl),
                                                            };
                                                            let owned_term =
                                                                matches!(vt, VT::StrOwned);
                                                            let mk =
                                                                |kind: RExprKind| ResolvedExpr {
                                                                    span: node.span,
                                                                    vt,
                                                                    kind,
                                                                };
                                                            fused = Some(if owned_term {
                                                                mk(RExprKind::ElemView {
                                                                    list: Box::new(xs_read),
                                                                    index: Box::new(idx_read),
                                                                    path,
                                                                })
                                                            } else {
                                                                mk(RExprKind::ListFieldGet {
                                                                    list: Box::new(xs_read),
                                                                    index: Box::new(idx_read),
                                                                    path,
                                                                })
                                                            });
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            if let Some(fr) = fused {
                                out[slot] = fr;
                                continue;
                            }
                        }
                        let bslot = out.len();
                        out.push(dummy_resolved());
                        stack.push(Task::FieldOut {
                            name: name.clone(),
                            span: node.span,
                            bslot,
                            dest: slot,
                        });
                        stack.push(Task::Visit {
                            node: base,
                            slot: bslot,
                        });
                    } else if let ExprKind::MethodCall {
                        recv,
                        name,
                        args,
                        instance,
                    } = &node.kind
                    {
                        if self.profile.is_auto_language()
                            && instance.is_none()
                            && name.name == "spawn"
                            && matches!(&recv.kind, ExprKind::Var(id) if id.name == "Task" && self.lookup(&id.name).is_none() && !self.rec_index.contains_key(&id.name))
                        {
                            out[slot] = self.call(
                                &ast::Ident {
                                    name: "__sched_spawn".into(),
                                    span: name.span,
                                },
                                args,
                                node.span,
                            );
                            continue;
                        }
                        if self.profile.is_auto_language() && instance.is_none() {
                            if let ExprKind::Var(id) = &recv.kind {
                                if self.lookup(&id.name).is_none()
                                    && self.sched.and_then(|s| s.by_name(&id.name)).is_some()
                                    && matches!(name.name.as_str(), "send" | "ask" | "stop")
                                {
                                    let handle = Expr {
                                        span: id.span,
                                        kind: ExprKind::Call {
                                            callee: ast::Ident {
                                                name: "__sched_singleton".into(),
                                                span: id.span,
                                            },
                                            args: vec![
                                                Arg::Pos(Expr {
                                                    kind: ExprKind::StrLit(id.name.clone()),
                                                    span: id.span,
                                                }),
                                                Arg::Pos(Expr {
                                                    kind: ExprKind::IntLit(64),
                                                    span: id.span,
                                                }),
                                            ],
                                        },
                                    };
                                    let mut arguments = Vec::with_capacity(args.len() + 1);
                                    arguments.push(Arg::Pos(handle));
                                    arguments.extend(args.iter().cloned());
                                    out[slot] = self.call(
                                        &ast::Ident {
                                            name: format!("__sched_{}", name.name),
                                            span: name.span,
                                        },
                                        &arguments,
                                        node.span,
                                    );
                                    continue;
                                }
                            }
                        }
                        let text_alias = match &recv.kind {
                            ExprKind::Var(alias) => self.lookup_text_mut_alias(&alias.name),
                            _ => None,
                        };
                        if let Some(root) = text_alias {
                            if name.name == "append" {
                                out[slot] = self.resolve_text_append(root, recv, args, node.span);
                            } else {
                                self.bind_err(
                                    "source.view-escape",
                                    "a mutable text binding can only be used as the receiver of append",
                                    recv.span,
                                );
                            }
                            continue;
                        }
                        if instance.is_none() && self.profile.is_aggregates() {
                            if let Some(se) = self.try_resolve_scalar_enum(node, None) {
                                out[slot] = se;
                                continue;
                            } else if let Some(va) = self.try_resolve_variant_arm(node, expected) {
                                out[slot] = va;
                                continue;
                            } else if let Some(may) = self.try_resolve_may_ctor(node, None) {
                                out[slot] = may;
                                continue;
                            }
                        }
                        // PLAN-1082 (T-03): the List.new sugar intercepts
                        // in the iterative walker too (every expression
                        // position funnels through here).
                        if instance.is_none() {
                            if let Some(list) = self.try_resolve_legacy_list_new(node, None) {
                                out[slot] = list;
                                continue;
                            }
                        }
                        // static pre-classification: bare record type name
                        // receiver resolves to no local — the receiver
                        // expression is NOT evaluated for static calls.
                        // PLAN-864: classify the receiver before resolving —
                        // generic type ctors (`List<T>.new()`) and qualified
                        // variant ctors (`E.A(e)`) carry a TYPE, not a value;
                        // their receiver slot stays empty.
                        let recv_class = if instance.is_some() {
                            match &recv.kind {
                                ExprKind::Var(id) if self.lookup(&id.name).is_none() => self
                                    .rec_index
                                    .get(&id.name)
                                    .copied()
                                    .map_or(RecvClass::Value, RecvClass::Static),
                                _ => RecvClass::Value,
                            }
                        } else if let Some((template, types)) = self.generic_enum_receiver(recv) {
                            RecvClass::GenericEnum { template, types }
                        } else if let ExprKind::GenericType { name, args } = &recv.kind {
                            self.classify_generic_recv(name, args)
                        } else if let ExprKind::Var(id) = &recv.kind {
                            // PLAN-981：AGE 资源服务（`material2d.color`）——接收者
                            // 是**命名空间**不是值；按声明表分类，绝不求值它。
                            if self.lookup(&id.name).is_none()
                                && crate::source::age_resources::lookup(&format!(
                                    "{}.{}",
                                    id.name, name.name
                                ))
                                .is_some()
                            {
                                let dotted = format!("{}.{}", id.name, name.name);
                                RecvClass::AgeService(
                                    crate::source::age_resources::lookup(&dotted)
                                        .expect("checked above"),
                                )
                            } else if self.lookup(&id.name).is_none()
                                && crate::source::age_resources::gfx_lookup(&format!(
                                    "{}.{}",
                                    id.name, name.name
                                ))
                                .is_some()
                            {
                                let dotted = format!("{}.{}", id.name, name.name);
                                RecvClass::AgeGfx(
                                    crate::source::age_resources::gfx_lookup(&dotted)
                                        .expect("checked above"),
                                )
                            } else if self.lookup(&id.name).is_none()
                                && self.rec_index.contains_key(&id.name)
                            {
                                RecvClass::Static(self.rec_index[&id.name])
                            } else if self.lookup(&id.name).is_none()
                                && self.profile.is_collections()
                            {
                                match self.qualified_variant_arm(&id.name, &name.name) {
                                    Some((vi, disc)) => {
                                        self.check_qualified_ctor_args(vi, disc, args, name.span);
                                        RecvClass::VariantArm { vi, disc }
                                    }
                                    None => RecvClass::Value,
                                }
                            } else {
                                RecvClass::Value
                            }
                        } else {
                            RecvClass::Value
                        };
                        // PLAN-864: `xs.get(i).len()` over a List<Str>
                        // local — the get receiver is a whole-element view,
                        // not a value. (PLAN-904 fix: the index visit is
                        // DEFERRED into this node's subtask list — pushing
                        // it during classification put it UNDER the
                        // RecordMethodOut combiner on the LIFO stack, so
                        // the combiner read a dummy index and every fused
                        // view rendered index 0.)
                        let mut elemview_idx_task: Option<Task> = None;
                        let recv_class = if matches!(recv_class, RecvClass::Value)
                            && name.name == "len"
                            && args.is_empty()
                        {
                            if let ExprKind::MethodCall {
                                recv: gr,
                                name: gn,
                                args: ga,
                                instance: None,
                            } = &recv.kind
                            {
                                if gn.name == "get"
                                    && ga.len() == 1
                                    && matches!(
                                        &ga[0],
                                        ast::Arg::Pos(_)
                                            | ast::Arg::MovePos(_)
                                            | ast::Arg::Named { .. }
                                    )
                                {
                                    if let ExprKind::Var(id) = &gr.kind {
                                        if let Some(xl) = self.lookup(&id.name) {
                                            let xvt = self.locals[xl.0].vt;
                                            if let VT::List(li) = xvt {
                                                let elem = self.coll.lists[li];
                                                if matches!(elem, VT::StrOwned) {
                                                    let idx_slot = out.len();
                                                    out.push(dummy_resolved());
                                                    let idx_ast = match &ga[0] {
                                                        ast::Arg::Pos(e) | ast::Arg::MovePos(e) => {
                                                            e
                                                        }
                                                        // PLAN-1035 (F33): same Named-index
                                                        // admission as the ordinary `get`.
                                                        ast::Arg::Named { value, .. } => value,
                                                    };
                                                    elemview_idx_task = Some(Task::Visit {
                                                        node: idx_ast,
                                                        slot: idx_slot,
                                                    });
                                                    RecvClass::ElemViewLen { xl, xvt, idx_slot }
                                                } else {
                                                    RecvClass::Value
                                                }
                                            } else {
                                                RecvClass::Value
                                            }
                                        } else {
                                            RecvClass::Value
                                        }
                                    } else {
                                        RecvClass::Value
                                    }
                                } else {
                                    RecvClass::Value
                                }
                            } else {
                                RecvClass::Value
                            }
                        } else {
                            recv_class
                        };
                        if self.profile.is_auto_language() {
                            if let RecvClass::ListCtor(list) = recv_class {
                                match (name.name.as_str(), args.as_slice()) {
                                    ("new", [Arg::Pos(ast::Expr { kind: ExprKind::ArrayLit(elems), .. })]) => {
                                        out[slot] = self.resolve_array_literal(
                                            elems, node.span, false, Some(list),
                                        );
                                        continue;
                                    }
                                    ("new", []) => {}
                                    _ => {
                                        self.bind_err(
                                            "source.argument",
                                            "use `List<T>.new()` or `List<T>.new([elements])`",
                                            node.span,
                                        );
                                        continue;
                                    }
                                }
                            }
                        }
                        let recv_slot = match recv_class {
                            RecvClass::Value => Some(out.len()),
                            RecvClass::ElemViewLen { .. } => None,
                            _ => None,
                        };
                        if recv_slot.is_some() {
                            out.push(dummy_resolved());
                        }
                        let mut arg_slots = Vec::with_capacity(args.len());
                        let mut names = Vec::with_capacity(args.len());
                        let mut spans = Vec::with_capacity(args.len());
                        let mut moved_flags: Vec<bool> = Vec::with_capacity(args.len());
                        let mut subtasks = Vec::with_capacity(args.len() + 1);
                        let mailbox = if self.profile.is_auto_language()
                            && matches!(name.name.as_str(), "send" | "ask")
                            && args.len() == 1
                        {
                            recv_slot
                        } else {
                            None
                        };
                        for a in args {
                            let sp = out.len();
                            out.push(dummy_resolved());
                            arg_slots.push(sp);
                            match a {
                                Arg::Pos(e) | Arg::MovePos(e) => {
                                    names.push(None);
                                    moved_flags.push(matches!(a, Arg::MovePos(_)));
                                    spans.push(e.span);
                                    subtasks.push(match mailbox {
                                        Some(receiver) => Task::MailboxArg {
                                            receiver,
                                            node: e,
                                            slot: sp,
                                        },
                                        None => Task::Visit { node: e, slot: sp },
                                    });
                                }
                                Arg::Named {
                                    name: n,
                                    value,
                                    span,
                                } => {
                                    names.push(Some(ast::Ident {
                                        name: n.name.clone(),
                                        span: n.span,
                                    }));
                                    moved_flags.push(false);
                                    spans.push(*span);
                                    subtasks.push(Task::Visit {
                                        node: value,
                                        slot: sp,
                                    });
                                }
                            }
                        }
                        // the deferred fused-view index visit must run
                        // BEFORE the combiner (vec order: args…, idx,
                        // RecordMethodOut — the reversed push pops the
                        // combiner last)
                        if let Some(t) = elemview_idx_task.take() {
                            subtasks.push(t);
                        }
                        subtasks.push(Task::RecordMethodOut {
                            name: name.clone(),
                            explicit: instance.as_deref(),
                            dest: slot,
                            recv_slot,
                            recv_kind: recv_class,
                            expected,
                            arg_slots,
                            names,
                            spans,
                            moved_flags,
                            node_span: node.span,
                        });
                        for t in subtasks.into_iter().rev() {
                            stack.push(t);
                        }
                        // LIFO: the receiver visit must pop FIRST (before
                        // the args and the MethodOut combiner)
                        if let Some(rs) = recv_slot {
                            stack.push(Task::Visit {
                                node: recv,
                                slot: rs,
                            });
                        }
                    } else if let ExprKind::Binary { op, lhs, rhs } = &node.kind {
                        if self.profile.is_aggregates() {
                            if *op == BinOp::NullCoalesce {
                                out[slot] = self.resolve_unwrap_or(lhs, rhs, None, node.span);
                                continue;
                            }
                            if matches!(op, BinOp::And | BinOp::Or) {
                                out[slot] =
                                    self.resolve_short_circuit_bin(*op, lhs, rhs, node.span);
                                continue;
                            }
                        }
                        let lslot = out.len();
                        out.push(dummy_resolved());
                        let rslot = out.len();
                        out.push(dummy_resolved());
                        // LIFO: lhs resolves first, then rhs, then combine.
                        stack.push(Task::Combine {
                            op: *op,
                            span: node.span,
                            lslot,
                            rslot,
                            dest: slot,
                        });
                        stack.push(Task::Visit {
                            node: rhs,
                            slot: rslot,
                        });
                        stack.push(Task::Visit {
                            node: lhs,
                            slot: lslot,
                        });
                    } else if let ExprKind::Unary { op, operand } = &node.kind {
                        let oslot = out.len();
                        out.push(dummy_resolved());
                        stack.push(Task::UnaryOut {
                            op: *op,
                            span: node.span,
                            oslot,
                            dest: slot,
                        });
                        stack.push(Task::Visit {
                            node: operand,
                            slot: oslot,
                        });
                    } else if let ExprKind::CtorNamed { ty, fields, .. } = &node.kind {
                        // every field value is its own subtask: the outer
                        // ctor's frame returns before inner ctors resolve
                        let mut arg_slots = Vec::with_capacity(fields.len());
                        let mut names = Vec::with_capacity(fields.len());
                        let mut fspans = Vec::with_capacity(fields.len());
                        let mut subtasks = Vec::with_capacity(fields.len() + 1);
                        for f in fields {
                            let sp = out.len();
                            out.push(dummy_resolved());
                            arg_slots.push(sp);
                            names.push(f.name.clone());
                            fspans.push(f.span);
                            subtasks.push(Task::Visit {
                                node: &f.value,
                                slot: sp,
                            });
                        }
                        subtasks.push(Task::CtorOut {
                            ty: ty.clone(),
                            dest: slot,
                            arg_slots,
                            names,
                            fspans,
                            node_span: node.span,
                        });
                        for t in subtasks.into_iter().rev() {
                            stack.push(t);
                        }
                    } else if let ExprKind::Propagate { inner } = &node.kind {
                        out[slot] = if self.profile.is_aggregates() {
                            self.resolve_propagate(inner, node.span)
                        } else {
                            self.expr(node)
                        };
                    } else if let ExprKind::UnwrapOr { inner, fallback } = &node.kind {
                        out[slot] = if self.profile.is_aggregates() {
                            self.resolve_unwrap_or(inner, fallback, None, node.span)
                        } else {
                            self.expr(node)
                        };
                    } else if let ExprKind::IsExpr { scrutinee, arms } = &node.kind {
                        if self.profile.is_aggregates() {
                            let (stmts, expr_opt) =
                                self.resolve_agg_is(scrutinee, arms, None, true, node.span, false);
                            for s in stmts {
                                self.emit_stmt(s);
                            }
                            out[slot] = expr_opt.unwrap_or(ResolvedExpr {
                                span: node.span,
                                vt: VT::Unit,
                                kind: RExprKind::Unit,
                            });
                        } else {
                            out[slot] = self.expr(node);
                        }
                    } else {
                        out[slot] = self.expr(node);
                    }
                }
                Task::CtorOut {
                    ty,
                    dest,
                    arg_slots,
                    names,
                    fspans,
                    node_span,
                } => {
                    let mut values = Vec::with_capacity(arg_slots.len());
                    for (i, sp) in arg_slots.iter().enumerate() {
                        let v = std::mem::replace(&mut out[*sp], dummy_resolved());
                        values.push((names[i].clone(), fspans[i], v));
                    }
                    out[dest] = self.ctor_named_out(&ty, values, node_span);
                }
                Task::FieldOut {
                    name,
                    span,
                    bslot,
                    dest,
                } => {
                    let mut base = std::mem::replace(&mut out[bslot], dummy_resolved());
                    // PLAN-1026: attribute read on a Python object —
                    // py_getattr(receiver, name); the result is a Python
                    // object (never an early field projection).
                    if self.py_is_object(&base) {
                        let mut parts: Vec<PyFramePart> =
                            vec![PyFramePart::Txt("p967.frame/1\nr ".to_string())];
                        let chunk = self.py_chunk_call("o", &base);
                        parts.push(PyFramePart::TextExpr(chunk));
                        parts.push(PyFramePart::Txt(format!(
                            "p 1\nt {}\n{}\nk 0\n",
                            name.name.len(),
                            name.name
                        )));
                        let frame = self.py_frame(name.span, parts);
                        // PLAN-1107 T-02: attribute operation fact.
                        if crate::py_ops::recording_active() {
                            crate::py_ops::record(
                                PyOpFact::new(PyOpKind::GetAttr, name.span)
                                    .with_member(name.name.clone())
                                    .operand_lit(
                                        PyRole::Receiver,
                                        base.span,
                                        self.py_val_class(&base),
                                        self.py_operand_literal(&base),
                                    ),
                            );
                        }
                        out[dest] = self.py_op_call("py_getattr", name.span, frame);
                        continue;
                    }
                    if self.profile == SourceProfile::Auto {
                        if base.vt.is_text() && name.name == "len" {
                            out[dest] = self.resolve_method(&name, base, Vec::new(), None, span);
                            continue;
                        }
                        if base.vt == VT::Dyn {
                            let object = self.coll.intern_object(VT::Dyn);
                            base = Self::coerce_dynamic(base, VT::Object(object));
                        }
                        if let VT::Object(object) = base.vt {
                            out[dest] = Self::object_get(
                                base,
                                Self::object_field_key(&name),
                                self.coll.objects[object],
                                span,
                            );
                            continue;
                        }
                    }
                    let VT::Rec(ri) = base.vt else {
                        self.bind_err(
                            "source.type-mismatch",
                            format!(
                                "field access requires a record-typed value, found `{}`",
                                base.vt.as_str()
                            ),
                            name.span,
                        );
                        out[dest] = ResolvedExpr {
                            span,
                            vt: VT::I32,
                            kind: RExprKind::ReadLocal(LocalRef(usize::MAX)),
                        };
                        continue;
                    };
                    match self.records[ri]
                        .fields
                        .iter()
                        .position(|fl| fl.name == name.name)
                    {
                        Some(fi) => {
                            // PLAN-782 5.2: reading an owned Str field is a
                            // BORROWING VIEW of the root's buffer, never a
                            // copy of the owner - owning a field value takes
                            // an explicit owned construction (`.to(Str)`,
                            // clone, concat) or moving the whole root. The
                            // plain record profile has no Str fields (its
                            // member-type guard), so this rewrites nothing
                            // there.
                            let vt = match self.records[ri].fields[fi].vt {
                                // PLAN-782 5.2: a field read is a borrowing
                                // view on the combination surface.
                                // PLAN-864: the collections surface keeps
                                // the DECLARED type — profile 4's FieldGet
                                // contract is exact-type (the runtime
                                // handle-shares, never deep-copies), so
                                // `.len()` reads ride the owned value.
                                VT::StrOwned if !self.profile.is_collections() => VT::Str,
                                v => v,
                            };
                            out[dest] = ResolvedExpr {
                                span,
                                vt,
                                kind: RExprKind::FieldGet {
                                    base: Box::new(base),
                                    path: vec![(ri, fi)],
                                },
                            };
                        }
                        None => {
                            self.bind_err(
                                "source.undefined",
                                format!(
                                    "record `{}` has no field named `{}`",
                                    self.records[ri].name, name.name
                                ),
                                name.span,
                            );
                            out[dest] = ResolvedExpr {
                                span,
                                vt: VT::I32,
                                kind: RExprKind::ReadLocal(LocalRef(usize::MAX)),
                            };
                        }
                    }
                }
                Task::MailboxArg {
                    receiver,
                    node,
                    slot,
                } => {
                    if let Some(value) = self.resolve_implicit_actor_message(out[receiver].vt, node)
                    {
                        out[slot] = value;
                    } else {
                        stack.push(Task::Visit { node, slot });
                    }
                }
                Task::RecordMethodOut {
                    name,
                    explicit,
                    dest,
                    recv_slot,
                    recv_kind,
                    expected,
                    arg_slots,
                    names,
                    spans,
                    moved_flags,
                    node_span,
                } => {
                    let mut rargs = Vec::with_capacity(arg_slots.len());
                    for (i, sp) in arg_slots.iter().enumerate() {
                        let value = std::mem::replace(&mut out[*sp], dummy_resolved());
                        // PLAN-105 T-02（U03 真语料 013）：记录接收者上的列表方法
                        // 也要保留 `.move` 实参标记（`push(<整局部>)`）。
                        let moved = moved_flags.get(i).copied().unwrap_or(false)
                            || matches!(value.kind, RExprKind::Move { .. });
                        rargs.push(ResolvedArg {
                            span: spans[i],
                            name: names[i].clone(),
                            moved,
                            value,
                        });
                    }
                    let rrecv = match recv_slot {
                        None => None,
                        Some(rs) => {
                            Some(Box::new(std::mem::replace(&mut out[rs], dummy_resolved())))
                        }
                    };
                    match recv_kind {
                        // PLAN-981（IF-981-ORIGINAL）：AGE 资源服务调用。接收者是
                        // 命名空间（未求值）；参数按声明表校验后落成固定符号的
                        // intrinsic 调用，返回类型是该 op 的**端口类型**。
                        RecvClass::AgeService(op) => {
                            if rargs.len() != op.params.len() {
                                self.bind_err(
                                    "source.age.resource-arity",
                                    format!(
                                        "`{}` takes {} argument(s), got {}",
                                        op.callee,
                                        op.params.len(),
                                        rargs.len()
                                    ),
                                    node_span,
                                );
                            }
                            let mut args = Vec::with_capacity(rargs.len());
                            for (arg, expected) in rargs.into_iter().zip(op.params.iter()) {
                                if arg.value.vt != *expected {
                                    self.bind_err(
                                        "source.age.resource-argument",
                                        format!(
                                            "`{}` argument must be `{}`, got `{}`",
                                            op.callee,
                                            expected.as_str(),
                                            arg.value.vt.as_str()
                                        ),
                                        arg.value.span,
                                    );
                                }
                                args.push(arg.value);
                            }
                            // 端口在解析开始时已登记；缺失是内部不变量破坏，
                            // not a silent downgrade to i32.
                            let port = match self.rec_index.get(op.result_port) {
                                Some(&i) => VT::Rec(i),
                                None => {
                                    self.bind_err(
                                        "source.age.resource-port",
                                        format!(
                                            "AGE port type `{}` is not registered for this document",
                                            op.result_port
                                        ),
                                        node_span,
                                    );
                                    VT::I32
                                }
                            };
                            self.used_intrinsics.insert(op.symbol);
                            out[dest] = ResolvedExpr {
                                span: node_span,
                                vt: port,
                                kind: RExprKind::CallIntrinsic {
                                    symbol: op.symbol,
                                    args,
                                },
                            };
                        }
                        // PLAN-981（IF-981-GFX-AV-SEAM，UI058 AV 接面）：AGE 图形
                        // 服务调用——参数按 gfx 声明表逐位校验（标量 VT 或端口
                        // 引用），结果按 Port/Scalar 定型；执行车道由 lowering
                        // 按声明分类（ValueCtor=Aggregate 可执行；HostAlgorithm/
                        // HostIdentity 在 AV 观察车道具名拒，图形宿主车道消费）。
                        RecvClass::AgeGfx(op) => {
                            if rargs.len() != op.params.len() {
                                self.bind_err(
                                    "source.age.gfx-arity",
                                    format!(
                                        "`{}` takes {} argument(s), got {}",
                                        op.callee,
                                        op.params.len(),
                                        rargs.len()
                                    ),
                                    node_span,
                                );
                            }
                            let mut args = Vec::with_capacity(rargs.len());
                            for (i, arg) in rargs.into_iter().enumerate() {
                                let want = op
                                    .params
                                    .get(i)
                                    .map(|gp| match gp {
                                        crate::source::age_resources::AgeGfxParam::Scalar(vt) => {
                                            *vt
                                        }
                                        crate::source::age_resources::AgeGfxParam::Port(name) => {
                                            match self.rec_index.get(*name) {
                                                Some(&ri) => VT::Rec(ri),
                                                None => {
                                                    self.bind_err(
                                                        "source.age.gfx-port",
                                                        format!(
                                                            "AGE port type `{name}` is not \
                                                             registered for this document"
                                                        ),
                                                        node_span,
                                                    );
                                                    VT::I32
                                                }
                                            }
                                        }
                                    })
                                    .unwrap_or(VT::I32);
                                if arg.value.vt != want {
                                    self.bind_err(
                                        "source.age.gfx-type",
                                        format!(
                                            "`{}` argument #{i} expects `{}`, got `{}`",
                                            op.callee,
                                            want.as_str(),
                                            arg.value.vt.as_str()
                                        ),
                                        arg.value.span,
                                    );
                                }
                                args.push(arg.value);
                            }
                            let vt = match op.result {
                                crate::source::age_resources::AgeGfxResult::Port(name) => {
                                    match self.rec_index.get(name) {
                                        Some(&i) => VT::Rec(i),
                                        None => {
                                            self.bind_err(
                                                "source.age.gfx-port",
                                                format!(
                                                    "AGE port type `{name}` is not registered \
                                                     for this document"
                                                ),
                                                node_span,
                                            );
                                            VT::I32
                                        }
                                    }
                                }
                                crate::source::age_resources::AgeGfxResult::Scalar(vt) => vt,
                            };
                            self.used_intrinsics.insert(op.symbol);
                            out[dest] = ResolvedExpr {
                                span: node_span,
                                vt,
                                kind: RExprKind::CallIntrinsic {
                                    symbol: op.symbol,
                                    args,
                                },
                            };
                        }
                        RecvClass::Value => {
                            out[dest] = self
                                .method_call_out(rrecv, None, &name, rargs, node_span, explicit, expected);
                        }
                        RecvClass::Static(ri) => {
                            out[dest] = self.method_call_out(
                                rrecv,
                                Some(ri),
                                &name,
                                rargs,
                                node_span,
                                explicit,
                                expected,
                            );
                        }
                        RecvClass::ListCtor(li) => {
                            // PLAN-864: `List<T>.new()` — no receiver value;
                            // zero args (enforced at classification).
                            out[dest] = ResolvedExpr {
                                span: node_span,
                                vt: VT::List(li),
                                kind: RExprKind::ListNew { list: li },
                            };
                        }
                        RecvClass::ElemViewLen { xl, xvt, idx_slot } => {
                            // PLAN-864: `xs.get(i).len()` on List<Str> — the
                            // receiver is a whole-element view ( ElemView );
                            // the lowering fuses remove/view/strlen/insert.
                            let idx = std::mem::replace(&mut out[idx_slot], dummy_resolved());
                            out[dest] = ResolvedExpr {
                                span: node_span,
                                vt: VT::I32,
                                kind: RExprKind::Method {
                                    recv: Box::new(ResolvedExpr {
                                        span: node_span,
                                        vt: VT::StrOwned,
                                        kind: RExprKind::ElemView {
                                            list: Box::new(ResolvedExpr {
                                                span: node_span,
                                                vt: xvt,
                                                kind: RExprKind::ReadLocal(xl),
                                            }),
                                            index: Box::new(idx),
                                            path: Vec::new(),
                                        },
                                    }),
                                    method: crate::source::resolve::BuiltinMethod::Len,
                                    to: None,
                                    args: Vec::new(),
                                },
                            };
                        }
                        RecvClass::GenericEnum { template, types } => {
                            let hint = self.contextual_enum(template, types, expected);
                            out[dest] = self.probe_enum_constructor(
                                template, hint.as_deref().unwrap_or(types), &name, &rargs, node_span,
                            );
                        }
                        RecvClass::VariantArm { vi, disc } => {
                            // PLAN-864: `E.A(e)` / `E.A` — qualified variant
                            // construction; the payload (if any) is the
                            // single resolved argument. PLAN-1083: multi-
                            // element tuple payloads assemble a TupleLit;
                            // record payloads assemble the positional/named
                            // record constructor (reusing the record
                            // machinery — never a second representation).
                            let arm_payload_vt = self.coll.variants[vi].arms[disc as usize].1;
                            let named_arm = self.coll.named_field_arms.contains(&(vi, disc));
                            let payload = match (arm_payload_vt, rargs.len()) {
                                (_, 0) => None,
                                (Some(single_vt), 1)
                                    if !matches!(single_vt, VT::Tuple(_))
                                        && !(matches!(single_vt, VT::Rec(_)) && named_arm) =>
                                {
                                    let mut value = rargs.remove(0).value;
                                    retag_literal(&mut value, single_vt);
                                    Some(Box::new(value))
                                }
                                (Some(VT::Tuple(ti)), n) if n >= 1 => {
                                    let elem_vts: Vec<VT> = self
                                        .coll
                                        .tuples
                                        .get(ti)
                                        .cloned()
                                        .unwrap_or_default();
                                    if n == elem_vts.len() {
                                        let mut elems = Vec::with_capacity(n);
                                        for (i, mut ra) in rargs.drain(..).enumerate() {
                                            if let Some(evt) = elem_vts.get(i) {
                                                retag_literal(&mut ra.value, *evt);
                                            }
                                            elems.push(ra.value);
                                        }
                                        Some(Box::new(ResolvedExpr {
                                            span: node_span,
                                            vt: arm_payload_vt.unwrap(),
                                            kind: RExprKind::TupleLit { elems },
                                        }))
                                    } else {
                                        None // arity rejected at classification
                                    }
                                }
                                (Some(VT::Rec(ri)), n) if n >= 1 => {
                                    let rec_vt = VT::Rec(ri);
                                    let named = rargs.iter().any(|a| a.name.is_some());
                                    if named {
                                        let fields = self
                                            .records
                                            .get(ri)
                                            .map(|r| {
                                                r.fields.iter().map(|f| (f.name.clone(), f.vt)).collect::<Vec<_>>()
                                            })
                                            .unwrap_or_default();
                                        let mut pairs = Vec::with_capacity(n);
                                        for mut ra in rargs.drain(..) {
                                            let fname = ra
                                                .name
                                                .as_ref()
                                                .map(|x| x.name.clone())
                                                .unwrap_or_default();
                                            let evt = fields
                                                .iter()
                                                .find(|(n2, _)| *n2 == fname)
                                                .map(|(_, vt)| *vt);
                                            if let Some(evt) = evt {
                                                retag_literal(&mut ra.value, evt);
                                            }
                                            pairs.push((fname, ra.value));
                                        }
                                        Some(Box::new(ResolvedExpr {
                                            span: node_span,
                                            vt: rec_vt,
                                            kind: RExprKind::CtorNamed {
                                                rec: ri,
                                                fields: pairs,
                                            },
                                        }))
                                    } else {
                                        let elems_vt: Vec<VT> = self
                                            .records
                                            .get(ri)
                                            .map(|r| r.fields.iter().map(|f| f.vt).collect())
                                            .unwrap_or_default();
                                        if n == elems_vt.len() {
                                            for (i, ra) in rargs.iter_mut().enumerate() {
                                                if let Some(evt) = elems_vt.get(i).copied() {
                                                    retag_literal(&mut ra.value, evt);
                                                }
                                            }
                                            Some(Box::new(ResolvedExpr {
                                                span: node_span,
                                                vt: rec_vt,
                                                kind: RExprKind::CtorPos {
                                                    rec: ri,
                                                    args: rargs,
                                                },
                                            }))
                                        } else {
                                            None // arity rejected at classification
                                        }
                                    }
                                }
                                (_, _) => None, // rejected at classification
                            };
                            out[dest] = ResolvedExpr {
                                span: node_span,
                                vt: VT::Variant(vi),
                                kind: RExprKind::VariantCtor {
                                    variant: vi,
                                    disc,
                                    payload,
                                },
                            };
                        }
                    }
                }
                Task::CallOut {
                    callee,
                    explicit,
                    expected,
                    dest,
                    arg_slots,
                    names,
                    spans,
                    node_span,
                } => {
                    let mut rargs = Vec::with_capacity(arg_slots.len());
                    for (i, s) in arg_slots.iter().enumerate() {
                        let mut value = std::mem::replace(&mut out[*s], dummy_resolved());
                        // Record constructors and calls consume the explicit
                        // move expression. The text-only call schema carries
                        // the same transfer in its argument binding instead.
                        let moved = matches!(value.kind, RExprKind::Move { .. })
                            && !self.profile.is_record();
                        let value = if moved {
                            match std::mem::replace(&mut value.kind, RExprKind::IntLit(0)) {
                                RExprKind::Move { inner } => *inner,
                                _ => unreachable!("checked above"),
                            }
                        } else {
                            value
                        };
                        rargs.push(ResolvedArg {
                            span: spans[i],
                            name: names[i].clone(),
                            moved,
                            value,
                        });
                    }
                    // the node's own span already covers callee..
                    out[dest] = if explicit.is_some() || self.is_template_call(callee) {
                        self.probe_template_call(callee, rargs, explicit, node_span, expected)
                            .expect("explicit template call dispatched only in discovery")
                    } else {
                        self.resolve_call(callee, rargs, node_span)
                    };
                }
                Task::CallValueOut {
                    dest,
                    callee_slot,
                    arg_slots,
                    names,
                    spans,
                    node_span,
                } => {
                    let callee = std::mem::replace(&mut out[callee_slot], dummy_resolved());
                    let mut args = Vec::with_capacity(arg_slots.len());
                    for (index, slot) in arg_slots.iter().enumerate() {
                        let mut value = std::mem::replace(&mut out[*slot], dummy_resolved());
                        let moved = matches!(value.kind, RExprKind::Move { .. });
                        if moved {
                            value = match std::mem::replace(&mut value.kind, RExprKind::IntLit(0)) {
                                RExprKind::Move { inner } => *inner,
                                _ => unreachable!("checked above"),
                            };
                        }
                        args.push(ResolvedArg {
                            value,
                            name: names[index].clone(),
                            moved,
                            span: spans[index],
                        });
                    }
                    // PLAN-1026: a call through a Python object value —
                    // the direct py_call form (receiver = the callable).
                    if self.py_is_object(&callee) {
                        out[dest] = self.py_call_sugar(&callee, "", args, node_span);
                    } else {
                        out[dest] = self.resolve_closure_call(callee, args, node_span);
                    }
                }
                Task::MethodOut {
                    name,
                    dest,
                    recv_slot,
                    arg_slots,
                    names,
                    spans,
                    moved_flags,
                    node_span,
                    to_word,
                } => {
                    let recv = std::mem::replace(&mut out[recv_slot], dummy_resolved());
                    let mut rargs = Vec::with_capacity(arg_slots.len());
                    for (i, sl) in arg_slots.iter().enumerate() {
                        let mut value = std::mem::replace(&mut out[*sl], dummy_resolved());
                        // PLAN-782: in the combination profile the
                        // first-class move wrapper IS the argument value
                        // (CallBinding.moved stays text-profile only).
                        // PLAN-105 T-02（U03 真语料 013）：collections 域也要保留
                        // `.move` 实参标记——列表 `push(<整局部>)` 的所有权转移靠它
                        // （此前只有文本域保留，collections 被归一成 moved=false）。
                        let flagged = moved_flags.get(i).copied().unwrap_or(false);
                        let moved = (flagged || matches!(value.kind, RExprKind::Move { .. }))
                            && (self.profile.is_text()
                                || self.profile.is_collections()
                                || matches!(
                                    self.profile,
                                    SourceProfile::AgeComponent
                                        | SourceProfile::AgePackage
                                        | SourceProfile::AgeH0
                                ));
                        let value = if moved {
                            match std::mem::replace(&mut value.kind, RExprKind::IntLit(0)) {
                                RExprKind::Move { inner } => *inner,
                                _ => unreachable!("checked above"),
                            }
                        } else {
                            value
                        };
                        rargs.push(ResolvedArg {
                            span: spans[i],
                            name: names[i].clone(),
                            moved,
                            value,
                        });
                    }
                    out[dest] = self.resolve_method(&name, recv, rargs, to_word, node_span);
                }
                Task::Combine {
                    op,
                    span,
                    lslot,
                    rslot,
                    dest,
                } => {
                    let mut l = std::mem::replace(&mut out[lslot], dummy_resolved());
                    let mut r = std::mem::replace(&mut out[rslot], dummy_resolved());
                    if self.profile.is_auto_language()
                        && matches!(op, BinOp::Eq | BinOp::Ne)
                        && (l.vt == VT::Nil || r.vt == VT::Nil)
                    {
                        out[dest] = self.compare_null(l, r, op, span);
                        continue;
                    }
                    if self.profile.is_auto_language() && matches!(op, BinOp::Eq | BinOp::Ne) {
                        if let Some(guard) = self
                            .compare_empty_option(&l, &r, op, span)
                            .or_else(|| self.compare_empty_option(&r, &l, op, span))
                        {
                            out[dest] = guard;
                            continue;
                        }
                    }
                    if self.profile.is_auto_language()
                        && matches!(
                            op,
                            BinOp::Add
                                | BinOp::Sub
                                | BinOp::Mul
                                | BinOp::Div
                                | BinOp::Rem
                                | BinOp::Lt
                                | BinOp::Le
                                | BinOp::Gt
                                | BinOp::Ge
                                | BinOp::Eq
                                | BinOp::Ne
                        )
                    {
                        if r.vt == VT::I32 {
                            l = self.legacy_text_find_index(l);
                        }
                        if l.vt == VT::I32 {
                            r = self.legacy_text_find_index(r);
                        }
                    }
                    let script_arithmetic = self.coll.script_errors
                        && matches!(
                            op,
                            BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Rem
                        );
                    let type_error = |lhs, rhs| {
                        format!(
                            "TypeError: unsupported operand type(s) for {}: '{}' and '{}'",
                            op.as_str(),
                            Self::script_type_name(lhs),
                            Self::script_type_name(rhs),
                        )
                    };
                    if script_arithmetic && (l.vt == VT::Nil || r.vt == VT::Nil) {
                        let message = type_error(l.vt, r.vt);
                        self.eval_for_effect(l);
                        self.eval_for_effect(r);
                        out[dest] = self.script_type_error(message, VT::I32, span);
                        continue;
                    }
                    if script_arithmetic && (l.vt == VT::Dyn || r.vt == VT::Dyn) {
                        l = self.eval_once(l);
                        r = self.eval_once(r);
                        if l.vt == VT::Dyn {
                            self.guard_boxed_nil(l.clone(), type_error(VT::Nil, r.vt), span);
                        }
                        if r.vt == VT::Dyn {
                            self.guard_boxed_nil(r.clone(), type_error(l.vt, VT::Nil), span);
                        }
                    }
                    if l.vt == VT::Dyn && r.vt == VT::Dyn {
                        let ty = if matches!(op, BinOp::And | BinOp::Or) {
                            VT::Bool
                        } else {
                            VT::I32
                        };
                        l = Self::coerce_dynamic(l, ty);
                        r = Self::coerce_dynamic(r, ty);
                    } else if l.vt == VT::Dyn {
                        l = Self::coerce_dynamic(l, r.vt);
                    } else if r.vt == VT::Dyn {
                        r = Self::coerce_dynamic(r, l.vt);
                    }
                    // PLAN-964 (L-22): Eq/Ne over an owned text value and a
                    // string literal — the literal defaults to the borrow
                    // view `str`; retag it to the text side's type so the
                    // pair compares cleanly (`Str`/`StrOwned` == `str`).
                    // Collections-family only: the text profile keeps its
                    // frozen mixed Str/str rejection.
                    if matches!(op, BinOp::Eq | BinOp::Ne) && self.profile.is_collections() {
                        if l.vt.is_text() && matches!(r.kind, RExprKind::StrLit(_)) {
                            retag_literal(&mut r, l.vt);
                        } else if r.vt.is_text() && matches!(l.kind, RExprKind::StrLit(_)) {
                            retag_literal(&mut l, r.vt);
                        }
                    }
                    // PLAN-964 (L-21): Nil is a zero-width value — the only
                    // admitted operator pair is `==`/`!=` over two Nil
                    // operands (constant-folded: nil==nil is true). Any
                    // other operator on Nil is a named reject.
                    // PLAN-1026: a Python object operand routes the
                    // arithmetic through py_binop (the frozen v1 op set);
                    // a py object on the RIGHT only is a named v1 reject.
                    if self.py_is_object(&l) {
                        let opname = match op {
                            BinOp::Add => Some("add"),
                            BinOp::Sub => Some("sub"),
                            BinOp::Mul => Some("mul"),
                            BinOp::Div => Some("truediv"),
                            _ => None,
                        };
                        match opname {
                            Some(opname) => {
                                let mut parts: Vec<PyFramePart> =
                                    vec![PyFramePart::Txt("p967.frame/1\nr ".to_string())];
                                let chunk = self.py_chunk_call("o", &l);
                                parts.push(PyFramePart::TextExpr(chunk));
                                parts.push(PyFramePart::Txt("p 1\n".to_string()));
                                self.py_frame_val(&mut parts, &r);
                                parts.push(PyFramePart::Txt(format!(
                                    "k 1\n2\nopt {}\n{opname}\n",
                                    opname.len()
                                )));
                                let frame = self.py_frame(span, parts);
                                // PLAN-1107 T-02: binary operation fact.
                                if crate::py_ops::recording_active() {
                                    crate::py_ops::record(
                                        PyOpFact::new(PyOpKind::BinOp, span)
                                            .with_member(opname)
                                            .operand_lit(
                                                PyRole::Receiver,
                                                l.span,
                                                self.py_val_class(&l),
                                                self.py_operand_literal(&l),
                                            )
                                            .operand_lit(
                                                PyRole::Positional(0),
                                                r.span,
                                                self.py_val_class(&r),
                                                self.py_operand_literal(&r),
                                            ),
                                    );
                                }
                                out[dest] = self.py_op_call("py_binop", span, frame);
                                continue;
                            }
                            None => {
                                self.bind_err(
                                    "source.py-domain",
                                    "this operator on a Python object is outside the v1 op set (add/sub/mul/truediv)",
                                    span,
                                );
                                out[dest] = ResolvedExpr {
                                    span,
                                    vt: VT::I32,
                                    kind: RExprKind::ReadLocal(LocalRef(usize::MAX)),
                                };
                                continue;
                            }
                        }
                    } else if self.py_is_object(&r) {
                        self.bind_err(
                            "source.py-domain",
                            "a Python object is only admitted on the LEFT of an operator (v1)",
                            span,
                        );
                        out[dest] = ResolvedExpr {
                            span,
                            vt: VT::I32,
                            kind: RExprKind::ReadLocal(LocalRef(usize::MAX)),
                        };
                        continue;
                    }
                    if l.vt == VT::Nil || r.vt == VT::Nil {
                        let nil_eq = l.vt == VT::Nil && r.vt == VT::Nil;
                        if nil_eq {
                            match op {
                                BinOp::Eq | BinOp::Ne => {
                                    out[dest] = ResolvedExpr {
                                        span,
                                        vt: VT::Bool,
                                        kind: RExprKind::BoolLit(op == BinOp::Eq),
                                    };
                                    continue;
                                }
                                _ => {
                                    self.bind_err(
                                        "source.nil-op",
                                        format!(
                                            "operator `{}` is not defined for nil operands",
                                            op.as_str()
                                        ),
                                        span,
                                    );
                                }
                            }
                        } else {
                            self.bind_err(
                                "source.nil-op",
                                format!(
                                    "binary `{}` cannot mix nil with `{}`",
                                    op.as_str(),
                                    if l.vt == VT::Nil {
                                        r.vt.as_str()
                                    } else {
                                        l.vt.as_str()
                                    }
                                ),
                                span,
                            );
                        }
                    }
                    if op == BinOp::Add && self.coll.legacy_age_widget_at(span) {
                        let display_int = |operand: ResolvedExpr| ResolvedExpr {
                            span: operand.span,
                            vt: VT::StrOwned,
                            kind: RExprKind::TextDisplay { operand: Box::new(operand) },
                        };
                        if l.vt.is_text() && r.vt == VT::I32 {
                            r = display_int(r);
                        } else if r.vt.is_text() && l.vt == VT::I32 {
                            l = display_int(l);
                        }
                    }
                    if self.profile.is_auto_language()
                        && matches!(op, BinOp::Add | BinOp::Eq | BinOp::Ne)
                        && l.vt.is_text()
                        && r.vt.is_text()
                    {
                        l = Self::clone_text_element(l);
                        r = Self::clone_text_element(r);
                    }
                    if matches!(op, BinOp::Eq | BinOp::Ne)
                        && self.coll.legacy_age_widget_at(span)
                        && ((l.vt == VT::I32 && matches!(r.kind, RExprKind::StrLit(_)))
                            || (r.vt == VT::I32 && matches!(l.kind, RExprKind::StrLit(_))))
                    {
                        self.eval_for_effect(l);
                        self.eval_for_effect(r);
                        out[dest] = ResolvedExpr {
                            span,
                            vt: VT::Bool,
                            kind: RExprKind::BoolLit(op == BinOp::Ne),
                        };
                        continue;
                    }
                    let vt = bin_op_vt(op);
                    if matches!((l.vt, r.vt), (VT::I32, VT::F32) | (VT::F32, VT::I32))
                        && matches!(
                            op,
                            BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Rem
                                | BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge | BinOp::Eq | BinOp::Ne
                        )
                        && self.coll.legacy_age_widget_at(span)
                    {
                        if l.vt == VT::I32 {
                            l = self.coll.age_widget_numeric_cast(l, VT::F32);
                        } else {
                            r = self.coll.age_widget_numeric_cast(r, VT::F32);
                        }
                    }
                    // PLAN-961: the typed numeric surface — same-type
                    // operands (no implicit promotion), comparisons → bool,
                    // bit ops int-only; the result carries the operand type.
                    // The typed branch fires only when a NON-I32 numeric
                    // participates (pure i32/bool pairs keep the legacy
                    // walk and its frozen diagnostics).
                    let typed_num_present = vt_is_numeric(l.vt) && !matches!(l.vt, VT::I32)
                        || vt_is_numeric(r.vt) && !matches!(r.vt, VT::I32);
                    let vt = if typed_num_present {
                        let is_cmp = matches!(
                            op,
                            BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge | BinOp::Eq | BinOp::Ne
                        );
                        if l.vt != r.vt || !vt_is_numeric(l.vt) || !vt_is_numeric(r.vt) {
                            self.bind_err(
                                "source.type-mismatch",
                                format!(
                                    "binary `{}` requires same-type numeric operands ({} vs {}); mixed widths need an explicit conversion",
                                    op.as_str(),
                                    l.vt.as_str(),
                                    r.vt.as_str()
                                ),
                                span,
                            );
                            if is_cmp {
                                VT::Bool
                            } else {
                                l.vt
                            }
                        } else if matches!(op, BinOp::BitAnd | BinOp::BitOr | BinOp::BitXor)
                            && vt_is_float(l.vt)
                        {
                            self.bind_err(
                                "source.type-mismatch",
                                format!(
                                    "bit operator `{}` is not defined for float operands",
                                    op.as_str()
                                ),
                                span,
                            );
                            l.vt
                        } else if is_cmp {
                            VT::Bool
                        } else {
                            l.vt
                        }
                    } else if op == BinOp::Add && l.vt.is_text() && r.vt.is_text() {
                        // PLAN-770: `+` over two text operands is
                        // concatenation (owned result).
                        VT::StrOwned
                    } else {
                        vt
                    };
                    let vt = self.node_ty(span, vt);
                    out[dest] = ResolvedExpr {
                        span,
                        vt,
                        kind: RExprKind::Binary {
                            op,
                            lhs: Box::new(l),
                            rhs: Box::new(r),
                        },
                    };
                }
                Task::UnaryOut {
                    op,
                    span,
                    oslot,
                    dest,
                } => {
                    let o = std::mem::replace(&mut out[oslot], dummy_resolved());
                    if op == UnaryOp::Neg && o.vt == VT::Nil && self.profile.is_auto_language() {
                        if self.coll.script_errors {
                            self.eval_for_effect(o);
                            out[dest] = self.script_type_error(
                                "TypeError: bad operand type for unary -: 'NoneType'".to_string(),
                                VT::I32,
                                span,
                            );
                        } else {
                            self.bind_err(
                                "source.nil-op",
                                "unary `-` is not defined for nil",
                                span,
                            );
                            out[dest] = self.dummy_of(VT::I32, span);
                        }
                        continue;
                    }
                    let o = if op == UnaryOp::Not {
                        self.condition_value(o)
                    } else {
                        o
                    };
                    // PLAN-961: typed negation — signed checked / float
                    // sign-flip; `-lit` folds into the payload bits (an
                    // in-range positive literal never overflows on fold;
                    // unsigned negation is a named rejection).
                    if op == UnaryOp::Neg && vt_is_numeric(o.vt) {
                        if vt_is_float(o.vt) {
                            if let RExprKind::TypedLit { vt, bits } = o.kind {
                                let flipped = match vt {
                                    VT::F32 => ((bits as u32) ^ 0x8000_0000) as u64,
                                    VT::F64 => bits ^ 0x8000_0000_0000_0000,
                                    other => unreachable!("float fold on {other:?}"),
                                };
                                out[dest] = ResolvedExpr {
                                    span,
                                    vt,
                                    kind: RExprKind::TypedLit { vt, bits: flipped },
                                };
                                continue;
                            }
                        } else if matches!(o.vt, VT::I8 | VT::I32 | VT::I64) {
                            if let RExprKind::TypedLit { vt, bits } = o.kind {
                                let folded = match vt {
                                    VT::I8 => (bits as u8).wrapping_neg() as u64,
                                    VT::I32 => (bits as u32).wrapping_neg() as u64,
                                    VT::I64 => (bits as u64).wrapping_neg(),
                                    other => unreachable!("int fold on {other:?}"),
                                };
                                out[dest] = ResolvedExpr {
                                    span,
                                    vt,
                                    kind: RExprKind::TypedLit { vt, bits: folded },
                                };
                                continue;
                            }
                        } else {
                            self.bind_err(
                                "source.type-mismatch",
                                format!("unary `-` is not defined for unsigned {}", o.vt.as_str()),
                                span,
                            );
                        }
                        let vt = self.node_ty(span, o.vt);
                        out[dest] = ResolvedExpr {
                            span,
                            vt,
                            kind: RExprKind::Unary {
                                op,
                                operand: Box::new(o),
                            },
                        };
                        continue;
                    }
                    let vt = match op {
                        UnaryOp::Neg => VT::I32,
                        UnaryOp::Not => VT::Bool,
                    };
                    let vt = self.node_ty(span, vt);
                    out[dest] = ResolvedExpr {
                        span,
                        vt,
                        kind: RExprKind::Unary {
                            op,
                            operand: Box::new(o),
                        },
                    };
                }
            }
        }
        std::mem::replace(&mut out[0], dummy_resolved())
    }

    /// Bind a builtin text/char method (PLAN-770): receiver-type table
    /// lookup, the type-level `char.from_codepoint` form, `.to` type-word
    /// conversion, and the bare `.move` wrapper. User names never shadow
    /// the builtin table and the builtins never hijack user functions
    /// (distinct namespaces).
    /// Preserve the integer returned by a text search before its Option
    /// is constructed. Both uses observe the one checked search.
    fn cache_text_find_index(&mut self, value: &mut ResolvedExpr) -> Option<LocalRef> {
        let RExprKind::CallFn { fn_index, args } = &value.kind else {
            return None;
        };
        let find = self.sigs.get(super::adapter::STDLIB_TEXT_FIND)?;
        if find.index != *fn_index {
            return None;
        }
        let wrap = self.sigs.get("__av_text_find_option")?.index;
        let span = value.span;
        let mut scan_args: Vec<_> = args.iter().map(|arg| arg.value.clone()).collect();
        scan_args.push(ResolvedExpr {
            span,
            vt: VT::I32,
            kind: RExprKind::IntLit(0),
        });
        let index = self.agg_temp("find_index", VT::I32, false, span);
        self.used_intrinsics.insert("__ace_text_find");
        self.emit_stmt(RStmt::Let {
            local: index,
            value: ResolvedExpr {
                span,
                vt: VT::I32,
                kind: RExprKind::CallIntrinsic {
                    symbol: "__ace_text_find",
                    args: scan_args,
                },
            },
            span,
        });
        value.kind = RExprKind::CallFn {
            fn_index: wrap,
            args: vec![ResolvedArg {
                span,
                name: None,
                moved: false,
                value: ResolvedExpr {
                    span,
                    vt: VT::I32,
                    kind: RExprKind::ReadLocal(index),
                },
            }],
        };
        Some(index)
    }

    /// Legacy text find returns -1 in integer expressions. Pattern
    /// matching and explicit typed storage continue to use the Option.
    fn legacy_text_find_index(&mut self, value: ResolvedExpr) -> ResolvedExpr {
        let (VT::Variant(variant), RExprKind::ReadLocal(source)) = (value.vt, &value.kind) else {
            return value;
        };
        let Some(&index) = self.legacy_text_find_results.get(source) else {
            return value;
        };
        let span = value.span;
        // The numeric read still checks liveness of its option binding,
        // so a prior move cannot be hidden by the cached scalar.
        self.eval_for_effect(ResolvedExpr {
            span,
            vt: VT::Bool,
            kind: RExprKind::VariantGuardTest {
                local: *source,
                variant,
                disc: 0,
            },
        });
        ResolvedExpr {
            span,
            vt: VT::I32,
            kind: RExprKind::ReadLocal(index),
        }
    }

    fn resolve_std_text_method(
        &mut self,
        name: &ast::Ident,
        recv: ResolvedExpr,
        args: Vec<ResolvedArg>,
        span: Span,
    ) -> ResolvedExpr {
        if name.name == "split" {
            if let (
                RExprKind::StrLit(text),
                [ResolvedArg {
                    name: None,
                    moved: false,
                    value:
                        ResolvedExpr {
                            kind: RExprKind::StrLit(separator),
                            ..
                        },
                    ..
                }],
            ) = (&recv.kind, args.as_slice())
            {
                // Freeze only the literal split boundaries. The list and
                // every owned string still allocate at the original site.
                const MAX_FOLDED_SPLIT_PARTS: usize = 32;
                let parts: Vec<_> = if separator.is_empty() {
                    vec![text.as_str()]
                } else {
                    text.split(separator.as_str())
                        .take(MAX_FOLDED_SPLIT_PARTS + 1)
                        .collect()
                };
                if parts.len() <= MAX_FOLDED_SPLIT_PARTS {
                    let elements: Vec<_> = parts
                        .into_iter()
                        .map(|part| Expr {
                            span,
                            kind: ExprKind::StrLit(part.into()),
                        })
                        .collect();
                    return self.resolve_array_literal(&elements, span, false, None);
                }
            }
        }
        let mangled = format!("{}{}", super::adapter::STDLIB_TEXT_MANGLE, name.name);
        let Some(sig) = self.sigs.get(&mangled) else {
            self.bind_err(
                "source.undefined",
                "the stdlib text method is not linked",
                name.span,
            );
            return ResolvedExpr {
                span,
                vt: VT::Unit,
                kind: RExprKind::Unit,
            };
        };
        let fn_index = sig.index;
        let result = sig.result;
        if args.len() + 1 != sig.params.len()
            || args
                .iter()
                .zip(sig.params.iter().skip(1))
                .any(|(arg, expected)| {
                    arg.moved
                        || arg.name.is_some()
                        || (arg.value.vt != *expected
                            && !(expected.is_text() && arg.value.vt.is_text()))
                })
        {
            self.bind_err(
                "source.type-mismatch",
                format!(
                    "text method {} expects {} positional argument(s)",
                    name.name,
                    sig.params.len() - 1
                ),
                span,
            );
        }
        let recv = ResolvedArg {
            span: recv.span,
            value: recv,
            name: None,
            moved: false,
        };
        let args: Vec<ResolvedArg> = std::iter::once(recv)
            .chain(args)
            .enumerate()
            .map(|(index, mut arg)| {
                // These methods read their inputs. The assembled helpers take
                // ownership, so local owners and element views supply copies.
                let value = arg.value;
                arg.value = if sig.params.get(index) == Some(&VT::Str) {
                    if value.vt == VT::StrOwned {
                        // A slice materialized only to feed a read-only
                        // operation can be passed straight through as a view.
                        if let RExprKind::Method {
                            method: BuiltinMethod::To,
                            to: Some(VT::StrOwned),
                            recv,
                            args,
                        } = &value.kind
                        {
                            if args.is_empty()
                                && matches!(
                                    recv.kind,
                                    RExprKind::Method {
                                        method: BuiltinMethod::Slice,
                                        ..
                                    }
                                )
                            {
                                arg.value = (**recv).clone();
                                return arg;
                            }
                        }
                        let value = Self::clone_text_element(value);
                        ResolvedExpr {
                            span: value.span,
                            vt: VT::Str,
                            kind: RExprKind::Method {
                                recv: Box::new(value),
                                method: BuiltinMethod::To,
                                to: Some(VT::Str),
                                args: Vec::new(),
                            },
                        }
                    } else {
                        value
                    }
                } else {
                    match value.vt {
                        VT::Str => Self::own_dynamic_text(value),
                        VT::StrOwned
                            if matches!(
                                value.kind,
                                RExprKind::ReadLocal(_)
                                    | RExprKind::FieldGet { .. }
                                    | RExprKind::ElemView { .. }
                            ) =>
                        {
                            Self::clone_owned_text(value)
                        }
                        _ => value,
                    }
                };
                arg
            })
            .collect();
        if name.name == "trim" {
            self.used_intrinsics.insert("__ace_text_trim");
            return ResolvedExpr {
                span,
                vt: VT::StrOwned,
                kind: RExprKind::CallIntrinsic {
                    symbol: "__ace_text_trim",
                    args: args.into_iter().map(|arg: ResolvedArg| arg.value).collect(),
                },
            };
        }
        ResolvedExpr {
            span,
            vt: result,
            kind: RExprKind::CallFn { fn_index, args },
        }
    }

    fn resolve_text_contains(
        &mut self,
        text: ResolvedExpr,
        mut args: Vec<ResolvedArg>,
        span: Span,
    ) -> ResolvedExpr {
        if args.len() != 1 || args[0].name.is_some() || args[0].moved || !args[0].value.vt.is_text() {
            self.bind_err("source.type-mismatch", "text `contains` expects one positional text argument", span);
            return ResolvedExpr { span, vt: VT::Bool, kind: RExprKind::BoolLit(false) };
        }
        let text_view = |value: ResolvedExpr| {
            if value.vt == VT::StrOwned {
                ResolvedExpr {
                    span: value.span,
                    vt: VT::Str,
                    kind: RExprKind::Method {
                        recv: Box::new(Self::clone_text_element(value)),
                        method: BuiltinMethod::To,
                        to: Some(VT::Str),
                        args: Vec::new(),
                    },
                }
            } else {
                value
            }
        };
        self.used_intrinsics.insert("__ace_text_find");
        let zero = ResolvedExpr { span, vt: VT::I32, kind: RExprKind::IntLit(0) };
        ResolvedExpr {
            span,
            vt: VT::Bool,
            kind: RExprKind::Binary {
                op: BinOp::Ge,
                lhs: Box::new(ResolvedExpr {
                    span,
                    vt: VT::I32,
                    kind: RExprKind::CallIntrinsic {
                        symbol: "__ace_text_find",
                        args: vec![text_view(text), text_view(args.remove(0).value), zero.clone()],
                    },
                }),
                rhs: Box::new(zero),
            },
        }
    }

    fn age_widget_to_int(&mut self, recv: ResolvedExpr, span: Span) -> ResolvedExpr {
        // The authored VM conversion truncates and clamps to i32, with NaN
        // giving zero. Guard the existing checked cast with those bounds.
        let value = self.eval_once(recv);
        let result = self.agg_temp("legacy_to_int", VT::I32, true, span);
        let integer = |n| ResolvedExpr {
            span,
            vt: VT::I32,
            kind: RExprKind::IntLit(n),
        };
        let compare = |op, rhs| ResolvedExpr {
            span,
            vt: VT::Bool,
            kind: RExprKind::Binary {
                op,
                lhs: Box::new(value.clone()),
                rhs: Box::new(rhs),
            },
        };
        let float = |n: f32| ResolvedExpr {
            span,
            vt: VT::F32,
            kind: RExprKind::TypedLit {
                vt: VT::F32,
                bits: u64::from(n.to_bits()),
            },
        };
        let assign = |value| RStmt::Assign {
            local: result,
            path: Vec::new(),
            value,
            span,
        };
        let body = |stmt| RBlock {
            stmts: vec![stmt],
            span,
        };
        let converted = self.coll.age_widget_numeric_cast(value.clone(), VT::I32);
        self.emit_stmt(RStmt::Let {
            local: result,
            value: integer(0),
            span,
        });
        self.emit_stmt(RStmt::If {
            cond: compare(BinOp::Eq, value.clone()),
            then: body(RStmt::If {
                cond: compare(BinOp::Ge, float(2147483648.0)),
                then: body(assign(integer(i32::MAX))),
                els: Some(body(RStmt::If {
                    cond: compare(BinOp::Le, float(-2147483648.0)),
                    then: body(assign(integer(i32::MIN))),
                    els: Some(body(assign(converted))),
                    span,
                })),
                span,
            }),
            els: None,
            span,
        });
        ResolvedExpr {
            span,
            vt: VT::I32,
            kind: RExprKind::ReadLocal(result),
        }
    }

    fn resolve_method(
        &mut self,
        name: &ast::Ident,
        recv: ResolvedExpr,
        args: Vec<ResolvedArg>,
        to_word: Option<TypeWord>,
        span: Span,
    ) -> ResolvedExpr {
        if self.profile.is_auto_language() {
            if name.name == "len" && to_word.is_none() && args.is_empty() {
                if let RExprKind::CallIntrinsic {
                    symbol: "__ace_text_trim",
                    args,
                } = &recv.kind
                {
                    self.used_intrinsics.insert("__ace_text_trim_len");
                    return ResolvedExpr {
                        span,
                        vt: VT::I32,
                        kind: RExprKind::CallIntrinsic {
                            symbol: "__ace_text_trim_len",
                            args: args.clone(),
                        },
                    };
                }
            }
            if recv.vt.is_text() && to_word.is_none() {
                if name.name == "to_upper" && args.is_empty() {
                    let mut view = recv;
                    self.adapt_argument_type(&mut view, VT::Str);
                    self.used_intrinsics.insert("__ace_text_upper");
                    return ResolvedExpr {
                        span,
                        vt: VT::StrOwned,
                        kind: RExprKind::CallIntrinsic {
                            symbol: "__ace_text_upper",
                            args: vec![view],
                        },
                    };
                }
                if name.name == "contains" {
                    return self.resolve_text_contains(recv, args, span);
                }
                if matches!(
                    name.name.as_str(), "find" | "trim" | "split" | "substr" | "ends_with",
                ) {
                    return self.resolve_std_text_method(name, recv, args, span);
                }
                let suffix = name.name == "slice" && args.len() == 1;
                if name.name == "sub" || suffix {
                    let (recv, args) = if suffix {
                        let recv = self.eval_once(recv);
                        let len = ResolvedExpr {
                            span,
                            vt: VT::I32,
                            kind: RExprKind::Method {
                                recv: Box::new(recv.clone()),
                                method: BuiltinMethod::Len,
                                to: None,
                                args: Vec::new(),
                            },
                        };
                        let mut args = args;
                        args.push(ResolvedArg {
                            span,
                            value: len,
                            name: None,
                            moved: false,
                        });
                        (recv, args)
                    } else {
                        (recv, args)
                    };
                    let slice = ast::Ident {
                        name: "slice".into(),
                        span: name.span,
                    };
                    let value = self.resolve_method(&slice, recv, args, None, span);
                    return Self::own_dynamic_text(value);
                }
            }
            if recv.vt.is_text()
                && name.name == "to"
                && to_word.is_some_and(|word| vt_of_word(word) == VT::I32)
                && self.sigs.contains_key(super::adapter::STDLIB_TEXT_TO_INT)
            {
                let callee = ast::Ident {
                    name: super::adapter::STDLIB_TEXT_TO_INT.into(),
                    span: name.span,
                };
                let value_span = recv.span;
                return self.resolve_call(
                    &callee,
                    vec![ResolvedArg {
                        name: None,
                        value: recv,
                        moved: false,
                        span: value_span,
                    }],
                    span,
                );
            }
            if recv.vt == VT::I32
                && !self.py_is_object(&recv)
                && ((name.name == "to" && to_word.is_some_and(|word| vt_of_word(word).is_text()))
                    || (name.name == "to_str" && args.is_empty() && to_word.is_none()))
            {
                return ResolvedExpr {
                    span,
                    vt: VT::StrOwned,
                    kind: RExprKind::TextDisplay {
                        operand: Box::new(recv),
                    },
                };
            }
            if recv.vt == VT::Dyn && matches!(name.name.as_str(), "as" | "to") {
                if let Some(word) = to_word {
                    return Self::coerce_dynamic(recv, vt_of_word(word));
                }
            }
            if recv.vt == VT::Nil && matches!(name.name.as_str(), "as" | "to") {
                if let Some(word) = to_word {
                    let target = vt_of_word(word);
                    if target.is_text() {
                        self.eval_for_effect(recv);
                        return ResolvedExpr {
                            span,
                            vt: target,
                            kind: RExprKind::StrLit("None".into()),
                        };
                    }
                    if self.coll.script_errors && matches!(target, VT::I32 | VT::F32 | VT::F64) {
                        let name = Self::script_type_name(target);
                        self.eval_for_effect(recv);
                        return self.script_type_error(
                            format!("TypeError: {name}() argument must be a string or a real number, not 'NoneType'"),
                            target,
                            span,
                        );
                    }
                }
            }
            if self.coll.script_errors
                && recv.vt == VT::Nil
                && name.name == "len"
                && args.is_empty()
            {
                self.eval_for_effect(recv);
                return self.script_type_error(
                    "TypeError: 'NoneType' object is not iterable".to_string(),
                    VT::I32,
                    span,
                );
            }
        }
        // PLAN-1026: every method-position operation on a Python object
        // routes through the frozen op set — `.to(T)` is the explicit
        // scalar/text extraction; any other name is a member call.
        if self.py_is_object(&recv) {
            if name.name == "to" || name.name == "as" {
                let target = to_word.map(super::resolve::vt_of_word).unwrap_or(VT::I32);
                return self.py_to_sugar(&recv, target, span);
            }
            // PLAN-1094: `.len()` on a Python object is Python's builtin
            // `len(obj)` — the `__len__` protocol (the legacy obj_len
            // combinator's ordinary-AV form), not an attribute call.
            if name.name == "len" && args.is_empty() {
                return self.py_call_sugar(&recv, "__len__", args, span);
            }
            if to_word.is_some() {
                self.bind_err(
                    "source.py-domain",
                    "a type-word argument is only legal on `.to(...)` over a Python object",
                    span,
                );
            }
            return self.py_call_sugar(&recv, &name.name, args, span);
        }
        // bare `.move`: ownership marker, only legal on a named local and
        // only as a direct call argument (the call arms unwrap it)
        if name.name == "move" && to_word.is_none() {
            if !args.is_empty() {
                self.bind_err("source.syntax", "`.move` takes no arguments", span);
            }
            // On the combination surface the wrapper stays first-class:
            // `.move` on a field path / sub-expression flows into verify
            // as `verify.partial-move` (the frozen single oracle); the
            // plain text profile keeps its 770 front-door rejection.
            if !matches!(recv.kind, RExprKind::ReadLocal(_))
                && self.profile != SourceProfile::TextRecord
            {
                self.bind_err(
                    "source.unsupported",
                    "`.move` requires a named local (bind the value to a local first)",
                    span,
                );
            }
            let vt = recv.vt;
            return ResolvedExpr {
                span,
                vt,
                kind: RExprKind::Move {
                    inner: Box::new(recv),
                },
            };
        }
        // type-level form: `char.from_codepoint(n)` — the parser rewrote
        // the bare `char` receiver into a TypeWord marker, so a local
        // actually named `char` can never be hijacked
        if name.name == "from_codepoint" && matches!(recv.kind, RExprKind::TypeWord(TypeWord::Char))
        {
            if args.len() != 1 || args[0].name.is_some() || args[0].moved {
                self.bind_err(
                    "source.type-mismatch",
                    "`from_codepoint` expects one positional i32 argument",
                    span,
                );
            } else if args[0].value.vt != VT::I32 {
                self.bind_err(
                    "source.type-mismatch",
                    format!(
                        "argument of `from_codepoint` must be i32 (found {})",
                        args[0].value.vt.as_str()
                    ),
                    args[0].span,
                );
            }
            return ResolvedExpr {
                span,
                vt: VT::Char,
                kind: RExprKind::Method {
                    recv: Box::new(recv),
                    method: BuiltinMethod::FromCodepoint,
                    to: None,
                    args,
                },
            };
        }
        // PLAN-864: List receivers route to the collections method table.
        if let VT::List(li) = recv.vt {
            return self.resolve_list_method(li, name, recv, args, span);
        }
        // PLAN-1083: `opt.or(default)` — the legacy Option method with
        // `.?(default)` semantics (Some projects, None takes the default;
        // the default evaluates only on the None path).
        if name.name == "or" && args.len() == 1 && self.profile.is_aggregates() {
            if let VT::Variant(_) = recv.vt {
                if let Some(info) = agg_lower::variant_unwrap_info(recv.vt, self.coll) {
                    if self.coll.variants[info.vi].name == "Option" {
                        let fallback = args[0].value.clone();
                        let scr_l = self.agg_temp("or_scr", recv.vt, false, span);
                        self.emit_stmt(RStmt::Let {
                            local: scr_l,
                            value: recv,
                            span,
                        });
                        let res_l = self.agg_temp("or_res", info.success_vt, true, span);
                        let dummy = self.dummy_of(info.success_vt, span);
                        self.emit_stmt(RStmt::Let {
                            local: res_l,
                            value: dummy,
                            span,
                        });
                        self.emit_stmt(RStmt::If {
                            cond: ResolvedExpr {
                                span,
                                vt: VT::Bool,
                                kind: RExprKind::VariantGuardTest {
                                    local: scr_l,
                                    variant: info.vi,
                                    disc: info.success_disc,
                                },
                            },
                            then: RBlock {
                                stmts: vec![RStmt::Assign {
                                    local: res_l,
                                    path: Vec::new(),
                                    value: ResolvedExpr {
                                        span,
                                        vt: info.success_vt,
                                        kind: RExprKind::VariantProject {
                                            local: scr_l,
                                            variant: info.vi,
                                            disc: info.success_disc,
                                        },
                                    },
                                    span,
                                }],
                                span,
                            },
                            els: Some(RBlock {
                                stmts: vec![RStmt::Assign {
                                    local: res_l,
                                    path: Vec::new(),
                                    value: fallback,
                                    span,
                                }],
                                span,
                            }),
                            span,
                        });
                        return ResolvedExpr {
                            span,
                            vt: info.success_vt,
                            kind: RExprKind::ReadLocal(res_l),
                        };
                    }
                }
            }
        }
        if let VT::Function(signature) = recv.vt {
            if name.name == "clone" && args.is_empty() && self.profile.is_closures() {
                return ResolvedExpr {
                    span,
                    vt: VT::Function(signature),
                    kind: RExprKind::Method {
                        recv: Box::new(recv),
                        method: BuiltinMethod::ClosureClone,
                        to: None,
                        args: Vec::new(),
                    },
                };
            }
        }
        // PLAN-961: numeric-receiver builtins — shifts take one
        // same-type amount; `.as(T)`/`.to(T)` convert to the numeric
        // target word (frozen matrix; checked narrowing at runtime).
        if vt_is_numeric(recv.vt) {
            match name.name.as_str() {
                "to_int"
                    if recv.vt == VT::F32
                        && args.is_empty()
                        && to_word.is_none()
                        && self.coll.legacy_age_widget_at(span) =>
                {
                    return self.age_widget_to_int(recv, span);
                }
                "shl" | "shr" => {
                    if args.len() != 1 {
                        self.bind_err(
                            "source.syntax",
                            format!("`.{}()` takes exactly one amount argument", name.name),
                            span,
                        );
                    }
                    let amount_vt = args.first().map(|a| a.value.vt).unwrap_or(recv.vt);
                    if amount_vt != recv.vt {
                        self.bind_err(
                            "source.type-mismatch",
                            format!(
                                "`.{}()` amount must have the receiver's type {} (got {})",
                                name.name,
                                recv.vt.as_str(),
                                amount_vt.as_str()
                            ),
                            span,
                        );
                    }
                    let method = if name.name == "shl" {
                        BuiltinMethod::Shl
                    } else {
                        BuiltinMethod::Shr
                    };
                    let vt = self.node_ty(span, recv.vt);
                    return ResolvedExpr {
                        span,
                        vt,
                        kind: RExprKind::Method {
                            recv: Box::new(recv),
                            method,
                            to: None,
                            args,
                        },
                    };
                }
                "as" | "to" => {
                    let Some(word) = to_word else {
                        self.bind_err(
                            "source.syntax",
                            format!("`.{}()` takes a numeric type word argument", name.name),
                            span,
                        );
                        let rv = recv.vt;
                        return ResolvedExpr {
                            span,
                            vt: rv,
                            kind: RExprKind::Method {
                                recv: Box::new(recv),
                                method: BuiltinMethod::NumTo,
                                to: Some(rv),
                                args,
                            },
                        };
                    };
                    let target = vt_of_word(word);
                    if !vt_is_numeric(target) {
                        self.bind_err(
                            "source.type-mismatch",
                            format!(
                                "`.{}({})` target is not a numeric type word",
                                name.name,
                                word_text(word)
                            ),
                            span,
                        );
                    }
                    let vt = self.node_ty(span, target);
                    return ResolvedExpr {
                        span,
                        vt,
                        kind: RExprKind::Method {
                            recv: Box::new(recv),
                            method: BuiltinMethod::NumTo,
                            to: Some(target),
                            args,
                        },
                    };
                }
                other => {
                    self.bind_err(
                        "source.unsupported",
                        format!("numeric receiver has no method `{other}` in this profile"),
                        span,
                    );
                }
            }
        }
        let Some((method, params, _result)) = builtin_method(recv.vt, &name.name) else {
            self.bind_err(
                "source.undefined",
                format!(
                    "no builtin method `{}` on receiver type {} in this profile",
                    name.name,
                    recv.vt.as_str()
                ),
                span,
            );
            return ResolvedExpr {
                span,
                vt: VT::I32,
                kind: RExprKind::IntLit(0),
            };
        };
        if method == BuiltinMethod::To {
            let Some(w) = to_word else {
                self.bind_err(
                    "source.syntax",
                    "`.to` expects a single type argument: `.to(str)` or `.to(Str)`",
                    span,
                );
                return ResolvedExpr {
                    span,
                    vt: VT::Str,
                    kind: RExprKind::StrLit(String::new()),
                };
            };
            let target = vt_of_word(w);
            if !target.is_text() {
                self.bind_err(
                    "source.type-mismatch",
                    "`.to` converts between str and Str only",
                    span,
                );
            }
            // PLAN-975 B1: a char receiver has no source buffer to borrow —
            // char `.to` is the owned materialization only.
            if recv.vt == VT::Char && target != VT::StrOwned {
                self.bind_err(
                    "source.type-mismatch",
                    "char `.to` materializes an owned Str: use `.to(Str)`",
                    span,
                );
            }
            if args.len() > 1 {
                self.bind_err("source.syntax", "`.to` takes a single type argument", span);
            }
            return ResolvedExpr {
                span,
                vt: target,
                kind: RExprKind::Method {
                    recv: Box::new(recv),
                    method: BuiltinMethod::To,
                    to: Some(target),
                    args,
                },
            };
        }
        if to_word.is_some() {
            self.bind_err(
                "source.syntax",
                "a type word is only allowed inside `.to(...)`",
                span,
            );
        }
        // positional arity + type check against the frozen table
        if args.len() != params.len() || args.iter().any(|a| a.name.is_some() || a.moved) {
            self.bind_err(
                "source.type-mismatch",
                format!(
                    "method `{}` expects {} positional argument(s)",
                    name.name,
                    params.len()
                ),
                span,
            );
        } else {
            for (i, a) in args.iter().enumerate() {
                if a.value.vt != params[i] {
                    self.bind_err(
                        "source.type-mismatch",
                        format!(
                            "argument {} of `{}` must be {} (found {})",
                            i + 1,
                            name.name,
                            params[i].as_str(),
                            a.value.vt.as_str()
                        ),
                        a.span,
                    );
                }
            }
        }
        let result = match method {
            BuiltinMethod::Len | BuiltinMethod::LenBytes | BuiltinMethod::ToCodepoint => VT::I32,
            BuiltinMethod::CharAt | BuiltinMethod::FromCodepoint => VT::Char,
            BuiltinMethod::Slice => VT::Str,
            BuiltinMethod::Clone => VT::StrOwned,
            BuiltinMethod::ClosureClone => recv.vt,
            BuiltinMethod::To => VT::Str,
            // Unreachable: numeric methods return before the table walk.
            BuiltinMethod::Shl | BuiltinMethod::Shr | BuiltinMethod::NumTo => recv.vt,
        };
        ResolvedExpr {
            span,
            vt: result,
            kind: RExprKind::Method {
                recv: Box::new(recv),
                method,
                to: None,
                args,
            },
        }
    }

    /// PLAN-936（T-04）：meta 查询内在的绑定（ct_field_count/ct_has_field）。
    /// 类型/字段串是 query selector（数据），绝不作为运行时值解析；全部
    /// 拒绝精确且具名（§5.2 “缺cap/缺record/字段串非字面量/错误位置的
    /// 拒绝明确且配合法近邻”）。
    fn meta_query(
        &mut self,
        op: &'static str,
        callee: &ast::Ident,
        args: &[Arg],
        call_span: Span,
    ) -> ResolvedExpr {
        let bad = |r: &mut Self, code: &str, msg: String, span: Span| -> ResolvedExpr {
            r.bind_err(code, msg, span);
            ResolvedExpr {
                span,
                vt: VT::I32,
                kind: RExprKind::IntLit(0),
            }
        };
        if !self.meta_enabled {
            return bad(
                self,
                "source.cte.meta-capability",
                format!(
                    "`{}` requires the cte.meta.fields/v1 capability (meta queries default to closed)",
                    callee.name
                ),
                call_span,
            );
        }
        if self.ct_depth == 0 {
            return bad(
                self,
                "source.cte.meta-outside-ct",
                format!(
                    "`{}` is a CT meta query — it is only legal inside a `#{{ ... }}` compile-time expression",
                    callee.name
                ),
                call_span,
            );
        }
        // arity: field_count 1 / has_field 2；参数必须位置式（selector 位）。
        let want = if op == "cte.meta.field_count" { 1 } else { 2 };
        let positional: Vec<&Expr> = args
            .iter()
            .filter_map(|a| match a {
                Arg::Pos(e) | Arg::MovePos(e) => Some(e),
                Arg::Named { .. } => None,
            })
            .collect();
        if positional.len() != want || positional.len() != args.len() {
            return bad(
                self,
                "source.argument",
                format!(
                    "`{}` expects {want} positional selector argument(s)",
                    callee.name
                ),
                call_span,
            );
        }
        // selector 0：裸 record 名（Var 且在 rec_index）。
        let type_name = match &positional[0].kind {
            ExprKind::Var(id) => id.name.clone(),
            _ => {
                return bad(
                    self,
                    "source.cte.meta-type-selector",
                    format!(
                        "the type operand of `{}` must be a bare declared record name (a query selector, never a value)",
                        callee.name
                    ),
                    positional[0].span,
                )
            }
        };
        if !self.rec_index.contains_key(&type_name) {
            return bad(
                self,
                "source.cte.meta-unknown-type",
                format!("`{type_name}` is not a declared record type in this module"),
                positional[0].span,
            );
        }
        // selector 1（has_field）：字段名必须是字符串字面量。
        let field = if want == 2 {
            match &positional[1].kind {
                ExprKind::StrLit(f) => Some(f.clone()),
                _ => {
                    return bad(
                        self,
                        "source.cte.meta-field-selector",
                        format!(
                            "the field operand of `{}` must be a string literal (a query selector, never a value)",
                            callee.name
                        ),
                        positional[1].span,
                    )
                }
            }
        } else {
            None
        };
        // 字段存在性不在 bind 判定：缺字段是 has_field 的语义结果（false）
        // 与负依赖轴（§5.2“HasField 缺→有真实负依赖”），非 bind 错误。
        let vt = if op == "cte.meta.field_count" {
            VT::I32
        } else {
            VT::Bool
        };
        ResolvedExpr {
            span: call_span,
            vt: self.node_ty(call_span, vt),
            kind: RExprKind::MetaQuery {
                op,
                type_name,
                field,
            },
        }
    }

    fn call(&mut self, callee: &ast::Ident, args: &[Arg], call_span: Span) -> ResolvedExpr {
        // PLAN-936（T-04）：meta 查询内在在此拦截——selector 实参绝不进入
        // 值解析（裸 record 名/字符串字面量不是值位置合法形状）。
        // A grant protects the names. Without it, declared user functions
        // remain ordinary; an undeclared selector inside CT gets the named
        // capability rejection rather than being evaluated by the scheduler.
        if self.meta_enabled || (self.ct_depth > 0 && !self.sigs.contains_key(&callee.name)) {
            if let Some(op) = meta_intrinsic(&callee.name) {
                return self.meta_query(op, callee, args, call_span);
            }
        }
        let mut rargs: Vec<ResolvedArg> = Vec::new();
        for a in args {
            match a {
                Arg::Pos(e) => {
                    let mut v = self.expr_iterative(e);
                    let moved = matches!(v.kind, RExprKind::Move { .. });
                    if moved {
                        v = match std::mem::replace(&mut v.kind, RExprKind::IntLit(0)) {
                            RExprKind::Move { inner } => *inner,
                            _ => unreachable!("checked above"),
                        };
                    }
                    rargs.push(ResolvedArg {
                        value: v,
                        name: None,
                        moved,
                        span: e.span,
                    })
                }
                Arg::MovePos(e) => {
                    let v = self.expr_iterative(e);
                    rargs.push(ResolvedArg {
                        value: v,
                        name: None,
                        moved: true,
                        span: e.span,
                    })
                }
                Arg::Named { name, value, span } => rargs.push(ResolvedArg {
                    value: self.expr_iterative(value),
                    moved: false,
                    name: Some(ast::Ident {
                        name: name.name.clone(),
                        span: name.span,
                    }),
                    span: *span,
                }),
            }
        }
        self.resolve_call(callee, rargs, call_span)
    }

    /// Callee resolution + argument mapping over already-resolved args
    /// (the iterative walk reaches this directly for nested calls).
    fn resolve_call(
        &mut self,
        callee: &ast::Ident,
        mut rargs: Vec<ResolvedArg>,
        call_span: Span,
    ) -> ResolvedExpr {
        if self.is_template_call(callee) {
            return self
                .probe_template_call(callee, rargs, None, call_span, None)
                .expect("known template in discovery");
        }
        if self.profile.is_auto_language()
            && self.lookup(&callee.name).is_none()
            && !self.sigs.contains_key(&callee.name)
        {
            let symbol = match callee.name.as_str() {
                "__ace_text_find" => Some("__ace_text_find"),
                "__ace_text_trim" => Some("__ace_text_trim"),
                _ => None,
            };
            if let Some(symbol) = symbol {
                let (params, vt) = super::adapter::text_scan_signature(symbol).expect("known scan");
                if rargs.len() != params.len()
                    || rargs
                        .iter()
                        .zip(params)
                        .any(|(arg, ty)| arg.name.is_some() || arg.moved || arg.value.vt != *ty)
                {
                    self.bind_err(
                        "source.type-mismatch",
                        "invalid text scan operands",
                        call_span,
                    );
                }
                self.used_intrinsics.insert(symbol);
                return ResolvedExpr {
                    span: call_span,
                    vt,
                    kind: RExprKind::CallIntrinsic {
                        symbol,
                        args: rargs.into_iter().map(|arg| arg.value).collect(),
                    },
                };
            }
        }
        // PLAN-1026: a DIRECT Python call — the callee resolves to a
        // hoisted use.py member local (or any marked py local): the
        // receiver IS the callable object (no member label).
        if let Some(l) = self.lookup(&callee.name) {
            if self.py_locals.contains(&l) {
                let recv = ResolvedExpr {
                    span: callee.span,
                    vt: VT::I32,
                    kind: RExprKind::ReadLocal(l),
                };
                return self.py_call_sugar(&recv, "", rargs, call_span);
            }
        }
        if self.profile == SourceProfile::Auto
            && matches!(callee.name.as_str(), "assert" | "assert_eq")
            && self.lookup(&callee.name).is_none()
            && !self.sigs.contains_key(&callee.name)
        {
            let equality = callee.name == "assert_eq";
            let arity = if equality { 2 } else { 1 };
            if rargs.len() != arity || rargs.iter().any(|arg| arg.name.is_some() || arg.moved) {
                self.bind_err(
                    "source.assert-arity",
                    if equality {
                        "assert_eq requires two positional values"
                    } else {
                        "assert requires one positional condition"
                    },
                    call_span,
                );
            } else {
                let operand = rargs.pop().expect("arity checked").value;
                let kind = if equality {
                    let mut rhs = operand;
                    let mut lhs = rargs.pop().expect("arity checked").value;
                    if lhs.vt == VT::Dyn {
                        let expected = if rhs.vt == VT::Dyn { VT::I32 } else { rhs.vt };
                        lhs = Self::coerce_dynamic(lhs, expected);
                    }
                    if rhs.vt == VT::Dyn {
                        rhs = Self::coerce_dynamic(rhs, lhs.vt);
                    }
                    if lhs.vt.is_text() && rhs.vt.is_text() {
                        let lt = lhs.vt;
                        let rt = rhs.vt;
                        retag_literal(&mut lhs, rt);
                        retag_literal(&mut rhs, lt);
                    }
                    RExprKind::Binary {
                        op: BinOp::Ne,
                        lhs: Box::new(lhs),
                        rhs: Box::new(rhs),
                    }
                } else {
                    RExprKind::Unary {
                        op: UnaryOp::Not,
                        operand: Box::new(Self::coerce_dynamic(operand, VT::Bool)),
                    }
                };
                let condition = ResolvedExpr {
                    span: call_span,
                    vt: VT::Bool,
                    kind,
                };
                let mut failure = Vec::new();
                self.emit_fail_to_out(
                    ResolvedExpr {
                        span: call_span,
                        vt: VT::StrOwned,
                        kind: RExprKind::StrLit(format!("{} failed", callee.name)),
                    },
                    call_span,
                    &mut failure,
                );
                self.emit_stmt(RStmt::If {
                    cond: condition,
                    then: RBlock {
                        stmts: failure,
                        span: call_span,
                    },
                    els: None,
                    span: call_span,
                });
            }
            return ResolvedExpr {
                span: call_span,
                vt: VT::Unit,
                kind: RExprKind::Unit,
            };
        }
        // PLAN-962（T-08，L-09）：`take(x)` = Move 弃用别名（用户同名 fn 优先）。
        if callee.name == "take"
            && !self.sigs.contains_key("take")
            && rargs.len() == 1
            && rargs[0].name.is_none()
        {
            let inner = rargs.remove(0).value;
            let vt = inner.vt;
            let span = callee.span.join(inner.span);
            return ResolvedExpr {
                span,
                vt,
                kind: RExprKind::Move {
                    inner: Box::new(inner),
                },
            };
        }
        // PLAN-964 (L-22/FStr): the `__fstr` builtin (produced by the
        // front-door f-string transform) flattens into the interpolation
        // chain: literal Str/StrOwned segments pass through, I32 segments
        // display via TextDisplay, all concatenated in source order.
        if callee.name == "__fstr" && !self.sigs.contains_key("__fstr") {
            let mut parts: Vec<FStrPart> = Vec::new();
            let display_float = self.coll.age_package;
            for a in rargs.iter() {
                match (&a.value.kind, a.value.vt) {
                    // PLAN-964: any Str/StrOwned-typed part passes through
                    // as text (literals AND reads of Str locals).
                    (_, VT::Str | VT::StrOwned) => {
                        let value = a.value.clone();
                        parts.push(FStrPart::Text(if self.profile.is_auto_language() {
                            Self::clone_text_element(value)
                        } else {
                            value
                        }));
                    }
                    (_, VT::Dyn) if self.profile.is_auto_language() => {
                        let text = Self::coerce_dynamic(a.value.clone(), VT::StrOwned);
                        parts.push(FStrPart::Text(text));
                    }
                    (_, ty) if ty == VT::I32 || (display_float && ty == VT::F32) => {
                        let operand = a.value.clone();
                        let display = ResolvedExpr {
                            span: operand.span,
                            vt: VT::StrOwned,
                            kind: RExprKind::TextDisplay {
                                operand: Box::new(operand),
                            },
                        };
                        parts.push(FStrPart::Expr(display));
                    }
                    (_, VT::Variant(variant))
                        if self.profile.is_auto_language()
                            && self.coll.variants[variant].name == "Option"
                            && self.coll.variants[variant].arms.first().and_then(|arm| arm.1)
                                == Some(VT::I32) =>
                    {
                        parts.push(FStrPart::Expr(ResolvedExpr {
                            span: a.value.span,
                            vt: VT::StrOwned,
                            kind: RExprKind::TextDisplay {
                                operand: Box::new(a.value.clone()),
                            },
                        }));
                    }
                    (_, ty)
                        if self.profile.is_auto_language()
                            && vt_is_numeric(ty)
                            && !vt_is_float(ty) =>
                    {
                        parts.push(FStrPart::Expr(ResolvedExpr {
                            span: a.value.span,
                            vt: VT::StrOwned,
                            kind: RExprKind::TextDisplay {
                                operand: Box::new(a.value.clone()),
                            },
                        }));
                    }
                    (_, other_vt) => {
                        let admitted = if display_float { "`Str`/`int`/`float`" } else { "`Str`/`int`" };
                        self.bind_err(
                            "source.fstr-arg",
                            format!(
                                "FStr interpolation admits {admitted} parts, found `{}`",
                                other_vt.as_str()
                            ),
                            a.span,
                        );
                    }
                }
            }
            if parts.is_empty() {
                self.bind_err(
                    "source.fstr-empty",
                    "an FStr must have at least one part".to_string(),
                    call_span,
                );
            }
            let span = callee.span.join(call_span);
            return ResolvedExpr {
                span,
                vt: VT::StrOwned,
                kind: RExprKind::FStr { parts },
            };
        }
        if self.profile.is_aggregates() {
            if matches!(callee.name.as_str(), "Some" | "Ok" | "Err")
                && self.ctor_name_is_free(&callee.name)
            {
                if rargs.len() != 1 || rargs[0].name.is_some() {
                    self.bind_err(
                        "source.arity",
                        format!("`{}` expects 1 positional argument", callee.name),
                        call_span,
                    );
                    return self.dummy_of(VT::I32, call_span);
                }
                let pl = rargs.remove(0).value;
                return self.resolve_bare_ctor(&callee.name, Some(pl), None, call_span);
            }
            if callee.name == "fail" && !self.sigs.contains_key("fail") {
                if rargs.len() != 1 || rargs[0].name.is_some() {
                    self.bind_err(
                        "source.arity",
                        "`fail` expects 1 positional string argument".to_string(),
                        call_span,
                    );
                    return self.dummy_of(VT::Unit, call_span);
                }
                let msg = rargs.remove(0).value;
                if !matches!(msg.vt, VT::Str | VT::StrOwned) {
                    self.bind_err(
                        "source.type-mismatch",
                        format!(
                            "`fail` expects a string message, found `{}`",
                            msg.vt.as_str()
                        ),
                        msg.span,
                    );
                    return self.dummy_of(VT::Unit, call_span);
                }
                let mut stmts = Vec::new();
                self.emit_fail_to_out(msg, call_span, &mut stmts);
                for st in stmts {
                    self.emit_stmt(st);
                }
                self.expr_diverged = true;
                return self.dummy_of(VT::Unit, call_span);
            }
            if callee.name == "rethrow" && !self.sigs.contains_key("rethrow") {
                if self.catch_err_locals.is_empty() {
                    self.bind_err(
                        "source.rethrow-outside-catch",
                        "`rethrow` can only be used inside a `catch` block".to_string(),
                        call_span,
                    );
                    return self.dummy_of(VT::Unit, call_span);
                }
                let msg = if rargs.is_empty() {
                    let err_l = *self.catch_err_locals.last().expect("catch local");
                    ResolvedExpr {
                        span: call_span,
                        vt: VT::StrOwned,
                        kind: RExprKind::ReadLocal(err_l),
                    }
                } else if rargs.len() == 1 && rargs[0].name.is_none() {
                    let m = rargs.remove(0).value;
                    if !matches!(m.vt, VT::Str | VT::StrOwned) {
                        self.bind_err(
                            "source.type-mismatch",
                            format!(
                                "`rethrow` expects a string message, found `{}`",
                                m.vt.as_str()
                            ),
                            m.span,
                        );
                        return self.dummy_of(VT::Unit, call_span);
                    }
                    m
                } else {
                    self.bind_err(
                        "source.arity",
                        "`rethrow` takes 0 or 1 positional string argument".to_string(),
                        call_span,
                    );
                    return self.dummy_of(VT::Unit, call_span);
                };
                let mut stmts = Vec::new();
                self.emit_fail_to_out(msg, call_span, &mut stmts);
                for st in stmts {
                    self.emit_stmt(st);
                }
                self.expr_diverged = true;
                return self.dummy_of(VT::Unit, call_span);
            }
        }
        for arg in &rargs {
            if matches!(
                &arg.value.kind,
                RExprKind::CreateClosure { captures, .. }
                    if captures.iter().any(|(_, mode)| *mode == crate::hir::CaptureMode::Borrow)
            ) {
                self.bind_err(
                    "source.closure-borrow-escape",
                    "a borrow-capturing closure cannot be passed as an argument",
                    arg.span,
                );
            }
        }
        let args = &rargs[..];
        let _ = &args;

        // PLAN-960: module global accessors (packages binder rewrite). The
        // pseudo-signature carries the declared type; reads take no
        // arguments, writes exactly one positional of that type.
        let get = callee.name.strip_prefix(super::packages::GLOBAL_GET_PREFIX);
        let take = callee
            .name
            .strip_prefix(super::packages::GLOBAL_TAKE_PREFIX);
        if let Some(sym) = get.or(take) {
            let Some(sig) = self.sigs.get(&callee.name).cloned() else {
                self.bind_err(
                    "source.undefined",
                    format!("unknown module global `{sym}`"),
                    callee.span,
                );
                return ResolvedExpr {
                    span: call_span,
                    vt: VT::I32,
                    kind: RExprKind::Unit,
                };
            };
            if !rargs.is_empty() {
                self.bind_err(
                    "source.arity",
                    format!("module global `{sym}` is read without arguments"),
                    call_span,
                );
            }
            return ResolvedExpr {
                span: call_span,
                vt: self.node_ty(call_span, sig.result),
                kind: RExprKind::CallGlobalGet {
                    global: sym.to_string(),
                    take: take.is_some(),
                },
            };
        }
        if let Some(sym) = callee.name.strip_prefix("module.global.set:") {
            let Some(sig) = self.sigs.get(&callee.name).cloned() else {
                self.bind_err(
                    "source.undefined",
                    format!("unknown module global `{sym}`"),
                    callee.span,
                );
                return ResolvedExpr {
                    span: call_span,
                    vt: VT::I32,
                    kind: RExprKind::Unit,
                };
            };
            if rargs.len() != 1 || rargs[0].name.is_some() {
                self.bind_err(
                    "source.arity",
                    format!("module store `{sym}` takes exactly one value"),
                    call_span,
                );
                return ResolvedExpr {
                    span: call_span,
                    vt: VT::I32,
                    kind: RExprKind::Unit,
                };
            }
            let mut arg = rargs.remove(0);
            retag_literal(&mut arg.value, sig.params[0]);
            self.check_args(
                &callee.name,
                &sig.param_names,
                &sig.params,
                std::slice::from_ref(&arg),
                call_span,
            );
            return ResolvedExpr {
                span: call_span,
                vt: VT::I32,
                kind: RExprKind::CallGlobalSet {
                    global: sym.to_string(),
                    value: Box::new(arg.value),
                },
            };
        }

        if (self.profile == SourceProfile::HostConsole
            // PLAN-968: the scheduler face admits the same console calls
            // (task handlers observe through stdout).
            || self.profile == SourceProfile::Scheduler
            // PLAN-1026: the py training lane (a use.py program under
            // Aggregates) admits the same console calls for its body.
            // PLAN-982 (Q-982-2 step 2): the sys script lane — plain
            // Aggregates app faces gain the same console calls; the run's
            // console host (capture or direct) is the door's choice.
            || self.profile == SourceProfile::Aggregates
            // PLAN-1078: the Auto face admits the same console calls.
            || self.profile == SourceProfile::Auto)
            && matches!(callee.name.as_str(), "print" | "eprint" | "say")
        {
            if rargs.len() != 1 || rargs.iter().any(|a| a.name.is_some()) {
                self.bind_err(
                    "source.console-argument",
                    "console calls require exactly one positional argument",
                    call_span,
                );
                return ResolvedExpr {
                    span: call_span,
                    vt: VT::Unit,
                    kind: RExprKind::Unit,
                };
            }
            let mut arg = rargs.remove(0).value;
            if self.profile.is_auto_language()
                && arg.vt == VT::StrOwned
                && matches!(arg.kind, RExprKind::ElemView { .. })
            {
                arg = Self::clone_owned_text(arg);
            }
            if self.profile.is_auto_language() && arg.vt == VT::Nil {
                let span = arg.span;
                self.eval_for_effect(arg);
                arg = ResolvedExpr {
                    span,
                    vt: VT::StrOwned,
                    kind: RExprKind::StrLit("None".into()),
                };
            }
            // PLAN-1026: a Python object prints through the adapter's
            // display channel (stage + word-wise readback → Str), never
            // the raw i32 token.
            if self.py_is_object(&arg) {
                let span = arg.span;
                // PLAN-1107 T-03: the display extraction is ONE shared
                // operation fact — a source-generating target lowers it to its
                // native text conversion instead of replaying the staged
                // word-readback chain.
                if crate::py_ops::recording_active() {
                    crate::py_ops::record(
                        PyOpFact::new(PyOpKind::ToText, span)
                            .with_member("str")
                            .operand_lit(
                                PyRole::Receiver,
                                span,
                                self.py_val_class(&arg),
                                self.py_operand_literal(&arg),
                            ),
                    );
                }
                let len = self.py_stage_len(&arg, span);
                self.used_intrinsics.insert("__py_staged_text");
                arg = ResolvedExpr {
                    span,
                    vt: self.node_ty(span, VT::StrOwned),
                    kind: RExprKind::CallIntrinsic {
                        symbol: "__py_staged_text",
                        args: vec![len],
                    },
                };
            }
            let Some(symbol) = super::console::symbol(&callee.name, arg.vt) else {
                self.bind_err(
                    "source.console-argument",
                    if callee.name.as_str() == "say" {
                        // PLAN-947: the public spelling is frozen str-only.
                        "say requires a str (or owned Str) argument"
                    } else {
                        "console argument must be int, bool, str, Str or char"
                    },
                    call_span,
                );
                return ResolvedExpr {
                    span: call_span,
                    vt: VT::Unit,
                    kind: RExprKind::Unit,
                };
            };
            self.used_intrinsics.insert(symbol);
            return ResolvedExpr {
                span: call_span,
                vt: VT::Unit,
                kind: RExprKind::CallIntrinsic {
                    symbol,
                    args: vec![arg],
                },
            };
        }
        // PLAN-982（B-CONFIG-I E3）：配置文档原语（`config.document/1`）只在
        // config-document 面绑定；标量重载按**实参 VT**选择（节点/对象/列表
        // 句柄与 i32 同型，种类由物化侧的显式 callee 决定）。未知/不匹配都
        // 具名拒绝，不静默退回普通函数解析。
        if self.profile == SourceProfile::ConfigDocument
            && super::config_ops::is_callee(&callee.name)
        {
            if rargs.iter().any(|a| a.name.is_some()) {
                self.bind_err(
                    "source.config-argument",
                    format!(
                        "configuration primitive `{}` takes positional arguments only",
                        callee.name
                    ),
                    call_span,
                );
                return ResolvedExpr {
                    span: call_span,
                    vt: VT::I32,
                    kind: RExprKind::Unit,
                };
            }
            let vts: Vec<VT> = rargs.iter().map(|a| a.value.vt).collect();
            return match super::config_ops::lookup_callee(&callee.name, &vts) {
                Some(op) => {
                    let args = rargs.into_iter().map(|a| a.value).collect();
                    self.used_intrinsics.insert(op.symbol);
                    ResolvedExpr {
                        span: call_span,
                        vt: op.result,
                        kind: RExprKind::CallIntrinsic {
                            symbol: op.symbol,
                            args,
                        },
                    }
                }
                None => {
                    let got = vts
                        .iter()
                        .map(|v| v.as_str())
                        .collect::<Vec<_>>()
                        .join(", ");
                    self.bind_err(
                        "source.config-argument",
                        format!(
                            "configuration primitive `{}` has no overload for ({got})",
                            callee.name
                        ),
                        call_span,
                    );
                    ResolvedExpr {
                        span: call_span,
                        vt: VT::I32,
                        kind: RExprKind::Unit,
                    }
                }
            };
        }
        if self.profile == SourceProfile::HostConsole
            // PLAN-1041: 包装配面（TextRecord 经 bind_packages/resolve_with）同样
            // 准入平台保留名——公共 stdlib 源（如 io 输出循环）单源服务
            // console 单模块面与 1043 装配面；能力仍在准入层核。
            || self.profile == SourceProfile::TextRecord
            // PLAN-973: 装配 aggregates 面（std 系统库集：sysplat_win/lnx 的
            // 平台原语包装）同样准入平台保留名；能力/授权仍在运行准入层核。
            || self.profile == SourceProfile::Aggregates
            // PLAN-1078: the Auto face admits the same platform reserved
            // names (authority still checked at the runtime admission layer).
            || self.profile == SourceProfile::Auto
        {
            if let Some(op) = super::platform_io::lookup_callee(&callee.name) {
                if rargs.len() != op.params.len() || rargs.iter().any(|a| a.name.is_some()) {
                    self.bind_err(
                        "source.platform-argument",
                        format!(
                            "platform operation `{}` requires {} positional argument(s)",
                            op.callee,
                            op.params.len()
                        ),
                        call_span,
                    );
                    return ResolvedExpr {
                        span: call_span,
                        vt: op.result,
                        kind: RExprKind::Unit,
                    };
                }
                let mut args = Vec::with_capacity(rargs.len());
                for (arg, expected) in rargs.into_iter().zip(op.params.iter()) {
                    let ok = match expected {
                        VT::Str => matches!(arg.value.vt, VT::Str | VT::StrOwned),
                        _ => arg.value.vt == *expected,
                    };
                    if !ok {
                        self.bind_err(
                            "source.platform-argument",
                            format!("platform operation `{}` argument type mismatch", op.callee),
                            arg.value.span,
                        );
                    }
                    args.push(arg.value);
                }
                self.used_intrinsics.insert(op.symbol);
                return ResolvedExpr {
                    span: call_span,
                    vt: op.result,
                    kind: RExprKind::CallIntrinsic {
                        symbol: op.symbol,
                        args,
                    },
                };
            }
        }
        // Locals shadow functions and intrinsics in callee position.
        if let Some(local) = self.lookup(&callee.name) {
            let vt = self.locals[local.0].vt;
            if matches!(vt, VT::Function(_))
                || (self.coll.script_errors && matches!(vt, VT::Nil | VT::Dyn))
            {
                if self.moved_locals.contains(&local) {
                    self.bind_err(
                        "source.closure-use-after-move",
                        format!(
                            "closure local `{}` was moved and cannot be called",
                            callee.name
                        ),
                        callee.span,
                    );
                }
                let callee_value = ResolvedExpr {
                    span: callee.span,
                    vt,
                    kind: RExprKind::ReadLocal(local),
                };
                return self.resolve_closure_call(callee_value, rargs, call_span);
            }
            self.bind_err(
                "source.not-callable",
                format!(
                    "`{}` is a local, not a function; it cannot be called",
                    callee.name
                ),
                call_span,
            );
            return ResolvedExpr {
                span: call_span,
                vt: VT::I32,
                kind: RExprKind::CallFn {
                    fn_index: usize::MAX,
                    args: rargs,
                },
            };
        }

        // PLAN-1081 (T-03): a nested fn item's BARE name resolves through
        // the per-fn nested scope first (call-only); module sigs follow.
        let callee_sig = self
            .nested
            .scope
            .get(&callee.name)
            .cloned()
            .or_else(|| self.sigs.get(&callee.name).cloned());
        if let Some(sig) = callee_sig {
            if self.coll.scoped_iter_fns.contains(&sig.index)
                && self.scoped_iter_site != Some(call_span)
            {
                self.bind_err(
                    "source.generator-borrow-escape",
                    "an iterator with borrowed inputs requires a direct for-in consumer",
                    call_span,
                );
            }
            // PLAN-963（T-08，L-24）：缺席实参在调用者作用域补默认值（每调用一次）。
            self.fill_default_args(
                &sig.param_names,
                &sig.params,
                &sig.defaults,
                &mut rargs,
                call_span,
            );
            // contextual literal typing: a string literal adapts to a text
            // formal (D-03 promotion) so the missing-`.move` rejection
            // lands in the verifier, not as a type mismatch here
            for (i, a) in rargs.iter_mut().enumerate() {
                let pos = if a.name.is_some() {
                    sig.param_names
                        .iter()
                        .position(|n| Some(n) == a.name.as_ref().map(|x| &x.name))
                } else {
                    Some(i)
                };
                if let Some(pos) = pos.filter(|&p| p < sig.params.len()) {
                    self.adapt_argument_type(&mut a.value, sig.params[pos]);
                }
            }
            self.check_args(
                &callee.name,
                &sig.param_names,
                &sig.params,
                &rargs,
                call_span,
            );
            return ResolvedExpr {
                span: call_span,
                vt: self.node_ty(call_span, sig.result),
                kind: RExprKind::CallFn {
                    fn_index: sig.index,
                    args: rargs,
                },
            };
        }

        // Positional record construction `Q(7)` (PLAN-771 §5.2,
        // F-771-R1-02): the callee name is neither a local nor a function —
        // resolve it against the record table. Like functions, record
        // types are user declarations and win over intrinsic name
        // hijacking; a function sharing the record's name keeps the call
        // form (the namespaces stay separate, mirroring the fn rule).
        if let Some(&ri) = self.rec_index.get(&callee.name) {
            return self.ctor_positional(ri, callee, rargs, call_span);
        }

        // User functions win over trace intrinsics (no name hijacking);
        // intrinsics only exist with the trace capability provided.
        if let Some(symbol) = trace_intrinsic(&callee.name) {
            if self.trace_enabled {
                if !rargs.is_empty() {
                    self.bind_err(
                        "source.argument",
                        format!("`{}` expects no arguments", callee.name),
                        call_span,
                    );
                }
                self.used_intrinsics.insert(symbol);
                return ResolvedExpr {
                    span: call_span,
                    vt: self.node_ty(call_span, VT::I32),
                    kind: RExprKind::CallIntrinsic {
                        symbol,
                        args: Vec::new(),
                    },
                };
            }
        }

        // PLAN-968: the desugared scheduler intrinsics. They exist only
        // on the scheduler face (with its side-table); every other face
        // falls through to the historical `source.undefined` rejection.
        if callee.name.starts_with("__sched_") {
            return self.resolve_sched_call(callee, rargs, call_span);
        }
        if self.profile.is_auto_language() && callee.name == "sleep" {
            return self.scheduler_operation("__sched_sleep", rargs, call_span);
        }

        self.bind_err(
            "source.undefined",
            format!("no function named `{}` in this module", callee.name),
            callee.span,
        );
        ResolvedExpr {
            span: call_span,
            vt: VT::I32,
            kind: RExprKind::CallFn {
                fn_index: usize::MAX,
                args: rargs,
            },
        }
    }

    /// PLAN-968: intern one per-task/per-callee scheduler intrinsic symbol.
    /// Symbols are process-lifetime strings (bounded by distinct task and
    /// function names across a process — a compiler-process registry, not
    /// per-iteration growth).
    fn sched_sym(prefix: &str, suffix: &str) -> &'static str {
        use std::collections::BTreeSet;
        use std::sync::{Mutex, OnceLock};
        static REG: OnceLock<Mutex<BTreeSet<&'static str>>> = OnceLock::new();
        let reg = REG.get_or_init(|| Mutex::new(BTreeSet::new()));
        let mut g = reg.lock().expect("sched symbol registry");
        let want = format!("{prefix}_{suffix}");
        if let Some(have) = g.get(want.as_str()) {
            return have;
        }
        let leaked: &'static str = Box::leak(want.into_boxed_str());
        g.insert(leaked);
        leaked
    }

    fn resolve_implicit_actor_message(&mut self, receiver: VT, e: &Expr) -> Option<ResolvedExpr> {
        let VT::TaskRef(message) = receiver else {
            return None;
        };
        let VT::Variant(variant) = self.coll.task_messages[message] else {
            return None;
        };
        if !self.coll.variants[variant]
            .name
            .starts_with(super::sched_desugar::ACTOR_MESSAGE_PREFIX)
        {
            return None;
        }
        let (name, args) = match &e.kind {
            ExprKind::Var(name) => (name, &[][..]),
            ExprKind::Call { callee, args } => (callee, args.as_slice()),
            _ => return None,
        };
        if self.lookup(&name.name).is_some() || self.sigs.contains_key(&name.name) {
            return None;
        }
        let (disc, want) = self.coll.variants[variant]
            .arms
            .iter()
            .enumerate()
            .find_map(|(i, (arm, ty))| (arm == &name.name).then_some((i, *ty)))?;
        let arity = match want {
            None => 0,
            Some(VT::Tuple(tuple)) => self.coll.tuples[tuple].len(),
            _ => 1,
        };
        let mut values = self.resolve_args(args);
        if values.len() != arity || values.iter().any(|arg| arg.name.is_some()) {
            self.bind_err(
                "source.sched-argument",
                format!("message `{}` expects {arity} positional values", name.name),
                e.span,
            );
            return Some(instances::pending_expression(e.span));
        }
        let payload = match want {
            None => None,
            Some(VT::Tuple(tuple)) => {
                let elems = values
                    .into_iter()
                    .map(|arg| Self::coerce_dynamic(arg.value, VT::Dyn))
                    .collect();
                Some(Box::new(ResolvedExpr {
                    span: e.span,
                    vt: VT::Tuple(tuple),
                    kind: RExprKind::TupleLit { elems },
                }))
            }
            Some(_) => Some(Box::new(Self::coerce_dynamic(
                values.remove(0).value,
                VT::Dyn,
            ))),
        };
        Some(ResolvedExpr {
            span: e.span,
            vt: VT::Variant(variant),
            kind: RExprKind::VariantCtor {
                variant,
                disc: disc as u32,
                payload,
            },
        })
    }

    fn scheduler_operation(
        &mut self,
        operation: &str,
        args: Vec<ResolvedArg>,
        span: Span,
    ) -> ResolvedExpr {
        self.resolve_sched_call(
            &ast::Ident {
                name: operation.into(),
                span,
            },
            args,
            span,
        )
    }

    /// PLAN-968: type one desugared `__sched_*` intrinsic call against
    /// the scheduler side-table. Handles/futures are monomorphic per
    /// declared task (`VT::Task(i)`/`VT::Future(i)`, `i` = the surface's
    /// task index); every arity/type/lifetime violation is a named
    /// `source.sched-*` rejection.
    fn resolve_sched_call(
        &mut self,
        callee: &ast::Ident,
        rargs: Vec<ResolvedArg>,
        call_span: Span,
    ) -> ResolvedExpr {
        let named_i32 = |span: Span| ResolvedExpr {
            span,
            vt: VT::I32,
            kind: RExprKind::Unit,
        };
        let Some(surface) = self.sched else {
            self.bind_err(
                "source.sched-unsupported",
                format!(
                    "`{}` is a scheduler-surface operation and is not part of this source profile",
                    callee.name
                ),
                callee.span,
            );
            return named_i32(call_span);
        };
        let name = callee.name.as_str();
        let mut args: Vec<ResolvedExpr> = Vec::new();
        for a in rargs {
            if self.profile.is_auto_language() && a.name.is_some() {
                self.bind_err(
                    "source.sched-argument",
                    "scheduler operations require positional arguments",
                    a.span,
                );
                return named_i32(call_span);
            }
            args.push(a.value);
        }
        let arity_err = |r: &mut Self, want: &str| {
            r.bind_err(
                "source.sched-arity",
                format!("`{name}` expects {want}"),
                call_span,
            );
        };
        let task_of_driver = |r: &mut Self| -> Option<usize> {
            let owner = r.owner_name.clone();
            surface
                .tasks
                .iter()
                .position(|t| t.driver_fn == owner)
                .or_else(|| {
                    r.bind_err(
                        "source.sched-driver",
                        format!(
                            "`{name}` is only legal inside a task's synthesized driver function"
                        ),
                        call_span,
                    );
                    None
                })
        };
        match name {
            "__sched_future" if self.profile.is_auto_language() => {
                let [thunk] = args.as_slice() else {
                    arity_err(self, "one zero-argument async body");
                    return named_i32(call_span);
                };
                let VT::Function(signature) = thunk.vt else {
                    self.bind_err(
                        "source.sched-argument",
                        "a future requires an async body",
                        thunk.span,
                    );
                    return named_i32(call_span);
                };
                if !self.coll.function_types[signature].params.is_empty() {
                    self.bind_err(
                        "source.sched-argument",
                        "an async body takes no arguments",
                        thunk.span,
                    );
                    return named_i32(call_span);
                }
                let symbol = Self::sched_sym("__sched_future", &signature.to_string());
                ResolvedExpr {
                    span: call_span,
                    vt: VT::Deferred(signature),
                    kind: RExprKind::CallIntrinsic { symbol, args },
                }
            }
            "__sched_detach" if self.profile.is_auto_language() => {
                if !matches!(args.as_slice(), [value] if matches!(value.vt, VT::Deferred(_))) {
                    self.bind_err(
                        "source.sched-argument",
                        "`go` requires a future or a direct call",
                        call_span,
                    );
                    return named_i32(call_span);
                }
                ResolvedExpr {
                    span: call_span,
                    vt: VT::I32,
                    kind: RExprKind::CallIntrinsic {
                        symbol: "__sched_detach",
                        args,
                    },
                }
            }
            "__sched_spawn" | "__sched_singleton" => {
                // PLAN-1067 review R-1067-11: spawning a task from inside a
                // task driver leaves the scheduler surface (the driver's own
                // callee table is fixed when the program is desugared, so such
                // a call reached the backend as an unnamed lowering error).
                // Name it here, at the front door, like every other
                // scheduler-surface boundary. NOTE: this must not go through
                // `task_of_driver` — that helper reports its own "only legal
                // inside a driver" error for every ordinary caller.
                let inside_driver = surface.tasks.iter().any(|t| t.driver_fn == self.owner_name);
                if !self.profile.is_auto_language() && inside_driver {
                    self.bind_err(
                        "source.sched-task",
                        "`Task.spawn` inside a task driver is not part of the scheduler surface",
                        call_span,
                    );
                    return named_i32(call_span);
                }
                // PLAN-1067 review2 N-1067-9: the same boundary applies to any
                // non-entry function (a plain helper reached from `main` or from
                // a driver): the backend's callee table is built for the entry
                // unit, so such a call surfaced as an unnamed lowering error.
                // One named rule for every non-entry position.
                if !self.profile.is_auto_language() && self.owner_name != "main" {
                    self.bind_err(
                        "source.sched-task",
                        "`Task.spawn` is only legal in the program entry (`main`)",
                        call_span,
                    );
                    return named_i32(call_span);
                }
                if args.len() < 2 {
                    arity_err(
                        self,
                        "a task name string, a mailbox capacity and the spawn initializers",
                    );
                    return named_i32(call_span);
                }
                let task_idx = match &args[0].kind {
                    RExprKind::StrLit(tname) => {
                        match surface.tasks.iter().position(|t| &t.name == tname) {
                            Some(i) => i,
                            None => {
                                self.bind_err(
                                    "source.sched-task",
                                    format!("no task named `{tname}` is declared in this module"),
                                    args[0].span,
                                );
                                return named_i32(call_span);
                            }
                        }
                    }
                    _ => {
                        self.bind_err(
                            "source.sched-task",
                            "`Task.spawn` takes the task's name as a string literal",
                            args[0].span,
                        );
                        return named_i32(call_span);
                    }
                };
                let info = &surface.tasks[task_idx];
                if name == "__sched_singleton" && !info.single {
                    self.bind_err(
                        "source.sched-singleton",
                        format!("task `{}` needs `#[single]` for static access", info.name),
                        call_span,
                    );
                    return named_i32(call_span);
                }
                if args[1].vt != VT::I32 {
                    self.bind_err(
                        "source.sched-argument",
                        "`Task.spawn`'s mailbox capacity is an int",
                        args[1].span,
                    );
                    return named_i32(call_span);
                }
                if args.len() - 2 > info.fields.len() {
                    self.bind_err(
                        "source.sched-argument",
                        format!(
                            "task `{}` takes at most {} spawn initializers (one per state field)",
                            info.name,
                            info.fields.len()
                        ),
                        call_span,
                    );
                    return named_i32(call_span);
                }
                if self.profile.is_auto_language() {
                    for (_, _, default) in info.fields.iter().skip(args.len() - 2) {
                        args.push(self.expr_iterative(default));
                    }
                }
                for (k, init) in args[2..].iter_mut().enumerate() {
                    let want = vt_of_word(info.fields[k].1);
                    if self.profile.is_auto_language() {
                        retag_literal(init, want);
                    }
                    // Text literals arrive as borrowed Str; owned Str
                    // fields accept either face of the literal.
                    let matches = init.vt == want
                        || (matches!(want, VT::StrOwned)
                            && matches!(init.vt, VT::Str | VT::StrOwned))
                        || (want == VT::Str && init.vt == VT::StrOwned);
                    if !matches {
                        self.bind_err(
                            "source.sched-argument",
                            format!(
                                "spawn initializer {k} for task `{}` must be {} (field `{}`)",
                                info.name,
                                want.as_str(),
                                info.fields[k].0
                            ),
                            init.span,
                        );
                        return named_i32(call_span);
                    }
                }
                let sym = Self::sched_sym(name, &info.name);
                let ty = if self.profile.is_auto_language() {
                    let message = self.coll.sched_tasks[task_idx].msg;
                    VT::TaskRef(self.coll.intern_task_message(message))
                } else {
                    VT::Task(task_idx)
                };
                ResolvedExpr {
                    span: call_span,
                    vt: self.node_ty(call_span, ty),
                    kind: RExprKind::CallIntrinsic { symbol: sym, args },
                }
            }
            "__sched_send" | "__sched_ask" => {
                if args.len() != 2 {
                    arity_err(self, "a task handle and one message");
                    return named_i32(call_span);
                }
                let is_send = name == "__sched_send";
                let (want_msg, task_type) = match args[0].vt {
                    VT::Task(i) => (self.coll.sched_tasks[i].msg, Some(i)),
                    VT::TaskRef(i) => {
                        let message = self.coll.task_messages[i];
                        let mut candidates = self
                            .coll
                            .sched_tasks
                            .iter()
                            .enumerate()
                            .filter(|(_, task)| task.msg == message);
                        let first = candidates.next().map(|(index, _)| index);
                        if !is_send
                            && (first.is_none()
                                || candidates.any(|(_, task)| {
                                    task.reply
                                        != self.coll.sched_tasks[first.expect("first task")].reply
                                }))
                        {
                            self.bind_err(
                                "source.sched-argument",
                                "ask requires an unambiguous reply type for this TaskRef",
                                args[0].span,
                            );
                            return named_i32(call_span);
                        }
                        (message, first)
                    }
                    _ => {
                        self.bind_err(
                            "source.sched-argument",
                            format!(
                                "`{}`'s first argument must be a task handle",
                                if name == "__sched_send" {
                                    "send"
                                } else {
                                    "ask"
                                }
                            ),
                            args[0].span,
                        );
                        return named_i32(call_span);
                    }
                };
                if self.profile.is_auto_language() {
                    let message = &mut args[1];
                    retag_integer_literal(message, want_msg);
                    if want_msg == VT::StrOwned && message.vt == VT::Str {
                        let borrowed = std::mem::replace(
                            message,
                            instances::pending_expression(message.span),
                        );
                        *message = Self::own_dynamic_text(borrowed);
                    }
                }
                if args[1].vt != want_msg {
                    self.bind_err(
                        "source.sched-argument",
                        format!(
                            "the message type for task `{}` is {}",
                            task_type
                                .map_or("TaskRef", |ti| self.coll.sched_tasks[ti].name.as_str()),
                            want_msg.as_str()
                        ),
                        args[1].span,
                    );
                    return named_i32(call_span);
                }
                // Side-effect ops carry a dummy I32 return value in HIR.
                let vt = if is_send {
                    VT::I32
                } else {
                    let task = task_type.expect("ask reply type");
                    if self.profile.is_auto_language() {
                        let result = self.coll.sched_tasks[task].reply;
                        VT::Deferred(self.coll.intern_function(ClosureFnType {
                            params: Vec::new(),
                            result,
                        }))
                    } else {
                        VT::Future(task)
                    }
                };
                let Some(task_type) = task_type else {
                    self.bind_err(
                        "source.sched-argument",
                        "no actor has this mailbox type",
                        args[0].span,
                    );
                    return named_i32(call_span);
                };
                let tname = self.coll.sched_tasks[task_type].name.clone();
                let sym = if is_send {
                    Self::sched_sym("__sched_send", &tname)
                } else {
                    Self::sched_sym("__sched_ask", &tname)
                };
                ResolvedExpr {
                    span: call_span,
                    vt: self.node_ty(call_span, vt),
                    kind: RExprKind::CallIntrinsic { symbol: sym, args },
                }
            }
            "__sched_stop" => {
                if args.len() != 1 {
                    arity_err(self, "a task handle");
                    return named_i32(call_span);
                }
                if !matches!(args[0].vt, VT::Task(_) | VT::TaskRef(_)) {
                    self.bind_err(
                        "source.sched-argument",
                        "`stop` takes a task handle",
                        args[0].span,
                    );
                    return named_i32(call_span);
                }
                ResolvedExpr {
                    span: call_span,
                    vt: VT::I32,
                    kind: RExprKind::CallIntrinsic {
                        symbol: "__sched_stop",
                        args,
                    },
                }
            }
            "__sched_reply" => {
                if args.len() != 1 {
                    arity_err(self, "one reply value");
                    return named_i32(call_span);
                }
                let ti = match task_of_driver(self) {
                    Some(i) => i,
                    None => return named_i32(call_span),
                };
                let want = self.coll.sched_tasks[ti].reply;
                let reply_sym = Self::sched_sym("__sched_reply", &self.coll.sched_tasks[ti].name);
                if args[0].vt != want {
                    self.bind_err(
                        "source.sched-argument",
                        format!(
                            "`reply` on task `{}` takes {}",
                            self.coll.sched_tasks[ti].name,
                            want.as_str()
                        ),
                        args[0].span,
                    );
                    return named_i32(call_span);
                }
                ResolvedExpr {
                    span: call_span,
                    vt: VT::I32,
                    kind: RExprKind::CallIntrinsic {
                        symbol: reply_sym,
                        args,
                    },
                }
            }
            "__sched_sleep" => {
                if args.len() != 1 || args[0].vt != VT::I32 {
                    arity_err(self, "one int duration in milliseconds");
                    return named_i32(call_span);
                }
                ResolvedExpr {
                    span: call_span,
                    vt: VT::I32,
                    kind: RExprKind::CallIntrinsic {
                        symbol: "__sched_sleep",
                        args,
                    },
                }
            }
            "__sched_await" => {
                if args.len() != 1 {
                    arity_err(self, "one ask future");
                    return named_i32(call_span);
                }
                let ti = match args[0].vt {
                    VT::Future(i) => i,
                    VT::Deferred(signature) => {
                        let result = self.coll.function_types[signature].result;
                        let symbol =
                            Self::sched_sym("__sched_await_deferred", &signature.to_string());
                        return ResolvedExpr {
                            span: call_span,
                            vt: self.node_ty(call_span, result),
                            kind: RExprKind::CallIntrinsic { symbol, args },
                        };
                    }
                    _ => {
                        self.bind_err(
                            "source.sched-argument",
                            "`await` takes an `ask` future",
                            args[0].span,
                        );
                        return named_i32(call_span);
                    }
                };
                let want = self.coll.sched_tasks[ti].reply;
                let await_sym = Self::sched_sym("__sched_await", &self.coll.sched_tasks[ti].name);
                ResolvedExpr {
                    span: call_span,
                    vt: self.node_ty(call_span, want),
                    kind: RExprKind::CallIntrinsic {
                        symbol: await_sym,
                        args,
                    },
                }
            }
            "__sched_go" => {
                if args.is_empty() {
                    arity_err(self, "a function name and its arguments");
                    return named_i32(call_span);
                }
                let fname = match &args[0].kind {
                    RExprKind::StrLit(f) => f.clone(),
                    _ => {
                        self.bind_err(
                            "source.sched-argument",
                            "`go` names its function by a string literal (synthesized)",
                            args[0].span,
                        );
                        return named_i32(call_span);
                    }
                };
                let Some(sig) = self.sigs.get(&fname) else {
                    self.bind_err(
                        "source.sched-argument",
                        format!("`go` names no function `{fname}` in this module"),
                        args[0].span,
                    );
                    return named_i32(call_span);
                };
                let want: Vec<VT> = sig.params.clone();
                if args.len() - 1 != want.len() {
                    self.bind_err(
                        "source.sched-arity",
                        format!(
                            "`go {fname}` expects {} argument(s), found {}",
                            want.len(),
                            args.len() - 1
                        ),
                        call_span,
                    );
                    return named_i32(call_span);
                }
                for (k, a) in args[1..].iter().enumerate() {
                    if a.vt != want[k] {
                        self.bind_err(
                            "source.sched-argument",
                            format!("`go {fname}` argument {k} must be {}", want[k].as_str()),
                            a.span,
                        );
                        return named_i32(call_span);
                    }
                }
                // The symbol carries the resolved fn INDEX (stable across
                // the def-order MIR function table) — the lowering does not
                // depend on body-lowering order.
                let go_sym = Self::sched_sym("__sched_go", &format!("{}_{fname}", sig.index));
                ResolvedExpr {
                    span: call_span,
                    vt: VT::I32,
                    kind: RExprKind::CallIntrinsic {
                        symbol: go_sym,
                        args,
                    },
                }
            }
            "__sched_recv" | "__sched_port" | "__sched_stopping" => {
                if !args.is_empty() {
                    arity_err(self, "no arguments");
                    return named_i32(call_span);
                }
                let ti = match task_of_driver(self) {
                    Some(i) => i,
                    None => return named_i32(call_span),
                };
                let tname = self.coll.sched_tasks[ti].name.clone();
                let (vt, symbol): (VT, &'static str) = match name {
                    "__sched_recv" => (
                        self.coll.sched_tasks[ti].msg,
                        Self::sched_sym("__sched_recv", &tname),
                    ),
                    "__sched_port" => (VT::I32, Self::sched_sym("__sched_port", &tname)),
                    _ => (VT::Bool, Self::sched_sym("__sched_stopping", &tname)),
                };
                ResolvedExpr {
                    span: call_span,
                    vt: self.node_ty(call_span, vt),
                    kind: RExprKind::CallIntrinsic { symbol, args },
                }
            }
            _ => {
                self.bind_err(
                    "source.sched-unsupported",
                    format!("`{name}` is not a scheduler operation"),
                    callee.span,
                );
                named_i32(call_span)
            }
        }
    }

    fn resolve_closure_call(
        &mut self,
        callee: ResolvedExpr,
        mut args: Vec<ResolvedArg>,
        call_span: Span,
    ) -> ResolvedExpr {
        if self.coll.script_errors && matches!(callee.vt, VT::Nil | VT::Dyn) {
            let query = if callee.vt == VT::Dyn {
                let local = self.agg_temp("script_callee", VT::Dyn, false, callee.span);
                self.emit_stmt(RStmt::Let {
                    local,
                    span: callee.span,
                    value: callee,
                });
                Some(Self::dynamic_nil_test(
                    ResolvedExpr {
                        span: call_span,
                        vt: VT::Dyn,
                        kind: RExprKind::ReadLocal(local),
                    },
                    call_span,
                ))
            } else {
                self.eval_for_effect(callee);
                None
            };
            for arg in args {
                self.eval_for_effect(arg.value);
            }
            let nil_message = "TypeError: 'NoneType' object is not callable";
            if let Some(cond) = query {
                let local = self.agg_temp("script_error", VT::StrOwned, true, call_span);
                let message = |text: &str| ResolvedExpr {
                    span: call_span,
                    vt: VT::StrOwned,
                    kind: RExprKind::StrLit(text.to_owned()),
                };
                self.emit_stmt(RStmt::Let {
                    local,
                    value: message("TypeError: dynamic value is not callable"),
                    span: call_span,
                });
                self.emit_stmt(RStmt::If {
                    cond,
                    then: RBlock {
                        stmts: vec![RStmt::Assign {
                            local,
                            path: Vec::new(),
                            value: message(nil_message),
                            span: call_span,
                        }],
                        span: call_span,
                    },
                    els: None,
                    span: call_span,
                });
                return self.script_error_value(
                    ResolvedExpr {
                        span: call_span,
                        vt: VT::StrOwned,
                        kind: RExprKind::Move {
                            inner: Box::new(ResolvedExpr {
                                span: call_span,
                                vt: VT::StrOwned,
                                kind: RExprKind::ReadLocal(local),
                            }),
                        },
                    },
                    VT::I32,
                    call_span,
                );
            }
            return self.script_type_error(nil_message.to_string(), VT::I32, call_span);
        }
        for arg in &args {
            if matches!(
                &arg.value.kind,
                RExprKind::CreateClosure { captures, .. }
                    if captures.iter().any(|(_, mode)| *mode == crate::hir::CaptureMode::Borrow)
            ) {
                self.bind_err(
                    "source.closure-borrow-escape",
                    "a borrow-capturing closure cannot be passed as an argument",
                    arg.span,
                );
            }
        }
        let VT::Function(type_index) = callee.vt else {
            self.bind_err(
                "source.not-callable",
                format!("a value of type `{}` is not callable", callee.vt.as_str()),
                call_span,
            );
            return ResolvedExpr {
                span: call_span,
                vt: VT::I32,
                kind: RExprKind::CallClosure {
                    callee: Box::new(callee),
                    args,
                },
            };
        };
        let Some(signature) = self.coll.function_types.get(type_index).cloned() else {
            self.bind_err(
                "source.internal",
                "closure signature index is out of range",
                call_span,
            );
            return ResolvedExpr {
                span: call_span,
                vt: VT::I32,
                kind: RExprKind::CallClosure {
                    callee: Box::new(callee),
                    args,
                },
            };
        };
        for (index, arg) in args.iter_mut().enumerate() {
            if arg.name.is_some() {
                self.bind_err(
                    "source.argument",
                    "indirect closure calls use positional arguments",
                    arg.span,
                );
            }
            if let Some(want) = signature.params.get(index).copied() {
                self.adapt_argument_type(&mut arg.value, want);
                if arg.value.vt != want {
                    self.bind_err(
                        "source.type-mismatch",
                        format!(
                            "closure argument #{} expects `{}`, found `{}`",
                            index + 1,
                            want.as_str(),
                            arg.value.vt.as_str()
                        ),
                        arg.value.span,
                    );
                }
            }
        }
        if args.len() != signature.params.len() {
            self.bind_err(
                "source.argument",
                format!(
                    "closure expects {} argument(s), found {}",
                    signature.params.len(),
                    args.len()
                ),
                call_span,
            );
        }
        ResolvedExpr {
            span: call_span,
            vt: signature.result,
            kind: RExprKind::CallClosure {
                callee: Box::new(callee),
                args,
            },
        }
    }

    /// Argument mapping: all-positional or all-named; names must exist and
    /// be unique; every formal is bound exactly once.
    /// Promote borrowing string literals to owning `Str` when they bind an
    /// owning formal (D-03 / AP-4): the same contextual typing a `let s Str`
    /// initializer gets, applied to METHOD call arguments — regular function
    /// calls retag in their own arm; `from` skips the receiver param.
    fn implicit_record_field(&self, name: &ast::Ident) -> Option<ResolvedExpr> {
        if !self.implicit_record_fields {
            return None;
        }
        let receiver = self.lookup("self")?;
        let VT::Rec(record) = self.locals[receiver.0].vt else {
            return None;
        };
        let (index, field) = self.records[record]
            .fields
            .iter()
            .enumerate()
            .find(|(_, field)| field.name == name.name)?;
        Some(ResolvedExpr {
            span: name.span,
            vt: field.vt,
            kind: RExprKind::FieldGet {
                base: Box::new(ResolvedExpr {
                    span: name.span,
                    vt: VT::Rec(record),
                    kind: RExprKind::ReadLocal(receiver),
                }),
                path: vec![(record, index)],
            },
        })
    }

    fn adapt_spec_value(&self, value: &mut ResolvedExpr, want: VT) {
        let (VT::Rec(_), VT::Variant(variant)) = (value.vt, want) else {
            return;
        };
        if !self.coll.spec_variants.contains(&variant) {
            return;
        }
        let Some(disc) = self.coll.variants[variant]
            .arms
            .iter()
            .position(|(_, payload)| *payload == Some(value.vt))
        else {
            return;
        };
        let mut original = std::mem::replace(value, instances::pending_expression(value.span));
        let payload = if self.vt_is_coll_owned(original.vt)
            && !matches!(original.kind, RExprKind::Move { .. })
        {
            self.clone_collection_value(original)
        } else {
            self.copy_record_parameter(&mut original);
            original
        };
        value.kind = RExprKind::VariantCtor {
            variant,
            disc: disc as u32,
            payload: Some(Box::new(payload)),
        };
        value.vt = want;
    }

    /// A copyable record parameter uses a view in MIR. When an ordinary
    /// expression needs a record value, copy that view at the boundary.
    fn copy_record_parameter(&self, value: &mut ResolvedExpr) {
        if !self.profile.is_auto_language() || !matches!(value.vt, VT::Rec(_)) {
            return;
        }
        let RExprKind::ReadLocal(local) = value.kind else {
            return;
        };
        let local = &self.locals[local.0];
        if !local.is_param || local.mode == ast::ParamMode::Move || self.vt_is_coll_owned(value.vt) {
            return;
        }
        let base = std::mem::replace(value, instances::pending_expression(value.span));
        value.vt = base.vt;
        value.kind = RExprKind::RecordClone { base: Box::new(base) };
    }

    fn adapt_argument_type(&self, value: &mut ResolvedExpr, want: VT) {
        retag_literal(value, want);
        self.adapt_spec_value(value, want);
        if self.profile.is_auto_language() && want == VT::Str && value.vt == VT::StrOwned {
            let span = value.span;
            let owned = std::mem::replace(
                value,
                ResolvedExpr {
                    span,
                    vt: VT::Str,
                    kind: RExprKind::Unit,
                },
            );
            value.kind = RExprKind::Method {
                recv: Box::new(owned),
                method: BuiltinMethod::To,
                to: Some(VT::Str),
                args: Vec::new(),
            };
        }
    }

    fn retag_method_args(&mut self, rargs: &mut [ResolvedArg], fn_index: usize, from: usize) {
        let param_names = self.method_sig_names(fn_index, from);
        let params = self.method_sig_params(fn_index, from);
        for (i, a) in rargs.iter_mut().enumerate() {
            let pos = if a.name.is_some() {
                param_names
                    .iter()
                    .position(|n| Some(n) == a.name.as_ref().map(|x| &x.name))
            } else {
                Some(i)
            };
            if let Some(pos) = pos.filter(|&p| p < params.len()) {
                self.adapt_argument_type(&mut a.value, params[pos]);
            }
        }
    }

    fn check_args(
        &mut self,
        callee: &str,
        param_names: &[String],
        params: &[VT],
        args: &[ResolvedArg],
        call_span: Span,
    ) {
        // PLAN-962: an argument may not materialize a whole-element view.
        self.check_argument_shape(callee, param_names, params.len(), args, call_span);
    }

    fn check_argument_shape(
        &mut self,
        callee: &str,
        param_names: &[String],
        param_count: usize,
        args: &[ResolvedArg],
        call_span: Span,
    ) {
        for a in args {
            self.reject_elem_view_use(&a.value, "the argument");
        }
        let named = args.iter().filter(|a| a.name.is_some()).count();
        if named > 0 && named < args.len() {
            // Mixed form: point at the first argument whose shape differs
            // from the first argument's shape.
            let first_is_named = args[0].name.is_some();
            if let Some(offender) = args.iter().find(|a| a.name.is_some() != first_is_named) {
                self.bind_err(
                    "source.argument",
                    "arguments must be either all positional or all named".to_string(),
                    offender.span,
                );
            }
            return;
        }
        if named == 0 {
            if args.len() != param_count {
                self.bind_err(
                    "source.argument",
                    format!(
                        "`{}` expects {} argument(s), found {}",
                        callee,
                        param_count,
                        args.len()
                    ),
                    call_span,
                );
            }
            return;
        }
        // All named.
        let mut seen: BTreeSet<&str> = BTreeSet::new();
        for a in args {
            let Some(name) = &a.name else { continue };
            if !seen.insert(name.name.as_str()) {
                self.bind_err(
                    "source.argument",
                    format!("argument `{}` is passed more than once", name.name),
                    a.span,
                );
            }
            if !param_names.iter().any(|p| p == &name.name) {
                self.bind_err(
                    "source.argument",
                    format!("`{}` has no parameter named `{}`", callee, name.name),
                    name.span,
                );
            }
        }
        for p in param_names {
            if !seen.contains(p.as_str()) {
                self.bind_err(
                    "source.argument",
                    format!("parameter `{}` of `{}` is not bound", p, callee),
                    call_span,
                );
            }
        }
    }
}

/// Refresh declaration coordinates after a source-map-only movement. Bodies
/// remain immutable resident payloads; callers publish their own span maps.
pub fn reanchor_env_declarations(
    env: &mut CheckedEnv,
    program: &ast::Program,
) -> Result<(), Vec<Diagnostic>> {
    let fns: Vec<_> = program
        .items
        .iter()
        .filter_map(|i| match i {
            ast::Item::Fn(f) => Some(f),
            _ => None,
        })
        .collect();
    if env.closed
        || fns.len() != env.program.fns.len()
        || env
            .program
            .fns
            .iter()
            .zip(&fns)
            .any(|(r, f)| r.name != f.name.name)
    {
        return Err(vec![Diagnostic::new(
            Stage::Bind,
            "source.module-context",
            "declaration context mismatch",
            Span { start: 0, end: 0 },
        )]);
    }
    for (r, f) in env.program.fns.iter_mut().zip(fns) {
        r.name_span = f.name.span;
        r.item_span = f.span;
    }
    Ok(())
}
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
use std::collections::{BTreeMap, BTreeSet};

/// Value type of the profile: `int`/`i32` are the same exact signed 32-bit
/// type; `bool` is distinct with no implicit conversions.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
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
        }
    }

    /// PLAN-864: whether this type owns resources under the collections
    /// contract (List/Variant containers, owned Str, records reaching
    /// Str). Copyable element types copy out; owned ones move.
    pub fn is_owned_coll(self, prog: &ResolvedProgram) -> bool {
        match self {
            VT::StrOwned | VT::List(_) | VT::Variant(_) | VT::Function(_) | VT::Object(_) => true,
            VT::Rec(ri) => record_reaches_str(ri, &prog.records),
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
        keys: Vec<String>,
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
            RExprKind::FieldGet { base, .. } | RExprKind::RecordClone { base } => {
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
            RExprKind::TextDisplay { operand } => {
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
}

/// The mutable type tables grown while bodies resolve (PLAN-864): `List<T>`
/// annotations and variant constructors intern entries on first use.
#[derive(Default)]
pub struct CollTables {
    pub lists: Vec<VT>,
    pub variants: Vec<ResolvedVariant>,
    pub function_types: Vec<ClosureFnType>,
    variant_names: BTreeMap<String, usize>,
    /// PLAN-962: aggregates-profile admission — nested List, composite
    /// record/variant field and payload positions, and the wide numeric
    /// element kinds are legal. The frozen 822 one-layer element domain
    /// stays in force for every older profile.
    pub aggregates_admission: bool,
    /// PLAN-962 (L-15): structural tuple shapes (element VT lists,
    /// interned/deduplicated; `VT::Tuple(i)` indexes here).
    pub tuples: Vec<Vec<VT>>,
    /// PLAN-964 (L-16/L-20): object element types (structural,
    /// deduplicated; `VT::Object(i)` indexes here).
    pub objects: Vec<VT>,
}

impl CollTables {
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
        if let Some(&i) = self.variant_names.get(name) {
            return i;
        }
        let i = self.variants.len();
        self.variant_names.insert(name.to_string(), i);
        self.variants.push(ResolvedVariant {
            name: name.to_string(),
            name_span,
            arms,
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

/// Transitive Str-reachability over the record table (PLAN-782 derivation,
/// reused by the collections owned-classification, PLAN-864).
pub(crate) fn record_reaches_str(ri: usize, records: &[ResolvedRecord]) -> bool {
    let mut stack: Vec<usize> = vec![ri];
    let mut seen: BTreeSet<usize> = BTreeSet::new();
    while let Some(r) = stack.pop() {
        if !seen.insert(r) {
            continue;
        }
        for f in &records[r].fields {
            match f.vt {
                VT::StrOwned => return true,
                VT::Rec(inner) => stack.push(inner),
                _ => {}
            }
        }
    }
    false
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
            // PLAN-960: module globals never reach the single-module closures
            // cache (the packages binder lowers them to init functions); their
            // declaration text is still part of the context identity.
            ast::Item::Const(c) => mix(&src[c.span.start..c.span.end], &mut h),
            ast::Item::Store(v) => mix(&src[v.span.start..v.span.end], &mut h),
            // PLAN-967 F0: the frontdoor spelling participates in the
            // module content identity (provider identity is load-bearing).
            ast::Item::UseRs(u) => mix(&src[u.span.start..u.span.end], &mut h),
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
            },
            rec_index: ix.rec_index,
            sigs: ix.sigs,
            trace_enabled: ix.trace_enabled,
            profile: SourceProfile::Closures,
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
    let (fns, records, coll, _) = resolve_detailed(program, capabilities, surface, None, None)?;
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
        resolve_detailed(program, capabilities, surface, None, Some(supply))?;
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
        resolve_detailed(program, capabilities, surface, None, Some(supply))?;
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
            // PLAN-960: module globals are callable-surface declarations for
            // the foreign directory (name/spans only; accessors are not fns).
            ast::Item::Const(c) => (c.name.name.as_str(), c.name.span, false),
            ast::Item::Store(v) => (v.name.name.as_str(), v.name.span, false),
            // PLAN-967 F0: the import frontdoor contributes no callable
            // name; it is validated against the admitted provider below.
            ast::Item::UseRs(_) => ("", Span { start: 0, end: 0 }, false),
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
            if u.provider.name != foreign.manifest().provider {
                return Err(vec![Diagnostic::new(
                    Stage::Bind,
                    "source.use-rs.provider",
                    format!(
                        "`use.rs {}` does not match the admitted provider `{}`",
                        u.provider.name,
                        foreign.manifest().provider
                    ),
                    u.provider.span,
                )]);
            }
            use_rs_seen = Some(u.span);
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

fn resolve_detailed(
    program: &ast::Program,
    capabilities: &[String],
    surface: SourceProfile,
    mut warm: Option<&mut WarmIo<'_>>,
    supply: Option<&dyn TypeSupply>,
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
    let trace_enabled = capabilities.iter().any(|c| c == TRACE_CAPABILITY);
    // PLAN-936（T-04）：meta 查询与 CT 表达式能力（各自独立门；meta 默认关）。
    let meta_enabled = capabilities.iter().any(|c| c == META_CAPABILITY);
    let ct_enabled = capabilities.iter().any(|c| c == super::cst::CT_CAPABILITY);
    let mut diags = Vec::new();
    let mut coll = CollTables::default();
    // PLAN-962: the aggregates surface widens the collections admission.
    coll.aggregates_admission = surface.is_aggregates();
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
            rec_index.insert(t.name.name.clone(), records.len());
            records.push(ResolvedRecord {
                name: t.name.name.clone(),
                name_span: t.name.span,
                fields: Vec::new(),
                methods: Vec::new(),
            });
        }
    }
    // PLAN-889: resolve nominal variant references through the same type
    // table in every annotation position. Register names before payloads,
    // so acyclic forward references do not depend on declaration order.
    // The completed graph is checked below before any body is resolved.
    let mut enum_indices = BTreeMap::new();
    if is_coll {
        for (item_index, item) in program.items.iter().enumerate() {
            if let ast::Item::Enum(e) = item {
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
            }
        }
    }
    // Fill field shapes (all record names are known now).
    for item in &program.items {
        if let ast::Item::Type(t) = item {
            let Some(&ri) = rec_index.get(&t.name.name) else {
                continue;
            };
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
                    if !t.methods.is_empty() {
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
            return Err(diags);
        }
    }

    // Pass 1: signature pre-registration with duplicate detection.
    let mut sigs: BTreeMap<String, FnSig> = BTreeMap::new();
    let mut fn_receiver: Vec<ast::Receiver> = Vec::new();
    let mut fn_owner: Vec<Option<usize>> = Vec::new();
    let mut flat: Vec<&ast::FnItem> = Vec::new();
    for item in &program.items {
        match item {
            ast::Item::Fn(f) => {
                fn_receiver.push(f.receiver);
                fn_owner.push(None);
                flat.push(f);
            }
            // PLAN-967 F0: the frontdoor declares no function.
            ast::Item::UseRs(_) => {}
            ast::Item::Type(t) => {
                for m in &t.methods {
                    fn_receiver.push(m.receiver);
                    fn_owner.push(rec_index.get(&t.name.name).copied());
                    flat.push(m);
                }
            }
            // PLAN-864: enums carry no functions.
            ast::Item::Enum(_) => {}
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
                let vt = vt_of_ann(ty, &rec_index, &mut diags);
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
        }
    }
    for (i, f) in flat.iter().enumerate() {
        let _ = &f;
        let owner_ty = fn_owner[i]
            .map(|oi| records[oi].name.clone())
            .unwrap_or_default();
        let sig_key = if fn_receiver[i] == ast::Receiver::Free {
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
            &rec_index,
            &mut coll,
            &mut diags,
            i,
            is_coll,
        );
        sigs.insert(sig_key.clone(), sig);
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
                fn_index: i,
                span: f.span,
            });
        }
    }

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
                    index: flat.len() + index,
                },
            );
        }
        if !diags.is_empty() {
            return Err(diags);
        }
    }
    // Pass 2: per-function body resolution. PLAN-909 §5.5: with a warm
    // cache present, an owner whose ITEM TEXT and the module's DECLARATION
    // CONTEXT are unchanged reuses its resolved entry (and its lambdas)
    // without a resolve walk — the reuse is a real work skip, observed as
    // zero `resolve` body-work for that owner.
    let mut fns = Vec::new();
    let mut closure_fns = Vec::new();
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
        let lambdas_before = closure_fns.len();
        let mut sites = Vec::new();
        fns.push(resolve_fn(
            f,
            &sigs,
            trace_enabled,
            meta_enabled,
            ct_enabled,
            surface,
            &mut diags,
            fn_receiver[i],
            fn_owner[i],
            &records,
            &rec_index,
            &mut coll,
            &mut closure_fns,
            &mut sites,
            flat.len(),
            supply,
        ));
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
    fns.extend(closure_fns);
    if let Some(w) = warm.as_mut() {
        w.out.function_types = coll.function_types.clone();
    }

    // PLAN-862 entry matrix: the entry-unit profile has exactly one entry
    // and a frozen accept/reject set (`entry-contract.md` §2). Runs after
    // Pass 2 so a script's result type can be derived from its tail
    // statement's RESOLVED type.
    if surface.is_entry_unit() {
        check_entry_matrix(&mut fns, &flat, &mut diags);
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
fn check_entry_matrix(fns: &mut [ResolvedFn], flat: &[&ast::FnItem], diags: &mut Vec<Diagnostic>) {
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
    let (fns, records, coll, ix) = resolve_detailed(program, capabilities, surface, None, supply)?;
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
        },
        rec_index: ix.rec_index,
        sigs: ix.sigs,
        trace_enabled: ix.trace_enabled,
        profile: surface,
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
        &mut diags,
        recv,
        owner_rec,
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
            sigs: &env.sigs,
            trace_enabled: env.trace_enabled,
            meta_enabled: false,
            ct_enabled: false,
            ct_depth: 0,
            profile: SourceProfile::Closures,
            // callback-by-name is a text-record-only surface; the closures
            // face never reads the boundary.
            source_fn_count: 0,
            diags: &mut diags,
            locals: Vec::new(),
            scopes: vec![Scope {
                names: BTreeMap::new(),
                views: BTreeMap::new(),
            }],
            declared_names: BTreeSet::new(),
            fn_result: site.result,
            loop_stack: Vec::new(),
            loop_count: 0,
            used_intrinsics: BTreeSet::new(),
            closure_defs: &mut nested_defs,
            moved_locals: BTreeSet::new(),
            borrowed_closure_locals: BTreeSet::new(),
            records: &env.program.records,
            rec_index: &env.rec_index,
            coll: &mut coll,
            iterate_targets: Vec::new(),
            shared_cell_spans,
            owner_name: site.owner.clone(),
            closure_count: site.ordinal,
            lambda_sites: &mut nested_sites,
            reuse: None,
            supply: None,
        };
        r.lambda_body(params, body, &site.capture_decls, expr.span)
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
        &mut diags,
        ast::Receiver::Free,
        None,
        &env.program.records,
        &env.rec_index,
        &mut coll,
        &mut closure_defs,
        &mut sites,
        env.program.fns.len(),
        Some(&mut reuse),
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
    let copy = matches!(source_vt, VT::I32 | VT::Bool | VT::Char);
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
fn fn_sig_of_coll(
    f: &ast::FnItem,
    recv: ast::Receiver,
    owner_rec: Option<usize>,
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
    if recv == ast::Receiver::View {
        param_names.push("self".to_string());
        params.push(VT::Rec(owner_rec.unwrap_or(usize::MAX)));
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
    let mut defaults: Vec<Option<Box<ast::Expr>>> =
        vec![None; usize::from(recv == ast::Receiver::View)];
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
    match &ann.kind {
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
                let vts: Vec<VT> = args
                    .iter()
                    .map(|a| vt_of_ann_coll(a, rec_index, coll, diags, TypePos::Local))
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
            if matches!(pos, TypePos::Field | TypePos::Param) {
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
            // PLAN-962 (T-08, L-08): `Box<T>` — explicit single-value
            // indirection. Representation: the single-element List cell
            // (t01-freeze D9 — the closure-cell precedent); a Box-typed
            // field is therefore a LIST-typed field: fixed-size handle,
            // and the record containment cycle walk does not follow it
            // (legal recursion: `type Node { next Box<Node> }`).
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
                    VT::StrOwned | VT::I32 | VT::Bool | VT::Rec(_) | VT::Variant(_) => {
                        VT::List(coll.intern_list(elem))
                    }
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
                if matches!(pos, TypePos::Field | TypePos::Param) {
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
    }
}

struct Scope {
    names: BTreeMap<String, LocalRef>,
    /// PLAN-864: owned-element `for-in` bindings — a read-only projection
    /// context, never a materialized local. Lookup falls here when `names`
    /// misses; any use other than a field path / `.len()` is
    /// `source.view-escape`.
    views: BTreeMap<String, ViewBinding>,
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
enum RecvClass {
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
    /// PLAN-864: `<list<Str>-local>.get(i).len()` — a whole-Str element
    /// view consumed by len (contract C4).
    ElemViewLen {
        xl: LocalRef,
        xvt: VT,
        idx_slot: usize,
    },
}

struct FnResolver<'a, 's> {
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
    /// Result type of the function being resolved (retag context).
    fn_result: VT,
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
}

fn resolve_fn(
    f: &FnItem,
    sigs: &BTreeMap<String, FnSig>,
    trace_enabled: bool,
    meta_enabled: bool,
    ct_enabled: bool,
    surface: SourceProfile,
    diags: &mut Vec<Diagnostic>,
    receiver: ast::Receiver,
    owner_rec: Option<usize>,
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
        diags,
        receiver,
        owner_rec,
        records,
        rec_index,
        coll,
        closure_defs,
        lambda_sites,
        closure_base,
        None,
        supply,
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
    diags: &'a mut Vec<Diagnostic>,
    receiver: ast::Receiver,
    owner_rec: Option<usize>,
    records: &'a [ResolvedRecord],
    rec_index: &'a BTreeMap<String, usize>,
    coll: &'a mut CollTables,
    closure_defs: &'a mut Vec<ResolvedFn>,
    lambda_sites: &'a mut Vec<LambdaSiteFacts>,
    _closure_base: usize,
    reuse: Option<&'a mut LambdaReuse<'s>>,
    supply: Option<&'a dyn TypeSupply>,
) -> ResolvedFn {
    let is_coll = surface.is_collections();
    let fn_result = if is_coll {
        vt_of_ann_coll(&f.ret, rec_index, coll, diags, TypePos::Result)
    } else {
        vt_of_ann(&f.ret, rec_index, diags)
    };
    let mut r = FnResolver {
        sigs,
        trace_enabled,
        meta_enabled,
        ct_enabled,
        ct_depth: 0,
        profile: surface,
        diags,
        locals: Vec::new(),
        scopes: vec![Scope {
            names: BTreeMap::new(),
            views: BTreeMap::new(),
        }],
        declared_names: BTreeSet::new(),
        fn_result,
        loop_stack: Vec::new(),
        loop_count: 0,
        used_intrinsics: BTreeSet::new(),
        closure_defs,
        moved_locals: BTreeSet::new(),
        borrowed_closure_locals: BTreeSet::new(),
        records,
        rec_index,
        coll,
        iterate_targets: Vec::new(),
        shared_cell_spans: super::closure_capture::scan_shared_capture_cells(&f.params, &f.body),
        owner_name: f.name.name.clone(),
        closure_count: 0,
        lambda_sites,
        reuse,
        supply,
        source_fn_count: _closure_base,
    };
    let mut param_names: Vec<String> = Vec::new();
    let mut params: Vec<VT> = Vec::new();
    let mut param_modes: Vec<bool> = Vec::new();
    // Params live in the function's outermost scope. A view receiver is
    // the implicit `self` param 0 (PLAN-771 §5.3).
    if receiver == ast::Receiver::View {
        let ri = owner_rec.unwrap_or(usize::MAX);
        let lref = LocalRef(r.locals.len());
        r.locals.push(LocalDecl {
            name: "self".to_string(),
            vt: VT::Rec(ri),
            mutable: false,
            is_param: true,
            param_index: Some(0),
            capture_index: None,
            cell: false,
            is_capture: false,
            mode: super::ast::ParamMode::Default,
            span: f.name.span,
        });
        r.scopes[0].names.insert("self".to_string(), lref);
        r.declared_names.insert("self".to_string());
        param_names.push("self".to_string());
        params.push(VT::Rec(ri));
        param_modes.push(false);
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
            param_index: Some(
                (i + if receiver == ast::Receiver::View {
                    1
                } else {
                    0
                }) as u32,
            ),
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
    let body = r.resolve_body(&f.body);
    let sig_key = if receiver == ast::Receiver::Free {
        f.name.name.clone()
    } else {
        format!(
            "{}::{}",
            owner_rec
                .map(|oi| records[oi].name.clone())
                .unwrap_or_default(),
            f.name.name
        )
    };
    let sig = sigs.get(&sig_key);
    ResolvedFn {
        name: f.name.name.clone(),
        name_span: f.name.span,
        item_span: f.span,
        param_names: if receiver == ast::Receiver::View {
            param_names
        } else {
            sig.map(|g| g.param_names.clone()).unwrap_or(param_names)
        },
        params: if receiver == ast::Receiver::View {
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

    fn lookup(&self, name: &str) -> Option<LocalRef> {
        for scope in self.scopes.iter().rev() {
            if let Some(l) = scope.names.get(name) {
                return Some(*l);
            }
        }
        None
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
            // PLAN-962: a range is only a for-in iterable, never a value.
            ExprKind::Range { .. } => VT::I32,
            // PLAN-962: literals/indexes never reach closure inference
            // (statement-level desugar owns them); totals for
            // exhaustiveness.
            ExprKind::ArrayLit(_) => VT::I32,
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
                    // PLAN-962: an index assign is an effect statement.
                    Stmt::IndexAssign(_) => {}
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
                    | Stmt::Continue { .. } => {}
                }
            }
        }
        VT::Unit
    }

    fn resolve_closure(
        &mut self,
        move_capture: bool,
        params_ast: &[ast::Param],
        body_ast: &ast::ClosureBody,
        span: Span,
    ) -> ResolvedExpr {
        if self.profile != SourceProfile::Closures {
            self.bind_err(
                "source.unsupported",
                "lambda expressions are available only in the closures source profile",
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
        let lb = self.lambda_body(params_ast, body_ast, &capture_decls, span);
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
        let inferred_result = match body_ast {
            ast::ClosureBody::Expr(expr) => self.infer_closure_expr(expr, &visible),
            ast::ClosureBody::Block(block) => self.infer_closure_block_result(block, &visible),
        };
        self.fn_result = inferred_result;
        let (resolved_body, result) = match body_ast {
            ast::ClosureBody::Expr(expr) => {
                let value = self.expr_iterative(expr);
                let result = value.vt;
                (
                    RBlock {
                        stmts: vec![RStmt::Return {
                            value,
                            span: expr.span,
                        }],
                        span,
                    },
                    result,
                )
            }
            ast::ClosureBody::Block(block) => {
                let mut resolved = self.resolve_body(block);
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
            VT::StrOwned => ResolvedExpr {
                span,
                vt: VT::StrOwned,
                kind: RExprKind::StrLit(String::new()),
            },
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
            VT::Rec(ri) => record_reaches_str(ri, self.records),
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
    fn classify_generic_recv(&mut self, name: &ast::Ident, args: &[ast::TypeAnn]) -> RecvClass {
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
    fn check_qualified_ctor_args(&mut self, vi: usize, disc: u32, args: &[ast::Arg], span: Span) {
        let has_payload = self.coll.variants[vi].arms[disc as usize].1.is_some();
        let n = args.len();
        match (has_payload, n) {
            (true, 1) => {}
            (false, 0) => {}
            (true, _) => self.bind_err(
                "source.argument",
                "this variant arm takes exactly one payload value".to_string(),
                span,
            ),
            (false, _) => self.bind_err(
                "source.argument",
                "this variant arm carries no payload".to_string(),
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
            None => self.expr_iterative(v),
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
            if self.coll.variants[vi].name != q.name {
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
    fn resolve_list_method(
        &mut self,
        li: usize,
        name: &ast::Ident,
        recv: ResolvedExpr,
        mut rargs: Vec<ResolvedArg>,
        span: Span,
    ) -> ResolvedExpr {
        // structural modification of an iterated list is a compile reject
        if matches!(name.name.as_str(), "push" | "insert" | "remove" | "clear") {
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
        let elem = self.coll.lists[li];
        let mk = |vt: VT, kind: RExprKind| ResolvedExpr { span, vt, kind };
        match name.name.as_str() {
            "push" if rargs.len() == 1 => {
                let mut value = rargs.remove(0).value;
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
                let mut value = rargs.remove(0).value;
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
                    if !self.coll.aggregates_admission {
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
                let mut value = rargs.remove(0).value;
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

    /// PLAN-864: push/insert/set values of owned element type must be a
    /// move-source (`.move` local or a fresh rvalue); Copy elements pass
    /// by value.
    fn check_push_value(&mut self, elem: &VT, value: &mut ResolvedExpr, span: Span) {
        let _ = span;
        // PLAN-962: storing a whole-element view would alias an owned
        // element into another container.
        self.reject_elem_view_use(value, "the stored value");
        if !self.vt_is_coll_owned(*elem) {
            return;
        }
        match &value.kind {
            RExprKind::Move { .. } => {}
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
            RExprKind::StrLit(_) | RExprKind::VariantCtor { .. } => {
                // a fresh rvalue transfers at commit
                value.vt = *elem;
            }
            RExprKind::ReadLocal(_) => {
                self.bind_err(
                    "source.move-required",
                    "pushing an owned element requires `.move` (or a fresh value); an implicit transfer is rejected".to_string(),
                    span,
                );
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

    /// Resolve one function body with an explicit block-frame stack (R6:
    /// block nesting is unlimited, so no recursion). Each frame owns a
    /// lexical scope (pushed on open, popped on close); while/loop frames
    /// additionally hold their loop target on the shared loop stack for the
    /// duration of the body. Statement order — and therefore diagnostic
    /// order, loop-id registration order and shadowing — is identical to
    /// the previous recursive walk.
    fn resolve_body(&mut self, root: &ast::Block) -> RBlock {
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
        });
        let mut stack: Vec<RFrame> = vec![RFrame {
            src: root,
            idx: 0,
            stmts: Vec::new(),
            cont: RCont::FnBody,
        }];
        loop {
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
                let block = RBlock {
                    stmts: frame.stmts,
                    span: frame.src.span,
                };
                match frame.cont {
                    RCont::FnBody => return block,
                    RCont::ThenArm { cond, els, span } => match els {
                        Some(eb) => {
                            self.scopes.push(Scope {
                                names: BTreeMap::new(),
                                views: BTreeMap::new(),
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
                        stack
                            .last_mut()
                            .expect("parent frame")
                            .stmts
                            .push(RStmt::While {
                                cond,
                                body: block,
                                span,
                                target,
                            });
                    }
                    RCont::LoopArm { span, target } => {
                        self.loop_stack.pop();
                        stack
                            .last_mut()
                            .expect("parent frame")
                            .stmts
                            .push(RStmt::Loop {
                                body: block,
                                span,
                                target,
                            });
                    }
                }
                continue;
            };
            match s {
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
                    });
                    self.scopes
                        .last_mut()
                        .unwrap()
                        .names
                        .insert(bind.name.clone(), xl);
                    let mut b = self.resolve_body(body);
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
                    if let VT::Object(oi) = base_vt {
                        let elem = self.coll.objects[oi];
                        let index = self.expr_iterative(&ia.index);
                        if !index.vt.is_text() {
                            self.bind_err(
                                "source.object-key",
                                "an object key must be `Str` in this profile".to_string(),
                                ia.index.span,
                            );
                        }
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
                Stmt::Let {
                    name,
                    ann,
                    mutable,
                    value,
                    span,
                    ..
                } => {
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
                    if let ExprKind::ArrayLit(elems_ast) = &value.kind {
                        if elems_ast.is_empty() {
                            self.bind_err(
                                "source.array-lit-empty",
                                "an empty array literal has no element type; write List<T>.new() instead"
                                    .to_string(),
                                value.span,
                            );
                            continue;
                        }
                        let mut elems = Vec::with_capacity(elems_ast.len());
                        for e in elems_ast {
                            elems.push(self.expr_iterative(e));
                        }
                        let elem_vt = elems[0].vt;
                        for (e, ast_e) in elems.iter().zip(elems_ast.iter()) {
                            if e.vt != elem_vt {
                                self.bind_err(
                                    "source.type-mismatch",
                                    format!(
                                        "array literal elements must share one type; first is `{}`, found `{}`",
                                        elem_vt.as_str(),
                                        e.vt.as_str()
                                    ),
                                    ast_e.span,
                                );
                            }
                            self.reject_elem_view_use(e, "the literal element");
                        }
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
                    let ann_vt: Option<VT> = match ann {
                        Some(a) => {
                            if self.profile.is_collections() {
                                Some(vt_of_ann_coll(
                                    a,
                                    self.rec_index,
                                    self.coll,
                                    &mut Vec::new(),
                                    TypePos::Local,
                                ))
                            } else {
                                Some(vt_of_ann(a, self.rec_index, &mut Vec::new()))
                            }
                        }
                        None => None,
                    };
                    let mut value = self.expr_with_expect(value, ann_vt);
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
                    let lref = LocalRef(self.locals.len());
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
                    let expect = self.lookup(&target.base.name).map(|l| self.locals[l.0].vt);
                    let mut value = self.expr_with_expect(value, expect);
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
                Stmt::ExprStmt { value, span } => {
                    let value = self.expr_iterative(value);
                    stack
                        .last_mut()
                        .expect("resolve frame")
                        .stmts
                        .push(RStmt::ExprStmt { value, span: *span });
                }
                Stmt::If {
                    cond,
                    then,
                    els,
                    span,
                } => {
                    let cond = self.expr_iterative(cond);
                    self.scopes.push(Scope {
                        names: BTreeMap::new(),
                        views: BTreeMap::new(),
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
                    let cond = self.expr_iterative(cond);
                    let target = LoopRef(self.loop_count);
                    self.loop_count += 1;
                    self.loop_stack.push(target);
                    self.scopes.push(Scope {
                        names: BTreeMap::new(),
                        views: BTreeMap::new(),
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
                Stmt::Loop { body, span } => {
                    let target = LoopRef(self.loop_count);
                    self.loop_count += 1;
                    self.loop_stack.push(target);
                    self.scopes.push(Scope {
                        names: BTreeMap::new(),
                        views: BTreeMap::new(),
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
                Stmt::Continue { span } => {
                    let target = self.loop_stack.last().copied();
                    stack
                        .last_mut()
                        .expect("resolve frame")
                        .stmts
                        .push(RStmt::Continue {
                            target,
                            span: *span,
                        });
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
                        let body = self.resolve_body(&arm.body);
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
                    let filler = self.default_return_filler(*span);
                    let mut current = Some(RBlock {
                        stmts: vec![RStmt::Return {
                            value: filler,
                            span: *span,
                        }],
                        span: *span,
                    });
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
                        });
                        self.scopes
                            .last_mut()
                            .unwrap()
                            .names
                            .insert(var.name.clone(), i_l);
                        self.declared_names.insert(var.name.clone());
                        let mut b = self.resolve_body(body);
                        self.scopes.pop();
                        self.loop_stack.pop();
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
                        b.stmts.push(incr);
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
                            body: RBlock {
                                stmts: vec![if_stmt],
                                span: body.span,
                            },
                            span: *span,
                        });
                        stack
                            .last_mut()
                            .expect("resolve frame")
                            .stmts
                            .extend(prologue);
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
                    let body_res = if owned {
                        self.scopes.push(Scope {
                            names: BTreeMap::new(),
                            views: BTreeMap::new(),
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
                        let b = self.resolve_body(body);
                        self.scopes.pop();
                        b
                    } else {
                        self.scopes.push(Scope {
                            names: BTreeMap::new(),
                            views: BTreeMap::new(),
                        });
                        let blref = self.fresh_local(&var.name, elem, false);
                        self.scopes
                            .last_mut()
                            .unwrap()
                            .names
                            .insert(var.name.clone(), blref);
                        self.declared_names.insert(var.name.clone());
                        let mut b = self.resolve_body(body);
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
                    let mut body_res = body_res;
                    let mut then_stmts = std::mem::take(&mut body_res.stmts);
                    then_stmts.push(incr);
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
                    });
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
            // PLAN-962 (L-16): array literals desugar ONLY in the
            // let/var initializer position (the Let arm intercepts);
            // every other expression position is a named rejection.
            ExprKind::ArrayLit(elems) => {
                let _ = elems;
                self.bind_err(
                    "source.array-lit-position",
                    "an array literal is only a let/var initializer in this profile (bind it to a name, then use the list)"
                        .to_string(),
                    e.span,
                );
                ResolvedExpr {
                    span: e.span,
                    vt: VT::I32,
                    kind: RExprKind::IntLit(0),
                }
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
                let recv = self.expr_iterative(base);
                let idx = self.expr_iterative(index);
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
                            if !self.coll.aggregates_admission {
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
                    // PLAN-964 (L-20): object dynamic-key read.
                    VT::Object(oi) => {
                        if !idx.vt.is_text() {
                            self.bind_err(
                                "source.object-key",
                                "an object key must be `Str` in this profile".to_string(),
                                index.span,
                            );
                        }
                        let elem = self.coll.objects[oi];
                        ResolvedExpr {
                            span: e.span,
                            vt: elem,
                            kind: RExprKind::ObjectGet {
                                base: Box::new(recv),
                                key: Box::new(idx),
                            },
                        }
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
                let mut elem: Option<VT> = None;
                let mut dup: std::collections::HashSet<&str> = std::collections::HashSet::new();
                for f in fields {
                    let value = self.expr_iterative(&f.value);
                    if !dup.insert(f.key.as_str()) {
                        self.bind_err(
                            "source.object-duplicate",
                            format!("duplicate object key `{}`", f.key),
                            f.span,
                        );
                    }
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
                body,
            } => self.resolve_closure(*move_capture, params, body, e.span),
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
                    if self.sigs.contains_key(&id.name) {
                        self.bind_err(
                            "source.fn-as-value",
                            format!(
                                "function `{}` cannot be used as a value in this profile",
                                id.name
                            ),
                            id.span,
                        );
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
                    if !matches!(vt, VT::Rec(_)) {
                        self.bind_err(
                            "source.type-mismatch",
                            format!(
                                "`.view` requires a record-typed local, `{}` is `{}`",
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
    ) -> ResolvedExpr {
        if let Some(rrecv) = recv {
            // PLAN-782: on the combination surface the record dot postfix
            // claims method chains first, so a TEXT receiver's builtin
            // call (`t.text.len()`) arrives here — route it into the text
            // builtin table instead of rejecting.
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
                let sig_params = self.method_sig_params(m.fn_index, 1);
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
        let mut rargs = self.resolve_args(args);
        match self.records[ri]
            .methods
            .iter()
            .find(|m| m.name == name.name)
        {
            Some(m) if !m.is_static => {
                // receiver occupies param 0; visible args bind from param 1
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
                        recv: Some(Box::new(rrecv)),
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
        if defaults.iter().all(|d| d.is_none()) || rargs.len() >= params.len() {
            return;
        }
        let all_named = !rargs.is_empty() && rargs.iter().all(|a| a.name.is_some());
        if !all_named && rargs.iter().any(|a| a.name.is_some()) {
            // 混合形式：由 check_args 报同一诊断，不在此处再报一次。
            return;
        }
        let mut covered = vec![false; params.len()];
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
            /// All argument slots are filled; resolve callee/bindings into
            /// `dest` (R5: nested calls must not recurse through call()).
            CallOut {
                callee: &'a ast::Ident,
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
                node_span: Span,
                to_word: Option<TypeWord>,
            },
            /// Receiver + arg slots are filled (receiver slot `None` for
            /// pre-classified static calls); resolve the RECORD method
            /// into `dest` (PLAN-771).
            RecordMethodOut {
                name: ast::Ident,
                dest: usize,
                recv_slot: Option<usize>,
                recv_kind: RecvClass,
                arg_slots: Vec<usize>,
                names: Vec<Option<ast::Ident>>,
                spans: Vec<Span>,
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
                        body,
                    } = &node.kind
                    {
                        out[slot] = self.resolve_closure(*move_capture, params, body, node.span);
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
                    if let ExprKind::Call { callee, args } = &node.kind {
                        // PLAN-936（T-04）：meta 查询内在——selector 实参在
                        // 调度前拦截（不作为子表达式任务解析）。
                        // With no meta grant an actual user declaration keeps
                        // its ordinary meaning, including inside a CT demand.
                        if self.meta_enabled
                            || (self.ct_depth > 0 && !self.sigs.contains_key(&callee.name))
                        {
                            if let Some(op) = meta_intrinsic(&callee.name) {
                                out[slot] = self.meta_query(op, callee, args, node.span);
                                continue;
                            }
                        }
                        if args.is_empty() {
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
                                    spans.push(e.span);
                                    if let ExprKind::TypeWordArg(w) = &e.kind {
                                        to_word = Some(*w);
                                    } else {
                                        subtasks.push(Task::Visit { node: e, slot: sl });
                                    }
                                }
                                Arg::MovePos(e) => {
                                    names.push(None);
                                    spans.push(e.span);
                                    subtasks.push(Task::Visit { node: e, slot: sl });
                                }
                                Arg::Named { name, value, span } => {
                                    names.push(Some(ast::Ident {
                                        name: name.name.clone(),
                                        span: name.span,
                                    }));
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
                                                                _ => unreachable!(
                                                                    "get takes one positional index"
                                                                ),
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
                    } else if let ExprKind::MethodCall { recv, name, args } = &node.kind {
                        // static pre-classification: bare record type name
                        // receiver resolves to no local — the receiver
                        // expression is NOT evaluated for static calls.
                        // PLAN-864: classify the receiver before resolving —
                        // generic type ctors (`List<T>.new()`) and qualified
                        // variant ctors (`E.A(e)`) carry a TYPE, not a value;
                        // their receiver slot stays empty.
                        let recv_class = if let ExprKind::GenericType { name, args } = &recv.kind {
                            self.classify_generic_recv(name, args)
                        } else if let ExprKind::Var(id) = &recv.kind {
                            if self.lookup(&id.name).is_none()
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
                            } = &recv.kind
                            {
                                if gn.name == "get"
                                    && ga.len() == 1
                                    && matches!(&ga[0], ast::Arg::Pos(_) | ast::Arg::MovePos(_))
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
                                                        _ => unreachable!(
                                                            "get takes one positional index"
                                                        ),
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
                        let mut subtasks = Vec::with_capacity(args.len() + 1);
                        for a in args {
                            let sp = out.len();
                            out.push(dummy_resolved());
                            arg_slots.push(sp);
                            match a {
                                Arg::Pos(e) | Arg::MovePos(e) => {
                                    names.push(None);
                                    spans.push(e.span);
                                    subtasks.push(Task::Visit { node: e, slot: sp });
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
                            dest: slot,
                            recv_slot,
                            recv_kind: recv_class,
                            arg_slots,
                            names,
                            spans,
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
                    let base = std::mem::replace(&mut out[bslot], dummy_resolved());
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
                Task::RecordMethodOut {
                    name,
                    dest,
                    recv_slot,
                    recv_kind,
                    arg_slots,
                    names,
                    spans,
                    node_span,
                } => {
                    let mut rargs = Vec::with_capacity(arg_slots.len());
                    for (i, sp) in arg_slots.iter().enumerate() {
                        let value = std::mem::replace(&mut out[*sp], dummy_resolved());
                        rargs.push(ResolvedArg {
                            span: spans[i],
                            name: names[i].clone(),
                            moved: false,
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
                        RecvClass::Value => {
                            out[dest] = self.method_call_out(rrecv, None, &name, rargs, node_span);
                        }
                        RecvClass::Static(ri) => {
                            out[dest] =
                                self.method_call_out(rrecv, Some(ri), &name, rargs, node_span);
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
                        RecvClass::VariantArm { vi, disc } => {
                            // PLAN-864: `E.A(e)` / `E.A` — qualified variant
                            // construction; the payload (if any) is the
                            // single resolved argument.
                            let payload = match rargs.len() {
                                0 => None,
                                1 => {
                                    let mut value = rargs.remove(0).value;
                                    if let Some(expected) =
                                        self.coll.variants[vi].arms[disc as usize].1
                                    {
                                        retag_literal(&mut value, expected);
                                    }
                                    Some(Box::new(value))
                                }
                                _ => None, // rejected at classification
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
                    dest,
                    arg_slots,
                    names,
                    spans,
                    node_span,
                } => {
                    let mut rargs = Vec::with_capacity(arg_slots.len());
                    for (i, s) in arg_slots.iter().enumerate() {
                        let mut value = std::mem::replace(&mut out[*s], dummy_resolved());
                        // PLAN-782: in the combination profile the
                        // first-class move wrapper IS the argument value
                        // (CallBinding.moved stays text-profile only).
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
                    out[dest] = self.resolve_call(callee, rargs, node_span);
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
                    out[dest] = self.resolve_closure_call(callee, args, node_span);
                }
                Task::MethodOut {
                    name,
                    dest,
                    recv_slot,
                    arg_slots,
                    names,
                    spans,
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
                    let vt = bin_op_vt(op);
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
    fn resolve_method(
        &mut self,
        name: &ast::Ident,
        recv: ResolvedExpr,
        args: Vec<ResolvedArg>,
        to_word: Option<TypeWord>,
        span: Span,
    ) -> ResolvedExpr {
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
            for a in rargs.iter() {
                match (&a.value.kind, a.value.vt) {
                    // PLAN-964: any Str/StrOwned-typed part passes through
                    // as text (literals AND reads of Str locals).
                    (_, VT::Str | VT::StrOwned) => {
                        parts.push(FStrPart::Text(a.value.clone()));
                    }
                    (_, VT::I32) => {
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
                    (_, other_vt) => {
                        self.bind_err(
                            "source.fstr-arg",
                            format!(
                                "FStr interpolation admits `Str`/`int` parts, found `{}`",
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
        if let Some(sym) = callee.name.strip_prefix("module.global.get:") {
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

        if self.profile == SourceProfile::HostConsole
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
            let arg = rargs.remove(0).value;
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
        if self.profile == SourceProfile::HostConsole {
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
            if matches!(self.locals[local.0].vt, VT::Function(_)) {
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
                    vt: self.locals[local.0].vt,
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

        if let Some(sig) = self.sigs.get(&callee.name).cloned() {
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
                    retag_literal(&mut a.value, sig.params[pos]);
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

    fn resolve_closure_call(
        &mut self,
        callee: ResolvedExpr,
        mut args: Vec<ResolvedArg>,
        call_span: Span,
    ) -> ResolvedExpr {
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
                retag_literal(&mut arg.value, want);
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
                retag_literal(&mut a.value, params[pos]);
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
            if args.len() != params.len() {
                self.bind_err(
                    "source.argument",
                    format!(
                        "`{}` expects {} argument(s), found {}",
                        callee,
                        params.len(),
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

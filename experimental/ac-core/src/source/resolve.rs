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
use super::{trace_intrinsic, SourceProfile, TRACE_CAPABILITY};
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
}

impl VT {
    pub fn as_str(&self) -> &'static str {
        match self {
            VT::I32 => "i32",
            VT::Bool => "bool",
            VT::Str => "str",
            VT::StrOwned => "Str",
            VT::Char => "char",
            VT::Rec(_) => "record",
            VT::Unit => "unit",
            VT::List(_) => "list",
            VT::Variant(_) => "variant",
            VT::Function(_) => "function",
        }
    }

    /// PLAN-864: whether this type owns resources under the collections
    /// contract (List/Variant containers, owned Str, records reaching
    /// Str). Copyable element types copy out; owned ones move.
    pub fn is_owned_coll(self, prog: &ResolvedProgram) -> bool {
        match self {
            VT::StrOwned | VT::List(_) | VT::Variant(_) | VT::Function(_) => true,
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
    To,
    ToCodepoint,
    FromCodepoint,
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

/// Index into [`ResolvedFn::locals`].
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
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
    /// PLAN-862: the unit value — the operand of the synthesized return at
    /// the end of a Unit function (and of a fall-off script). It carries no
    /// payload and is never produced by user syntax.
    Unit,
    CallFn {
        fn_index: usize,
        args: Vec<ResolvedArg>,
    },
    CallIntrinsic {
        symbol: &'static str,
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
            // PLAN-862: leaf, nothing to detach.
            RExprKind::Unit => {}
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
            RExprKind::CallFn { args, .. } | RExprKind::CtorPos { args, .. } => {
                for a in args.iter_mut() {
                    let v = std::mem::replace(&mut a.value, dummy());
                    stack.push(v);
                }
            }
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
            | RExprKind::CallIntrinsic { .. }
            | RExprKind::BorrowMut { .. } => {}
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

pub struct ResolvedProgram {
    pub fns: Vec<ResolvedFn>,
    /// Nominal record types in declaration order (PLAN-771). `VT::Rec(i)`
    /// indexes here.
    pub records: Vec<ResolvedRecord>,
    /// PLAN-864: List element types (structural, deduplicated).
    /// `VT::List(i)` indexes here.
    pub lists: Vec<VT>,
    /// PLAN-864: nominal variant types (`?T`/`Result<T,E>`/declared
    /// `enum`s). `VT::Variant(i)` indexes here.
    pub variants: Vec<ResolvedVariant>,
    /// PLAN-865: closure function signatures referenced by `VT::Function`.
    pub function_types: Vec<ClosureFnType>,
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
}

impl CollTables {
    fn intern_function(&mut self, signature: ClosureFnType) -> usize {
        if let Some(i) = self.function_types.iter().position(|s| *s == signature) {
            return i;
        }
        self.function_types.push(signature);
        self.function_types.len() - 1
    }

    fn intern_list(&mut self, elem: VT) -> usize {
        if let Some(i) = self.lists.iter().position(|e| *e == elem) {
            return i;
        }
        self.lists.push(elem);
        self.lists.len() - 1
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
    result: VT,
    index: usize,
}

impl Clone for FnSig {
    fn clone(&self) -> Self {
        FnSig {
            param_names: self.param_names.clone(),
            params: self.params.clone(),
            result: self.result,
            index: self.index,
        }
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
pub fn resolve_with(
    program: &ast::Program,
    capabilities: &[String],
    surface: SourceProfile,
) -> Result<ResolvedProgram, Vec<Diagnostic>> {
    let (fns, records, coll, _) = resolve_detailed(program, capabilities, surface)?;
    Ok(ResolvedProgram {
        fns,
        records,
        lists: coll.lists,
        variants: coll.variants,
        function_types: coll.function_types,
    })
}

/// PLAN-828 D-01: the retained checked-environment indexes of one cold
/// resolve — what a later single-body re-check consults.
struct DeclIndexes {
    rec_index: BTreeMap<String, usize>,
    sigs: BTreeMap<String, FnSig>,
    trace_enabled: bool,
}

/// The declaration passes + per-function resolve shared by the
/// whole-module entry ([`resolve_with`]) and the retained-environment
/// cold entry ([`resolve_env`]).
fn resolve_detailed(
    program: &ast::Program,
    capabilities: &[String],
    surface: SourceProfile,
) -> Result<
    (
        Vec<ResolvedFn>,
        Vec<ResolvedRecord>,
        CollTables,
        DeclIndexes,
    ),
    Vec<Diagnostic>,
> {
    let trace_enabled = capabilities.iter().any(|c| c == TRACE_CAPABILITY);
    let mut diags = Vec::new();
    let mut coll = CollTables::default();
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

    // Pass 0b (PLAN-864): declared `enum`s pre-register as nominal
    // variants — arm payloads may reference records declared earlier.
    // Collections `type` blocks declare fields only (no methods).
    if is_coll {
        for item in &program.items {
            match item {
                ast::Item::Enum(e) => {
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
                    // Non-recursive: no arm payload may transitively reach
                    // this enum itself (payloads resolve BEFORE the enum
                    // name registers — a self-reference is an unknown type).
                    let _ = ok;
                    coll.intern_variant(&e.name.name, e.name.span, arms);
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
            ast::Item::Type(t) => {
                for m in &t.methods {
                    fn_receiver.push(m.receiver);
                    fn_owner.push(rec_index.get(&t.name.name).copied());
                    flat.push(m);
                }
            }
            // PLAN-864: enums carry no functions.
            ast::Item::Enum(_) => {}
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
        if sigs.contains_key(&sig_key) {
            // PLAN-862 (entry-contract §2.2): in the entry-unit profile the
            // ENTRY MATRIX owns the duplicate-`main` verdict — it reports
            // `source.entry-duplicate`, and that is the diagnostic a caller
            // must see. Emitting the generic duplicate-fn code here would
            // surface FIRST and hide the entry-specific one; every other
            // name (and every other profile) keeps the generic rule.
            let entry_main_duplicate = surface == SourceProfile::EntryUnit
                && fn_receiver[i] == ast::Receiver::Free
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

    // Pass 2: per-function body resolution.
    let mut fns = Vec::new();
    for (i, f) in flat.iter().enumerate() {
        crate::body_work::record_resolve(&f.name.name);
        fns.push(resolve_fn(
            f,
            &sigs,
            trace_enabled,
            surface,
            &mut diags,
            fn_receiver[i],
            fn_owner[i],
            &records,
            &rec_index,
            &mut coll,
        ));
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
    for (i, f) in fns.iter().enumerate() {
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
                f.body.stmts.push(RStmt::Return {
                    value: ResolvedExpr {
                        kind: RExprKind::Unit,
                        span: end_span,
                        vt: VT::Unit,
                    },
                    span: end_span,
                });
            } else if flat[i].script {
                if let Some(last) = f.body.stmts.last_mut() {
                    let taken = std::mem::replace(
                        last,
                        RStmt::Break {
                            target: None,
                            span: end_span,
                        },
                    );
                    *last = match taken {
                        RStmt::ExprStmt { value, span } => RStmt::Return { value, span },
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
            if let RStmt::ExprStmt { span, .. } = s {
                out.push(*span);
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

/// First expression statement in the body (any depth, root included).
fn first_expr_stmt(root: &RBlock) -> Option<Span> {
    walk_stmts(root).into_iter().find_map(|s| match s {
        RStmt::ExprStmt { span, .. } => Some(*span),
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

impl CheckedEnv {
    /// PLAN-828 D-03 (T01-C05): EXPLICIT release of the environment's
    /// owned body entries (the resolved `fns` and the record/method
    /// registries). The session's close calls this — dropping the session
    /// struct is not the mechanism. Retained for post-close diagnostics:
    /// the signature/record indexes, profile and walk counters. Idempotent.
    pub fn close(&mut self) {
        self.program.fns = Vec::new();
        self.program.records = Vec::new();
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

fn next_env_id() -> u64 {
    static SEQ: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
    SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}

/// PLAN-828 D-01: cold resolve that ALSO retains the checked environment
/// for later single-body re-checks. [`resolve_with`] stays
/// allocation-identical for whole-module consumers.
pub fn resolve_env(
    program: &ast::Program,
    capabilities: &[String],
    surface: SourceProfile,
) -> Result<CheckedEnv, Vec<Diagnostic>> {
    let (fns, records, coll, ix) = resolve_detailed(program, capabilities, surface)?;
    Ok(CheckedEnv {
        program: ResolvedProgram {
            fns,
            records,
            lists: coll.lists,
            variants: coll.variants,
            function_types: coll.function_types,
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
pub(crate) fn recheck_swap(
    env: &mut CheckedEnv,
    item: &ast::FnItem,
    fn_index: usize,
) -> Result<ResolvedFn, Vec<Diagnostic>> {
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
                    "source.incremental",
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
        )
    };
    let expected = &env.sigs[&sig_key];
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
    env.resolve_walks += 1;
    // PLAN-881: body-work meters under the OWNER-QUALIFIED key — a bare
    // method name is ambiguous against a same-named free function.
    crate::body_work::record_resolve(&sig_key);
    // PLAN-864: rebuild the collections tables from the retained program
    // (the same lists/variants the cold pass interned); new interning
    // inside the rechecked body is rejected by the drift guards below it.
    let mut coll = CollTables::default();
    coll.lists = env.program.lists.clone();
    coll.function_types = env.program.function_types.clone();
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
        env.profile,
        &mut diags,
        recv,
        owner_rec,
        &env.program.records,
        &env.rec_index,
        &mut coll,
    );
    if !diags.is_empty() {
        return Err(diags);
    }
    // Graft, then typecheck the ONE body against the grafted environment
    // (self-recursion sees the new entry); restore the old entry on
    // failure so a rejected body never pollutes the environment.
    let previous = std::mem::replace(&mut env.program.fns[fn_index], resolved);
    env.check_walks += 1;
    let check_diags = super::typecheck::check_body_in(&env.program, fn_index);
    if !check_diags.is_empty() {
        env.program.fns[fn_index] = previous;
        return Err(check_diags);
    }
    Ok(previous)
}

// D-03 (T01-C05): the candidate/transaction types live in `adapter.rs`
// next to `prepare_function_candidate` — the pipeline that produces them.

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
) -> FnSig {
    fn_sig_of_coll(
        f,
        recv,
        owner_rec,
        rec_index,
        &mut CollTables::default(),
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
    let result = if is_coll {
        vt_of_ann_coll(&f.ret, rec_index, coll, diags, TypePos::Result)
    } else {
        vt_of_ann(&f.ret, rec_index, diags)
    };
    FnSig {
        param_names,
        params,
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
        ast::TypeRef::Builtin(w) => vt_of_word(*w),
        ast::TypeRef::Record(id) => match rec_index.get(&id.name) {
            Some(&i) => VT::Rec(i),
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
            if matches!(inner_vt, VT::List(_)) {
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
            ("List", 1) => {
                if matches!(pos, TypePos::Field | TypePos::Param | TypePos::Result) {
                    return reject(
                            diags,
                            VT::I32,
                            "List-typed fields/parameters/results are not part of this source profile (v1 lists are function-local values)".to_string(),
                            ann.span,
                        );
                }
                let elem = vt_of_ann_coll(&args[0], rec_index, coll, diags, TypePos::Local);
                match elem {
                    VT::List(_) => reject(
                        diags,
                        VT::I32,
                        "List does not nest List (822 element domain)".to_string(),
                        args[0].span,
                    ),
                    VT::StrOwned | VT::I32 | VT::Bool | VT::Rec(_) | VT::Variant(_) => {
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
                if matches!(ok_vt, VT::List(_)) || matches!(err_vt, VT::List(_)) {
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

struct FnResolver<'a> {
    sigs: &'a BTreeMap<String, FnSig>,
    trace_enabled: bool,
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
    /// PLAN-864: active `for-in` iterations — (iterated local, element
    /// type). Method resolution rejects structural modification of the
    /// iterated list inside the body (`source.iterate-mutate`).
    iterate_targets: Vec<(LocalRef, VT, Span)>,
}

fn resolve_fn(
    f: &FnItem,
    sigs: &BTreeMap<String, FnSig>,
    trace_enabled: bool,
    surface: SourceProfile,
    diags: &mut Vec<Diagnostic>,
    receiver: ast::Receiver,
    owner_rec: Option<usize>,
    records: &[ResolvedRecord],
    rec_index: &BTreeMap<String, usize>,
    coll: &mut CollTables,
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
        records,
        rec_index,
        coll,
        iterate_targets: Vec::new(),
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

impl<'a> FnResolver<'a> {
    fn bind_err(&mut self, code: &str, msg: impl Into<String>, span: Span) {
        self.diags
            .push(Diagnostic::new(Stage::Bind, code, msg, span));
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

    fn vt_is_coll_owned(&self, vt: VT) -> bool {
        match vt {
            VT::StrOwned | VT::List(_) | VT::Variant(_) | VT::Function(_) => true,
            VT::Rec(ri) => record_reaches_str(ri, self.records),
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
                    self.bind_err(
                        "source.view-escape",
                        "a whole owned element is not a value; project a field (`xs.get(i).f`) or use `xs.remove(i)`".to_string(),
                        span,
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
                Stmt::Let {
                    name,
                    ann,
                    mutable,
                    value,
                    span,
                    ..
                } => {
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
                    let vt = match ann_vt {
                        Some(v) => v,
                        None => value.vt,
                    };
                    retag_literal(&mut value, vt);
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
                    self.locals.push(LocalDecl {
                        name: name.name.clone(),
                        vt,
                        mutable: *mutable,
                        mode: ast::ParamMode::Default,
                        is_param: false,
                        param_index: None,
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
                    let expect = self.lookup(&target.base.name).map(|l| self.locals[l.0].vt);
                    let mut value = self.expr_with_expect(value, expect);
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
                    retag_literal(&mut value, self.fn_result);
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
            ExprKind::Closure { .. } => {
                self.bind_err(
                    "source.unsupported",
                    "closure expression has not passed capture analysis in this source profile",
                    e.span,
                );
                ResolvedExpr {
                    span: e.span,
                    vt: VT::I32,
                    kind: RExprKind::IntLit(0),
                }
            }
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
            ExprKind::IntLit(v) => ResolvedExpr {
                span: e.span,
                vt: VT::I32,
                kind: RExprKind::IntLit(*v),
            },
            ExprKind::BoolLit(v) => ResolvedExpr {
                span: e.span,
                vt: VT::Bool,
                kind: RExprKind::BoolLit(*v),
            },
            ExprKind::Var(id) => match self.lookup(&id.name) {
                Some(l) => ResolvedExpr {
                    span: e.span,
                    vt: self.locals[l.0].vt,
                    kind: RExprKind::ReadLocal(l),
                },
                None => {
                    if self.sigs.contains_key(&id.name) {
                        self.bind_err(
                            "source.undefined",
                            format!(
                                "function `{}` cannot be used as a value in this profile",
                                id.name
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
            ExprKind::StrLit(v) => ResolvedExpr {
                span: e.span,
                // default context: a literal is a borrowing view (D-03);
                // owning contexts retag it via `retag_literal`
                vt: VT::Str,
                kind: RExprKind::StrLit(v.clone()),
            },
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
                let sig_params = self.method_sig_params(m.fn_index, 0);
                self.retag_method_args(&mut rargs, m.fn_index, 0);
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
                    if let ExprKind::Call { callee, args } = &node.kind {
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
                        // not a value.
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
                                                    stack.push(Task::Visit {
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
                    } else if let ExprKind::CtorNamed { ty, fields } = &node.kind {
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
                                1 => Some(Box::new(rargs.remove(0).value)),
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
                    let l = std::mem::replace(&mut out[lslot], dummy_resolved());
                    let r = std::mem::replace(&mut out[rslot], dummy_resolved());
                    let vt = bin_op_vt(op);
                    // PLAN-770: `+` over two text operands is concatenation
                    // (owned result); the arithmetic i32 rule is unchanged.
                    let vt = if op == BinOp::Add && l.vt.is_text() && r.vt.is_text() {
                        VT::StrOwned
                    } else {
                        vt
                    };
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
                    let vt = match op {
                        UnaryOp::Neg => VT::I32,
                        UnaryOp::Not => VT::Bool,
                    };
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
            BuiltinMethod::To => VT::Str,
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

    fn call(&mut self, callee: &ast::Ident, args: &[Arg], call_span: Span) -> ResolvedExpr {
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
        let args = &rargs[..];
        let _ = &args;

        // Locals shadow functions and intrinsics in callee position.
        if self.lookup(&callee.name).is_some() {
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
                vt: sig.result,
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
                    vt: VT::I32,
                    kind: RExprKind::CallIntrinsic { symbol },
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

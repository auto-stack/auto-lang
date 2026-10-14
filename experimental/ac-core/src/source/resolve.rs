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
}

pub fn vt_of_word(w: TypeWord) -> VT {
    match w {
        TypeWord::Int | TypeWord::I32 => VT::I32,
        TypeWord::Bool => VT::Bool,
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
    /// PLAN-771: explicit `mut` param mode (exclusive borrow); `false` for
    /// every non-record param and every binding.
    pub param_mut: bool,
    /// Name through the initializer (let/var) or the whole param phrase.
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct ResolvedArg {
    /// Source argument order (evaluation order, left to right).
    pub value: ResolvedExpr,
    /// Present for named arguments.
    pub name: Option<ast::Ident>,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub enum RExprKind {
    IntLit(i32),
    BoolLit(bool),
    ReadLocal(LocalRef),
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
            RExprKind::FieldGet { base, .. } => {
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
            RExprKind::IntLit(_)
            | RExprKind::BoolLit(_)
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

pub struct ResolvedProgram {
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

struct FnSig {
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
    _surface: SourceProfile,
) -> Result<ResolvedProgram, Vec<Diagnostic>> {
    let trace_enabled = capabilities.iter().any(|c| c == TRACE_CAPABILITY);
    let mut diags = Vec::new();

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
                let vt = vt_of_ann(&f.ty, &rec_index, &mut diags);
                records[ri].fields.push(ResolvedField {
                    name: f.name.name.clone(),
                    vt,
                    span: f.name.span,
                });
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
            diags.push(Diagnostic::new(
                Stage::Bind,
                "source.duplicate",
                format!("function `{}` is declared more than once", f.name.name),
                f.span,
            ));
            continue;
        }
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
        let recv = fn_receiver[i];
        let mut param_names: Vec<String> = Vec::new();
        let mut params: Vec<VT> = Vec::new();
        if recv == ast::Receiver::View {
            param_names.push("self".to_string());
            params.push(VT::Rec(fn_owner[i].unwrap_or(usize::MAX)));
        }
        for p in &f.params {
            param_names.push(p.name.name.clone());
            params.push(vt_of_ann(&p.ty, &rec_index, &mut diags));
        }
        let result = vt_of_ann(&f.ret, &rec_index, &mut diags);
        sigs.insert(
            sig_key.clone(),
            FnSig {
                param_names,
                params,
                result,
                index: i,
            },
        );
        // method-side registry on the owning record
        if let Some(oi) = fn_owner[i] {
            records[oi].methods.push(ResolvedMethod {
                name: f.name.name.clone(),
                is_static: recv == ast::Receiver::Static,
                fn_index: i,
                span: f.span,
            });
        }
    }

    // Pass 2: per-function body resolution.
    let mut fns = Vec::new();
    for (i, f) in flat.iter().enumerate() {
        fns.push(resolve_fn(
            f,
            &sigs,
            trace_enabled,
            &mut diags,
            fn_receiver[i],
            fn_owner[i],
            &records,
            &rec_index,
        ));
    }

    if diags.is_empty() {
        Ok(ResolvedProgram { fns, records })
    } else {
        Err(diags)
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
    }
}

struct Scope {
    names: BTreeMap<String, LocalRef>,
}

struct FnResolver<'a> {
    sigs: &'a BTreeMap<String, FnSig>,
    trace_enabled: bool,
    diags: &'a mut Vec<Diagnostic>,
    locals: Vec<LocalDecl>,
    scopes: Vec<Scope>,
    /// PLAN-771: the module's record table (read-only here).
    records: &'a [ResolvedRecord],
    rec_index: &'a BTreeMap<String, usize>,
    /// Every local name declared anywhere in this function (any scope), to
    /// distinguish "never declared" from "declared in a scope that ended".
    declared_names: BTreeSet<String>,
    loop_stack: Vec<LoopRef>,
    loop_count: usize,
    used_intrinsics: BTreeSet<&'static str>,
}

fn resolve_fn(
    f: &FnItem,
    sigs: &BTreeMap<String, FnSig>,
    trace_enabled: bool,
    diags: &mut Vec<Diagnostic>,
    receiver: ast::Receiver,
    owner_rec: Option<usize>,
    records: &[ResolvedRecord],
    rec_index: &BTreeMap<String, usize>,
) -> ResolvedFn {
    let mut r = FnResolver {
        sigs,
        trace_enabled,
        diags,
        locals: Vec::new(),
        scopes: vec![Scope {
            names: BTreeMap::new(),
        }],
        declared_names: BTreeSet::new(),
        loop_stack: Vec::new(),
        loop_count: 0,
        used_intrinsics: BTreeSet::new(),
        records,
        rec_index,
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
            param_mut: false,
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
        let vt = vt_of_ann(&p.ty, rec_index, &mut Vec::new());
        // mut mode is only meaningful for record params (774 D-04)
        if p.mode == ast::ParamMode::Mut && !matches!(vt, VT::Rec(_)) {
            r.bind_err(
                "source.type-mismatch",
                format!(
                    "`mut` parameter `{}` requires a record type; i32/bool params are passed by value",
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
            param_index: Some(
                (i + if receiver == ast::Receiver::View {
                    1
                } else {
                    0
                }) as u32,
            ),
            param_mut: p.mode == ast::ParamMode::Mut,
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
                    // RHS resolves in the pre-declaration environment.
                    let value = self.expr_iterative(value);
                    let vt = match ann {
                        Some(a) => vt_of_ann(a, self.rec_index, &mut Vec::new()),
                        None => value.vt,
                    };
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
                        is_param: false,
                        param_index: None,
                        param_mut: false,
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
                    let value = self.expr_iterative(value);
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
                    let value = self.expr_iterative(value);
                    stack
                        .last_mut()
                        .expect("resolve frame")
                        .stmts
                        .push(RStmt::Return { value, span: *span });
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
            }
        }
    }

    fn expr(&mut self, e: &Expr) -> ResolvedExpr {
        match &e.kind {
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
        rargs: Vec<ResolvedArg>,
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
        for (i, a) in rargs.iter().enumerate() {
            let Some(want) = fields.get(i).map(|f| f.vt) else {
                continue;
            };
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
        for (name, fspan, v) in values {
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
        name: &ast::Ident,
        rargs: Vec<ResolvedArg>,
        span: Span,
    ) -> ResolvedExpr {
        if let Some(rrecv) = recv {
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
            return self.instance_or_static(Some(rrecv), ri, name, rargs, span);
        }
        // static: no receiver expression was evaluated — the type name was
        // consumed at classification time; recover it from the fn index of
        // any record declaring `name` as static is ambiguous, so static
        // calls carry the type via the classification path instead: this
        // arm only fires for a `None` slot that was classified static.
        // The classification stored nothing, so treat as unreachable-safe:
        // look the method up across all records (single definition per
        // name+static in well-formed modules; duplicates were reported).
        for rec in self.records.iter() {
            if let Some(m) = rec
                .methods
                .iter()
                .find(|m| m.name == name.name && m.is_static)
            {
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
        rargs: Vec<ResolvedArg>,
        span: Span,
    ) -> ResolvedExpr {
        match self.records[ri]
            .methods
            .iter()
            .find(|m| m.name == name.name)
        {
            Some(m) if !m.is_static => {
                let sig_params = self.method_sig_params(m.fn_index, 1);
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
        let rargs = self.resolve_args(args);
        match self.records[ri]
            .methods
            .iter()
            .find(|m| m.name == name.name)
        {
            Some(m) if !m.is_static => {
                // receiver occupies param 0; visible args bind from param 1
                let sig_params = self.method_sig_params(m.fn_index, 1);
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
                        value: v,
                    });
                }
                ast::Arg::Named { name, value, span } => {
                    let v = self.expr_iterative(value);
                    out.push(ResolvedArg {
                        span: *span,
                        name: Some(name.clone()),
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
            /// Receiver + arg slots are filled (receiver slot `None` for
            /// pre-classified static calls); resolve the method into `dest`.
            MethodOut {
                name: ast::Ident,
                dest: usize,
                recv_slot: Option<usize>,
                arg_slots: Vec<usize>,
                names: Vec<Option<ast::Ident>>,
                spans: Vec<Span>,
                node_span: Span,
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
                                Arg::Pos(e) => {
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
                    if let ExprKind::Field { base, name } = &node.kind {
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
                        let static_recv = matches!(&recv.kind, ExprKind::Var(id)
                            if self.lookup(&id.name).is_none()
                                && self.rec_index.contains_key(&id.name));
                        let recv_slot = if static_recv { None } else { Some(out.len()) };
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
                                Arg::Pos(e) => {
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
                        subtasks.push(Task::MethodOut {
                            name: name.clone(),
                            dest: slot,
                            recv_slot,
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
                            let vt = self.records[ri].fields[fi].vt;
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
                Task::MethodOut {
                    name,
                    dest,
                    recv_slot,
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
                            value,
                        });
                    }
                    let rrecv = match recv_slot {
                        None => None,
                        Some(rs) => {
                            Some(Box::new(std::mem::replace(&mut out[rs], dummy_resolved())))
                        }
                    };
                    out[dest] = self.method_call_out(rrecv, &name, rargs, node_span);
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
                        let value = std::mem::replace(&mut out[*s], dummy_resolved());
                        rargs.push(ResolvedArg {
                            span: spans[i],
                            name: names[i].clone(),
                            value,
                        });
                    }
                    // the node's own span already covers callee..
                    out[dest] = self.resolve_call(callee, rargs, node_span);
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

    fn call(&mut self, callee: &ast::Ident, args: &[Arg], call_span: Span) -> ResolvedExpr {
        let mut rargs: Vec<ResolvedArg> = Vec::new();
        for a in args {
            match a {
                Arg::Pos(e) => rargs.push(ResolvedArg {
                    value: self.expr_iterative(e),
                    name: None,
                    span: e.span,
                }),
                Arg::Named { name, value, span } => rargs.push(ResolvedArg {
                    value: self.expr_iterative(value),
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
        rargs: Vec<ResolvedArg>,
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

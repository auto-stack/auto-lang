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
}

impl VT {
    pub fn as_str(&self) -> &'static str {
        match self {
            VT::I32 => "i32",
            VT::Bool => "bool",
            VT::Str => "str",
            VT::StrOwned => "Str",
            VT::Char => "char",
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
            _ => {}
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
    pub result: VT,
    pub locals: Vec<LocalDecl>,
    pub body: RBlock,
    /// Number of registered loops (While + Loop statements) in walk order.
    pub loop_count: usize,
    /// Trace intrinsic symbols referenced by this function's body.
    pub used_intrinsics: BTreeSet<&'static str>,
}

pub struct ResolvedProgram {
    pub fns: Vec<ResolvedFn>,
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

    // Pass 1: signature pre-registration with duplicate detection.
    let mut sigs: BTreeMap<String, FnSig> = BTreeMap::new();
    for (i, f) in program.items.iter().enumerate() {
        if sigs.contains_key(&f.name.name) {
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
        sigs.insert(
            f.name.name.clone(),
            FnSig {
                param_names: f.params.iter().map(|p| p.name.name.clone()).collect(),
                params: f.params.iter().map(|p| vt_of_word(p.ty.word)).collect(),
                result: vt_of_word(f.ret.word),
                index: i,
            },
        );
    }

    // Pass 2: per-function body resolution.
    let mut fns = Vec::new();
    for f in &program.items {
        fns.push(resolve_fn(f, &sigs, trace_enabled, &mut diags));
    }

    if diags.is_empty() {
        Ok(ResolvedProgram { fns })
    } else {
        Err(diags)
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
    /// Every local name declared anywhere in this function (any scope), to
    /// distinguish "never declared" from "declared in a scope that ended".
    declared_names: BTreeSet<String>,
    /// Result type of the function being resolved (retag context).
    fn_result: VT,
    loop_stack: Vec<LoopRef>,
    loop_count: usize,
    used_intrinsics: BTreeSet<&'static str>,
}

fn resolve_fn(
    f: &FnItem,
    sigs: &BTreeMap<String, FnSig>,
    trace_enabled: bool,
    diags: &mut Vec<Diagnostic>,
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
        fn_result: vt_of_word(f.ret.word),
        loop_stack: Vec::new(),
        loop_count: 0,
        used_intrinsics: BTreeSet::new(),
    };
    // Params live in the function's outermost scope.
    for (i, p) in f.params.iter().enumerate() {
        let lref = LocalRef(r.locals.len());
        r.locals.push(LocalDecl {
            name: p.name.name.clone(),
            vt: vt_of_word(p.ty.word),
            mutable: false,
            is_param: true,
            mode: p.mode,
            param_index: Some(i as u32),
            span: p.span,
        });
        r.scopes[0].names.insert(p.name.name.clone(), lref);
        r.declared_names.insert(p.name.name.clone());
    }
    let body = r.resolve_body(&f.body);
    let sig = &sigs[&f.name.name];
    ResolvedFn {
        name: f.name.name.clone(),
        name_span: f.name.span,
        item_span: f.span,
        param_names: sig.param_names.clone(),
        params: f.params.iter().map(|p| vt_of_word(p.ty.word)).collect(),
        result: vt_of_word(f.ret.word),
        locals: r.locals,
        body,
        loop_count: r.loop_count,
        used_intrinsics: r.used_intrinsics,
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
                    let mut value = self.expr_iterative(value);
                    let vt = match ann {
                        Some(a) => vt_of_word(a.word),
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
                Stmt::Assign { name, value, span } => {
                    let value = self.expr_iterative(value);
                    // Immutability itself is re-checked by the semantic verifier
                    // on the emitted place/local pair (verify.immutable-write);
                    // here only resolution failures are reported.
                    let rstmt = match self.lookup(&name.name) {
                        Some(l) => RStmt::Assign {
                            local: l,
                            value,
                            span: *span,
                        },
                        None => {
                            if self.sigs.contains_key(&name.name) {
                                self.bind_err(
                                    "source.undefined",
                                    format!(
                                        "cannot assign to `{}`: functions are not assignable locals",
                                        name.name
                                    ),
                                    name.span,
                                );
                            } else {
                                self.bind_err(
                                    "source.undefined",
                                    format!("no local named `{}` in scope", name.name),
                                    name.span,
                                );
                            }
                            RStmt::Assign {
                                local: LocalRef(usize::MAX),
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
                        .push(RStmt::Return {
                            value: {
                                let mut v = value;
                                retag_literal(&mut v, self.fn_result);
                                v
                            },
                            span: *span,
                        });
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
            /// Receiver and argument slots are filled; bind the builtin
            /// method into `dest` (PLAN-770).
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
                    if let ExprKind::Binary { op, lhs, rhs } = &node.kind {
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
                    } else {
                        out[slot] = self.expr(node);
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
                        let moved = matches!(value.kind, RExprKind::Move { .. });
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
                        let moved = matches!(value.kind, RExprKind::Move { .. });
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
            if !matches!(recv.kind, RExprKind::ReadLocal(_)) {
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

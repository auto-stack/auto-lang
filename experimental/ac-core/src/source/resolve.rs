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

use super::ast::{self, Arg, BinOp, Expr, ExprKind, FnItem, Stmt, TypeWord};
use super::{trace_intrinsic, TRACE_CAPABILITY};
use crate::atom_text::{Diagnostic, Span, Stage};
use std::collections::{BTreeMap, BTreeSet};

/// Value type of the profile: `int`/`i32` are the same exact signed 32-bit
/// type; `bool` is distinct with no implicit conversions.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
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
}

pub fn vt_of_word(w: TypeWord) -> VT {
    match w {
        TypeWord::Int | TypeWord::I32 => VT::I32,
        TypeWord::Bool => VT::Bool,
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
        let detach = |node: &mut ResolvedExpr, stack: &mut Vec<ResolvedExpr>| {
            if let RExprKind::Binary { lhs, rhs, .. } = &mut node.kind {
                let l = std::mem::replace(lhs.as_mut(), dummy());
                let r = std::mem::replace(rhs.as_mut(), dummy());
                stack.push(l);
                stack.push(r);
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
            param_index: Some(i as u32),
            span: p.span,
        });
        r.scopes[0].names.insert(p.name.name.clone(), lref);
        r.declared_names.insert(p.name.name.clone());
    }
    let body = r.block(&f.body);
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

    fn block(&mut self, b: &ast::Block) -> RBlock {
        self.scopes.push(Scope {
            names: BTreeMap::new(),
        });
        let mut stmts = Vec::new();
        for s in &b.stmts {
            stmts.push(self.stmt(s));
        }
        self.scopes.pop();
        RBlock {
            stmts,
            span: b.span,
        }
    }

    fn stmt(&mut self, s: &Stmt) -> RStmt {
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
                    Some(a) => vt_of_word(a.word),
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
                let lref = LocalRef(self.locals.len());
                self.locals.push(LocalDecl {
                    name: name.name.clone(),
                    vt,
                    mutable: *mutable,
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
                RStmt::Let {
                    local: lref,
                    value,
                    span: *span,
                }
            }
            Stmt::Assign { name, value, span } => {
                let value = self.expr_iterative(value);
                // Immutability itself is re-checked by the semantic verifier
                // on the emitted place/local pair (verify.immutable-write);
                // here only resolution failures are reported.
                match self.lookup(&name.name) {
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
                }
            }
            Stmt::Return { value, span } => {
                let value = self.expr_iterative(value);
                RStmt::Return { value, span: *span }
            }
            Stmt::If {
                cond,
                then,
                els,
                span,
            } => {
                let cond = self.expr_iterative(cond);
                let then = self.block(then);
                let els = els.as_ref().map(|e| self.block(e));
                RStmt::If {
                    cond,
                    then,
                    els,
                    span: *span,
                }
            }
            Stmt::While {
                cond, body, span, ..
            } => {
                let cond = self.expr_iterative(cond);
                let target = LoopRef(self.loop_count);
                self.loop_count += 1;
                self.loop_stack.push(target);
                let body = self.block(body);
                self.loop_stack.pop();
                RStmt::While {
                    cond,
                    body,
                    span: *span,
                    target,
                }
            }
            Stmt::Loop { body, span } => {
                let target = LoopRef(self.loop_count);
                self.loop_count += 1;
                self.loop_stack.push(target);
                let body = self.block(body);
                self.loop_stack.pop();
                RStmt::Loop {
                    body,
                    span: *span,
                    target,
                }
            }
            Stmt::Break { span } => RStmt::Break {
                target: self.loop_stack.last().copied(),
                span: *span,
            },
            Stmt::Continue { span } => RStmt::Continue {
                target: self.loop_stack.last().copied(),
                span: *span,
            },
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
            /// Both operand slots are filled; combine into `dest`.
            Combine {
                op: BinOp,
                span: Span,
                lslot: usize,
                rslot: usize,
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
                    } else {
                        out[slot] = self.expr(node);
                    }
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
                    let vt = match op {
                        BinOp::Add | BinOp::Mul => VT::I32,
                        BinOp::Lt => VT::Bool,
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

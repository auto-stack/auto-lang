//! Limited bidirectional inference profile (PLAN-796 §5.2).
//!
//! Real solver — never a call into the old resolver/typecheck with the
//! result relabeled. The surface: i32/bool literals, let/var with
//! shadowing, explicit fn params/returns, direct calls, the operator
//! set, condition/loop constraints, assignment consistency.
//!
//! Architecture (Design 42 §5): bidirectional check/synth over an
//! explicit task stack; unannotated bindings mint type variables solved
//! by union-find substitution (occurs-checked). Variables, substitutions
//! and the worklist are TASK WORK STATE and never leave the session.
//! Unsolved variables, occurs failures, ambiguous candidates and unknown
//! names are definitive diagnostics — never an Any/Unknown fallback.
//! Resource limits and cancellation are infrastructure states reported
//! separately (and never cached).
//!
//! Deep-surface safety: expression spines and nested blocks are walked
//! with explicit stacks — no native recursion — mirroring the input
//! model's iterative Drop.

use crate::budget::{Budget, BudgetError};
use crate::input::{
    Ann, BinOpKind, InputBlock, InputExpr, InputExprKind, InputFn, InputStmt, SemaInput, Span,
};
use ace_semantics_contract::{DeclKind, FactDiag, InferFacts, LogicalId, PackageId, TypeDesc};
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

/// Solver-level type of the limited profile: scalars plus unsolved
/// variables. (Function VALUES are outside the infer surface; call
/// results resolve against pre-registered signatures.)
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Ty {
    I32,
    Bool,
    /// PLAN-963（T-02）：文本值（记录域 `str`/`Str` 面）。
    Text,
    /// PLAN-963（T-02）：声明记录（按名义名；字段/方法按记录表解析）。
    Record(&'static str),
    Var(u32),
}

/// What the session read while inferring — the positive/negative
/// dependency inputs of the query layer.
#[derive(Clone, Debug, Default)]
pub struct Reads {
    /// Callee signatures actually read (fn names, insertion order).
    pub sigs: Vec<String>,
    /// Unknown/ambiguous name lookups: (domain, name, candidate-set fp)
    /// — the negative dependencies.
    pub negatives: Vec<(&'static str, String, String)>,
}

/// The frozen summary of one inference run (FACTS, no work state).
#[derive(Clone, Debug)]
pub struct InferSummary {
    pub facts: InferFacts,
    pub reads: Reads,
    pub steps: u64,
    pub elapsed: Duration,
}

#[derive(Debug)]
pub enum InferOutcome {
    /// Checked clean.
    Ok(InferSummary),
    /// Definitive type diagnostics (cacheable under the full key).
    Diags(InferSummary),
    /// Infrastructure resource state — NEVER cached as a program verdict.
    ResourceExhausted(BudgetError),
    /// Cancellation — never cached.
    Cancelled,
}

impl InferOutcome {
    pub fn summary(&self) -> Option<&InferSummary> {
        match self {
            InferOutcome::Ok(s) | InferOutcome::Diags(s) => Some(s),
            _ => None,
        }
    }
}

/// Statement-block frames: explicit block stack + per-block scope map.
/// An `if` pushes its else-arm frame BEFORE its then-arm frame (LIFO →
/// then is processed first, else follows when then is exhausted); arms
/// get independent scopes.
struct Frames<'b> {
    blocks: Vec<&'b InputBlock>,
    idx: Vec<usize>,
    scopes: Vec<BTreeMap<String, (Ty, bool)>>,
}

impl<'b> Frames<'b> {
    fn empty() -> Self {
        Frames {
            blocks: Vec::new(),
            idx: Vec::new(),
            scopes: vec![BTreeMap::new()],
        }
    }
    fn push_block(&mut self, b: &'b InputBlock) {
        self.blocks.push(b);
        self.idx.push(0);
        self.scopes.push(BTreeMap::new());
    }
    fn lookup(&self, name: &str) -> Option<(Ty, bool)> {
        self.scopes.iter().rev().find_map(|s| s.get(name).copied())
    }
}

/// PLAN-935 (T-04): one template call site's minted type-parameter vars,
/// resolved into a concrete substitution AFTER the fn's solve.
#[derive(Clone, Debug)]
pub struct TplDemand {
    pub callee: String,
    pub vars: Vec<Ty>,
    /// The call-site span (source bytes) — the desugar layer's rebinding
    /// key. Spans identify call sites uniquely within one source.
    pub span: (u32, u32),
}

/// PLAN-963（T-02）：字段读取的延迟约束。接收者的类型要等本体的类型表
/// 完整后才可知（例如 `pick(a, b).val`——模板调用的结果类型只在求解后
/// 确定），故在 `finish` 的阶段按已记录类型解析。
struct FieldRead {
    base: Span,
    field: String,
    site: Span,
    out_ty: Ty,
}

/// PLAN-963（T-02）：方法调用的延迟约束（同上——接收者类型延迟可知）。
struct MethodCallCheck {
    /// 接收者类型的直接来源：`Left` = 已走过表达式的类型（按 span 查表），
    /// `Right` = 静态调用（`Score.new(...)`，接收者是记录名本身）。
    recv: MethodRecv,
    name: String,
    /// 参数的（源序下标、名字、span）——类型按 span 查表。
    args: Vec<(usize, Option<String>, Span)>,
    site: Span,
    out_ty: Ty,
}

enum MethodRecv {
    Value(Span),
    /// 静态调用：接收者是记录名本身（`Score.new(...)`）。
    Record(String),
}

pub struct InferSession<'a> {
    input: &'a SemaInput,
    budget: Budget,
    cancel: Option<Arc<AtomicBool>>,
    budget_err: Option<BudgetError>,
    // -- work state (never persisted) --
    vars: Vec<Option<Ty>>,
    diags: Vec<FactDiag>,
    reads: Reads,
    eq_pairs: Vec<(Span, Span, Span)>,
    // PLAN-963（T-02）：记录字段读取 / 方法调用的延迟约束。
    field_reads: Vec<FieldRead>,
    method_calls: Vec<MethodCallCheck>,
    // -- facts being recorded --
    bindings: Vec<(String, Ty)>,
    expr_types: Vec<(u32, u32, Ty)>,
    // PLAN-935 (T-04): template call demands of this run.
    tpl_demands: Vec<TplDemand>,
}

impl<'a> InferSession<'a> {
    pub fn new(input: &'a SemaInput) -> Self {
        InferSession {
            input,
            budget: Budget::default(),
            cancel: None,
            budget_err: None,
            vars: Vec::new(),
            diags: Vec::new(),
            reads: Reads::default(),
            eq_pairs: Vec::new(),
            field_reads: Vec::new(),
            method_calls: Vec::new(),
            bindings: Vec::new(),
            expr_types: Vec::new(),
            tpl_demands: Vec::new(),
        }
    }

    pub fn with_budget(mut self, max_steps: u64, wall: Duration) -> Self {
        self.budget = Budget::new(max_steps, wall);
        self
    }

    pub fn with_cancel(mut self, flag: Arc<AtomicBool>) -> Self {
        self.cancel = Some(flag);
        self
    }

    /// PLAN-935 (T-04): resolve and take this run's template-call demands.
    /// Call AFTER [`Self::infer_function`] — the vars must be read through
    /// the final union-find. An unresolved var yields `None` for its slot
    /// (the run's diagnostics already carry the definitive unsolved
    /// verdict; the demand is not admitted).
    pub fn take_instance_demands(&mut self) -> Vec<(String, Vec<Option<Ty>>, (u32, u32))> {
        std::mem::take(&mut self.tpl_demands)
            .into_iter()
            .map(|d| {
                let tys = d
                    .vars
                    .into_iter()
                    .map(|v| match self.resolve(v) {
                        // PLAN-963（T-02）：记录与文本实参同样是可实例化的
                        // 具体类型（聚合/文本面的泛型调用）。
                        t @ (Ty::I32 | Ty::Bool | Ty::Text | Ty::Record(_)) => Some(t),
                        Ty::Var(_) => None,
                    })
                    .collect();
                (d.callee, tys, d.span)
            })
            .collect()
    }

    /// Infer one function body by name. Unknown function → definitive
    /// diagnostic outcome (with the negative dependency recorded).
    pub fn infer_function(&mut self, name: &str) -> InferOutcome {
        if self.check_cancel() {
            return InferOutcome::Cancelled;
        }
        let input: &'a SemaInput = self.input;
        let Some(f) = input.fns.iter().find(|f| f.name == name) else {
            self.record_negative("fn", name);
            return self.finish_outcome(name);
        };
        if let Err(e) = self.run_body(f) {
            return InferOutcome::ResourceExhausted(e);
        }
        if self.check_cancel() {
            return InferOutcome::Cancelled;
        }
        let summary = self.finish(name);
        if self.diags.is_empty() {
            InferOutcome::Ok(summary)
        } else {
            InferOutcome::Diags(summary)
        }
    }

    fn finish_outcome(&mut self, name: &str) -> InferOutcome {
        let summary = self.finish(name);
        if self.diags.is_empty() {
            InferOutcome::Ok(summary)
        } else {
            InferOutcome::Diags(summary)
        }
    }

    fn check_cancel(&self) -> bool {
        self.cancel
            .as_ref()
            .is_some_and(|c| c.load(Ordering::Relaxed))
    }

    fn charge(&mut self, n: u64) {
        if self.budget_err.is_none() {
            if let Err(e) = self.budget.charge(n) {
                self.budget_err = Some(e);
            }
        }
    }

    fn mint_var(&mut self) -> Ty {
        self.vars.push(None);
        Ty::Var(self.vars.len() as u32 - 1)
    }

    /// Annotation → solver type. Out-of-profile annotations inside a BODY
    /// surface as unsupported-construct (never silently widened); PLAN-963
    /// （T-02）把记录与文本纳入求解域——泛型函数的实参可为聚合与文本。
    fn ann_ty(&mut self, ann: Ann, span: Span) -> Ty {
        match ann {
            Ann::I32 => Ty::I32,
            Ann::Bool => Ty::Bool,
            Ann::Text => Ty::Text,
            Ann::Record(name) => Ty::Record(name),
            Ann::Unsupported(_) => {
                self.diag(
                    "unsupported-construct",
                    span,
                    "out-of-profile annotations are outside the inference profile".into(),
                );
                self.mint_var()
            }
            Ann::Inferred => self.mint_var(),
            // PLAN-935 (T-04): defensive — the template call hook
            // substitutes TyParam positions BEFORE ann_ty; reaching here
            // means a template signature leaked onto a non-substituted
            // path. Mint a var (never widen to a concrete) — the finish
            // pass then flags it unsolved, which is the honest verdict.
            Ann::TyParam(_) => self.mint_var(),
        }
    }

    fn ty_desc(&self, t: Ty) -> TypeDesc {
        match self.resolve(t) {
            Ty::I32 => TypeDesc::I32,
            Ty::Bool => TypeDesc::Bool,
            // PLAN-963（T-02）：文本按字符面 opaqueness 记录（934 事实页
            // 不承载文本值类型，见 PortableTy 的可移植面）。
            Ty::Text => TypeDesc::Opaque {
                name: "text".into(),
            },
            Ty::Record(name) => TypeDesc::Record {
                decl: ace_semantics_contract::DeclRef::new(
                    &ace_semantics_contract::PackageId::new(&self.input.package),
                    &self.input.module,
                    ace_semantics_contract::DeclKind::Record,
                    name,
                    ace_semantics_contract::ContentVersion::of_canonical(name.as_bytes()),
                ),
            },
            Ty::Var(_) => TypeDesc::Opaque {
                name: "?unsolved".into(),
            },
        }
    }

    fn diag(&mut self, code: &'static str, span: Span, message: String) {
        self.diags.push(FactDiag {
            code,
            start: span.start,
            end: span.end,
            message,
        });
    }

    fn record_negative(&mut self, domain: &'static str, name: &str) {
        self.record_negative_at(domain, name, Span::new(0, 0));
    }

    fn record_negative_at(&mut self, domain: &'static str, name: &str, span: Span) {
        // The established local-only diagnostic/cache wire stays byte-stable;
        // source foreign inputs have a distinct signature-key generation.
        let span = if self.input.external_signatures.is_empty() {
            Span::new(0, 0)
        } else {
            span
        };
        let set_fp = self.candidate_fp(domain);
        self.reads
            .negatives
            .push((domain, name.to_string(), set_fp));
        self.diags.push(FactDiag {
            code: "unknown-name",
            start: span.start,
            end: span.end,
            message: format!("unknown {} name `{}`", domain, name),
        });
    }

    fn candidate_fp(&self, domain: &str) -> String {
        let set = match domain {
            "fn" => self.input.fn_candidates(),
            "type" => self.input.record_candidates(),
            // PLAN-963（T-02）：记录字段/方法名域（负依赖指纹）。
            "field" => self.input.field_candidates(),
            "method" => self.input.method_candidates(),
            _ => Vec::new(),
        };
        let joined = set.join("\u{1}");
        ace_semantics_contract::identity::domain_hash(
            "ac796/candidate-set/v1",
            &[joined.as_bytes()],
        )
    }

    fn read_sig(&mut self, name: &str) {
        if !self.reads.sigs.iter().any(|s| s == name) {
            self.reads.sigs.push(name.to_string());
        }
    }

    // -- union-find over vars (iterative resolve) ----------------------

    /// Follow a substitution chain to its representative. Cycle-safe:
    /// a corrupted (cyclic) chain stops at the first revisit instead of
    /// hanging — the guard in [`Self::unify`] is what reports it.
    fn resolve(&self, mut t: Ty) -> Ty {
        let mut visited: Vec<u32> = Vec::new();
        while let Ty::Var(v) = t {
            if visited.contains(&v) {
                break;
            }
            visited.push(v);
            match self.vars.get(v as usize).and_then(|s| *s) {
                Some(next) => t = next,
                None => break,
            }
        }
        t
    }

    /// Resolve that FAILS on a cyclic chain (the occurs-check trigger):
    /// returns None when the substitution graph contains a cycle through
    /// this type.
    fn resolve_guarded(&self, t: Ty) -> Option<Ty> {
        let mut cur = t;
        let mut visited: Vec<u32> = Vec::new();
        while let Ty::Var(v) = cur {
            if visited.contains(&v) {
                return None;
            }
            visited.push(v);
            match self.vars.get(v as usize).and_then(|s| *s) {
                Some(next) => cur = next,
                None => break,
            }
        }
        Some(cur)
    }

    /// Unify with occurs check. Returns false when a definitive mismatch
    /// or occurs failure was recorded.
    fn unify(&mut self, a: Ty, b: Ty, span: Span, origin: &'static str) -> bool {
        // occurs guard: a cyclic substitution chain is a controlled
        // failure here — never a hang and never a wrong binding
        let (Some(a), Some(b)) = (self.resolve_guarded(a), self.resolve_guarded(b)) else {
            self.diag(
                "occurs-check",
                span,
                format!("cyclic type substitution at {origin}"),
            );
            return false;
        };
        if a == b {
            return true;
        }
        match (a, b) {
            (Ty::Var(v), other) | (other, Ty::Var(v)) => {
                // `other` is fully resolved and acyclic (guard above), so
                // binding the representative keeps the substitution a
                // forest — cycles cannot be created through unify
                self.vars[v as usize] = Some(other);
                true
            }
            _ => {
                self.diag(
                    "type-mismatch",
                    span,
                    format!("expected {}, found {}", self.name_of(a), self.name_of(b)),
                );
                false
            }
        }
    }

    // -- statement walk -------------------------------------------------

    fn run_body(&mut self, f: &InputFn) -> Result<(), BudgetError> {
        let ret = self.ann_ty(f.ret, Span::new(0, 0));
        let mut frames = Frames::empty();
        for (name, ann) in &f.params {
            let t = self.ann_ty(*ann, Span::new(0, 0));
            frames
                .scopes
                .last_mut()
                .unwrap()
                .insert(name.clone(), (t, true));
            self.bindings.push((name.clone(), t));
        }
        frames.push_block(&f.body);
        self.run_frames(&mut frames, ret);
        match self.budget_err.take() {
            Some(e) => Err(e),
            None => Ok(()),
        }
    }

    fn run_frames<'b>(&mut self, frames: &mut Frames<'b>, ret: Ty) {
        while let Some(bi) = frames.blocks.len().checked_sub(1) {
            if self.check_cancel() || self.budget_err.is_some() {
                return;
            }
            self.charge(1);
            let blk: &'b InputBlock = frames.blocks[bi];
            let next = blk.stmts.get(frames.idx[bi]);
            if next.is_some() {
                frames.idx[bi] += 1;
            }
            let Some(stmt) = next else {
                frames.blocks.pop();
                frames.idx.pop();
                frames.scopes.pop();
                continue;
            };
            match stmt {
                InputStmt::Let {
                    name,
                    ann,
                    mutable,
                    value,
                    ..
                } => {
                    let want = match ann {
                        Ann::Inferred => None,
                        a => Some(self.ann_ty(*a, value.span)),
                    };
                    self.walk(value, want, frames);
                    let got = self.type_of_recorded(value);
                    let bind = match (want, got) {
                        (Some(w), _) => w,
                        (None, Some(g)) => g,
                        (None, None) => self.mint_var(),
                    };
                    self.bindings.push((name.clone(), bind));
                    frames
                        .scopes
                        .last_mut()
                        .unwrap()
                        .insert(name.clone(), (bind, *mutable));
                }
                InputStmt::Assign {
                    target,
                    path,
                    value,
                    ..
                } => {
                    if !path.is_empty() {
                        self.diag(
                            "unsupported-construct",
                            value.span,
                            "field-path assignment is outside the limited i32/bool inference profile".into(),
                        );
                    }
                    match frames.lookup(&target) {
                        Some((t, mutable)) => {
                            if !mutable {
                                self.diag(
                                    "assign-immutable",
                                    value.span,
                                    format!("cannot assign to immutable `{target}`"),
                                );
                            }
                            self.walk(value, Some(t), frames);
                        }
                        None => self.record_negative("local", &target),
                    }
                }
                InputStmt::Return { value, .. } => {
                    self.walk(value, Some(ret), frames);
                }
                InputStmt::If {
                    cond, then, els, ..
                } => {
                    self.walk(cond, Some(Ty::Bool), frames);
                    if let Some(e) = els {
                        frames.push_block(e);
                    }
                    frames.push_block(then);
                }
                InputStmt::While { cond, body, .. } => {
                    self.walk(cond, Some(Ty::Bool), frames);
                    frames.push_block(body);
                }
                InputStmt::Loop { body, .. } => frames.push_block(body),
                InputStmt::Break { .. } | InputStmt::Continue { .. } => {}
            }
        }
    }

    fn type_of_recorded(&self, e: &InputExpr) -> Option<Ty> {
        self.expr_types
            .iter()
            .rev()
            .find(|(s, en, _)| *s == e.span.start && *en == e.span.end)
            .map(|(_, _, t)| *t)
    }

    // -- expression walk: bidirectional, explicit task stack ------------

    fn walk(&mut self, root: &InputExpr, expected: Option<Ty>, frames: &mut Frames<'_>) {
        enum Task<'e> {
            Synth(&'e InputExpr),
            Check(&'e InputExpr, Ty),
        }
        let mut stack = Vec::new();
        match expected {
            Some(t) => stack.push(Task::Check(root, t)),
            None => stack.push(Task::Synth(root)),
        }
        while let Some(task) = stack.pop() {
            if self.check_cancel() || self.budget_err.is_some() {
                return;
            }
            self.charge(1);
            let (e, expected) = match task {
                Task::Synth(e) => (e, None),
                Task::Check(e, t) => (e, Some(t)),
            };
            match &e.kind {
                InputExprKind::IntLit(_) => {
                    self.record_expr(e, Ty::I32, expected);
                }
                InputExprKind::BoolLit(_) => {
                    self.record_expr(e, Ty::Bool, expected);
                }
                InputExprKind::Var(name) => match frames.lookup(name) {
                    Some((t, _)) => {
                        self.record_expr(e, t, expected);
                    }
                    None => self.record_negative_at("local", name, e.span),
                },
                InputExprKind::Call { callee, args } => {
                    let input: &'a SemaInput = self.input;
                    let cands: Vec<_> = input.signatures().filter(|f| f.name == *callee).collect();
                    match cands.len() {
                        0 => self.record_negative_at("fn", callee, e.span),
                        1 => {
                            let f = cands[0];
                            self.read_sig(callee);
                            // PLAN-935 (T-04): a TEMPLATE call mints one
                            // shared var per declared type parameter, sub-
                            // stitutes them through the signature, and
                            // checks args/expected by REAL unification —
                            // heterogeneous bindings conflict here, an
                            // unconstrained T stays unsolved (definitive
                            // diagnostics, never Any/defaults). The solved
                            // substitution is recorded as an instance
                            // demand (collected after the fn's solve).
                            let tvars: Vec<Ty> = if f.type_params.is_empty() {
                                Vec::new()
                            } else {
                                (0..f.type_params.len()).map(|_| self.mint_var()).collect()
                            };
                            let subst = |ann: &Ann| -> Option<Ty> {
                                match ann {
                                    Ann::TyParam(i) => tvars.get(*i as usize).copied(),
                                    _ => None,
                                }
                            };
                            let fret = match subst(&f.ret) {
                                Some(t) => t,
                                None => self.ann_ty(f.ret, e.span),
                            };
                            self.record_expr(e, fret, expected);
                            // PLAN-934 T-04: arguments bind by NAME when
                            // written named, by position otherwise; the
                            // vector order remains the SOURCE evaluation
                            // order (binding must never reorder reads).
                            let mut bound: Vec<Option<usize>> = vec![None; f.params.len()];
                            let mut plan: Vec<(usize, Option<usize>)> =
                                Vec::with_capacity(args.len());
                            for (i, arg) in args.iter().enumerate() {
                                let pos = match &arg.name {
                                    None => Some(i),
                                    Some(n) => match f.params.iter().position(|(pn, _)| pn == n) {
                                        Some(p) => Some(p),
                                        None => {
                                            self.diag(
                                                "unknown-parameter",
                                                arg.value.span,
                                                format!("`{callee}` has no parameter `{n}`"),
                                            );
                                            None
                                        }
                                    },
                                };
                                if let Some(p) = pos {
                                    if p < bound.len() {
                                        match bound[p] {
                                            Some(prev) => self.diag(
                                                "duplicate-argument",
                                                arg.value.span,
                                                format!(
                                                    "`{callee}` binds parameter `{}` more than once (also argument {prev})",
                                                    f.params[p].0
                                                ),
                                            ),
                                            None => bound[p] = Some(i),
                                        }
                                    }
                                }
                                plan.push((i, pos));
                            }
                            // ARITY admission (F-796-R4-01): a direct call
                            // passes exactly the declared parameter count;
                            // a missing or extra binding is a definitive
                            // diagnostic, never an unsolved variable.
                            if args.len() != f.params.len() || bound.iter().any(|b| b.is_none()) {
                                self.diag(
                                    "arity-mismatch",
                                    e.span,
                                    format!(
                                        "`{callee}` expects {} argument(s), found {}",
                                        f.params.len(),
                                        args.len()
                                    ),
                                );
                            }
                            // Reverse source order on the LIFO stack keeps
                            // the written evaluation order.
                            for (i, pos) in plan.into_iter().rev() {
                                let arg = &args[i];
                                match pos {
                                    Some(p) if p < f.params.len() => {
                                        let t = match subst(&f.params[p].1) {
                                            Some(t) => t,
                                            None => self.ann_ty(f.params[p].1, arg.value.span),
                                        };
                                        stack.push(Task::Check(&arg.value, t));
                                    }
                                    _ => stack.push(Task::Synth(&arg.value)),
                                }
                            }
                            // PLAN-935 (T-04): record the instance demand for
                            // this call site (resolved after the fn solve).
                            if !tvars.is_empty() {
                                self.tpl_demands.push(TplDemand {
                                    callee: f.name.to_string(),
                                    vars: tvars,
                                    span: (e.span.start, e.span.end),
                                });
                            }
                        }
                        n => {
                            self.diag(
                                "ambiguous-candidates",
                                e.span,
                                format!("`{callee}` has {n} candidates in the fn domain"),
                            );
                            let set_fp = self.candidate_fp("fn");
                            self.reads.negatives.push(("fn", callee.clone(), set_fp));
                        }
                    }
                }
                InputExprKind::Binary { op, lhs, rhs } => {
                    use BinOpKind::*;
                    match op {
                        Eq | Ne => {
                            self.record_expr(e, Ty::Bool, expected);
                            self.eq_pairs.push((lhs.span, rhs.span, e.span));
                            stack.push(Task::Synth(rhs));
                            stack.push(Task::Synth(lhs));
                        }
                        _ => {
                            let (operand, result) = match op {
                                Lt | Le | Gt | Ge => (Ty::I32, Ty::Bool),
                                And | Or => (Ty::Bool, Ty::Bool),
                                _ => (Ty::I32, Ty::I32),
                            };
                            self.record_expr(e, result, expected);
                            stack.push(Task::Check(rhs, operand));
                            stack.push(Task::Check(lhs, operand));
                        }
                    }
                }
                InputExprKind::Unary { neg, operand } => {
                    let t = if *neg { Ty::I32 } else { Ty::Bool };
                    self.record_expr(e, t, expected);
                    stack.push(Task::Check(operand, t));
                }
                // PLAN-963（T-02）：文本字面量是文本面值。
                InputExprKind::StrLit(_) => {
                    self.record_expr(e, Ty::Text, expected);
                }
                // PLAN-963（T-02）：记录构造——结果是声明记录名，字段值按
                // 声明字段注解逐项核对（具名形；缺字段/未知名具名拒）。
                InputExprKind::Ctor { name, fields } => {
                    let input: &'a SemaInput = self.input;
                    match input.lookup_record(name) {
                        None => self.record_negative_at("type", name, e.span),
                        Some(rec) => {
                            self.record_expr(e, Ty::Record(name), expected);
                            let mut bound: Vec<bool> = vec![false; rec.fields.len()];
                            for f in fields {
                                match &f.name {
                                    Some(n) => {
                                        match rec.fields.iter().position(|(pn, _)| pn == n) {
                                            Some(p) => {
                                                bound[p] = true;
                                                let t = self.ann_ty(rec.fields[p].1, f.value.span);
                                                stack.push(Task::Check(&f.value, t));
                                            }
                                            None => self.diag(
                                                "unknown-field",
                                                f.value.span,
                                                format!("record `{name}` has no field `{n}`"),
                                            ),
                                        }
                                    }
                                    None => self.diag(
                                        "unknown-field",
                                        f.value.span,
                                        format!(
                                            "record construction `{name}` requires named field values"
                                        ),
                                    ),
                                }
                            }
                            for (i, (fname, _)) in rec.fields.iter().enumerate() {
                                if !bound[i] {
                                    self.diag(
                                        "missing-field",
                                        e.span,
                                        format!(
                                            "record construction `{name}` is missing field `{fname}`"
                                        ),
                                    );
                                }
                            }
                        }
                    }
                }
                // PLAN-963（T-02）：字段读取——结果类型取决于接收者类型，
                // 接收者的类型表项在 LIFO 次序上晚于本节点，故把约束延到
                // `finish`（那时本体的类型表已完整、联合查找已收敛）。
                InputExprKind::Field { base, name } => {
                    let out = self.mint_var();
                    self.field_reads.push(FieldRead {
                        base: base.span,
                        field: name.clone(),
                        site: e.span,
                        out_ty: out,
                    });
                    self.record_expr(e, out, expected);
                    stack.push(Task::Synth(base));
                }
                // PLAN-963（T-02）：方法调用——同上延迟；形参类型/返回类型
                // 由接收者记录的声明方法给出。`Recv.m(...)` 且 `Recv` 不是
                // 局部变量时按静态调用（接收者是记录名本身）。
                InputExprKind::MethodCall { recv, name, args } => {
                    let out = self.mint_var();
                    self.record_expr(e, out, expected);
                    let mut recv_task = true;
                    let recv_src = match &recv.kind {
                        InputExprKind::Var(n) if frames.lookup(n).is_none() => {
                            if self.input.lookup_record(n).is_some() {
                                recv_task = false;
                                MethodRecv::Record(n.clone())
                            } else {
                                MethodRecv::Value(recv.span)
                            }
                        }
                        _ => MethodRecv::Value(recv.span),
                    };
                    self.method_calls.push(MethodCallCheck {
                        recv: recv_src,
                        name: name.clone(),
                        args: args
                            .iter()
                            .enumerate()
                            .map(|(i, a)| (i, a.name.clone(), a.value.span))
                            .collect(),
                        site: e.span,
                        out_ty: out,
                    });
                    for a in args.iter().rev() {
                        stack.push(Task::Synth(&a.value));
                    }
                    if recv_task {
                        stack.push(Task::Synth(recv));
                    }
                }
                InputExprKind::Unsupported { what } => {
                    self.diag(
                        "unsupported-construct",
                        e.span,
                        format!("`{what}` is outside the limited i32/bool inference profile"),
                    );
                }
            }
        }
    }

    fn record_expr(&mut self, e: &InputExpr, t: Ty, expected: Option<Ty>) {
        self.expr_types.push((e.span.start, e.span.end, t));
        if let Some(want) = expected {
            self.unify(t, want, e.span, "check");
        }
    }

    // -- finalization ---------------------------------------------------

    fn fn_logical(&self, fn_name: &str) -> LogicalId {
        ace_semantics_contract::logical_id(
            &PackageId::new(&self.input.package),
            &self.input.module,
            DeclKind::Fn,
            fn_name,
        )
    }

    /// The recorded type of the expression at `span` (last write wins — a
    /// span belongs to exactly one expression in a well-formed body).
    fn expr_ty_at(&self, span: Span) -> Option<Ty> {
        self.expr_types
            .iter()
            .rev()
            .find(|(s, e, _)| *s == span.start && *e == span.end)
            .map(|(_, _, t)| *t)
    }

    /// PLAN-963（T-02）：解析延迟记录约束。接收者类型在 LIFO 次序上晚于
    /// 使用点可知，故约束在此统一求解；链式读取（`a.b.c`）需要多轮—
    /// 每轮只解出接收者已定的项，有界迭代至不动点。接收者仍是变元的项
    /// 保持未解（unsolved-variable 诊断已覆盖该判定），不虚报类型错误。
    fn resolve_deferred_record_ops(&mut self) {
        let mut fields: Vec<FieldRead> = std::mem::take(&mut self.field_reads);
        let mut calls: Vec<MethodCallCheck> = std::mem::take(&mut self.method_calls);
        let rounds = fields.len() + calls.len() + 1;
        for _ in 0..rounds {
            let mut moved = false;
            let mut next_fields: Vec<FieldRead> = Vec::new();
            for fr in fields.drain(..) {
                let base = match self.expr_ty_at(fr.base) {
                    Some(t) => self.resolve(t),
                    None => {
                        // 未被走到的接收者表达式：没有类型证据，具名拒。
                        self.diag(
                            "unsupported-construct",
                            fr.site,
                            format!("no recorded receiver type for field `{}`", fr.field),
                        );
                        moved = true;
                        continue;
                    }
                };
                match base {
                    Ty::Var(_) => next_fields.push(fr),
                    Ty::Record(rec_name) => {
                        moved = true;
                        let input: &'a SemaInput = self.input;
                        match input.lookup_record(rec_name) {
                            None => self.record_negative_at("type", rec_name, fr.site),
                            Some(rec) => match rec.fields.iter().find(|(n, _)| *n == fr.field) {
                                Some((_, ann)) => {
                                    let t = self.ann_ty(*ann, fr.site);
                                    self.unify(fr.out_ty, t, fr.site, "field");
                                }
                                None => {
                                    self.record_negative_at("field", &fr.field, fr.site);
                                }
                            },
                        }
                    }
                    other => {
                        moved = true;
                        self.diag(
                            "type-mismatch",
                            fr.site,
                            format!(
                                "field read `{}` on non-record type {}",
                                fr.field,
                                self.name_of(other)
                            ),
                        );
                    }
                }
            }
            fields = next_fields;
            let mut next_calls: Vec<MethodCallCheck> = Vec::new();
            for call in calls.drain(..) {
                let rec_name: Option<String> = match &call.recv {
                    MethodRecv::Record(n) => Some(n.clone()),
                    MethodRecv::Value(span) => match self.expr_ty_at(*span) {
                        Some(t) => match self.resolve(t) {
                            Ty::Record(n) => Some(n.to_string()),
                            Ty::Var(_) => None,
                            other => {
                                moved = true;
                                self.diag(
                                    "type-mismatch",
                                    call.site,
                                    format!(
                                        "method call `{}` on non-record type {}",
                                        call.name,
                                        self.name_of(other)
                                    ),
                                );
                                continue;
                            }
                        },
                        None => {
                            moved = true;
                            self.diag(
                                "unsupported-construct",
                                call.site,
                                format!("no recorded receiver type for method `{}`", call.name),
                            );
                            continue;
                        }
                    },
                };
                let Some(rec_name) = rec_name else {
                    next_calls.push(call);
                    continue;
                };
                moved = true;
                let input: &'a SemaInput = self.input;
                let Some(rec) = input.lookup_record(&rec_name) else {
                    self.record_negative_at("type", &rec_name, call.site);
                    continue;
                };
                let Some(m) = rec.methods.iter().find(|m| m.name == call.name) else {
                    self.record_negative_at("method", &call.name, call.site);
                    continue;
                };
                if !m.type_params.is_empty() {
                    self.diag(
                        "unsupported-construct",
                        call.site,
                        format!(
                            "generic method `{}` on `{rec_name}` is outside the inference profile",
                            call.name
                        ),
                    );
                    continue;
                }
                // 形参绑定：具名按名、否则按位；源序即求值序。
                let mut plan: Vec<Option<usize>> = vec![None; m.params.len()];
                for (i, wname, span) in &call.args {
                    let pos = match wname {
                        None => Some(*i),
                        Some(n) => match m.params.iter().position(|(pn, _)| pn == n) {
                            Some(p) => Some(p),
                            None => {
                                self.diag(
                                    "unknown-parameter",
                                    *span,
                                    format!("`{}` has no parameter `{n}`", call.name),
                                );
                                None
                            }
                        },
                    };
                    if let Some(p) = pos {
                        if p < plan.len() {
                            match plan[p] {
                                Some(prev) => self.diag(
                                    "duplicate-argument",
                                    *span,
                                    format!(
                                        "`{}` binds parameter `{}` more than once (also argument {prev})",
                                        call.name, m.params[p].0
                                    ),
                                ),
                                None => plan[p] = Some(*i),
                            }
                        }
                    }
                }
                if call.args.len() != m.params.len() || plan.iter().any(|p| p.is_none()) {
                    self.diag(
                        "arity-mismatch",
                        call.site,
                        format!(
                            "`{}` expects {} argument(s), found {}",
                            call.name,
                            m.params.len(),
                            call.args.len()
                        ),
                    );
                }
                for (p, argi) in plan.iter().enumerate() {
                    let Some(argi) = *argi else { continue };
                    let (_, _, span) = call.args[argi];
                    if let Some(at) = self.expr_ty_at(span) {
                        let want = self.ann_ty(m.params[p].1, span);
                        self.unify(at, want, span, "check");
                    }
                }
                let ret = self.ann_ty(m.ret, call.site);
                self.unify(call.out_ty, ret, call.site, "call");
            }
            calls = next_calls;
            if !moved {
                break;
            }
        }
        // 未收敛的残余约束：接收者仍是变元（unsolved-variable 已报）。
        self.field_reads = fields;
        self.method_calls = calls;
    }

    fn finish(&mut self, fn_name: &str) -> InferSummary {
        // PLAN-963（T-02）：先解析延迟的记录操作约束（字段读取/方法调用）
        // ——它们可能解出变元，必须先于 unsolved 判定。
        self.resolve_deferred_record_ops();
        // unsolved variables are definitive diagnostics — never Any
        for (i, slot) in self.vars.iter().enumerate() {
            if slot.is_none() {
                self.diags.push(FactDiag {
                    code: "unsolved-variable",
                    start: 0,
                    end: 0,
                    message: format!("type variable ∘{i} left unsolved"),
                });
            }
        }
        // ==/!= operand equality (checked once both sides are typed)
        let pairs = std::mem::take(&mut self.eq_pairs);
        for (ls, rs, site) in pairs {
            let lt = self
                .expr_types
                .iter()
                .find(|(s, en, _)| *s == ls.start && *en == ls.end)
                .map(|(_, _, t)| *t);
            let rt = self
                .expr_types
                .iter()
                .find(|(s, en, _)| *s == rs.start && *en == rs.end)
                .map(|(_, _, t)| *t);
            if let (Some(l), Some(r)) = (lt, rt) {
                let (l, r) = (self.resolve(l), self.resolve(r));
                if l != r {
                    self.diags.push(FactDiag {
                        code: "type-mismatch",
                        start: site.start,
                        end: site.end,
                        message: format!(
                            "both operands of ==/!= must have the same type ({} vs {})",
                            self.name_of(l),
                            self.name_of(r)
                        ),
                    });
                }
            }
        }
        // dedupe negative deps (same name looked up repeatedly)
        let mut seen = std::collections::BTreeSet::new();
        self.reads
            .negatives
            .retain(|(d, n, _)| seen.insert(format!("{d}\u{1}{n}")));
        let facts = InferFacts {
            fn_logical: self.fn_logical(fn_name),
            bindings: self
                .bindings
                .iter()
                .map(|(n, t)| (n.clone(), self.ty_desc(*t)))
                .collect(),
            expr_types: self
                .expr_types
                .iter()
                .map(|(s, en, t)| (*s, *en, self.ty_desc(*t)))
                .collect(),
            diagnostics: self.diags.clone(),
        };
        let steps = self.budget.steps_used();
        let elapsed = self.budget.elapsed();
        InferSummary {
            facts,
            reads: self.reads.clone(),
            steps,
            elapsed,
        }
    }

    fn name_of(&self, t: Ty) -> &'static str {
        match t {
            Ty::I32 => "i32",
            Ty::Bool => "bool",
            Ty::Text => "text",
            Ty::Record(name) => name,
            Ty::Var(_) => "unsolved",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::{InputBlock, InputExpr};

    use std::sync::atomic::{AtomicU32, Ordering as O};
    static NEXT_SPAN: AtomicU32 = AtomicU32::new(1);

    /// Distinct span per node (real adapter spans are unique; the
    /// eq-pair type lookup depends on it).
    fn expr(kind: InputExprKind) -> InputExpr {
        let s = NEXT_SPAN.fetch_add(1, O::Relaxed);
        InputExpr {
            kind,
            span: Span::new(s, s + 1),
        }
    }

    fn block(stmts: Vec<InputStmt>) -> InputBlock {
        InputBlock { stmts }
    }

    fn fn_of(name: &str, params: Vec<(&str, Ann)>, ret: Ann, body: InputBlock) -> InputFn {
        InputFn {
            name: name.into(),
            type_params: Vec::new(),
            params: params
                .into_iter()
                .map(|(n, a)| (n.to_string(), a))
                .collect(),
            ret,
            body,
        }
    }

    fn input(fns: Vec<InputFn>) -> SemaInput {
        SemaInput {
            external_signatures: Vec::new(),
            package: "test".into(),
            module: "m".into(),
            fns,
            records: Vec::new(),
            source_fp: String::new(),
        }
    }

    /// Positive: annotated arithmetic + condition + call consistency.
    #[test]
    fn infer_core_annotated_body_ok() {
        let inp = input(vec![
            fn_of(
                "add",
                vec![("a", Ann::I32), ("b", Ann::I32)],
                Ann::I32,
                block(vec![InputStmt::Return {
                    value: expr(InputExprKind::Binary {
                        op: BinOpKind::Add,
                        lhs: Box::new(expr(InputExprKind::Var("a".into()))),
                        rhs: Box::new(expr(InputExprKind::Var("b".into()))),
                    }),
                    span: Span::new(0, 0),
                }]),
            ),
            fn_of(
                "main",
                vec![],
                Ann::Bool,
                block(vec![
                    InputStmt::Let {
                        name: "x".into(),
                        ann: Ann::Inferred,
                        mutable: false,
                        value: expr(InputExprKind::Call {
                            callee: "add".into(),
                            args: vec![
                                crate::input::InputArg::pos(expr(InputExprKind::IntLit(1))),
                                crate::input::InputArg::pos(expr(InputExprKind::IntLit(2))),
                            ],
                        }),
                        span: Span::new(0, 0),
                    },
                    InputStmt::Return {
                        value: expr(InputExprKind::Binary {
                            op: BinOpKind::Lt,
                            lhs: Box::new(expr(InputExprKind::Var("x".into()))),
                            rhs: Box::new(expr(InputExprKind::IntLit(4))),
                        }),
                        span: Span::new(0, 0),
                    },
                ]),
            ),
        ]);
        let mut s = InferSession::new(&inp);
        match s.infer_function("main") {
            InferOutcome::Ok(sum) => {
                // un-annotated let took the call's i32
                assert!(sum
                    .facts
                    .bindings
                    .iter()
                    .any(|(n, t)| n == "x" && *t == TypeDesc::I32));
            }
            other => panic!("expected Ok, got {other:?}",),
        }
    }

    /// Negative: mixed-type == operands — a mismatch, never Any.
    #[test]
    fn infer_core_eq_mixed_types_diag() {
        let inp = input(vec![fn_of(
            "f",
            vec![],
            Ann::Bool,
            block(vec![InputStmt::Return {
                value: expr(InputExprKind::Binary {
                    op: BinOpKind::Eq,
                    lhs: Box::new(expr(InputExprKind::IntLit(1))),
                    rhs: Box::new(expr(InputExprKind::BoolLit(true))),
                }),
                span: Span::new(0, 0),
            }]),
        )]);
        let mut s = InferSession::new(&inp);
        match s.infer_function("f") {
            InferOutcome::Diags(sum) => {
                assert!(sum
                    .facts
                    .diagnostics
                    .iter()
                    .any(|d| d.code == "type-mismatch"));
            }
            other => panic!("expected Diags, got {other:?}"),
        }
    }

    /// Negative: unknown callee records the NEGATIVE dependency.
    #[test]
    fn infer_core_unknown_callee_negative_dep() {
        let inp = input(vec![fn_of(
            "f",
            vec![],
            Ann::I32,
            block(vec![InputStmt::Return {
                value: expr(InputExprKind::Call {
                    callee: "g".into(),
                    args: vec![],
                }),
                span: Span::new(0, 0),
            }]),
        )]);
        let mut s = InferSession::new(&inp);
        match s.infer_function("f") {
            InferOutcome::Diags(sum) => {
                assert!(sum
                    .facts
                    .diagnostics
                    .iter()
                    .any(|d| d.code == "unknown-name"));
                assert!(sum
                    .reads
                    .negatives
                    .iter()
                    .any(|(d, n, _)| *d == "fn" && n == "g"));
            }
            other => panic!("expected Diags, got {other:?}"),
        }
    }

    /// Negative: assigning to an immutable let.
    #[test]
    fn infer_core_assign_immutable_diag() {
        let inp = input(vec![fn_of(
            "f",
            vec![],
            Ann::I32,
            block(vec![
                InputStmt::Let {
                    name: "x".into(),
                    ann: Ann::Inferred,
                    mutable: false,
                    value: expr(InputExprKind::IntLit(1)),
                    span: Span::new(0, 0),
                },
                InputStmt::Assign {
                    target: "x".into(),
                    path: vec![],
                    value: expr(InputExprKind::IntLit(2)),
                    span: Span::new(0, 0),
                },
                InputStmt::Return {
                    value: expr(InputExprKind::Var("x".into())),
                    span: Span::new(0, 0),
                },
            ]),
        )]);
        let mut s = InferSession::new(&inp);
        match s.infer_function("f") {
            InferOutcome::Diags(sum) => {
                assert!(sum
                    .facts
                    .diagnostics
                    .iter()
                    .any(|d| d.code == "assign-immutable"));
            }
            other => panic!("expected Diags, got {other:?}"),
        }
    }

    /// Positive: shadowing — inner `x` shadows outer, types independent.
    #[test]
    fn infer_core_shadowing_independent_types() {
        let inp = input(vec![fn_of(
            "f",
            vec![],
            Ann::Bool,
            block(vec![
                InputStmt::Let {
                    name: "x".into(),
                    ann: Ann::Inferred,
                    mutable: false,
                    value: expr(InputExprKind::IntLit(7)),
                    span: Span::new(0, 0),
                },
                InputStmt::If {
                    cond: expr(InputExprKind::BoolLit(true)),
                    then: block(vec![InputStmt::Let {
                        name: "x".into(),
                        ann: Ann::Inferred,
                        mutable: false,
                        value: expr(InputExprKind::BoolLit(false)),
                        span: Span::new(0, 0),
                    }]),
                    els: None,
                    span: Span::new(0, 0),
                },
                InputStmt::Return {
                    value: expr(InputExprKind::Binary {
                        op: BinOpKind::Eq,
                        lhs: Box::new(expr(InputExprKind::Var("x".into()))),
                        rhs: Box::new(expr(InputExprKind::IntLit(7))),
                    }),
                    span: Span::new(0, 0),
                },
            ]),
        )]);
        let mut s = InferSession::new(&inp);
        match s.infer_function("f") {
            // outer x is still i32 after the inner bool shadow left scope
            InferOutcome::Ok(_) => {}
            other => panic!("expected Ok, got {other:?}"),
        }
    }

    /// Negative: ambiguity — two same-name candidates in the fn domain.
    #[test]
    fn infer_core_ambiguous_candidates_diag() {
        let body = block(vec![InputStmt::Return {
            value: expr(InputExprKind::Call {
                callee: "g".into(),
                args: vec![],
            }),
            span: Span::new(0, 0),
        }]);
        let inp = input(vec![
            fn_of("g", vec![], Ann::I32, body.clone()),
            fn_of("g", vec![], Ann::I32, body),
            fn_of(
                "main",
                vec![],
                Ann::I32,
                block(vec![InputStmt::Return {
                    value: expr(InputExprKind::Call {
                        callee: "g".into(),
                        args: vec![],
                    }),
                    span: Span::new(0, 0),
                }]),
            ),
        ]);
        let mut s = InferSession::new(&inp);
        match s.infer_function("main") {
            InferOutcome::Diags(sum) => {
                assert!(sum
                    .facts
                    .diagnostics
                    .iter()
                    .any(|d| d.code == "ambiguous-candidates"));
                // the candidate-set negative dep was recorded
                assert!(sum
                    .reads
                    .negatives
                    .iter()
                    .any(|(d, n, _)| *d == "fn" && n == "g"));
            }
            other => panic!("expected Diags, got {other:?}"),
        }
    }

    /// Solver invariant: a cyclic variable substitution (however it
    /// arose) is a controlled occurs-check failure — never a hang, an
    /// Any, or a wrong binding. The resolve-first binding discipline in
    /// `unify` cannot CREATE cycles through the public surface; this
    /// injects one out-of-band to prove the guard holds anyway.
    #[test]
    fn infer_core_occurs_check_cyclic_constraint() {
        let empty = input(vec![]);
        let mut s = InferSession::new(&empty);
        let v0 = s.mint_var();
        let v1 = s.mint_var();
        let v2 = s.mint_var();
        // corrupt the substitution into a cycle: ∘0 → ∘1 → ∘2 → ∘0
        s.vars[0] = Some(v1);
        s.vars[1] = Some(v2);
        s.vars[2] = Some(v0);
        let span = Span::new(1, 2);
        assert!(!s.unify(v0, v2, span, "cycle-probe"));
        assert!(s.diags.iter().any(|d| d.code == "occurs-check"));
        // plain resolve is cycle-safe (stops at revisit) — no hang
        assert!(matches!(s.resolve(v0), Ty::Var(_)));
    }

    /// Resource state: step budget crossing is an infrastructure state
    /// (not a diagnostic, not cacheable).
    #[test]
    fn infer_core_step_budget_infra_state() {
        // a chain of lets long enough to cross a tiny budget
        let mut stmts = vec![InputStmt::Let {
            name: "x0".into(),
            ann: Ann::Inferred,
            mutable: false,
            value: expr(InputExprKind::IntLit(1)),
            span: Span::new(0, 0),
        }];
        for i in 1..200 {
            stmts.push(InputStmt::Let {
                name: format!("x{i}"),
                ann: Ann::Inferred,
                mutable: false,
                value: expr(InputExprKind::Var(format!("x{}", i - 1))),
                span: Span::new(0, 0),
            });
        }
        stmts.push(InputStmt::Return {
            value: expr(InputExprKind::Var("x199".into())),
            span: Span::new(0, 0),
        });
        let inp = input(vec![fn_of("f", vec![], Ann::I32, block(stmts))]);
        let mut s = InferSession::new(&inp).with_budget(50, Duration::from_secs(5));
        match s.infer_function("f") {
            InferOutcome::ResourceExhausted(_) => {}
            other => panic!("expected ResourceExhausted, got {other:?}"),
        }
    }

    /// Cancellation: a set flag turns the run into Cancelled (never
    /// cached), with no diagnostics claimed.
    #[test]
    fn infer_core_cancel_flag() {
        let inp = input(vec![fn_of(
            "f",
            vec![],
            Ann::I32,
            block(vec![InputStmt::Return {
                value: expr(InputExprKind::IntLit(1)),
                span: Span::new(0, 0),
            }]),
        )]);
        let flag = Arc::new(AtomicBool::new(true));
        let mut s = InferSession::new(&inp).with_cancel(flag);
        assert!(matches!(s.infer_function("f"), InferOutcome::Cancelled));
    }

    /// Deep-surface safety: a 5000-deep operator spine infers without
    /// native recursion (drop safety is covered by the input model's
    /// iterative Drop; here the walker must survive the spine).
    #[test]
    fn infer_core_deep_spine_iterative() {
        let mut e = expr(InputExprKind::IntLit(1));
        for _ in 0..5000 {
            e = expr(InputExprKind::Binary {
                op: BinOpKind::Add,
                lhs: Box::new(e),
                rhs: Box::new(expr(InputExprKind::IntLit(1))),
            });
        }
        let inp = input(vec![fn_of(
            "f",
            vec![],
            Ann::I32,
            block(vec![InputStmt::Return {
                value: e,
                span: Span::new(0, 0),
            }]),
        )]);
        let mut s = InferSession::new(&inp);
        match s.infer_function("f") {
            InferOutcome::Ok(_) => {}
            other => panic!("expected Ok, got {other:?}"),
        }
    }

    // -- PLAN-934 T-04: named-argument input ---------------------------

    use crate::input::InputArg;

    fn pick() -> InputFn {
        // pick(a int, b bool) int { if b { return a } return 0 }
        fn_of(
            "pick",
            vec![("a", Ann::I32), ("b", Ann::Bool)],
            Ann::I32,
            block(vec![
                InputStmt::If {
                    cond: expr(InputExprKind::Var("b".into())),
                    then: block(vec![InputStmt::Return {
                        value: expr(InputExprKind::Var("a".into())),
                        span: Span::new(0, 0),
                    }]),
                    els: None,
                    span: Span::new(0, 0),
                },
                InputStmt::Return {
                    value: expr(InputExprKind::IntLit(0)),
                    span: Span::new(0, 0),
                },
            ]),
        )
    }

    fn call_pick(args: Vec<InputArg>) -> SemaInput {
        input(vec![
            pick(),
            fn_of(
                "main",
                vec![],
                Ann::I32,
                block(vec![InputStmt::Return {
                    value: expr(InputExprKind::Call {
                        callee: "pick".into(),
                        args,
                    }),
                    span: Span::new(0, 0),
                }]),
            ),
        ])
    }

    /// Named binding: `pick(b: true, a: 7)` binds `true` to bool `b` and
    /// `7` to int `a` — a positional reading would be a type error.
    #[test]
    fn infer_core_named_arguments_bind_by_name() {
        let inp = call_pick(vec![
            InputArg::named("b", expr(InputExprKind::BoolLit(true))),
            InputArg::named("a", expr(InputExprKind::IntLit(7))),
        ]);
        let mut s = InferSession::new(&inp);
        match s.infer_function("main") {
            InferOutcome::Ok(_) => {}
            other => panic!("expected Ok, got {other:?}"),
        }
    }

    /// Control: the SAME operands positionally are a real type error, so
    /// the passing case above cannot come from ignoring the names.
    #[test]
    fn infer_core_named_positional_control_is_a_type_error() {
        let inp = call_pick(vec![
            InputArg::pos(expr(InputExprKind::BoolLit(true))),
            InputArg::pos(expr(InputExprKind::IntLit(7))),
        ]);
        let mut s = InferSession::new(&inp);
        match s.infer_function("main") {
            InferOutcome::Diags(sum) => assert!(sum
                .facts
                .diagnostics
                .iter()
                .any(|d| d.code == "type-mismatch")),
            other => panic!("expected Diags, got {other:?}"),
        }
    }

    /// Mixed form: positional first, then a named argument.
    #[test]
    fn infer_core_named_mixed_with_positional() {
        let inp = call_pick(vec![
            InputArg::pos(expr(InputExprKind::IntLit(7))),
            InputArg::named("b", expr(InputExprKind::BoolLit(true))),
        ]);
        let mut s = InferSession::new(&inp);
        match s.infer_function("main") {
            InferOutcome::Ok(_) => {}
            other => panic!("expected Ok, got {other:?}"),
        }
    }

    /// Unknown parameter, duplicate binding and a missing parameter are
    /// definitive diagnostics (never silently ignored).
    #[test]
    fn infer_core_named_admission_diagnostics() {
        let unknown = call_pick(vec![
            InputArg::named("a", expr(InputExprKind::IntLit(1))),
            InputArg::named("zz", expr(InputExprKind::BoolLit(true))),
        ]);
        let mut s = InferSession::new(&unknown);
        match s.infer_function("main") {
            InferOutcome::Diags(sum) => {
                assert!(sum
                    .facts
                    .diagnostics
                    .iter()
                    .any(|d| d.code == "unknown-parameter"));
            }
            other => panic!("expected Diags, got {other:?}"),
        }

        let duplicate = call_pick(vec![
            InputArg::named("a", expr(InputExprKind::IntLit(1))),
            InputArg::named("a", expr(InputExprKind::IntLit(2))),
        ]);
        let mut s = InferSession::new(&duplicate);
        match s.infer_function("main") {
            InferOutcome::Diags(sum) => {
                assert!(sum
                    .facts
                    .diagnostics
                    .iter()
                    .any(|d| d.code == "duplicate-argument"));
            }
            other => panic!("expected Diags, got {other:?}"),
        }

        let missing = call_pick(vec![InputArg::named("a", expr(InputExprKind::IntLit(1)))]);
        let mut s = InferSession::new(&missing);
        match s.infer_function("main") {
            InferOutcome::Diags(sum) => {
                assert!(sum
                    .facts
                    .diagnostics
                    .iter()
                    .any(|d| d.code == "arity-mismatch"));
            }
            other => panic!("expected Diags, got {other:?}"),
        }
    }

    /// A named argument still checks its TYPE against the bound parameter.
    #[test]
    fn infer_core_named_argument_type_is_checked_against_its_parameter() {
        let inp = call_pick(vec![
            InputArg::named("a", expr(InputExprKind::BoolLit(true))),
            InputArg::named("b", expr(InputExprKind::BoolLit(false))),
        ]);
        let mut s = InferSession::new(&inp);
        match s.infer_function("main") {
            InferOutcome::Diags(sum) => assert!(sum
                .facts
                .diagnostics
                .iter()
                .any(|d| d.code == "type-mismatch")),
            other => panic!("expected Diags, got {other:?}"),
        }
    }
}

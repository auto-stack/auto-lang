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
use ac_sema_contract::{FactDiag, InferFacts, LogicalId, PackageId, TypeDesc, DeclKind};
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
        Frames { blocks: Vec::new(), idx: Vec::new(), scopes: vec![BTreeMap::new()] }
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
    // -- facts being recorded --
    bindings: Vec<(String, Ty)>,
    expr_types: Vec<(u32, u32, Ty)>,
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
            bindings: Vec::new(),
            expr_types: Vec::new(),
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
        self.cancel.as_ref().is_some_and(|c| c.load(Ordering::Relaxed))
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

    /// Annotation → solver type. Record and out-of-profile annotations
    /// inside a BODY are outside the limited i32/bool profile: they
    /// surface as unsupported-construct (never silently widened).
    fn ann_ty(&mut self, ann: Ann, span: Span) -> Ty {
        match ann {
            Ann::I32 => Ty::I32,
            Ann::Bool => Ty::Bool,
            Ann::Record(_) | Ann::Unsupported(_) => {
                self.diag(
                    "unsupported-construct",
                    span,
                    "record/out-of-profile annotations are outside the limited i32/bool inference profile".into(),
                );
                self.mint_var()
            }
            Ann::Inferred => self.mint_var(),
        }
    }

    fn ty_desc(&self, t: Ty) -> TypeDesc {
        match self.resolve(t) {
            Ty::I32 => TypeDesc::I32,
            Ty::Bool => TypeDesc::Bool,
            Ty::Var(_) => TypeDesc::Opaque { name: "?unsolved".into() },
        }
    }

    fn diag(&mut self, code: &'static str, span: Span, message: String) {
        self.diags.push(FactDiag { code, start: span.start, end: span.end, message });
    }

    fn record_negative(&mut self, domain: &'static str, name: &str) {
        let set_fp = self.candidate_fp(domain);
        self.reads.negatives.push((domain, name.to_string(), set_fp));
        self.diags.push(FactDiag {
            code: "unknown-name",
            start: 0,
            end: 0,
            message: format!("unknown {} name `{}`", domain, name),
        });
    }

    fn candidate_fp(&self, domain: &str) -> String {
        let set = match domain {
            "fn" => self.input.fn_candidates(),
            "type" => self.input.record_candidates(),
            _ => Vec::new(),
        };
        let joined = set.join("\u{1}");
        ac_sema_contract::identity::domain_hash("ac796/candidate-set/v1", &[joined.as_bytes()])
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
            frames.scopes.last_mut().unwrap().insert(name.clone(), (t, true));
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
                InputStmt::Let { name, ann, mutable, value, .. } => {
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
                    frames.scopes.last_mut().unwrap().insert(name.clone(), (bind, *mutable));
                }
                InputStmt::Assign { target, path, value, .. } => {
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
                InputStmt::If { cond, then, els, .. } => {
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
                    None => self.record_negative("local", name),
                },
                InputExprKind::Call { callee, args } => {
                    let input: &'a SemaInput = self.input;
                    let cands: Vec<&'a InputFn> =
                        input.fns.iter().filter(|f| f.name == *callee).collect();
                    match cands.len() {
                        0 => self.record_negative("fn", callee),
                        1 => {
                            let f = cands[0];
                            self.read_sig(callee);
                            // ARITY admission (F-796-R4-01): a direct call
                            // must pass exactly the declared parameter
                            // count — missing or extra arguments are
                            // definitive diagnostics, never unsolved
                            // variables or silently ignored extras
                            if args.len() != f.params.len() {
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
                            let fret = self.ann_ty(f.ret, e.span);
                            self.record_expr(e, fret, expected);
                            for (i, arg) in args.iter().enumerate().rev() {
                                match f.params.get(i).map(|(_, ann)| *ann) {
                                    Some(a) => {
                                        let t = self.ann_ty(a, arg.span);
                                        stack.push(Task::Check(arg, t));
                                    }
                                    None => stack.push(Task::Synth(arg)),
                                }
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
        ac_sema_contract::logical_id(
            &PackageId::new(&self.input.package),
            &self.input.module,
            DeclKind::Fn,
            fn_name,
        )
    }

    fn finish(&mut self, fn_name: &str) -> InferSummary {
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
        self.reads.negatives.retain(|(d, n, _)| seen.insert(format!("{d}\u{1}{n}")));
        let facts = InferFacts {
            fn_logical: self.fn_logical(fn_name),
            bindings: self.bindings.iter().map(|(n, t)| (n.clone(), self.ty_desc(*t))).collect(),
            expr_types: self
                .expr_types
                .iter()
                .map(|(s, en, t)| (*s, *en, self.ty_desc(*t)))
                .collect(),
            diagnostics: self.diags.clone(),
        };
        let steps = self.budget.steps_used();
        let elapsed = self.budget.elapsed();
        InferSummary { facts, reads: self.reads.clone(), steps, elapsed }
    }

    fn name_of(&self, t: Ty) -> &'static str {
        match t {
            Ty::I32 => "i32",
            Ty::Bool => "bool",
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
        InputExpr { kind, span: Span::new(s, s + 1) }
    }

    fn block(stmts: Vec<InputStmt>) -> InputBlock {
        InputBlock { stmts }
    }

    fn fn_of(name: &str, params: Vec<(&str, Ann)>, ret: Ann, body: InputBlock) -> InputFn {
        InputFn {
            name: name.into(),
            params: params.into_iter().map(|(n, a)| (n.to_string(), a)).collect(),
            ret,
            body,
        }
    }

    fn input(fns: Vec<InputFn>) -> SemaInput {
        SemaInput { package: "test".into(), module: "m".into(), fns, records: Vec::new(), source_fp: String::new() }
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
                            args: vec![expr(InputExprKind::IntLit(1)), expr(InputExprKind::IntLit(2))],
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
                assert!(sum.facts.bindings.iter().any(|(n, t)| n == "x" && *t == TypeDesc::I32));
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
                assert!(sum.facts.diagnostics.iter().any(|d| d.code == "type-mismatch"));
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
                value: expr(InputExprKind::Call { callee: "g".into(), args: vec![] }),
                span: Span::new(0, 0),
            }]),
        )]);
        let mut s = InferSession::new(&inp);
        match s.infer_function("f") {
            InferOutcome::Diags(sum) => {
                assert!(sum.facts.diagnostics.iter().any(|d| d.code == "unknown-name"));
                assert!(sum.reads.negatives.iter().any(|(d, n, _)| *d == "fn" && n == "g"));
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
                assert!(sum.facts.diagnostics.iter().any(|d| d.code == "assign-immutable"));
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
            value: expr(InputExprKind::Call { callee: "g".into(), args: vec![] }),
            span: Span::new(0, 0),
        }]);
        let inp = input(vec![
            fn_of("g", vec![], Ann::I32, body.clone()),
            fn_of("g", vec![], Ann::I32, body),
            fn_of("main", vec![], Ann::I32, block(vec![InputStmt::Return {
                value: expr(InputExprKind::Call { callee: "g".into(), args: vec![] }),
                span: Span::new(0, 0),
            }])),
        ]);
        let mut s = InferSession::new(&inp);
        match s.infer_function("main") {
            InferOutcome::Diags(sum) => {
                assert!(sum.facts.diagnostics.iter().any(|d| d.code == "ambiguous-candidates"));
                // the candidate-set negative dep was recorded
                assert!(sum.reads.negatives.iter().any(|(d, n, _)| *d == "fn" && n == "g"));
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
            block(vec![InputStmt::Return { value: e, span: Span::new(0, 0) }]),
        )]);
        let mut s = InferSession::new(&inp);
        match s.infer_function("f") {
            InferOutcome::Ok(_) => {}
            other => panic!("expected Ok, got {other:?}"),
        }
    }
}

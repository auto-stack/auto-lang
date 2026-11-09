//! The CT demand planner/scheduler of the source lane (PLAN-936 T-05,
//! §5.2/§5.8).
//!
//! One ROOT operation (compiling an owner program that carries `#{ ... }`
//! demands) walks a SHARED cumulative budget and serves every demand as a
//! node of an implicit DAG:
//!
//! * **Phase rule (§5.8.1)** — an EXPLICIT `#{ ... }` is a compile-time
//!   demand wherever it appears: in the owner's body, nested inside
//!   another hash expression, or in the body of any callee in the CT
//!   closure. All of them are planned and served before the wrapper is
//!   built; a single hash's un-taken runtime trap stays latent (the AV's
//!   own control flow decides).
//! * **Meta queries are local** — `ct_field_count`/`ct_has_field` read
//!   the CURRENT nominal declarations (field count / membership). They
//!   are counted as `meta_query`, never as AV execution, and their
//!   results become REAL AV arguments by literal substitution.
//! * **Cycles are named** — a demand whose closure needs a CT value from
//!   a function already on the serving stack is a query cycle and is
//!   rejected with the whole chain; ordinary function recursion is NOT a
//!   cycle (it executes on the AV under fuel).
//! * **Root-internal single-flight** — two sites with the same complete
//!   key in one root share ONE execution (the store dedups across roots;
//!   this dedups before any freeze, within the root).
//! * **Cumulative deadline** — the root budget (10s total) is shared by
//!   every demand in the root; each AV call narrows to
//!   `min(MAX_CALL_WALL, remaining)` and NO demand resets it. Exhaustion
//!   is a resource state; nothing is published.

use crate::executor::CtExecError;
use crate::identity::{CtValue, ExpectTy};
use crate::service::{CtRequest, CtService};
use crate::store::CtEnvelopeSink;
use ace_core::atom_text::Span;
use ace_core::source::ast::{Arg, Block, Expr, ExprKind, Item, Program, Stmt};
use std::collections::BTreeMap;
use std::time::{Duration, Instant};

/// The shared root budget (§5.8.4: one cumulative remaining deadline).
#[derive(Clone, Copy, Debug)]
pub struct RootBudget {
    started: Instant,
    total: Duration,
}

impl RootBudget {
    pub fn new(total: Duration) -> RootBudget {
        RootBudget {
            started: Instant::now(),
            total,
        }
    }
    /// The frozen default (the 817 REQUEST_WALL_LIMIT axis).
    pub fn default_total() -> Duration {
        Duration::from_secs(10)
    }
    pub fn remaining(&self) -> Duration {
        self.total.saturating_sub(self.started.elapsed())
    }
    pub fn exceeded(&self) -> bool {
        self.remaining().is_zero()
    }
    /// One AV call's wall: the per-call cap narrowed by what remains —
    /// never the reverse (F-817-R6-01).
    pub fn av_wall(&self) -> Duration {
        crate::MAX_CALL_WALL.min(self.remaining())
    }
}

/// One meta selector axis of a demand (the key component §5.7 requires:
/// "meta 字段集/缺席探针"). The field-set canonical is the record's
/// declared field list — adding/removing a field changes the axis, so a
/// frozen result from the old shape misses.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct MetaAxis {
    pub record: String,
    pub field_set: String,
}

/// Why a root could not be served.
#[derive(Clone, Debug)]
pub enum PlanError {
    /// A query cycle: the chain of (owner) names that closed the loop.
    Cycle { chain: Vec<String> },
    /// The root budget ran out (resource state; nothing published).
    Deadline { elapsed_ms: u128 },
    /// A demand failed (executor/admission); forwarded with its site.
    Demand { owner: String, err: CtExecError },
}

impl std::fmt::Display for PlanError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PlanError::Cycle { chain } => write!(
                f,
                "cte.query-cycle: {} (these functions need each other's compile-time values)",
                chain.join(" -> ")
            ),
            PlanError::Deadline { elapsed_ms } => write!(
                f,
                "cte.resource.deadline: the root budget was exhausted after {elapsed_ms}ms (resource state; nothing published)"
            ),
            PlanError::Demand { owner, err } => {
                write!(f, "cte demand in `{owner}` failed: {err}")
            }
        }
    }
}

/// The served root: every hash site with its frozen value, keyed by span
/// (the structural backfill input of T-06/T-07 — substitute before
/// resolve, never text).
#[derive(Clone, Debug, Default)]
pub struct ServedRoot {
    pub values: BTreeMap<(u64, u64), CtValue>,
    /// Measured meta queries (never AV executions).
    pub meta_queries: u64,
    /// The meta selector axes this root read (record + field-set canon).
    /// The substituted arguments already carry the VALUES (invalidation
    /// rides the args axis on this surface); the explicit keying seam is
    /// the T-08 typed-dependency axis.
    pub meta_axes: Vec<MetaAxis>,
}

impl ServedRoot {
    /// The frozen value of one hash site (by span).
    pub fn value_at(&self, span: Span) -> Option<CtValue> {
        self.values
            .get(&(span.start as u64, span.end as u64))
            .copied()
    }
}

/// Serve every CT demand of one program under one shared root budget.
///
/// Demands are discovered in three layers (all explicit — the phase
/// rule): hash sites in ANY function body (an owner's callee closure
/// carrying a hash makes that hash a demand of this root), nested hashes
/// inside a hash expression, and meta selectors inside a hash expression.
pub fn serve_program<S: CtEnvelopeSink>(
    svc: &mut CtService<S>,
    program: &Program,
    src: &str,
    module_path: &str,
    package: &str,
    module: &str,
    budget: &RootBudget,
) -> Result<ServedRoot, PlanError> {
    let mut out = ServedRoot::default();
    let mut stack: Vec<(String, (u64, u64))> = Vec::new();
    let sites = all_hash_sites(program);
    for (owner, span) in sites {
        if out
            .values
            .contains_key(&(span.start as u64, span.end as u64))
        {
            continue; // already served as a nested/closure demand
        }
        serve_site(
            svc,
            program,
            src,
            module_path,
            package,
            module,
            &owner,
            span,
            &mut stack,
            &mut out,
            budget,
        )?;
    }
    Ok(out)
}

/// Serve one hash site (by owner + span of the HashExpr node).
#[allow(clippy::too_many_arguments)]
fn serve_site<S: CtEnvelopeSink>(
    svc: &mut CtService<S>,
    program: &Program,
    src: &str,
    module_path: &str,
    package: &str,
    module: &str,
    owner: &str,
    span: Span,
    stack: &mut Vec<(String, (u64, u64))>,
    out: &mut ServedRoot,
    budget: &RootBudget,
) -> Result<(), PlanError> {
    if budget.exceeded() {
        return Err(PlanError::Deadline {
            elapsed_ms: budget.started.elapsed().as_millis(),
        });
    }
    // Cycle: the SAME site (owner + span) is already being served on the
    // chain (a nested site of the same owner is a different node — it is
    // part of the enclosing site's plan, not a loop).
    let node = (owner.to_string(), (span.start as u64, span.end as u64));
    if let Some(hit) = stack.iter().position(|o| *o == node) {
        let mut chain: Vec<String> = stack[hit..].iter().map(|(o, _)| o.clone()).collect();
        chain.push(owner.to_string());
        return Err(PlanError::Cycle { chain });
    }
    stack.push(node);
    let result = serve_site_inner(
        svc,
        program,
        src,
        module_path,
        package,
        module,
        owner,
        span,
        stack,
        out,
        budget,
    );
    stack.pop();
    result
}

#[allow(clippy::too_many_arguments)]
fn serve_site_inner<S: CtEnvelopeSink>(
    svc: &mut CtService<S>,
    program: &Program,
    src: &str,
    module_path: &str,
    package: &str,
    module: &str,
    owner: &str,
    span: Span,
    stack: &mut Vec<(String, (u64, u64))>,
    out: &mut ServedRoot,
    budget: &RootBudget,
) -> Result<(), PlanError> {
    // Locate the HashExpr node (owner fn body).
    let Some(hash_expr) = find_hash(program, owner, span) else {
        return Err(PlanError::Demand {
            owner: owner.to_string(),
            err: CtExecError {
                code: "cte.plan/site-missing".into(),
                message: format!("no `#{{ ... }}` at the recorded span of `{owner}`"),
                av_reached: false,
            },
        });
    };
    let ExprKind::HashExpr { expr: inner } = &hash_expr.kind else {
        return Err(PlanError::Demand {
            owner: owner.to_string(),
            err: CtExecError {
                code: "cte.plan/not-a-hash".into(),
                message: "the recorded span does not carry a HashExpr".into(),
                av_reached: false,
            },
        });
    };
    // 1) Nested hashes FIRST (they are demands of this demand): serve and
    //    record their values for substitution.
    for (nested_owner, nested_span) in hashes_in_expr(inner, owner) {
        if !out
            .values
            .contains_key(&(nested_span.start as u64, nested_span.end as u64))
        {
            serve_site(
                svc,
                program,
                src,
                module_path,
                package,
                module,
                &nested_owner,
                nested_span,
                stack,
                out,
                budget,
            )?;
        }
    }
    // 2) Hash sites inside the CALLEE closure bodies (phase rule: an
    //    explicit hash in a needed callee is a demand of this root).
    for callee in closure_of(program, owner) {
        for (closure_owner, closure_span) in hashes_in_body_of(program, &callee) {
            if !out
                .values
                .contains_key(&(closure_span.start as u64, closure_span.end as u64))
            {
                serve_site(
                    svc,
                    program,
                    src,
                    module_path,
                    package,
                    module,
                    &closure_owner,
                    closure_span,
                    stack,
                    out,
                    budget,
                )?;
            }
        }
    }
    // 3) Meta selectors: evaluate LOCALLY against the current
    //    declarations (meta_query counter; never an AV execution).
    let mut meta_vals: BTreeMap<(u64, u64), CtValue> = BTreeMap::new();
    let mut meta_axes: Vec<MetaAxis> = Vec::new();
    for sel in meta_selectors_in(inner) {
        // Product admission has already rejected protected declarations
        // under the meta grant. Otherwise a declared same-name function is
        // ordinary, even when its arguments resemble record selectors. Do
        // not fold an ill-typed ordinary call into a successful meta result.
        if program.items.iter().any(|item| {
            matches!(item, Item::Fn(f)
                if ace_core::source::meta_intrinsic(&f.name.name) == Some(sel.op))
        }) {
            continue;
        }
        let Some(rec) = record_of(program, &sel.type_name) else {
            return Err(PlanError::Demand {
                owner: owner.to_string(),
                err: CtExecError {
                    code: "cte.meta/unknown-type".into(),
                    message: format!("`{}` is not a declared record", sel.type_name),
                    av_reached: false,
                },
            });
        };
        let field_set = rec
            .fields
            .iter()
            .map(|f| f.name.name.clone())
            .collect::<Vec<_>>()
            .join(",");
        meta_axes.push(MetaAxis {
            record: sel.type_name.clone(),
            field_set: field_set.clone(),
        });
        let value = match &sel.field {
            None => CtValue::I32(rec.fields.len() as i32),
            Some(f) => CtValue::Bool(rec.fields.iter().any(|x| x.name.name == *f)),
        };
        out.meta_queries += 1;
        meta_vals.insert((sel.span.start as u64, sel.span.end as u64), value);
    }
    // 4) Substitute (nested hash values + meta values -> literals) and
    //    serve through the store-keyed service (single-flight by key).
    let substituted = substitute(inner, out, &meta_vals);
    out.meta_axes.append(&mut meta_axes);
    // The demand's compile program: CLOSURE-FILTERED (the owner, its
    // transitive callee closure and the type declarations — §5.3 "callee/
    // instance and executable dependencies first obtain shared Checked
    // HIR"; the module's unrelated functions are not this demand's
    // admission), with every ALREADY-SERVED site replaced by its frozen
    // literal and THIS site's hash replaced by its substituted inner
    // expression. The wrapper compile (resolve of the extended program,
    // caps = none) then sees no hash and no meta selector at all: the CT
    // closure is compiled under the CT domain's caps (none granted), and
    // still-unserved sites elsewhere in the module cannot pollute this
    // demand.
    let keep = closure_of(program, owner);
    let mut site_program = Program {
        items: program
            .items
            .iter()
            .filter(|i| match i {
                Item::Fn(f) => keep.contains(&f.name.name),
                Item::Type(_) => true,
                _ => false,
            })
            .cloned()
            .collect(),
    };
    substitute_program_in_place(&mut site_program, out, span, &substituted);
    // A substituted LITERAL (pure-literal or meta-only demand) IS the
    // value — plan-level, never an AV execution (§5.2 literal+Meta 解析).
    if let Some(v) = lit_value(&substituted) {
        out.values.insert((span.start as u64, span.end as u64), v);
        return Ok(());
    }
    let expect = expect_of_in(Some(&site_program), &substituted);
    let args = direct_call_args(&substituted);
    let callee = direct_call_callee(&substituted);
    svc.set_wall_budget(budget.av_wall());
    // The demand's value lands in the root map either way.
    let ans = if let (Some(callee), Some(args)) = (&callee, &args) {
        // the plain call form keeps T-02's key shape
        let req = CtRequest {
            callee: callee.clone(),
            args: args.clone(),
            expect,
            binding: crate::identity::UseBinding {
                package: package.into(),
                module: module.into(),
                owner: owner.into(),
                expect,
            },
        };
        svc.execute_ct(&site_program, src, module_path, &req)
            .map_err(|e| PlanError::Demand {
                owner: owner.to_string(),
                err: e,
            })?
    } else {
        // T-06: the general-expression wrapper (same cache/admission/
        // counter/log discipline; key = expr canonical + closure axes).
        let expr_is_bool = expect == ExpectTy::Bool;
        svc.execute_expr_ct(
            &site_program,
            src,
            module_path,
            &substituted,
            expr_is_bool,
            expect,
            &crate::identity::UseBinding {
                package: package.into(),
                module: module.into(),
                owner: owner.into(),
                expect,
            },
        )
        .map_err(|e| PlanError::Demand {
            owner: owner.to_string(),
            err: e,
        })?
    };
    out.values
        .insert((span.start as u64, span.end as u64), ans.value);
    Ok(())
}

// -- discovery walks ------------------------------------------------------

/// Every hash site of the program: (owner fn name, HashExpr span).
pub fn all_hash_sites(program: &Program) -> Vec<(String, Span)> {
    let mut out = Vec::new();
    for item in &program.items {
        if let Item::Fn(f) = item {
            collect_hashes_block(&f.body, &f.name.name, &mut out);
        }
    }
    out
}

fn collect_hashes_block(b: &Block, owner: &str, out: &mut Vec<(String, Span)>) {
    for s in &b.stmts {
        collect_hashes_stmt(s, owner, out);
    }
}

fn collect_hashes_stmt(s: &Stmt, owner: &str, out: &mut Vec<(String, Span)>) {
    match s {
        Stmt::Return { value, .. } | Stmt::ExprStmt { value, .. } | Stmt::Let { value, .. } => {
            collect_hashes_expr(value, owner, out)
        }
        Stmt::Assign { value, .. } => collect_hashes_expr(value, owner, out),
        Stmt::If {
            cond, then, els, ..
        } => {
            collect_hashes_expr(cond, owner, out);
            collect_hashes_block(then, owner, out);
            if let Some(b) = els {
                collect_hashes_block(b, owner, out);
            }
        }
        Stmt::While { cond, body, .. } => {
            collect_hashes_expr(cond, owner, out);
            collect_hashes_block(body, owner, out);
        }
        Stmt::Loop { body, .. } => collect_hashes_block(body, owner, out),
        Stmt::Break { .. } | Stmt::Continue { .. } => {}
        Stmt::IsMatch { arms, .. } => {
            for a in arms {
                collect_hashes_block(&a.body, owner, out);
            }
        }
        Stmt::ForIn { iter, body, .. } => {
            collect_hashes_expr(iter, owner, out);
            collect_hashes_block(body, owner, out);
        }
    }
}

fn collect_hashes_expr(e: &Expr, owner: &str, out: &mut Vec<(String, Span)>) {
    if let ExprKind::HashExpr { .. } = e.kind {
        out.push((owner.to_string(), e.span));
        return; // nested hashes belong to the enclosing site's plan
    }
    match &e.kind {
        ExprKind::Call { args, .. }
        | ExprKind::Method { args, .. }
        | ExprKind::MethodCall { args, .. } => {
            for a in args {
                if let Arg::Pos(x) | Arg::MovePos(x) = a {
                    collect_hashes_expr(x, owner, out);
                }
            }
        }
        ExprKind::CallValue { callee, args } => {
            collect_hashes_expr(callee, owner, out);
            for a in args {
                if let Arg::Pos(x) | Arg::MovePos(x) = a {
                    collect_hashes_expr(x, owner, out);
                }
            }
        }
        ExprKind::Binary { lhs, rhs, .. } => {
            collect_hashes_expr(lhs, owner, out);
            collect_hashes_expr(rhs, owner, out);
        }
        ExprKind::Unary { operand, .. } => collect_hashes_expr(operand, owner, out),
        ExprKind::Field { base, .. } => collect_hashes_expr(base, owner, out),
        _ => {}
    }
}

/// Nested hash sites inside one hash expression (owner = the enclosing
/// fn — nested hashes execute in the enclosing site's owner context).
fn hashes_in_expr(e: &Expr, owner: &str) -> Vec<(String, Span)> {
    let mut out = Vec::new();
    collect_hashes_expr(e, owner, &mut out);
    out
}

/// Hash sites in one function's body (named by that fn).
fn hashes_in_body_of(program: &Program, name: &str) -> Vec<(String, Span)> {
    let mut out = Vec::new();
    for item in &program.items {
        if let Item::Fn(f) = item {
            if f.name.name == name {
                collect_hashes_block(&f.body, name, &mut out);
            }
        }
    }
    out
}

/// The direct free-call closure of one fn (names; resolver owns binding).
fn closure_of(program: &Program, root: &str) -> Vec<String> {
    let mut seen = vec![root.to_string()];
    let mut stack = direct_callees_of(program, root);
    while let Some(n) = stack.pop() {
        if seen.contains(&n) {
            continue;
        }
        seen.push(n.clone());
        for c in direct_callees_of(program, &n) {
            if !seen.contains(&c) {
                stack.push(c);
            }
        }
    }
    seen.remove(0);
    seen
}

fn direct_callees_of(program: &Program, name: &str) -> Vec<String> {
    for item in &program.items {
        if let Item::Fn(f) = item {
            if f.name.name == name {
                let mut out = Vec::new();
                callees_block(&f.body, &mut out);
                return out;
            }
        }
    }
    Vec::new()
}

fn callees_block(b: &Block, out: &mut Vec<String>) {
    for s in &b.stmts {
        callees_stmt(s, out);
    }
}

fn callees_stmt(s: &Stmt, out: &mut Vec<String>) {
    match s {
        Stmt::Return { value, .. } | Stmt::ExprStmt { value, .. } | Stmt::Let { value, .. } => {
            callees_expr(value, out)
        }
        Stmt::Assign { value, .. } => callees_expr(value, out),
        Stmt::If {
            cond, then, els, ..
        } => {
            callees_expr(cond, out);
            callees_block(then, out);
            if let Some(b) = els {
                callees_block(b, out);
            }
        }
        Stmt::While { cond, body, .. } => {
            callees_expr(cond, out);
            callees_block(body, out);
        }
        Stmt::Loop { body, .. } => callees_block(body, out),
        Stmt::Break { .. } | Stmt::Continue { .. } => {}
        Stmt::IsMatch { arms, .. } => {
            for a in arms {
                callees_block(&a.body, out);
            }
        }
        Stmt::ForIn { iter, body, .. } => {
            callees_expr(iter, out);
            callees_block(body, out);
        }
    }
}

fn callees_expr(e: &Expr, out: &mut Vec<String>) {
    match &e.kind {
        // PLAN-936：Hash 内的被调方是本需求的调用闭包成员。
        ExprKind::HashExpr { expr: inner } => callees_expr(inner, out),
        ExprKind::Call { callee, args } => {
            out.push(callee.name.clone());
            for a in args {
                if let Arg::Pos(x) | Arg::MovePos(x) = a {
                    callees_expr(x, out);
                }
            }
        }
        ExprKind::Method { args, .. } | ExprKind::MethodCall { args, .. } => {
            for a in args {
                if let Arg::Pos(x) | Arg::MovePos(x) = a {
                    callees_expr(x, out);
                }
            }
        }
        ExprKind::CallValue { callee, args } => {
            callees_expr(callee, out);
            for a in args {
                if let Arg::Pos(x) | Arg::MovePos(x) = a {
                    callees_expr(x, out);
                }
            }
        }
        ExprKind::Binary { lhs, rhs, .. } => {
            callees_expr(lhs, out);
            callees_expr(rhs, out);
        }
        ExprKind::Unary { operand, .. } => callees_expr(operand, out),
        ExprKind::Field { base, .. } => callees_expr(base, out),
        _ => {}
    }
}

// -- meta selectors --------------------------------------------------------

/// One meta selector occurrence inside a hash expression.
pub struct MetaSelector {
    pub op: &'static str,
    pub type_name: String,
    pub field: Option<String>,
    pub span: Span,
}

/// The meta selector calls inside one expression (recognized by the
/// protected names; the resolver re-validates the full contract at bind).
pub fn meta_selectors_in(e: &Expr) -> Vec<MetaSelector> {
    let mut out = Vec::new();
    walk_meta(e, &mut out);
    out
}

fn walk_meta(e: &Expr, out: &mut Vec<MetaSelector>) {
    if let ExprKind::Call { callee, args } = &e.kind {
        if let Some(op) = ace_core::source::meta_intrinsic(&callee.name) {
            let type_name = args.first().and_then(|a| match a {
                Arg::Pos(x) | Arg::MovePos(x) => match &x.kind {
                    ExprKind::Var(id) => Some(id.name.clone()),
                    _ => None,
                },
                _ => None,
            });
            let field = args.get(1).and_then(|a| match a {
                Arg::Pos(x) | Arg::MovePos(x) => match &x.kind {
                    ExprKind::StrLit(s) => Some(s.clone()),
                    _ => None,
                },
                _ => None,
            });
            if let Some(type_name) = type_name {
                out.push(MetaSelector {
                    op,
                    type_name,
                    field,
                    span: e.span,
                });
            }
        }
        for a in args {
            if let Arg::Pos(x) | Arg::MovePos(x) = a {
                walk_meta(x, out);
            }
        }
    }
    match &e.kind {
        ExprKind::Method { args, .. } | ExprKind::MethodCall { args, .. } => {
            for a in args {
                if let Arg::Pos(x) | Arg::MovePos(x) = a {
                    walk_meta(x, out);
                }
            }
        }
        ExprKind::CallValue { callee, args } => {
            walk_meta(callee, out);
            for a in args {
                if let Arg::Pos(x) | Arg::MovePos(x) = a {
                    walk_meta(x, out);
                }
            }
        }
        ExprKind::Binary { lhs, rhs, .. } => {
            walk_meta(lhs, out);
            walk_meta(rhs, out);
        }
        ExprKind::Unary { operand, .. } => walk_meta(operand, out),
        _ => {}
    }
}

fn record_of<'a>(program: &'a Program, name: &str) -> Option<&'a ace_core::source::ast::TypeItem> {
    program.items.iter().find_map(|i| match i {
        Item::Type(t) if t.name.name == name => Some(t),
        _ => None,
    })
}

// -- substitution & shape helpers ------------------------------------------

/// Substitute nested-hash values and meta values with literals (pure AST
/// substitution — no text is formatted or re-parsed, §2).
fn substitute(e: &Expr, hash_vals: &ServedRoot, meta_vals: &BTreeMap<(u64, u64), CtValue>) -> Expr {
    if let Some(v) = meta_vals.get(&(e.span.start as u64, e.span.end as u64)) {
        return Expr {
            span: e.span,
            kind: lit_of(*v),
        };
    }
    if let ExprKind::HashExpr { .. } = e.kind {
        if let Some(v) = hash_vals.value_at(e.span) {
            return Expr {
                span: e.span,
                kind: lit_of(v),
            };
        }
    }
    // rebuild children
    let kind = match &e.kind {
        ExprKind::HashExpr { expr: inner } => ExprKind::HashExpr {
            expr: Box::new(substitute(inner, hash_vals, meta_vals)),
        },
        ExprKind::Call { callee, args } => ExprKind::Call {
            callee: callee.clone(),
            args: args
                .iter()
                .map(|a| match a {
                    Arg::Pos(x) => Arg::Pos(substitute(x, hash_vals, meta_vals)),
                    other => other.clone(),
                })
                .collect(),
        },
        ExprKind::Binary { op, lhs, rhs } => ExprKind::Binary {
            op: *op,
            lhs: Box::new(substitute(lhs, hash_vals, meta_vals)),
            rhs: Box::new(substitute(rhs, hash_vals, meta_vals)),
        },
        ExprKind::Unary { op, operand } => ExprKind::Unary {
            op: *op,
            operand: Box::new(substitute(operand, hash_vals, meta_vals)),
        },
        other => other.clone(),
    };
    Expr { span: e.span, kind }
}

/// The per-site compile program: every served hash site replaced by its
/// frozen literal, and one chosen site replaced by an arbitrary
/// expression (its substituted inner). Pure structural substitution.
fn substitute_program_in_place(p: &mut Program, served: &ServedRoot, site: Span, site_expr: &Expr) {
    for item in &mut p.items {
        if let Item::Fn(f) = item {
            let mut stmts = std::mem::take(&mut f.body.stmts);
            for st in stmts.iter_mut() {
                substitute_stmt(st, served, site, site_expr);
            }
            f.body.stmts = stmts;
        }
    }
}

fn replace_at(e: &mut Expr, served: &ServedRoot, site: Span, site_expr: &Expr) {
    if let ExprKind::HashExpr { .. } = e.kind {
        if e.span.start == site.start && e.span.end == site.end {
            *e = site_expr.clone();
            return;
        }
        if let Some(v) = served.value_at(e.span) {
            e.kind = lit_of(v);
            return;
        }
    }
    match &mut e.kind {
        ExprKind::HashExpr { expr: inner } => replace_at(inner, served, site, site_expr),
        ExprKind::Call { args, .. }
        | ExprKind::Method { args, .. }
        | ExprKind::MethodCall { args, .. } => {
            for a in args.iter_mut() {
                match a {
                    Arg::Pos(x) | Arg::MovePos(x) => replace_at(x, served, site, site_expr),
                    _ => {}
                }
            }
        }
        ExprKind::CallValue { callee, args } => {
            replace_at(callee, served, site, site_expr);
            for a in args.iter_mut() {
                match a {
                    Arg::Pos(x) | Arg::MovePos(x) => replace_at(x, served, site, site_expr),
                    _ => {}
                }
            }
        }
        ExprKind::Binary { lhs, rhs, .. } => {
            replace_at(lhs, served, site, site_expr);
            replace_at(rhs, served, site, site_expr);
        }
        ExprKind::Unary { operand, .. } => replace_at(operand, served, site, site_expr),
        ExprKind::Field { base, .. } => replace_at(base, served, site, site_expr),
        _ => {}
    }
}

fn substitute_stmt(s: &mut Stmt, served: &ServedRoot, site: Span, site_expr: &Expr) {
    match s {
        Stmt::Return { value, .. }
        | Stmt::ExprStmt { value, .. }
        | Stmt::Let { value, .. }
        | Stmt::Assign { value, .. } => replace_at(value, served, site, site_expr),
        Stmt::If {
            cond, then, els, ..
        } => {
            replace_at(cond, served, site, site_expr);
            for st in then.stmts.iter_mut() {
                substitute_stmt(st, served, site, site_expr);
            }
            if let Some(b) = els {
                for st in b.stmts.iter_mut() {
                    substitute_stmt(st, served, site, site_expr);
                }
            }
        }
        Stmt::While { cond, body, .. } => {
            replace_at(cond, served, site, site_expr);
            for st in body.stmts.iter_mut() {
                substitute_stmt(st, served, site, site_expr);
            }
        }
        Stmt::Loop { body, .. } => {
            for st in body.stmts.iter_mut() {
                substitute_stmt(st, served, site, site_expr);
            }
        }
        Stmt::Break { .. } | Stmt::Continue { .. } => {}
        Stmt::IsMatch { arms, .. } => {
            for a in arms.iter_mut() {
                for st in a.body.stmts.iter_mut() {
                    substitute_stmt(st, served, site, site_expr);
                }
            }
        }
        Stmt::ForIn { iter, body, .. } => {
            replace_at(iter, served, site, site_expr);
            for st in body.stmts.iter_mut() {
                substitute_stmt(st, served, site, site_expr);
            }
        }
    }
}

fn lit_of(v: CtValue) -> ExprKind {
    match v {
        CtValue::I32(x) => ExprKind::IntLit(x),
        CtValue::Bool(b) => ExprKind::BoolLit(b),
    }
}

/// The direct call form of an expression (`callee(literal-args)`), if the
/// substituted expression is exactly that (T-02's key shape).
fn direct_call_callee(e: &Expr) -> Option<String> {
    match &e.kind {
        ExprKind::Call { callee, .. } => Some(callee.name.clone()),
        _ => None,
    }
}

fn direct_call_args(e: &Expr) -> Option<Vec<CtValue>> {
    match &e.kind {
        ExprKind::Call { args, .. } => {
            let mut out = Vec::with_capacity(args.len());
            for a in args {
                match a {
                    Arg::Pos(x) => out.push(lit_value(x)?),
                    _ => return None,
                }
            }
            Some(out)
        }
        _ => None,
    }
}

fn lit_value(e: &Expr) -> Option<CtValue> {
    match &e.kind {
        ExprKind::IntLit(v) => Some(CtValue::I32(*v)),
        ExprKind::BoolLit(v) => Some(CtValue::Bool(*v)),
        _ => None,
    }
}

/// Structural expectation of the (substituted) expression: the direct
/// call's declared return class, else literal classes, else binary
/// operator class (arithmetic/comparison per the scalar rules).
pub fn expect_of(e: &Expr) -> ExpectTy {
    expect_of_in(None, e)
}

/// The program-aware form: a direct call's expectation is its callee's
/// DECLARED return class (the executor re-checks; this is the request's
/// admission hint).
pub fn expect_of_in(program: Option<&Program>, e: &Expr) -> ExpectTy {
    if let Some(p) = program {
        if let ExprKind::Call { callee, .. } = &e.kind {
            if let Some(Item::Fn(f)) = p
                .items
                .iter()
                .find(|i| matches!(i, Item::Fn(f) if f.name.name == callee.name))
            {
                return match &f.ret.kind {
                    ace_core::source::ast::TypeRef::Builtin(
                        ace_core::source::ast::TypeWord::Bool,
                    ) => ExpectTy::Bool,
                    _ => ExpectTy::I32,
                };
            }
        }
    }
    match &e.kind {
        ExprKind::BoolLit(_) => ExpectTy::Bool,
        ExprKind::Binary {
            op:
                ace_core::source::ast::BinOp::Lt
                | ace_core::source::ast::BinOp::Le
                | ace_core::source::ast::BinOp::Gt
                | ace_core::source::ast::BinOp::Ge
                | ace_core::source::ast::BinOp::Eq
                | ace_core::source::ast::BinOp::Ne
                | ace_core::source::ast::BinOp::And
                | ace_core::source::ast::BinOp::Or,
            ..
        } => ExpectTy::Bool,
        // Call form: the service/executor re-checks the declared class
        // (typed admission); default i32 is only the planning hint.
        _ => ExpectTy::I32,
    }
}

fn find_hash<'a>(program: &'a Program, owner: &str, span: Span) -> Option<&'a Expr> {
    for item in &program.items {
        if let Item::Fn(f) = item {
            if f.name.name == owner {
                let mut found: Option<&Expr> = None;
                find_hash_block(&f.body, span, &mut found);
                return found;
            }
        }
    }
    None
}

fn find_hash_block<'a>(b: &'a Block, span: Span, found: &mut Option<&'a Expr>) {
    for s in &b.stmts {
        find_hash_stmt(s, span, found);
    }
}

fn find_hash_stmt<'a>(s: &'a Stmt, span: Span, found: &mut Option<&'a Expr>) {
    if found.is_some() {
        return;
    }
    match s {
        Stmt::Return { value, .. } | Stmt::ExprStmt { value, .. } | Stmt::Let { value, .. } => {
            find_hash_expr(value, span, found)
        }
        Stmt::Assign { value, .. } => find_hash_expr(value, span, found),
        Stmt::If {
            cond, then, els, ..
        } => {
            find_hash_expr(cond, span, found);
            find_hash_block(then, span, found);
            if let Some(b) = els {
                find_hash_block(b, span, found);
            }
        }
        Stmt::While { cond, body, .. } => {
            find_hash_expr(cond, span, found);
            find_hash_block(body, span, found);
        }
        Stmt::Loop { body, .. } => find_hash_block(body, span, found),
        Stmt::Break { .. } | Stmt::Continue { .. } => {}
        Stmt::IsMatch { arms, .. } => {
            for a in arms {
                find_hash_block(&a.body, span, found);
            }
        }
        Stmt::ForIn { iter, body, .. } => {
            find_hash_expr(iter, span, found);
            find_hash_block(body, span, found);
        }
    }
}

fn find_hash_expr<'a>(e: &'a Expr, span: Span, found: &mut Option<&'a Expr>) {
    if found.is_some() {
        return;
    }
    if e.span.start as u64 == span.start as u64 && e.span.end as u64 == span.end as u64 {
        *found = Some(e);
        return;
    }
    match &e.kind {
        ExprKind::HashExpr { expr: inner } => find_hash_expr(inner, span, found),
        ExprKind::Call { args, .. }
        | ExprKind::Method { args, .. }
        | ExprKind::MethodCall { args, .. } => {
            for a in args {
                if let Arg::Pos(x) | Arg::MovePos(x) = a {
                    find_hash_expr(x, span, found);
                }
            }
        }
        ExprKind::CallValue { callee, args } => {
            find_hash_expr(callee, span, found);
            for a in args {
                if let Arg::Pos(x) | Arg::MovePos(x) = a {
                    find_hash_expr(x, span, found);
                }
            }
        }
        ExprKind::Binary { lhs, rhs, .. } => {
            find_hash_expr(lhs, span, found);
            find_hash_expr(rhs, span, found);
        }
        ExprKind::Unary { operand, .. } => find_hash_expr(operand, span, found),
        ExprKind::Field { base, .. } => find_hash_expr(base, span, found),
        _ => {}
    }
}

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
use crate::walk::{read, write};
use ace_core::atom_text::Span;
use ace_core::source::ast::{Arg, Block, Expr, ExprKind, FnItem, Item, Program, Receiver, Stmt};
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
    Cycle { chain: Vec<String>, span: Span },
    /// The root budget ran out (resource state; nothing published).
    Deadline { elapsed_ms: u128, span: Span },
    /// A demand failed (executor/admission); forwarded with its site.
    Demand {
        owner: String,
        err: CtExecError,
        span: Span,
    },
}

impl std::fmt::Display for PlanError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PlanError::Cycle { chain, .. } => write!(
                f,
                "cte.query-cycle: {} (these functions need each other's compile-time values)",
                chain.join(" -> ")
            ),
            PlanError::Deadline { elapsed_ms, .. } => write!(
                f,
                "cte.resource.deadline: the root budget was exhausted after {elapsed_ms}ms (resource state; nothing published)"
            ),
            PlanError::Demand { owner, err, .. } => {
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
            span,
        });
    }
    // Cycle: the SAME site (owner + span) is already being served on the
    // chain (a nested site of the same owner is a different node — it is
    // part of the enclosing site's plan, not a loop).
    let node = (owner.to_string(), (span.start as u64, span.end as u64));
    if let Some(hit) = stack.iter().position(|o| *o == node) {
        let mut chain: Vec<String> = stack[hit..].iter().map(|(o, _)| o.clone()).collect();
        chain.push(owner.to_string());
        return Err(PlanError::Cycle { chain, span });
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
            span,
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
            span,
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
    let keep = if svc.is_auto() {
        let mut direct = Vec::new();
        callees_in(read::Node::Value(inner), &mut direct);
        transitive_callees(program, direct, Vec::new())
    } else {
        closure_of(program, owner)
    };
    for callee in &keep {
        for (closure_owner, closure_span) in hashes_in_body_of(program, callee) {
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
                span,
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
    let mut substituted = substitute(inner, out, &meta_vals);
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
    let mut site_program = Program {
        items: program
            .items
            .iter()
            .filter(|i| match i {
                Item::Fn(f) => keep.contains(&f.name.name),
                Item::Const(c) => svc.is_auto() && keep.contains(&c.name.name),
                Item::Enum(_) => svc.is_auto(),
                Item::Type(_) => true,
                _ => false,
            })
            .cloned()
            .collect(),
    };
    substitute_program_in_place(&mut site_program, out, Some((span, &substituted)));
    if svc.is_auto() {
        prepare_readonly_globals(&mut site_program, &mut substituted).map_err(|err| {
            PlanError::Demand {
                owner: owner.to_string(),
                span,
                err,
            }
        })?;
    }
    // A substituted LITERAL (pure-literal or meta-only demand) IS the
    // value — plan-level, never an AV execution (§5.2 literal+Meta 解析).
    if let Some(v) = lit_value(&substituted) {
        out.values.insert((span.start as u64, span.end as u64), v);
        return Ok(());
    }
    if svc.is_auto()
        && matches!(
            substituted.kind,
            ExprKind::StrLit(_)
                | ExprKind::CharLit(_)
                | ExprKind::ArrayLit(_)
                | ExprKind::TupleLit(_)
                | ExprKind::ObjectLit { .. }
                | ExprKind::TypedLit { .. }
        )
    {
        return Err(PlanError::Demand {
            owner: owner.to_string(),
            span,
            err: CtExecError {
                code: "cte.type/value-domain".into(),
                message: "a frozen CT value must be int or bool".into(),
                av_reached: false,
            },
        });
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
                span,
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
            span,
            err: e,
        })?
    };
    out.values
        .insert((span.start as u64, span.end as u64), ans.value);
    Ok(())
}

// -- discovery walks ------------------------------------------------------

/// The value expression of one argument in ANY source position — `Pos`,
/// `MovePos` and `Named` carry the same `Arg.value` subtree, so every CT
/// walk (discovery, callee closure, meta selectors, substitution, site
/// lookup and product backfill) must traverse exactly these values (F26).
fn arg_value(a: &Arg) -> &Expr {
    match a {
        Arg::Pos(x) | Arg::MovePos(x) => x,
        Arg::Named { value, .. } => value,
    }
}

/// Discover the outer demands under every source root.
pub fn all_hash_sites(program: &Program) -> Vec<(String, Span)> {
    let mut sites = Vec::new();
    read::program_roots(program, |owner, root| {
        collect_hash_sites(root, owner, &mut sites)
    });
    sites
}

fn collect_hash_sites(root: read::Node<'_>, owner: &str, out: &mut Vec<(String, Span)>) {
    read::visit(root, |value| {
        if matches!(value.kind, ExprKind::HashExpr { .. }) {
            out.push((owner.to_string(), value.span));
            false // Nested hashes belong to this demand.
        } else {
            true
        }
    });
}

fn hashes_in_expr(expr: &Expr, owner: &str) -> Vec<(String, Span)> {
    let mut sites = Vec::new();
    collect_hash_sites(read::Node::Value(expr), owner, &mut sites);
    sites
}

fn hashes_in_body_of(program: &Program, name: &str) -> Vec<(String, Span)> {
    let mut sites = Vec::new();
    read::program_roots(program, |owner, root| {
        if owner == name {
            collect_hash_sites(root, owner, &mut sites);
        }
    });
    sites
}

/// The direct free-call closure of one fn (names; resolver owns binding).
fn closure_of(program: &Program, root: &str) -> Vec<String> {
    let mut result = transitive_callees(
        program,
        direct_callees_of(program, root),
        vec![root.to_string()],
    );
    result.remove(0);
    result
}

fn transitive_callees(
    program: &Program,
    mut stack: Vec<String>,
    mut seen: Vec<String>,
) -> Vec<String> {
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
    seen
}

fn direct_callees_of(program: &Program, name: &str) -> Vec<String> {
    let mut callees = Vec::new();
    read::program_roots(program, |owner, root| {
        if owner == name {
            callees_in(root, &mut callees);
        }
    });
    callees
}

fn callees_in(root: read::Node<'_>, out: &mut Vec<String>) {
    read::visit(root, |value| {
        if let ExprKind::Call { callee, .. } | ExprKind::ConstCall { callee, .. } = &value.kind {
            let name = callee
                .name
                .strip_prefix(ace_core::source::packages::GLOBAL_GET_PREFIX)
                .unwrap_or(&callee.name);
            out.push(name.to_string());
        }
        true
    });
}

// A package constant's checked initializer can run inside a CT demand.
// Its accessor is a pure function of the other immutable dependencies;
// this gives the CT closure no access to the runtime package's globals.
fn prepare_readonly_globals(program: &mut Program, inner: &mut Expr) -> Result<(), CtExecError> {
    use ace_core::source::packages::{GLOBAL_GET_PREFIX, GLOBAL_SET_PREFIX, GLOBAL_TAKE_PREFIX};
    let names: std::collections::BTreeSet<_> = program
        .items
        .iter()
        .filter_map(|item| {
            if let Item::Const(constant) = item {
                Some(constant.name.name.clone())
            } else {
                None
            }
        })
        .collect();
    let mut illegal = None;
    let mut rewrite = |expr: &mut Expr| {
        if let ExprKind::Call { callee, .. } = &mut expr.kind {
            if let Some(name) = callee.name.strip_prefix(GLOBAL_GET_PREFIX) {
                if names.contains(name) {
                    callee.name = format!("__ace_ct_readonly_{name}");
                } else {
                    illegal.get_or_insert_with(|| callee.name.clone());
                }
            } else if callee.name.starts_with(GLOBAL_SET_PREFIX)
                || callee.name.starts_with(GLOBAL_TAKE_PREFIX)
            {
                illegal.get_or_insert_with(|| callee.name.clone());
            }
        }
        true
    };
    write::visit(write::Node::Value(inner), &mut rewrite);
    write::program_roots(program, |_, root| write::visit(root, &mut rewrite));
    if let Some(name) = illegal {
        return Err(CtExecError {
            code: "cte.capability/denied".into(),
            message: format!("`{name}` would access a mutable runtime global during CT evaluation"),
            av_reached: false,
        });
    }
    for item in &mut program.items {
        if let Item::Const(constant) = item {
            let name = ace_core::source::ast::Ident {
                name: format!("__ace_ct_readonly_{}", constant.name.name),
                span: constant.name.span,
            };
            *item = Item::Fn(FnItem {
                name,
                type_params: Vec::new(),
                params: Vec::new(),
                ret: constant.ty.clone(),
                body: Block {
                    stmts: vec![Stmt::Return {
                        value: constant.value.clone(),
                        span: constant.value.span,
                    }],
                    span: constant.span,
                },
                span: constant.span,
                receiver: Receiver::Free,
                owner: None,
                script: false,
                proto: false,
            });
        }
    }
    Ok(())
}

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
            walk_meta(arg_value(a), out);
        }
    }
    match &e.kind {
        ExprKind::Method { args, .. } | ExprKind::MethodCall { args, .. } => {
            for a in args {
                walk_meta(arg_value(a), out);
            }
        }
        ExprKind::CallValue { callee, args } => {
            walk_meta(callee, out);
            for a in args {
                walk_meta(arg_value(a), out);
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
    let mut value = e.clone();
    write::visit(write::Node::Value(&mut value), |expr| {
        let key = (expr.span.start as u64, expr.span.end as u64);
        let frozen = meta_vals.get(&key).copied().or_else(|| {
            if matches!(expr.kind, ExprKind::HashExpr { .. }) {
                hash_vals.value_at(expr.span)
            } else {
                None
            }
        });
        if let Some(frozen) = frozen {
            expr.kind = lit_of(frozen);
            return false;
        }
        true
    });
    value
}

/// Replace the served sites with literals. During preparation for one
/// execution the demand being evaluated can be replaced by its inner value.
pub(crate) fn substitute_program_in_place(
    program: &mut Program,
    served: &ServedRoot,
    executing: Option<(Span, &Expr)>,
) {
    write::program_roots(program, |_, root| {
        write::visit(root, |expr| {
            if matches!(expr.kind, ExprKind::HashExpr { .. }) {
                if let Some((span, replacement)) = executing {
                    if expr.span == span {
                        *expr = replacement.clone();
                        return false;
                    }
                }
                if let Some(value) = served.value_at(expr.span) {
                    expr.kind = lit_of(value);
                    return false;
                }
            }
            true
        });
    });
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
        ExprKind::BoolLit(_)
        | ExprKind::Unary {
            op: ace_core::source::ast::UnaryOp::Not,
            ..
        } => ExpectTy::Bool,
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

fn find_hash<'a>(program: &'a Program, name: &str, span: Span) -> Option<&'a Expr> {
    let mut found = None;
    read::program_roots(program, |owner, root| {
        if owner == name && found.is_none() {
            read::visit(root, |expr| {
                if expr.span == span && matches!(expr.kind, ExprKind::HashExpr { .. }) {
                    found = Some(expr);
                }
                found.is_none()
            });
        }
    });
    found
}

//! Per-function structured preparation summary (plan §2: "每函数共同
//! prepare/需求/依赖摘要"; T-03). Target-neutral: it records WHAT a
//! function needs and depends on — operators, call targets, trace marks,
//! sizes and depth — never how a target renders it.
//!
//! Downstream consumers:
//! * cache keys (T-04): the summary feeds the function-semantic part of the
//!   key (ops/calls/marks are identity inputs, spans are NOT — plan §5.4:
//!   "源码绝对span/snapshot…不改变语义fp");
//! * budgets (§5.3): size/depth counters feed the walk/print budget guards
//!   before any target recursion starts;
//! * fragment closure assembly (T-04): `calls` is the per-function edge.

use crate::semantic_view::{FnFacts, SemanticView};
use crate::A2xReject;
use auto_ac_prototype::hir;
use std::collections::BTreeSet;

/// Frozen v1 walk budget: expressions/blocks deeper than this are a named
/// rejection, not a host stack overflow (plan §5.3: "无固定语言嵌套数排除，
/// 新增walk/print/drop/类型/调用图路径受预算并避免宿主递归栈溢出").
pub const MAX_STRUCTURE_DEPTH: usize = 400;

/// Target-neutral preparation summary of ONE function.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FunctionPlan {
    pub source_name: String,
    pub def_index: usize,
    /// (start, end) byte span of the def in the compiled source.
    pub origin: (u32, u32),
    /// Distinct binary/unary/logic operator tags used, in canonical order.
    pub ops: BTreeSet<&'static str>,
    /// Def indices of called functions (internal + external signatures).
    pub calls: BTreeSet<usize>,
    /// Authorized trace symbols used.
    pub marks: BTreeSet<&'static str>,
    /// Numbers of structural nodes (budget/observation inputs).
    pub expr_count: usize,
    pub stmt_count: usize,
    pub block_count: usize,
    pub local_count: usize,
    /// Deepest block/expr nesting (1-based).
    pub max_depth: usize,
}

/// Prepare one function's summary from a view. The walk is ITERATIVE (an
/// explicit work stack) — no host recursion on adversarial shapes; the
/// depth guard fires as a named `a2x.too-deep` rejection.
pub fn plan_function(view: &SemanticView<'_>, f: &FnFacts<'_>) -> Result<FunctionPlan, A2xReject> {
    let mut ops: BTreeSet<&'static str> = BTreeSet::new();
    let mut calls: BTreeSet<usize> = BTreeSet::new();
    let mut marks: BTreeSet<&'static str> = BTreeSet::new();
    let mut expr_count = 0usize;
    let mut stmt_count = 0usize;
    let mut max_depth = 0usize;

    // Work items: (block id, depth). Statements and expressions are counted
    // level by level; short-circuit/call operands push their blocks/args as
    // new work at depth + 1.
    let mut work: Vec<(hir::BlockId, usize)> = vec![(f.body.entry, 1)];
    while let Some((block_id, depth)) = work.pop() {
        if depth > MAX_STRUCTURE_DEPTH {
            return Err(A2xReject::new(
                "a2x.too-deep",
                format!(
                    "function `{}` nests deeper than the frozen walk budget {}",
                    f.source_name, MAX_STRUCTURE_DEPTH
                ),
            ));
        }
        max_depth = max_depth.max(depth);
        let block = f
            .body
            .blocks
            .get(block_id.0 as usize)
            .ok_or_else(|| A2xReject::new("a2x.internal", "block index out of range"))?;
        for stmt in &block.stmts {
            stmt_count += 1;
            match stmt {
                hir::Stmt::If { cond, then, els } => {
                    collect_expr(
                        view,
                        f,
                        *cond,
                        depth,
                        &mut ops,
                        &mut calls,
                        &mut marks,
                        &mut expr_count,
                        &mut max_depth,
                    )?;
                    work.push((*then, depth + 1));
                    if let Some(e) = els {
                        work.push((*e, depth + 1));
                    }
                }
                hir::Stmt::Loop { body, .. } => {
                    work.push((*body, depth + 1));
                }
                hir::Stmt::Let { value, .. }
                | hir::Stmt::Return { value }
                | hir::Stmt::Assign { value, .. } => {
                    collect_expr(
                        view,
                        f,
                        *value,
                        depth,
                        &mut ops,
                        &mut calls,
                        &mut marks,
                        &mut expr_count,
                        &mut max_depth,
                    )?;
                }
                hir::Stmt::Break { .. } | hir::Stmt::Continue { .. } => {}
                _ => {
                    return Err(A2xReject::unsupported_stmt(format!(
                        "in `{}`: statement outside the compute surface",
                        f.source_name
                    )))
                }
            }
        }
    }

    Ok(FunctionPlan {
        source_name: f.source_name.to_string(),
        def_index: f.def_index,
        origin: f.origin,
        ops,
        calls,
        marks,
        expr_count,
        stmt_count,
        block_count: f.body.blocks.len(),
        local_count: f.body.locals.len(),
        max_depth,
    })
}

/// Collect one expression subtree iteratively: operand children become work
/// items at depth + 1 (children of a call/logical push through `work`).
#[allow(clippy::too_many_arguments)]
fn collect_expr(
    view: &SemanticView<'_>,
    f: &FnFacts<'_>,
    root: hir::ExprId,
    depth: usize,
    ops: &mut BTreeSet<&'static str>,
    calls: &mut BTreeSet<usize>,
    marks: &mut BTreeSet<&'static str>,
    expr_count: &mut usize,
    max_depth: &mut usize,
) -> Result<(), A2xReject> {
    // (expr id, depth)
    let mut stack: Vec<(hir::ExprId, usize)> = vec![(root, depth)];
    while let Some((id, d)) = stack.pop() {
        if d > MAX_STRUCTURE_DEPTH {
            return Err(A2xReject::new(
                "a2x.too-deep",
                format!(
                    "function `{}` nests expressions deeper than the frozen walk budget {}",
                    f.source_name, MAX_STRUCTURE_DEPTH
                ),
            ));
        }
        *max_depth = (*max_depth).max(d);
        *expr_count += 1;
        let e = f
            .body
            .exprs
            .get(id.0 as usize)
            .ok_or_else(|| A2xReject::new("a2x.internal", "expr index out of range"))?;
        match &e.kind {
            hir::ExprKind::ReadLocal { .. } | hir::ExprKind::Constant { .. } => {}
            hir::ExprKind::Binary { op, lhs, rhs, .. } => {
                ops.insert(op.as_str());
                stack.push((*lhs, d + 1));
                stack.push((*rhs, d + 1));
            }
            hir::ExprKind::Unary { op, operand } => {
                ops.insert(op.as_str());
                stack.push((*operand, d + 1));
            }
            hir::ExprKind::Logical { op, lhs, rhs } => {
                ops.insert(op.as_str());
                stack.push((*lhs, d + 1));
                stack.push((*rhs, d + 1));
            }
            hir::ExprKind::Call {
                callee, eval_args, ..
            } => {
                let target = callee.0 as usize;
                if let Some(sym) = view.trace_symbol(target) {
                    marks.insert(sym);
                } else {
                    calls.insert(target);
                }
                for a in eval_args {
                    stack.push((*a, d + 1));
                }
            }
            other => {
                return Err(A2xReject::unsupported_expr(format!(
                    "in `{}`: expression `{:?}` outside the compute surface",
                    f.source_name, other
                )))
            }
        }
    }
    Ok(())
}

impl<'a> SemanticView<'a> {
    /// Prepare summaries for every admitted function.
    pub fn plan_all(&self) -> Result<Vec<FunctionPlan>, A2xReject> {
        self.functions()
            .iter()
            .map(|f| plan_function(self, f))
            .collect()
    }
}

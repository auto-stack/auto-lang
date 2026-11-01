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

use crate::semantic_view::{FnFacts, FnKind, SemanticView, Ty};
use crate::A2xReject;
use ace_core::hir;
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
            // ---- PLAN-902 record/text surface (same op tags feed the cache
            // key; new tags never appear for scalar functions) ----
            hir::ExprKind::StrLit { .. } => {
                ops.insert("str_lit");
            }
            hir::ExprKind::CharLit { .. } => {
                ops.insert("char_lit");
            }
            hir::ExprKind::TextLen { operand }
            | hir::ExprKind::TextLenBytes { operand }
            | hir::ExprKind::TextClone { operand }
            | hir::ExprKind::TextToOwned { operand }
            | hir::ExprKind::TextToBorrowed { operand }
            | hir::ExprKind::CharToCodepoint { operand }
            | hir::ExprKind::CharFromCodepoint { operand } => {
                let tag = match &e.kind {
                    hir::ExprKind::TextLen { .. } => "text_len",
                    hir::ExprKind::TextLenBytes { .. } => "text_len_bytes",
                    hir::ExprKind::TextClone { .. } => "text_clone",
                    hir::ExprKind::TextToOwned { .. } => "text_to_owned",
                    hir::ExprKind::TextToBorrowed { .. } => "text_to_borrowed",
                    hir::ExprKind::CharToCodepoint { .. } => "char_to_codepoint",
                    _ => "char_from_codepoint",
                };
                ops.insert(tag);
                stack.push((*operand, d + 1));
            }
            hir::ExprKind::TextCharAt { operand, index } => {
                ops.insert("text_char_at");
                stack.push((*operand, d + 1));
                stack.push((*index, d + 1));
            }
            hir::ExprKind::TextSlice {
                operand,
                start,
                end,
            } => {
                ops.insert("text_slice");
                stack.push((*operand, d + 1));
                stack.push((*start, d + 1));
                stack.push((*end, d + 1));
            }
            hir::ExprKind::TextConcat { lhs, rhs } => {
                ops.insert("text_concat");
                stack.push((*lhs, d + 1));
                stack.push((*rhs, d + 1));
            }
            hir::ExprKind::RecordCtor { eval_args, .. } => {
                ops.insert("record_ctor");
                for a in eval_args {
                    stack.push((*a, d + 1));
                }
            }
            hir::ExprKind::FieldGet { base, .. } => {
                ops.insert("field_get");
                stack.push((*base, d + 1));
            }
            hir::ExprKind::BorrowView { base } => {
                ops.insert("borrow_view");
                stack.push((*base, d + 1));
            }
            hir::ExprKind::BorrowMut { .. } => {
                ops.insert("borrow_mut");
            }
            hir::ExprKind::RecordClone { base, .. } => {
                ops.insert("record_clone");
                stack.push((*base, d + 1));
            }
            hir::ExprKind::Move { inner } => {
                ops.insert("move");
                stack.push((*inner, d + 1));
            }
            // ---- PLAN-904 collections surface (new tags never appear for
            // scalar/902 functions) ----
            hir::ExprKind::ListNew { .. } => {
                ops.insert("list_new");
            }
            hir::ExprKind::ListLen { list } => {
                ops.insert("list_len");
                stack.push((*list, d + 1));
            }
            hir::ExprKind::ListPush { list, value } => {
                ops.insert("list_push");
                stack.push((*list, d + 1));
                stack.push((*value, d + 1));
            }
            hir::ExprKind::ListInsert { list, index, value } => {
                ops.insert("list_insert");
                stack.push((*list, d + 1));
                stack.push((*index, d + 1));
                stack.push((*value, d + 1));
            }
            hir::ExprKind::ListGet { list, index } => {
                ops.insert("list_get");
                stack.push((*list, d + 1));
                stack.push((*index, d + 1));
            }
            hir::ExprKind::ListSet { list, index, value } => {
                ops.insert("list_set");
                stack.push((*list, d + 1));
                stack.push((*index, d + 1));
                stack.push((*value, d + 1));
            }
            hir::ExprKind::ListRemove { list, index } => {
                ops.insert("list_remove");
                stack.push((*list, d + 1));
                stack.push((*index, d + 1));
            }
            hir::ExprKind::ListClear { list } => {
                ops.insert("list_clear");
                stack.push((*list, d + 1));
            }
            hir::ExprKind::VariantCtor { payload, .. } => {
                ops.insert("variant_ctor");
                if let Some(p) = payload {
                    stack.push((*p, d + 1));
                }
            }
            hir::ExprKind::VariantGuardTest { root, .. } => {
                ops.insert("variant_guard");
                stack.push((*root, d + 1));
            }
            hir::ExprKind::VariantProject { root, .. } => {
                ops.insert("variant_project");
                stack.push((*root, d + 1));
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

// ---------------------------------------------------------------------------
// Semantic fingerprint (plan §5.4 key inputs)
// ---------------------------------------------------------------------------

/// Canonical signature text: param types in positional order + result.
/// Names/spans excluded on purpose — they are not semantic identity.
pub fn canonical_signature(f: &FnFacts<'_>) -> String {
    let params: Vec<&str> = f.params.iter().map(|p| p.ty.as_str()).collect();
    format!("({})->{}", params.join(","), f.result.as_str())
}

/// The SEMANTIC fingerprint of one function body: sha256 over a canonical
/// serialization of exactly what the compute surface means — types, ops,
/// structure, evaluation order, call bindings, trace marks — with spans,
/// table ordering and local/param NAMES excluded (plan §5.4). Local
/// identity is canonicalized by first appearance, so a reordered locals
/// table produces the same bytes.
pub fn semantic_fingerprint(view: &SemanticView<'_>, f: &FnFacts<'_>) -> Result<String, A2xReject> {
    let mut ser = String::new();
    // PLAN-902: the view-aware canonical signature carries param modes and
    // nominal names; for scalar functions it renders byte-identically to
    // the 892 `canonical_signature` (all-view params, builtin types). On
    // the 902 domains the kind marker (free vs owner-qualified method)
    // leads the line — a same-named method and free function are different
    // semantic objects.
    if is_902_family(view) {
        let kind = match f.kind {
            FnKind::Free => "fn",
            FnKind::Method { .. } => "mt",
        };
        ser.push_str(&format!("sig {kind} {}\n", view.canonical_signature(f)));
    } else {
        ser.push_str(&format!("sig {}\n", view.canonical_signature(f)));
    }
    let mut canon: std::collections::BTreeMap<u32, usize> = std::collections::BTreeMap::new();
    // Params intern FIRST in positional order (0..n-1): param identity is
    // semantic (callers bind arguments positionally); a first-use-only
    // interning would erase the distinction between reading param0 and
    // param1 first.
    let mut param_locals: Vec<(u32, u32)> = f
        .body
        .locals
        .iter()
        .enumerate()
        .filter(|(_, l)| matches!(l.role, hir::LocalRole::Param))
        .map(|(i, l)| (l.index.unwrap_or(u32::MAX), i as u32))
        .collect();
    param_locals.sort();
    for (positional, (_, local_idx)) in param_locals.iter().enumerate() {
        canon.insert(*local_idx, positional);
    }
    let mut next = f.params.len();
    let mut work: Vec<(hir::BlockId, usize)> = vec![(f.body.entry, 0)];
    while let Some((block_id, d)) = work.pop() {
        if d > MAX_STRUCTURE_DEPTH {
            return Err(A2xReject::new(
                "a2x.too-deep",
                format!(
                    "function `{}` nests past the frozen walk budget",
                    f.source_name
                ),
            ));
        }
        let block = f
            .body
            .blocks
            .get(block_id.0 as usize)
            .ok_or_else(|| A2xReject::new("a2x.internal", "block index out of range"))?;
        for stmt in &block.stmts {
            match stmt {
                hir::Stmt::Let { local, value } => {
                    let id = intern(local.0, &mut canon, &mut next);
                    let ty = view.resolve_use_ty(f.body.locals[local.0 as usize].ty)?;
                    ser.push_str(&format!("let ${}:{} =", id, view.ty_name(ty)));
                    ser_expr(view, f, *value, &mut canon, &mut next, &mut ser, 0)?;
                    ser.push(';');
                }
                hir::Stmt::Return { value } => {
                    ser.push_str("ret:");
                    ser_expr(view, f, *value, &mut canon, &mut next, &mut ser, 0)?;
                    ser.push(';');
                }
                hir::Stmt::If { cond, then, els } => {
                    ser.push_str("if:");
                    ser_expr(view, f, *cond, &mut canon, &mut next, &mut ser, 0)?;
                    ser.push('{');
                    work.push((*then, d + 1));
                    if let Some(e) = els {
                        ser.push_str("else{");
                        work.push((*e, d + 1));
                    }
                    ser.push('}');
                }
                hir::Stmt::Loop { body, .. } => {
                    ser.push_str("loop{");
                    work.push((*body, d + 1));
                    ser.push('}');
                }
                hir::Stmt::Assign { place, value } => {
                    let place = f.body.places.get(place.0 as usize).ok_or_else(|| {
                        A2xReject::new("a2x.internal", "place index out of range")
                    })?;
                    let id = intern(place.local.0, &mut canon, &mut next);
                    // PLAN-902: a whole-local target stays `asgn $n:`; a
                    // field-path target serializes the path (field ids are
                    // declaration indices, made semantic by the record
                    // shape digest appended below).
                    match &place.kind {
                        hir::PlaceKind::Local => ser.push_str(&format!("asgn ${}:", id)),
                        hir::PlaceKind::FieldPath(path) => {
                            ser.push_str(&format!("asgn ${}", id));
                            for fid in path {
                                ser.push_str(&format!(".{}", fid.0));
                            }
                            ser.push(':');
                        }
                    }
                    ser_expr(view, f, *value, &mut canon, &mut next, &mut ser, 0)?;
                    ser.push(';');
                }
                hir::Stmt::Break { .. } => ser.push_str("brk;"),
                hir::Stmt::Continue { .. } => ser.push_str("cont;"),
                // PLAN-904: the collections unit-effect statement
                hir::Stmt::Eval { value } => {
                    ser.push_str("eval:");
                    ser_expr(view, f, *value, &mut canon, &mut next, &mut ser, 0)?;
                    ser.push(';');
                }
                _ => return Err(A2xReject::unsupported_stmt("outside the compute surface")),
            }
        }
    }
    // PLAN-902 §5.4: nominal record closures the function touches are part
    // of its semantic identity — a field/type change must invalidate even
    // when the body serialization alone would not show it. Scalar
    // functions reference no records, so their bytes are unchanged.
    for line in record_digest(view, f)? {
        ser.push_str(&line);
        ser.push('\n');
    }
    Ok(crate::sha256::sha256_hex(ser.as_bytes()))
}

/// Whether the view sits on a PLAN-902/PLAN-904 keyed source domain (the
/// fingerprint extensions are gated here so the 892 scalar bytes never
/// move; the collections family joins with its own fresh bytes).
fn is_902_family(view: &SemanticView<'_>) -> bool {
    matches!(
        view.domain,
        hir::HirDomain::Text
            | hir::HirDomain::Record
            | hir::HirDomain::TextRecord
            | hir::HirDomain::Collections
    )
}

/// Sorted nominal shape digest of every record/variant/list type the
/// function's signature, locals or expressions reference (name + ordered
/// fields/arms with nominal type names; lists structurally). Empty for
/// scalar functions. PLAN-904: the closure crosses variant arms and list
/// elements too — an arm/payload/element change invalidates even when the
/// body serialization alone would not show it.
fn record_digest(view: &SemanticView<'_>, f: &FnFacts<'_>) -> Result<Vec<String>, A2xReject> {
    #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
    enum Nominal {
        Rec(u32),
        Var(u32),
        List(u32),
    }
    fn note(refs: &mut Vec<Nominal>, ty: Ty) {
        match ty {
            Ty::Record(id) => refs.push(Nominal::Rec(id)),
            Ty::Variant(id) => refs.push(Nominal::Var(id)),
            Ty::List(id) => refs.push(Nominal::List(id)),
            _ => {}
        }
    }
    let mut refs = Vec::new();
    for p in &f.params {
        note(&mut refs, p.ty);
    }
    note(&mut refs, f.result);
    for l in &f.body.locals {
        note(&mut refs, view.resolve_use_ty(l.ty)?);
    }
    for e in &f.body.exprs {
        match &e.kind {
            hir::ExprKind::RecordCtor { record, .. }
            | hir::ExprKind::RecordClone { record, .. } => refs.push(Nominal::Rec(record.0)),
            hir::ExprKind::VariantCtor { variant, .. } => {
                refs.push(Nominal::Var(variant.0));
                note(&mut refs, view.resolve_use_ty(e.ty)?);
            }
            _ => note(&mut refs, view.resolve_use_ty(e.ty)?),
        }
    }
    // close over nested shapes (transitive identity; iterative walk)
    let mut closed = std::collections::BTreeSet::<Nominal>::new();
    let mut queue: Vec<Nominal> = refs;
    while let Some(n) = queue.pop() {
        if !closed.insert(n) {
            continue;
        }
        match n {
            Nominal::Rec(id) => {
                if let Some(rec) = view.record(id) {
                    for fld in &rec.fields {
                        note(&mut queue, fld.ty);
                    }
                }
            }
            Nominal::Var(id) => {
                if let Some(v) = view.variant(id) {
                    for arm in &v.arms {
                        if let Some(p) = arm.payload {
                            note(&mut queue, p);
                        }
                    }
                }
            }
            Nominal::List(id) => {
                if let Ok(elem) = view.list_elem(id) {
                    note(&mut queue, elem);
                }
            }
        }
    }
    let mut out = Vec::new();
    for n in closed {
        match n {
            Nominal::Rec(id) => {
                if let Some(rec) = view.record(id) {
                    let fields: Vec<String> = rec
                        .fields
                        .iter()
                        .map(|fld| format!("{}:{}", fld.name, view.ty_name(fld.ty)))
                        .collect();
                    out.push(format!("rec {}{{{}}}", rec.name, fields.join(",")));
                }
            }
            Nominal::Var(id) => {
                if let Some(v) = view.variant(id) {
                    let arms: Vec<String> = v
                        .arms
                        .iter()
                        .map(|a| match &a.payload {
                            Some(p) => format!("{}:{}", a.name, view.ty_name(*p)),
                            None => a.name.clone(),
                        })
                        .collect();
                    out.push(format!("var {}{{{}}}", v.name, arms.join(",")));
                }
            }
            Nominal::List(id) => {
                if let Ok(elem) = view.list_elem(id) {
                    out.push(format!("list<{}>", view.ty_name(elem)));
                }
            }
        }
    }
    Ok(out)
}

/// First-appearance interning of a real local index to a canonical id.
fn intern(
    local: u32,
    canon: &mut std::collections::BTreeMap<u32, usize>,
    next: &mut usize,
) -> usize {
    if let Some(&id) = canon.get(&local) {
        return id;
    }
    let id = *next;
    *next += 1;
    canon.insert(local, id);
    id
}

fn ser_expr(
    view: &SemanticView<'_>,
    f: &FnFacts<'_>,
    id: hir::ExprId,
    canon: &mut std::collections::BTreeMap<u32, usize>,
    next: &mut usize,
    out: &mut String,
    depth: usize,
) -> Result<(), A2xReject> {
    if depth > MAX_STRUCTURE_DEPTH {
        return Err(A2xReject::new(
            "a2x.too-deep",
            format!(
                "function `{}` nests expressions past the budget",
                f.source_name
            ),
        ));
    }
    let e = f
        .body
        .exprs
        .get(id.0 as usize)
        .ok_or_else(|| A2xReject::new("a2x.internal", "expr index out of range"))?;
    match &e.kind {
        hir::ExprKind::ReadLocal { local } => {
            let id = intern(local.0, canon, next);
            out.push_str(&format!("${}", id));
        }
        hir::ExprKind::Constant { value } => out.push_str(&format!("c{}", value)),
        hir::ExprKind::Binary { op, lhs, rhs, .. } => {
            out.push_str(&format!("{}(", op.as_str()));
            ser_expr(view, f, *lhs, canon, next, out, depth + 1)?;
            out.push(',');
            ser_expr(view, f, *rhs, canon, next, out, depth + 1)?;
            out.push(')');
        }
        hir::ExprKind::Unary { op, operand } => {
            out.push_str(&format!("{}(", op.as_str()));
            ser_expr(view, f, *operand, canon, next, out, depth + 1)?;
            out.push(')');
        }
        hir::ExprKind::Logical { op, lhs, rhs } => {
            out.push_str(&format!("{}(", op.as_str()));
            ser_expr(view, f, *lhs, canon, next, out, depth + 1)?;
            out.push(',');
            ser_expr(view, f, *rhs, canon, next, out, depth + 1)?;
            out.push(')');
        }
        hir::ExprKind::Call {
            callee,
            eval_args,
            bindings,
        } => {
            let target = callee.0 as usize;
            let mut name = if let Some(sym) = view.trace_symbol(target) {
                sym.to_string()
            } else if let Some(t) = view.fn_by_def(target) {
                t.source_name.to_string()
            } else if let Some(x) = view.external_by_def(target) {
                x.source_name.clone()
            } else {
                return Err(A2xReject::unsupported_expr("call target not admitted"));
            };
            // PLAN-902 §5.4: on the 902 domains the callee's canonical
            // signature (modes + nominal types + kind) is part of the
            // caller's semantic identity — a callee signature/mode change
            // invalidates even an otherwise-unchanged body. Scalar
            // fingerprints keep their frozen bytes.
            if is_902_family(view) && view.trace_symbol(target).is_none() {
                let sig = if let Some(t) = view.fn_by_def(target) {
                    view.canonical_signature(t)
                } else if let Some(x) = view.external_by_def(target) {
                    view.canonical_external_signature(x)
                } else {
                    String::new()
                };
                name = format!("{name}[{sig}]");
            }
            let mut canon_bindings: Vec<(u32, u32, bool)> =
                bindings.iter().map(|b| (b.param, b.arg, b.moved)).collect();
            canon_bindings.sort();
            out.push_str(&format!("call {}(", name));
            for (i, a) in eval_args.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                ser_expr(view, f, *a, canon, next, out, depth + 1)?;
            }
            out.push_str(")|");
            for (i, (p, a, m)) in canon_bindings.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                if *m {
                    out.push_str(&format!("{}>{}!", p, a));
                } else {
                    out.push_str(&format!("{}>{}", p, a));
                }
            }
        }
        // ---- PLAN-902 record/text surface ----
        hir::ExprKind::StrLit { value } => {
            out.push_str(&format!("str{:?}", value));
        }
        hir::ExprKind::CharLit { scalar } => {
            out.push_str(&format!("chr{}", scalar));
        }
        hir::ExprKind::TextLen { operand } => {
            out.push_str("len(");
            ser_expr(view, f, *operand, canon, next, out, depth + 1)?;
            out.push(')');
        }
        hir::ExprKind::TextLenBytes { operand } => {
            out.push_str("lenb(");
            ser_expr(view, f, *operand, canon, next, out, depth + 1)?;
            out.push(')');
        }
        hir::ExprKind::TextCharAt { operand, index } => {
            out.push_str("char_at(");
            ser_expr(view, f, *operand, canon, next, out, depth + 1)?;
            out.push(',');
            ser_expr(view, f, *index, canon, next, out, depth + 1)?;
            out.push(')');
        }
        hir::ExprKind::TextSlice {
            operand,
            start,
            end,
        } => {
            out.push_str("slice(");
            ser_expr(view, f, *operand, canon, next, out, depth + 1)?;
            out.push(',');
            ser_expr(view, f, *start, canon, next, out, depth + 1)?;
            out.push(',');
            ser_expr(view, f, *end, canon, next, out, depth + 1)?;
            out.push(')');
        }
        hir::ExprKind::TextConcat { lhs, rhs } => {
            out.push_str("cat(");
            ser_expr(view, f, *lhs, canon, next, out, depth + 1)?;
            out.push(',');
            ser_expr(view, f, *rhs, canon, next, out, depth + 1)?;
            out.push(')');
        }
        hir::ExprKind::TextClone { operand } => {
            out.push_str("clone(");
            ser_expr(view, f, *operand, canon, next, out, depth + 1)?;
            out.push(')');
        }
        hir::ExprKind::TextToOwned { operand } => {
            out.push_str("to_owned(");
            ser_expr(view, f, *operand, canon, next, out, depth + 1)?;
            out.push(')');
        }
        hir::ExprKind::TextToBorrowed { operand } => {
            out.push_str("to_borrowed(");
            ser_expr(view, f, *operand, canon, next, out, depth + 1)?;
            out.push(')');
        }
        hir::ExprKind::CharToCodepoint { operand } => {
            out.push_str("cp(");
            ser_expr(view, f, *operand, canon, next, out, depth + 1)?;
            out.push(')');
        }
        hir::ExprKind::CharFromCodepoint { operand } => {
            out.push_str("from_cp(");
            ser_expr(view, f, *operand, canon, next, out, depth + 1)?;
            out.push(')');
        }
        hir::ExprKind::RecordCtor {
            record,
            eval_args,
            bindings,
        } => {
            let owner = view
                .record(record.0)
                .map(|r| r.name.clone())
                .unwrap_or_else(|| format!("#{}", record.0));
            let mut canon_bindings: Vec<(u32, u32)> =
                bindings.iter().map(|b| (b.field, b.arg)).collect();
            canon_bindings.sort();
            out.push_str(&format!("new {}(", owner));
            for (i, a) in eval_args.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                ser_expr(view, f, *a, canon, next, out, depth + 1)?;
            }
            out.push_str(")|");
            for (i, (fl, a)) in canon_bindings.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                out.push_str(&format!("{}>{}", fl, a));
            }
        }
        hir::ExprKind::FieldGet { base, path } => {
            out.push_str("get(");
            ser_expr(view, f, *base, canon, next, out, depth + 1)?;
            for fid in path {
                out.push_str(&format!(".{}", fid.0));
            }
            out.push(')');
        }
        hir::ExprKind::BorrowView { base } => {
            out.push_str("view(");
            ser_expr(view, f, *base, canon, next, out, depth + 1)?;
            out.push(')');
        }
        hir::ExprKind::BorrowMut { local } => {
            let id = intern(local.0, canon, next);
            out.push_str(&format!("mutb(${})", id));
        }
        hir::ExprKind::RecordClone { record, base } => {
            let owner = view
                .record(record.0)
                .map(|r| r.name.clone())
                .unwrap_or_else(|| format!("#{}", record.0));
            out.push_str(&format!("rclone {}(", owner));
            ser_expr(view, f, *base, canon, next, out, depth + 1)?;
            out.push(')');
        }
        hir::ExprKind::Move { inner } => {
            out.push_str("move(");
            ser_expr(view, f, *inner, canon, next, out, depth + 1)?;
            out.push(')');
        }
        // ---- PLAN-904 collections surface ----
        hir::ExprKind::ListNew { list } => {
            let elem = view.list_elem(list.0).map(|t| view.ty_name(t)).unwrap_or_default();
            out.push_str(&format!("lnew<{elem}>"));
        }
        hir::ExprKind::ListLen { list } => {
            out.push_str("llen(");
            ser_expr(view, f, *list, canon, next, out, depth + 1)?;
            out.push(')');
        }
        hir::ExprKind::ListPush { list, value } => {
            out.push_str("lpush(");
            ser_expr(view, f, *list, canon, next, out, depth + 1)?;
            out.push(',');
            ser_expr(view, f, *value, canon, next, out, depth + 1)?;
            out.push(')');
        }
        hir::ExprKind::ListInsert { list, index, value } => {
            out.push_str("lins(");
            ser_expr(view, f, *list, canon, next, out, depth + 1)?;
            out.push(',');
            ser_expr(view, f, *index, canon, next, out, depth + 1)?;
            out.push(',');
            ser_expr(view, f, *value, canon, next, out, depth + 1)?;
            out.push(')');
        }
        hir::ExprKind::ListGet { list, index } => {
            out.push_str("lget(");
            ser_expr(view, f, *list, canon, next, out, depth + 1)?;
            out.push(',');
            ser_expr(view, f, *index, canon, next, out, depth + 1)?;
            out.push(')');
        }
        hir::ExprKind::ListSet { list, index, value } => {
            out.push_str("lset(");
            ser_expr(view, f, *list, canon, next, out, depth + 1)?;
            out.push(',');
            ser_expr(view, f, *index, canon, next, out, depth + 1)?;
            out.push(',');
            ser_expr(view, f, *value, canon, next, out, depth + 1)?;
            out.push(')');
        }
        hir::ExprKind::ListRemove { list, index } => {
            out.push_str("lrem(");
            ser_expr(view, f, *list, canon, next, out, depth + 1)?;
            out.push(',');
            ser_expr(view, f, *index, canon, next, out, depth + 1)?;
            out.push(')');
        }
        hir::ExprKind::ListClear { list } => {
            out.push_str("lclr(");
            ser_expr(view, f, *list, canon, next, out, depth + 1)?;
            out.push(')');
        }
        hir::ExprKind::VariantCtor {
            variant,
            disc,
            payload,
        } => {
            let name = view
                .variant(variant.0)
                .map(|v| v.name.clone())
                .unwrap_or_else(|| format!("#{}", variant.0));
            out.push_str(&format!("vnew {name}#{disc}("));
            if let Some(p) = payload {
                ser_expr(view, f, *p, canon, next, out, depth + 1)?;
            }
            out.push(')');
        }
        hir::ExprKind::VariantGuardTest { root, disc } => {
            out.push_str(&format!("vis#{disc}("));
            ser_expr(view, f, *root, canon, next, out, depth + 1)?;
            out.push(')');
        }
        hir::ExprKind::VariantProject { root, disc } => {
            out.push_str(&format!("vproj#{disc}("));
            ser_expr(view, f, *root, canon, next, out, depth + 1)?;
            out.push(')');
        }
        other => {
            return Err(A2xReject::unsupported_expr(format!(
                "expression `{:?}` outside the compute surface",
                other
            )))
        }
    }
    Ok(())
}

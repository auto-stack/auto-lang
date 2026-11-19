//! PLAN-1080 (D-04/D-05): the ordinary-Auto-face signature completion pass.
//!
//! The Auto grammar admits the legacy untyped spellings the frozen narrow
//! faces reject: omitted parameter/return annotations (`fn add(a, b) {
//! a + b }`) and the signature-only prototype (`fn test() int;`). The
//! projection marks both with [`crate::source::ast::TypeRef::Inferred`] /
//! `FnItem::proto`; THIS pass runs between parse and resolve and rewrites
//! every `Inferred` into a concrete annotation, or reports a named
//! `bind/source.inference` diagnostic — resolve never sees an `Inferred`
//! and no silent default type exists anywhere.
//!
//! Inference rule (monomorphic, call-site driven, body-checked):
//! - a parameter's type is fixed by the argument types at the call sites
//!   (positional by index, named by parameter name); all sites must agree;
//! - a return type is derived from the body's tail / `return` expressions
//!   under the inferred parameters (all must agree; `fn main()` with no
//!   value tail is Unit);
//! - unresolved (no constraining site) or conflicting shapes are named
//!   rejections, never defaults;
//! - a prototype unifies with the same-named body (its annotations may
//!   fill body omissions); a prototype without a body is
//!   `bind/source.unresolved`.
//!
//! The pass is a whole-module fixed point over the small untyped set:
//! argument expressions may themselves be calls to untyped functions
//! (`add(add(1, 2), add(3, 4))`), so site collection and return
//! derivation alternate until the assignment stops changing. A round cap
//! (untyped slots + 2) turns a non-converging shape into a named
//! `source.inference` diagnostic instead of a loop.

use crate::atom_text::{Diagnostic, Span, Stage};
use crate::source::ast::{
    Arg, BinOp, Block, Expr, ExprKind, FnItem, Item, Program, Stmt, TypeAnn, TypeItem, TypeRef,
    TypeWord,
};
use std::collections::BTreeMap;

/// One inferred-or-declared signature slot value used inside the pass.
#[derive(Clone, PartialEq, Debug)]
enum Ty {
    /// A concrete annotation (or a fixed inference result).
    Concrete(TypeRef),
    /// Not yet constrained.
    Unknown,
}

impl Ty {
    fn of_ann(ann: &TypeAnn) -> Ty {
        match &ann.kind {
            TypeRef::Inferred => Ty::Unknown,
            other => Ty::Concrete(other.clone()),
        }
    }
}

/// Per untyped free function (methods keep the typed record face in this
/// batch): param slots + return slot, keyed by flat item index.
struct UntypedFn {
    name: String,
    param_names: Vec<String>,
    params: Vec<Ty>,
    ret: Ty,
}

struct Ctx<'p> {
    records: BTreeMap<String, &'p TypeItem>,
    /// Typed free-function returns by name.
    typed_rets: BTreeMap<String, Ty>,
    /// Typed method returns by `Rec::name`.
    method_rets: BTreeMap<String, Ty>,
    /// Item index of each untyped free function by name.
    untyped_idx: BTreeMap<String, usize>,
    untyped: BTreeMap<usize, UntypedFn>,
}

pub(crate) fn complete(program: &mut Program) -> Result<(), Vec<Diagnostic>> {
    let mut diags = Vec::new();
    merge_prototypes(program, &mut diags);
    let has_inferred = program.items.iter().any(|i| match i {
        Item::Fn(f) => {
            f.ret.kind == TypeRef::Inferred
                || f.params.iter().any(|p| p.ty.kind == TypeRef::Inferred)
        }
        _ => false,
    });
    if !has_inferred {
        // Fast path: fully typed module — the pass is a no-op (every
        // frozen-narrow-face-shaped Auto program lands here).
        return if diags.is_empty() { Ok(()) } else { Err(diags) };
    }

    let mut ctx = Ctx {
        records: BTreeMap::new(),
        typed_rets: BTreeMap::new(),
        method_rets: BTreeMap::new(),
        untyped_idx: BTreeMap::new(),
        untyped: BTreeMap::new(),
    };
    for (idx, item) in program.items.iter().enumerate() {
        match item {
            Item::Fn(f) => {
                if f.proto {
                    continue;
                }
                let untyped = f.ret.kind == TypeRef::Inferred
                    || f.params.iter().any(|p| p.ty.kind == TypeRef::Inferred);
                if untyped {
                    ctx.untyped_idx.entry(f.name.name.clone()).or_insert(idx);
                    ctx.untyped.insert(
                        idx,
                        UntypedFn {
                            name: f.name.name.clone(),
                            param_names: f.params.iter().map(|p| p.name.name.clone()).collect(),
                            params: f.params.iter().map(|p| Ty::of_ann(&p.ty)).collect(),
                            ret: Ty::of_ann(&f.ret),
                        },
                    );
                } else {
                    ctx.typed_rets
                        .entry(f.name.name.clone())
                        .or_insert_with(|| Ty::of_ann(&f.ret));
                }
            }
            Item::Type(t) => {
                for m in &t.methods {
                    ctx.method_rets
                        .entry(format!("{}::{}", t.name.name, m.name.name))
                        .or_insert_with(|| Ty::of_ann(&m.ret));
                }
                ctx.records.entry(t.name.name.clone()).or_insert(t);
            }
            _ => {}
        }
    }

    // Fixed point: alternate site collection and return derivation until
    // the slot assignment stops changing (bounded; overrun falls through
    // to the unresolved-slot rejections below).
    let slot_count: usize = ctx
        .untyped
        .values()
        .map(|u| u.params.len() + 1)
        .sum::<usize>()
        .max(1);
    let cap = slot_count + 2;
    for _round in 0..cap {
        let before = fingerprint(&ctx);
        let mut sites: Vec<(String, Vec<(usize, Ty)>, Span)> = Vec::new();
        for item in program.items.iter() {
            match item {
                Item::Fn(f) => {
                    if f.proto {
                        continue;
                    }
                    let env = fn_env(f, &ctx);
                    walk_block(&f.body, &env, &mut ctx, &mut sites);
                }
                Item::Type(t) => {
                    for m in &t.methods {
                        let env = fn_env(m, &ctx);
                        walk_block(&m.body, &env, &mut ctx, &mut sites);
                    }
                }
                _ => {}
            }
        }
        feed_sites(&mut ctx, sites, &mut diags);
        derive_returns(program, &mut ctx, &mut diags);
        if fingerprint(&ctx) == before {
            break;
        }
    }

    // Rewrite annotations + reject unresolved/conflicting slots.
    for (idx, u) in &ctx.untyped {
        let Item::Fn(f) = &mut program.items[*idx] else {
            unreachable!("untyped slot keyed by fn item");
        };
        for (pi, slot) in u.params.iter().enumerate() {
            match slot {
                Ty::Concrete(tr) => {
                    f.params[pi].ty = TypeAnn {
                        kind: tr.clone(),
                        span: f.params[pi].ty.span,
                    };
                }
                Ty::Unknown => {
                    diags.push(Diagnostic::new(
                        Stage::Bind,
                        "source.inference",
                        format!(
                            "the type of parameter `{}` of `{}` cannot be inferred: no call site constrains it here",
                            u.param_names[pi], u.name
                        ),
                        f.params[pi].name.span,
                    ));
                }
            }
        }
        match &u.ret {
            Ty::Concrete(tr) => {
                f.ret = TypeAnn {
                    kind: tr.clone(),
                    span: f.ret.span,
                };
            }
            Ty::Unknown => {
                // An omitted return with no value tail is Unit (the
                // worker's unit channel); anything else was already
                // reported by derive_returns.
                if !returns_value(&f.body) {
                    f.ret = TypeAnn {
                        kind: TypeRef::Builtin(TypeWord::Void),
                        span: f.ret.span,
                    };
                } else {
                    diags.push(Diagnostic::new(
                        Stage::Bind,
                        "source.inference",
                        format!(
                            "the return type of `{}` cannot be inferred from its body",
                            u.name
                        ),
                        f.name.span,
                    ));
                }
            }
        }
    }

    if diags.is_empty() {
        Ok(())
    } else {
        Err(diags)
    }
}

fn fn_env(f: &FnItem, ctx: &Ctx) -> Env {
    let mut env = Env::default();
    // An untyped fn's own params resolve through its CURRENT slots so a
    // body like `let x = a + a; x` propagates the pending parameter type.
    if let Some(&idx) = ctx.untyped_idx.get(&f.name.name) {
        if let Some(u) = ctx.untyped.get(&idx) {
            for (p, slot) in f.params.iter().zip(u.params.iter()) {
                env.0.insert(p.name.name.clone(), slot.clone());
            }
            return env;
        }
    }
    for p in &f.params {
        env.0.insert(p.name.name.clone(), Ty::of_ann(&p.ty));
    }
    env
}

fn fingerprint(ctx: &Ctx) -> String {
    let mut out = String::new();
    for (idx, u) in &ctx.untyped {
        out.push_str(&idx.to_string());
        for s in u.params.iter().chain(std::iter::once(&u.ret)) {
            out.push_str(&ty_text(s));
        }
    }
    out
}

fn ty_text(t: &Ty) -> String {
    match t {
        Ty::Concrete(TypeRef::Builtin(w)) => format!("b{w:?}"),
        Ty::Concrete(TypeRef::Record(r)) => format!("r{}", r.name),
        Ty::Concrete(_) => "x".to_string(),
        Ty::Unknown => "?".to_string(),
    }
}

/// A body "returns a value" when it has a trailing expression statement or
/// any `return <expr>` — used to distinguish Unit mains from unresolved
/// returns.
fn returns_value(b: &Block) -> bool {
    block_tail(b).is_some() || has_return(b)
}

fn has_return(b: &Block) -> bool {
    b.stmts.iter().any(|s| match s {
        Stmt::Return { .. } => true,
        Stmt::If { then, els, .. } => has_return(then) || els.as_ref().is_some_and(has_return),
        Stmt::While { body, .. } | Stmt::Loop { body, .. } => has_return(body),
        _ => false,
    })
}

/// The trailing expression statement of a block (the tail value).
fn block_tail(b: &Block) -> Option<&Expr> {
    match b.stmts.last() {
        Some(Stmt::ExprStmt { value, .. }) => Some(value),
        _ => None,
    }
}

/// Prototype unification: `fn f(..) R;` merges into the same-named body —
/// the prototype's annotations may fill the body's omissions — and the
/// prototype item is removed. Unmatched prototypes are named rejections.
fn merge_prototypes(program: &mut Program, diags: &mut Vec<Diagnostic>) {
    let protos: Vec<FnItem> = program
        .items
        .iter()
        .filter(|i| matches!(i, Item::Fn(f) if f.proto))
        .map(|i| match i {
            Item::Fn(f) => f.clone(),
            _ => unreachable!(),
        })
        .collect();
    if protos.is_empty() {
        return;
    }
    for proto in &protos {
        let body_idx = program
            .items
            .iter()
            .position(|i| matches!(i, Item::Fn(f) if !f.proto && f.name.name == proto.name.name));
        match body_idx {
            Some(bi) => {
                let Item::Fn(body) = &mut program.items[bi] else {
                    unreachable!();
                };
                if body.ret.kind == TypeRef::Inferred && proto.ret.kind != TypeRef::Inferred {
                    body.ret = proto.ret.clone();
                }
                for (pp, bp) in proto.params.iter().zip(body.params.iter_mut()) {
                    if bp.ty.kind == TypeRef::Inferred && pp.ty.kind != TypeRef::Inferred {
                        bp.ty = pp.ty.clone();
                    }
                }
            }
            None => diags.push(Diagnostic::new(
                Stage::Bind,
                "source.unresolved",
                format!(
                    "prototype `{}` has no matching function body in this module",
                    proto.name.name
                ),
                proto.span,
            )),
        }
    }
    program.items.retain(|i| !matches!(i, Item::Fn(f) if f.proto));
}

/// One estimation environment: parameter/local name → Ty.
#[derive(Clone, Default)]
struct Env(BTreeMap<String, Ty>);

/// Walk one block, keeping a sequential local env, recording call sites of
/// untyped functions and estimating every subexpression once.
fn walk_block(b: &Block, env: &Env, ctx: &mut Ctx, sites: &mut Vec<(String, Vec<(usize, Ty)>, Span)>) {
    let mut env = env.clone();
    for s in &b.stmts {
        match s {
            Stmt::Let { name, value, .. } => {
                let t = estimate(value, &env, ctx, sites);
                env.0.insert(name.name.clone(), t);
            }
            Stmt::Assign { value, .. } => {
                estimate(value, &env, ctx, sites);
            }
            Stmt::ExprStmt { value, .. } => {
                estimate(value, &env, ctx, sites);
            }
            Stmt::Return { value, .. } => {
                estimate(value, &env, ctx, sites);
            }
            Stmt::If {
                cond, then, els, ..
            } => {
                estimate(cond, &env, ctx, sites);
                walk_block(then, &env, ctx, sites);
                if let Some(e) = els {
                    walk_block(e, &env, ctx, sites);
                }
            }
            Stmt::While { cond, body, .. } => {
                estimate(cond, &env, ctx, sites);
                walk_block(body, &env, ctx, sites);
            }
            Stmt::Loop { body, .. } => walk_block(body, &env, ctx, sites),
            _ => {}
        }
    }
}

/// Estimate an expression's type under the environment, recording call
/// sites of untyped functions along the way. Unknown shapes return
/// `Ty::Unknown` (deferred to a later fixed-point round).
fn estimate(
    e: &Expr,
    env: &Env,
    ctx: &mut Ctx,
    sites: &mut Vec<(String, Vec<(usize, Ty)>, Span)>,
) -> Ty {
    match &e.kind {
        ExprKind::IntLit(_) => Ty::Concrete(TypeRef::Builtin(TypeWord::Int)),
        ExprKind::BoolLit(_) => Ty::Concrete(TypeRef::Builtin(TypeWord::Bool)),
        ExprKind::StrLit(_) => Ty::Concrete(TypeRef::Builtin(TypeWord::StrOwned)),
        ExprKind::CharLit(_) => Ty::Concrete(TypeRef::Builtin(TypeWord::Char)),
        ExprKind::TypedLit { ty, .. } => Ty::Concrete(TypeRef::Builtin(*ty)),
        ExprKind::Var(id) => env.0.get(&id.name).cloned().unwrap_or(Ty::Unknown),
        ExprKind::Binary { op, lhs, rhs } => {
            let lt = estimate(lhs, env, ctx, sites);
            let _ = estimate(rhs, env, ctx, sites);
            match op {
                BinOp::Lt
                | BinOp::Le
                | BinOp::Gt
                | BinOp::Ge
                | BinOp::Eq
                | BinOp::Ne
                | BinOp::And
                | BinOp::Or => Ty::Concrete(TypeRef::Builtin(TypeWord::Bool)),
                _ => lt,
            }
        }
        ExprKind::Unary { op, operand } => {
            let t = estimate(operand, env, ctx, sites);
            match op {
                crate::source::ast::UnaryOp::Not => {
                    Ty::Concrete(TypeRef::Builtin(TypeWord::Bool))
                }
                _ => t,
            }
        }
        ExprKind::Call { callee, args } => {
            // A positional constructor routes to the record when the name
            // is one; otherwise the free-function signature answers.
            for a in args {
                estimate_arg(a, env, ctx, sites);
            }
            if let Some(rec) = ctx.records.get(&callee.name) {
                return Ty::Concrete(TypeRef::Record(rec.name.clone()));
            }
            // The console builtins are Unit-result intrinsics (not user
            // fns); a trailing `print(..)` therefore types a void body.
            if matches!(callee.name.as_str(), "print" | "eprint" | "say")
                && !ctx.typed_rets.contains_key(&callee.name)
                && !ctx.untyped_idx.contains_key(&callee.name)
            {
                return Ty::Concrete(TypeRef::Builtin(TypeWord::Void));
            }
            if ctx.untyped_idx.contains_key(&callee.name) {
                // Record the site: positional by index, named by name.
                let names: Vec<String> = ctx
                    .untyped_idx
                    .get(&callee.name)
                    .and_then(|&i| ctx.untyped.get(&i))
                    .map(|u| u.param_names.clone())
                    .unwrap_or_default();
                let slots: Vec<(usize, Ty)> = args
                    .iter()
                    .enumerate()
                    .filter_map(|(i, a)| {
                        let pos = match a {
                            Arg::Pos(_) | Arg::MovePos(_) => Some(i),
                            Arg::Named { name, .. } => {
                                names.iter().position(|n| *n == name.name)
                            }
                        }?;
                        let val = match a {
                            Arg::Pos(x) | Arg::MovePos(x) => x,
                            Arg::Named { value, .. } => value,
                        };
                        Some((pos, estimate(val, env, ctx, sites)))
                    })
                    .collect();
                sites.push((callee.name.clone(), slots, e.span));
                let ret = ctx
                    .untyped_idx
                    .get(&callee.name)
                    .and_then(|&i| ctx.untyped.get(&i))
                    .map(|u| u.ret.clone())
                    .unwrap_or(Ty::Unknown);
                return ret;
            }
            ctx.typed_rets.get(&callee.name).cloned().unwrap_or(Ty::Unknown)
        }
        ExprKind::CtorNamed { ty, fields, .. } => {
            for f in fields {
                estimate(&f.value, env, ctx, sites);
            }
            Ty::Concrete(TypeRef::Record(ty.clone()))
        }
        ExprKind::Field { base, name } => {
            let bt = estimate(base, env, ctx, sites);
            if let Ty::Concrete(TypeRef::Record(r)) = bt {
                if let Some(rec) = ctx.records.get(&r.name) {
                    for f in &rec.fields {
                        if f.name.name == name.name {
                            return Ty::of_ann(&f.ty);
                        }
                    }
                }
            }
            Ty::Unknown
        }
        ExprKind::MethodCall { recv, name, args } => {
            let rt = estimate(recv, env, ctx, sites);
            for a in args {
                estimate_arg(a, env, ctx, sites);
            }
            if let Ty::Concrete(TypeRef::Record(r)) = rt {
                let key = format!("{}::{}", r.name, name.name);
                if let Some(t) = ctx.method_rets.get(&key) {
                    return t.clone();
                }
            }
            Ty::Unknown
        }
        ExprKind::MutArg { .. } | ExprKind::ViewArg { .. } => Ty::Unknown,
        _ => Ty::Unknown,
    }
}

fn estimate_arg(
    a: &Arg,
    env: &Env,
    ctx: &mut Ctx,
    sites: &mut Vec<(String, Vec<(usize, Ty)>, Span)>,
) {
    match a {
        Arg::Pos(x) | Arg::MovePos(x) => {
            estimate(x, env, ctx, sites);
        }
        Arg::Named { value, .. } => {
            estimate(value, env, ctx, sites);
        }
    }
}

/// Feed the collected argument types into the untyped parameter slots.
/// The FIRST concrete shape wins; a later DIVERGENT shape at another site
/// is a named `bind/source.inference` rejection at that site (never a
/// silent first-wins under conflicting evidence).
fn feed_sites(ctx: &mut Ctx, sites: Vec<(String, Vec<(usize, Ty)>, Span)>, diags: &mut Vec<Diagnostic>) {
    for (callee, slots, span) in sites {
        let Some(&idx) = ctx.untyped_idx.get(&callee) else {
            continue;
        };
        let Some(u) = ctx.untyped.get_mut(&idx) else {
            continue;
        };
        for (pos, ty) in slots {
            if let Ty::Concrete(t) = ty {
                if let Some(slot) = u.params.get_mut(pos) {
                    match slot {
                        Ty::Unknown => *slot = Ty::Concrete(t),
                        Ty::Concrete(prev) if prev != &t => {
                            diags.push(Diagnostic::new(
                                Stage::Bind,
                                "source.inference",
                                format!(
                                    "conflicting argument types for parameter `{}` of `{}` ({} vs {})",
                                    u.param_names.get(pos).map(|n| n.as_str()).unwrap_or("?"),
                                    u.name,
                                    ty_text(&Ty::Concrete(prev.clone())),
                                    ty_text(&Ty::Concrete(t.clone()))
                                ),
                                span,
                            ));
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}

/// Derive each untyped function's return slot from its body (tail + return
/// statements) under the CURRENT parameter slots.
fn derive_returns(program: &Program, ctx: &mut Ctx, diags: &mut Vec<Diagnostic>) {
    let idxs: Vec<usize> = ctx.untyped.keys().cloned().collect();
    for idx in idxs {
        let Item::Fn(f) = &program.items[idx] else {
            continue;
        };
        if f.ret.kind != TypeRef::Inferred {
            continue;
        }
        let env = fn_env(f, ctx);
        let mut dummy_sites: Vec<(String, Vec<(usize, Ty)>, Span)> = Vec::new();
        let mut tails: Vec<Ty> = Vec::new();
        collect_block_value(&f.body, &env, ctx, &mut dummy_sites, &mut tails);
        let mut agreed: Option<Ty> = None;
        for t in tails {
            if let Ty::Concrete(tr) = t {
                match &agreed {
                    None => agreed = Some(Ty::Concrete(tr)),
                    Some(Ty::Concrete(prev)) if prev == &tr => {}
                    Some(Ty::Concrete(prev)) => {
                        diags.push(Diagnostic::new(
                            Stage::Bind,
                            "source.inference",
                            format!(
                                "the body of `{}` produces conflicting return types ({} vs {})",
                                f.name.name,
                                ty_text(&Ty::Concrete(prev.clone())),
                                ty_text(&Ty::Concrete(tr.clone()))
                            ),
                            f.name.span,
                        ));
                    }
                    _ => {}
                }
            }
        }
        if let Some(Ty::Concrete(tr)) = agreed {
            if let Some(u) = ctx.untyped.get_mut(&idx) {
                u.ret = Ty::Concrete(tr);
            }
        }
    }
}

fn collect_block_value(
    b: &Block,
    env: &Env,
    ctx: &mut Ctx,
    sites: &mut Vec<(String, Vec<(usize, Ty)>, Span)>,
    tails: &mut Vec<Ty>,
) {
    let mut env = env.clone();
    for s in &b.stmts {
        match s {
            Stmt::Let { name, value, .. } => {
                let t = estimate(value, &env, ctx, sites);
                env.0.insert(name.name.clone(), t);
            }
            Stmt::Return { value, .. } => tails.push(estimate(value, &env, ctx, sites)),
            Stmt::ExprStmt { value, .. } => {
                // Only the trailing statement is the tail value.
                if std::ptr::eq(s, b.stmts.last().expect("checked in loop")) {
                    tails.push(estimate(value, &env, ctx, sites));
                } else {
                    estimate(value, &env, ctx, sites);
                }
            }
            Stmt::If {
                cond, then, els, ..
            } => {
                estimate(cond, &env, ctx, sites);
                collect_block_value(then, &env, ctx, sites, tails);
                if let Some(e) = els {
                    collect_block_value(e, &env, ctx, sites, tails);
                }
            }
            Stmt::While { cond, body, .. } => {
                estimate(cond, &env, ctx, sites);
                collect_block_value(body, &env, ctx, sites, tails);
            }
            Stmt::Loop { body, .. } => collect_block_value(body, &env, ctx, sites, tails),
            Stmt::Assign { value, .. } => {
                estimate(value, &env, ctx, sites);
            }
            _ => {}
        }
    }
}

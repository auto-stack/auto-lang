//! Lower the shared semantic view to a typed Rust program (plan §2:
//! view → structured prepare → Rust provider).
//!
//! Correctness contracts this lowering is responsible for:
//!
//! * **Trap semantics** — every arithmetic op becomes an explicit checked
//!   helper returning `Result<_, AutoTrap>`; `?` propagates. No reliance on
//!   Rust overflow options (debug/release behave identically).
//! * **Source-order, single evaluation of call arguments** — `eval_args`
//!   (source order) are lowered first; every effectful (trap- or
//!   trace-capable) argument is pinned into a temporary `let` BEFORE the
//!   call, then placed per `bindings` (param ← arg). The target never
//!   re-guesses evaluation order from placement.
//! * **Short-circuit laziness** — `&&`/`||` lower to `if`-expressions whose
//!   RHS (including its own temporaries) lives inside the taken branch; an
//!   untaken RHS is never evaluated, temporaries and all.
//! * **Nearest-loop discipline** — Auto break/continue always target the
//!   innermost enclosing loop; the lowering asserts that on the explicit
//!   `LoopId` before emitting plain `break`/`continue`.
//! * **Hygiene** — locals render as `l{idx}_{name}`, functions as
//!   `f{def}_{name}`, temporaries as `t{n}`; runtime helpers own the
//!   reserved `a2x_` prefix. User names can never hijack or collide.

use ace_a2x_core::{A2xReject, FnFacts, SemanticView, Ty};
use ace_core::hir;
use std::collections::HashMap;

/// Comparison operators (infallible; operands carry their own `?`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CmpOp {
    Lt,
    Le,
    Gt,
    Ge,
    Eq,
    Ne,
}

impl CmpOp {
    fn rust_str(self) -> &'static str {
        match self {
            CmpOp::Lt => "<",
            CmpOp::Le => "<=",
            CmpOp::Gt => ">",
            CmpOp::Ge => ">=",
            CmpOp::Eq => "==",
            CmpOp::Ne => "!=",
        }
    }
}

/// One emitted Rust statement.
#[derive(Clone, Debug)]
pub enum RStmt {
    Let {
        name: String,
        ty: Ty,
        mutable: bool,
        init: RExpr,
    },
    Assign {
        name: String,
        value: RExpr,
    },
    If {
        cond: RExpr,
        then: Vec<RStmt>,
        els: Option<Vec<RStmt>>,
    },
    Loop {
        body: Vec<RStmt>,
    },
    Break,
    Continue,
    Return {
        value: RExpr,
    },
}

/// Statements + tail value of a lazily-evaluated short-circuit side. Used
/// only inside [`RExpr::AndSC`]/[`RExpr::OrSC`] so the RHS temporaries stay
/// inside the taken branch.
#[derive(Clone, Debug)]
pub struct RBlock {
    pub stmts: Vec<RStmt>,
    pub value: RExpr,
}

/// One emitted Rust expression. Rendering (print.rs) appends `?` where a
/// fallible root is consumed as a value (`render_val`); nested fallible
/// operands are pinned into temporaries by the lowering, so call/checked
/// arguments are always side-effect-free here.
#[derive(Clone, Debug)]
pub enum RExpr {
    I32Lit(i32),
    BoolLit(bool),
    Local(String),
    /// Checked arithmetic helper call (`a2x_add`/`a2x_sub`/`a2x_mul`/
    /// `a2x_neg`/`a2x_div`/`a2x_rem`) — fallible.
    Checked {
        func: &'static str,
        args: Vec<RExpr>,
    },
    Cmp {
        op: CmpOp,
        lhs: Box<RExpr>,
        rhs: Box<RExpr>,
    },
    Not(Box<RExpr>),
    /// Lazy `lhs && { rhs-block }` (renders as an if-expression).
    AndSC {
        lhs: Box<RExpr>,
        rhs: Box<RBlock>,
    },
    /// Lazy `lhs || { rhs-block }`.
    OrSC {
        lhs: Box<RExpr>,
        rhs: Box<RBlock>,
    },
    /// Call of a lowered user function — fallible.
    Call {
        func: String,
        args: Vec<RExpr>,
    },
    /// Authorized trace intrinsic call — fallible, effectful.
    Trace {
        func: &'static str,
    },
}

impl RExpr {
    /// True when this root returns `Result` and needs `?` at a value use.
    /// Trace marks return a plain i32 (frozen native contract) — they are
    /// EFFECTFUL but not fallible.
    pub fn is_fallible_root(&self) -> bool {
        matches!(self, RExpr::Checked { .. } | RExpr::Call { .. })
    }

    /// True when the root must be pinned into a temporary before reuse
    /// (traps OR observable effects — single evaluation in source order).
    pub fn needs_pin(&self) -> bool {
        self.is_fallible_root() || matches!(self, RExpr::Trace { .. })
    }

    /// Pretty operator tag for diagnostics/dumps.
    pub fn tag(&self) -> &'static str {
        match self {
            RExpr::I32Lit(_) => "i32",
            RExpr::BoolLit(_) => "bool",
            RExpr::Local(_) => "local",
            RExpr::Checked { func, .. } => func,
            RExpr::Cmp { .. } => "cmp",
            RExpr::Not(_) => "not",
            RExpr::AndSC { .. } => "and",
            RExpr::OrSC { .. } => "or",
            RExpr::Call { .. } => "call",
            RExpr::Trace { func } => func,
        }
    }
}

/// One lowered user function. `origin` is the def's byte span in the
/// compiled source (source map v1 — printed as a comment and carried in the
/// manifest so target diagnostics map back to the original `.at`).
#[derive(Clone, Debug)]
pub struct RustFn {
    pub name: String,
    pub params: Vec<(String, Ty)>,
    pub result: Ty,
    pub origin: (u32, u32),
    pub body: Vec<RStmt>,
}

/// The whole lowered program (all module functions + entry name).
#[derive(Clone, Debug)]
pub struct RustProgram {
    pub functions: Vec<RustFn>,
    /// Rust name of the entry function.
    pub entry: String,
    /// Distinct trace symbols used (runtime emission input).
    pub marks: Vec<&'static str>,
}

/// Sanitize a source name into Rust-identifier-safe ASCII ([A-Za-z0-9_]).
fn sanitize(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    for c in name.chars() {
        if c.is_ascii_alphanumeric() || c == '_' {
            out.push(c);
        } else {
            out.push('_');
        }
    }
    out
}

/// Hygienic Rust name of a module function — derived from the SOURCE name
/// only. A module's source names are unique (frontend rule) and STABLE
/// across whole-module and single-body bundles, unlike def-table indices
/// which shift when a fragment bundle carries scaffolds in another order
/// (F-828-R1). The `fn_` prefix can never collide with Rust keywords or
/// the runtime's reserved `a2x_` helpers.
pub fn rust_fn_name(name: &str) -> String {
    let s = sanitize(name);
    if s.is_empty() {
        "fn_".to_string()
    } else {
        format!("fn_{}", s)
    }
}

/// Lower a whole module for the given explicit entry function name.
pub fn lower_functions<'a>(view: &SemanticView<'a>) -> Result<RustProgram, A2xReject> {
    let mut functions = Vec::new();
    for f in view.functions() {
        functions.push(lower_fn(view, f)?);
    }
    let mut marks: Vec<&'static str> = view.trace_symbols().into_iter().collect();
    marks.sort_unstable();
    Ok(RustProgram {
        functions,
        entry: String::new(),
        marks,
    })
}

/// Lower a whole module and select its entry by explicit source name.
pub fn lower_program<'a>(
    view: &SemanticView<'a>,
    entry_name: &str,
) -> Result<RustProgram, A2xReject> {
    let entry = view.entry(entry_name)?;
    let mut p = lower_functions(view)?;
    p.entry = rust_fn_name(entry.source_name);
    Ok(p)
}

fn lower_fn<'a>(view: &SemanticView<'a>, f: &FnFacts<'a>) -> Result<RustFn, A2xReject> {
    let mut lx = FnLowering::new(view, f)?;
    let body = lx.lower_block(f.body.entry)?;
    let params = lx.rust_params();
    Ok(RustFn {
        name: rust_fn_name(f.source_name),
        params,
        result: f.result,
        origin: f.origin,
        body,
    })
}

struct FnLowering<'v, 'a> {
    view: &'v SemanticView<'a>,
    facts: &'v FnFacts<'a>,
    /// LocalId index → hygienic Rust local name.
    local_names: HashMap<u32, String>,
    /// LocalId index → target-neutral type.
    local_tys: HashMap<u32, Ty>,
    /// Param LocalId indices in signature order.
    param_locals: Vec<u32>,
    /// LocalId indices that are assignment targets (Rust `mut`).
    assigned: std::collections::BTreeSet<u32>,
    temp_counter: usize,
    /// LoopId stack of enclosing loops (innermost last).
    loop_stack: Vec<u32>,
    /// Current expression/statement nesting depth (budget guard).
    depth: usize,
}

impl<'v, 'a> FnLowering<'v, 'a> {
    fn new(view: &'v SemanticView<'a>, facts: &'v FnFacts<'a>) -> Result<Self, A2xReject> {
        let mut local_names = HashMap::new();
        let mut local_tys = HashMap::new();
        for (idx, local) in facts.body.locals.iter().enumerate() {
            let s = sanitize(&local.name);
            let name = if s.is_empty() {
                format!("l{}", idx)
            } else {
                format!("l{}_{}", idx, s)
            };
            local_names.insert(idx as u32, name);
            local_tys.insert(idx as u32, view.resolve_use_ty(local.ty)?);
        }
        let mut param_locals: Vec<(u32, u32)> = facts
            .body
            .locals
            .iter()
            .enumerate()
            .filter(|(_, l)| matches!(l.role, hir::LocalRole::Param))
            .map(|(i, l)| (l.index.unwrap_or(u32::MAX), i as u32))
            .collect();
        param_locals.sort();
        let param_locals: Vec<u32> = param_locals.into_iter().map(|(_, i)| i).collect();
        if param_locals.len() != facts.params.len() {
            return Err(A2xReject::new(
                "a2x.internal",
                "param locals disagree with collected params",
            ));
        }

        // Assignment targets → Rust `mut` (params and vars alike).
        let mut assigned = std::collections::BTreeSet::new();
        collect_assignees(facts, &mut assigned);

        Ok(FnLowering {
            view,
            facts,
            local_names,
            local_tys,
            param_locals,
            assigned,
            temp_counter: 0,
            loop_stack: Vec::new(),
            depth: 0,
        })
    }

    fn rust_params(&self) -> Vec<(String, Ty)> {
        self.param_locals
            .iter()
            .map(|&id| {
                let mut name = self.local_names[&id].clone();
                if self.assigned.contains(&id) {
                    name = format!("mut {}", name);
                }
                (name, self.local_tys[&id])
            })
            .collect()
    }

    fn fresh_temp(&mut self) -> String {
        let t = format!("t{}", self.temp_counter);
        self.temp_counter += 1;
        t
    }

    fn lower_block(&mut self, block: hir::BlockId) -> Result<Vec<RStmt>, A2xReject> {
        let block = self
            .facts
            .body
            .blocks
            .get(block.0 as usize)
            .ok_or_else(|| A2xReject::new("a2x.internal", "block index out of range"))?;
        let mut out = Vec::new();
        for stmt in &block.stmts {
            self.lower_stmt(stmt, &mut out)?;
        }
        Ok(out)
    }

    fn lower_stmt(&mut self, s: &hir::Stmt, out: &mut Vec<RStmt>) -> Result<(), A2xReject> {
        match s {
            hir::Stmt::Let { local, value } => {
                let (v, _) = self.lower_expr(*value, out)?;
                let name = self.local_name(local.0)?;
                let ty = self.local_ty(local.0)?;
                let mutable = self.assigned.contains(&local.0);
                out.push(RStmt::Let {
                    name,
                    ty,
                    mutable,
                    init: v,
                });
            }
            hir::Stmt::Return { value } => {
                let (v, _) = self.lower_expr(*value, out)?;
                out.push(RStmt::Return { value: v });
            }
            hir::Stmt::If { cond, then, els } => {
                let (c, _) = self.lower_expr(*cond, out)?;
                let t = self.lower_block(*then)?;
                let e = match els {
                    Some(b) => Some(self.lower_block(*b)?),
                    None => None,
                };
                out.push(RStmt::If {
                    cond: c,
                    then: t,
                    els: e,
                });
            }
            hir::Stmt::Loop { id, body } => {
                self.loop_stack.push(id.0);
                let b = self.lower_block(*body)?;
                self.loop_stack.pop();
                out.push(RStmt::Loop { body: b });
            }
            hir::Stmt::Assign { place, value } => {
                let place = self
                    .facts
                    .body
                    .places
                    .get(place.0 as usize)
                    .ok_or_else(|| A2xReject::new("a2x.internal", "place index out of range"))?;
                let (v, _) = self.lower_expr(*value, out)?;
                let name = self.local_name(place.local.0)?;
                out.push(RStmt::Assign { name, value: v });
            }
            hir::Stmt::Break { target } => {
                self.assert_nearest(target.0, "break")?;
                out.push(RStmt::Break);
            }
            hir::Stmt::Continue { target } => {
                self.assert_nearest(target.0, "continue")?;
                out.push(RStmt::Continue);
            }
            other => {
                return Err(A2xReject::unsupported_stmt(format!(
                    "in `{}`: statement kind `{:?}` is outside the compute surface",
                    self.facts.name, other
                )))
            }
        }
        Ok(())
    }

    /// Auto break/continue always target the innermost enclosing loop; a
    /// non-nearest target would mistranslate to Rust's plain break/continue.
    fn assert_nearest(&self, target: u32, what: &str) -> Result<(), A2xReject> {
        match self.loop_stack.last() {
            Some(inner) if *inner == target => Ok(()),
            other => Err(A2xReject::new(
                "a2x.internal",
                format!(
                    "`{}` targets loop #{} but the innermost enclosing loop is {:?}",
                    what, target, other
                ),
            )),
        }
    }

    fn local_name(&self, id: u32) -> Result<String, A2xReject> {
        self.local_names
            .get(&id)
            .cloned()
            .ok_or_else(|| A2xReject::new("a2x.internal", format!("unknown local #{}", id)))
    }

    fn local_ty(&self, id: u32) -> Result<Ty, A2xReject> {
        self.local_tys
            .get(&id)
            .copied()
            .ok_or_else(|| A2xReject::new("a2x.internal", format!("unknown local #{}", id)))
    }

    /// Lower one expression, appending any pinning preludes to `out`.
    /// Returns the value expression and its type. Effectful subexpressions
    /// (fallible roots) are pinned into temporaries at their source-order
    /// position — EXCEPT inside a short-circuit RHS, whose preludes land in
    /// that side's own lazy block.
    fn lower_expr(
        &mut self,
        id: hir::ExprId,
        out: &mut Vec<RStmt>,
    ) -> Result<(RExpr, Ty), A2xReject> {
        self.depth += 1;
        if self.depth > ace_a2x_core::MAX_STRUCTURE_DEPTH {
            self.depth -= 1;
            return Err(A2xReject::new(
                "a2x.too-deep",
                format!(
                    "function `{}` lowers deeper than the frozen budget {}",
                    self.facts.source_name,
                    ace_a2x_core::MAX_STRUCTURE_DEPTH
                ),
            ));
        }
        let r = self.lower_expr_inner(id, out);
        self.depth -= 1;
        r
    }

    fn lower_expr_inner(
        &mut self,
        id: hir::ExprId,
        out: &mut Vec<RStmt>,
    ) -> Result<(RExpr, Ty), A2xReject> {
        let e = self
            .facts
            .body
            .exprs
            .get(id.0 as usize)
            .ok_or_else(|| A2xReject::new("a2x.internal", "expr index out of range"))?;
        match &e.kind {
            hir::ExprKind::ReadLocal { local } => Ok((
                RExpr::Local(self.local_name(local.0)?),
                self.local_ty(local.0)?,
            )),
            hir::ExprKind::Constant { value } => Ok((RExpr::I32Lit(*value), Ty::I32)),
            hir::ExprKind::Binary { op, lhs, rhs, .. } => {
                let (l, _) = self.lower_pure(*lhs, out)?;
                let (r, _) = self.lower_pure(*rhs, out)?;
                match op {
                    hir::BinOp::AddI32 => Ok((arith("a2x_add", l, r), Ty::I32)),
                    hir::BinOp::SubI32 => Ok((arith("a2x_sub", l, r), Ty::I32)),
                    hir::BinOp::MulI32 => Ok((arith("a2x_mul", l, r), Ty::I32)),
                    hir::BinOp::DivI32 => Ok((arith("a2x_div", l, r), Ty::I32)),
                    hir::BinOp::RemI32 => Ok((arith("a2x_rem", l, r), Ty::I32)),
                    hir::BinOp::LtI32 => Ok((cmp(CmpOp::Lt, l, r), Ty::Bool)),
                    hir::BinOp::LeI32 => Ok((cmp(CmpOp::Le, l, r), Ty::Bool)),
                    hir::BinOp::GtI32 => Ok((cmp(CmpOp::Gt, l, r), Ty::Bool)),
                    hir::BinOp::GeI32 => Ok((cmp(CmpOp::Ge, l, r), Ty::Bool)),
                    hir::BinOp::Eq => Ok((cmp(CmpOp::Eq, l, r), Ty::Bool)),
                    hir::BinOp::Ne => Ok((cmp(CmpOp::Ne, l, r), Ty::Bool)),
                }
            }
            hir::ExprKind::Unary { op, operand } => {
                let (o, _) = self.lower_pure(*operand, out)?;
                match op {
                    hir::UnaryOp::NegI32 => Ok((
                        RExpr::Checked {
                            func: "a2x_neg",
                            args: vec![o],
                        },
                        Ty::I32,
                    )),
                    hir::UnaryOp::NotBool => Ok((RExpr::Not(Box::new(o)), Ty::Bool)),
                }
            }
            hir::ExprKind::Logical { op, lhs, rhs } => {
                // LHS always evaluates: its preludes go to the outer stream.
                let (l, _) = self.lower_pure(*lhs, out)?;
                // RHS is lazy: its preludes live inside the side block.
                let mut side = Vec::new();
                let (r, _) = self.lower_pure(*rhs, &mut side)?;
                let block = RBlock {
                    stmts: side,
                    value: r,
                };
                match op {
                    hir::LogicOp::And => Ok((
                        RExpr::AndSC {
                            lhs: Box::new(l),
                            rhs: Box::new(block),
                        },
                        Ty::Bool,
                    )),
                    hir::LogicOp::Or => Ok((
                        RExpr::OrSC {
                            lhs: Box::new(l),
                            rhs: Box::new(block),
                        },
                        Ty::Bool,
                    )),
                }
            }
            hir::ExprKind::Call {
                callee,
                eval_args,
                bindings,
            } => {
                let target_id = callee.0 as usize;
                if let Some(sym) = self.view.trace_symbol(target_id) {
                    if !eval_args.is_empty() {
                        return Err(A2xReject::new(
                            "a2x.internal",
                            "trace intrinsic call carries arguments",
                        ));
                    }
                    let func = match sym {
                        "hir.test.mark_a" => "a2x_mark_a",
                        _ => "a2x_mark_b",
                    };
                    return Ok((RExpr::Trace { func }, Ty::I32));
                }
                // Internal function or fragment external (signature-only).
                let (rust_name, param_count, result_ty) =
                    if let Some(t) = self.view.fn_by_def(target_id) {
                        (rust_fn_name(t.source_name), t.params.len(), t.result)
                    } else if let Some(x) = self.view.external_by_def(target_id) {
                        (rust_fn_name(&x.source_name), x.params.len(), x.result)
                    } else {
                        return Err(A2xReject::unsupported_expr(format!(
                            "call target def #{} is not an admitted function",
                            target_id
                        )));
                    };
                // Source order first, pinned once each.
                let mut slots: Vec<RExpr> = Vec::with_capacity(eval_args.len());
                for &arg in eval_args {
                    let (v, _) = self.lower_pure(arg, out)?;
                    slots.push(v);
                }
                // Placement by the binding map, in param order.
                let n = param_count;
                let mut placed: Vec<Option<RExpr>> = vec![None; n];
                for b in bindings {
                    let arg = slots.get(b.arg as usize).cloned().ok_or_else(|| {
                        A2xReject::new("a2x.internal", "binding arg index out of range")
                    })?;
                    let p = b.param as usize;
                    if p >= n {
                        return Err(A2xReject::new(
                            "a2x.internal",
                            "binding param index out of range",
                        ));
                    }
                    if placed[p].is_some() {
                        return Err(A2xReject::new("a2x.internal", "duplicate param binding"));
                    }
                    placed[p] = Some(arg);
                }
                let mut args = Vec::with_capacity(n);
                for (i, p) in placed.into_iter().enumerate() {
                    args.push(p.ok_or_else(|| {
                        A2xReject::new(
                            "a2x.internal",
                            format!(
                                "param #{} of call target def #{} left unbound",
                                i, target_id
                            ),
                        )
                    })?);
                }
                Ok((
                    RExpr::Call {
                        func: rust_name,
                        args,
                    },
                    result_ty,
                ))
            }
            other => Err(A2xReject::unsupported_expr(format!(
                "in `{}`: expression kind `{:?}` is outside the compute surface",
                self.facts.name, other
            ))),
        }
    }

    /// Lower a subexpression and pin its effectful root into a temporary so
    /// the caller receives a side-effect-free value (source-order, exactly
    /// once). Pure roots pass through unchanged.
    fn lower_pure(
        &mut self,
        id: hir::ExprId,
        out: &mut Vec<RStmt>,
    ) -> Result<(RExpr, Ty), A2xReject> {
        let (v, ty) = self.lower_expr(id, out)?;
        if v.needs_pin() {
            let t = self.fresh_temp();
            out.push(RStmt::Let {
                name: t.clone(),
                ty,
                mutable: false,
                init: v,
            });
            Ok((RExpr::Local(t), ty))
        } else {
            Ok((v, ty))
        }
    }
}

fn arith(func: &'static str, l: RExpr, r: RExpr) -> RExpr {
    RExpr::Checked {
        func,
        args: vec![l, r],
    }
}

fn cmp(op: CmpOp, l: RExpr, r: RExpr) -> RExpr {
    RExpr::Cmp {
        op,
        lhs: Box::new(l),
        rhs: Box::new(r),
    }
}

/// Collect every local that is an assignment target anywhere in the body
/// (Rust `mut` decision; defensive recursion with a depth guard).
fn collect_assignees(f: &FnFacts<'_>, into: &mut std::collections::BTreeSet<u32>) {
    for block in &f.body.blocks {
        for stmt in &block.stmts {
            if let hir::Stmt::Assign { place, .. } = stmt {
                if let Some(p) = f.body.places.get(place.0 as usize) {
                    into.insert(p.local.0);
                }
            }
        }
    }
}

pub(super) fn cmp_rust_str(op: CmpOp) -> &'static str {
    op.rust_str()
}

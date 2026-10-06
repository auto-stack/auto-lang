//! Atom text adapter: checked source frontend → Schema binder → verifier
//! (PLAN-745 T-03).
//!
//! Lowers a resolved + type-checked source program into core-i32 Atom text
//! and feeds it through the *existing* pipeline (`descriptor::bind` → span
//! re-anchoring → `verify::verify`). The binder stays the category/reference
//! binding entry point and the verifier the only way to obtain a
//! `CheckedModule`; this adapter never bypasses either.
//!
//! Deterministic encoding choices (plan §5.2/§5.3):
//! - every syntactic occurrence gets a fresh expr/block/place id — no
//!   sharing, no folding, no reordering;
//! - `true`/`false` encode as fresh `0 < 1` / `0 < 0` subgraphs whose
//!   synthesized spans point back at the literal token;
//! - `while cond { body }` lowers to `loop { if cond { body } else { break } }`
//!   so the condition is re-evaluated every iteration and after `continue`;
//!   the synthesized blocks/loop id project onto the `while` condition;
//! - statements after a terminator lower into separate unreferenced (dead)
//!   blocks — faithful to their unreachability; the verifier rejects them
//!   with `verify.block-structure` at the dead statement;
//! - a `break`/`continue` outside any loop targets a phantom loop emitted
//!   just before it in the same block, so the existing verifier owns the
//!   rejection (`verify.loop-target-scope`).
//!
//! Span re-anchoring: binder output spans live in generated-text
//! coordinates. Each table entry (type/def/body/local/place/expr/block) is
//! re-anchored to its intended source span by emission order, and every
//! emitted token records a fragment mapping used as the fallback for any
//! other generated position. Missing coverage is a controlled internal
//! diagnostic — never a panic, never a fabricated source error.

use super::ast::BinOp;
use super::resolve::{
    param_position, RBlock, RExprKind, RStmt, ResolvedExpr, ResolvedFn, ResolvedProgram, VT,
};
use super::typecheck;
use super::{SOURCE_PROFILE, TRACE_CAPABILITY};
use crate::atom_text::{Diagnostic, Span, Stage};
use crate::verify::CheckedModule;
use std::collections::BTreeMap;

/// Successful source compilation: the checked module plus the receipt data
/// traveling with the adapter result (profile identity, input content hash,
/// the generated internal Atom encoding).
#[derive(Clone, Debug)]
pub struct CompiledSource {
    pub profile: &'static str,
    /// FNV-1a 64-bit content hash of the input bytes (change detection;
    /// not cryptographic).
    pub input_hash: String,
    /// The generated Atom text (deterministic internal encoding, not a
    /// public interchange format).
    pub atom_text: String,
    pub checked: CheckedModule,
}

pub fn compile_source(
    src: &str,
    module_path: &str,
    capabilities: &[String],
) -> Result<CompiledSource, Vec<Diagnostic>> {
    let parts = compile_parts(src, module_path, capabilities)?;
    let checked = crate::verify::verify(parts.bundle)?;
    Ok(CompiledSource {
        profile: SOURCE_PROFILE,
        input_hash: parts.input_hash,
        atom_text: parts.atom_text,
        checked,
    })
}

/// Successful compilation up to (but not including) verification: the
/// span-projected bundle plus the adapter receipt data. This is the seam
/// the §6 mutation matrix exercises — a tampered bundle must go back
/// through the real verifier and can never reuse Checked credentials.
/// Production callers use [`compile_source`], which always verifies.
pub fn compile_parts(
    src: &str,
    module_path: &str,
    capabilities: &[String],
) -> Result<SourceParts, Vec<Diagnostic>> {
    let program = super::parse_program(src).map_err(|d| vec![d])?;
    let resolved = super::resolve::resolve(&program, capabilities)?;
    typecheck::typecheck(&resolved)?;

    let mut em = Emitter::new(&resolved, src.len());
    em.plan_module();
    let atom_text = em.render(module_path);

    let mut bundle = match crate::descriptor::bind(crate::descriptor(), &atom_text) {
        Ok(b) => b,
        Err(ds) => {
            let detail: Vec<String> = ds.iter().map(|d| d.render("atom", &atom_text)).collect();
            return Err(vec![Diagnostic::new(
                Stage::Text,
                "source.internal",
                format!(
                    "internal adapter error: generated Atom text failed to bind\n{}",
                    detail.join("\n")
                ),
                Span { start: 0, end: 0 },
            )]);
        }
    };
    em.reanchor_spans(&mut bundle, src).map_err(|d| vec![d])?;
    Ok(SourceParts {
        profile: SOURCE_PROFILE,
        input_hash: fnv1a_hex(src.as_bytes()),
        atom_text,
        bundle,
    })
}

/// Unverified compilation output (see [`compile_parts`]).
pub struct SourceParts {
    pub profile: &'static str,
    /// FNV-1a 64-bit content hash of the input bytes.
    pub input_hash: String,
    /// The generated Atom text (deterministic internal encoding).
    pub atom_text: String,
    /// The bound, span-projected bundle; verify it before any use.
    pub bundle: crate::hir::Bundle,
}

fn fnv1a_hex(bytes: &[u8]) -> String {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for &b in bytes {
        h ^= b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("{:016x}", h)
}

fn internal_diag(msg: String) -> Diagnostic {
    Diagnostic::new(
        Stage::Text,
        "source.internal",
        msg,
        Span { start: 0, end: 0 },
    )
}

// ---------------------------------------------------------------------------
// Planning: assign ids and build the lowered body shapes
// ---------------------------------------------------------------------------

#[derive(Clone, PartialEq, Eq, Debug)]
enum EKind {
    Const(i32),
    Read(usize),
    Bin(BinOp, usize, usize),
    Call {
        callee_fn: Option<usize>,
        callee_intrinsic: Option<&'static str>,
        eval: Vec<usize>,
        bindings: Vec<(u32, u32)>,
    },
}

struct ENode {
    kind: EKind,
    vt: VT,
    span: Span,
}

#[derive(Clone, Copy)]
enum SKind {
    Let {
        local: usize,
        value: usize,
    },
    Assign {
        place: usize,
        value: usize,
    },
    Return {
        value: usize,
    },
    If {
        cond: usize,
        then: usize,
        els: Option<usize>,
    },
    Loop {
        target: usize,
        body: usize,
    },
    Break {
        target: usize,
    },
    Continue {
        target: usize,
    },
}

struct SNode {
    kind: SKind,
    span: Span,
}

struct BNode {
    stmts: Vec<SNode>,
    span_override: Option<Span>,
    /// Statement index (inclusive) the block span covers; statements after
    /// the first terminator stay in the block for the dataflow walk but are
    /// dead, so they do not widen the block's diagnostic span.
    span_cut: Option<usize>,
    /// Fallback for blocks whose lowered statement list is empty.
    fallback: Span,
    span: Span,
}

struct PNode {
    local: usize,
    vt: VT,
    span: Span,
}

struct FnPlan {
    exprs: Vec<ENode>,
    blocks: Vec<BNode>,
    places: Vec<PNode>,
    /// Span of each registered loop id (while → condition, loop/phantom →
    /// the statement).
    loops: Vec<Span>,
}

/// Planning context: shares the program (immutable) and the intrinsic
/// first-use map (mutable) across functions without borrowing conflicts.
struct PlanCtx<'p> {
    program: &'p ResolvedProgram,
    intrinsic_first_use: &'p mut BTreeMap<&'static str, Span>,
}

impl PlanCtx<'_> {
    fn plan_fn(&mut self, f: &ResolvedFn) -> FnPlan {
        let mut plan = FnPlan {
            exprs: Vec::new(),
            blocks: Vec::new(),
            places: Vec::new(),
            loops: Vec::new(),
        };
        let entry = self.plan_block(&mut plan, f, &f.body);
        debug_assert_eq!(entry, 0);
        // Finalize block spans: override, else the join of the source spans
        // of the statements lowered into the block (cut at the first
        // terminator — trailing statements are dead), else the source
        // block's own span.
        for b in plan.blocks.iter_mut() {
            b.span = match b.span_override {
                Some(s) => s,
                None => {
                    let up_to = b
                        .span_cut
                        .map_or(b.stmts.len(), |c| (c + 1).min(b.stmts.len()));
                    b.stmts[..up_to]
                        .iter()
                        .map(|s| s.span)
                        .reduce(|a, c| a.join(c))
                        .unwrap_or(b.fallback)
                }
            };
        }
        plan
    }

    fn plan_block(&mut self, plan: &mut FnPlan, f: &ResolvedFn, rb: &RBlock) -> usize {
        let bid = plan.blocks.len();
        plan.blocks.push(BNode {
            stmts: Vec::new(),
            span_override: None,
            span_cut: None,
            fallback: rb.span,
            span: rb.span,
        });
        let mut cur = bid;
        let mut split_after = false;
        for s in &rb.stmts {
            let term = stmt_term(s);
            if split_after {
                // Dead code after a terminator (return, a valid
                // break/continue, or an if/loop whose every path left):
                // its own unreferenced block — faithful to its
                // unreachability, and the verifier rejects it with
                // `verify.block-structure` anchored AT the dead statement.
                // After a STRAY break/continue trailing statements stay in
                // the same block so the dataflow walk still runs and owns
                // the loop-target-scope rejection.
                cur = plan.blocks.len();
                // The dead block's span stays pinned to its FIRST dead
                // statement: the sharpest anchor for the verifier's
                // unreachable-block rejection.
                plan.blocks.push(BNode {
                    stmts: Vec::new(),
                    span_override: Some(stmt_span(s)),
                    span_cut: None,
                    fallback: stmt_span(s),
                    span: stmt_span(s),
                });
                split_after = false;
            }
            self.plan_stmt(plan, f, cur, s);
            if term != Term::No {
                if plan.blocks[cur].span_cut.is_none() {
                    let n = plan.blocks[cur].stmts.len();
                    plan.blocks[cur].span_cut = Some(n - 1);
                }
                split_after = matches!(term, Term::Ret | Term::Div);
            }
        }
        bid
    }

    /// Returns whether the statement terminates the block.
    fn plan_stmt(&mut self, plan: &mut FnPlan, f: &ResolvedFn, cur: usize, s: &RStmt) -> bool {
        match s {
            RStmt::Let { local, value, span } => {
                let v = self.plan_expr(plan, f, value);
                plan.blocks[cur].stmts.push(SNode {
                    kind: SKind::Let {
                        local: local.0,
                        value: v,
                    },
                    span: *span,
                });
                false
            }
            RStmt::Assign { local, value, span } => {
                let v = self.plan_expr(plan, f, value);
                let vt = f.locals[local.0].vt;
                let place = plan.places.len();
                plan.places.push(PNode {
                    local: local.0,
                    vt,
                    span: *span,
                });
                plan.blocks[cur].stmts.push(SNode {
                    kind: SKind::Assign { place, value: v },
                    span: *span,
                });
                false
            }
            RStmt::Return { value, span } => {
                let v = self.plan_expr(plan, f, value);
                plan.blocks[cur].stmts.push(SNode {
                    kind: SKind::Return { value: v },
                    span: *span,
                });
                true
            }
            RStmt::If {
                cond,
                then,
                els,
                span,
            } => {
                let c = self.plan_expr(plan, f, cond);
                let t = self.plan_block(plan, f, then);
                let e = els.as_ref().map(|b| self.plan_block(plan, f, b));
                plan.blocks[cur].stmts.push(SNode {
                    kind: SKind::If {
                        cond: c,
                        then: t,
                        els: e,
                    },
                    span: *span,
                });
                false
            }
            RStmt::While {
                cond,
                body,
                span,
                target,
            } => {
                // loop { if cond { body } else { break } }: the condition is
                // owned by the if inside the loop, so it re-runs every
                // iteration and after every continue.
                ensure_loop(plan, target.0, cond.span);
                let c = self.plan_expr(plan, f, cond);
                let k_body = self.plan_block(plan, f, body);
                let k_stop = plan.blocks.len();
                plan.blocks.push(BNode {
                    stmts: vec![SNode {
                        kind: SKind::Break { target: target.0 },
                        span: cond.span,
                    }],
                    span_override: Some(cond.span),
                    span_cut: None,
                    fallback: cond.span,
                    span: cond.span,
                });
                let k_test = plan.blocks.len();
                plan.blocks.push(BNode {
                    stmts: vec![SNode {
                        kind: SKind::If {
                            cond: c,
                            then: k_body,
                            els: Some(k_stop),
                        },
                        span: cond.span,
                    }],
                    span_override: Some(cond.span),
                    span_cut: None,
                    fallback: cond.span,
                    span: cond.span,
                });
                plan.blocks[cur].stmts.push(SNode {
                    kind: SKind::Loop {
                        target: target.0,
                        body: k_test,
                    },
                    span: *span,
                });
                false
            }
            RStmt::Loop { body, span, target } => {
                ensure_loop(plan, target.0, *span);
                let k_body = self.plan_block(plan, f, body);
                plan.blocks[cur].stmts.push(SNode {
                    kind: SKind::Loop {
                        target: target.0,
                        body: k_body,
                    },
                    span: *span,
                });
                false
            }
            RStmt::Break { target, span } => {
                let t = self.phantom_target(plan, cur, *target, *span);
                plan.blocks[cur].stmts.push(SNode {
                    kind: SKind::Break { target: t },
                    span: *span,
                });
                true
            }
            RStmt::Continue { target, span } => {
                let t = self.phantom_target(plan, cur, *target, *span);
                plan.blocks[cur].stmts.push(SNode {
                    kind: SKind::Continue { target: t },
                    span: *span,
                });
                true
            }
        }
    }

    /// Target id for a break/continue; stray ones get a phantom loop
    /// registered just before them in the same block so the existing
    /// verifier owns the rejection.
    fn phantom_target(
        &mut self,
        plan: &mut FnPlan,
        cur: usize,
        target: Option<super::resolve::LoopRef>,
        span: Span,
    ) -> usize {
        match target {
            Some(t) => t.0,
            None => {
                let ph = plan.loops.len();
                plan.loops.push(span);
                let k_empty = plan.blocks.len();
                plan.blocks.push(BNode {
                    stmts: Vec::new(),
                    span_override: Some(span),
                    span_cut: None,
                    fallback: span,
                    span: span,
                });
                plan.blocks[cur].stmts.push(SNode {
                    kind: SKind::Loop {
                        target: ph,
                        body: k_empty,
                    },
                    span,
                });
                ph
            }
        }
    }

    fn plan_expr(&mut self, plan: &mut FnPlan, f: &ResolvedFn, e: &ResolvedExpr) -> usize {
        match &e.kind {
            RExprKind::IntLit(v) => {
                plan.exprs.push(ENode {
                    kind: EKind::Const(*v),
                    vt: VT::I32,
                    span: e.span,
                });
                plan.exprs.len() - 1
            }
            RExprKind::BoolLit(v) => {
                // true = 0 < 1, false = 0 < 0 — a fresh subgraph per
                // occurrence, spans pointing at the literal token.
                plan.exprs.push(ENode {
                    kind: EKind::Const(0),
                    vt: VT::I32,
                    span: e.span,
                });
                let z = plan.exprs.len() - 1;
                plan.exprs.push(ENode {
                    kind: EKind::Const(i32::from(*v)),
                    vt: VT::I32,
                    span: e.span,
                });
                let o = plan.exprs.len() - 1;
                plan.exprs.push(ENode {
                    kind: EKind::Bin(BinOp::Lt, z, o),
                    vt: VT::Bool,
                    span: e.span,
                });
                plan.exprs.len() - 1
            }
            RExprKind::ReadLocal(l) => {
                plan.exprs.push(ENode {
                    kind: EKind::Read(l.0),
                    vt: f.locals[l.0].vt,
                    span: e.span,
                });
                plan.exprs.len() - 1
            }
            RExprKind::Binary { op, lhs, rhs } => {
                let l = self.plan_expr(plan, f, lhs);
                let r = self.plan_expr(plan, f, rhs);
                let vt = match op {
                    BinOp::Add | BinOp::Mul => VT::I32,
                    BinOp::Lt => VT::Bool,
                };
                plan.exprs.push(ENode {
                    kind: EKind::Bin(*op, l, r),
                    vt,
                    span: e.span,
                });
                plan.exprs.len() - 1
            }
            RExprKind::CallFn { fn_index, args } => {
                let mut eval = Vec::new();
                for a in args {
                    eval.push(self.plan_expr(plan, f, &a.value));
                }
                let mut bindings = Vec::new();
                if let Some(callee) = self.program.fns.get(*fn_index) {
                    for (idx, a) in args.iter().enumerate() {
                        if let Some(pos) = param_position(idx, a, callee) {
                            bindings.push((pos as u32, idx as u32));
                        }
                    }
                }
                plan.exprs.push(ENode {
                    kind: EKind::Call {
                        callee_fn: Some(*fn_index),
                        callee_intrinsic: None,
                        eval,
                        bindings,
                    },
                    vt: e.vt,
                    span: e.span,
                });
                plan.exprs.len() - 1
            }
            RExprKind::CallIntrinsic { symbol } => {
                self.intrinsic_first_use.entry(symbol).or_insert(e.span);
                plan.exprs.push(ENode {
                    kind: EKind::Call {
                        callee_fn: None,
                        callee_intrinsic: Some(symbol),
                        eval: Vec::new(),
                        bindings: Vec::new(),
                    },
                    vt: VT::I32,
                    span: e.span,
                });
                plan.exprs.len() - 1
            }
        }
    }
}

/// How a statement's execution leaves its block, mirroring the verifier's
/// conservative Flow semantics (used only to decide where dead code starts
/// so the verifier can anchor its rejection at the illegal trailing
/// statement; the verifier itself stays the authority).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Term {
    /// Control falls through to the next statement.
    No,
    /// All paths leave via `return`.
    Ret,
    /// All paths leave via a valid `break`/`continue` (or a stray one —
    /// still Diverged for the verifier's flow lattice).
    Div,
    /// A stray `break`/`continue` with no enclosing loop: a terminator for
    /// the walk, but trailing statements must stay in the SAME block so the
    /// dataflow walk still runs and owns the loop-target-scope rejection.
    Stray,
}

fn stmt_term(s: &RStmt) -> Term {
    match s {
        RStmt::Return { .. } => Term::Ret,
        RStmt::Break { target, .. } | RStmt::Continue { target, .. } => match target {
            Some(_) => Term::Div,
            None => Term::Stray,
        },
        RStmt::If { then, els, .. } => match els {
            // (Returned, Returned) -> Returned; (Diverged, Diverged) ->
            // Diverged; anything mixed (or no else) continues conservatively,
            // exactly like verify's flow lattice.
            Some(e) => match (block_flow(then), block_flow(e)) {
                (Some(Term::Ret), Some(Term::Ret)) => Term::Ret,
                (Some(Term::Div) | Some(Term::Stray), Some(Term::Div) | Some(Term::Stray)) => {
                    Term::Div
                }
                _ => Term::No,
            },
            None => Term::No,
        },
        RStmt::Loop { body, .. } => {
            if block_flow(body) == Some(Term::Ret) {
                Term::Ret
            } else {
                Term::No
            }
        }
        RStmt::While { .. } | RStmt::Let { .. } | RStmt::Assign { .. } => Term::No,
    }
}

/// First terminator of a block, if any (None = falls through).
fn block_flow(b: &RBlock) -> Option<Term> {
    b.stmts.iter().map(stmt_term).find(|t| *t != Term::No)
}

fn ensure_loop(plan: &mut FnPlan, id: usize, span: Span) {
    while plan.loops.len() <= id {
        plan.loops.push(span);
    }
    plan.loops[id] = span;
}

fn stmt_span(s: &RStmt) -> Span {
    match s {
        RStmt::Let { span, .. }
        | RStmt::Assign { span, .. }
        | RStmt::Return { span, .. }
        | RStmt::If { span, .. }
        | RStmt::While { span, .. }
        | RStmt::Loop { span, .. }
        | RStmt::Break { span, .. }
        | RStmt::Continue { span, .. } => *span,
    }
}

// ---------------------------------------------------------------------------
// Rendering: deterministic Atom text + fragment/entry span records
// ---------------------------------------------------------------------------

/// Entry spans recorded during rendering, aligned with the binder's table
/// order (the re-anchoring projection).
struct EmitSpans {
    types: Vec<Span>,
    defs: Vec<Span>,
    bodies: Vec<BodySpans>,
}

struct BodySpans {
    span: Span,
    locals: Vec<Span>,
    places: Vec<Span>,
    exprs: Vec<Span>,
    blocks: Vec<Span>,
}

struct Emitter<'p> {
    program: &'p ResolvedProgram,
    src_len: usize,
    plans: Vec<FnPlan>,
    intrinsic_first_use: BTreeMap<&'static str, Span>,
    /// Filled by [`Emitter::render`]: token-level fallback map.
    frags: Vec<(Span, Span)>,
    spans: Option<EmitSpans>,
}

impl<'p> Emitter<'p> {
    fn new(program: &'p ResolvedProgram, src_len: usize) -> Self {
        Emitter {
            program,
            src_len,
            plans: Vec::new(),
            intrinsic_first_use: BTreeMap::new(),
            frags: Vec::new(),
            spans: None,
        }
    }

    fn plan_module(&mut self) {
        for f in &self.program.fns {
            let mut ctx = PlanCtx {
                program: self.program,
                intrinsic_first_use: &mut self.intrinsic_first_use,
            };
            let plan = ctx.plan_fn(f);
            self.plans.push(plan);
        }
    }

    fn ty_name(vt: VT) -> &'static str {
        match vt {
            VT::I32 => "t_i32",
            VT::Bool => "t_bool",
        }
    }

    fn esc(s: &str) -> String {
        let mut out = String::new();
        for c in s.chars() {
            match c {
                '"' => out.push_str("\\\""),
                '\\' => out.push_str("\\\\"),
                '\n' => out.push_str("\\n"),
                '\t' => out.push_str("\\t"),
                _ => out.push(c),
            }
        }
        out
    }

    fn render(&mut self, module_path: &str) -> String {
        let program_span = Span {
            start: 0,
            end: self.src_len,
        };
        let mut t = String::new();
        let mut frags = Vec::new();
        let mut spans = EmitSpans {
            types: Vec::new(),
            defs: Vec::new(),
            bodies: Vec::new(),
        };

        macro_rules! frag {
            ($text:expr, $src:expr) => {{
                let start = t.len();
                t.push_str($text);
                frags.push((
                    Span {
                        start,
                        end: t.len(),
                    },
                    $src,
                ));
            }};
        }
        macro_rules! nl {
            ($indent:expr) => {{
                t.push('\n');
                for _ in 0..$indent {
                    t.push(' ');
                }
            }};
        }

        let use_intrinsics: Vec<&'static str> = self.intrinsic_first_use.keys().copied().collect();

        frag!(
            "hir(schema: \"auto.hir.core.draft\", revision: 1u32, profile: \"core-i32-draft\")",
            program_span
        );
        t.push(' ');
        frag!("{", program_span);
        nl!(4);

        if !use_intrinsics.is_empty() {
            let req_span = self.intrinsic_first_use[use_intrinsics[0]];
            frag!(&format!("requires: [\"{}\"]", TRACE_CAPABILITY), req_span);
            nl!(4);
        }

        frag!("module m_src", program_span);
        t.push(' ');
        frag!("{", program_span);
        nl!(8);
        frag!(
            &format!("path: \"{}\"", Self::esc(module_path)),
            program_span
        );
        nl!(8);

        // -- types ----------------------------------------------------------
        frag!("types", program_span);
        t.push(' ');
        frag!("{", program_span);
        nl!(12);
        frag!("type t_i32 i32 {}", program_span);
        spans.types.push(program_span);
        nl!(12);
        frag!("type t_bool bool {}", program_span);
        spans.types.push(program_span);
        nl!(12);
        for f in &self.program.fns {
            let params: Vec<String> = f
                .params
                .iter()
                .map(|p| format!("\"{}\"", Self::ty_name(*p)))
                .collect();
            frag!(
                &format!(
                    "type t_f_{} function {{ params: [{}]; result: \"{}\" }}",
                    f.name,
                    params.join(", "),
                    Self::ty_name(f.result)
                ),
                f.item_span
            );
            spans.types.push(f.item_span);
            nl!(12);
        }
        for sym in &use_intrinsics {
            let span = self.intrinsic_first_use[*sym];
            frag!(
                &format!(
                    "type {} function {{ params: []; result: \"t_i32\" }}",
                    intrinsic_type_id(sym)
                ),
                span
            );
            spans.types.push(span);
            nl!(12);
        }
        frag!("}", program_span);
        nl!(8);

        // -- declarations -----------------------------------------------------
        frag!("declarations", program_span);
        t.push(' ');
        frag!("{", program_span);
        nl!(12);
        for f in &self.program.fns {
            // The declared `name` is the object-file symbol. Source
            // functions carry an `sf_` prefix so the native namespace stays
            // isolated from the backend's fixed imports/exports
            // (`ExitProcess`, `ac_start`): a source function may legally
            // carry either name (QA-02) without a symbol-table collision.
            // Identity stays in the def id (`d_<name>`), so resolution,
            // recursion and the CLI's entry-by-name lookup are unaffected.
            frag!(
                &format!(
                    "function d_{} t_f_{} {{ name: \"sf_{}\"; body: \"b_{}\" }}",
                    f.name,
                    f.name,
                    Self::esc(&f.name),
                    f.name
                ),
                f.item_span
            );
            spans.defs.push(f.item_span);
            nl!(12);
        }
        for sym in &use_intrinsics {
            let span = self.intrinsic_first_use[*sym];
            frag!(
                &format!(
                    "intrinsic {} {} {{ symbol: \"{}\"; effect: observable }}",
                    intrinsic_def_id(sym),
                    intrinsic_type_id(sym),
                    sym
                ),
                span
            );
            spans.defs.push(span);
            nl!(12);
        }
        frag!("}", program_span);
        nl!(8);

        // -- bodies ------------------------------------------------------------
        frag!("bodies", program_span);
        t.push(' ');
        frag!("{", program_span);
        nl!(12);
        for (fi, f) in self.program.fns.iter().enumerate() {
            let plan = &self.plans[fi];
            let mut bs = BodySpans {
                span: f.name_span,
                locals: Vec::new(),
                places: Vec::new(),
                exprs: Vec::new(),
                blocks: Vec::new(),
            };
            frag!(&format!("body b_{} {{", f.name), f.name_span);
            nl!(16);
            frag!(&format!("owner: \"d_{}\"", f.name), f.name_span);
            nl!(16);
            frag!("entry: \"k0\"", f.name_span);
            nl!(16);

            // locals
            frag!("locals", f.name_span);
            t.push(' ');
            frag!("{", f.name_span);
            nl!(20);
            for (li, l) in f.locals.iter().enumerate() {
                let line = if l.is_param {
                    format!(
                        "local l{} {} {{ name: \"{}\"; role: param; index: {}u32 }}",
                        li,
                        Self::ty_name(l.vt),
                        Self::esc(&l.name),
                        l.param_index.unwrap_or(0)
                    )
                } else {
                    format!(
                        "local l{} {} {{ name: \"{}\"; role: binding; mutable: {} }}",
                        li,
                        Self::ty_name(l.vt),
                        Self::esc(&l.name),
                        l.mutable
                    )
                };
                frag!(&line, l.span);
                bs.locals.push(l.span);
                nl!(20);
            }
            frag!("}", f.name_span);
            nl!(16);

            // places
            if !plan.places.is_empty() {
                frag!("places", f.name_span);
                t.push(' ');
                frag!("{", f.name_span);
                nl!(20);
                for (pi, p) in plan.places.iter().enumerate() {
                    frag!(
                        &format!(
                            "place p{} local {{ local: l{}; type: {}; access: mutable }}",
                            pi,
                            p.local,
                            Self::ty_name(p.vt)
                        ),
                        p.span
                    );
                    bs.places.push(p.span);
                    nl!(20);
                }
                frag!("}", f.name_span);
                nl!(16);
            }

            // expressions
            if !plan.exprs.is_empty() {
                frag!("expressions", f.name_span);
                t.push(' ');
                frag!("{", f.name_span);
                nl!(20);
                for (ei, e) in plan.exprs.iter().enumerate() {
                    let ty = Self::ty_name(e.vt);
                    let line = match &e.kind {
                        EKind::Const(v) => {
                            format!("expr e{} constant {{ type: {}; value: {}i32 }}", ei, ty, v)
                        }
                        EKind::Read(l) => {
                            format!("expr e{} read_local {{ type: {}; local: l{} }}", ei, ty, l)
                        }
                        EKind::Bin(op, l, r) => {
                            let (ops, overflow) = match op {
                                BinOp::Add => ("add_i32", "; overflow: trap"),
                                BinOp::Mul => ("mul_i32", "; overflow: trap"),
                                BinOp::Lt => ("lt_i32", ""),
                            };
                            format!(
                                "expr e{} binary {{ type: {}; op: {}{}; lhs: e{}; rhs: e{} }}",
                                ei, ty, ops, overflow, l, r
                            )
                        }
                        EKind::Call {
                            callee_fn,
                            callee_intrinsic,
                            eval,
                            bindings,
                        } => {
                            let callee = match (callee_fn, callee_intrinsic) {
                                (Some(i), _) => format!("d_{}", self.program.fns[*i].name),
                                (None, Some(s)) => intrinsic_def_id(s).to_string(),
                                _ => "d_unknown".to_string(),
                            };
                            let eval_s: Vec<String> =
                                eval.iter().map(|x| format!("e{}", x)).collect();
                            let bind_s: Vec<String> = bindings
                                .iter()
                                .map(|(p, a)| format!("{{ param: {}u32, arg: {}u32 }}", p, a))
                                .collect();
                            format!(
                                "expr e{} call {{ type: {}; callee: {}; eval_args: [{}]; bindings: [{}] }}",
                                ei,
                                ty,
                                callee,
                                eval_s.join(", "),
                                bind_s.join(", ")
                            )
                        }
                    };
                    frag!(&line, e.span);
                    bs.exprs.push(e.span);
                    nl!(20);
                }
                frag!("}", f.name_span);
                nl!(16);
            }

            // blocks
            frag!("blocks", f.name_span);
            t.push(' ');
            frag!("{", f.name_span);
            nl!(20);
            for (bi, b) in plan.blocks.iter().enumerate() {
                frag!(&format!("block k{} {{", bi), b.span);
                nl!(24);
                for s in &b.stmts {
                    let line = match &s.kind {
                        SKind::Let { local, value } => {
                            format!("let(local: l{}, value: e{}) {{}}", local, value)
                        }
                        SKind::Assign { place, value } => {
                            format!("assign(place: p{}, value: e{}) {{}}", place, value)
                        }
                        SKind::Return { value } => format!("return(value: e{}) {{}}", value),
                        SKind::If { cond, then, els } => match els {
                            Some(e) => {
                                format!("if(cond: e{}, then: k{}, else: k{}) {{}}", cond, then, e)
                            }
                            None => format!("if(cond: e{}, then: k{}) {{}}", cond, then),
                        },
                        SKind::Loop { target, body } => {
                            format!("loop(id: q{}, body: k{}) {{}}", target, body)
                        }
                        SKind::Break { target } => format!("break(target: q{}) {{}}", target),
                        SKind::Continue { target } => {
                            format!("continue(target: q{}) {{}}", target)
                        }
                    };
                    frag!(&line, s.span);
                    nl!(24);
                }
                frag!("}", b.span);
                bs.blocks.push(b.span);
                nl!(20);
            }
            frag!("}", f.name_span);
            nl!(16);

            frag!("}", f.name_span);
            nl!(12);
            spans.bodies.push(bs);
        }
        frag!("}", program_span);
        nl!(8);
        frag!("}", program_span);
        nl!(4);
        frag!("}", program_span);
        t.push('\n');

        self.frags = frags;
        self.spans = Some(spans);
        t
    }

    /// Re-anchor every binder-produced span to its intended source span
    /// (entry order), with the token fragment map as fallback, then assert
    /// the §5.3 post-conditions (in-bounds, UTF-8 boundaries, no generated
    /// positions left).
    fn reanchor_spans(&self, bundle: &mut crate::hir::Bundle, src: &str) -> Result<(), Diagnostic> {
        let Some(spans) = &self.spans else {
            return Err(internal_diag("render output missing".to_string()));
        };
        let frag_project = |s: Span| -> Option<Span> {
            let mut acc: Option<Span> = None;
            for (g, o) in &self.frags {
                if g.end <= s.start {
                    continue;
                }
                if g.start >= s.end {
                    break;
                }
                acc = Some(acc.map_or(*o, |p: Span| p.join(*o)));
            }
            acc
        };
        let check = |s: Span| -> Result<Span, Diagnostic> {
            if s.start > src.len() || s.end > src.len() {
                return Err(internal_diag(format!(
                    "span {:?} escapes the source after projection",
                    s
                )));
            }
            if !src.is_char_boundary(s.start) || !src.is_char_boundary(s.end) {
                return Err(internal_diag(format!(
                    "span {:?} is not on UTF-8 boundaries after projection",
                    s
                )));
            }
            Ok(s)
        };

        let mismatch = |what: &str, expect: usize, got: usize| -> Option<Diagnostic> {
            if expect != got {
                Some(internal_diag(format!(
                    "generated table size mismatch for {}: {} emitted, {} bound",
                    what, expect, got
                )))
            } else {
                None
            }
        };

        // Single module per this profile's emission.
        let m = match bundle.modules.get_mut(0) {
            Some(m) => m,
            None => {
                return Err(internal_diag("binder produced no module".to_string()));
            }
        };
        if let Some(d) = mismatch("types", spans.types.len(), m.types.len())
            .or_else(|| mismatch("defs", spans.defs.len(), m.defs.len()))
            .or_else(|| mismatch("bodies", spans.bodies.len(), m.bodies.len()))
        {
            return Err(d);
        }

        m.span = check(frag_project(m.span).unwrap_or(m.span))?;
        for (i, ty) in m.types.iter_mut().enumerate() {
            ty.span = check(spans.types[i])?;
        }
        for (i, d) in m.defs.iter_mut().enumerate() {
            d.span = check(spans.defs[i])?;
        }
        for (j, (b, eb)) in m.bodies.iter_mut().zip(spans.bodies.iter()).enumerate() {
            let _ = j;
            if let Some(d) = mismatch("locals", eb.locals.len(), b.locals.len())
                .or_else(|| mismatch("places", eb.places.len(), b.places.len()))
                .or_else(|| mismatch("exprs", eb.exprs.len(), b.exprs.len()))
                .or_else(|| mismatch("blocks", eb.blocks.len(), b.blocks.len()))
            {
                return Err(d);
            }
            b.span = check(eb.span)?;
            for (i, l) in b.locals.iter_mut().enumerate() {
                l.span = check(eb.locals[i])?;
            }
            for (i, p) in b.places.iter_mut().enumerate() {
                p.span = check(eb.places[i])?;
            }
            for (i, e) in b.exprs.iter_mut().enumerate() {
                e.span = check(eb.exprs[i])?;
            }
            for (i, bl) in b.blocks.iter_mut().enumerate() {
                bl.span = check(eb.blocks[i])?;
            }
        }
        Ok(())
    }
}

fn intrinsic_type_id(sym: &str) -> String {
    format!("t_i_{}", sym.rsplit('.').next().unwrap_or("x"))
}

fn intrinsic_def_id(sym: &str) -> String {
    format!("i_{}", sym.rsplit('.').next().unwrap_or("x"))
}

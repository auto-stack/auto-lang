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

use super::ast::{BinOp, UnaryOp};
use super::resolve::{
    param_position, RBlock, RExprKind, RStmt, ResolvedExpr, ResolvedFn, ResolvedProgram, VT,
};
use super::typecheck;
use super::{SourceProfile, SOURCE_PROFILE, TRACE_CAPABILITY};
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
    let parts = compile_parts_with(src, module_path, capabilities, SourceProfile::Core)?;
    let checked = crate::verify::verify(parts.bundle)?;
    Ok(CompiledSource {
        profile: SOURCE_PROFILE,
        input_hash: parts.input_hash,
        atom_text: parts.atom_text,
        checked,
    })
}

/// Compile under the scalar-i32 source profile (PLAN-767): parse/resolve/
/// typecheck on the scalar surface, bind against the scalar descriptor and
/// verify through the scalar entry. Receipt data mirrors [`compile_source`]
/// with the scalar profile identity.
pub fn compile_scalar_source(
    src: &str,
    module_path: &str,
    capabilities: &[String],
) -> Result<CompiledSource, Vec<Diagnostic>> {
    let parts = compile_parts_with(src, module_path, capabilities, SourceProfile::Scalar)?;
    let checked = crate::verify::verify_scalar(parts.bundle)?;
    Ok(CompiledSource {
        profile: super::SCALAR_SOURCE_PROFILE,
        input_hash: parts.input_hash,
        atom_text: parts.atom_text,
        checked,
    })
}

/// Compile up to (but not including) verification under the explicit
/// profile: the span-projected bundle plus the adapter receipt data. This
/// is the seam the §6 mutation matrix exercises — a tampered bundle must
/// go back through the real verifier of ITS domain and can never reuse
/// Checked credentials.
pub fn compile_parts_with(
    src: &str,
    module_path: &str,
    capabilities: &[String],
    profile: SourceProfile,
) -> Result<SourceParts, Vec<Diagnostic>> {
    let program = super::parse_program_with(src, profile).map_err(|d| vec![d])?;
    let resolved = super::resolve_with(&program, capabilities, profile)?;
    typecheck::typecheck_with(&resolved, profile)?;

    let mut em = Emitter::new(&resolved, src.len(), profile);
    em.plan_module();
    let atom_text = em.render(module_path);

    let descriptor = match profile.hir_domain() {
        crate::hir::HirDomain::Core => crate::descriptor(),
        crate::hir::HirDomain::Scalar => crate::scalar_descriptor(),
        crate::hir::HirDomain::Record => crate::record_descriptor(),
    };
    let mut bundle = match crate::descriptor::bind(descriptor, &atom_text) {
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
        profile: profile.as_str(),
        input_hash: fnv1a_hex(src.as_bytes()),
        atom_text,
        bundle,
    })
}

/// Unverified core-profile compilation output (see [`compile_parts_with`]);
/// the 745-era mutation-matrix test seam.
pub fn compile_parts(
    src: &str,
    module_path: &str,
    capabilities: &[String],
) -> Result<SourceParts, Vec<Diagnostic>> {
    compile_parts_with(src, module_path, capabilities, SourceProfile::Core)
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
    /// Lazy boolean short-circuit (PLAN-767): `&&`/`||` surface Binary
    /// nodes lower to the HIR `Logical` kind — never to an eager binary.
    Logical(crate::hir::LogicOp, usize, usize),
    Unary(crate::hir::UnaryOp, usize),
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
        // Termination classification for every block, computed bottom-up
        // with an explicit stack (R6: block nesting is unlimited; the
        // previous recursive stmt_term/block_flow walked per level and
        // revisited shared spines, which was both a depth and an O(n²)
        // hazard).
        let terms = block_terms(&f.body);
        let entry = self.plan_block_machine(&mut plan, f, &f.body, &terms);
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

    /// Plan one function body with an explicit block-frame stack (R6: no
    /// recursion, unlimited nesting). Each frame owns the plan-side block
    /// it is filling (`cur`, switching to a dead block after a terminator);
    /// closing a frame assembles the if/while/loop statement into its
    /// parent's home block. Block-id allocation order matches the previous
    /// recursive walk exactly (then before else, body before the while
    /// test/stop blocks).
    fn plan_block_machine(
        &mut self,
        plan: &mut FnPlan,
        f: &ResolvedFn,
        root: &RBlock,
        terms: &BTreeMap<usize, Option<Term>>,
    ) -> usize {
        struct PFrame<'a> {
            src: &'a RBlock,
            idx: usize,
            /// Plan-side block allocated for this source block — the arm a
            /// closing continuation assembles.
            bid: usize,
            /// Current fill target; switches to a dead block after a
            /// terminator.
            cur: usize,
            split_after: bool,
            /// Termination of the block-opening statement currently being
            /// planned (if/while/loop): its SNode lands only when the child
            /// frame closes, so the span_cut/split_after bookkeeping waits
            /// for that close (the recursive form got this for free — the
            /// recursion completed before the caller's term check ran).
            pending_term: Option<Term>,
            cont: PCont<'a>,
        }
        enum PCont<'a> {
            FnBody,
            /// The then-arm closed; `els` is the pending else source block.
            ThenArm {
                cond: usize,
                els: Option<&'a RBlock>,
                home: usize,
                span: Span,
            },
            ElseArm {
                cond: usize,
                then: usize,
                home: usize,
                span: Span,
            },
            WhileArm {
                cond: usize,
                home: usize,
                /// The while condition's span (k_stop/k_test anchors).
                cond_span: Span,
                /// The whole while statement's span (the loop SNode anchor).
                stmt_span: Span,
                target: usize,
            },
            LoopArm {
                home: usize,
                span: Span,
                target: usize,
            },
        }
        let entry = plan.blocks.len();
        plan.blocks.push(BNode {
            stmts: Vec::new(),
            span_override: None,
            span_cut: None,
            fallback: root.span,
            span: root.span,
        });
        let mut stack: Vec<PFrame> = vec![PFrame {
            src: root,
            idx: 0,
            bid: entry,
            cur: entry,
            split_after: false,
            pending_term: None,
            cont: PCont::FnBody,
        }];
        loop {
            let action = {
                let frame = stack.last_mut().expect("plan frame");
                let src: &RBlock = frame.src;
                let next = src.stmts.get(frame.idx);
                if next.is_some() {
                    frame.idx += 1;
                }
                next
            };
            let Some(s) = action else {
                let frame = stack.pop().expect("plan frame");
                let bid = frame.bid;
                // Whether this close lands the pending if/while/loop SNode
                // in its parent (the else-arm push defers it one more
                // frame).
                let mut landed = true;
                match frame.cont {
                    PCont::FnBody => {}
                    PCont::ThenArm {
                        cond,
                        els,
                        home,
                        span,
                    } => match els {
                        Some(eb) => {
                            let k_else = plan.blocks.len();
                            plan.blocks.push(BNode {
                                stmts: Vec::new(),
                                span_override: None,
                                span_cut: None,
                                fallback: eb.span,
                                span: eb.span,
                            });
                            stack.push(PFrame {
                                src: eb,
                                idx: 0,
                                bid: k_else,
                                cur: k_else,
                                split_after: false,
                                pending_term: None,
                                cont: PCont::ElseArm {
                                    cond,
                                    then: bid,
                                    home,
                                    span,
                                },
                            });
                            landed = false;
                        }
                        None => {
                            plan.blocks[home].stmts.push(SNode {
                                kind: SKind::If {
                                    cond,
                                    then: bid,
                                    els: None,
                                },
                                span,
                            });
                        }
                    },
                    PCont::ElseArm {
                        cond,
                        then,
                        home,
                        span,
                    } => {
                        plan.blocks[home].stmts.push(SNode {
                            kind: SKind::If {
                                cond,
                                then,
                                els: Some(bid),
                            },
                            span,
                        });
                    }
                    PCont::WhileArm {
                        cond,
                        home,
                        cond_span,
                        stmt_span,
                        target,
                    } => {
                        let k_body = bid;
                        let k_stop = plan.blocks.len();
                        plan.blocks.push(BNode {
                            stmts: vec![SNode {
                                kind: SKind::Break { target },
                                span: cond_span,
                            }],
                            span_override: Some(cond_span),
                            span_cut: None,
                            fallback: cond_span,
                            span: cond_span,
                        });
                        let k_test = plan.blocks.len();
                        plan.blocks.push(BNode {
                            stmts: vec![SNode {
                                kind: SKind::If {
                                    cond,
                                    then: k_body,
                                    els: Some(k_stop),
                                },
                                span: cond_span,
                            }],
                            span_override: Some(cond_span),
                            span_cut: None,
                            fallback: cond_span,
                            span: cond_span,
                        });
                        plan.blocks[home].stmts.push(SNode {
                            kind: SKind::Loop {
                                target,
                                body: k_test,
                            },
                            span: stmt_span,
                        });
                    }
                    PCont::LoopArm { home, span, target } => {
                        plan.blocks[home].stmts.push(SNode {
                            kind: SKind::Loop { target, body: bid },
                            span,
                        });
                    }
                }
                // Only the FnBody frame closes with no parent: the whole
                // function body is planned.
                if stack.is_empty() {
                    return entry;
                }
                // The pending if/while/loop statement's SNode just landed
                // in the parent's home block: apply its termination
                // bookkeeping now (span_cut counts the SNode, exactly like
                // the recursive form did after plan_stmt returned).
                if landed {
                    let parent = stack.last_mut().expect("plan frame");
                    let home = parent.cur;
                    if let Some(term) = parent.pending_term.take() {
                        if term != Term::No {
                            if plan.blocks[home].span_cut.is_none() {
                                let n = plan.blocks[home].stmts.len();
                                plan.blocks[home].span_cut = Some(n - 1);
                            }
                            parent.split_after = matches!(term, Term::Ret | Term::Div);
                        }
                    }
                }
                continue;
            };
            // Termination bookkeeping. Simple statements land their SNode
            // synchronously; block-opening statements (if/while/loop) land
            // it when the child frame closes, so their bookkeeping is
            // deferred (pending_term) to that close.
            let term = stmt_term_memo(s, terms);
            let opens_block = matches!(
                s,
                RStmt::If { .. } | RStmt::While { .. } | RStmt::Loop { .. }
            );
            // captured BEFORE the match: block statements push child frames
            // on top of this one
            let parent_idx = stack.len() - 1;
            let frame = stack.last_mut().expect("plan frame");
            if frame.split_after {
                // Dead code after a terminator (return, a valid
                // break/continue, or an if/loop whose every path left):
                // its own unreferenced block — faithful to its
                // unreachability, and the verifier rejects it with
                // `verify.block-structure` anchored AT the dead statement.
                // After a STRAY break/continue trailing statements stay in
                // the same block so the dataflow walk still runs and owns
                // the loop-target-scope rejection.
                frame.cur = plan.blocks.len();
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
                frame.split_after = false;
            }
            let home = frame.cur;
            match s {
                RStmt::Let { local, value, span } => {
                    let v = self.plan_expr(plan, f, value);
                    plan.blocks[home].stmts.push(SNode {
                        kind: SKind::Let {
                            local: local.0,
                            value: v,
                        },
                        span: *span,
                    });
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
                    plan.blocks[home].stmts.push(SNode {
                        kind: SKind::Assign { place, value: v },
                        span: *span,
                    });
                }
                RStmt::Return { value, span } => {
                    let v = self.plan_expr(plan, f, value);
                    plan.blocks[home].stmts.push(SNode {
                        kind: SKind::Return { value: v },
                        span: *span,
                    });
                }
                RStmt::If {
                    cond,
                    then,
                    els,
                    span,
                } => {
                    let c = self.plan_expr(plan, f, cond);
                    let k_then = plan.blocks.len();
                    plan.blocks.push(BNode {
                        stmts: Vec::new(),
                        span_override: None,
                        span_cut: None,
                        fallback: then.span,
                        span: then.span,
                    });
                    stack.push(PFrame {
                        src: then,
                        idx: 0,
                        bid: k_then,
                        cur: k_then,
                        split_after: false,
                        pending_term: None,
                        cont: PCont::ThenArm {
                            cond: c,
                            els: els.as_ref(),
                            home,
                            span: *span,
                        },
                    });
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
                    let k_body = plan.blocks.len();
                    plan.blocks.push(BNode {
                        stmts: Vec::new(),
                        span_override: None,
                        span_cut: None,
                        fallback: body.span,
                        span: body.span,
                    });
                    stack.push(PFrame {
                        src: body,
                        idx: 0,
                        bid: k_body,
                        cur: k_body,
                        split_after: false,
                        pending_term: None,
                        cont: PCont::WhileArm {
                            cond: c,
                            home,
                            cond_span: cond.span,
                            stmt_span: *span,
                            target: target.0,
                        },
                    });
                }
                RStmt::Loop { body, span, target } => {
                    ensure_loop(plan, target.0, *span);
                    let k_body = plan.blocks.len();
                    plan.blocks.push(BNode {
                        stmts: Vec::new(),
                        span_override: None,
                        span_cut: None,
                        fallback: body.span,
                        span: body.span,
                    });
                    stack.push(PFrame {
                        src: body,
                        idx: 0,
                        bid: k_body,
                        cur: k_body,
                        split_after: false,
                        pending_term: None,
                        cont: PCont::LoopArm {
                            home,
                            span: *span,
                            target: target.0,
                        },
                    });
                }
                RStmt::Break { target, span } => {
                    let t = self.phantom_target(plan, home, *target, *span);
                    plan.blocks[home].stmts.push(SNode {
                        kind: SKind::Break { target: t },
                        span: *span,
                    });
                }
                RStmt::Continue { target, span } => {
                    let t = self.phantom_target(plan, home, *target, *span);
                    plan.blocks[home].stmts.push(SNode {
                        kind: SKind::Continue { target: t },
                        span: *span,
                    });
                }
            }
            if term != Term::No {
                let frame = &mut stack[parent_idx];
                if opens_block {
                    frame.pending_term = Some(term);
                } else {
                    if plan.blocks[home].span_cut.is_none() {
                        let n = plan.blocks[home].stmts.len();
                        plan.blocks[home].span_cut = Some(n - 1);
                    }
                    frame.split_after = matches!(term, Term::Ret | Term::Div);
                }
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
        // Iterative planning (R3-QA-02): legal operator chains are
        // unbounded in depth, so no recursion. Leaves plan through
        // `plan_leaf`; binary nodes combine pre-allocated expr indices
        // after both operands are planned (lhs first — same ordering as
        // the previous recursive form).
        enum Task<'a> {
            Plan {
                node: &'a ResolvedExpr,
                slot: usize,
            },
            /// nested-call assembly: all argument slots planned, build the
            /// call node (R5: calls must not recurse through plan_leaf)
            CallOut {
                node: &'a ResolvedExpr,
                dest: usize,
                arg_slots: Vec<usize>,
            },
            Combine {
                op: BinOp,
                span: Span,
                lslot: usize,
                rslot: usize,
                dest: usize,
            },
            /// operand slot planned; build the unary node
            UnaryOut {
                op: UnaryOp,
                span: Span,
                oslot: usize,
                dest: usize,
            },
        }
        // slot -> planned expr table index
        let mut slot_idx: Vec<usize> = vec![usize::MAX];
        let mut stack: Vec<Task> = vec![Task::Plan { node: e, slot: 0 }];
        while let Some(task) = stack.pop() {
            match task {
                Task::Plan { node, slot } => {
                    if let RExprKind::CallFn { args, .. } = &node.kind {
                        if !args.is_empty() {
                            let mut arg_slots = Vec::with_capacity(args.len());
                            let mut subtasks: Vec<Task> = Vec::with_capacity(args.len() + 1);
                            for a in args {
                                let s = slot_idx.len();
                                slot_idx.push(usize::MAX);
                                arg_slots.push(s);
                                subtasks.push(Task::Plan {
                                    node: &a.value,
                                    slot: s,
                                });
                            }
                            subtasks.push(Task::CallOut {
                                node,
                                dest: slot,
                                arg_slots,
                            });
                            for t in subtasks.into_iter().rev() {
                                stack.push(t);
                            }
                            continue;
                        }
                    }
                    if let RExprKind::Binary { op, lhs, rhs } = &node.kind {
                        let lslot = slot_idx.len();
                        slot_idx.push(usize::MAX);
                        let rslot = slot_idx.len();
                        slot_idx.push(usize::MAX);
                        stack.push(Task::Combine {
                            op: *op,
                            span: node.span,
                            lslot,
                            rslot,
                            dest: slot,
                        });
                        stack.push(Task::Plan {
                            node: rhs,
                            slot: rslot,
                        });
                        stack.push(Task::Plan {
                            node: lhs,
                            slot: lslot,
                        });
                    } else if let RExprKind::Unary { op, operand } = &node.kind {
                        let oslot = slot_idx.len();
                        slot_idx.push(usize::MAX);
                        stack.push(Task::UnaryOut {
                            op: *op,
                            span: node.span,
                            oslot,
                            dest: slot,
                        });
                        stack.push(Task::Plan {
                            node: operand,
                            slot: oslot,
                        });
                    } else {
                        slot_idx[slot] = self.plan_leaf(plan, f, node);
                    }
                }
                Task::CallOut {
                    node,
                    dest,
                    arg_slots,
                } => {
                    let RExprKind::CallFn { fn_index, args } = &node.kind else {
                        unreachable!("CallOut on non-call node")
                    };
                    let mut eval = Vec::with_capacity(arg_slots.len());
                    for s in &arg_slots {
                        eval.push(slot_idx[*s]);
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
                        vt: node.vt,
                        span: node.span,
                    });
                    slot_idx[dest] = plan.exprs.len() - 1;
                }
                Task::Combine {
                    op,
                    span,
                    lslot,
                    rslot,
                    dest,
                } => {
                    let l = slot_idx[lslot];
                    let r = slot_idx[rslot];
                    match op {
                        BinOp::And => {
                            plan.exprs.push(ENode {
                                kind: EKind::Logical(crate::hir::LogicOp::And, l, r),
                                vt: VT::Bool,
                                span,
                            });
                            slot_idx[dest] = plan.exprs.len() - 1;
                        }
                        BinOp::Or => {
                            plan.exprs.push(ENode {
                                kind: EKind::Logical(crate::hir::LogicOp::Or, l, r),
                                vt: VT::Bool,
                                span,
                            });
                            slot_idx[dest] = plan.exprs.len() - 1;
                        }
                        _ => {
                            let vt = super::resolve::bin_op_vt(op);
                            plan.exprs.push(ENode {
                                kind: EKind::Bin(op, l, r),
                                vt,
                                span,
                            });
                            slot_idx[dest] = plan.exprs.len() - 1;
                        }
                    }
                }
                Task::UnaryOut {
                    op,
                    span,
                    oslot,
                    dest,
                } => {
                    let o = slot_idx[oslot];
                    let (hop, vt) = match op {
                        UnaryOp::Neg => (crate::hir::UnaryOp::NegI32, VT::I32),
                        UnaryOp::Not => (crate::hir::UnaryOp::NotBool, VT::Bool),
                    };
                    plan.exprs.push(ENode {
                        kind: EKind::Unary(hop, o),
                        vt,
                        span,
                    });
                    slot_idx[dest] = plan.exprs.len() - 1;
                }
            }
        }
        slot_idx[0]
    }

    /// Plan a non-binary leaf (called only from the iterative walk).
    fn plan_leaf(&mut self, plan: &mut FnPlan, f: &ResolvedFn, e: &ResolvedExpr) -> usize {
        match &e.kind {
            RExprKind::Binary { .. } | RExprKind::Unary { .. } => {
                unreachable!("binary/unary handled by plan_expr walk")
            }
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

fn stmt_term_memo(s: &RStmt, terms: &BTreeMap<usize, Option<Term>>) -> Term {
    let flow_of = |b: &RBlock| -> Option<Term> {
        terms
            .get(&(b as *const RBlock as usize))
            .copied()
            .unwrap_or(None)
    };
    match s {
        RStmt::Return { .. } => Term::Ret,
        RStmt::Break { target, .. } | RStmt::Continue { target, .. } => match target {
            Some(_) => Term::Div,
            None => Term::Stray,
        },
        RStmt::If { then, els, .. } => match els {
            // (Returned, Returned) -> Returned; (Diverged, Diverged) ->
            // Diverged; anything mixed (or no else) continues conservatively,
            // exactly like verify's flow lattice. A Stray arm NEVER counts
            // as a valid terminator for the combination (R2-QA-03): folding
            // (Stray, Stray) into Diverged split trailing statements into
            // disconnected dead blocks, verify's structure gate failed
            // first and skipped the dataflow walk — hiding the real
            // loop-target-scope errors (D10). Stray arms keep trailing
            // statements in the same block so the walk runs and owns the
            // rejection; the pure (Div, Div) case (D08) still splits.
            Some(e) => {
                let (a, b) = (flow_of(then), flow_of(e));
                if a == Some(Term::Stray) || b == Some(Term::Stray) {
                    Term::Stray
                } else {
                    match (a, b) {
                        (Some(Term::Ret), Some(Term::Ret)) => Term::Ret,
                        (Some(Term::Div), Some(Term::Div)) => Term::Div,
                        _ => Term::No,
                    }
                }
            }
            None => Term::No,
        },
        RStmt::Loop { body, .. } => {
            if flow_of(body) == Some(Term::Ret) {
                Term::Ret
            } else {
                Term::No
            }
        }
        RStmt::While { .. } | RStmt::Let { .. } | RStmt::Assign { .. } => Term::No,
    }
}

/// First-terminator classification of every block in the tree, keyed by
/// block address, computed bottom-up with an explicit stack (R6: unlimited
/// block nesting — the recursive per-level restatement was both a depth
/// hazard and quadratic on deep chains).
fn block_terms(root: &RBlock) -> BTreeMap<usize, Option<Term>> {
    enum Task<'a> {
        Enter(&'a RBlock),
        Exit(&'a RBlock),
    }
    let key = |b: &RBlock| b as *const RBlock as usize;
    let mut memo: BTreeMap<usize, Option<Term>> = BTreeMap::new();
    let mut stack: Vec<Task> = vec![Task::Enter(root)];
    while let Some(task) = stack.pop() {
        match task {
            Task::Enter(b) => {
                if memo.contains_key(&key(b)) {
                    continue;
                }
                stack.push(Task::Exit(b));
                for s in &b.stmts {
                    match s {
                        RStmt::If { then, els, .. } => {
                            stack.push(Task::Enter(then));
                            if let Some(e) = els {
                                stack.push(Task::Enter(e));
                            }
                        }
                        RStmt::While { body, .. } | RStmt::Loop { body, .. } => {
                            stack.push(Task::Enter(body));
                        }
                        _ => {}
                    }
                }
            }
            Task::Exit(b) => {
                let flow = b
                    .stmts
                    .iter()
                    .map(|s| stmt_term_memo(s, &memo))
                    .find(|t| *t != Term::No);
                memo.insert(key(b), flow);
            }
        }
    }
    memo
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
    profile: SourceProfile,
    plans: Vec<FnPlan>,
    intrinsic_first_use: BTreeMap<&'static str, Span>,
    /// Filled by [`Emitter::render`]: token-level fallback map.
    frags: Vec<(Span, Span)>,
    spans: Option<EmitSpans>,
}

impl<'p> Emitter<'p> {
    fn new(program: &'p ResolvedProgram, src_len: usize, profile: SourceProfile) -> Self {
        Emitter {
            program,
            src_len,
            profile,
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

        // The generated document carries the profile's HIR identity triple;
        // the binder stamps the matching domain and the domain-gated verify
        // entry owns the rest (PLAN-767 §5.1).
        let (schema, profile_word) = match self.profile.hir_domain() {
            crate::hir::HirDomain::Core => ("auto.hir.core.draft", "core-i32-draft"),
            crate::hir::HirDomain::Scalar => ("auto.hir.scalar.draft", "scalar-i32-draft"),
            crate::hir::HirDomain::Record => ("auto.hir.record.draft", "record-i32-draft"),
        };
        frag!(
            &format!(
                "hir(schema: \"{}\", revision: 1u32, profile: \"{}\")",
                schema, profile_word
            ),
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
                            let (ops, overflow) = bin_op_atom(*op);
                            format!(
                                "expr e{} binary {{ type: {}; op: {}{}; lhs: e{}; rhs: e{} }}",
                                ei, ty, ops, overflow, l, r
                            )
                        }
                        EKind::Logical(op, l, r) => {
                            format!(
                                "expr e{} logical {{ type: {}; op: {}; lhs: e{}; rhs: e{} }}",
                                ei,
                                ty,
                                op.as_str(),
                                l,
                                r
                            )
                        }
                        EKind::Unary(op, o) => {
                            format!(
                                "expr e{} unary {{ type: {}; op: {}; operand: e{} }}",
                                ei,
                                ty,
                                op.as_str(),
                                o
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

/// Atom-text encoding of a surface binary operator: the descriptor word
/// plus the `overflow: trap` attribute for the arithmetic whose overflow
/// is policy-governed (div/rem traps are inherent; comparisons cannot
/// overflow).
fn bin_op_atom(op: BinOp) -> (&'static str, &'static str) {
    match op {
        BinOp::Add => ("add_i32", "; overflow: trap"),
        BinOp::Sub => ("sub_i32", "; overflow: trap"),
        BinOp::Mul => ("mul_i32", "; overflow: trap"),
        BinOp::Div => ("div_i32", ""),
        BinOp::Rem => ("rem_i32", ""),
        BinOp::Lt => ("lt_i32", ""),
        BinOp::Le => ("le_i32", ""),
        BinOp::Gt => ("gt_i32", ""),
        BinOp::Ge => ("ge_i32", ""),
        BinOp::Eq => ("eq", ""),
        BinOp::Ne => ("ne", ""),
        BinOp::And | BinOp::Or => unreachable!("logical ops lower to EKind::Logical"),
    }
}

fn intrinsic_def_id(sym: &str) -> String {
    format!("i_{}", sym.rsplit('.').next().unwrap_or("x"))
}

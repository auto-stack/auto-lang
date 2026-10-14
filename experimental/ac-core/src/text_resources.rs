//! Text resource analysis (PLAN-770): the verify-layer front door for the
//! 774 data-resource contract's ownership/borrow rules, and the producer of
//! the `TextResourcePlan` the backend consumes verbatim.
//!
//! Runs only for text-domain bundles AFTER the common verifier checks
//! (types, initialization, block structure) have passed, so it may assume
//! well-typed, initialized, structurally valid input. It enforces:
//!
//! - owned locals live block-lexically: readable only inside the subtree of
//!   the block that owns their `let` (`verify.owned-scope-escape`), which
//!   keeps every exit's free set derivable from the containment tree;
//! - `=` assignment to an owned local and explicit drops of non-owned
//!   locals are rejected (`verify.resource-assign`, Design 04 F8);
//! - `let y Str = x` (implicit ownership transfer) requires an explicit
//!   owning construction instead (`verify.ownership-required`);
//! - borrowed (`str`) locals carry a view provenance: the set of owned
//!   locals whose buffers the view may reach. Storing a view of a temporary
//!   (any owned-producing node in a str-typed initializer) or returning a
//!   borrowed value that reads owned locals is `verify.borrow-escape`;
//! - `.move` call bindings transfer ownership: the argument must be a live
//!   owned local, the form parameter must declare `move Str`
//!   (`verify.ownership-required`), later uses are `verify.use-after-move`,
//!   same-statement view+move overlap is `verify.mut-conflict`, a second
//!   move is `verify.double-move`;
//! - a local moved anywhere inside a loop body is dead for the whole body
//!   (re-entry) and after the loop;
//! - explicit `drop` statements free exactly once; a use after a drop is
//!   `verify.use-after-free`.
//!
//! Output: per-exit LIFO free schedules (reverse declaration order) at
//! `return`/`break`/`continue`/block fall-through points, plus the
//! owned-return set. Temporary owned values (expression-owned concat/clone/
//! promotion results) are NOT scheduled here: they have a single owning
//! consumer by the expression graph invariant, so their free point is a
//! purely local lowering rule ("free immediately after the consumer that is
//! not a let/return"), with no dataflow content.

use crate::atom_text::{Diagnostic, Span, Stage};
use crate::hir;
use crate::verify::ValueType;
use std::collections::{BTreeMap, BTreeSet};

/// Capability every text-profile module whose content lowers to runtime
/// calls must declare and the caller must explicitly provide (PLAN-770 §2;
/// frozen string, shared by the verifier front door, the source adapter's
/// requires emission and the build gates).
pub const RUNTIME_TEXT_CAPABILITY: &str = "ac.runtime.text.v1";

/// Whether any expression in the bundle lowers to an `ac_rt_*` call
/// (review r2 QA-02): the DERIVED runtime need, independent of the
/// document's own `requires` declaration. Mirrors the lowering's
/// `scan_body_rt_needs` operation set (own_free rides the same library).
pub fn bundle_needs_text_runtime(bundle: &hir::Bundle) -> bool {
    bundle
        .modules
        .iter()
        .any(|m| m.bodies.iter().any(|b| body_needs_text_runtime(m, b)))
}

pub fn body_needs_text_runtime(m: &hir::Module, body: &hir::Body) -> bool {
    let text_ty = |e: &hir::Expr| type_use_is_text(&e.ty, m);
    body.exprs.iter().any(|e| match &e.kind {
        hir::ExprKind::TextLen { .. }
        | hir::ExprKind::TextLenBytes { .. }
        | hir::ExprKind::TextCharAt { .. }
        | hir::ExprKind::TextSlice { .. }
        | hir::ExprKind::TextConcat { .. }
        | hir::ExprKind::TextClone { .. }
        | hir::ExprKind::TextToOwned { .. }
        | hir::ExprKind::CharToCodepoint { .. }
        | hir::ExprKind::CharFromCodepoint { .. } => true,
        hir::ExprKind::StrLit { .. } => type_use_is_owned(&e.ty, m),
        hir::ExprKind::Binary {
            op: hir::BinOp::Eq | hir::BinOp::Ne,
            lhs,
            ..
        } => text_ty(&body.exprs[lhs.0 as usize]),
        _ => false,
    })
}

/// Nominal text-type test (str or Str, builtin shorthand or table ref).
pub(crate) fn type_use_is_text(u: &hir::TypeUse, m: &hir::Module) -> bool {
    match u {
        hir::TypeUse::Builtin(hir::BuiltinType::StrSlice | hir::BuiltinType::StrOwned) => true,
        hir::TypeUse::Ref(tid) => matches!(
            m.types.get(tid.0 as usize).map(|t| &t.kind),
            Some(hir::TypeKind::Str | hir::TypeKind::StrOwned)
        ),
        _ => false,
    }
}

/// Nominal owned-Str type test.
pub(crate) fn type_use_is_owned(u: &hir::TypeUse, m: &hir::Module) -> bool {
    match u {
        hir::TypeUse::Builtin(hir::BuiltinType::StrOwned) => true,
        hir::TypeUse::Ref(tid) => matches!(
            m.types.get(tid.0 as usize).map(|t| &t.kind),
            Some(hir::TypeKind::StrOwned)
        ),
        _ => false,
    }
}

/// Verified resource obligations for one function body.
#[derive(Clone, Debug, Default)]
pub struct BodyResourcePlan {
    /// Owned locals of this body (owned bindings and `move Str` parameters):
    /// every one is freed exactly once on every exit path.
    pub owned: BTreeSet<u32>,
    /// LIFO-ordered (reverse declaration order) free lists to emit at exit
    /// points. Key: (block id, statement index) for return/break/continue/
    /// drop positions; `(block, None)` is the block's natural fall-through
    /// end (frees only that block's own lets).
    pub exit_frees: BTreeMap<(u32, Option<usize>), Vec<u32>>,
    /// Return statements whose value is an owned `Str` transferring out:
    /// the returned value itself is NOT freed; everything else live is.
    pub owned_returns: BTreeSet<(u32, usize)>,
}

/// The verifier-computed text resource plan for a whole bundle.
#[derive(Clone, Debug, Default)]
pub struct TextResourcePlan {
    /// (module index, body index) → plan.
    pub bodies: BTreeMap<(usize, u32), BodyResourcePlan>,
}

/// Check one body; on success produce its resource plan. Diagnostics are
/// appended to `diags` (caller decides pass/fail).
pub(crate) fn check_body(
    body: &hir::Body,
    m: &hir::Module,
    type_map: &BTreeMap<u32, ValueType>,
    diags: &mut Vec<Diagnostic>,
) -> Option<BodyResourcePlan> {
    let fn_result = fn_result_of(body, m, type_map);
    let local_type = |id: u32| -> ValueType {
        match &body.locals[id as usize].ty {
            hir::TypeUse::Builtin(hir::BuiltinType::StrSlice) => ValueType::Str,
            hir::TypeUse::Builtin(hir::BuiltinType::StrOwned) => ValueType::StrOwned,
            hir::TypeUse::Builtin(hir::BuiltinType::Char) => ValueType::Char,
            hir::TypeUse::Builtin(hir::BuiltinType::I32) => ValueType::I32,
            hir::TypeUse::Builtin(hir::BuiltinType::Bool) => ValueType::Bool,
            hir::TypeUse::Ref(t) => type_map.get(&t.0).copied().unwrap_or(ValueType::I32),
        }
    };
    // Owned locals: Str bindings plus `move Str` parameters. View/mut Str
    // parameters borrow the caller's buffer and are never freed here.
    let owned: BTreeSet<u32> = body
        .locals
        .iter()
        .enumerate()
        .filter(|(i, l)| {
            local_type(*i as u32) == ValueType::StrOwned
                && (l.role == hir::LocalRole::Binding || l.param_mode == hir::ParamMode::MoveOwn)
        })
        .map(|(i, _)| i as u32)
        .collect();

    let mut w = ResourceWalk {
        body,
        owned: &owned,
        local_type: &local_type,
        fn_result,
        diags,
        plan: BodyResourcePlan {
            owned: owned.clone(),
            exit_frees: BTreeMap::new(),
            owned_returns: BTreeSet::new(),
        },
    };
    w.walk(m);
    Some(w.plan)
}

fn fn_result_of(
    body: &hir::Body,
    m: &hir::Module,
    type_map: &BTreeMap<u32, ValueType>,
) -> ValueType {
    let Some(def) = m.defs.get(body.owner.0 as usize) else {
        return ValueType::I32;
    };
    let ty_id = match &def.kind {
        hir::DefKind::Function { ty, .. } | hir::DefKind::Intrinsic { ty, .. } => *ty,
    };
    match m.types.get(ty_id.0 as usize).map(|t| &t.kind) {
        Some(hir::TypeKind::Function(sig)) => match &sig.result {
            hir::TypeUse::Builtin(hir::BuiltinType::StrSlice) => ValueType::Str,
            hir::TypeUse::Builtin(hir::BuiltinType::StrOwned) => ValueType::StrOwned,
            hir::TypeUse::Builtin(hir::BuiltinType::Char) => ValueType::Char,
            hir::TypeUse::Builtin(hir::BuiltinType::Bool) => ValueType::Bool,
            hir::TypeUse::Builtin(hir::BuiltinType::I32) => ValueType::I32,
            hir::TypeUse::Ref(t) => type_map.get(&t.0).copied().unwrap_or(ValueType::I32),
        },
        _ => ValueType::I32,
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum DeadCause {
    Moved,
    Dropped,
}

/// Static scan of one expression tree: every `ReadLocal` leaf, whether any
/// owned-producing node exists, and every call's moved-binding arguments
/// (review r2 QA-03/QA-05: moves are classified by EXECUTION REACHABILITY —
/// a short-circuit RHS under a constant-polarity lhs is either definitely
/// executed or definitely skipped; under an unknown lhs its moves are
/// CONDITIONAL and the statement must be rejected — a maybe-transfer has
/// no consistent static cleanup responsibility. Owned-producing now
/// includes calls returning Str and promoted literals: a view of either
/// stored across statements is a `verify.borrow-escape`).
#[derive(Default)]
struct ExprScan {
    /// (expr id, local id, span) of every ReadLocal leaf
    reads: Vec<(u32, u32, Span)>,
    has_owned_producing: bool,
    /// definitely-executed (expr id, span) for `.move` bindings
    moves: Vec<(hir::ExprId, Span)>,
    /// maybe-executed moves (short-circuit RHS under unknown polarity)
    conditional_moves: Vec<(hir::ExprId, Span)>,
    /// Call nodes with their bindings (param index -> moved flag), for
    /// mode validation and mut-aliasing checks.
    call_moves: Vec<(hir::ExprId, Vec<(u32, bool)>, Span)>,
}

fn scan_expr(m: &hir::Module, body: &hir::Body, root: hir::ExprId) -> ExprScan {
    #[derive(Clone, Copy, PartialEq)]
    enum Mode {
        /// moves count as definite
        Normal,
        /// moves are conditional (maybe executed)
        Conditional,
        /// statically unreachable subtree: reads still recorded (static
        /// legality), moves ignored entirely
        Dead,
    }
    let mut scan = ExprScan::default();
    let mut stack: Vec<(hir::ExprId, Mode)> = vec![(root, Mode::Normal)];
    while let Some((id, mode)) = stack.pop() {
        let e = &body.exprs[id.0 as usize];
        match &e.kind {
            hir::ExprKind::ReadLocal { local } => scan.reads.push((id.0, local.0, e.span)),
            hir::ExprKind::TextConcat { lhs, rhs } => {
                scan.has_owned_producing = true;
                stack.push((*rhs, mode));
                stack.push((*lhs, mode));
            }
            hir::ExprKind::TextClone { operand } | hir::ExprKind::TextToOwned { operand } => {
                scan.has_owned_producing = true;
                stack.push((*operand, mode));
            }
            hir::ExprKind::StrLit { .. } => {
                if type_use_is_owned(&e.ty, m) {
                    scan.has_owned_producing = true;
                }
            }
            hir::ExprKind::Unary { operand, .. }
            | hir::ExprKind::TextLen { operand }
            | hir::ExprKind::TextLenBytes { operand }
            | hir::ExprKind::TextToBorrowed { operand }
            | hir::ExprKind::CharToCodepoint { operand }
            | hir::ExprKind::CharFromCodepoint { operand } => {
                stack.push((*operand, mode));
            }
            hir::ExprKind::Call {
                callee,
                eval_args,
                bindings,
            } => {
                if let Some(hir::DefKind::Function { ty, .. }) =
                    m.defs.get(callee.0 as usize).map(|d| &d.kind)
                {
                    if let Some(hir::TypeKind::Function(sig)) =
                        m.types.get(ty.0 as usize).map(|t| &t.kind)
                    {
                        if type_use_is_owned(&sig.result, m) {
                            scan.has_owned_producing = true;
                        }
                    }
                }
                let mut bm = Vec::new();
                for b in bindings {
                    bm.push((b.param, b.moved));
                    if b.moved && mode == Mode::Normal && (b.arg as usize) < eval_args.len() {
                        scan.moves.push((eval_args[b.arg as usize], e.span));
                    } else if b.moved
                        && mode == Mode::Conditional
                        && (b.arg as usize) < eval_args.len()
                    {
                        scan.conditional_moves
                            .push((eval_args[b.arg as usize], e.span));
                    }
                }
                scan.call_moves.push((id, bm, e.span));
                for a in eval_args.iter().rev() {
                    stack.push((*a, mode));
                }
            }
            hir::ExprKind::Logical { op, lhs, rhs } => {
                // lhs always evaluates; the rhs runs only under the lhs's
                // polarity (`&&`: true; `||`: false). A constant lhs pins
                // the rhs's execution; an unknown lhs makes its moves
                // conditional.
                stack.push((*lhs, mode));
                let lhs_truth = const_truth(body, *lhs);
                let rhs_mode = match op {
                    hir::LogicOp::And => match lhs_truth {
                        Some(true) => mode,
                        Some(false) => Mode::Dead,
                        None => Mode::Conditional,
                    },
                    hir::LogicOp::Or => match lhs_truth {
                        Some(false) => mode,
                        Some(true) => Mode::Dead,
                        None => Mode::Conditional,
                    },
                };
                // a nested Dead/Conditional mode only narrows further
                let rhs_mode = if mode == Mode::Dead {
                    Mode::Dead
                } else if mode == Mode::Conditional && rhs_mode == Mode::Normal {
                    Mode::Conditional
                } else {
                    rhs_mode
                };
                stack.push((*rhs, rhs_mode));
            }
            hir::ExprKind::Binary { lhs, rhs, .. } => {
                stack.push((*rhs, mode));
                stack.push((*lhs, mode));
            }
            hir::ExprKind::TextCharAt { operand, index } => {
                stack.push((*operand, mode));
                stack.push((*index, mode));
            }
            hir::ExprKind::TextSlice {
                operand,
                start,
                end,
            } => {
                stack.push((*operand, mode));
                stack.push((*start, mode));
                stack.push((*end, mode));
            }
            hir::ExprKind::Constant { .. } | hir::ExprKind::CharLit { .. } => {}
        }
    }
    scan
}

/// Constant evaluation of a boolean expression (bool constants are the
/// canonical i32 0/1; comparisons and arithmetic over constants fold).
/// `None` = not statically known. Iterative post-order with short-circuit
/// early exit — a decisive logical lhs settles the node without needing
/// the rhs (which may contain calls), and operator chains are unbounded
/// (V-09), so no recursion.
fn const_truth(body: &hir::Body, root: hir::ExprId) -> Option<bool> {
    enum St {
        Enter(hir::ExprId),
        /// logical node whose lhs value decides whether the rhs must be
        /// evaluated at all
        LogWait(hir::ExprId),
        Reduce(hir::ExprId),
    }
    let mut stack = vec![St::Enter(root)];
    let mut vals: BTreeMap<u32, i32> = BTreeMap::new();
    while let Some(st) = stack.pop() {
        match st {
            St::Enter(id) => {
                let e = &body.exprs[id.0 as usize];
                match &e.kind {
                    hir::ExprKind::Constant { value } => {
                        vals.insert(id.0, *value);
                    }
                    hir::ExprKind::Unary { operand, .. } => {
                        stack.push(St::Reduce(id));
                        stack.push(St::Enter(*operand));
                    }
                    hir::ExprKind::Binary { lhs, rhs, .. } => {
                        stack.push(St::Reduce(id));
                        stack.push(St::Enter(*rhs));
                        stack.push(St::Enter(*lhs));
                    }
                    hir::ExprKind::Logical { lhs, .. } => {
                        stack.push(St::LogWait(id));
                        stack.push(St::Enter(*lhs));
                    }
                    _ => return None,
                }
            }
            St::LogWait(id) => {
                let hir::ExprKind::Logical { op, lhs, rhs } = &body.exprs[id.0 as usize].kind
                else {
                    unreachable!("LogWait on logical expr")
                };
                let a = vals.remove(&lhs.0)?;
                let decisive = match op {
                    hir::LogicOp::And => a == 0,
                    hir::LogicOp::Or => a != 0,
                };
                if decisive {
                    vals.insert(id.0, a);
                } else {
                    stack.push(St::Reduce(id));
                    stack.push(St::Enter(*rhs));
                }
            }
            St::Reduce(id) => {
                let e = &body.exprs[id.0 as usize];
                let v = match &e.kind {
                    hir::ExprKind::Unary { op, operand } => {
                        let a = vals.remove(&operand.0)?;
                        match op {
                            hir::UnaryOp::NotBool => Some(1 - a),
                            hir::UnaryOp::NegI32 => 0i32.checked_sub(a),
                        }
                    }
                    hir::ExprKind::Binary { op, lhs, rhs, .. } => {
                        let a = vals.remove(&lhs.0)?;
                        let b = vals.remove(&rhs.0)?;
                        use hir::BinOp::*;
                        match op {
                            AddI32 => a.checked_add(b),
                            SubI32 => a.checked_sub(b),
                            MulI32 => a.checked_mul(b),
                            LtI32 => Some((a < b) as i32),
                            LeI32 => Some((a <= b) as i32),
                            GtI32 => Some((a > b) as i32),
                            GeI32 => Some((a >= b) as i32),
                            Eq => Some((a == b) as i32),
                            Ne => Some((a != b) as i32),
                            DivI32 | RemI32 => None,
                        }
                    }
                    hir::ExprKind::Logical { op, lhs, rhs } => {
                        let a = vals.remove(&lhs.0)?;
                        let b = vals.remove(&rhs.0)?;
                        match op {
                            hir::LogicOp::And => Some(((a != 0) && (b != 0)) as i32),
                            hir::LogicOp::Or => Some(((a != 0) || (b != 0)) as i32),
                        }
                    }
                    _ => return None,
                }?;
                vals.insert(id.0, v);
            }
        }
    }
    vals.get(&root.0).copied().map(|v| v != 0)
}

/// Whether a block subtree cannot fall through to its join (every path
/// ends in return/break/continue). Iterative with a memo — block nesting
/// is unbounded (V-09). A loop always has a fall-through edge (break), so
/// it never terminates its parent's flow by itself.
fn block_terminates(body: &hir::Body, bid: u32) -> bool {
    let mut memo: BTreeMap<u32, bool> = BTreeMap::new();
    block_terminates_impl(body, bid, &mut memo)
}

fn block_terminates_impl(body: &hir::Body, root: u32, memo: &mut BTreeMap<u32, bool>) -> bool {
    enum W {
        Enter(u32),
        Finish(u32),
    }
    let mut stack = vec![W::Enter(root)];
    while let Some(w) = stack.pop() {
        match w {
            W::Enter(bid) => {
                if memo.contains_key(&bid) {
                    continue;
                }
                memo.insert(bid, false); // cycle guard (verifier rejects cycles)
                stack.push(W::Finish(bid));
                if let Some(bl) = body.blocks.get(bid as usize) {
                    for s in &bl.stmts {
                        if let hir::Stmt::If {
                            then, els: Some(e), ..
                        } = s
                        {
                            stack.push(W::Enter(then.0));
                            stack.push(W::Enter(e.0));
                        }
                    }
                }
            }
            W::Finish(bid) => {
                let Some(bl) = body.blocks.get(bid as usize) else {
                    continue;
                };
                let mut result = false;
                for s in &bl.stmts {
                    match s {
                        hir::Stmt::Return { .. }
                        | hir::Stmt::Break { .. }
                        | hir::Stmt::Continue { .. } => {
                            result = true;
                            break;
                        }
                        hir::Stmt::If { then, els, .. } => {
                            if let Some(e) = els {
                                if memo.get(&then.0).copied().unwrap_or(false)
                                    && memo.get(&e.0).copied().unwrap_or(false)
                                {
                                    result = true;
                                    break;
                                }
                            }
                        }
                        _ => {}
                    }
                }
                memo.insert(bid, result);
            }
        }
    }
    memo.get(&root).copied().unwrap_or(false)
}

/// Definitely-executed moved locals anywhere inside a block subtree (used
/// by the loop re-entry kill and the if-join responsibility analysis).
/// Conditional moves (unknown short-circuit arms) are NOT definite — their
/// own statement/if reports them.
fn block_definite_moves(m: &hir::Module, body: &hir::Body, bid: u32, out: &mut BTreeSet<u32>) {
    let mut stack = vec![bid];
    while let Some(b) = stack.pop() {
        let Some(bl) = body.blocks.get(b as usize) else {
            continue;
        };
        for s in &bl.stmts {
            match s {
                hir::Stmt::Let { value, .. }
                | hir::Stmt::Return { value }
                | hir::Stmt::Assign { value, .. } => {
                    let scan = scan_expr(m, body, *value);
                    for (argid, _) in &scan.moves {
                        if let hir::ExprKind::ReadLocal { local } =
                            &body.exprs[argid.0 as usize].kind
                        {
                            out.insert(local.0);
                        }
                    }
                }
                hir::Stmt::If { cond, then, els } => {
                    let scan = scan_expr(m, body, *cond);
                    for (argid, _) in &scan.moves {
                        if let hir::ExprKind::ReadLocal { local } =
                            &body.exprs[argid.0 as usize].kind
                        {
                            out.insert(local.0);
                        }
                    }
                    stack.push(then.0);
                    if let Some(e) = els {
                        stack.push(e.0);
                    }
                }
                hir::Stmt::Loop { body: lb, .. } => stack.push(lb.0),
                _ => {}
            }
        }
    }
}

struct ResourceWalk<'a> {
    body: &'a hir::Body,
    owned: &'a BTreeSet<u32>,
    local_type: &'a dyn Fn(u32) -> ValueType,
    fn_result: ValueType,
    diags: &'a mut Vec<Diagnostic>,
    plan: BodyResourcePlan,
}

/// Per-frame dataflow state carried through the iterative walk.
#[derive(Clone)]
struct WalkState {
    /// owned local -> owning (let) block id
    live: BTreeMap<u32, u32>,
    /// dead owned locals and why (moved / dropped)
    dead: BTreeMap<u32, DeadCause>,
    /// str local -> backing owned locals (path-union over branches)
    backing: BTreeMap<u32, BTreeSet<u32>>,
}

impl WalkState {
    fn new() -> Self {
        WalkState {
            live: BTreeMap::new(),
            dead: BTreeMap::new(),
            backing: BTreeMap::new(),
        }
    }
}

enum RState {
    Run,
    AwaitIf { then_slot: usize, else_slot: usize },
    AwaitLoop { body_slot: usize },
}

struct RFrame {
    bid: u32,
    loops_len: usize,
    idx: usize,
    st: WalkState,
    /// owned locals let in THIS frame (its natural-end free set)
    own_lets: Vec<u32>,
    /// terminator reached (return/break/continue): no further statements
    flow_done: bool,
    ret_slot: usize,
    state: RState,
    /// r2 QA-03: owned locals moved on EVERY path joining at this frame's
    /// pending if — applied to `st.live` when the arms complete (the callee
    /// owns them; this frame drops the free responsibility).
    if_join_dead: BTreeSet<u32>,
}

enum Action {
    /// Run this statement (the frame cursor already advanced); idx is the
    /// statement's position in the block.
    Stmt {
        bid: u32,
        idx: usize,
        stmt: hir::Stmt,
    },
    /// A terminator precedes remaining statements (already reported by the
    /// common structure walk); complete the frame.
    AfterTerm,
    /// The frame ran out of statements.
    Complete,
    /// Children finished; merge their results.
    Resume(RState),
}

impl<'a> ResourceWalk<'a> {
    fn error(&mut self, code: &str, msg: impl Into<String>, span: Span) {
        self.diags
            .push(Diagnostic::new(Stage::Verify, code, msg, span));
    }

    fn local_name(&self, id: u32) -> &str {
        self.body
            .locals
            .get(id as usize)
            .map(|l| l.id_text.as_str())
            .unwrap_or("<unknown>")
    }

    /// LIFO (reverse declaration order) list from a set of owned locals.
    fn lifo(set: &BTreeSet<u32>) -> Vec<u32> {
        set.iter().rev().copied().collect()
    }

    /// Owned locals reachable from an expression's reads: direct owned
    /// reads, `mut`-parameter reads (r2 QA-06: a mut borrow is an aliasing
    /// source for conflict and escape purposes, like an owned buffer), and
    /// the backings of borrowed str locals read (path-union).
    fn owned_reach(&self, st: &WalkState, scan: &ExprScan) -> BTreeSet<u32> {
        let mut out = BTreeSet::new();
        for (_, l, _) in &scan.reads {
            if self.owned.contains(l) || is_mut_param(&self.body.locals[*l as usize]) {
                out.insert(*l);
            } else if let Some(bs) = st.backing.get(l) {
                out.extend(bs.iter().copied());
            }
        }
        out
    }

    /// Validate every read in a statement's expression trees against the
    /// current dataflow state: dead (moved/dropped) locals, and the
    /// block-lexical scope of owned locals (direct reads and via view
    /// provenance).
    fn check_reads(&mut self, st: &WalkState, chain: &[u32], reads: &[(u32, u32, Span)]) {
        for (_, l, span) in reads {
            if let Some(cause) = st.dead.get(l) {
                let code = match cause {
                    DeadCause::Moved => "verify.use-after-move",
                    DeadCause::Dropped => "verify.use-after-free",
                };
                let why = match cause {
                    DeadCause::Moved => "moved",
                    DeadCause::Dropped => "dropped",
                };
                self.error(
                    code,
                    format!("read of {} local `{}`", why, self.local_name(*l)),
                    *span,
                );
                continue;
            }
            if self.owned.contains(l) {
                if let Some(&let_block) = st.live.get(l) {
                    if !chain.contains(&let_block) {
                        self.error(
                            "verify.owned-scope-escape",
                            format!(
                                "owned local `{}` is read outside the block subtree of its `let`",
                                self.local_name(*l)
                            ),
                            *span,
                        );
                    }
                }
                // not in `live`: uninitialized — the common verifier walk
                // already reported it; nothing resource-specific to add.
                continue;
            }
            if (self.local_type)(*l) == ValueType::Str {
                if let Some(bs) = st.backing.get(l) {
                    for b in bs {
                        if let Some(cause) = st.dead.get(b) {
                            let code = match cause {
                                DeadCause::Moved => "verify.use-after-move",
                                DeadCause::Dropped => "verify.use-after-free",
                            };
                            self.error(
                                code,
                                format!(
                                    "read of borrowed local `{}` whose owner `{}` was moved or dropped",
                                    self.local_name(*l),
                                    self.local_name(*b)
                                ),
                                *span,
                            );
                            continue;
                        }
                        if let Some(&let_block) = st.live.get(b) {
                            if !chain.contains(&let_block) {
                                self.error(
                                    "verify.owned-scope-escape",
                                    format!(
                                        "borrowed local `{}` outlives its backing owner `{}`'s scope",
                                        self.local_name(*l),
                                        self.local_name(*b)
                                    ),
                                    *span,
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    /// Validate `.move` bindings in a statement: the argument must be a
    /// named, live, in-scope owned local; the form parameter must declare
    /// `move Str`; same-statement read+move overlap is a mut conflict; a
    /// second move of one local is a double move. Returns the set of
    /// validated moved locals (to be marked dead by the caller).
    fn validate_moves(
        &mut self,
        m: &hir::Module,
        st: &WalkState,
        scan: &ExprScan,
        chain: &[u32],
    ) -> BTreeSet<u32> {
        // Conservative same-statement view set (774 D-04 with statement
        // granularity): every owned local read directly or via a str
        // local's provenance anywhere in the statement counts as viewed —
        // EXCEPT the reads that ARE the moved arguments (the moving read
        // itself is not a view).
        let moved_arg_exprs: BTreeSet<u32> = scan.moves.iter().map(|(id, _)| id.0).collect();
        let mut viewed = BTreeSet::new();
        for (eid, l, _) in &scan.reads {
            if moved_arg_exprs.contains(eid) {
                continue;
            }
            if self.owned.contains(l) {
                viewed.insert(*l);
            } else if let Some(bs) = st.backing.get(l) {
                viewed.extend(bs.iter().copied());
            }
        }
        let mut moved = BTreeSet::new();
        for (argid, span) in &scan.moves {
            let local = match &self.body.exprs[argid.0 as usize].kind {
                hir::ExprKind::ReadLocal { local } => Some(local),
                _ => None,
            };
            let Some(local) = local else {
                self.error(
                    "verify.ownership-required",
                    "`.move` requires a named owned Str argument (bind the value to a local first)",
                    *span,
                );
                continue;
            };
            if !self.owned.contains(&local.0) {
                self.error(
                    "verify.ownership-required",
                    format!(
                        "`.move` argument `{}` is not an owned Str local",
                        self.local_name(local.0)
                    ),
                    *span,
                );
                continue;
            }
            if st.dead.contains_key(&local.0) {
                self.error(
                    "verify.use-after-move",
                    format!(
                        "local `{}` is moved again in the same statement",
                        self.local_name(local.0)
                    ),
                    *span,
                );
                continue;
            }
            if !st.live.contains_key(&local.0) {
                // uninitialized owned move: the common walk reported the
                // init problem; still reject here for a precise code.
                self.error(
                    "verify.ownership-required",
                    format!(
                        "`.move` argument `{}` is not an initialized owned local",
                        self.local_name(local.0)
                    ),
                    *span,
                );
                continue;
            }
            if let Some(&let_block) = st.live.get(&local.0) {
                if !chain.contains(&let_block) {
                    self.error(
                        "verify.owned-scope-escape",
                        format!(
                            "owned local `{}` is moved outside the block subtree of its `let`",
                            self.local_name(local.0)
                        ),
                        *span,
                    );
                    continue;
                }
            }
            if !moved.insert(local.0) {
                self.error(
                    "verify.double-move",
                    format!(
                        "local `{}` is moved by more than one binding in this statement",
                        self.local_name(local.0)
                    ),
                    *span,
                );
            }
        }
        // Callee-side mode check: each moved binding must target a
        // parameter declared `move Str` in the callee's body.
        for (callid, bindings, span) in &scan.call_moves {
            let hir::ExprKind::Call { callee, .. } = &self.body.exprs[callid.0 as usize].kind
            else {
                continue;
            };
            let Some(def) = m.defs.get(callee.0 as usize) else {
                continue;
            };
            let hir::DefKind::Function { body: cb, ty, .. } = &def.kind else {
                continue;
            };
            let callee_body = match m.bodies.get(cb.0 as usize) {
                Some(b) => b,
                None => continue,
            };
            let sig_text = match m.types.get(ty.0 as usize).map(|t| &t.kind) {
                Some(hir::TypeKind::Function(sig)) => sig,
                _ => continue,
            };
            for (param_idx, is_moved) in bindings {
                let is_owned_type = |u: &hir::TypeUse| match u {
                    hir::TypeUse::Builtin(hir::BuiltinType::StrOwned) => true,
                    // the source adapter emits named text types (t_Str)
                    hir::TypeUse::Ref(tid) => matches!(
                        m.types.get(tid.0 as usize).map(|t| &t.kind),
                        Some(hir::TypeKind::StrOwned)
                    ),
                    _ => false,
                };
                let param_is_move = sig_text
                    .params
                    .get(*param_idx as usize)
                    .map(|p| is_owned_type(p))
                    .unwrap_or(false)
                    && callee_body.locals.iter().any(|l| {
                        l.role == hir::LocalRole::Param
                            && l.index == Some(*param_idx)
                            && l.param_mode == hir::ParamMode::MoveOwn
                    });
                if *is_moved && !param_is_move {
                    self.error(
                        "verify.ownership-required",
                        format!(
                            "argument is marked `.move` but parameter {} of `{}` does not declare `move Str`",
                            param_idx, def.id_text
                        ),
                        *span,
                    );
                }
                if !*is_moved && param_is_move {
                    // The callee frees a `move Str` parameter at its exits;
                    // passing without the `.move` marker would leave the
                    // caller's ownership intact — a double free.
                    self.error(
                        "verify.ownership-required",
                        format!(
                            "parameter {} of `{}` declares `move Str` but the argument is not marked `.move`",
                            param_idx, def.id_text
                        ),
                        *span,
                    );
                }
            }
            // r2 QA-06: a `mut` parameter borrows exclusively — its
            // argument's reach (owned buffers or mut sources it views)
            // must be disjoint from every other argument's reach.
            let hir::ExprKind::Call {
                eval_args,
                bindings: call_bindings,
                ..
            } = &self.body.exprs[callid.0 as usize].kind
            else {
                continue;
            };
            let mut reaches: Vec<(u32, BTreeSet<u32>)> = Vec::new();
            for b in call_bindings {
                if (b.arg as usize) >= eval_args.len() {
                    continue;
                }
                let ascan = scan_expr(m, self.body, eval_args[b.arg as usize]);
                let r = self.owned_reach(st, &ascan);
                reaches.push((b.param, r));
            }
            for (pi, reach_i) in &reaches {
                let is_mut = callee_body.locals.iter().any(|l| {
                    l.role == hir::LocalRole::Param
                        && l.index == Some(*pi)
                        && l.param_mode == hir::ParamMode::MutBorrow
                });
                if !is_mut {
                    continue;
                }
                for (pj, reach_j) in &reaches {
                    if pj == pi {
                        continue;
                    }
                    if let Some(&hit) = reach_i.intersection(reach_j).next() {
                        self.error(
                            "verify.mut-conflict",
                            format!(
                                "argument for `mut` parameter {} of `{}` views `{}`, which parameter {} also views; a mut borrow is exclusive",
                                pi, def.id_text, self.local_name(hit), pj
                            ),
                            *span,
                        );
                        break;
                    }
                }
            }
        }
        for l in &moved {
            if viewed.contains(l) {
                let span = scan
                    .reads
                    .iter()
                    .find(|(_, r, _)| {
                        r == l || st.backing.get(r).map_or(false, |bs| bs.contains(l))
                    })
                    .map(|(_, _, s)| *s)
                    .unwrap_or_else(|| *span_of_last(&scan.reads));
                self.error(
                    "verify.mut-conflict",
                    format!(
                        "owned local `{}` is viewed and moved within the same statement",
                        self.local_name(*l)
                    ),
                    span,
                );
            }
        }
        moved
    }

    fn reject_conditional_moves(&mut self, scan: &ExprScan) {
        // r2 QA-03: a move under an unknown-polarity short-circuit operand
        // would maybe-transfer — no single static free responsibility.
        for (_, span) in &scan.conditional_moves {
            self.error(
                "verify.ownership-required",
                "a `.move` inside a conditionally-executed `&&`/`||` right side has no consistent static cleanup responsibility; hoist it into an unconditional statement",
                *span,
            );
        }
    }

    fn walk(&mut self, m: &hir::Module) {
        let entry = self.body.entry.0;
        let mut init = WalkState::new();
        // move-in parameters are owned from the entry block's scope
        for (i, l) in self.body.locals.iter().enumerate() {
            if l.role == hir::LocalRole::Param
                && l.param_mode == hir::ParamMode::MoveOwn
                && (self.local_type)(i as u32) == ValueType::StrOwned
            {
                init.live.insert(i as u32, entry);
            }
        }
        let mut results: Vec<Option<WalkState>> = vec![None];
        let mut frames: Vec<RFrame> = Vec::new();
        let mut loops: Vec<(hir::LoopId, u32)> = Vec::new();
        frames.push(RFrame {
            bid: entry,
            loops_len: 0,
            idx: 0,
            st: init,
            own_lets: Vec::new(),
            flow_done: false,
            ret_slot: 0,
            state: RState::Run,
            if_join_dead: BTreeSet::new(),
        });
        while !frames.is_empty() {
            let action = {
                let frame = frames.last_mut().expect("resource walk frame");
                if let RState::Run = frame.state {
                    let bid = frame.bid;
                    if frame.flow_done {
                        // a terminated frame never reaches a natural end:
                        // its terminator already emitted the exit frees
                        Action::AfterTerm
                    } else if frame.idx >= self.body.blocks[bid as usize].stmts.len() {
                        Action::Complete
                    } else {
                        let stmt = self.body.blocks[bid as usize].stmts[frame.idx].clone();
                        frame.idx += 1;
                        let idx = frame.idx - 1;
                        Action::Stmt { bid, idx, stmt }
                    }
                } else {
                    Action::Resume(std::mem::replace(&mut frame.state, RState::Run))
                }
            };
            match action {
                Action::Stmt { bid, idx, stmt } => {
                    let chain: Vec<u32> = frames.iter().map(|f| f.bid).collect();
                    let frame = frames.last_mut().expect("resource walk frame");
                    match stmt {
                        hir::Stmt::Let { local, value } => {
                            let scan = scan_expr(m, self.body, value);
                            self.reject_conditional_moves(&scan);
                            let st_snapshot = frame.st.clone();
                            self.check_reads(&st_snapshot, &chain, &scan.reads);
                            let moved = self.validate_moves(m, &st_snapshot, &scan, &chain);
                            let vt = (self.local_type)(local.0);
                            match vt {
                                ValueType::StrOwned => {
                                    if let hir::ExprKind::ReadLocal { local: src } =
                                        &self.body.exprs[value.0 as usize].kind
                                    {
                                        // r2 QA-04: ANY read-local initializer
                                        // is an implicit transfer — of an owned
                                        // local (old rule) or of a view/mut
                                        // parameter the callee/caller owns
                                        // (new): either way the binding must
                                        // construct ownership explicitly.
                                        self.error(
                                            "verify.ownership-required",
                                            format!(
                                                "`let` cannot implicitly transfer ownership of `{}`; use `.clone()`, `.to(Str)` or a call result",
                                                self.local_name(src.0)
                                            ),
                                            self.body.exprs[value.0 as usize].span,
                                        );
                                    }
                                    frame.st.live.insert(local.0, bid);
                                    frame.own_lets.push(local.0);
                                }
                                ValueType::Str => {
                                    if scan.has_owned_producing {
                                        self.error(
                                            "verify.borrow-escape",
                                            format!(
                                                "str local `{}` stores a view of a temporary; owned values live only to the end of their statement",
                                                self.local_name(local.0)
                                            ),
                                            self.body.blocks[bid as usize].span,
                                        );
                                    }
                                    let prov = self.owned_reach(&st_snapshot, &scan);
                                    frame.st.backing.insert(local.0, prov);
                                }
                                _ => {}
                            }
                            for l in moved {
                                frame.st.live.remove(&l);
                                frame.st.dead.insert(l, DeadCause::Moved);
                            }
                        }
                        hir::Stmt::Assign { place, value } => {
                            let scan = scan_expr(m, self.body, value);
                            self.reject_conditional_moves(&scan);
                            let st_snapshot = frame.st.clone();
                            self.check_reads(&st_snapshot, &chain, &scan.reads);
                            let moved = self.validate_moves(m, &st_snapshot, &scan, &chain);
                            let target = place_local(self.body, &place);
                            let vt = (self.local_type)(target);
                            match vt {
                                ValueType::StrOwned => {
                                    self.error(
                                        "verify.resource-assign",
                                        format!(
                                            "owned local `{}` cannot be reassigned with `=`; resource assignment is forbidden (Design 04)",
                                            self.local_name(target)
                                        ),
                                        self.body.blocks[bid as usize].span,
                                    );
                                }
                                ValueType::Str => {
                                    if scan.has_owned_producing {
                                        self.error(
                                            "verify.borrow-escape",
                                            format!(
                                                "assignment to `{}` stores a view of a temporary",
                                                self.local_name(target)
                                            ),
                                            self.body.blocks[bid as usize].span,
                                        );
                                    }
                                    let prov = self.owned_reach(&st_snapshot, &scan);
                                    frame.st.backing.insert(target, prov);
                                }
                                _ => {}
                            }
                            for l in moved {
                                frame.st.live.remove(&l);
                                frame.st.dead.insert(l, DeadCause::Moved);
                            }
                        }
                        hir::Stmt::Return { value } => {
                            let scan = scan_expr(m, self.body, value);
                            self.reject_conditional_moves(&scan);
                            let st_snapshot = frame.st.clone();
                            self.check_reads(&st_snapshot, &chain, &scan.reads);
                            let moved = self.validate_moves(m, &st_snapshot, &scan, &chain);
                            // r2 QA-03: apply the return expression's moves
                            // FIRST — a local transferred into the callee is
                            // the CALLEE's cleanup responsibility; the caller
                            // must not schedule its own free for it.
                            for l in &moved {
                                frame.st.live.remove(l);
                                frame.st.dead.insert(*l, DeadCause::Moved);
                            }
                            match self.fn_result {
                                ValueType::StrOwned => {
                                    self.plan.owned_returns.insert((bid, idx));
                                    if let hir::ExprKind::ReadLocal { local } =
                                        &self.body.exprs[value.0 as usize].kind
                                    {
                                        // r2 QA-04: only a local this body OWNS
                                        // (an owned binding or a move-in
                                        // parameter) may transfer out; a view
                                        // or mut parameter's buffer belongs to
                                        // the caller.
                                        if !self.owned.contains(&local.0) {
                                            self.error(
                                                "verify.ownership-required",
                                                format!(
                                                    "cannot return `{}` as owned `Str`: it is a borrowed parameter/view whose buffer this function does not own; return an owned construction or a `move` parameter",
                                                    self.local_name(local.0)
                                                ),
                                                self.body.exprs[value.0 as usize].span,
                                            );
                                        }
                                    }
                                }
                                ValueType::Str => {
                                    let prov = self.owned_reach(&st_snapshot, &scan);
                                    if scan.has_owned_producing || !prov.is_empty() {
                                        self.error(
                                            "verify.borrow-escape",
                                            "returned borrowed str must not read owned or mut parameters, owned locals, or view temporaries (legal sources: view parameters, static literals)",
                                            self.body.blocks[bid as usize].span,
                                        );
                                    }
                                }
                                _ => {}
                            }
                            // frees are the POST-move live set: the returned
                            // owned local itself escapes (not freed here)
                            let mut frees: BTreeSet<u32> = frame.st.live.keys().copied().collect();
                            if self.fn_result == ValueType::StrOwned {
                                if let hir::ExprKind::ReadLocal { local } =
                                    &self.body.exprs[value.0 as usize].kind
                                {
                                    frees.remove(&local.0);
                                }
                            }
                            if !frees.is_empty() {
                                self.plan
                                    .exit_frees
                                    .insert((bid, Some(idx)), Self::lifo(&frees));
                            }
                            frame.flow_done = true;
                        }
                        hir::Stmt::If { cond, then, els } => {
                            let scan = scan_expr(m, self.body, cond);
                            self.reject_conditional_moves(&scan);
                            let st_snapshot = frame.st.clone();
                            self.check_reads(&st_snapshot, &chain, &scan.reads);
                            let moved = self.validate_moves(m, &st_snapshot, &scan, &chain);
                            for l in moved {
                                frame.st.live.remove(&l);
                                frame.st.dead.insert(l, DeadCause::Moved);
                            }
                            // r2 QA-03: join responsibility analysis. Every
                            // arm REACHABLE at the join (constant conditions
                            // prune statically; terminating arms exit
                            // elsewhere and never join) must agree on each
                            // owned local: moved on ALL incoming paths (the
                            // callee owns it — this frame drops the free
                            // responsibility) or on NONE (this frame keeps
                            // it). Anything else is rejected: a maybe-move
                            // has no consistent static free.
                            let cond_t = const_truth(self.body, cond);
                            let mut then_moved = BTreeSet::new();
                            block_definite_moves(m, self.body, then.0, &mut then_moved);
                            let then_terms = block_terminates(self.body, then.0);
                            let mut incoming: Vec<(&str, BTreeSet<u32>)> = Vec::new();
                            if cond_t != Some(false) && !then_terms {
                                incoming.push(("then arm", then_moved.clone()));
                            }
                            match els {
                                Some(e) => {
                                    let mut else_moved = BTreeSet::new();
                                    block_definite_moves(m, self.body, e.0, &mut else_moved);
                                    let else_terms = block_terminates(self.body, e.0);
                                    if cond_t != Some(true) && !else_terms {
                                        incoming.push(("else arm", else_moved));
                                    }
                                }
                                None => {
                                    // the implicit fall-through path
                                    if cond_t != Some(true) {
                                        incoming.push(("implicit fall-through", BTreeSet::new()));
                                    }
                                }
                            }
                            let union: BTreeSet<u32> = incoming
                                .iter()
                                .flat_map(|(_, ms)| ms.iter().copied())
                                .collect();
                            let mut join_dead = BTreeSet::new();
                            for l in &union {
                                let missing = incoming
                                    .iter()
                                    .find(|(_, ms)| !ms.contains(l))
                                    .map(|(arm, _)| *arm);
                                match missing {
                                    Some(arm) => self.error(
                                        "verify.ownership-required",
                                        format!(
                                            "owned local `{}` is moved inside this conditional but not on the {} path; the join would have inconsistent cleanup responsibility (hoist the move, or move on every path)",
                                            self.local_name(*l),
                                            arm
                                        ),
                                        self.body.blocks[bid as usize].span,
                                    ),
                                    None => {
                                        join_dead.insert(*l);
                                    }
                                }
                            }
                            let cur_in = frame.st.clone();
                            let loops_len = frame.loops_len;
                            let parent_idx = frames.len() - 1;
                            let else_slot = match els {
                                Some(e) => push_child(
                                    &mut frames,
                                    &mut results,
                                    e.0,
                                    loops_len,
                                    cur_in.clone(),
                                ),
                                None => usize::MAX,
                            };
                            let then_slot =
                                push_child(&mut frames, &mut results, then.0, loops_len, cur_in);
                            frames[parent_idx].if_join_dead = join_dead;
                            frames[parent_idx].state = RState::AwaitIf {
                                then_slot,
                                else_slot,
                            };
                        }
                        hir::Stmt::Loop { id, body: b } => {
                            // Loop re-entry rule: a local LIVE AT LOOP ENTRY
                            // (initialized before the loop) that is moved
                            // anywhere in the body is dead for the whole
                            // body — a second iteration would move it again.
                            // Locals LET INSIDE the body are fresh owners
                            // each iteration (r2 QA-07) and are not killed.
                            let mut moves_inside = BTreeSet::new();
                            block_definite_moves(m, self.body, b.0, &mut moves_inside);
                            moves_inside.retain(|l| frame.st.live.contains_key(l));
                            let mut child = frame.st.clone();
                            for l in &moves_inside {
                                child.live.remove(l);
                                child.dead.entry(*l).or_insert(DeadCause::Moved);
                            }
                            let parent_idx = frames.len() - 1;
                            loops.push((id, b.0));
                            let body_slot =
                                push_child(&mut frames, &mut results, b.0, loops.len(), child);
                            frames[parent_idx].state = RState::AwaitLoop { body_slot };
                        }
                        hir::Stmt::Break { target } | hir::Stmt::Continue { target } => {
                            // Free everything scoped inside the exited loop:
                            // live owned locals whose let-block lies at or
                            // below the target loop's body block on the
                            // current block chain.
                            if let Some(&(_, body_bid)) =
                                loops.iter().find(|(lid, _)| *lid == target)
                            {
                                let start = chain.iter().position(|&c| c == body_bid);
                                if let Some(start) = start {
                                    let frees: BTreeSet<u32> = frame
                                        .st
                                        .live
                                        .iter()
                                        .filter(|(_, &let_block)| {
                                            chain
                                                .iter()
                                                .position(|&c| c == let_block)
                                                .map_or(false, |p| p >= start)
                                        })
                                        .map(|(&l, _)| l)
                                        .collect();
                                    if !frees.is_empty() {
                                        self.plan
                                            .exit_frees
                                            .insert((bid, Some(idx)), Self::lifo(&frees));
                                    }
                                }
                            }
                            frame.flow_done = true;
                        }
                        hir::Stmt::Drop { local } => {
                            if !self.owned.contains(&local.0) {
                                self.error(
                                    "verify.resource-assign",
                                    format!(
                                        "drop of `{}`: only owned Str locals can be dropped",
                                        self.local_name(local.0)
                                    ),
                                    self.body.blocks[bid as usize].span,
                                );
                            } else if frame.st.dead.contains_key(&local.0) {
                                self.error(
                                    "verify.use-after-free",
                                    format!(
                                        "local `{}` is dropped again after an earlier drop or move",
                                        self.local_name(local.0)
                                    ),
                                    self.body.blocks[bid as usize].span,
                                );
                            } else if !frame.st.live.contains_key(&local.0) {
                                self.error(
                                    "verify.uninitialized-read",
                                    format!(
                                        "drop of owned local `{}` before initialization",
                                        self.local_name(local.0)
                                    ),
                                    self.body.blocks[bid as usize].span,
                                );
                            } else {
                                self.plan.exit_frees.insert((bid, Some(idx)), vec![local.0]);
                                frame.st.live.remove(&local.0);
                                frame.st.dead.insert(local.0, DeadCause::Dropped);
                            }
                        }
                    }
                }
                Action::AfterTerm | Action::Complete => {
                    let frame = frames.pop().expect("resource walk frame");
                    if matches!(action, Action::Complete) {
                        // Fall-through end: free this block's own lets that
                        // are still live (descendant blocks already freed
                        // theirs at their own ends; parents' locals survive
                        // the join). AfterTerm frees nothing — the
                        // terminator already emitted its exit frees.
                        let own: BTreeSet<u32> = frame
                            .own_lets
                            .iter()
                            .copied()
                            .filter(|l| frame.st.live.contains_key(l))
                            .collect();
                        if !own.is_empty() {
                            self.plan
                                .exit_frees
                                .insert((frame.bid, None), Self::lifo(&own));
                        }
                    }
                    results[frame.ret_slot] = Some(frame.st);
                    loops.truncate(frame.loops_len);
                }
                Action::Resume(state) => {
                    let frame = frames.last_mut().expect("resource walk frame");
                    match state {
                        RState::AwaitIf {
                            then_slot,
                            else_slot,
                        } => {
                            let then_st = results[then_slot].clone().expect("then arm state");
                            let else_st = if else_slot == usize::MAX {
                                None
                            } else {
                                results[else_slot].clone()
                            };
                            merge_children(&mut frame.st, &[Some(then_st), else_st]);
                            // r2 QA-03: locals moved on every joining path
                            // are the callee's responsibility — drop this
                            // frame's free duty (dead already merged).
                            for l in std::mem::take(&mut frame.if_join_dead) {
                                frame.st.live.remove(&l);
                            }
                        }
                        RState::AwaitLoop { body_slot } => {
                            let body_st = results[body_slot].clone().expect("loop body state");
                            merge_children(&mut frame.st, &[Some(body_st)]);
                            loops.truncate(frame.loops_len);
                        }
                        RState::Run => unreachable!("Run is not resumed"),
                    }
                }
            }
        }
    }
}

fn span_of_last(reads: &[(u32, u32, Span)]) -> &Span {
    reads
        .last()
        .map(|(_, _, s)| s)
        .unwrap_or(&Span { start: 0, end: 0 })
}

/// Whether a local is a `mut` parameter (an exclusive-borrow aliasing
/// source for conflict and escape purposes, r2 QA-06).
fn is_mut_param(l: &hir::LocalDef) -> bool {
    l.role == hir::LocalRole::Param && l.param_mode == hir::ParamMode::MutBorrow
}

fn place_local(body: &hir::Body, place: &hir::PlaceId) -> u32 {
    body.places
        .get(place.0 as usize)
        .map(|p| p.local.0)
        .unwrap_or(u32::MAX)
}

fn push_child(
    frames: &mut Vec<RFrame>,
    results: &mut Vec<Option<WalkState>>,
    bid: u32,
    loops_len: usize,
    init: WalkState,
) -> usize {
    let slot = results.len();
    results.push(None);
    frames.push(RFrame {
        bid,
        loops_len,
        idx: 0,
        st: init,
        own_lets: Vec::new(),
        flow_done: false,
        ret_slot: slot,
        state: RState::Run,
        if_join_dead: BTreeSet::new(),
    });
    slot
}

/// Merge child states into the parent at a join: dead and backing are the
/// UNION over reachable paths (moved/borrowed on any path ⇒ moved/borrowed
/// after the join — conservative and sound); live needs no merge because a
/// parent's own live set is only ever extended by its own lets.
fn merge_children(parent: &mut WalkState, children: &[Option<WalkState>]) {
    for c in children.iter().flatten() {
        for (l, cause) in &c.dead {
            parent.dead.entry(*l).or_insert(*cause);
        }
        for (v, bs) in &c.backing {
            parent
                .backing
                .entry(*v)
                .or_default()
                .extend(bs.iter().copied());
        }
    }
}

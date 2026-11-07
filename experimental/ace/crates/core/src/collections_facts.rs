//! PLAN-904 (AX-C): verified collections body facts — the authoritative
//! own/consume/exit responsibility summary for the collections source
//! domain (`auto.source.collections.draft`).
//!
//! Contract (plan §5.3/§5.4): the collections domain carries no
//! `TextResourcePlan`; its frame contract is frame-owned LIFO release.
//! This module derives, from the ALREADY-VERIFIED HIR body, the schedule
//! the Rust target must obey — never a second ownership analyzer: the
//! walk consumes exactly the verifier-admitted node set and the type
//! table the binder produced. The facts bind to one body of one owner in
//! one domain; the credential layer (`verify::verify_function_ref`)
//! pairs them with the body content digest so a stale / foreign / mixed
//! credential cannot pass them off (V-13).
//!
//! Schedules produced:
//! * `owned` — locals whose type transitively owns resources (Str buffer,
//!   list backing, record/variant with resource payloads) and therefore
//!   participates in scheduled cleanup.
//! * `exit_frees` — at every NORMAL exit (return / break / continue /
//!   block fall-through = loop back-edge) the owned slots to free, in
//!   reverse creation order (LIFO), EXCLUDING values transferred out.
//!   Branch joins are path-exact: each branch fall-through frees what
//!   does not survive the join (scoped deaths plus branch-live values
//!   not in the common set); a moved local (ownership transferred out)
//!   never appears in any free set.
//! * `replace_frees` — an assignment to a live owned local frees the old
//!   value at commit (after the RHS is fully evaluated and pinned).
//! * `err_frees` — at every fallible expression node, the LIFO free set
//!   (pending temps of the current expression, then live locals) the
//!   target must run on the ERROR propagation path: trap/cancel unwind
//!   cleans the frame exactly once, like every other exit.
//!
//! Pending temps: lowering-visible owned intermediates (clone/concat
//! results, literals promoted at owned positions, owned ctor/call
//! results) live from their producing node until their consuming node
//! commits; on a sibling failure they are the "uncommitted input
//! responsibility" the frame reclaims. The commit protocol truncates a
//! node's children temps when the node's own value takes custody.

use std::collections::{BTreeMap, BTreeSet};

use crate::hir::{self, ExprId, PlaceKind, Stmt, TypeUse};

/// One scheduled free target: a named local or a pending expression temp.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FreeRef {
    /// An owned local (LocalId num) — free via its type's helper.
    Local(u32),
    /// A pending temp produced by expr `ExprId num` — free via `kind`.
    Temp(u32, OwnedKind),
}

/// How an owned value is released (mirrors the T-02 representation freeze).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum OwnedKind {
    /// Owned Str buffer (`a2x_str_free`).
    Str,
    /// List backing (`a2x_coll_list_free` — elements LIFO, buffer last).
    List,
    /// Nominal record/variant with resource payloads (generated deep free
    /// keyed by the module type id).
    Deep(hir::TypeId),
}

/// The verified cleanup/responsibility facts of ONE collections body.
#[derive(Clone, Debug, Default)]
pub struct CollectionsBodyFacts {
    /// Owned locals (LocalId nums) in creation order (= id order; frees
    /// run in reverse).
    pub owned: BTreeSet<u32>,
    /// (block id, stmt index or fall-through None) → LIFO frees.
    pub exit_frees: BTreeMap<(u32, Option<usize>), Vec<FreeRef>>,
    /// (block id, stmt index) → local whose OLD value frees at commit.
    pub replace_frees: BTreeMap<(u32, usize), u32>,
    /// fallible ExprId num → LIFO frees on the error path.
    pub err_frees: BTreeMap<u32, Vec<FreeRef>>,
    /// loop BODY block num → the owned locals that SURVIVE the loop (the
    /// back-edge poll's error path must free them alongside the per-
    /// iteration set).
    pub loop_survivors: BTreeMap<u32, Vec<u32>>,
}

impl CollectionsBodyFacts {
    pub fn is_empty(&self) -> bool {
        self.owned.is_empty()
            && self.exit_frees.is_empty()
            && self.replace_frees.is_empty()
            && self.err_frees.is_empty()
            && self.loop_survivors.is_empty()
    }
}

/// Whole-module plan: (module index, body id num) → facts. Mirrors the
/// `TextResourcePlan` shape.
#[derive(Clone, Debug, Default)]
pub struct CollectionsPlan {
    pub bodies: BTreeMap<(usize, u32), CollectionsBodyFacts>,
}

/// Per-type owned-resource classification of a module type table.
struct TypeClassifier {
    kind: BTreeMap<u32, Option<OwnedKind>>,
}

impl TypeClassifier {
    fn build(module: &hir::Module) -> Self {
        let mut kind: BTreeMap<u32, Option<OwnedKind>> = BTreeMap::new();
        // fixpoint: record/variant payloads may reference later table
        // entries (types are acyclic, so this converges)
        loop {
            let mut changed = false;
            for (i, td) in module.types.iter().enumerate() {
                let tid = hir::TypeId(i as u32);
                let owns = match &td.kind {
                    hir::TypeKind::Record(shape) => shape
                        .fields
                        .iter()
                        .any(|f| owned_of_use(&kind, &f.ty).is_some()),
                    hir::TypeKind::Variant { arms } => arms.iter().any(|a| {
                        a.payload
                            .as_ref()
                            .is_some_and(|p| owned_of_use(&kind, p).is_some())
                    }),
                    _ => false,
                };
                let next = match &td.kind {
                    hir::TypeKind::I32
                    | hir::TypeKind::Bool
                    | hir::TypeKind::Char
                    | hir::TypeKind::Str
                    | hir::TypeKind::Function(_)
                    | hir::TypeKind::Closure(_)
                    // PLAN-961: numeric scalars own nothing.
                    | hir::TypeKind::I8
                    | hir::TypeKind::U8
                    | hir::TypeKind::U32
                    | hir::TypeKind::I64
                    | hir::TypeKind::U64
                    | hir::TypeKind::F32
                    | hir::TypeKind::F64 => None,
                    hir::TypeKind::StrOwned => Some(OwnedKind::Str),
                    hir::TypeKind::List { .. } => Some(OwnedKind::List),
                    // records/variants release through a generated deep
                    // free keyed by their OWN type id, even when the
                    // resource comes from a payload
                    hir::TypeKind::Record(_) | hir::TypeKind::Variant { .. } => {
                        owns.then_some(OwnedKind::Deep(tid))
                    }
                };
                if kind.get(&(i as u32)) != Some(&next) {
                    kind.insert(i as u32, next);
                    changed = true;
                }
            }
            if !changed {
                break;
            }
        }
        TypeClassifier { kind }
    }

    fn owned_of(&self, u: &TypeUse) -> Option<OwnedKind> {
        owned_of_use(&self.kind, u)
    }
}

fn owned_of_use(kind: &BTreeMap<u32, Option<OwnedKind>>, u: &TypeUse) -> Option<OwnedKind> {
    match u {
        TypeUse::Builtin(hir::BuiltinType::StrOwned) => Some(OwnedKind::Str),
        TypeUse::Builtin(_) => None,
        TypeUse::Ref(tid) => kind.get(&tid.0).copied().flatten(),
    }
}

/// Compute the facts of one body. The body must already have passed the
/// collections verifier rules (C2..C10); this walk never re-derives
/// legality, only the cleanup schedule.
pub fn compute_body_facts(module: &hir::Module, body: &hir::Body) -> CollectionsBodyFacts {
    let classifier = TypeClassifier::build(module);
    let mut w = Walker {
        body,
        classifier: &classifier,
        facts: CollectionsBodyFacts::default(),
        live: BTreeSet::new(),
        def_block: BTreeMap::new(),
        pending: Vec::new(),
        loops: Vec::new(),
    };
    for (i, l) in body.locals.iter().enumerate() {
        if classifier.owned_of(&l.ty).is_none() {
            continue;
        }
        // borrowed-mode params are not owned frame slots; a MoveOwn param
        // enters live at the frame head
        if l.role == hir::LocalRole::Param {
            match l.param_mode {
                hir::ParamMode::View | hir::ParamMode::MutBorrow => continue,
                hir::ParamMode::MoveOwn => {
                    w.facts.owned.insert(i as u32);
                    w.live.insert(i as u32);
                    w.def_block.insert(i as u32, body.entry.0);
                }
            }
        } else {
            w.facts.owned.insert(i as u32);
        }
    }
    let _ = w.block(body.entry);
    w.facts
}

struct Walker<'a> {
    body: &'a hir::Body,
    classifier: &'a TypeClassifier,
    facts: CollectionsBodyFacts,
    /// Live owned locals at the current program point.
    live: BTreeSet<u32>,
    /// owned local → defining block num (scoped death at fall-through).
    def_block: BTreeMap<u32, u32>,
    /// Pending owned temps of the current statement's expression (in
    /// creation order; frees run in reverse).
    pending: Vec<FreeRef>,
    /// Enclosing loop frames: (loop id, live set that survives the loop).
    loops: Vec<(hir::LoopId, BTreeSet<u32>)>,
}

impl<'a> Walker<'a> {
    /// Dry scan of every kill site (Move / VariantProject / moved call
    /// binding rooted at a whole-local read) within a block subtree.
    fn collect_kills(&self, id: hir::BlockId, out: &mut BTreeSet<u32>) {
        let block = &self.body.blocks[id.0 as usize];
        for stmt in &block.stmts {
            match stmt {
                Stmt::Let { value, .. } | Stmt::Return { value } | Stmt::Eval { value } => {
                    self.collect_kills_expr(*value, out)
                }
                Stmt::Assign { value, .. } => self.collect_kills_expr(*value, out),
                Stmt::If { cond, then, els } => {
                    self.collect_kills_expr(*cond, out);
                    self.collect_kills(*then, out);
                    if let Some(e) = els {
                        self.collect_kills(*e, out);
                    }
                }
                Stmt::Loop { body: lb, .. } => self.collect_kills(*lb, out),
                _ => {}
            }
        }
    }

    fn collect_kills_expr(&self, e: hir::ExprId, out: &mut BTreeSet<u32>) {
        use hir::ExprKind as E;
        let kind = &self.body.exprs[e.0 as usize].kind;
        let kids: Vec<hir::ExprId> = match kind {
            E::Move { inner } => {
                if let E::ReadLocal { local } = self.body.exprs[inner.0 as usize].kind {
                    out.insert(local.0);
                }
                vec![*inner]
            }
            E::VariantProject { root, .. } => {
                if let E::ReadLocal { local } = self.body.exprs[root.0 as usize].kind {
                    out.insert(local.0);
                }
                vec![*root]
            }
            E::Call {
                eval_args,
                bindings,
                ..
            } => {
                for b in bindings {
                    if b.moved {
                        if let Some(&aid) = eval_args.get(b.arg as usize) {
                            if let E::ReadLocal { local } = self.body.exprs[aid.0 as usize].kind {
                                out.insert(local.0);
                            }
                        }
                    }
                }
                eval_args.clone()
            }
            E::Binary { lhs, rhs, .. } => vec![*lhs, *rhs],
            E::Unary { operand, .. } => vec![*operand],
            E::Logical { lhs, rhs, .. } => vec![*lhs, *rhs],
            E::CallClosure { callee, eval_args } => {
                let mut v = vec![*callee];
                v.extend(eval_args.iter().copied());
                v
            }
            E::TextLen { operand }
            | E::TextLenBytes { operand }
            | E::TextClone { operand }
            | E::ClosureClone { operand }
            | E::TextToOwned { operand }
            | E::TextToBorrowed { operand }
            | E::CharToCodepoint { operand }
            | E::CharFromCodepoint { operand }
            | E::FieldGet { base: operand, .. }
            | E::BorrowView { base: operand }
            | E::RecordClone { base: operand, .. }
            | E::ListLen { list: operand }
            | E::ListClear { list: operand }
            | E::VariantGuardTest { root: operand, .. } => vec![*operand],
            E::TextCharAt { operand, index } => vec![*operand, *index],
            E::TextSlice {
                operand,
                start,
                end,
            } => vec![*operand, *start, *end],
            E::TextConcat { lhs, rhs } => vec![*lhs, *rhs],
            E::RecordCtor { eval_args, .. } => eval_args.clone(),
            E::ListPush { list, value } => vec![*list, *value],
            E::ListInsert { list, index, value } => vec![*list, *index, *value],
            E::ListGet { list, index } => vec![*list, *index],
            E::ListSet { list, index, value } => vec![*list, *index, *value],
            E::ListRemove { list, index } => vec![*list, *index],
            E::VariantCtor { payload, .. } => payload.iter().copied().collect(),
            E::ReadLocal { .. }
            | E::Constant { .. }
            | E::Unit
            | E::StrLit { .. }
            | E::CharLit { .. }
            | E::BorrowMut { .. }
            | E::CreateClosure { .. }
            | E::ListNew { .. } => Vec::new(),
        };
        for k in kids {
            self.collect_kills_expr(k, out);
        }
    }

    fn loop_after(&self, target: hir::LoopId) -> BTreeSet<u32> {
        self.loops
            .iter()
            .rev()
            .find(|(id, _)| *id == target)
            .map(|(_, after)| after.clone())
            .unwrap_or_default()
    }
    /// Walk one block; returns whether the block TERMINATED (a return/
    /// break/continue made its tail unreachable — a terminated branch
    /// contributes nothing to a join's intersection).
    fn block(&mut self, id: hir::BlockId) -> bool {
        let mut terminated = false;
        let stmts = self.body.blocks[id.0 as usize].stmts.clone();
        for (si, stmt) in stmts.iter().enumerate() {
            match stmt {
                Stmt::Let { local, value } => {
                    let mark = self.pending.len();
                    self.expr(*value);
                    // the value's own pending becomes the local
                    self.pending.truncate(mark);
                    if self.facts.owned.contains(&local.0) {
                        self.live.insert(local.0);
                        self.def_block.insert(local.0, id.0);
                    }
                }
                Stmt::Return { value } => {
                    let mark = self.pending.len();
                    self.expr(*value);
                    // the returned value transfers out (whole-local reads
                    // and fresh temps alike)
                    self.pending.truncate(mark);
                    if let hir::ExprKind::ReadLocal { local } =
                        self.body.exprs[value.0 as usize].kind
                    {
                        self.live.remove(&local.0);
                    }
                    let frees = self.lifo_live();
                    self.facts.exit_frees.insert((id.0, Some(si)), frees);
                    // unreachable tail: nothing stays live on this path
                    self.live.clear();
                    terminated = true;
                }
                Stmt::Assign { place, value } => {
                    let mark = self.pending.len();
                    self.expr(*value);
                    self.pending.truncate(mark);
                    let p = &self.body.places[place.0 as usize];
                    if p.kind == PlaceKind::Local && self.facts.owned.contains(&p.local.0) {
                        if self.live.contains(&p.local.0) {
                            self.facts.replace_frees.insert((id.0, si), p.local.0);
                        }
                        self.live.insert(p.local.0);
                    }
                }
                Stmt::If { cond, then, els } => {
                    self.expr(*cond);
                    let before = self.live.clone();
                    let then_term = self.block(*then);
                    let then_end = std::mem::take(&mut self.live);
                    let (els_end, els_term) = match els {
                        Some(e) => {
                            self.live = before.clone();
                            let t = self.block(*e);
                            (std::mem::take(&mut self.live), t)
                        }
                        None => (before.clone(), false),
                    };
                    // join: a branch that TERMINATED (returned / broke /
                    // continued) has no fall-through endpoint and does
                    // not constrain the join — PLAN-926 fix of the
                    // no-else early-return shape (a `common = ∅` from a
                    // terminated then-arm used to orphan the implicit
                    // else path's live set: no block ever freed it)
                    let common: BTreeSet<u32> = match (then_term, els_term) {
                        (true, true) => BTreeSet::new(),
                        (true, false) => els_end.clone(),
                        (false, true) => then_end.clone(),
                        (false, false) => {
                            let mut c: BTreeSet<u32> = then_end.clone();
                            c.retain(|l| els_end.contains(l));
                            c
                        }
                    };
                    let then_res: Vec<u32> = then_end.difference(&common).copied().collect();
                    self.append_frees(then.0, &then_res);
                    if let Some(e) = els {
                        let els_res: Vec<u32> = els_end.difference(&common).copied().collect();
                        self.append_frees(e.0, &els_res);
                    }
                    self.live = common;
                    if then_term && els_term {
                        terminated = true;
                    }
                }
                Stmt::Loop {
                    id: loop_id,
                    body: lb,
                } => {
                    let before = self.live.clone();
                    // dry-scan the kill sites first: the frame's survivor
                    // set must be known BEFORE the body walk so nested
                    // break/continue statements can compute their frees
                    let mut killed = BTreeSet::new();
                    self.collect_kills(*lb, &mut killed);
                    let after: BTreeSet<u32> = before.difference(&killed).copied().collect();
                    self.loops.push((*loop_id, after.clone()));
                    self.facts
                        .loop_survivors
                        .insert(lb.0, after.iter().copied().collect());
                    // a loop body's terminator never falls through to the
                    // loop's continuation; the survivor set governs
                    let _ = self.block(*lb);
                    self.loops.pop();
                    // the body block's fall-through (recorded inside)
                    // frees its scoped locals per iteration; the survivor
                    // set after the loop excludes anything moved in it
                    self.live = after;
                }
                Stmt::Break { target } => {
                    let after = self.loop_after(*target);
                    let frees: Vec<u32> = self.live.difference(&after).copied().collect();
                    self.facts
                        .exit_frees
                        .insert((id.0, Some(si)), self.lifo(&frees));
                    self.live.clear();
                    terminated = true;
                }
                Stmt::Continue { target } => {
                    let after = self.loop_after(*target);
                    let frees: Vec<u32> = self.live.difference(&after).copied().collect();
                    self.facts
                        .exit_frees
                        .insert((id.0, Some(si)), self.lifo(&frees));
                    self.live.clear();
                    terminated = true;
                }
                Stmt::Drop { local } => {
                    self.live.remove(&local.0);
                }
                Stmt::Eval { value } => {
                    let mark = self.pending.len();
                    self.expr(*value);
                    // the effect value consumes its temps
                    self.pending.truncate(mark);
                }
            }
        }
        // fall-through: scoped death of locals defined in this block
        let this_block = id.0;
        let dying: Vec<u32> = self
            .live
            .iter()
            .copied()
            .filter(|l| self.def_block.get(l) == Some(&this_block))
            .collect();
        self.append_frees(id.0, &dying);
        for l in dying {
            self.live.remove(&l);
        }
        terminated
    }

    fn append_frees(&mut self, block: u32, locals: &[u32]) {
        if locals.is_empty() {
            return;
        }
        let frees = self.lifo(locals);
        self.facts
            .exit_frees
            .entry((block, None))
            .or_default()
            .extend(frees);
    }

    fn lifo(&self, locals: &[u32]) -> Vec<FreeRef> {
        let mut ls = locals.to_vec();
        ls.sort_unstable_by(|a, b| b.cmp(a));
        ls.into_iter().map(FreeRef::Local).collect()
    }

    fn lifo_live(&mut self) -> Vec<FreeRef> {
        let live: Vec<u32> = self.live.iter().copied().collect();
        self.lifo(&live)
    }

    /// Record the error-path free set for a fallible node: pending temps
    /// (reverse creation order) then live locals (reverse creation order).
    fn record_err(&mut self, e: ExprId) {
        // idempotent per node (a literal promotion records once here even
        // though the parent's owned-slot walk visits it again)
        if self.facts.err_frees.contains_key(&e.0) {
            return;
        }
        let mut frees: Vec<FreeRef> = self.pending.iter().rev().copied().collect();
        let live: Vec<u32> = self.live.iter().copied().collect();
        frees.extend(self.lifo(&live));
        if !frees.is_empty() {
            self.facts.err_frees.insert(e.0, frees);
        }
    }

    /// Walk one expression with the commit protocol: children temps are
    /// recorded for this node's error path, then subsume into this node's
    /// own pending (or vanish when the node consumes them for effect).
    fn expr(&mut self, id: ExprId) {
        let mark = self.pending.len();
        self.expr_children(id);
        self.record_err(id);
        match self.temp_kind(id) {
            Some(k) => {
                self.pending.truncate(mark);
                self.pending.push(FreeRef::Temp(id.0, k));
            }
            None => self.pending.truncate(mark),
        }
    }

    /// Children in source evaluation order; owned-value child slots push
    /// their temp (owned-producing children do it themselves; a literal at
    /// an owned position is promoted by the parent).
    fn expr_children(&mut self, id: ExprId) {
        let kind = self.body.exprs[id.0 as usize].kind.clone();
        use hir::ExprKind as E;
        match kind {
            E::ReadLocal { .. }
            | E::Constant { .. }
            | E::Unit
            | E::CharLit { .. }
            | E::StrLit { .. }
            | E::BorrowMut { .. }
            | E::CreateClosure { .. } => {}
            E::Binary { lhs, rhs, .. } => {
                self.expr(lhs);
                self.expr(rhs);
            }
            E::Unary { operand, .. } => self.expr(operand),
            E::Logical { lhs, rhs, .. } => {
                self.expr(lhs);
                self.expr(rhs);
            }
            E::Call {
                eval_args,
                bindings,
                ..
            } => {
                for a in &eval_args {
                    self.expr(*a);
                }
                for b in &bindings {
                    if b.moved {
                        if let Some(&aid) = eval_args.get(b.arg as usize) {
                            if let E::ReadLocal { local } = self.body.exprs[aid.0 as usize].kind {
                                self.kill_local(local);
                            }
                        }
                    }
                }
            }
            E::CallClosure { callee, eval_args } => {
                self.expr(callee);
                for a in eval_args {
                    self.expr(a);
                }
            }
            E::TextLen { operand } | E::TextLenBytes { operand } => self.expr(operand),
            E::TextCharAt { operand, index } => {
                self.expr(operand);
                self.expr(index);
            }
            E::TextSlice {
                operand,
                start,
                end,
            } => {
                self.expr(operand);
                self.expr(start);
                self.expr(end);
            }
            E::TextConcat { lhs, rhs } => {
                self.expr(lhs);
                self.expr(rhs);
            }
            E::TextClone { operand } | E::ClosureClone { operand } => self.expr(operand),
            E::TextToOwned { operand } | E::TextToBorrowed { operand } => self.expr(operand),
            E::CharToCodepoint { operand } | E::CharFromCodepoint { operand } => self.expr(operand),
            E::RecordCtor { eval_args, .. } => {
                for a in eval_args {
                    self.owned_value_child(a);
                }
            }
            E::FieldGet { base, .. } => self.expr(base),
            E::BorrowView { base } => self.expr(base),
            E::RecordClone { base, .. } => self.expr(base),
            E::Move { inner } => {
                if let E::ReadLocal { local } = self.body.exprs[inner.0 as usize].kind {
                    self.kill_local(local);
                } else {
                    self.expr(inner);
                }
            }
            E::ListNew { .. } => {}
            E::ListLen { list } => self.expr(list),
            E::ListPush { list, value } => {
                self.expr(list);
                self.owned_value_child(value);
            }
            E::ListInsert { list, index, value } => {
                self.expr(list);
                self.expr(index);
                self.owned_value_child(value);
            }
            E::ListGet { list, index } => {
                self.expr(list);
                self.expr(index);
            }
            E::ListSet { list, index, value } => {
                self.expr(list);
                self.expr(index);
                self.owned_value_child(value);
            }
            E::ListRemove { list, index } => {
                self.expr(list);
                self.expr(index);
            }
            E::ListClear { list } => self.expr(list),
            E::VariantCtor { payload, .. } => {
                if let Some(p) = payload {
                    self.owned_value_child(p);
                }
            }
            E::VariantGuardTest { root, .. } => self.expr(root),
            E::VariantProject { root, .. } => {
                if let E::ReadLocal { local } = self.body.exprs[root.0 as usize].kind {
                    self.kill_local(local);
                } else {
                    self.expr(root);
                }
            }
        }
    }

    /// An owned-value child slot: the child's owned temp (pushed by its
    /// own walk) or, for a literal, the promotion temp the parent owns.
    /// The literal promotion (`a2x_str_from_lit`) is itself fallible —
    /// its error path frees what came before it, exactly like any other
    /// fallible node.
    fn owned_value_child(&mut self, child: ExprId) {
        let is_lit = matches!(
            self.body.exprs[child.0 as usize].kind,
            hir::ExprKind::StrLit { .. }
        );
        self.expr(child);
        if is_lit {
            self.record_err(child);
            self.pending.push(FreeRef::Temp(child.0, OwnedKind::Str));
        }
    }

    fn kill_local(&mut self, local: hir::LocalId) {
        self.live.remove(&local.0);
    }

    fn temp_kind(&self, id: ExprId) -> Option<OwnedKind> {
        let e = &self.body.exprs[id.0 as usize];
        match &e.kind {
            hir::ExprKind::TextConcat { .. }
            | hir::ExprKind::TextClone { .. }
            | hir::ExprKind::TextToOwned { .. } => Some(OwnedKind::Str),
            hir::ExprKind::ListNew { .. } => Some(OwnedKind::List),
            hir::ExprKind::RecordCtor { .. }
            | hir::ExprKind::RecordClone { .. }
            | hir::ExprKind::VariantCtor { .. }
            | hir::ExprKind::ListRemove { .. }
            | hir::ExprKind::VariantProject { .. }
            | hir::ExprKind::Call { .. } => self.classifier.owned_of(&e.ty),
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Body content digest (credential binding, V-13)
// ---------------------------------------------------------------------------

/// FNV-1a 64 over a canonical span-free body serialization. Used by the
/// credential layer to bind facts (and the credential itself) to the
/// exact body content: same owner + same declaration context + a
/// DIFFERENT body must not mix (PLAN-904 V-13).
pub fn body_content_digest(body: &hir::Body) -> String {
    let mut h = Fnv64::new();
    digest_body(&mut h, body);
    format!("{:016x}", h.finish())
}

struct Fnv64(u64);

impl Fnv64 {
    fn new() -> Self {
        Fnv64(0xcbf29ce484222325)
    }
    fn byte(&mut self, b: u8) {
        self.0 ^= b as u64;
        self.0 = self.0.wrapping_mul(0x100000001b3);
    }
    fn str(&mut self, s: &str) {
        for b in s.as_bytes() {
            self.byte(*b);
        }
        self.byte(0xff);
    }
    fn u32(&mut self, n: u32) {
        for b in n.to_le_bytes() {
            self.byte(b);
        }
    }
    fn finish(&self) -> u64 {
        self.0
    }
}

fn digest_body(h: &mut Fnv64, body: &hir::Body) {
    h.str("body");
    h.u32(body.entry.0);
    for l in &body.locals {
        h.str("local");
        h.str(&l.name);
        digest_type_use(h, &l.ty);
        h.str(l.role.as_str());
        h.str(l.param_mode.as_str());
        h.u32(l.mutable as u32);
    }
    for p in &body.places {
        h.str("place");
        h.str(&p.id_text);
        digest_type_use(h, &p.ty);
        h.u32(p.local.0);
        h.u32(match p.kind {
            PlaceKind::Local => 0,
            PlaceKind::FieldPath(_) => 1,
        });
    }
    for e in &body.exprs {
        h.str("expr");
        h.str(&e.id_text);
        digest_type_use(h, &e.ty);
        digest_expr_kind(h, &e.kind);
    }
    for b in &body.blocks {
        h.str("block");
        h.u32(b.id_text.len() as u32);
        for s in &b.stmts {
            digest_stmt(h, s);
        }
    }
}

fn digest_stmt(h: &mut Fnv64, s: &Stmt) {
    match s {
        Stmt::Let { local, value } => {
            h.str("let");
            h.u32(local.0);
            h.u32(value.0);
        }
        Stmt::Return { value } => {
            h.str("ret");
            h.u32(value.0);
        }
        Stmt::If { cond, then, els } => {
            h.str("if");
            h.u32(cond.0);
            h.u32(then.0);
            h.u32(els.map(|b| b.0).unwrap_or(u32::MAX));
        }
        Stmt::Loop { id, body } => {
            h.str("loop");
            h.u32(id.0);
            h.u32(body.0);
        }
        Stmt::Assign { place, value } => {
            h.str("assign");
            h.u32(place.0);
            h.u32(value.0);
        }
        Stmt::Break { target } => {
            h.str("break");
            h.u32(target.0);
        }
        Stmt::Continue { target } => {
            h.str("cont");
            h.u32(target.0);
        }
        Stmt::Drop { local } => {
            h.str("drop");
            h.u32(local.0);
        }
        Stmt::Eval { value } => {
            h.str("eval");
            h.u32(value.0);
        }
    }
}

fn digest_type_use(h: &mut Fnv64, t: &TypeUse) {
    match t {
        TypeUse::Builtin(b) => {
            h.str("b");
            h.str(b.as_str());
        }
        TypeUse::Ref(t) => {
            h.str("r");
            h.u32(t.0);
        }
    }
}

fn digest_expr_kind(h: &mut Fnv64, k: &hir::ExprKind) {
    use hir::ExprKind as E;
    match k {
        E::ReadLocal { local } => {
            h.str("rl");
            h.u32(local.0);
        }
        E::Constant { value } => {
            h.str("k");
            h.u32(*value as u32);
        }
        E::Unit => h.str("unit"),
        E::Binary {
            op,
            overflow,
            lhs,
            rhs,
        } => {
            h.str("bin");
            h.str(&format!("{op:?}"));
            h.str(&format!("{overflow:?}"));
            h.u32(lhs.0);
            h.u32(rhs.0);
        }
        E::Unary { op, operand } => {
            h.str("un");
            h.str(&format!("{op:?}"));
            h.u32(operand.0);
        }
        E::Logical { op, lhs, rhs } => {
            h.str("log");
            h.str(&format!("{op:?}"));
            h.u32(lhs.0);
            h.u32(rhs.0);
        }
        E::Call {
            callee,
            eval_args,
            bindings,
        } => {
            h.str("call");
            h.u32(callee.0);
            h.u32(eval_args.len() as u32);
            for a in eval_args {
                h.u32(a.0);
            }
            for b in bindings {
                h.u32(b.param);
                h.u32(b.arg);
                h.u32(b.moved as u32);
            }
        }
        E::CallClosure { callee, eval_args } => {
            h.str("callc");
            h.u32(callee.0);
            for a in eval_args {
                h.u32(a.0);
            }
        }
        E::CreateClosure { callee, captures } => {
            h.str("mkc");
            h.u32(callee.0);
            for c in captures {
                h.u32(c.local.0);
                h.str(&format!("{:?}", c.mode));
            }
        }
        E::StrLit { value } => {
            h.str("slit");
            h.str(value);
        }
        E::CharLit { scalar } => {
            h.str("clit");
            h.u32(*scalar);
        }
        E::TextLen { operand } => {
            h.str("tlen");
            h.u32(operand.0);
        }
        E::TextLenBytes { operand } => {
            h.str("tlenb");
            h.u32(operand.0);
        }
        E::TextCharAt { operand, index } => {
            h.str("tca");
            h.u32(operand.0);
            h.u32(index.0);
        }
        E::TextSlice {
            operand,
            start,
            end,
        } => {
            h.str("tsl");
            h.u32(operand.0);
            h.u32(start.0);
            h.u32(end.0);
        }
        E::TextConcat { lhs, rhs } => {
            h.str("tcat");
            h.u32(lhs.0);
            h.u32(rhs.0);
        }
        E::TextClone { operand } => {
            h.str("tcl");
            h.u32(operand.0);
        }
        E::ClosureClone { operand } => {
            h.str("ccl");
            h.u32(operand.0);
        }
        E::TextToOwned { operand } => {
            h.str("t2o");
            h.u32(operand.0);
        }
        E::TextToBorrowed { operand } => {
            h.str("t2b");
            h.u32(operand.0);
        }
        E::CharToCodepoint { operand } => {
            h.str("c2i");
            h.u32(operand.0);
        }
        E::CharFromCodepoint { operand } => {
            h.str("i2c");
            h.u32(operand.0);
        }
        E::RecordCtor {
            record,
            eval_args,
            bindings,
        } => {
            h.str("rct");
            h.u32(record.0);
            h.u32(eval_args.len() as u32);
            for b in bindings {
                h.u32(b.field);
                h.u32(b.arg);
            }
        }
        E::FieldGet { base, path } => {
            h.str("fget");
            h.u32(base.0);
            for f in path {
                h.u32(f.0);
            }
        }
        E::BorrowView { base } => {
            h.str("bv");
            h.u32(base.0);
        }
        E::BorrowMut { local } => {
            h.str("bm");
            h.u32(local.0);
        }
        E::RecordClone { record, base } => {
            h.str("rcl");
            h.u32(record.0);
            h.u32(base.0);
        }
        E::Move { inner } => {
            h.str("mv");
            h.u32(inner.0);
        }
        E::ListNew { list } => {
            h.str("ln");
            h.u32(list.0);
        }
        E::ListLen { list } => {
            h.str("ll");
            h.u32(list.0);
        }
        E::ListPush { list, value } => {
            h.str("lp");
            h.u32(list.0);
            h.u32(value.0);
        }
        E::ListInsert { list, index, value } => {
            h.str("li");
            h.u32(list.0);
            h.u32(index.0);
            h.u32(value.0);
        }
        E::ListGet { list, index } => {
            h.str("lg");
            h.u32(list.0);
            h.u32(index.0);
        }
        E::ListSet { list, index, value } => {
            h.str("ls");
            h.u32(list.0);
            h.u32(index.0);
            h.u32(value.0);
        }
        E::ListRemove { list, index } => {
            h.str("lr");
            h.u32(list.0);
            h.u32(index.0);
        }
        E::ListClear { list } => {
            h.str("lc");
            h.u32(list.0);
        }
        E::VariantCtor {
            variant,
            disc,
            payload,
        } => {
            h.str("vc");
            h.u32(variant.0);
            h.u32(*disc);
            h.u32(payload.map(|p| p.0).unwrap_or(u32::MAX));
        }
        E::VariantGuardTest { root, disc } => {
            h.str("vg");
            h.u32(root.0);
            h.u32(*disc);
        }
        E::VariantProject { root, disc } => {
            h.str("vp");
            h.u32(root.0);
            h.u32(*disc);
        }
    }
}

//! Owned-profile ownership dataflow (PLAN-821, contract-decision §5).
//!
//! The extension of 820's definite-initialization must-worklist to the
//! full slot-state lattice of the owned profile:
//!
//! * `init`    — MUST (intersection at merges): the slot has been defined.
//! * `moved`   — MAY (union at merges): killed by MOVE / CALL move-in /
//!   AGGREGATE/FIELD_SET operand transfer on ANY reachable path.
//! * `dropped` — MAY (union): explicit DROP consumed the value (T-03 gate).
//! * live borrow tokens — MAY (union): StrView slots holding an active
//!   borrow, each carrying a root-set (slots it is rooted at, plus the
//!   EXTERNAL sentinel for params that arrived as views from the caller).
//!
//! §5.5 r3 rules: reads need init ∧ ¬moved; a moved root on any path
//! blocks subsequent reads (V-11/V-12 branch-merge discrimination);
//! active tokens block root writes/moves (root-conservative); RETURN of a
//! StrView requires the root-set to be empty-or-external (borrow-escape,
//! 782). Loop back-edges iterate to a fixpoint from the optimistic empty
//! MAY states (zero-iteration loops stay conservative: anything the back
//! edge may carry flows in).
//!
//! Only slots the program actually references participate (the tokens and
//! roots ARE slot indices — no per-field explosion; worklist steps charge
//! the shared verification budget).

use crate::contract::MAX_TABLE_ENTRIES;
use crate::mir::{CalleeRef, MirInstr, MirModule, MirType, SlotKind, Terminator};
use crate::mir_verify::MirVerifyError;
use crate::SpanInfo;
use std::collections::VecDeque;

/// The sentinel root bit (one past the last slot): a token whose roots
/// include it flows from a StrView PARAM (the caller's frame — the callee
/// cannot and does not enumerate it).
// The EXTERNAL sentinel root bit is index `n_slots` (set up per function);
// no standalone constant is needed.

/// Whether a type owns resources (PLAN-782 derived classification — never
/// input-claimed): `Str`, or a record whose transitive field closure
/// reaches `Str`. Copyable records/scalars return false.
pub fn type_is_owned(m: &MirModule, ty: u32) -> bool {
    match m.types.get(ty as usize) {
        Some(t) => type_is_owned_type_of(m, t),
        None => false,
    }
}

/// Type-value form of [] (adapter-side convenience).
pub fn type_is_owned_type_of(m: &MirModule, t: &MirType) -> bool {
    match t {
        MirType::Str => true,
        MirType::Record(r) => record_reaches_str(m, r),
        // PLAN-822: containers own (buffers/elements/payloads need drop).
        MirType::List(_) | MirType::Variant(_) => true,
        _ => false,
    }
}

/// Transitive Str-reachability from one record shape (the derived resource
/// classification of PLAN-782; shared with layout's `is_copy` derivation).
pub fn record_reaches_str(m: &MirModule, r: &crate::mir::RecordTypeRef) -> bool {
    // Iterative transitive closure over the (verified acyclic) field graph.
    let mut stack: Vec<u32> = r.fields.iter().map(|f| f.ty).collect();
    let mut seen = std::collections::BTreeSet::new();
    while let Some(t) = stack.pop() {
        if !seen.insert(t) {
            continue;
        }
        match m.types.get(t as usize) {
            Some(MirType::Str) => return true,
            Some(MirType::Record(inner)) => {
                for f in &inner.fields {
                    stack.push(f.ty);
                }
            }
            _ => {}
        }
    }
    false
}

/// Whether a slot type can ROOT a borrow (VIEW sources and FIELD_GET bases).
pub fn type_is_rootable(m: &MirModule, ty: u32) -> bool {
    matches!(
        m.types.get(ty as usize),
        Some(MirType::Str) | Some(MirType::Record(_))
    )
}

/// PLAN-822: whether the given variant arm's payload is an OWNED type (its
/// projection is a transfer; Copy payloads stay readable). `false` for
/// no-payload arms and unknown slots (typing caught those earlier).
pub fn variant_payload_is_owned(
    m: &MirModule,
    f: &crate::mir::MirFunction,
    slot: usize,
    disc: u32,
) -> bool {
    let Some(slot_ty) = f.slots.get(slot).map(|s| s.ty) else {
        return false;
    };
    let Some(MirType::Variant(v)) = m.types.get(slot_ty as usize) else {
        return false;
    };
    v.arms
        .get(disc as usize)
        .and_then(|a| a.payload)
        .map_or(false, |pt| type_is_owned(m, pt))
}

#[derive(Clone)]
struct Bitsets {
    words: Vec<u64>,
    n_bits: usize,
}

impl Bitsets {
    fn new(n_bits: usize) -> Self {
        Bitsets {
            words: vec![0u64; n_bits.div_ceil(64)],
            n_bits,
        }
    }
    fn set(&mut self, i: usize) {
        if i < self.n_bits {
            self.words[i / 64] |= 1 << (i % 64);
        }
    }
    fn clear(&mut self, i: usize) {
        if i < self.n_bits {
            self.words[i / 64] &= !(1 << (i % 64));
        }
    }
    fn get(&self, i: usize) -> bool {
        i < self.n_bits && (self.words[i / 64] >> (i % 64)) & 1 == 1
    }
    fn union_with(&mut self, o: &Bitsets) {
        for (a, b) in self.words.iter_mut().zip(o.words.iter()) {
            *a |= *b;
        }
    }
    fn intersect_with(&mut self, o: &Bitsets) {
        for (a, b) in self.words.iter_mut().zip(o.words.iter()) {
            *a &= *b;
        }
    }
    fn eq(&self, o: &Bitsets) -> bool {
        self.words == o.words
    }
}

/// One program-point ownership state.
#[derive(Clone)]
struct OwnState {
    init: Bitsets,    // must
    moved: Bitsets,   // may
    dropped: Bitsets, // may
    tokens_live: Bitsets,
    /// Per-slot root-set, valid where `tokens_live`. Bit domain: slots +
    /// one EXTERNAL sentinel at index `n_slots`. SPARSE (F821-R1-05): a
    /// slot that has never been a token carries `None` — the frozen
    /// contract interns only USED roots; the old all-slots matrix both
    /// blew the verify-memory budget unaccounted (n x (n+1) bits x
    /// states) and contradicted "只 intern 实际使用根".
    token_roots: Vec<Option<Bitsets>>,
    /// R3-F08 per-field consumption: the set of field paths already MOVED
    /// OUT of a root via FIELD_MOVE. SPARSE — only roots that saw a
    /// FIELD_MOVE carry `Some`. A recorded path may not be extracted (or
    /// read) again; the ROOT itself stays live for its remaining fields
    /// (the internal t_MutRet wrapper is consumed field by field — the
    /// multi-mut / owned-result source shapes), while any WHOLE-root use
    /// of a partially-consumed root is a use-after-move (the language
    /// keeps rejecting partial moves of source values; FIELD_MOVE is the
    /// adapter's internal channel).
    moved_fields: Vec<Option<std::collections::BTreeSet<Vec<u16>>>>,
    /// PLAN-822 variant guard facts — MUST facts (intersected at joins):
    /// the (root slot, disc) pairs proven by a VARIANT_GUARD since the
    /// root's last (re)definition. Facts die on the root's redefinition,
    /// a VARIANT_SWITCH, or a possibly-writing CALL (§5.5).
    guards: std::collections::BTreeSet<(usize, u32)>,
    /// PLAN-822 payload-moved arms — MAY facts (unioned at joins): per
    /// variant root, the arms whose OWNED payload was already projected
    /// out (the arm reads as moved; Copy payloads never land here).
    variant_moved: std::collections::BTreeMap<usize, std::collections::BTreeSet<u32>>,
}

impl OwnState {
    fn clone_empty_like(&self) -> OwnState {
        OwnState {
            init: Bitsets::new(self.init.n_bits),
            moved: Bitsets::new(self.moved.n_bits),
            dropped: Bitsets::new(self.dropped.n_bits),
            tokens_live: Bitsets::new(self.tokens_live.n_bits),
            token_roots: vec![None; self.token_roots.len()],
            moved_fields: vec![None; self.moved_fields.len()],
            guards: std::collections::BTreeSet::new(),
            variant_moved: std::collections::BTreeMap::new(),
        }
    }

    /// R4-F08: an accessed path is dead when ANY moved path OVERLAPS it
    /// — an already-extracted ANCESTOR took the whole subtree with it,
    /// and a value whose DESCENDANT was extracted is no longer a
    /// complete initialized value. Exact equality is not enough (the
    /// parent-then-child shape used to slip to the executor as an
    /// unresolvable-path Infra).
    fn field_moved(&self, root: usize, path: &[u16]) -> bool {
        self.moved_fields[root].as_ref().is_some_and(|m| {
            m.iter().any(|p| {
                let n = p.len().min(path.len());
                p[..n] == path[..n]
            })
        })
    }

    /// A DEFINITION establishes a fresh complete value in the slot: all
    /// whole-slot may facts reset — and (R4-F08) the per-field
    /// consumption state dies with the old owner: a re-initialized root
    /// (fresh Call wrapper, re-Aggregated record, loop-carried redefinition)
    /// is consumable again. PLAN-822: a re-defined variant root loses its
    /// guard facts and payload-moved arms with the old value.
    fn define_slot(&mut self, s: usize) {
        self.init.set(s);
        self.moved.clear(s);
        self.dropped.clear(s);
        self.moved_fields[s] = None;
        self.guards.retain(|(root, _)| *root != s);
        self.variant_moved.remove(&s);
    }
}

fn words_of(n: usize) -> usize {
    n.div_ceil(64)
}

/// Run the ownership analysis for one function of an OWNED-profile module.
/// `steps` and `analysis_bytes` charge the shared verification budgets.
/// Errors carry the function's span mapping via `span_of`.
pub fn verify_owned_function(
    m: &MirModule,
    f: &crate::mir::MirFunction,
    steps: &mut u64,
    analysis_bytes: &mut u64,
    span_of: &dyn Fn(u32) -> Option<SpanInfo>,
) -> Result<(), MirVerifyError> {
    let n = f.slots.len();
    let blocks = f.blocks.len();
    let root_bits = n + 1; // + EXTERNAL sentinel
                           // Composite-analysis memory guard (same shape as check_init's):
                           // init/moved/dropped/tokens matrices are blocks × slot-words; the
                           // per-token root matrices are tokens × (slots+1)-words.
    let token_slots: Vec<usize> = f
        .slots
        .iter()
        .enumerate()
        .filter(|(_, s)| {
            matches!(
                m.types.get(s.ty as usize),
                Some(MirType::StrView) | Some(MirType::MutRef)
            )
        })
        .map(|(i, _)| i)
        .collect();
    let sw = words_of(n);
    let rw = words_of(root_bits);
    // The real peak layout (F821-R1-05): per block the state vectors
    // (in_state + out_state + the transient new_in and st) hold the 4
    // plain bitsets (sw words each) and the SPARSE root vector — an
    // Option<Bitsets> header per slot for ALL n slots plus rw words per
    // token-capable slot. The x6 factor covers the up-to-4 live copies
    // and Vec-growth transients (realloc old+new) — measured against the
    // probe's allocator peak (8192-slot fn: ~1.37MB real vs this bound).
    let sparse_header = std::mem::size_of::<Option<Bitsets>>() as u64;
    // R3-F08: the per-root moved-field sets are equally sparse — only
    // RECORD-typed slots can ever be FIELD_MOVE roots (i32/bool/Str
    // slots pay nothing); a t_MutRet wrapper carries one field per mut
    // param, so the 64B per-root constant covers realistic shapes.
    let record_slots: usize = f
        .slots
        .iter()
        .filter(|s| matches!(m.types.get(s.ty as usize), Some(MirType::Record(_))))
        .count();
    let mf_header = std::mem::size_of::<Option<std::collections::BTreeSet<Vec<u16>>>>() as u64;
    let bound = (blocks as u64)
        .saturating_mul(
            (sw as u64 * 8 * 4
                + n as u64 * sparse_header
                + record_slots as u64 * (mf_header + 64)
                + token_slots.len() as u64 * rw as u64 * 8)
                .saturating_mul(6),
        )
        .saturating_add(blocks as u64 * 64);
    let bound = bound as usize;
    *analysis_bytes = (*analysis_bytes).max(bound as u64);
    if bound > crate::contract::MAX_VERIFY_ANALYSIS_BYTES {
        return Err(MirVerifyError::Resource {
            kind: crate::contract::ResourceKind::VerifyMemory,
            message: format!(
                "ownership analysis for `{}` needs ~{} bytes, above the frozen {} budget",
                f.name,
                bound,
                crate::contract::MAX_VERIFY_ANALYSIS_BYTES
            ),
        });
    }

    let err = |code: &str, msg: String, flat: u32| MirVerifyError::Invalid {
        code: code.to_string(),
        message: msg,
        span: span_of(flat),
    };

    // CFG (same construction as check_init).
    let bases: Vec<u32> = {
        let mut v = Vec::with_capacity(blocks + 1);
        let mut acc = 0u32;
        v.push(0);
        for b in &f.blocks {
            acc += b.instrs.len() as u32;
            v.push(acc);
        }
        v
    };
    let succs: Vec<Vec<u32>> = f
        .blocks
        .iter()
        .map(|b| match &b.term {
            Terminator::Jump { target } => vec![*target],
            Terminator::Branch { then, else_, .. } => {
                if then == else_ {
                    vec![*then]
                } else {
                    vec![*then, *else_]
                }
            }
            Terminator::Return { .. } => vec![],
        })
        .collect();
    let mut preds: Vec<Vec<u32>> = vec![Vec::new(); blocks];
    let mut reachable = vec![false; blocks];
    let mut stack = vec![f.entry];
    reachable[f.entry as usize] = true;
    while let Some(b) = stack.pop() {
        for s in &succs[b as usize] {
            if !reachable[*s as usize] {
                reachable[*s as usize] = true;
                stack.push(*s);
            }
        }
    }
    for b in 0..blocks {
        if !reachable[b] {
            continue;
        }
        for s in &succs[b] {
            if !preds[*s as usize].contains(&(b as u32)) {
                preds[*s as usize].push(b as u32);
            }
        }
    }

    // Slot facts.
    let is_owned: Vec<bool> = f.slots.iter().map(|s| type_is_owned(m, s.ty)).collect();
    let is_token: Vec<bool> = f
        .slots
        .iter()
        .map(|s| {
            matches!(
                m.types.get(s.ty as usize),
                Some(MirType::StrView) | Some(MirType::MutRef)
            )
        })
        .collect();
    let is_rootable: Vec<bool> = f.slots.iter().map(|s| type_is_rootable(m, s.ty)).collect();
    let is_mutref: Vec<bool> = f
        .slots
        .iter()
        .map(|s| matches!(m.types.get(s.ty as usize), Some(MirType::MutRef)))
        .collect();
    // R1-F08: BORROWED view params (StrView / RecordView kinds) root
    // EXTERNALLY — a field view rooted at such a param is legal to
    // return (the caller owns the root); any other local root is not.
    let external_borrow: Vec<bool> = f
        .slots
        .iter()
        .map(|s| {
            s.kind == SlotKind::Param
                && matches!(
                    m.types.get(s.ty as usize),
                    Some(MirType::StrView) | Some(MirType::RecordView(_))
                )
        })
        .collect();

    let entry_state = || -> OwnState {
        let mut st = OwnState {
            init: Bitsets::new(n),
            moved: Bitsets::new(n),
            dropped: Bitsets::new(n),
            tokens_live: Bitsets::new(n),
            token_roots: vec![None; n],
            moved_fields: vec![None; n],
            guards: std::collections::BTreeSet::new(),
            variant_moved: std::collections::BTreeMap::new(),
        };
        for (si, s) in f.slots.iter().enumerate() {
            if s.kind == SlotKind::Param {
                st.init.set(si);
                if is_token[si] {
                    // A view param flows in LIVE, rooted EXTERNALLY.
                    st.tokens_live.set(si);
                    let mut roots = Bitsets::new(root_bits);
                    roots.set(n); // sentinel index
                    st.token_roots[si] = Some(roots);
                }
            }
        }
        st
    };

    // in/out states: entry starts exact; others optimistic (all-ones init
    // for the must lattice, empty MAY sets) — mirrors check_init.
    let mut in_state: Vec<OwnState> = (0..blocks)
        .map(|b| {
            if (b as u32) == f.entry {
                entry_state()
            } else {
                let mut st = entry_state().clone_empty_like();
                st.init.words = vec![u64::MAX; sw];
                st
            }
        })
        .collect();
    let mut out_state: Vec<OwnState> = in_state.clone();

    // RPO seed.
    let mut rpo: Vec<u32> = Vec::with_capacity(blocks);
    {
        let mut seen = vec![false; blocks];
        let mut st = vec![(f.entry, 0usize)];
        seen[f.entry as usize] = true;
        while let Some((b, ci)) = st.pop() {
            let ss = &succs[b as usize];
            if ci < ss.len() {
                st.push((b, ci + 1));
                let s = ss[ci];
                if !seen[s as usize] {
                    seen[s as usize] = true;
                    st.push((s, 0));
                }
            } else {
                rpo.push(b);
            }
        }
        rpo.reverse();
    }
    let mut queue: VecDeque<u32> = rpo.iter().copied().collect();
    let mut queued = vec![false; blocks];
    for b in &queue {
        queued[*b as usize] = true;
    }
    let mut visits: u64 = 0;

    while let Some(b) = queue.pop_front() {
        queued[b as usize] = false;
        visits += 1;
        *steps += 1 + sw as u64;
        if *steps > crate::contract::MAX_VERIFY_WORK_STEPS {
            return Err(MirVerifyError::Resource {
                kind: crate::contract::ResourceKind::VerifySteps,
                message: "ownership analysis exceeded the shared work-step budget".to_string(),
            });
        }

        // in = must-∩ init, may-∪ the rest, over reachable preds.
        // NOTE: the ENTRY block pins its must-init lattice to the param
        // seed (a first-iteration uninitialized read is still rejected),
        // but its MAY sets (moved/dropped/live tokens) MERGE back edges —
        // a loop re-entry may carry killed slots into later iterations
        // (Design44 I-05: any-path moved blocks subsequent reads).
        let mut new_in = if b == f.entry {
            entry_state()
        } else {
            let mut st = entry_state().clone_empty_like();
            st.init.words = vec![u64::MAX; sw];
            st
        };
        let mut first_pred = true;
        for p in preds[b as usize].iter().filter(|p| reachable[**p as usize]) {
            let po = &out_state[*p as usize];
            if b != f.entry {
                new_in.init.intersect_with(&po.init);
            }
            new_in.moved.union_with(&po.moved);
            new_in.dropped.union_with(&po.dropped);
            new_in.tokens_live.union_with(&po.tokens_live);
            // PLAN-822: guard facts are MUST facts — seed from the first
            // reachable predecessor, intersect the rest.
            if first_pred {
                new_in.guards = po.guards.clone();
                first_pred = false;
            } else {
                let pg = po.guards.clone();
                new_in.guards.retain(|g| pg.contains(g));
            }
            for (root, arms) in po.variant_moved.iter() {
                new_in
                    .variant_moved
                    .entry(*root)
                    .or_default()
                    .extend(arms.iter().copied());
            }
            for t in 0..n {
                if new_in.is_token_live(t) && po.tokens_live.get(t) {
                    match (&mut new_in.token_roots[t], po.token_roots[t].as_ref()) {
                        (Some(r), Some(pr)) => r.union_with(pr),
                        (None, Some(pr)) => new_in.token_roots[t] = Some(pr.clone()),
                        _ => {}
                    }
                }
                // R3-F08: moved field paths are MAY facts — union across
                // predecessors (a path moved on ANY path stays moved).
                match (&mut new_in.moved_fields[t], po.moved_fields[t].as_ref()) {
                    (Some(a), Some(b)) => a.extend(b.iter().cloned()),
                    (None, Some(b)) => new_in.moved_fields[t] = Some(b.clone()),
                    _ => {}
                }
            }
        }

        // Walk the block.
        let mut st = new_in.clone();
        let mut fail: Option<MirVerifyError> = None;
        'blk: for (ii, instr) in f.blocks[b as usize].instrs.iter().enumerate() {
            let flat = bases[b as usize] + ii as u32;
            macro_rules! check_read {
                ($slot:expr, $what:expr) => {{
                    let s = *$slot as usize;
                    if !st.init.get(s) {
                        fail = Some(err(
                            "mir.uninitialized-read",
                            format!(
                                "read of uninitialized slot `{}` ({})",
                                slot_name(f, s),
                                $what
                            ),
                            flat,
                        ));
                        break 'blk;
                    }
                    if st.moved.get(s) {
                        fail = Some(err(
                            "mir.use-after-move",
                            format!("read of moved slot `{}` ({})", slot_name(f, s), $what),
                            flat,
                        ));
                        break 'blk;
                    }
                    if st.dropped.get(s) {
                        fail = Some(err(
                            "mir.use-after-move",
                            format!("read of dropped slot `{}` ({})", slot_name(f, s), $what),
                            flat,
                        ));
                        break 'blk;
                    }
                    // R3-F08: a WHOLE read of a partially-consumed root is
                    // a use-after-move (partial moves of source values stay
                    // rejected; FIELD_MOVE/FIELD_GET reach fields through
                    // their own path-aware checks below).
                    if st.moved_fields[s].as_ref().is_some_and(|m| !m.is_empty()) {
                        fail = Some(err(
                            "mir.use-after-move",
                            format!(
                                "read of partially-moved slot `{}` ({})",
                                slot_name(f, s),
                                $what
                            ),
                            flat,
                        ));
                        break 'blk;
                    }
                }};
            }
            macro_rules! check_def_conflict {
                ($slot:expr, $what:expr) => {{
                    // A def RE-INITIALIZES a moved/dropped slot (782: 完整
                    // 根重赋 is legal after a move) — only a live borrow on
                    // the destination root conflicts.
                    let s = *$slot as usize;
                    if root_has_live_token(&st, s) {
                        fail = Some(err(
                            "mir.mut-conflict",
                            format!(
                                "definition of root `{}` ({}) conflicts an active borrow",
                                slot_name(f, s),
                                $what
                            ),
                            flat,
                        ));
                        break 'blk;
                    }
                }};
            }
            match instr {
                MirInstr::Const { dst, .. }
                | MirInstr::Bin { dst, .. }
                | MirInstr::Un { dst, .. } => {
                    if is_rootable[*dst as usize] {
                        check_def_conflict!(dst, "re-initialization");
                    }
                    st.define_slot(*dst as usize);
                }
                MirInstr::Copy { dst, src } => {
                    check_read!(src, "copy source");
                    if is_rootable[*dst as usize] {
                        check_def_conflict!(dst, "copy destination");
                    }
                    if is_token[*dst as usize] && is_token[*src as usize] {
                        // View alias: the copy shares the borrow.
                        st.token_roots[*dst as usize] = st.token_roots[*src as usize].clone();
                        st.tokens_live.set(*dst as usize);
                    } else if is_token[*dst as usize] {
                        st.token_roots[*dst as usize] = None;
                    }
                    st.define_slot(*dst as usize);
                }
                MirInstr::Call { dst, args, callee } => {
                    // R1-F08: the kill is SIG-DRIVEN — an owned arg dies
                    // only where the callee's signature declares an OWNED
                    // param (move-in); a borrowed (RecordView/StrView)
                    // param keeps the caller's ownership (782 ABI-D).
                    let callee_sig: Option<&MirType> = match callee {
                        CalleeRef::Func(fi) => m
                            .funcs
                            .get(*fi as usize)
                            .and_then(|cf| m.types.get(cf.sig as usize)),
                        CalleeRef::Import(ii) => m
                            .imports
                            .get(*ii as usize)
                            .and_then(|imp| m.types.get(imp.sig as usize)),
                    };
                    for (ai, a) in args.iter().enumerate() {
                        check_read!(a, "call argument");
                        let param_borrows = callee_sig
                            .and_then(|t| match t {
                                MirType::Fn(sig) => sig.params.get(ai).copied(),
                                _ => None,
                            })
                            .map(|pt| !type_is_owned(m, pt))
                            .unwrap_or(false);
                        if is_owned[*a as usize] && !param_borrows {
                            // Move-in: the callee owns the value; the caller
                            // slot dies HERE (contract-decision §4 CALL).
                            if root_has_live_token(&st, *a as usize) {
                                fail = Some(err(
                                    "mir.mut-conflict",
                                    format!(
                                        "move-in argument `{}` conflicts an active borrow",
                                        slot_name(f, *a as usize)
                                    ),
                                    flat,
                                ));
                                break 'blk;
                            }
                            st.moved.set(*a as usize);
                        }
                    }
                    if is_rootable[*dst as usize] {
                        check_def_conflict!(dst, "call result");
                    }
                    // F821-R1-02a: a view-typed result carries the REAL
                    // provenance — the union of the VIEW arguments' roots
                    // (a callee may return a view of any borrowed param;
                    // those params are exactly the view args) plus the
                    // EXTERNAL sentinel (a static const view is always a
                    // legal return). Roots at move-in params are rejected
                    // callee-side, so they never appear here.
                    if is_token[*dst as usize] {
                        let mut roots = Bitsets::new(root_bits);
                        roots.set(n); // static/external possibility
                        for a in args {
                            if is_token[*a as usize] {
                                if let Some(ar) = st.token_roots[*a as usize].as_ref() {
                                    let mut merged =
                                        std::mem::replace(&mut roots, Bitsets::new(root_bits));
                                    merged.union_with(ar);
                                    roots = merged;
                                }
                            }
                        }
                        st.token_roots[*dst as usize] = Some(roots);
                        st.tokens_live.set(*dst as usize);
                    }
                    st.define_slot(*dst as usize);
                    // PLAN-822 (§5.5): a possibly-writing call invalidates
                    // ALL variant guard facts — conservative (any callee
                    // may re-pack/switch roots reachable through params).
                    st.guards.clear();
                }
                MirInstr::Trace { .. } => {}
                MirInstr::FieldGet { dst, base, path } => {
                    // R3-F08: the BASE check is path-aware — a
                    // partially-consumed root still serves reads of its
                    // REMAINING fields; the already-moved path itself is a
                    // use-after-move.
                    let bs = *base as usize;
                    if !st.init.get(bs) {
                        fail = Some(err(
                            "mir.uninitialized-read",
                            format!(
                                "read of uninitialized slot `{}` (field-get base)",
                                slot_name(f, bs)
                            ),
                            flat,
                        ));
                        break 'blk;
                    }
                    if st.moved.get(bs) || st.dropped.get(bs) {
                        fail = Some(err(
                            "mir.use-after-move",
                            format!(
                                "read of moved/dropped slot `{}` (field-get base)",
                                slot_name(f, bs)
                            ),
                            flat,
                        ));
                        break 'blk;
                    }
                    if st.field_moved(bs, path) {
                        fail = Some(err(
                            "mir.use-after-move",
                            format!(
                                "field-get of already-moved path {:?} on `{}`",
                                path,
                                slot_name(f, bs)
                            ),
                            flat,
                        ));
                        break 'blk;
                    }
                    if is_rootable[*dst as usize] {
                        check_def_conflict!(dst, "field-get destination");
                    }
                    // A Str-field read IS a borrow view rooted at the base
                    // (CD §4 07, 782 "字段读取=视图"): register the token so
                    // root-conservative conflicts see it across merges.
                    if is_token[*dst as usize] {
                        // F821-R1-02b: a live EXCLUSIVE borrow on the root
                        // blocks every access, reads included (782 mut).
                        if root_has_live_exclusive(&st, &is_mutref, *base as usize) {
                            fail = Some(err(
                                "mir.mut-conflict",
                                format!(
                                    "field read of `{}` conflicts an active exclusive borrow on the root",
                                    slot_name(f, *base as usize)
                                ),
                                flat,
                            ));
                            break 'blk;
                        }
                        let mut roots = Bitsets::new(root_bits);
                        roots.set(*base as usize);
                        st.token_roots[*dst as usize] = Some(roots);
                        st.tokens_live.set(*dst as usize);
                    }
                    st.define_slot(*dst as usize);
                    let _ = path;
                }
                MirInstr::FieldSet { root, value, .. } => {
                    check_read!(value, "field-set value");
                    if !st.init.get(*root as usize) {
                        fail = Some(err(
                            "mir.uninitialized-read",
                            format!(
                                "field write through uninitialized root `{}`",
                                slot_name(f, *root as usize)
                            ),
                            flat,
                        ));
                        break 'blk;
                    }
                    if st.moved.get(*root as usize) {
                        // Discriminate (V-01 vs V-02): the frozen
                        // reservation shape is a write whose VALUE was just
                        // moved OUT of the same root (self-replacement);
                        // any other dead root is a plain use-after-move
                        // (782: 字段写永不复活已移出的根). Scan back to the
                        // LAST definition of `value` in this block.
                        let last_def_is_self_move = {
                            let instrs = &f.blocks[b as usize].instrs[..ii];
                            let mut found = false;
                            'scan: for prev in instrs.iter().rev() {
                                let defines_value = match prev {
                                    MirInstr::Const { dst, .. }
                                    | MirInstr::Copy { dst, .. }
                                    | MirInstr::Bin { dst, .. }
                                    | MirInstr::Un { dst, .. }
                                    | MirInstr::Call { dst, .. }
                                    | MirInstr::FieldGet { dst, .. }
                                    | MirInstr::Aggregate { dst, .. }
                                    | MirInstr::Clone { dst, .. }
                                    | MirInstr::Move { dst, .. }
                                    | MirInstr::View { dst, .. }
                                    | MirInstr::MutRef { dst, .. }
                                    | MirInstr::StrLen { dst, .. }
                                    | MirInstr::StrLenBytes { dst, .. }
                                    | MirInstr::StrCharAt { dst, .. }
                                    | MirInstr::StrSlice { dst, .. }
                                    | MirInstr::StrConcat { dst, .. }
                                    | MirInstr::StrToOwned { dst, .. }
                                    | MirInstr::StrCmp { dst, .. }
                                    | MirInstr::CharToCodepoint { dst, .. }
                                    | MirInstr::CharFromCodepoint { dst, .. } => *dst == *value,
                                    _ => false,
                                };
                                if defines_value {
                                    found = matches!(
                                        prev,
                                        MirInstr::Move { dst, src } if dst == value && src == root
                                    );
                                    break 'scan;
                                }
                            }
                            found
                        };
                        fail = Some(err(
                            if last_def_is_self_move {
                                "mir.reservation-conflict"
                            } else {
                                "mir.use-after-move"
                            },
                            format!(
                                "field write through moved root `{}`",
                                slot_name(f, *root as usize)
                            ),
                            flat,
                        ));
                        break 'blk;
                    }
                    // 782 ROOT-CONSERVATIVE rule (observed at the source
                    // gate: "a borrow of any resource field holds the whole
                    // root"): a field replace while ANY view of the root is
                    // live is `mir.mut-conflict` — the 770 plain-text
                    // "stale read" semantics do NOT carry into records.
                    if root_has_live_token(&st, *root as usize) {
                        fail = Some(err(
                            "mir.mut-conflict",
                            format!(
                                "field replace of root `{}` conflicts an active borrow (a borrow of any resource field holds the whole root)",
                                slot_name(f, *root as usize)
                            ),
                            flat,
                        ));
                        break 'blk;
                    }
                    if is_owned[*value as usize] {
                        // F821-R1-02d: transferring an owned value out of
                        // its slot must not strand an active loan on it.
                        if root_has_live_token(&st, *value as usize) {
                            fail = Some(err(
                                "mir.mut-conflict",
                                format!(
                                    "field-set value `{}` conflicts an active borrow on its root",
                                    slot_name(f, *value as usize)
                                ),
                                flat,
                            ));
                            break 'blk;
                        }
                        // The value's ownership moves into the root.
                        st.moved.set(*value as usize);
                    }
                }
                MirInstr::Aggregate { dst, operands, .. } => {
                    for o in operands {
                        check_read!(o, "aggregate operand");
                        if is_owned[*o as usize] {
                            // F821-R1-02d: an operand transfer strands any
                            // active loan rooted at the operand.
                            if root_has_live_token(&st, *o as usize) {
                                fail = Some(err(
                                    "mir.mut-conflict",
                                    format!(
                                        "aggregate operand `{}` conflicts an active borrow on its root",
                                        slot_name(f, *o as usize)
                                    ),
                                    flat,
                                ));
                                break 'blk;
                            }
                            st.moved.set(*o as usize);
                        }
                    }
                    if is_rootable[*dst as usize] {
                        check_def_conflict!(dst, "aggregate destination");
                    }
                    st.define_slot(*dst as usize);
                }
                MirInstr::Clone { dst, src } => {
                    check_read!(src, "clone source");
                    if is_rootable[*dst as usize] {
                        check_def_conflict!(dst, "clone destination");
                    }
                    st.define_slot(*dst as usize);
                }
                MirInstr::Move { dst, src } => {
                    check_read!(src, "move source");
                    if dst == src {
                        fail = Some(err(
                            "mir.reservation-conflict",
                            "self-move overlaps source and destination".to_string(),
                            flat,
                        ));
                        break 'blk;
                    }
                    if !is_owned[*src as usize] {
                        fail = Some(err(
                            "mir.ownership-required",
                            format!(
                                "MOVE of copyable slot `{}` (only Str / resource records transfer)",
                                slot_name(f, *src as usize)
                            ),
                            flat,
                        ));
                        break 'blk;
                    }
                    if root_has_live_token(&st, *src as usize) {
                        fail = Some(err(
                            "mir.mut-conflict",
                            format!(
                                "MOVE of `{}` conflicts an active borrow",
                                slot_name(f, *src as usize)
                            ),
                            flat,
                        ));
                        break 'blk;
                    }
                    if st.moved.get(*dst as usize) || !st.init.get(*dst as usize) {
                        // dst is (re)initialized by the transfer; only the
                        // stale-state read matters, handled by check macros.
                    }
                    if is_rootable[*dst as usize] {
                        // MOVE defines dst — a live token rooted at dst
                        // would alias the new value; root-write gate.
                        if root_has_live_token(&st, *dst as usize) {
                            fail = Some(err(
                                "mir.mut-conflict",
                                format!(
                                    "MOVE destination `{}` conflicts an active borrow",
                                    slot_name(f, *dst as usize)
                                ),
                                flat,
                            ));
                            break 'blk;
                        }
                    }
                    st.moved.set(*src as usize);
                    st.define_slot(*dst as usize);
                }
                MirInstr::View { dst, src } => {
                    check_read!(src, "view source");
                    if !matches!(
                        m.types.get(f.slots[*src as usize].ty as usize),
                        Some(MirType::Str)
                    ) {
                        fail = Some(err(
                            "mir.type-op",
                            "VIEW sources must be an owned Str slot".to_string(),
                            flat,
                        ));
                        break 'blk;
                    }
                    // F821-R1-02b: a live EXCLUSIVE borrow blocks every
                    // access to the root — a new shared view included.
                    if root_has_live_exclusive(&st, &is_mutref, *src as usize) {
                        fail = Some(err(
                            "mir.mut-conflict",
                            format!(
                                "VIEW of `{}` conflicts an active exclusive borrow on the root",
                                slot_name(f, *src as usize)
                            ),
                            flat,
                        ));
                        break 'blk;
                    }
                    let mut roots = Bitsets::new(root_bits);
                    roots.set(*src as usize);
                    st.token_roots[*dst as usize] = Some(roots);
                    st.tokens_live.set(*dst as usize);
                    st.define_slot(*dst as usize);
                }
                MirInstr::MutRef { dst, src } => {
                    check_read!(src, "mut borrow source");
                    // Root-conservative exclusivity (CD §4 0D): no other
                    // live token may hold the root when the exclusive
                    // token is created.
                    if root_has_live_token(&st, *src as usize) {
                        fail = Some(err(
                            "mir.mut-conflict",
                            format!(
                                "MUTREF of `{}` conflicts an active borrow on the root",
                                slot_name(f, *src as usize)
                            ),
                            flat,
                        ));
                        break 'blk;
                    }
                    let mut roots = Bitsets::new(root_bits);
                    roots.set(*src as usize);
                    st.token_roots[*dst as usize] = Some(roots);
                    st.tokens_live.set(*dst as usize);
                    st.define_slot(*dst as usize);
                }
                MirInstr::FieldMove { dst, root, path } => {
                    // R3-F08: PER-FIELD consumption — the root must be
                    // live as a BASE (init, not whole-moved/dropped) but
                    // partial consumption is the point of this
                    // instruction: each DISTINCT path extracts exactly
                    // once and the root keeps serving its remaining
                    // fields. A repeated path is a use-after-move.
                    let rs = *root as usize;
                    if !st.init.get(rs) {
                        fail = Some(err(
                            "mir.uninitialized-read",
                            format!(
                                "read of uninitialized slot `{}` (field-move root)",
                                slot_name(f, rs)
                            ),
                            flat,
                        ));
                        break 'blk;
                    }
                    if st.moved.get(rs) || st.dropped.get(rs) {
                        fail = Some(err(
                            "mir.use-after-move",
                            format!(
                                "read of moved/dropped slot `{}` (field-move root)",
                                slot_name(f, rs)
                            ),
                            flat,
                        ));
                        break 'blk;
                    }
                    if st.field_moved(rs, path) {
                        fail = Some(err(
                            "mir.use-after-move",
                            format!(
                                "field-move of already-moved path {:?} on `{}`",
                                path,
                                slot_name(f, rs)
                            ),
                            flat,
                        ));
                        break 'blk;
                    }
                    // Both sides must be borrow-free.
                    if root_has_live_token(&st, rs) {
                        fail = Some(err(
                            "mir.mut-conflict",
                            format!(
                                "field-move root `{}` conflicts an active borrow",
                                slot_name(f, rs)
                            ),
                            flat,
                        ));
                        break 'blk;
                    }
                    if is_rootable[*dst as usize] {
                        check_def_conflict!(dst, "field-move destination");
                    }
                    st.moved_fields[rs]
                        .get_or_insert_with(Default::default)
                        .insert(path.clone());
                    st.define_slot(*dst as usize);
                }
                MirInstr::Drop { slot } => {
                    // Explicit drop: owned + Init + not already
                    // moved/dropped (mir.drop-uninit); no live borrows on
                    // the root (root-conservative).
                    if !st.init.get(*slot as usize)
                        || st.moved.get(*slot as usize)
                        || st.dropped.get(*slot as usize)
                    {
                        fail = Some(err(
                            "mir.drop-uninit",
                            format!(
                                "DROP of an uninitialised/moved/dropped slot `{}`",
                                slot_name(f, *slot as usize)
                            ),
                            flat,
                        ));
                        break 'blk;
                    }
                    if root_has_live_token(&st, *slot as usize) {
                        fail = Some(err(
                            "mir.mut-conflict",
                            format!(
                                "DROP of `{}` conflicts an active borrow on the root",
                                slot_name(f, *slot as usize)
                            ),
                            flat,
                        ));
                        break 'blk;
                    }
                    st.dropped.set(*slot as usize);
                }
                MirInstr::BorrowEnd { token } => {
                    check_read!(token, "borrow-end token");
                    st.tokens_live.clear(*token as usize);
                }
                MirInstr::StrLen { dst, view } | MirInstr::StrLenBytes { dst, view } => {
                    check_read!(view, "str operand");
                    st.define_slot(*dst as usize);
                }
                MirInstr::StrToOwned { dst, view } => {
                    check_read!(view, "str operand");
                    // F821-R1-02c: the materialized OWNED result is a def
                    // of its destination — the destination conflict gate
                    // applies exactly as for CLONE.
                    if is_rootable[*dst as usize] {
                        check_def_conflict!(dst, "to-owned destination");
                    }
                    st.define_slot(*dst as usize);
                }
                MirInstr::StrCharAt { dst, view, index } => {
                    check_read!(view, "str operand");
                    check_read!(index, "char_at index");
                    st.define_slot(*dst as usize);
                }
                MirInstr::StrSlice {
                    dst,
                    view,
                    start,
                    end,
                } => {
                    check_read!(view, "str operand");
                    check_read!(start, "slice start");
                    check_read!(end, "slice end");
                    // A slice of a view shares the SAME root set.
                    st.token_roots[*dst as usize] = st.token_roots[*view as usize].clone();
                    st.tokens_live.set(*dst as usize);
                    st.define_slot(*dst as usize);
                }
                MirInstr::StrConcat { dst, lhs, rhs } => {
                    check_read!(lhs, "concat lhs");
                    check_read!(rhs, "concat rhs");
                    // F821-R1-02c: same destination conflict gate.
                    if is_rootable[*dst as usize] {
                        check_def_conflict!(dst, "concat destination");
                    }
                    st.define_slot(*dst as usize);
                }
                MirInstr::StrCmp { dst, lhs, rhs, .. } => {
                    check_read!(lhs, "cmp lhs");
                    check_read!(rhs, "cmp rhs");
                    st.define_slot(*dst as usize);
                }
                MirInstr::CharToCodepoint { dst, ch } => {
                    check_read!(ch, "char operand");
                    st.define_slot(*dst as usize);
                }
                MirInstr::CharFromCodepoint { dst, code } => {
                    check_read!(code, "codepoint operand");
                    st.define_slot(*dst as usize);
                }
                // -- PLAN-822 collections surface (lattice transfers) -----
                MirInstr::ListNew { dst, .. } => {
                    if is_rootable[*dst as usize] {
                        check_def_conflict!(dst, "list-new destination");
                    }
                    st.define_slot(*dst as usize);
                }
                MirInstr::ListLen { dst, src } | MirInstr::ListCap { dst, src } => {
                    check_read!(src, "list source");
                    st.define_slot(*dst as usize);
                }
                MirInstr::ListGet { dst, base, index } => {
                    check_read!(base, "list base");
                    check_read!(index, "list index");
                    st.define_slot(*dst as usize);
                }
                MirInstr::ListPush { root, value } => {
                    // Structural change: no live borrow may straddle it.
                    check_read!(root, "list-push root");
                    check_read!(value, "list-push value");
                    if root_has_live_token(&st, *root as usize) {
                        fail = Some(err(
                            "mir.list.place-borrowed",
                            format!(
                                "list-push of `{}` conflicts an active borrow on the root",
                                slot_name(f, *root as usize)
                            ),
                            flat,
                        ));
                        break 'blk;
                    }
                    if is_owned[*value as usize] {
                        // Owned element: the push TRANSFERS the source.
                        if root_has_live_token(&st, *value as usize) {
                            fail = Some(err(
                                "mir.mut-conflict",
                                format!(
                                    "list-push of `{}` conflicts an active borrow on the value",
                                    slot_name(f, *value as usize)
                                ),
                                flat,
                            ));
                            break 'blk;
                        }
                        st.moved.set(*value as usize);
                    }
                }
                MirInstr::ListInsert { root, index, value } => {
                    check_read!(root, "list-insert root");
                    check_read!(index, "list-insert index");
                    check_read!(value, "list-insert value");
                    if root_has_live_token(&st, *root as usize) {
                        fail = Some(err(
                            "mir.list.place-borrowed",
                            format!(
                                "list-insert into `{}` conflicts an active borrow on the root",
                                slot_name(f, *root as usize)
                            ),
                            flat,
                        ));
                        break 'blk;
                    }
                    if is_owned[*value as usize] {
                        if root_has_live_token(&st, *value as usize) {
                            fail = Some(err(
                                "mir.mut-conflict",
                                format!(
                                    "list-insert of `{}` conflicts an active borrow on the value",
                                    slot_name(f, *value as usize)
                                ),
                                flat,
                            ));
                            break 'blk;
                        }
                        st.moved.set(*value as usize);
                    }
                }
                MirInstr::ListSet { root, index, value } => {
                    check_read!(root, "list-set root");
                    check_read!(index, "list-set index");
                    check_read!(value, "list-set value");
                    if root_has_live_token(&st, *root as usize) {
                        fail = Some(err(
                            "mir.list.place-borrowed",
                            format!(
                                "list-set on `{}` conflicts an active borrow on the root",
                                slot_name(f, *root as usize)
                            ),
                            flat,
                        ));
                        break 'blk;
                    }
                    if is_owned[*value as usize] {
                        if root_has_live_token(&st, *value as usize) {
                            fail = Some(err(
                                "mir.mut-conflict",
                                format!(
                                    "list-set of `{}` conflicts an active borrow on the value",
                                    slot_name(f, *value as usize)
                                ),
                                flat,
                            ));
                            break 'blk;
                        }
                        st.moved.set(*value as usize);
                    }
                }
                MirInstr::ListRemove { dst, root, index } => {
                    check_read!(root, "list-remove root");
                    check_read!(index, "list-remove index");
                    if root_has_live_token(&st, *root as usize) {
                        fail = Some(err(
                            "mir.list.place-borrowed",
                            format!(
                                "list-remove from `{}` conflicts an active borrow on the root",
                                slot_name(f, *root as usize)
                            ),
                            flat,
                        ));
                        break 'blk;
                    }
                    if is_rootable[*dst as usize] {
                        check_def_conflict!(dst, "list-remove destination");
                    }
                    st.define_slot(*dst as usize);
                }
                MirInstr::ListFieldGet {
                    dst, base, index, ..
                } => {
                    check_read!(base, "list base");
                    check_read!(index, "list index");
                    st.define_slot(*dst as usize);
                }
                MirInstr::ListFieldSet {
                    root, index, value, ..
                } => {
                    check_read!(root, "list-field-set root");
                    check_read!(index, "list-field-set index");
                    check_read!(value, "list-field-set value");
                    if root_has_live_token(&st, *root as usize) {
                        fail = Some(err(
                            "mir.list.place-borrowed",
                            format!(
                                "list-field-set on `{}` conflicts an active borrow on the root",
                                slot_name(f, *root as usize)
                            ),
                            flat,
                        ));
                        break 'blk;
                    }
                    if is_owned[*value as usize] {
                        if root_has_live_token(&st, *value as usize) {
                            fail = Some(err(
                                "mir.mut-conflict",
                                format!(
                                    "list-field-set of `{}` conflicts an active borrow on the value",
                                    slot_name(f, *value as usize)
                                ),
                                flat,
                            ));
                            break 'blk;
                        }
                        st.moved.set(*value as usize);
                    }
                }
                MirInstr::VariantPack { dst, payload, .. } => {
                    if let Some(p) = payload {
                        check_read!(p, "variant-pack payload");
                        if is_owned[*p as usize] {
                            if root_has_live_token(&st, *p as usize) {
                                fail = Some(err(
                                    "mir.mut-conflict",
                                    format!(
                                        "variant-pack of `{}` conflicts an active borrow",
                                        slot_name(f, *p as usize)
                                    ),
                                    flat,
                                ));
                                break 'blk;
                            }
                            st.moved.set(*p as usize);
                        }
                    }
                    if is_rootable[*dst as usize] {
                        check_def_conflict!(dst, "variant-pack destination");
                    }
                    st.define_slot(*dst as usize);
                }
                MirInstr::VariantDisc { dst, src } => {
                    check_read!(src, "variant source");
                    st.define_slot(*dst as usize);
                }
                MirInstr::VariantGuard { dst, src, disc } => {
                    check_read!(src, "variant source");
                    st.define_slot(*dst as usize);
                    st.guards.insert((*src as usize, *disc));
                }
                MirInstr::VariantProject { dst, root, disc } => {
                    check_read!(root, "variant root");
                    let rs = *root as usize;
                    if !st.guards.contains(&(rs, *disc)) {
                        fail = Some(err(
                            "mir.variant.guard-missing",
                            format!(
                                "project of `{}` disc {} without a live guard fact",
                                slot_name(f, rs),
                                disc
                            ),
                            flat,
                        ));
                        break 'blk;
                    }
                    if st.variant_moved.get(&rs).is_some_and(|m| m.contains(disc)) {
                        fail = Some(err(
                            "mir.variant.payload-moved",
                            format!(
                                "project of already-moved payload (disc {}) on `{}`",
                                disc,
                                slot_name(f, rs)
                            ),
                            flat,
                        ));
                        break 'blk;
                    }
                    // An OWNED payload transfers out (arm -> payload-moved);
                    // Copy payloads stay readable.
                    let payload_owned = variant_payload_is_owned(m, f, rs, *disc);
                    if payload_owned {
                        st.variant_moved.entry(rs).or_default().insert(*disc);
                    }
                    if is_rootable[*dst as usize] {
                        check_def_conflict!(dst, "variant-project destination");
                    }
                    st.define_slot(*dst as usize);
                }
                MirInstr::ListPlace { dst, base, index } => {
                    check_read!(base, "list-place base");
                    check_read!(index, "list-place index");
                    st.define_slot(*dst as usize);
                }
                MirInstr::PlaceGet { dst, src } => {
                    check_read!(src, "place source");
                    st.define_slot(*dst as usize);
                }
                MirInstr::PlaceSet { place, value } => {
                    check_read!(place, "place source");
                    check_read!(value, "place-set value");
                    if is_owned[*value as usize] {
                        if root_has_live_token(&st, *value as usize) {
                            fail = Some(err(
                                "mir.mut-conflict",
                                format!(
                                    "place-set of `{}` conflicts an active borrow on the value",
                                    slot_name(f, *value as usize)
                                ),
                                flat,
                            ));
                            break 'blk;
                        }
                        st.moved.set(*value as usize);
                    }
                }
                MirInstr::VariantSwitch { root, payload, .. } => {
                    if let Some(p) = payload {
                        check_read!(p, "variant-switch payload");
                        if is_owned[*p as usize] {
                            if root_has_live_token(&st, *p as usize) {
                                fail = Some(err(
                                    "mir.mut-conflict",
                                    format!(
                                        "variant-switch of `{}` conflicts an active borrow",
                                        slot_name(f, *p as usize)
                                    ),
                                    flat,
                                ));
                                break 'blk;
                            }
                            st.moved.set(*p as usize);
                        }
                    }
                    if is_rootable[*root as usize] {
                        check_def_conflict!(root, "variant-switch root");
                    }
                    // The re-assignment kills the old value's facts: the
                    // previous live arm's cleanup is runtime work; all guard
                    // facts on the root die here (§5.5).
                    st.define_slot(*root as usize);
                }
            }
        }
        if fail.is_none() {
            // Terminator.
            let flat = bases[b as usize + 1];
            match &f.blocks[b as usize].term {
                Terminator::Jump { .. } => {}
                Terminator::Branch { cond, .. } => {
                    if !st.init.get(*cond as usize) {
                        fail = Some(err(
                            "mir.uninitialized-read",
                            format!(
                                "branch on uninitialized slot `{}`",
                                slot_name(f, *cond as usize)
                            ),
                            flat,
                        ));
                    } else if st.moved.get(*cond as usize) {
                        fail = Some(err(
                            "mir.use-after-move",
                            format!("branch on moved slot `{}`", slot_name(f, *cond as usize)),
                            flat,
                        ));
                    }
                }
                Terminator::Return { value } => {
                    let vs = *value as usize;
                    if !st.init.get(vs) {
                        fail = Some(err(
                            "mir.uninitialized-read",
                            format!("return of uninitialized slot `{}`", slot_name(f, vs)),
                            flat,
                        ));
                    } else if st.moved.get(vs) {
                        fail = Some(err(
                            "mir.use-after-move",
                            format!("return of moved slot `{}`", slot_name(f, vs)),
                            flat,
                        ));
                    } else if st.dropped.get(vs) {
                        // F821-R1-01: DROPPED is a terminal state — a
                        // consumed owner never rides a return as a typed
                        // zero (same family as check_read's dropped gate).
                        fail = Some(err(
                            "mir.use-after-move",
                            format!("return of dropped slot `{}`", slot_name(f, vs)),
                            flat,
                        ));
                    } else if is_token[vs] {
                        // Borrow-escape gate (782): a returned view must be
                        // rooted EXTERNALLY (a caller view param / a
                        // callee-returned provenance) or at nothing (a
                        // static const view); roots inside this frame die
                        // with it.
                        // F821-R1-01: the root provenance gates the escape
                        // REGARDLESS of token liveness — ending the borrow
                        // does not legitimize it (the view still references
                        // this frame's slot).
                        let offending = st.token_roots[vs].as_ref().and_then(|roots| {
                            (0..n).find(|r| roots.get(*r) && !external_borrow[*r])
                        });
                        if let Some(r) = offending {
                            fail = Some(err(
                                "mir.borrow-escape",
                                format!(
                                    "returned view `{}` is rooted at this frame's slot `{}` (dies at return)",
                                    slot_name(f, vs),
                                    slot_name(f, r)
                                ),
                                flat,
                            ));
                        }
                    }
                }
            }
        }
        if let Some(e) = fail {
            return Err(e);
        }

        let new_out = st;
        if !new_in.eq_of(&in_state[b as usize])
            || !new_out.out_eq(&in_state[b as usize], &out_state[b as usize])
        {
            in_state[b as usize] = new_in;
            out_state[b as usize] = new_out;
            for s in &succs[b as usize] {
                if !queued[*s as usize] {
                    queued[*s as usize] = true;
                    queue.push_back(*s);
                }
            }
        } else {
            out_state[b as usize] = new_out;
        }
        if visits > (blocks as u64) * 16 + 1024 {
            return Err(MirVerifyError::Resource {
                kind: crate::contract::ResourceKind::VerifySteps,
                message: "ownership worklist exceeded its defensive visit bound".to_string(),
            });
        }
    }
    let _ = MAX_TABLE_ENTRIES; // budget families live in mir_verify's run()
    Ok(())
}

impl OwnState {
    fn is_token_live(&self, t: usize) -> bool {
        self.tokens_live.get(t)
    }
    /// in-state equality (drives the worklist).
    fn eq_of(&self, o: &OwnState) -> bool {
        self.init.eq(&o.init)
            && self.moved.eq(&o.moved)
            && self.dropped.eq(&o.dropped)
            && self.tokens_live.eq(&o.tokens_live)
            && self.roots_eq(o)
            && self.moved_fields == o.moved_fields
    }
    /// out vs the stored (in, out) pair — a may-set growth with an equal
    /// in-state still needs the stored out refreshed for later merges.
    fn out_eq(&self, pin: &OwnState, prev_out: &OwnState) -> bool {
        let _ = pin;
        self.init.eq(&prev_out.init)
            && self.moved.eq(&prev_out.moved)
            && self.dropped.eq(&prev_out.dropped)
            && self.tokens_live.eq(&prev_out.tokens_live)
            && self.roots_eq(prev_out)
            && self.moved_fields == prev_out.moved_fields
    }
    /// Root sets of LIVE tokens participate in equality: a root growth
    /// must keep the worklist converging (F821-R1-05 hardening).
    fn roots_eq(&self, o: &OwnState) -> bool {
        (0..self.token_roots.len()).all(|t| {
            if !self.tokens_live.get(t) && !o.tokens_live.get(t) {
                return true;
            }
            match (&self.token_roots[t], &o.token_roots[t]) {
                (Some(a), Some(b)) => a.eq(b),
                (None, None) => true,
                _ => false,
            }
        })
    }
}

fn root_has_live_token(st: &OwnState, root: usize) -> bool {
    (0..st.tokens_live.n_bits)
        .any(|t| st.tokens_live.get(t) && st.token_roots[t].as_ref().is_some_and(|r| r.get(root)))
}

/// F821-R1-02b: a live EXCLUSIVE (MutRef) borrow on the root — blocks
/// every access to it, shared reads included (782 mut semantics).
fn root_has_live_exclusive(st: &OwnState, is_mutref: &[bool], root: usize) -> bool {
    (0..st.tokens_live.n_bits).any(|t| {
        st.tokens_live.get(t)
            && is_mutref.get(t).copied().unwrap_or(false)
            && st.token_roots[t].as_ref().is_some_and(|r| r.get(root))
    })
}

fn slot_name(f: &crate::mir::MirFunction, s: usize) -> String {
    f.slots
        .get(s)
        .and_then(|x| x.name.clone())
        .unwrap_or_else(|| format!("slot#{}", s))
}

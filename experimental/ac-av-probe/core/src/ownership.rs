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
use crate::mir::{MirInstr, MirModule, MirType, SlotKind, Terminator};
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
    /// one EXTERNAL sentinel at index `n_slots`.
    token_roots: Vec<Bitsets>,
}

impl OwnState {
    fn clone_empty_like(&self) -> OwnState {
        OwnState {
            init: Bitsets::new(self.init.n_bits),
            moved: Bitsets::new(self.moved.n_bits),
            dropped: Bitsets::new(self.dropped.n_bits),
            tokens_live: Bitsets::new(self.tokens_live.n_bits),
            token_roots: (0..self.token_roots.len())
                .map(|_| Bitsets::new(self.init.n_bits + 1))
                .collect(),
        }
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
        .filter(|(_, s)| matches!(m.types.get(s.ty as usize), Some(MirType::StrView) | Some(MirType::MutRef)))
        .map(|(i, _)| i)
        .collect();
    let sw = words_of(n);
    let rw = words_of(root_bits);
    let bound = blocks
        .saturating_mul(sw * 8 * 4)
        .saturating_add(token_slots.len().saturating_mul(blocks * rw * 8))
        .saturating_add(blocks * 64);
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

    let entry_state = || -> OwnState {
        let mut st = OwnState {
            init: Bitsets::new(n),
            moved: Bitsets::new(n),
            dropped: Bitsets::new(n),
            tokens_live: Bitsets::new(n),
            token_roots: (0..n).map(|_| Bitsets::new(root_bits)).collect(),
        };
        for (si, s) in f.slots.iter().enumerate() {
            if s.kind == SlotKind::Param {
                st.init.set(si);
                if is_token[si] {
                    // A view param flows in LIVE, rooted EXTERNALLY.
                    st.tokens_live.set(si);
                    st.token_roots[si].set(n); // sentinel index
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
        for p in preds[b as usize]
            .iter()
            .filter(|p| reachable[**p as usize])
        {
            let po = &out_state[*p as usize];
            if b != f.entry {
                new_in.init.intersect_with(&po.init);
            }
            new_in.moved.union_with(&po.moved);
            new_in.dropped.union_with(&po.dropped);
            new_in.tokens_live.union_with(&po.tokens_live);
            for t in 0..n {
                if new_in.is_token_live(t) && po.tokens_live.get(t) {
                    new_in.token_roots[t].union_with(&po.token_roots[t]);
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
                            format!("read of uninitialized slot `{}` ({})", slot_name(f, s), $what),
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
                MirInstr::Const { dst, .. } | MirInstr::Bin { dst, .. } | MirInstr::Un { dst, .. } => {
                    if is_rootable[*dst as usize] {
                        check_def_conflict!(dst, "re-initialization");
                    }
                    st.init.set(*dst as usize);
                    st.moved.clear(*dst as usize);
                    st.dropped.clear(*dst as usize);
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
                    }
                    st.init.set(*dst as usize);
                    st.moved.clear(*dst as usize);
                    st.dropped.clear(*dst as usize);
                }
                MirInstr::Call { dst, args, .. } => {
                    for a in args {
                        check_read!(a, "call argument");
                        if is_owned[*a as usize] {
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
                    st.init.set(*dst as usize);
                    st.moved.clear(*dst as usize);
                    st.dropped.clear(*dst as usize);
                }
                MirInstr::Trace { .. } => {}
                MirInstr::FieldGet { dst, base, path } => {
                    check_read!(base, "field-get base");
                    if is_rootable[*dst as usize] {
                        check_def_conflict!(dst, "field-get destination");
                    }
                    st.init.set(*dst as usize);
                    st.moved.clear(*dst as usize);
                    st.dropped.clear(*dst as usize);
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
                        // The value's ownership moves into the root.
                        st.moved.set(*value as usize);
                    }
                }
                MirInstr::Aggregate { dst, operands, .. } => {
                    for o in operands {
                        check_read!(o, "aggregate operand");
                        if is_owned[*o as usize] {
                            st.moved.set(*o as usize);
                        }
                    }
                    if is_rootable[*dst as usize] {
                        check_def_conflict!(dst, "aggregate destination");
                    }
                    st.init.set(*dst as usize);
                    st.moved.clear(*dst as usize);
                    st.dropped.clear(*dst as usize);
                }
                MirInstr::Clone { dst, src } => {
                    check_read!(src, "clone source");
                    if is_rootable[*dst as usize] {
                        check_def_conflict!(dst, "clone destination");
                    }
                    st.init.set(*dst as usize);
                    st.moved.clear(*dst as usize);
                    st.dropped.clear(*dst as usize);
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
                    st.init.set(*dst as usize);
                    st.moved.clear(*dst as usize);
                    st.dropped.clear(*dst as usize);
                }
                MirInstr::View { dst, src } => {
                    check_read!(src, "view source");
                    if !matches!(m.types.get(f.slots[*src as usize].ty as usize), Some(MirType::Str)) {
                        fail = Some(err(
                            "mir.type-op",
                            "VIEW sources must be an owned Str slot".to_string(),
                            flat,
                        ));
                        break 'blk;
                    }
                    st.token_roots[*dst as usize] = Bitsets::new(root_bits);
                    st.token_roots[*dst as usize].set(*src as usize);
                    st.tokens_live.set(*dst as usize);
                    st.init.set(*dst as usize);
                    st.moved.clear(*dst as usize);
                    st.dropped.clear(*dst as usize);
                }
                MirInstr::MutRef { .. } | MirInstr::Drop { .. } => {
                    // T-03 surface: not admitted at this stage.
                    fail = Some(err(
                        "mir.profile-instr",
                        "MUTREF/DROP land with PLAN-821 T-03 (not admitted at T-02)"
                            .to_string(),
                        flat,
                    ));
                    break 'blk;
                }
                MirInstr::BorrowEnd { token } => {
                    check_read!(token, "borrow-end token");
                    st.tokens_live.clear(*token as usize);
                }
                MirInstr::StrLen { dst, view }
                | MirInstr::StrLenBytes { dst, view }
                | MirInstr::StrToOwned { dst, view } => {
                    check_read!(view, "str operand");
                    st.init.set(*dst as usize);
                    st.moved.clear(*dst as usize);
                    st.dropped.clear(*dst as usize);
                }
                MirInstr::StrCharAt { dst, view, index } => {
                    check_read!(view, "str operand");
                    check_read!(index, "char_at index");
                    st.init.set(*dst as usize);
                    st.moved.clear(*dst as usize);
                    st.dropped.clear(*dst as usize);
                }
                MirInstr::StrSlice { dst, view, start, end } => {
                    check_read!(view, "str operand");
                    check_read!(start, "slice start");
                    check_read!(end, "slice end");
                    // A slice of a view shares the SAME root set.
                    st.token_roots[*dst as usize] = st.token_roots[*view as usize].clone();
                    st.tokens_live.set(*dst as usize);
                    st.init.set(*dst as usize);
                    st.moved.clear(*dst as usize);
                    st.dropped.clear(*dst as usize);
                }
                MirInstr::StrConcat { dst, lhs, rhs } => {
                    check_read!(lhs, "concat lhs");
                    check_read!(rhs, "concat rhs");
                    st.init.set(*dst as usize);
                    st.moved.clear(*dst as usize);
                    st.dropped.clear(*dst as usize);
                }
                MirInstr::StrCmp { dst, lhs, rhs, .. } => {
                    check_read!(lhs, "cmp lhs");
                    check_read!(rhs, "cmp rhs");
                    st.init.set(*dst as usize);
                    st.moved.clear(*dst as usize);
                    st.dropped.clear(*dst as usize);
                }
                MirInstr::CharToCodepoint { dst, ch } => {
                    check_read!(ch, "char operand");
                    st.init.set(*dst as usize);
                    st.moved.clear(*dst as usize);
                    st.dropped.clear(*dst as usize);
                }
                MirInstr::CharFromCodepoint { dst, code } => {
                    check_read!(code, "codepoint operand");
                    st.init.set(*dst as usize);
                    st.moved.clear(*dst as usize);
                    st.dropped.clear(*dst as usize);
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
                            format!(
                                "branch on moved slot `{}`",
                                slot_name(f, *cond as usize)
                            ),
                            flat,
                        ));
                    }
                }
                Terminator::Return { value } => {
                    let vs = *value as usize;
                    if !st.init.get(vs) {
                        fail = Some(err(
                            "mir.uninitialized-read",
                            format!(
                                "return of uninitialized slot `{}`",
                                slot_name(f, vs)
                            ),
                            flat,
                        ));
                    } else if st.moved.get(vs) {
                        fail = Some(err(
                            "mir.use-after-move",
                            format!("return of moved slot `{}`", slot_name(f, vs)),
                            flat,
                        ));
                    } else if is_token[vs] {
                        // Borrow-escape gate (782): a returned view must be
                        // rooted EXTERNALLY (a caller view param) or at
                        // nothing (static); roots inside this frame die with
                        // it. Record-view params arrive with T-03.
                        let mut offending: Option<usize> = None;
                        for r in 0..n {
                            if st.tokens_live.get(vs) && st.token_roots[vs].get(r) {
                                offending = Some(r);
                                break;
                            }
                        }
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
    }
    /// out vs the stored (in, out) pair — a may-set growth with an equal
    /// in-state still needs the stored out refreshed for later merges.
    fn out_eq(&self, pin: &OwnState, prev_out: &OwnState) -> bool {
        let _ = pin;
        self.init.eq(&prev_out.init)
            && self.moved.eq(&prev_out.moved)
            && self.dropped.eq(&prev_out.dropped)
            && self.tokens_live.eq(&prev_out.tokens_live)
            && (0..self.token_roots.len())
                .all(|t| self.token_roots[t].eq(&prev_out.token_roots[t]))
    }
}

fn root_has_live_token(st: &OwnState, root: usize) -> bool {
    (0..st.tokens_live.n_bits).any(|t| st.tokens_live.get(t) && st.token_roots[t].get(root))
}

fn slot_name(f: &crate::mir::MirFunction, s: usize) -> String {
    f.slots
        .get(s)
        .and_then(|x| x.name.clone())
        .unwrap_or_else(|| format!("slot#{}", s))
}

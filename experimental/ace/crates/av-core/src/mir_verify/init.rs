//! Initialization dataflow family of the MIR verifier (PLAN-1070, from
//! archived PLAN-1038 D13): the definite-initialization pass over the
//! entry-reachable CFG, its analysis-workspace bound and the bitset helpers
//! used only by this pass.
//!
//! This is a private maintenance boundary inside `mir_verify`; `CheckedMir`
//! minting and every other verification rule stay in the parent module, and no
//! public API is added here. The stage order, budgets, rejection codes and
//! statistics are unchanged by the move.

use super::{MirVerifyError, Verifier};
use crate::contract::ResourceKind;
use crate::mir::*;
use std::collections::VecDeque;

pub(super) fn check_init(
    v: &mut Verifier,
    f: &MirFunction,
    bases: &[u32],
) -> Result<(), MirVerifyError> {
    let n = f.slots.len();
    let words = words_for(n);
    let blocks = f.blocks.len();
    // check_function admitted this workspace before allocating bases.
    // The same reservation spans all matrices and CFG/queue scratch.
    // Def sets per block (fixed after the typing pass).
    let defs: Vec<Vec<u64>> = f
        .blocks
        .iter()
        .map(|b| {
            let mut m = vec![0u64; words];
            for instr in &b.instrs {
                if let Some(d) = instr.def_slot() {
                    set_bit(&mut m, d as usize);
                }
            }
            m
        })
        .collect();
    // Successors.
    let succs: Vec<Vec<u32>> = f.blocks.iter().map(|b| b.term.successors()).collect();
    // Predecessors (reachable ones only, computed on the fly below).
    let mut preds: Vec<Vec<u32>> = vec![Vec::new(); blocks];
    // Reachability from entry.
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

    // In/out states; non-entry blocks start optimistic (all ones).
    let mut in_state: Vec<Vec<u64>> = (0..blocks)
        .map(|b| {
            if (b as u32) == f.entry {
                let mut m = vec![0u64; words];
                for (si, s) in f.slots.iter().enumerate() {
                    if s.kind == SlotKind::Param {
                        set_bit(&mut m, si);
                    }
                }
                m
            } else {
                vec![u64::MAX; words]
            }
        })
        .collect();
    let mut out_state: Vec<Vec<u64>> = in_state
        .iter()
        .zip(defs.iter())
        .map(|(i, d)| union(i, d))
        .collect();
    // Terminator reads.
    let term_reads: Vec<Option<(u32, u32)>> = f
        .blocks
        .iter()
        .enumerate()
        .map(|(bi, b)| {
            let flat = bases[bi + 1];
            match &b.term {
                Terminator::Branch { cond, .. } => Some((*cond, flat)),
                Terminator::Return { value } => Some((*value, flat)),
                Terminator::LangFail { message, .. } | Terminator::Raise { message, .. } => {
                    Some((*message, flat))
                }
                Terminator::Jump { .. } | Terminator::Invoke { .. } => None,
            }
        })
        .collect();

    // Reverse postorder for a fast convergence seed.
    let mut rpo: Vec<u32> = Vec::with_capacity(blocks);
    {
        let mut seen = vec![false; blocks];
        let mut stack: Vec<(u32, usize)> = vec![(f.entry, 0)];
        seen[f.entry as usize] = true;
        while let Some((b, ci)) = stack.pop() {
            let ss = &succs[b as usize];
            if ci < ss.len() {
                stack.push((b, ci + 1));
                let s = ss[ci];
                if !seen[s as usize] {
                    seen[s as usize] = true;
                    stack.push((s, 0));
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
        v.tick(1 + words as u64)?;
        // in = ∩ preds (reachable) out
        let mut new_in = if b == f.entry {
            in_state[b as usize].clone()
        } else {
            match preds[b as usize]
                .iter()
                .filter(|p| reachable[**p as usize])
                .next()
            {
                Some(first) => out_state[*first as usize].clone(),
                None => vec![0u64; words], // unreachable-defensive
            }
        };
        if b != f.entry {
            for p in preds[b as usize]
                .iter()
                .filter(|p| reachable[**p as usize])
                .skip(1)
            {
                intersect(&mut new_in, &out_state[*p as usize]);
            }
        }
        // Reads must be initialized in `in ∪ defs-so-far`: walk instructions
        // in order, checking uses then applying the def.
        let mut live = new_in.clone();
        // PLAN-862: Unit-typed slots are ZERO-WIDTH and carry no value
        // to be initialized — the entry's unit return operand is exactly
        // such a slot (its placement was checked in `check_function`).
        // Treating them as always-defined is the width-zero rule, not a
        // weakened initialization check: no data can be read out of one.
        for (si, s) in f.slots.iter().enumerate() {
            if matches!(v.module.types.get(s.ty as usize), Some(MirType::Unit)) {
                set_bit(&mut live, si);
            }
        }
        let mut fail: Option<(u32, u32)> = None;
        'blk: for (ii, instr) in f.blocks[b as usize].instrs.iter().enumerate() {
            let flat = bases[b as usize] + ii as u32;
            for u in instr.use_slots() {
                if !bit_is_set(&live, u as usize) {
                    fail = Some((u, flat));
                    break 'blk;
                }
            }
            if let Some(d) = instr.def_slot() {
                set_bit(&mut live, d as usize);
            }
        }
        if fail.is_none() {
            if let Some((slot, flat)) = term_reads[b as usize] {
                if !bit_is_set(&live, slot as usize) {
                    fail = Some((slot, flat));
                }
            }
        }
        if let Some((slot, flat)) = fail {
            let sp = v.span_of(f, flat);
            let name = f
                .slots
                .get(slot as usize)
                .and_then(|s| s.name.clone())
                .unwrap_or_else(|| format!("slot#{}", slot));
            return Err(MirVerifyError::Invalid {
                code: "mir.uninitialized-read".to_string(),
                message: format!(
                    "read of uninitialized slot `{}` (kind {:?}) in `{}`",
                    name,
                    f.slots
                        .get(slot as usize)
                        .map(|s| s.kind)
                        .unwrap_or(SlotKind::Temp),
                    f.name
                ),
                span: sp,
            });
        }
        // out = in ∪ defs
        let mut new_out = new_in.clone();
        union_into(&mut new_out, &defs[b as usize]);
        if new_in != in_state[b as usize] {
            in_state[b as usize] = new_in;
            out_state[b as usize] = new_out;
            for s in &succs[b as usize] {
                if !queued[*s as usize] {
                    queued[*s as usize] = true;
                    queue.push_back(*s);
                }
            }
        } else if new_out != out_state[b as usize] {
            out_state[b as usize] = new_out;
        }
        if visits > (blocks as u64) * 16 + 1024 {
            // Defensive: the lattice guarantees termination; this only
            // catches implementation bugs without looping forever.
            return Err(MirVerifyError::Resource {
                kind: ResourceKind::VerifySteps,
                message: "init worklist exceeded its defensive visit bound".to_string(),
            });
        }
    }
    Ok(())
}

pub(super) fn init_analysis_bound(f: &MirFunction) -> usize {
    let blocks = f.blocks.len();
    let words = words_for(f.slots.len());
    let args = f
        .blocks
        .iter()
        .flat_map(|b| &b.instrs)
        .map(|i| match i {
            MirInstr::Call { args, .. } | MirInstr::ForeignCall { args, .. } => args.len(),
            MirInstr::Aggregate { operands, .. } => operands.len(),
            _ => 2,
        })
        .max()
        .unwrap_or(2);
    // Three blocks×words matrices and up to three simultaneous worklist
    // bitsets. The 256B/block allowance covers all three matrix Vec headers,
    // successor/predecessor headers and storage (including predecessor growth
    // overlap), reachable/queued flags, retained DFS and RPO stacks (including
    // their growth overlap), term reads, RPO and deque. Bases and the largest
    // transient use_slots copy are separate. Input/signature tables are borrowed.
    blocks
        .saturating_mul(words)
        .saturating_mul(8)
        .saturating_mul(3)
        .saturating_add(words.saturating_mul(8).saturating_mul(3))
        .saturating_add(blocks.saturating_mul(256))
        .saturating_add(blocks.saturating_add(1).saturating_mul(4))
        .saturating_add(args.saturating_mul(4))
        // Small Vec bootstrap capacities are independent of block count.
        // Diagnostic strings are not initialization-analysis workspace.
        .saturating_add(128)
}

fn words_for(bits: usize) -> usize {
    bits.div_ceil(64)
}

fn set_bit(m: &mut [u64], i: usize) {
    m[i / 64] |= 1u64 << (i % 64);
}

fn bit_is_set(m: &[u64], i: usize) -> bool {
    m.get(i / 64)
        .map(|w| *w >> (i % 64) & 1 == 1)
        .unwrap_or(false)
}

fn union(a: &[u64], b: &[u64]) -> Vec<u64> {
    a.iter().zip(b.iter()).map(|(x, y)| x | y).collect()
}

fn union_into(a: &mut [u64], b: &[u64]) {
    for (x, y) in a.iter_mut().zip(b.iter()) {
        *x |= *y;
    }
}

fn intersect(a: &mut [u64], b: &[u64]) {
    for (x, y) in a.iter_mut().zip(b.iter()) {
        *x &= *y;
    }
}

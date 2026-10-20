//! Program snapshot: the transactional install root (PLAN-828 T-01 first
//! case).
//!
//! A `ProgramSnapshot` holds one revision's resident function-code set and
//! an internally LINKED execution view of it. Construction ([`link`]) is the
//! checked linking environment: every cross-function call site (`fn:<name>`
//! unit import) must resolve to another admitted unit with a structurally
//! equal signature (interned-type equality in the merged table). This is
//! signature-checked binding over ALREADY-VERIFIED units — no whole-closure
//! codec roundtrip and no whole-module re-verification runs here; unchanged
//! functions enter by `Arc` identity from the previous snapshot.
//!
//! Snapshots are immutable once linked: a new revision is a new snapshot
//! object, in-flight executions keep the old one alive, and a failed
//! candidate never becomes one (the caller simply keeps the previous root).

use crate::contract::MIR_SCHEMA;
use crate::function_code::FunctionCode;
use crate::mir::{CalleeRef, FnSig, MirImport, MirInstr, MirModule, MirType};
use std::sync::Arc;

/// One immutable program revision: resident code set + linked dispatch view.
#[derive(Debug)]
pub struct ProgramSnapshot {
    revision: u64,
    entry: String,
    code: Vec<Arc<FunctionCode>>,
    /// Linked execution view: single merged module, all call sites resolved
    /// to `CalleeRef::Func`. Built ONLY by [`ProgramSnapshot::link`] from
    /// admitted units with signature-checked bindings — never re-verified as
    /// a whole, never exposed mutable.
    merged: MirModule,
    /// Per-code-slot diagnostics-only source maps: sorted edit regions
    /// `(start, len, delta)` in the CURRENT source's coordinates; a
    /// position x maps to x + Σ delta of regions with `start < x`.
    /// Executor span-bearing outcomes map through this immutable
    /// snapshot-owned view (F-828-R1-04/R3a).
    span_maps: Vec<Vec<(u64, u64, i64)>>,
}

impl ProgramSnapshot {
    /// Link an ordered code set into one program snapshot. Fails (leaving
    /// the caller's previous root untouched) on duplicate function names,
    /// unresolved `fn:<name>` callees, signature mismatches or a missing
    /// entry.
    pub fn link(
        code: Vec<Arc<FunctionCode>>,
        entry: &str,
        revision: u64,
    ) -> Result<ProgramSnapshot, String> {
        let span_maps = vec![Vec::new(); code.len()];
        Self::link_with_maps(code, entry, revision, span_maps)
    }

    /// Link with per-code-slot source maps (see the field docs).
    pub fn link_with_maps(
        code: Vec<Arc<FunctionCode>>,
        entry: &str,
        revision: u64,
        span_maps: Vec<Vec<(u64, u64, i64)>>,
    ) -> Result<ProgramSnapshot, String> {
        let mut merged = MirModule::new(crate::mir::OwnerScope {
            schema: MIR_SCHEMA.to_string(),
            module: format!("snap-{revision}"),
        });
        merged.capabilities = code
            .iter()
            .flat_map(|c| c.unit().module().capabilities.iter().cloned())
            .collect::<Vec<_>>();
        merged.capabilities.sort();
        merged.capabilities.dedup();

        struct Fused {
            name: String,
            idx: u32,
            sig: u32,
        }
        let mut fused: Vec<Fused> = Vec::new();
        // (func index, block, instr, target name, merged-space sig)
        let mut patches: Vec<(usize, usize, usize, String, u32)> = Vec::new();
        let mut names: Vec<String> = Vec::new();
        for c in &code {
            let u = c.unit().module();
            if u.funcs.len() != 1 {
                return Err(format!(
                    "code `{}` does not hold exactly one function",
                    c.name()
                ));
            }
            let ty_map = merge_types(&mut merged, u);
            let c_map = merge_consts(&mut merged, u);
            let mut blocks = u.funcs[0].blocks.clone();
            for (bi, b) in blocks.iter_mut().enumerate() {
                for (ii, instr) in b.instrs.iter_mut().enumerate() {
                    match instr {
                        MirInstr::Const { value, .. } => *value = c_map[*value as usize],
                        MirInstr::Call { callee, .. } => {
                            if let CalleeRef::Import(impo) = *callee {
                                let imp = &u.imports[impo as usize];
                                let sig_merged = ty_map[imp.sig as usize];
                                if let Some(fname) = imp.symbol.strip_prefix("fn:") {
                                    // Deferred resolution: mutual/recursive
                                    // callees link after all units merged.
                                    patches.push((
                                        merged.funcs.len(),
                                        bi,
                                        ii,
                                        fname.to_string(),
                                        sig_merged,
                                    ));
                                    *callee = CalleeRef::Func(u32::MAX);
                                } else {
                                    let idx = merged.intern_import(MirImport {
                                        symbol: imp.symbol.clone(),
                                        effect: imp.effect,
                                        sig: sig_merged,
                                    });
                                    *callee = CalleeRef::Import(idx);
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            let slots = u.funcs[0]
                .slots
                .iter()
                .map(|s| crate::mir::MirSlot {
                    kind: s.kind,
                    ty: ty_map[s.ty as usize],
                    name: s.name.clone(),
                })
                .collect();
            let sig = ty_map[u.funcs[0].sig as usize];
            if !matches!(merged.types.get(sig as usize), Some(MirType::Fn(_))) {
                return Err(format!("code `{}` sig is not a function type", c.name()));
            }
            fused.push(Fused {
                name: u.funcs[0].name.clone(),
                idx: merged.funcs.len() as u32,
                sig,
            });
            names.push(u.funcs[0].name.clone());
            merged.funcs.push(crate::mir::MirFunction {
                owner: merged.owner.clone(),
                name: u.funcs[0].name.clone(),
                sig,
                slots,
                blocks,
                entry: u.funcs[0].entry,
                source_snapshot: u.funcs[0].source_snapshot,
                spans: u.funcs[0].spans.clone(),
            });
        }
        {
            let mut sorted = names.clone();
            sorted.sort();
            sorted.dedup();
            if sorted.len() != names.len() {
                return Err("duplicate function names in snapshot".to_string());
            }
        }
        // Patch deferred callees; interned-type equality is content equality.
        for (fi, bi, ii, target, sig_merged) in patches {
            let t = fused.iter().find(|f| f.name == target).ok_or_else(|| {
                format!("snapshot import `fn:{target}` does not resolve to any function")
            })?;
            if t.sig != sig_merged {
                return Err(format!(
                    "snapshot signature mismatch for `fn:{target}`: call site type #{sig_merged} != unit type #{}",
                    t.sig
                ));
            }
            merged.funcs[fi].blocks[bi].instrs[ii] = match &merged.funcs[fi].blocks[bi].instrs[ii] {
                MirInstr::Call { dst, args, .. } => MirInstr::Call {
                    callee: CalleeRef::Func(t.idx),
                    dst: *dst,
                    args: args.clone(),
                },
                other => unreachable!("patch site is a call: {other:?}"),
            };
        }
        let entry_idx = merged
            .funcs
            .iter()
            .position(|f| f.name == entry)
            .ok_or_else(|| format!("entry function `{entry}` not among code units"))?;
        merged.entry = entry_idx as u32;
        Ok(ProgramSnapshot {
            revision,
            entry: entry.to_string(),
            code,
            merged,
            span_maps,
        })
    }

    /// The per-slot source maps this snapshot maps through.
    pub fn span_maps(&self) -> &[Vec<(u64, u64, i64)>] {
        &self.span_maps
    }

    /// Map a half-open span [s, e) through the region list with TOKEN
    /// boundary semantics (F-828-R1-04/R5a): a token START moves when the
    /// edit is entirely before it (p + len <= s) or when a pure insertion
    /// sits exactly at it (p == s, len 0, d > 0 — inserting whitespace
    /// directly before a token pushes the token right); a token END moves
    /// when the edit starts strictly before it (p < e — an insertion
    /// exactly at the end does not extend the token).
    pub fn map_span(regions: &[(u64, u64, i64)], s: u64, e: u64) -> (u64, u64) {
        let (mut ds, mut de) = (0i64, 0i64);
        for &(p, len, d) in regions {
            if p + len <= s || (p == s && len == 0 && d > 0) {
                ds += d;
            }
            if p < e {
                de += d;
            }
        }
        ((s as i64 + ds) as u64, (e as i64 + de) as u64)
    }

    /// Inverse map for COMPOSITION (F-828-R1-04/R6a): a current-coordinate
    /// point back into mint coordinates, region-aware: a point inside a
    /// previously INSERTED region collapses to that insertion's anchor;
    /// a point inside a REPLACEMENT's output collapses to the replaced
    /// block's mint start; a point after a deletion resolves past the
    /// deleted block (the preimage can exceed the current offset).
    pub fn unmap_pos(regions: &[(u64, u64, i64)], c: u64) -> u64 {
        let mut acc = 0i64;
        for &(p, len, d) in regions {
            let img_start = (p as i64 + acc) as u64;
            if c < img_start {
                return (c as i64 - acc) as u64;
            }
            let img_end = img_start + ((len as i64 + d).max(0) as u64);
            if c < img_end {
                // Inside an insertion's or a replacement's stretched
                // output: the edit rewrites NEW text — the mint anchor.
                return p;
            }
            acc += d;
        }
        (c as i64 - acc) as u64
    }

    /// Compose one edit — current-coordinate range [c0, c1) replaced by
    /// `new_len` bytes — into a MINT-coordinate region list
    /// (F-828-R1-04/R6a). BOTH ends are inverse-mapped (the mint length
    /// of the replaced text is `unmap(c1) - unmap(c0)`, NOT the current
    /// length: deleting previously-inserted bytes replaces ZERO mint
    /// bytes), and regions fully re-edited by this edit are dropped:
    /// an insertion deleted end-to-end leaves NO trace (undo = identity),
    /// while a partial deletion keeps the insertion and records the trim.
    pub fn compose_region(regions: &mut Vec<(u64, u64, i64)>, c0: u64, c1: u64, new_len: u64) {
        let m0 = Self::unmap_pos(regions, c0);
        let m1 = Self::unmap_pos(regions, c1);
        let mint_len = m1.saturating_sub(m0);
        // Drop regions this edit fully re-writes. Pure insertions are
        // points in mint coordinates — containment is decided in CURRENT
        // coordinates (their image); replaced blocks in MINT coordinates.
        let cur_len = c1 - c0;
        let mut acc = 0i64;
        let mut dropped_img_len: u64 = 0;
        let mut kept: Vec<(u64, u64, i64)> = Vec::with_capacity(regions.len() + 1);
        for &r @ (p, len, d) in regions.iter() {
            let img_start = (p as i64 + acc) as u64;
            let img_len = (len as i64 + d).max(0) as u64;
            let fully_rewritten = if len == 0 && d > 0 {
                c0 <= img_start && img_start + d as u64 <= c1
            } else {
                m0 <= p && p + len <= m1
            };
            if fully_rewritten {
                dropped_img_len += img_len;
            } else {
                kept.push(r);
            }
            acc += d;
        }
        // The recorded delta is the CURRENT-length change (deleting
        // previously-inserted bytes removes current bytes but ZERO mint
        // bytes). When the edit's whole current range came from dropped
        // regions and touched no mint text, it leaves no trace (undo).
        let erased_only_rewritten_output =
            cur_len > 0 && mint_len == 0 && dropped_img_len >= cur_len;
        if !erased_only_rewritten_output {
            let delta = new_len as i64 - cur_len as i64;
            if delta != 0 || mint_len != 0 {
                kept.push((m0, mint_len, delta));
            }
        }
        kept.sort();
        *regions = kept;
    }

    /// This snapshot's revision counter.
    pub fn revision(&self) -> u64 {
        self.revision
    }

    /// The entry function name.
    pub fn entry(&self) -> &str {
        &self.entry
    }

    /// The resident code set; unchanged functions keep their `Arc` identity
    /// across revisions.
    pub fn code(&self) -> &[Arc<FunctionCode>] {
        &self.code
    }

    /// Stable-id lookup: find the resident code of one function by its
    /// cross-revision def id (the identity axis — never a positional
    /// index).
    pub fn find_code(&self, def_id: &str) -> Option<&Arc<FunctionCode>> {
        self.code.iter().find(|c| c.name() == def_id)
    }

    /// The generation (mint revision) of one function's resident code in
    /// THIS snapshot; unchanged functions report an unchanged generation.
    pub fn generation_of(&self, def_id: &str) -> Option<u64> {
        self.find_code(def_id).map(|c| c.generation())
    }

    /// The linked execution view (crate-internal: dispatch goes through
    /// [`crate::av::execute_program`]).
    pub(crate) fn merged(&self) -> &MirModule {
        &self.merged
    }
}

fn merge_types(merged: &mut MirModule, u: &MirModule) -> Vec<u32> {
    let mut map = vec![u32::MAX; u.types.len()];
    // Scalars intern immediately; Fn sigs intern once their children are
    // mapped (fixpoint loop, general over nested sigs).
    loop {
        let mut progress = false;
        for (i, t) in u.types.iter().enumerate() {
            if map[i] != u32::MAX {
                continue;
            }
            match t {
                MirType::I32 | MirType::Bool => {
                    map[i] = merged.intern_type(t.clone());
                    progress = true;
                }
                MirType::Fn(sig) => {
                    let pending = sig
                        .params
                        .iter()
                        .chain(std::iter::once(&sig.result))
                        .any(|p| map[*p as usize] == u32::MAX);
                    if !pending {
                        let mut ns = FnSig {
                            params: Vec::with_capacity(sig.params.len()),
                            result: 0,
                        };
                        for p in &sig.params {
                            ns.params.push(map[*p as usize]);
                        }
                        ns.result = map[sig.result as usize];
                        let idx = merged.intern_type(MirType::Fn(ns));
                        map[i] = idx;
                        progress = true;
                    }
                }
            }
        }
        if !progress {
            break;
        }
    }
    map
}

fn merge_consts(merged: &mut MirModule, u: &MirModule) -> Vec<u32> {
    let mut map = vec![u32::MAX; u.consts.len()];
    for (i, c) in u.consts.iter().enumerate() {
        map[i] = merged.intern_const(c.clone());
    }
    map
}

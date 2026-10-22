//! Program snapshot: the transactional install root (PLAN-828 T-01 first
//! case; T-05 unit dispatch).
//!
//! A `ProgramSnapshot` holds one revision's resident function-code set and
//! the LINK-TIME BINDING TABLE over it. Construction ([`link`]) is the
//! checked linking environment: every cross-function call site (`fn:<name>`
//! unit import) resolves to another admitted unit's code slot with a
//! structurally equal signature (checked through a scratch interned type
//! table — the same content equality the merged space used). This is
//! signature-checked binding over ALREADY-VERIFIED units — no whole-closure
//! codec roundtrip and no whole-module re-verification runs here; unchanged
//! functions enter by `Arc` identity from the previous snapshot.
//!
//! T-05: the snapshot no longer materializes a merged execution payload
//! (blocks/slots/spans copies per revision). Execution reads each
//! function's ORIGINAL immutable unit payload through [`UnitView`], with
//! cross-function calls resolved by the link-time binding table. Snapshots
//! stay immutable once linked: a new revision is a new snapshot object,
//! in-flight executions keep the old one alive, and a failed candidate
//! never becomes one.

use crate::contract::MIR_SCHEMA;
use crate::function_code::FunctionCode;
use crate::mir::{CalleeRef, FnSig, MirModule, MirType};
use std::sync::Arc;

/// One immutable program revision: resident code set + link-time bindings.
#[derive(Debug)]
pub struct ProgramSnapshot {
    revision: u64,
    entry: String,
    /// The entry's code slot (link-computed). `u32::MAX` on a vacant root —
    /// dispatch's entry guard turns it into a controlled `Infra`.
    entry_slot: u32,
    code: Vec<Arc<FunctionCode>>,
    /// LINK-TIME BINDING TABLE (T-05): per code slot, per import index →
    /// the resolved target code slot. Built ONLY by
    /// [`ProgramSnapshot::link_with_maps`] with signature checks; `None` =
    /// an unresolved import (a non-`fn:` import — a dispatch-time
    /// admission violation, exactly as unresolved imports always were).
    bindings: Vec<Vec<Option<u32>>>,
    /// Per-code-slot diagnostics-only source maps: sorted edit regions
    /// `(start, len, delta)` in the CURRENT source's coordinates; a
    /// position x maps to x + Σ delta of regions with `start < x`.
    /// Executor span-bearing outcomes map through this immutable
    /// snapshot-owned view (F-828-R1-04/R3a).
    span_maps: Vec<crate::span_map::SpanSegments>,
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
        let span_maps = (0..code.len())
            .map(|_| crate::span_map::SpanSegments::default())
            .collect();
        Self::link_with_maps(code, entry, revision, span_maps)
    }

    /// Link with per-code-slot source maps (see the field docs). This is
    /// the checked linking environment: it interns every unit's types into
    /// a SCRATCH table solely to check call-site↔callee signature equality
    /// (interned equality is content equality), resolves each unit's
    /// `fn:<name>` imports to code slots, and stores the binding table.
    /// NO blocks, slots, spans or instruction payloads are copied — the
    /// execution view is the units themselves.
    pub fn link_with_maps(
        code: Vec<Arc<FunctionCode>>,
        entry: &str,
        revision: u64,
        span_maps: Vec<crate::span_map::SpanSegments>,
    ) -> Result<ProgramSnapshot, String> {
        // Scratch interned type space: used ONLY for signature equality
        // checks (content equality), never retained.
        let mut scratch = MirModule::new(crate::mir::OwnerScope {
            schema: MIR_SCHEMA.to_string(),
            module: format!("snap-{revision}-types"),
        });
        // (unit sig mapped into the scratch space, unit function name)
        let mut sigs: Vec<(u32, String)> = Vec::with_capacity(code.len());
        let mut ty_maps: Vec<Vec<u32>> = Vec::with_capacity(code.len());
        // B2-C01 owner context: a snapshot is ONE module's program — every
        // unit's admission owner (schema + module id) must be the SAME
        // owner. A unit verified for another owner — however matching its
        // name and signature — is a foreign callee and is refused here,
        // before any dispatch could run it.
        let mut snapshot_owner: Option<(String, String)> = None;
        for c in &code {
            let u = c.unit().module();
            let owner = (u.owner.schema.clone(), u.owner.module.clone());
            match &snapshot_owner {
                None => snapshot_owner = Some(owner),
                Some(expected) => {
                    if &owner != expected {
                        return Err(format!(
                            "owner mismatch for code `{}`: unit owner `{}::{}` != snapshot owner `{}::{}` (foreign unit refused)",
                            c.name(), owner.1, owner.0, expected.1, expected.0
                        ));
                    }
                }
            }
            if u.funcs.len() != 1 {
                return Err(format!(
                    "code `{}` does not hold exactly one function",
                    c.name()
                ));
            }
            let ty_map = merge_types(&mut scratch, u);
            let sig = ty_map[u.funcs[0].sig as usize];
            if !matches!(scratch.types.get(sig as usize), Some(MirType::Fn(_))) {
                return Err(format!("code `{}` sig is not a function type", c.name()));
            }
            sigs.push((sig, u.funcs[0].name.clone()));
            ty_maps.push(ty_map);
        }
        {
            let mut sorted: Vec<&str> = sigs.iter().map(|(_, n)| n.as_str()).collect();
            sorted.sort_unstable();
            sorted.dedup();
            if sorted.len() != sigs.len() {
                return Err("duplicate function names in snapshot".to_string());
            }
        }
        // The binding table: each unit's `fn:<name>` imports resolve to a
        // code slot with an interned-equal (content-equal) signature.
        let mut bindings: Vec<Vec<Option<u32>>> = Vec::with_capacity(code.len());
        for (si, c) in code.iter().enumerate() {
            let u = c.unit().module();
            let mut slot_binds = Vec::with_capacity(u.imports.len());
            for imp in u.imports.iter() {
                let bound = match imp_target(imp.symbol.as_str()) {
                    Some(target) => {
                        let tslot =
                            code.iter()
                                .position(|c| c.name() == target)
                                .ok_or_else(|| {
                                    format!(
                                    "snapshot import `fn:{target}` does not resolve to any function"
                                )
                                })?;
                        let call_site_sig = ty_maps[si][imp.sig as usize];
                        if sigs[tslot].0 != call_site_sig {
                            return Err(format!(
                                "snapshot signature mismatch for `fn:{target}`: call site type #{} != unit type #{}",
                                call_site_sig, sigs[tslot].0
                            ));
                        }
                        Some(tslot as u32)
                    }
                    // A non-`fn:` import stays unbound: dispatch refuses it
                    // as an admission violation (the same contract an
                    // unresolved import always had).
                    None => None,
                };
                slot_binds.push(bound);
            }
            bindings.push(slot_binds);
        }
        // B2-C02 call closure: an import that a unit's instructions
        // ACTUALLY CALL must carry a link-time binding BEFORE the
        // snapshot becomes executable. Import METADATA that is never
        // called may stay unbound (a dispatch-time admission violation,
        // as before) — but a called-but-unbound import used to surface
        // only at the Call itself, AFTER earlier guest instructions had
        // already produced side effects. The real frontend lowers test
        // marks to Trace instructions, so a Call on an unbound import is
        // a foreign payload shape; refuse it at the checked link.
        for (si, c) in code.iter().enumerate() {
            let u = c.unit().module();
            for b in u.funcs[0].blocks.iter() {
                for instr in b.instrs.iter() {
                    if let crate::mir::MirInstr::Call { callee, .. } = instr {
                        if let CalleeRef::Import(ix) = callee {
                            let bound = bindings[si].get(*ix as usize).and_then(|b| *b);
                            if bound.is_none() {
                                return Err(format!(
                                    "code `{}` calls import #{} (`{}`), which has no link-time binding — the call closure is unresolvable",
                                    c.name(),
                                    ix,
                                    u.imports
                                        .get(*ix as usize)
                                        .map(|i| i.symbol.as_str())
                                        .unwrap_or("?")
                                ));
                            }
                        }
                    }
                }
            }
        }
        let entry_slot = code
            .iter()
            .position(|c| c.name() == entry)
            .ok_or_else(|| format!("entry function `{entry}` not among code units"))?
            as u32;
        Ok(ProgramSnapshot {
            revision,
            entry: entry.to_string(),
            entry_slot,
            code,
            bindings,
            span_maps,
        })
    }

    /// A terminal VACANT root for a CLOSED session (PLAN-828 F-05b):
    /// owns no code, bindings, span maps or payloads. Direct execution is
    /// a CONTROLLED `Infra` (dispatch's entry guard), so a closed session's
    /// `current` accessor stays total — it never hands back the released
    /// program and never panics.
    pub fn vacant(entry: &str, revision: u64) -> ProgramSnapshot {
        ProgramSnapshot {
            revision,
            entry: entry.to_string(),
            entry_slot: u32::MAX,
            code: Vec::new(),
            bindings: Vec::new(),
            span_maps: Vec::new(),
        }
    }

    /// The per-slot source maps this snapshot maps through.
    pub fn span_maps(&self) -> &[crate::span_map::SpanSegments] {
        &self.span_maps
    }

    /// PLAN-828 D-02/T-05: the UNIT dispatch view over this snapshot's
    /// ORIGINAL code payloads — blocks/slots/PC/consts/types are read
    /// unit-locally (the immutable `FunctionCode` objects the F-02
    /// identity gate requires), and cross-function calls resolve through
    /// the LINK-TIME binding table (no runtime name lookup, no merged
    /// copy).
    pub fn unit_view(&self) -> UnitView<'_> {
        UnitView { snap: self }
    }

    /// PLAN-828 D-02 binding-rejection control: every code slot holds a
    /// unit payload, every unit's imports carry a link-time binding, and
    /// the entry slot resolves. A snapshot whose unit bindings are
    /// incomplete refuses unit dispatch.
    pub fn unit_binding_complete(&self) -> Result<(), String> {
        if self.code.is_empty() || self.entry_slot == u32::MAX {
            return Err(format!(
                "entry function `{}` not among code units (0 units)",
                self.entry
            ));
        }
        if self.entry_slot as usize >= self.code.len() {
            return Err(format!("entry slot {} out of range", self.entry_slot));
        }
        for (i, c) in self.code.iter().enumerate() {
            let u = c.unit().module();
            let Some(f) = u.funcs.first() else {
                return Err(format!("code slot {i} (`{}`) has no function", c.name()));
            };
            if f.name != c.name() {
                return Err(format!(
                    "code slot {i} name `{}` != unit function `{}`",
                    c.name(),
                    f.name
                ));
            }
            let binds = self
                .bindings
                .get(i)
                .ok_or_else(|| format!("code slot {i} has no binding row"))?;
            if binds.len() != u.imports.len() {
                return Err(format!("code slot {i} binding row is incomplete"));
            }
            for (ix, b) in binds.iter().enumerate() {
                if b.is_none() {
                    return Err(format!(
                        "unit `{}` import #{} `{}` is unbound",
                        c.name(),
                        ix,
                        u.imports[ix].symbol
                    ));
                }
            }
        }
        Ok(())
    }

    /// PLAN-828 D-02/T-05 observation: whether the payload AV actually
    /// CONSUMES for function `fn_index` (the dispatched unit payload, the
    /// DEFAULT path since T-05) is the SAME OBJECT across two snapshots.
    /// Green by construction on the unit path — unchanged functions enter
    /// by `Arc` identity; any copy turns it red.
    pub fn dispatched_payload_same_object(&self, other: &ProgramSnapshot, fn_index: usize) -> bool {
        match (self.code.get(fn_index), other.code.get(fn_index)) {
            (Some(x), Some(y)) => Arc::ptr_eq(x, y),
            _ => false,
        }
    }

    /// Same-slot CONTENT view of the D-02 observation: the dispatched
    /// unit payloads must be content-equal across snapshots even where an
    /// identity comparison is red (a forced same-content copy).
    pub fn dispatched_payload_same_content(
        &self,
        other: &ProgramSnapshot,
        fn_index: usize,
    ) -> bool {
        match (self.unit_function(fn_index), other.unit_function(fn_index)) {
            (Some(x), Some(y)) => format!("{x:?}") == format!("{y:?}"),
            _ => false,
        }
    }

    fn unit_function(&self, fn_index: usize) -> Option<&crate::mir::MirFunction> {
        self.code.get(fn_index)?.unit().module().funcs.first()
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

/// The link-time import-name convention: `fn:<name>` targets the code
/// slot whose unit function is `<name>`.
fn imp_target(symbol: &str) -> Option<&str> {
    symbol.strip_prefix("fn:")
}

/// PLAN-828 D-02/T-05: the unit-local dispatch view (see
/// [`ProgramSnapshot::unit_view`]). Lifetime-bound to the snapshot; the
/// payloads it hands the interpreter are the SAME immutable objects the
/// identity gate observes, and cross-function calls resolve through the
/// snapshot's link-time binding table.
pub struct UnitView<'a> {
    snap: &'a ProgramSnapshot,
}

impl crate::av::ModuleView for UnitView<'_> {
    fn fn_at(&self, idx: u32) -> Option<&crate::mir::MirFunction> {
        self.snap.unit_function(idx as usize)
    }
    fn const_at(&self, func: u32, idx: u32) -> Option<crate::mir::MirConst> {
        let u = self.snap.code.get(func as usize)?.unit().module();
        u.consts.get(idx as usize).copied()
    }
    fn zero_of(&self, func: u32, ty: u32) -> crate::contract::Value {
        let u = self.snap.code.get(func as usize).unwrap().unit().module();
        match u.types.get(ty as usize) {
            Some(MirType::Bool) => crate::contract::Value::Bool(false),
            _ => crate::contract::Value::I32(0),
        }
    }
    fn type_is_bool(&self, func: u32, ty: u32) -> Option<bool> {
        let u = self.snap.code.get(func as usize)?.unit().module();
        match u.types.get(ty as usize) {
            Some(MirType::Bool) => Some(true),
            Some(MirType::I32) => Some(false),
            _ => None,
        }
    }
    fn resolve_callee(&self, func: u32, callee: &crate::mir::CalleeRef) -> Option<u32> {
        match callee {
            // Unit payloads carry their calls as imports; the LINK-TIME
            // binding table resolves each import to its target code slot.
            CalleeRef::Import(ix) => {
                let binds = self.snap.bindings.get(func as usize)?;
                *binds.get(*ix as usize)?
            }
            // Unit-local instructions never carry merged-space Func
            // indices; one showing up is a foreign index space.
            CalleeRef::Func(_) => None,
        }
    }
    fn entry(&self) -> u32 {
        self.snap.entry_slot
    }

    /// B2-C03: a trap span of the function executing in frame slot
    /// `func` maps through THAT slot's source map — payload func numbers
    /// are per-producer coordinates (two real same-owner units may share
    /// one), the executing slot is unique. The published `func` field
    /// keeps the payload's stable identity axis.
    fn map_span(&self, func: u32, span: crate::contract::SpanInfo) -> crate::contract::SpanInfo {
        match self.snap.span_maps.get(func as usize) {
            Some(segs) => {
                let (s, e) = segs.map_span(span.start as u64, span.end as u64);
                crate::contract::SpanInfo {
                    func: span.func,
                    start: s as u32,
                    end: e as u32,
                }
            }
            None => span,
        }
    }
}

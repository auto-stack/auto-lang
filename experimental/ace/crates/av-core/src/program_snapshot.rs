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
use crate::mir::{
    CalleeRef, FnSig, ListTypeRef, MirFieldDef, MirModule, MirType, RecordTypeRef, VariantArm,
    VariantTypeRef,
};
use std::sync::Arc;

mod host_input;
mod saved_cells;
pub use host_input::HostInput;
pub use saved_cells::SavedGlobalCells;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum InitState {
    Pending,
    Running,
    Ready,
}

/// Reset the unpublished cells if initialization does not reach commit.
pub(crate) struct Initialization<'a>(&'a ProgramSnapshot);

impl Initialization<'_> {
    pub(crate) fn commit(self) {
        self.0.initialization.set(InitState::Ready);
    }
}

impl Drop for Initialization<'_> {
    fn drop(&mut self) {
        if self.0.initialization.get() == InitState::Running {
            self.0.reset_global_cells();
            self.0.initialization.set(InitState::Pending);
        }
    }
}

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
    loaded: crate::loaded_program::ResidentProgram,
    /// PLAN-960: module global cells (program-scoped; never persisted) and
    /// the init functions' code slots in dependency order.
    globals: Vec<crate::mir::MirGlobal>,
    cells: std::cell::RefCell<Vec<Option<crate::av::GlobalValue>>>,
    init_slots: Vec<u32>,
    initialization: std::cell::Cell<InitState>,
    host_inputs: Vec<u32>,
    cell_owner: std::rc::Rc<()>,
    /// PLAN-946 (T-02): link-computed — EVERY linked unit is a
    /// collections-family MIR profile (`av.collections/1`/`av.closures/1`).
    /// Drives the unit view's shared-param answer so a List/Variant
    /// argument crossing a linked unit boundary KEEPS the caller's
    /// ownership (Rc alias) exactly as the single-module collections
    /// execution does; a mixed/text-record program keeps move-in.
    collections_family: bool,
}

impl ProgramSnapshot {
    /// PLAN-960: link with module global cells and init functions (by
    /// function name, dependency order). Missing init names fail the link.
    pub fn link_with_globals(
        code: Vec<Arc<FunctionCode>>,
        entry: &str,
        revision: u64,
        span_maps: Vec<crate::span_map::SpanSegments>,
        globals: Vec<crate::mir::MirGlobal>,
        init_names: &[String],
    ) -> Result<ProgramSnapshot, String> {
        let mut snap = Self::link_with_maps_and_previous(code, entry, revision, span_maps, None)?;
        let mut slots = Vec::with_capacity(init_names.len());
        for name in init_names {
            let slot = snap
                .code
                .iter()
                .position(|c| c.name() == name)
                .ok_or_else(|| format!("init function `{name}` is not linked"))?;
            slots.push(slot as u32);
        }
        snap.cells = std::cell::RefCell::new(vec![
            Some(crate::av::GlobalValue::plain(
                crate::contract::Value::I32(0)
            ));
            globals.len()
        ]);
        snap.globals = globals;
        snap.init_slots = slots;
        Ok(snap)
    }
    /// Recheck the portable units' globals against the producer's typed
    /// table before installing a package generation. Unit ABC carries
    /// local slot types and global ordinals; mutable cells are link-owned.
    pub fn link_with_checked_globals(
        code: Vec<Arc<FunctionCode>>,
        entry: &str,
        revision: u64,
        span_maps: Vec<crate::span_map::SpanSegments>,
        producer: &MirModule,
        init_names: &[String],
    ) -> Result<Self, String> {
        Self::link_with_host_inputs(code, entry, revision, span_maps, producer, init_names, &[])
    }

    /// Link a program with designated, immutable cells for scoped host inputs.
    pub fn link_with_host_inputs(
        code: Vec<Arc<FunctionCode>>,
        entry: &str,
        revision: u64,
        span_maps: Vec<crate::span_map::SpanSegments>,
        producer: &MirModule,
        init_names: &[String],
        host_inputs: &[u32],
    ) -> Result<Self, String> {
        use crate::mir::MirInstr;

        for &input in host_inputs {
            if !producer.globals.get(input as usize).is_some_and(|global| !global.mutable) {
                return Err(format!("global.host-input: cell #{input} must be immutable"));
            }
        }
        let mut scratch = MirModule::new(producer.owner.clone());
        let global_types = merge_types(&mut scratch, producer);
        for code_unit in &code {
            let unit = code_unit.unit().module();
            if !unit
                .funcs
                .iter()
                .flat_map(|f| &f.blocks)
                .flat_map(|b| &b.instrs)
                .any(|instr| {
                    matches!(
                        instr,
                        MirInstr::GlobalGet { .. }
                            | MirInstr::GlobalTake { .. }
                            | MirInstr::GlobalSet { .. }
                    )
                })
            {
                continue;
            }
            let unit_types = merge_types(&mut scratch, unit);
            for func in &unit.funcs {
                for instr in func.blocks.iter().flat_map(|block| &block.instrs) {
                    let (ordinal, slot, take) = match instr {
                        MirInstr::GlobalGet { dst, global } => (*global, *dst, false),
                        MirInstr::GlobalTake { dst, global } => (*global, *dst, true),
                        MirInstr::GlobalSet { src, global, .. } => (*global, *src, false),
                        _ => continue,
                    };
                    if host_inputs.contains(&ordinal) && !matches!(instr, MirInstr::GlobalGet { .. }) {
                        return Err(format!("global.host-input: code cannot write cell #{ordinal}"));
                    }
                    let global = producer
                        .globals
                        .get(ordinal as usize)
                        .ok_or_else(|| format!("global.reference: cell #{ordinal} missing"))?;
                    let expected = global_types[global.ty as usize];
                    let actual = unit_types[func.slots[slot as usize].ty as usize];
                    if expected == u32::MAX || expected != actual {
                        return Err(format!(
                            "global.type: cell `{}` has a different type in unit `{}`",
                            global.name, func.name
                        ));
                    }
                    if take && !global.mutable {
                        return Err(format!(
                            "global.immutable: cell `{}` cannot be moved",
                            global.name
                        ));
                    }
                }
            }
        }
        let mut snapshot = Self::link_with_globals(
            code,
            entry,
            revision,
            span_maps,
            producer.globals.clone(),
            init_names,
        )?;
        snapshot.host_inputs = host_inputs.to_vec();
        Ok(snapshot)
    }

    pub fn initialized(&self) -> bool {
        self.initialization.get() == InitState::Ready
    }
    pub(crate) fn begin_initialization(&self) -> Result<Option<Initialization<'_>>, &'static str> {
        match self.initialization.get() {
            InitState::Ready => Ok(None),
            InitState::Running => {
                Err("av.module.init-reentrant: module initialization is already running")
            }
            InitState::Pending => {
                self.initialization.set(InitState::Running);
                self.reset_global_cells();
                Ok(Some(Initialization(self)))
            }
        }
    }
    pub(crate) fn init_slots(&self) -> &[u32] {
        &self.init_slots
    }
    pub(crate) fn reset_global_cells(&self) {
        let mut cells = self.cells.borrow_mut();
        for c in cells.iter_mut() {
            *c = Some(crate::av::GlobalValue::plain(crate::contract::Value::I32(
                0,
            )));
        }
    }
    /// PLAN-960: the global cell table (for observation; never serialized).
    pub fn global_cells(&self) -> Vec<crate::contract::Value> {
        self.cells
            .borrow()
            .iter()
            .map(|cell| {
                cell.as_ref()
                    .map(|value| value.value.clone())
                    .unwrap_or(crate::contract::Value::Nil)
            })
            .collect()
    }

    /// Keep the current global cells if an operation fails. Save their complete
    /// runtime storage, including any environment owned by a global closure.
    pub fn with_global_transaction<T, E>(
        &self,
        operation: impl FnOnce() -> Result<T, E>,
    ) -> Result<T, E> {
        let initialization = self.initialization.get();
        let previous = self.cells.borrow().clone();
        let result = operation();
        if result.is_err() {
            *self.cells.borrow_mut() = previous;
            self.initialization.set(initialization);
        }
        result
    }

    /// Evaluate a candidate update and restore the original cells. Its result
    /// and full state can be committed later by the same snapshot.
    pub fn preview_global_update<T, E>(
        &self,
        operation: impl FnOnce() -> Result<T, E>,
    ) -> Result<PreparedGlobalUpdate<'_, T>, E> {
        let original = self.cells.borrow().clone();
        let initialization = self.initialization.get();
        let result = operation();
        let candidate = self.cells.replace(original);
        let candidate_initialization = self.initialization.replace(initialization);
        result.map(|value| PreparedGlobalUpdate {
            snapshot: self,
            candidate,
            initialization: candidate_initialization,
            value,
        })
    }

    /// PLAN-1106（事件事务回滚）：整表恢复全局 cell（长度必须与 globals
    /// 一致；逐槽写回）。会话层在 handler 派发前备份、失败（trap/预算/取消）
    /// 时恢复——失败不发布半状态。这不是公开的任意写口：恢复值只能来自
    /// 同一快照的 [`ProgramSnapshot::global_cells`] 备份。
    pub fn restore_global_cells(&self, values: &[crate::contract::Value]) -> Result<(), String> {
        if values.len() != self.globals.len() {
            return Err(format!(
                "global cell restore arity mismatch: {} values for {} cells",
                values.len(),
                self.globals.len()
            ));
        }
        let mut cells = self.cells.borrow_mut();
        for (c, v) in cells.iter_mut().zip(values.iter()) {
            *c = Some(crate::av::GlobalValue::plain(v.clone()));
        }
        Ok(())
    }
    pub fn globals(&self) -> &[crate::mir::MirGlobal] {
        &self.globals
    }
    pub fn unit_view_at(&self, entry: u32) -> UnitView<'_> {
        UnitView {
            snap: self,
            use_loaded: true,
            entry_override: Some(entry),
        }
    }
    pub fn reference_unit_view_at(&self, entry: u32) -> UnitView<'_> {
        UnitView {
            snap: self,
            use_loaded: false,
            entry_override: Some(entry),
        }
    }
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
        Self::link_with_maps_and_previous(code, entry, revision, span_maps, None)
    }

    pub fn link_with_maps_and_previous(
        code: Vec<Arc<FunctionCode>>,
        entry: &str,
        revision: u64,
        span_maps: Vec<crate::span_map::SpanSegments>,
        previous: Option<&ProgramSnapshot>,
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
        let mut envs: Vec<Option<Vec<u32>>> = Vec::with_capacity(code.len());
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
            // PLAN-823 r8: the unit's DECLARED env layout, mapped into the
            // merged table — the link's env-equivalence axis (V-13
            // sub-case 3: a layout drift refuses the binding precisely).
            envs.push(u.funcs[0].env.as_ref().map(|e| {
                e.iter()
                    .map(|t| ty_maps.last().expect("just pushed")[*t as usize])
                    .collect::<Vec<u32>>()
            }));
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
                        // PLAN-823 r8 env contract: an import that BINDS a
                        // closure body declares the env layout it captures
                        // against — the target unit's declaration must be
                        // interned-equal (a drift refuses the link BEFORE
                        // the snapshot turns executable; a None import
                        // binding a body unit is the same refusal).
                        if imp.env.is_some() || envs[tslot].is_some() {
                            let want = imp.env.as_ref().map(|e| {
                                e.iter()
                                    .map(|t| ty_maps[si][*t as usize])
                                    .collect::<Vec<u32>>()
                            });
                            if want != envs[tslot] {
                                return Err(format!(
                                    "snapshot env-contract mismatch for `fn:{target}`: call site declares {:?} != unit declares {:?}",
                                    imp.env, envs[tslot]
                                ));
                            }
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
        let loaded =
            crate::loaded_program::ResidentProgram::prepare(&code, previous.map(|p| &p.loaded))
                .map_err(|e| format!("loaded-program preparation: {e}"))?;
        let collections_family = code.iter().all(|c| {
            let m = c.unit().module();
            m.shares_collection_arguments()
        });
        Ok(ProgramSnapshot {
            revision,
            entry: entry.to_string(),
            entry_slot,
            code,
            bindings,
            span_maps,
            loaded,
            globals: Vec::new(),
            cells: std::cell::RefCell::new(Vec::new()),
            init_slots: Vec::new(),
            initialization: std::cell::Cell::new(InitState::Pending),
            host_inputs: Vec::new(),
            cell_owner: std::rc::Rc::new(()),
            collections_family,
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
            loaded: crate::loaded_program::ResidentProgram::vacant(),
            globals: Vec::new(),
            cells: std::cell::RefCell::new(Vec::new()),
            init_slots: Vec::new(),
            initialization: std::cell::Cell::new(InitState::Pending),
            host_inputs: Vec::new(),
            cell_owner: std::rc::Rc::new(()),
            collections_family: false,
        }
    }

    /// The per-slot source maps this snapshot maps through.
    pub fn span_maps(&self) -> &[crate::span_map::SpanSegments] {
        &self.span_maps
    }

    pub fn loaded_program(&self) -> &crate::loaded_program::ResidentProgram {
        &self.loaded
    }

    /// PLAN-828 D-02/T-05: the UNIT dispatch view over this snapshot's
    /// ORIGINAL code payloads — blocks/slots/PC/consts/types are read
    /// unit-locally (the immutable `FunctionCode` objects the F-02
    /// identity gate requires), and cross-function calls resolve through
    /// the LINK-TIME binding table (no runtime name lookup, no merged
    /// copy).
    pub fn unit_view(&self) -> UnitView<'_> {
        UnitView {
            snap: self,
            use_loaded: true,
            entry_override: None,
        }
    }

    pub fn reference_unit_view(&self) -> UnitView<'_> {
        UnitView {
            snap: self,
            use_loaded: false,
            entry_override: None,
        }
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

/// A checked branch of an update that belongs to one mounted snapshot.
pub struct PreparedGlobalUpdate<'a, T> {
    snapshot: &'a ProgramSnapshot,
    candidate: Vec<Option<crate::av::GlobalValue>>,
    initialization: InitState,
    value: T,
}

impl<T> PreparedGlobalUpdate<'_, T> {
    pub fn commit(self) -> T {
        self.snapshot.cells.replace(self.candidate);
        self.snapshot.initialization.set(self.initialization);
        self.value
    }
}

fn merge_types(merged: &mut MirModule, u: &MirModule) -> Vec<u32> {
    let mut map = vec![u32::MAX; u.types.len()];
    // Scalars intern immediately; Fn sigs and record types intern once their
    // children are mapped (fixpoint loop, general over nested sigs/records).
    loop {
        let mut progress = false;
        for (i, t) in u.types.iter().enumerate() {
            if map[i] != u32::MAX {
                continue;
            }
            match t {
                MirType::I32
                | MirType::Bool
                | MirType::I64
                | MirType::U64
                | MirType::F32
                | MirType::F64
                // PLAN-961 numeric surface.
                | MirType::I8
                | MirType::U8
                | MirType::U32
                | MirType::Char
                | MirType::Str
                | MirType::StrView
                | MirType::MutRef
                // PLAN-824: the erased Dyn type interns atomically (it has
                // no type-table children).
                | MirType::Dyn
                // PLAN-862: Unit is atomic (no children).
                | MirType::Unit
                // PLAN-964: Nil interns atomically; Object interns once its
                // elem type is mapped (same fixpoint rule as Place).
                | MirType::Nil => {
                    map[i] = merged.intern_type(t.clone());
                    progress = true;
                }
                MirType::Object { elem } => {
                    if map[*elem as usize] != u32::MAX {
                        map[i] = merged.intern_type(MirType::Object {
                            elem: map[*elem as usize],
                        });
                        progress = true;
                    }
                }
                // PLAN-822 R3: a Place interns once its REFERENT element
                // type is mapped (same fixpoint rule as List).
                MirType::Place(l) => {
                    if map[l.elem as usize] != u32::MAX {
                        let np = ListTypeRef {
                            elem: map[l.elem as usize],
                        };
                        map[i] = merged.intern_type(MirType::Place(np));
                        progress = true;
                    }
                }
                // Record-view types intern once every FIELD type is
                // mapped (PLAN-821 R1-F08) - same nominal identity rule.
                MirType::RecordView(r) => {
                    let pending = r.fields.iter().any(|f| map[f.ty as usize] == u32::MAX);
                    if !pending {
                        let nr = RecordTypeRef {
                            owner_module: r.owner_module.clone(),
                            name: r.name.clone(),
                            fields: r
                                .fields
                                .iter()
                                .map(|f| MirFieldDef {
                                    name: f.name.clone(),
                                    ty: map[f.ty as usize],
                                })
                                .collect(),
                        };
                        let idx = merged.intern_type(MirType::RecordView(nr));
                        map[i] = idx;
                        progress = true;
                    }
                }
                // Record types intern once every FIELD type is mapped
                // (PLAN-820): nominal structure (owner+name+fields) interns
                // by the frozen structural-equality rule.
                MirType::Record(r) => {
                    let pending = r.fields.iter().any(|f| map[f.ty as usize] == u32::MAX);
                    if !pending {
                        let nr = RecordTypeRef {
                            owner_module: r.owner_module.clone(),
                            name: r.name.clone(),
                            fields: r
                                .fields
                                .iter()
                                .map(|f| MirFieldDef {
                                    name: f.name.clone(),
                                    ty: map[f.ty as usize],
                                })
                                .collect(),
                        };
                        let idx = merged.intern_type(MirType::Record(nr));
                        map[i] = idx;
                        progress = true;
                    }
                }
                // PLAN-822: List interns once its element type is mapped;
                // Variant interns once every PAYLOAD type is mapped (same
                // fixpoint rule as records).
                MirType::List(l) => {
                    if map[l.elem as usize] != u32::MAX {
                        let nl = ListTypeRef {
                            elem: map[l.elem as usize],
                        };
                        map[i] = merged.intern_type(MirType::List(nl));
                        progress = true;
                    }
                }
                MirType::Variant(v) => {
                    let pending = v
                        .arms
                        .iter()
                        .any(|a| a.payload.map_or(false, |t| map[t as usize] == u32::MAX));
                    if !pending {
                        let nv = VariantTypeRef {
                            owner_module: v.owner_module.clone(),
                            name: v.name.clone(),
                            arms: v
                                .arms
                                .iter()
                                .map(|a| VariantArm {
                                    name: a.name.clone(),
                                    payload: a.payload.map(|t| map[t as usize]),
                                })
                                .collect(),
                        };
                        let idx = merged.intern_type(MirType::Variant(nv));
                        map[i] = idx;
                        progress = true;
                    }
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
                // PLAN-823: a Closure type interns like Fn (its signature's
                // children map first).
                MirType::Closure(sig) => {
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
                        let idx = merged.intern_type(MirType::Closure(ns));
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
    use_loaded: bool,
    /// PLAN-960: dispatch entry override (module init functions).
    entry_override: Option<u32>,
}

impl crate::av::ModuleView for UnitView<'_> {
    /// PLAN-982（aggregates 文本入口裁定，2026-11-19）：入口实参准入按**入口
    /// 所属单元**的声明回答——车道门在入口带借用 `str` 形参时把
    /// `ac.runtime.text.v1` 写入受检产物（`split_units` 逐单元携带），
    /// `execute_program_text_platform` 的入口单元核对同源。
    fn has_text_entry_capability(&self) -> bool {
        self.snap
            .code
            .get(self.entry() as usize)
            .map(|c| c.unit().module().has_text_entry_capability())
            .unwrap_or(false)
    }

    /// PLAN-973: host imports resolve against the OWNING unit's module
    /// (indices are unit-local on the split-unit link path).
    fn host_import_at(&self, func: u32, idx: u32) -> Option<crate::mir::MirHostImport> {
        self.snap
            .code
            .get(func as usize)?
            .unit()
            .module()
            .host_imports
            .get(idx as usize)
            .cloned()
    }

    /// PLAN-946 (T-02): the linked-program answer for the collections
    /// family — List/Variant params share their cell across unit calls.
    fn shares_collection_arguments(&self) -> bool {
        self.snap.collections_family
    }
    fn fn_at(&self, idx: u32) -> Option<&crate::mir::MirFunction> {
        self.snap.unit_function(idx as usize)
    }
    fn const_at(&self, func: u32, idx: u32) -> Option<crate::mir::MirConst> {
        let u = self.snap.code.get(func as usize)?.unit().module();
        u.consts.get(idx as usize).cloned()
    }
    fn zero_of(&self, func: u32, ty: u32) -> crate::contract::Value {
        let u = self.snap.code.get(func as usize).unwrap().unit().module();
        crate::av::typed_zero(u.types.get(ty as usize))
    }
    fn scalar_shape_of(&self, func: u32, ty: u32) -> Option<crate::mir::ScalarShape> {
        let u = self.snap.code.get(func as usize)?.unit().module();
        u.type_shape(ty)
    }
    // PLAN-821 T-09: the unit view resolves types against the OWNING
    // unit's own table (slot type indices are unit-local — the merged
    // table's indices never matched them).
    fn is_strview_type(&self, func: u32, ty: u32) -> bool {
        let Some(u) = self.snap.code.get(func as usize) else {
            return false;
        };
        matches!(
            u.unit().module().types.get(ty as usize),
            Some(crate::mir::MirType::StrView)
        )
    }
    fn mir_type_of(&self, func: u32, ty: u32) -> Option<&crate::mir::MirType> {
        self.snap
            .code
            .get(func as usize)?
            .unit()
            .module()
            .types
            .get(ty as usize)
    }
    fn type_is_owned(&self, func: u32, ty: u32) -> bool {
        let Some(u) = self.snap.code.get(func as usize) else {
            return false;
        };
        crate::ownership::type_is_owned(u.unit().module(), ty)
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
    fn logical_pc(&self, func: u32, block: u32, ip: usize) -> Option<(bool, u32)> {
        if self.use_loaded && self.snap.loaded.has_function(func) {
            self.snap
                .loaded
                .logical_pc(func, block, ip)
                .map(|pc| (true, pc))
        } else {
            self.fn_at(func)
                .map(|f| (false, f.block_base(block) + ip as u32))
        }
    }
    fn param_slots(&self, func: u32) -> Option<&[u32]> {
        if self.use_loaded {
            self.snap.loaded.param_slots(func)
        } else {
            None
        }
    }
    fn entry(&self) -> u32 {
        self.entry_override.unwrap_or(self.snap.entry_slot)
    }
    fn global_read(&self, idx: u32) -> Option<Option<crate::av::GlobalValue>> {
        self.snap.cells.borrow().get(idx as usize).cloned()
    }
    fn global_can_take(&self, idx: u32) -> bool {
        self.snap
            .globals
            .get(idx as usize)
            .is_some_and(|cell| cell.mutable)
    }
    fn clear_global(&self, idx: u32) {
        let old = self.snap.cells.borrow_mut()[idx as usize].take();
        drop(old);
    }
    fn global_write(&self, idx: u32, value: crate::av::GlobalValue) -> Result<(), String> {
        let mut cells = self.snap.cells.borrow_mut();
        let Some(cell) = cells.get_mut(idx as usize) else {
            return Err(format!("global.reference: cell #{idx} missing"));
        };
        let old = cell.replace(value);
        drop(cells);
        drop(old);
        Ok(())
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

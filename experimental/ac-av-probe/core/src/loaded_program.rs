//! PLAN-825 private derived tables; no executor, wire format or trust constructor.
//!
//! This source is initially compiled by parallel/825. Production registration
//! and dispatch integration require the shared-core handoff. Tables borrow a
//! CheckedMir or retain the admitted resident code; they never clone its MIR.
use crate::contract::AV_PROFILE;
use crate::function_code::SharedFunctionCode;
use crate::mir::{MirFunction, SlotKind};
use crate::mir_verify::CheckedMir;
use std::mem::size_of;
use std::sync::{Arc, Mutex};

#[derive(Debug, PartialEq, Eq)]
pub enum IndexError {
    UnsupportedProfile,
    FunctionRange,
    Budget,
    Allocation,
    IndexOverflow,
    BudgetDomain,
}

/// An accounting slice supplied by the caller, NOT a new memory entitlement.
/// Integration must subtract existing resident/staging charges from its cap.
#[derive(Clone, Debug)]
pub struct IndexBudget(Arc<Mutex<Ledger>>);
#[derive(Debug)]
struct Ledger {
    limit: usize,
    live: usize,
    peak: usize,
    attempts: usize,
    #[cfg(test)]
    fail_attempt: Option<usize>,
}
impl IndexBudget {
    pub fn new(remaining_bytes: usize) -> Self {
        Self(Arc::new(Mutex::new(Ledger {
            limit: remaining_bytes.min(8 * 1024 * 1024),
            live: 0,
            peak: 0,
            attempts: 0,
            #[cfg(test)]
            fail_attempt: None,
        })))
    }
    /// (live charged bytes, high water, table allocation attempts).
    pub fn usage(&self) -> (usize, usize, usize) {
        let l = self.0.lock().unwrap_or_else(|e| e.into_inner());
        (l.live, l.peak, l.attempts)
    }
    fn reserve(&self, bytes: usize) -> Result<Charge, IndexError> {
        let mut l = self.0.lock().unwrap_or_else(|e| e.into_inner());
        let live = l.live.checked_add(bytes).ok_or(IndexError::Budget)?;
        if live > l.limit {
            return Err(IndexError::Budget);
        }
        l.live = live;
        l.peak = l.peak.max(live);
        Ok(Charge {
            budget: self.clone(),
            bytes,
        })
    }
    fn attempt(&self) -> Result<(), IndexError> {
        let mut l = self.0.lock().unwrap_or_else(|e| e.into_inner());
        l.attempts += 1;
        #[cfg(test)]
        if l.fail_attempt == Some(l.attempts) {
            return Err(IndexError::Allocation);
        }
        Ok(())
    }
}
#[derive(Debug)]
struct Charge {
    budget: IndexBudget,
    bytes: usize,
}
impl Drop for Charge {
    fn drop(&mut self) {
        self.budget.0.lock().unwrap_or_else(|e| e.into_inner()).live -= self.bytes;
    }
}
#[derive(Clone, Copy, Debug)]
struct BlockPc {
    base: u32,
    len: u32,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BuildWork {
    pub block_visits: usize,
    pub slot_visits: usize,
}
#[derive(Debug)]
struct Tables {
    blocks: Vec<BlockPc>,
    params: Vec<u32>,
    work: BuildWork,
    _charge: Charge,
}
impl Tables {
    fn requested_bytes(f: &MirFunction) -> Result<usize, IndexError> {
        f.blocks
            .len()
            .checked_mul(size_of::<BlockPc>())
            .and_then(|n| {
                f.slots
                    .len()
                    .checked_mul(size_of::<u32>())
                    .and_then(|s| n.checked_add(s))
            })
            .and_then(|n| n.checked_add(size_of::<Self>()))
            .ok_or(IndexError::IndexOverflow)
    }
    fn build(f: &MirFunction, budget: &IndexBudget) -> Result<Self, IndexError> {
        // Reserve both tables and their headers BEFORE either allocation. The
        // parameter table uses the slot-count upper bound (no preliminary scan).
        let bytes = Self::requested_bytes(f)?;
        let charge = budget.reserve(bytes)?;
        let mut blocks = Vec::new();
        let mut params = Vec::new();
        budget.attempt()?;
        blocks
            .try_reserve_exact(f.blocks.len())
            .map_err(|_| IndexError::Allocation)?;
        budget.attempt()?;
        params
            .try_reserve_exact(f.slots.len())
            .map_err(|_| IndexError::Allocation)?;
        // Do not silently undercharge an allocator that grants extra capacity.
        let actual = blocks
            .capacity()
            .checked_mul(size_of::<BlockPc>())
            .and_then(|n| {
                params
                    .capacity()
                    .checked_mul(size_of::<u32>())
                    .and_then(|s| n.checked_add(s))
            })
            .and_then(|n| n.checked_add(size_of::<Self>()))
            .ok_or(IndexError::IndexOverflow)?;
        if actual > bytes {
            return Err(IndexError::Allocation);
        }
        let mut base = 0u32;
        let mut work = BuildWork::default();
        for block in &f.blocks {
            let len = u32::try_from(block.instrs.len()).map_err(|_| IndexError::IndexOverflow)?;
            blocks.push(BlockPc { base, len });
            base = base.checked_add(len).ok_or(IndexError::IndexOverflow)?;
            work.block_visits += 1;
        }
        for (i, slot) in f.slots.iter().enumerate() {
            if slot.kind == SlotKind::Param {
                params.push(u32::try_from(i).map_err(|_| IndexError::IndexOverflow)?);
            }
            work.slot_visits += 1;
        }
        Ok(Self {
            blocks,
            params,
            work,
            _charge: charge,
        })
    }
    fn logical_pc(&self, block: u32, ip: usize) -> Option<u32> {
        let pc = self.blocks.get(block as usize)?;
        // ip==len denotes the terminator, exactly matching reference AV span
        // projection. A terminal-only block may share a flat source index.
        let ip = u32::try_from(ip).ok()?;
        (ip <= pc.len).then(|| pc.base.checked_add(ip)).flatten()
    }
}

/// Derived tables tied to the exact immutable verifier credential by borrowing.
#[derive(Debug)]
pub struct CheckedFunctionIndex<'a> {
    checked: &'a CheckedMir,
    function: usize,
    tables: Tables,
}
impl<'a> CheckedFunctionIndex<'a> {
    pub fn build(
        checked: &'a CheckedMir,
        function: usize,
        budget: &IndexBudget,
    ) -> Result<Self, IndexError> {
        if checked.profile() != AV_PROFILE {
            return Err(IndexError::UnsupportedProfile);
        }
        let f = checked
            .module()
            .funcs
            .get(function)
            .ok_or(IndexError::FunctionRange)?;
        Ok(Self {
            checked,
            function,
            tables: Tables::build(f, budget)?,
        })
    }
    pub fn function(&self) -> &MirFunction {
        &self.checked.module().funcs[self.function]
    }
    pub fn logical_pc(&self, block: u32, ip: usize) -> Option<u32> {
        self.tables.logical_pc(block, ip)
    }
    pub fn param_slots(&self) -> &[u32] {
        &self.tables.params
    }
    pub fn work(&self) -> BuildWork {
        self.tables.work
    }
}

/// Resident per-function tables retain the ORIGINAL admitted unit. Reusing an
/// unchanged code Arc reuses this entire object, including its tables/generation.
/// This is not an executable credential: unit linkage stays with ProgramSnapshot.
#[derive(Debug)]
pub struct ResidentFunctionIndex {
    code: SharedFunctionCode,
    tables: Tables,
    _charge: Charge,
}
impl ResidentFunctionIndex {
    pub fn prepare(
        code: SharedFunctionCode,
        previous: Option<&Arc<Self>>,
        budget: &IndexBudget,
    ) -> Result<Arc<Self>, IndexError> {
        if let Some(old) = previous {
            if Arc::ptr_eq(&code, &old.code) {
                if !Arc::ptr_eq(&budget.0, &old._charge.budget.0) {
                    return Err(IndexError::BudgetDomain);
                }
                return Ok(Arc::clone(old));
            }
        }
        let module = code.unit().module();
        if module.profile != AV_PROFILE {
            return Err(IndexError::UnsupportedProfile);
        }
        // Tables charges its own headers. Charge the rest of the resident Arc
        // allocation here, including the two standard Arc reference counters.
        let extra = size_of::<Self>() - size_of::<Tables>() + 2 * size_of::<usize>();
        let charge = budget.reserve(extra)?;
        let tables = Tables::build(&module.funcs[0], budget)?;
        Ok(Arc::new(Self {
            code,
            tables,
            _charge: charge,
        }))
    }
    pub fn code(&self) -> &SharedFunctionCode {
        &self.code
    }
    pub fn logical_pc(&self, block: u32, ip: usize) -> Option<u32> {
        self.tables.logical_pc(block, ip)
    }
    pub fn param_slots(&self) -> &[u32] {
        &self.tables.params
    }
    pub fn work(&self) -> BuildWork {
        self.tables.work
    }
}

impl std::fmt::Display for IndexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "loaded-program {:?} rejection", self)
    }
}

/// Derived execution tables for one ABC-admitted complete module. The module
/// remains the semantic credential; these tables are rebuilt, never persisted.
#[derive(Debug)]
pub struct LoadedProgram<'a> {
    _module: &'a crate::abc_verify::LoadedModule,
    budget: IndexBudget,
    functions: Vec<Tables>,
    _functions_charge: Charge,
    builds: u64,
}

impl<'a> LoadedProgram<'a> {
    pub fn build(
        module: &'a crate::abc_verify::LoadedModule,
        execution_budget_bytes: usize,
    ) -> Result<Self, IndexError> {
        if module.module().profile != AV_PROFILE {
            return Err(IndexError::UnsupportedProfile);
        }
        let budget = IndexBudget::new(execution_budget_bytes);
        let functions_bytes = module
            .module()
            .funcs
            .len()
            .checked_mul(size_of::<Tables>())
            .ok_or(IndexError::IndexOverflow)?;
        let functions_charge = budget.reserve(functions_bytes)?;
        let mut functions = Vec::new();
        functions
            .try_reserve_exact(module.module().funcs.len())
            .map_err(|_| IndexError::Allocation)?;
        if functions.capacity() * size_of::<Tables>() > functions_bytes {
            return Err(IndexError::Allocation);
        }
        for function in &module.module().funcs {
            functions.push(Tables::build(function, &budget)?);
        }
        let builds = functions.len() as u64;
        Ok(Self {
            _module: module,
            budget,
            functions,
            _functions_charge: functions_charge,
            builds,
        })
    }

    pub fn has_function(&self, function: u32) -> bool {
        self.functions.get(function as usize).is_some()
    }

    pub fn logical_pc(&self, function: u32, block: u32, ip: usize) -> Option<u32> {
        self.functions.get(function as usize)?.logical_pc(block, ip)
    }

    pub fn param_slots(&self, function: u32) -> Option<&[u32]> {
        Some(&self.functions.get(function as usize)?.params)
    }

    pub fn usage(&self) -> (usize, usize, usize) {
        self.budget.usage()
    }

    pub fn builds(&self) -> u64 {
        self.builds
    }

    pub fn reuses(&self) -> u64 {
        0
    }

    pub fn work(&self) -> BuildWork {
        self.functions
            .iter()
            .fold(BuildWork::default(), |mut total, f| {
                total.block_visits += f.work.block_visits;
                total.slot_visits += f.work.slot_visits;
                total
            })
    }
}

/// Snapshot-resident derived tables. Only the scalar profile obtains an
/// optimized index; all other delivered profiles keep their reference view.
#[derive(Debug)]
pub struct ResidentProgram {
    budget: IndexBudget,
    functions: Vec<Option<Arc<ResidentFunctionIndex>>>,
    _functions_charge: Charge,
    builds: u64,
    reuses: u64,
    fn_builds: Vec<u8>,
    fn_reuses: Vec<u8>,
}

impl ResidentProgram {
    pub fn vacant() -> Self {
        let budget = IndexBudget::new(0);
        let functions_charge = budget.reserve(0).expect("zero-byte vacant charge");
        Self {
            budget,
            functions: Vec::new(),
            _functions_charge: functions_charge,
            builds: 0,
            reuses: 0,
            fn_builds: Vec::new(),
            fn_reuses: Vec::new(),
        }
    }

    pub fn prepare(
        code: &[SharedFunctionCode],
        previous: Option<&ResidentProgram>,
    ) -> Result<Self, IndexError> {
        let budget = previous
            .map(|p| p.budget.clone())
            .unwrap_or_else(|| IndexBudget::new(crate::contract::MAX_AV_SLOT_BYTES));
        let functions_bytes = code
            .len()
            .checked_mul(size_of::<Option<Arc<ResidentFunctionIndex>>>())
            .ok_or(IndexError::IndexOverflow)?;
        let functions_charge = budget.reserve(functions_bytes)?;
        let mut functions = Vec::new();
        functions
            .try_reserve_exact(code.len())
            .map_err(|_| IndexError::Allocation)?;
        if functions.capacity() * size_of::<Option<Arc<ResidentFunctionIndex>>>() > functions_bytes
        {
            return Err(IndexError::Allocation);
        }
        let mut builds = 0;
        let mut reuses = 0;
        let mut fn_builds = Vec::with_capacity(code.len());
        let mut fn_reuses = Vec::with_capacity(code.len());
        for function in code {
            if function.unit().module().profile != AV_PROFILE {
                functions.push(None);
                fn_builds.push(0);
                fn_reuses.push(0);
                continue;
            }
            let old = previous.and_then(|p| {
                p.functions
                    .iter()
                    .flatten()
                    .find(|index| Arc::ptr_eq(index.code(), function))
            });
            let index = ResidentFunctionIndex::prepare(function.clone(), old, &budget)?;
            if old.is_some_and(|old| Arc::ptr_eq(old, &index)) {
                reuses += 1;
                fn_builds.push(0);
                fn_reuses.push(1);
            } else {
                builds += 1;
                fn_builds.push(1);
                fn_reuses.push(0);
            }
            functions.push(Some(index));
        }
        Ok(Self {
            budget,
            functions,
            _functions_charge: functions_charge,
            builds,
            reuses,
            fn_builds,
            fn_reuses,
        })
    }

    pub fn has_function(&self, function: u32) -> bool {
        self.functions
            .get(function as usize)
            .is_some_and(Option::is_some)
    }

    pub fn logical_pc(&self, function: u32, block: u32, ip: usize) -> Option<u32> {
        self.functions
            .get(function as usize)?
            .as_ref()?
            .logical_pc(block, ip)
    }

    pub fn param_slots(&self, function: u32) -> Option<&[u32]> {
        Some(
            self.functions
                .get(function as usize)?
                .as_ref()?
                .param_slots(),
        )
    }

    pub fn function(&self, slot: usize) -> Option<&Arc<ResidentFunctionIndex>> {
        self.functions.get(slot)?.as_ref()
    }

    pub fn usage(&self) -> (usize, usize, usize) {
        self.budget.usage()
    }

    pub fn builds(&self) -> u64 {
        self.builds
    }

    pub fn reuses(&self) -> u64 {
        self.reuses
    }

    /// Per-slot attribution of the link that produced this program: 1 where
    /// that link built a fresh index for the slot, 0 where it reused the
    /// previous resident (or the slot carries no scalar index). Fn-scoped
    /// consumers (the 827 whole-module structure gate only lets
    /// `axis:function` counters grow on an edit) judge per function from
    /// these instead of the module-level aggregates.
    pub fn fn_builds(&self) -> &[u8] {
        &self.fn_builds
    }

    /// Per-slot reuse attribution, counterpart of [`Self::fn_builds`].
    pub fn fn_reuses(&self) -> &[u8] {
        &self.fn_reuses
    }

    pub fn work(&self) -> BuildWork {
        self.functions
            .iter()
            .flatten()
            .fold(BuildWork::default(), |mut total, f| {
                let work = f.work();
                total.block_visits += work.block_visits;
                total.slot_visits += work.slot_visits;
                total
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::abc::encode_abc;
    use crate::abc_verify::verify_abc_bytes;
    use crate::function_code::FunctionCode;
    use crate::mir::*;
    use crate::mir_verify::verify_mir;

    fn fixture(blocks: usize, width: usize) -> MirModule {
        let owner = OwnerScope {
            schema: crate::contract::MIR_SCHEMA.into(),
            module: "pc".into(),
        };
        let mut m = MirModule::unit(owner.clone());
        let i32_t = m.intern_type(MirType::I32);
        let bool_t = m.intern_type(MirType::Bool);
        let sig = m.intern_type(MirType::Fn(FnSig {
            params: vec![i32_t, bool_t],
            result: i32_t,
        }));
        let mut body = Vec::new();
        for b in 0..blocks {
            body.push(MirBlock {
                instrs: (0..width)
                    .map(|_| MirInstr::Copy { dst: 1, src: 0 })
                    .collect(),
                term: if b + 1 == blocks {
                    Terminator::Return { value: 0 }
                } else {
                    Terminator::Jump {
                        target: b as u32 + 1,
                    }
                },
            });
        }
        m.funcs.push(MirFunction {
            owner,
            name: "main".into(),
            sig,
            slots: vec![
                MirSlot {
                    kind: SlotKind::Param,
                    ty: i32_t,
                    name: None,
                },
                MirSlot {
                    kind: SlotKind::Temp,
                    ty: i32_t,
                    name: None,
                },
                MirSlot {
                    kind: SlotKind::Param,
                    ty: bool_t,
                    name: None,
                },
            ],
            blocks: body,
            entry: 0,
            source_snapshot: 1,
            spans: vec![],
            env: None,
        });
        m
    }
    fn unit(generation: u64, width: usize) -> SharedFunctionCode {
        let mut m = fixture(3, width);
        m.is_unit = true;
        let wire = encode_abc(&m).unwrap().to_flat();
        Arc::new(
            FunctionCode::admit_unit(verify_abc_bytes(&wire, &m.owner).unwrap(), generation)
                .unwrap(),
        )
    }
    #[test]
    fn av_runtime_825_pc_lengths_and_param_order() {
        // Unequal lengths and a terminal-only block defeat block*width and
        // off-by-one implementations. Independent hand-computed expected PCs.
        let mut m = fixture(4, 1);
        m.funcs[0].blocks[1].instrs = vec![];
        m.funcs[0].blocks[2].instrs = vec![MirInstr::Copy { dst: 1, src: 0 }; 3];
        let checked = verify_mir(&m).unwrap();
        let budget = IndexBudget::new(4096);
        let index = CheckedFunctionIndex::build(&checked, 0, &budget).unwrap();
        assert_eq!(index.param_slots(), &[0, 2]);
        for (block, ip, expected) in [
            (0, 0, 0),
            (0, 1, 1),
            (1, 0, 1),
            (2, 0, 1),
            (2, 2, 3),
            (2, 3, 4),
            (3, 0, 4),
            (3, 1, 5),
        ] {
            assert_eq!(index.logical_pc(block, ip), Some(expected));
        }
        assert_eq!(index.logical_pc(4, 0), None);
        assert_eq!(index.logical_pc(1, 1), None);
        assert_eq!(index.logical_pc(0, usize::MAX), None);
        assert!(std::ptr::eq(index.function(), &checked.module().funcs[0]));
        assert_eq!(
            index.work(),
            BuildWork {
                block_visits: 4,
                slot_visits: 3
            }
        );
        assert!(budget.usage().0 > 0);
        drop(index);
        assert_eq!(budget.usage().0, 0);
    }
    #[test]
    fn av_runtime_825_budget_both_sides_shared_and_rollback() {
        let checked = verify_mir(&fixture(4, 2)).unwrap();
        let bytes = Tables::requested_bytes(&checked.module().funcs[0]).unwrap();
        let small = IndexBudget::new(bytes - 1);
        assert_eq!(
            CheckedFunctionIndex::build(&checked, 0, &small).unwrap_err(),
            IndexError::Budget
        );
        assert_eq!(small.usage(), (0, 0, 0));
        let exact = IndexBudget::new(bytes);
        let first = CheckedFunctionIndex::build(&checked, 0, &exact).unwrap();
        assert_eq!(exact.usage(), (bytes, bytes, 2));
        assert_eq!(
            CheckedFunctionIndex::build(&checked, 0, &exact).unwrap_err(),
            IndexError::Budget
        );
        assert_eq!(exact.usage(), (bytes, bytes, 2));
        drop(first);
        assert_eq!(exact.usage().0, 0);
        let next = CheckedFunctionIndex::build(&checked, 0, &exact).unwrap();
        assert_eq!(next.logical_pc(3, 1), Some(7));
    }
    #[test]
    fn av_runtime_825_resident_identity_generation_and_old_pin() {
        let budget = IndexBudget::new(16384);
        let code = unit(7, 1);
        let weak_code = Arc::downgrade(&code);
        let first = ResidentFunctionIndex::prepare(code.clone(), None, &budget).unwrap();
        let charged = budget.usage();
        let same = ResidentFunctionIndex::prepare(code.clone(), Some(&first), &budget).unwrap();
        assert!(Arc::ptr_eq(&same, &first));
        assert_eq!(budget.usage(), charged); // zero build/allocation on unchanged function
        assert!(Arc::ptr_eq(first.code(), &code));
        assert_eq!(same.code().generation(), 7);
        let recopied = ResidentFunctionIndex::prepare(unit(7, 1), Some(&first), &budget).unwrap();
        assert!(!Arc::ptr_eq(&recopied, &first)); // same name/content/generation is not payload identity
        drop(recopied);
        let next_code = unit(8, 3);
        let next = ResidentFunctionIndex::prepare(next_code, Some(&first), &budget).unwrap();
        assert!(!Arc::ptr_eq(&next, &first));
        assert_eq!(first.logical_pc(2, 0), Some(2));
        assert_eq!(next.logical_pc(2, 0), Some(6));
        assert_eq!(next.code().generation(), 8);
        drop(code);
        drop(first);
        assert!(weak_code.upgrade().is_some()); // old snapshot pin retains original
        drop(same);
        assert!(weak_code.upgrade().is_none());
        drop(next);
        assert_eq!(budget.usage().0, 0);
    }
    #[test]
    fn av_runtime_825_partial_resident_failure_returns_to_baseline() {
        let budget = IndexBudget::new(size_of::<ResidentFunctionIndex>());
        let code = unit(1, 3);
        let weak = Arc::downgrade(&code);
        assert_eq!(
            ResidentFunctionIndex::prepare(code, None, &budget).unwrap_err(),
            IndexError::Budget
        );
        assert_eq!(budget.usage().0, 0);
        assert_eq!(budget.usage().2, 0); // reserve failed before table allocation
        assert!(weak.upgrade().is_none());
    }
    #[test]
    fn av_runtime_825_scaling_and_long_block_control() {
        for blocks in [1, 8, 64, 256] {
            let checked = verify_mir(&fixture(blocks, 1)).unwrap();
            let budget = IndexBudget::new(65536);
            let index = CheckedFunctionIndex::build(&checked, 0, &budget).unwrap();
            assert_eq!(index.work().block_visits, blocks);
            for b in 0..blocks {
                assert_eq!(index.logical_pc(b as u32, 0), Some(b as u32));
            }
            // Queries cannot mutate the build-work counts or allocate tables.
            assert_eq!(budget.usage().2, 2);
            assert_eq!(index.work().block_visits, blocks);
        }
        let checked = verify_mir(&fixture(1, 256)).unwrap();
        let index = CheckedFunctionIndex::build(&checked, 0, &IndexBudget::new(65536)).unwrap();
        assert_eq!(index.logical_pc(0, 255), Some(255));
        assert_eq!(index.work().block_visits, 1);
    }
    #[test]
    fn av_runtime_825_bad_mir_rejected_and_data_uses_reference() {
        let mut bad = fixture(2, 1);
        bad.funcs[0].blocks[0].term = Terminator::Jump { target: 99 };
        assert!(verify_mir(&bad).is_err());
        let mut data = fixture(2, 1);
        data.profile = crate::contract::AV_DATA_PROFILE.into();
        data.owner.schema = crate::contract::MIR_DATA_SCHEMA.into();
        data.funcs[0].owner = data.owner.clone();
        let checked = verify_mir(&data).unwrap();
        let budget = IndexBudget::new(4096);
        assert_eq!(
            CheckedFunctionIndex::build(&checked, 0, &budget).unwrap_err(),
            IndexError::UnsupportedProfile
        );
        assert_eq!(budget.usage(), (0, 0, 0));
        let checked = verify_mir(&fixture(1, 1)).unwrap();
        assert_eq!(
            CheckedFunctionIndex::build(&checked, 1, &budget).unwrap_err(),
            IndexError::FunctionRange
        );
    }
    #[test]
    fn av_runtime_825_oracle_detects_wrong_last_block_and_rebuild() {
        let code = unit(1, 2);
        let budget = IndexBudget::new(65536);
        let first = ResidentFunctionIndex::prepare(code.clone(), None, &budget).unwrap();
        let mut broken = Tables::build(&code.unit().module().funcs[0], &budget).unwrap();
        broken.blocks[2].base = 0; // controlled faulty derived table, not guest mutation
        assert_ne!(broken.logical_pc(2, 0), Some(4));
        let rebuilt = ResidentFunctionIndex::prepare(code.clone(), None, &budget).unwrap();
        assert!(!Arc::ptr_eq(&first, &rebuilt)); // identity oracle detects forced rebuild
        let accepted = ResidentFunctionIndex::prepare(code, Some(&first), &budget).unwrap();
        assert!(Arc::ptr_eq(&first, &accepted));
        assert_eq!(first.logical_pc(2, 0), Some(4));
    }

    #[test]
    fn av_runtime_825_program_containers_share_execution_budget() {
        let mut m = fixture(2, 1);
        m.is_unit = false;
        m.funcs[0].slots.retain(|slot| slot.kind == SlotKind::Temp);
        for block in &mut m.funcs[0].blocks {
            for instr in &mut block.instrs {
                if let MirInstr::Copy { dst, .. } = instr {
                    *dst = 0;
                }
            }
        }
        let zero = m.intern_const(MirConst::I32(0));
        m.funcs[0].blocks[0].instrs.insert(
            0,
            MirInstr::Const {
                dst: 0,
                value: zero,
            },
        );
        let result = m.funcs[0].slots[0].ty;
        m.funcs[0].sig = m.intern_type(MirType::Fn(FnSig {
            params: vec![],
            result,
        }));
        let loaded = verify_abc_bytes(&encode_abc(&m).unwrap().to_flat(), &m.owner).unwrap();
        let program = LoadedProgram::build(&loaded, crate::contract::MAX_AV_SLOT_BYTES).unwrap();
        let expected = size_of::<Tables>() + Tables::requested_bytes(&m.funcs[0]).unwrap();
        assert_eq!(program.usage().0, expected);

        let resident = ResidentProgram::prepare(&[unit(1, 1)], None).unwrap();
        let expected = size_of::<Option<Arc<ResidentFunctionIndex>>>()
            + size_of::<ResidentFunctionIndex>()
            - size_of::<Tables>()
            + 2 * size_of::<usize>()
            + Tables::requested_bytes(&unit(1, 1).unit().module().funcs[0]).unwrap();
        assert_eq!(resident.usage().0, expected);
    }
}

#[cfg(test)]
mod failure_tests {
    use super::*;
    use crate::mir::*;
    use crate::mir_verify::verify_mir;

    fn scalar() -> MirModule {
        let owner = OwnerScope {
            schema: crate::contract::MIR_SCHEMA.into(),
            module: "failure".into(),
        };
        let mut m = MirModule::unit(owner.clone());
        let i = m.intern_type(MirType::I32);
        let sig = m.intern_type(MirType::Fn(FnSig {
            params: vec![i],
            result: i,
        }));
        m.funcs.push(MirFunction {
            owner,
            name: "id".into(),
            sig,
            slots: vec![MirSlot {
                kind: SlotKind::Param,
                ty: i,
                name: None,
            }],
            blocks: vec![MirBlock {
                instrs: vec![],
                term: Terminator::Return { value: 0 },
            }],
            entry: 0,
            source_snapshot: 0,
            spans: vec![],
            env: None,
        });
        m
    }
    #[test]
    fn av_runtime_825_second_allocation_failure_reclaims_and_next_request_works() {
        let checked = verify_mir(&scalar()).unwrap();
        let budget = IndexBudget::new(4096);
        budget.0.lock().unwrap().fail_attempt = Some(2);
        assert_eq!(
            CheckedFunctionIndex::build(&checked, 0, &budget).unwrap_err(),
            IndexError::Allocation
        );
        assert_eq!(budget.usage().0, 0);
        assert_eq!(budget.usage().2, 2);
        budget.0.lock().unwrap().fail_attempt = None;
        let next = CheckedFunctionIndex::build(&checked, 0, &budget).unwrap();
        assert_eq!(next.logical_pc(0, 0), Some(0));
        assert_eq!(next.param_slots(), &[0]);
        drop(next);
        assert_eq!(budget.usage().0, 0);
    }
    #[test]
    fn av_runtime_825_resident_reuse_cannot_escape_its_budget_domain() {
        let m = scalar();
        let wire = crate::abc::encode_abc(&m).unwrap().to_flat();
        let code = Arc::new(
            crate::function_code::FunctionCode::admit_unit(
                crate::abc_verify::verify_abc_bytes(&wire, &m.owner).unwrap(),
                1,
            )
            .unwrap(),
        );
        let budget = IndexBudget::new(4096);
        let index = ResidentFunctionIndex::prepare(code.clone(), None, &budget).unwrap();
        let other = IndexBudget::new(0);
        assert_eq!(
            ResidentFunctionIndex::prepare(code, Some(&index), &other).unwrap_err(),
            IndexError::BudgetDomain
        );
        assert_eq!(other.usage(), (0, 0, 0));
        drop(index);
        assert_eq!(budget.usage().0, 0);
    }
    #[test]
    fn av_runtime_825_delivered_owned_and_unknown_profiles_never_fallback() {
        let mut owned = scalar();
        owned.profile = crate::contract::AV_OWNED_PROFILE.into();
        owned.owner.schema = crate::contract::MIR_OWNED_SCHEMA.into();
        owned.funcs[0].owner = owned.owner.clone();
        let checked = verify_mir(&owned).unwrap();
        let budget = IndexBudget::new(4096);
        assert_eq!(
            CheckedFunctionIndex::build(&checked, 0, &budget).unwrap_err(),
            IndexError::UnsupportedProfile
        );
        let wire = crate::abc::encode_abc(&owned).unwrap().to_flat();
        let code = Arc::new(
            crate::function_code::FunctionCode::admit_unit(
                crate::abc_verify::verify_abc_bytes(&wire, &owned.owner).unwrap(),
                1,
            )
            .unwrap(),
        );
        assert_eq!(
            ResidentFunctionIndex::prepare(code, None, &budget).unwrap_err(),
            IndexError::UnsupportedProfile
        );
        assert_eq!(budget.usage(), (0, 0, 0));
        owned.profile = "unknown/825".into();
        assert!(verify_mir(&owned).is_err());
    }
}

#[cfg(test)]
mod candidate_tests {
    use super::*;
    use crate::mir::*;
    use crate::mir_verify::verify_mir;
    #[test]
    fn av_runtime_825_parameter_candidate_real_helper_counts() {
        let owner = OwnerScope {
            schema: crate::contract::MIR_SCHEMA.into(),
            module: "param-candidate".into(),
        };
        let mut m = MirModule::unit(owner.clone());
        let i = m.intern_type(MirType::I32);
        let sig = m.intern_type(MirType::Fn(FnSig {
            params: vec![i, i],
            result: i,
        }));
        m.funcs.push(MirFunction {
            owner,
            name: "candidate".into(),
            sig,
            slots: vec![
                MirSlot {
                    kind: SlotKind::Param,
                    ty: i,
                    name: None,
                },
                MirSlot {
                    kind: SlotKind::Temp,
                    ty: i,
                    name: None,
                },
                MirSlot {
                    kind: SlotKind::Param,
                    ty: i,
                    name: None,
                },
            ],
            blocks: vec![MirBlock {
                instrs: vec![],
                term: Terminator::Return { value: 0 },
            }],
            entry: 0,
            source_snapshot: 0,
            spans: vec![],
            env: None,
        });
        let checked = verify_mir(&m).unwrap();
        let f = &checked.module().funcs[0];
        // Invoke the delivered production helper in the same pattern as root
        // argument validation (av.rs): once for arity, once per supplied arg.
        let mut owned_maps = Vec::new();
        owned_maps.push(f.param_slots());
        let arity = owned_maps[0].len();
        for _ in 0..arity {
            owned_maps.push(f.param_slots());
        }
        assert_eq!(owned_maps.len(), 3);
        assert!(owned_maps.iter().all(|slots| slots == &[0, 2]));
        let payload_bytes: usize = owned_maps
            .iter()
            .map(|v| v.capacity() * size_of::<u32>())
            .sum();
        assert!(payload_bytes >= 24);
        let budget = IndexBudget::new(4096);
        let index = CheckedFunctionIndex::build(&checked, 0, &budget).unwrap();
        let before = budget.usage();
        let arity_new = index.param_slots().len();
        let mapping = index.param_slots().as_ptr();
        let args = [42, 7];
        let mut slots = [0i32; 3];
        for (n, arg) in args.iter().enumerate() {
            assert_eq!(index.param_slots().as_ptr(), mapping);
            slots[index.param_slots()[n] as usize] = *arg;
        }
        assert_eq!(arity_new, arity);
        assert_eq!(slots, [42, 0, 7]);
        assert_eq!(budget.usage(), before);
        eprintln!("parameter microcase: slots=3 params=2 reference_owned_maps=3 reference_payload_bytes={payload_bytes} derived_query_owned_maps=0 index_charged_bytes={}", before.0);
    }
}

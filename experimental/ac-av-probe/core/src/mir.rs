//! Typed MIR data model for the `av.core-i32/1` profile (PLAN-798 §5.2).
//!
//! Shape: explicit basic blocks with a UNIQUE terminator each, typed mutable
//! local/temp slots (deliberately NOT SSA), sequential instructions, and a
//! per-function source map. Module-level tables carry types, constants and
//! logical imports (test trace intrinsics).
//!
//! Trust model: [`MirModule`] is UNCHECKED. The only constructor of
//! [`crate::mir_verify::CheckedMir`] is `verify_mir`; mutating checked
//! content is not possible through the public API (the inner module is
//! private), so any rewrite invalidates the credential by construction.
//!
//! Block/slot ids are positional identities within one function — they are
//! not stable cross-revision ids. Persistence goes through the ABC wire
//! codec plus the probe's unit/manifest envelope, never an enum dump.

use crate::contract::{SpanInfo, AV_PROFILE};

/// Index into a module/unit type table.
pub type TypeIdx = u32;
/// Index into a module/unit const table.
pub const NO_CONST: u32 = u32::MAX;
/// Index into a module/unit function table.
pub type FuncIdx = u32;
/// Index into a module/unit import table.
pub type ImportIdx = u32;
/// Slot id within one function.
pub type SlotId = u32;
/// Block id within one function.
pub type BlockId = u32;

/// Logical owner scope of a module or function unit: schema + logical module
/// name. Absolute filesystem paths never enter this identity (PLAN-798
/// §5.3); content-identical units from different projects share cache keys.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct OwnerScope {
    pub schema: String,
    pub module: String,
}

/// A MIR type. The v1 profile has exactly two value types plus function
/// signatures for call admission.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum MirType {
    I32,
    Bool,
    Fn(FnSig),
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct FnSig {
    pub params: Vec<TypeIdx>,
    pub result: TypeIdx,
}

/// Module constants: i32 or bool. Bool constants are `true`/`false` only.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MirConst {
    I32(i32),
    Bool(bool),
}

/// Effect classification of an import (mirrors the HIR intrinsic effect).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Effect {
    Observable,
    Pure,
}

/// A logical import: test trace intrinsics in v1. The symbol is the HIR
/// intrinsic symbol (`hir.test.mark_a`), never a host function pointer.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct MirImport {
    pub symbol: String,
    pub effect: Effect,
    pub sig: TypeIdx,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SlotKind {
    /// Parameter slot — initialized on frame entry.
    Param,
    /// Named local (`let`/`var`) — initialized by instructions.
    Local,
    /// Compiler temp — initialized by instructions.
    Temp,
}

#[derive(Clone, Debug)]
pub struct MirSlot {
    pub kind: SlotKind,
    pub ty: TypeIdx,
    pub name: Option<String>,
}

/// Binary operations. Arithmetic traps on overflow/div-by-zero/MIN%-1 per
/// the scalar contract; the trap policy is implicit in the profile (frozen
/// by T-01), so it is not carried per instruction.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BinOp {
    AddI32,
    SubI32,
    MulI32,
    DivI32,
    RemI32,
    LtI32,
    LeI32,
    GtI32,
    GeI32,
    EqI32,
    NeI32,
}

impl BinOp {
    pub fn result_is_bool(&self) -> bool {
        matches!(
            self,
            BinOp::LtI32 | BinOp::LeI32 | BinOp::GtI32 | BinOp::GeI32 | BinOp::EqI32 | BinOp::NeI32
        )
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum UnOp {
    NegI32,
    NotBool,
}

/// Test trace events (PLAN-798 §5.1): observation only, no I/O capability.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TraceEvent {
    MarkA,
    MarkB,
}

#[derive(Clone, Debug)]
pub enum CalleeRef {
    Func(FuncIdx),
    Import(ImportIdx),
}

#[derive(Clone, Debug)]
pub enum MirInstr {
    Const {
        dst: SlotId,
        value: u32,
    },
    Copy {
        dst: SlotId,
        src: SlotId,
    },
    Bin {
        op: BinOp,
        dst: SlotId,
        lhs: SlotId,
        rhs: SlotId,
    },
    Un {
        op: UnOp,
        dst: SlotId,
        operand: SlotId,
    },
    Call {
        callee: CalleeRef,
        dst: SlotId,
        args: Vec<SlotId>,
    },
    Trace {
        event: TraceEvent,
    },
}

#[derive(Clone, Debug)]
pub enum Terminator {
    Jump {
        target: BlockId,
    },
    Branch {
        cond: SlotId,
        then: BlockId,
        else_: BlockId,
    },
    Return {
        value: SlotId,
    },
}

#[derive(Clone, Debug)]
pub struct MirBlock {
    pub instrs: Vec<MirInstr>,
    pub term: Terminator,
}

/// One function body in MIR. `spans` maps FLAT instruction indices
/// (block-major order: block 0 instrs, block 1 instrs, …) to source info;
/// entries are sorted by `instr` and are diagnostics-only.
#[derive(Clone, Debug)]
pub struct MirFunction {
    pub owner: OwnerScope,
    pub name: String,
    pub sig: TypeIdx,
    pub slots: Vec<MirSlot>,
    pub blocks: Vec<MirBlock>,
    pub entry: BlockId,
    /// Source snapshot fingerprint this function was lowered from.
    pub source_snapshot: u64,
    pub spans: Vec<(u32, SpanInfo)>,
}

impl MirFunction {
    /// Flat instruction count (spans key domain).
    pub fn flat_instr_count(&self) -> u32 {
        self.blocks.iter().map(|b| b.instrs.len() as u32).sum()
    }

    /// Flat index of the first instruction of `block`.
    pub fn block_base(&self, block: BlockId) -> u32 {
        self.blocks[..block as usize]
            .iter()
            .map(|b| b.instrs.len() as u32)
            .sum()
    }
}

/// An UNCHECKED MIR module (or single-function unit when `funcs.len() == 1`
/// and `is_unit` is set — units address callees through imports only).
#[derive(Clone, Debug)]
pub struct MirModule {
    pub profile: String,
    pub owner: OwnerScope,
    pub is_unit: bool,
    pub types: Vec<MirType>,
    pub consts: Vec<MirConst>,
    pub imports: Vec<MirImport>,
    pub funcs: Vec<MirFunction>,
    pub entry: FuncIdx,
    /// Document-level capabilities this unit requires (e.g. trace).
    pub capabilities: Vec<String>,
}

impl MirModule {
    pub fn new(owner: OwnerScope) -> Self {
        MirModule {
            profile: AV_PROFILE.to_string(),
            owner,
            is_unit: false,
            types: Vec::new(),
            consts: Vec::new(),
            imports: Vec::new(),
            funcs: Vec::new(),
            entry: 0,
            capabilities: Vec::new(),
        }
    }

    pub fn unit(owner: OwnerScope) -> Self {
        let mut m = MirModule::new(owner);
        m.is_unit = true;
        m
    }

    /// Resolve a type to its scalar shape for verification (`None` = Fn).
    pub fn type_shape(&self, idx: TypeIdx) -> Option<ScalarShape> {
        match self.types.get(idx as usize)? {
            MirType::I32 => Some(ScalarShape::I32),
            MirType::Bool => Some(ScalarShape::Bool),
            MirType::Fn(_) => None,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ScalarShape {
    I32,
    Bool,
}

/// Marker consumed by builders to avoid accidentally treating an unchecked
/// module as verified (the Checked credential lives in
/// [`crate::mir_verify::CheckedMir`]).
pub struct MirModuleUncheckedMarker;

impl MirModuleUncheckedMarker {
    pub fn note() -> &'static str {
        "MirModule is unchecked; only verify_mir produces a CheckedMir"
    }
}

/// Interning helpers shared by the HIR adapter and unit assembly.
impl MirModule {
    pub fn intern_type(&mut self, t: MirType) -> TypeIdx {
        if let Some(i) = self.types.iter().position(|x| *x == t) {
            return i as TypeIdx;
        }
        self.types.push(t);
        (self.types.len() - 1) as TypeIdx
    }

    pub fn intern_const(&mut self, c: MirConst) -> u32 {
        if let Some(i) = self.consts.iter().position(|x| *x == c) {
            return i as u32;
        }
        self.consts.push(c);
        self.consts.len() as u32 - 1
    }

    pub fn intern_import(&mut self, imp: MirImport) -> ImportIdx {
        if let Some(i) = self.imports.iter().position(|x| *x == imp) {
            return i as ImportIdx;
        }
        self.imports.push(imp);
        self.imports.len() as ImportIdx - 1
    }

    pub fn sig_of(&self, idx: TypeIdx) -> Option<&FnSig> {
        match self.types.get(idx as usize)? {
            MirType::Fn(s) => Some(s),
            _ => None,
        }
    }

    /// Total table-entry count (reporting helper; the FROZEN budget is
    /// enforced per table family — see mir_verify and contract-decision §6).
    pub fn table_entries(&self) -> usize {
        self.types.len()
            + self.consts.len()
            + self.imports.len()
            + self.funcs.len()
            + self
                .funcs
                .iter()
                .map(|f| f.slots.len() + f.blocks.len() + f.flat_instr_count() as usize)
                .sum::<usize>()
    }

    /// Deterministic structural heap estimate (bytes) of this module as it
    /// lives in memory — the staging/hot partition accounting primitive.
    /// Wire length is NOT a working-set proxy (F798-R3-08): decoded clones,
    /// slot/block tables and span maps are real allocations the ledger must
    /// cover. The estimate uses `len()` (not capacity) so it is stable
    /// across allocator behavior; it deliberately OVER-counts fixed
    /// per-allocation headers, never under-counts the payload bytes.
    pub fn heap_estimate(&self) -> usize {
        const STR: usize = std::mem::size_of::<String>(); // 24
        const VEC: usize = std::mem::size_of::<usize>() * 3; // 24
        let mut b = std::mem::size_of::<MirModule>()
            + self.profile.len()
            + self.owner.schema.len()
            + self.owner.module.len()
            + STR * 3
            + self
                .capabilities
                .iter()
                .map(|c| c.len() + STR)
                .sum::<usize>()
            + VEC * 4;
        b += self
            .types
            .iter()
            .map(|t| match t {
                MirType::I32 | MirType::Bool => 0, // inline enum, table vec counted below
                MirType::Fn(s) => s.params.len() * std::mem::size_of::<TypeIdx>() + VEC,
            })
            .sum::<usize>()
            + self.types.len() * std::mem::size_of::<MirType>()
            + VEC;
        b += self.consts.len() * std::mem::size_of::<MirConst>() + VEC;
        b += self
            .imports
            .iter()
            .map(|i| i.symbol.len() + STR)
            .sum::<usize>()
            + self.imports.len() * std::mem::size_of::<MirImport>()
            + VEC;
        b += self.funcs.len() * std::mem::size_of::<MirFunction>() + VEC;
        for f in &self.funcs {
            b += f.name.len() + STR;
            b += f
                .slots
                .iter()
                .map(|s| s.name.as_ref().map_or(0, |n| n.len() + STR))
                .sum::<usize>()
                + f.slots.len() * std::mem::size_of::<MirSlot>()
                + VEC;
            b += f.blocks.len() * std::mem::size_of::<MirBlock>() + VEC;
            for blk in &f.blocks {
                for instr in &blk.instrs {
                    if let MirInstr::Call { args, .. } = instr {
                        b += args.len() * std::mem::size_of::<SlotId>() + VEC;
                    }
                }
                b += blk.instrs.len() * std::mem::size_of::<MirInstr>() + VEC;
            }
            b += f.spans.len() * std::mem::size_of::<(u32, SpanInfo)>() + VEC;
        }
        b
    }
}

//! PLAN-891 native typed-data entry: the six-scalar typed face
//! (i32/bool/i64/u64/f32/f64) over an INDEPENDENT versioned mn-private
//! ABI (`msvc-x64/priv-typed-data-v1`) and a one-shot bitwise result
//! channel (`mn891.typed.frames/v1`).
//!
//! Freeze (plan §2/§5.2–§5.3):
//! - Logical types stay distinct in MIR/verifier/signatures/fingerprints
//!   even when they share a carrier: I32/Bool/F32 cross as I32 (low 32
//!   bits, 4-byte width), I64/U64/F64 cross as I64 (8 bytes). Floats are
//!   OPAQUE bit patterns end to end — no arithmetic, formatting, NaN
//!   canonicalization or f32↔f64 widening anywhere in the channel.
//! - Pure-value records (wide fields allowed, no Str/borrow/owning
//!   resources at any depth) cross internally by address; the callee
//!   copies into its own value slots; a record RESULT uses the trailing
//!   hidden out-aggregate address (never a typed entry result).
//! - The generated typed wrapper reads the request frame at RUNTIME and
//!   loads arguments from it — parameters are never baked into the user
//!   objects or the wrapper. Same signature + new args/nonce ⇒ the
//!   cached wrapper is reused verbatim.
//! - Checked i32 arithmetic traps through `mn_typed_trap(reason)` (a
//!   bounded trap frame), never a bare exit-70 guess.

use crate::config::*;
use crate::lower::{emit_function, LowerMode};
use crate::symbols;
use crate::MnError;
use ace_av_core::contract::{fnv1a64, AV_DATA_PROFILE};
use ace_av_core::mir::{MirInstr, MirModule, MirType};
use cranelift_codegen::ir::{AbiParam, InstBuilder, MemFlags, Signature, StackSlotData};
use cranelift_codegen::isa::CallConv;
use cranelift_module::{default_libcall_names, Linkage, Module};
use cranelift_object::{ObjectBuilder, ObjectModule};

const WINCALL: CallConv = CallConv::WindowsFastcall;

// ---- frame freeze (mirrors ace_typed_entry_runtime byte-for-byte) -------

pub const REQ_MAGIC: u32 = 0x4D4E_5431; // "MNT1"
pub const RES_MAGIC: u32 = 0x4D4E_5231; // "MNR1"
pub const FRAME_VERSION: u32 = 1;
pub const REQ_HEADER_BYTES: usize = 32;
pub const FRAME_HASH_BYTES: usize = 8;
pub const RES_FRAME_BYTES: usize = 44;

/// Value kind codes (request validation + response framing).
pub const KIND_I32: u32 = 1;
pub const KIND_BOOL: u32 = 2;
pub const KIND_I64: u32 = 3;
pub const KIND_U64: u32 = 4;
pub const KIND_F32: u32 = 5;
pub const KIND_F64: u32 = 6;

/// Response statuses (mutually exclusive outcome discriminants).
pub const STATUS_VALUE: u32 = 0;
pub const STATUS_TRAP: u32 = 1;
pub const STATUS_RESOURCE: u32 = 2;
pub const STATUS_ERROR: u32 = 3;

/// Trap reason codes (bounded, named — no exit-code guessing).
pub const TRAP_DIV_ZERO: u32 = 1;
pub const TRAP_DIV_OVERFLOW: u32 = 2;
pub const TRAP_ADD_OVERFLOW: u32 = 3;
pub const TRAP_SUB_OVERFLOW: u32 = 4;
pub const TRAP_MUL_OVERFLOW: u32 = 5;
pub const TRAP_NEG_OVERFLOW: u32 = 6;

/// PLAN-1000 F-1000-01: the native machine's own trap names for the
/// bounded reasons (the `mn.trap.*` taxonomy — a native-side observation
/// authority, never an AV-engine inference). Shared by the mn-worker's
/// response-side trap naming and the probe's frame admission.
pub fn trap_label(reason: u32) -> Option<&'static str> {
    match reason {
        TRAP_DIV_ZERO => Some("mn.trap.div-zero"),
        TRAP_DIV_OVERFLOW => Some("mn.trap.div-overflow"),
        TRAP_ADD_OVERFLOW => Some("mn.trap.add-overflow"),
        TRAP_SUB_OVERFLOW => Some("mn.trap.sub-overflow"),
        TRAP_MUL_OVERFLOW => Some("mn.trap.mul-overflow"),
        TRAP_NEG_OVERFLOW => Some("mn.trap.neg-overflow"),
        _ => None,
    }
}

/// The six admitted typed entry scalars.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TypedKind {
    I32,
    Bool,
    I64,
    U64,
    F32,
    F64,
}

impl TypedKind {
    pub fn code(self) -> u32 {
        match self {
            TypedKind::I32 => KIND_I32,
            TypedKind::Bool => KIND_BOOL,
            TypedKind::I64 => KIND_I64,
            TypedKind::U64 => KIND_U64,
            TypedKind::F32 => KIND_F32,
            TypedKind::F64 => KIND_F64,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            TypedKind::I32 => "i32",
            TypedKind::Bool => "bool",
            TypedKind::I64 => "i64",
            TypedKind::U64 => "u64",
            TypedKind::F32 => "f32",
            TypedKind::F64 => "f64",
        }
    }

    pub fn from_label(label: &str) -> Option<TypedKind> {
        Some(match label {
            "i32" => TypedKind::I32,
            "bool" => TypedKind::Bool,
            "i64" => TypedKind::I64,
            "u64" => TypedKind::U64,
            "f32" => TypedKind::F32,
            "f64" => TypedKind::F64,
            _ => return None,
        })
    }

    /// Carrier byte width (frame payload and storage width).
    pub fn width(self) -> usize {
        match self {
            TypedKind::I32 | TypedKind::Bool | TypedKind::F32 => 4,
            TypedKind::I64 | TypedKind::U64 | TypedKind::F64 => 8,
        }
    }

    pub fn carrier(self) -> cranelift_codegen::ir::types::Type {
        match self.width() {
            4 => cranelift_codegen::ir::types::I32,
            _ => cranelift_codegen::ir::types::I64,
        }
    }

    fn of_type(ty: &MirType) -> Option<TypedKind> {
        Some(match ty {
            MirType::I32 => TypedKind::I32,
            MirType::Bool => TypedKind::Bool,
            MirType::I64 => TypedKind::I64,
            MirType::U64 => TypedKind::U64,
            MirType::F32 => TypedKind::F32,
            MirType::F64 => TypedKind::F64,
            _ => return None,
        })
    }
}

/// The frozen transport plan of one typed signature: canonical signature
/// text, its id, per-argument payload offsets and the exact frame length.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypedSigPlan {
    pub sig_text: String,
    pub sig_id: u32,
    pub args: Vec<TypedKind>,
    /// Payload byte offset of each argument (width-aligned).
    pub offsets: Vec<usize>,
    pub payload_len: usize,
    pub result: TypedKind,
}

impl TypedSigPlan {
    pub fn frame_len(&self) -> usize {
        REQ_HEADER_BYTES + self.payload_len + FRAME_HASH_BYTES
    }

    /// Canonical signature text of a MIR function type under the typed
    /// ABI: six scalars by logical label; a record parameter renders by
    /// nominal shape (records never appear in an ENTRY plan but internal
    /// units carry record-bearing import signatures).
    pub fn of_sig(m: &MirModule, sig_idx: u32) -> Result<TypedSigPlan, MnError> {
        let Some(sig) = m.sig_of(sig_idx) else {
            return Err(MnError::Unsupported(
                "typed-sig",
                "type is not a function signature".to_string(),
            ));
        };
        let mut args = Vec::with_capacity(sig.params.len());
        let mut params = Vec::with_capacity(sig.params.len());
        for p in &sig.params {
            match m.types.get(*p as usize) {
                Some(
                    ty @ (MirType::I32
                    | MirType::Bool
                    | MirType::I64
                    | MirType::U64
                    | MirType::F32
                    | MirType::F64),
                ) => {
                    let k = TypedKind::of_type(ty).expect("scalar arm");
                    args.push(k);
                    params.push(k.label().to_string());
                }
                Some(MirType::Record(_) | MirType::RecordView(_)) => {
                    // Internal record parameters cross by address; they are
                    // not entry payload (never part of the frame plan).
                    params.push(crate::fingerprint::shape(m, *p));
                }
                _ => {
                    return Err(MnError::Unsupported(
                        "typed-param",
                        "parameter type outside the typed-data ABI".to_string(),
                    ))
                }
            }
        }
        let result = match m.types.get(sig.result as usize) {
            Some(
                ty @ (MirType::I32
                | MirType::Bool
                | MirType::I64
                | MirType::U64
                | MirType::F32
                | MirType::F64),
            ) => TypedKind::of_type(ty).expect("scalar arm"),
            _ => {
                return Err(MnError::Unsupported(
                    "typed-result",
                    "result type outside the typed-data ABI".to_string(),
                ))
            }
        };
        let sig_text = format!("({})->{}", params.join(","), result.label());
        let mut offsets = Vec::with_capacity(args.len());
        let mut off = 0usize;
        for k in &args {
            off = off.next_multiple_of(k.width());
            offsets.push(off);
            off += k.width();
        }
        if off > TYPED_MAX_ARGS_BYTES {
            return Err(MnError::Resource(
                "typed-args",
                format!(
                    "typed argument payload {off} bytes exceeds the frozen {TYPED_MAX_ARGS_BYTES} cap"
                ),
            ));
        }
        Ok(TypedSigPlan {
            sig_id: fnv1a64(sig_text.as_bytes()) as u32,
            sig_text,
            args,
            offsets,
            payload_len: off,
            result,
        })
    }
}

/// Entry plans carry ONLY the six scalars (records never exit the typed
/// entry — plan G1). Used for the entry unit's wrapper + frame codec.
pub fn entry_plan(unit: &MirModule) -> Result<TypedSigPlan, MnError> {
    let f = unit.funcs.first().ok_or_else(|| {
        MnError::Unsupported("unit-shape", "typed entry unit has no function".to_string())
    })?;
    let plan = TypedSigPlan::of_sig(unit, f.sig)?;
    if plan.sig_text.contains("rec(") || plan.sig_text.contains("view:") {
        return Err(MnError::Unsupported(
            "typed-entry",
            "records cannot cross the typed entry".to_string(),
        ));
    }
    Ok(plan)
}

// ---- typed admission -----------------------------------------------------

/// Admit ONE typed unit at the codegen door. The shared `verify_mir` has
/// already run (whole module, then split); this re-checks the typed
/// closure: six scalars + PURE records only (no Str/StrView/Char/MutRef/
/// Fn-slot/List/Variant/Place/Closure/Dyn/Unit anywhere reachable), the
/// frozen instruction subset, and named rejections for everything else —
/// codegen count stays 0 for rejected units. `is_entry` additionally
/// requires the six-scalar-only entry signature.
pub fn admit_typed_unit(unit: &MirModule, is_entry: bool) -> Result<(), MnError> {
    if unit.profile != AV_DATA_PROFILE {
        return Err(MnError::Unsupported(
            "profile",
            format!(
                "typed admission requires `{AV_DATA_PROFILE}`, got `{}`",
                unit.profile
            ),
        ));
    }
    if !unit.is_unit || unit.funcs.len() != 1 {
        return Err(MnError::Unsupported(
            "unit-shape",
            "typed codegen consumes single-function units only".to_string(),
        ));
    }
    if unit.is_host_console_profile() || !unit.host_imports.is_empty() {
        return Err(MnError::Unsupported(
            "host-console",
            "console declarations are outside the typed surface".to_string(),
        ));
    }
    let f = &unit.funcs[0];
    // Touched-type closure (iterative; mirrors the scalar walk's shape).
    let mut seen = std::collections::BTreeSet::new();
    let mut touched: Vec<u32> = vec![f.sig];
    for s in &f.slots {
        touched.push(s.ty);
    }
    for imp in &unit.imports {
        touched.push(imp.sig);
    }
    for b in &f.blocks {
        for ins in &b.instrs {
            if let MirInstr::Aggregate { record_ty, .. } = ins {
                touched.push(*record_ty);
            }
        }
    }
    while let Some(t) = touched.pop() {
        if !seen.insert(t) {
            continue;
        }
        match unit.types.get(t as usize) {
            Some(
                MirType::I32
                | MirType::Bool
                | MirType::I64
                | MirType::U64
                | MirType::F32
                | MirType::F64,
            ) => {}
            Some(MirType::Record(r)) => {
                for fd in &r.fields {
                    touched.push(fd.ty);
                }
            }
            Some(MirType::Fn(s)) => {
                for p in &s.params {
                    touched.push(*p);
                }
                touched.push(s.result);
            }
            Some(other) => {
                return Err(MnError::Unsupported(
                    "type",
                    format!("type {t} ({other:?}) is outside the typed-data closure"),
                ));
            }
            None => {
                return Err(MnError::Unsupported(
                    "type",
                    format!("type {t} is missing from the unit table"),
                ));
            }
        }
    }
    // Instruction subset: Const/Copy/Bin(i32/bool)/Un(i32/bool)/Call/Trace/
    // Aggregate/FieldGet/FieldSet. Wide arithmetic stays a NAMED rejection
    // (the verifier already refuses wide Bin/Un; admission re-checks).
    for b in &f.blocks {
        for ins in &b.instrs {
            match ins {
                MirInstr::Const { dst, value } => {
                    let want = TypedKind::of_type(
                        unit.types
                            .get(f.slots[*dst as usize].ty as usize)
                            .unwrap_or(&MirType::Unit),
                    );
                    let got = unit.consts.get(*value as usize).and_then(|c| match c {
                        ace_av_core::mir::MirConst::I32(_) => Some(TypedKind::I32),
                        ace_av_core::mir::MirConst::Bool(_) => Some(TypedKind::Bool),
                        ace_av_core::mir::MirConst::I64(_) => Some(TypedKind::I64),
                        ace_av_core::mir::MirConst::U64(_) => Some(TypedKind::U64),
                        ace_av_core::mir::MirConst::F32Bits(_) => Some(TypedKind::F32),
                        ace_av_core::mir::MirConst::F64Bits(_) => Some(TypedKind::F64),
                        _ => None,
                    });
                    if want != got {
                        return Err(MnError::Unsupported(
                            "typed-const",
                            format!(
                                "constant {value} and destination slot {dst} disagree on typed kind"
                            ),
                        ));
                    }
                }
                MirInstr::Copy { .. } => {}
                MirInstr::Bin { lhs, dst, .. }
                | MirInstr::Un {
                    operand: lhs, dst, ..
                } => {
                    let op_ty = |s: u32| unit.types.get(f.slots[s as usize].ty as usize);
                    let scalar32 =
                        |s: u32| matches!(op_ty(s), Some(MirType::I32) | Some(MirType::Bool));
                    if !scalar32(*lhs) || !scalar32(*dst) {
                        return Err(MnError::Unsupported(
                            "typed-op",
                            "wide/control arithmetic is outside the typed data closure".to_string(),
                        ));
                    }
                }
                MirInstr::Call { callee, .. } => {
                    if !matches!(callee, ace_av_core::mir::CalleeRef::Import(_)) {
                        return Err(MnError::Unsupported(
                            "unit-callee",
                            "a typed unit must address callees through imports".to_string(),
                        ));
                    }
                }
                MirInstr::Trace { .. } => {}
                MirInstr::Aggregate { .. }
                | MirInstr::FieldGet { .. }
                | MirInstr::FieldSet { .. } => {}
                other => {
                    return Err(MnError::Unsupported(
                        "instruction",
                        format!("`{other:?}` is outside the typed data closure"),
                    ));
                }
            }
        }
    }
    for imp in &unit.imports {
        symbols::typed_import_symbol(&unit.owner, &imp.symbol)?;
    }
    if is_entry {
        entry_plan(unit)?;
    }
    Ok(())
}

/// Cranelift signature of a MIR function type under the TYPED ABI: the
/// six scalars as integer bit carriers, records by address with the
/// hidden trailing out-aggregate for record results (same lowering shape
/// as the data-domain legacy path — never a platform C aggregate ABI).
pub fn typed_sig_of(module: &MirModule, ty: u32) -> Result<Signature, MnError> {
    let Some(sig) = module.sig_of(ty) else {
        return Err(MnError::Unsupported(
            "typed-sig",
            "type is not a function signature".to_string(),
        ));
    };
    let mut params = Vec::with_capacity(sig.params.len() + 1);
    for p in &sig.params {
        match module.types.get(*p as usize) {
            Some(ty) => match TypedKind::of_type(ty) {
                Some(k) => params.push(AbiParam::new(k.carrier())),
                None => match ty {
                    MirType::Record(_) | MirType::RecordView(_) => {
                        params.push(AbiParam::new(cranelift_codegen::ir::types::I64));
                    }
                    _ => {
                        return Err(MnError::Unsupported(
                            "typed-param",
                            "parameter type outside the typed-data ABI".to_string(),
                        ))
                    }
                },
            },
            None => {
                return Err(MnError::Unsupported(
                    "typed-param",
                    "unknown parameter type is outside the typed-data ABI".to_string(),
                ))
            }
        }
    }
    let ret = match module.types.get(sig.result as usize) {
        Some(ty) => match TypedKind::of_type(ty) {
            Some(k) => vec![AbiParam::new(k.carrier())],
            None => match ty {
                MirType::Record(_) | MirType::RecordView(_) => {
                    params.push(AbiParam::new(cranelift_codegen::ir::types::I64));
                    vec![]
                }
                _ => {
                    return Err(MnError::Unsupported(
                        "typed-result",
                        "result type outside the typed-data ABI".to_string(),
                    ))
                }
            },
        },
        None => {
            return Err(MnError::Unsupported(
                "typed-result",
                "unknown result type is outside the typed-data ABI".to_string(),
            ))
        }
    };
    Ok(Signature {
        call_conv: WINCALL,
        params,
        returns: ret,
    })
}

/// One emitted typed unit object.
#[derive(Debug)]
pub struct TypedUnitObject {
    pub symbol: String,
    pub bytes: Vec<u8>,
    pub imports: Vec<(String, String)>,
}

/// Lower and emit one verified typed unit to a relocatable COFF object
/// (typed symbol domain; checked-arithmetic traps route to the typed
/// trap helper, not exit 70).
pub fn emit_typed_unit_coff(unit: &MirModule, is_entry: bool) -> Result<TypedUnitObject, MnError> {
    admit_typed_unit(unit, is_entry)?;
    let f = &unit.funcs[0];
    let isa = crate::emit::make_isa()?;
    let builder = ObjectBuilder::new(isa, format!("mn9-{}", f.name), default_libcall_names())
        .map_err(|e| MnError::Backend(format!("object builder: {e}")))?;
    let mut om = ObjectModule::new(builder);

    let exit_id = declare_exit(&mut om)?;
    let mut import_ids = Vec::with_capacity(unit.imports.len());
    let mut import_syms = Vec::with_capacity(unit.imports.len());
    for imp in &unit.imports {
        let logical = symbols::typed_import_symbol(&unit.owner, &imp.symbol)?;
        let sig = typed_sig_of(unit, imp.sig)?;
        let id = om
            .declare_function(&logical, Linkage::Import, &sig)
            .map_err(|e| MnError::Backend(format!("declare import {}: {e}", logical)))?;
        import_ids.push(id);
        import_syms.push((imp.symbol.clone(), logical));
    }
    // Bare Trace instructions still need their runtime symbols.
    let mut trace_ids: Vec<(String, cranelift_module::FuncId)> = Vec::new();
    for sym in ["hir.test.mark_a", "hir.test.mark_b"] {
        let uses = f.blocks.iter().any(|b| {
            b.instrs.iter().any(|i| {
                matches!(i, MirInstr::Trace { event } if crate::emit::event_symbol(*event) == sym)
            })
        });
        if uses && !unit.imports.iter().any(|i| i.symbol == sym) {
            let sig = trace_call_sig();
            let id = om
                .declare_function(sym, Linkage::Import, &sig)
                .map_err(|e| MnError::Backend(format!("declare {sym}: {e}")))?;
            trace_ids.push((sym.to_string(), id));
        }
    }

    let fn_sig = typed_sig_of(unit, f.sig)?;
    let symbol = symbols::typed_function_symbol(&unit.owner, &f.name);
    let fid = om
        .declare_function(&symbol, Linkage::Export, &fn_sig)
        .map_err(|e| MnError::Backend(format!("declare {symbol}: {e}")))?;

    let mut ctx = om.make_context();
    ctx.func.signature = fn_sig;
    emit_function(
        unit,
        &mut om,
        fid,
        &import_ids,
        &trace_ids,
        exit_id,
        &mut ctx,
        LowerMode::Typed,
        None,
    )
    .map_err(|e| MnError::Backend(e))?;
    om.define_function(fid, &mut ctx)
        .map_err(|e| MnError::Backend(format!("define {symbol}: {e}")))?;
    let bytes = om
        .finish()
        .emit()
        .map_err(|e| MnError::Backend(format!("emit object: {e}")))?;
    if bytes.len() > MAX_COFF_BYTES {
        return Err(MnError::Resource(
            "object-size",
            format!(
                "typed COFF for `{}` is {} bytes, above the frozen {} cap",
                f.name,
                bytes.len(),
                MAX_COFF_BYTES
            ),
        ));
    }
    Ok(TypedUnitObject {
        symbol,
        bytes,
        imports: import_syms,
    })
}

fn declare_exit(om: &mut ObjectModule) -> Result<cranelift_module::FuncId, MnError> {
    let sig = Signature {
        call_conv: WINCALL,
        params: vec![AbiParam::new(cranelift_codegen::ir::types::I32)],
        returns: vec![],
    };
    om.declare_function("ExitProcess", Linkage::Import, &sig)
        .map_err(|e| MnError::Backend(format!("declare ExitProcess: {e}")))
}

fn trace_call_sig() -> Signature {
    Signature {
        call_conv: WINCALL,
        params: vec![],
        returns: vec![AbiParam::new(cranelift_codegen::ir::types::I32)],
    }
}

// ---- the typed wrapper ---------------------------------------------------

/// The signature-shaped typed startup wrapper: `ac_start` reads ONE
/// request frame into its own stack buffer at runtime, checks the frame
/// against this wrapper's frozen signature identity, loads the arguments
/// from the payload, calls the entry, and writes the machine result as
/// the single response frame. Argument VALUES never enter the wrapper.
pub fn emit_typed_wrapper_coff(
    entry_symbol: &str,
    plan: &TypedSigPlan,
) -> Result<Vec<u8>, MnError> {
    let isa = crate::emit::make_isa()?;
    let builder = ObjectBuilder::new(isa, "mn9-ac-start", default_libcall_names())
        .map_err(|e| MnError::Backend(format!("object builder: {e}")))?;
    let mut om = ObjectModule::new(builder);

    let entry_sig = typed_entry_sig(plan);
    let entry_id = om
        .declare_function(entry_symbol, Linkage::Import, &entry_sig)
        .map_err(|e| MnError::Backend(format!("declare entry {entry_symbol}: {e}")))?;
    let read_id = declare_helper(
        &mut om,
        "mn_typed_read",
        &[
            cranelift_codegen::ir::types::I64,
            cranelift_codegen::ir::types::I32,
        ],
        &[cranelift_codegen::ir::types::I32],
    )?;
    let check_id = declare_helper(
        &mut om,
        "mn_typed_check",
        &[
            cranelift_codegen::ir::types::I64,
            cranelift_codegen::ir::types::I32,
            cranelift_codegen::ir::types::I32,
            cranelift_codegen::ir::types::I32,
        ],
        &[],
    )?;
    let write_id = declare_helper(
        &mut om,
        "mn_typed_write_value",
        &[
            cranelift_codegen::ir::types::I32,
            cranelift_codegen::ir::types::I64,
        ],
        &[],
    )?;
    let start_sig = Signature {
        call_conv: WINCALL,
        params: vec![],
        returns: vec![],
    };
    let start_id = om
        .declare_function(START_SYMBOL, Linkage::Export, &start_sig)
        .map_err(|e| MnError::Backend(format!("declare {}: {e}", START_SYMBOL)))?;

    let mut ctx = om.make_context();
    ctx.func.signature = start_sig.clone();
    {
        let mut fb_ctx = cranelift_frontend::FunctionBuilderContext::new();
        let mut fb = cranelift_frontend::FunctionBuilder::new(&mut ctx.func, &mut fb_ctx);
        let blk = fb.create_block();
        fb.switch_to_block(blk);
        fb.seal_block(blk);

        let frame_cap = plan.frame_len() as u32;
        let frame_slot = fb.create_sized_stack_slot(StackSlotData::new(
            cranelift_codegen::ir::StackSlotKind::ExplicitSlot,
            frame_cap,
            3, // 8-byte aligned
        ));
        let buf = fb
            .ins()
            .stack_addr(cranelift_codegen::ir::types::I64, frame_slot, 0);
        let cap = fb
            .ins()
            .iconst(cranelift_codegen::ir::types::I32, frame_cap as i64);

        let read_ref = om.declare_func_in_func(read_id, fb.func);
        fb.ins().call(read_ref, &[buf, cap]);
        let check_ref = om.declare_func_in_func(check_id, fb.func);
        let sig_id = fb
            .ins()
            .iconst(cranelift_codegen::ir::types::I32, plan.sig_id as i64);
        let want_len = fb
            .ins()
            .iconst(cranelift_codegen::ir::types::I32, frame_cap as i64);
        let want_argc = fb
            .ins()
            .iconst(cranelift_codegen::ir::types::I32, plan.args.len() as i64);
        fb.ins()
            .call(check_ref, &[buf, sig_id, want_len, want_argc]);

        // Load each argument from the payload at its planned offset.
        let mut argv: Vec<cranelift_codegen::ir::Value> = Vec::with_capacity(plan.args.len());
        for (k, off) in plan.args.iter().zip(plan.offsets.iter()) {
            let at = (REQ_HEADER_BYTES + off) as i32;
            let v = fb.ins().load(k.carrier(), MemFlags::trusted(), buf, at);
            argv.push(v);
        }
        let entry_ref = om.declare_func_in_func(entry_id, fb.func);
        let call = fb.ins().call(entry_ref, &argv);
        let result = fb.inst_results(call)[0];
        // Widen I32-carrier results into the I64 response field (zero
        // extension: only the low 32 bits are defined for 4-byte kinds).
        let bits = if plan.result.width() == 4 {
            fb.ins().uextend(cranelift_codegen::ir::types::I64, result)
        } else {
            result
        };
        let kind = fb
            .ins()
            .iconst(cranelift_codegen::ir::types::I32, plan.result.code() as i64);
        let write_ref = om.declare_func_in_func(write_id, fb.func);
        fb.ins().call(write_ref, &[kind, bits]);
        // never reached (the write helper exits); any trap code suffices
        fb.ins()
            .trap(cranelift_codegen::ir::TrapCode::user(1).expect("valid user trap code"));
        fb.finalize();
    }
    om.define_function(start_id, &mut ctx)
        .map_err(|e| MnError::Backend(format!("define {}: {e}", START_SYMBOL)))?;
    om.finish()
        .emit()
        .map_err(|e| MnError::Backend(format!("emit typed wrapper: {e}")))
}

fn typed_entry_sig(plan: &TypedSigPlan) -> Signature {
    Signature {
        call_conv: WINCALL,
        params: plan
            .args
            .iter()
            .map(|k| AbiParam::new(k.carrier()))
            .collect(),
        returns: vec![AbiParam::new(plan.result.carrier())],
    }
}

fn declare_helper(
    om: &mut ObjectModule,
    sym: &str,
    params: &[cranelift_codegen::ir::types::Type],
    rets: &[cranelift_codegen::ir::types::Type],
) -> Result<cranelift_module::FuncId, MnError> {
    let sig = Signature {
        call_conv: WINCALL,
        params: params.iter().map(|t| AbiParam::new(*t)).collect(),
        returns: rets.iter().map(|t| AbiParam::new(*t)).collect(),
    };
    om.declare_function(sym, Linkage::Import, &sig)
        .map_err(|e| MnError::Backend(format!("declare {sym}: {e}")))
}

// ---- host-side frame codec -----------------------------------------------

/// One typed argument as canonical raw bits (hex string, lowercase) plus
/// its kind — the parent/worker request payload form. `rawbits` never
/// ride JSON numbers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypedArg {
    pub kind: TypedKind,
    pub bits: u64,
}

/// Parse a canonical hex argument under `kind` (width-exact, bool 0/1).
pub fn parse_arg(kind: TypedKind, hex: &str) -> Result<TypedArg, MnError> {
    let lowered = hex.to_ascii_lowercase();
    if lowered.is_empty() || lowered.chars().any(|c| !c.is_ascii_hexdigit()) {
        return Err(MnError::Admit(
            "typed-arg",
            format!("argument `{hex}` is not canonical hex"),
        ));
    }
    // Canonical form (the same rules the parent-side typed judgement
    // uses): i64/u64/f64 exactly 16 lowercase digits, f32 exactly 8,
    // i32 unpadded 1..=8 digits (the unique zero encoding is "0").
    let exact = match kind {
        TypedKind::I64 | TypedKind::U64 | TypedKind::F64 => Some(16usize),
        TypedKind::F32 => Some(8),
        TypedKind::I32 | TypedKind::Bool => None,
    };
    if let Some(width) = exact {
        if lowered.len() != width {
            return Err(MnError::Admit(
                "typed-arg",
                format!(
                    "argument `{hex}` must be exactly {width} hex digits for {}",
                    kind.label()
                ),
            ));
        }
    } else if lowered.len() > 8 || (lowered.len() > 1 && lowered.starts_with('0')) {
        return Err(MnError::Admit(
            "typed-arg",
            format!(
                "argument `{hex}` is not a canonical unpadded {} form",
                kind.label()
            ),
        ));
    }
    let bits = u64::from_str_radix(&lowered, 16)
        .map_err(|e| MnError::Admit("typed-arg", format!("argument `{hex}`: {e}")))?;
    if kind == TypedKind::Bool && bits > 1 {
        return Err(MnError::Admit(
            "typed-arg",
            "bool arguments must be exactly 0 or 1".to_string(),
        ));
    }
    Ok(TypedArg { kind, bits })
}

/// Encode ONE request frame for `plan` from actual arguments. The frame
/// binds the signature id, arg count, nonce and the exact payload bytes;
/// the fnv1a64 checksum closes it. Trailing bytes never exist.
pub fn encode_request(
    plan: &TypedSigPlan,
    nonce: u64,
    args: &[TypedArg],
) -> Result<Vec<u8>, MnError> {
    if args.len() != plan.args.len() {
        return Err(MnError::Admit(
            "typed-argc",
            format!(
                "plan expects {} arguments, got {}",
                plan.args.len(),
                args.len()
            ),
        ));
    }
    let mut payload = vec![0u8; plan.payload_len];
    for (arg, (k, off)) in args.iter().zip(plan.args.iter().zip(plan.offsets.iter())) {
        if arg.kind != *k {
            return Err(MnError::Admit(
                "typed-arg-kind",
                format!(
                    "argument position expects {}, got {}",
                    k.label(),
                    arg.kind.label()
                ),
            ));
        }
        payload[*off..*off + k.width()].copy_from_slice(&arg.bits.to_le_bytes()[..k.width()]);
    }
    let mut frame = Vec::with_capacity(plan.frame_len());
    frame.extend_from_slice(&REQ_MAGIC.to_le_bytes());
    frame.extend_from_slice(&FRAME_VERSION.to_le_bytes());
    frame.extend_from_slice(&plan.sig_id.to_le_bytes());
    frame.extend_from_slice(&(plan.args.len() as u32).to_le_bytes());
    frame.extend_from_slice(&nonce.to_le_bytes());
    frame.extend_from_slice(&(plan.payload_len as u32).to_le_bytes());
    frame.extend_from_slice(&0u32.to_le_bytes());
    frame.extend_from_slice(&payload);
    frame.extend_from_slice(&fnv1a64(&frame).to_le_bytes());
    debug_assert_eq!(frame.len(), plan.frame_len());
    Ok(frame)
}

/// The decoded single response frame (exactly 44 bytes — trailing bytes
/// are a protocol violation the caller reports, never truncation).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TypedOutcome {
    Value { kind_code: u32, bits: u64 },
    Trap { reason: u32 },
    Resource { reason: u32 },
    Error { reason: u32 },
}

impl TypedOutcome {
    pub fn canonical_bits(&self) -> Option<String> {
        match self {
            TypedOutcome::Value { kind_code, bits } => Some(canonical_hex(*kind_code, *bits)),
            _ => None,
        }
    }

    pub fn kind_label(&self) -> Option<&'static str> {
        match self {
            TypedOutcome::Value { kind_code, .. } => kind_label_of(*kind_code),
            _ => None,
        }
    }
}

pub fn kind_label_of(code: u32) -> Option<&'static str> {
    Some(match code {
        KIND_I32 => "i32",
        KIND_BOOL => "bool",
        KIND_I64 => "i64",
        KIND_U64 => "u64",
        KIND_F32 => "f32",
        KIND_F64 => "f64",
        _ => return None,
    })
}

/// Canonical hex of a value under its kind code (parent judgement form):
/// i64/u64/f64 → 16 lowercase digits, f32 → 8, i32/bool → unpadded.
pub fn canonical_hex(kind_code: u32, bits: u64) -> String {
    match kind_code {
        KIND_I64 | KIND_U64 | KIND_F64 => format!("{:016x}", bits),
        KIND_F32 => format!("{:08x}", bits as u32),
        KIND_BOOL => format!("{}", bits & 1),
        _ => format!("{:x}", bits as u32),
    }
}

/// Decode the response frame from the exact stdout bytes. Any shape
/// violation (short/long output, bad magic/version/checksum) is a named
/// protocol error — a success-looking fragment never passes.
/// F-891 (§5.3 freeze): a frame is a RESULT only together with a clean
/// process exit — "成功=进程 exit0 且恰一完整合法结果;异常 exit+结果均
/// 不能 PASS". Any other exit state is a named admission rejection, never
/// a Value outcome (late failures cannot pass).
pub fn admit_frame_and_exit(
    stdout: &[u8],
    exit_code: Option<i32>,
) -> Result<(u32, u64, TypedOutcome), MnError> {
    let (sig, nonce, outcome) = decode_response(stdout)?;
    if exit_code != Some(0) {
        return Err(MnError::Admit(
            "typed-frame",
            format!(
                "typed PE exited {exit_code:?} — a response frame without a clean exit0 is not a result (late failure)"
            ),
        ));
    }
    Ok((sig, nonce, outcome))
}

pub fn decode_response(stdout: &[u8]) -> Result<(u32, u64, TypedOutcome), MnError> {
    if stdout.len() != RES_FRAME_BYTES {
        return Err(MnError::Admit(
            "typed-frame",
            format!(
                "response must be exactly {RES_FRAME_BYTES} bytes, got {}",
                stdout.len()
            ),
        ));
    }
    let le32 = |at: usize| {
        u32::from_le_bytes([stdout[at], stdout[at + 1], stdout[at + 2], stdout[at + 3]])
    };
    let le64 = |at: usize| {
        let mut b = [0u8; 8];
        b.copy_from_slice(&stdout[at..at + 8]);
        u64::from_le_bytes(b)
    };
    if le32(0) != RES_MAGIC {
        return Err(MnError::Admit(
            "typed-frame",
            "response magic mismatch".into(),
        ));
    }
    if le32(4) != FRAME_VERSION {
        return Err(MnError::Admit(
            "typed-frame",
            "response version mismatch".into(),
        ));
    }
    if fnv1a64(&stdout[..RES_FRAME_BYTES - FRAME_HASH_BYTES])
        != le64(RES_FRAME_BYTES - FRAME_HASH_BYTES)
    {
        return Err(MnError::Admit(
            "typed-frame",
            "response checksum mismatch".into(),
        ));
    }
    let sig_id = le32(8);
    let nonce = le64(16);
    let outcome = match le32(12) {
        STATUS_VALUE => {
            let kind_code = le32(24);
            if kind_label_of(kind_code).is_none() {
                return Err(MnError::Admit(
                    "typed-frame",
                    format!("unknown value kind code {kind_code}"),
                ));
            }
            TypedOutcome::Value {
                kind_code,
                bits: le64(28),
            }
        }
        STATUS_TRAP => TypedOutcome::Trap { reason: le32(24) },
        STATUS_RESOURCE => TypedOutcome::Resource { reason: le32(24) },
        STATUS_ERROR => TypedOutcome::Error { reason: le32(24) },
        other => {
            return Err(MnError::Admit(
                "typed-frame",
                format!("unknown response status {other}"),
            ))
        }
    };
    Ok((sig_id, nonce, outcome))
}

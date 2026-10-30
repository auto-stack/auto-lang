//! CLIF assembly and COFF emission for MIR units (contract-decision §2/§5):
//! one relocatable object per function unit, external symbols per the
//! logical scheme, imports resolved at link time by the A10' manifest.
//! The target triple is the FROZEN `x86_64-pc-windows-msvc` — never the
//! host triple silently.

use crate::config::*;
use crate::lower;
use crate::symbols;
use crate::MnError;
use ace_av_core::contract::{AV_DATA_PROFILE, AV_OWNED_PROFILE, AV_PROFILE};
use ace_av_core::mir::{MirModule, MirType};
use cranelift_codegen::ir::{AbiParam, InstBuilder, Signature};
use cranelift_codegen::isa::CallConv;
use cranelift_codegen::settings::Configurable;
use cranelift_module::{default_libcall_names, Linkage, Module};
use cranelift_object::{ObjectBuilder, ObjectModule};
use target_lexicon::Triple;

const WINCALL: CallConv = CallConv::WindowsFastcall;

fn triple() -> Result<Triple, MnError> {
    MN_TARGET
        .parse::<Triple>()
        .map_err(|e| MnError::Infra(format!("frozen target triple rejected: {e:?}")))
}

pub(crate) fn make_isa() -> Result<cranelift_codegen::isa::OwnedTargetIsa, MnError> {
    let mut b = cranelift_codegen::settings::builder();
    b.set("opt_level", "none")
        .and_then(|_| b.set("is_pic", "false"))
        .map_err(|e| MnError::Backend(format!("flags: {e}")))?;
    let flags = cranelift_codegen::settings::Flags::new(b);
    let builder = cranelift_codegen::isa::lookup(triple()?)
        .map_err(|e| MnError::Infra(format!("ISA lookup for {MN_TARGET}: {e:?}")))?;
    builder
        .finish(flags)
        .map_err(|e| MnError::Backend(format!("isa finish: {e}")))
}

/// Cranelift signature of a MIR `MirType::Fn` under the private ABI
/// (PLAN-870 freeze §3): i32/bool/char cross as I32 (bool keeps logical
/// 0/1); Str/StrView cross as an (I64 ptr, I32 len) pair; Record and
/// RecordView cross as one I64 address; a Record RESULT crosses through a
/// trailing hidden `*mut` out parameter. Win x64 call conv.
pub fn cranelift_sig_of(module: &MirModule, ty: u32) -> Result<Signature, MnError> {
    let Some(sig) = module.sig_of(ty) else {
        return Err(MnError::Unsupported(
            "fn-sig",
            "type is not a function signature".to_string(),
        ));
    };
    let mut params = Vec::with_capacity(sig.params.len() + 1);
    for p in &sig.params {
        match module.types.get(*p as usize) {
            Some(MirType::I32) | Some(MirType::Bool) | Some(MirType::Char) => {
                params.push(AbiParam::new(cranelift_codegen::ir::types::I32))
            }
            Some(MirType::Str) | Some(MirType::StrView) => {
                params.push(AbiParam::new(cranelift_codegen::ir::types::I64));
                params.push(AbiParam::new(cranelift_codegen::ir::types::I32));
            }
            Some(MirType::Record(_)) | Some(MirType::RecordView(_)) => {
                params.push(AbiParam::new(cranelift_codegen::ir::types::I64));
            }
            None => {
                return Err(MnError::Unsupported(
                    "param",
                    "unknown parameter type is outside the private ABI".to_string(),
                ))
            }
            // Function-typed, wide-numeric, and out-of-scope (R2-N /
            // collections / closures / dynamic) parameter shapes stay a
            // controlled rejection, never implicit support.
            Some(_) => {
                return Err(MnError::Unsupported(
                    "param",
                    "parameter type is outside the mn data/owned ABI".to_string(),
                ))
            }
        }
    }
    let ret = match module.types.get(sig.result as usize) {
        Some(MirType::I32) | Some(MirType::Bool) | Some(MirType::Char) => {
            vec![AbiParam::new(cranelift_codegen::ir::types::I32)]
        }
        Some(MirType::Str) | Some(MirType::StrView) => vec![
            AbiParam::new(cranelift_codegen::ir::types::I64),
            AbiParam::new(cranelift_codegen::ir::types::I32),
        ],
        Some(MirType::Record(_)) | Some(MirType::RecordView(_)) => {
            // Hidden out-aggregate parameter (782 ABI-E shape): the caller
            // passes the destination aggregate's address; no return value.
            params.push(AbiParam::new(cranelift_codegen::ir::types::I64));
            vec![]
        }
        None => {
            return Err(MnError::Unsupported(
                "result",
                "unknown result type is outside the private ABI".to_string(),
            ))
        }
        Some(_) => {
            return Err(MnError::Unsupported(
                "result",
                "result type is outside the mn data/owned ABI".to_string(),
            ))
        }
    };
    Ok(Signature {
        call_conv: WINCALL,
        params,
        returns: ret,
    })
}

fn exit_sig() -> Signature {
    Signature {
        call_conv: WINCALL,
        params: vec![AbiParam::new(cranelift_codegen::ir::types::I32)],
        returns: vec![],
    }
}

/// Admission re-checks at the codegen door (contract §7.3): the input must
/// be a single-function unit of an ADMITTED profile with an admitted
/// import surface. PLAN-870 widens the admitted set to the three shared
/// MIR profiles (scalar + av.data/1 + av.owned/1); everything else stays a
/// controlled rejection. Rejected here with codegen count 0 — never NOP'd
/// past.
pub fn admit_unit(unit: &MirModule) -> Result<(), MnError> {
    let admitted = unit.profile == AV_PROFILE
        || unit.profile == AV_DATA_PROFILE
        || unit.profile == AV_OWNED_PROFILE;
    if !admitted {
        return Err(MnError::Unsupported(
            "profile",
            format!(
                "profile `{}` is not one of `{AV_PROFILE}`/`{AV_DATA_PROFILE}`/`{AV_OWNED_PROFILE}`",
                unit.profile
            ),
        ));
    }
    if !unit.is_unit || unit.funcs.len() != 1 {
        return Err(MnError::Unsupported(
            "unit-shape",
            "codegen consumes single-function units only".to_string(),
        ));
    }
    // Defensive wide-scalar touch check (freeze §4): R2-N stays closed —
    // any REACHED wide numeric type (slot, signature, or record field)
    // rejects by name even though the type table itself may carry dead
    // entries.
    let mut seen = std::collections::BTreeSet::new();
    let f = &unit.funcs[0];
    let mut touched: Vec<u32> = vec![f.sig];
    for s in &f.slots {
        touched.push(s.ty);
    }
    for imp in &unit.imports {
        touched.push(imp.sig);
    }
    for i in &f.blocks {
        for ins in &i.instrs {
            if let ace_av_core::mir::MirInstr::Aggregate { record_ty, .. } = ins {
                touched.push(*record_ty);
            }
        }
    }
    while let Some(t) = touched.pop() {
        if !seen.insert(t) {
            continue;
        }
        match unit.types.get(t as usize) {
            Some(MirType::I64) | Some(MirType::U64) | Some(MirType::F32) | Some(MirType::F64) => {
                return Err(MnError::Unsupported(
                    "type",
                    format!("wide numeric type {t} is outside the mn surface (R2-N)"),
                ));
            }
            Some(MirType::Record(r)) | Some(MirType::RecordView(r)) => {
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
            _ => {}
        }
    }
    for imp in &unit.imports {
        symbols::import_symbol(&unit.owner, &imp.symbol)?;
    }
    // Touched wide CONSTANTS (freeze §4/§7): R2-N stays closed — a Const
    // instruction referencing a wide numeric constant rejects by name
    // (unreferenced dead table entries do not).
    for i in &f.blocks {
        for ins in &i.instrs {
            if let ace_av_core::mir::MirInstr::Const { value, .. } = ins {
                match unit.consts.get(*value as usize) {
                    Some(ace_av_core::mir::MirConst::I64(_))
                    | Some(ace_av_core::mir::MirConst::U64(_))
                    | Some(ace_av_core::mir::MirConst::F32Bits(_))
                    | Some(ace_av_core::mir::MirConst::F64Bits(_)) => {
                        return Err(MnError::Unsupported(
                            "const",
                            format!(
                                "constant {value} is a wide numeric value outside the mn surface (R2-N)"
                            ),
                        ));
                    }
                    _ => {}
                }
            }
        }
    }
    Ok(())
}

/// One emitted unit object: the COFF bytes plus the declared identities
/// the A10' manifest records (export symbol + resolved import symbols).
#[derive(Debug)]
pub struct UnitObject {
    pub symbol: String,
    pub bytes: Vec<u8>,
    /// (MIR import symbol, resolved link symbol) in table order.
    pub imports: Vec<(String, String)>,
}

/// Lower and emit one verified MIR unit to a relocatable COFF object.
pub fn emit_unit_coff(unit: &MirModule) -> Result<UnitObject, MnError> {
    admit_unit(unit)?;
    let f = &unit.funcs[0];
    let isa = make_isa()?;
    let obj_name = format!("mn8-{}", f.name);
    let builder = ObjectBuilder::new(isa, obj_name, default_libcall_names())
        .map_err(|e| MnError::Backend(format!("object builder: {e}")))?;
    let mut om = ObjectModule::new(builder);

    let exit_id = om
        .declare_function("ExitProcess", Linkage::Import, &exit_sig())
        .map_err(|e| MnError::Backend(format!("declare ExitProcess: {e}")))?;

    // Imports first: logical symbols per the scheme.
    let mut import_ids = Vec::with_capacity(unit.imports.len());
    let mut import_syms = Vec::with_capacity(unit.imports.len());
    for imp in &unit.imports {
        let logical = symbols::import_symbol(&unit.owner, &imp.symbol)?;
        let sig = cranelift_sig_of(unit, imp.sig)?;
        let id = om
            .declare_function(&logical, Linkage::Import, &sig)
            .map_err(|e| MnError::Backend(format!("declare import {}: {e}", logical)))?;
        import_ids.push(id);
        import_syms.push((imp.symbol.clone(), logical));
    }
    // A bare Trace instruction without a matching table import still needs
    // its runtime symbol (verify admits trace via capability; the unit
    // split copies only declared imports).
    let mut trace_ids: Vec<(String, cranelift_module::FuncId)> = Vec::new();
    for sym in ["hir.test.mark_a", "hir.test.mark_b"] {
        let uses = f.blocks.iter().any(|b| {
            b.instrs
                .iter()
                .any(|i| matches!(i, ace_av_core::mir::MirInstr::Trace { event } if event_symbol(*event) == sym))
        });
        if uses && !unit.imports.iter().any(|i| i.symbol == sym) {
            let sig = trace_sig();
            let id = om
                .declare_function(sym, Linkage::Import, &sig)
                .map_err(|e| MnError::Backend(format!("declare {sym}: {e}")))?;
            trace_ids.push((sym.to_string(), id));
        }
    }

    let fn_sig = cranelift_sig_of(unit, f.sig)?;
    let symbol = symbols::function_symbol(&unit.owner, &f.name);
    let fid = om
        .declare_function(&symbol, Linkage::Export, &fn_sig)
        .map_err(|e| MnError::Backend(format!("declare {symbol}: {e}")))?;

    let mut ctx = om.make_context();
    ctx.func.signature = fn_sig;
    lower::emit_function(
        unit,
        &mut om,
        fid,
        &import_ids,
        &trace_ids,
        exit_id,
        &mut ctx,
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
                "COFF for `{}` is {} bytes, above the frozen {} cap",
                f.name,
                bytes.len(),
                MAX_COFF_BYTES
            ),
        ));
    }
    Ok(UnitObject {
        symbol,
        bytes,
        imports: import_syms,
    })
}

fn trace_sig() -> Signature {
    Signature {
        call_conv: WINCALL,
        params: vec![],
        returns: vec![AbiParam::new(cranelift_codegen::ir::types::I32)],
    }
}

pub(crate) fn event_symbol(e: ace_av_core::mir::TraceEvent) -> &'static str {
    match e {
        ace_av_core::mir::TraceEvent::MarkA => "hir.test.mark_a",
        ace_av_core::mir::TraceEvent::MarkB => "hir.test.mark_b",
    }
}

/// The startup wrapper object: `ac_start` calls the entry function and
/// forwards its i32 result BIT-EXACT to `ExitProcess` (observers compare
/// the full 32-bit exit code). Emitted per request — it is not a cached
/// unit artifact (contract-decision §8 note).
pub fn emit_wrapper_coff(entry_symbol: &str) -> Result<Vec<u8>, MnError> {
    let isa = make_isa()?;
    let builder = ObjectBuilder::new(isa, "mn8-ac-start", default_libcall_names())
        .map_err(|e| MnError::Backend(format!("object builder: {e}")))?;
    let mut om = ObjectModule::new(builder);
    let entry_sig = trace_sig(); // zero-param i32 — the frozen entry contract
    let entry_id = om
        .declare_function(entry_symbol, Linkage::Import, &entry_sig)
        .map_err(|e| MnError::Backend(format!("declare entry {entry_symbol}: {e}")))?;
    let exit_id = om
        .declare_function("ExitProcess", Linkage::Import, &exit_sig())
        .map_err(|e| MnError::Backend(format!("declare ExitProcess: {e}")))?;
    let start_sig = Signature {
        call_conv: WINCALL,
        params: vec![],
        returns: vec![],
    };
    let start_id = om
        .declare_function(START_SYMBOL, Linkage::Export, &start_sig)
        .map_err(|e| MnError::Backend(format!("declare {}: {e}", START_SYMBOL)))?;
    let mut ctx = om.make_context();
    ctx.func.signature = start_sig;
    {
        let mut fb_ctx = cranelift_frontend::FunctionBuilderContext::new();
        let mut fb = cranelift_frontend::FunctionBuilder::new(&mut ctx.func, &mut fb_ctx);
        let blk = fb.create_block();
        fb.switch_to_block(blk);
        fb.seal_block(blk);
        let entry_ref = om.declare_func_in_func(entry_id, &mut fb.func);
        let exit_ref = om.declare_func_in_func(exit_id, &mut fb.func);
        let call = fb.ins().call(entry_ref, &[]);
        let result = fb.inst_results(call)[0];
        fb.ins().call(exit_ref, &[result]);
        fb.ins().return_(&[]);
        fb.finalize();
    }
    om.define_function(start_id, &mut ctx)
        .map_err(|e| MnError::Backend(format!("define {}: {e}", START_SYMBOL)))?;
    om.finish()
        .emit()
        .map_err(|e| MnError::Backend(format!("emit wrapper: {e}")))
}

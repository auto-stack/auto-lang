//! CLIF assembly and COFF emission for MIR units (contract-decision §2/§5):
//! one relocatable object per function unit, external symbols per the
//! logical scheme, imports resolved at link time by the A10' manifest.
//! The target triple is the FROZEN `x86_64-pc-windows-msvc` — never the
//! host triple silently.

use crate::config::*;
use crate::lower;
use crate::symbols;
use crate::MnError;
use auto_av_core::contract::AV_PROFILE;
use auto_av_core::mir::{MirModule, ScalarShape};
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

fn make_isa() -> Result<cranelift_codegen::isa::OwnedTargetIsa, MnError> {
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

/// Cranelift signature of a MIR `MirType::Fn` under the private ABI:
/// i32/bool both cross as I32 (bool keeps logical 0/1), Win x64 call conv.
pub fn cranelift_sig_of(module: &MirModule, ty: u32) -> Result<Signature, MnError> {
    let Some(sig) = module.sig_of(ty) else {
        return Err(MnError::Unsupported(
            "fn-sig",
            "type is not a function signature".to_string(),
        ));
    };
    let mut params = Vec::with_capacity(sig.params.len());
    for p in &sig.params {
        match module.type_shape(*p) {
            Some(ScalarShape::I32) | Some(ScalarShape::Bool) => {
                params.push(AbiParam::new(cranelift_codegen::ir::types::I32))
            }
            None => {
                return Err(MnError::Unsupported(
                    "param",
                    "function-typed parameters are outside the private ABI".to_string(),
                ))
            }
            // PLAN-820 shared-core widening (av.data/1): the wide scalar and
            // record shapes are outside mn's frozen scalar ABI — a
            // controlled rejection, never implicit support (F820-R10-01
            // class; 818 owner reviews semantics at its own merge).
            Some(_) => {
                return Err(MnError::Unsupported(
                    "param",
                    "av.data/1 wide/record params are outside the mn scalar ABI".to_string(),
                ))
            }
        }
    }
    let ret = match module.type_shape(sig.result) {
        Some(_) => vec![AbiParam::new(cranelift_codegen::ir::types::I32)],
        None => {
            return Err(MnError::Unsupported(
                "result",
                "function-typed results are outside the private ABI".to_string(),
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
/// be a single-function unit of the one profile with an admitted import
/// surface. Rejected here with codegen count 0 — never NOP'd past.
pub fn admit_unit(unit: &MirModule) -> Result<(), MnError> {
    if unit.profile != AV_PROFILE {
        return Err(MnError::Unsupported(
            "profile",
            format!("profile `{}` is not `{AV_PROFILE}`", unit.profile),
        ));
    }
    if !unit.is_unit || unit.funcs.len() != 1 {
        return Err(MnError::Unsupported(
            "unit-shape",
            "codegen consumes single-function units only".to_string(),
        ));
    }
    for imp in &unit.imports {
        symbols::import_symbol(&unit.owner, &imp.symbol)?;
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
                .any(|i| matches!(i, auto_av_core::mir::MirInstr::Trace { event } if event_symbol(*event) == sym))
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

pub(crate) fn event_symbol(e: auto_av_core::mir::TraceEvent) -> &'static str {
    match e {
        auto_av_core::mir::TraceEvent::MarkA => "hir.test.mark_a",
        auto_av_core::mir::TraceEvent::MarkB => "hir.test.mark_b",
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

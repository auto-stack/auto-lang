//! Cranelift emitter for AutoABI-side objects (plan §5.2).
//!
//! Every module is emitted from the declaration IR in [`crate::decl`] —
//! there is no Auto-source parser in this path and reports must not
//! claim one. The callee and caller objects are produced by SEPARATE
//! `ObjectModule` instances with separate outputs: the AutoABI<->AutoABI
//! leg is two independently emitted objects linked together, never one
//! codegen unit testing itself.
//!
//! Calling convention: Windows x64 (`WindowsFastcall`), same Cranelift
//! stack as ac-core (0.126). The mixed i32/i64/f64 signature of
//! `p797_mix` exercises the register banks deliberately: 4 integer args
//! in RCX/RDX/R8/R9, the 5th/6th integer args on the stack, 2 fp args in
//! XMM0/XMM1 — any misclassification between producers changes the
//! result because every parameter carries a distinct weight.

use crate::decl::{AbiScalar, BodyKind, FnDecl, MixInputs, P797_MIX};

use cranelift_codegen::ir::condcodes::IntCC;
use cranelift_codegen::ir::types;
use cranelift_codegen::ir::{AbiParam, InstBuilder, Signature};
use cranelift_codegen::isa::CallConv;
use cranelift_codegen::settings::Configurable;
use cranelift_codegen::settings::{builder as flags_builder, Flags};
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext};
use cranelift_module::{Linkage, Module};
use cranelift_object::{ObjectBuilder, ObjectModule};
use target_lexicon::Triple;

const WINCALL: CallConv = CallConv::WindowsFastcall;
const ENTRY_SYMBOL: &str = "p797_entry";

fn cranelift_type(s: AbiScalar) -> types::Type {
    match s {
        AbiScalar::I32 => types::I32,
        AbiScalar::I64 => types::I64,
        AbiScalar::F64 => types::F64,
    }
}

fn mix_signature(decl: &FnDecl) -> Signature {
    Signature {
        call_conv: WINCALL,
        params: decl.params.iter().map(|p| abi_scalar_param(*p)).collect(),
        returns: vec![abi_scalar_param(decl.ret)],
    }
}

fn abi_scalar_param(s: AbiScalar) -> AbiParam {
    AbiParam::new(cranelift_type(s))
}

fn new_module(name: &str) -> Result<ObjectModule, String> {
    let triple = Triple::host();
    let mut fb = flags_builder();
    fb.set("opt_level", "none").map_err(|e| e.to_string())?;
    fb.set("is_pic", "false").map_err(|e| e.to_string())?;
    let flags = Flags::new(fb);
    let isa = cranelift_codegen::isa::lookup(triple)
        .map_err(|e| format!("isa lookup: {e}"))?
        .finish(flags)
        .map_err(|e| format!("isa finish: {e}"))?;
    let builder = ObjectBuilder::new(
        isa,
        name,
        cranelift_module::default_libcall_names(),
    )
    .map_err(|e| format!("object builder: {e}"))?;
    Ok(ObjectModule::new(builder))
}

/// Lower the frozen `MixV1` body over the call arguments. Weights are
/// distinct per parameter position (see module doc): the body is a THIRD
/// independent implementation next to the Rust oracle (`decl.rs`) and the
/// hand-written C bodies (`cside/`).
fn sext_i32_to_i64(
    fb: &mut FunctionBuilder,
    v: cranelift_codegen::ir::Value,
) -> cranelift_codegen::ir::Value {
    fb.ins().sextend(types::I64, v)
}

fn weighted_i64(
    fb: &mut FunctionBuilder,
    v: cranelift_codegen::ir::Value,
    w: i64,
) -> cranelift_codegen::ir::Value {
    let c = fb.ins().iconst(types::I64, w);
    fb.ins().imul(v, c)
}

fn emit_mix_v1_body(
    fb: &mut FunctionBuilder,
    args: &[cranelift_codegen::ir::Value],
) -> cranelift_codegen::ir::Value {
    // MixV1 is bound to the frozen signature; the i32 positions are
    // [0,4) and 6, the i64 is 4, the f64s are 5 and 7.
    // f64 -> i64: frozen profile rule is truncation toward zero with
    // in-range inputs (fcvt_to_sint traps only on out-of-range/NaN,
    // which the frozen fixtures never supply).
    let e2 = weighted_i64(fb, args[4], 2);
    let a = sext_i32_to_i64(fb, args[0]);
    let a1 = weighted_i64(fb, a, 1);
    let b = sext_i32_to_i64(fb, args[1]);
    let b3 = weighted_i64(fb, b, 3);
    let c = sext_i32_to_i64(fb, args[2]);
    let c7 = weighted_i64(fb, c, 7);
    let d = sext_i32_to_i64(fb, args[3]);
    let d11 = weighted_i64(fb, d, 11);
    let g = sext_i32_to_i64(fb, args[6]);
    let g13 = weighted_i64(fb, g, 13);
    let fi = fb.ins().fcvt_to_sint(types::I64, args[5]);
    let f1000 = weighted_i64(fb, fi, 1000);
    let hi = fb.ins().fcvt_to_sint(types::I64, args[7]);
    let h17 = weighted_i64(fb, hi, 17);

    let acc = fb.ins().iadd(e2, a1);
    let acc = fb.ins().iadd(acc, b3);
    let acc = fb.ins().iadd(acc, c7);
    let acc = fb.ins().iadd(acc, d11);
    let acc = fb.ins().iadd(acc, g13);
    let acc = fb.ins().iadd(acc, f1000);
    fb.ins().iadd(acc, h17)
}

/// Emit the CALLEE object: defines `p797_mix` from the declaration IR.
/// Exported with a plain C-identifier symbol — the frozen v0.1 candidate
/// symbol scheme keeps public names C-compatible on purpose (see
/// contract-decision.md §2).
pub fn emit_callee(decl: &FnDecl) -> Result<Vec<u8>, String> {
    let mut om = new_module("p797_abi_callee")?;
    let sig = mix_signature(decl);
    let fid = om
        .declare_function(decl.symbol, Linkage::Export, &sig)
        .map_err(|e| format!("declare {}: {e}", decl.symbol))?;
    let mut ctx = om.make_context();
    ctx.func.signature = sig;
    {
        let mut fb_ctx = FunctionBuilderContext::new();
        let mut fb = FunctionBuilder::new(&mut ctx.func, &mut fb_ctx);
        let entry = fb.create_block();
        fb.append_block_params_for_function_params(entry);
        fb.switch_to_block(entry);
        fb.seal_block(entry);
        let params = fb.block_params(entry).to_vec();
        let result = match decl.body {
            BodyKind::MixV1 => emit_mix_v1_body(&mut fb, &params),
        };
        fb.ins().return_(&[result]);
        fb.finalize();
    }
    om.define_function(fid, &mut ctx)
        .map_err(|e| format!("define {}: {e}", decl.symbol))?;
    om.finish().emit().map_err(|e| format!("emit callee: {e}"))
}

/// Emit the CALLER object: imports `p797_mix`, calls it with the frozen
/// inputs, compares against `expected` (computed by the INDEPENDENT Rust
/// oracle and baked in as a constant — the PE carries the check itself),
/// and exits with 0 on match / 1 on mismatch. A wrong `expected` or a
/// garbled call both surface as a nonzero exit code.
pub fn emit_caller(decl: &FnDecl, inputs: &MixInputs, expected: i64) -> Result<Vec<u8>, String> {
    let mut om = new_module("p797_abi_caller")?;

    // Import the callee surface.
    let sig = mix_signature(decl);
    let callee = om
        .declare_function(decl.symbol, Linkage::Import, &sig)
        .map_err(|e| format!("declare import {}: {e}", decl.symbol))?;

    // ExitProcess(u32) from kernel32.
    let exit_sig = Signature {
        call_conv: WINCALL,
        params: vec![AbiParam::new(types::I32)],
        returns: vec![],
    };
    let exit_id = om
        .declare_function("ExitProcess", Linkage::Import, &exit_sig)
        .map_err(|e| format!("declare ExitProcess: {e}"))?;

    // p797_main() -> i32 violation flag.
    let main_sig = Signature {
        call_conv: WINCALL,
        params: vec![],
        returns: vec![AbiParam::new(types::I32)],
    };
    let main_id = om
        .declare_function("p797_main", Linkage::Export, &main_sig)
        .map_err(|e| format!("declare p797_main: {e}"))?;
    let mut ctx = om.make_context();
    ctx.func.signature = main_sig;
    {
        let mut fb_ctx = FunctionBuilderContext::new();
        let mut fb = FunctionBuilder::new(&mut ctx.func, &mut fb_ctx);
        let callee_ref = om.declare_func_in_func(callee, &mut fb.func);
        let blk = fb.create_block();
        fb.switch_to_block(blk);
        fb.seal_block(blk);

        let a = fb.ins().iconst(types::I32, inputs.a as i64);
        let b = fb.ins().iconst(types::I32, inputs.b as i64);
        let c = fb.ins().iconst(types::I32, inputs.c as i64);
        let d = fb.ins().iconst(types::I32, inputs.d as i64);
        let e = fb.ins().iconst(types::I64, inputs.e);
        let f = fb.ins().f64const(inputs.f);
        let g = fb.ins().iconst(types::I32, inputs.g as i64);
        let h = fb.ins().f64const(inputs.h);
        let call = fb.ins().call(callee_ref, &[a, b, c, d, e, f, g, h]);
        let got = fb.inst_results(call)[0];

        let want = fb.ins().iconst(types::I64, expected);
        let ne = fb.ins().icmp(IntCC::NotEqual, got, want);
        let flag = fb.ins().uextend(types::I32, ne);
        fb.ins().return_(&[flag]);
        fb.finalize();
    }
    om.define_function(main_id, &mut ctx)
        .map_err(|e| format!("define p797_main: {e}"))?;

    // p797_entry() -> ExitProcess(p797_main()) — the entry is a generic
    // wrapper, never the probe function itself (ac-core convention).
    let entry_sig = Signature {
        call_conv: WINCALL,
        params: vec![],
        returns: vec![],
    };
    let entry_id = om
        .declare_function(ENTRY_SYMBOL, Linkage::Export, &entry_sig)
        .map_err(|e| format!("declare {ENTRY_SYMBOL}: {e}"))?;
    let mut ctx = om.make_context();
    ctx.func.signature = entry_sig;
    {
        let mut fb_ctx = FunctionBuilderContext::new();
        let mut fb = FunctionBuilder::new(&mut ctx.func, &mut fb_ctx);
        let main_ref = om.declare_func_in_func(main_id, &mut fb.func);
        let exit_ref = om.declare_func_in_func(exit_id, &mut fb.func);
        let blk = fb.create_block();
        fb.switch_to_block(blk);
        fb.seal_block(blk);
        let call = fb.ins().call(main_ref, &[]);
        let result = fb.inst_results(call)[0];
        fb.ins().call(exit_ref, &[result]);
        fb.ins().return_(&[]);
        fb.finalize();
    }
    om.define_function(entry_id, &mut ctx)
        .map_err(|e| format!("define {ENTRY_SYMBOL}: {e}"))?;
    om.finish().emit().map_err(|e| format!("emit caller: {e}"))
}

/// Convenience: the T-01 probe declaration.
pub fn t01_mix_decl() -> &'static FnDecl {
    &P797_MIX
}

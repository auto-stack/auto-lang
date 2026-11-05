//! PLAN-937 CA-D1 (T-04): the Cranelift emitter for the TYPED public
//! lanes (`i32/u32/i64/u64/f64/bool/char` — no f32).
//!
//! Same Cranelift stack and calling convention as the 797 probe
//! (Windows x64 `WindowsFastcall`, POSITIONAL slots — see
//! foreign-contract `plan::ARG_SLOTS`), but signatures come from the
//! NEUTRAL foreign-contract lanes instead of the 797 `AbiScalar` set,
//! and the business bodies come from the frozen typed families in
//! [`crate::typed_decl`]. The f64 lane folds by RAW BITS (stack-slot
//! round-trip — `raw_bitcast` does not exist in this Cranelift), so
//! quiet-NaN payloads and -0.0 survive bit-exactly into the checksum.
//!
//! Callee and caller are SEPARATE ObjectModule instances (never one
//! codegen unit testing itself); the independent implementations are
//! the Rust oracle (`typed_decl.rs`) and the hand-written freestanding C
//! (`cside/bridge_typed_callee.c` / `bridge_typed_caller.c`).

use crate::decl::status;
use crate::typed_decl::{t937_status, TypedInputs, T937_MIX_C};
use ace_foreign_contract::lane::LaneKind;

use cranelift_codegen::ir::condcodes::IntCC;
use cranelift_codegen::ir::types;
use cranelift_codegen::ir::{
    AbiParam, InstBuilder, MemFlags, Signature, StackSlotData, StackSlotKind, Value,
};
use cranelift_codegen::isa::CallConv;
use cranelift_codegen::settings::builder as flags_builder;
use cranelift_codegen::settings::Configurable;
use cranelift_codegen::settings::Flags;
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext};
use cranelift_module::{Linkage, Module};
use cranelift_object::{ObjectBuilder, ObjectModule};
use target_lexicon::Triple;

const WINCALL: CallConv = CallConv::WindowsFastcall;
const ENTRY_SYMBOL: &str = "t937_entry";

/// Cranelift machine type of a typed public lane.
fn typed_param_type(k: LaneKind) -> types::Type {
    match k {
        LaneKind::I32 | LaneKind::U32 | LaneKind::Char => types::I32,
        LaneKind::I64 | LaneKind::U64 => types::I64,
        LaneKind::F64 => types::F64,
        LaneKind::Bool => types::I8,
    }
}

fn mix_signature(params: &[LaneKind], ret: types::Type) -> Signature {
    Signature {
        call_conv: WINCALL,
        params: params
            .iter()
            .map(|p| AbiParam::new(typed_param_type(*p)))
            .collect(),
        returns: vec![AbiParam::new(ret)],
    }
}

fn div_signature() -> Signature {
    // (ctx ptr-width, n i64, d i64, out* i64) -> i32 status
    Signature {
        call_conv: WINCALL,
        params: vec![
            AbiParam::new(types::I64),
            AbiParam::new(types::I64),
            AbiParam::new(types::I64),
            AbiParam::new(types::I64),
        ],
        returns: vec![AbiParam::new(types::I32)],
    }
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
    let builder = ObjectBuilder::new(isa, name, cranelift_module::default_libcall_names())
        .map_err(|e| format!("object builder: {e}"))?;
    Ok(ObjectModule::new(builder))
}

/// Reinterpret an f64 Value's RAW BITS as i64 (stack-slot round-trip;
/// payload-preserving).
fn f64_bits(fb: &mut FunctionBuilder, v: Value) -> Value {
    let slot = fb.create_sized_stack_slot(StackSlotData::new(StackSlotKind::ExplicitSlot, 8, 3));
    fb.ins().stack_store(v, slot, 0);
    fb.ins().stack_load(types::I64, slot, 0)
}

/// Widen a typed lane value to its i64 bit pattern for the checksum
/// fold: i32 SIGN-extends (the value's arithmetic width), u32/char/bool
/// ZERO-extend, i64/u64 pass bits, f64 folds raw bits.
fn widen_to_i64(fb: &mut FunctionBuilder, k: LaneKind, v: Value) -> Value {
    match k {
        LaneKind::I32 => fb.ins().sextend(types::I64, v),
        LaneKind::U32 | LaneKind::Char => fb.ins().uextend(types::I64, v),
        LaneKind::Bool => fb.ins().uextend(types::I64, v),
        LaneKind::I64 | LaneKind::U64 => v,
        LaneKind::F64 => f64_bits(fb, v),
    }
}

/// The frozen ChecksumV1 body over the typed lanes (mirrors
/// `typed_decl::oracle_typed_mix` and `cside/bridge_typed_callee.c`).
fn emit_typed_mix_body(
    fb: &mut FunctionBuilder,
    lanes: &[LaneKind],
    args: &[Value],
) -> Result<Value, String> {
    if lanes.len() != 8 || args.len() != 8 {
        return Err("ChecksumV1 binds exactly 8 lanes".into());
    }
    let c0 = fb.ins().iconst(types::I64, T937_MIX_C[0] as i64);
    let c1 = fb.ins().iconst(types::I64, T937_MIX_C[1] as i64);
    let c2 = fb.ins().iconst(types::I64, T937_MIX_C[2] as i64);
    let c3 = fb.ins().iconst(types::I64, T937_MIX_C[3] as i64);
    let c4 = fb.ins().iconst(types::I64, T937_MIX_C[4] as i64);
    let c5 = fb.ins().iconst(types::I64, T937_MIX_C[5] as i64);
    let c6 = fb.ins().iconst(types::I64, T937_MIX_C[6] as i64);
    let c7 = fb.ins().iconst(types::I64, T937_MIX_C[7] as i64);
    let w0 = widen_to_i64(fb, lanes[0], args[0]);
    let w1 = widen_to_i64(fb, lanes[1], args[1]);
    let w2 = widen_to_i64(fb, lanes[2], args[2]);
    let w3 = widen_to_i64(fb, lanes[3], args[3]);
    let w4 = widen_to_i64(fb, lanes[4], args[4]);
    let w5 = widen_to_i64(fb, lanes[5], args[5]);
    let w6 = widen_to_i64(fb, lanes[6], args[6]);
    let w7 = widen_to_i64(fb, lanes[7], args[7]);
    let wa = fb.ins().imul(w0, c0);
    let wb = fb.ins().imul(w1, c1);
    let wc = fb.ins().imul(w2, c2);
    let wd = fb.ins().imul(w3, c3);
    let wf = fb.ins().imul(w4, c4);
    let wg = fb.ins().imul(w5, c5);
    let wh = fb.ins().imul(w6, c6);
    let wi = fb.ins().imul(w7, c7);

    let acc = fb.ins().bxor(wa, wb);
    let acc = fb.ins().iadd(acc, wc);
    let acc = fb.ins().bxor(acc, wd);
    let acc = fb.ins().iadd(acc, wf);
    let acc = fb.ins().bxor(acc, wg);
    let acc = fb.ins().iadd(acc, wh);
    Ok(fb.ins().bxor(acc, wi))
}

/// Emit the TYPED callee object: `t937_mix` (pure checksum) and
/// `t937_div` (fallible: d==0 -> status -1, out UNTOUCHED; else wrapping
/// trunc quotient, status 0).
pub fn emit_typed_callee() -> Result<Vec<u8>, String> {
    let mut om = new_module("t937_abi_callee")?;
    let decls = crate::typed_decl::typed_decls();

    // t937_mix
    let mix = &decls.decls[0];
    let mix_sig = mix_signature(&mix.params, types::I64);
    let mix_id = om
        .declare_function("t937_mix", Linkage::Export, &mix_sig)
        .map_err(|e| format!("declare t937_mix: {e}"))?;
    let mut ctx = om.make_context();
    ctx.func.signature = mix_sig;
    {
        let mut fb_ctx = FunctionBuilderContext::new();
        let mut fb = FunctionBuilder::new(&mut ctx.func, &mut fb_ctx);
        let entry = fb.create_block();
        fb.append_block_params_for_function_params(entry);
        fb.switch_to_block(entry);
        fb.seal_block(entry);
        let params = fb.block_params(entry).to_vec();
        let result = emit_typed_mix_body(&mut fb, &mix.params, &params)?;
        fb.ins().return_(&[result]);
        fb.finalize();
    }
    om.define_function(mix_id, &mut ctx)
        .map_err(|e| format!("define t937_mix: {e}"))?;

    // t937_div
    let div_sig = div_signature();
    let div_id = om
        .declare_function("t937_div", Linkage::Export, &div_sig)
        .map_err(|e| format!("declare t937_div: {e}"))?;
    let mut ctx = om.make_context();
    ctx.func.signature = div_sig;
    {
        let mut fb_ctx = FunctionBuilderContext::new();
        let mut fb = FunctionBuilder::new(&mut ctx.func, &mut fb_ctx);
        let entry = fb.create_block();
        fb.append_block_params_for_function_params(entry);
        fb.switch_to_block(entry);
        // (ctx, n, d, out)
        let ctx_v = fb.block_params(entry)[0];
        let n = fb.block_params(entry)[1];
        let d = fb.block_params(entry)[2];
        let out = fb.block_params(entry)[3];
        let _ = ctx_v; // nullable ctx — never dereferenced

        let zero = fb.ins().iconst(types::I32, 0);
        let zero64 = fb.ins().iconst(types::I64, 0);
        let dzero = fb.ins().icmp(IntCC::Equal, d, zero64);
        let fail = fb.create_block();
        let body = fb.create_block();
        fb.seal_block(entry); // entry's successors are now known
        fb.ins().brif(dzero, fail, &[], body, &[]);

        fb.switch_to_block(fail);
        fb.seal_block(fail);
        let err = fb
            .ins()
            .iconst(types::I32, t937_status::ERR_DIV_ZERO as i64);
        fb.ins().return_(&[err]);

        fb.switch_to_block(body);
        fb.seal_block(body);
        // wrapping div without branches: when d == -1 divide by a SAFE
        // divisor (1) and select the wrapping form 0 - n (0 - MIN == MIN);
        // the sdiv trap can never execute.
        let neg1 = fb.ins().iconst(types::I64, -1);
        let one = fb.ins().iconst(types::I64, 1);
        let dm1 = fb.ins().icmp(IntCC::Equal, d, neg1);
        let d_safe = fb.ins().select(dm1, one, d);
        let q_div = fb.ins().sdiv(n, d_safe);
        let q_neg = fb.ins().isub(zero64, n);
        let q = fb.ins().select(dm1, q_neg, q_div);
        fb.ins().store(MemFlags::new(), q, out, 0);
        fb.ins().return_(&[zero]);
        fb.finalize();
    }
    om.define_function(div_id, &mut ctx)
        .map_err(|e| format!("define t937_div: {e}"))?;
    om.finish()
        .emit()
        .map_err(|e| format!("emit typed callee: {e}"))
}

/// One in-PE typed check. The PE exits with the number of FAILED
/// checks (0 = all passed).
pub enum TypedCheck {
    /// Call t937_mix with `inputs`, compare against `expected` (from the
    /// independent Rust oracle / hand-computed literals).
    Mix { inputs: TypedInputs, expected: i64 },
    /// Call t937_div(n, d, &out) with `out` pre-set to `sentinel`:
    /// Ok(q) requires status 0 AND out == q; Err(st) requires status st
    /// AND out UNTOUCHED (still the sentinel).
    Div {
        n: i64,
        d: i64,
        expect: Result<i64, i32>,
        sentinel: i64,
    },
}

/// Emit the TYPED caller object: imports `t937_mix`/`t937_div`, runs the
/// checks, exits with the violation count via ExitProcess.
pub fn emit_typed_caller(checks: &[TypedCheck]) -> Result<Vec<u8>, String> {
    let mut om = new_module("t937_abi_caller")?;

    let mix_sig = mix_signature(
        &[
            LaneKind::I32,
            LaneKind::U32,
            LaneKind::I64,
            LaneKind::U64,
            LaneKind::F64,
            LaneKind::Bool,
            LaneKind::Char,
            LaneKind::I64,
        ],
        types::I64,
    );
    let mix_callee = om
        .declare_function("t937_mix", Linkage::Import, &mix_sig)
        .map_err(|e| format!("declare import t937_mix: {e}"))?;
    let div_callee = om
        .declare_function("t937_div", Linkage::Import, &div_signature())
        .map_err(|e| format!("declare import t937_div: {e}"))?;

    let exit_sig = Signature {
        call_conv: WINCALL,
        params: vec![AbiParam::new(types::I32)],
        returns: vec![],
    };
    let exit_id = om
        .declare_function("ExitProcess", Linkage::Import, &exit_sig)
        .map_err(|e| format!("declare ExitProcess: {e}"))?;

    let main_sig = Signature {
        call_conv: WINCALL,
        params: vec![],
        returns: vec![AbiParam::new(types::I32)],
    };
    let main_id = om
        .declare_function("t937_main", Linkage::Export, &main_sig)
        .map_err(|e| format!("declare t937_main: {e}"))?;
    let mut ctx = om.make_context();
    ctx.func.signature = main_sig;
    {
        let mut fb_ctx = FunctionBuilderContext::new();
        let mut fb = FunctionBuilder::new(&mut ctx.func, &mut fb_ctx);
        let mix_ref = om.declare_func_in_func(mix_callee, &mut fb.func);
        let div_ref = om.declare_func_in_func(div_callee, &mut fb.func);

        let blk = fb.create_block();
        fb.switch_to_block(blk);
        fb.seal_block(blk);

        let viol = fb.declare_var(types::I32);
        let zero_viol = fb.ins().iconst(types::I32, 0);
        fb.def_var(viol, zero_viol);

        for check in checks {
            match check {
                TypedCheck::Mix { inputs, expected } => {
                    let x = inputs;
                    let a = fb.ins().iconst(types::I32, x.a as i64);
                    let b = fb.ins().iconst(types::I32, x.b as i32 as i64);
                    let c = fb.ins().iconst(types::I64, x.c);
                    let d = fb.ins().iconst(types::I64, x.d as i64);
                    let f = fb.ins().f64const(f64::from_bits(x.f_bits));
                    let g = fb.ins().iconst(types::I8, x.g as i64);
                    let h = fb.ins().iconst(types::I32, x.h as i32 as i64);
                    let i = fb.ins().iconst(types::I64, x.i);
                    let call = fb.ins().call(mix_ref, &[a, b, c, d, f, g, h, i]);
                    let got = fb.inst_results(call)[0];
                    let want = fb.ins().iconst(types::I64, *expected);
                    let ne = fb.ins().icmp(IntCC::NotEqual, got, want);
                    let flag = fb.ins().uextend(types::I32, ne);
                    let cur = fb.use_var(viol);
                    let acc = fb.ins().iadd(cur, flag);
                    fb.def_var(viol, acc);
                }
                TypedCheck::Div {
                    n,
                    d,
                    expect,
                    sentinel,
                } => {
                    let slot = fb.create_sized_stack_slot(StackSlotData::new(
                        StackSlotKind::ExplicitSlot,
                        8,
                        3,
                    ));
                    let sent_v = fb.ins().iconst(types::I64, *sentinel);
                    fb.ins().stack_store(sent_v, slot, 0);
                    let out = fb.ins().stack_addr(types::I64, slot, 0);
                    let ctxp = fb.ins().iconst(types::I64, 0); // nullable ctx = null
                    let cvn = fb.ins().iconst(types::I64, *n);
                    let cvd = fb.ins().iconst(types::I64, *d);
                    let call = fb.ins().call(div_ref, &[ctxp, cvn, cvd, out]);
                    let st = fb.inst_results(call)[0];
                    let after = fb.ins().stack_load(types::I64, slot, 0);
                    let (want_st, want_out) = match expect {
                        Ok(q) => (0i64, *q),
                        Err(s) => (*s as i64, *sentinel),
                    };
                    let want_st_v = fb.ins().iconst(types::I32, want_st);
                    let st_bad = fb.ins().icmp(IntCC::NotEqual, st, want_st_v);
                    let want_out_v = fb.ins().iconst(types::I64, want_out);
                    let out_bad = fb.ins().icmp(IntCC::NotEqual, after, want_out_v);
                    let f1 = fb.ins().uextend(types::I32, st_bad);
                    let f2 = fb.ins().uextend(types::I32, out_bad);
                    let f12 = fb.ins().iadd(f1, f2);
                    let cur = fb.use_var(viol);
                    let acc = fb.ins().iadd(cur, f12);
                    fb.def_var(viol, acc);
                }
            }
        }
        let total = fb.use_var(viol);
        fb.ins().return_(&[total]);
        fb.finalize();
    }
    om.define_function(main_id, &mut ctx)
        .map_err(|e| format!("define t937_main: {e}"))?;

    // t937_entry() -> ExitProcess(t937_main()) — generic wrapper entry.
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
    om.finish()
        .emit()
        .map_err(|e| format!("emit typed caller: {e}"))
}

/// The frozen typed check set (V-03): extremes, NaN payload, -0.0,
/// bool both values, char boundaries, wrapping div, controlled failure.
pub fn frozen_typed_checks() -> Vec<TypedCheck> {
    use crate::typed_decl::{oracle_typed_div, oracle_typed_mix, FROZEN_TYPED_INPUTS};
    let neg0 = TypedInputs {
        f_bits: (-0.0f64).to_bits(),
        ..FROZEN_TYPED_INPUTS
    };
    let bool_f = TypedInputs {
        g: false,
        ..FROZEN_TYPED_INPUTS
    };
    let char_min = TypedInputs {
        h: 0,
        ..FROZEN_TYPED_INPUTS
    };
    let zeros = TypedInputs {
        a: 0,
        b: 0,
        c: 0,
        d: 0,
        f_bits: 0,
        g: false,
        h: 0,
        i: 0,
    };
    let sent = 0x5EC7_0000_0000_0001i64;
    vec![
        TypedCheck::Mix {
            inputs: FROZEN_TYPED_INPUTS,
            expected: crate::typed_decl::HAND_COMPUTED_TYPED_EXPECTED,
        },
        TypedCheck::Mix {
            inputs: neg0,
            expected: oracle_typed_mix(&neg0),
        },
        TypedCheck::Mix {
            inputs: bool_f,
            expected: oracle_typed_mix(&bool_f),
        },
        TypedCheck::Mix {
            inputs: char_min,
            expected: oracle_typed_mix(&char_min),
        },
        TypedCheck::Mix {
            inputs: zeros,
            expected: oracle_typed_mix(&zeros),
        },
        TypedCheck::Div {
            n: 20,
            d: 2,
            expect: oracle_typed_div(20, 2),
            sentinel: sent,
        },
        TypedCheck::Div {
            n: -7,
            d: 3,
            expect: oracle_typed_div(-7, 3),
            sentinel: sent,
        },
        TypedCheck::Div {
            n: i64::MIN,
            d: -1,
            expect: oracle_typed_div(i64::MIN, -1),
            sentinel: sent,
        },
        TypedCheck::Div {
            n: 42,
            d: 0,
            expect: oracle_typed_div(42, 0),
            sentinel: sent,
        },
    ]
}

// status re-export guard: the 797 module keeps its own frozen codes; the
// typed chain must not accidentally bind them.
const _: () = assert!(status::ERR_DIV_ZERO == t937_status::ERR_DIV_ZERO);

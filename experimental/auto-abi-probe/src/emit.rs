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

use crate::decl::{AbiScalar, BodyKind, FnDecl, MixInputs, RecValues};

use std::collections::BTreeMap;

use cranelift_codegen::ir::condcodes::{FloatCC, IntCC};
use cranelift_codegen::ir::types;
use cranelift_codegen::ir::{AbiParam, InstBuilder, Signature};
use cranelift_codegen::isa::CallConv;
use cranelift_codegen::settings::Configurable;
use cranelift_codegen::settings::{builder as flags_builder, Flags};
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext};
use cranelift_module::{DataDescription, Linkage, Module};
use cranelift_object::{ObjectBuilder, ObjectModule};
use target_lexicon::Triple;

const WINCALL: CallConv = CallConv::WindowsFastcall;
const ENTRY_SYMBOL: &str = "p797_entry";

fn cranelift_type(s: AbiScalar) -> types::Type {
    match s {
        AbiScalar::I32 => types::I32,
        AbiScalar::I64 | AbiScalar::Ptr => types::I64,
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
    let builder = ObjectBuilder::new(isa, name, cranelift_module::default_libcall_names())
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
        let result = lower_body(&mut fb, decl.body, &params, None, None)?;
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

// ---------------------------------------------------------------------------
// T-03 suite: multi-function callee modules + check-driven callers
// ---------------------------------------------------------------------------

use crate::decl::P797_MIX;
use cranelift_codegen::ir::GlobalValue;

/// One in-PE check a suite caller performs. Each check lowers to a 0/1
/// violation flag; the PE exits with the number of FAILED checks, so a
/// single-check caller exiting 1 is the exact negative-control shape.
pub enum SuiteCheck {
    Mix {
        symbol: &'static str,
        inputs: MixInputs,
        expected: i64,
        /// Negative control: pass f and h swapped — the frozen weights
        /// make the result differ.
        swap_fh: bool,
    },
    Facc {
        symbol: &'static str,
        a: f64,
        b: f64,
        expected: f64,
    },
    Outer {
        symbol: &'static str,
        a: i32,
        b: f64,
        expected: i64,
    },
    DivOk {
        symbol: &'static str,
        n: i64,
        d: i64,
        expected_q: i64,
    },
    /// Failure contract: d == 0 must return status -1 AND leave the out
    /// slot at its pre-call sentinel value.
    DivZero {
        symbol: &'static str,
        n: i64,
        sentinel: i64,
    },
    RecSum {
        symbol: &'static str,
        rec: RecValues,
        expected: i64,
        /// Negative control: place `tag` at a WRONG offset (12) — the
        /// callee's frozen offsets then read garbage and diverge.
        bad_tag_offset: bool,
    },
    /// Observable counter: call once per x; the LAST call's return is
    /// the cumulative total — a missed or extra call changes it.
    BumpSeq {
        symbol: &'static str,
        xs: &'static [i32],
        expected_total: i64,
    },
    // ---- T-04 resource/ownership/callback fixtures (V-04/V-05) ----
    /// Create an owned handle; `dst_reg` receives it (0 on failure).
    HCreate {
        symbol: &'static str,
        size: i32,
        dst_reg: usize,
        expect_fail: bool,
    },
    HSet {
        symbol: &'static str,
        handle_reg: usize,
        idx: i32,
        val: i64,
        expect_status: i32,
    },
    HGet {
        symbol: &'static str,
        handle_reg: usize,
        idx: i32,
        expected: i64,
        expect_status: i32,
    },
    HDestroy {
        symbol: &'static str,
        handle_reg: usize,
        expect_status: i32,
    },
    /// REAL ledger facts: live/allocs/frees must match exactly.
    Ledger {
        symbol: &'static str,
        expect_live: i64,
        expect_allocs: i64,
        expect_frees: i64,
    },
    /// Borrowed-buffer sum over real bytes.
    BufSumOk {
        symbol: &'static str,
        bytes: &'static [u8],
        expected: i64,
    },
    /// null + nonzero length is the frozen buffer rejection; the out
    /// slot must stay at its sentinel.
    BufSumNull {
        symbol: &'static str,
        len: u64,
        sentinel: i64,
    },
    /// Copy that fits; dst must equal `expect` afterwards.
    BufCopyOk {
        symbol: &'static str,
        src: &'static [u8],
        expect: &'static [u8],
    },
    /// Copy that does NOT fit: ERR_CAP, dst stays at the 0xAA fill.
    BufCopyCap {
        symbol: &'static str,
        cap: u64,
        src: &'static [u8],
    },
    /// Arm the Nth-allocation failure injection.
    FailNth { symbol: &'static str, n: i32 },
    /// Composite: on success the out handle lands in `dst_reg`; on the
    /// injected failure the out slot keeps its sentinel and the ledger
    /// shows full rollback.
    TwoStep {
        symbol: &'static str,
        x: i32,
        expect_status: i32,
        sentinel: i64,
        dst_reg: Option<usize>,
    },
    /// Synchronous callback fold with explicit ctx.
    Fold {
        symbol: &'static str,
        cb: CbKind,
        ctx: i64,
        items: &'static [i64],
        expected: i64,
    },
    /// The provider stores no callbacks — observably 0.
    StoredCb { symbol: &'static str },
    /// F-797-R4-02: borrowed UTF-8 buffer — the provider VALIDATES the
    /// encoding and returns the character count; invalid input is a
    /// typed status rejection with the out slot untouched.
    BufUtf8Ok {
        symbol: &'static str,
        bytes: &'static [u8],
        expected_chars: i64,
    },
    BufUtf8Invalid {
        symbol: &'static str,
        bytes: &'static [u8],
        sentinel: i64,
    },
    /// F-797-R4-02: record-view VALIDATION — illegal bool (not 0/1) or
    /// non-scalar char are typed rejections; a valid record sums.
    RecValidOk {
        symbol: &'static str,
        rec: RecValues,
        expected_sum: i64,
    },
    RecValidBad {
        symbol: &'static str,
        bad_flag: Option<u8>,
        bad_tag: Option<u32>,
        sentinel: i64,
    },
}

/// Callback shapes the caller can hand to the provider (both defined
/// in the caller object; passing DIFFERENT callbacks in sequence proves
/// the provider never kept the first one).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum CbKind {
    /// ctx*1000 + item
    Ctx1kAddItem,
    /// ctx*2000 + item*3
    Ctx2kItem3,
}

/// Signature of a helper the suite uses: (symbol, params, ret).
fn helper_sig(params: &[AbiScalar], ret: AbiScalar) -> Signature {
    Signature {
        call_conv: WINCALL,
        params: params
            .iter()
            .map(|p| AbiParam::new(cranelift_type(*p)))
            .collect(),
        returns: vec![AbiParam::new(cranelift_type(ret))],
    }
}

/// Emit a multi-function AutoABI callee module. `defs` are defined and
/// exported; `import_inner == false` additionally imports `p797_inner`
/// (nested chain: the C caller object provides it at link time).
pub fn emit_callee_suite(
    module_name: &str,
    defs: &[FnDecl],
    import_inner: bool,
) -> Result<Vec<u8>, String> {
    let mut om = new_module(module_name)?;

    // p797_inner: imported (the C side provides it) or simply part of
    // `defs` (the defs loop declares it as Export). Never pre-declared
    // as an Export that no def will define.
    let inner_import = if import_inner {
        let sig = Signature {
            call_conv: WINCALL,
            params: vec![AbiParam::new(types::I64)],
            returns: vec![AbiParam::new(types::I64)],
        };
        Some(
            om.declare_function("p797_inner", Linkage::Import, &sig)
                .map_err(|e| format!("declare p797_inner: {e}"))?,
        )
    } else {
        None
    };

    // Module-global counter for BumpV1 (writable, zero-initialized).
    let mut bump_ctr: Option<cranelift_module::DataId> = None;
    if defs.iter().any(|d| d.body == BodyKind::BumpV1) {
        let id = om
            .declare_data("p797_bump_ctr", Linkage::Export, true, false)
            .map_err(|e| format!("declare bump ctr: {e}"))?;
        let mut dd = DataDescription::new();
        dd.define(vec![0u8; 8].into_boxed_slice());
        dd.set_align(8);
        om.define_data(id, &dd)
            .map_err(|e| format!("define bump ctr: {e}"))?;
        bump_ctr = Some(id);
    }

    // Pre-declare every function so bodies can reference each other.
    let mut ids = Vec::new();
    for d in defs {
        let sig = decl_signature(d);
        let id = om
            .declare_function(d.symbol, Linkage::Export, &sig)
            .map_err(|e| format!("declare {}: {e}", d.symbol))?;
        ids.push((d, id));
    }
    let inner_id = inner_import.or_else(|| {
        ids.iter()
            .find(|(d, _)| d.symbol == "p797_inner")
            .map(|(_, id)| *id)
    });

    for (d, fid) in &ids {
        let sig = decl_signature(d);
        let mut ctx = om.make_context();
        ctx.func.signature = sig.clone();
        {
            let mut fb_ctx = FunctionBuilderContext::new();
            let mut fb = FunctionBuilder::new(&mut ctx.func, &mut fb_ctx);
            let entry = fb.create_block();
            fb.append_block_params_for_function_params(entry);
            fb.switch_to_block(entry);
            fb.seal_block(entry);
            let params = fb.block_params(entry).to_vec();
            let inner_ref = if d.body == BodyKind::OuterV1 {
                Some(om.declare_func_in_func(
                    inner_id.expect("OuterV1 requires p797_inner (imported or in defs)"),
                    &mut fb.func,
                ))
            } else {
                None
            };
            let ctr = bump_ctr.map(|id| om.declare_data_in_func(id, &mut fb.func));
            let result = lower_body(&mut fb, d.body, &params, inner_ref, ctr)?;
            fb.ins().return_(&[result]);
            fb.finalize();
        }
        om.define_function(*fid, &mut ctx)
            .map_err(|e| format!("define {}: {e}", d.symbol))?;
    }
    om.finish()
        .emit()
        .map_err(|e| format!("emit suite callee: {e}"))
}

fn decl_signature(d: &FnDecl) -> Signature {
    Signature {
        call_conv: WINCALL,
        params: d
            .params
            .iter()
            .map(|p| AbiParam::new(cranelift_type(*p)))
            .collect(),
        returns: vec![AbiParam::new(cranelift_type(d.ret))],
    }
}

/// Lower one frozen body family. This is the AutoABI-side implementation
/// of every frozen semantic; the independent Rust oracles (`decl.rs`)
/// and hand-written C (`cside/interop_*.c`) are the other two.
fn lower_body(
    fb: &mut FunctionBuilder,
    body: BodyKind,
    args: &[cranelift_codegen::ir::Value],
    inner_ref: Option<cranelift_codegen::ir::FuncRef>,
    bump_ctr: Option<GlobalValue>,
) -> Result<cranelift_codegen::ir::Value, String> {
    match body {
        BodyKind::MixV1 => Ok(emit_mix_v1_body(fb, args)),
        BodyKind::MixAlt(seed) => {
            // Same eight terms as MixV1, but the accumulation START is
            // rotated by the seed and each weighted multiply is split
            // (x*k as x*(k-1)+x) on alternating seeds — a distinct
            // instruction shape per seed with an IDENTICAL wrapping
            // result. No FP reassociation: f64 lanes are truncated to
            // i64 before any addition.
            let a = sext_i32_to_i64(fb, args[0]);
            let b = sext_i32_to_i64(fb, args[1]);
            let c = sext_i32_to_i64(fb, args[2]);
            let d = sext_i32_to_i64(fb, args[3]);
            let g = sext_i32_to_i64(fb, args[6]);
            let fi = fb.ins().fcvt_to_sint(types::I64, args[5]);
            let hi = fb.ins().fcvt_to_sint(types::I64, args[7]);
            let split = |fb: &mut FunctionBuilder,
                         v: cranelift_codegen::ir::Value,
                         k: i64|
             -> cranelift_codegen::ir::Value {
                if seed % 2 == 0 {
                    weighted_i64(fb, v, k)
                } else {
                    let lo = weighted_i64(fb, v, k - 1);
                    fb.ins().iadd(lo, v)
                }
            };
            let terms = [
                split(fb, args[4], 2),
                split(fb, a, 1),
                split(fb, b, 3),
                split(fb, c, 7),
                split(fb, d, 11),
                split(fb, g, 13),
                split(fb, fi, 1000),
                split(fb, hi, 17),
            ];
            let start = (seed % 8) as usize;
            let mut acc = terms[start];
            for k in 1..8 {
                let idx = (start + k) % 8;
                acc = fb.ins().iadd(acc, terms[idx]);
            }
            Ok(acc)
        }
        BodyKind::MixW(w) => {
            let a = sext_i32_to_i64(fb, args[0]);
            let b = sext_i32_to_i64(fb, args[1]);
            let c = sext_i32_to_i64(fb, args[2]);
            let d = sext_i32_to_i64(fb, args[3]);
            let g = sext_i32_to_i64(fb, args[6]);
            let fi = fb.ins().fcvt_to_sint(types::I64, args[5]);
            let hi = fb.ins().fcvt_to_sint(types::I64, args[7]);
            let ew = weighted_i64(fb, args[4], w);
            let a1 = weighted_i64(fb, a, 1);
            let b3 = weighted_i64(fb, b, 3);
            let c7 = weighted_i64(fb, c, 7);
            let d11 = weighted_i64(fb, d, 11);
            let g13 = weighted_i64(fb, g, 13);
            let f1000 = weighted_i64(fb, fi, 1000);
            let h17 = weighted_i64(fb, hi, 17);
            let acc = fb.ins().iadd(ew, a1);
            let acc = fb.ins().iadd(acc, b3);
            let acc = fb.ins().iadd(acc, c7);
            let acc = fb.ins().iadd(acc, d11);
            let acc = fb.ins().iadd(acc, g13);
            let acc = fb.ins().iadd(acc, f1000);
            Ok(fb.ins().iadd(acc, h17))
        }
        BodyKind::MixV1Alt => {
            // Same frozen semantics, different instruction shape:
            // pairs first, then fold — wrapping iadd is associative, so
            // the result equals oracle_mix_v1 exactly.
            let a = sext_i32_to_i64(fb, args[0]);
            let b = sext_i32_to_i64(fb, args[1]);
            let c = sext_i32_to_i64(fb, args[2]);
            let d = sext_i32_to_i64(fb, args[3]);
            let g = sext_i32_to_i64(fb, args[6]);
            let fi = fb.ins().fcvt_to_sint(types::I64, args[5]);
            let hi = fb.ins().fcvt_to_sint(types::I64, args[7]);
            let e2 = weighted_i64(fb, args[4], 2);
            let a1 = weighted_i64(fb, a, 1);
            let b3 = weighted_i64(fb, b, 3);
            let c7 = weighted_i64(fb, c, 7);
            let d11 = weighted_i64(fb, d, 11);
            let g13 = weighted_i64(fb, g, 13);
            let f1000 = weighted_i64(fb, fi, 1000);
            let h17 = weighted_i64(fb, hi, 17);
            let p1 = fb.ins().iadd(e2, h17);
            let p2 = fb.ins().iadd(a1, b3);
            let p3 = fb.ins().iadd(c7, d11);
            let p4 = fb.ins().iadd(g13, f1000);
            let q1 = fb.ins().iadd(p1, p2);
            let q2 = fb.ins().iadd(p3, p4);
            Ok(fb.ins().iadd(q1, q2))
        }
        BodyKind::InnerV1 => {
            let m3 = weighted_i64(fb, args[0], 3);
            let c11 = fb.ins().iconst(types::I64, 11);
            Ok(fb.ins().iadd(m3, c11))
        }
        BodyKind::FAccV1 => {
            let c25 = fb.ins().f64const(2.5);
            let c125 = fb.ins().f64const(1.25);
            let m1 = fb.ins().fmul(args[0], c25);
            let m2 = fb.ins().fmul(args[1], c125);
            Ok(fb.ins().fadd(m1, m2))
        }
        BodyKind::OuterV1 => {
            let inner = inner_ref.ok_or("OuterV1 needs the inner reference")?;
            let a = sext_i32_to_i64(fb, args[0]);
            let t = fb.ins().fcvt_to_sint(types::I64, args[1]);
            let a7 = weighted_i64(fb, a, 7);
            let t100 = weighted_i64(fb, t, 100);
            let x = fb.ins().iadd(a7, t100);
            let call = fb.ins().call(inner, &[x]);
            let r = fb.inst_results(call)[0];
            Ok(fb.ins().iadd(r, a))
        }
        BodyKind::FallibleDivV1 => {
            // (ctx, n, d, q*) -> i32 status; d==0 -> -1 with q untouched;
            // ctx is declared nullable and never dereferenced (F-797-R4-02).
            let zero = fb.ins().iconst(types::I64, 0);
            let is_zero = fb.ins().icmp(IntCC::Equal, args[2], zero);
            let ok_bb = fb.create_block();
            let fail_bb = fb.create_block();
            fb.ins().brif(is_zero, fail_bb, &[], ok_bb, &[]);
            fb.switch_to_block(fail_bb);
            fb.seal_block(fail_bb);
            let neg1 = fb.ins().iconst(types::I32, -1);
            fb.ins().return_(&[neg1]);
            fb.switch_to_block(ok_bb);
            fb.seal_block(ok_bb);
            let q = fb.ins().sdiv(args[1], args[2]);
            fb.ins()
                .store(cranelift_codegen::ir::MemFlags::new(), q, args[3], 0);
            let ok_status = fb.ins().iconst(types::I32, 0);
            Ok(ok_status)
        }
        BodyKind::RecSumV1 => {
            // flag@0 (u8)*1000 + count@4 (i32)*3 + tag@8 (u32)*5 + big@16.
            let mf = cranelift_codegen::ir::MemFlags::new();
            let flag8 = fb.ins().load(types::I8, mf, args[0], 0);
            let flag = fb.ins().uextend(types::I64, flag8);
            let count32 = fb.ins().load(types::I32, mf, args[0], 4);
            let count = fb.ins().sextend(types::I64, count32);
            let tag32 = fb.ins().load(types::I32, mf, args[0], 8);
            let tag = fb.ins().uextend(types::I64, tag32);
            let big = fb.ins().load(types::I64, mf, args[0], 16);
            let f1000 = weighted_i64(fb, flag, 1000);
            let c3 = weighted_i64(fb, count, 3);
            let t5 = weighted_i64(fb, tag, 5);
            let acc = fb.ins().iadd(f1000, c3);
            let acc = fb.ins().iadd(acc, t5);
            Ok(fb.ins().iadd(acc, big))
        }
        BodyKind::BumpV1 => {
            let gv = bump_ctr.ok_or("BumpV1 needs the module counter")?;
            let p = fb.ins().global_value(types::I64, gv);
            let mf = cranelift_codegen::ir::MemFlags::new();
            let cur = fb.ins().load(types::I64, mf, p, 0);
            let x = sext_i32_to_i64(fb, args[0]);
            let next = fb.ins().iadd(cur, x);
            fb.ins().store(mf, next, p, 0);
            Ok(next)
        }
    }
}

/// Emit a suite caller: imports the checked symbols, declares the check
/// data blobs (record bytes, out slots), runs every check in
/// `p797_main` accumulating i32 violation flags, exits with the flag
/// count through the generic entry wrapper.
pub fn emit_caller_suite(module_name: &str, checks: &[SuiteCheck]) -> Result<Vec<u8>, String> {
    let mut om = new_module(module_name)?;

    // Collect distinct imported symbols with their machine signatures.
    // Resource fixtures use arbitrary shapes (multi-out, void), so the
    // collector takes full Signatures.
    let mut imports: Vec<(&'static str, Signature)> = Vec::new();
    let mut need_sig = |sym: &'static str, sig: Signature| {
        if !imports.iter().any(|(s, _)| *s == sym) {
            imports.push((sym, sig));
        }
    };
    macro_rules! need {
        ($sym:expr, $params:expr, $ret:expr) => {
            need_sig($sym, helper_sig($params, $ret))
        };
    }
    let sig_of = |params: &[types::Type], rets: &[types::Type]| Signature {
        call_conv: WINCALL,
        params: params.iter().map(|t| AbiParam::new(*t)).collect(),
        returns: rets.iter().map(|t| AbiParam::new(*t)).collect(),
    };
    let t_i32 = types::I32;
    for c in checks {
        match c {
            SuiteCheck::Mix { symbol, .. } => {
                need![symbol, crate::decl::MIX_PARAMS, AbiScalar::I64]
            }
            SuiteCheck::Facc { symbol, .. } => {
                need![symbol, &[AbiScalar::F64, AbiScalar::F64], AbiScalar::F64]
            }
            SuiteCheck::Outer { symbol, .. } => {
                need![symbol, &[AbiScalar::I32, AbiScalar::F64], AbiScalar::I64]
            }
            SuiteCheck::DivOk { symbol, .. } | SuiteCheck::DivZero { symbol, .. } => {
                // ctx-first fallible form (F-797-R4-02)
                need![
                    symbol,
                    &[
                        AbiScalar::Ptr,
                        AbiScalar::I64,
                        AbiScalar::I64,
                        AbiScalar::Ptr
                    ],
                    AbiScalar::I32
                ]
            }
            SuiteCheck::BufUtf8Ok { symbol, .. } | SuiteCheck::BufUtf8Invalid { symbol, .. } => {
                need![
                    symbol,
                    &[AbiScalar::Ptr, AbiScalar::I64, AbiScalar::Ptr],
                    AbiScalar::I32
                ]
            }
            SuiteCheck::RecValidOk { symbol, .. } | SuiteCheck::RecValidBad { symbol, .. } => {
                need![symbol, &[AbiScalar::Ptr, AbiScalar::Ptr], AbiScalar::I32]
            }
            SuiteCheck::RecSum { symbol, .. } => need![symbol, &[AbiScalar::Ptr], AbiScalar::I64],
            SuiteCheck::BumpSeq { symbol, .. } => {
                need![symbol, &[AbiScalar::I32], AbiScalar::I64]
            }
            SuiteCheck::HCreate { symbol, .. } => {
                need![symbol, &[AbiScalar::I32], AbiScalar::I64]
            }
            SuiteCheck::HSet { symbol, .. } => {
                need![
                    symbol,
                    &[AbiScalar::I64, AbiScalar::I32, AbiScalar::I64],
                    AbiScalar::I32
                ]
            }
            SuiteCheck::HGet { symbol, .. } => {
                need![
                    symbol,
                    &[AbiScalar::I64, AbiScalar::I32, AbiScalar::Ptr],
                    AbiScalar::I32
                ]
            }
            SuiteCheck::HDestroy { symbol, .. } => {
                need![symbol, &[AbiScalar::I64], AbiScalar::I32]
            }
            SuiteCheck::Ledger { symbol, .. } => {
                need![
                    symbol,
                    &[AbiScalar::Ptr, AbiScalar::Ptr, AbiScalar::Ptr],
                    AbiScalar::I32
                ]
            }
            SuiteCheck::BufSumOk { symbol, .. } | SuiteCheck::BufSumNull { symbol, .. } => {
                need![
                    symbol,
                    &[AbiScalar::Ptr, AbiScalar::I64, AbiScalar::Ptr],
                    AbiScalar::I32
                ]
            }
            SuiteCheck::BufCopyOk { symbol, .. } | SuiteCheck::BufCopyCap { symbol, .. } => {
                need![
                    symbol,
                    &[
                        AbiScalar::Ptr,
                        AbiScalar::I64,
                        AbiScalar::Ptr,
                        AbiScalar::I64
                    ],
                    AbiScalar::I32
                ]
            }
            SuiteCheck::FailNth { symbol, .. } => {
                need_sig(symbol, sig_of(&[t_i32], &[])); // void return
            }
            SuiteCheck::TwoStep { symbol, .. } => {
                need![symbol, &[AbiScalar::I32, AbiScalar::Ptr], AbiScalar::I32]
            }
            SuiteCheck::Fold { symbol, .. } => {
                need![
                    symbol,
                    &[
                        AbiScalar::Ptr,
                        AbiScalar::I64,
                        AbiScalar::Ptr,
                        AbiScalar::I64
                    ],
                    AbiScalar::I64
                ]
            }
            SuiteCheck::StoredCb { symbol } => {
                need![symbol, &[], AbiScalar::I64]
            }
        }
    }
    let mut import_ids = Vec::new();
    for (sym, sig) in &imports {
        let id = om
            .declare_function(sym, Linkage::Import, sig)
            .map_err(|e| format!("declare import {sym}: {e}"))?;
        import_ids.push((*sym, id));
    }

    // ExitProcess import.
    let exit_sig = Signature {
        call_conv: WINCALL,
        params: vec![AbiParam::new(types::I32)],
        returns: vec![],
    };
    let exit_id = om
        .declare_function("ExitProcess", Linkage::Import, &exit_sig)
        .map_err(|e| format!("declare ExitProcess: {e}"))?;

    // Data blobs: per-check record bytes and out slots (unique names).
    let mut data_ids: Vec<(String, cranelift_module::DataId)> = Vec::new();
    let add_data = |om: &mut ObjectModule,
                    name: String,
                    bytes: Vec<u8>,
                    align: u64|
     -> Result<cranelift_module::DataId, String> {
        let id = om
            .declare_data(&name, Linkage::Export, true, false)
            .map_err(|e| format!("declare data {name}: {e}"))?;
        let mut dd = DataDescription::new();
        dd.define(bytes.into_boxed_slice());
        dd.set_align(align);
        om.define_data(id, &dd)
            .map_err(|e| format!("define data {name}: {e}"))?;
        Ok(id)
    };
    for (i, c) in checks.iter().enumerate() {
        match c {
            SuiteCheck::RecSum {
                rec,
                bad_tag_offset,
                ..
            } => {
                let bytes =
                    crate::decl::encode_rec_bytes(rec, if *bad_tag_offset { 12 } else { 8 });
                let id = add_data(&mut om, format!("p797_d_rec{i}"), bytes.to_vec(), 8)?;
                data_ids.push((format!("rec{i}"), id));
            }
            SuiteCheck::DivOk { .. } => {
                let id = add_data(&mut om, format!("p797_d_q{i}"), vec![0u8; 8], 8)?;
                data_ids.push((format!("q{i}"), id));
            }
            SuiteCheck::DivZero { sentinel, .. } => {
                let id = add_data(
                    &mut om,
                    format!("p797_d_q{i}"),
                    sentinel.to_le_bytes().to_vec(),
                    8,
                )?;
                data_ids.push((format!("q{i}"), id));
            }
            SuiteCheck::HGet { .. } => {
                let id = add_data(&mut om, format!("p797_d_q{i}"), vec![0u8; 8], 8)?;
                data_ids.push((format!("q{i}"), id));
            }
            SuiteCheck::BufSumNull { sentinel, .. } | SuiteCheck::TwoStep { sentinel, .. } => {
                let id = add_data(
                    &mut om,
                    format!("p797_d_q{i}"),
                    sentinel.to_le_bytes().to_vec(),
                    8,
                )?;
                data_ids.push((format!("q{i}"), id));
            }
            SuiteCheck::BufSumOk { bytes, .. } => {
                let q = add_data(&mut om, format!("p797_d_q{i}"), vec![0u8; 8], 8)?;
                data_ids.push((format!("q{i}"), q));
                let b = add_data(&mut om, format!("p797_d_b{i}"), bytes.to_vec(), 1)?;
                data_ids.push((format!("b{i}"), b));
            }
            SuiteCheck::Ledger { .. } => {
                for k in 0..3 {
                    let id = add_data(&mut om, format!("p797_d_l{i}_{k}"), vec![0u8; 8], 8)?;
                    data_ids.push((format!("l{i}_{k}"), id));
                }
            }
            SuiteCheck::BufCopyOk { src, .. } => {
                let src_id = add_data(&mut om, format!("p797_d_src{i}"), src.to_vec(), 1)?;
                data_ids.push((format!("src{i}"), src_id));
                let dst_id = add_data(
                    &mut om,
                    format!("p797_d_dst{i}"),
                    vec![0u8; src.len().max(1)],
                    1,
                )?;
                data_ids.push((format!("dst{i}"), dst_id));
            }
            SuiteCheck::BufCopyCap { cap, src, .. } => {
                let src_id = add_data(&mut om, format!("p797_d_src{i}"), src.to_vec(), 1)?;
                data_ids.push((format!("src{i}"), src_id));
                let dst_id = add_data(
                    &mut om,
                    format!("p797_d_dst{i}"),
                    vec![0xAAu8; (*cap).max(1) as usize],
                    1,
                )?;
                data_ids.push((format!("dst{i}"), dst_id));
            }
            SuiteCheck::BufUtf8Ok { bytes, .. } => {
                let q = add_data(&mut om, format!("p797_d_q{i}"), vec![0u8; 8], 8)?;
                data_ids.push((format!("q{i}"), q));
                let b = add_data(&mut om, format!("p797_d_b{i}"), bytes.to_vec(), 1)?;
                data_ids.push((format!("b{i}"), b));
            }
            SuiteCheck::BufUtf8Invalid {
                bytes, sentinel, ..
            } => {
                let q = add_data(
                    &mut om,
                    format!("p797_d_q{i}"),
                    sentinel.to_le_bytes().to_vec(),
                    8,
                )?;
                data_ids.push((format!("q{i}"), q));
                let b = add_data(&mut om, format!("p797_d_b{i}"), bytes.to_vec(), 1)?;
                data_ids.push((format!("b{i}"), b));
            }
            SuiteCheck::RecValidOk { rec, .. } => {
                let bytes = crate::decl::encode_rec_bytes(rec, 8);
                let r = add_data(&mut om, format!("p797_d_rec{i}"), bytes.to_vec(), 8)?;
                data_ids.push((format!("rec{i}"), r));
                let q = add_data(&mut om, format!("p797_d_q{i}"), vec![0u8; 8], 8)?;
                data_ids.push((format!("q{i}"), q));
            }
            SuiteCheck::RecValidBad {
                bad_flag,
                bad_tag,
                sentinel,
                ..
            } => {
                let mut bytes = crate::decl::encode_rec_bytes(&crate::decl::FROZEN_REC, 8);
                if let Some(f) = bad_flag {
                    bytes[0] = *f;
                }
                if let Some(t) = bad_tag {
                    bytes[8..12].copy_from_slice(&t.to_le_bytes());
                }
                let r = add_data(&mut om, format!("p797_d_rec{i}"), bytes.to_vec(), 8)?;
                data_ids.push((format!("rec{i}"), r));
                let q = add_data(
                    &mut om,
                    format!("p797_d_q{i}"),
                    sentinel.to_le_bytes().to_vec(),
                    8,
                )?;
                data_ids.push((format!("q{i}"), q));
            }
            SuiteCheck::Fold { items, .. } => {
                let mut b = Vec::with_capacity(items.len() * 8);
                for v in *items {
                    b.extend_from_slice(&v.to_le_bytes());
                }
                let id = add_data(&mut om, format!("p797_d_it{i}"), b, 8)?;
                data_ids.push((format!("it{i}"), id));
            }
            _ => {}
        }
    }

    // Callback shapes (only when some Fold check uses them). Defined in
    // the CALLER object; their addresses are handed across the boundary
    // per call — the provider must never store them.
    let cb_needed = |k: CbKind| {
        checks
            .iter()
            .any(|c| matches!(c, SuiteCheck::Fold { cb, .. } if *cb == k))
    };
    let cb_sig = Signature {
        call_conv: WINCALL,
        params: vec![AbiParam::new(types::I64), AbiParam::new(types::I64)],
        returns: vec![AbiParam::new(types::I64)],
    };
    let mut cb_ids: Vec<(CbKind, cranelift_module::FuncId)> = Vec::new();
    for (k, name, w_ctx, w_item) in [
        (CbKind::Ctx1kAddItem, "p797_cb1", 1000i64, 1i64),
        (CbKind::Ctx2kItem3, "p797_cb2", 2000i64, 3i64),
    ] {
        if !cb_needed(k) {
            continue;
        }
        let fid = om
            .declare_function(name, Linkage::Export, &cb_sig)
            .map_err(|e| format!("declare {name}: {e}"))?;
        let mut cctx = om.make_context();
        cctx.func.signature = cb_sig.clone();
        {
            let mut fbc = FunctionBuilderContext::new();
            let mut fbb = FunctionBuilder::new(&mut cctx.func, &mut fbc);
            let entry = fbb.create_block();
            fbb.append_block_params_for_function_params(entry);
            fbb.switch_to_block(entry);
            fbb.seal_block(entry);
            let ps = fbb.block_params(entry).to_vec();
            let a = weighted_i64(&mut fbb, ps[0], w_ctx);
            let b = weighted_i64(&mut fbb, ps[1], w_item);
            let r = fbb.ins().iadd(a, b);
            fbb.ins().return_(&[r]);
            fbb.finalize();
        }
        om.define_function(fid, &mut cctx)
            .map_err(|e| format!("define {name}: {e}"))?;
        cb_ids.push((k, fid));
    }

    // p797_main: run checks, accumulate flags.
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

        let mut frefs = BTreeMap::new();
        for (sym, id) in &import_ids {
            let r = om.declare_func_in_func(*id, &mut fb.func);
            frefs.insert(*sym, r);
        }
        let mut drefs = BTreeMap::new();
        for (name, id) in &data_ids {
            let g = om.declare_data_in_func(*id, &mut fb.func);
            drefs.insert(name.clone(), g);
        }
        let mut cb_refs = BTreeMap::new();
        for (k, id) in &cb_ids {
            let r = om.declare_func_in_func(*id, &mut fb.func);
            cb_refs.insert(*k, r);
        }

        let blk = fb.create_block();
        fb.switch_to_block(blk);
        fb.seal_block(blk);

        // Handle-flow registers (i64 values crossing checks).
        let regs_used = checks
            .iter()
            .filter_map(|c| match c {
                SuiteCheck::HCreate { dst_reg, .. } => Some(*dst_reg),
                SuiteCheck::TwoStep { dst_reg, .. } => *dst_reg,
                _ => None,
            })
            .max()
            .map(|m| m + 1)
            .unwrap_or(0);
        let mut regs: Vec<cranelift_codegen::ir::Value> = (0..regs_used)
            .map(|_| fb.ins().iconst(types::I64, 0))
            .collect();

        let zero = fb.ins().iconst(types::I32, 0);
        let mut flags = zero;
        let mut acc_flag = |fb: &mut FunctionBuilder, f: cranelift_codegen::ir::Value| {
            flags = fb.ins().iadd(flags, f);
        };

        let mf = cranelift_codegen::ir::MemFlags::new();
        let expect_i64 =
            |fb: &mut FunctionBuilder, got: cranelift_codegen::ir::Value, want: i64| {
                let w = fb.ins().iconst(types::I64, want);
                let ne = fb.ins().icmp(IntCC::NotEqual, got, w);
                fb.ins().uextend(types::I32, ne)
            };
        let expect_i32 =
            |fb: &mut FunctionBuilder, got: cranelift_codegen::ir::Value, want: i64| {
                let w = fb.ins().iconst(types::I32, want);
                let ne = fb.ins().icmp(IntCC::NotEqual, got, w);
                fb.ins().uextend(types::I32, ne)
            };

        for (i, c) in checks.iter().enumerate() {
            match c {
                SuiteCheck::Mix {
                    symbol,
                    inputs,
                    expected,
                    swap_fh,
                } => {
                    let f = *frefs.get(symbol).unwrap();
                    let (fv, hv) = if *swap_fh {
                        (inputs.h, inputs.f)
                    } else {
                        (inputs.f, inputs.h)
                    };
                    let a = fb.ins().iconst(types::I32, inputs.a as i64);
                    let b = fb.ins().iconst(types::I32, inputs.b as i64);
                    let cc = fb.ins().iconst(types::I32, inputs.c as i64);
                    let d = fb.ins().iconst(types::I32, inputs.d as i64);
                    let e = fb.ins().iconst(types::I64, inputs.e);
                    let fv = fb.ins().f64const(fv);
                    let g = fb.ins().iconst(types::I32, inputs.g as i64);
                    let hv = fb.ins().f64const(hv);
                    let call = fb.ins().call(f, &[a, b, cc, d, e, fv, g, hv]);
                    let got = fb.inst_results(call)[0];
                    let fl = expect_i64(&mut fb, got, *expected);
                    acc_flag(&mut fb, fl);
                }
                SuiteCheck::Facc {
                    symbol,
                    a,
                    b,
                    expected,
                } => {
                    let f = *frefs.get(symbol).unwrap();
                    let av = fb.ins().f64const(*a);
                    let bv = fb.ins().f64const(*b);
                    let call = fb.ins().call(f, &[av, bv]);
                    let got = fb.inst_results(call)[0];
                    let w = fb.ins().f64const(*expected);
                    let ne = fb.ins().fcmp(FloatCC::NotEqual, got, w);
                    let fl = fb.ins().uextend(types::I32, ne);
                    acc_flag(&mut fb, fl);
                }
                SuiteCheck::Outer {
                    symbol,
                    a,
                    b,
                    expected,
                } => {
                    let f = *frefs.get(symbol).unwrap();
                    let av = fb.ins().iconst(types::I32, *a as i64);
                    let bv = fb.ins().f64const(*b);
                    let call = fb.ins().call(f, &[av, bv]);
                    let got = fb.inst_results(call)[0];
                    let fl = expect_i64(&mut fb, got, *expected);
                    acc_flag(&mut fb, fl);
                }
                SuiteCheck::DivOk {
                    symbol,
                    n,
                    d,
                    expected_q,
                } => {
                    let f = *frefs.get(symbol).unwrap();
                    let qp = fb
                        .ins()
                        .global_value(types::I64, *drefs.get(&format!("q{i}")).unwrap());
                    let null_ctx = fb.ins().iconst(types::I64, 0); // nullable ctx (§2)
                    let nv = fb.ins().iconst(types::I64, *n);
                    let dv = fb.ins().iconst(types::I64, *d);
                    let call = fb.ins().call(f, &[null_ctx, nv, dv, qp]);
                    let status = fb.inst_results(call)[0];
                    let f1 = expect_i32(&mut fb, status, 0);
                    acc_flag(&mut fb, f1);
                    let q = fb.ins().load(types::I64, mf, qp, 0);
                    let f2 = expect_i64(&mut fb, q, *expected_q);
                    acc_flag(&mut fb, f2);
                }
                SuiteCheck::DivZero {
                    symbol,
                    n,
                    sentinel,
                } => {
                    let f = *frefs.get(symbol).unwrap();
                    let qp = fb
                        .ins()
                        .global_value(types::I64, *drefs.get(&format!("q{i}")).unwrap());
                    let null_ctx = fb.ins().iconst(types::I64, 0); // nullable ctx (§2)
                    let nv = fb.ins().iconst(types::I64, *n);
                    let dz = fb.ins().iconst(types::I64, 0);
                    let call = fb.ins().call(f, &[null_ctx, nv, dz, qp]);
                    let status = fb.inst_results(call)[0];
                    let f1 = expect_i32(&mut fb, status, crate::decl::status::ERR_DIV_ZERO as i64);
                    acc_flag(&mut fb, f1);
                    let q = fb.ins().load(types::I64, mf, qp, 0);
                    let f2 = expect_i64(&mut fb, q, *sentinel);
                    acc_flag(&mut fb, f2);
                }
                SuiteCheck::RecSum {
                    symbol,
                    rec,
                    expected,
                    ..
                } => {
                    let f = *frefs.get(symbol).unwrap();
                    let rp = fb
                        .ins()
                        .global_value(types::I64, *drefs.get(&format!("rec{i}")).unwrap());
                    let call = fb.ins().call(f, &[rp]);
                    let got = fb.inst_results(call)[0];
                    let _ = rec;
                    let fl = expect_i64(&mut fb, got, *expected);
                    acc_flag(&mut fb, fl);
                }
                SuiteCheck::BumpSeq {
                    symbol,
                    xs,
                    expected_total,
                } => {
                    let f = *frefs.get(symbol).unwrap();
                    let mut last = fb.ins().iconst(types::I64, 0);
                    for x in *xs {
                        let xv = fb.ins().iconst(types::I32, *x as i64);
                        let call = fb.ins().call(f, &[xv]);
                        last = fb.inst_results(call)[0];
                    }
                    let f1 = expect_i64(&mut fb, last, *expected_total);
                    acc_flag(&mut fb, f1);
                }
                SuiteCheck::HCreate {
                    symbol,
                    size,
                    dst_reg,
                    expect_fail,
                } => {
                    let f = *frefs.get(symbol).unwrap();
                    let sv = fb.ins().iconst(types::I32, *size as i64);
                    let call = fb.ins().call(f, &[sv]);
                    let h = fb.inst_results(call)[0];
                    regs[*dst_reg] = h;
                    let zero = fb.ins().iconst(types::I64, 0);
                    let is_zero = fb.ins().icmp(IntCC::Equal, h, zero);
                    let failed = fb.ins().uextend(types::I32, is_zero);
                    // want: failed == expect_fail
                    let wf = fb.ins().iconst(types::I32, *expect_fail as i64);
                    let ne = fb.ins().icmp(IntCC::NotEqual, failed, wf);
                    let fl = fb.ins().uextend(types::I32, ne);
                    acc_flag(&mut fb, fl);
                }
                SuiteCheck::HSet {
                    symbol,
                    handle_reg,
                    idx,
                    val,
                    expect_status,
                } => {
                    let f = *frefs.get(symbol).unwrap();
                    let iv = fb.ins().iconst(types::I32, *idx as i64);
                    let vv = fb.ins().iconst(types::I64, *val);
                    let call = fb.ins().call(f, &[regs[*handle_reg], iv, vv]);
                    let st = fb.inst_results(call)[0];
                    let f1 = expect_i32(&mut fb, st, *expect_status as i64);
                    acc_flag(&mut fb, f1);
                }
                SuiteCheck::HGet {
                    symbol,
                    handle_reg,
                    idx,
                    expected,
                    expect_status,
                } => {
                    let f = *frefs.get(symbol).unwrap();
                    let qp = fb
                        .ins()
                        .global_value(types::I64, *drefs.get(&format!("q{i}")).unwrap());
                    let iv = fb.ins().iconst(types::I32, *idx as i64);
                    let call = fb.ins().call(f, &[regs[*handle_reg], iv, qp]);
                    let st = fb.inst_results(call)[0];
                    let f1 = expect_i32(&mut fb, st, *expect_status as i64);
                    acc_flag(&mut fb, f1);
                    let q = fb.ins().load(types::I64, mf, qp, 0);
                    let f2 = expect_i64(&mut fb, q, *expected);
                    acc_flag(&mut fb, f2);
                }
                SuiteCheck::HDestroy {
                    symbol,
                    handle_reg,
                    expect_status,
                } => {
                    let f = *frefs.get(symbol).unwrap();
                    let call = fb.ins().call(f, &[regs[*handle_reg]]);
                    let st = fb.inst_results(call)[0];
                    let f1 = expect_i32(&mut fb, st, *expect_status as i64);
                    acc_flag(&mut fb, f1);
                }
                SuiteCheck::Ledger {
                    symbol,
                    expect_live,
                    expect_allocs,
                    expect_frees,
                } => {
                    let f = *frefs.get(symbol).unwrap();
                    let p0 = fb
                        .ins()
                        .global_value(types::I64, *drefs.get(&format!("l{i}_0")).unwrap());
                    let p1 = fb
                        .ins()
                        .global_value(types::I64, *drefs.get(&format!("l{i}_1")).unwrap());
                    let p2 = fb
                        .ins()
                        .global_value(types::I64, *drefs.get(&format!("l{i}_2")).unwrap());
                    let call = fb.ins().call(f, &[p0, p1, p2]);
                    let st = fb.inst_results(call)[0];
                    let f1 = expect_i32(&mut fb, st, 0);
                    acc_flag(&mut fb, f1);
                    let l0 = fb.ins().load(types::I64, mf, p0, 0);
                    let l1 = fb.ins().load(types::I64, mf, p1, 0);
                    let l2 = fb.ins().load(types::I64, mf, p2, 0);
                    let f2 = expect_i64(&mut fb, l0, *expect_live);
                    let f3 = expect_i64(&mut fb, l1, *expect_allocs);
                    let f4 = expect_i64(&mut fb, l2, *expect_frees);
                    acc_flag(&mut fb, f2);
                    acc_flag(&mut fb, f3);
                    acc_flag(&mut fb, f4);
                }
                SuiteCheck::BufSumOk {
                    symbol,
                    bytes,
                    expected,
                } => {
                    let f = *frefs.get(symbol).unwrap();
                    let bp = fb
                        .ins()
                        .global_value(types::I64, *drefs.get(&format!("b{i}")).unwrap());
                    let qp = fb
                        .ins()
                        .global_value(types::I64, *drefs.get(&format!("q{i}")).unwrap());
                    let lv = fb.ins().iconst(types::I64, bytes.len() as i64);
                    let call = fb.ins().call(f, &[bp, lv, qp]);
                    let st = fb.inst_results(call)[0];
                    let f1 = expect_i32(&mut fb, st, 0);
                    acc_flag(&mut fb, f1);
                    let q = fb.ins().load(types::I64, mf, qp, 0);
                    let f2 = expect_i64(&mut fb, q, *expected);
                    acc_flag(&mut fb, f2);
                }
                SuiteCheck::BufSumNull {
                    symbol,
                    len,
                    sentinel,
                } => {
                    let f = *frefs.get(symbol).unwrap();
                    let qp = fb
                        .ins()
                        .global_value(types::I64, *drefs.get(&format!("q{i}")).unwrap());
                    let null = fb.ins().iconst(types::I64, 0);
                    let lv = fb.ins().iconst(types::I64, *len as i64);
                    let call = fb.ins().call(f, &[null, lv, qp]);
                    let st = fb.inst_results(call)[0];
                    let f1 = expect_i32(&mut fb, st, -3); // R797_ERR_BUFNULL
                    acc_flag(&mut fb, f1);
                    let q = fb.ins().load(types::I64, mf, qp, 0);
                    let f2 = expect_i64(&mut fb, q, *sentinel);
                    acc_flag(&mut fb, f2);
                }
                SuiteCheck::BufCopyOk {
                    symbol,
                    src,
                    expect,
                } => {
                    let f = *frefs.get(symbol).unwrap();
                    let dp = fb
                        .ins()
                        .global_value(types::I64, *drefs.get(&format!("dst{i}")).unwrap());
                    let sp = fb
                        .ins()
                        .global_value(types::I64, *drefs.get(&format!("src{i}")).unwrap());
                    let cap = fb.ins().iconst(types::I64, expect.len() as i64);
                    let lv = fb.ins().iconst(types::I64, src.len() as i64);
                    let call = fb.ins().call(f, &[dp, cap, sp, lv]);
                    let st = fb.inst_results(call)[0];
                    let f1 = expect_i32(&mut fb, st, 0);
                    acc_flag(&mut fb, f1);
                    for (k, b) in expect.iter().enumerate() {
                        let got = fb.ins().load(types::I8, mf, dp, k as i32);
                        let w = fb.ins().iconst(types::I8, *b as i64);
                        let ne = fb.ins().icmp(IntCC::NotEqual, got, w);
                        let fk = fb.ins().uextend(types::I32, ne);
                        acc_flag(&mut fb, fk);
                    }
                }
                SuiteCheck::BufCopyCap { symbol, cap, src } => {
                    let f = *frefs.get(symbol).unwrap();
                    let dp = fb
                        .ins()
                        .global_value(types::I64, *drefs.get(&format!("dst{i}")).unwrap());
                    let sp = fb
                        .ins()
                        .global_value(types::I64, *drefs.get(&format!("src{i}")).unwrap());
                    let cv = fb.ins().iconst(types::I64, *cap as i64);
                    let lv = fb.ins().iconst(types::I64, src.len() as i64);
                    let call = fb.ins().call(f, &[dp, cv, sp, lv]);
                    let st = fb.inst_results(call)[0];
                    let f1 = expect_i32(&mut fb, st, -2); // R797_ERR_CAP
                    acc_flag(&mut fb, f1);
                    // every dst byte must still be the 0xAA fill
                    for k in 0..*cap as usize {
                        let got = fb.ins().load(types::I8, mf, dp, k as i32);
                        let w = fb.ins().iconst(types::I8, 0xAA);
                        let ne = fb.ins().icmp(IntCC::NotEqual, got, w);
                        let fk = fb.ins().uextend(types::I32, ne);
                        acc_flag(&mut fb, fk);
                    }
                }
                SuiteCheck::FailNth { symbol, n } => {
                    let f = *frefs.get(symbol).unwrap();
                    let nv = fb.ins().iconst(types::I32, *n as i64);
                    fb.ins().call(f, &[nv]); // void return
                }
                SuiteCheck::TwoStep {
                    symbol,
                    x,
                    expect_status,
                    sentinel,
                    dst_reg,
                } => {
                    let f = *frefs.get(symbol).unwrap();
                    let qp = fb
                        .ins()
                        .global_value(types::I64, *drefs.get(&format!("q{i}")).unwrap());
                    let xv = fb.ins().iconst(types::I32, *x as i64);
                    let call = fb.ins().call(f, &[xv, qp]);
                    let st = fb.inst_results(call)[0];
                    let f1 = expect_i32(&mut fb, st, *expect_status as i64);
                    acc_flag(&mut fb, f1);
                    let q = fb.ins().load(types::I64, mf, qp, 0);
                    if let Some(r) = dst_reg {
                        // success path: the handle flows onward
                        regs[*r] = q;
                        let zero = fb.ins().iconst(types::I64, 0);
                        let is_zero = fb.ins().icmp(IntCC::Equal, q, zero);
                        let fl = fb.ins().uextend(types::I32, is_zero);
                        acc_flag(&mut fb, fl);
                    } else {
                        // failure path: the sentinel must survive untouched
                        let f2 = expect_i64(&mut fb, q, *sentinel);
                        acc_flag(&mut fb, f2);
                    }
                }
                SuiteCheck::Fold {
                    symbol,
                    cb,
                    ctx,
                    items,
                    expected,
                } => {
                    let f = *frefs.get(symbol).unwrap();
                    let cbf = *cb_refs.get(cb).expect("callback kind was defined");
                    let cbp = fb.ins().func_addr(types::I64, cbf);
                    let cv = fb.ins().iconst(types::I64, *ctx);
                    let ip = fb
                        .ins()
                        .global_value(types::I64, *drefs.get(&format!("it{i}")).unwrap());
                    let nv = fb.ins().iconst(types::I64, items.len() as i64);
                    let call = fb.ins().call(f, &[cbp, cv, ip, nv]);
                    let got = fb.inst_results(call)[0];
                    let f1 = expect_i64(&mut fb, got, *expected);
                    acc_flag(&mut fb, f1);
                }
                SuiteCheck::BufUtf8Ok {
                    symbol,
                    bytes,
                    expected_chars,
                } => {
                    let f = *frefs.get(symbol).unwrap();
                    let bp = fb
                        .ins()
                        .global_value(types::I64, *drefs.get(&format!("b{i}")).unwrap());
                    let qp = fb
                        .ins()
                        .global_value(types::I64, *drefs.get(&format!("q{i}")).unwrap());
                    let lv = fb.ins().iconst(types::I64, bytes.len() as i64);
                    let call = fb.ins().call(f, &[bp, lv, qp]);
                    let st = fb.inst_results(call)[0];
                    let f1 = expect_i32(&mut fb, st, 0);
                    acc_flag(&mut fb, f1);
                    let q = fb.ins().load(types::I64, mf, qp, 0);
                    let f2 = expect_i64(&mut fb, q, *expected_chars);
                    acc_flag(&mut fb, f2);
                }
                SuiteCheck::BufUtf8Invalid {
                    symbol,
                    bytes,
                    sentinel,
                } => {
                    let f = *frefs.get(symbol).unwrap();
                    let bp = fb
                        .ins()
                        .global_value(types::I64, *drefs.get(&format!("b{i}")).unwrap());
                    let qp = fb
                        .ins()
                        .global_value(types::I64, *drefs.get(&format!("q{i}")).unwrap());
                    let lv = fb.ins().iconst(types::I64, bytes.len() as i64);
                    let call = fb.ins().call(f, &[bp, lv, qp]);
                    let st = fb.inst_results(call)[0];
                    // R797_ERR_UTF8 = -7 (interop_resource.h)
                    let f1 = expect_i32(&mut fb, st, -7);
                    acc_flag(&mut fb, f1);
                    let q = fb.ins().load(types::I64, mf, qp, 0);
                    let f2 = expect_i64(&mut fb, q, *sentinel);
                    acc_flag(&mut fb, f2);
                }
                SuiteCheck::RecValidOk {
                    symbol,
                    expected_sum,
                    ..
                } => {
                    let f = *frefs.get(symbol).unwrap();
                    let rp = fb
                        .ins()
                        .global_value(types::I64, *drefs.get(&format!("rec{i}")).unwrap());
                    let qp = fb
                        .ins()
                        .global_value(types::I64, *drefs.get(&format!("q{i}")).unwrap());
                    let call = fb.ins().call(f, &[rp, qp]);
                    let st = fb.inst_results(call)[0];
                    let f1 = expect_i32(&mut fb, st, 0);
                    acc_flag(&mut fb, f1);
                    let q = fb.ins().load(types::I64, mf, qp, 0);
                    let f2 = expect_i64(&mut fb, q, *expected_sum);
                    acc_flag(&mut fb, f2);
                }
                SuiteCheck::RecValidBad {
                    symbol, sentinel, ..
                } => {
                    let f = *frefs.get(symbol).unwrap();
                    let rp = fb
                        .ins()
                        .global_value(types::I64, *drefs.get(&format!("rec{i}")).unwrap());
                    let qp = fb
                        .ins()
                        .global_value(types::I64, *drefs.get(&format!("q{i}")).unwrap());
                    let call = fb.ins().call(f, &[rp, qp]);
                    let st = fb.inst_results(call)[0];
                    // R797_ERR_INVALID_VAL = -8
                    let f1 = expect_i32(&mut fb, st, -8);
                    acc_flag(&mut fb, f1);
                    let q = fb.ins().load(types::I64, mf, qp, 0);
                    let f2 = expect_i64(&mut fb, q, *sentinel);
                    acc_flag(&mut fb, f2);
                }
                SuiteCheck::StoredCb { symbol } => {
                    let f = *frefs.get(symbol).unwrap();
                    let call = fb.ins().call(f, &[]);
                    let got = fb.inst_results(call)[0];
                    let f1 = expect_i64(&mut fb, got, 0);
                    acc_flag(&mut fb, f1);
                }
            }
        }
        fb.ins().return_(&[flags]);
        fb.finalize();
    }
    om.define_function(main_id, &mut ctx)
        .map_err(|e| format!("define p797_main: {e}"))?;

    // Generic entry wrapper.
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
        .map_err(|e| format!("emit suite caller: {e}"))
}

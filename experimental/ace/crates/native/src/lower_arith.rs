//! PLAN-1069 (from archived PLAN-1038 D04): the single owner of the i32
//! checked-arithmetic CLIF emission shared by the collections and closures
//! native families.
//!
//! The two `Lower::binary` implementations had a 50-line byte-identical body
//! (comparison matrix, Div/Rem zero and MIN/-1 guards, i64 intermediate
//! arithmetic with the i32 range check, trap codes -30/-31). This module holds
//! that mechanism once; each family keeps its own `Lower` state, operand
//! load/store and — for collections only — the `BinOp::Num` typed-numeric lane.
//!
//! Emission order is part of the contract: the guard blocks are created in the
//! same sequence as before the extraction, so the emitted COFF bytes for a
//! fixed unit are unchanged (proven by the D04 CASE-09 probe; see PLAN-1069).

use ace_av_core::mir::BinOp;
use cranelift_codegen::ir::condcodes::IntCC;
use cranelift_codegen::ir::{types, Block, InstBuilder, Value};
use cranelift_frontend::FunctionBuilder;

/// Emit the i32 comparison/arithmetic lane for `op` over the already-loaded
/// operand values `a`/`b`, returning the value the caller stores into `dst`.
///
/// `None` means the op is not part of this mechanism (for example the typed
/// `BinOp::Num` lane, which only the collections family owns); callers keep
/// their previous rejection/dispatch for those ops.
pub(crate) fn emit_i32_binary(
    fb: &mut FunctionBuilder<'_>,
    trap: Block,
    op: BinOp,
    a: Value,
    b: Value,
) -> Option<Value> {
    let condition = match op {
        BinOp::LtI32 => Some(IntCC::SignedLessThan),
        BinOp::LeI32 => Some(IntCC::SignedLessThanOrEqual),
        BinOp::GtI32 => Some(IntCC::SignedGreaterThan),
        BinOp::GeI32 => Some(IntCC::SignedGreaterThanOrEqual),
        BinOp::EqI32 => Some(IntCC::Equal),
        BinOp::NeI32 => Some(IntCC::NotEqual),
        _ => None,
    };
    let value = if let Some(c) = condition {
        let b = fb.ins().icmp(c, a, b);
        fb.ins().uextend(types::I32, b)
    } else {
        if matches!(op, BinOp::DivI32 | BinOp::RemI32) {
            let nz = fb.ins().icmp_imm(IntCC::NotEqual, b, 0);
            let e = i32_const(fb, -31);
            guard(fb, trap, nz, e);
            let min = fb.ins().icmp_imm(IntCC::Equal, a, i32::MIN as i64);
            let neg = fb.ins().icmp_imm(IntCC::Equal, b, -1);
            let overflow = fb.ins().band(min, neg);
            let good = fb.ins().icmp_imm(IntCC::Equal, overflow, 0);
            let e = i32_const(fb, -30);
            guard(fb, trap, good, e);
        }
        let a = fb.ins().sextend(types::I64, a);
        let b = fb.ins().sextend(types::I64, b);
        let value = match op {
            BinOp::AddI32 => fb.ins().iadd(a, b),
            BinOp::SubI32 => fb.ins().isub(a, b),
            BinOp::MulI32 => fb.ins().imul(a, b),
            BinOp::DivI32 => fb.ins().sdiv(a, b),
            BinOp::RemI32 => fb.ins().srem(a, b),
            _ => return None,
        };
        let low = fb
            .ins()
            .icmp_imm(IntCC::SignedGreaterThanOrEqual, value, i32::MIN as i64);
        let high = fb
            .ins()
            .icmp_imm(IntCC::SignedLessThanOrEqual, value, i32::MAX as i64);
        let good = fb.ins().band(low, high);
        let e = i32_const(fb, -30);
        guard(fb, trap, good, e);
        fb.ins().ireduce(types::I32, value)
    };
    Some(value)
}

fn i32_const(fb: &mut FunctionBuilder<'_>, n: i32) -> Value {
    fb.ins().iconst(types::I32, n as i64)
}

/// Identical to each family's previous `Lower::guard`: a failure branches to
/// the shared trap block carrying `error`, success falls through to `next`.
fn guard(fb: &mut FunctionBuilder<'_>, trap: Block, okay: Value, error: Value) {
    let next = fb.create_block();
    fb.ins().brif(okay, next, &[], trap, &[error.into()]);
    fb.switch_to_block(next);
}

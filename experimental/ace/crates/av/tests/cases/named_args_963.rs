//! PLAN-963（T-08，V-08）：命名实参**源序求值**（不是参数序）。
//!
//! 绑定层把给定实参按源序保留、再用 binding permute 到参数位；本文件用
//! Typed MIR 的指令序作结构见证（两个实参各是一次真实调用），并用 AV 执行
//! 结果证明显式绑定正确：
//!
//! * 源序 `sub(b: five(), a: twenty())` —— `five` 的调用先于 `twenty`；
//! * 结果 = 20 - 5 = 15（`a`/`b` 按名字绑定，不按位置）。

use ace_av_core::av::{execute, CancelFlag, ExecLimits};
use ace_av_core::contract::{ExecOutcome, Value};
use ace_av_core::mir::{CalleeRef, MirInstr};
use ace_av_core::{abc_verify, mir_verify};
use ace_av_core::abc::{decode_abc, encode_abc};
use ace_core::source::compile_record_source;

const NAMED_OUT_OF_ORDER: &str = "\
fn sub(a int, b int) int { return a - b }
fn twenty() int { return 20 }
fn five() int { return 5 }
fn main() int {
    return sub(b: five(), a: twenty())
}
";

fn mir_of(src: &str) -> ace_av_core::mir::MirModule {
    let compiled = compile_record_source(src, "n963.at", &[]).expect("named-argument source compiles");
    let mir = ace_lowering::lower_checked_data(&compiled.checked, "d_main", 963)
        .expect("record-domain lowering");
    mir_verify::verify_mir(&mir).expect("MIR verifies");
    mir
}

/// O-963-N1：命名实参逆序给出时，求值顺序仍为**源序**（five 先于 twenty），
/// 而值按名字绑定 → 20 - 5 = 15。
#[test]
fn named_arguments_evaluate_in_source_order_963() {
    let mir = mir_of(NAMED_OUT_OF_ORDER);
    let entry = mir
        .funcs
        .iter()
        .find(|f| f.name == "d_main")
        .expect("entry function present in MIR");

    let call_pos = |want: &str| {
        // block-major flat instruction order (same order the interpreter runs)
        entry
            .blocks
            .iter()
            .flat_map(|b| b.instrs.iter())
            .position(|i| match i {
                MirInstr::Call {
                    callee: CalleeRef::Func(idx),
                    ..
                } => mir
                    .funcs
                    .get(*idx as usize)
                    .map(|f| f.name == want)
                    .unwrap_or(false),
                _ => false,
            })
    };
    let five = call_pos("d_five").expect("call to d_five");
    let twenty = call_pos("d_twenty").expect("call to d_twenty");
    assert!(
        five < twenty,
        "named arguments must evaluate in SOURCE order (five before twenty), got five@{five} twenty@{twenty}"
    );

    let wire = encode_abc(&mir).expect("AVAD encode").to_flat();
    let decoded = decode_abc(&wire).expect("AVAD decode");
    let loaded = abc_verify::verify_abc(decoded, &mir.owner).expect("AVAD admission");
    let (out, _trace) = execute(&loaded, &ExecLimits::default(), &CancelFlag::new(false));
    let ExecOutcome::Success { value, .. } = out else {
        panic!("expected success, got {out:?}")
    };
    assert_eq!(value, Value::I32(15), "named binding: a=twenty, b=five");
}

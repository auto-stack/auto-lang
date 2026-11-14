//! PLAN-963（T-08，L-24，AV 端）：声明位默认实参 `fn rep(x int, n int = 3)`。
//!
//! 补位发生在绑定层：缺席实参按声明位解析成**普通实参**，之后与显式实参同路
//! （Typed MIR → AVAD ABC → 真实 AV 执行），因此 AV 端不需要任何默认值专用
//! 指令——本文件用闭式 oracle 证明补位后的值真的参与执行。
//!
//! oracle（手算闭式）：`rep(14)` = 14*3 = 42；`rep(14, 2)` = 28；
//! 两调用相加 = 70（同时证明显式实参不被默认值覆盖）。
//! 命名式补洞：`box(b: 2)` 的 `a` 取默认 40 → 42。

use ace_av_core::abc::{decode_abc, encode_abc};
use ace_av_core::av::{execute, CancelFlag, ExecLimits};
use ace_av_core::contract::{ExecOutcome, Value, AV_DATA_PROFILE};
use ace_av_core::mir::{CalleeRef, MirInstr};
use ace_av_core::{abc_verify, mir_verify};
use ace_core::source::compile_record_source;
use ace_core::verify::CheckedModule;

const OMITTED_DEFAULT: &str = "\
fn rep(x int, n int = 3) int { return x * n }
fn main() int {
    return rep(14)
}
";

const EXPLICIT_OVERRIDE: &str = "\
fn rep(x int, n int = 3) int { return x * n }
fn main() int {
    let a int = rep(14)
    let b int = rep(14, 2)
    return a + b
}
";

const NAMED_HOLE: &str = "\
fn box(a int = 40, b int = 2) int { return a + b }
fn main() int {
    return box(b: 2)
}
";

fn checked(src: &str) -> CheckedModule {
    compile_record_source(src, "d963.at", &[])
        .expect("default-argument source compiles")
        .checked
}

fn run_value(src: &str) -> Value {
    let checked = checked(src);
    let mir =
        ace_lowering::lower_checked_data(&checked, "d_main", 963).expect("record-domain lowering");
    assert_eq!(mir.profile, AV_DATA_PROFILE);
    mir_verify::verify_mir(&mir).expect("MIR verifies");
    let wire = encode_abc(&mir).expect("AVAD encode").to_flat();
    let decoded = decode_abc(&wire).expect("AVAD decode");
    let loaded = abc_verify::verify_abc(decoded, &mir.owner).expect("AVAD admission");
    let (out, _trace) = execute(&loaded, &ExecLimits::default(), &CancelFlag::new(false));
    let ExecOutcome::Success { value, .. } = out else {
        panic!("expected success, got {out:?}")
    };
    value
}

/// O-963-D7：省略末尾默认实参 → 42。
#[test]
fn av_default_argument_omitted_executes_42() {
    assert_eq!(
        run_value(OMITTED_DEFAULT),
        Value::I32(42),
        "closed-form oracle"
    );
}

/// O-963-D8：显式实参覆盖默认值，两调用相加 → 70。
#[test]
fn av_default_argument_explicit_override_executes_70() {
    assert_eq!(
        run_value(EXPLICIT_OVERRIDE),
        Value::I32(70),
        "closed-form oracle"
    );
}

/// O-963-D9：命名式补洞（省略 `a`）→ 40 + 2 = 42。
#[test]
fn av_default_argument_named_hole_executes_42() {
    assert_eq!(run_value(NAMED_HOLE), Value::I32(42), "closed-form oracle");
}

const DEFAULT_BY_CALL: &str = "\
fn seven() int { return 7 }
fn twice(x int, n int = seven()) int { return x * n }
fn main() int {
    return twice(6)
}
";

/// O-963-D17（V-08「默认实参只求值一次」）：补位后的默认实参是一次普通实参
/// 求值——`d_main` 的 block-major 指令序里 `d_seven` 调用恰出现 **1** 次
/// （既不重复求值，也不把求值搬进被调方），且 AV 执行 = 6*7 = 42。
#[test]
fn av_default_argument_evaluates_exactly_once_963() {
    let checked = checked(DEFAULT_BY_CALL);
    let mir =
        ace_lowering::lower_checked_data(&checked, "d_main", 963).expect("record-domain lowering");
    mir_verify::verify_mir(&mir).expect("MIR verifies");
    let entry = mir
        .funcs
        .iter()
        .find(|f| f.name == "d_main")
        .expect("entry function present in MIR");
    let default_calls = entry
        .blocks
        .iter()
        .flat_map(|b| b.instrs.iter())
        .filter(|i| match i {
            MirInstr::Call {
                callee: CalleeRef::Func(idx),
                ..
            } => mir
                .funcs
                .get(*idx as usize)
                .map(|f| f.name == "d_seven")
                .unwrap_or(false),
            _ => false,
        })
        .count();
    assert_eq!(
        default_calls, 1,
        "the default expression must be evaluated exactly once per call, got {default_calls}"
    );
    assert_eq!(run_value(DEFAULT_BY_CALL), Value::I32(42), "6 * 7");
}

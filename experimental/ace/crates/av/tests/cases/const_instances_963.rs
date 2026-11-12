//! PLAN-963 T-04 / V-G04（AV 端）：值参数（`<const N>`）实例的具体 Checked
//! HIR 经 MIR → ABC → 真实 AV 执行。
//!
//! oracle（闭式手算）：`rep<const N>(x) = N * x`
//! - `rep<2>(21)` = 42；`rep<3>(21)` = 63；
//! - `rep<2>(21) + rep<3>(21)` = 105（两个值实例各自真实执行）。

use ace_av_core::av::{execute, CancelFlag, ExecLimits};
use ace_av_core::contract::{ExecOutcome, Value, AV_PROFILE};
use ace_av_core::{abc_verify, decode_abc, encode_abc};
use ace_core::source::cst::INSTANCES_CAPABILITY;
use ace_core::verify::CheckedModule;
use ace_source_query::{compile_scalar_product, CompileLane, SourceQuerySession};

const REP_TWO: &str = "\
fn rep<const N>(x int) int { return N * x }
fn main() int { return rep<2>(21) }
";

const REP_THREE: &str = "\
fn rep<const N>(x int) int { return N * x }
fn main() int { return rep<3>(21) }
";

const REP_BOTH: &str = "\
fn rep<const N>(x int) int { return N * x }
fn main() int { return rep<2>(21) + rep<3>(21) }
";

fn compile_checked(src: &str, want_instances: usize) -> CheckedModule {
    let mut session = SourceQuerySession::new("plan963", "constrep.at");
    let p = compile_scalar_product(
        src,
        "constrep.at",
        &[INSTANCES_CAPABILITY.to_string()],
        "plan963",
        "constrep.at",
        &mut session,
    )
    .expect("instances-lane compile");
    assert_eq!(p.lane, CompileLane::Instances);
    assert_eq!(p.instances.len(), want_instances, "value instances");
    p.compiled.checked
}

fn run_value(checked: &CheckedModule) -> Value {
    let mir = ace_lowering::lower_checked(checked, "d_main", 963).expect("lower real HIR");
    assert_eq!(mir.profile, AV_PROFILE);
    ace_av_core::verify_mir(&mir).expect("MIR verifies");
    let wire = encode_abc(&mir).expect("ABC encode").to_flat();
    let decoded = decode_abc(&wire).expect("ABC decode");
    let loaded = abc_verify::verify_abc(decoded, &mir.owner).expect("ABC admission");
    let (out, _trace) = execute(&loaded, &ExecLimits::default(), &CancelFlag::new(false));
    let ExecOutcome::Success { value, .. } = out else {
        panic!("expected success, got {out:?}")
    };
    value
}

/// V-G04：N=2 → 42（值已代入实例体）。
#[test]
fn av_const_value_two_executes_42() {
    let v = run_value(&compile_checked(REP_TWO, 1));
    assert_eq!(v, Value::I32(42), "rep<2>(21)");
}

/// V-G04：N=3 → 63（不同值 = 不同实例）。
#[test]
fn av_const_value_three_executes_63() {
    let v = run_value(&compile_checked(REP_THREE, 1));
    assert_eq!(v, Value::I32(63), "rep<3>(21)");
}

/// V-G04：同一模板两个值实例 → 105（身份不串体）。
#[test]
fn av_two_values_execute_105() {
    let v = run_value(&compile_checked(REP_BOTH, 2));
    assert_eq!(v, Value::I32(105), "rep<2>(21) + rep<3>(21)");
}

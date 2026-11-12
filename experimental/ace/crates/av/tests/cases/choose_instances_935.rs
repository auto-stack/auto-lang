//! PLAN-935 T-02（AV 端）：choose 双实例经实例 lane → 具体 Checked HIR →
//! MIR → ABC → **真实 AV 执行**。
//!
//! oracle（Design42 §10.3 闭式，独立手算）：原模板 main=42；交换模板两个
//! return 分支（调用点不动）main=0。两个方向都必须真实执行，不是模型 echo。

use ace_av_core::av::{execute, CancelFlag, ExecLimits};
use ace_av_core::contract::{ExecOutcome, Value, AV_PROFILE};
use ace_av_core::{abc_verify, decode_abc, encode_abc};
use ace_core::source::cst::INSTANCES_CAPABILITY;
use ace_core::verify::CheckedModule;
use ace_source_query::{compile_scalar_product, CompileLane, SourceQuerySession};

const CHOOSE: &str = "\
fn choose<T>(a T, b T, flag bool) T {
    if flag { return a }
    return b
}
fn main() int {
    if choose(true, false, true) { return choose(40, 42, false) }
    return 0
}
";

const CHOOSE_SWAPPED: &str = "\
fn choose<T>(a T, b T, flag bool) T {
    if flag { return b }
    return a
}
fn main() int {
    if choose(true, false, true) { return choose(40, 42, false) }
    return 0
}
";

/// PLAN-963 T-02：多类型参数 `<T, U>` 实例在 AV 端真实执行。
/// `pick(true, 7, false)` 解 T=bool/U=int → true；`pick(42, true, false)`
/// 解 T=int/U=bool → 42；main 闭式结果 42。
const PICK963: &str = "\
fn pick<T, U>(a T, b U, flag bool) T {
    if flag { return a }
    return a
}
fn main() int {
    if pick(true, 7, false) { return pick(42, true, false) }
    return 0
}
";

fn compile_checked(src: &str) -> CheckedModule {
    let mut session = SourceQuerySession::new("plan935", "choose.at");
    let p = compile_scalar_product(
        src,
        "choose.at",
        &[INSTANCES_CAPABILITY.to_string()],
        "plan935",
        "choose.at",
        &mut session,
    )
    .expect("instances-lane compile");
    assert_eq!(p.lane, CompileLane::Instances);
    assert_eq!(p.instances.len(), 2, "dual instances (T=int/T=bool)");
    p.compiled.checked
}

fn run_value(checked: &CheckedModule) -> Value {
    let mir = ace_lowering::lower_checked(checked, "d_main", 935).expect("lower real HIR");
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

/// O-3：choose(40,42,false)=42 经 bool 实例守卫的真实执行。
#[test]
fn av_choose_original_executes_42() {
    let v = run_value(&compile_checked(CHOOSE));
    assert_eq!(v, Value::I32(42), "closed-form oracle (Design42 §10.3)");
}

/// O-4：交换模板分支（调用点不动）→ main=0——实例体真实换版。
#[test]
fn av_choose_swapped_executes_0() {
    let v = run_value(&compile_checked(CHOOSE_SWAPPED));
    assert_eq!(v, Value::I32(0), "swapped-branch oracle");
}

/// O-963：多类型参数实例（两种排列）在 AV 端真实执行 → 42。
#[test]
fn av_multi_param_instances_execute_42() {
    let v = run_value(&compile_checked(PICK963));
    assert_eq!(v, Value::I32(42), "T/U 两排列实例闭式 42");
}

//! PLAN-963（T-02，AV 端）：用户泛型记录 `type Point<T>` 的每个使用位显式写
//! 类型实参，投影期单态化为具体 nominal 记录（`Point_int`），之后与普通记录
//! 同路：resolve/布局/Checked HIR → Typed MIR → AVAD ABC → **真实 AV 执行**。
//!
//! oracle（手算闭式）：`Point.both(150)` 两个字段都是 150，`first()+y` = 300；
//! 换模板体（`y: v + 1`，调用点不动）= 301——证明实例体真实换版，不是常量回显。

use ace_av_core::abc::{decode_abc, encode_abc};
use ace_av_core::av::{execute, CancelFlag, ExecLimits};
use ace_av_core::contract::{ExecOutcome, Value, AV_DATA_PROFILE};
use ace_av_core::{abc_verify, mir_verify};
use ace_core::source::compile_record_source;
use ace_core::verify::CheckedModule;

/// 单态化实例 + 方法：静态构造（返回 `Point<T>`）→ 实例方法返回 `T`。
const POINT963: &str = "\
type Point<T> { x T; y T
    static fn both(v T) Point<T> { return Point { x: v, y: v } }
    fn first() T { return .x }
}
fn main() int {
    let p Point<int> = Point.both(150)
    return p.first() + p.y
}
";

/// 交换模板体（`y: v + 1`）：同一调用点 → 301。
const POINT963_SHIFTED: &str = "\
type Point<T> { x T; y T
    static fn both(v T) Point<T> { return Point { x: v, y: v + 1 } }
    fn first() T { return .x }
}
fn main() int {
    let p Point<int> = Point.both(150)
    return p.first() + p.y
}
";

fn checked(src: &str) -> CheckedModule {
    compile_record_source(src, "p963.at", &[])
        .expect("generic record source compiles")
        .checked
}

fn run_value(src: &str) -> Value {
    let checked = checked(src);
    let mir = ace_lowering::lower_checked_data(&checked, "d_main", 963)
        .expect("record-domain lowering");
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

/// P1：泛型记录字段 + 方法经真实 AV 执行 → 300。
#[test]
fn av_generic_record_executes_300() {
    assert_eq!(run_value(POINT963), Value::I32(300), "closed-form oracle");
}

/// 对照：模板体改动（调用点不动）→ 301，实例体真实参与执行。
#[test]
fn av_generic_record_body_change_executes_301() {
    assert_eq!(
        run_value(POINT963_SHIFTED),
        Value::I32(301),
        "shifted-template oracle"
    );
}

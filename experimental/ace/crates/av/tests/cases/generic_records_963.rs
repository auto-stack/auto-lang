//! PLAN-963（T-02，AV 端）：用户泛型记录 `type Point<T>` 的每个使用位显式写
//! 类型实参，投影期单态化为具体 nominal 记录（`Point_int`），之后与普通记录
//! 同路：resolve/布局/Checked HIR → Typed MIR → AVAD ABC → **真实 AV 执行**。
//!
//! oracle（手算闭式）：`Point<int> { x: 100, y: 200 }` 的 `sum()` = 300；换模板
//! 方法体（`+ 1`，构造点不动）= 301——证明实例体真实参与执行，不是常量回显。

use ace_av_core::abc::{decode_abc, encode_abc};
use ace_av_core::av::{execute, CancelFlag, ExecLimits};
use ace_av_core::contract::{ExecOutcome, Value, AV_DATA_PROFILE};
use ace_av_core::{abc_verify, mir_verify};
use ace_core::source::compile_record_source;
use ace_core::verify::CheckedModule;

/// P1（计划原拼写）：显式类型实参构造 + 泛型方法 → 300。
const POINT963: &str = "\
type Point<T> { x T; y T
    fn sum() T { return .x + .y }
}
fn main() int {
    let p = Point<int> { x: 100, y: 200 }
    return p.sum()
}
";

/// 对照：模板方法体 +1（构造点不动）→ 301。
const POINT963_SHIFTED: &str = "\
type Point<T> { x T; y T
    fn sum() T { return .x + .y + 1 }
}
fn main() int {
    let p = Point<int> { x: 100, y: 200 }
    return p.sum()
}
";

/// 两个实参各自单态化（`Point<int>` + `Point<bool>`）+ 模板静态构造 → 300。
const POINT963_TWO_INSTANCES: &str = "\
type Point<T> { x T; y T
    static fn both(v T) Point<T> { return Point { x: v, y: v } }
    fn first() T { return .x }
}
fn main() int {
    let p Point<int> = Point.both(150)
    let q = Point<bool> { x: true, y: false }
    if q.first() { return p.first() + p.y } else { return 0 }
}
";

fn checked(src: &str) -> CheckedModule {
    compile_record_source(src, "p963.at", &[])
        .expect("generic record source compiles")
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

/// P1：泛型记录字段 + 方法经真实 AV 执行 → 300。
#[test]
fn av_generic_record_executes_300() {
    assert_eq!(run_value(POINT963), Value::I32(300), "closed-form oracle");
}

/// 对照：模板方法体改动（构造点不动）→ 301，实例体真实参与执行。
#[test]
fn av_generic_record_body_change_executes_301() {
    assert_eq!(
        run_value(POINT963_SHIFTED),
        Value::I32(301),
        "shifted-template oracle"
    );
}

/// 同一模板的 int/bool 两个实参各自合成、各自执行 → 300。
#[test]
fn av_generic_record_two_instances_execute_300() {
    assert_eq!(
        run_value(POINT963_TWO_INSTANCES),
        Value::I32(300),
        "two-instantiation oracle"
    );
}

// ---------------------------------------------------------------------------
// PLAN-963（T-03）：spec 声明 + `as` 遵从（投影期擦除 + 默认方法注入）
// ---------------------------------------------------------------------------

/// 显式实现：`Score { val: 40 }.compare(3)` = 37（AV 真实执行）。
const SPEC_EXPLICIT: &str = "\
spec Comparable { fn compare(other int) int }
type Score as Comparable {
    val int
    fn compare(other int) int { return .val - other }
}
fn main() int {
    let s Score = Score { val: 40 }
    return s.compare(3)
}
";

/// 默认体注入：实现者未声明 `greet`，调用命中 spec 默认体 → 7。
const SPEC_DEFAULT: &str = "\
spec Greeter { fn greet() int { return 7 } }
type Thing as Greeter { v int }
fn main() int {
    let t Thing = Thing { v: 1 }
    return t.greet()
}
";

/// 显式覆盖默认体：`greet` = `.v + 8` → 9（与默认体 7 不同，证明覆盖生效）。
const SPEC_OVERRIDE: &str = "\
spec Greeter { fn greet() int { return 7 } }
type Thing as Greeter { v int
    fn greet() int { return .v + 8 }
}
fn main() int {
    let t Thing = Thing { v: 1 }
    return t.greet()
}
";

/// 显式实现经真实 AV 执行 → 37。
#[test]
fn av_spec_explicit_impl_executes_37() {
    assert_eq!(run_value(SPEC_EXPLICIT), Value::I32(37), "spec oracle");
}

/// 默认方法注入经真实 AV 执行 → 7。
#[test]
fn av_spec_default_body_executes_7() {
    assert_eq!(
        run_value(SPEC_DEFAULT),
        Value::I32(7),
        "default-body oracle"
    );
}

/// 显式覆盖默认体 → 9（不是默认体的 7）。
#[test]
fn av_spec_override_executes_9() {
    assert_eq!(
        run_value(SPEC_OVERRIDE),
        Value::I32(9),
        "override oracle (not the default body)"
    );
}

// ---------------------------------------------------------------------------
// PLAN-963（T-03）：`<T has S>` 约束经单态化核验后仍真实执行
// ---------------------------------------------------------------------------

/// `Box<T has Summable>` 以遵从记录实例化：`Score { val: 5 }.total()` = 5。
const BOUNDED_WRAPPER: &str = "\
spec Summable { fn total() int }
type Score as Summable { val int
    fn total() int { return .val }
}
type Box<T has Summable> { inner T }
fn main() int {
    let b Box<Score> = Box { inner: Score { val: 5 } }
    let s Score = b.inner
    return s.total()
}
";

/// 约束满足的 wrapper（真实 AV 执行）→ 5。
#[test]
fn av_bounded_wrapper_executes_5() {
    assert_eq!(run_value(BOUNDED_WRAPPER), Value::I32(5), "bounded oracle");
}

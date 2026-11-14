//! PLAN-963（T-02，native 端）：用户泛型记录的合成具体记录经 **同一 mn 管线**
//! 真实构建+运行（`MnDriver::request_checked`），exit code = main 结果。
//!
//! oracle：`Point<int> { x: 100, y: 200 }` 的 `sum()` → 300；换模板方法体
//! （`+ 1`，构造点不动）→ 301。

use ace_native::driver::MnDriver;

const POINT963: &str = "\
type Point<T> { x T; y T
    fn sum() T { return .x + .y }
}
fn main() int {
    let p = Point<int> { x: 100, y: 200 }
    return p.sum()
}
";

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

fn run_native(src: &str, tag: &str) -> i32 {
    let checked = ace_core::source::compile_record_source(src, "p963.at", &[])
        .expect("generic record source compiles")
        .checked;
    let tag = format!("mn963-{tag}-{}", std::process::id());
    let mn_dir = std::env::temp_dir().join(format!("{tag}-mn"));
    let _ = std::fs::remove_dir_all(&mn_dir);
    let snapshot = ace_av_core::contract::fnv1a64(src.as_bytes());
    let mut driver = MnDriver::ephemeral();
    let mn = driver.request_checked(&checked, "d_main", &mn_dir, None, snapshot);
    let _ = std::fs::remove_dir_all(&mn_dir);
    assert!(mn.ok, "mn leg failed: {:?}", mn.error);
    mn.exit_code.unwrap_or(-1)
}

/// P1（native 端）：泛型记录字段 + 方法 → exit=300。
#[test]
fn native_generic_record_exit_300() {
    assert_eq!(run_native(POINT963, "p1"), 300);
}

/// 对照（native 端）：模板方法体改动（构造点不动）→ exit=301。
#[test]
fn native_generic_record_body_change_exit_301() {
    assert_eq!(run_native(POINT963_SHIFTED, "p1s"), 301);
}

/// 同一模板的 int/bool 两个实参各自合成、各自执行 → exit=300。
#[test]
fn native_generic_record_two_instances_exit_300() {
    assert_eq!(run_native(POINT963_TWO_INSTANCES, "p1d"), 300);
}

// ---------------------------------------------------------------------------
// PLAN-963（T-03）：spec 声明 + `as` 遵从（投影期擦除 + 默认方法注入）
// ---------------------------------------------------------------------------

/// 显式实现：`Score { val: 40 }.compare(3)` = 37。
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

/// 默认体注入：实现者未声明 `greet` → 7。
const SPEC_DEFAULT: &str = "\
spec Greeter { fn greet() int { return 7 } }
type Thing as Greeter { v int }
fn main() int {
    let t Thing = Thing { v: 1 }
    return t.greet()
}
";

/// 显式覆盖默认体 → 9。
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

/// 显式实现（native 真实构建+运行）→ exit=37。
#[test]
fn native_spec_explicit_impl_exit_37() {
    assert_eq!(run_native(SPEC_EXPLICIT, "s1"), 37);
}

/// 默认方法注入（native）→ exit=7。
#[test]
fn native_spec_default_body_exit_7() {
    assert_eq!(run_native(SPEC_DEFAULT, "s2"), 7);
}

/// 显式覆盖默认体（native）→ exit=9。
#[test]
fn native_spec_override_exit_9() {
    assert_eq!(run_native(SPEC_OVERRIDE, "s3"), 9);
}

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

/// 约束满足的 wrapper（native 真实构建运行）→ exit=5。
#[test]
fn native_bounded_wrapper_exit_5() {
    assert_eq!(run_native(BOUNDED_WRAPPER, "b1"), 5);
}

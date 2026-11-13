//! PLAN-963 T-03 / V-G05（native 端，类型别名）：`type Age = int`、
//! `type P = Point<int>`、`type Q = P` 在投影期展开后，同一 mn 管线真实
//! 构建+运行，exit code = main 结果（别名不残留到后端）。
//!
//! oracle（闭式手算）：
//! - `bump(41)` = 42；
//! - `q.y = 60`，`q.y > 50` 成立 → 42 + 60 = 102。

use ace_native::driver::MnDriver;

const ALIASED: &str = "\
type Age = int
type P = Point<int>
type Q = P
type Point<T> { x T; y T }
fn bump(a Age) Age { return a + 1 }
fn main() int {
    let q Q = Point { x: 41, y: 60 }
    let a Age = bump(q.x)
    if q.y > 50 { return a + q.y }
    return 0
}
";

/// V-G05：别名（含别名链与泛型目标）真实执行 → 102。
#[test]
fn native_alias_chain_exit_102() {
    let checked = ace_core::source::compile_record_source(ALIASED, "alias963.at", &[])
        .expect("alias program compiles")
        .checked;
    let tag = format!("mn963a-{}", std::process::id());
    let mn_dir = std::env::temp_dir().join(format!("{tag}-mn"));
    let _ = std::fs::remove_dir_all(&mn_dir);
    let snapshot = ace_av_core::contract::fnv1a64(ALIASED.as_bytes());
    let mut driver = MnDriver::ephemeral();
    let mn = driver.request_checked(&checked, "d_main", &mn_dir, None, snapshot);
    let _ = std::fs::remove_dir_all(&mn_dir);
    assert!(mn.ok, "mn leg failed: {:?}", mn.error);
    assert_eq!(mn.exit_code.unwrap_or(-1), 102);
}

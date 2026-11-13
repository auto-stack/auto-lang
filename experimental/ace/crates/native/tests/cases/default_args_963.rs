//! PLAN-963（T-08，L-24，native 端）：声明位默认实参经同一 mn 管线真实构建+运行，
//! exit code = main 结果。
//!
//! oracle（手算闭式）：`rep(14)` = 14*3 = 42；`rep(14)`+`rep(14,2)` = 42+28 = 70；
//! 命名式补洞 `box(b: 2)` = 40+2 = 42。

use ace_native::driver::MnDriver;

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

fn run_native(src: &str, tag: &str) -> i32 {
    let checked = ace_core::source::compile_record_source(src, "d963.at", &[])
        .expect("default-argument source compiles")
        .checked;
    let tag = format!("mn963d-{tag}-{}", std::process::id());
    let mn_dir = std::env::temp_dir().join(format!("{tag}-mn"));
    let _ = std::fs::remove_dir_all(&mn_dir);
    let snapshot = ace_av_core::contract::fnv1a64(src.as_bytes());
    let mut driver = MnDriver::ephemeral();
    let mn = driver.request_checked(&checked, "d_main", &mn_dir, None, snapshot);
    let _ = std::fs::remove_dir_all(&mn_dir);
    assert!(mn.ok, "mn leg failed: {:?}", mn.error);
    mn.exit_code.unwrap_or(-1)
}

/// O-963-D10：省略末尾默认实参 → exit 42。
#[test]
fn native_default_argument_omitted_exit_42() {
    assert_eq!(run_native(OMITTED_DEFAULT, "d10"), 42);
}

/// O-963-D11：显式实参覆盖默认值，两调用相加 → exit 70。
#[test]
fn native_default_argument_explicit_override_exit_70() {
    assert_eq!(run_native(EXPLICIT_OVERRIDE, "d11"), 70);
}

/// O-963-D12：命名式补洞 → exit 42。
#[test]
fn native_default_argument_named_hole_exit_42() {
    assert_eq!(run_native(NAMED_HOLE, "d12"), 42);
}

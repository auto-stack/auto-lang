//! PLAN-963 T-04 / V-G04（native 端）：值参数（`<const N>`）实例经实例 lane
//! 产出的具体 Checked HIR 由同一 mn 管线真实构建+运行，exit code = main 结果。
//!
//! oracle（闭式手算）：`rep<const N>(x) = N * x`
//! - `rep<2>(21)` = 42；
//! - `rep<3>(21)` = 63；
//! - `rep<2>(21) + rep<3>(21)` = 105（两个值实例各自真实执行）。
//!
//! 值实参进实例身份：N=2/N=3 是两个实例（不同装配名 `rep_u32_2`/`rep_u32_3`）。

use ace_av::query_entry;
use ace_core::source::cst::INSTANCES_CAPABILITY;
use ace_native::driver::MnDriver;

const REP: &str = "\
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

fn run_native(src: &str, tag: &str, want_instances: usize) -> i32 {
    let caps = [INSTANCES_CAPABILITY.to_string()];
    let p = query_entry::compile_scalar_source_product(src, "constrep.at", &caps, "plan963")
        .expect("instances-lane compile");
    assert_eq!(
        p.instances.len(),
        want_instances,
        "value instances: {:?}",
        p.instances
    );
    let tag = format!("mn963c-{tag}-{}", std::process::id());
    let mn_dir = std::env::temp_dir().join(format!("{tag}-mn"));
    let _ = std::fs::remove_dir_all(&mn_dir);
    let snapshot = ace_av_core::contract::fnv1a64(src.as_bytes());
    let mut driver = MnDriver::ephemeral();
    let mn = driver.request_checked(&p.compiled.checked, "d_main", &mn_dir, None, snapshot);
    let _ = std::fs::remove_dir_all(&mn_dir);
    assert!(mn.ok, "mn leg failed: {:?}", mn.error);
    mn.exit_code.unwrap_or(-1)
}

/// V-G04：N=2 → 42（真实执行，体里的 N 已代入字面量）。
#[test]
fn native_const_value_two_exit_42() {
    assert_eq!(run_native(REP, "n2", 1), 42);
}

/// V-G04：N=3 → 63（不同值 = 不同实例，布局/值随值实参变）。
#[test]
fn native_const_value_three_exit_63() {
    assert_eq!(run_native(REP_THREE, "n3", 1), 63);
}

/// V-G04：两个值实例各自执行 → 42 + 63 = 105（同模板不同身份不串体）。
#[test]
fn native_const_two_values_exit_105() {
    assert_eq!(run_native(REP_BOTH, "both", 2), 105);
}

//! PLAN-935 T-02（native 端）：choose 双实例经实例 lane 产出的具体
//! Checked HIR 由 **同一 mn 管线消费**（`MnDriver::request_checked`——
//! 调用方自带 checked 模块），真实构建+运行，exit code = main 结果。
//!
//! oracle：原模板 exit=42；交换模板分支（调用点不动）exit=0。

use ace_av::query_entry;
use ace_core::source::cst::INSTANCES_CAPABILITY;
use ace_native::driver::MnDriver;

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

fn run_native(src: &str, tag: &str) -> i32 {
    let caps = [INSTANCES_CAPABILITY.to_string()];
    let p = query_entry::compile_scalar_source_product(src, "choose.at", &caps, "plan935")
        .expect("instances-lane compile");
    assert_eq!(p.instances.len(), 2, "dual instances (T=int/T=bool)");
    let tag = format!("mn935-{tag}-{}", std::process::id());
    let mn_dir = std::env::temp_dir().join(format!("{tag}-mn"));
    let _ = std::fs::remove_dir_all(&mn_dir);
    let snapshot = ace_av_core::contract::fnv1a64(src.as_bytes());
    let mut driver = MnDriver::ephemeral();
    let mn = driver.request_checked(&p.compiled.checked, "d_main", &mn_dir, None, snapshot);
    let _ = std::fs::remove_dir_all(&mn_dir);
    assert!(mn.ok, "mn leg failed: {:?}", mn.error);
    mn.exit_code.unwrap_or(-1)
}

/// O-3（native 端）：原模板 exit=42。
#[test]
fn native_choose_original_exit_42() {
    assert_eq!(run_native(CHOOSE, "orig"), 42);
}

/// O-4（native 端）：交换模板分支 → exit=0。
#[test]
fn native_choose_swapped_exit_0() {
    assert_eq!(run_native(CHOOSE_SWAPPED, "swap"), 0);
}

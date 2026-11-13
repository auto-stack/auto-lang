//! PLAN-963 T-03 / V-G05（native 端，`ext` 扩展块）：扩展方法在投影期合入
//! 具体记录，经同一 mn 管线真实构建+运行，exit code = main 结果。
//!
//! oracle（闭式手算）：`Heap { ptr: 40 }`，`bump(2) = .ptr + by = 42`。

use ace_native::driver::MnDriver;

const EXT_PROG: &str = "\
type Heap { ptr int }
ext Heap {
    fn get() int { return .ptr }
    fn bump(by int) int { return .ptr + by }
}
fn main() int {
    let h Heap = Heap { ptr: 40 }
    return h.bump(h.get() - 38)
}
";

/// V-G05：ext 方法真实执行 → 42。
#[test]
fn native_ext_method_exit_42() {
    let checked = ace_core::source::compile_record_source(EXT_PROG, "ext963.at", &[])
        .expect("ext program compiles")
        .checked;
    let tag = format!("mn963e-{}", std::process::id());
    let mn_dir = std::env::temp_dir().join(format!("{tag}-mn"));
    let _ = std::fs::remove_dir_all(&mn_dir);
    let snapshot = ace_av_core::contract::fnv1a64(EXT_PROG.as_bytes());
    let mut driver = MnDriver::ephemeral();
    let mn = driver.request_checked(&checked, "d_main", &mn_dir, None, snapshot);
    let _ = std::fs::remove_dir_all(&mn_dir);
    assert!(mn.ok, "mn leg failed: {:?}", mn.error);
    assert_eq!(mn.exit_code.unwrap_or(-1), 42);
}

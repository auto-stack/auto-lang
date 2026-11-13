//! PLAN-963 T-03 / V-G05（native 端，通用 spec 实例化）：`spec S<T>` 的默认
//! 体按 `as S<int>` 的实参具体化后注入，经 mn 管线真实构建+运行。
//!
//! oracle（闭式手算）：`pick(41, 7) = a = 41`，`+ v(1)` = 42。

use ace_native::driver::MnDriver;

const SPEC_GENERIC_PROG: &str = "\
spec Picker<T> { fn pick(a T, b T) T { return a } }
type AsInt as Picker<int> { v int }
fn main() int {
    let x AsInt = AsInt { v: 1 }
    return x.pick(41, 7) + x.v
}
";

/// V-G05：通用 spec 默认体真实执行 → 42。
#[test]
fn native_generic_spec_default_body_exit_42() {
    let checked = ace_core::source::compile_record_source(SPEC_GENERIC_PROG, "spec963.at", &[])
        .expect("generic spec program compiles")
        .checked;
    let tag = format!("mn963s-{}", std::process::id());
    let mn_dir = std::env::temp_dir().join(format!("{tag}-mn"));
    let _ = std::fs::remove_dir_all(&mn_dir);
    let snapshot = ace_av_core::contract::fnv1a64(SPEC_GENERIC_PROG.as_bytes());
    let mut driver = MnDriver::ephemeral();
    let mn = driver.request_checked(&checked, "d_main", &mn_dir, None, snapshot);
    let _ = std::fs::remove_dir_all(&mn_dir);
    assert!(mn.ok, "mn leg failed: {:?}", mn.error);
    assert_eq!(mn.exit_code.unwrap_or(-1), 42);
}

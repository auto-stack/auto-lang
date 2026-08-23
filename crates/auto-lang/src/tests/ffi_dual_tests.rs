// Plan 212 Phase 3D.1: FFI Dual-Test Infrastructure
//
// Tests that FFI functions produce consistent output through the AutoVM path.
// Each test reads input.at and compares stdout against expected_output.txt.

use crate::error::AutoResult;
use crate::run_with_capture;
use std::fs::read_to_string;
use std::path::PathBuf;

fn test_ffi_dual(case: &str) -> AutoResult<()> {
    let d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src = read_to_string(d.join(format!("test/ffi_dual/{}/input.at", case)))?;
    let expected =
        read_to_string(d.join(format!("test/ffi_dual/{}/expected_output.txt", case)))?;

    let (_, stdout) = run_with_capture(&src)?;
    let trimmed = stdout.trim();
    let expected_trimmed = expected.trim();
    if trimmed != expected_trimmed {
        let wrong_path = d.join(format!("test/ffi_dual/{}/.wrong.out", case));
        std::fs::write(&wrong_path, &stdout)?;
    }
    assert_eq!(
        trimmed, expected_trimmed,
        "VM output mismatch for {}",
        case
    );
    Ok(())
}

// === FFI Dual Tests ===

#[test]
fn ffi_dual_001_file_exists() {
    test_ffi_dual("001_file_exists").unwrap();
}

#[test]
fn ffi_dual_002_str_operations() {
    test_ffi_dual("002_str_operations").unwrap();
}

#[test]
fn ffi_dual_003_json_encode_parse() {
    test_ffi_dual("003_json_encode_parse").unwrap();
}

#[test]
fn ffi_dual_004_math_abs() {
    test_ffi_dual("004_math_abs").unwrap();
}

#[test]
fn ffi_dual_005_url_parts() {
    test_ffi_dual("005_url_parts").unwrap();
}

#[test]
fn ffi_dual_006_regex_is_match() {
    test_ffi_dual("006_regex_is_match").unwrap();
}

#[test]
fn ffi_dual_007_path_join() {
    test_ffi_dual("007_path_join").unwrap();
}

#[test]
fn ffi_dual_008_json_array() {
    test_ffi_dual("008_json_array").unwrap();
}

#[test]
fn ffi_dual_009_json_keys() {
    test_ffi_dual("009_json_keys").unwrap();
}

#[test]
fn ffi_dual_010_env_get_set() {
    test_ffi_dual("010_env_get_set").unwrap();
}

#[test]
fn ffi_dual_011_char_operations() {
    test_ffi_dual("011_char_operations").unwrap();
}

#[test]
fn ffi_dual_012_str_find_replace() {
    test_ffi_dual("012_str_find_replace").unwrap();
}

// Plan 430 C2: 端到端 —— dep 声明的三方 crate 自动出方法 shim 包,
// VM 侧经 dispatch 3000 兜底段调用其方法(构造器/&mut/&self/静态/字符串/i64)。
// 依赖 nightly rustdoc 提取元信息 + cargo 编译 shim 包,任一缺失时跳过(非失败)。
#[test]
fn ffi_dual_013_dep_method() {
    if !auto_cache::methods_pack::nightly_available() {
        eprintln!("skipped: nightly toolchain unavailable for methods pack");
        return;
    }
    let d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let fixture = d
        .join("test/ffi_dual/013_dep_method/fixture/autolang_counter")
        .to_string_lossy()
        .replace('\\', "/");
    let src = format!(
        r#"dep autolang_counter(path: "{fixture}")
use.rust autolang_counter::{{Counter, Config}}
let c = Counter.new("hits")
c.increment()
c.increment()
print(c.value())
print(c.label())
c.set_label("misses")
print(c.label())
print(c.add(5))
let d = c.clone_reset()
print(d.value())
print(Counter.version())

let cfg = Config.new()
let c2 = cfg.verbose(true)
print(c2.is_verbose())
let c3 = c2.level(7)
print(c3.level_value())
print(cfg.is_verbose())
let m = c3.merge(cfg)
print(m.level_value())
"#
    );
    let (_, stdout) = run_with_capture(&src).expect("run");
    let expected = "2\nhits\nmisses\n7\n7\n1.0.0\n1\n7\n1\n7";
    assert_eq!(stdout.trim(), expected, "dep method e2e output mismatch:\n{stdout}");
}

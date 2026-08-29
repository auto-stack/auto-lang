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
use.rust autolang_counter::{{Counter, Config, Point}}
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
let p = Config.parse("42")
print(p.level_value())

let pt = Point.new(3, 4, "origin")
print(pt.x())
print(pt.y())
print(pt.tag())
print(pt.to_string())
"#
    );
    let (_, stdout) = run_with_capture(&src).expect("run");
    let expected = "2\nhits\nmisses\n7\n7\n1.0.0\ntrue\n7\ntrue\n7\n42\n3\n4\norigin\n(3, 4) origin"; // Plan 474 待澄清#3: bool 显示形态 true/false
    assert_eq!(stdout.trim(), expected, "dep method e2e output mismatch:\n{stdout}");

    // unwrap_ok 错误传播:Result 构造失败 → VMError(带 cdylib 侧错误消息)
    let bad = format!(
        r#"dep autolang_counter(path: "{fixture}")
use.rust autolang_counter::{{Config}}
let p = Config.parse("not-a-number")
"#
    );
    let err = run_with_capture(&bad).expect_err("parse error must propagate");
    let msg = format!("{err:?}");
    assert!(
        msg.contains("Config.parse") && msg.contains("invalid level"),
        "error should carry dep-side message, got: {msg}"
    );
}

// Plan 430 复审补网:std 臂 VM 路径回归网。
// 守护 dispatch 3000 生成段(generated_std.rs):Vec 14 臂/Duration 5 臂/
// Instant 2 臂/PathBuf.from/String.new|from。复审发现 430 迁移 std 手写臂后,
// VM 模式的 19_rust_std goldens 当时全部 #[ignore],迁移主战场零活跃守护——
// 本用例补上;19_rust_std 的陈旧 ignore 亦已解除。
#[test]
fn ffi_dual_015_musk_backend_wave1() {
    test_ffi_dual("015_musk_backend_wave1").unwrap();
}

#[test]
fn ffi_dual_014_std_generated_segment() {
    test_ffi_dual("014_std_generated_segment").unwrap();
}

// 跨测试路由污染回归(ffi_dual_014 发现,2026-08-25):BIGVM_NATIVES 惰性注册 +
// "已有 native 优先"启发式,使 use.rust 的 String.from 路由取决于同进程内是否有
// 先前程序用过原生 String API。本测试在**同一测试内**先跑原生(无 use.rust)
// String.from——修复前第二条会 print 出裸堆 ID(4000011 形态)而非 "42"。
#[test]
fn ffi_dual_015_rust_type_route_not_hijacked_by_native_registry() {
    // 1. 原生 String API(无 use.rust)——副作用:auto.str.from 惰性注册进全局表
    let (_, out1) = crate::run_with_capture(r#"
fn main() {
    let s = String.from("native")
    print(s.len())
}
"#).expect("native String.from runs");
    assert_eq!(out1.trim(), "6");

    // 2. use.rust 的 String.from——必须仍走 dispatch 3000 生成段,
    //    不得被已注册的 auto.str.from 劫持
    let (_, out2) = crate::run_with_capture(r#"
use.rust std::string::String
fn main() {
    print(String.from("42"))
}
"#).expect("rust String.from runs");
    assert_eq!(out2.trim(), "42", "rust type import must not be hijacked by lazily-registered auto.str native");
}

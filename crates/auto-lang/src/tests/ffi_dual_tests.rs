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

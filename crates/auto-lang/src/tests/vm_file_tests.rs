// Plan 177: VM File-Based Test Framework
// Similar to a2r_tests, reads .at files from test/vm/ directory
// Supports three assertion types:
//   .expected.out    — stdout output from print()
//   .expected.result — return value (last expression)
//   .expected.error  — expected runtime error

use crate::error::AutoResult;
use crate::{run, run_with_capture};
use std::fs::read_to_string;
use std::path::PathBuf;

fn test_vm(case: &str) -> AutoResult<()> {
    // Parse test case name: "01_basics/001_hello" -> "hello"
    let dir_name = case.rsplit('/').next().unwrap_or(case);
    let parts: Vec<&str> = dir_name.splitn(2, '_').collect();
    let name = parts[1..].join("_");

    let d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src = read_to_string(d.join(format!("test/vm/{}/{}.at", case, name)))?;

    // Check .expected.error — expect runtime error
    let err_path = d.join(format!("test/vm/{}/{}.expected.error", case, name));
    if err_path.is_file() {
        let result = run(&src);
        assert!(
            result.is_err(),
            "Expected error but got: {:?}",
            result
        );
        return Ok(());
    }

    // Execute with stdout capture
    let (result, stdout) = run_with_capture(&src)?;

    // Check .expected.out — stdout output
    let out_path = d.join(format!("test/vm/{}/{}.expected.out", case, name));
    if out_path.is_file() {
        let expected_out = read_to_string(&out_path)?;
        if stdout != expected_out {
            let wrong_path = d.join(format!("test/vm/{}/{}.wrong.out", case, name));
            std::fs::write(&wrong_path, &stdout)?;
        }
        assert_eq!(stdout, expected_out);
    }

    // Check .expected.result — return value
    let res_path = d.join(format!("test/vm/{}/{}.expected.result", case, name));
    if res_path.is_file() {
        let expected_res = read_to_string(&res_path)?;
        if result != expected_res {
            let wrong_path = d.join(format!("test/vm/{}/{}.wrong.result", case, name));
            std::fs::write(&wrong_path, &result)?;
        }
        assert_eq!(result, expected_res);
    }

    Ok(())
}

// === 01_basics ===
#[test] fn test_01_basics_001_hello() { test_vm("01_basics/001_hello").unwrap(); }
#[test] fn test_01_basics_002_arithmetic() { test_vm("01_basics/002_arithmetic").unwrap(); }
#[test] fn test_01_basics_003_str_upper() { test_vm("01_basics/003_str_upper").unwrap(); }

// === 02_bit_ops ===
#[test] fn test_02_bit_ops_001_binary_literal() { test_vm("02_bit_ops/001_binary_literal").unwrap(); }
#[test] fn test_02_bit_ops_002_bitwise_ops() { test_vm("02_bit_ops/002_bitwise_ops").unwrap(); }
#[test] fn test_02_bit_ops_003_bit_scan() { test_vm("02_bit_ops/003_bit_scan").unwrap(); }
#[test] fn test_02_bit_ops_004_not_flip() { test_vm("02_bit_ops/004_not_flip").unwrap(); }
#[test] fn test_02_bit_ops_005_bitfield() { test_vm("02_bit_ops/005_bitfield").unwrap(); }

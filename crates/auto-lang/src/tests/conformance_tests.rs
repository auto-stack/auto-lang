// Plan 266 Phase 2: Dual-Execution Conformance Tests
//
// Tests that AutoVM and a2r produce identical output for the same Auto code.
// Each test case has input.at + expected_output.txt in test/a2r/conformance/.
//
// Strategy:
// 1. Run input.at through AutoVM → capture stdout
// 2. Compare AutoVM output against expected_output.txt
// 3. (Future) Transpile via a2r, compile with rustc, execute, compare

use crate::error::AutoResult;
use crate::run_autovm_capture;
use std::fs::read_to_string;
use std::path::PathBuf;

fn conformance_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test/a2r/conformance")
}

/// Run a single conformance test case.
/// Returns Ok if AutoVM output matches expected_output.txt.
fn run_conformance_test(case: &str) -> AutoResult<()> {
    let dir = conformance_dir().join(case);
    let src = read_to_string(dir.join("input.at"))?;
    let expected = read_to_string(dir.join("expected_output.txt"))?;

    // Path 1: AutoVM execution
    let (_, stdout) = run_autovm_capture(&src)?;
    let vm_output = stdout.trim();
    let expected_trimmed = expected.trim();

    if vm_output != expected_trimmed {
        // Write .wrong.out for debugging
        let wrong_path = dir.join(".wrong.out");
        std::fs::write(&wrong_path, &stdout)?;
    }

    assert_eq!(
        vm_output, expected_trimmed,
        "AutoVM output mismatch for conformance case '{}'",
        case
    );

    Ok(())
}

/// Generate expected_output.txt from AutoVM execution.
/// Used once to bootstrap test cases, then expected_output.txt is committed.
#[allow(dead_code)]
fn generate_expected(case: &str) -> AutoResult<String> {
    let dir = conformance_dir().join(case);
    let src = read_to_string(dir.join("input.at"))?;
    let (_, stdout) = run_autovm_capture(&src)?;
    let output = stdout.trim().to_string();
    std::fs::write(dir.join("expected_output.txt"), format!("{}\n", output))?;
    Ok(output)
}

// === Conformance Test Cases ===

/// Bootstrap: generate expected_output.txt files from AutoVM execution.
/// Run once with `cargo test -p auto-lang -- conformance_bootstrap --nocapture`,
/// then commit the generated files.
#[test]
fn conformance_bootstrap() {
    let cases = [
        "001_int_add", "002_str_concat", "003_if_else", "004_for_range",
        "005_fstring", "006_struct_access", "007_func_call", "008_arithmetic",
        "009_comparison", "010_array_index",
    ];
    for case in &cases {
        match generate_expected(case) {
            Ok(output) => println!("✓ {}:\n{}", case, output),
            Err(e) => println!("✗ {}: {}", case, e),
        }
    }
}

#[test]
fn conformance_001_int_add() {
    run_conformance_test("001_int_add").unwrap();
}

#[test]
fn conformance_002_str_concat() {
    run_conformance_test("002_str_concat").unwrap();
}

#[test]
fn conformance_003_if_else() {
    run_conformance_test("003_if_else").unwrap();
}

#[test]
fn conformance_004_for_range() {
    run_conformance_test("004_for_range").unwrap();
}

#[test]
fn conformance_005_fstring() {
    run_conformance_test("005_fstring").unwrap();
}

#[test]
fn conformance_006_struct_access() {
    run_conformance_test("006_struct_access").unwrap();
}

#[test]
fn conformance_007_func_call() {
    run_conformance_test("007_func_call").unwrap();
}

#[test]
fn conformance_008_arithmetic() {
    run_conformance_test("008_arithmetic").unwrap();
}

#[test]
fn conformance_009_comparison() {
    run_conformance_test("009_comparison").unwrap();
}

#[test]
fn conformance_010_array_index() {
    run_conformance_test("010_array_index").unwrap();
}

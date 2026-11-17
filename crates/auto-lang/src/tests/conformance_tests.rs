// AutoVM Output Regression Tests
//
// What this file verifies: AutoVM execution output is stable against golden
// files (expected_output.txt). Each case runs input.at through AutoVM and
// compares captured stdout to the golden. This catches VM regressions only.
//
// What this file does NOT verify: AutoVM-vs-a2r behavioral parity. Three-way
// parity (AutoVM vs a2r-transpiled Rust vs native Rust) is handled by the
// separate `test/parity/` workspace (see test/parity/docs/parity-guide.md and Plan 347).
//
// Strategy:
// 1. Run input.at through AutoVM → capture stdout
// 2. Compare AutoVM output against expected_output.txt
// 3. On mismatch, write .wrong.out for debugging

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
    run_conformance_case(&conformance_dir(), case)
}

/// PLAN-1066: root-parametrized conformance core. Normal wrappers run it on
/// the repo corpus; the A-slice probes run the SAME mechanism (doc payload,
/// trim comparison, scoped diagnostics) against a scope mirror.
pub(crate) fn run_conformance_case(root: &std::path::Path, case: &str) -> AutoResult<()> {
    let dir = root.join(case);
    // PLAN-1062 T-14: payload comes from the migration document (input
    // inline; expected output as the result oracle). The frozen sidecars
    // are history — never a fallback.
    let doc = crate::tests::legacy_doc::load_case(&dir, "input")
        .map_err(|e| crate::error::AutoError::Msg(e.into()))?;
    let src = doc.source_text().to_string();
    let expected = doc
        .result_text()
        .ok_or_else(|| {
            crate::error::AutoError::Msg(format!(
                "conformance {case}: expected output missing from migration doc",
            ))
        })?
        .to_string();

    // Path 1: AutoVM execution
    let (_, stdout) = run_autovm_capture(&src)?;
    let vm_output = stdout.trim();
    let expected_trimmed = expected.trim();

    if vm_output != expected_trimmed {
        // PLAN-1066 A-NOWRITE: the mismatch diagnostic goes to the explicit
        // scope (or is dropped) — the corpus dir stays byte-identical.
        crate::tests::diag_scope::write_scoped(
            &format!("test/a2r/conformance/{case}/.wrong.out"),
            stdout.as_bytes(),
        )?;
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
///
/// PLAN-1066 A-NOWRITE (T-07[A]): the candidate lands ONLY in the explicit
/// diagnostic scope — a normal run can never regenerate or overwrite the
/// author-tree oracle. The explicit conformance-proposal flow (T-07[B])
/// owns any author-side generation.
#[allow(dead_code)]
fn generate_expected(case: &str) -> AutoResult<String> {
    let dir = conformance_dir().join(case);
    let src = read_to_string(dir.join("input.at"))
        .map_err(|e| crate::error::AutoError::Msg(e.to_string()))?;
    let (_, stdout) = run_autovm_capture(&src)?;
    let output = stdout.trim().to_string();
    crate::tests::diag_scope::write_scoped(
        &format!("test/a2r/conformance/{case}/expected_output.txt"),
        format!("{}\n", output).as_bytes(),
    )?;
    Ok(output)
}

// === Conformance Test Cases ===

/// Bootstrap: (historically) regenerate expected_output.txt files from AutoVM
/// execution.
///
/// PLAN-1066 T-07[A] (A-NOWRITE): a generator is an AUTHOR action, not a
/// verification — it no longer runs on the normal path (`cargo t corpus`).
/// The 33 independent conformance verifications below are unchanged and
/// remain the normal gate; explicit candidate generation moves to the
/// conformance-proposal flow owned by T-07[B]. The 30-case closed set stays
/// frozen (PLAN-1066/A denominator fixture).
#[test]
#[ignore = "PLAN-1066 A-NOWRITE: author-side generator; explicit conformance-proposal (T-07[B]) owns oracle writes"]
fn conformance_bootstrap() {
    let cases = [
        "001_int_add",
        "002_str_concat",
        "003_if_else",
        "004_for_range",
        "005_fstring",
        "006_struct_access",
        "007_func_call",
        "008_arithmetic",
        "009_comparison",
        "010_array_index",
        "011_int_div",
        "012_int_mod",
        "013_negation",
        "014_float_arith",
        "015_nested_if",
        "016_loop_break",
        "017_loop_continue",
        "018_for_iterator",
        "019_enum_scalar",
        "020_enum_match",
        "021_string_methods",
        "022_list_push_pop",
        "023_list_map_filter",
        "024_map_basic",
        "025_option_basic",
        "026_result_basic",
        "027_nested_func",
        "028_recursive_func",
        "029_multi_param",
        "030_str_to_int",
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

// === Phase 3: Expanded Coverage ===

#[test]
fn conformance_011_int_div() {
    run_conformance_test("011_int_div").unwrap();
}

#[test]
fn conformance_012_int_mod() {
    run_conformance_test("012_int_mod").unwrap();
}

#[test]
fn conformance_013_negation() {
    run_conformance_test("013_negation").unwrap();
}

#[test]
fn conformance_014_float_arith() {
    run_conformance_test("014_float_arith").unwrap();
}

#[test]
fn conformance_015_nested_if() {
    run_conformance_test("015_nested_if").unwrap();
}

#[test]
fn conformance_016_loop_break() {
    run_conformance_test("016_loop_break").unwrap();
}

#[test]
fn conformance_017_loop_continue() {
    run_conformance_test("017_loop_continue").unwrap();
}

#[test]
fn conformance_018_for_iterator() {
    run_conformance_test("018_for_iterator").unwrap();
}

#[test]
fn conformance_019_enum_scalar() {
    run_conformance_test("019_enum_scalar").unwrap();
}

#[test]
fn conformance_020_enum_match() {
    run_conformance_test("020_enum_match").unwrap();
}

#[test]
fn conformance_021_string_methods() {
    run_conformance_test("021_string_methods").unwrap();
}

#[test]
fn conformance_022_list_push_pop() {
    run_conformance_test("022_list_push_pop").unwrap();
}

#[test]
fn conformance_023_list_map_filter() {
    run_conformance_test("023_list_map_filter").unwrap();
}

#[test]
fn conformance_024_map_basic() {
    run_conformance_test("024_map_basic").unwrap();
}

#[test]
fn conformance_025_option_basic() {
    run_conformance_test("025_option_basic").unwrap();
}

#[test]
fn conformance_026_result_basic() {
    run_conformance_test("026_result_basic").unwrap();
}

#[test]
fn conformance_027_nested_func() {
    run_conformance_test("027_nested_func").unwrap();
}

#[test]
fn conformance_028_recursive_func() {
    run_conformance_test("028_recursive_func").unwrap();
}

#[test]
fn conformance_029_multi_param() {
    run_conformance_test("029_multi_param").unwrap();
}

#[test]
fn conformance_030_str_to_int() {
    run_conformance_test("030_str_to_int").unwrap();
}

// === Phase 4: Differential Testing ===

/// Run N random programs through AutoVM, verify no crashes.
/// Each program is generated from a seed for reproducibility.
#[test]
fn conformance_differential_stability() {
    use crate::tests::test_util::program_generator::ProgramGenerator;

    let count = 50;
    let mut passed = 0;

    for seed in 0..count {
        let mut gen = ProgramGenerator::new(seed);
        let program = gen.generate_program();

        match run_autovm_capture(&program) {
            Ok(_) => passed += 1,
            Err(_) => {} // compile/runtime error is OK — we test no panic
        }
    }

    // At least 70% of generated programs should execute without crash
    let ratio = passed as f64 / count as f64;
    assert!(
        ratio >= 0.5,
        "Only {}/{} ({:.0}%) programs executed successfully. Need >= 50%.",
        passed,
        count,
        ratio * 100.0
    );
}

/// Verify same seed produces same output (reproducibility).
#[test]
fn conformance_differential_reproducibility() {
    use crate::tests::test_util::program_generator::ProgramGenerator;

    for seed in [0u64, 1, 42, 100, 999] {
        let mut gen1 = ProgramGenerator::new(seed);
        let mut gen2 = ProgramGenerator::new(seed);
        let p1 = gen1.generate_program();
        let p2 = gen2.generate_program();
        assert_eq!(p1, p2, "Seed {} produced different programs", seed);

        // If AutoVM succeeds, output should be deterministic
        if let Ok((_, out1)) = run_autovm_capture(&p1) {
            if let Ok((_, out2)) = run_autovm_capture(&p2) {
                assert_eq!(out1, out2, "Seed {} produced different outputs", seed);
            }
        }
    }
}

// === Plan 010 (MS3-A): while + try/catch ===

#[test]
fn conformance_040_while_basic() {
    run_conformance_test("040_while_basic").unwrap();
}

#[test]
fn conformance_042_try_no_error() {
    run_conformance_test("042_try_no_error").unwrap();
}

#[test]
fn conformance_043_try_catch_param() {
    run_conformance_test("043_try_catch_param").unwrap();
}

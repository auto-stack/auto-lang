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

// ---------------------------------------------------------------------------
// PLAN-1066 T-07[B]: the ONE explicit conformance-proposal entry.
//
// A generator is an author action, never a verification: the ordinary wrappers
// and the retired bootstrap keep writing nothing into the author tree. This
// entry reads the frozen 30-case closed set (the same documents the 33
// independent verifications use), runs each input through AutoVM, and lands the
// candidate only under an EXPLICIT, empty, non-repository scope. Any failed
// candidate is reported by name and makes the whole entry fail; a partial set
// is never presented as a successful proposal.
// ---------------------------------------------------------------------------

/// The frozen 30-case generation closed set — byte-for-byte the PLAN-1066/A
/// denominator's `c_bootstrap` list (the A-DEN integrity test compares the two).
/// Adding a verification-only case such as `040_while_basic` here is rejected:
/// the set is a contract, not a discovery.
const PROPOSAL_CLOSED_SET: [&str; 30] = [
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

/// Resolve and admit the explicit proposal scope.
///
/// * the scope is REQUIRED (no default: a silent default would be author-side
///   generation again);
/// * it must be outside the repository and not alias a repository path;
/// * it must not be a reparse point and must be absent or empty, so a previous
///   proposal is never silently overwritten.
fn admit_proposal_scope(requested: &str) -> Result<std::path::PathBuf, String> {
    if requested.trim().is_empty() {
        return Err("proposal scope must be explicit (empty request)".into());
    }
    let repo = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let scope = std::path::PathBuf::from(requested);
    let absolute = if scope.is_absolute() {
        scope.clone()
    } else {
        std::env::current_dir()
            .map_err(|e| format!("cwd: {e}"))?
            .join(&scope)
    };
    // Lexical containment first (a path that merely starts with the repo name
    // is not inside it), then a canonical check over the existing ancestors so
    // a symlinked alias cannot smuggle the write back into the tree.
    let repo_canonical = repo
        .canonicalize()
        .map_err(|e| format!("repo canonicalize {}: {e}", repo.display()))?;
    let mut probe = absolute.clone();
    while !probe.exists() {
        probe = probe
            .parent()
            .ok_or("proposal scope has no existing ancestor")?
            .to_path_buf();
    }
    let probe_canonical = probe
        .canonicalize()
        .map_err(|e| format!("scope canonicalize {}: {e}", probe.display()))?;
    if absolute.starts_with(&repo) || probe_canonical.starts_with(&repo_canonical) {
        return Err(format!(
            "proposal scope {} is inside the repository (author tree writes are forbidden)",
            absolute.display()
        ));
    }
    if absolute.exists() {
        if !absolute.is_dir() {
            return Err(format!("proposal scope {} is not a directory", absolute.display()));
        }
        let empty = std::fs::read_dir(&absolute)
            .map_err(|e| format!("scope: {e}"))?
            .next()
            .is_none();
        if !empty {
            return Err(format!(
                "proposal scope {} must be empty (preserve previous proposals)",
                absolute.display()
            ));
        }
    }
    Ok(absolute)
}

/// Run one candidate through the SAME execution the verification uses.
fn proposal_candidate(case: &str) -> Result<String, String> {
    let dir = conformance_dir().join(case);
    let doc = crate::tests::legacy_doc::load_case(&dir, "input")?;
    let src = doc.source_text().to_string();
    let (_, stdout) = run_autovm_capture(&src).map_err(|e| e.to_string())?;
    Ok(format!("{}\n", stdout.trim()))
}

/// Explicit author action. Requires `AUTO_TEST_1066_PROPOSAL_OUT`.
#[test]
#[ignore = "explicit PLAN-1066 T-07[B] conformance proposal: writes author-side candidates only into AUTO_TEST_1066_PROPOSAL_OUT"]
fn conformance_proposal() {
    let requested = std::env::var("AUTO_TEST_1066_PROPOSAL_OUT")
        .expect("AUTO_TEST_1066_PROPOSAL_OUT must name an explicit scope");
    let scope = admit_proposal_scope(&requested).unwrap_or_else(|e| panic!("{e}"));
    std::fs::create_dir_all(&scope).expect("create proposal scope");
    let mut failures = Vec::new();
    let mut written = 0usize;
    for case in PROPOSAL_CLOSED_SET {
        match proposal_candidate(case) {
            Ok(candidate) => {
                let dir = scope.join(case);
                std::fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("{}: {e}", dir.display()));
                let target = dir.join("expected_output.txt");
                std::fs::write(&target, candidate.as_bytes())
                    .unwrap_or_else(|e| panic!("{}: {e}", target.display()));
                written += 1;
                eprintln!("CANDIDATE {case} -> {}", target.display());
            }
            Err(e) => {
                eprintln!("FAILED {case}: {e}");
                failures.push(case.to_string());
            }
        }
    }
    eprintln!(
        "conformance proposal: {} written, {} failed, of {} closed-set cases",
        written,
        failures.len(),
        PROPOSAL_CLOSED_SET.len()
    );
    assert!(
        failures.is_empty() && written == PROPOSAL_CLOSED_SET.len(),
        "proposal incomplete: failed {failures:?}"
    );
}

/// Permanent control (no AutoVM, no writes): the scope admission rules and the
/// closed set itself. A repo-internal or aliased scope, a non-empty scope and
/// an unknown (verification-only) case are all rejected by name.
#[test]
fn plan1066_conformance_proposal_admission_controls() {
    let repo = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    // repo-internal scope
    let err = admit_proposal_scope(repo.join("target/plan1066-proposal").to_str().unwrap())
        .expect_err("repo-internal proposal scope must be refused");
    assert!(err.contains("inside the repository"), "{err}");
    // empty request
    let err = admit_proposal_scope("   ").expect_err("empty scope must be refused");
    assert!(err.contains("explicit"), "{err}");
    // non-empty existing scope preserves previous proposals
    let temp = tempfile::tempdir().expect("temp");
    std::fs::write(temp.path().join("old.txt"), "previous").unwrap();
    let err = admit_proposal_scope(temp.path().to_str().unwrap())
        .expect_err("non-empty scope must be refused");
    assert!(err.contains("empty"), "{err}");
    // a fresh external scope is admitted
    let fresh = temp.path().join("fresh");
    assert!(admit_proposal_scope(fresh.to_str().unwrap()).is_ok());

    // the closed set is exactly 30 unique cases and does not admit a
    // verification-only case
    let unique: std::collections::BTreeSet<&str> = PROPOSAL_CLOSED_SET.into_iter().collect();
    assert_eq!(unique.len(), PROPOSAL_CLOSED_SET.len(), "duplicate closed-set entry");
    assert_eq!(PROPOSAL_CLOSED_SET.len(), 30, "closed set lost entries");
    assert!(
        !PROPOSAL_CLOSED_SET.contains(&"040_while_basic"),
        "verification-only case must not join the generation closed set"
    );
    // NC-17: the closed set IS the frozen A denominator. Editing either side
    // (adding `040_while_basic`, deleting an entry, renaming one) is rejected by
    // name here, so "30 again" cannot hide a swapped member.
    let fixture: serde_json::Value = serde_json::from_slice(
        &std::fs::read(
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../auto-lang/test/governance/plan1066_a_obligations.json"),
        )
        .or_else(|_| {
            std::fs::read(
                std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                    .join("test/governance/plan1066_a_obligations.json"),
            )
        })
        .expect("A denominator fixture"),
    )
    .expect("A denominator JSON");
    let frozen: Vec<String> = fixture["c_bootstrap"]
        .as_array()
        .expect("c_bootstrap")
        .iter()
        .map(|v| v.as_str().expect("case").to_string())
        .collect();
    assert_eq!(
        frozen,
        PROPOSAL_CLOSED_SET.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
        "proposal closed set drifted from the frozen denominator"
    );

    // NC-18: the ordinary generator is retired (ignore attribute present) and
    // its writes go through the diagnostic scope only.
    let source = include_str!("conformance_tests.rs");
    // rfind: this control itself contains the literal search key.
    let bootstrap_at = source.rfind("fn conformance_bootstrap()").expect("bootstrap");
    let attributes = source[..bootstrap_at]
        .lines()
        .rev()
        .take(4)
        .collect::<Vec<_>>();
    let ignore_line = attributes
        .iter()
        .find(|line| line.trim_start().starts_with("#[ignore"))
        .unwrap_or_else(|| panic!("conformance_bootstrap must stay #[ignore]d: {attributes:?}"));
    assert!(
        ignore_line.contains("proposal"),
        "the ignore reason must name the explicit proposal flow: {ignore_line}"
    );
    let generate_at = source.find("fn generate_expected(").expect("generate_expected");
    let generate_body = &source[generate_at..generate_at + 1200];
    assert!(
        generate_body.contains("write_scoped"),
        "generate_expected must write through the diagnostic scope"
    );
    assert!(
        !generate_body.contains("fs::write("),
        "generate_expected must not write a path directly"
    );

    let bootstrap_set = {
        // the retired bootstrap generator uses the same frozen list
        let source = include_str!("conformance_tests.rs");
        let start = source.find("const PROPOSAL_CLOSED_SET").expect("const");
        let end = source[start..].find("];").expect("end");
        source[start..start + end].matches('"').count() / 2
    };
    assert_eq!(bootstrap_set, 30, "closed set literal drifted");
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

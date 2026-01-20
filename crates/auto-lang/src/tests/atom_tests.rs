//! Auto-Atom Tests
//!
//! This module runs markdown-based tests for auto-atom functionality.
//! Tests evaluate Auto code using AtomReader and compare the output Atom representation.

use crate::atom::AtomReader;
use std::path::PathBuf;

/// Represents a single test case parsed from markdown
pub struct AtomTestCase {
    pub name: String,
    pub input: String,
    pub expected_output: String,
}

/// Parse test cases from a markdown file
pub fn parse_atom_test_file(content: &str) -> Vec<AtomTestCase> {
    let mut tests = Vec::new();
    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        // Look for test headers (## Test Name)
        if lines[i].starts_with("## ") {
            let name = lines[i][3..].trim().to_string();
            i += 1;

            // Skip empty lines
            while i < lines.len() && lines[i].trim().is_empty() {
                i += 1;
            }

            // Collect input code until we find ---
            let mut input = String::new();
            while i < lines.len() && !lines[i].starts_with("---") {
                if !input.is_empty() {
                    input.push('\n');
                }
                input.push_str(lines[i]);
                i += 1;
            }

            // Skip the --- line
            if i < lines.len() && lines[i].starts_with("---") {
                i += 1;
            }

            // Collect expected output until next ## or end of file
            let mut expected_output = String::new();
            while i < lines.len() && !lines[i].starts_with("## ") {
                if !expected_output.is_empty() {
                    expected_output.push('\n');
                }
                expected_output.push_str(lines[i]);
                i += 1;
            }

            tests.push(AtomTestCase {
                name,
                input,
                expected_output,
            });

            continue;
        }

        i += 1;
    }

    tests
}

/// Normalize whitespace for comparison
pub fn normalize_for_compare(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Run a single atom test case
pub fn run_atom_test(test: &AtomTestCase) -> Result<(), String> {
    // Evaluate the input Auto code using AtomReader
    let mut reader = AtomReader::new();
    let atom = reader
        .parse(test.input.as_str())
        .map_err(|e| format!("Failed to evaluate Auto code: {}", e))?;

    // Get the actual output as an Atom string
    let actual_output = atom.to_astr().to_string();
    let expected_normalized = normalize_for_compare(&test.expected_output);
    let actual_normalized = normalize_for_compare(&actual_output);

    // Compare the evaluated result with expected output
    if actual_normalized != expected_normalized {
        return Err(format!(
            "Test '{}' failed:\nInput:    {}\nExpected: {}\nActual:   {}",
            test.name, test.input, expected_normalized, actual_normalized
        ));
    }

    Ok(())
}

/// Run all atom tests from a markdown file
pub fn run_atom_tests_from_file(path: &PathBuf) -> Result<(), String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read file {}: {}", path.display(), e))?;

    let tests = parse_atom_test_file(&content);

    if tests.is_empty() {
        return Err(format!("No tests found in {}", path.display()));
    }

    println!("Running {} tests from {}", tests.len(), path.display());

    let mut passed = 0;
    let mut failed = Vec::new();

    for test in &tests {
        match run_atom_test(test) {
            Ok(()) => {
                println!("  ✓ {}", test.name);
                passed += 1;
            }
            Err(e) => {
                println!("  ✗ {}: {}", test.name, e);
                failed.push((test.name.clone(), e));
            }
        }
    }

    println!("\nResults: {} passed, {} failed", passed, failed.len());

    if !failed.is_empty() {
        return Err(format!(
            "{} test(s) failed:\n{}",
            failed.len(),
            failed
                .iter()
                .map(|(name, err)| format!("  - {}: {}", name, err))
                .collect::<Vec<_>>()
                .join("\n")
        ));
    }

    Ok(())
}

#[test]
fn test_atom_basics() {
    let test_file = PathBuf::from("test/atom/atom_basics.md");
    match run_atom_tests_from_file(&test_file) {
        Ok(()) => println!("All atom_basics tests passed!"),
        Err(e) => panic!("Atom basics tests failed:\n{}", e),
    }
}

/// Test framework for auto-gen markdown-based tests
///
/// Test format:
/// ## Test Name
///
/// [data section - Auto code]
///
/// ---
///
/// [template section - Auto template]
///
/// ---
///
/// [expected output]
use std::path::PathBuf;

/// Represents a single test case parsed from markdown
pub struct GenTestCase {
    pub name: String,
    pub data: String,
    pub template: String,
    pub expected: String,
}

/// Parse a markdown test file into test cases
pub fn parse_gen_test_file(content: &str) -> Result<Vec<GenTestCase>, String> {
    let mut cases = Vec::new();
    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        // Look for test header (## Test Name)
        let line = lines[i].trim();

        if line.starts_with("## ") {
            let name = line[3..].trim().to_string();
            i += 1;

            // Skip empty lines after header
            while i < lines.len() && lines[i].trim().is_empty() {
                i += 1;
            }

            // Collect data section (until ---)
            let mut data = String::new();
            while i < lines.len() && !lines[i].trim().starts_with("---") {
                data.push_str(lines[i]);
                data.push('\n');
                i += 1;
            }

            // Skip --- separator
            if i < lines.len() && lines[i].trim().starts_with("---") {
                i += 1;
            }

            // Skip empty lines after separator
            while i < lines.len() && lines[i].trim().is_empty() {
                i += 1;
            }

            // Collect template section (until ---)
            let mut template = String::new();
            while i < lines.len() && !lines[i].trim().starts_with("---") {
                template.push_str(lines[i]);
                template.push('\n');
                i += 1;
            }

            // Skip --- separator
            if i < lines.len() && lines[i].trim().starts_with("---") {
                i += 1;
            }

            // Skip empty lines after separator
            while i < lines.len() && lines[i].trim().is_empty() {
                i += 1;
            }

            // Collect expected output (until next ## or ---)
            let mut expected = String::new();
            while i < lines.len()
                && !lines[i].trim().starts_with("## ")
                && !lines[i].trim().starts_with("---")
            {
                expected.push_str(lines[i]);
                expected.push('\n');
                i += 1;
            }

            cases.push(GenTestCase {
                name,
                data,
                template,
                expected,
            });
        } else {
            i += 1;
        }
    }

    Ok(cases)
}

/// Convert Atom format data to AutoLang let statements
///
/// Atom format: name: "value"  or  name: [1, 2, 3]  or  name: { key: value }
/// AutoLang: let name = "value"  or  let name = [1, 2, 3]  or  let name = { key: value }
fn convert_atom_to_let(data: &str) -> String {
    let mut result = String::new();
    let lines: Vec<&str> = data.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i].trim();

        // Skip empty lines
        if line.is_empty() {
            i += 1;
            continue;
        }

        // Check if line contains colon (Atom format)
        if let Some(colon_pos) = line.find(':') {
            let name = line[..colon_pos].trim();
            let value_part = line[colon_pos + 1..].trim();

            // Check if value starts with [ or { (multi-line array/object)
            if value_part.starts_with('[') || value_part.starts_with('{') {
                // Find the closing bracket/brace
                let opener = if value_part.starts_with('[') {
                    '['
                } else {
                    '{'
                };
                let closer = if value_part.starts_with('[') {
                    ']'
                } else {
                    '}'
                };

                let mut full_value = String::from(value_part);
                let mut depth = full_value.chars().filter(|&c| c == opener).count() as i32
                    - full_value.chars().filter(|&c| c == closer).count() as i32;

                // Collect lines until we find the closing bracket/brace
                while depth > 0 && i + 1 < lines.len() {
                    i += 1;
                    let trimmed = lines[i].trim();

                    // Add comma before the line if needed (not after [ or {)
                    if !full_value.ends_with('[')
                        && !full_value.ends_with('{')
                        && !trimmed.starts_with(']')
                        && !trimmed.starts_with('}')
                    {
                        full_value.push_str(", ");
                    } else {
                        full_value.push(' ');
                    }

                    full_value.push_str(trimmed);
                    depth += lines[i].chars().filter(|&c| c == opener).count() as i32
                        - lines[i].chars().filter(|&c| c == closer).count() as i32;
                }

                result.push_str(&format!("let {} = {}\n", name, full_value));
            } else {
                // Simple single-line value
                result.push_str(&format!("let {} = {}\n", name, value_part));
            }
        } else {
            // Keep as-is (might already be AutoLang code)
            result.push_str(line);
            result.push('\n');
        }

        i += 1;
    }

    result
}

/// Run a single generation test case
pub fn run_gen_test(test: &GenTestCase) -> Result<(), String> {
    use auto_lang::interp::Interpreter;

    // Create interpreter and evaluate data code first
    let mut inter = Interpreter::new();

    // Convert Atom format to let statements if needed
    let data_code = convert_atom_to_let(&test.data);

    // Interpret the data code to populate the universe
    inter
        .interpret(&data_code)
        .map_err(|e| format!("Failed to interpret data: {}", e))?;

    // Now evaluate the template with the populated universe
    let result = inter
        .eval_template(&test.template)
        .map_err(|e| format!("Failed to render template: {}", e))?;

    let output = result.to_astr();

    // Compare with expected (normalize whitespace)
    let normalized_output = normalize_output(&output);
    let normalized_expected = normalize_output(&test.expected);

    if normalized_output != normalized_expected {
        return Err(format!(
            "Output mismatch for test '{}'\n\nExpected:\n{}\n\nGot:\n{}\n",
            test.name, normalized_expected, normalized_output
        ));
    }

    Ok(())
}

/// Normalize output for comparison (trim whitespace aggressively)
fn normalize_output(s: &str) -> String {
    s.trim()
        .lines()
        .map(|line| line.trim())
        .collect::<Vec<_>>()
        .join("\n")
}

/// Run all tests from a markdown file
pub fn run_gen_tests_from_file(path: &PathBuf) -> Result<(), String> {
    let content =
        std::fs::read_to_string(path).map_err(|e| format!("Failed to read test file: {}", e))?;

    let cases = parse_gen_test_file(&content)?;

    if cases.is_empty() {
        return Err("No test cases found in file".to_string());
    }

    println!("Running {} tests from {}", cases.len(), path.display());

    let mut passed = 0;
    let mut failed = Vec::new();

    for test in &cases {
        match run_gen_test(test) {
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

    println!("\nResults: {}/{} passed", passed, cases.len());

    if !failed.is_empty() {
        let mut msg = format!("{} tests failed:\n", failed.len());
        for (name, error) in &failed {
            msg.push_str(&format!("  - {}: {}\n", name, error));
        }
        Err(msg)
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_case() {
        let content = r#"
## Test 1

let x = 42

---

$x

---

42
"#;
        let cases = parse_gen_test_file(content).unwrap();
        assert_eq!(cases.len(), 1);
        assert_eq!(cases[0].name, "Test 1");
        assert_eq!(cases[0].data.trim(), "let x = 42");
        assert_eq!(cases[0].template.trim(), "$x");
        assert_eq!(cases[0].expected.trim(), "42");
    }

    #[test]
    fn test_parse_multiple_cases() {
        let content = r#"
## Test 1

let x = 1

---

$x

---

1

## Test 2

let y = 2

---

$y

---

2
"#;
        let cases = parse_gen_test_file(content).unwrap();
        assert_eq!(cases.len(), 2);
        assert_eq!(cases[0].name, "Test 1");
        assert_eq!(cases[1].name, "Test 2");
    }
}

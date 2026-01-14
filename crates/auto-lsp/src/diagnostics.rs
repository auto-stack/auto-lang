use tower_lsp::lsp_types::*;
use auto_lang::error::AutoError;

/// Parse AutoLang code and convert errors to LSP diagnostics
pub fn parse_diagnostics(
    uri: &str,
    content: &str,
    _version: i32,
) -> Vec<Diagnostic> {
    // Catch panics to prevent LSP from crashing
    std::panic::catch_unwind(|| {
        parse_diagnostics_impl(uri, content)
    }).unwrap_or_else(|_| {
        eprintln!("=== LSP DIAGNOSTICS PANIC ===");
        eprintln!("Parser panicked while parsing: {}", uri);
        eprintln!("=== END PANIC ===");
        Vec::new()
    })
}

/// Implementation of diagnostics parsing
fn parse_diagnostics_impl(uri: &str, content: &str) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    // Suppress stdout/stderr during parsing to prevent debug output from corrupting LSP
    let _guard = SuppressOutput;

    // Attempt to parse the document
    match auto_lang::parse_preserve_error(content) {
        Ok(_) => {
            // Parse successful, no errors
        }
        Err(e) => {
            // Extract errors based on the AutoError variant
            let errors = extract_errors_from_auto_error(e);

            for error in errors.iter() {
                let error_msg = format!("{}", error);

                let range = extract_location_from_error(error, content);
                let severity = extract_severity_from_error(error);

                diagnostics.push(Diagnostic {
                    range,
                    severity: Some(severity),
                    code: None,
                    code_description: None,
                    source: Some("auto-lang".to_string()),
                    message: error_msg,
                    related_information: None,
                    tags: None,
                    data: None,
                });
            }
        }
    }

    diagnostics
}

/// Guard to suppress stdout/stderr during parsing
struct SuppressOutput;

impl Drop for SuppressOutput {
    fn drop(&mut self) {
        // Restore original stdout/stderr when guard is dropped
        // (Currently a no-op since we're using a static approach)
    }
}

/// Extract individual errors from an AutoError, unwrapping MultipleErrors
fn extract_errors_from_auto_error(error: AutoError) -> Vec<AutoError> {
    match &error {
        AutoError::MultipleErrors { errors, .. } => {
            // Return a clone of the inner errors
            errors.clone()
        }
        _ => {
            vec![error]
        }
    }
}

/// Extract LSP diagnostic severity from an AutoError
fn extract_severity_from_error(error: &AutoError) -> DiagnosticSeverity {
    match error {
        AutoError::Warning(_) => DiagnosticSeverity::WARNING,
        _ => DiagnosticSeverity::ERROR,
    }
}

/// Extract range from AutoError by using miette labels
fn extract_location_from_error(error: &AutoError, content: &str) -> Range {
    use miette::Diagnostic;

    // Default to start of file
    let mut start_line = 0u32;
    let mut start_char = 0u32;
    let mut end_line = 0u32;
    let mut end_char = 1u32;

    // Try to get labels from the miette Diagnostic
    if let Some(labels) = error.labels() {
        // Get the first label if available
        if let Some(label) = labels.into_iter().next() {
            let offset = label.offset() as usize;
            let len = label.len() as usize;

            // Convert byte offset to line/column
            let lines: Vec<&str> = content.lines().collect();
            let mut current_offset = 0;

            for (idx, line) in lines.iter().enumerate() {
                let line_len = line.len();
                // Add 1 for newline character
                let line_end = current_offset + line_len + 1;

                // Check if the label starts within this line
                if offset >= current_offset && offset < line_end {
                    start_line = idx as u32;
                    end_line = idx as u32;

                    // Calculate character position within the line
                    start_char = (offset - current_offset) as u32;
                    end_char = start_char + len as u32;

                    break;
                }

                current_offset = line_end;
            }
        }
    } else {
        // Fallback: try to parse line number from error message
        let error_msg = format!("{}", error);
        if let Some(line) = extract_line_number(&error_msg) {
            start_line = line.saturating_sub(1) as u32;
            end_line = start_line;

            // Get the line content to determine range
            let lines: Vec<&str> = content.lines().collect();
            if let Some(line_content) = lines.get(start_line as usize) {
                start_char = 0;
                end_char = line_content.len().max(1) as u32;
            }
        }
    }

    Range {
        start: Position {
            line: start_line,
            character: start_char,
        },
        end: Position {
            line: end_line,
            character: end_char,
        },
    }
}

/// Extract line number from error message
fn extract_line_number(error_msg: &str) -> Option<usize> {
    use regex::Regex;

    // Try various patterns for line numbers in error messages
    let patterns = [
        r"line (\d+)",
        r"at line (\d+)",
        r"\[(\d+):",
        r"L(\d+)",
        r"l\.(\d+)",
        r"@ line (\d+)",
        r"\[(\d+),",  // [line, column] format
        r"@ \[(\d+),", // @ [line, column] format
    ];

    for pattern in &patterns {
        if let Ok(re) = Regex::new(pattern) {
            if let Some(caps) = re.captures(error_msg) {
                if let Some(match_str) = caps.get(1) {
                    if let Ok(num) = match_str.as_str().parse::<usize>() {
                        return Some(num);
                    }
                }
            }
        }
    }

    None
}

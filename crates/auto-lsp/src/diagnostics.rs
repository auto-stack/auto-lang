use tower_lsp_server::ls_types::*;
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
fn parse_diagnostics_impl(_uri: &str, content: &str) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    // Parse with the modern Parser API to collect errors and warnings
    let mut parser = auto_lang::Parser::from(content);
    let _ = parser.parse();

    // Convert parser errors to diagnostics
    for error in &parser.errors {
        diagnostics.push(auto_error_to_diagnostic(error, content, DiagnosticSeverity::ERROR));
    }

    // Convert parser warnings to diagnostics
    for warning in &parser.warnings {
        diagnostics.push(warning_to_diagnostic(warning, content));
    }

    // If no errors/warnings were collected but parse returned Err,
    // also check the result (defensive)
    if diagnostics.is_empty() {
        if let Err(e) = auto_lang::parse_preserve_error(content) {
            let errors = extract_errors_from_auto_error(e);
            for error in errors.iter() {
                diagnostics.push(auto_error_to_diagnostic(error, content, DiagnosticSeverity::ERROR));
            }
        }
    }

    diagnostics
}

/// Convert an AutoError to an LSP Diagnostic
fn auto_error_to_diagnostic(error: &AutoError, content: &str, severity: DiagnosticSeverity) -> Diagnostic {
    let error_msg = format!("{}", error);
    let range = extract_location_from_error(error, content);

    Diagnostic {
        range,
        severity: Some(severity),
        code: None,
        code_description: None,
        source: Some("auto-lang".to_string()),
        message: error_msg,
        related_information: None,
        tags: None,
        data: None,
    }
}

/// Convert a Warning to an LSP Diagnostic
fn warning_to_diagnostic(warning: &auto_lang::error::Warning, _content: &str) -> Diagnostic {
    let message = format!("{}", warning);
    // Warnings may not have precise spans; default to start of file
    let range = Range {
        start: Position { line: 0, character: 0 },
        end: Position { line: 0, character: 0 },
    };

    Diagnostic {
        range,
        severity: Some(DiagnosticSeverity::WARNING),
        code: None,
        code_description: None,
        source: Some("auto-lang".to_string()),
        message,
        related_information: None,
        tags: None,
        data: None,
    }
}

/// Extract individual errors from an AutoError, unwrapping MultipleErrors
fn extract_errors_from_auto_error(error: AutoError) -> Vec<AutoError> {
    match &error {
        AutoError::MultipleErrors { errors, .. } => {
            errors.clone()
        }
        _ => {
            vec![error]
        }
    }
}

/// Extract range from AutoError by using miette labels
fn extract_location_from_error(error: &AutoError, content: &str) -> Range {
    use miette::Diagnostic;

    // Default to start of file
    let mut range = Range {
        start: Position { line: 0, character: 0 },
        end: Position { line: 0, character: 0 },
    };

    // Try to get labels from the miette Diagnostic
    if let Some(labels) = error.labels() {
        if let Some(label) = labels.into_iter().next() {
            let offset = label.offset() as usize;
            let len = label.len() as usize;
            range = byte_range_to_lsp_range(content, offset, len);
        }
    } else {
        // Fallback: try to parse line number from error message
        let error_msg = format!("{}", error);
        if let Some(line) = extract_line_number(&error_msg) {
            let line_idx = line.saturating_sub(1) as u32;
            let lines: Vec<&str> = content.lines().collect();
            let end_char = lines.get(line_idx as usize).map(|l| l.len().max(1) as u32).unwrap_or(1);
            range = Range {
                start: Position { line: line_idx, character: 0 },
                end: Position { line: line_idx, character: end_char },
            };
        }
    }

    range
}

/// Convert a byte offset range to an LSP Range (line/character)
/// LSP uses UTF-16 code units for character offsets.
fn byte_range_to_lsp_range(content: &str, start_offset: usize, len: usize) -> Range {
    let end_offset = start_offset.saturating_add(len);

    Range {
        start: byte_offset_to_position(content, start_offset),
        end: byte_offset_to_position(content, end_offset),
    }
}

/// Convert a byte offset to an LSP Position
fn byte_offset_to_position(content: &str, target_offset: usize) -> Position {
    let mut line = 0u32;
    let mut character = 0u32;
    let mut current_offset = 0usize;

    for ch in content.chars() {
        if current_offset >= target_offset {
            break;
        }

        if ch == '\n' {
            line += 1;
            character = 0;
        } else {
            // LSP counts UTF-16 code units
            character += ch.len_utf16() as u32;
        }

        current_offset += ch.len_utf8();
    }

    Position { line, character }
}

/// Extract line number from error message using simple string parsing
fn extract_line_number(error_msg: &str) -> Option<usize> {
    // Try common patterns in AutoLang error messages without regex
    for prefix in &["line ", "at line ", "@ line "] {
        if let Some(pos) = error_msg.find(prefix) {
            let after = &error_msg[pos + prefix.len()..];
            let num_str: String = after.chars().take_while(|c| c.is_ascii_digit()).collect();
            if let Ok(num) = num_str.parse::<usize>() {
                return Some(num);
            }
        }
    }

    // Try bracket patterns like "[42:" or "[42,"
    for (i, ch) in error_msg.char_indices() {
        if ch == '[' && i > 0 {
            let after = &error_msg[i + 1..];
            let num_str: String = after.chars().take_while(|c| c.is_ascii_digit()).collect();
            if !num_str.is_empty() {
                let rest = &after[num_str.len()..];
                if rest.starts_with(':') || rest.starts_with(',') {
                    if let Ok(num) = num_str.parse::<usize>() {
                        return Some(num);
                    }
                }
            }
        }
    }

    None
}

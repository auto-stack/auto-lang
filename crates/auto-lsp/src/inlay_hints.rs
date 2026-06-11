//! Inlay hints: show inferred types after `let`/`var` bindings
//!
//! Example: `let x = 42` → shows `let x /*: int */ = 42`

use tower_lsp_server::ls_types::*;

/// Compute inlay hints for a range of the document
pub fn get_inlay_hints(content: &str, range: &Range) -> Vec<InlayHint> {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        get_inlay_hints_impl(content, range)
    })).unwrap_or_else(|_| Vec::new())
}

fn get_inlay_hints_impl(content: &str, range: &Range) -> Vec<InlayHint> {
    let mut hints = Vec::new();

    // Parse the document to get type information
    let mut parser = auto_lang::Parser::from(content);
    let _ = parser.parse();
    let infer_ctx = &parser.infer_ctx;

    // Get all variable names and their types from InferenceContext
    let var_names = infer_ctx.get_defined_var_names();

    // Build a map from variable name to inferred type
    let mut type_map: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    for name in &var_names {
        if let Some(ty) = infer_ctx.lookup_type(&auto_lang::ast::Name::from(name.as_str())) {
            let type_str = ty.unique_name().to_string();
            // Only show non-trivial types (skip "unknown", "any", etc.)
            if type_str != "unknown" && type_str != "any" && type_str != "_" {
                type_map.insert(name.clone(), type_str);
            }
        }
    }

    // Scan lines to find `let`/`var` declarations that lack explicit type annotations
    for (line_num, line) in content.lines().enumerate() {
        // Skip lines outside the requested range
        if line_num < range.start.line as usize || line_num > range.end.line as usize {
            continue;
        }

        let trimmed = line.trim();

        // Match patterns: "let name =" or "var name ="
        // But skip lines that already have explicit type annotations like "let name int ="
        if let Some(rest) = trimmed.strip_prefix("let ").or_else(|| trimmed.strip_prefix("var ")) {
            // Extract the variable name
            let name_part = if let Some(eq_pos) = rest.find('=') {
                rest[..eq_pos].trim()
            } else {
                continue;
            };

            // Split to check for explicit type annotation
            let parts: Vec<&str> = name_part.split_whitespace().collect();
            if parts.len() != 1 {
                // Already has type annotation (e.g., "let x int" has 2 parts)
                continue;
            }

            let var_name = parts[0];

            // Only hint for valid identifiers
            if !var_name.chars().all(|c| c.is_alphanumeric() || c == '_') || var_name.chars().next().map_or(true, |c| c.is_numeric()) {
                continue;
            }

            if let Some(type_str) = type_map.get(var_name) {
                // Find the position right after the variable name
                let name_end_char = find_name_end(line, var_name);

                hints.push(InlayHint {
                    position: Position {
                        line: line_num as u32,
                        character: name_end_char,
                    },
                    label: InlayHintLabel::String(format!(": {}", type_str)),
                    kind: Some(InlayHintKind::TYPE),
                    text_edits: None,
                    tooltip: None,
                    padding_left: Some(true),
                    padding_right: None,
                    data: None,
                });
            }
        }
    }

    hints
}

/// Find the character offset right after the variable name in a declaration line
fn find_name_end(line: &str, var_name: &str) -> u32 {
    // Find "let " or "var " prefix
    let prefix_len = if line.trim_start().starts_with("let ") {
        let start = line.len() - line.trim_start().len();
        start + 4 // "let "
    } else if line.trim_start().starts_with("var ") {
        let start = line.len() - line.trim_start().len();
        start + 4 // "var "
    } else {
        return 0;
    };

    // Find the variable name after the prefix
    let after_prefix = &line[prefix_len.min(line.len())..];
    if let Some(pos) = after_prefix.find(var_name) {
        // The end of the variable name in the original line
        let byte_end = prefix_len + pos + var_name.len();
        // Convert byte offset to character offset
        let char_offset = line[..byte_end.min(line.len())].chars().count();
        return char_offset as u32;
    }

    0
}

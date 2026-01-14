use tower_lsp::lsp_types::*;

/// Find the definition location for a symbol at a given position
pub fn find_definition(content: &str, position: Position, uri: &str) -> Option<GotoDefinitionResponse> {
    // Catch panics to prevent LSP from crashing
    std::panic::catch_unwind(|| {
        find_definition_impl(content, position, uri)
    }).unwrap_or_else(|_| None)
}

/// Implementation of definition finding
fn find_definition_impl(content: &str, position: Position, uri: &str) -> Option<GotoDefinitionResponse> {
    // Get the line at the cursor position
    let lines: Vec<&str> = content.lines().collect();
    let line = lines.get(position.line as usize)?;

    // Get the word at the cursor position
    let word = get_word_at_position(line, position.character as usize)?;

    eprintln!("=== GOTO DEF DEBUG ===");
    eprintln!("Line: '{}'", line);
    eprintln!("Word: '{}'", word);
    eprintln!("Position: {:?}", position);

    // Parse the code and get the universe with symbol locations
    let scope = std::rc::Rc::new(std::cell::RefCell::new(auto_lang::Universe::new()));
    {
        let mut parser = auto_lang::Parser::new(content, scope.clone());

        // Parse to populate the symbol_locations table
        // We ignore parse errors for go-to-definition
        let _ = parser.parse();
    }

    let universe = scope.borrow();

    // First, try to look up the simple name directly
    if let Some(loc) = universe.get_symbol_location(&word) {
        eprintln!("Found simple name: '{}' -> {:?}", word, loc);
        return create_location(uri, loc);
    }

    // If not found, check if this is a member/field access (e.g., p.x or p.square())
    if let Some(var_name) = get_variable_before_dot(line, position.character as usize) {
        eprintln!("Detected member access: var='{}', member='{}'", var_name, word);
        // Try to infer the type of the variable using the parser's type information
        let type_name = infer_variable_type_from_parser(&universe, &var_name)
            .or_else(|| {
                // Fallback to heuristic text-based parsing
                eprintln!("Parser lookup failed, trying heuristic text parsing");
                infer_variable_type_heuristic(content, &var_name)
            });

        if let Some(type_name) = type_name {
            eprintln!("Inferred type: '{}'", type_name);
            // Try qualified name with the type (AutoLang uses ".", not "::")
            let qualified_name = format!("{}.{}", type_name, word);
            eprintln!("Trying qualified name: '{}'", qualified_name);
            if let Some(loc) = universe.get_symbol_location(&qualified_name) {
                eprintln!("Found qualified name: '{}' -> {:?}", qualified_name, loc);
                return create_location(uri, loc);
            } else {
                eprintln!("Qualified name NOT found");
            }
        } else {
            eprintln!("Could not infer type for var '{}'", var_name);
        }
    }

    eprintln!("=== END GOTO DEF DEBUG ===");
    None
}

/// Create an LSP Location from a SymbolLocation
fn create_location(uri: &str, loc: &auto_lang::SymbolLocation) -> Option<GotoDefinitionResponse> {
    let uri_parsed = Url::parse(uri).ok()?;

    // Adjust character position (subtract 1 to fix off-by-one issue)
    let char = loc.character.saturating_sub(1);

    let location = Location {
        uri: uri_parsed,
        range: Range {
            start: Position {
                line: loc.line as u32,
                character: char as u32,
            },
            end: Position {
                line: loc.line as u32,
                character: char as u32,
            },
        },
    };

    Some(GotoDefinitionResponse::Scalar(location))
}

/// Get the word at the given cursor position
fn get_word_at_position(line: &str, cursor: usize) -> Option<String> {
    let chars: Vec<char> = line.chars().collect();

    if cursor >= chars.len() {
        return None;
    }

    // Find the start of the word
    let mut start = cursor;
    while start > 0 && is_identifier_char(chars[start - 1]) {
        start -= 1;
    }

    // Find the end of the word
    let mut end = cursor;
    while end < chars.len() && is_identifier_char(chars[end]) {
        end += 1;
    }

    if start < end {
        Some(chars[start..end].iter().collect())
    } else {
        None
    }
}

/// Check if a character is part of an identifier
fn is_identifier_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// Get the variable name before a dot (for member/field access)
/// Returns the variable name if the cursor is after a dot, e.g., for "p.x" returns "p"
fn get_variable_before_dot(line: &str, cursor: usize) -> Option<String> {
    let chars: Vec<char> = line.chars().collect();

    // Look backwards from cursor to find a dot
    let mut i = cursor;
    while i > 0 && !chars[i - 1].is_whitespace() {
        i -= 1;
        if chars[i] == '.' {
            // Found a dot, now extract the variable name before it
            let var_end = i;
            let mut var_start = i;
            while var_start > 0 && is_identifier_char(chars[var_start - 1]) {
                var_start -= 1;
            }
            if var_start < var_end {
                return Some(chars[var_start..var_end].iter().collect());
            }
        }
    }

    None
}

/// Infer the type of a variable using the parser's type information
/// This tries to use the parser's metadata, with a fallback to text-based heuristics
fn infer_variable_type_from_parser(universe: &auto_lang::Universe, var_name: &str) -> Option<String> {
    use auto_lang::scope::Meta;

    // Try to use the parser's type information first
    if let Some(meta) = universe.lookup_meta(var_name) {
        match meta.as_ref() {
            Meta::Store(store) => {
                let type_name = store.ty.unique_name();
                eprintln!("Parser inferred type for '{}': '{}'", var_name, type_name);
                return Some(type_name.to_string());
            }
            _ => {
                eprintln!("Variable '{}' found but is not a Store: {:?}", var_name, meta);
            }
        }
    }

    // lookup_meta failed, return None to signal we need fallback
    None
}

/// Fallback: Infer the type of a variable by looking at its declaration (text-based heuristic)
/// This is used when the parser's scope lookup fails
fn infer_variable_type_heuristic(content: &str, var_name: &str) -> Option<String> {
    let lines: Vec<&str> = content.lines().collect();

    for line in lines {
        let trimmed = line.trim();

        // Look for patterns like "let p Point" or "mut p Point" (AutoLang syntax)
        if trimmed.starts_with("let ") || trimmed.starts_with("mut ") {
            // Split by '=' to separate declaration from initialization
            if let Some(eq_pos) = trimmed.find('=') {
                let before_eq = trimmed[..eq_pos].trim();
                // Split by whitespace to get parts
                let parts: Vec<&str> = before_eq.split_whitespace().collect();

                // Pattern: "let p Point" or "mut p Point"
                // parts[0] = "let" or "mut"
                // parts[1] = variable name
                // parts[2] = type name
                if parts.len() >= 3 {
                    let keyword = parts[0];
                    if (keyword == "let" || keyword == "mut") && parts[1] == var_name {
                        return Some(parts[2].to_string());
                    }
                }
            }

            // Also look for patterns like "let p = Point {" where type can be inferred from initialization
            if let Some(eq_pos) = trimmed.find('=') {
                let before_eq = trimmed[..eq_pos].trim();
                let parts: Vec<&str> = before_eq.split_whitespace().collect();

                // Check if it's "let p" or "mut p" without explicit type
                if (parts[0] == "let" || parts[0] == "mut") && parts.len() == 2 && parts[1] == var_name {
                    let after_eq = trimmed[eq_pos + 1..].trim();
                    // Look for type name before '{'
                    if let Some(brace_pos) = after_eq.find('{') {
                        let type_name = after_eq[..brace_pos].trim();
                        return Some(type_name.to_string());
                    }
                }
            }
        }
    }

    None
}

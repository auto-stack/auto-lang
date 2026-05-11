use tower_lsp_server::ls_types::*;
use auto_lang::ast::Stmt;

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

    // Parse the code to get AST and inference context
    let mut parser = auto_lang::Parser::from(content);
    let _ = parser.parse();

    // First, try AST-based lookup for simple names
    if let Some(loc) = find_definition_in_ast(content, &word) {
        return create_location(uri, &loc);
    }

    // If not found, check if this is a member/field access (e.g., p.x or p.square())
    if let Some(var_name) = get_variable_before_dot(line, position.character as usize) {
        // Try to infer the type of the variable using the parser's type information
        let type_name = infer_variable_type_from_parser(&parser.infer_ctx, &var_name)
            .or_else(|| {
                // Fallback to heuristic text-based parsing
                infer_variable_type_heuristic(content, &var_name)
            });

        if let Some(type_name) = type_name {
            // Try qualified name with the type (AutoLang uses ".", not "::")
            let qualified_name = format!("{}.{}", type_name, word);
            if let Some(loc) = find_definition_in_ast(content, &qualified_name) {
                return create_location(uri, &loc);
            }
        }
    }

    None
}

/// Find the definition of a symbol using the compiler's Indexer + Database
fn find_definition_in_ast(content: &str, name: &str) -> Option<auto_lang::SymbolLocation> {
    use auto_lang::database::Database;
    use auto_lang::indexer::Indexer;
    use auto_val::AutoStr;

    let mut parser = auto_lang::Parser::from(content);
    let ast = parser.parse().ok()?;

    // Create a Database and index the AST for accurate line numbers
    let mut db = Database::new();
    let file_id = db.insert_source("document.at", AutoStr::from(content));
    let mut indexer = Indexer::new(&mut db);
    let _ = indexer.index_ast(&ast, file_id);

    // Check for qualified name (Type.member)
    if name.contains('.') {
        let parts: Vec<&str> = name.split('.').collect();
        if parts.len() == 2 {
            let type_name = parts[0];
            let member_name = parts[1];

            // Try TypeStore for qualified lookups
            if let Ok(store) = parser.type_store.read() {
                if let Some(type_decl) = store.lookup_type_decl_str(type_name) {
                    // Check methods
                    for method in &type_decl.methods {
                        if method.name.as_str() == member_name {
                            return Some(auto_lang::SymbolLocation::new(0, 0, 0));
                        }
                    }
                    // Check fields
                    for member in &type_decl.members {
                        if member.name.as_str() == member_name {
                            return Some(auto_lang::SymbolLocation::new(0, 0, 0));
                        }
                    }
                }
            }
        }
        return None;
    }

    // Search indexed fragments for the symbol name
    for frag_id in db.get_fragments_in_file(file_id) {
        if let Some(meta) = db.get_fragment_meta(&frag_id) {
            if meta.name.as_str() == name {
                return Some(auto_lang::SymbolLocation::new(
                    meta.span.line.saturating_sub(1),
                    meta.span.column.saturating_sub(1),
                    meta.span.offset,
                ));
            }
        }
    }

    // Fallback: search for top-level definitions by line heuristic
    for (line_num, line_str) in content.lines().enumerate() {
        let trimmed = line_str.trim();
        if trimmed.starts_with("fn ") && trimmed.contains(&format!("{}(", name)) {
            return Some(auto_lang::SymbolLocation::new(line_num, 0, 0));
        }
        if trimmed.starts_with("type ") && trimmed.contains(name) {
            return Some(auto_lang::SymbolLocation::new(line_num, 0, 0));
        }
        if trimmed.starts_with("enum ") && trimmed.contains(name) {
            return Some(auto_lang::SymbolLocation::new(line_num, 0, 0));
        }
        if trimmed.starts_with("spec ") && trimmed.contains(name) {
            return Some(auto_lang::SymbolLocation::new(line_num, 0, 0));
        }
        if trimmed.starts_with("const ") && trimmed.contains(name) {
            return Some(auto_lang::SymbolLocation::new(line_num, 0, 0));
        }
    }

    None
}

/// Find definition using workspace state (cross-file aware)
pub fn find_definition_workspace(
    content: &str,
    position: Position,
    uri: &str,
    ws_state: &crate::workspace::WorkspaceState,
) -> Option<GotoDefinitionResponse> {
    let lines: Vec<&str> = content.lines().collect();
    let line = lines.get(position.line as usize)?;
    let word = get_word_at_position(line, position.character as usize)?;

    let db = &ws_state.db;

    // Check for qualified name (Type.member)
    if word.contains('.') {
        let parts: Vec<&str> = word.split('.').collect();
        if parts.len() == 2 {
            let type_name = parts[0];
            let member_name = parts[1];

            if let Ok(store) = ws_state.type_store.read() {
                if let Some(type_decl) = store.lookup_type_decl_str(type_name) {
                    // Check methods
                    for method in &type_decl.methods {
                        if method.name.as_str() == member_name {
                            // Return location in current file (type is in current file)
                            return create_location(uri, &auto_lang::SymbolLocation::new(0, 0, 0));
                        }
                    }
                }
            }
        }
        return None;
    }

    // Search all fragments across the workspace for the symbol
    for frag_id in db.all_fragment_ids() {
        if let Some(meta) = db.get_fragment_meta(&frag_id) {
            if meta.name.as_str() == word {
                // Get the file path for this fragment
                let file_path = db.get_file_path(meta.file_id)
                    .map(|p| p.to_string())
                    .unwrap_or_else(|| uri.to_string());

                let target_uri = if file_path.starts_with("file://") {
                    file_path.parse().ok()?
                } else {
                    format!("file://{}", file_path).parse().ok()?
                };

                let location = Location {
                    uri: target_uri,
                    range: Range {
                        start: Position {
                            line: meta.span.line.saturating_sub(1) as u32,
                            character: meta.span.column.saturating_sub(1) as u32,
                        },
                        end: Position {
                            line: meta.span.line.saturating_sub(1) as u32,
                            character: meta.span.column.saturating_sub(1) as u32,
                        },
                    },
                };

                return Some(GotoDefinitionResponse::Scalar(location));
            }
        }
    }

    None
}

/// Create an LSP Location from a SymbolLocation
fn create_location(uri: &str, loc: &auto_lang::SymbolLocation) -> Option<GotoDefinitionResponse> {
    let uri_parsed: Uri = uri.parse().ok()?;

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
fn infer_variable_type_from_parser(infer_ctx: &auto_lang::infer::InferenceContext, var_name: &str) -> Option<String> {
    use auto_lang::scope::Meta;

    // Try to use the parser's type information first
    if let Some(meta) = infer_ctx.lookup_meta(var_name) {
        match meta.as_ref() {
            Meta::Store(store) => {
                let type_name = store.ty.unique_name();
                return Some(type_name.to_string());
            }
            _ => {}
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

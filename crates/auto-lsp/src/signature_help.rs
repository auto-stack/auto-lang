use tower_lsp_server::ls_types::*;

/// Get signature help at the given position (single-file)
pub fn get_signature_help(content: &str, position: Position) -> Option<SignatureHelp> {
    let lines: Vec<&str> = content.lines().collect();
    let line = lines.get(position.line as usize)?;
    let before_cursor = &line[..position.character.min(line.len() as u32) as usize];

    // Find the function name before the last unmatched '('
    let (func_name, active_param) = extract_function_call(before_cursor)?;

    // Parse the document to get TypeStore
    let mut parser = auto_lang::Parser::from(content);
    let _ = parser.parse();

    let store = parser.type_store.read().ok()?;
    let fn_decl = store.lookup_fn_decl_str(&func_name)?;

    build_signature_help(fn_decl, active_param)
}

/// Workspace-aware signature help: checks workspace TypeStore too
pub fn get_signature_help_workspace(
    content: &str,
    position: Position,
    ws_state: &crate::workspace::WorkspaceState,
) -> Option<SignatureHelp> {
    let lines: Vec<&str> = content.lines().collect();
    let line = lines.get(position.line as usize)?;
    let before_cursor = &line[..position.character.min(line.len() as u32) as usize];

    let (func_name, active_param) = extract_function_call(before_cursor)?;

    // Try workspace TypeStore first (includes imported functions)
    if let Ok(store) = ws_state.type_store.read() {
        if let Some(fn_decl) = store.lookup_fn_decl_str(&func_name) {
            return build_signature_help(fn_decl, active_param);
        }
    }

    // Fall back to single-file lookup
    get_signature_help(content, position)
}

/// Build a SignatureHelp from a function declaration
fn build_signature_help(fn_decl: &auto_lang::ast::Fn, active_param: u32) -> Option<SignatureHelp> {
    let mut params = Vec::new();
    for param in &fn_decl.params {
        params.push(ParameterInformation {
            label: ParameterLabel::Simple(format!("{} {}", param.name, param.ty)),
            documentation: None,
        });
    }

    let param_labels: Vec<String> = fn_decl.params.iter()
        .map(|p| format!("{} {}", p.name, p.ty))
        .collect();

    let sig = SignatureInformation {
        label: format!("fn {}({}) {}", fn_decl.name, param_labels.join(", "), fn_decl.ret),
        documentation: None,
        parameters: Some(params),
        active_parameter: None,
    };

    Some(SignatureHelp {
        signatures: vec![sig],
        active_signature: Some(0),
        active_parameter: Some(active_param),
    })
}

/// Extract the function name and active parameter index from text before cursor
fn extract_function_call(before_cursor: &str) -> Option<(String, u32)> {
    // Find the last unmatched '('
    let mut paren_depth = 0;
    let mut last_open_paren = None;

    for (i, ch) in before_cursor.chars().rev().enumerate() {
        let pos = before_cursor.len() - i - 1;
        match ch {
            ')' => paren_depth += 1,
            '(' => {
                if paren_depth == 0 {
                    last_open_paren = Some(pos);
                    break;
                } else {
                    paren_depth -= 1;
                }
            }
            _ => {}
        }
    }

    let open_paren_pos = last_open_paren?;
    let inside_parens = &before_cursor[open_paren_pos + 1..];

    // Count commas to determine active parameter
    let active_param = inside_parens.matches(',').count() as u32;

    // Extract function name before the '('
    let before_paren = &before_cursor[..open_paren_pos];
    let func_name = extract_last_identifier(before_paren)?;

    Some((func_name, active_param))
}

/// Extract the last identifier before the given position
fn extract_last_identifier(text: &str) -> Option<String> {
    let chars: Vec<char> = text.chars().collect();
    let mut end = chars.len();

    // Skip trailing whitespace
    while end > 0 && chars[end - 1].is_whitespace() {
        end -= 1;
    }

    if end == 0 {
        return None;
    }

    let mut start = end;
    while start > 0 && is_identifier_char(chars[start - 1]) {
        start -= 1;
    }

    if start < end {
        Some(chars[start..end].iter().collect())
    } else {
        None
    }
}

fn is_identifier_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

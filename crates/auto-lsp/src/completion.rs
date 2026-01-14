use tower_lsp::lsp_types::*;

/// Complete code at a given position
pub fn complete(
    content: &str,
    position: Position,
    uri: &str,
    trigger_character: Option<char>,
) -> Vec<CompletionItem> {
    // Catch panics to prevent LSP from crashing
    std::panic::catch_unwind(|| {
        complete_impl(content, position, uri, trigger_character)
    }).unwrap_or_else(|_| {
        eprintln!("=== LSP COMPLETION PANIC ===");
        eprintln!("Completion panicked for: {}", uri);
        eprintln!("=== END PANIC ===");
        Vec::new()
    })
}

/// Implementation of code completion
fn complete_impl(
    content: &str,
    position: Position,
    _uri: &str,
    trigger_character: Option<char>,
) -> Vec<CompletionItem> {
    let mut items = Vec::new();

    // Get the line up to the cursor
    let lines: Vec<&str> = content.lines().collect();
    let line = if let Some(l) = lines.get(position.line as usize) {
        l
    } else {
        return keyword_completions();
    };

    let before_cursor = &line[..position.character.min(line.len() as u32) as usize];

    // If explicitly triggered by '.', force field access mode
    // and pass the trigger info to the var extraction function
    let context = if trigger_character == Some('.') {
        CompletionContext::FieldAccess
    } else {
        determine_completion_context(before_cursor, content, position)
    };

    match context {
        CompletionContext::Type => {
            items.extend(type_completions(content));
        }
        CompletionContext::Function => {
            items.extend(function_name_completions(content));
        }
        CompletionContext::Variable => {
            items.extend(variable_completions(content, position));
        }
        CompletionContext::FieldAccess => {
            // Extract the variable name before the dot
            // Pass trigger character to handle case where dot isn't in document yet
            if let Some(var_name) = extract_var_before_dot(before_cursor, trigger_character) {
                items.extend(member_completions(content, &var_name));
            } else {
                // Fallback to generic completions if we can't extract the variable
                items.extend(generic_field_completions());
            }
        }
        CompletionContext::Keyword | CompletionContext::Unknown => {
            items.extend(keyword_completions());
        }
    }

    items
}

/// Context for completion
#[derive(Debug)]
enum CompletionContext {
    /// After a colon (type annotation)
    Type,
    /// After 'fn' keyword (function name)
    Function,
    /// In expression (variable/function reference)
    Variable,
    /// After a dot (field access)
    FieldAccess,
    /// Default keyword context
    Keyword,
    /// Unknown context
    Unknown,
}

/// Determine the completion context from the code before the cursor
fn determine_completion_context(
    before_cursor: &str,
    _full_content: &str,
    _position: Position,
) -> CompletionContext {
    // Trim whitespace
    let trimmed = before_cursor.trim_end();

    // Check for field access (dot)
    if trimmed.ends_with('.') {
        return CompletionContext::FieldAccess;
    }

    // Check for type annotation (colon)
    if trimmed.ends_with(':') {
        return CompletionContext::Type;
    }

    // Check for function declaration
    if let Some(idx) = trimmed.rfind("fn ") {
        let after_fn = &trimmed[idx + 3..];
        // If there's only whitespace after 'fn', we're completing the function name
        if after_fn.trim().is_empty() || after_fn.chars().all(|c| c.is_whitespace()) {
            return CompletionContext::Function;
        }
    }

    // Check if we're in an expression (default to variable completion)
    if trimmed.chars().last().map_or(false, |c| c.is_alphanumeric() || c == '_') {
        return CompletionContext::Variable;
    }

    // Default to keyword completion
    CompletionContext::Keyword
}

/// Get all keyword completions
fn keyword_completions() -> Vec<CompletionItem> {
    vec![
        completion_item("fn", "Define a function", CompletionItemKind::FUNCTION, "fn name() {\n    \n}"),
        completion_item("let", "Declare immutable variable", CompletionItemKind::KEYWORD, "let ${1:name} = ${2:value};"),
        completion_item("mut", "Declare mutable variable", CompletionItemKind::KEYWORD, "mut ${1:name} = ${2:value};"),
        completion_item("const", "Declare constant", CompletionItemKind::CONSTANT, "const ${1:name} = ${2:value};"),
        completion_item("var", "Declare variable (inferred type)", CompletionItemKind::KEYWORD, "var ${1:name} = ${2:value};"),
        completion_item("type", "Define a type alias", CompletionItemKind::KEYWORD, "type ${1:Name} = ${2:Type};"),
        completion_item("if", "If statement", CompletionItemKind::KEYWORD, "if ${1:condition} {\n    \n}"),
        completion_item("else", "Else statement", CompletionItemKind::KEYWORD, "else {\n    \n}"),
        completion_item("elif", "Else if statement", CompletionItemKind::KEYWORD, "elif ${1:condition} {\n    \n}"),
        completion_item("while", "While loop", CompletionItemKind::KEYWORD, "while ${1:condition} {\n    \n}"),
        completion_item("for", "For loop", CompletionItemKind::KEYWORD, "for ${1:item} in ${2:iterable} {\n    \n}"),
        completion_item("loop", "Infinite loop", CompletionItemKind::KEYWORD, "loop {\n    \n}"),
        completion_item("break", "Break from loop", CompletionItemKind::KEYWORD, "break;"),
        completion_item("continue", "Continue to next iteration", CompletionItemKind::KEYWORD, "continue;"),
        completion_item("return", "Return from function", CompletionItemKind::KEYWORD, "return ${1:value};"),
        completion_item("match", "Pattern matching", CompletionItemKind::KEYWORD, "match ${1:value} {\n    ${2:pattern} => ${3:result}\n}"),
        completion_item("use", "Import module", CompletionItemKind::KEYWORD, "use ${1:module};"),
        completion_item("mod", "Define module", CompletionItemKind::KEYWORD, "mod ${1:name};"),
        completion_item("struct", "Define struct", CompletionItemKind::STRUCT, "struct ${1:Name} {\n    ${2:field}: ${3:Type}\n}"),
        completion_item("enum", "Define enum", CompletionItemKind::ENUM, "enum ${1:Name} {\n    ${2:Variant}\n}"),
        completion_item("trait", "Define trait", CompletionItemKind::INTERFACE, "trait ${1:Name} {\n    ${2:method}()\n}"),
        completion_item("impl", "Implement trait or methods", CompletionItemKind::KEYWORD, "impl ${1:Type} {\n    \n}"),
        completion_item("true", "Boolean true", CompletionItemKind::KEYWORD, "true"),
        completion_item("false", "Boolean false", CompletionItemKind::KEYWORD, "false"),
        completion_item("nil", "Nil value", CompletionItemKind::KEYWORD, "nil"),
    ]
}

/// Get all type completions
fn type_completions(content: &str) -> Vec<CompletionItem> {
    let mut items = vec![
        // Primitive types
        completion_item("int", "Signed integer", CompletionItemKind::TYPE_PARAMETER, "int"),
        completion_item("uint", "Unsigned integer", CompletionItemKind::TYPE_PARAMETER, "uint"),
        completion_item("float", "Floating point number", CompletionItemKind::TYPE_PARAMETER, "float"),
        completion_item("bool", "Boolean value", CompletionItemKind::TYPE_PARAMETER, "bool"),
        completion_item("str", "String", CompletionItemKind::TYPE_PARAMETER, "str"),
        completion_item("char", "Character", CompletionItemKind::TYPE_PARAMETER, "char"),
        // Composite types
        completion_item("array", "Array type", CompletionItemKind::TYPE_PARAMETER, "array[T]"),
        completion_item("list", "List type", CompletionItemKind::TYPE_PARAMETER, "list[T]"),
        completion_item("dict", "Dictionary type", CompletionItemKind::TYPE_PARAMETER, "dict[K, V]"),
        completion_item("object", "Object type", CompletionItemKind::TYPE_PARAMETER, "object"),
    ];

    // Add user-defined types from AST
    items.extend(user_defined_types(content));

    items
}

/// Get function name completions
fn function_name_completions(content: &str) -> Vec<CompletionItem> {
    let mut items = vec![
        completion_item("main", "Main entry point", CompletionItemKind::FUNCTION, "fn main() {\n    \n}"),
    ];

    // Add standard library functions
    items.extend(stdlib_function_completions());

    // Add user-defined functions from content (TODO: parse and extract functions)
    items.extend(user_defined_functions(content));

    items
}

/// Get variable completions
fn variable_completions(content: &str, position: Position) -> Vec<CompletionItem> {
    let mut items = Vec::new();

    // Extract variables from the AST
    if let Ok(ast) = auto_lang::parse_preserve_error(content) {
        items.extend(extract_variables_from_ast(&ast, content, position));
    }

    items
}

/// Get field completions (after dot) - DEPRECATED, use member_completions instead
fn field_completions() -> Vec<CompletionItem> {
    generic_field_completions()
}

/// Get generic field completions for built-in types
fn generic_field_completions() -> Vec<CompletionItem> {
    vec![
        completion_item("len", "Get length", CompletionItemKind::METHOD, "len"),
        completion_item("push", "Add element", CompletionItemKind::METHOD, "push(${1:value})"),
        completion_item("pop", "Remove last element", CompletionItemKind::METHOD, "pop()"),
        completion_item("get", "Get element at index", CompletionItemKind::METHOD, "get(${1:index})"),
        completion_item("set", "Set element at index", CompletionItemKind::METHOD, "set(${1:index}, ${2:value})"),
    ]
}

/// Extract the variable name before a dot
/// If trigger_dot is Some('.'), it means the dot was just typed but might not be in the document yet
fn extract_var_before_dot(before_cursor: &str, trigger_dot: Option<char>) -> Option<String> {
    // Check if it ends with a dot, or if the dot was just triggered
    let has_dot = before_cursor.ends_with('.') || trigger_dot == Some('.');

    if !has_dot {
        return None;
    }

    // Remove the dot if present and get the variable name
    let before_dot = if before_cursor.ends_with('.') {
        &before_cursor[..before_cursor.len() - 1]
    } else {
        before_cursor
    };

    // Extract the identifier (walk backwards while alphanumeric or underscore)
    let chars: Vec<char> = before_dot.chars().collect();
    let mut end = chars.len();

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

/// Check if a character is part of an identifier
fn is_identifier_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// Get member completions (fields and methods) for a variable
fn member_completions(content: &str, var_name: &str) -> Vec<CompletionItem> {
    let mut items = Vec::new();

    // Try to infer the type of the variable using scope-aware type inference
    let type_name = infer_variable_type_from_parser_with_scope(content, var_name)
        .or_else(|| infer_variable_type_heuristic(content, var_name));

    if let Some(type_name) = type_name {
        // Parse the code to get AST
        if let Ok(ast) = auto_lang::parse_preserve_error(content) {
            // Look for the type definition
            for stmt in ast.stmts.iter() {
                if let auto_lang::ast::Stmt::TypeDecl(type_decl) = stmt {
                    if type_decl.name.as_str() == type_name {

                        // Add fields
                        for member in &type_decl.members {
                            items.push(completion_item(
                                &member.name.to_string(),
                                &format!("Field: {}", member.ty),
                                CompletionItemKind::FIELD,
                                &member.name.to_string(),
                            ));
                        }

                        // Add methods
                        for method in &type_decl.methods {
                            let sig = format_function_signature_for_completion(method);
                            items.push(completion_item(
                                &method.name.to_string(),
                                &sig,
                                CompletionItemKind::METHOD,
                                &format!("{}(", method.name),
                            ));
                        }

                        break;
                    }
                }
            }
        }
    }

    // If no type found, return generic completions
    if items.is_empty() {
        items.extend(generic_field_completions());
    }

    items
}

/// Infer the type of a variable using the parser's type information with proper scope navigation
/// This is a wrapper that parses the code and infers type from the correct scope
fn infer_variable_type_from_parser_with_scope(content: &str, var_name: &str) -> Option<String> {
    use auto_lang::scope::Meta;

    // Find which function contains the variable (use any line, just need to find the function)
    // We'll search through the AST to find which function contains this variable
    if let Ok(ast) = auto_lang::parse_preserve_error(content) {
        for stmt in ast.stmts.iter() {
            if let auto_lang::ast::Stmt::Fn(fn_decl) = stmt {
                // Check if this function body contains the variable
                let mut found = false;
                for body_stmt in fn_decl.body.stmts.iter() {
                    if let auto_lang::ast::Stmt::Store(store) = body_stmt {
                        if store.name.as_str() == var_name {
                            found = true;
                            break;
                        }
                    }
                }

                // Also check parameters
                if !found {
                    for param in &fn_decl.params {
                        if param.name.as_str() == var_name {
                            found = true;
                            break;
                        }
                    }
                }

                if found {
                    // Found the function, now navigate to its scope
                    let navigator = std::rc::Rc::new(std::cell::RefCell::new(auto_lang::Universe::new()));
                    {
                        let mut parser = auto_lang::Parser::new(content, navigator.clone());
                        let _ = parser.parse();
                    }

                    // Navigate to the function scope
                    navigator.borrow_mut().enter_fn(&fn_decl.name.to_string());

                    // Try to lookup the variable from within the function scope
                    let universe = navigator.borrow();
                    if let Some(meta) = universe.lookup_meta(var_name) {
                        match meta.as_ref() {
                            Meta::Store(store) => {
                                let type_name = store.ty.unique_name();
                                return Some(type_name.to_string());
                            }
                            _ => {}
                        }
                    }

                    return None;
                }
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
                return Some(type_name.to_string());
            }
            _ => {}
        }
    }

    None
}

/// Fallback: Infer the type of a variable by looking at its declaration (text-based heuristic)
fn infer_variable_type_heuristic(content: &str, var_name: &str) -> Option<String> {
    let lines: Vec<&str> = content.lines().collect();

    for line in lines {
        let trimmed = line.trim();

        if trimmed.starts_with("let ") || trimmed.starts_with("mut ") {
            if let Some(eq_pos) = trimmed.find('=') {
                let before_eq = trimmed[..eq_pos].trim();
                let parts: Vec<&str> = before_eq.split_whitespace().collect();

                if parts.len() >= 3 {
                    let keyword = parts[0];
                    if (keyword == "let" || keyword == "mut") && parts[1] == var_name {
                        return Some(parts[2].to_string());
                    }
                }
            }

            if let Some(eq_pos) = trimmed.find('=') {
                let before_eq = trimmed[..eq_pos].trim();
                let parts: Vec<&str> = before_eq.split_whitespace().collect();

                if (parts[0] == "let" || parts[0] == "mut") && parts.len() == 2 && parts[1] == var_name {
                    let after_eq = trimmed[eq_pos + 1..].trim();
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

/// Format function signature for completion display
fn format_function_signature_for_completion(fn_decl: &auto_lang::ast::Fn) -> String {
    let mut sig = format!("fn {}(", fn_decl.name);

    // Format parameters
    let params: Vec<String> = fn_decl
        .params
        .iter()
        .map(|p| format!("{} {}", p.name, p.ty))
        .collect();

    sig.push_str(&params.join(", "));
    sig.push(')');

    // Add return type
    sig.push_str(&format!(" {}", fn_decl.ret));

    sig
}

/// Get standard library function completions
fn stdlib_function_completions() -> Vec<CompletionItem> {
    vec![
        // I/O functions
        completion_item("print", "Print to stdout", CompletionItemKind::FUNCTION, "print(${1:value})"),
        completion_item("println", "Print with newline", CompletionItemKind::FUNCTION, "println(${1:value})"),
        completion_item("input", "Read from stdin", CompletionItemKind::FUNCTION, "input()"),
        completion_item("read_file", "Read file contents", CompletionItemKind::FUNCTION, "read_file(${1:path})"),
        completion_item("write_file", "Write to file", CompletionItemKind::FUNCTION, "write_file(${1:path}, ${2:content})"),
        // String functions
        completion_item("str_len", "Get string length", CompletionItemKind::FUNCTION, "str_len(${1:s})"),
        completion_item("str_substr", "Get substring", CompletionItemKind::FUNCTION, "str_substr(${1:s}, ${2:start}, ${3:length})"),
        completion_item("str_split", "Split string", CompletionItemKind::FUNCTION, "str_split(${1:s}, ${2:delimiter})"),
        completion_item("str_trim", "Trim whitespace", CompletionItemKind::FUNCTION, "str_trim(${1:s})"),
        completion_item("str_upper", "Convert to uppercase", CompletionItemKind::FUNCTION, "str_upper(${1:s})"),
        completion_item("str_lower", "Convert to lowercase", CompletionItemKind::FUNCTION, "str_lower(${1:s})"),
        // Math functions
        completion_item("abs", "Absolute value", CompletionItemKind::FUNCTION, "abs(${1:n})"),
        completion_item("min", "Minimum of two values", CompletionItemKind::FUNCTION, "min(${1:a}, ${2:b})"),
        completion_item("max", "Maximum of two values", CompletionItemKind::FUNCTION, "max(${1:a}, ${2:b})"),
        completion_item("pow", "Power function", CompletionItemKind::FUNCTION, "pow(${1:base}, ${2:exp})"),
        completion_item("sqrt", "Square root", CompletionItemKind::FUNCTION, "sqrt(${1:n})"),
        completion_item("sin", "Sine", CompletionItemKind::FUNCTION, "sin(${1:x})"),
        completion_item("cos", "Cosine", CompletionItemKind::FUNCTION, "cos(${1:x})"),
        completion_item("tan", "Tangent", CompletionItemKind::FUNCTION, "tan(${1:x})"),
        // Array functions
        completion_item("array_new", "Create new array", CompletionItemKind::FUNCTION, "array_new()"),
        completion_item("array_push", "Push to array", CompletionItemKind::FUNCTION, "array_push(${1:arr}, ${2:value})"),
        completion_item("array_pop", "Pop from array", CompletionItemKind::FUNCTION, "array_pop(${1:arr})"),
        completion_item("array_len", "Get array length", CompletionItemKind::FUNCTION, "array_len(${1:arr})"),
        // Conversion functions
        completion_item("int", "Convert to integer", CompletionItemKind::FUNCTION, "int(${1:value})"),
        completion_item("float", "Convert to float", CompletionItemKind::FUNCTION, "float(${1:value})"),
        completion_item("str", "Convert to string", CompletionItemKind::FUNCTION, "str(${1:value})"),
        completion_item("bool", "Convert to boolean", CompletionItemKind::FUNCTION, "bool(${1:value})"),
    ]
}

/// Extract user-defined types from AST
fn user_defined_types(content: &str) -> Vec<CompletionItem> {
    let mut items = Vec::new();

    // Parse the code to get AST
    if let Ok(ast) = auto_lang::parse_preserve_error(content) {
        // Extract type definitions
        for stmt in ast.stmts.iter() {
            if let auto_lang::ast::Stmt::TypeDecl(type_decl) = stmt {
                items.push(completion_item(
                    &type_decl.name.to_string(),
                    &format!("Type: {}", type_decl.name),
                    CompletionItemKind::STRUCT,
                    &type_decl.name.to_string(),
                ));
            }
        }
    }

    items
}

/// Extract user-defined functions from content
fn user_defined_functions(content: &str) -> Vec<CompletionItem> {
    let mut items = Vec::new();

    // Simple regex-based extraction for now
    // TODO: Parse AST properly to extract functions with signatures
    use regex::Regex;

    let fn_regex = Regex::new(r"fn\s+(\w+)\s*\(").unwrap();
    for cap in fn_regex.captures_iter(content) {
        if let Some(name) = cap.get(1) {
            items.push(completion_item(
                name.as_str(),
                &format!("Function: {}", name.as_str()),
                CompletionItemKind::FUNCTION,
                &format!("{}(", name.as_str()),
            ));
        }
    }

    items
}

/// Extract variables in scope from AST
fn extract_variables_from_ast(
    _ast: &auto_lang::ast::Code,
    content: &str,
    _position: Position,
) -> Vec<CompletionItem> {
    let mut items = Vec::new();

    // Simple regex-based extraction for now
    // TODO: Parse AST properly to extract variables in scope
    use regex::Regex;

    // Extract let, mut, const, var declarations
    let var_regex = Regex::new(r"(let|mut|const|var)\s+(\w+)").unwrap();
    for cap in var_regex.captures_iter(content) {
        if let Some(name) = cap.get(2) {
            let kind = match &cap[1] {
                "const" => CompletionItemKind::CONSTANT,
                _ => CompletionItemKind::VARIABLE,
            };
            items.push(completion_item(
                name.as_str(),
                &format!("Variable: {}", name.as_str()),
                kind,
                name.as_str(),
            ));
        }
    }

    // Extract function parameters
    let param_regex = Regex::new(r"fn\s+\w+\s*\(([^)]*)\)").unwrap();
    for cap in param_regex.captures_iter(content) {
        if let Some(params) = cap.get(1) {
            let params_str = params.as_str();
            // Extract parameter names
            for param in params_str.split(',') {
                let param = param.trim();
                if let Some(name) = param.split_whitespace().next() {
                    if name != ":" {
                        items.push(completion_item(
                            name,
                            &format!("Parameter: {}", name),
                            CompletionItemKind::VARIABLE,
                            name,
                        ));
                    }
                }
            }
        }
    }

    items
}

/// Create a completion item
fn completion_item(
    label: &str,
    detail: &str,
    kind: CompletionItemKind,
    insert_text: &str,
) -> CompletionItem {
    CompletionItem {
        label: label.to_string(),
        label_details: None,
        kind: Some(kind),
        detail: Some(detail.to_string()),
        documentation: None,
        deprecated: None,
        preselect: None,
        sort_text: None,
        filter_text: None,
        insert_text: Some(insert_text.to_string()),
        insert_text_format: Some(InsertTextFormat::SNIPPET),
        insert_text_mode: None,
        text_edit: None,
        additional_text_edits: None,
        commit_characters: None,
        command: None,
        data: None,
        tags: None,
    }
}

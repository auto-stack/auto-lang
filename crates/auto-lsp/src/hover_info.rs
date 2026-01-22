use tower_lsp::lsp_types::*;
use auto_lang::ast::Stmt;

/// Provide hover information at a given position
pub fn hover(content: &str, position: Position, uri: &str) -> Option<Hover> {
    // Catch panics to prevent LSP from crashing
    std::panic::catch_unwind(|| {
        hover_impl(content, position, uri)
    }).unwrap_or_else(|_| None)
}

/// Implementation of hover
fn hover_impl(content: &str, position: Position, _uri: &str) -> Option<Hover> {
    // Get the line at the cursor position
    let lines: Vec<&str> = content.lines().collect();
    let line = lines.get(position.line as usize)?;

    // Get the word at the cursor position
    let word = get_word_at_position(line, position.character as usize)?;

    // First, check if it's a user-defined type or function in the current file
    if let Some(docs) = get_user_defined_docs(content, &word) {
        return Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: docs,
            }),
            range: None,
        });
    }

    // Check if it's a method or field access (e.g., p.x or p.square())
    if let Some(var_name) = get_variable_before_dot(line, position.character as usize) {
        // Try to infer the type of the variable using the parser's type information
        let type_name = infer_variable_type_from_parser_with_scope(content, position, &var_name)
            .or_else(|| infer_variable_type_heuristic(content, &var_name));

        if let Some(type_name) = type_name {
            // Try qualified name with the type (AutoLang uses ".", not "::")
            let qualified_name = format!("{}.{}", type_name, word);
            if let Some(docs) = get_user_defined_docs(content, &qualified_name) {
                return Some(Hover {
                    contents: HoverContents::Markup(MarkupContent {
                        kind: MarkupKind::Markdown,
                        value: docs,
                    }),
                    range: None,
                });
            }
        }
    }

    // Check if it's a keyword
    if let Some(docs) = get_keyword_docs(&word) {
        return Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: docs,
            }),
            range: None,
        });
    }

    // Check if it's a built-in type
    if let Some(docs) = get_type_docs(&word) {
        return Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: docs,
            }),
            range: None,
        });
    }

    // Check if it's a stdlib function
    if let Some(docs) = get_stdlib_function_docs(&word) {
        return Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: docs,
            }),
            range: None,
        });
    }

    None
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

/// Infer the type of a variable using the parser's type information with proper scope navigation
/// This is a wrapper that parses the code and infers type from the correct scope
fn infer_variable_type_from_parser_with_scope(content: &str, position: Position, var_name: &str) -> Option<String> {
    use auto_lang::scope::Meta;

    // If we can find which function contains the cursor, navigate to its scope
    if let Some(fn_name) = find_function_at_position(content, position) {
        eprintln!("HOVER: Cursor is in function '{}', trying to navigate to its scope", fn_name);

        // Create a universe to navigate scopes
        let navigator = std::rc::Rc::new(std::cell::RefCell::new(auto_lang::Universe::new()));
        {
            let mut parser = auto_lang::Parser::new(content, navigator.clone());
            let _ = parser.parse();
        }

        // Navigate to the function scope
        navigator.borrow_mut().enter_fn(&fn_name);
        eprintln!("HOVER: Entered function scope '{}'", fn_name);

        // Now try to lookup the variable from within the function scope
        let universe = navigator.borrow();
        if let Some(meta) = universe.lookup_meta(var_name) {
            match meta.as_ref() {
                Meta::Store(store) => {
                    let type_name = store.ty.unique_name();
                    eprintln!("HOVER: Found '{}' in function scope '{}': '{}'", var_name, fn_name, type_name);
                    return Some(type_name.to_string());
                }
                _ => {
                    eprintln!("HOVER: Variable '{}' found in function scope but is not a Store: {:?}", var_name, meta);
                }
            }
        }

        drop(universe);
        navigator.borrow_mut().exit_fn();
        eprintln!("HOVER: Exited function scope '{}'", fn_name);

        eprintln!("HOVER: Could not find variable '{}' in function scope", var_name);
        None
    } else {
        // Not in a function, try global scope
        eprintln!("HOVER: Not in a function scope, trying global scope");

        let scope = std::rc::Rc::new(std::cell::RefCell::new(auto_lang::Universe::new()));
        {
            let mut parser = auto_lang::Parser::new(content, scope.clone());
            let _ = parser.parse();
        }
        let universe = scope.borrow();

        if let Some(meta) = universe.lookup_meta(var_name) {
            match meta.as_ref() {
                Meta::Store(store) => {
                    let type_name = store.ty.unique_name();
                    eprintln!("HOVER: Found '{}' at global scope: '{}'", var_name, type_name);
                    return Some(type_name.to_string());
                }
                _ => {
                    eprintln!("HOVER: Variable '{}' found but is not a Store: {:?}", var_name, meta);
                }
            }
        }

        eprintln!("HOVER: Could not find variable '{}' using parser metadata", var_name);
        None
    }
}

/// Find which function contains the given cursor position
/// Returns the function name if the position is inside a function body
fn find_function_at_position(content: &str, position: Position) -> Option<String> {
    use auto_lang::ast::Stmt;

    let cursor_line = position.line as usize;

    // Parse the code to get the AST
    let ast = auto_lang::parse_preserve_error(content).ok()?;

    // Iterate through statements to find functions
    for stmt in ast.stmts.iter() {
        if let Stmt::Fn(fn_decl) = stmt {
            // We need to find the line range of this function
            // The AST doesn't store line numbers, so we'll need to find the function in the source
            if let Some((start_line, end_line)) = find_function_range_in_source(content, &fn_decl.name) {
                if cursor_line >= start_line && cursor_line <= end_line {
                    eprintln!("HOVER: Found function '{}' at lines {}-{}, cursor at line {}",
                        fn_decl.name, start_line, end_line, cursor_line);
                    return Some(fn_decl.name.to_string());
                }
            }
        }
    }

    None
}

/// Find the line range of a function in the source code
/// Returns (start_line, end_line) or None if not found
fn find_function_range_in_source(content: &str, fn_name: &str) -> Option<(usize, usize)> {
    let lines: Vec<&str> = content.lines().collect();

    // Find the function declaration
    let mut start_line = None;
    let mut brace_count = 0;
    let mut found_opening_brace = false;

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        // Look for "fn name" pattern
        if !found_opening_brace && trimmed.starts_with("fn ") && trimmed.contains(&format!("{}(", fn_name)) {
            start_line = Some(i);
            // Count braces in this line
            brace_count += line.matches('{').count() as i32 - line.matches('}').count() as i32;
            if brace_count > 0 {
                found_opening_brace = true;
            }
            continue;
        }

        // If we found the function, count braces to find the end
        if start_line.is_some() {
            brace_count += line.matches('{').count() as i32 - line.matches('}').count() as i32;
            if brace_count > 0 {
                found_opening_brace = true;
            }

            // When brace count returns to 0 (or negative), we've found the end
            if found_opening_brace && brace_count <= 0 {
                return Some((start_line.unwrap(), i));
            }
        }
    }

    // If we never found the closing brace, return the start line to end of file
    start_line.map(|s| (s, lines.len() - 1))
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

/// Get documentation for user-defined types and functions in the current file
fn get_user_defined_docs(content: &str, name: &str) -> Option<String> {
    // Parse the code to get the AST
    match auto_lang::parse_preserve_error(content) {
        Ok(ast) => {
            // Check if it's a qualified name (e.g., "Point.square" or "Point.x")
            if name.contains('.') {
                let parts: Vec<&str> = name.split('.').collect();
                if parts.len() == 2 {
                    let type_name = parts[0];
                    let member_name = parts[1];

                    // Look for the type definition
                    for stmt in ast.stmts.iter() {
                        if let Stmt::TypeDecl(type_decl) = stmt {
                            if type_decl.name.as_str() == type_name {
                                // Check if it's a method
                                for method in &type_decl.methods {
                                    if method.name.as_str() == member_name {
                                        let mut docs = format_function_signature(method);
                                        docs.push_str(&format!("\n\n**Method of type `{}`**", type_name));
                                        return Some(docs);
                                    }
                                }

                                // Check if it's a field
                                for member in &type_decl.members {
                                    if member.name.as_str() == member_name {
                                        let ty_str = format!("{}", member.ty);
                                        let docs = format!("**Field** `{}` of type `{}`\n\n**Type:** `{}`",
                                            member_name, type_name, ty_str);
                                        return Some(docs);
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Look for local variables and function parameters (Store statements)
            // Also search inside function bodies (Bodies)
            for stmt in ast.stmts.iter() {
                // Check at top level
                if let Stmt::Store(store) = stmt {
                    if store.name.as_str() == name {
                        let ty_str = store.ty.unique_name().to_string();
                        let kind_str = match store.kind {
                            auto_lang::ast::StoreKind::Let => "let",
                            auto_lang::ast::StoreKind::Var => "var",
                            _ => "variable",
                        };
                        let docs = format!("**{}** `{}`\n\n**Type:** `{}`",
                            kind_str, name, ty_str);
                        return Some(docs);
                    }
                }

                // Check inside function bodies
                if let Stmt::Fn(fn_decl) = stmt {
                    // Look in the function body for local variables
                    for body_stmt in fn_decl.body.stmts.iter() {
                        if let Stmt::Store(store) = body_stmt {
                            if store.name.as_str() == name {
                                let ty_str = store.ty.unique_name().to_string();
                                let kind_str = match store.kind {
                                    auto_lang::ast::StoreKind::Let => "let",
                                    auto_lang::ast::StoreKind::Var => "var",
                                    _ => "variable",
                                };
                                let docs = format!("**{}** `{}`\n\n**Type:** `{}`\n\n**In function `{}`**",
                                    kind_str, name, ty_str, fn_decl.name);
                                return Some(docs);
                            }
                        }

                        // Check function parameters
                        for param in &fn_decl.params {
                            if param.name.as_str() == name {
                                let ty_str = param.ty.unique_name().to_string();
                                let docs = format!("**Parameter** `{}`\n\n**Type:** `{}`\n\n**Of function:** `{}`",
                                    name, ty_str, fn_decl.name);
                                return Some(docs);
                            }
                        }
                    }
                }

                // Look for type definitions
                if let Some(type_name) = extract_type_definition(stmt) {
                    if type_name == name {
                        let docs = format_type_definition(stmt);
                        return Some(docs);
                    }
                }

                if let Some(fn_name) = extract_function_definition(stmt) {
                    if fn_name == name {
                        let docs = format_function_definition(stmt);
                        return Some(docs);
                    }
                }
            }
            None
        }
        Err(_) => None,
    }
}

/// Extract the name of a type definition from a statement
fn extract_type_definition(stmt: &auto_lang::ast::Stmt) -> Option<String> {
    use auto_lang::ast::Stmt;

    match stmt {
        Stmt::TypeDecl(type_decl) => Some(type_decl.name.to_string()),
        Stmt::EnumDecl(enum_decl) => Some(enum_decl.name.to_string()),
        _ => None,
    }
}

/// Extract the name of a function definition from a statement
fn extract_function_definition(stmt: &auto_lang::ast::Stmt) -> Option<String> {
    use auto_lang::ast::Stmt;

    match stmt {
        Stmt::Fn(fn_decl) => Some(fn_decl.name.to_string()),
        _ => None,
    }
}

/// Format a type definition for hover display
fn format_type_definition(stmt: &auto_lang::ast::Stmt) -> String {
    use auto_lang::ast::Stmt;

    match stmt {
        Stmt::TypeDecl(type_decl) => {
            let mut docs = format!("**Type** `{}`\n\n", type_decl.name);

            if !type_decl.members.is_empty() {
                docs.push_str("**Members:**\n\n");
                for member in &type_decl.members {
                    let ty_str = format!("{}", member.ty);
                    docs.push_str(&format!("- `{}`: `{}`\n", member.name, ty_str));
                }
            }

            if !type_decl.methods.is_empty() {
                docs.push_str("\n**Methods:**\n\n");
                for method in &type_decl.methods {
                    docs.push_str(&format!("- `{}`\n", format_function_signature(method)));
                }
            }

            docs
        }
        Stmt::EnumDecl(enum_decl) => {
            let mut docs = format!("**Enum** `{}`\n\n", enum_decl.name);

            if !enum_decl.items.is_empty() {
                docs.push_str("**Variants:**\n\n");
                for item in &enum_decl.items {
                    docs.push_str(&format!("- `{}` = {}\n", item.name, item.value));
                }
            }

            docs
        }
        _ => "Unknown type definition".to_string(),
    }
}

/// Format a function definition for hover display
fn format_function_definition(stmt: &auto_lang::ast::Stmt) -> String {
    use auto_lang::ast::Stmt;

    match stmt {
        Stmt::Fn(fn_decl) => {
            let signature = format_function_signature(fn_decl);
            format!("**Function**\n\n```auto\n{}\n```", signature)
        }
        _ => "Unknown function definition".to_string(),
    }
}

/// Format a function signature
fn format_function_signature(fn_decl: &auto_lang::ast::Fn) -> String {
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

/// Get documentation for keywords
fn get_keyword_docs(keyword: &str) -> Option<String> {
    let docs = match keyword {
        "fn" => "Declare a function\n\n**Example:**\n```auto\nfn add(a int, b int) int {\n    return a + b\n}\n```",
        "type" => "Define a custom type\n\n**Example:**\n```auto\ntype Point {\n    x int\n    y int\n}\n```",
        "let" => "Declare a local variable with type inference\n\n**Example:**\n```auto\nlet x = 42\nlet name = \"hello\"\n```",
        "var" => "Declare a mutable variable\n\n**Example:**\n```auto\nvar counter = 0\ncounter = counter + 1\n```",
        "if" => "Conditional execution\n\n**Example:**\n```auto\nif x > 0 {\n    return \"positive\"\n}\n```",
        "else" => "Alternative branch for if statement",
        "while" => "Loop while condition is true\n\n**Example:**\n```auto\nwhile i < 10 {\n    i = i + 1\n}\n```",
        "for" => "Iterate over a range or collection\n\n**Example:**\n```auto\nfor i in 0..10 {\n    print(i)\n}\n```",
        "return" => "Return a value from a function\n\n**Example:**\n```auto\nreturn result\n```",
        "use" => "Import a library or module\n\n**Example:**\n```auto\nuse auto.io: open, File\n```",
        "import" => "Import symbols from a library",
        "as" => "Alias for imported symbols",
        "match" => "Pattern matching on values\n\n**Example:**\n```auto\nmatch value {\n    1 => \"one\"\n    2 => \"two\"\n    _ => \"other\"\n}\n```",
        "enum" => "Define an enumeration\n\n**Example:**\n```auto\nenum Color {\n    Red\n    Green\n    Blue\n}\n```",
        "struct" => "Synonym for `type` - define a structure type",
        "interface" => "Define an interface contract",
        "impl" => "Implement methods for a type",
        "true" | "false" => "Boolean literal",
        "nil" => "Null/empty value",
        _ => return None,
    };

    Some(docs.to_string())
}

/// Get documentation for built-in types
fn get_type_docs(type_name: &str) -> Option<String> {
    let docs = match type_name {
        "int" => "Signed integer number\n\n**Example:**\n```auto\nlet x int = 42\n```",
        "uint" => "Unsigned integer number\n\n**Example:**\n```auto\nlet x uint = 42u\n```",
        "i8" => "8-bit signed integer (-128 to 127)",
        "u8" => "8-bit unsigned integer (0 to 255)",
        "i16" => "16-bit signed integer",
        "u16" => "16-bit unsigned integer",
        "i32" => "32-bit signed integer",
        "u32" => "32-bit unsigned integer",
        "i64" => "64-bit signed integer",
        "u64" => "64-bit unsigned integer",
        "float" => "Floating-point number\n\n**Example:**\n```auto\nlet pi float = 3.14159\n```",
        "f32" => "32-bit floating-point number",
        "f64" => "64-bit floating-point number",
        "str" => "String text\n\n**Example:**\n```auto\nlet name str = \"hello\"\n```",
        "string" => "Alias for `str` type",
        "bool" => "Boolean value (true or false)\n\n**Example:**\n```auto\nlet flag bool = true\n```",
        "array" => "Array of values\n\n**Example:**\n```auto\nlet numbers array = [1, 2, 3]\n```",
        "Array" => "Generic array type\n\n**Example:**\n```auto\nlet numbers Array<int> = [1, 2, 3]\n```",
        "object" => "Key-value object\n\n**Example:**\n```auto\nlet config object = {name: \"test\", count: 42}\n```",
        "Object" => "Generic object type",
        "Map" => "Key-value map type\n\n**Example:**\n```auto\nlet Map<str, int> scores = {}\n```",
        _ => return None,
    };

    Some(docs.to_string())
}

/// Get documentation for stdlib functions
fn get_stdlib_function_docs(func_name: &str) -> Option<String> {
    let docs = match func_name {
        "print" => "Print a value to stdout\n\n**Signature:**\n```auto\nfn print(value any)\n```\n\n**Example:**\n```auto\nprint(\"Hello, World!\")\nprint(42)\n```",
        "println" => "Print a value to stdout with newline\n\n**Signature:**\n```auto\nfn println(value any)\n```",
        "len" => "Get the length of a string, array, or object\n\n**Signature:**\n```auto\nfn len(collection any) int\n```\n\n**Example:**\n```auto\nlet count = len([1, 2, 3])  // returns 3\n```",
        "push" => "Add an element to an array\n\n**Signature:**\n```auto\nfn push(arr array, item any)\n```\n\n**Example:**\n```auto\nlet numbers = [1, 2]\npush(numbers, 3)  // numbers is now [1, 2, 3]\n```",
        "pop" => "Remove and return the last element from an array\n\n**Signature:**\n```auto\nfn pop(arr array) any\n```",
        "first" => "Get the first element of an array",
        "last" => "Get the last element of an array",
        "rest" => "Get all elements except the first",
        "append" => "Concatenate arrays\n\n**Signature:**\n```auto\nfn append(arr1 array, arr2 array) array\n```",
        "keys" => "Get all keys from an object\n\n**Signature:**\n```auto\nfn keys(obj object) array\n```",
        "values" => "Get all values from an object",
        "entries" => "Get all key-value pairs from an object",
        "open" => "Open a file for reading\n\n**Signature:**\n```auto\nfn open(path str) File\n```\n\n**Example:**\n```auto\nlet file = open(\"data.txt\")\n```",
        "read" => "Read content from a file\n\n**Signature:**\n```auto\nfn read(file File) str\n```",
        "write" => "Write content to a file\n\n**Signature:**\n```auto\nfn write(file File, content str)\n```",
        "close" => "Close a file",
        "exists" => "Check if a file exists\n\n**Signature:**\n```auto\nfn exists(path str) bool\n```",
        "int" => "Convert a value to integer\n\n**Signature:**\n```auto\nfn int(value any) int\n```\n\n**Example:**\n```auto\nlet x = int(\"42\")  // converts string to int\n```",
        "str" => "Convert a value to string\n\n**Signature:**\n```auto\nfn str(value any) str\n```",
        "float" => "Convert a value to floating-point\n\n**Signature:**\n```auto\nfn float(value any) float\n```",
        "bool" => "Convert a value to boolean",
        "type" => "Get the type of a value\n\n**Signature:**\n```auto\nfn type(value any) str\n```",
        "abs" => "Absolute value of a number\n\n**Signature:**\n```auto\nfn abs(x int) int\n```\n\n**Example:**\n```auto\nlet x = abs(-5)  // returns 5\n```",
        "min" => "Minimum of two values",
        "max" => "Maximum of two values",
        "sqrt" => "Square root of a number",
        "pow" => "Power function\n\n**Signature:**\n```auto\nfn pow(base int, exp int) int\n```",
        "sin" => "Sine trigonometric function (radians)",
        "cos" => "Cosine trigonometric function (radians)",
        "tan" => "Tangent trigonometric function (radians)",
        "log" => "Natural logarithm",
        "log10" => "Base-10 logarithm",
        "floor" => "Round down to nearest integer",
        "ceil" => "Round up to nearest integer",
        "round" => "Round to nearest integer",
        _ => return None,
    };

    Some(docs.to_string())
}

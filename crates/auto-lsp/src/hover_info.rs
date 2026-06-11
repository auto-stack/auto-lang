use tower_lsp_server::ls_types::*;

/// Workspace-aware hover: tries workspace TypeStore before single-file lookup
pub fn hover_workspace(
    content: &str,
    position: Position,
    uri: &str,
    ws_state: &crate::workspace::WorkspaceState,
) -> Option<Hover> {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        hover_workspace_impl(content, position, uri, ws_state)
    })).unwrap_or_else(|_| None)
}

fn hover_workspace_impl(
    content: &str,
    position: Position,
    _uri: &str,
    ws_state: &crate::workspace::WorkspaceState,
) -> Option<Hover> {
    let lines: Vec<&str> = content.lines().collect();
    let line = lines.get(position.line as usize)?;
    let word = get_word_at_position(line, position.character as usize)?;

    // Try workspace TypeStore first (includes imported symbols)
    if let Ok(store) = ws_state.type_store.read() {
        if let Some(docs) = get_typestore_docs_direct(&store, &word) {
            return Some(Hover {
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: docs,
                }),
                range: None,
            });
        }
    }

    // Fall back to single-file hover
    hover(content, position, _uri)
}

/// Get documentation from TypeStore directly (for workspace-level lookups)
fn get_typestore_docs_direct(
    store: &auto_lang::types::TypeStore,
    name: &str,
) -> Option<String> {
    if let Some(fn_decl) = store.lookup_fn_decl_str(name) {
        let sig = format_function_signature(fn_decl);
        return Some(format!("**Function**\n\n```auto\n{}\n```", sig));
    }

    if let Some(type_decl) = store.lookup_type_decl_str(name) {
        let mut docs = format!("**Type** `{}`\n\n", type_decl.name);
        if !type_decl.members.is_empty() {
            docs.push_str("**Members:**\n\n");
            for member in &type_decl.members {
                docs.push_str(&format!("- `{}`: `{}`\n", member.name, member.ty));
            }
        }
        if !type_decl.methods.is_empty() {
            docs.push_str("\n**Methods:**\n\n");
            for method in &type_decl.methods {
                docs.push_str(&format!("- `{}`\n", format_function_signature(method)));
            }
        }
        return Some(docs);
    }

    if let Some(spec_decl) = store.lookup_spec_decl_str(name) {
        let mut docs = format!("**Spec** `{}`\n\n", spec_decl.name);
        if !spec_decl.methods.is_empty() {
            docs.push_str("**Methods:**\n\n");
            for method in &spec_decl.methods {
                let params: Vec<String> = method.params.iter().map(|p| format!("{} {}", p.name, p.ty)).collect();
                docs.push_str(&format!("- `fn {}({}) {}`\n", method.name, params.join(", "), method.ret));
            }
        }
        return Some(docs);
    }

    None
}

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
fn infer_variable_type_from_parser_with_scope(content: &str, _position: Position, var_name: &str) -> Option<String> {
    use auto_lang::scope::Meta;

    let mut parser = auto_lang::Parser::from(content);
    let _ = parser.parse();
    let infer_ctx = &parser.infer_ctx;

    if let Some(meta) = infer_ctx.lookup_meta(var_name) {
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

/// Get documentation for user-defined types and functions in the current file
fn get_user_defined_docs(content: &str, name: &str) -> Option<String> {
    let mut parser = auto_lang::Parser::from(content);
    let ast = parser.parse().ok()?;

    // Try TypeStore lookup first (most accurate for types, functions, specs)
    if let Some(docs) = get_typestore_docs(&parser.type_store, name) {
        return Some(docs);
    }

    // Fall back to AST traversal for local variables and parameters
    get_ast_docs(&ast, name)
}

/// Get documentation from the parser's TypeStore
fn get_typestore_docs(
    type_store: &std::sync::Arc<std::sync::RwLock<auto_lang::types::TypeStore>>,
    name: &str,
) -> Option<String> {
    let store = type_store.read().ok()?;

    // Check if it's a qualified name (e.g., "Point.square" or "Point.x")
    if name.contains('.') {
        let parts: Vec<&str> = name.split('.').collect();
        if parts.len() == 2 {
            let type_name = parts[0];
            let member_name = parts[1];

            if let Some(type_decl) = store.lookup_type_decl_str(type_name) {
                // Check methods
                for method in &type_decl.methods {
                    if method.name.as_str() == member_name {
                        let sig = format_function_signature(method);
                        return Some(format!("{}\n\n**Method of type `{}`**", sig, type_name));
                    }
                }
                // Check fields
                for member in &type_decl.members {
                    if member.name.as_str() == member_name {
                        let ty_str = format!("{}", member.ty);
                        return Some(format!(
                            "**Field** `{}` of type `{}`\n\n**Type:** `{}`",
                            member_name, type_name, ty_str
                        ));
                    }
                }
            }
        }
        return None;
    }

    // Simple name lookups
    if let Some(fn_decl) = store.lookup_fn_decl_str(name) {
        let sig = format_function_signature(fn_decl);
        return Some(format!("**Function**\n\n```auto\n{}\n```", sig));
    }

    if let Some(type_decl) = store.lookup_type_decl_str(name) {
        let mut docs = format!("**Type** `{}`\n\n", type_decl.name);
        if !type_decl.members.is_empty() {
            docs.push_str("**Members:**\n\n");
            for member in &type_decl.members {
                docs.push_str(&format!("- `{}`: `{}`\n", member.name, member.ty));
            }
        }
        if !type_decl.methods.is_empty() {
            docs.push_str("\n**Methods:**\n\n");
            for method in &type_decl.methods {
                docs.push_str(&format!("- `{}`\n", format_function_signature(method)));
            }
        }
        return Some(docs);
    }

    if let Some(spec_decl) = store.lookup_spec_decl_str(name) {
        let mut docs = format!("**Spec** `{}`\n\n", spec_decl.name);
        if !spec_decl.methods.is_empty() {
            docs.push_str("**Methods:**\n\n");
            for method in &spec_decl.methods {
                let params: Vec<String> = method.params.iter().map(|p| format!("{} {}", p.name, p.ty)).collect();
                docs.push_str(&format!("- `fn {}({}) {}`\n", method.name, params.join(", "), method.ret));
            }
        }
        return Some(docs);
    }

    None
}

/// Get documentation from AST traversal (for local variables and parameters)
fn get_ast_docs(ast: &auto_lang::ast::Code, name: &str) -> Option<String> {
    use auto_lang::ast::Stmt;
    for stmt in ast.stmts.iter() {
        // Check top-level variables
        if let Stmt::Store(store) = stmt {
            if store.name.as_str() == name {
                let ty_str = store.ty.unique_name().to_string();
                let kind_str = match store.kind {
                    auto_lang::ast::StoreKind::Let => "let",
                    auto_lang::ast::StoreKind::Var => "var",
                    _ => "variable",
                };
                return Some(format!("**{}** `{}`\n\n**Type:** `{}`", kind_str, name, ty_str));
            }
        }

        // Check inside function bodies
        if let Stmt::Fn(fn_decl) = stmt {
            for body_stmt in fn_decl.body.stmts.iter() {
                if let Stmt::Store(store) = body_stmt {
                    if store.name.as_str() == name {
                        let ty_str = store.ty.unique_name().to_string();
                        let kind_str = match store.kind {
                            auto_lang::ast::StoreKind::Let => "let",
                            auto_lang::ast::StoreKind::Var => "var",
                            _ => "variable",
                        };
                        return Some(format!(
                            "**{}** `{}`\n\n**Type:** `{}`\n\n**In function `{}`**",
                            kind_str, name, ty_str, fn_decl.name
                        ));
                    }
                }
            }

            // Check function parameters
            for param in &fn_decl.params {
                if param.name.as_str() == name {
                    let ty_str = param.ty.unique_name().to_string();
                    return Some(format!(
                        "**Parameter** `{}`\n\n**Type:** `{}`\n\n**Of function:** `{}`",
                        name, ty_str, fn_decl.name
                    ));
                }
            }
        }
    }

    None
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

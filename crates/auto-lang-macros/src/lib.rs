use proc_macro::{TokenStream, TokenTree, Delimiter};
use quote::quote;

/// Value 宏 - 将 AutoLang 语法转换为 Value 结构体
///
/// # 语法
///
/// ```rust
/// use auto_lang::value;
/// use auto_val::Value;
///
/// // 节点
/// let val = value!{
///     config {
///         version: "1.0",
///         debug: true,
///     }
/// };
///
/// // 数组
/// let val = value![1, 2, 3, 4, 5];
///
/// // 对象
/// let val = value!{name: "Alice", age: 30};
/// ```
#[proc_macro]
pub fn value(input: TokenStream) -> TokenStream {
    // 将 TokenStream 转换为字符串
    let code = token_stream_to_string(input);

    // 生成解析代码
    let expanded = quote! {{
        use auto_lang::atom::AtomReader;
        use auto_lang::atom::Atom;
        use auto_val::Kid;

        let mut reader = AtomReader::new();
        let code = #code;

        let atom = reader.parse(code)
            .unwrap_or_else(|e| panic!("value! macro failed: {}", e));

        // AtomReader 在 CONFIG 模式下会包装在 root 节点中
        // 提取实际内容
        let atom = match atom {
            Atom::Node(root_node) => {
                let kids: Vec<_> = root_node.kids_iter().collect();
                let has_props = root_node.props_iter().next().is_some();

                if kids.len() == 1 && !has_props {
                    match &kids[0] {
                        (_, Kid::Node(first_kid)) => Atom::Node(first_kid.clone()),
                        _ => Atom::Node(root_node),
                    }
                } else if kids.is_empty() && has_props && root_node.name == "root" {
                    Atom::Obj(root_node.props_clone())
                } else {
                    Atom::Node(root_node)
                }
            }
            other => other,
        };

        // 转换为 Value
        atom.to_value()
    }};

    expanded.into()
}

/// Atom 宏 - 将 AutoLang 语法转换为 Atom 结构体
///
/// # 语法
///
/// ```rust
/// use auto_lang::atom;
///
/// // 基本语法
/// let atom = atom!{
///     config {
///         version: "1.0",
///         debug: true,
///     }
/// };
///
/// // 数组
/// let atom = atom![1, 2, 3, 4, 5];
///
/// // 对象
/// let atom = atom!{name: "Alice", age: 30};
/// ```
#[proc_macro]
pub fn atom(input: TokenStream) -> TokenStream {
    // 将 TokenStream 转换为字符串
    let code = token_stream_to_string(input);

    // 生成解析代码
    let expanded = quote! {{
        use auto_lang::atom::AtomReader;
        use auto_lang::atom::Atom;
        use auto_val::Kid;

        let mut reader = AtomReader::new();
        let code = #code;

        let atom = reader.parse(code)
            .unwrap_or_else(|e| panic!("atom! macro failed: {}", e));

        // AtomReader 在 CONFIG 模式下会包装在 root 节点中
        // 提取实际内容
        match atom {
            Atom::Node(root_node) => {
                // 检查是否有子节点
                let kids: Vec<_> = root_node.kids_iter().collect();
                let has_props = root_node.props_iter().next().is_some();

                if kids.len() == 1 && !has_props {
                    // 只有一个子节点且无属性 -> 返回该子节点
                    match &kids[0] {
                        (_, Kid::Node(first_kid)) => Atom::Node(first_kid.clone()),
                        _ => Atom::Node(root_node),
                    }
                } else if kids.is_empty() && has_props && root_node.name == "root" {
                    // 没有子节点但有属性，且是 root 节点 -> 提取为对象
                    Atom::Obj(root_node.props_clone())
                } else {
                    // 其他情况 -> 返回原始 atom
                    Atom::Node(root_node)
                }
            }
            // 对于数组和对象，直接返回
            other => other,
        }
    }};

    expanded.into()
}

/// Node 宏 - 将 AutoLang 语法转换为 Node 结构体
///
/// # 语法
///
/// ```rust
/// use auto_lang::node;
///
/// let node = node!{
///     config {
///         version: "1.0",
///         debug: true,
///     }
/// };
/// ```
#[proc_macro]
pub fn node(input: TokenStream) -> TokenStream {
    let code = token_stream_to_string(input);

    let expanded = quote! {{
        use auto_lang::atom::AtomReader;
        use auto_val::Kid;

        let mut reader = AtomReader::new();
        let code = #code;
        let atom = reader.parse(code)
            .unwrap_or_else(|e| panic!("node! macro failed: {}", e));

        // AtomReader 在 CONFIG 模式下会包装在 root 节点中
        // 提取第一个子节点（如果有的话）
        match atom {
            auto_lang::atom::Atom::Node(root_node) => {
                // 尝试获取第一个子节点
                let first_kid = root_node.kids_iter().next();
                if let Some((_, Kid::Node(kid_node))) = first_kid {
                    kid_node.clone()
                } else {
                    // 如果没有子节点，返回 root 节点本身
                    root_node
                }
            }
            _ => panic!("node! macro expected a node"),
        }
    }};

    expanded.into()
}

/// 将 TokenStream 转换为字符串表示
///
/// 这个函数会：
/// 1. 手动遍历 TokenStream 构建字符串
/// 2. 将逗号转换为分号（用于属性分隔，仅在花括号内）
/// 3. 智能处理外层括号
fn token_stream_to_string(tokens: TokenStream) -> proc_macro2::TokenStream {
    let mut result = String::new();
    let mut iter = tokens.into_iter();

    // 检查第一个 token 来确定外层结构
    if let Some(token) = iter.next() {
        match token {
            TokenTree::Group(group) => {
                match group.delimiter() {
                    Delimiter::Brace => {
                        // 花括号 - 检查内容类型
                        let inner = group.stream().into_iter().collect::<Vec<_>>();
                        if let Some(TokenTree::Ident(_)) = inner.first() {
                            // 节点形式 `name { ... }` - 去掉外层花括号
                            result = tokens_to_string_helper(inner, 0);
                        } else {
                            // 对象或嵌套结构 - 保留外层花括号
                            result.push('{');
                            result.push(' ');
                            result.push_str(&tokens_to_string_helper(inner, 0));
                            result.push(' ');
                            result.push('}');
                        }
                    }
                    Delimiter::Bracket => {
                        // 数组 - 保留方括号，不转换逗号
                        result.push('[');
                        for (i, token) in group.stream().into_iter().enumerate() {
                            if i > 0 {
                                result.push_str(", ");
                            }
                            result.push_str(&token_to_string(token));
                        }
                        result.push(']');
                        // DEBUG: 立即返回，避免后续修改
                        let string_literal = syn::LitStr::new(&result, proc_macro2::Span::call_site());
                        return quote! { #string_literal };
                    }
                    _ => {
                        // 其他分隔符 - 递归处理
                        result.push_str(&tokens_to_string_helper(group.stream().into_iter().collect::<Vec<_>>(), 0));
                    }
                }
            }
            token => {
                // 第一个 token 不是组 - 检查是否是数组（逗号分隔的多个元素）
                let mut all_tokens = vec![token];
                for t in iter {
                    all_tokens.push(t);
                }

                // 检查是否包含逗号（数组特征）
                let has_comma = all_tokens.iter().any(|t| matches!(t, TokenTree::Punct(p) if p.as_char() == ','));

                if has_comma && all_tokens.len() > 1 {
                    // 需要判断是数组还是对象
                    // 数组元素通常是字面量，对象元素通常是标识符（key）+ : + value
                    let is_object = all_tokens.iter().any(|t| matches!(t, TokenTree::Ident(_)));

                    if is_object {
                        // 对象 - 使用花括号和分号
                        result.push('{');
                        result.push(' ');
                        for tok in &all_tokens {
                            if matches!(tok, TokenTree::Punct(p) if p.as_char() == ',') {
                                // 将逗号转换为分号
                                result.push_str("; ");
                                continue;
                            }
                            result.push_str(&token_to_string(tok.clone()));
                        }
                        result.push(' ');
                        result.push('}');
                    } else {
                        // 数组 - 添加方括号，处理时跳过逗号 tokens
                        result.push('[');
                        let mut first = true;
                        for tok in &all_tokens {
                            // 跳过逗号 tokens
                            if matches!(tok, TokenTree::Punct(p) if p.as_char() == ',') {
                                continue;
                            }
                            if !first {
                                result.push_str(", ");
                            }
                            first = false;
                            result.push_str(&token_to_string(tok.clone()));
                        }
                        result.push(']');
                    }
                } else {
                    // 不是数组 - 使用 helper 函数处理以正确添加空格
                    result = tokens_to_string_helper(all_tokens, 0);
                }
            }
        }
    }

    // 创建字符串字面量 token stream
    let string_literal = syn::LitStr::new(&result, proc_macro2::Span::call_site());
    quote! { #string_literal }
}

/// 辅助函数：将 token 转换为字符串
fn token_to_string(token: TokenTree) -> String {
    match token {
        TokenTree::Ident(ident) => ident.to_string(),
        TokenTree::Punct(punct) => punct.as_char().to_string(),
        TokenTree::Literal(lit) => lit.to_string(),
        TokenTree::Group(group) => {
            let (open, close) = match group.delimiter() {
                Delimiter::Parenthesis => ("(", ")"),
                Delimiter::Brace => ("{ ", " }"),
                Delimiter::Bracket => ("[", "]"),
                Delimiter::None => ("", ""),
            };
            let inner = tokens_to_string_helper(group.stream().into_iter().collect::<Vec<_>>(), 0);
            format!("{}{}{}", open, inner, close)
        }
    }
}

/// 辅助函数：将 token 列表转换为字符串（处理逗号转换和空格）
fn tokens_to_string_helper(tokens: Vec<TokenTree>, brace_depth: usize) -> String {
    let mut result = String::new();
    let mut in_brace = brace_depth > 0;

    // AutoLang 关键字列表
    let keywords = ["let", "mut", "fn", "return", "if", "else", "for", "while", "loop", "break", "use", "type"];

    for (i, token) in tokens.iter().enumerate() {
        // 检测语句边界并插入分号
        // 语句边界：字面量/Group 后跟关键字或新语句的标识符
        if i > 0 && !result.is_empty() {
            let prev_token = &tokens[i - 1];
            let needs_semicolon = match (prev_token, token) {
                // 字面量后面跟关键字（新语句开始）
                (TokenTree::Literal(_), TokenTree::Ident(ident)) => {
                    let ident_str = ident.to_string();
                    keywords.contains(&ident_str.as_str()) || ident_str == "let"
                }
                // Group 后跟关键字（新语句开始）
                (TokenTree::Group(_), TokenTree::Ident(ident)) => {
                    let ident_str = ident.to_string();
                    keywords.contains(&ident_str.as_str()) || ident_str == "let"
                }
                // Group 后跟字面量（可能是新语句）
                (TokenTree::Group(_), TokenTree::Literal(_)) => true,
                _ => false,
            };

            if needs_semicolon {
                result.push_str("; ");
            }
        }

        // 在 token 之间添加适当的空格
        if i > 0 && !result.is_empty() {
            let prev_token = &tokens[i - 1];
            let last_char = result.chars().last().unwrap_or(' ');

            // 如果最后一个字符已经是空格或开括号，不需要再添加
            if !last_char.is_whitespace() && last_char != '{' && last_char != '[' && last_char != '(' {
                match (prev_token, token) {
                    // 标识符后面跟标点（如 `name:`）不需要空格
                    (TokenTree::Ident(_), TokenTree::Punct(_)) => {}
                    // 字面量后面跟标点不需要空格
                    (TokenTree::Literal(_), TokenTree::Punct(_)) => {}
                    // 其他情况添加空格
                    _ => {
                        result.push(' ');
                    }
                }
            }
        }

        match token {
            TokenTree::Ident(ident) => {
                let ident_str = ident.to_string();
                result.push_str(&ident_str);

                // 关键字后面必须有空格
                if keywords.contains(&ident_str.as_str()) {
                    result.push(' ');
                }
            }
            TokenTree::Punct(punct) => {
                let ch = punct.as_char();
                if ch == ',' {
                    // 在花括号内，将逗号转换为分号+空格
                    if in_brace {
                        result.push_str("; ");
                    } else {
                        result.push_str(", ");
                    }
                } else if ch == '=' || ch == ':' || ch == ';' {
                    // 这些符号后面需要空格
                    result.push(ch);
                    result.push(' ');
                } else {
                    result.push(ch);
                }
            }
            TokenTree::Literal(lit) => {
                result.push_str(&lit.to_string());
            }
            TokenTree::Group(group) => {
                let (open_str, close_str) = match group.delimiter() {
                    Delimiter::Parenthesis => ("(", ")"),
                    Delimiter::Brace => ("{ ", " }"),
                    Delimiter::Bracket => ("[", "]"),
                    Delimiter::None => ("", ""),
                };
                if group.delimiter() == Delimiter::Brace {
                    in_brace = true;
                }
                let inner = tokens_to_string_helper(group.stream().into_iter().collect::<Vec<_>>(), brace_depth + (if group.delimiter() == Delimiter::Brace { 1 } else { 0 }));
                result.push_str(open_str);
                result.push_str(&inner);
                result.push_str(close_str);
            }
        }
    }

    result
}

/// 仅在花括号内转换逗号为分号（已弃用，保留用于兼容性）
#[allow(dead_code)]
fn convert_commas_in_braces(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.chars().peekable();
    let mut brace_depth = 0;

    while let Some(ch) = chars.next() {
        if ch == '{' {
            brace_depth += 1;
            result.push(ch);
        } else if ch == '}' {
            brace_depth -= 1;
            result.push(ch);
        } else if ch == ',' && brace_depth > 0 {
            // 在花括号内，将逗号转换为分号+空格
            result.push_str("; ");
        } else {
            result.push(ch);
        }
    }

    result
}

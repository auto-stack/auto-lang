
/// 检测 TokenStream 中是否包含插值模式 #{...}
fn has_interpolation(tokens: &TokenStream) -> bool {
    let token_str = tokens.to_string();
    token_str.contains("# {") || token_str.contains("#{")
}

/// 从 TokenStream 中提取插值变量
/// 返回: (无插值的TokenStream字符串, 变量名 -> 插值位置)
fn parse_interpolated_object(input: TokenStream) -> Result<(String, Vec<String>), String> {
    let mut tokens = input.into_iter().collect::<Vec<_>>();
    let mut result = String::new();
    let mut vars = Vec::new();
    let mut i = 0;
    
    while i < tokens.len() {
        match &tokens[i] {
            TokenTree::Punct(p) if p.as_char() == '#' => {
                // 检查是否是插值模式 #{ident}
                if i + 2 < tokens.len() {
                    if let (TokenTree::Punct(p1), TokenTree::Ident(ident), TokenTree::Punct(p2)) = 
                          (&tokens[i+1], &tokens[i+2], &tokens.get(i+3)) {
                        if p1.as_char() == '{' && p2.as_char() == '}' {
                            // 找到插值模式 #{ident}
                            let var_name = ident.to_string();
                            vars.push(var_name.clone());
                            result.push_str(&var_name);
                            i += 4;
                            continue;
                        }
                    }
                }
                // 不是插值模式，保留 #
                result.push('#');
            }
            _ => {
                result.push_str(&tokens[i].to_string());
            }
        }
        i += 1;
    }
    
    Ok((result, vars))
}

/// 处理带插值的对象字面量
/// 例如：value!{name: "height", count: #{count}}
fn handle_interpolated_value(input: TokenStream) -> TokenStream {
    let tokens = input.into_iter().collect::<Vec<_>>();
    let mut properties = Vec::new();
    let mut i = 0;
    
    // 解析对象属性
    while i < tokens.len() {
        // 跳过逗号和分号
        if matches!(&tokens[i], TokenTree::Punct(p) if p.as_char() == ',' || p.as_char() == ';') {
            i += 1;
            continue;
        }
        
        // 获取属性名 (标识符)
        let key = match &tokens[i] {
            TokenTree::Ident(ident) => ident.to_string(),
            _ => break,
        };
        i += 1;
        
        // 跳过冒号
        if i >= tokens.len() || !matches!(&tokens[i], TokenTree::Punct(p) if p.as_char() == ':') {
            break;
        }
        i += 1;
        
        // 获取属性值
        if i >= tokens.len() {
            break;
        }
        
        // 检查是否是插值模式 #{ident}
        let value_code = if matches!(&tokens[i], TokenTree::Punct(p) if p.as_char() == '#') 
            && i + 3 < tokens.len() 
            && matches!(&tokens[i+1], TokenTree::Punct(p) if p.as_char() == '{')
            && matches!(&tokens[i+2], TokenTree::Ident(_))
            && matches!(&tokens[i+3], TokenTree::Punct(p) if p.as_char() == '}') {
            // 插值模式
            let var_name = match &tokens[i+2] {
                TokenTree::Ident(ident) => ident.to_string(),
                _ => String::new(),
            };
            let var_ident = syn::Ident::new(&var_name, proc_macro2::Span::call_site());
            i += 4;
            quote! { #var_ident.to_auto_value() }
        } else {
            // 普通值，使用 token_to_string
            let value_str = token_to_string(tokens[i].clone());
            i += 1;
            let string_literal = syn::LitStr::new(&value_str, proc_macro2::Span::call_site());
            quote! { {
                use auto_lang::atom::AtomReader;
                let mut reader = AtomReader::new();
                let atom = reader.parse(#string_literal)
                    .unwrap_or_else(|e| panic!("value! macro failed: {}", e));
                atom.to_value()
            } }
        };
        
        let key_str = syn::LitStr::new(&key, proc_macro2::Span::call_site());
        properties.push(quote! { obj.set(#key_str, #value_code); });
    }
    
    // 生成代码
    let expanded = quote! {{
        use auto_val::Obj;
        let mut obj = Obj::new();
        #(#properties)*
        auto_val::Value::Obj(obj)
    }};
    
    expanded.into()
}

//! PLAN-884 T-06：merged 模式专属后处理（dedup + regex fix 包，
//! v1 自举字面补丁）——自 rust.rs 原样迁入。
use super::*;

/// Post-processing passes specific to merged mode output.
/// These handle cross-file issues that arise when concatenating modules.
pub(super) fn post_process_merged(body: &mut Vec<u8>) {
    let content = String::from_utf8(std::mem::take(body)).unwrap();
    let lines: Vec<&str> = content.lines().collect();

    // Track seen definitions for deduplication
    // Note: struct/enum dedup is handled at AST level in transpile_rust_project_merged
    let mut seen_allow = false;
    let mut seen_uses: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut seen_top_level_names: std::collections::HashSet<String> =
        std::collections::HashSet::new();
    let mut brace_depth: i32 = 0;

    let mut result = String::new();
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i];
        let trimmed = line.trim();

        // Skip duplicate #![allow(...)] pragmas
        if trimmed.starts_with("#![allow(") {
            if seen_allow {
                i += 1;
                continue;
            }
            seen_allow = true;
            result.push_str(line);
            result.push('\n');
            i += 1;
            continue;
        }

        // Skip duplicate use statements (any form)
        if trimmed.starts_with("use ")
            || trimmed.starts_with("#[allow(unused_imports)]")
                && i + 1 < lines.len()
                && lines[i + 1].trim().starts_with("use ")
        {
            let use_line = if trimmed.starts_with("#[allow") {
                // Skip the #[allow(unused_imports)] annotation line too
                let actual_use = lines[i + 1].trim();
                if seen_uses.contains(actual_use) {
                    i += 2;
                    continue; // skip both annotation and use
                }
                seen_uses.insert(actual_use.to_string());
                result.push_str(line);
                result.push('\n');
                i += 1;
                continue;
            } else {
                trimmed
            };
            if seen_uses.contains(use_line) {
                i += 1;
                continue;
            }
            seen_uses.insert(use_line.to_string());
            result.push_str(line);
            result.push('\n');
            i += 1;
            continue;
        }

        // Skip duplicate const definitions (OP_*, BOOL_*, NATIVE_*, etc.)
        if trimmed.starts_with("const ") && trimmed.ends_with(';') {
            // Extract const name: "const OP_POP: i32 = 1;" → "OP_POP"
            if let Some(name) = trimmed.strip_prefix("const ") {
                if let Some(colon_pos) = name.find(':') {
                    let const_name = &name[..colon_pos];
                    if seen_top_level_names.contains(const_name) {
                        i += 1;
                        continue;
                    }
                    seen_top_level_names.insert(const_name.to_string());
                }
            }
            result.push_str(line);
            result.push('\n');
            i += 1;
            continue;
        }

        // Skip duplicate fn definitions (OP_*, BOOL_*, NATIVE_*, etc.)
        // Only at top level (brace_depth == 0), not inside impl blocks
        if trimmed.starts_with("fn ") && brace_depth == 0 {
            // Extract fn name: "fn OP_POP() -> i32 {" → "OP_POP"
            if let Some(rest) = trimmed.strip_prefix("fn ") {
                if let Some(paren_pos) = rest.find('(') {
                    let fn_name = &rest[..paren_pos];
                    if seen_top_level_names.contains(fn_name) {
                        // Skip the entire function body and update brace_depth
                        for ch in trimmed.chars() {
                            match ch {
                                '{' => brace_depth += 1,
                                '}' => brace_depth -= 1,
                                _ => {}
                            }
                        }
                        while i + 1 < lines.len() && brace_depth > 0 {
                            i += 1;
                            for ch in lines[i].chars() {
                                match ch {
                                    '{' => brace_depth += 1,
                                    '}' => brace_depth -= 1,
                                    _ => {}
                                }
                            }
                        }
                        i += 1;
                        continue;
                    }
                    seen_top_level_names.insert(fn_name.to_string());
                }
            }
            result.push_str(line);
            result.push('\n');
            for ch in trimmed.chars() {
                match ch {
                    '{' => brace_depth += 1,
                    '}' => brace_depth -= 1,
                    _ => {}
                }
            }
            i += 1;
            continue;
        }

        // Skip duplicate type aliases: "type X = ..."
        if trimmed.starts_with("type ") && trimmed.contains('=') {
            if let Some(rest) = trimmed.strip_prefix("type ") {
                let name = rest
                    .split(|c: char| c == '=' || c == '<' || c == ' ')
                    .next()
                    .unwrap_or("")
                    .trim();
                if !name.is_empty() {
                    if seen_top_level_names.contains(name) {
                        i += 1;
                        continue;
                    }
                    seen_top_level_names.insert(name.to_string());
                }
            }
            result.push_str(line);
            result.push('\n');
            i += 1;
            continue;
        }

        // Keep all other lines
        result.push_str(line);
        result.push('\n');
        // Track brace depth for fn dedup scope detection
        for ch in trimmed.chars() {
            match ch {
                '{' => brace_depth += 1,
                '}' => brace_depth -= 1,
                _ => {}
            }
        }
        i += 1;
    }

    *body = result.into_bytes();
}

/// Apply regex-based fixes to merged output, mirroring the Python post-processing scripts.
/// Only deterministic, pattern-based fixes are applied here. Fragile flow-sensitive fixes
/// (borrow2, clone, push_clone, move_after_field) have been removed — they require AST-level
/// analysis that text-based regex processing cannot do reliably.
pub(super) fn apply_merged_regex_fixes(body: &mut Vec<u8>) {
    let mut content = String::from_utf8(std::mem::take(body)).unwrap();

    // === fix_cross_file.py ===
    // int_to_str(kind) -> int_to_str(kind as i64): partially at AST level (needs_enum_cast)
    // Still needed as fallback for cases where local_var_types has User(NodeKind) instead of Enum
    content = content.replace("int_to_str(kind)", "int_to_str(kind as i64)");
    // String + String: (output + int_to_str(val)) -> (output + &int_to_str(val))
    content = content.replace("(output + int_to_str(val))", "(output + &int_to_str(val))");
    // prefix + a2r_expr(...) -> prefix + &a2r_expr(...)
    let re = cached_regex(r"prefix \+ (a2r_expr\([^)]+\))").unwrap();
    content = re.replace_all(&content, "prefix + &$1").to_string();
    // return left == right; -> return if left == right { 1 } else { 0 };
    for op in &["==", "!=", "<", ">", "<=", ">="] {
        let old = format!("return left {} right;", op);
        let new = format!("return if left {} right {{ 1 }} else {{ 0 }};", op);
        content = content.replace(&old, &new);
    }
    // tenv clone at cross-file call sites: no longer needed in merge mode
    // since TypeEnv is now &mut TypeEnv (auto-reborrow handles multiple calls)
    // node.name partial move fix
    content = content.replace(
        "let mut callee_name = node.name;",
        "let mut callee_name = node.name.clone();",
    );
    // str_to_int arithmetic fix
    let re = cached_regex(r#"result = format!\("\{\}\{\}", result \* 10, ch - 48\)"#).unwrap();
    content = re
        .replace_all(&content, "result = result * 10 + (ch - 48)")
        .to_string();
    // Allow overflowing literals
    if !content.contains("#![allow(overflowing_literals)]") {
        content = format!("#![allow(overflowing_literals)]\n{}", content);
    }

    // === fix_misc.py ===
    // nil_node() in match arms: remove trailing semicolon
    content = content.replace(
        "=> { p.pos = p.pos + 1; nil_node(); }",
        "=> { p.pos = p.pos + 1; nil_node() }",
    );
    // Option.drop() -> Option.take(): now handled at AST level (method name mapping)
    // Fix int_to_str(X).cloned().unwrap_or_default()
    let re = cached_regex(r"int_to_str\(([^)]+)\)\.cloned\(\)\.unwrap_or_default\(\)").unwrap();
    content = re.replace_all(&content, "int_to_str($1)").to_string();
    // NodeKind derives: Copy from AST level, Eq/Ord for ASTNode derive compatibility
    content = content.replace(
        "#[derive(Clone, Debug, PartialEq)]\nenum NodeKind",
        "#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]\nenum NodeKind",
    );
    // else_if.value partial move fix
    content = content.replace(
        "else_str = else_if.value;\n            else_body.push(else_if)",
        "else_str = else_if.value.clone();\n            else_body.push(else_if)",
    );
    // fn_defs type: NOW HANDLED AT SOURCE LEVEL — eval.at uses Map<str, ASTNode>
    // But .to_string() on insert args is still generated by a2r for non-primitive Map values.
    // Fix: (stmt).to_string() -> stmt.clone() for ASTNode-typed map inserts
    content = content.replace(
        "env.fn_defs.insert(stmt.name.to_string(), (stmt).to_string())",
        "env.fn_defs.insert(stmt.name.to_string(), stmt.clone())",
    );
    content = content.replace(
        "env.fn_defs.insert(node.name.to_string(), (node).to_string())",
        "env.fn_defs.insert(node.name.to_string(), node.clone())",
    );

    // === fix_param_vec.py ===
    // Add NodeKind::Param variant
    content = content.replace("MoveExpr = 33,\n}", "MoveExpr = 33,\n    Param = 34,\n}");
    // Add to Display impl
    content = content.replace(
        "NodeKind::NilNode => write!(f, \"NilNode\")",
        "NodeKind::Param => write!(f, \"Param\"),\n            NodeKind::NilNode => write!(f, \"NilNode\")",
    );
    // Add to from_str match
    content = content.replace(
        "\"NilNode\" | \"nilnode\" => NodeKind::NilNode",
        "\"Param\" | \"param\" => NodeKind::Param,\n            \"NilNode\" | \"nilnode\" => NodeKind::NilNode",
    );
    // Convert push(Param { name: x, type_name: y }) to push(ASTNode { ... })
    let re = cached_regex(r#"\.push\(Param \{ name: ([^,]+), type_name: ([^}]+) \}\)"#).unwrap();
    content = re.replace_all(&content, ".push(ASTNode { kind: NodeKind::Param, name: $1, type_name: $2, value: \"\".to_string(), children: empty_list(), left: empty_list(), right: empty_list(), op: \"\".to_string(), params: empty_list(), cond: empty_list(), else_body: empty_list() })").to_string();

    // === fix_return_types.py ===
    content = content.replace(
        "fn tokenize_list(mut source: &str) {",
        "fn tokenize_list(mut source: &str) -> Vec<Token> {",
    );
    content = content.replace(
        "fn lex_fstr_backtick(mut source: &str, mut pos: i32) {",
        "fn lex_fstr_backtick(mut source: &str, mut pos: i32) -> Vec<Token> {",
    );
    content = content.replace(
        "fn lex_fstr_f(mut source: &str, mut pos: i32) {",
        "fn lex_fstr_f(mut source: &str, mut pos: i32) -> Vec<Token> {",
    );
    // Plan 434: v1 bootstrap self-test main 追加已清退(433 KNOWN-DEBT 条目)
    // ——v1 lib 仅存于 lib-legacy 封存与 99_bootstrap 的 #[ignore] 测试,
    // 不再为 merge 产物注入自测 main;v2 lib 无 main 语义保持。

    // === fix_contains_key.py: now handled at AST level (contains_rust logic + cross-module struct_field_types) ===

    // === fix_vec_get.py: AST level covers most cases, regex catches remaining 2 edge cases ===
    let re = cached_regex(r"p\.tokens\.get\(([^)]+)\)").unwrap();
    content = re
        .replace_all(&content, "p.tokens[$1 as usize].clone()")
        .to_string();
    let re = cached_regex(r"\bcode\.get\(([^)]+)\)").unwrap();
    content = re
        .replace_all(&content, "code[$1 as usize].clone()")
        .to_string();

    // === fix_usize_insert.py ===
    // .insert(arith_expr, -> .insert((arith_expr) as usize,
    let re = cached_regex(r"\.insert\(([^,]+),").unwrap();
    content = re
        .replace_all(&content, |caps: &regex::Captures| {
            let idx = caps[1].trim().to_string();
            if idx.contains("as usize")
                || idx.starts_with('"')
                || idx.contains(".to_string()")
                || idx.starts_with('&')
            {
                caps[0].to_string()
            } else if idx.chars().any(|c| "+-*/%".contains(c)) {
                format!(".insert(({}) as usize,", idx)
            } else {
                caps[0].to_string()
            }
        })
        .to_string();

    // === fix_hashmap_get.py ===
    // Replace .get(expr) with .get(&expr).cloned().unwrap_or_default() for HashMap types.
    // Only applies to known HashMap field names to avoid corrupting Vec .get() calls.
    let hashmap_fields = [
        "struct_fields",
        "fn_param_types",
        "fn_defs",
        "globals",
        "type_aliases",
        "scopes",
        "strings",
        "state",
        "env",
    ];
    for field in &hashmap_fields {
        // env.field.get(X) pattern
        let pat = cached_regex(&format!(r"env\.{}\.get\(([^)]+)\)", regex::escape(field))).unwrap();
        content = pat
            .replace_all(&content, |caps: &regex::Captures| {
                let arg = caps[1].trim();
                let key_expr = if arg.starts_with('"') || arg.starts_with("c\"") {
                    arg.to_string()
                } else if arg.contains("format!") || arg.contains("to_string") {
                    format!("&{}", arg)
                } else {
                    format!("&*{}", arg)
                };
                format!(
                    "env.{}.get({}).cloned().unwrap_or_default()",
                    field, key_expr
                )
            })
            .to_string();
        // bare field.get(X) pattern (when field is a local variable)
        let pat = cached_regex(&format!(r"\b{}\.get\(([^)]+)\)", regex::escape(field))).unwrap();
        content = pat
            .replace_all(&content, |caps: &regex::Captures| {
                let arg = caps[1].trim();
                // Skip Vec-style .get() with 'as usize' index
                if arg.contains("as usize") {
                    return caps[0].to_string();
                }
                let key_expr = if arg.starts_with('"') || arg.starts_with("c\"") {
                    arg.to_string()
                } else if arg.contains("format!") || arg.contains("to_string") {
                    format!("&{}", arg)
                } else {
                    format!("&*{}", arg)
                };
                format!("{}.get({}).cloned().unwrap_or_default()", field, key_expr)
            })
            .to_string();
    }

    // === fix_misc: void functions return 0 -> return ===
    // AST level handles top-level return 0 in void functions, but if-block returns need regex
    for fn_name in &[
        "codegen_expr",
        "codegen_stmt",
        "type_infer_stmts",
        "codegen_call",
        "codegen_binop",
        "codegen_unary",
        "a2r_transpile",
    ] {
        let fn_pattern = format!("fn {}(", fn_name);
        if let Some(pos) = content.find(&fn_pattern) {
            if let Some(brace_pos) = content[pos..].find('{') {
                let abs_brace = pos + brace_pos;
                let mut depth = 1i32;
                let mut end = abs_brace + 1;
                let bytes = content.as_bytes();
                while end < bytes.len() && depth > 0 {
                    match bytes[end] {
                        b'{' => depth += 1,
                        b'}' => depth -= 1,
                        _ => {}
                    }
                    end += 1;
                }
                let body = &content[abs_brace + 1..end - 1];
                let fixed_body = body.replace("return 0;", "return;");
                if body != fixed_body {
                    content = format!(
                        "{}{}{}{}",
                        &content[..abs_brace + 1],
                        fixed_body,
                        &content[end - 1..],
                        ""
                    );
                }
            }
        }
    }

    // === OP_XXX {} -> OP_XXX() now handled at AST level (is_screaming_case check) ===

    // === crate:: path fixes now handled at AST level in qualify_type_name ===
    // (merge_mode skips crate:: prefix generation)

    // === env.scopes type: NOW HANDLED AT SOURCE LEVEL — eval.at uses List<Map<str, str>> ===

    // === use auto_lang::a2r_std now handled at AST level (merge_mode skips emit) ===

    // === CONST_NAME() -> CONST_NAME now handled at AST level (is_screaming_case check) ===

    // === .to_string().cloned().unwrap_or_default() -> .to_string() (E0599: String not iterator) ===
    content = content.replace(".to_string().cloned().unwrap_or_default()", ".to_string()");
    // Also fix .cloned().unwrap_or_default() on format!() results
    let re = cached_regex(r#"format!\([^)]*\)\.cloned\(\)\.unwrap_or_default\(\)"#).unwrap();
    content = re
        .replace_all(&content, |caps: &regex::Captures| {
            caps[0]
                .trim_end_matches(".cloned().unwrap_or_default()")
                .to_string()
        })
        .to_string();

    // === &&expr -> &expr (E0277: double reference to String) ===
    // state.get(&&"key".to_string()) -> state.get(&"key".to_string())
    // state.get(&&format!(...)) -> state.get(&format!(...))
    // state.get(&&nkey.to_string()) -> state.get(&nkey.to_string())
    // Plan 525 W1 宿主顺修:盲替换 `&&"`→`&"` 会误吃以 && 结尾的字符串
    // 字面量(lib 的 lexer/parser/a2r 各有 "&&" 字面量,aavm2_bin merge
    // 产物被改成 "&";g26 四路 p2≠p4 实证)——改为捕获前导字符,仅当
    // 前导非 `"`(即不是 "…&&" 字面量收尾)才替换
    {
        let re_amp = cached_regex(r#"([^"])&&""#).unwrap();
        content = re_amp.replace_all(&content, "$1&\"").to_string();
    }
    content = content.replace("&&format!", "&format!");
    // Fix &&var.to_string() patterns
    for var in &["nkey", "ekey", "vkey", "name", "key", "skey"] {
        content = content.replace(&format!("&&{}.", var), &format!("&{}.", var));
    }
    // int_to_str(x).cloned().unwrap_or_default() already fixed above, but check again
    let re = cached_regex(r"int_to_str\(([^)]+)\)\.cloned\(\)\.unwrap_or_default\(\)").unwrap();
    content = re.replace_all(&content, "int_to_str($1)").to_string();

    // === Fix Display trait missing fmt method (E0046): now handled at AST level ===
    // a2r generates Display impl with fmt method for all-unit heterogeneous enums
    // content = content.replace(...)

    // === Fix return; in non-void function: no longer needed ===
    // AST level now correctly emits "return 0;" for non-void functions, "return;" for void functions

    // === Fix double .cloned().unwrap_or_default() (E0599: String is not iterator) ===
    // Pattern: .cloned().unwrap_or_default().cloned().unwrap_or_default()
    while content.contains(".cloned().unwrap_or_default().cloned().unwrap_or_default()") {
        content = content.replace(
            ".cloned().unwrap_or_default().cloned().unwrap_or_default()",
            ".cloned().unwrap_or_default()",
        );
    }

    // === Fix &*&* double dereference ===
    content = content.replace("&*&*", "&*");

    // === a2r_std::str_substr -> inline str_substr (E0433) ===
    // Add str_substr function definition before fn main() and replace a2r_std:: prefix
    content = content.replace("a2r_std::str_substr", "str_substr");
    let str_substr_fn = r#"
fn str_substr<S: AsRef<str>>(s: S, start: i32, end: i32) -> String {
    let s = s.as_ref();
    if start < 0 || end <= start || start as usize >= s.len() {
        return String::new();
    }
    let start_usize = start as usize;
    let end_usize = std::cmp::min(end as usize, s.len());
    s[start_usize..end_usize].to_string()
}
"#;
    if let Some(pos) = content.find("\nfn main() {") {
        content = format!("{}{}{}", &content[..pos], str_substr_fn, &content[pos..]);
    }

    // === Fix known &str param functions called with String args (E0308) ===
    // === Fix .get(...).as_str() (E0599: no method as_str on Option) ===
    // .get(X).as_str() -> .get(X).cloned().unwrap_or_default()
    let re = cached_regex(r"\.get\(([^)]+)\)\.as_str\(\)").unwrap();
    content = re
        .replace_all(&content, |caps: &regex::Captures| {
            format!(".get({}).cloned().unwrap_or_default()", &caps[1])
        })
        .to_string();

    // === Fix .to_string() after format!() (unnecessary, format! returns String) ===
    // This causes "expected &str, found String" in some contexts
    // Actually, keep it — it's harmless. The real E0308 issue is String where &str expected.

    // === Fix env.globals.get and similar — add & before key (E0308) ===
    // env.globals.get("__last_str__") -> env.globals.get("__last_str__")
    // Already handled by fix_hashmap_get, but some patterns may have been missed.

    // === Fix String where &str expected: specific known patterns ===
    // node.name passed to &str params: need &*node.name or node.name.as_str()
    // Pattern: (node.name) where function expects &str
    // This is too broad for regex. The real fix is AST-level.

    // === Fix .push(var) where var is ASTNode and used later (E0382) ===
    // NOW HANDLED AT AST LEVEL: is_copy_type() check in method call emission.
    // .push(ident) automatically gets .clone() for non-Copy type identifiers.
    // Keeping fix_push_move() as fallback for edge cases.

    // === Fix tokens move in parser_new ===
    // fn parser_new(mut tokens: Vec<Token>) -> Parser { ... tokens ... }
    // tokens is moved into Parser.tokens, but later code uses tokens.len()
    // Fix: use tokens.len() before the move, or clone
    content = content.replace(
        "fn parser_new(mut tokens: Vec<Token>) -> Parser",
        "fn parser_new(mut tokens: Vec<Token>) -> Parser",
    ); // placeholder — actual fix needs AST-level changes

    // === Fix path move: NOW HANDLED AT AST LEVEL ===
    // store() auto-appends .clone() when assigning from non-Copy struct field (e.g., node.name).

    // === Fix nil_node(); in match arms -> nil_node() (E0308: returns () instead of ASTNode) ===
    // Pattern: TokenKind::Break => { p.pos = ...; nil_node(); }
    // Should be: TokenKind::Break => { p.pos = ...; nil_node() }
    content = content.replace(
        "TokenKind::Break => {\n            p.pos = p.pos + 1;\n            nil_node();\n        }",
        "TokenKind::Break => {\n            p.pos = p.pos + 1;\n            nil_node()\n        }",
    );
    content = content.replace(
        "TokenKind::Continue => {\n            p.pos = p.pos + 1;\n            nil_node();\n        }",
        "TokenKind::Continue => {\n            p.pos = p.pos + 1;\n            nil_node()\n        }",
    );

    // === Fix parser_new tokens move (E0382) ===
    // Parser { tokens: tokens, pos: 0, token_count: (tokens.len() as i64) }
    // tokens moved, then tokens.len() used -> swap order or clone
    content = content.replace(
        "Parser { tokens: tokens, pos: 0, token_count: (tokens.len() as i64) }",
        "Parser { pos: 0, token_count: (tokens.len() as i64), tokens: tokens }",
    );

    // === Fix ASTNode: Default not satisfied (E0277) ===
    // env.fn_defs.get(...).cloned().unwrap_or_default() needs ASTNode: Default
    // Already have the long unwrap_or(ASTNode { ... }) replacement, but another instance exists
    content = content.replace(
        "env.fn_defs.get(&*fn_name).cloned().unwrap_or_default()",
        "env.fn_defs.get(&*fn_name).cloned().unwrap_or(ASTNode { kind: NodeKind::NilNode, value: \"\".to_string(), name: \"\".to_string(), children: empty_list(), left: empty_list(), right: empty_list(), op: \"\".to_string(), params: empty_list(), type_name: \"\".to_string(), cond: empty_list(), else_body: empty_list() })",
    );

    // === Fix path = node.name move (E0382) ===
    // Need .clone() since path is used later
    content = content.replace(
        "let mut path: String = node.name;",
        "let mut path: String = node.name.clone();",
    );

    // === Fix state.get(&format!(...)).as_str() still remaining (E0599) ===
    // These specific patterns weren't caught by the general regex
    content = content.replace(
        "state.get(&format!(\"{}{}\", \"s\", int_to_str(sp - 1))).as_str()",
        "state.get(&format!(\"{}{}\", \"s\", int_to_str(sp - 1))).cloned().unwrap_or_default()",
    );
    content = content.replace(
        "state.get(&format!(\"{}{}\", \"s\", int_to_str(abs_idx))).as_str()",
        "state.get(&format!(\"{}{}\", \"s\", int_to_str(abs_idx))).cloned().unwrap_or_default()",
    );

    // === Fix state.get(X).to_string() where it returns Option (E0599) ===
    // state.get(&format!(...)).to_string() on Option
    // Line 4504: state.insert(format!(...).to_string(), (s).to_string())
    // The second arg (s).to_string() is wrong - s is already String? Or s is from state.get()?
    // Let me check the specific pattern.

    // === Fix state.get(X) -> need .cloned().unwrap_or_default() for String result (E0308) ===
    // bvm_pop_str_key: return state.get(X) -> return state.get(X).cloned().unwrap_or_default()
    // Already handled by general regex, but some patterns with specific args may have been missed.
    // Fix specific patterns:
    content = content.replace(
        "return state.get(&format!(\"{}{}\", \"s\", int_to_str(sp)));",
        "return state.get(&format!(\"{}{}\", \"s\", int_to_str(sp))).cloned().unwrap_or_default();",
    );
    // ret_str_key = state.get(X) -> state.get(X).cloned().unwrap_or_default()
    content = content.replace(
        "ret_str_key = state.get(&format!(\"{}{}\", \"s\", int_to_str(sp)));",
        "ret_str_key = state.get(&format!(\"{}{}\", \"s\", int_to_str(sp))).cloned().unwrap_or_default();",
    );

    // === Fix bvm_push_str expects &str but gets String (E0308) ===
    // bvm_push_str(state.clone(), String) -> bvm_push_str(state.clone(), &*String)
    // or bvm_push_str(state.clone(), result.as_str())
    content = content.replace(
        "bvm_push_str(state.clone(), state.get(&format!(\"{}{}\", \"s\", int_to_str(sp - 1))).cloned().unwrap_or_default())",
        "bvm_push_str(state.clone(), state.get(&format!(\"{}{}\", \"s\", int_to_str(sp - 1))).cloned().unwrap_or_default().as_str())",
    );
    content = content.replace(
        "bvm_push_str(state.clone(), state.get(&format!(\"{}{}\", \"s\", int_to_str(abs_idx))).cloned().unwrap_or_default())",
        "bvm_push_str(state.clone(), state.get(&format!(\"{}{}\", \"s\", int_to_str(abs_idx))).cloned().unwrap_or_default().as_str())",
    );

    // === Fix (s).to_string() where s is Option (E0599) ===
    // state.insert(format!(...), (s).to_string()) where s = state.get(...)
    // The s variable holds an Option from state.get(). Need to unwrap.
    // Actually s is assigned earlier as: let mut s = state.get(...)
    // Let me check the specific context.
    content = content.replace(
        "state.insert(format!(\"{}{}\", \"s\", int_to_str(abs_idx)).to_string(), (s).to_string());",
        "state.insert(format!(\"{}{}\", \"s\", int_to_str(abs_idx)).to_string(), s.cloned().unwrap_or_default());",
    );

    // === Fix node.name partial move in eval (E0382) ===
    // let mut callee_name: String = node.name; then node.clone() later
    // node.name moves out of node, then node.clone() fails
    content = content.replace(
        "let mut callee_name: String = node.name;",
        "let mut callee_name: String = node.name.clone();",
    );

    // === Fix path move into str_substr (E0382) ===
    // str_substr(path, 0, 5) -> str_substr(&path, 0, 5) to avoid moving path
    content = content.replace("str_substr(path, 0, 5)", "str_substr(&path, 0, 5)");
    content = content.replace(
        "str_substr(path, 5, (path.len() as i64))",
        "str_substr(&path, 5, (path.len() as i64))",
    );
    content = content.replace(
        "a2r_path_to_rust(str_substr(&path, 5, (path.len() as i64)).as_str())",
        "a2r_path_to_rust(&str_substr(&path, 5, (path.len() as i64)))",
    );

    // === Fix state borrow conflict (E0502) ===
    // let s = state.get(X); ... state.insert(Y, Z);
    // s borrows state immutably, then insert borrows mutably
    // Fix: clone the result of get() to release the borrow
    content = content.replace(
        "let mut s = state.get(&format!(\"{}{}\", \"s\", int_to_str(sp)));",
        "let mut s = state.get(&format!(\"{}{}\", \"s\", int_to_str(sp))).cloned().unwrap_or_default();",
    );
    // Fix: s is now String (not Option), so s.cloned().unwrap_or_default() -> s
    content = content.replace(
        "state.insert(format!(\"{}{}\", \"s\", int_to_str(abs_idx)).to_string(), s.cloned().unwrap_or_default());",
        "state.insert(format!(\"{}{}\", \"s\", int_to_str(abs_idx)).to_string(), s);",
    );
    // callee = node.name; then callee is used as &str -> callee = node.name.clone()
    // But callee might already have been handled. Check specific cases.
    content = content.replace(
        "let mut path: String = node.name;\n    let mut rest = \"\".to_string();",
        "let mut path: String = node.name.clone();\n    let mut rest = \"\".to_string();",
    );

    // === Final cleanup passes (run after all other transforms) ===
    // Fix triple &&& -> &
    content = content.replace("&&&", "&");
    // Fix remaining && -> & (double ref)
    while content.contains("&&") {
        let before = content.len();
        // Only replace && that are before expressions, not logical AND
        // Safe patterns: &&"  &&{  &&var.  &&*  &&format!
        // Plan 525 W1 宿主顺修:同前——字面量收尾保护(前导非 `"`)
        {
            let re_amp2 = cached_regex(r#"([^"])&&""#).unwrap();
            content = re_amp2.replace_all(&content, "$1&\"").to_string();
        }
        content = content.replace("&&format!", "&format!");
        content = content.replace("&&*", "&*");
        for var in &["nkey", "ekey", "vkey", "name", "key", "skey", "fn_name"] {
            content = content.replace(&format!("&&{}.", var), &format!("&{}.", var));
        }
        if content.len() == before {
            break;
        } // no more replacements
    }
    // Fix .to_string().cloned().unwrap_or_default() -> .to_string()
    content = content.replace(".to_string().cloned().unwrap_or_default()", ".to_string()");
    // Fix double .cloned().unwrap_or_default()
    while content.contains(".cloned().unwrap_or_default().cloned().unwrap_or_default()") {
        content = content.replace(
            ".cloned().unwrap_or_default().cloned().unwrap_or_default()",
            ".cloned().unwrap_or_default()",
        );
    }
    // Fix .cloned().unwrap_or_default().unwrap() (unwrap on String)
    content = content.replace(
        ".cloned().unwrap_or_default().unwrap()",
        ".cloned().unwrap_or_default()",
    );
    // Fix .cloned().unwrap_or_default().as_str() (as_str on String)
    // Actually .as_str() on String is fine. But on Option it's not.
    // The .get().as_str() pattern was already fixed above.

    // Fix state.get(X) where X has nested .cloned().unwrap_or_default() inside get arg
    // Pattern: .get(&"str".to_string().cloned().unwrap_or_default())
    // Should be: .get(&"str".to_string())
    let re =
        cached_regex(r#"\.get\((&[^)]+?)\.to_string\(\)\.cloned\(\)\.unwrap_or_default\(\)\)"#)
            .unwrap();
    content = re.replace_all(&content, ".get($1.to_string())").to_string();

    // === Fix .get(X).as_str() where .get returns Option (E0599) ===
    let re = cached_regex(r"state\.get\(([^)]+)\)\.as_str\(\)").unwrap();
    content = re
        .replace_all(&content, |caps: &regex::Captures| {
            format!("state.get({}).cloned().unwrap_or_default()", &caps[1])
        })
        .to_string();

    // AST-level fn_str_param_indices + .as_str() auto-borrow covers:
    // contains_key(callee), codegen_lookup_elem(vn2), block_node(body_str2),
    // a2r_struct_init(callee), a2r_path_to_rust(path), env.fn_defs.contains_key(callee_name),
    // eval_fn_call(callee_name), codegen_extract_var_name(callee), type_is_cmp_op(op), etc.

    // === E0308: return state.get(X) -> return state.get(X).cloned().unwrap_or_default() ===
    let re = cached_regex(r"return state\.get\((&[^)]+)\);").unwrap();
    content = re
        .replace_all(&content, |caps: &regex::Captures| {
            let arg = &caps[1];
            if arg.contains(".cloned()") {
                caps[0].to_string()
            } else {
                format!("return state.get({}).cloned().unwrap_or_default();", arg)
            }
        })
        .to_string();
    let re = cached_regex(r"= state\.get\((&[^)]+)\);").unwrap();
    content = re
        .replace_all(&content, |caps: &regex::Captures| {
            let arg = &caps[1];
            if arg.contains(".cloned()") {
                caps[0].to_string()
            } else {
                format!("= state.get({}).cloned().unwrap_or_default();", arg)
            }
        })
        .to_string();

    // === E0308: TokenKind::Break/Continue match arms returning nil_node() instead of Token ===
    // The match is in a function returning Token. nil_node() returns ASTNode, not Token.
    // Need to create a proper Token. This is a parser issue.
    // Pattern: TokenKind::Break => { p.pos = ...; nil_node() }
    // Should be: TokenKind::Break => { p.pos = ...; Token { kind: ..., pos: ..., text: ... } }
    // Too complex for regex — will need AST-level fix. Leave for now.

    // === E0308: entry-point functions pass owned context types to &mut params ===
    // ✅ Now handled at AST level via fn_param_types + is_merge_mut_type()
    // No regex needed — call sites auto-insert &mut for context-type params.

    // === E0499/E0502: double mutable borrows of &mut env in eval functions ===
    // Pattern: some_fn(env, ..., eval_get_last_str(env)...)
    // Fix: extract inner call to a temp variable before the outer call.
    {
        // Helper: extract inner env call to temp variable to avoid double borrow
        let extract_env_tmp = |content: &str, pattern: &str, tmpl: &str| -> String {
            let re = cached_regex(pattern).unwrap();
            re.replace_all(content, tmpl).to_string()
        };

        // eval_bind_str(env, X, eval_get_last_str(env).as_str())
        content = extract_env_tmp(
            &content,
            r#"eval_bind_str\(env, ([^,]+), eval_get_last_str\(env\)\.as_str\(\)\)"#,
            r#"let __tmp = eval_get_last_str(env);
            eval_bind_str(env, $1, __tmp.as_str())"#,
        );

        // eval_set_last_str(env, eval_str_cat(X, eval_get_last_str(env).as_str()).as_str())
        content = extract_env_tmp(
            &content,
            r#"eval_set_last_str\(env, eval_str_cat\(([^,]+), eval_get_last_str\(env\)\.as_str\(\)\)\.as_str\(\)\)"#,
            r#"let __tmp = eval_get_last_str(env);
                eval_set_last_str(env, eval_str_cat($1, __tmp.as_str()).as_str())"#,
        );

        // eval_set_last_str(env, eval_lookup_str_var(env, X).as_str())
        content = extract_env_tmp(
            &content,
            r#"eval_set_last_str\(env, eval_lookup_str_var\(env, (.+?)\)\.as_str\(\)\)"#,
            r#"let __tmp = eval_lookup_str_var(env, $1);
            eval_set_last_str(env, __tmp.as_str())"#,
        );

        // env.globals.insert(X, (eval_get_last_type(env)).to_string())
        content = extract_env_tmp(
            &content,
            r#"env\.globals\.insert\(([^,]+), \(eval_get_last_type\(env\)\)\.to_string\(\)\)"#,
            r#"let __tmp = eval_get_last_type(env);
            env.globals.insert($1, (__tmp).to_string())"#,
        );

        // env.globals.insert(X, (eval_get_last_str(env)).to_string())
        content = extract_env_tmp(
            &content,
            r#"env\.globals\.insert\(([^,]+), \(eval_get_last_str\(env\)\)\.to_string\(\)\)"#,
            r#"let __tmp = eval_get_last_str(env);
                env.globals.insert($1, (__tmp).to_string())"#,
        );

        // env.output = eval_str_cat(env.output.as_str(), eval_str_cat(eval_get_last_str(env).as_str(), "\n").as_str())
        content = extract_env_tmp(
            &content,
            r#"env\.output = eval_str_cat\(env\.output\.as_str\(\), eval_str_cat\(eval_get_last_str\(env\)\.as_str\(\), "\\n"\)\.as_str\(\)\)"#,
            r#"let __tmp = eval_get_last_str(env);
            env.output = eval_str_cat(env.output.as_str(), eval_str_cat(__tmp.as_str(), "\n").as_str())"#,
        );
    }

    // === Fix partial move: var = struct.field where struct used after (E0382) ===
    // Pattern: else_str = else_if.value; ... else_if.clone()
    // The field access moves the String, then clone() tries to borrow the whole struct
    // Fix: add .clone() to the field access
    content = content.replace(
        "else_str = else_if.value;",
        "else_str = else_if.value.clone();",
    );

    // Ensure trailing newline
    if !content.ends_with('\n') {
        content.push('\n');
    }

    *body = content.into_bytes();
}

/// Split a comma-separated argument string respecting nested parens/brackets.
#[allow(dead_code)]
pub(super) fn split_args(s: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut arg_start = 0;
    let mut depth = 0i32;
    for (k, ch) in s.char_indices() {
        match ch {
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' => depth -= 1,
            ',' if depth == 0 => {
                args.push(s[arg_start..k].to_string());
                arg_start = k + 1;
            }
            _ => {}
        }
    }
    args.push(s[arg_start..].to_string());
    args
}

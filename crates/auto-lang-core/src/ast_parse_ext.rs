//! PLAN-799：解析便利层（自 `ast/parsers.rs` 迁入）。
//!
//! `ParserExt` trait 本体在 `crate::parser`（旧导入路径不变）；本模块持有
//! 其对 AST 模型六类类型的 impl：`Code`/`Name`/`Expr`/`Is`/`Arrow`/
//! `OnEvents`（含 OnEvents 旧 `skip_check` 语义）。模型本体在 auto-ast，
//! 解析注入不进底层（auto-ast 零 Parser 依赖）。依赖 Parser 的 markdown
//! 投影测试（test/ast/*.test.md oracle）也随迁至此。

use crate::ast::{Arrow, Code, Expr, Is, Name, OnEvents};
use crate::error::AutoResult;
use crate::parser::{Parser, ParserExt};
use auto_val::AutoStr;

impl ParserExt for Code {
    fn parse(code: impl Into<AutoStr>) -> AutoResult<Self> {
        let code = code.into();
        let mut parser = Parser::from(code.as_str());
        let ast = parser.parse()?;
        Ok(ast)
    }
}

impl ParserExt for Name {
    fn parse(name: impl Into<AutoStr>) -> AutoResult<Self> {
        let n = Name::from(name.into());
        Ok(n)
    }
}

impl ParserExt for Expr {
    fn parse(code: impl Into<AutoStr>) -> AutoResult<Self> {
        let code = code.into();
        let mut parser = Parser::from(code.as_str());
        let ast = parser.parse_expr()?;
        Ok(ast)
    }
}

impl ParserExt for Is {
    fn parse(code: impl Into<AutoStr>) -> AutoResult<Self> {
        let code = code.into();
        let mut parser = Parser::from(code.as_str());
        parser.parse_is()
    }
}

impl ParserExt for Arrow {
    fn parse(code: impl Into<AutoStr>) -> AutoResult<Self> {
        let code = code.into();
        let mut parser = Parser::from(code.as_str());
        let ev = parser.parse_event_src()?;
        let arrow = parser.parse_arrow(ev)?;
        Ok(arrow)
    }
}

impl ParserExt for OnEvents {
    fn parse(code: impl Into<AutoStr>) -> AutoResult<Self> {
        let code = code.into();
        let mut parser = Parser::from(code.as_str()).skip_check();
        parser.parse_on_events()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Code;

    #[test]
    fn test_parse_code() {
        let ast = Code::parse("let x = 1").unwrap();
        assert_eq!(ast.to_string(), "(code (let (name x) (type int) (int 1)))");
    }

    #[test]
    fn test_parse_name() {
        let name = Name::parse("x").unwrap();
        assert_eq!(name.to_string(), "x");
    }

    #[test]
    fn test_parse_expr() {
        let expr = Expr::parse("1 + 2").unwrap();
        assert_eq!(expr.to_string(), "(bina (int 1) (op +) (int 2))");
    }

    #[test]
    fn test_parse_arrow() {
        let arrow = Arrow::parse("5 -> 7 : 8").unwrap();
        assert_eq!(
            arrow.to_string(),
            "(arrow (from (int 5)) (to (int 7)) (with (int 8)))"
        );
    }

    #[test]
    fn test_parse_on_events() {
        let code = r#"
            on {
                5 -> 7 : 10
                9 -> 10 : 11
            }
        "#;
        let on = OnEvents::parse(code.trim()).unwrap();
        assert_eq!(
            on.to_string(),
            "(on (arrow (from (int 5)) (to (int 7)) (with (int 10))) (arrow (from (int 9)) (to (int 10)) (with (int 11))))"
        );
    }
}

#[cfg(test)]
mod markdown_tests {
    use auto_val::AutoStr;
    use std::cell::RefCell;
    use std::fs;
    use std::path::Path;
    use std::rc::Rc;

    use crate::ast::*;
    use crate::parser::Parser;

    #[derive(Debug)]
    struct TestCase {
        name: String,
        input: String,
        expected: String,
    }

    fn parse_markdown_tests(content: &str) -> Vec<TestCase> {
        let mut cases = Vec::new();
        let lines: Vec<&str> = content.lines().collect();
        let mut i = 0;

        while i < lines.len() {
            // Look for "##" to start a test case
            if lines[i].starts_with("## ") {
                let name = lines[i][3..].trim().to_string();
                i += 1;

                // Skip empty lines
                while i < lines.len() && lines[i].trim().is_empty() {
                    i += 1;
                }

                // Read input code until we hit "---"
                let mut input = String::new();
                while i < lines.len() && !lines[i].starts_with("---") {
                    if !input.is_empty() {
                        input.push('\n');
                    }
                    input.push_str(lines[i]);
                    i += 1;
                }

                // Skip "---" line
                if i < lines.len() && lines[i].starts_with("---") {
                    i += 1;
                }

                // Skip empty lines
                while i < lines.len() && lines[i].trim().is_empty() {
                    i += 1;
                }

                // Read expected output until next "##" or end
                let mut expected = String::new();
                while i < lines.len() && !lines[i].starts_with("##") {
                    if !expected.is_empty() {
                        expected.push('\n');
                    }
                    expected.push_str(lines[i]);
                    i += 1;
                }

                cases.push(TestCase {
                    name,
                    input: input.trim().to_string(),
                    expected: expected.trim().to_string(),
                });
            } else {
                i += 1;
            }
        }

        cases
    }

    /// Format ATOM output with proper indentation and newlines for readability
    #[allow(dead_code)]
    fn pretty_atom(atom: &str) -> String {
        let mut result = String::new();
        let mut indent = 0;
        let mut in_braces = false;
        let mut paren_depth = 0; // Track parenthesis depth
        let mut in_string = false; // Track if we're inside a string literal
        let mut chars = atom.chars().peekable();

        while let Some(c) = chars.next() {
            match c {
                '\n' => {
                    if !in_string {
                        // Check if next word is "else" and add proper indentation
                        let mut lookahead = chars.clone();
                        while let Some(next) = lookahead.next() {
                            if next == ' ' {
                                continue;
                            } else if next.is_alphabetic() {
                                // Check if this spells "else"
                                let mut word = String::new();
                                word.push(next);
                                let mut more_lookahead = lookahead.clone();
                                let mut found_else = true;
                                for expected in "else".chars().skip(1) {
                                    if let Some(next_char) = more_lookahead.next() {
                                        if next_char == expected {
                                            word.push(next_char);
                                        } else {
                                            found_else = false;
                                            break;
                                        }
                                    } else {
                                        found_else = false;
                                        break;
                                    }
                                }
                                if found_else {
                                    result.push('\n');
                                    for _ in 0..indent {
                                        result.push(' ');
                                    }
                                } else {
                                    result.push(c);
                                }
                                break;
                            } else {
                                result.push(c);
                                break;
                            }
                        }
                    } else {
                        result.push(c);
                    }
                }
                '"' => {
                    result.push(c);
                    in_string = !in_string;
                }
                '{' => {
                    if !in_string {
                        in_braces = true;
                        result.push(c);
                        // Add newline and indentation after opening brace
                        result.push('\n');
                        indent += 4;
                        for _ in 0..indent {
                            result.push(' ');
                        }
                        // Consume any spaces that immediately follow { to avoid double spacing
                        while let Some(&next) = chars.peek() {
                            if next == ' ' {
                                chars.next();
                            } else {
                                break;
                            }
                        }
                    } else {
                        result.push(c);
                    }
                }
                '}' => {
                    if in_string {
                        result.push(c);
                    } else if in_braces {
                        // Only trim trailing whitespace on the current line
                        while result.ends_with(' ') || result.ends_with('\n') {
                            result.pop();
                        }
                        result.push('\n');
                        indent -= 4;
                        for _ in 0..indent {
                            result.push(' ');
                        }
                        result.push(c);
                        in_braces = false;
                    } else {
                        result.push(c);
                    }
                }
                '(' => {
                    result.push(c);
                    if !in_string {
                        paren_depth += 1;
                    }
                }
                ')' => {
                    result.push(c);
                    if !in_string {
                        paren_depth -= 1;
                        // Add newline after ) if we're in braces and not inside another function call
                        // Check if next chars are space followed by alphabetic (start of new statement)
                        if in_braces && paren_depth == 0 {
                            if let Some(&next) = chars.peek() {
                                if next.is_alphabetic() {
                                    // Case: `)call` - direct alphabetic after )
                                    result.push('\n');
                                    for _ in 0..indent {
                                        result.push(' ');
                                    }
                                } else if next == ' ' {
                                    // Case: `) call` - space after ), check if it's followed by alphabetic
                                    // Peek ahead to see what comes after the space
                                    let mut chars_copy = chars.clone();
                                    chars_copy.next(); // Skip the space
                                    if let Some(&after_space) = chars_copy.peek() {
                                        if after_space.is_alphabetic() {
                                            // This is `) <space> <alphabetic>` - skip the space and add newline
                                            chars.next(); // Consume the space
                                            result.push('\n');
                                            for _ in 0..indent {
                                                result.push(' ');
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                ',' => {
                    result.push(c);
                    // Add newline after comma if we're in braces AND not inside parens or strings
                    // But if we're inside parens (like in function calls), add a space instead
                    if in_braces && !in_string && paren_depth == 0 {
                        result.push('\n');
                        for _ in 0..indent {
                            result.push(' ');
                        }
                    } else if !in_string {
                        // Only add space if next char is not already a space
                        if let Some(&next) = chars.peek() {
                            if next != ' ' && next != '\n' {
                                result.push(' ');
                            }
                        } else {
                            result.push(' ');
                        }
                    }
                }
                ' ' => {
                    // Preserve spaces inside strings
                    if in_string {
                        result.push(c);
                    } else if !result.ends_with('\n') && !result.is_empty() {
                        if let Some(&next) = chars.peek() {
                            // Skip space if next char is certain punctuation
                            if next != ' ' && next != '}' && next != ',' {
                                result.push(c);
                            }
                        }
                    }
                }
                _ => {
                    result.push(c);
                }
            }
        }

        result.trim().to_string()
    }

    fn run_markdown_test_file(path: &str) {
        let full_path = Path::new("../auto-lang/test/ast").join(path);
        let content = fs::read_to_string(&full_path)
            .unwrap_or_else(|_| panic!("Failed to read test file: {:?}", full_path));

        let cases = parse_markdown_tests(&content);

        for tc in cases {
            // Plan 091: Universe removed
            let _scope = Rc::new(RefCell::new(crate::scope_manager::ScopeManager::new()));
            let mut parser = Parser::from(&tc.input);
            let code = parser
                .parse()
                .unwrap_or_else(|e| panic!("{}: Parse failed: {:?}", tc.name, e));

            // Extract the first statement's atom representation, or join multiple statements with "; "
            let actual = if code.stmts.len() == 1 {
                code.stmts[0].to_atom()
            } else {
                // Join multiple statements with "; " instead of wrapping in (code ...)
                let stmts: Vec<AutoStr> =
                    code.stmts.iter().map(|stmt| stmt.to_atom_str()).collect();
                AutoStr::from(stmts.join("; "))
            };

            // let actual_normalized = pretty_atom(&actual.replace("\r\n", "\n"));
            let actual_normalized = actual.replace("\r\n", "\n");
            let expected_normalized = tc.expected.replace("\r\n", "\n");

            if actual_normalized != expected_normalized {
                // Debug: show the difference
                eprintln!("Comparing:");
                eprintln!(
                    "  Expected len: {}, repr: {:?}",
                    expected_normalized.len(),
                    expected_normalized
                );
                eprintln!(
                    "  Actual len: {}, repr: {:?}",
                    actual_normalized.len(),
                    actual_normalized
                );
                panic!(
                    "\nTest '{}' failed:\nInput:\n{}\n\nExpected:\n{}\n\nActual:\n{}\n",
                    tc.name, tc.input, expected_normalized, actual_normalized
                );
            }
        }
    }

    #[test]
    fn test_01_literals() {
        run_markdown_test_file("01_literals.test.md");
    }

    #[test]
    fn test_02_exprs() {
        run_markdown_test_file("02_exprs.test.md");
    }

    #[test]
    fn test_03_functions() {
        run_markdown_test_file("03_functions.test.md");
    }

    #[test]
    fn test_04_controls() {
        run_markdown_test_file("04_controls.test.md");
    }

    #[test]
    fn test_05_types() {
        run_markdown_test_file("05_types.test.md");
    }

    // TODO: Uncomment when test files are ready
    // #[test]
    // fn test_06_declarations() {
    //     run_markdown_test_file("06_declarations.test.md");
    // }

    // #[test]
    // fn test_07_advanced_control() {
    //     run_markdown_test_file("07_advanced_control.test.md");
    // }

    // #[test]
    // fn test_08_statements() {
    //     run_markdown_test_file("08_statements.test.md");
    // }

    // #[test]
    // fn test_09_events() {
    //     run_markdown_test_file("09_events.test.md");
    // }

    // #[test]
    // fn test_10_complex_cases() {
    //     run_markdown_test_file("10_complex_cases.test.md");
    // }

    // #[test]
    // fn test_11_more_expressions() {
    //     run_markdown_test_file("11_more_expressions.test.md");
    // }

    /// Remove ANSI color codes from string for testing
    fn strip_ansi_codes(s: &str) -> String {
        // This regex matches ANSI escape sequences like \x1b[...m or \033[...m
        let ansi_regex = regex::Regex::new(r"\x1b\[[0-9;]*m").unwrap();
        ansi_regex.replace_all(s, "").to_string()
    }

    /// Run markdown tests that expect errors
    fn run_markdown_error_test_file(path: &str) {
        let full_path = Path::new("../auto-lang/test/ast").join(path);
        let content = fs::read_to_string(&full_path)
            .unwrap_or_else(|_| panic!("Failed to read test file: {:?}", full_path));

        let cases = parse_markdown_tests(&content);

        for tc in cases {
            // Plan 091: Universe removed
            let _scope = Rc::new(RefCell::new(crate::scope_manager::ScopeManager::new()));
            let mut parser = Parser::from(&tc.input);

            match parser.parse() {
                Ok(_) => {
                    panic!(
                        "Test '{}' expected an error but parsing succeeded.\nInput:\n{}",
                        tc.name, tc.input
                    );
                }
                Err(err) => {
                    // Convert error to miette report and format it
                    let report = miette::Report::new(err);
                    let error_output = format!("{:?}", report);

                    // Strip ANSI color codes for reliable string comparison
                    let error_clean = strip_ansi_codes(&error_output);

                    // Check if the error output contains expected text
                    // Normalize both for comparison (handle line endings)
                    let error_normalized = error_clean.replace("\r\n", "\n");
                    let expected_normalized = tc.expected.replace("\r\n", "\n");

                    // Check if all expected lines are present in the error output
                    for expected_line in expected_normalized.lines() {
                        let trimmed = expected_line.trim();
                        if !trimmed.is_empty() && !error_normalized.contains(trimmed) {
                            eprintln!("Error output:\n{}", error_normalized);
                            panic!(
                                "\nTest '{}' failed:\nInput:\n{}\n\nExpected to find:\n{}\n\nIn error output:\n{}\n",
                                tc.name, tc.input, trimmed, error_normalized
                            );
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn test_06_errors() {
        run_markdown_error_test_file("06_errors.test.md");
    }
}

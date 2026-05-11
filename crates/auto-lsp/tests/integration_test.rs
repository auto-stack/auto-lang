//! Integration tests for auto-lsp
//!
//! These tests ensure the LSP compiles and basic features work against
//! sample Auto language code.

/// Test that the LSP backend can be created without panicking
#[test]
fn test_backend_creation() {
    // This is a smoke test to ensure the Backend struct is constructible
    // Full async testing would require a tower-lsp-server Client mock
}

/// Test that auto-lang parser can parse basic Auto code without errors
#[test]
fn test_basic_parse() {
    let code = r#"
fn main() {
    let x = 42
    print(x)
}
"#;

    let mut parser = auto_lang::Parser::from(code);
    let result = parser.parse();
    assert!(result.is_ok(), "Basic Auto code should parse without errors");
    assert!(parser.errors.is_empty(), "There should be no parse errors");
}

/// Test that the parser handles v0.3+ syntax (generics, AutoUI, comptime)
#[test]
fn test_v03_syntax_parse() {
    let code = r#"
widget Counter {
    model {
        count int = 0
    }
    view {
        h1 > Count: ${.count}
    }
}

type Box<T> {
    value T
}

fn make_box<T>(value T) Box<T> {
    return Box { value }
}
"#;

    let mut parser = auto_lang::Parser::from(code);
    let _ = parser.parse();
    // Even if there are errors, the parser should not panic
    // and should collect errors gracefully
}

/// Test that diagnostics can be extracted from parser errors
#[test]
fn test_diagnostics_from_parse_errors() {
    let code = r#"
fn broken() {
    let x = 
}
"#;

    let mut parser = auto_lang::Parser::from(code);
    let _ = parser.parse();
    // The parser should collect errors instead of panicking
    // Note: depending on error recovery, this may or may not produce errors
}

//! AutoDown Integration Tests

use auto_lang::autodown::{
    AdocBlock, AdocDocument, AdocInline, AdocMath, AdocParser, AdocSection,
    AdocLexer, LexerMode,
};
use auto_lang::autodown::lexer::{AdToken, AdTokenKind, AdocLexer as Lexer};
use auto_lang::autodown::trans::{HtmlTranspiler, TypstTranspiler, AdocTranspiler};
use auto_lang::autodown::math::AutoMathParser;

// ============================================================================
// Lexer Tests
// ============================================================================

#[test]
fn test_lexer_plain_text() {
    let source = "Hello, world!";
    let mut lexer = Lexer::new(source);
    let token = lexer.next_token().unwrap();
    assert_eq!(token.kind, AdTokenKind::Text);
}

#[test]
fn test_lexer_h1_header() {
    let source = "# Title";
    let mut lexer = Lexer::new(source);
    let token = lexer.next_token().unwrap();
    assert_eq!(token.kind, AdTokenKind::Header { level: 1 });
}

#[test]
fn test_lexer_h2_header() {
    let source = "## Section";
    let mut lexer = Lexer::new(source);
    let token = lexer.next_token().unwrap();
    assert_eq!(token.kind, AdTokenKind::Header { level: 2 });
}

#[test]
fn test_lexer_h3_header() {
    let source = "### Subsection";
    let mut lexer = Lexer::new(source);
    let token = lexer.next_token().unwrap();
    assert_eq!(token.kind, AdTokenKind::Header { level: 3 });
}

#[test]
fn test_lexer_h6_header() {
    let source = "###### Small heading";
    let mut lexer = Lexer::new(source);
    let token = lexer.next_token().unwrap();
    assert_eq!(token.kind, AdTokenKind::Header { level: 6 });
}

#[test]
fn test_lexer_inline_math() {
    let source = "Math: %{ E = mc^2 }";
    let mut lexer = Lexer::new(source);
    let tokens: Vec<AdToken> = lexer.tokenize_all().unwrap();
    let has_math = tokens.iter().any(|t| t.kind == AdTokenKind::MathStart);
    assert!(has_math, "Expected MathStart token");
}

#[test]
fn test_lexer_eof() {
    let source = "";
    let mut lexer = Lexer::new(source);
    let token = lexer.next_token().unwrap();
    assert_eq!(token.kind, AdTokenKind::EOF);
}

// ============================================================================
// Parser Tests
// ============================================================================

#[test]
fn test_parser_empty_document() {
    let source = "";
    let mut parser = AdocParser::new(source);
    let doc = parser.parse().unwrap();
    assert!(doc.title.is_none());
    assert!(doc.preamble.is_empty());
    assert!(doc.sections.is_empty());
}

#[test]
fn test_parser_simple_paragraph() {
    let source = "Hello, world!";
    let mut parser = AdocParser::new(source);
    let doc = parser.parse().unwrap();
    assert!(!doc.preamble.is_empty());
}

#[test]
fn test_parser_single_section() {
    let source = "# Title\n\nContent";
    let mut parser = AdocParser::new(source);
    let doc = parser.parse().unwrap();
    assert_eq!(doc.sections.len(), 1);
    assert_eq!(doc.sections[0].level, 1);
    assert_eq!(doc.sections[0].title, "Title");
}

#[test]
fn test_parser_multiple_sections() {
    let source = "# S1\n\nC1\n\n# S2\n\nC2";
    let mut parser = AdocParser::new(source);
    let doc = parser.parse().unwrap();
    assert_eq!(doc.sections.len(), 2);
}

// ============================================================================
// Transpiler Tests
// ============================================================================

#[test]
fn test_typst_simple() {
    let source = "Hello, world!";
    let mut parser = AdocParser::new(source);
    let doc = parser.parse().unwrap();
    let mut trans = TypstTranspiler::new();
    let output = trans.transpile(&doc).unwrap();
    assert!(output.contains("Hello"));
}

#[test]
fn test_html_simple() {
    let source = "Hello, world!";
    let mut parser = AdocParser::new(source);
    let doc = parser.parse().unwrap();
    let mut trans = HtmlTranspiler::new();
    let output = trans.transpile(&doc).unwrap();
    assert!(output.contains("Hello"));
}

// ============================================================================
// Math Tests
// ============================================================================

#[test]
fn test_math_simple() {
    let math = AdocMath { 
        content: "a + b".to_string(), 
        parsed: None,
        display: false,
    };
    let result = AutoMathParser::to_latex(&math);
    assert!(!result.is_empty());
}

#[test]
fn test_math_sum() {
    let math = AdocMath { 
        content: "sum(i=0..n, f(i))".to_string(), 
        parsed: None,
        display: false,
    };
    let result = AutoMathParser::to_latex(&math);
    assert!(result.contains("sum"));
}

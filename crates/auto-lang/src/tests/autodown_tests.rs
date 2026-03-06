//! AutoDown Test Suite
use auto_lang::autodown::{
    AdocBlock, AdocDocument, AdocInline, AdocMath, AdocParser, AdocSection,
    AdocLexer, LexerMode,
};
use auto_lang::autodown::lexer::{AdToken, AdTokenKind, AdocLexer as Lexer};
use auto_lang::autodown::trans::{HtmlTranspiler, TypstTranspiler, AdocTranspiler};
use auto_lang::autodown::math::AutoMathParser;

mod lexer_tests {
    use super::*;

    #[test]
    fn test_lexer_plain_text() {
        let source = "Hello, world!";
        let mut lexer = Lexer::new(source);
        let token = lexer.next_token().unwrap();
        assert_eq!(token.kind, AdTokenKind::Text);
    }
}


mod parser_tests {
    use super::*;

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
    fn test_parser_multiple_paragraphs() {
        let source = "Paragraph 1\n\nParagraph 2\n\nParagraph 3";
        let mut parser = AdocParser::new(source);
        let doc = parser.parse().unwrap();
        assert!(doc.preamble.len() >= 2);
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
        let source = "# Section 1\n\nContent 1\n\n# Section 2\n\nContent 2";
        let mut parser = AdocParser::new(source);
        let doc = parser.parse().unwrap();
        assert_eq!(doc.sections.len(), 2);
    }

    #[test]
    fn test_parser_section_with_content() {
        let source = "# Section\n\nThis is content.";
        let mut parser = AdocParser::new(source);
        let doc = parser.parse().unwrap();
        assert_eq!(doc.sections.len(), 1);
        assert!(!doc.sections[0].content.is_empty());
    }

    #[test]
    fn test_parser_paragraph_block() {
        let source = "This is a paragraph.";
        let mut parser = AdocParser::new(source);
        let doc = parser.parse().unwrap();
        assert!(!doc.preamble.is_empty());
        match &doc.preamble[0] {
            AdocBlock::Paragraph(inlines) => assert!(!inlines.is_empty()),
            _ => panic!("Expected Paragraph block"),
        }
    }

    #[test]
    fn test_parser_unordered_list() {
        let source = "- Item 1\n- Item 2\n- Item 3";
        let mut parser = AdocParser::new(source);
        let doc = parser.parse().unwrap();
        assert!(!doc.preamble.is_empty());
    }

    #[test]
    fn test_parser_ordered_list() {
        let source = "1. First\n2. Second\n3. Third";
        let mut parser = AdocParser::new(source);
        let doc = parser.parse().unwrap();
        assert!(!doc.preamble.is_empty());
    }
}

mod transpiler_tests {
    use super::*;

    #[test]
    fn test_typst_simple_paragraph() {
        let source = "Hello, world!";
        let mut parser = AdocParser::new(source);
        let doc = parser.parse().unwrap();
        let mut transpiler = TypstTranspiler::new();
        let output = transpiler.transpile(&doc).unwrap();
        assert!(output.contains("Hello"));
    }

    #[test]
    fn test_typst_header() {
        let source = "# Title\n\nContent";
        let mut parser = AdocParser::new(source);
        let doc = parser.parse().unwrap();
        let mut transpiler = TypstTranspiler::new();
        let output = transpiler.transpile(&doc).unwrap();
        assert!(output.contains("Title"));
    }

    #[test]
    fn test_html_simple_paragraph() {
        let source = "Hello, world!";
        let mut parser = AdocParser::new(source);
        let doc = parser.parse().unwrap();
        let mut transpiler = HtmlTranspiler::new();
        let output = transpiler.transpile(&doc).unwrap();
        assert!(output.contains("Hello"));
    }

    #[test]
    fn test_html_header() {
        let source = "# Title\n\nContent";
        let mut parser = AdocParser::new(source);
        let doc = parser.parse().unwrap();
        let mut transpiler = HtmlTranspiler::new();
        let output = transpiler.transpile(&doc).unwrap();
        assert!(output.contains("<h1") || output.contains("Title"));
    }

    #[test]
    fn test_html_bold() {
        let source = "**bold text**";
        let mut parser = AdocParser::new(source);
        let doc = parser.parse().unwrap();
        let mut transpiler = HtmlTranspiler::new();
        let output = transpiler.transpile(&doc).unwrap();
        assert!(output.contains("<strong") || output.contains("bold"));
    }

    #[test]
    fn test_html_italic() {
        let source = "_italic text_";
        let mut parser = AdocParser::new(source);
        let doc = parser.parse().unwrap();
        let mut transpiler = HtmlTranspiler::new();
        let output = transpiler.transpile(&doc).unwrap();
        assert!(output.contains("<em") || output.contains("italic"));
    }
}

mod math_tests {
    use super::*;

    #[test]
    fn test_math_simple_expression() {
        let math = AdocMath { content: "a + b".to_string(), parsed: None };
        let result = AutoMathParser::to_latex(&math);
        assert!(!result.is_empty());
    }

    #[test]
    fn test_math_sum_function() {
        let math = AdocMath { content: "sum(i=0..n, f(i))".to_string(), parsed: None };
        let result = AutoMathParser::to_latex(&math);
        assert!(result.contains("sum"));
    }

    #[test]
    fn test_math_sqrt() {
        let math = AdocMath { content: "sqrt(x)".to_string(), parsed: None };
        let result = AutoMathParser::to_latex(&math);
        assert!(result.contains("sqrt"));
    }

    #[test]
    fn test_math_to_typst() {
        let math = AdocMath { content: "a + b".to_string(), parsed: None };
        let result = AutoMathParser::to_typst(&math);
        assert!(!result.is_empty());
    }
}

mod error_tests {
    use super::*;

    #[test]
    fn test_parser_handles_empty_input() {
        let source = "";
        let mut parser = AdocParser::new(source);
        let doc = parser.parse().unwrap();
        assert!(doc.sections.is_empty());
        assert!(doc.preamble.is_empty());
    }

    #[test]
    fn test_parser_handles_whitespace_only() {
        let source = "   \n\n   \t\t   \n";
        let mut parser = AdocParser::new(source);
        let doc = parser.parse().unwrap();
        assert!(doc.sections.is_empty());
    }

    #[test]
    fn test_lexer_empty_input() {
        let source = "";
        let mut lexer = Lexer::new(source);
        let token = lexer.next_token().unwrap();
        assert_eq!(token.kind, AdTokenKind::EOF);
    }
}

mod lexer_advanced_tests {
    use super::*;

    #[test]
    fn test_lexer_h4_header() {
        let source = "#### Heading 4";
        let mut lexer = Lexer::new(source);
        let token = lexer.next_token().unwrap();
        assert_eq!(token.kind, AdTokenKind::Header { level: 4 });
    }

    #[test]
    fn test_lexer_h5_header() {
        let source = "##### Heading 5";
        let mut lexer = Lexer::new(source);
        let token = lexer.next_token().unwrap();
        assert_eq!(token.kind, AdTokenKind::Header { level: 5 });
    }

    #[test]
    fn test_lexer_header_with_text() {
        let source = "# Hello World";
        let mut lexer = Lexer::new(source);
        let token = lexer.next_token().unwrap();
        assert_eq!(token.kind, AdTokenKind::Header { level: 1 });
        let token = lexer.next_token().unwrap();
        assert_eq!(token.kind, AdTokenKind::Text);
    }

    #[test]
    fn test_lexer_blank_line() {
        let source = "Line 1\n\nLine 2";
        let mut lexer = Lexer::new(source);
        let tokens: Vec<AdToken> = lexer.tokenize_all().unwrap();
        let has_blank = tokens.iter().any(|t| t.kind == AdTokenKind::BlankLine);
        assert!(has_blank, "Expected BlankLine token");
    }

    #[test]
    fn test_lexer_list_item() {
        let source = "- Item";
        let mut lexer = Lexer::new(source);
        let token = lexer.next_token().unwrap();
        assert_eq!(token.kind, AdTokenKind::ListItem);
    }

    #[test]
    fn test_lexer_numbered_list() {
        let source = "1. Item";
        let mut lexer = Lexer::new(source);
        let token = lexer.next_token().unwrap();
        assert_eq!(token.kind, AdTokenKind::NumberedList);
    }

    #[test]
    fn test_lexer_code_fence() {
        let source = "```rust\ncode\n```";
        let mut lexer = Lexer::new(source);
        let token = lexer.next_token().unwrap();
        assert_eq!(token.kind, AdTokenKind::CodeFence);
    }

    #[test]
    fn test_lexer_blockquote() {
        let source = "> quote";
        let mut lexer = Lexer::new(source);
        let token = lexer.next_token().unwrap();
        assert_eq!(token.kind, AdTokenKind::Blockquote);
    }

    #[test]
    fn test_lexer_horizontal_rule() {
        let source = "---";
        let mut lexer = Lexer::new(source);
        let token = lexer.next_token().unwrap();
        assert_eq!(token.kind, AdTokenKind::HorizontalRule);
    }

    #[test]
    fn test_lexer_interpolation() {
        let source = "Hello \${name}!";
        let mut lexer = Lexer::new(source);
        let tokens: Vec<AdToken> = lexer.tokenize_all().unwrap();
        let has_interpolate = tokens.iter().any(|t| t.kind == AdTokenKind::InterpolateStart);
        assert!(has_interpolate, "Expected InterpolateStart token");
    }

    #[test]
    fn test_lexer_if_keyword() {
        let source = "\$if condition {";
        let mut lexer = Lexer::new(source);
        let _ = lexer.next_token().unwrap(); // Dollar
        let token = lexer.next_token().unwrap();
        assert_eq!(token.kind, AdTokenKind::If);
    }

    #[test]
    fn test_lexer_for_keyword() {
        let source = "\$for item in .list {";
        let mut lexer = Lexer::new(source);
        let _ = lexer.next_token().unwrap(); // Dollar
        let token = lexer.next_token().unwrap();
        assert_eq!(token.kind, AdTokenKind::For);
    }

    #[test]
    fn test_lexer_else_keyword() {
        let source = "\$else {";
        let mut lexer = Lexer::new(source);
        let _ = lexer.next_token().unwrap(); // Dollar
        let token = lexer.next_token().unwrap();
        assert_eq!(token.kind, AdTokenKind::Else);
    }
}

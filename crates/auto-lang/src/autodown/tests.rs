//! Comprehensive tests for AutoDown document format
//!
//! Tests cover lexer, parser, transpilers, and math conversion.

#[cfg(test)]
mod lexer_tests {
    use crate::autodown::lexer::{AdTokenKind, AdocLexer, LexerMode};

    #[test]
    fn test_lexer_text_mode() {
        let mut lexer = AdocLexer::new("Hello, world!");
        let token = lexer.next_token().unwrap();
        assert_eq!(token.kind, AdTokenKind::Text);
        assert_eq!(token.text, "Hello,");
    }

    #[test]
    fn test_lexer_header() {
        let mut lexer = AdocLexer::new("# Title");
        let token = lexer.next_token().unwrap();
        assert_eq!(token.kind, AdTokenKind::Header { level: 1 });
    }

    #[test]
    fn test_lexer_code_mode_switch() {
        let mut lexer = AdocLexer::new("$x = 42");
        let token = lexer.next_token().unwrap();
        assert_eq!(token.kind, AdTokenKind::Dollar);
    }

    #[test]
    fn test_lexer_math_mode() {
        let mut lexer = AdocLexer::new("%{E = mc^2}");
        let token = lexer.next_token().unwrap();
        assert_eq!(token.kind, AdTokenKind::MathStart);
    }
}

#[cfg(test)]
mod parser_tests {
    use crate::autodown::parser::AdocParser;

    #[test]
    fn test_parser_empty_document() {
        let mut parser = AdocParser::new("");
        let doc = parser.parse().unwrap();
        assert!(doc.title.is_none());
        assert!(doc.preamble.is_empty());
        assert!(doc.sections.is_empty());
    }

    #[test]
    fn test_parser_simple_paragraph() {
        let mut parser = AdocParser::new("Hello, world!");
        let doc = parser.parse().unwrap();
        assert_eq!(doc.preamble.len(), 1);
    }

    #[test]
    fn test_parser_section_with_header() {
        let mut parser = AdocParser::new("# Introduction\n\nThis is content.");
        let doc = parser.parse().unwrap();
        assert_eq!(doc.sections.len(), 1);
        assert_eq!(doc.sections[0].title, "Introduction");
        assert_eq!(doc.sections[0].level, 1);
    }
}

#[cfg(test)]
mod transpiler_tests {
    use crate::autodown::ast::*;
    use crate::autodown::trans::html::HtmlTranspiler;
    use crate::autodown::trans::typst::TypstTranspiler;
    use crate::autodown::trans::AdocTranspiler;

    #[test]
    fn test_html_simple_document() {
        let doc = AdocDocument {
            title: Some("Test Document".to_string()),
            metadata: AdocMetadata::default(),
            preamble: vec![],
            sections: vec![],
        };

        let mut transpiler = HtmlTranspiler::new();
        let result = transpiler.transpile(&doc).unwrap();
        assert!(result.contains("<!DOCTYPE html>"));
        assert!(result.contains("<title>Test Document</title>"));
    }

    #[test]
    fn test_html_paragraph() {
        let doc = AdocDocument {
            title: None,
            metadata: AdocMetadata::default(),
            preamble: vec![AdocBlock::Paragraph(vec![AdocInline::Text(
                "Hello, world!".to_string(),
            )])],
            sections: vec![],
        };

        let mut transpiler = HtmlTranspiler::new();
        let result = transpiler.transpile(&doc).unwrap();
        assert!(result.contains("<p"));
        assert!(result.contains("Hello, world!"));
    }

    #[test]
    fn test_html_bold_text() {
        let doc = AdocDocument {
            title: None,
            metadata: AdocMetadata::default(),
            preamble: vec![AdocBlock::Paragraph(vec![AdocInline::Bold(vec![
                AdocInline::Text("bold".to_string()),
            ])])],
            sections: vec![],
        };

        let mut transpiler = HtmlTranspiler::new();
        let result = transpiler.transpile(&doc).unwrap();
        assert!(result.contains("<strong"));
        assert!(result.contains("bold"));
    }

    #[test]
    fn test_html_code_block() {
        let doc = AdocDocument {
            title: None,
            metadata: AdocMetadata::default(),
            preamble: vec![AdocBlock::CodeBlock {
                lang: Some("rust".to_string()),
                code: "fn main() {}".to_string(),
            }],
            sections: vec![],
        };

        let mut transpiler = HtmlTranspiler::new();
        let result = transpiler.transpile(&doc).unwrap();
        assert!(result.contains("<pre"));
        assert!(result.contains("<code"));
        assert!(result.contains("fn main()"));
    }

    #[test]
    fn test_html_escape() {
        let doc = AdocDocument {
            title: None,
            metadata: AdocMetadata::default(),
            preamble: vec![AdocBlock::Paragraph(vec![AdocInline::Text(
                "<script>alert('XSS')</script>".to_string(),
            )])],
            sections: vec![],
        };

        let mut transpiler = HtmlTranspiler::new();
        let result = transpiler.transpile(&doc).unwrap();
        assert!(!result.contains("<script>"));
        assert!(result.contains("&lt;script&gt;"));
    }

    #[test]
    fn test_typst_simple_document() {
        let doc = AdocDocument {
            title: Some("Test Document".to_string()),
            metadata: AdocMetadata::default(),
            preamble: vec![],
            sections: vec![],
        };

        let mut transpiler = TypstTranspiler::new();
        let result = transpiler.transpile(&doc).unwrap();
        assert!(result.contains("#set document(title:"));
        assert!(result.contains("Test Document"));
    }

    #[test]
    fn test_typst_paragraph() {
        let doc = AdocDocument {
            title: None,
            metadata: AdocMetadata::default(),
            preamble: vec![AdocBlock::Paragraph(vec![AdocInline::Text(
                "Hello, world!".to_string(),
            )])],
            sections: vec![],
        };

        let mut transpiler = TypstTranspiler::new();
        let result = transpiler.transpile(&doc).unwrap();
        assert!(result.contains("Hello, world!"));
    }

    #[test]
    fn test_typst_code_block() {
        let doc = AdocDocument {
            title: None,
            metadata: AdocMetadata::default(),
            preamble: vec![AdocBlock::CodeBlock {
                lang: Some("rust".to_string()),
                code: "fn main() {}".to_string(),
            }],
            sections: vec![],
        };

        let mut transpiler = TypstTranspiler::new();
        let result = transpiler.transpile(&doc).unwrap();
        assert!(result.contains("```rust"));
        assert!(result.contains("fn main()"));
    }

    #[test]
    fn test_typst_section() {
        let doc = AdocDocument {
            title: None,
            metadata: AdocMetadata::default(),
            preamble: vec![],
            sections: vec![AdocSection {
                level: 1,
                title: "Introduction".to_string(),
                id: None,
                content: vec![AdocBlock::Paragraph(vec![AdocInline::Text(
                    "Content".to_string(),
                )])],
                subsections: vec![],
            }],
        };

        let mut transpiler = TypstTranspiler::new();
        let result = transpiler.transpile(&doc).unwrap();
        assert!(result.contains("= Introduction"));
        assert!(result.contains("Content"));
    }
}

#[cfg(test)]
mod math_tests {
    use crate::autodown::ast::AdocMath;
    use crate::autodown::math::AutoMathParser;

    #[test]
    fn test_math_to_latex() {
        let math = AdocMath {
            content: "E = mc^2".to_string(),
            display: false,
        };
        let latex = AutoMathParser::to_latex(&math);
        assert!(latex.contains("E = mc^2"));
    }

    #[test]
    fn test_math_display_mode() {
        let math = AdocMath {
            content: "x^2".to_string(),
            display: true,
        };
        assert!(math.display);
    }

    #[test]
    fn test_math_inline_mode() {
        let math = AdocMath {
            content: "a + b".to_string(),
            display: false,
        };
        assert!(!math.display);
    }
}

#[cfg(test)]
mod ast_tests {
    use crate::autodown::ast::*;

    #[test]
    fn test_document_builder() {
        let doc = AdocDocument::with_title("Test");
        assert_eq!(doc.title, Some("Test".to_string()));
    }

    #[test]
    fn test_section_creation() {
        let section = AdocSection::new(1, "Introduction");
        assert_eq!(section.level, 1);
        assert_eq!(section.title, "Introduction");
    }

    #[test]
    fn test_list_item_creation() {
        let item = AdocListItem::simple(vec![AdocInline::Text("Item".to_string())]);
        assert_eq!(item.content.len(), 1);
        assert!(item.nested.is_none());
    }

    #[test]
    fn test_inline_text() {
        let inline = AdocInline::Text("Hello".to_string());
        assert!(matches!(inline, AdocInline::Text(_)));
    }

    #[test]
    fn test_inline_bold() {
        let inline = AdocInline::Bold(vec![AdocInline::Text("bold".to_string())]);
        assert!(matches!(inline, AdocInline::Bold(_)));
    }

    #[test]
    fn test_inline_italic() {
        let inline = AdocInline::Italic(vec![AdocInline::Text("italic".to_string())]);
        assert!(matches!(inline, AdocInline::Italic(_)));
    }

    #[test]
    fn test_block_paragraph() {
        let block = AdocBlock::Paragraph(vec![AdocInline::Text("text".to_string())]);
        assert!(matches!(block, AdocBlock::Paragraph(_)));
    }

    #[test]
    fn test_block_code() {
        let block = AdocBlock::CodeBlock {
            lang: Some("rust".to_string()),
            code: "fn main() {}".to_string(),
        };
        assert!(matches!(block, AdocBlock::CodeBlock { .. }));
    }

    #[test]
    fn test_block_list() {
        let block = AdocBlock::List {
            items: vec![AdocListItem::simple(vec![AdocInline::Text(
                "item".to_string(),
            )])],
            ordered: false,
        };
        assert!(matches!(block, AdocBlock::List { .. }));
    }
}

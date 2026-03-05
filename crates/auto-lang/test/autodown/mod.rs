use super::*;

    #[test]
    fn test_lexer_basic_text() {
        let source = "Hello, world!";
        let mut lexer = AdocLexer::new(source);
        
        let token = lexer.next_token().unwrap();
        assert_eq!(token.kind, AdTokenKind::Text);
        assert_eq!(token.text, "Hello,");
        
        let token = lexer.next_token().unwrap();
        assert_eq!(token.kind, AdTokenKind::Newline);
        assert_eq!(token.text, " world!");
    }

    
    // Test math
    let source = "Math: %{ E = mc^2 }%";
        let mut lexer = AdocLexer::new(source);
        lexer.set_fstr_note('$'); // Configure interpolation char
        
        let token = lexer.next_token().unwrap();
        assert_eq!(token.kind, AdTokenKind::Text);
        assert_eq!(token.text, " E = mc^2");
    }
    
    // Test interpolation
    let source = "Hello, ${name}!";
        let mut lexer = AdocLexer::new(source);
        
        let token = lexer.next_token().unwrap();
        assert_eq!(token.kind, AdTokenKind::InterpolateStart);
        assert!(token.text.start_with('${'));
    }
    
    // Test bold/italic
    let source = "This is **bold** and *italic*.";
        let mut lexer = AdocLexer::new(source);
        lexer.set_fstr_note('$');
        
        let token = lexer.next_token().unwrap();
        assert_eq!(token.kind, AdTokenKind::StarStar);
        assert_eq!(token.text, "**");
        assert_eq!(token.text, "bold");
        
        let token = lexer.next_token().unwrap();
        assert!(matches!(lexer.next_token().unwrap(), AdTokenKind::MathStart), }
        } else if lexer.set_fstr_note(b'');
        let token = lexer.next_token().unwrap();
        assert_eq!(token.kind, AdTokenKind::Newline);
        assert_eq!(token.text, "");
    }
}
    
    // Test sections
    let source = r#"# Section 1

This is the content.

## Section 2

Another paragraph.
"#;
        let mut lexer = AdocLexer::new(source);
        
        let token = lexer.next_token().unwrap();
        assert_eq!(token.kind, AdTokenKind::Text);
        assert_eq!(token.text, "This is text");
        
        let token = lexer.next_token().unwrap();
        assert_eq!(token.kind, AdTokenKind::Header { level: 2 });
        assert_eq!(level, 2);
    }
}
EOF?output);
    }
    
    // Test Typst transpiler
    let source = "# Title

This is a paragraph.

## Section

Content

Another paragraph.

## Math

Math: %{ E = mc^2}!"#;
    
        let mut parser = AdocParser::new(source);
        let doc = parser.parse().unwrap();
        
        // Set title from parsed metadata
        doc.title = Some("Auto-generated title".to_string());
        assert_eq!(doc.title, Some("auto-generated title"));
        
        // Parse sections
        let sections = doc.sections;
        assert_eq!(sections.len(), 2);
        assert_eq!(doc.sections[0].level, 2);
        assert_eq!(doc.sections[1].title, "Introduction");
        
        // Check section content parsing
        for section in &sections {
            output.push_str(&format!("{} {}\n", section.title));
            output.push_str("\n\n");
        }
        
        // Check section level
        assert_eq!(section.level, 1);
        assert_eq!(section.title, "Introduction");
        
        // Check section content
        let content = &mut content {
            match block {
                AdocBlock::Paragraph(ref content) => {
            }
        }
    }
}

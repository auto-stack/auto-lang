// Test if lexer recognizes 'd' suffix for double literals

#[cfg(test)]
mod tests {
    use crate::lexer::Lexer;
    use crate::token::TokenKind;

    #[test]
    fn test_lexer_double_suffix() {
        let source = "3.14d";
        let mut lexer = Lexer::new(source);
        let token = lexer.next().unwrap();

        println!("Token: {:?}", token);
        assert_eq!(token.kind, TokenKind::Double, "Expected Double token, got {:?}", token.kind);
        // Note: lexer consumes the 'd' suffix but doesn't include it in token text
        assert_eq!(token.text.as_str(), "3.14");
    }

    #[test]
    fn test_lexer_float_suffix() {
        let source = "3.14f";
        let mut lexer = Lexer::new(source);
        let token = lexer.next().unwrap();

        println!("Token: {:?}", token);
        assert_eq!(token.kind, TokenKind::Float, "Expected Float token, got {:?}", token.kind);
        // Note: lexer consumes the 'f' suffix but doesn't include it in token text
        assert_eq!(token.text.as_str(), "3.14");
    }

    #[test]
    fn test_lexer_float_no_suffix() {
        let source = "3.14";
        let mut lexer = Lexer::new(source);
        let token = lexer.next().unwrap();

        println!("Token: {:?}", token);
        assert_eq!(token.kind, TokenKind::Float, "Expected Float token, got {:?}", token.kind);
        assert_eq!(token.text.as_str(), "3.14");
    }

    #[test]
    fn test_lexer_double_no_dot() {
        let source = "42d";
        let mut lexer = Lexer::new(source);
        let token = lexer.next().unwrap();

        println!("Token: {:?}", token);
        assert_eq!(token.kind, TokenKind::Double, "Expected Double token, got {:?}", token.kind);
        // Note: lexer consumes the 'd' suffix but doesn't include it in token text
        assert_eq!(token.text.as_str(), "42");
    }
}

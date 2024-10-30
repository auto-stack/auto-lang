use crate::token::Pos;
use crate::token::{Token, TokenKind};
use std::iter::Peekable;
use std::str::Chars;

pub struct Lexer<'a> {
    chars: Peekable<Chars<'a>>,
    line: usize,
    pos: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(code: &'a str) -> Self {
        Lexer {
            chars: code.chars().peekable(),
            line: 0,
            pos: 0,
        }
    }

    pub fn pos(&mut self, len: usize) -> Pos {
        let p = Pos {
            line: self.line,
            pos: self.pos,
            len,
        };
        self.pos += len;
        p
    }

    pub fn single(&mut self, kind: TokenKind, c: char) -> Token {
        let tok = Token::new(kind, self.pos(1), c.to_string());
        self.chars.next();
        tok
    }

    pub fn peek(&mut self, c: char) -> bool {
        if let Some(&nc) = self.chars.peek() {
            if nc == c {
                return true;
            }
        }
        false
    }

    pub fn with_equal(&mut self, kind1: TokenKind, kind2: TokenKind, c: char) -> Token {
        self.chars.next(); // skip c
        if self.peek('=') {
            self.chars.next(); // skip =
            return Token::new(kind2, self.pos(2), format!("{}{}", c, '='));
        }
        Token::new(kind1, self.pos(1), c.to_string())
    }
}

// Lexer methods for various token types
impl<'a> Lexer<'a> {
    pub fn number(&mut self) -> Token {
        let mut text = String::new();
        let mut has_dot = false;
        while let Some(&c) = self.chars.peek() {
            if c.is_digit(10) {
                text.push(c);
                self.chars.next();
            } else if c == '.' {
                let mut more = self.chars.clone();
                more.next();
                if more.peek() == Some(&'.') {
                    break;
                }
                has_dot = true;
                text.push(c);
                self.chars.next();
            } else {
                break;
            }
        }
        if has_dot {
            Token::float(self.pos(text.len()), text)
        } else {
            Token::int(self.pos(text.len()), text)
        }
    }

    pub fn str(&mut self) -> Token {
        let mut text = String::new();
        self.chars.next();
        while let Some(&c) = self.chars.peek() {
            if c == '"' {
                self.chars.next();
                break;
            }
            text.push(c);
            self.chars.next();
        }
        Token::str(self.pos(text.len()), text)
    }

    fn dot_or_range(&mut self) -> Token {
        self.chars.next(); // skip .
        if self.peek('.') {
            self.chars.next();
            if self.peek('=') {
                self.chars.next();
                return Token::new(TokenKind::RangeEq, self.pos(3), "..=".to_string());
            }
            return Token::new(TokenKind::Range, self.pos(2), "..".to_string());
        }
        Token::new(TokenKind::Dot, self.pos(1), ".".to_string())
    }

    fn keyword_tok(&mut self, kind: TokenKind, text: &str) -> Option<Token> {
        Some(Token::new(kind, self.pos(text.len()), text.to_string()))
    }

    pub fn keyword(&mut self, text: String) -> Option<Token> {
        match text.as_str() {
            "true" => self.keyword_tok(TokenKind::True, &text),
            "false" => self.keyword_tok(TokenKind::False, &text),
            "nil" => self.keyword_tok(TokenKind::Nil, &text),
            "if" => self.keyword_tok(TokenKind::If, &text),
            "else" => self.keyword_tok(TokenKind::Else, &text),
            "for" => self.keyword_tok(TokenKind::For, &text),
            "var" => self.keyword_tok(TokenKind::Var, &text),
            "in" => self.keyword_tok(TokenKind::In, &text),
            "fn" => self.keyword_tok(TokenKind::Fn, &text),
            "type" => self.keyword_tok(TokenKind::Type, &text),
            _ => {
                // AutoUI Keywords
                // TODO: Add an Option to not check these keywords
                match text.as_str() {
                    "widget" => self.keyword_tok(TokenKind::Widget, &text),
                    "model" => self.keyword_tok(TokenKind::Model, &text),
                    "view" => self.keyword_tok(TokenKind::View, &text),
                    "style" => self.keyword_tok(TokenKind::Style, &text),
                    _ => None,
                }
            }
        }
    }

    pub fn identifier(&mut self) -> Token {
        let mut text = String::new();
        while let Some(&c) = self.chars.peek() {
            if c.is_alphabetic() {
                text.push(c);
                self.chars.next();
            } else {
                break;
            }
        }
        if let Some(keyword) = self.keyword(text.clone()) {
            keyword
        } else {
            Token::ident(self.pos(text.len()), text)
        }
    }
}

impl<'a> Lexer<'a> {
    pub fn skip_whitespace(&mut self) {
        while let Some(&c) = self.chars.peek() {
            if c.is_whitespace() && c != '\n' {
                self.chars.next();
            } else {
                break;
            }
        }
    }
    pub fn next(&mut self) -> Token {
        // skip whitespace
        self.skip_whitespace();
        while let Some(&c) = self.chars.peek() {
            match c {
                '(' => {
                    return self.single(TokenKind::LParen, c);
                }
                ')' => {
                    return self.single(TokenKind::RParen, c);
                }
                '[' => {
                    return self.single(TokenKind::LSquare, c);
                }
                ']' => {
                    return self.single(TokenKind::RSquare, c);
                }
                '{' => {
                    return self.single(TokenKind::LBrace, c);
                }
                '}' => {
                    return self.single(TokenKind::RBrace, c);
                }
                '"' => {
                    return self.str();
                }
                ':' => {
                    return self.single(TokenKind::Colon, c);
                }
                ',' => {
                    return self.single(TokenKind::Comma, c);
                }
                ';' => {
                    return self.single(TokenKind::Semi, c);
                }
                '\n' => {
                    return self.single(TokenKind::Newline, c);
                }
                '+' => {
                    return self.single(TokenKind::Add, c);
                }
                '-' => {
                    return self.single(TokenKind::Sub, c);
                }
                '*' => {
                    return self.single(TokenKind::Mul, c);
                }
                '/' => {
                    return self.single(TokenKind::Div, c);
                }
                '!' => {
                    return self.with_equal(TokenKind::Not, TokenKind::Neq, c);
                }
                '>' => {
                    return self.with_equal(TokenKind::Gt, TokenKind::Ge, c);
                }
                '<' => {
                    return self.with_equal(TokenKind::Lt, TokenKind::Le, c);
                }
                '=' => {
                    return self.with_equal(TokenKind::Asn, TokenKind::Eq, c);
                }
                '.' => {
                    return self.dot_or_range();
                }
                '|' => {
                    return self.single(TokenKind::VBar, c);
                }
                _ => {
                    if c.is_digit(10) {
                        return self.number();
                    }

                    if c.is_alphabetic() {
                        return self.identifier();
                    }

                    panic!("unknown character: {}", c);
                    
                }
            }
        }
        Token::eof(self.pos(0))
    }

    fn tokens(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        loop {
            let token = self.next();
            if token.kind == TokenKind::EOF {
                break;
            }
            tokens.push(token);
        }
        tokens
    }

    fn tokens_str(&mut self) -> String {
        let tokens = self.tokens();
        tokens.iter().map(|t| t.to_string()).collect::<Vec<String>>().join("")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_token_strings(code: &str) -> String {
        let mut lexer = Lexer::new(code);
        lexer.tokens_str()
    }

    #[test]
    fn test_lexer() {
        let code = "(123)";
        let tokens = parse_token_strings(code);
        assert_eq!(
            tokens,
            "<(><int:123><)>"
        );
    }

    #[test]
    fn test_str() {
        let code = "\"Hello, World!\"";
        let tokens = parse_token_strings(code);
        assert_eq!(
            tokens,
            "<str:Hello, World!>"
        );
    }

    #[test]
    fn test_range() {
        let code = "1..5";
        let tokens = parse_token_strings(code);
        assert_eq!(tokens, "<int:1><..><int:5>");
    }


    #[test]
    fn test_pair() {
        let code = r#"a: 3
        b: 4"#;
        let tokens = parse_token_strings(code);
        assert_eq!(tokens, "<ident:a><:><int:3><nl><ident:b><:><int:4>");
    }
}

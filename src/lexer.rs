use crate::token::Pos;
use crate::token::{Token, TokenKind};
use std::iter::Peekable;
use std::str::Chars;

pub struct Lexer<'a> {
    code: &'a str,
    chars: Peekable<Chars<'a>>,
    line: usize,
    pos: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(code: &'a str) -> Self {
        Lexer {
            code,
            chars: code.chars().peekable(),
            line: 0,
            pos: 0,
        }
    }

    pub fn pos(&mut self, len: usize) -> Pos {
        let p = Pos {
            line: self.line,
            pos: self.pos,
            len: len,
        };
        self.pos += len;
        p
    }

    pub fn single(&mut self, kind: TokenKind, c: char) -> Token {
        let tok = Token::new(kind, self.pos(1), c.to_string());
        self.chars.next();
        tok
    }

    pub fn with_equal(&mut self, kind1: TokenKind, kind2: TokenKind, c: char) -> Token {
        self.chars.next(); // skip c
        if let Some(&nc) = self.chars.peek() {
            if nc == '=' {      
                self.chars.next(); // skip =
                return Token::new(kind2, self.pos(2), format!("{}{}", c, nc))
            }
        }
        Token::new(kind1, self.pos(1), c.to_string())
    }

    pub fn print(&mut self) {
        println!("--- Tokens ---");
        loop {
            let token = self.next();
            println!("  {:?}: '{}' at line {}, position {}",
                token.kind,
                token.text,
                token.pos.line,
                token.pos.pos
            );
            if token.kind == TokenKind::EOF {
                break;
            }
        }
        println!("--- Tokens End ---");
    }

}

// Lexer methods for various token types
impl<'a> Lexer<'a> {

    pub fn number(&mut self) -> Token {
        let mut text = String::new();
        let mut has_dot = false;
        while let Some(&c) = self.chars.peek() {
            if c.is_digit(10) || c == '.' {
                text.push(c);
                self.chars.next();
            } else if c == '.' {
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
            _ => None,
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
            if c.is_whitespace() {
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
                _ => {
                    if c.is_digit(10) {
                        return self.number();
                    }

                    if c.is_alphabetic() {
                        return self.identifier();
                    }
                }
            }
        }
        Token::eof(self.pos(0))
    }

    fn tokens(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        while let token = self.next() {
            if token.kind == TokenKind::EOF {
                break;
            }
            tokens.push(token);
        }
        tokens
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lexer() {
        let code = "(123)";
        let mut lexer = Lexer::new(code);
        let tokens = lexer.tokens();
        assert_eq!(
            tokens,
            vec![
                Token {
                    kind: TokenKind::LParen,
                    pos: Pos {
                        line: 0,
                        pos: 0,
                        len: 1
                    },
                    text: "(".to_string()
                },
                Token {
                    kind: TokenKind::Integer,
                    pos: Pos {
                        line: 0,
                        pos: 1,
                        len: 3
                    },
                    text: "123".to_string()
                },
                Token {
                    kind: TokenKind::RParen,
                    pos: Pos {
                        line: 0,
                        pos: 4,
                        len: 1
                    },
                    text: ")".to_string()
                },
            ]
        );
    }

    #[test]
    fn test_str() {
        let code = "\"Hello, World!\"";
        let mut lexer = Lexer::new(code);
        let tokens = lexer.tokens();
        assert_eq!(
            tokens,
            vec![Token {
                kind: TokenKind::Str,
                pos: Pos {
                    line: 0,
                    pos: 0,
                    len: 13
                },
                text: "Hello, World!".to_string()
            }]
        );
    }
}

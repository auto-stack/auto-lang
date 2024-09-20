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

    pub fn print(&mut self) {
        println!("--- Tokens ---");
        for token in self.into_iter() {
            println!("  {:?}: '{}' at line {}, position {}",
                token.kind,
                token.text,
                token.pos.line,
                token.pos.pos
            );
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

    pub fn keyword(&mut self, text: String) -> Option<Token> {
        match text.as_str() {
            "true" => Some(Token::true_(self.pos(text.len()))),
            "false" => Some(Token::false_(self.pos(text.len()))),
            "nil" => Some(Token::nil(self.pos(text.len()))),
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

impl<'a> Iterator for Lexer<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(&c) = self.chars.peek() {
            match c {
                '(' => {
                    return Some(self.single(TokenKind::LParen, c));
                }
                ')' => {
                    return Some(self.single(TokenKind::RParen, c));
                }
                '"' => {
                    return Some(self.str());
                }
                ';' => {
                    return Some(self.single(TokenKind::Semi, c));
                }
                '\n' => {
                    return Some(self.single(TokenKind::Newline, c));
                }
                '+' => {
                    return Some(self.single(TokenKind::Add, c));
                }
                '-' => {
                    return Some(self.single(TokenKind::Sub, c));
                }
                '*' => {
                    return Some(self.single(TokenKind::Mul, c));
                }
                '/' => {
                    return Some(self.single(TokenKind::Div, c));
                }
                _ => {
                    if c.is_digit(10) {
                        return Some(self.number());
                    }

                    if c.is_alphabetic() {
                        return Some(self.identifier());
                    }
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lexer() {
        let code = "(123)";
        let lexer = Lexer::new(code);
        let tokens = lexer.collect::<Vec<Token>>();
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
        let lexer = Lexer::new(code);
        let tokens = lexer.collect::<Vec<Token>>();
        assert_eq!(
            tokens,
            vec![Token {
                kind: TokenKind::Str,
                pos: Pos {
                    line: 0,
                    pos: 0,
                    len: 13
                },
                text: "\"Hello, World!\"".to_string()
            }]
        );
    }
}

use std::fmt;
use strum_macros;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Pos {
    pub line: usize,
    pub pos: usize,
    pub len: usize,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, strum_macros::Display)]
#[strum(serialize_all = "snake_case")]
pub enum TokenKind {

    // Literals
    Int,
    Float,
    Str,
    Ident,

    // Operators
    LParen,
    RParen,
    LSquare,
    RSquare,
    LBrace,
    RBrace,
    Comma,
    Semi,
    Newline,
    Add,
    Sub,
    Mul,
    Div,
    Not,
    Lt,
    Gt,
    Le,
    Ge,
    Asn,
    Eq,
    Neq,
    Dot, // .
    Range, // ..
    RangeEq, // ..= 

    // Keywords
    True,
    False,
    Nil,
    If,
    Else,
    For,
    Var,


    // EOF
    EOF,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub pos: Pos,
    pub text: String,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<{}:{}>", self.kind, self.text)
    }
}

impl Token {
    pub fn new(kind: TokenKind, pos: Pos, text: String) -> Self {
        Token { kind, pos, text }
    }

    pub fn int(pos: Pos, text: String) -> Self {
        Token::new(TokenKind::Int, pos, text)
    }

    pub fn float(pos: Pos, text: String) -> Self {
        Token::new(TokenKind::Float, pos, text)
    }

    pub fn str(pos: Pos, text: String) -> Self {
        Token::new(TokenKind::Str, pos, text)
    }

    pub fn ident(pos: Pos, text: String) -> Self {
        Token::new(TokenKind::Ident, pos, text)
    }

    pub fn nil(pos: Pos) -> Self {
        Token::new(TokenKind::Nil, pos, "nil".to_string())
    }

    pub fn eof(pos: Pos) -> Self {
        Token::new(TokenKind::EOF, pos, "".to_string())
    }

}



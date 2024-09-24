#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Pos {
    pub line: usize,
    pub pos: usize,
    pub len: usize,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum TokenKind {

    // Literals
    Integer,
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

    // Keywords
    True,
    False,
    Nil,
    If,
    Else,

    // EOF
    EOF,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub pos: Pos,
    pub text: String,
}

impl Token {
    pub fn new(kind: TokenKind, pos: Pos, text: String) -> Self {
        Token { kind, pos, text }
    }

    pub fn int(pos: Pos, text: String) -> Self {
        Token::new(TokenKind::Integer, pos, text)
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



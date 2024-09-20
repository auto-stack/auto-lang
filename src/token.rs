#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Pos {
    pub line: usize,
    pub pos: usize,
    pub len: usize,
}

#[derive(Debug, PartialEq, Eq)]
pub enum TokenKind {
    LParen,
    RParen,
    Integer,
    Float,
    True,
    False,
    Str,
    Ident,
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

    pub fn true_(pos: Pos) -> Self {
        Token::new(TokenKind::True, pos, "true".to_string())
    }

    pub fn false_(pos: Pos) -> Self {
        Token::new(TokenKind::False, pos, "false".to_string())
    }

    pub fn str(pos: Pos, text: String) -> Self {
        Token::new(TokenKind::Str, pos, text)
    }

    pub fn lparen(pos: Pos) -> Self {
        Token::new(TokenKind::LParen, pos, "(".to_string())
    }

    pub fn rparen(pos: Pos) -> Self {
        Token::new(TokenKind::RParen, pos, ")".to_string())
    }

    pub fn ident(pos: Pos, text: String) -> Self {
        Token::new(TokenKind::Ident, pos, text)
    }
}



use std::fmt;
use strum_macros;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
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
    Uint,
    Float,
    Str,
    Ident,

    // Operators
    LParen, // (
    RParen, // )
    LSquare, // [
    RSquare, // ]
    LBrace, // {
    RBrace, // }
    Comma, // ,
    Semi, // ;
    Newline, // \n
    Add, // +
    Sub, // -
    Mul, // *
    Div, // /
    Not, // !
    Lt, // <
    Gt, // >
    Le, // <=
    Ge, // >=
    Asn, // =
    Eq, // ==
    Neq, // !=
    Dot, // .
    Range, // ..
    RangeEq, // ..= 
    Colon, // :
    VBar, // |
    Dollar, // $
    Backtick, // `
    CommentLine, // //
    CommentContent, // any text in comment
    CommentStart, // /*
    CommentEnd, // */

    // Keywords
    True,
    False,
    Nil,
    If,
    Else,
    For,
    Var,
    In,
    Fn,
    Type,
    Ref,
    Let,
    Mut,
    Has,


    // Format Str
    FStrStart,
    FStrPart,
    FStrEnd,

    // Keywords For AutoData
    Grid,

    // Keywords For AutoUI
    Widget,
    Model,
    View,
    Style,

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
        match self.kind {
            TokenKind::Newline => write!(f, "<nl>"),
            TokenKind::Colon => write!(f, "<:>"),
            TokenKind::Range => write!(f, "<..>"),
            TokenKind::RangeEq => write!(f, "<..=>"),
            TokenKind::Dot => write!(f, "<.>"),
            TokenKind::Comma => write!(f, "<,>"),
            TokenKind::Semi => write!(f, "<;>"),
            TokenKind::LParen => write!(f, "<(>"),
            TokenKind::RParen => write!(f, "<)>"),
            TokenKind::LSquare => write!(f, "<[>"),
            TokenKind::RSquare => write!(f, "<]>"),
            TokenKind::LBrace => write!(f, "<{{>"),
            TokenKind::RBrace => write!(f, "<}}>"),
            TokenKind::Add => write!(f, "<+>"),
            TokenKind::Sub => write!(f, "<->"),
            TokenKind::Mul => write!(f, "<*>"),
            TokenKind::Div => write!(f, "</>"),
            TokenKind::Not => write!(f, "<!>"),
            TokenKind::Lt => write!(f, "<lt>"),
            TokenKind::Gt => write!(f, "<gt>"),
            TokenKind::Le => write!(f, "<le>"),
            TokenKind::Ge => write!(f, "<ge>"),
            TokenKind::Asn => write!(f, "<=>"),
            TokenKind::Eq => write!(f, "<==>"),
            TokenKind::Neq => write!(f, "<!=>"),
            TokenKind::True => write!(f, "<true>"),
            TokenKind::False => write!(f, "<false>"),
            TokenKind::Nil => write!(f, "<nil>"),
            TokenKind::If => write!(f, "<if>"),
            TokenKind::Else => write!(f, "<else>"),
            TokenKind::For => write!(f, "<for>"),
            TokenKind::Var => write!(f, "<var>"),
            TokenKind::Let => write!(f, "<let>"),
            TokenKind::Mut => write!(f, "<mut>"),
            TokenKind::In => write!(f, "<in>"),
            TokenKind::Fn => write!(f, "<fn>"),
            TokenKind::Type => write!(f, "<type>"),
            TokenKind::Widget => write!(f, "<widget>"),
            TokenKind::Model => write!(f, "<model>"),
            TokenKind::View => write!(f, "<view>"),
            TokenKind::Style => write!(f, "<style>"),
            TokenKind::Dollar => write!(f, "<$>"),
            TokenKind::FStrStart => write!(f, "<fstrs>"),
            TokenKind::FStrEnd => write!(f, "<fstre>"),
            TokenKind::FStrPart => write!(f, "<fstrp:{}>", self.text),
            TokenKind::CommentLine => write!(f, "<//>"),
            TokenKind::CommentContent => write!(f, "<comment:...>"),
            TokenKind::CommentStart => write!(f, "</*>"),
            TokenKind::CommentEnd => write!(f, "<*/>"),
            TokenKind::Backtick => write!(f, "<`>"),
            TokenKind::Ref => write!(f, "<ref>"),
            TokenKind::EOF => write!(f, "<eof>"),
            _ => write!(f, "<{}:{}>", self.kind, self.text),
        }
    }
}

impl Token {
    pub fn new(kind: TokenKind, pos: Pos, text: String) -> Self {
        Token { kind, pos, text }
    }

    pub fn int(pos: Pos, text: String) -> Self {
        Token::new(TokenKind::Int, pos, text)
    }

    pub fn uint(pos: Pos, text: String) -> Self {
        Token::new(TokenKind::Uint, pos, text)
    }

    pub fn float(pos: Pos, text: String) -> Self {
        Token::new(TokenKind::Float, pos, text)
    }

    pub fn str(pos: Pos, text: String) -> Self {
        Token::new(TokenKind::Str, pos, text)
    }

    pub fn fstr_start(pos: Pos, text: String) -> Self {
        Token::new(TokenKind::FStrStart, pos, text)
    }

    pub fn fstr_part(pos: Pos, text: String) -> Self {
        Token::new(TokenKind::FStrPart, pos, text)
    }

    pub fn fstr_end(pos: Pos, text: String) -> Self {
        Token::new(TokenKind::FStrEnd, pos, text)
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

impl fmt::Display for Pos {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}:{}", self.line, self.pos, self.len)
    }
}

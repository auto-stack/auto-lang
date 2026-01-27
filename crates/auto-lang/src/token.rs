use auto_val::AutoStr;
use std::fmt;
use strum_macros;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Pos {
    pub line: usize,
    pub at: usize,
    pub pos: usize,
    pub len: usize,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, strum_macros::Display)]
#[strum(serialize_all = "snake_case")]
pub enum TokenKind {
    // Literals
    Int,
    Uint,
    U8,
    I8,
    Bool,  // ADDED: bool literal
    Byte,  // ADDED: byte literal
    Float,
    Double,
    Str,  // "hello"
    CStr, // c"hello"
    Char, // 'c'
    Ident,

    // Operators
    LParen,         // (
    RParen,         // )
    LSquare,        // [
    RSquare,        // ]
    LBrace,         // {
    RBrace,         // }
    Comma,          // ,
    Semi,           // ;
    Newline,        // \n
    Add,            // +
    Sub,            // -
    Star,           // *
    Div,            // /
    Not,            // !
    Lt,             // <
    Gt,             // >
    Le,             // <=
    Ge,             // >=
    Asn,            // =
    Eq,             // ==
    Neq,            // !=
    AddEq,          // +=
    SubEq,          // -=
    MulEq,          // *=
    DivEq,          // /=
    Dot,            // .
    Range,          // ..
    RangeEq,        // ..=
    Colon,          // :
    VBar,           // |
    CommentLine,    // //
    CommentContent, // any text in comment
    CommentStart,   // /*
    CommentEnd,     // */
    Arrow,          // ->
    DoubleArrow,    // =>
    Question,       // ?
    QuestionQuestion, // ??
    DotQuestion,    // ?.
    At,             // @
    Hash,           // #
    Tilde,          // ~

    // Keywords
    True,
    False,
    Nil,  // nil for variables
    Null, // null for pointers
    If,
    Else,
    For,
    When,
    Break,
    Is,
    Var,
    In,
    Fn,
    Type,
    Union,
    Tag,
    Let,
    Mut,
    Const, // ADDED: const keyword for const generics (Plan 052)
    View,  // ADDED: view keyword for immutable borrow (Phase 3)
    Take,  // ADDED: take keyword for move semantics (Phase 3)
    Hold,  // ADDED: hold keyword for temporary path binding (Phase 3)
    Has,
    Spec,
    Use,
    As,
    Enum,
    On,
    Alias,
    Node, // ADDED: node keyword for typed node definitions
    Ext,   // ADDED: ext keyword for type extensions (Plan 035)
    Static, // ADDED: static keyword for static methods (Plan 035)

    // Property Keywords (Phase 3: postfix property syntax)
    DotView,  // .view
    DotMut,   // .mut
    DotTake,  // .take

    // Format Str
    FStrStart,
    FStrPart,
    FStrEnd,
    FStrNote,

    // Keywords For AutoData
    Grid,

    // EOF
    EOF,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub pos: Pos,
    pub text: AutoStr,
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
            TokenKind::Star => write!(f, "<*>"),
            TokenKind::Div => write!(f, "</>"),
            TokenKind::Not => write!(f, "<!>"),
            TokenKind::Lt => write!(f, "<lt>"),
            TokenKind::Gt => write!(f, "<gt>"),
            TokenKind::Le => write!(f, "<le>"),
            TokenKind::Ge => write!(f, "<ge>"),
            TokenKind::Asn => write!(f, "<=>"),
            TokenKind::Eq => write!(f, "<==>"),
            TokenKind::Neq => write!(f, "<!=>"),
            TokenKind::Arrow => write!(f, "<->>"),
            TokenKind::DoubleArrow => write!(f, "<=>>"),
            TokenKind::Hash => write!(f, "<#>"),
            TokenKind::True => write!(f, "<true>"),
            TokenKind::False => write!(f, "<false>"),
            TokenKind::Nil => write!(f, "<nil>"),
            TokenKind::If => write!(f, "<if>"),
            TokenKind::Else => write!(f, "<else>"),
            TokenKind::For => write!(f, "<for>"),
            TokenKind::Var => write!(f, "<var>"),
            TokenKind::Let => write!(f, "<let>"),
            TokenKind::Mut => write!(f, "<mut>"),
            TokenKind::View => write!(f, "<view>"),
            TokenKind::Take => write!(f, "<take>"),
            TokenKind::Hold => write!(f, "<hold>"),
            TokenKind::In => write!(f, "<in>"),
            TokenKind::Fn => write!(f, "<fn>"),
            TokenKind::Type => write!(f, "<type>"),
            TokenKind::Union => write!(f, "<union>"),
            TokenKind::Tag => write!(f, "<tag>"),
            TokenKind::Alias => write!(f, "<alias>"),
            TokenKind::Node => write!(f, "<node>"),
            TokenKind::FStrNote => write!(f, "<{}>", self.text),
            TokenKind::FStrStart => write!(f, "<fstrs>"),
            TokenKind::FStrEnd => write!(f, "<fstre>"),
            TokenKind::FStrPart => write!(f, "<fstrp:{}>", self.text),
            TokenKind::CommentLine => write!(f, "<//>"),
            TokenKind::CommentContent => write!(f, "<comment:...>"),
            TokenKind::CommentStart => write!(f, "</*>"),
            TokenKind::CommentEnd => write!(f, "<*/>"),
            TokenKind::EOF => write!(f, "<eof>"),
            TokenKind::Char => write!(f, "<'{}'>", self.text),
            TokenKind::Is => write!(f, "<is>"),
            TokenKind::When => write!(f, "<when>"),
            TokenKind::On => write!(f, "<on>"),
            TokenKind::Question => write!(f, "<?>"),
            TokenKind::QuestionQuestion => write!(f, "??"),
            TokenKind::DotQuestion => write!(f, "?."),
            TokenKind::Use => write!(f, "<use>"),
            TokenKind::Spec => write!(f, "<spec>"),
            TokenKind::CStr => write!(f, "<cstr:{}>", self.text),
            TokenKind::At => write!(f, "<@>"),
            TokenKind::Tilde => write!(f, "<~>"),
            _ => write!(f, "<{}:{}>", self.kind, self.text),
        }
    }
}

impl Token {
    pub fn new(kind: TokenKind, pos: Pos, text: AutoStr) -> Self {
        Token { kind, pos, text }
    }

    pub fn int(pos: Pos, text: AutoStr) -> Self {
        Token::new(TokenKind::Int, pos, text)
    }

    pub fn uint(pos: Pos, text: AutoStr) -> Self {
        Token::new(TokenKind::Uint, pos, text)
    }

    pub fn u8(pos: Pos, text: AutoStr) -> Self {
        Token::new(TokenKind::U8, pos, text)
    }

    pub fn i8(pos: Pos, text: AutoStr) -> Self {
        Token::new(TokenKind::I8, pos, text)
    }

    pub fn float(pos: Pos, text: AutoStr) -> Self {
        Token::new(TokenKind::Float, pos, text)
    }

    pub fn double(pos: Pos, text: AutoStr) -> Self {
        Token::new(TokenKind::Double, pos, text)
    }

    pub fn char(pos: Pos, text: AutoStr) -> Self {
        Token::new(TokenKind::Char, pos, text)
    }

    pub fn str(pos: Pos, text: AutoStr) -> Self {
        Token::new(TokenKind::Str, pos, text)
    }

    pub fn fstr_part(pos: Pos, text: AutoStr) -> Self {
        Token::new(TokenKind::FStrPart, pos, text)
    }

    pub fn ident(pos: Pos, text: AutoStr) -> Self {
        Token::new(TokenKind::Ident, pos, text)
    }

    pub fn eof(pos: Pos) -> Self {
        Token::new(TokenKind::EOF, pos, "".into())
    }
}

impl fmt::Display for Pos {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}:{}", self.line, self.at, self.len)
    }
}

impl Token {
    pub fn keyword_kind(text: &str) -> Option<TokenKind> {
        match text {
            "true" => Some(TokenKind::True),
            "false" => Some(TokenKind::False),
            "nil" => Some(TokenKind::Nil),
            "null" => Some(TokenKind::Null),
            "if" => Some(TokenKind::If),
            "else" => Some(TokenKind::Else),
            "for" => Some(TokenKind::For),
            "when" => Some(TokenKind::When),
            "is" => Some(TokenKind::Is),
            "var" => Some(TokenKind::Var),
            "in" => Some(TokenKind::In),
            "fn" => Some(TokenKind::Fn),
            "type" => Some(TokenKind::Type),
            "union" => Some(TokenKind::Union),
            "tag" => Some(TokenKind::Tag),
            "let" => Some(TokenKind::Let),
            "mut" => Some(TokenKind::Mut),
            "view" => Some(TokenKind::View),
            "take" => Some(TokenKind::Take),
            "hold" => Some(TokenKind::Hold),
            "has" => Some(TokenKind::Has),
            "spec" => Some(TokenKind::Spec),
            "use" => Some(TokenKind::Use),
            "as" => Some(TokenKind::As),
            "enum" => Some(TokenKind::Enum),
            "grid" => Some(TokenKind::Grid),
            "alias" => Some(TokenKind::Alias),
            "break" => Some(TokenKind::Break),
            "ext" => Some(TokenKind::Ext),
            "static" => Some(TokenKind::Static),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spec_keyword() {
        // Test that "spec" is recognized as a keyword
        let kind = Token::keyword_kind("spec");
        assert_eq!(kind, Some(TokenKind::Spec));

        // Test that "Spec" (capitalized) is NOT recognized as a keyword
        let kind = Token::keyword_kind("Spec");
        assert_eq!(kind, None);

        // Test that other keywords still work
        assert_eq!(Token::keyword_kind("has"), Some(TokenKind::Has));
        assert_eq!(Token::keyword_kind("fn"), Some(TokenKind::Fn));
        assert_eq!(Token::keyword_kind("type"), Some(TokenKind::Type));
    }

    #[test]
    fn test_spec_display() {
        let pos = Pos {
            line: 1,
            at: 1,
            pos: 0,
            len: 4,
        };
        let token = Token::new(TokenKind::Spec, pos, "spec".into());
        assert_eq!(format!("{}", token), "<spec>");
    }
}

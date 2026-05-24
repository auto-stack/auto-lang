// Phase 3 Bootstrap: Merged auto/lib compilation test
// Auto-generated from 12 auto/lib/*.a2r.rs files

#[allow(unused_imports)]
use auto_lang::a2r_std;
use auto_lang::a2r_std::*;

// === pos ===
// a2r Standard Library (from crate)

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Pos {
    pub line: i32,
    pub at: i32,
    pub total: i32,
}

// === error ===
// a2r Standard Library (from crate)

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Error {
    pub message: String,
    pub line: u32,
    pub at: u32,
}

// === token ===
// a2r Standard Library (from crate)

#[derive(Clone, Debug, PartialEq)]
enum TokenKind {
    Int = 0,
    Uint = 1,
    U8 = 2,
    I8 = 3,
    Bool = 4,
    Byte = 5,
    Float = 6,
    Double = 7,
    Str = 8,
    CStr = 9,
    Char = 10,
    Ident = 11,
    LParen = 12,
    RParen = 13,
    LSquare = 14,
    RSquare = 15,
    LBrace = 16,
    RBrace = 17,
    Comma = 18,
    Semi = 19,
    Newline = 20,
    Add = 21,
    Sub = 22,
    Star = 23,
    Div = 24,
    Mod = 25,
    Not = 26,
    Lt = 27,
    Gt = 28,
    Le = 29,
    Ge = 30,
    Asn = 31,
    Eq = 32,
    Neq = 33,
    AddEq = 34,
    SubEq = 35,
    MulEq = 36,
    DivEq = 37,
    ModEq = 38,
    Dot = 39,
    Range = 40,
    RangeEq = 41,
    Colon = 42,
    VBar = 43,
    Amp = 44,
    Arrow = 45,
    DoubleArrow = 46,
    Question = 47,
    QuestionQuestion = 48,
    DotQuestion = 49,
    DotQuest = 50,
    At = 51,
    Hash = 52,
    Tilde = 53,
    CommentLine = 54,
    CommentContent = 55,
    CommentStart = 56,
    CommentEnd = 57,
    DocComment = 58,
    HashIf = 59,
    HashFor = 60,
    HashIs = 61,
    HashBrace = 62,
    True = 63,
    False = 64,
    Nil = 65,
    Null = 66,
    If = 67,
    Else = 68,
    For = 69,
    Loop = 70,
    Break = 71,
    Continue = 72,
    Return = 73,
    Is = 74,
    In = 75,
    Fn = 76,
    Type = 77,
    Union = 78,
    Tag = 79,
    Let = 80,
    Var = 81,
    Mut = 82,
    Move = 83,
    Copy = 84,
    Const = 85,
    View = 86,
    Take = 87,
    Hold = 88,
    Spec = 89,
    Use = 90,
    Pac = 91,
    Super = 92,
    As = 93,
    To = 94,
    Enum = 95,
    On = 96,
    Alias = 97,
    Node = 98,
    Ext = 99,
    Static = 100,
    Shared = 101,
    Impl = 102,
    Has = 103,
    Dep = 104,
    Routes = 105,
    Outlet = 106,
    Link = 107,
    Route = 108,
    Nav = 109,
    NoneKW = 110,
    SomeKW = 111,
    OkKW = 112,
    ErrKW = 113,
    Task = 114,
    Spawn = 115,
    Await = 116,
    Reply = 117,
    Go = 118,
    DotView = 119,
    DotMut = 120,
    DotMove = 121,
    DotTake = 122,
    And = 123,
    Or = 124,
    FStrStart = 125,
    FStrPart = 126,
    FStrEnd = 127,
    FStrNote = 128,
    Grid = 129,
    EOF = 130,
}

impl std::fmt::Display for TokenKind {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            TokenKind::Int => write!(f, "Int"),
            TokenKind::Uint => write!(f, "Uint"),
            TokenKind::U8 => write!(f, "U8"),
            TokenKind::I8 => write!(f, "I8"),
            TokenKind::Bool => write!(f, "Bool"),
            TokenKind::Byte => write!(f, "Byte"),
            TokenKind::Float => write!(f, "Float"),
            TokenKind::Double => write!(f, "Double"),
            TokenKind::Str => write!(f, "Str"),
            TokenKind::CStr => write!(f, "CStr"),
            TokenKind::Char => write!(f, "Char"),
            TokenKind::Ident => write!(f, "Ident"),
            TokenKind::LParen => write!(f, "LParen"),
            TokenKind::RParen => write!(f, "RParen"),
            TokenKind::LSquare => write!(f, "LSquare"),
            TokenKind::RSquare => write!(f, "RSquare"),
            TokenKind::LBrace => write!(f, "LBrace"),
            TokenKind::RBrace => write!(f, "RBrace"),
            TokenKind::Comma => write!(f, "Comma"),
            TokenKind::Semi => write!(f, "Semi"),
            TokenKind::Newline => write!(f, "Newline"),
            TokenKind::Add => write!(f, "Add"),
            TokenKind::Sub => write!(f, "Sub"),
            TokenKind::Star => write!(f, "Star"),
            TokenKind::Div => write!(f, "Div"),
            TokenKind::Mod => write!(f, "Mod"),
            TokenKind::Not => write!(f, "Not"),
            TokenKind::Lt => write!(f, "Lt"),
            TokenKind::Gt => write!(f, "Gt"),
            TokenKind::Le => write!(f, "Le"),
            TokenKind::Ge => write!(f, "Ge"),
            TokenKind::Asn => write!(f, "Asn"),
            TokenKind::Eq => write!(f, "Eq"),
            TokenKind::Neq => write!(f, "Neq"),
            TokenKind::AddEq => write!(f, "AddEq"),
            TokenKind::SubEq => write!(f, "SubEq"),
            TokenKind::MulEq => write!(f, "MulEq"),
            TokenKind::DivEq => write!(f, "DivEq"),
            TokenKind::ModEq => write!(f, "ModEq"),
            TokenKind::Dot => write!(f, "Dot"),
            TokenKind::Range => write!(f, "Range"),
            TokenKind::RangeEq => write!(f, "RangeEq"),
            TokenKind::Colon => write!(f, "Colon"),
            TokenKind::VBar => write!(f, "VBar"),
            TokenKind::Amp => write!(f, "Amp"),
            TokenKind::Arrow => write!(f, "Arrow"),
            TokenKind::DoubleArrow => write!(f, "DoubleArrow"),
            TokenKind::Question => write!(f, "Question"),
            TokenKind::QuestionQuestion => write!(f, "QuestionQuestion"),
            TokenKind::DotQuestion => write!(f, "DotQuestion"),
            TokenKind::DotQuest => write!(f, "DotQuest"),
            TokenKind::At => write!(f, "At"),
            TokenKind::Hash => write!(f, "Hash"),
            TokenKind::Tilde => write!(f, "Tilde"),
            TokenKind::CommentLine => write!(f, "CommentLine"),
            TokenKind::CommentContent => write!(f, "CommentContent"),
            TokenKind::CommentStart => write!(f, "CommentStart"),
            TokenKind::CommentEnd => write!(f, "CommentEnd"),
            TokenKind::DocComment => write!(f, "DocComment"),
            TokenKind::HashIf => write!(f, "HashIf"),
            TokenKind::HashFor => write!(f, "HashFor"),
            TokenKind::HashIs => write!(f, "HashIs"),
            TokenKind::HashBrace => write!(f, "HashBrace"),
            TokenKind::True => write!(f, "True"),
            TokenKind::False => write!(f, "False"),
            TokenKind::Nil => write!(f, "Nil"),
            TokenKind::Null => write!(f, "Null"),
            TokenKind::If => write!(f, "If"),
            TokenKind::Else => write!(f, "Else"),
            TokenKind::For => write!(f, "For"),
            TokenKind::Loop => write!(f, "Loop"),
            TokenKind::Break => write!(f, "Break"),
            TokenKind::Continue => write!(f, "Continue"),
            TokenKind::Return => write!(f, "Return"),
            TokenKind::Is => write!(f, "Is"),
            TokenKind::In => write!(f, "In"),
            TokenKind::Fn => write!(f, "Fn"),
            TokenKind::Type => write!(f, "Type"),
            TokenKind::Union => write!(f, "Union"),
            TokenKind::Tag => write!(f, "Tag"),
            TokenKind::Let => write!(f, "Let"),
            TokenKind::Var => write!(f, "Var"),
            TokenKind::Mut => write!(f, "Mut"),
            TokenKind::Move => write!(f, "Move"),
            TokenKind::Copy => write!(f, "Copy"),
            TokenKind::Const => write!(f, "Const"),
            TokenKind::View => write!(f, "View"),
            TokenKind::Take => write!(f, "Take"),
            TokenKind::Hold => write!(f, "Hold"),
            TokenKind::Spec => write!(f, "Spec"),
            TokenKind::Use => write!(f, "Use"),
            TokenKind::Pac => write!(f, "Pac"),
            TokenKind::Super => write!(f, "Super"),
            TokenKind::As => write!(f, "As"),
            TokenKind::To => write!(f, "To"),
            TokenKind::Enum => write!(f, "Enum"),
            TokenKind::On => write!(f, "On"),
            TokenKind::Alias => write!(f, "Alias"),
            TokenKind::Node => write!(f, "Node"),
            TokenKind::Ext => write!(f, "Ext"),
            TokenKind::Static => write!(f, "Static"),
            TokenKind::Shared => write!(f, "Shared"),
            TokenKind::Impl => write!(f, "Impl"),
            TokenKind::Has => write!(f, "Has"),
            TokenKind::Dep => write!(f, "Dep"),
            TokenKind::Routes => write!(f, "Routes"),
            TokenKind::Outlet => write!(f, "Outlet"),
            TokenKind::Link => write!(f, "Link"),
            TokenKind::Route => write!(f, "Route"),
            TokenKind::Nav => write!(f, "Nav"),
            TokenKind::NoneKW => write!(f, "NoneKW"),
            TokenKind::SomeKW => write!(f, "SomeKW"),
            TokenKind::OkKW => write!(f, "OkKW"),
            TokenKind::ErrKW => write!(f, "ErrKW"),
            TokenKind::Task => write!(f, "Task"),
            TokenKind::Spawn => write!(f, "Spawn"),
            TokenKind::Await => write!(f, "Await"),
            TokenKind::Reply => write!(f, "Reply"),
            TokenKind::Go => write!(f, "Go"),
            TokenKind::DotView => write!(f, "DotView"),
            TokenKind::DotMut => write!(f, "DotMut"),
            TokenKind::DotMove => write!(f, "DotMove"),
            TokenKind::DotTake => write!(f, "DotTake"),
            TokenKind::And => write!(f, "And"),
            TokenKind::Or => write!(f, "Or"),
            TokenKind::FStrStart => write!(f, "FStrStart"),
            TokenKind::FStrPart => write!(f, "FStrPart"),
            TokenKind::FStrEnd => write!(f, "FStrEnd"),
            TokenKind::FStrNote => write!(f, "FStrNote"),
            TokenKind::Grid => write!(f, "Grid"),
            TokenKind::EOF => write!(f, "EOF"),
        }
    }
}
impl TokenKind {
    pub fn from_id(id: &str) -> Self {
        match id {
            "Int" | "int" => TokenKind::Int,
            "Uint" | "uint" => TokenKind::Uint,
            "U8" | "u8" => TokenKind::U8,
            "I8" | "i8" => TokenKind::I8,
            "Bool" | "bool" => TokenKind::Bool,
            "Byte" | "byte" => TokenKind::Byte,
            "Float" | "float" => TokenKind::Float,
            "Double" | "double" => TokenKind::Double,
            "Str" | "str" => TokenKind::Str,
            "CStr" | "cstr" => TokenKind::CStr,
            "Char" | "char" => TokenKind::Char,
            "Ident" | "ident" => TokenKind::Ident,
            "LParen" | "lparen" => TokenKind::LParen,
            "RParen" | "rparen" => TokenKind::RParen,
            "LSquare" | "lsquare" => TokenKind::LSquare,
            "RSquare" | "rsquare" => TokenKind::RSquare,
            "LBrace" | "lbrace" => TokenKind::LBrace,
            "RBrace" | "rbrace" => TokenKind::RBrace,
            "Comma" | "comma" => TokenKind::Comma,
            "Semi" | "semi" => TokenKind::Semi,
            "Newline" | "newline" => TokenKind::Newline,
            "Add" | "add" => TokenKind::Add,
            "Sub" | "sub" => TokenKind::Sub,
            "Star" | "star" => TokenKind::Star,
            "Div" | "div" => TokenKind::Div,
            "Mod" | "mod" => TokenKind::Mod,
            "Not" | "not" => TokenKind::Not,
            "Lt" | "lt" => TokenKind::Lt,
            "Gt" | "gt" => TokenKind::Gt,
            "Le" | "le" => TokenKind::Le,
            "Ge" | "ge" => TokenKind::Ge,
            "Asn" | "asn" => TokenKind::Asn,
            "Eq" | "eq" => TokenKind::Eq,
            "Neq" | "neq" => TokenKind::Neq,
            "AddEq" | "addeq" => TokenKind::AddEq,
            "SubEq" | "subeq" => TokenKind::SubEq,
            "MulEq" | "muleq" => TokenKind::MulEq,
            "DivEq" | "diveq" => TokenKind::DivEq,
            "ModEq" | "modeq" => TokenKind::ModEq,
            "Dot" | "dot" => TokenKind::Dot,
            "Range" | "range" => TokenKind::Range,
            "RangeEq" | "rangeeq" => TokenKind::RangeEq,
            "Colon" | "colon" => TokenKind::Colon,
            "VBar" | "vbar" => TokenKind::VBar,
            "Amp" | "amp" => TokenKind::Amp,
            "Arrow" | "arrow" => TokenKind::Arrow,
            "DoubleArrow" | "doublearrow" => TokenKind::DoubleArrow,
            "Question" | "question" => TokenKind::Question,
            "QuestionQuestion" | "questionquestion" => TokenKind::QuestionQuestion,
            "DotQuestion" | "dotquestion" => TokenKind::DotQuestion,
            "DotQuest" | "dotquest" => TokenKind::DotQuest,
            "At" | "at" => TokenKind::At,
            "Hash" | "hash" => TokenKind::Hash,
            "Tilde" | "tilde" => TokenKind::Tilde,
            "CommentLine" | "commentline" => TokenKind::CommentLine,
            "CommentContent" | "commentcontent" => TokenKind::CommentContent,
            "CommentStart" | "commentstart" => TokenKind::CommentStart,
            "CommentEnd" | "commentend" => TokenKind::CommentEnd,
            "DocComment" | "doccomment" => TokenKind::DocComment,
            "HashIf" | "hashif" => TokenKind::HashIf,
            "HashFor" | "hashfor" => TokenKind::HashFor,
            "HashIs" | "hashis" => TokenKind::HashIs,
            "HashBrace" | "hashbrace" => TokenKind::HashBrace,
            "True" | "true" => TokenKind::True,
            "False" | "false" => TokenKind::False,
            "Nil" | "nil" => TokenKind::Nil,
            "Null" | "null" => TokenKind::Null,
            "If" | "if" => TokenKind::If,
            "Else" | "else" => TokenKind::Else,
            "For" | "for" => TokenKind::For,
            "Loop" | "loop" => TokenKind::Loop,
            "Break" | "break" => TokenKind::Break,
            "Continue" | "continue" => TokenKind::Continue,
            "Return" | "return" => TokenKind::Return,
            "Is" | "is" => TokenKind::Is,
            "In" | "in" => TokenKind::In,
            "Fn" | "fn" => TokenKind::Fn,
            "Type" | "type" => TokenKind::Type,
            "Union" | "union" => TokenKind::Union,
            "Tag" | "tag" => TokenKind::Tag,
            "Let" | "let" => TokenKind::Let,
            "Var" | "var" => TokenKind::Var,
            "Mut" | "mut" => TokenKind::Mut,
            "Move" | "move" => TokenKind::Move,
            "Copy" | "copy" => TokenKind::Copy,
            "Const" | "const" => TokenKind::Const,
            "View" | "view" => TokenKind::View,
            "Take" | "take" => TokenKind::Take,
            "Hold" | "hold" => TokenKind::Hold,
            "Spec" | "spec" => TokenKind::Spec,
            "Use" | "use" => TokenKind::Use,
            "Pac" | "pac" => TokenKind::Pac,
            "Super" | "super" => TokenKind::Super,
            "As" | "as" => TokenKind::As,
            "To" | "to" => TokenKind::To,
            "Enum" | "enum" => TokenKind::Enum,
            "On" | "on" => TokenKind::On,
            "Alias" | "alias" => TokenKind::Alias,
            "Node" | "node" => TokenKind::Node,
            "Ext" | "ext" => TokenKind::Ext,
            "Static" | "static" => TokenKind::Static,
            "Shared" | "shared" => TokenKind::Shared,
            "Impl" | "impl" => TokenKind::Impl,
            "Has" | "has" => TokenKind::Has,
            "Dep" | "dep" => TokenKind::Dep,
            "Routes" | "routes" => TokenKind::Routes,
            "Outlet" | "outlet" => TokenKind::Outlet,
            "Link" | "link" => TokenKind::Link,
            "Route" | "route" => TokenKind::Route,
            "Nav" | "nav" => TokenKind::Nav,
            "NoneKW" | "nonekw" => TokenKind::NoneKW,
            "SomeKW" | "somekw" => TokenKind::SomeKW,
            "OkKW" | "okkw" => TokenKind::OkKW,
            "ErrKW" | "errkw" => TokenKind::ErrKW,
            "Task" | "task" => TokenKind::Task,
            "Spawn" | "spawn" => TokenKind::Spawn,
            "Await" | "await" => TokenKind::Await,
            "Reply" | "reply" => TokenKind::Reply,
            "Go" | "go" => TokenKind::Go,
            "DotView" | "dotview" => TokenKind::DotView,
            "DotMut" | "dotmut" => TokenKind::DotMut,
            "DotMove" | "dotmove" => TokenKind::DotMove,
            "DotTake" | "dottake" => TokenKind::DotTake,
            "And" | "and" => TokenKind::And,
            "Or" | "or" => TokenKind::Or,
            "FStrStart" | "fstrstart" => TokenKind::FStrStart,
            "FStrPart" | "fstrpart" => TokenKind::FStrPart,
            "FStrEnd" | "fstrend" => TokenKind::FStrEnd,
            "FStrNote" | "fstrnote" => TokenKind::FStrNote,
            "Grid" | "grid" => TokenKind::Grid,
            "EOF" | "eof" => TokenKind::EOF,
            _ => TokenKind::Int
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
struct Token {
    pub kind: TokenKind,
    pub pos: Pos,
    pub text: String,
}

fn is_keyword(mut kind: TokenKind) -> bool {
    match kind {
        TokenKind::True => true,
        TokenKind::False => true,
        TokenKind::Nil => true,
        TokenKind::Null => true,
        TokenKind::If => true,
        TokenKind::Else => true,
        TokenKind::For => true,
        TokenKind::Loop => true,
        TokenKind::Break => true,
        TokenKind::Continue => true,
        TokenKind::Return => true,
        TokenKind::Is => true,
        TokenKind::Fn => true,
        TokenKind::Type => true,
        TokenKind::Union => true,
        TokenKind::Tag => true,
        TokenKind::Let => true,
        TokenKind::Var => true,
        TokenKind::Mut => true,
        TokenKind::Move => true,
        TokenKind::Copy => true,
        TokenKind::Const => true,
        TokenKind::View => true,
        TokenKind::Take => true,
        TokenKind::Hold => true,
        TokenKind::Spec => true,
        TokenKind::Use => true,
        TokenKind::Pac => true,
        TokenKind::Super => true,
        TokenKind::As => true,
        TokenKind::To => true,
        TokenKind::Enum => true,
        TokenKind::On => true,
        TokenKind::Alias => true,
        TokenKind::Node => true,
        TokenKind::Ext => true,
        TokenKind::Static => true,
        TokenKind::Shared => true,
        TokenKind::Impl => true,
        TokenKind::Has => true,
        TokenKind::Dep => true,
        TokenKind::Routes => true,
        TokenKind::Outlet => true,
        TokenKind::Link => true,
        TokenKind::Route => true,
        TokenKind::Nav => true,
        TokenKind::NoneKW => true,
        TokenKind::SomeKW => true,
        TokenKind::OkKW => true,
        TokenKind::ErrKW => true,
        TokenKind::Task => true,
        TokenKind::Spawn => true,
        TokenKind::Await => true,
        TokenKind::Reply => true,
        TokenKind::Go => true,
        TokenKind::Grid => true,
        TokenKind::And => true,
        TokenKind::Or => true,
        _ => false,
    }
}

fn keyword_kind(mut text: &str) -> TokenKind {
    if text == "true" {
        return TokenKind::True;
    }
    if text == "false" {
        return TokenKind::False;
    }
    if text == "nil" {
        return TokenKind::Nil;
    }
    if text == "null" {
        return TokenKind::Null;
    }
    if text == "if" {
        return TokenKind::If;
    }
    if text == "else" {
        return TokenKind::Else;
    }
    if text == "for" {
        return TokenKind::For;
    }
    if text == "loop" {
        return TokenKind::Loop;
    }
    if text == "is" {
        return TokenKind::Is;
    }
    if text == "var" {
        return TokenKind::Var;
    }
    if text == "fn" {
        return TokenKind::Fn;
    }
    if text == "type" {
        return TokenKind::Type;
    }
    if text == "union" {
        return TokenKind::Union;
    }
    if text == "tag" {
        return TokenKind::Tag;
    }
    if text == "let" {
        return TokenKind::Let;
    }
    if text == "mut" {
        return TokenKind::Mut;
    }
    if text == "move" {
        return TokenKind::Move;
    }
    if text == "copy" {
        return TokenKind::Copy;
    }
    if text == "hold" {
        return TokenKind::Hold;
    }
    if text == "has" {
        return TokenKind::Has;
    }
    if text == "spec" {
        return TokenKind::Spec;
    }
    if text == "use" {
        return TokenKind::Use;
    }
    if text == "pac" {
        return TokenKind::Pac;
    }
    if text == "super" {
        return TokenKind::Super;
    }
    if text == "as" {
        return TokenKind::As;
    }
    if text == "to" {
        return TokenKind::To;
    }
    if text == "enum" {
        return TokenKind::Enum;
    }
    if text == "grid" {
        return TokenKind::Grid;
    }
    if text == "alias" {
        return TokenKind::Alias;
    }
    if text == "break" {
        return TokenKind::Break;
    }
    if text == "continue" {
        return TokenKind::Continue;
    }
    if text == "return" {
        return TokenKind::Return;
    }
    if text == "ext" {
        return TokenKind::Ext;
    }
    if text == "static" {
        return TokenKind::Static;
    }
    if text == "shared" {
        return TokenKind::Shared;
    }
    if text == "impl" {
        return TokenKind::Impl;
    }
    if text == "const" {
        return TokenKind::Const;
    }
    if text == "dep" {
        return TokenKind::Dep;
    }
    if text == "routes" {
        return TokenKind::Routes;
    }
    if text == "outlet" {
        return TokenKind::Outlet;
    }
    if text == "link" {
        return TokenKind::Link;
    }
    if text == "route" {
        return TokenKind::Route;
    }
    if text == "nav" {
        return TokenKind::Nav;
    }
    if text == "None" {
        return TokenKind::NoneKW;
    }
    if text == "Some" {
        return TokenKind::SomeKW;
    }
    if text == "Ok" {
        return TokenKind::OkKW;
    }
    if text == "Err" {
        return TokenKind::ErrKW;
    }
    if text == "task" {
        return TokenKind::Task;
    }
    if text == "spawn" {
        return TokenKind::Spawn;
    }
    if text == "await" {
        return TokenKind::Await;
    }
    if text == "reply" {
        return TokenKind::Reply;
    }
    if text == "go" {
        return TokenKind::Go;
    }
    if text == "on" {
        return TokenKind::On;
    }
    if text == "in" {
        return TokenKind::In;
    }
    if text == "node" {
        return TokenKind::Node;
    }
    if text == "view" {
        return TokenKind::View;
    }
    if text == "take" {
        return TokenKind::Take;
    }
    if text == "and" {
        return TokenKind::And;
    }
    if text == "or" {
        return TokenKind::Or;
    }
    return TokenKind::Ident;
}

// === opcode ===
// a2r Standard Library (from crate)

const OP_POP: i32 = 1;

const OP_DUP: i32 = 3;

const OP_RESERVE_STACK: i32 = 6;

const OP_CONST_I32: i32 = 16;

const OP_CONST_0: i32 = 18;

const OP_CONST_1: i32 = 19;

const OP_LOAD_STR: i32 = 31;

const OP_LOAD_LOCAL: i32 = 32;

const OP_STORE_LOCAL: i32 = 33;

const OP_ADD: i32 = 48;

const OP_SUB: i32 = 49;

const OP_MUL: i32 = 50;

const OP_DIV: i32 = 51;

const OP_MOD: i32 = 52;

const OP_EQ: i32 = 80;

const OP_NE: i32 = 81;

const OP_LT: i32 = 82;

const OP_GT: i32 = 83;

const OP_LE: i32 = 84;

const OP_GE: i32 = 85;

const OP_JMP: i32 = 96;

const OP_JMP_IF_Z: i32 = 97;

const OP_CALL: i32 = 112;

const OP_RET: i32 = 113;

const OP_CALL_NAT: i32 = 114;

const OP_LIST_NEW: i32 = 64;

const OP_LIST_PUSH: i32 = 65;

const OP_LIST_GET: i32 = 66;

const OP_LIST_LEN: i32 = 67;

const OP_MAP_NEW: i32 = 68;

const OP_MAP_INSERT_INT: i32 = 69;

const OP_MAP_GET_INT: i32 = 70;

const OP_MAP_CONTAINS: i32 = 71;

const OP_MAP_INSERT_STR: i32 = 72;

const OP_MAP_GET_STR: i32 = 73;

const OP_LIST_SET: i32 = 74;

const OP_LIST_POP: i32 = 75;

const OP_STR_LEN: i32 = 76;

const OP_STR_CHAR_AT: i32 = 77;

const OP_STR_SUBSTR: i32 = 78;

const OP_LIST_PUSH_STR: i32 = 79;

const OP_LIST_GET_STR: i32 = 86;

const OP_LIST_SET_STR: i32 = 87;

const OP_LIST_POP_STR: i32 = 88;

const OP_STR_CAT: i32 = 124;

const OP_HALT: i32 = 255;

const BOOL_TRUE: i32 = -2147483648;

const BOOL_FALSE: i32 = -2147483647;

const NATIVE_PRINT_I32: i32 = 1;

const NATIVE_PRINT_STR: i32 = 3;

// === ast ===
// a2r Standard Library (from crate)

#[derive(Clone, Copy, Debug, PartialEq)]
enum NodeKind {
    IntExpr = 0,
    StrExpr = 1,
    BoolExpr = 2,
    IdentExpr = 3,
    BinExpr = 4,
    UnaryExpr = 5,
    CallExpr = 6,
    DotExpr = 7,
    FnStmt = 8,
    LetStmt = 9,
    VarStmt = 10,
    ReturnStmt = 11,
    IfStmt = 12,
    ForStmt = 13,
    ForInStmt = 14,
    ExprStmt = 15,
    BlockStmt = 16,
    TypeStmt = 17,
    NilNode = 18,
    ClosureExpr = 19,
    FStrExpr = 20,
    IsStmt = 21,
    EnumStmt = 22,
    ExtStmt = 23,
    SpecStmt = 24,
    AliasStmt = 25,
    UseStmt = 26,
    ObjectExpr = 27,
    PairExpr = 28,
    ArrayExpr = 29,
    ErrorPropagateExpr = 30,
    ViewExpr = 31,
    MutExpr = 32,
    MoveExpr = 33,
    Param = 34,
}

impl std::fmt::Display for NodeKind {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            NodeKind::IntExpr => write!(f, "IntExpr"),
            NodeKind::StrExpr => write!(f, "StrExpr"),
            NodeKind::BoolExpr => write!(f, "BoolExpr"),
            NodeKind::IdentExpr => write!(f, "IdentExpr"),
            NodeKind::BinExpr => write!(f, "BinExpr"),
            NodeKind::UnaryExpr => write!(f, "UnaryExpr"),
            NodeKind::CallExpr => write!(f, "CallExpr"),
            NodeKind::DotExpr => write!(f, "DotExpr"),
            NodeKind::FnStmt => write!(f, "FnStmt"),
            NodeKind::LetStmt => write!(f, "LetStmt"),
            NodeKind::VarStmt => write!(f, "VarStmt"),
            NodeKind::ReturnStmt => write!(f, "ReturnStmt"),
            NodeKind::IfStmt => write!(f, "IfStmt"),
            NodeKind::ForStmt => write!(f, "ForStmt"),
            NodeKind::ForInStmt => write!(f, "ForInStmt"),
            NodeKind::ExprStmt => write!(f, "ExprStmt"),
            NodeKind::BlockStmt => write!(f, "BlockStmt"),
            NodeKind::TypeStmt => write!(f, "TypeStmt"),
            NodeKind::Param => write!(f, "Param"),
            NodeKind::NilNode => write!(f, "NilNode"),
            NodeKind::ClosureExpr => write!(f, "ClosureExpr"),
            NodeKind::FStrExpr => write!(f, "FStrExpr"),
            NodeKind::IsStmt => write!(f, "IsStmt"),
            NodeKind::EnumStmt => write!(f, "EnumStmt"),
            NodeKind::ExtStmt => write!(f, "ExtStmt"),
            NodeKind::SpecStmt => write!(f, "SpecStmt"),
            NodeKind::AliasStmt => write!(f, "AliasStmt"),
            NodeKind::UseStmt => write!(f, "UseStmt"),
            NodeKind::ObjectExpr => write!(f, "ObjectExpr"),
            NodeKind::PairExpr => write!(f, "PairExpr"),
            NodeKind::ArrayExpr => write!(f, "ArrayExpr"),
            NodeKind::ErrorPropagateExpr => write!(f, "ErrorPropagateExpr"),
            NodeKind::ViewExpr => write!(f, "ViewExpr"),
            NodeKind::MutExpr => write!(f, "MutExpr"),
            NodeKind::MoveExpr => write!(f, "MoveExpr"),
        }
    }
}
impl NodeKind {
    pub fn from_id(id: &str) -> Self {
        match id {
            "IntExpr" | "intexpr" => NodeKind::IntExpr,
            "StrExpr" | "strexpr" => NodeKind::StrExpr,
            "BoolExpr" | "boolexpr" => NodeKind::BoolExpr,
            "IdentExpr" | "identexpr" => NodeKind::IdentExpr,
            "BinExpr" | "binexpr" => NodeKind::BinExpr,
            "UnaryExpr" | "unaryexpr" => NodeKind::UnaryExpr,
            "CallExpr" | "callexpr" => NodeKind::CallExpr,
            "DotExpr" | "dotexpr" => NodeKind::DotExpr,
            "FnStmt" | "fnstmt" => NodeKind::FnStmt,
            "LetStmt" | "letstmt" => NodeKind::LetStmt,
            "VarStmt" | "varstmt" => NodeKind::VarStmt,
            "ReturnStmt" | "returnstmt" => NodeKind::ReturnStmt,
            "IfStmt" | "ifstmt" => NodeKind::IfStmt,
            "ForStmt" | "forstmt" => NodeKind::ForStmt,
            "ForInStmt" | "forinstmt" => NodeKind::ForInStmt,
            "ExprStmt" | "exprstmt" => NodeKind::ExprStmt,
            "BlockStmt" | "blockstmt" => NodeKind::BlockStmt,
            "TypeStmt" | "typestmt" => NodeKind::TypeStmt,
            "Param" | "param" => NodeKind::Param,
            "NilNode" | "nilnode" => NodeKind::NilNode,
            "ClosureExpr" | "closureexpr" => NodeKind::ClosureExpr,
            "FStrExpr" | "fstrexpr" => NodeKind::FStrExpr,
            "IsStmt" | "isstmt" => NodeKind::IsStmt,
            "EnumStmt" | "enumstmt" => NodeKind::EnumStmt,
            "ExtStmt" | "extstmt" => NodeKind::ExtStmt,
            "SpecStmt" | "specstmt" => NodeKind::SpecStmt,
            "AliasStmt" | "aliasstmt" => NodeKind::AliasStmt,
            "UseStmt" | "usestmt" => NodeKind::UseStmt,
            "ObjectExpr" | "objectexpr" => NodeKind::ObjectExpr,
            "PairExpr" | "pairexpr" => NodeKind::PairExpr,
            "ArrayExpr" | "arrayexpr" => NodeKind::ArrayExpr,
            "ErrorPropagateExpr" | "errorpropagateexpr" => NodeKind::ErrorPropagateExpr,
            "ViewExpr" | "viewexpr" => NodeKind::ViewExpr,
            "MutExpr" | "mutexpr" => NodeKind::MutExpr,
            "MoveExpr" | "moveexpr" => NodeKind::MoveExpr,
            _ => NodeKind::IntExpr
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Param {
    pub name: String,
    pub type_name: String,
}

#[derive(Clone, Debug, PartialEq)]
struct ASTNode {
    pub kind: NodeKind,
    pub value: String,
    pub name: String,
    pub children: Vec<ASTNode>,
    pub left: Vec<ASTNode>,
    pub right: Vec<ASTNode>,
    pub op: String,
    pub params: Vec<ASTNode>,
    pub type_name: String,
    pub cond: Vec<ASTNode>,
    pub else_body: Vec<ASTNode>,
}

fn empty_list() -> Vec<ASTNode> {
    Vec::new()
}

fn int_node(mut val: &str) -> ASTNode {
    ASTNode { kind: NodeKind::IntExpr, value: val.to_string(), name: val.to_string(), children: empty_list(), left: empty_list(), right: empty_list(), op: "".to_string(), params: empty_list(), type_name: "".to_string(), cond: empty_list(), else_body: empty_list() }
}

fn str_node(mut val: &str) -> ASTNode {
    ASTNode { kind: NodeKind::StrExpr, value: format!("{}{}", format!("{}{}", "\"", val), "\"").to_string(), name: val.to_string(), children: empty_list(), left: empty_list(), right: empty_list(), op: "".to_string(), params: empty_list(), type_name: "".to_string(), cond: empty_list(), else_body: empty_list() }
}

fn bool_node(mut val: &str) -> ASTNode {
    ASTNode { kind: NodeKind::BoolExpr, value: val.to_string(), name: val.to_string(), children: empty_list(), left: empty_list(), right: empty_list(), op: "".to_string(), params: empty_list(), type_name: "".to_string(), cond: empty_list(), else_body: empty_list() }
}

fn ident_node(mut name: &str) -> ASTNode {
    ASTNode { kind: NodeKind::IdentExpr, value: name.to_string(), name: name.to_string(), children: empty_list(), left: empty_list(), right: empty_list(), op: "".to_string(), params: empty_list(), type_name: "".to_string(), cond: empty_list(), else_body: empty_list() }
}

fn nil_node() -> ASTNode {
    ASTNode { kind: NodeKind::NilNode, value: "nil".to_string(), name: "".to_string(), children: empty_list(), left: empty_list(), right: empty_list(), op: "".to_string(), params: empty_list(), type_name: "".to_string(), cond: empty_list(), else_body: empty_list() }
}

fn bin_node(mut op: &str, mut left: ASTNode, mut right: ASTNode) -> ASTNode {
    let mut l = empty_list();
    l.push(left.clone());
    let mut r = empty_list();
    r.push(right.clone());
    let mut s: String = format!("{}{}", format!("{}{}", format!("{}{}", format!("{}{}", format!("{}{}", format!("{}{}", "(", op), " "), left.value), " "), right.value), ")");
    ASTNode { kind: NodeKind::BinExpr, value: s.to_string(), name: "".to_string(), children: empty_list(), left: l.clone(), right: r.clone(), op: op.to_string(), params: empty_list(), type_name: "".to_string(), cond: empty_list(), else_body: empty_list() }
}

fn unary_node(mut op: &str, mut operand: ASTNode) -> ASTNode {
    let mut l = empty_list();
    l.push(operand.clone());
    let mut s: String = format!("{}{}", format!("{}{}", format!("{}{}", format!("{}{}", "(", op), " "), operand.value), ")");
    ASTNode { kind: NodeKind::UnaryExpr, value: s.to_string(), name: "".to_string(), children: empty_list(), left: l.clone(), right: empty_list(), op: op.to_string(), params: empty_list(), type_name: "".to_string(), cond: empty_list(), else_body: empty_list() }
}

fn call_node(mut callee: ASTNode, mut args_str: &str, mut args: Vec<ASTNode>) -> ASTNode {
    let mut s: String = format!("{}{}", "(", callee.value);
    if args_str != "" {
        s = format!("{}{}", format!("{}{}", s, " "), args_str)
    }
    s = format!("{}{}", s, ")");
    let mut l = empty_list();
    l.push(callee.clone());
    ASTNode { kind: NodeKind::CallExpr, value: s.to_string(), name: callee.value.to_string(), children: empty_list(), left: l.clone(), right: empty_list(), op: "".to_string(), params: args.clone(), type_name: "".to_string(), cond: empty_list(), else_body: empty_list() }
}

fn dot_node(mut obj: ASTNode, mut field: &str) -> ASTNode {
    let mut l = empty_list();
    l.push(obj.clone());
    let mut s: String = format!("{}{}", format!("{}{}", obj.value, "."), field);
    ASTNode { kind: NodeKind::DotExpr, value: s.to_string(), name: field.to_string(), children: empty_list(), left: l.clone(), right: empty_list(), op: "".to_string(), params: empty_list(), type_name: "".to_string(), cond: empty_list(), else_body: empty_list() }
}

fn fn_node(mut name: &str, mut params_count: i32, mut ret: &str, mut body_str: &str) -> ASTNode {
    let mut s: String = format!("{}{}", format!("{}{}", "(fn ", name), " (params");
    let mut i: i32 = 0;
    while i < params_count {
        s = format!("{}{}", s, " param");
        i = i + 1;
    }
    s = format!("{}{}", s, ")");
    if ret != "" {
        s = format!("{}{}", format!("{}{}", s, " "), ret)
    }
    s = format!("{}{}", format!("{}{}", format!("{}{}", s, " (body "), body_str), "))");
    ASTNode { kind: NodeKind::FnStmt, value: s.to_string(), name: name.to_string(), children: empty_list(), left: empty_list(), right: empty_list(), op: "".to_string(), params: empty_list(), type_name: ret.to_string(), cond: empty_list(), else_body: empty_list() }
}

fn store_node(mut kind: NodeKind, mut name: &str, mut type_name: &str, mut expr: ASTNode) -> ASTNode {
    let mut l = empty_list();
    l.push(expr.clone());
    let mut prefix: String = "".to_string();
    match kind {
        NodeKind::LetStmt => prefix = "let".to_string(),
        NodeKind::VarStmt => prefix = "var".to_string(),
        _ => prefix = "store".to_string(),
    }
    let mut s: String = format!("{}{}", format!("{}{}", format!("{}{}", "(", prefix), " "), name);
    if type_name != "" {
        s = format!("{}{}", format!("{}{}", s, " "), type_name)
    }
    s = format!("{}{}", format!("{}{}", format!("{}{}", s, " "), expr.value), ")");
    ASTNode { kind: kind, value: s.to_string(), name: name.to_string(), children: empty_list(), left: l, right: empty_list(), op: "".to_string(), params: empty_list(), type_name: type_name.to_string(), cond: empty_list(), else_body: empty_list() }
}

fn return_node(mut expr: ASTNode) -> ASTNode {
    let mut l = empty_list();
    l.push(expr.clone());
    let mut s: String = format!("{}{}", format!("{}{}", "(return ", expr.value), ")");
    ASTNode { kind: NodeKind::ReturnStmt, value: s.to_string(), name: "".to_string(), children: empty_list(), left: l.clone(), right: empty_list(), op: "".to_string(), params: empty_list(), type_name: "".to_string(), cond: empty_list(), else_body: empty_list() }
}

fn return_node_empty() -> ASTNode {
    ASTNode { kind: NodeKind::ReturnStmt, value: "(return)".to_string(), name: "".to_string(), children: empty_list(), left: empty_list(), right: empty_list(), op: "".to_string(), params: empty_list(), type_name: "".to_string(), cond: empty_list(), else_body: empty_list() }
}

fn if_node(mut cond: ASTNode, mut body_str: &str, mut else_str: &str) -> ASTNode {
    let mut s: String = format!("{}{}", format!("{}{}", format!("{}{}", format!("{}{}", "(if ", cond.value), " (body "), body_str), "))");
    if else_str != "" {
        s = format!("{}{}", format!("{}{}", format!("{}{}", format!("{}{}", format!("{}{}", format!("{}{}", "(if ", cond.value), " (body "), body_str), ") (else "), else_str), "))")
    }
    let mut cl = empty_list();
    cl.push(cond.clone());
    ASTNode { kind: NodeKind::IfStmt, value: s.to_string(), name: "".to_string(), children: empty_list(), left: empty_list(), right: empty_list(), op: "".to_string(), params: empty_list(), type_name: "".to_string(), cond: cl.clone(), else_body: empty_list() }
}

fn forin_node(mut name: &str, mut range: ASTNode, mut body_str: &str) -> ASTNode {
    let mut s: String = format!("{}{}", format!("{}{}", format!("{}{}", format!("{}{}", format!("{}{}", format!("{}{}", "(for ", name), " in "), range.value), " (body "), body_str), "))");
    let mut rl = empty_list();
    rl.push(range);
    ASTNode { kind: NodeKind::ForInStmt, value: s.to_string(), name: name.to_string(), children: empty_list(), left: rl.clone(), right: empty_list(), op: "".to_string(), params: empty_list(), type_name: "".to_string(), cond: empty_list(), else_body: empty_list() }
}

fn for_node(mut cond: ASTNode, mut body_str: &str) -> ASTNode {
    let mut s: String = format!("{}{}", format!("{}{}", format!("{}{}", format!("{}{}", "(for ", cond.value), " (body "), body_str), "))");
    let mut cl = empty_list();
    cl.push(cond.clone());
    ASTNode { kind: NodeKind::ForStmt, value: s.to_string(), name: "".to_string(), children: empty_list(), left: cl.clone(), right: empty_list(), op: "".to_string(), params: empty_list(), type_name: "".to_string(), cond: empty_list(), else_body: empty_list() }
}

fn expr_stmt_node(mut expr: ASTNode) -> ASTNode {
    let mut l = empty_list();
    l.push(expr.clone());
    ASTNode { kind: NodeKind::ExprStmt, value: expr.value.to_string(), name: "".to_string(), children: empty_list(), left: l.clone(), right: empty_list(), op: "".to_string(), params: empty_list(), type_name: "".to_string(), cond: empty_list(), else_body: empty_list() }
}

fn block_node(mut body_str: &str) -> ASTNode {
    ASTNode { kind: NodeKind::BlockStmt, value: format!("{}{}", format!("{}{}", "(block ", body_str), ")").to_string(), name: "".to_string(), children: empty_list(), left: empty_list(), right: empty_list(), op: "".to_string(), params: empty_list(), type_name: "".to_string(), cond: empty_list(), else_body: empty_list() }
}

fn type_node(mut name: &str, mut fields: Vec<ASTNode>) -> ASTNode {
    let mut s: String = format!("{}{}", "(type ", name);
    let mut i: i32 = 0;
    let mut n = (fields.len() as i32);
    while i < n {
        let mut f = fields[i as usize].clone();
        s = format!("{}{}", format!("{}{}", format!("{}{}", format!("{}{}", s, " "), f.name), ":"), f.type_name);
        i = i + 1;
    }
    s = format!("{}{}", s, ")");
    ASTNode { kind: NodeKind::TypeStmt, value: s.to_string(), name: name.to_string(), children: empty_list(), left: empty_list(), right: empty_list(), op: "".to_string(), params: fields.clone(), type_name: "".to_string(), cond: empty_list(), else_body: empty_list() }
}

fn closure_node(mut params_str: &str, mut params: Vec<ASTNode>, mut body: ASTNode) -> ASTNode {
    let mut s: String = format!("{}{}", format!("{}{}", format!("{}{}", format!("{}{}", "(closure (", params_str), ") => "), body.value), ")");
    let mut bl = empty_list();
    bl.push(body);
    ASTNode { kind: NodeKind::ClosureExpr, value: s.to_string(), name: "".to_string(), children: empty_list(), left: bl.clone(), right: empty_list(), op: "".to_string(), params: params.clone(), type_name: "".to_string(), cond: empty_list(), else_body: empty_list() }
}

fn fstr_node(mut parts_str: &str, mut parts: Vec<ASTNode>) -> ASTNode {
    let mut s: String = format!("{}{}", format!("{}{}", "(fstr ", parts_str), ")");
    ASTNode { kind: NodeKind::FStrExpr, value: s.to_string(), name: "".to_string(), children: parts.clone(), left: empty_list(), right: empty_list(), op: "".to_string(), params: empty_list(), type_name: "".to_string(), cond: empty_list(), else_body: empty_list() }
}

fn is_node(mut expr: ASTNode, mut branches: Vec<ASTNode>, mut else_branch: Vec<ASTNode>) -> ASTNode {
    let mut branches_str: String = "".to_string();
    let mut i: i32 = 0;
    let mut n = (branches.len() as i32);
    while i < n {
        if branches_str != "" {
            branches_str = format!("{}{}", branches_str, " ")
        }
        branches_str = format!("{}{}", branches_str, branches[i as usize].clone().value);
        i = i + 1;
    }
    if (else_branch.len() as i32) > 0 {
        branches_str = format!("{}{}", format!("{}{}", format!("{}{}", branches_str, " (else "), else_branch[0 as usize].clone().value), ")")
    }
    let mut s: String = format!("{}{}", format!("{}{}", format!("{}{}", format!("{}{}", "(is ", expr.value), " "), branches_str), ")");
    let mut el = empty_list();
    el.push(expr);
    ASTNode { kind: NodeKind::IsStmt, value: s.to_string(), name: "".to_string(), children: branches.clone(), left: el.clone(), right: empty_list(), op: "".to_string(), params: empty_list(), type_name: "".to_string(), cond: empty_list(), else_body: else_branch }
}

fn enum_node(mut name: &str, mut variants: Vec<ASTNode>, mut repr_type: &str) -> ASTNode {
    let mut s: String = format!("{}{}", "(enum ", name);
    if repr_type != "" {
        s = format!("{}{}", format!("{}{}", s, " "), repr_type)
    }
    let mut i: i32 = 0;
    let mut n = (variants.len() as i32);
    while i < n {
        s = format!("{}{}", format!("{}{}", s, " "), variants[i as usize].clone().name);
        i = i + 1;
    }
    s = format!("{}{}", s, ")");
    ASTNode { kind: NodeKind::EnumStmt, value: s.to_string(), name: name.to_string(), children: empty_list(), left: empty_list(), right: empty_list(), op: "".to_string(), params: variants.clone(), type_name: repr_type.to_string(), cond: empty_list(), else_body: empty_list() }
}

fn ext_node(mut target: &str, mut methods: Vec<ASTNode>, mut for_spec: &str) -> ASTNode {
    let mut methods_str: String = "".to_string();
    let mut i: i32 = 0;
    let mut n = (methods.len() as i32);
    while i < n {
        if methods_str != "" {
            methods_str = format!("{}{}", methods_str, " ")
        }
        methods_str = format!("{}{}", methods_str, methods[i as usize].clone().value);
        i = i + 1;
    }
    let mut s: String = format!("{}{}", "(ext ", target);
    if for_spec != "" {
        s = format!("{}{}", format!("{}{}", s, " for "), for_spec)
    }
    s = format!("{}{}", format!("{}{}", format!("{}{}", s, " "), methods_str), ")");
    ASTNode { kind: NodeKind::ExtStmt, value: s.to_string(), name: target.to_string(), children: methods.clone(), left: empty_list(), right: empty_list(), op: for_spec.to_string(), params: empty_list(), type_name: "".to_string(), cond: empty_list(), else_body: empty_list() }
}

fn spec_node(mut name: &str, mut methods: Vec<ASTNode>) -> ASTNode {
    let mut methods_str: String = "".to_string();
    let mut i: i32 = 0;
    let mut n = (methods.len() as i32);
    while i < n {
        if methods_str != "" {
            methods_str = format!("{}{}", methods_str, " ")
        }
        methods_str = format!("{}{}", methods_str, methods[i as usize].clone().name);
        i = i + 1;
    }
    let mut s: String = format!("{}{}", format!("{}{}", format!("{}{}", format!("{}{}", "(spec ", name), " "), methods_str), ")");
    ASTNode { kind: NodeKind::SpecStmt, value: s.to_string(), name: name.to_string(), children: empty_list(), left: empty_list(), right: empty_list(), op: "".to_string(), params: methods.clone(), type_name: "".to_string(), cond: empty_list(), else_body: empty_list() }
}

fn alias_node(mut name: &str, mut target: &str) -> ASTNode {
    let mut s: String = format!("{}{}", format!("{}{}", format!("{}{}", format!("{}{}", "(alias ", name), " "), target), ")");
    ASTNode { kind: NodeKind::AliasStmt, value: s.to_string(), name: name.to_string(), children: empty_list(), left: empty_list(), right: empty_list(), op: "".to_string(), params: empty_list(), type_name: target.to_string(), cond: empty_list(), else_body: empty_list() }
}

fn use_node(mut path_str: &str, mut items_str: &str, mut items: Vec<ASTNode>) -> ASTNode {
    let mut s: String = format!("{}{}", "(use ", path_str);
    if items_str != "" {
        s = format!("{}{}", format!("{}{}", s, " "), items_str)
    }
    s = format!("{}{}", s, ")");
    ASTNode { kind: NodeKind::UseStmt, value: s.to_string(), name: path_str.to_string(), children: empty_list(), left: empty_list(), right: empty_list(), op: "".to_string(), params: items.clone(), type_name: "".to_string(), cond: empty_list(), else_body: empty_list() }
}

fn object_node(mut pairs_str: &str, mut pairs: Vec<ASTNode>) -> ASTNode {
    let mut s: String = format!("{}{}", format!("{}{}", "(object ", pairs_str), ")");
    ASTNode { kind: NodeKind::ObjectExpr, value: s.to_string(), name: "".to_string(), children: pairs.clone(), left: empty_list(), right: empty_list(), op: "".to_string(), params: empty_list(), type_name: "".to_string(), cond: empty_list(), else_body: empty_list() }
}

fn pair_node(mut key: &str, mut val: ASTNode) -> ASTNode {
    let mut s: String = format!("{}{}", format!("{}{}", format!("{}{}", format!("{}{}", "(pair ", key), " "), val.value), ")");
    let mut vl = empty_list();
    vl.push(val);
    ASTNode { kind: NodeKind::PairExpr, value: s.to_string(), name: key.to_string(), children: empty_list(), left: vl.clone(), right: empty_list(), op: "".to_string(), params: empty_list(), type_name: "".to_string(), cond: empty_list(), else_body: empty_list() }
}

fn array_node(mut elems: Vec<ASTNode>) -> ASTNode {
    let mut s: String = "(array".to_string();
    let mut i: i32 = 0;
    let mut n = (elems.len() as i32);
    while i < n {
        s = format!("{}{}", format!("{}{}", s, " "), elems[i as usize].clone().value);
        i = i + 1;
    }
    s = format!("{}{}", s, ")");
    ASTNode { kind: NodeKind::ArrayExpr, value: s.to_string(), name: "".to_string(), children: empty_list(), left: empty_list(), right: empty_list(), op: "".to_string(), params: elems.clone(), type_name: "".to_string(), cond: empty_list(), else_body: empty_list() }
}

fn error_propagate_node(mut expr: ASTNode) -> ASTNode {
    let mut l = empty_list();
    l.push(expr.clone());
    let mut s: String = format!("{}{}", format!("{}{}", "(? ", expr.value), ")");
    ASTNode { kind: NodeKind::ErrorPropagateExpr, value: s.to_string(), name: "".to_string(), children: empty_list(), left: l.clone(), right: empty_list(), op: "".to_string(), params: empty_list(), type_name: "".to_string(), cond: empty_list(), else_body: empty_list() }
}

fn view_node(mut expr: ASTNode) -> ASTNode {
    let mut l = empty_list();
    l.push(expr.clone());
    let mut s: String = format!("{}{}", format!("{}{}", "(& ", expr.value), ")");
    ASTNode { kind: NodeKind::ViewExpr, value: s.to_string(), name: "".to_string(), children: empty_list(), left: l.clone(), right: empty_list(), op: "".to_string(), params: empty_list(), type_name: "".to_string(), cond: empty_list(), else_body: empty_list() }
}

fn mut_node(mut expr: ASTNode) -> ASTNode {
    let mut l = empty_list();
    l.push(expr.clone());
    let mut s: String = format!("{}{}", format!("{}{}", "(&mut ", expr.value), ")");
    ASTNode { kind: NodeKind::MutExpr, value: s.to_string(), name: "".to_string(), children: empty_list(), left: l.clone(), right: empty_list(), op: "".to_string(), params: empty_list(), type_name: "".to_string(), cond: empty_list(), else_body: empty_list() }
}

fn move_node(mut expr: ASTNode) -> ASTNode {
    let mut l = empty_list();
    l.push(expr.clone());
    let mut s: String = format!("{}{}", format!("{}{}", "(move ", expr.value), ")");
    ASTNode { kind: NodeKind::MoveExpr, value: s.to_string(), name: "".to_string(), children: empty_list(), left: l.clone(), right: empty_list(), op: "".to_string(), params: empty_list(), type_name: "".to_string(), cond: empty_list(), else_body: empty_list() }
}

fn ast_to_string(mut node: ASTNode) -> String {
    node.value
}

// === lexer ===
// a2r Standard Library (from crate)

fn is_digit(mut c: i32) -> bool {
    c >= ('0' as i32) && c <= ('9' as i32)
}

fn is_alpha(mut c: i32) -> bool {
    c >= ('a' as i32) && c <= ('z' as i32) || c >= ('A' as i32) && c <= ('Z' as i32) || c == ('_' as i32)
}

fn is_alnum(mut c: i32) -> bool {
    is_digit(c) || is_alpha(c)
}

fn is_hex_digit(mut c: i32) -> bool {
    is_digit(c) || c >= ('a' as i32) && c <= ('f' as i32) || c >= ('A' as i32) && c <= ('F' as i32)
}

fn cur_char(mut source: &str, mut pos: i32) -> i32 {
    if pos >= (source.len() as i32) {
        return 0;
    }
    return source.chars().nth(pos as usize).unwrap_or('\0') as i32;
}

fn skip_ws(mut source: &str, mut pos: i32) -> i32 {
    let mut p: i32 = pos;
    let mut c = cur_char(&(source), p);
    while c == (' ' as i32) || c == ('\t' as i32) {
        p = p + 1;
        c = cur_char(&(source), p);
    }
    return p;
}

fn peek_char(mut source: &str, mut pos: i32, mut expected: i32) -> bool {
    cur_char(&(source), pos + 1) == expected
}

fn lex_number(mut source: &str, mut pos: i32) -> Token {
    let mut p: i32 = pos;
    let mut start: i32 = p;
    let mut text: String = "".to_string();
    let mut has_dot: bool = false;
    let mut is_hex: bool = false;
    let mut is_binary: bool = false;
    let mut c = cur_char(&(source), p);


    if c == ('0' as i32) {
        text = format!("{}{}", text, "0");
        p = p + 1;
        c = cur_char(&(source), p);
        if c == ('x' as i32) {
            text = format!("{}{}", text, "x");
            p = p + 1;
            is_hex = true;
            c = cur_char(&(source), p)
        } else if c == ('b' as i32) {
            text = format!("{}{}", text, "b");
            p = p + 1;
            is_binary = true;
            c = cur_char(&(source), p)
        }    }


    loop {
        if is_binary && c == ('0' as i32) || c == ('1' as i32) {
            text = format!("{}{}", text, source[(p) as usize..(p + 1) as usize].to_string());
            p = p + 1;
            c = cur_char(&(source), p)
        } else if is_hex && is_hex_digit(c) {
            text = format!("{}{}", text, source[(p) as usize..(p + 1) as usize].to_string());
            p = p + 1;
            c = cur_char(&(source), p)
        } else if is_digit(c) && !(is_binary) {
            text = format!("{}{}", text, source[(p) as usize..(p + 1) as usize].to_string());
            p = p + 1;
            c = cur_char(&(source), p)
        } else if c == ('_' as i32) {
            p = p + 1;
            c = cur_char(&(source), p)
        } else if c == ('.' as i32) && !(has_dot) && !(is_hex) && !(is_binary) {
            

            let mut next = cur_char(&(source), p + 1);
            if !(is_digit(next)) {
                break;
            }            has_dot = true;
            text = format!("{}{}", text, ".");
            p = p + 1;
            c = cur_char(&(source), p)
        } else {
            break;
        }

    }


    let mut kind = TokenKind::Int;
    if has_dot {
        kind = TokenKind::Float;
        let mut next = cur_char(&(source), p);
        if next == ('f' as i32) {
            p = p + 1;
            kind = TokenKind::Float
        } else if next == ('d' as i32) {
            p = p + 1;
            kind = TokenKind::Double
        }    } else {
        let mut next = cur_char(&(source), p);
        if next == ('f' as i32) {
            p = p + 1;
            kind = TokenKind::Float
        } else if next == ('d' as i32) {
            p = p + 1;
            kind = TokenKind::Double
        } else if next == ('u' as i32) {
            p = p + 1;
            if peek_char(&(source), p, ('8' as i32)) {
                p = p + 1;
                kind = TokenKind::U8
            } else {
                kind = TokenKind::Uint
            }
        } else if next == ('i' as i32) {
            p = p + 1;
            if peek_char(&(source), p, ('8' as i32)) {
                p = p + 1;
                kind = TokenKind::I8
            }        }    }


    return Token { kind: kind, pos: Pos { line: 1, at: start, total: p - start }, text: text };
}

fn lex_ident(mut source: &str, mut pos: i32) -> Token {
    let mut p: i32 = pos;
    let mut start: i32 = p;
    let mut c = cur_char(&(source), p);
    while is_alnum(c) || c == ('-' as i32) {
        

        if c == ('-' as i32) {
            let mut next = cur_char(&(source), p + 1);
            if !(is_alpha(next)) {
                break;
            }        }
        p = p + 1;
        c = cur_char(&(source), p);
    }
    let mut text = source[(start) as usize..(p) as usize].to_string();
    let mut kind = keyword_kind(&(text));
    return Token { kind: kind, pos: Pos { line: 1, at: start, total: p - start }, text: text };
}

fn lex_string(mut source: &str, mut pos: i32) -> Token {
    let mut p: i32 = pos + 1;
    let mut text: String = "".to_string();
    let mut c = cur_char(&(source), p);
    while c != ('"' as i32) && c != 0 {
        if c == ('\\' as i32) {
            p = p + 1;
            let mut esc = cur_char(&(source), p);
            if esc == ('n' as i32) {
                text = format!("{}{}", text, "\n")
            } else if esc == ('t' as i32) {
                text = format!("{}{}", text, "\t")
            } else if esc == ('r' as i32) {
                text = format!("{}{}", text, "\r")
            } else if esc == ('0' as i32) {
                text = format!("{}{}", text, "\0")
            } else if esc == ('\\' as i32) {
                text = format!("{}{}", text, "\\")
            } else if esc == ('"' as i32) {
                text = format!("{}{}", text, "\"")
            } else {
                text = format!("{}{}", format!("{}{}", text, "\\"), source[(p) as usize..(p + 1) as usize].to_string())
            }
            p = p + 1
        } else {
            text = format!("{}{}", text, source[(p) as usize..(p + 1) as usize].to_string());
            p = p + 1
        }

        c = cur_char(&(source), p);
    }
    p = p + 1;
    return Token { kind: TokenKind::Str, pos: Pos { line: 1, at: pos, total: p - pos }, text: text };
}

fn lex_operator(mut source: &str, mut pos: i32) -> Token {
    let mut c = cur_char(&(source), pos);
    let mut p: i32 = pos;


    if c == ('(' as i32) {
        return Token { kind: TokenKind::LParen, pos: Pos { line: 1, at: p, total: 1 }, text: "(".to_string() };
    }
    if c == (')' as i32) {
        return Token { kind: TokenKind::RParen, pos: Pos { line: 1, at: p, total: 1 }, text: ")".to_string() };
    }
    if c == ('[' as i32) {
        return Token { kind: TokenKind::LSquare, pos: Pos { line: 1, at: p, total: 1 }, text: "[".to_string() };
    }
    if c == (']' as i32) {
        return Token { kind: TokenKind::RSquare, pos: Pos { line: 1, at: p, total: 1 }, text: "]".to_string() };
    }
    if c == ('{' as i32) {
        return Token { kind: TokenKind::LBrace, pos: Pos { line: 1, at: p, total: 1 }, text: "{".to_string() };
    }
    if c == ('}' as i32) {
        return Token { kind: TokenKind::RBrace, pos: Pos { line: 1, at: p, total: 1 }, text: "}".to_string() };
    }
    if c == (',' as i32) {
        return Token { kind: TokenKind::Comma, pos: Pos { line: 1, at: p, total: 1 }, text: ",".to_string() };
    }
    if c == (';' as i32) {
        return Token { kind: TokenKind::Semi, pos: Pos { line: 1, at: p, total: 1 }, text: ";".to_string() };
    }
    if c == (':' as i32) {
        return Token { kind: TokenKind::Colon, pos: Pos { line: 1, at: p, total: 1 }, text: ":".to_string() };
    }
    if c == ('@' as i32) {
        return Token { kind: TokenKind::At, pos: Pos { line: 1, at: p, total: 1 }, text: "@".to_string() };
    }
    if c == ('~' as i32) {
        return Token { kind: TokenKind::Tilde, pos: Pos { line: 1, at: p, total: 1 }, text: "~".to_string() };
    }
    if c == ('#' as i32) {
        return Token { kind: TokenKind::Hash, pos: Pos { line: 1, at: p, total: 1 }, text: "#".to_string() };
    }
    if c == ('`' as i32) {
        return Token { kind: TokenKind::FStrStart, pos: Pos { line: 1, at: p, total: 1 }, text: "`".to_string() };
    }


    if c == ('+' as i32) {
        if peek_char(&(source), p, ('=' as i32)) {
            return Token { kind: TokenKind::AddEq, pos: Pos { line: 1, at: p, total: 2 }, text: "+=".to_string() };
        }        return Token { kind: TokenKind::Add, pos: Pos { line: 1, at: p, total: 1 }, text: "+".to_string() };
    }
    if c == ('-' as i32) {
        if peek_char(&(source), p, ('>' as i32)) {
            return Token { kind: TokenKind::Arrow, pos: Pos { line: 1, at: p, total: 2 }, text: "->".to_string() };
        }        if peek_char(&(source), p, ('=' as i32)) {
            return Token { kind: TokenKind::SubEq, pos: Pos { line: 1, at: p, total: 2 }, text: "-=".to_string() };
        }        return Token { kind: TokenKind::Sub, pos: Pos { line: 1, at: p, total: 1 }, text: "-".to_string() };
    }
    if c == ('*' as i32) {
        if peek_char(&(source), p, ('=' as i32)) {
            return Token { kind: TokenKind::MulEq, pos: Pos { line: 1, at: p, total: 2 }, text: "*=".to_string() };
        }        return Token { kind: TokenKind::Star, pos: Pos { line: 1, at: p, total: 1 }, text: "*".to_string() };
    }
    if c == ('/' as i32) {
        if peek_char(&(source), p, ('/' as i32)) {
            

            return Token { kind: TokenKind::CommentLine, pos: Pos { line: 1, at: p, total: 2 }, text: "//".to_string() };
        }        if peek_char(&(source), p, ('=' as i32)) {
            return Token { kind: TokenKind::DivEq, pos: Pos { line: 1, at: p, total: 2 }, text: "/=".to_string() };
        }        return Token { kind: TokenKind::Div, pos: Pos { line: 1, at: p, total: 1 }, text: "/".to_string() };
    }
    if c == ('^' as i32) {
        if peek_char(&(source), p, ('=' as i32)) {
            return Token { kind: TokenKind::ModEq, pos: Pos { line: 1, at: p, total: 2 }, text: "^=".to_string() };
        }        return Token { kind: TokenKind::Mod, pos: Pos { line: 1, at: p, total: 1 }, text: "^".to_string() };
    }
    if c == ('%' as i32) {
        if peek_char(&(source), p, ('=' as i32)) {
            return Token { kind: TokenKind::ModEq, pos: Pos { line: 1, at: p, total: 2 }, text: "%=".to_string() };
        }        return Token { kind: TokenKind::Mod, pos: Pos { line: 1, at: p, total: 1 }, text: "%".to_string() };
    }
    if c == ('<' as i32) {
        if peek_char(&(source), p, ('=' as i32)) {
            return Token { kind: TokenKind::Le, pos: Pos { line: 1, at: p, total: 2 }, text: "<=".to_string() };
        }        return Token { kind: TokenKind::Lt, pos: Pos { line: 1, at: p, total: 1 }, text: "<".to_string() };
    }
    if c == ('>' as i32) {
        if peek_char(&(source), p, ('=' as i32)) {
            return Token { kind: TokenKind::Ge, pos: Pos { line: 1, at: p, total: 2 }, text: ">=".to_string() };
        }        return Token { kind: TokenKind::Gt, pos: Pos { line: 1, at: p, total: 1 }, text: ">".to_string() };
    }
    if c == ('=' as i32) {
        if peek_char(&(source), p, ('=' as i32)) {
            return Token { kind: TokenKind::Eq, pos: Pos { line: 1, at: p, total: 2 }, text: "==".to_string() };
        }        if peek_char(&(source), p, ('>' as i32)) {
            return Token { kind: TokenKind::DoubleArrow, pos: Pos { line: 1, at: p, total: 2 }, text: "=>".to_string() };
        }        return Token { kind: TokenKind::Asn, pos: Pos { line: 1, at: p, total: 1 }, text: "=".to_string() };
    }
    if c == ('!' as i32) {
        if peek_char(&(source), p, ('=' as i32)) {
            return Token { kind: TokenKind::Neq, pos: Pos { line: 1, at: p, total: 2 }, text: "!=".to_string() };
        }        return Token { kind: TokenKind::Not, pos: Pos { line: 1, at: p, total: 1 }, text: "!".to_string() };
    }
    if c == ('.' as i32) {
        if peek_char(&(source), p, ('.' as i32)) {
            if peek_char(&(source), p + 1, ('=' as i32)) {
                return Token { kind: TokenKind::RangeEq, pos: Pos { line: 1, at: p, total: 3 }, text: "..=".to_string() };
            }            return Token { kind: TokenKind::Range, pos: Pos { line: 1, at: p, total: 2 }, text: "..".to_string() };
        }        if peek_char(&(source), p, ('?' as i32)) {
            return Token { kind: TokenKind::DotQuest, pos: Pos { line: 1, at: p, total: 2 }, text: ".?".to_string() };
        }        

        let mut next4: String = "".to_string();
        let mut pi: i32 = p + 1;
        let mut pc: i32 = 0;
        while pc < 6 {
            if pi >= (source.len() as i32) {
                break;
            }
            let mut ch = source.chars().nth(pi as usize).unwrap_or('\0') as i32;
            if ch >= 97 && ch <= 122 || ch >= 65 && ch <= 90 || ch == 95 {
                next4 = format!("{}{}", next4, source[(pi) as usize..(pi + 1) as usize].to_string())
            } else {
                break;
            }

            pi = pi + 1;
            pc = pc + 1;
        }
        if next4 == "view" {
            return Token { kind: TokenKind::DotView, pos: Pos { line: 1, at: p, total: 5 }, text: ".view".to_string() };
        }        if next4 == "mut" {
            return Token { kind: TokenKind::DotMut, pos: Pos { line: 1, at: p, total: 4 }, text: ".mut".to_string() };
        }        if next4 == "move" {
            return Token { kind: TokenKind::DotMove, pos: Pos { line: 1, at: p, total: 5 }, text: ".move".to_string() };
        }        if next4 == "take" {
            return Token { kind: TokenKind::DotTake, pos: Pos { line: 1, at: p, total: 5 }, text: ".take".to_string() };
        }        return Token { kind: TokenKind::Dot, pos: Pos { line: 1, at: p, total: 1 }, text: ".".to_string() };
    }
    if c == ('?' as i32) {
        if peek_char(&(source), p, ('?' as i32)) {
            return Token { kind: TokenKind::QuestionQuestion, pos: Pos { line: 1, at: p, total: 2 }, text: "??".to_string() };
        }        return Token { kind: TokenKind::Question, pos: Pos { line: 1, at: p, total: 1 }, text: "?".to_string() };
    }
    if c == ('|' as i32) {
        if peek_char(&(source), p, ('|' as i32)) {
            return Token { kind: TokenKind::Or, pos: Pos { line: 1, at: p, total: 2 }, text: "||".to_string() };
        }        return Token { kind: TokenKind::VBar, pos: Pos { line: 1, at: p, total: 1 }, text: "|".to_string() };
    }
    if c == ('&' as i32) {
        if peek_char(&(source), p, ('&' as i32)) {
            return Token { kind: TokenKind::And, pos: Pos { line: 1, at: p, total: 2 }, text: "&&".to_string() };
        }        return Token { kind: TokenKind::Amp, pos: Pos { line: 1, at: p, total: 1 }, text: "&".to_string() };
    }


    return Token { kind: TokenKind::EOF, pos: Pos { line: 1, at: p, total: 1 }, text: "".to_string() };
}

fn tokenize(mut source: &str) -> String {
    let mut p: i32 = 0;
    let mut result: String = "".to_string();
    let mut len = (source.len() as i32);

    while p < len {
        p = skip_ws(&(source), p);
        if p >= len {
            break;
        }
        
        let mut c = cur_char(&(source), p);
        

        if c == ('\n' as i32) {
            result = format!("{}{}", result, "<nl>");
            p = p + 1
        } else if is_digit(c) {
            let mut tok = lex_number(&(source), p);
            result = format!("{}{}", result, tok.text);
            

            p = p + tok.pos.total
        } else if is_alpha(c) {
            let mut tok = lex_ident(&(source), p);
            if is_keyword(tok.kind) {
                result = format!("{}{}", format!("{}{}", format!("{}{}", result, "<"), tok.text), ">")
            } else {
                result = format!("{}{}", result, tok.text)
            }
            p = p + tok.pos.total
        } else if c == ('"' as i32) {
            let mut tok = lex_string(&(source), p);
            result = format!("{}{}", format!("{}{}", format!("{}{}", result, "\""), tok.text), "\"");
            p = p + tok.pos.total
        } else {
            let mut tok = lex_operator(&(source), p);
            if tok.kind != TokenKind::EOF {
                result = format!("{}{}", result, tok.text);
                p = p + tok.pos.total
            } else {
                p = p + 1
            }
        }

    }

    return result;
}

fn lex_fstr_backtick(mut source: &str, mut pos: i32) -> Vec<Token> {
    let mut p: i32 = pos + 1;
    let mut start: i32 = pos;
    let mut list = Vec::new();
    list.push(Token { kind: TokenKind::FStrStart, pos: Pos { line: 1, at: start, total: 1 }, text: "`".to_string() });

    let mut text: String = "".to_string();
    let mut c = cur_char(&(source), p);
    while c != ('`' as i32) && c != 0 {
        if c == ('$' as i32) {
            

            if text != "" {
                list.push(Token { kind: TokenKind::FStrPart, pos: Pos { line: 1, at: p - (text.len() as i32), total: (text.len() as i32) }, text: text });
                text = "".to_string()
            }            

            list.push(Token { kind: TokenKind::FStrNote, pos: Pos { line: 1, at: p, total: 1 }, text: "$".to_string() });
            p = p + 1;
            c = cur_char(&(source), p);
            

            if c == ('{' as i32) {
                

                p = p + 1;
                c = cur_char(&(source), p);
                

                let mut ident: String = "".to_string();
                while is_alnum(c) {
                    ident = format!("{}{}", ident, source[(p) as usize..(p + 1) as usize].to_string());
                    p = p + 1;
                    c = cur_char(&(source), p);
                }
                if ident != "" {
                    list.push(Token { kind: TokenKind::Ident, pos: Pos { line: 1, at: p - (ident.len() as i32), total: (ident.len() as i32) }, text: ident });
                }                

                if c == ('}' as i32) {
                    p = p + 1;
                    c = cur_char(&(source), p)
                }            } else if is_alpha(c) {
                

                let mut ident: String = "".to_string();
                while is_alnum(c) {
                    ident = format!("{}{}", ident, source[(p) as usize..(p + 1) as usize].to_string());
                    p = p + 1;
                    c = cur_char(&(source), p);
                }
                list.push(Token { kind: TokenKind::Ident, pos: Pos { line: 1, at: p - (ident.len() as i32), total: (ident.len() as i32) }, text: ident });
            }        } else {
            text = format!("{}{}", text, source[(p) as usize..(p + 1) as usize].to_string());
            p = p + 1;
            c = cur_char(&(source), p)
        }

    }


    if text != "" {
        list.push(Token { kind: TokenKind::FStrPart, pos: Pos { line: 1, at: p - (text.len() as i32), total: (text.len() as i32) }, text: text });
    }


    p = p + 1;
    list.push(Token { kind: TokenKind::FStrEnd, pos: Pos { line: 1, at: p - 1, total: 1 }, text: "`".to_string() });


    let mut total_len: i32 = p - start;
    list.insert(0 as usize, Token { kind: TokenKind::FStrStart, pos: Pos { line: 1, at: start, total: total_len }, text: "`".to_string() });

    return list;
}

fn lex_fstr_f(mut source: &str, mut pos: i32) -> Vec<Token> {
    let mut p: i32 = pos + 2;
    let mut start: i32 = pos;
    let mut list = Vec::new();
    list.push(Token { kind: TokenKind::FStrStart, pos: Pos { line: 1, at: start, total: 2 }, text: "f\"".to_string() });

    let mut text: String = "".to_string();
    let mut c = cur_char(&(source), p);
    while c != ('"' as i32) && c != 0 {
        if c == ('$' as i32) {
            

            if text != "" {
                list.push(Token { kind: TokenKind::FStrPart, pos: Pos { line: 1, at: p - (text.len() as i32), total: (text.len() as i32) }, text: text });
                text = "".to_string()
            }            

            list.push(Token { kind: TokenKind::FStrNote, pos: Pos { line: 1, at: p, total: 1 }, text: "$".to_string() });
            p = p + 1;
            c = cur_char(&(source), p);
            if c == ('{' as i32) {
                p = p + 1;
                c = cur_char(&(source), p);
                let mut ident: String = "".to_string();
                while is_alnum(c) {
                    ident = format!("{}{}", ident, source[(p) as usize..(p + 1) as usize].to_string());
                    p = p + 1;
                    c = cur_char(&(source), p);
                }
                if ident != "" {
                    list.push(Token { kind: TokenKind::Ident, pos: Pos { line: 1, at: p - (ident.len() as i32), total: (ident.len() as i32) }, text: ident });
                }                if c == ('}' as i32) {
                    p = p + 1;
                    c = cur_char(&(source), p)
                }            } else if is_alpha(c) {
                let mut ident: String = "".to_string();
                while is_alnum(c) {
                    ident = format!("{}{}", ident, source[(p) as usize..(p + 1) as usize].to_string());
                    p = p + 1;
                    c = cur_char(&(source), p);
                }
                list.push(Token { kind: TokenKind::Ident, pos: Pos { line: 1, at: p - (ident.len() as i32), total: (ident.len() as i32) }, text: ident });
            }        } else {
            text = format!("{}{}", text, source[(p) as usize..(p + 1) as usize].to_string());
            p = p + 1;
            c = cur_char(&(source), p)
        }

    }

    if text != "" {
        list.push(Token { kind: TokenKind::FStrPart, pos: Pos { line: 1, at: p - (text.len() as i32), total: (text.len() as i32) }, text: text });
    }

    p = p + 1;
    list.push(Token { kind: TokenKind::FStrEnd, pos: Pos { line: 1, at: p - 1, total: 1 }, text: "\"".to_string() });

    let mut total_len: i32 = p - start;
    list.insert(0 as usize, Token { kind: TokenKind::FStrStart, pos: Pos { line: 1, at: start, total: total_len }, text: "f\"".to_string() });

    return list;
}

fn tokenize_list(mut source: &str) -> Vec<Token> {
    let mut p: i32 = 0;
    let mut list = Vec::new();
    let mut len = (source.len() as i32);

    while p < len {
        p = skip_ws(&(source), p);
        if p >= len {
            break;
        }
        
        let mut c = cur_char(&(source), p);
        
        if c == ('\n' as i32) {
            list.push(Token { kind: TokenKind::Newline, pos: Pos { line: 1, at: p, total: 1 }, text: "<nl>".to_string() });
            p = p + 1
        } else if is_digit(c) {
            let mut tok = lex_number(&(source), p);
            list.push(tok.clone());
            p = p + tok.pos.total
        } else if is_alpha(c) {
            

            if c == ('f' as i32) && peek_char(&(source), p, ('"' as i32)) {
                let mut toks = lex_fstr_f(&(source), p);
                let mut first_fstr = toks[0 as usize].clone();
                let mut ti: i32 = 0;
                while ti < (toks.len() as i32) {
                    list.push(toks[ti as usize].clone());
                    ti = ti + 1;
                }
                p = p + first_fstr.pos.total
            } else {
                let mut tok = lex_ident(&(source), p);
                list.push(tok.clone());
                p = p + tok.pos.total
            }
        } else if c == ('`' as i32) {
            

            let mut toks = lex_fstr_backtick(&(source), p);
            let mut first_fstr = toks[0 as usize].clone();
            let mut ti: i32 = 0;
            while ti < (toks.len() as i32) {
                list.push(toks[ti as usize].clone());
                ti = ti + 1;
            }
            p = p + first_fstr.pos.total
        } else if c == ('"' as i32) {
            let mut tok = lex_string(&(source), p);
            list.push(tok.clone());
            p = p + tok.pos.total
        } else {
            let mut tok = lex_operator(&(source), p);
            if tok.kind != TokenKind::EOF {
                list.push(tok.clone());
                p = p + tok.pos.total
            } else {
                p = p + 1
            }
        }

    }


    list.push(Token { kind: TokenKind::EOF, pos: Pos { line: 0, at: p, total: 0 }, text: "".to_string() });
    return list;
}

// === parser ===
// a2r Standard Library (from crate)

#[derive(Clone, Debug, PartialEq)]
struct Parser {
    pub tokens: Vec<Token>,
    pub pos: i32,
    pub token_count: i32,
}

fn parser_new(mut tokens: Vec<Token>) -> Parser {
    Parser { tokens: tokens.clone(), pos: 0, token_count: (tokens.len() as i32) }
}

fn parser_cur(mut p: Parser) -> Token {
    if p.pos >= p.token_count {
        return Token { kind: TokenKind::EOF, pos: Pos { line: 0, at: 0, total: 0 }, text: "".to_string() };
    }
    return p.tokens[p.pos as usize].clone();
}

fn parser_kind(mut p: Parser) -> TokenKind {
    return parser_cur(p.clone()).kind;
}

fn parser_text(mut p: Parser) -> String {
    return parser_cur(p.clone()).text;
}

fn parser_is(mut p: Parser, mut kind: TokenKind) -> bool {
    return parser_kind(p.clone()) == kind;
}

fn parser_peek_kind(mut p: Parser) -> TokenKind {
    let mut next: i32 = p.pos + 1;
    if next >= p.token_count {
        return TokenKind::EOF;
    }
    return p.tokens[next as usize].clone().kind;
}

fn parser_skip_newlines(mut p: Parser) {
    while parser_kind(p.clone()) == TokenKind::Newline {
        p.pos = p.pos + 1;
    }
}

fn parser_skip_semi(mut p: Parser) {
    while parser_kind(p.clone()) == TokenKind::Semi {
        p.pos = p.pos + 1;
    }
}

fn parser_skip_nl_semi(mut p: Parser) {
    loop {
        let mut k = parser_kind(p.clone());
        if k == TokenKind::Newline || k == TokenKind::Semi {
            p.pos = p.pos + 1
        } else {
            break;
        }

    }
}

fn infix_prec(mut kind: TokenKind) -> i32 {
    match kind {
        TokenKind::Asn => 1,
        TokenKind::AddEq => 1,
        TokenKind::SubEq => 1,
        TokenKind::MulEq => 1,
        TokenKind::DivEq => 1,
        TokenKind::Or => 2,
        TokenKind::And => 3,
        TokenKind::Eq => 4,
        TokenKind::Neq => 4,
        TokenKind::Lt => 5,
        TokenKind::Gt => 5,
        TokenKind::Le => 5,
        TokenKind::Ge => 5,
        TokenKind::Range => 5,
        TokenKind::RangeEq => 5,
        TokenKind::Add => 6,
        TokenKind::Sub => 6,
        TokenKind::Star => 7,
        TokenKind::Div => 7,
        TokenKind::Mod => 7,
        TokenKind::Dot => 8,
        TokenKind::DotQuest => 8,
        TokenKind::DotView => 8,
        TokenKind::DotMut => 8,
        TokenKind::DotMove => 8,
        TokenKind::DotTake => 8,
        TokenKind::LParen => 8,
        TokenKind::LSquare => 8,
        _ => 0,
    }
}

fn is_stop_token(mut kind: TokenKind) -> bool {
    match kind {
        TokenKind::EOF => true,
        TokenKind::Newline => true,
        TokenKind::Semi => true,
        TokenKind::RBrace => true,
        TokenKind::RParen => true,
        TokenKind::RSquare => true,
        TokenKind::Comma => true,
        TokenKind::Colon => true,
        TokenKind::Arrow => true,
        _ => false,
    }
}

fn parse_program(mut p: Parser) -> Vec<ASTNode> {
    let mut stmts = empty_list();
    parser_skip_nl_semi(p.clone());

    while parser_kind(p.clone()) != TokenKind::EOF {
        let mut stmt = parse_top_stmt(p.clone());
        if stmt.kind != NodeKind::NilNode {
            stmts.push(stmt);
        }
        parser_skip_nl_semi(p.clone());
    }
    return stmts;
}

fn parse_top_stmt(mut p: Parser) -> ASTNode {
    let mut kind = parser_kind(p.clone());
    match kind {
        TokenKind::Fn => parse_fn_decl(p.clone()),
        TokenKind::Type => parse_type_decl(p.clone()),
        TokenKind::Enum => parse_enum_decl(p.clone()),
        TokenKind::Tag => parse_enum_decl(p.clone()),
        TokenKind::Spec => parse_spec_decl(p.clone()),
        TokenKind::Ext => parse_ext_stmt(p.clone()),
        TokenKind::Alias => parse_alias_stmt(p.clone()),
        TokenKind::Use => parse_use_stmt(p.clone()),
        _ => parse_stmt(p.clone()),
    }
}

fn parse_body(mut p: Parser) -> Vec<ASTNode> {
    p.pos = p.pos + 1;
    parser_skip_nl_semi(p.clone());

    let mut stmts = empty_list();
    while parser_kind(p.clone()) != TokenKind::RBrace && parser_kind(p.clone()) != TokenKind::EOF {
        let mut stmt = parse_stmt(p.clone());
        if stmt.kind != NodeKind::NilNode {
            stmts.push(stmt);
        }
        parser_skip_nl_semi(p.clone());
    }

    if parser_kind(p.clone()) == TokenKind::RBrace {
        p.pos = p.pos + 1
    }
    return stmts;
}

fn body_to_str(mut body: Vec<ASTNode>) -> String {
    let mut result: String = "".to_string();
    let mut i: i32 = 0;
    let mut n = (body.len() as i32);
    while i < n {
        let mut stmt = body[i as usize].clone();
        if result != "" {
            result = format!("{}{}", result, " ")
        }
        result = format!("{}{}", result, stmt.value);
        i = i + 1;
    }
    return result;
}

fn parse_stmt(mut p: Parser) -> ASTNode {
    let mut kind = parser_kind(p.clone());
    match kind {
        TokenKind::Fn => parse_fn_decl(p.clone()),
        TokenKind::Let => parse_let_stmt(p.clone()),
        TokenKind::Var => parse_var_stmt(p.clone()),
        TokenKind::Return => parse_return_stmt(p.clone()),
        TokenKind::If => parse_if_stmt(p.clone()),
        TokenKind::For => parse_for_stmt(p.clone()),
        TokenKind::Is => parse_is_stmt(p.clone()),
        TokenKind::Break => { p.pos = p.pos + 1; nil_node() },
        TokenKind::Continue => { p.pos = p.pos + 1; nil_node() },
        TokenKind::LBrace => {
            


            let mut p1: i32 = p.pos + 1;
            if p1 < p.token_count {
                let mut t1 = p.tokens[p1 as usize].clone();
                if t1.kind == TokenKind::Ident {
                    let mut p2: i32 = p.pos + 2;
                    if p2 < p.token_count {
                        let mut t2 = p.tokens[p2 as usize].clone();
                        if t2.kind == TokenKind::Colon {
                            return parse_object(p.clone());
                        }                    }                }            }
            let mut body = parse_body(p.clone());
            let mut body_str = body_to_str(body.clone());
            let mut node = block_node(&(body_str));
            node.children = body;
            return node;
        },
        _ => parse_expr_stmt(p.clone()),
    }
}

fn parse_fn_decl(mut p: Parser) -> ASTNode {
    p.pos = p.pos + 1;
    let mut name = parser_text(p.clone());
    p.pos = p.pos + 1;


    let mut params = empty_list();
    let mut param_names: String = "".to_string();
    if parser_kind(p.clone()) == TokenKind::LParen {
        p.pos = p.pos + 1;
        parser_skip_nl_semi(p.clone());
        while parser_kind(p.clone()) != TokenKind::RParen && parser_kind(p.clone()) != TokenKind::EOF {
            let mut param_name = parser_text(p.clone());
            p.pos = p.pos + 1;
            
            let mut param_type: String = "".to_string();
            
            let mut pk = parser_kind(p.clone());
            if pk != TokenKind::RParen && pk != TokenKind::Comma && pk != TokenKind::Newline {
                param_type = parser_text(p.clone());
                p.pos = p.pos + 1;
                param_type = parser_try_read_generic_type(p.clone(), param_type.as_str())
            }
            params.push(ASTNode { kind: NodeKind::Param, name: param_name.clone(), type_name: param_type.clone(), value: "".to_string(), children: empty_list(), left: empty_list(), right: empty_list(), op: "".to_string(), params: empty_list(), cond: empty_list(), else_body: empty_list() });
            

            if param_names != "" {
                param_names = format!("{}{}", param_names, ",")
            }
            param_names = eval_str_cat(&(param_names), &(param_name));
            
            parser_skip_nl_semi(p.clone());
            if parser_kind(p.clone()) == TokenKind::Comma {
                p.pos = p.pos + 1;
                parser_skip_nl_semi(p.clone());
            }
        }
        if parser_kind(p.clone()) == TokenKind::RParen {
            p.pos = p.pos + 1
        }    }


    let mut ret_type: String = "".to_string();
    parser_skip_nl_semi(p.clone());
    let mut pk2 = parser_kind(p.clone());
    if pk2 == TokenKind::Arrow {
        p.pos = p.pos + 1;
        parser_skip_nl_semi(p.clone());
        ret_type = parser_text(p.clone());
        p.pos = p.pos + 1;
        ret_type = parser_try_read_generic_type(p.clone(), ret_type.as_str())
    } else if pk2 == TokenKind::Ident {
        


        let mut peek = parser_peek_kind(p.clone());
        if peek == TokenKind::LBrace {
            ret_type = parser_text(p.clone());
            p.pos = p.pos + 1;
            ret_type = parser_try_read_generic_type(p.clone(), ret_type.as_str())
        }    }

    parser_skip_nl_semi(p.clone());
    let mut body = parse_body(p.clone());
    let mut body_str = body_to_str(body.clone());
    let mut node = fn_node(&(name), (params.len() as i32), &(ret_type), &(body_str));
    node.children = body;
    node.params = params;
    node.op = param_names;
    return node;
}

fn parser_try_read_generic_type(mut p: Parser, mut type_name: &str) -> String {
    if parser_kind(p.clone()) != TokenKind::Lt {
        return type_name.to_string();
    }
    let mut result: String = format!("{}{}", type_name, "<");
    p.pos = p.pos + 1;
    parser_skip_nl_semi(p.clone());
    let mut depth: i32 = 1;
    while depth > 0 && parser_kind(p.clone()) != TokenKind::EOF {
        let mut kind = parser_kind(p.clone());
        if kind == TokenKind::Lt {
            depth = depth + 1;
            result = format!("{}{}", result, "<")
        } else if kind == TokenKind::Gt {
            depth = depth - 1;
            if depth > 0 {
                result = format!("{}{}", result, ">")
            }        } else if kind == TokenKind::Comma {
            result = format!("{}{}", result, ", ")
        } else {
            result = format!("{}{}", result, parser_text(p.clone()))
        }

        p.pos = p.pos + 1;
        parser_skip_nl_semi(p.clone());
    }
    result = format!("{}{}", result, ">");
    return result;
}

fn parse_let_stmt(mut p: Parser) -> ASTNode {
    p.pos = p.pos + 1;
    let mut name = parser_text(p.clone());
    p.pos = p.pos + 1;


    let mut type_name: String = "".to_string();
    let mut pk = parser_kind(p.clone());
    if pk == TokenKind::Ident {
        

        let mut peek = parser_peek_kind(p.clone());
        if peek != TokenKind::Asn {
            type_name = parser_text(p.clone());
            p.pos = p.pos + 1;
            type_name = parser_try_read_generic_type(p.clone(), type_name.as_str())
        }    }


    if parser_kind(p.clone()) == TokenKind::Asn {
        p.pos = p.pos + 1
    }

    let mut expr = parse_expr(p.clone());
    return store_node(NodeKind::LetStmt, &(name), &(type_name), expr.clone());
}

fn parse_var_stmt(mut p: Parser) -> ASTNode {
    p.pos = p.pos + 1;
    let mut name = parser_text(p.clone());
    p.pos = p.pos + 1;


    let mut type_name: String = "".to_string();
    if parser_kind(p.clone()) == TokenKind::Ident {
        let mut peek = parser_peek_kind(p.clone());
        if peek != TokenKind::Asn {
            type_name = parser_text(p.clone());
            p.pos = p.pos + 1;
            type_name = parser_try_read_generic_type(p.clone(), type_name.as_str())
        }    }


    if parser_kind(p.clone()) == TokenKind::Uint {
        type_name = "uint".to_string();
        p.pos = p.pos + 1
    }

    if parser_kind(p.clone()) == TokenKind::Asn {
        p.pos = p.pos + 1
    }

    let mut expr = parse_expr(p.clone());
    return store_node(NodeKind::VarStmt, &(name), &(type_name), expr.clone());
}

fn parse_return_stmt(mut p: Parser) -> ASTNode {
    p.pos = p.pos + 1;


    let mut k = parser_kind(p.clone());
    if k == TokenKind::Newline || k == TokenKind::RBrace || k == TokenKind::Semi || k == TokenKind::EOF {
        return return_node_empty();
    }

    let mut expr = parse_expr(p.clone());
    return return_node(expr.clone());
}

fn parse_if_stmt(mut p: Parser) -> ASTNode {
    p.pos = p.pos + 1;
    parser_skip_nl_semi(p.clone());

    let mut cond = parse_expr(p.clone());
    parser_skip_nl_semi(p.clone());
    let mut body = parse_body(p.clone());
    let mut body_str = body_to_str(body.clone());

    let mut else_str: String = "".to_string();
    let mut else_body = empty_list();
    parser_skip_nl_semi(p.clone());
    if parser_kind(p.clone()) == TokenKind::Else {
        p.pos = p.pos + 1;
        parser_skip_nl_semi(p.clone());
        if parser_kind(p.clone()) == TokenKind::If {
            let mut else_if = parse_if_stmt(p.clone());
            else_str = else_if.value.clone();
            else_body.push(else_if)
        } else {
            else_body = parse_body(p.clone());
            else_str = body_to_str(else_body.clone())
        }
    }

    let mut node = if_node(cond.clone(), &(body_str), &(else_str));
    node.children = body;
    node.else_body = else_body;
    return node;
}

fn parse_for_stmt(mut p: Parser) -> ASTNode {
    p.pos = p.pos + 1;


    if parser_kind(p.clone()) == TokenKind::LBrace {
        let mut body = parse_body(p.clone());
        let mut body_str = body_to_str(body.clone());
        let mut node = for_node(nil_node(), &(body_str));
        node.children = body;
        return node;
    }


    if parser_kind(p.clone()) == TokenKind::Ident {
        let mut name = parser_text(p.clone());
        let mut pk = parser_peek_kind(p.clone());
        if pk == TokenKind::In {
            p.pos = p.pos + 1;
            p.pos = p.pos + 1;
            parser_skip_nl_semi(p.clone());
            let mut range = parse_expr(p.clone());
            parser_skip_nl_semi(p.clone());
            let mut body = parse_body(p.clone());
            let mut body_str = body_to_str(body.clone());
            let mut node = forin_node(&(name), range.clone(), &(body_str));
            node.children = body;
            return node;
        }    }


    let mut cond = parse_expr(p.clone());
    parser_skip_nl_semi(p.clone());
    let mut body = parse_body(p.clone());
    let mut body_str = body_to_str(body.clone());
    let mut node = for_node(cond.clone(), &(body_str));
    node.children = body;
    return node;
}

fn parse_type_decl(mut p: Parser) -> ASTNode {
    p.pos = p.pos + 1;
    let mut name = parser_text(p.clone());
    p.pos = p.pos + 1;
    parser_skip_nl_semi(p.clone());

    let mut fields = empty_list();
    if parser_kind(p.clone()) == TokenKind::LBrace {
        p.pos = p.pos + 1;
        parser_skip_nl_semi(p.clone());
        while parser_kind(p.clone()) != TokenKind::RBrace && parser_kind(p.clone()) != TokenKind::EOF {
            if parser_kind(p.clone()) == TokenKind::Fn {
                

                let mut depth: i32 = 1;
                p.pos = p.pos + 1;
                while depth > 0 && parser_kind(p.clone()) != TokenKind::EOF {
                    let mut k = parser_kind(p.clone());
                    if k == TokenKind::LBrace {
                        depth = depth + 1
                    } else if k == TokenKind::RBrace {
                        depth = depth - 1
                    }
                    p.pos = p.pos + 1;
                }
            } else if parser_kind(p.clone()) == TokenKind::Ident {
                let mut field_name = parser_text(p.clone());
                p.pos = p.pos + 1;
                let mut field_type: String = "".to_string();
                if parser_kind(p.clone()) == TokenKind::Ident {
                    field_type = parser_text(p.clone());
                    p.pos = p.pos + 1;
                    field_type = parser_try_read_generic_type(p.clone(), field_type.as_str())
                }                fields.push(ASTNode { kind: NodeKind::Param, name: field_name.clone(), type_name: field_type.clone(), value: "".to_string(), children: empty_list(), left: empty_list(), right: empty_list(), op: "".to_string(), params: empty_list(), cond: empty_list(), else_body: empty_list() })
            } else {
                p.pos = p.pos + 1
            }

            parser_skip_nl_semi(p.clone());
        }
        if parser_kind(p.clone()) == TokenKind::RBrace {
            p.pos = p.pos + 1
        }    }

    return type_node(&(name), fields.clone());
}

fn parse_expr_stmt(mut p: Parser) -> ASTNode {
    let mut expr = parse_expr(p.clone());
    return expr_stmt_node(expr.clone());
}

fn parse_expr(mut p: Parser) -> ASTNode {
    return parse_expr_prec(p.clone(), 1);
}

fn parse_expr_prec(mut p: Parser, mut min_prec: i32) -> ASTNode {

    let mut lhs = parse_atom(p.clone());


    loop {
        let mut kind = parser_kind(p.clone());
        

        if is_stop_token(kind.clone()) {
            break;
        }
        
        let mut prec = infix_prec(kind.clone());
        if prec == 0 || prec < min_prec {
            break;
        }
        

        if kind == TokenKind::Asn {
            p.pos = p.pos + 1;
            let mut rhs = parse_expr_prec(p.clone(), prec);
            lhs = bin_node("=", lhs.clone(), rhs.clone());
            continue;
        }
        

        if kind == TokenKind::AddEq || kind == TokenKind::SubEq || kind == TokenKind::MulEq || kind == TokenKind::DivEq {
            let mut op_text = parser_text(p.clone());
            p.pos = p.pos + 1;
            let mut rhs = parse_expr_prec(p.clone(), prec);
            lhs = bin_node(&(op_text), lhs.clone(), rhs.clone());
            continue;
        }
        

        if kind == TokenKind::Or {
            p.pos = p.pos + 1;
            let mut rhs = parse_expr_prec(p.clone(), prec + 1);
            lhs = bin_node("||", lhs.clone(), rhs.clone());
            continue;
        }
        

        if kind == TokenKind::And {
            p.pos = p.pos + 1;
            let mut rhs = parse_expr_prec(p.clone(), prec + 1);
            lhs = bin_node("&&", lhs.clone(), rhs.clone());
            continue;
        }
        

        if kind == TokenKind::Eq || kind == TokenKind::Neq {
            let mut op_text = parser_text(p.clone());
            p.pos = p.pos + 1;
            let mut rhs = parse_expr_prec(p.clone(), prec + 1);
            lhs = bin_node(&(op_text), lhs.clone(), rhs.clone());
            continue;
        }
        

        if kind == TokenKind::Lt || kind == TokenKind::Gt || kind == TokenKind::Le || kind == TokenKind::Ge {
            let mut op_text = parser_text(p.clone());
            p.pos = p.pos + 1;
            let mut rhs = parse_expr_prec(p.clone(), prec + 1);
            lhs = bin_node(&(op_text), lhs.clone(), rhs.clone());
            continue;
        }
        
        if kind == TokenKind::Range || kind == TokenKind::RangeEq {
            let mut op_text = parser_text(p.clone());
            p.pos = p.pos + 1;
            let mut rhs = parse_expr_prec(p.clone(), prec + 1);
            lhs = bin_node(&(op_text), lhs.clone(), rhs.clone());
            continue;
        }
        

        if kind == TokenKind::Add || kind == TokenKind::Sub {
            let mut op_text = parser_text(p.clone());
            p.pos = p.pos + 1;
            let mut rhs = parse_expr_prec(p.clone(), prec + 1);
            lhs = bin_node(&(op_text), lhs.clone(), rhs.clone());
            continue;
        }
        

        if kind == TokenKind::Star || kind == TokenKind::Div || kind == TokenKind::Mod {
            let mut op_text = parser_text(p.clone());
            p.pos = p.pos + 1;
            let mut rhs = parse_expr_prec(p.clone(), prec + 1);
            lhs = bin_node(&(op_text), lhs.clone(), rhs.clone());
            continue;
        }
        

        if kind == TokenKind::Dot {
            p.pos = p.pos + 1;
            let mut field = parser_text(p.clone());
            p.pos = p.pos + 1;
            lhs = dot_node(lhs.clone(), &(field));
            continue;
        }
        

        if kind == TokenKind::DotQuest {
            p.pos = p.pos + 1;
            lhs = error_propagate_node(lhs.clone());
            continue;
        }
        

        if kind == TokenKind::DotView {
            p.pos = p.pos + 1;
            lhs = view_node(lhs.clone());
            continue;
        }
        if kind == TokenKind::DotMut {
            p.pos = p.pos + 1;
            lhs = mut_node(lhs.clone());
            continue;
        }
        if kind == TokenKind::DotMove || kind == TokenKind::DotTake {
            p.pos = p.pos + 1;
            lhs = move_node(lhs.clone());
            continue;
        }
        

        if kind == TokenKind::LParen {
            let mut args_str: String = "".to_string();
            let mut args = empty_list();
            p.pos = p.pos + 1;
            parser_skip_nl_semi(p.clone());
            while parser_kind(p.clone()) != TokenKind::RParen && parser_kind(p.clone()) != TokenKind::EOF {
                let mut arg = parse_expr_prec(p.clone(), 1);
                if args_str != "" {
                    args_str = format!("{}{}", args_str, " ")
                }
                args_str = format!("{}{}", args_str, arg.value);
                args.push(arg);
                parser_skip_nl_semi(p.clone());
                if parser_kind(p.clone()) == TokenKind::Comma {
                    p.pos = p.pos + 1;
                    parser_skip_nl_semi(p.clone());
                }
            }
            if parser_kind(p.clone()) == TokenKind::RParen {
                p.pos = p.pos + 1
            }            lhs = call_node(lhs.clone(), &(args_str), args.clone());
            continue;
        }
        

        if kind == TokenKind::LSquare {
            p.pos = p.pos + 1;
            let mut index = parse_expr(p.clone());
            if parser_kind(p.clone()) == TokenKind::RSquare {
                p.pos = p.pos + 1
            }            lhs = bin_node("[]", lhs.clone(), index.clone());
            continue;
        }
        

        break;
    }

    return lhs;
}

fn parse_atom(mut p: Parser) -> ASTNode {
    let mut kind = parser_kind(p.clone());
    let mut tok = parser_cur(p.clone());

    match kind {
        TokenKind::Int => {
            p.pos = p.pos + 1;
            return int_node(&(tok.text));
        },
        TokenKind::Uint => {
            p.pos = p.pos + 1;
            return int_node(&(tok.text));
        },
        TokenKind::Float => {
            p.pos = p.pos + 1;
            return str_node(&(tok.text));
        },
        TokenKind::Str => {
            p.pos = p.pos + 1;
            return str_node(&(tok.text));
        },
        TokenKind::True => {
            p.pos = p.pos + 1;
            return bool_node("true");
        },
        TokenKind::False => {
            p.pos = p.pos + 1;
            return bool_node("false");
        },
        TokenKind::Nil => {
            p.pos = p.pos + 1;
            return nil_node();
        },
        TokenKind::NoneKW => {
            p.pos = p.pos + 1;
            return ident_node("None");
        },
        TokenKind::SomeKW => {
            p.pos = p.pos + 1;
            if parser_kind(p.clone()) == TokenKind::LParen {
                p.pos = p.pos + 1;
                let mut inner = parse_expr(p.clone());
                if parser_kind(p.clone()) == TokenKind::RParen {
                    p.pos = p.pos + 1
                }                let mut sargs = empty_list();
                sargs.push(inner.clone());
                return call_node(ident_node("Some"), &(inner.value), sargs.clone());
            }
            return ident_node("Some");
        },
        TokenKind::OkKW => {
            p.pos = p.pos + 1;
            if parser_kind(p.clone()) == TokenKind::LParen {
                p.pos = p.pos + 1;
                let mut inner2 = parse_expr(p.clone());
                if parser_kind(p.clone()) == TokenKind::RParen {
                    p.pos = p.pos + 1
                }                let mut oargs = empty_list();
                oargs.push(inner2.clone());
                return call_node(ident_node("Ok"), &(inner2.value), oargs.clone());
            }
            return ident_node("Ok");
        },
        TokenKind::ErrKW => {
            p.pos = p.pos + 1;
            if parser_kind(p.clone()) == TokenKind::LParen {
                p.pos = p.pos + 1;
                let mut inner3 = parse_expr(p.clone());
                if parser_kind(p.clone()) == TokenKind::RParen {
                    p.pos = p.pos + 1
                }                let mut eargs = empty_list();
                eargs.push(inner3.clone());
                return call_node(ident_node("Err"), &(inner3.value), eargs.clone());
            }
            return ident_node("Err");
        },
        TokenKind::Ident => {
            

            if parser_peek_kind(p.clone()) == TokenKind::DoubleArrow {
                let mut pname = tok.text;
                p.pos = p.pos + 1;
                p.pos = p.pos + 1;
                let mut body = parse_expr(p.clone());
                let mut cparams = empty_list();
                cparams.push(ASTNode { kind: NodeKind::Param, name: pname.clone(), type_name: "".to_string(), value: "".to_string(), children: empty_list(), left: empty_list(), right: empty_list(), op: "".to_string(), params: empty_list(), cond: empty_list(), else_body: empty_list() });
                return closure_node(&(pname), cparams.clone(), body.clone());
            }
            p.pos = p.pos + 1;
            return ident_node(&(tok.text));
        },
        TokenKind::LParen => {
            


            let mut saved: i32 = p.pos;
            p.pos = p.pos + 1;
            if parser_kind(p.clone()) == TokenKind::RParen {
                

                p.pos = p.pos + 1;
                if parser_kind(p.clone()) == TokenKind::DoubleArrow {
                    p.pos = p.pos + 1;
                    let mut body = parse_expr(p.clone());
                    return closure_node("", empty_list(), body.clone());
                }                

                return nil_node();
            }
            

            let mut params_str: String = "".to_string();
            let mut mparams = empty_list();
            let mut is_closure: bool = false;
            while parser_kind(p.clone()) != TokenKind::RParen && parser_kind(p.clone()) != TokenKind::EOF {
                let mut pname = parser_text(p.clone());
                p.pos = p.pos + 1;
                
                parser_skip_nl_semi(p.clone());
                if parser_kind(p.clone()) == TokenKind::Ident {
                    let mut peek2 = parser_peek_kind(p.clone());
                    if peek2 != TokenKind::Comma && peek2 != TokenKind::RParen {
                        p.pos = p.pos + 1
                    }                }
                if params_str != "" {
                    params_str = format!("{}{}", params_str, " ")
                }
                params_str = format!("{}{}", params_str, pname);
                mparams.push(ASTNode { kind: NodeKind::Param, name: pname.clone(), type_name: "".to_string(), value: "".to_string(), children: empty_list(), left: empty_list(), right: empty_list(), op: "".to_string(), params: empty_list(), cond: empty_list(), else_body: empty_list() });
                parser_skip_nl_semi(p.clone());
                if parser_kind(p.clone()) == TokenKind::Comma {
                    p.pos = p.pos + 1;
                    parser_skip_nl_semi(p.clone());
                }
            }
            if parser_kind(p.clone()) == TokenKind::RParen {
                p.pos = p.pos + 1
            }
            if parser_kind(p.clone()) == TokenKind::DoubleArrow {
                

                p.pos = p.pos + 1;
                let mut body = parse_expr(p.clone());
                return closure_node(&(params_str), mparams.clone(), body.clone());
            }
            

            p.pos = saved;
            p.pos = p.pos + 1;
            let mut expr = parse_expr(p.clone());
            if parser_kind(p.clone()) == TokenKind::RParen {
                p.pos = p.pos + 1
            }
            return expr;
        },
        TokenKind::LSquare => {
            p.pos = p.pos + 1;
            let mut elems = empty_list();
            parser_skip_nl_semi(p.clone());
            while parser_kind(p.clone()) != TokenKind::RSquare && parser_kind(p.clone()) != TokenKind::EOF {
                let mut elem = parse_expr(p.clone());
                elems.push(elem);
                parser_skip_nl_semi(p.clone());
                if parser_kind(p.clone()) == TokenKind::Comma {
                    p.pos = p.pos + 1;
                    parser_skip_nl_semi(p.clone());
                }
            }
            if parser_kind(p.clone()) == TokenKind::RSquare {
                p.pos = p.pos + 1
            }
            return array_node(elems.clone());
        },
        TokenKind::Sub => {
            p.pos = p.pos + 1;
            let mut operand = parse_atom(p.clone());
            return unary_node("-", operand.clone());
        },
        TokenKind::Dot => {
            

            p.pos = p.pos + 1;
            let mut field = parser_text(p.clone());
            p.pos = p.pos + 1;
            return dot_node(ident_node("self"), &(field));
        },
        TokenKind::Not => {
            p.pos = p.pos + 1;
            let mut operand = parse_atom(p.clone());
            return unary_node("!", operand.clone());
        },
        TokenKind::CStr => {
            p.pos = p.pos + 1;
            return str_node(&(tok.text));
        },
        TokenKind::Char => {
            p.pos = p.pos + 1;
            return str_node(&(tok.text));
        },
        TokenKind::FStrStart => {
            

            p.pos = p.pos + 1;
            let mut parts_str: String = "".to_string();
            let mut parts = empty_list();
            while parser_kind(p.clone()) != TokenKind::FStrEnd && parser_kind(p.clone()) != TokenKind::EOF {
                let mut fk = parser_kind(p.clone());
                if fk == TokenKind::Str || fk == TokenKind::FStrPart {
                    

                    if parts_str != "" {
                        parts_str = format!("{}{}", parts_str, " ")
                    }                    let mut ptxt = parser_text(p.clone());
                    parts_str = format!("{}{}", format!("{}{}", format!("{}{}", parts_str, "\""), ptxt), "\"");
                    parts.push(str_node(&(ptxt)));
                    p.pos = p.pos + 1
                } else if fk == TokenKind::FStrNote {
                    

                    p.pos = p.pos + 1
                } else if fk == TokenKind::Ident {
                    

                    if parts_str != "" {
                        parts_str = format!("{}{}", parts_str, " ")
                    }                    let mut iname = parser_text(p.clone());
                    parts_str = format!("{}{}", parts_str, iname);
                    parts.push(ident_node(&(iname)));
                    p.pos = p.pos + 1
                } else {
                    

                    let mut inner = parse_expr(p.clone());
                    if parts_str != "" {
                        parts_str = format!("{}{}", parts_str, " ")
                    }                    parts_str = format!("{}{}", parts_str, inner.value);
                    parts.push(inner)
                }

            }
            if parser_kind(p.clone()) == TokenKind::FStrEnd {
                p.pos = p.pos + 1
            }
            return fstr_node(&(parts_str), parts.clone());
        },
        TokenKind::LBrace => {
            


            let mut p1: i32 = p.pos + 1;
            if p1 < p.token_count {
                let mut t1 = p.tokens[p1 as usize].clone();
                if t1.kind == TokenKind::Ident {
                    let mut p2: i32 = p.pos + 2;
                    if p2 < p.token_count {
                        let mut t2 = p.tokens[p2 as usize].clone();
                        if t2.kind == TokenKind::Colon {
                            return parse_object(p.clone());
                        }                    }                }            }
            


            p.pos = p.pos + 1;
            return nil_node();
        },
        _ => {
            

            p.pos = p.pos + 1;
            return nil_node();
        },
    }
}

fn parse_alias_stmt(mut p: Parser) -> ASTNode {
    p.pos = p.pos + 1;
    let mut name = parser_text(p.clone());
    p.pos = p.pos + 1;
    if parser_kind(p.clone()) == TokenKind::Asn {
        p.pos = p.pos + 1
    }
    let mut target = parser_text(p.clone());
    p.pos = p.pos + 1;
    return alias_node(&(name), &(target));
}

fn parse_enum_decl(mut p: Parser) -> ASTNode {
    p.pos = p.pos + 1;
    let mut name = parser_text(p.clone());
    p.pos = p.pos + 1;
    parser_skip_nl_semi(p.clone());


    let mut generic_params: String = "".to_string();
    if parser_kind(p.clone()) == TokenKind::Lt {
        generic_params = parser_try_read_generic_type(p.clone(), "");
        parser_skip_nl_semi(p.clone());
    }


    let mut pk = parser_kind(p.clone());
    if pk != TokenKind::LBrace && pk != TokenKind::EOF {
        

        p.pos = p.pos + 1;
        parser_skip_nl_semi(p.clone());
    }

    let mut variants = empty_list();
    if parser_kind(p.clone()) == TokenKind::LBrace {
        p.pos = p.pos + 1;
        parser_skip_nl_semi(p.clone());
        while parser_kind(p.clone()) != TokenKind::RBrace && parser_kind(p.clone()) != TokenKind::EOF {
            let mut vname = parser_text(p.clone());
            p.pos = p.pos + 1;
            let mut payload: String = "".to_string();
            

            parser_skip_nl_semi(p.clone());
            let mut vk = parser_kind(p.clone());
            if vk == TokenKind::LParen {
                

                let mut depth: i32 = 1;
                p.pos = p.pos + 1;
                while depth > 0 && parser_kind(p.clone()) != TokenKind::EOF {
                    if parser_kind(p.clone()) == TokenKind::LParen {
                        depth = depth + 1
                    }
                    if parser_kind(p.clone()) == TokenKind::RParen {
                        depth = depth - 1
                    }
                    p.pos = p.pos + 1;
                }
            } else if vk == TokenKind::LBrace {
                

                let mut depth: i32 = 1;
                p.pos = p.pos + 1;
                while depth > 0 && parser_kind(p.clone()) != TokenKind::EOF {
                    if parser_kind(p.clone()) == TokenKind::LBrace {
                        depth = depth + 1
                    }
                    if parser_kind(p.clone()) == TokenKind::RBrace {
                        depth = depth - 1
                    }
                    p.pos = p.pos + 1;
                }
            } else if vk == TokenKind::Ident {
                

                let mut peek = parser_peek_kind(p.clone());
                if peek != TokenKind::Comma && peek != TokenKind::Newline && peek != TokenKind::RBrace && peek != TokenKind::EOF {
                    p.pos = p.pos + 1
                }            }
            

            parser_skip_nl_semi(p.clone());
            if parser_kind(p.clone()) == TokenKind::Asn {
                p.pos = p.pos + 1;
                parser_skip_nl_semi(p.clone());
                payload = format!("{}{}", "=", parser_text(p.clone()));
                p.pos = p.pos + 1
            }
            
            variants.push(ASTNode { kind: NodeKind::Param, name: vname.clone(), type_name: payload.clone(), value: "".to_string(), children: empty_list(), left: empty_list(), right: empty_list(), op: "".to_string(), params: empty_list(), cond: empty_list(), else_body: empty_list() });
            
            parser_skip_nl_semi(p.clone());
            if parser_kind(p.clone()) == TokenKind::Comma {
                p.pos = p.pos + 1;
                parser_skip_nl_semi(p.clone());
            }
        }
        if parser_kind(p.clone()) == TokenKind::RBrace {
            p.pos = p.pos + 1
        }    }
    let mut node = enum_node(&(name), variants.clone(), "");
    node.type_name = generic_params;
    return node;
}

fn parse_use_stmt(mut p: Parser) -> ASTNode {
    p.pos = p.pos + 1;
    let mut path_str: String = "".to_string();
    let mut items_str: String = "".to_string();

    let mut kind = parser_kind(p.clone());
    if kind == TokenKind::Dot {
        

        p.pos = p.pos + 1;
        let mut ffi_kind = parser_text(p.clone());
        p.pos = p.pos + 1;
        path_str = ffi_kind;
        

        parser_skip_nl_semi(p.clone());
        if parser_kind(p.clone()) == TokenKind::Str || parser_kind(p.clone()) == TokenKind::Lt {
            

            let mut k = parser_kind(p.clone());
            if k == TokenKind::Lt {
                

                p.pos = p.pos + 1;
                while parser_kind(p.clone()) != TokenKind::Gt && parser_kind(p.clone()) != TokenKind::EOF {
                    p.pos = p.pos + 1;
                }
                if parser_kind(p.clone()) == TokenKind::Gt {
                    p.pos = p.pos + 1
                }            } else {
                p.pos = p.pos + 1
            }
        }    } else if kind == TokenKind::Pac {
        path_str = "pac".to_string();
        p.pos = p.pos + 1;
        

        while parser_kind(p.clone()) == TokenKind::Dot {
            p.pos = p.pos + 1;
            path_str = format!("{}{}", format!("{}{}", path_str, "."), parser_text(p.clone()));
            p.pos = p.pos + 1;
        }
    } else if kind == TokenKind::Super {
        path_str = "super".to_string();
        p.pos = p.pos + 1;
        while parser_kind(p.clone()) == TokenKind::Dot {
            p.pos = p.pos + 1;
            path_str = format!("{}{}", format!("{}{}", path_str, "."), parser_text(p.clone()));
            p.pos = p.pos + 1;
        }
    } else {
        

        path_str = parser_text(p.clone());
        p.pos = p.pos + 1;
        while parser_kind(p.clone()) == TokenKind::Dot {
            p.pos = p.pos + 1;
            path_str = format!("{}{}", format!("{}{}", path_str, "."), parser_text(p.clone()));
            p.pos = p.pos + 1;
        }
    }



    let mut items = empty_list();
    parser_skip_nl_semi(p.clone());
    if parser_kind(p.clone()) == TokenKind::Colon {
        p.pos = p.pos + 1;
        parser_skip_nl_semi(p.clone());
        loop {
            let mut ik = parser_kind(p.clone());
            if ik == TokenKind::Ident {
                if items_str != "" {
                    items_str = format!("{}{}", items_str, " ")
                }                let mut item_name = parser_text(p.clone());
                items_str = format!("{}{}", items_str, item_name);
                items.push(ASTNode { kind: NodeKind::Param, name: item_name.clone(), type_name: "".to_string(), value: "".to_string(), children: empty_list(), left: empty_list(), right: empty_list(), op: "".to_string(), params: empty_list(), cond: empty_list(), else_body: empty_list() });
                p.pos = p.pos + 1
            } else {
                break;
            }

            parser_skip_nl_semi(p.clone());
            if parser_kind(p.clone()) == TokenKind::Comma {
                p.pos = p.pos + 1;
                parser_skip_nl_semi(p.clone())
            } else {
                break;
            }

        }
    }

    return use_node(&(path_str), &(items_str), items.clone());
}

fn parse_spec_decl(mut p: Parser) -> ASTNode {
    p.pos = p.pos + 1;
    let mut name = parser_text(p.clone());
    p.pos = p.pos + 1;
    parser_skip_nl_semi(p.clone());


    let mut spec_generic: String = "".to_string();
    if parser_kind(p.clone()) == TokenKind::Lt {
        spec_generic = parser_try_read_generic_type(p.clone(), "");
        parser_skip_nl_semi(p.clone());
    }

    let mut methods = empty_list();
    if parser_kind(p.clone()) == TokenKind::LBrace {
        p.pos = p.pos + 1;
        parser_skip_nl_semi(p.clone());
        while parser_kind(p.clone()) != TokenKind::RBrace && parser_kind(p.clone()) != TokenKind::EOF {
            if parser_kind(p.clone()) == TokenKind::Fn {
                

                p.pos = p.pos + 1;
                let mut mname = parser_text(p.clone());
                p.pos = p.pos + 1;
                

                let mut ret_type: String = "".to_string();
                

                parser_skip_nl_semi(p.clone());
                if parser_kind(p.clone()) == TokenKind::LParen {
                    let mut depth: i32 = 1;
                    p.pos = p.pos + 1;
                    while depth > 0 && parser_kind(p.clone()) != TokenKind::EOF {
                        if parser_kind(p.clone()) == TokenKind::LParen {
                            depth = depth + 1
                        }
                        if parser_kind(p.clone()) == TokenKind::RParen {
                            depth = depth - 1
                        }
                        p.pos = p.pos + 1;
                    }
                }                


                parser_skip_nl_semi(p.clone());
                let mut rk = parser_kind(p.clone());
                if rk == TokenKind::Arrow {
                    p.pos = p.pos + 1;
                    parser_skip_nl_semi(p.clone());
                    ret_type = parser_text(p.clone());
                    p.pos = p.pos + 1
                } else if rk == TokenKind::Ident {
                    let mut peek = parser_peek_kind(p.clone());
                    if peek == TokenKind::LBrace || peek == TokenKind::Newline || peek == TokenKind::Semi {
                        ret_type = parser_text(p.clone());
                        p.pos = p.pos + 1
                    }                }                

                methods.push(ASTNode { kind: NodeKind::Param, name: mname.clone(), type_name: ret_type.clone(), value: "".to_string(), children: empty_list(), left: empty_list(), right: empty_list(), op: "".to_string(), params: empty_list(), cond: empty_list(), else_body: empty_list() });
                


                parser_skip_nl_semi(p.clone());
                if parser_kind(p.clone()) == TokenKind::LBrace {
                    let mut depth: i32 = 1;
                    p.pos = p.pos + 1;
                    while depth > 0 && parser_kind(p.clone()) != TokenKind::EOF {
                        if parser_kind(p.clone()) == TokenKind::LBrace {
                            depth = depth + 1
                        }
                        if parser_kind(p.clone()) == TokenKind::RBrace {
                            depth = depth - 1
                        }
                        p.pos = p.pos + 1;
                    }
                }            } else {
                p.pos = p.pos + 1
            }

            parser_skip_nl_semi(p.clone());
        }
        if parser_kind(p.clone()) == TokenKind::RBrace {
            p.pos = p.pos + 1
        }    }
    let mut snode = spec_node(&(name), methods.clone());
    snode.type_name = spec_generic;
    return snode;
}

fn parse_ext_stmt(mut p: Parser) -> ASTNode {
    p.pos = p.pos + 1;


    let mut ext_generic: String = "".to_string();
    if parser_kind(p.clone()) == TokenKind::Lt {
        ext_generic = parser_try_read_generic_type(p.clone(), "")
    }

    let mut target = parser_text(p.clone());
    p.pos = p.pos + 1;


    let mut for_spec: String = "".to_string();
    parser_skip_nl_semi(p.clone());
    if parser_kind(p.clone()) == TokenKind::For {
        p.pos = p.pos + 1;
        parser_skip_nl_semi(p.clone());
        for_spec = parser_text(p.clone());
        p.pos = p.pos + 1;
        parser_skip_nl_semi(p.clone());
    }


    if parser_kind(p.clone()) == TokenKind::Lt {
        let mut depth: i32 = 1;
        p.pos = p.pos + 1;
        while depth > 0 && parser_kind(p.clone()) != TokenKind::EOF {
            if parser_kind(p.clone()) == TokenKind::Lt {
                depth = depth + 1
            }
            if parser_kind(p.clone()) == TokenKind::Gt {
                depth = depth - 1
            }
            p.pos = p.pos + 1;
        }
    }

    parser_skip_nl_semi(p.clone());
    let mut methods = empty_list();
    if parser_kind(p.clone()) == TokenKind::LBrace {
        p.pos = p.pos + 1;
        parser_skip_nl_semi(p.clone());
        while parser_kind(p.clone()) != TokenKind::RBrace && parser_kind(p.clone()) != TokenKind::EOF {
            if parser_kind(p.clone()) == TokenKind::Fn {
                let mut method = parse_fn_decl(p.clone());
                methods.push(method.clone())
            } else if parser_kind(p.clone()) == TokenKind::Mut {
                

                p.pos = p.pos + 1;
                if parser_kind(p.clone()) == TokenKind::Fn {
                    let mut method = parse_fn_decl(p.clone());
                    methods.push(method.clone());
                }            } else if parser_kind(p.clone()) == TokenKind::Static {
                

                p.pos = p.pos + 1;
                if parser_kind(p.clone()) == TokenKind::Fn {
                    let mut method = parse_fn_decl(p.clone());
                    methods.push(method);
                }            } else {
                

                p.pos = p.pos + 1;
                parser_skip_nl_semi(p.clone());
                if parser_kind(p.clone()) == TokenKind::Ident {
                    p.pos = p.pos + 1
                }            }

            parser_skip_nl_semi(p.clone());
        }
        if parser_kind(p.clone()) == TokenKind::RBrace {
            p.pos = p.pos + 1
        }    }
    let mut enode = ext_node(&(target), methods.clone(), &(for_spec));
    enode.type_name = ext_generic;
    return enode;
}

fn parse_is_stmt(mut p: Parser) -> ASTNode {
    p.pos = p.pos + 1;
    parser_skip_nl_semi(p.clone());
    let mut expr = parse_expr(p.clone());
    parser_skip_nl_semi(p.clone());

    let mut branches = empty_list();
    let mut else_branch = empty_list();
    if parser_kind(p.clone()) == TokenKind::LBrace {
        p.pos = p.pos + 1;
        parser_skip_nl_semi(p.clone());
        while parser_kind(p.clone()) != TokenKind::RBrace && parser_kind(p.clone()) != TokenKind::EOF {
            parser_skip_nl_semi(p.clone());
            if parser_kind(p.clone()) == TokenKind::Else {
                

                p.pos = p.pos + 1;
                parser_skip_nl_semi(p.clone());
                if parser_kind(p.clone()) == TokenKind::Arrow {
                    p.pos = p.pos + 1
                }                parser_skip_nl_semi(p.clone());
                if parser_kind(p.clone()) == TokenKind::LBrace {
                    let mut body_list = parse_body(p.clone());
                    let mut body_str = body_to_str(body_list.clone());
                    let mut body_node = block_node(&(body_str));
                    body_node.children = body_list;
                    else_branch.push(body_node)
                } else {
                    let mut body = parse_expr(p.clone());
                    else_branch.push(body)
                }
            } else {
                

                let mut pattern = parse_expr(p.clone());
                parser_skip_nl_semi(p.clone());
                if parser_kind(p.clone()) == TokenKind::Arrow {
                    p.pos = p.pos + 1
                }                parser_skip_nl_semi(p.clone());
                if parser_kind(p.clone()) == TokenKind::LBrace {
                    let mut body_list2 = parse_body(p.clone());
                    let mut body_str2 = body_to_str(body_list2.clone());
                    let mut body_node2 = block_node(&(body_str2));
                    body_node2.children = body_list2;
                    branches.push(bin_node("->", pattern.clone(), body_node2.clone()))
                } else {
                    let mut body2 = parse_expr(p.clone());
                    branches.push(bin_node("->", pattern.clone(), body2.clone()))
                }
            }

            parser_skip_nl_semi(p.clone());
        }
        if parser_kind(p.clone()) == TokenKind::RBrace {
            p.pos = p.pos + 1
        }    }
    return is_node(expr.clone(), branches.clone(), else_branch.clone());
}

fn parse_object(mut p: Parser) -> ASTNode {
    p.pos = p.pos + 1;
    parser_skip_nl_semi(p.clone());
    let mut pairs_str: String = "".to_string();
    let mut pairs = empty_list();
    while parser_kind(p.clone()) != TokenKind::RBrace && parser_kind(p.clone()) != TokenKind::EOF {
        let mut key = parser_text(p.clone());
        p.pos = p.pos + 1;
        if parser_kind(p.clone()) == TokenKind::Colon {
            p.pos = p.pos + 1
        }
        let mut val = parse_expr(p.clone());
        if pairs_str != "" {
            pairs_str = format!("{}{}", pairs_str, " ")
        }
        pairs_str = format!("{}{}", format!("{}{}", format!("{}{}", format!("{}{}", format!("{}{}", pairs_str, "("), key), " "), val.value), ")");
        pairs.push(pair_node(&(key), val.clone()));
        parser_skip_nl_semi(p.clone());
        if parser_kind(p.clone()) == TokenKind::Comma {
            p.pos = p.pos + 1;
            parser_skip_nl_semi(p.clone());
        }
    }
    if parser_kind(p.clone()) == TokenKind::RBrace {
        p.pos = p.pos + 1
    }
    return object_node(&(pairs_str), pairs.clone());
}

// === typeinfer ===
// a2r Standard Library (from crate)

#[derive(Clone, Debug, PartialEq)]
struct TypeEnv {
    pub var_types: std::collections::HashMap<String, String>,
    pub fn_ret_types: std::collections::HashMap<String, String>,
    pub fn_param_types: std::collections::HashMap<String, String>,
    pub struct_fields: std::collections::HashMap<String, String>,
}

fn typeenv_new() -> TypeEnv {
    TypeEnv { var_types: std::collections::HashMap::new(), fn_ret_types: std::collections::HashMap::new(), fn_param_types: std::collections::HashMap::new(), struct_fields: std::collections::HashMap::new() }
}

fn type_bind(mut env: TypeEnv, mut name: &str, mut type_id: i32) {
    env.var_types.insert(name.to_string(), (type_id).to_string());
}

fn type_fn_sig(mut env: TypeEnv, mut name: &str, mut ret_type: i32, mut param_types: &str) {
    env.fn_ret_types.insert(name.to_string(), (ret_type).to_string());
    env.fn_param_types.insert(name.to_string(), (param_types).to_string());
}

fn type_lookup(mut env: TypeEnv, mut name: &str) -> i32 {
    if env.var_types.contains_key(name) {
        return (env.var_types.get(&name.to_string()).and_then(|v| v.parse::<i32>().ok()).unwrap_or(0));
    }
    return -1;
}

fn type_fn_ret(mut env: TypeEnv, mut name: &str) -> i32 {
    if env.fn_ret_types.contains_key(name) {
        return (env.fn_ret_types.get(&name.to_string()).and_then(|v| v.parse::<i32>().ok()).unwrap_or(0));
    }
    return -1;
}

fn type_fn_param(mut env: TypeEnv, mut fn_name: &str, mut index: i32) -> i32 {
    if env.fn_param_types.contains_key(fn_name) {
        let mut types_str = env.fn_param_types.get(&*fn_name).cloned().unwrap_or_default();
        let mut t = str_get_part(&(types_str), index);
        return str_to_int(&(t));
    }
    return -1;
}

fn type_from_name(mut name: &str) -> i32 {
    if name == "int" {
        return 0;
    }
    if name == "str" {
        return 1;
    }
    if name == "bool" {
        return 2;
    }
    if name == "void" {
        return 3;
    }
    return -1;
}

fn type_is_cmp_op(mut op: &str) -> i32 {
    if op == "==" {
        return 1;
    }
    if op == "!=" {
        return 1;
    }
    if op == "<" {
        return 1;
    }
    if op == ">" {
        return 1;
    }
    if op == "<=" {
        return 1;
    }
    if op == ">=" {
        return 1;
    }
    return 0;
}

fn type_infer_expr(mut tenv: TypeEnv, mut node: ASTNode) -> i32 {
    let mut kind = node.kind;
    if kind == NodeKind::IntExpr {
        return 0;
    }
    if kind == NodeKind::StrExpr {
        return 1;
    }
    if kind == NodeKind::BoolExpr {
        return 2;
    }
    if kind == NodeKind::NilNode {
        return 3;
    }
    if kind == NodeKind::IdentExpr {
        return type_lookup(tenv.clone(), node.name.as_str());
    }
    if kind == NodeKind::BinExpr {
        let mut op = node.op;
        if op == "=" {
            let mut left_node = node.left[0 as usize].clone();
            if left_node.kind == NodeKind::IdentExpr {
                return type_lookup(tenv.clone(), left_node.name.as_str());
            }            return -1;
        }        if type_is_cmp_op(op.as_str()) == 1 {
            return 2;
        }        if op == "||" {
            return 2;
        }        if op == "&&" {
            return 2;
        }        if op == ".." {
            return 0;
        }        if op == "..=" {
            return 0;
        }        if op == "+" {
            let mut lt = type_infer_expr(tenv.clone(), node.left[0 as usize].clone());
            let mut rt = type_infer_expr(tenv.clone(), node.right[0 as usize].clone());
            if lt == 1 {
                return 1;
            }            if rt == 1 {
                return 1;
            }            return 0;
        }        return 0;
    }
    if kind == NodeKind::UnaryExpr {
        if node.op == "!" {
            return 2;
        }        return 0;
    }
    if kind == NodeKind::CallExpr {
        let mut ret = type_fn_ret(tenv.clone(), node.name.as_str());
        if ret == -1 {
            

            let mut cn = (node.name.len() as i32);
            if cn >= 7 {
                let mut tail = a2r_std::str_substr(node.name.as_str(), cn - 7, cn);
                if tail == ".substr" {
                    return 1;
                }            }            if cn >= 8 {
                let mut tail2 = a2r_std::str_substr(node.name.as_str(), cn - 8, cn);
                if tail2 == ".get_str" {
                    return 1;
                }            }        }        return ret;
    }
    if kind == NodeKind::DotExpr {
        return 0;
    }
    return -1;
}

fn type_infer_stmts(mut tenv: TypeEnv, mut stmts: Vec<ASTNode>) {
    let mut i: i32 = 0;
    let mut n = (stmts.len() as i32);
    while i < n {
        let mut stmt = stmts[i as usize].clone();
        type_infer_stmt(tenv.clone(), stmt.clone());
        i = i + 1;
    }
}

fn type_infer_let_var(mut tenv: TypeEnv, mut stmt: ASTNode) {
    if (stmt.left.len() as i32) > 0 {
        let mut expr = stmt.left[0 as usize].clone();
        let mut t = type_infer_expr(tenv.clone(), expr.clone());
        if t == -1 {
            if stmt.type_name != "" {
                t = type_from_name(stmt.type_name.as_str())
            }        }        type_bind(tenv.clone(), stmt.name.as_str(), t);
    }
}

fn type_infer_stmt(mut tenv: TypeEnv, mut stmt: ASTNode) -> i32 {
    let mut kind = stmt.kind;
    if kind == NodeKind::FnStmt {
        let mut ret_type: i32 = -1;
        if stmt.type_name != "" {
            ret_type = type_from_name(stmt.type_name.as_str())
        }        let mut param_types: String = "".to_string();
        let mut params = stmt.params;
        let mut j: i32 = 0;
        let mut pn = (params.len() as i32);
        while j < pn {
            let mut param = params[j as usize].clone();
            let mut pt: i32 = -1;
            if param.type_name != "" {
                pt = type_from_name(param.type_name.as_str())
            }
            if j > 0 {
                param_types = format!("{}{}", param_types, ",")
            }
            param_types = eval_str_cat(&(param_types), &(int_to_str(pt)));
            type_bind(tenv.clone(), param.name.as_str(), pt);
            j = j + 1;
        }
        type_fn_sig(tenv.clone(), stmt.name.as_str(), ret_type, param_types.as_str());
        type_infer_stmts(tenv.clone(), stmt.children);
        return 0;
    }
    if kind == NodeKind::LetStmt {
        type_infer_let_var(tenv.clone(), stmt.clone());
        return 0;
    }
    if kind == NodeKind::VarStmt {
        type_infer_let_var(tenv.clone(), stmt.clone());
        return 0;
    }
    if kind == NodeKind::IfStmt {
        type_infer_stmts(tenv.clone(), stmt.children);
        if (stmt.else_body.len() as i32) > 0 {
            type_infer_stmts(tenv.clone(), stmt.else_body);
        }        return 0;
    }
    if kind == NodeKind::ForStmt {
        type_infer_stmts(tenv.clone(), stmt.children);
        return 0;
    }
    if kind == NodeKind::ForInStmt {
        type_bind(tenv.clone(), stmt.name.as_str(), 0);
        type_infer_stmts(tenv.clone(), stmt.children);
        return 0;
    }
    if kind == NodeKind::BlockStmt {
        type_infer_stmts(tenv.clone(), stmt.children);
        return 0;
    }
    return 0;
}

fn type_infer_program(mut tenv: TypeEnv, mut stmts: Vec<ASTNode>) {

    let mut i: i32 = 0;
    let mut n = (stmts.len() as i32);
    while i < n {
        let mut s = stmts[i as usize].clone();
        if s.kind == NodeKind::TypeStmt {
            let mut fields: String = "".to_string();
            let mut j: i32 = 0;
            let mut fn2 = (s.params.len() as i32);
            while j < fn2 {
                if j > 0 {
                    fields = format!("{}{}", fields, ",")
                }
                fields = format!("{}{}", fields, s.params[j as usize].clone().name);
                j = j + 1;
            }
            tenv.struct_fields.insert(s.name.to_string(), (fields).to_string());
        }
        i = i + 1;
    }
    type_infer_stmts(tenv.clone(), stmts.clone());
}

// === codegen ===
// a2r Standard Library (from crate)



















































#[derive(Clone, Debug, PartialEq)]
struct CodeGen {
    pub code: Vec<i32>,
    pub strings: std::collections::HashMap<String, String>,
    pub n_strings: i32,
    pub exports: std::collections::HashMap<String, String>,
    pub locals: std::collections::HashMap<String, String>,
    pub n_locals: i32,
    pub elem_types: std::collections::HashMap<String, String>,
}

fn codegen_new() -> CodeGen {
    CodeGen { code: Vec::new(), strings: std::collections::HashMap::new(), n_strings: 0, exports: std::collections::HashMap::new(), locals: std::collections::HashMap::new(), n_locals: 0, elem_types: std::collections::HashMap::new() }
}

fn codegen_offset(mut cg: CodeGen) -> i32 {
    return (cg.code.len() as i32);
}

fn codegen_record_elem(mut cg: CodeGen, mut name: &str, mut et: i32) {
    cg.elem_types.insert(name.to_string(), (et).to_string());
}

fn codegen_lookup_elem(mut cg: CodeGen, mut name: &str) -> i32 {
    if cg.elem_types.contains_key(name) {
        return (cg.elem_types.get(&name.to_string()).and_then(|v| v.parse::<i32>().ok()).unwrap_or(0));
    }
    return 0;
}

fn codegen_str_returning_call(mut cg: CodeGen, mut callee: &str) -> i32 {
    if codegen_extract_method_suffix(&(callee), ".get") == 1 {
        let mut vn = codegen_extract_var_name(&(callee));
        let mut et = codegen_lookup_elem(cg.clone(), vn.as_str());
        if et == 1 {
            return 1;
        }    }
    if codegen_extract_method_suffix(&(callee), ".pop") == 1 {
        let mut vn2 = codegen_extract_var_name(&(callee));
        let mut et2 = codegen_lookup_elem(cg.clone(), vn2.as_str());
        if et2 == 1 {
            return 1;
        }    }
    if codegen_extract_method_suffix(&(callee), ".substr") == 1 {
        return 1;
    }
    if codegen_extract_method_suffix(&(callee), ".get_str") == 1 {
        return 1;
    }
    return 0;
}

fn codegen_emit(mut cg: CodeGen, mut byte: i32) {
    cg.code.push(byte);
}

fn codegen_emit_byte(mut cg: CodeGen, mut val: i32) {
    let mut b = val % 256;
    if b < 0 {
        b = b + 256
    }
    cg.code.push(b);
}

fn codegen_emit_i32(mut cg: CodeGen, mut val: i32) {
    codegen_emit_byte(cg.clone(), val);
    codegen_emit_byte(cg.clone(), val / 256);
    codegen_emit_byte(cg.clone(), val / 65536);
    codegen_emit_byte(cg.clone(), val / 16777216);
}

fn codegen_emit_u16(mut cg: CodeGen, mut val: i32) {
    codegen_emit_byte(cg.clone(), val);
    codegen_emit_byte(cg.clone(), val / 256);
}

fn codegen_emit_u32(mut cg: CodeGen, mut val: i32) {
    codegen_emit_byte(cg.clone(), val);
    codegen_emit_byte(cg.clone(), val / 256);
    codegen_emit_byte(cg.clone(), val / 65536);
    codegen_emit_byte(cg.clone(), val / 16777216);
}

fn codegen_add_string(mut cg: CodeGen, mut s: &str) -> i32 {
    let mut idx: i32 = cg.n_strings;
    cg.strings.insert(int_to_str(idx).to_string(), (s).to_string());
    cg.n_strings = cg.n_strings + 1;
    return idx;
}

fn codegen_alloc_local(mut cg: CodeGen, mut name: &str) -> i32 {
    if cg.locals.contains_key(name) {
        return (cg.locals.get(&name.to_string()).and_then(|v| v.parse::<i32>().ok()).unwrap_or(0));
    }
    let mut slot: i32 = cg.n_locals;
    cg.n_locals = cg.n_locals + 1;
    cg.locals.insert(name.to_string(), (slot).to_string());
    return slot;
}

fn codegen_find_local(mut cg: CodeGen, mut name: &str) -> i32 {
    if cg.locals.contains_key(name) {
        return (cg.locals.get(&name.to_string()).and_then(|v| v.parse::<i32>().ok()).unwrap_or(0));
    }
    return -1;
}

fn codegen_patch_u16(mut cg: CodeGen, mut off: i32, mut val: i32) {
    let mut b0 = val % 256;
    if b0 < 0 {
        b0 = b0 + 256
    }
    let mut b1 = val / 256 % 256;
    if b1 < 0 {
        b1 = b1 + 256
    }
    cg.code.insert(off as usize, b0);
    cg.code.insert((off + 1) as usize, b1);
}

fn codegen_patch_u32(mut cg: CodeGen, mut off: i32, mut val: i32) {
    let mut b0 = val % 256;
    if b0 < 0 {
        b0 = b0 + 256
    }
    let mut b1 = val / 256 % 256;
    if b1 < 0 {
        b1 = b1 + 256
    }
    let mut b2 = val / 65536 % 256;
    if b2 < 0 {
        b2 = b2 + 256
    }
    let mut b3 = val / 16777216 % 256;
    if b3 < 0 {
        b3 = b3 + 256
    }
    cg.code.insert(off as usize, b0);
    cg.code.insert((off + 1) as usize, b1);
    cg.code.insert((off + 2) as usize, b2);
    cg.code.insert((off + 3) as usize, b3);
}

fn codegen_expr(mut cg: CodeGen, mut node: ASTNode, mut tenv: TypeEnv) {
    let mut kind = node.kind;

    if kind == NodeKind::IntExpr {
        codegen_emit(cg.clone(), OP_CONST_I32);
        codegen_emit_i32(cg.clone(), str_to_int(&(node.name)));
        return;
    }

    if kind == NodeKind::StrExpr {
        codegen_emit(cg.clone(), OP_LOAD_STR);
        let mut idx = codegen_add_string(cg.clone(), node.name.as_str());
        codegen_emit_u16(cg.clone(), idx);
        return;
    }

    if kind == NodeKind::BoolExpr {
        codegen_emit(cg.clone(), OP_CONST_I32);
        if node.name == "true" {
            codegen_emit_i32(cg.clone(), BOOL_TRUE)
        } else {
            codegen_emit_i32(cg.clone(), BOOL_FALSE)
        }
        return;
    }

    if kind == NodeKind::NilNode {
        codegen_emit(cg.clone(), OP_CONST_0);
        return;
    }

    if kind == NodeKind::IdentExpr {
        let mut slot = codegen_find_local(cg.clone(), node.name.as_str());
        if slot >= 0 {
            codegen_emit(cg.clone(), OP_LOAD_LOCAL);
            codegen_emit(cg.clone(), slot)
        } else {
            codegen_emit(cg.clone(), OP_CONST_0)
        }
        return;
    }

    if kind == NodeKind::BinExpr {
        codegen_binop(cg.clone(), node.clone(), tenv.clone());
        return;
    }

    if kind == NodeKind::UnaryExpr {
        codegen_expr(cg.clone(), node.left[0 as usize].clone(), tenv.clone());
        if node.op == "-" {
            codegen_emit(cg.clone(), OP_CONST_I32);
            codegen_emit_i32(cg.clone(), -1);
            codegen_emit(cg.clone(), OP_MUL);
        }        if node.op == "!" {
            codegen_emit(cg.clone(), OP_CONST_I32);
            codegen_emit_i32(cg.clone(), 0);
            codegen_emit(cg.clone(), OP_EQ);
        }        return;
    }

    if kind == NodeKind::CallExpr {
        codegen_call(cg.clone(), node.clone(), tenv.clone());
        return;
    }

    if kind == NodeKind::DotExpr {
        codegen_expr(cg.clone(), node.left[0 as usize].clone(), tenv.clone());
        return;
    }

    codegen_emit(cg.clone(), OP_CONST_0);
    return;
}

fn codegen_binop(mut cg: CodeGen, mut node: ASTNode, mut tenv: TypeEnv) {
    let mut op = node.op;

    if op == "=" {
        codegen_expr(cg.clone(), node.right[0 as usize].clone(), tenv.clone());
        let mut left_node = node.left[0 as usize].clone();
        if left_node.kind == NodeKind::IdentExpr {
            let mut slot = codegen_find_local(cg.clone(), left_node.name.as_str());
            if slot >= 0 {
                codegen_emit(cg.clone(), OP_DUP);
                codegen_emit(cg.clone(), OP_STORE_LOCAL);
                codegen_emit(cg.clone(), slot);
            }        }        return;
    }

    if op == "+" {
        let mut lt = type_infer_expr(tenv, node.left[0 as usize].clone());
        if lt == 1 {
            codegen_expr(cg.clone(), node.left[0 as usize].clone(), tenv.clone());
            codegen_expr(cg.clone(), node.right[0 as usize].clone(), tenv.clone());
            codegen_emit(cg.clone(), OP_STR_CAT)
        } else {
            codegen_expr(cg.clone(), node.left[0 as usize].clone(), tenv.clone());
            codegen_expr(cg.clone(), node.right[0 as usize].clone(), tenv.clone());
            codegen_emit(cg.clone(), OP_ADD)
        }
        return;
    }

    codegen_expr(cg.clone(), node.left[0 as usize].clone(), tenv.clone());
    codegen_expr(cg.clone(), node.right[0 as usize].clone(), tenv.clone());

    if op == "-" {
        codegen_emit(cg.clone(), OP_SUB);
        return;
    }
    if op == "*" {
        codegen_emit(cg.clone(), OP_MUL);
        return;
    }
    if op == "/" {
        codegen_emit(cg.clone(), OP_DIV);
        return;
    }
    if op == "%" {
        codegen_emit(cg.clone(), OP_MOD);
        return;
    }

    if op == "==" {
        codegen_emit(cg.clone(), OP_EQ);
        return;
    }
    if op == "!=" {
        codegen_emit(cg.clone(), OP_NE);
        return;
    }
    if op == "<" {
        codegen_emit(cg.clone(), OP_LT);
        return;
    }
    if op == ">" {
        codegen_emit(cg.clone(), OP_GT);
        return;
    }
    if op == "<=" {
        codegen_emit(cg.clone(), OP_LE);
        return;
    }
    if op == ">=" {
        codegen_emit(cg.clone(), OP_GE);
        return;
    }

    if op == "||" {
        codegen_emit(cg.clone(), OP_CONST_0);
        codegen_emit(cg.clone(), OP_NE);
        codegen_emit(cg.clone(), OP_DUP);
        let mut jmp_early = codegen_offset(cg.clone());
        codegen_emit(cg.clone(), OP_JMP_IF_Z);
        codegen_emit_u16(cg.clone(), 0);
        codegen_emit(cg.clone(), OP_POP);
        codegen_emit(cg.clone(), OP_CONST_1);
        codegen_patch_u16(cg.clone(), jmp_early + 1, codegen_offset(cg.clone()) - jmp_early - 3);
        return;
    }
    if op == "&&" {
        codegen_emit(cg.clone(), OP_CONST_0);
        codegen_emit(cg.clone(), OP_NE);
        codegen_emit(cg.clone(), OP_DUP);
        let mut jmp_short = codegen_offset(cg.clone());
        codegen_emit(cg.clone(), OP_JMP_IF_Z);
        codegen_emit_u16(cg.clone(), 0);
        codegen_emit(cg.clone(), OP_POP);
        codegen_expr(cg.clone(), node.right[0 as usize].clone(), tenv.clone());
        codegen_emit(cg.clone(), OP_CONST_0);
        codegen_emit(cg.clone(), OP_NE);
        codegen_patch_u16(cg.clone(), jmp_short + 1, codegen_offset(cg.clone()) - jmp_short - 3);
        return;
    }

    return;
}

fn codegen_extract_method_suffix(mut callee: &str, mut suffix: &str) -> i32 {

    let mut sn = (suffix.len() as i32);
    let mut cn = (callee.len() as i32);
    if cn < sn {
        return 0;
    }
    let mut start = cn - sn;
    let mut i: i32 = 0;
    while i < sn {
        if callee.chars().nth(start + i as usize).unwrap_or('\0') as i32 != suffix.chars().nth(i as usize).unwrap_or('\0') as i32 {
            return 0;
        }
        i = i + 1;
    }
    return 1;
}

fn codegen_extract_var_name(mut callee: &str) -> String {

    let mut i: i32 = (callee.len() as i32) - 1;
    while i >= 0 {
        if callee.chars().nth(i as usize).unwrap_or('\0') as i32 == 46 {
            return a2r_std::str_substr(callee, 0, i);
        }
        i = i - 1;
    }
    return callee.to_string();
}

fn codegen_call(mut cg: CodeGen, mut node: ASTNode, mut tenv: TypeEnv) {
    let mut callee = node.name;


    if callee == "List.new" {
        codegen_emit(cg.clone(), OP_LIST_NEW);
        return;
    }
    if callee == "Map.new" {
        codegen_emit(cg.clone(), OP_MAP_NEW);
        return;
    }


    let mut is_list_map_method: i32 = 0;
    if codegen_extract_method_suffix(callee.as_str(), ".push") == 1 {
        is_list_map_method = 1
    }
    if is_list_map_method == 0 && codegen_extract_method_suffix(callee.as_str(), ".get") == 1 {
        is_list_map_method = 1
    }
    if is_list_map_method == 0 && codegen_extract_method_suffix(callee.as_str(), ".len") == 1 {
        is_list_map_method = 1
    }
    if is_list_map_method == 0 && codegen_extract_method_suffix(callee.as_str(), ".insert_int") == 1 {
        is_list_map_method = 1
    }
    if is_list_map_method == 0 && codegen_extract_method_suffix(callee.as_str(), ".get_int") == 1 {
        is_list_map_method = 1
    }
    if is_list_map_method == 0 && codegen_extract_method_suffix(callee.as_str(), ".contains") == 1 {
        is_list_map_method = 1
    }
    if is_list_map_method == 0 && codegen_extract_method_suffix(callee.as_str(), ".insert_str") == 1 {
        is_list_map_method = 1
    }
    if is_list_map_method == 0 && codegen_extract_method_suffix(callee.as_str(), ".get_str") == 1 {
        is_list_map_method = 1
    }
    if is_list_map_method == 0 && codegen_extract_method_suffix(callee.as_str(), ".set") == 1 {
        is_list_map_method = 1
    }
    if is_list_map_method == 0 && codegen_extract_method_suffix(callee.as_str(), ".pop") == 1 {
        is_list_map_method = 1
    }
    if is_list_map_method == 0 && codegen_extract_method_suffix(callee.as_str(), ".char_at") == 1 {
        is_list_map_method = 1
    }
    if is_list_map_method == 0 && codegen_extract_method_suffix(callee.as_str(), ".substr") == 1 {
        is_list_map_method = 1
    }

    if is_list_map_method == 1 {
        let mut var_name = codegen_extract_var_name(callee.as_str());
        let mut slot = codegen_find_local(cg.clone(), var_name.as_str());
        if slot >= 0 {
            codegen_emit(cg.clone(), OP_LOAD_LOCAL);
            codegen_emit(cg.clone(), slot);
            let mut i: i32 = 0;
            let mut pn = (node.params.len() as i32);
            while i < pn {
                codegen_expr(cg.clone(), node.params[i as usize].clone(), tenv.clone());
                i = i + 1;
            }
            

            if codegen_extract_method_suffix(callee.as_str(), ".push") == 1 {
                let mut et: i32 = 0;
                if (node.params.len() as i32) > 0 {
                    et = type_infer_expr(tenv, node.params[0 as usize].clone())
                }                if et == 1 {
                    codegen_emit(cg.clone(), OP_LIST_PUSH_STR);
                    codegen_record_elem(cg.clone(), var_name.as_str(), 1)
                } else {
                    codegen_emit(cg.clone(), OP_LIST_PUSH)
                }
                codegen_emit(cg.clone(), OP_CONST_0);
                return;
            }            if codegen_extract_method_suffix(callee.as_str(), ".get") == 1 {
                let mut et = codegen_lookup_elem(cg.clone(), var_name.as_str());
                if et == 1 {
                    codegen_emit(cg.clone(), OP_LIST_GET_STR)
                } else {
                    codegen_emit(cg.clone(), OP_LIST_GET)
                }
                return;
            }            if codegen_extract_method_suffix(callee.as_str(), ".len") == 1 {
                let mut len_type = type_lookup(tenv, &(var_name));
                if len_type == 1 {
                    codegen_emit(cg.clone(), OP_STR_LEN)
                } else {
                    codegen_emit(cg.clone(), OP_LIST_LEN)
                }
                return;
            }            if codegen_extract_method_suffix(callee.as_str(), ".insert_int") == 1 {
                codegen_emit(cg.clone(), OP_MAP_INSERT_INT);
                codegen_emit(cg.clone(), OP_CONST_0);
                return;
            }            if codegen_extract_method_suffix(callee.as_str(), ".get_int") == 1 {
                codegen_emit(cg.clone(), OP_MAP_GET_INT);
                return;
            }            if codegen_extract_method_suffix(callee.as_str(), ".contains") == 1 {
                codegen_emit(cg.clone(), OP_MAP_CONTAINS);
                return;
            }            if codegen_extract_method_suffix(callee.as_str(), ".insert_str") == 1 {
                codegen_emit(cg.clone(), OP_MAP_INSERT_STR);
                codegen_emit(cg.clone(), OP_CONST_0);
                return;
            }            if codegen_extract_method_suffix(callee.as_str(), ".get_str") == 1 {
                codegen_emit(cg.clone(), OP_MAP_GET_STR);
                return;
            }            if codegen_extract_method_suffix(callee.as_str(), ".set") == 1 {
                let mut et = codegen_lookup_elem(cg.clone(), var_name.as_str());
                if et == 1 {
                    codegen_emit(cg.clone(), OP_LIST_SET_STR)
                } else {
                    codegen_emit(cg.clone(), OP_LIST_SET)
                }
                codegen_emit(cg.clone(), OP_CONST_0);
                return;
            }            if codegen_extract_method_suffix(callee.as_str(), ".pop") == 1 {
                let mut et = codegen_lookup_elem(cg.clone(), var_name.as_str());
                if et == 1 {
                    codegen_emit(cg.clone(), OP_LIST_POP_STR)
                } else {
                    codegen_emit(cg.clone(), OP_LIST_POP)
                }
                return;
            }            if codegen_extract_method_suffix(callee.as_str(), ".char_at") == 1 {
                codegen_emit(cg.clone(), OP_STR_CHAR_AT);
                return;
            }            if codegen_extract_method_suffix(callee.as_str(), ".substr") == 1 {
                codegen_emit(cg.clone(), OP_STR_SUBSTR);
                return;
            }        }    }

    if callee == "print" {
        if (node.params.len() as i32) == 0 {
            let mut idx = codegen_add_string(cg.clone(), "\n");
            codegen_emit(cg.clone(), OP_LOAD_STR);
            codegen_emit_u16(cg.clone(), idx);
            codegen_emit(cg.clone(), OP_CALL_NAT);
            codegen_emit_u16(cg.clone(), NATIVE_PRINT_STR)
        } else {
            let mut arg_node = node.params[0 as usize].clone();
            let mut arg_type = type_infer_expr(tenv, arg_node.clone());
            

            if arg_type == -1 && arg_node.kind == NodeKind::CallExpr {
                if codegen_str_returning_call(cg.clone(), arg_node.name.as_str()) == 1 {
                    arg_type = 1
                }            }            codegen_expr(cg.clone(), arg_node.clone(), tenv.clone());
            if arg_type == 1 {
                codegen_emit(cg.clone(), OP_CALL_NAT);
                codegen_emit_u16(cg.clone(), NATIVE_PRINT_STR)
            } else {
                codegen_emit(cg.clone(), OP_CALL_NAT);
                codegen_emit_u16(cg.clone(), NATIVE_PRINT_I32)
            }
        }
        let mut nl = codegen_add_string(cg.clone(), "\n");
        codegen_emit(cg.clone(), OP_LOAD_STR);
        codegen_emit_u16(cg.clone(), nl);
        codegen_emit(cg.clone(), OP_CALL_NAT);
        codegen_emit_u16(cg.clone(), NATIVE_PRINT_STR);
        codegen_emit(cg.clone(), OP_CONST_0);
        return;
    }

    if cg.exports.contains_key(&callee) {
        let mut i: i32 = 0;
        let mut pn = (node.params.len() as i32);
        while i < pn {
            codegen_expr(cg.clone(), node.params[i as usize].clone(), tenv.clone());
            i = i + 1;
        }
        codegen_emit(cg.clone(), OP_CALL);
        let mut addr = (cg.exports.get(&callee.to_string()).and_then(|v| v.parse::<i32>().ok()).unwrap_or(0));
        codegen_emit_u32(cg.clone(), addr);
        codegen_emit(cg.clone(), pn);
        return;
    }

    codegen_emit(cg.clone(), OP_CONST_0);
    return;
}

fn codegen_stmt(mut cg: CodeGen, mut node: ASTNode, mut tenv: TypeEnv) {
    let mut kind = node.kind;

    if kind == NodeKind::FnStmt {
        return;
    }

    if kind == NodeKind::LetStmt || kind == NodeKind::VarStmt {
        let mut slot = codegen_alloc_local(cg.clone(), node.name.as_str());
        if (node.left.len() as i32) > 0 {
            let mut init = node.left[0 as usize].clone();
            codegen_expr(cg.clone(), init.clone(), tenv.clone());
            

            if init.kind == NodeKind::CallExpr {
                if codegen_str_returning_call(cg.clone(), init.name.as_str()) == 1 {
                    type_bind(tenv, &(node.name), 1);
                }            }        } else {
            codegen_emit(cg.clone(), OP_CONST_0)
        }
        codegen_emit(cg.clone(), OP_STORE_LOCAL);
        codegen_emit(cg.clone(), slot);
        return;
    }

    if kind == NodeKind::ReturnStmt {
        if (node.left.len() as i32) > 0 {
            codegen_expr(cg.clone(), node.left[0 as usize].clone(), tenv.clone())
        } else {
            codegen_emit(cg.clone(), OP_CONST_0)
        }
        codegen_emit(cg.clone(), OP_RET);
        

        let mut n_args: i32 = 0;
        if cg.exports.contains_key("__current_n_args__") {
            n_args = (cg.exports.get(&"__current_n_args__".to_string()).and_then(|v| v.parse::<i32>().ok()).unwrap_or(0))
        }        codegen_emit(cg.clone(), n_args);
        return;
    }

    if kind == NodeKind::IfStmt {
        codegen_expr(cg.clone(), node.cond[0 as usize].clone(), tenv.clone());
        let mut jmp_else = codegen_offset(cg.clone());
        codegen_emit(cg.clone(), OP_JMP_IF_Z);
        codegen_emit_u16(cg.clone(), 0);
        codegen_stmts(cg.clone(), node.children, tenv.clone());
        let mut jmp_end = codegen_offset(cg.clone());
        codegen_emit(cg.clone(), OP_JMP);
        codegen_emit_u16(cg.clone(), 0);
        codegen_patch_u16(cg.clone(), jmp_else + 1, codegen_offset(cg.clone()) - jmp_else - 3);
        if (node.else_body.len() as i32) > 0 {
            let mut first = node.else_body[0 as usize].clone();
            if first.kind == NodeKind::IfStmt {
                codegen_stmt(cg.clone(), first.clone(), tenv.clone())
            } else {
                codegen_stmts(cg.clone(), node.else_body, tenv.clone())
            }
        }        codegen_patch_u16(cg.clone(), jmp_end + 1, codegen_offset(cg.clone()) - jmp_end - 3);
        return;
    }

    if kind == NodeKind::ForStmt {
        let mut loop_start = codegen_offset(cg.clone());
        codegen_expr(cg.clone(), node.left[0 as usize].clone(), tenv.clone());
        let mut jmp_exit = codegen_offset(cg.clone());
        codegen_emit(cg.clone(), OP_JMP_IF_Z);
        codegen_emit_u16(cg.clone(), 0);
        codegen_stmts(cg.clone(), node.children, tenv.clone());
        codegen_emit(cg.clone(), OP_JMP);
        let mut loop_size = codegen_offset(cg.clone()) - loop_start;
        let mut back_jmp: i32 = loop_size + 2;
        let mut neg_off: i32 = 0 - back_jmp;
        let mut off_u: i32 = neg_off + 65536;
        codegen_emit_byte(cg.clone(), off_u);
        codegen_emit_byte(cg.clone(), off_u / 256);
        codegen_patch_u16(cg.clone(), jmp_exit + 1, codegen_offset(cg.clone()) - jmp_exit - 3);
        return;
    }

    if kind == NodeKind::ForInStmt {
        let mut slot = codegen_alloc_local(cg.clone(), node.name.as_str());
        let mut range_node = node.left[0 as usize].clone();
        codegen_expr(cg.clone(), range_node.left[0 as usize].clone(), tenv.clone());
        codegen_emit(cg.clone(), OP_STORE_LOCAL);
        codegen_emit(cg.clone(), slot);
        let mut end_slot = codegen_alloc_local(cg.clone(), "__forin_end__");
        codegen_expr(cg.clone(), range_node.right[0 as usize].clone(), tenv.clone());
        if range_node.op == "..=" {
            codegen_emit(cg.clone(), OP_CONST_I32);
            codegen_emit_i32(cg.clone(), 1);
            codegen_emit(cg.clone(), OP_ADD);
        }        codegen_emit(cg.clone(), OP_STORE_LOCAL);
        codegen_emit(cg.clone(), end_slot);
        let mut loop_start = codegen_offset(cg.clone());
        codegen_emit(cg.clone(), OP_LOAD_LOCAL);
        codegen_emit(cg.clone(), slot);
        codegen_emit(cg.clone(), OP_LOAD_LOCAL);
        codegen_emit(cg.clone(), end_slot);
        codegen_emit(cg.clone(), OP_LT);
        let mut jmp_exit = codegen_offset(cg.clone());
        codegen_emit(cg.clone(), OP_JMP_IF_Z);
        codegen_emit_u16(cg.clone(), 0);
        codegen_stmts(cg.clone(), node.children, tenv.clone());
        codegen_emit(cg.clone(), OP_LOAD_LOCAL);
        codegen_emit(cg.clone(), slot);
        codegen_emit(cg.clone(), OP_CONST_I32);
        codegen_emit_i32(cg.clone(), 1);
        codegen_emit(cg.clone(), OP_ADD);
        codegen_emit(cg.clone(), OP_STORE_LOCAL);
        codegen_emit(cg.clone(), slot);
        codegen_emit(cg.clone(), OP_JMP);
        let mut loop_size = codegen_offset(cg.clone()) - loop_start;
        let mut back_jmp: i32 = loop_size + 2;
        let mut neg_off: i32 = 0 - back_jmp;
        let mut off_u: i32 = neg_off + 65536;
        codegen_emit_byte(cg.clone(), off_u);
        codegen_emit_byte(cg.clone(), off_u / 256);
        codegen_patch_u16(cg.clone(), jmp_exit + 1, codegen_offset(cg.clone()) - jmp_exit - 3);
        return;
    }

    if kind == NodeKind::ExprStmt {
        codegen_expr(cg.clone(), node.left[0 as usize].clone(), tenv.clone());
        codegen_emit(cg.clone(), OP_POP);
        return;
    }

    if kind == NodeKind::BlockStmt {
        codegen_stmts(cg.clone(), node.children, tenv.clone());
        return;
    }

    return;
}

fn codegen_stmts(mut cg: CodeGen, mut stmts: Vec<ASTNode>, mut tenv: TypeEnv) {
    let mut i: i32 = 0;
    let mut n = (stmts.len() as i32);
    while i < n {
        codegen_stmt(cg.clone(), stmts[i as usize].clone(), tenv.clone());
        i = i + 1;
    }
}

fn codegen_compile(mut cg: CodeGen, mut stmts: Vec<ASTNode>, mut tenv: TypeEnv) {
    let mut header_size: i32 = 7;
    let mut offset: i32 = 0;
    while offset < header_size {
        codegen_emit(cg.clone(), 0);
        offset = offset + 1;
    }

    let mut i: i32 = 0;
    let mut n = (stmts.len() as i32);
    while i < n {
        let mut stmt = stmts[i as usize].clone();
        if stmt.kind == NodeKind::FnStmt {
            let mut fn_addr = codegen_offset(cg.clone());
            cg.exports.insert(stmt.name.to_string(), (fn_addr).to_string());
            cg.locals = std::collections::HashMap::new();
            cg.n_locals = 0;
            cg.elem_types = std::collections::HashMap::new();
            let mut params = stmt.params;
            let mut j: i32 = 0;
            let mut pn = (params.len() as i32);
            while j < pn {
                let mut param = params[j as usize].clone();
                codegen_alloc_local(cg.clone(), param.name.as_str());
                j = j + 1;
            }
            codegen_emit(cg.clone(), OP_RESERVE_STACK);
            codegen_emit(cg.clone(), 0);
            let mut reserve_off: i32 = codegen_offset(cg.clone()) - 1;
            

            cg.exports.insert("__current_n_args__".to_string(), (pn).to_string());
            codegen_stmts(cg.clone(), stmt.children, tenv.clone());
            codegen_emit(cg.clone(), OP_CONST_0);
            codegen_emit(cg.clone(), OP_RET);
            codegen_emit(cg.clone(), pn);
            cg.code.insert(reserve_off as usize, cg.n_locals);
        }
        i = i + 1;
    }

    if cg.exports.contains_key("main") {
        let mut main_addr = (cg.exports.get(&"main".to_string()).and_then(|v| v.parse::<i32>().ok()).unwrap_or(0));
        cg.code.insert(0 as usize, OP_CALL);
        codegen_patch_u32(cg.clone(), 1, main_addr);
        cg.code.insert(5 as usize, 0);
        cg.code.insert(6 as usize, OP_HALT);
    }
}

// === vm ===
// a2r Standard Library (from crate)

fn bvm_init(mut state: std::collections::HashMap<String, String>) {
    state.insert("__ip".to_string(), (0).to_string());
    state.insert("__sp".to_string(), (0).to_string());
    state.insert("__bp".to_string(), (0).to_string());
    state.insert("__csn".to_string(), (0).to_string());
    state.insert("__output".to_string(), ("").to_string());
    state.insert("__nl".to_string(), (0).to_string());
    state.insert("__nm".to_string(), (0).to_string());
}

fn bvm_push_int(mut state: std::collections::HashMap<String, String>, mut val: i32) {
    let mut sp = (state.get(&"__sp".to_string()).and_then(|v| v.parse::<i32>().ok()).unwrap_or(0));
    state.insert(format!("{}{}", "v", int_to_str(sp)).to_string(), (val).to_string());
    state.insert(format!("{}{}", "s", int_to_str(sp)).to_string(), ("").to_string());
    state.insert(format!("{}{}", "t", int_to_str(sp)).to_string(), (0).to_string());
    state.insert("__sp".to_string(), (sp + 1).to_string());
}

fn bvm_push_list_ref(mut state: std::collections::HashMap<String, String>, mut heap_idx: i32) {
    let mut sp = (state.get(&"__sp".to_string()).and_then(|v| v.parse::<i32>().ok()).unwrap_or(0));
    state.insert(format!("{}{}", "v", int_to_str(sp)).to_string(), (heap_idx).to_string());
    state.insert(format!("{}{}", "s", int_to_str(sp)).to_string(), ("").to_string());
    state.insert(format!("{}{}", "t", int_to_str(sp)).to_string(), (2).to_string());
    state.insert("__sp".to_string(), (sp + 1).to_string());
}

fn bvm_push_map_ref(mut state: std::collections::HashMap<String, String>, mut heap_idx: i32) {
    let mut sp = (state.get(&"__sp".to_string()).and_then(|v| v.parse::<i32>().ok()).unwrap_or(0));
    state.insert(format!("{}{}", "v", int_to_str(sp)).to_string(), (heap_idx).to_string());
    state.insert(format!("{}{}", "s", int_to_str(sp)).to_string(), ("").to_string());
    state.insert(format!("{}{}", "t", int_to_str(sp)).to_string(), (3).to_string());
    state.insert("__sp".to_string(), (sp + 1).to_string());
}

fn bvm_push_str(mut state: std::collections::HashMap<String, String>, mut pool_key: &str) {
    let mut sp = (state.get(&"__sp".to_string()).and_then(|v| v.parse::<i32>().ok()).unwrap_or(0));
    state.insert(format!("{}{}", "v", int_to_str(sp)).to_string(), (0).to_string());
    state.insert(format!("{}{}", "s", int_to_str(sp)).to_string(), (pool_key).to_string());
    state.insert(format!("{}{}", "t", int_to_str(sp)).to_string(), (1).to_string());
    state.insert("__sp".to_string(), (sp + 1).to_string());
}

fn bvm_pop_int(mut state: std::collections::HashMap<String, String>) -> i32 {
    let mut sp = (state.get(&"__sp".to_string()).and_then(|v| v.parse::<i32>().ok()).unwrap_or(0));
    if sp > 0 {
        sp = sp - 1;
        state.insert("__sp".to_string(), (sp).to_string());
        return (state.get(&format!("{}{}", "v", int_to_str(sp)).to_string()).and_then(|v| v.parse::<i32>().ok()).unwrap_or(0));
    }
    return 0;
}

fn bvm_pop_str_key(mut state: std::collections::HashMap<String, String>) -> String {
    let mut sp = (state.get(&"__sp".to_string()).and_then(|v| v.parse::<i32>().ok()).unwrap_or(0));
    if sp > 0 {
        sp = sp - 1;
        state.insert("__sp".to_string(), (sp).to_string());
        return state.get(&format!("{}{}", "s", int_to_str(sp))).cloned().unwrap_or_default();
    }
    return "".to_string();
}

fn bvm_read_u8(mut code: Vec<i32>, mut state: std::collections::HashMap<String, String>) -> i32 {
    let mut ip = (state.get(&"__ip".to_string()).and_then(|v| v.parse::<i32>().ok()).unwrap_or(0));
    let mut val = code[ip as usize].clone();
    state.insert("__ip".to_string(), (ip + 1).to_string());
    return val;
}

fn bvm_read_i32(mut code: Vec<i32>, mut state: std::collections::HashMap<String, String>) -> i32 {
    let mut b0 = bvm_read_u8(code.clone(), state.clone());
    let mut b1 = bvm_read_u8(code.clone(), state.clone());
    let mut b2 = bvm_read_u8(code.clone(), state.clone());
    let mut b3 = bvm_read_u8(code.clone(), state.clone());
    let mut val: i32 = b0 + b1 * 256 + b2 * 65536 + b3 * 16777216;
    if b3 >= 128 {
        val = val - 4294967296
    }
    return val;
}

fn bvm_read_u16(mut code: Vec<i32>, mut state: std::collections::HashMap<String, String>) -> i32 {
    let mut b0 = bvm_read_u8(code.clone(), state.clone());
    let mut b1 = bvm_read_u8(code.clone(), state.clone());
    return b0 + b1 * 256;
}

fn bvm_read_i16(mut code: Vec<i32>, mut state: std::collections::HashMap<String, String>) -> i32 {
    let mut b0 = bvm_read_u8(code.clone(), state.clone());
    let mut b1 = bvm_read_u8(code.clone(), state.clone());
    let mut val: i32 = b0 + b1 * 256;
    if b1 >= 128 {
        val = val - 65536
    }
    return val;
}

fn bvm_read_u32(mut code: Vec<i32>, mut state: std::collections::HashMap<String, String>) -> i32 {
    let mut b0 = bvm_read_u8(code.clone(), state.clone());
    let mut b1 = bvm_read_u8(code.clone(), state.clone());
    let mut b2 = bvm_read_u8(code.clone(), state.clone());
    let mut b3 = bvm_read_u8(code.clone(), state.clone());
    let mut val: i32 = b0 + b1 * 256 + b2 * 65536 + b3 * 16777216;
    if b3 >= 128 {
        val = val - 4294967296
    }
    return val;
}

fn bvm_run(mut code: Vec<i32>, mut strings: std::collections::HashMap<String, String>, mut state: std::collections::HashMap<String, String>) -> String {
    let mut running: i32 = 1;
    while running == 1 {
        running = bvm_step(code.clone(), strings.clone(), state);
    }
    return state.get("__output").cloned().unwrap_or_default();
}

fn bvm_step(mut code: Vec<i32>, mut strings: std::collections::HashMap<String, String>, mut state: std::collections::HashMap<String, String>) -> i32 {
    let mut op = bvm_read_u8(code.clone(), state.clone());


    if op == 255 {
        return 0;
    }


    if op == 1 {
        bvm_pop_int(state.clone());
        return 1;
    }


    if op == 3 {
        let mut sp = (state.get(&"__sp".to_string()).and_then(|v| v.parse::<i32>().ok()).unwrap_or(0));
        if sp > 0 {
            let mut t = (state.get(&format!("{}{}", "t", int_to_str(sp - 1)).to_string()).and_then(|v| v.parse::<i32>().ok()).unwrap_or(0));
            if t == 1 {
                bvm_push_str(state.clone(), state.get(&format!("{}{}", "s", int_to_str(sp - 1))).cloned().unwrap_or_default().as_str())
            } else if t == 2 {
                bvm_push_list_ref(state.clone(), (state.get(&format!("{}{}", "v", int_to_str(sp - 1)).to_string()).and_then(|v| v.parse::<i32>().ok()).unwrap_or(0)))
            } else if t == 3 {
                bvm_push_map_ref(state.clone(), (state.get(&format!("{}{}", "v", int_to_str(sp - 1)).to_string()).and_then(|v| v.parse::<i32>().ok()).unwrap_or(0)))
            } else {
                bvm_push_int(state.clone(), (state.get(&format!("{}{}", "v", int_to_str(sp - 1)).to_string()).and_then(|v| v.parse::<i32>().ok()).unwrap_or(0)))
            }
        }        return 1;
    }


    if op == 16 {
        bvm_push_int(state.clone(), bvm_read_i32(code.clone(), state.clone()));
        return 1;
    }


    if op == 18 {
        bvm_push_int(state.clone(), 0);
        return 1;
    }


    if op == 19 {
        bvm_push_int(state.clone(), 1);
        return 1;
    }


    if op == 31 {
        let mut idx = bvm_read_u16(code.clone(), state.clone());
        bvm_push_str(state.clone(), int_to_str(idx).as_str());
        return 1;
    }


    if op == 6 {
        let mut n = bvm_read_u8(code.clone(), state.clone());
        let mut i: i32 = 0;
        while i < n {
            bvm_push_int(state.clone(), 0);
            i = i + 1;
        }
        return 1;
    }


    if op == 32 {
        let mut slot = bvm_read_u8(code.clone(), state.clone());
        let mut bp = (state.get(&"__bp".to_string()).and_then(|v| v.parse::<i32>().ok()).unwrap_or(0));
        let mut sp = (state.get(&"__sp".to_string()).and_then(|v| v.parse::<i32>().ok()).unwrap_or(0));
        let mut abs_idx = bp + slot;
        if abs_idx < sp {
            let mut t = (state.get(&format!("{}{}", "t", int_to_str(abs_idx)).to_string()).and_then(|v| v.parse::<i32>().ok()).unwrap_or(0));
            if t == 1 {
                bvm_push_str(state.clone(), state.get(&format!("{}{}", "s", int_to_str(abs_idx))).cloned().unwrap_or_default().as_str())
            } else if t == 2 {
                bvm_push_list_ref(state.clone(), (state.get(&format!("{}{}", "v", int_to_str(abs_idx)).to_string()).and_then(|v| v.parse::<i32>().ok()).unwrap_or(0)))
            } else if t == 3 {
                bvm_push_map_ref(state.clone(), (state.get(&format!("{}{}", "v", int_to_str(abs_idx)).to_string()).and_then(|v| v.parse::<i32>().ok()).unwrap_or(0)))
            } else {
                bvm_push_int(state.clone(), (state.get(&format!("{}{}", "v", int_to_str(abs_idx)).to_string()).and_then(|v| v.parse::<i32>().ok()).unwrap_or(0)))
            }
        } else {
            bvm_push_int(state.clone(), 0)
        }
        return 1;
    }




    if op == 33 {
        let mut slot = bvm_read_u8(code.clone(), state.clone());
        let mut sp = (state.get(&"__sp".to_string()).and_then(|v| v.parse::<i32>().ok()).unwrap_or(0));
        if sp > 0 {
            sp = sp - 1;
            state.insert("__sp".to_string(), (sp).to_string());
            let mut val = (state.get(&format!("{}{}", "v", int_to_str(sp)).to_string()).and_then(|v| v.parse::<i32>().ok()).unwrap_or(0));
            let mut s = state.get(&format!("{}{}", "s", int_to_str(sp))).cloned().unwrap_or_default();
            let mut t = (state.get(&format!("{}{}", "t", int_to_str(sp)).to_string()).and_then(|v| v.parse::<i32>().ok()).unwrap_or(0));
            let mut bp = (state.get(&"__bp".to_string()).and_then(|v| v.parse::<i32>().ok()).unwrap_or(0));
            let mut abs_idx = bp + slot;
            state.insert(format!("{}{}", "v", int_to_str(abs_idx)).to_string(), (val).to_string());
            state.insert(format!("{}{}", "s", int_to_str(abs_idx)).to_string(), (s).to_string());
            state.insert(format!("{}{}", "t", int_to_str(abs_idx)).to_string(), (t).to_string());
        }        return 1;
    }


    if op == 48 {
        let mut r = bvm_pop_int(state.clone());
        let mut l = bvm_pop_int(state.clone());
        bvm_push_int(state.clone(), l + r);
        return 1;
    }

    if op == 49 {
        let mut r = bvm_pop_int(state.clone());
        let mut l = bvm_pop_int(state.clone());
        bvm_push_int(state.clone(), l - r);
        return 1;
    }

    if op == 50 {
        let mut r = bvm_pop_int(state.clone());
        let mut l = bvm_pop_int(state.clone());
        bvm_push_int(state.clone(), l * r);
        return 1;
    }

    if op == 51 {
        let mut r = bvm_pop_int(state.clone());
        let mut l = bvm_pop_int(state.clone());
        if r == 0 {
            bvm_push_int(state.clone(), 0)
        } else {
            bvm_push_int(state.clone(), l / r)
        }
        return 1;
    }

    if op == 52 {
        let mut r = bvm_pop_int(state.clone());
        let mut l = bvm_pop_int(state.clone());
        if r == 0 {
            bvm_push_int(state.clone(), 0)
        } else {
            bvm_push_int(state.clone(), l % r)
        }
        return 1;
    }


    if op == 80 {
        let mut r = bvm_pop_int(state.clone());
        let mut l = bvm_pop_int(state.clone());
        if l == r {
            bvm_push_int(state.clone(), 1)
        } else {
            bvm_push_int(state.clone(), 0)
        }
        return 1;
    }

    if op == 81 {
        let mut r = bvm_pop_int(state.clone());
        let mut l = bvm_pop_int(state.clone());
        if l != r {
            bvm_push_int(state.clone(), 1)
        } else {
            bvm_push_int(state.clone(), 0)
        }
        return 1;
    }

    if op == 82 {
        let mut r = bvm_pop_int(state.clone());
        let mut l = bvm_pop_int(state.clone());
        if l < r {
            bvm_push_int(state.clone(), 1)
        } else {
            bvm_push_int(state.clone(), 0)
        }
        return 1;
    }

    if op == 83 {
        let mut r = bvm_pop_int(state.clone());
        let mut l = bvm_pop_int(state.clone());
        if l > r {
            bvm_push_int(state.clone(), 1)
        } else {
            bvm_push_int(state.clone(), 0)
        }
        return 1;
    }

    if op == 84 {
        let mut r = bvm_pop_int(state.clone());
        let mut l = bvm_pop_int(state.clone());
        if l <= r {
            bvm_push_int(state.clone(), 1)
        } else {
            bvm_push_int(state.clone(), 0)
        }
        return 1;
    }

    if op == 85 {
        let mut r = bvm_pop_int(state.clone());
        let mut l = bvm_pop_int(state.clone());
        if l >= r {
            bvm_push_int(state.clone(), 1)
        } else {
            bvm_push_int(state.clone(), 0)
        }
        return 1;
    }


    if op == 96 {
        let mut offset = bvm_read_i16(code.clone(), state.clone());
        let mut ip = (state.get(&"__ip".to_string()).and_then(|v| v.parse::<i32>().ok()).unwrap_or(0));
        state.insert("__ip".to_string(), (ip + offset).to_string());
        return 1;
    }


    if op == 97 {
        let mut offset = bvm_read_i16(code.clone(), state.clone());
        let mut cond = bvm_pop_int(state.clone());
        if cond == 0 {
            let mut ip = (state.get(&"__ip".to_string()).and_then(|v| v.parse::<i32>().ok()).unwrap_or(0));
            state.insert("__ip".to_string(), (ip + offset).to_string());
        }        return 1;
    }


    if op == 112 {
        let mut target = bvm_read_u32(code.clone(), state.clone());
        let mut n_args = bvm_read_u8(code.clone(), state.clone());
        let mut ip = (state.get(&"__ip".to_string()).and_then(|v| v.parse::<i32>().ok()).unwrap_or(0));
        let mut bp = (state.get(&"__bp".to_string()).and_then(|v| v.parse::<i32>().ok()).unwrap_or(0));
        let mut sp = (state.get(&"__sp".to_string()).and_then(|v| v.parse::<i32>().ok()).unwrap_or(0));
        let mut csn = (state.get(&"__csn".to_string()).and_then(|v| v.parse::<i32>().ok()).unwrap_or(0));
        state.insert(format!("{}{}", "c", int_to_str(csn)).to_string(), (ip).to_string());
        state.insert(format!("{}{}", "c", int_to_str(csn + 1)).to_string(), (bp).to_string());
        state.insert(format!("{}{}", "c", int_to_str(csn + 2)).to_string(), (n_args).to_string());
        state.insert("__csn".to_string(), (csn + 3).to_string());
        state.insert("__bp".to_string(), (sp - n_args).to_string());
        state.insert("__ip".to_string(), (target).to_string());
        return 1;
    }


    if op == 113 {
        let mut n_args = bvm_read_u8(code.clone(), state.clone());
        let mut ret_val: i32 = 0;
        let mut ret_str_key: String = "".to_string();
        let mut ret_type: i32 = 0;
        let mut sp = (state.get(&"__sp".to_string()).and_then(|v| v.parse::<i32>().ok()).unwrap_or(0));
        if sp > 0 {
            sp = sp - 1;
            ret_val = (state.get(&format!("{}{}", "v", int_to_str(sp)).to_string()).and_then(|v| v.parse::<i32>().ok()).unwrap_or(0));
            ret_str_key = state.get(&format!("{}{}", "s", int_to_str(sp))).cloned().unwrap_or_default();
            ret_type = (state.get(&format!("{}{}", "t", int_to_str(sp)).to_string()).and_then(|v| v.parse::<i32>().ok()).unwrap_or(0));
            state.insert("__sp".to_string(), (sp).to_string());
        }        let mut csn = (state.get(&"__csn".to_string()).and_then(|v| v.parse::<i32>().ok()).unwrap_or(0));
        if csn >= 3 {
            csn = csn - 3;
            let mut saved_nargs = (state.get(&format!("{}{}", "c", int_to_str(csn + 2)).to_string()).and_then(|v| v.parse::<i32>().ok()).unwrap_or(0));
            let mut saved_bp = (state.get(&format!("{}{}", "c", int_to_str(csn + 1)).to_string()).and_then(|v| v.parse::<i32>().ok()).unwrap_or(0));
            let mut saved_ip = (state.get(&format!("{}{}", "c", int_to_str(csn)).to_string()).and_then(|v| v.parse::<i32>().ok()).unwrap_or(0));
            state.insert("__csn".to_string(), (csn).to_string());
            let mut bp = (state.get(&"__bp".to_string()).and_then(|v| v.parse::<i32>().ok()).unwrap_or(0));
            let mut target_sp = bp;
            sp = (state.get(&"__sp".to_string()).and_then(|v| v.parse::<i32>().ok()).unwrap_or(0));
            while sp > target_sp {
                sp = sp - 1;
                state.insert("__sp".to_string(), (sp).to_string());
            }
            state.insert("__bp".to_string(), (saved_bp).to_string());
            state.insert("__ip".to_string(), (saved_ip).to_string());
        }        if ret_type == 1 {
            bvm_push_str(state.clone(), ret_str_key.as_str())
        } else if ret_type == 2 {
            bvm_push_list_ref(state.clone(), ret_val)
        } else if ret_type == 3 {
            bvm_push_map_ref(state.clone(), ret_val)
        } else {
            bvm_push_int(state.clone(), ret_val)
        }
        return 1;
    }


    if op == 114 {
        let mut nat_id = bvm_read_u16(code.clone(), state.clone());
        

        if nat_id == 1 {
            let mut val = bvm_pop_int(state.clone());
            let mut output = state.get("__output").cloned().unwrap_or_default();
            state.insert("__output".to_string(), (output + int_to_str(val)).to_string());
        }        

        if nat_id == 3 {
            let mut key = bvm_pop_str_key(state.clone());
            let mut s: String = "".to_string();
            if strings.contains_key(&key) {
                s = strings.get(&*key).cloned().unwrap_or_default()
            }            let mut output = state.get("__output").cloned().unwrap_or_default();
            state.insert("__output".to_string(), (format!("{}{}", output, s)).to_string());
        }        return 1;
    }


    if op == 124 {
        let mut rkey = bvm_pop_str_key(state.clone());
        let mut lkey = bvm_pop_str_key(state.clone());
        let mut l_str: String = "".to_string();
        let mut r_str: String = "".to_string();
        if strings.contains_key(&lkey) {
            l_str = strings.get(&*lkey).cloned().unwrap_or_default()
        }        if strings.contains_key(&rkey) {
            r_str = strings.get(&*rkey).cloned().unwrap_or_default()
        }        let mut result: String = format!("{}{}", l_str, r_str);
        let mut n = (state.get(&"__nstr".to_string()).and_then(|v| v.parse::<i32>().ok()).unwrap_or(0));
        strings.insert(int_to_str(n).to_string(), (result).to_string());
        state.insert("__nstr".to_string(), (n + 1).to_string());
        bvm_push_str(state.clone(), int_to_str(n).as_str());
        return 1;
    }


    if op == 64 {
        let mut nl = (state.get(&"__nl".to_string()).and_then(|v| v.parse::<i32>().ok()).unwrap_or(0));
        state.insert(format!("{}{}", format!("{}{}", "__l", int_to_str(nl)), "_n").to_string(), (0).to_string());
        bvm_push_list_ref(state.clone(), nl);
        state.insert("__nl".to_string(), (nl + 1).to_string());
        return 1;
    }


    if op == 65 {
        let mut val = bvm_pop_int(state.clone());
        let mut heap_idx = bvm_pop_int(state.clone());
        let mut nkey: String = format!("{}{}", format!("{}{}", "__l", int_to_str(heap_idx)), "_n");
        let mut n = (state.get(&nkey.to_string()).and_then(|v| v.parse::<i32>().ok()).unwrap_or(0));
        state.insert(format!("{}{}", format!("{}{}", format!("{}{}", "__l", int_to_str(heap_idx)), "_"), int_to_str(n)).to_string(), (val).to_string());
        state.insert(nkey.to_string(), (n + 1).to_string());
        return 1;
    }


    if op == 66 {
        let mut idx = bvm_pop_int(state.clone());
        let mut heap_idx = bvm_pop_int(state.clone());
        let mut ekey: String = format!("{}{}", format!("{}{}", format!("{}{}", "__l", int_to_str(heap_idx)), "_"), int_to_str(idx));
        if state.contains_key(&ekey) {
            bvm_push_int(state.clone(), (state.get(&ekey.to_string()).and_then(|v| v.parse::<i32>().ok()).unwrap_or(0)))
        } else {
            bvm_push_int(state.clone(), 0)
        }
        return 1;
    }


    if op == 67 {
        let mut heap_idx = bvm_pop_int(state.clone());
        let mut nkey: String = format!("{}{}", format!("{}{}", "__l", int_to_str(heap_idx)), "_n");
        bvm_push_int(state.clone(), (state.get(&nkey.to_string()).and_then(|v| v.parse::<i32>().ok()).unwrap_or(0)));
        return 1;
    }


    if op == 68 {
        let mut nm = (state.get(&"__nm".to_string()).and_then(|v| v.parse::<i32>().ok()).unwrap_or(0));
        bvm_push_map_ref(state.clone(), nm);
        state.insert("__nm".to_string(), (nm + 1).to_string());
        return 1;
    }


    if op == 69 {
        let mut val = bvm_pop_int(state.clone());
        let mut key = bvm_pop_int(state.clone());
        let mut heap_idx = bvm_pop_int(state.clone());
        let mut prefix: String = format!("{}{}", format!("{}{}", "__m", int_to_str(heap_idx)), "_");
        state.insert(format!("{}{}", prefix, int_to_str(key)).to_string(), (val).to_string());
        state.insert(format!("{}{}", format!("{}{}", prefix, "h"), int_to_str(key)).to_string(), (1).to_string());
        return 1;
    }


    if op == 70 {
        let mut key = bvm_pop_int(state.clone());
        let mut heap_idx = bvm_pop_int(state.clone());
        let mut vkey: String = format!("{}{}", format!("{}{}", format!("{}{}", "__m", int_to_str(heap_idx)), "_"), int_to_str(key));
        if state.contains_key(&vkey) {
            bvm_push_int(state.clone(), (state.get(&vkey.to_string()).and_then(|v| v.parse::<i32>().ok()).unwrap_or(0)))
        } else {
            bvm_push_int(state.clone(), 0)
        }
        return 1;
    }


    if op == 71 {
        let mut key = bvm_pop_int(state.clone());
        let mut heap_idx = bvm_pop_int(state.clone());
        let mut hkey: String = format!("{}{}", format!("{}{}", format!("{}{}", "__m", int_to_str(heap_idx)), "_h"), int_to_str(key));
        if state.contains_key(&hkey) {
            bvm_push_int(state.clone(), 1)
        } else {
            bvm_push_int(state.clone(), 0)
        }
        return 1;
    }


    if op == 72 {
        let mut val_str_key = bvm_pop_str_key(state.clone());
        let mut key_str_key = bvm_pop_str_key(state.clone());
        let mut heap_idx = bvm_pop_int(state.clone());
        let mut key_str: String = "".to_string();
        if strings.contains_key(&key_str_key) {
            key_str = strings.get(&*key_str_key).cloned().unwrap_or_default()
        }        let mut val_str: String = "".to_string();
        if strings.contains_key(&val_str_key) {
            val_str = strings.get(&*val_str_key).cloned().unwrap_or_default()
        }        let mut prefix: String = format!("{}{}", format!("{}{}", "__m", int_to_str(heap_idx)), "_");
        state.insert(format!("{}{}", format!("{}{}", prefix, "sv_"), key_str).to_string(), (val_str).to_string());
        state.insert(format!("{}{}", format!("{}{}", prefix, "sh_"), key_str).to_string(), (1).to_string());
        return 1;
    }


    if op == 73 {
        let mut key_str_key = bvm_pop_str_key(state.clone());
        let mut heap_idx = bvm_pop_int(state.clone());
        let mut key_str: String = "".to_string();
        if strings.contains_key(&key_str_key) {
            key_str = strings.get(&*key_str_key).cloned().unwrap_or_default()
        }        let mut vkey: String = format!("{}{}", format!("{}{}", format!("{}{}", "__m", int_to_str(heap_idx)), "_sv_"), key_str);
        if state.contains_key(&vkey) {
            let mut result = state.get(&*vkey).cloned().unwrap_or_default();
            let mut n = (state.get(&"__nstr".to_string()).and_then(|v| v.parse::<i32>().ok()).unwrap_or(0));
            strings.insert(int_to_str(n).to_string(), (result).to_string());
            state.insert("__nstr".to_string(), (n + 1).to_string());
            bvm_push_str(state.clone(), int_to_str(n).as_str())
        } else {
            let mut n2 = (state.get(&"__nstr".to_string()).and_then(|v| v.parse::<i32>().ok()).unwrap_or(0));
            strings.insert(int_to_str(n2).to_string(), ("").to_string());
            state.insert("__nstr".to_string(), (n2 + 1).to_string());
            bvm_push_str(state.clone(), int_to_str(n2).as_str())
        }
        return 1;
    }


    if op == 74 {
        let mut val = bvm_pop_int(state.clone());
        let mut idx = bvm_pop_int(state.clone());
        let mut heap_idx = bvm_pop_int(state.clone());
        state.insert(format!("{}{}", format!("{}{}", format!("{}{}", "__l", int_to_str(heap_idx)), "_"), int_to_str(idx)).to_string(), (val).to_string());
        return 1;
    }


    if op == 75 {
        let mut heap_idx = bvm_pop_int(state.clone());
        let mut nkey: String = format!("{}{}", format!("{}{}", "__l", int_to_str(heap_idx)), "_n");
        let mut n = (state.get(&nkey.to_string()).and_then(|v| v.parse::<i32>().ok()).unwrap_or(0));
        if n > 0 {
            n = n - 1;
            let mut val = (state.get(&format!("{}{}", format!("{}{}", format!("{}{}", "__l", int_to_str(heap_idx)), "_"), int_to_str(n)).to_string()).and_then(|v| v.parse::<i32>().ok()).unwrap_or(0));
            state.insert(nkey.to_string(), (n).to_string());
            bvm_push_int(state.clone(), val)
        } else {
            bvm_push_int(state.clone(), 0)
        }
        return 1;
    }


    if op == 79 {
        let mut val_str_key = bvm_pop_str_key(state.clone());
        let mut heap_idx = bvm_pop_int(state.clone());
        let mut nkey: String = format!("{}{}", format!("{}{}", "__l", int_to_str(heap_idx)), "_n");
        let mut n = (state.get(&nkey.to_string()).and_then(|v| v.parse::<i32>().ok()).unwrap_or(0));
        let mut val_str: String = "".to_string();
        if strings.contains_key(&val_str_key) {
            val_str = strings.get(&*val_str_key).cloned().unwrap_or_default()
        }        state.insert(format!("{}{}", format!("{}{}", format!("{}{}", "__l", int_to_str(heap_idx)), "_s_"), int_to_str(n)).to_string(), (val_str).to_string());
        state.insert(nkey.to_string(), (n + 1).to_string());
        return 1;
    }


    if op == 86 {
        let mut idx = bvm_pop_int(state.clone());
        let mut heap_idx = bvm_pop_int(state.clone());
        let mut ekey: String = format!("{}{}", format!("{}{}", format!("{}{}", "__l", int_to_str(heap_idx)), "_s_"), int_to_str(idx));
        if state.contains_key(&ekey) {
            let mut result = state.get(&*ekey).cloned().unwrap_or_default();
            let mut n = (state.get(&"__nstr".to_string()).and_then(|v| v.parse::<i32>().ok()).unwrap_or(0));
            strings.insert(int_to_str(n).to_string(), (result).to_string());
            state.insert("__nstr".to_string(), (n + 1).to_string());
            bvm_push_str(state.clone(), int_to_str(n).as_str())
        } else {
            let mut n2 = (state.get(&"__nstr".to_string()).and_then(|v| v.parse::<i32>().ok()).unwrap_or(0));
            strings.insert(int_to_str(n2).to_string(), ("").to_string());
            state.insert("__nstr".to_string(), (n2 + 1).to_string());
            bvm_push_str(state.clone(), int_to_str(n2).as_str())
        }
        return 1;
    }


    if op == 87 {
        let mut val_str_key = bvm_pop_str_key(state.clone());
        let mut idx = bvm_pop_int(state.clone());
        let mut heap_idx = bvm_pop_int(state.clone());
        let mut val_str: String = "".to_string();
        if strings.contains_key(&val_str_key) {
            val_str = strings.get(&*val_str_key).cloned().unwrap_or_default()
        }        state.insert(format!("{}{}", format!("{}{}", format!("{}{}", "__l", int_to_str(heap_idx)), "_s_"), int_to_str(idx)).to_string(), (val_str).to_string());
        return 1;
    }


    if op == 88 {
        let mut heap_idx = bvm_pop_int(state.clone());
        let mut nkey: String = format!("{}{}", format!("{}{}", "__l", int_to_str(heap_idx)), "_n");
        let mut n = (state.get(&nkey.to_string()).and_then(|v| v.parse::<i32>().ok()).unwrap_or(0));
        if n > 0 {
            n = n - 1;
            let mut ekey: String = format!("{}{}", format!("{}{}", format!("{}{}", "__l", int_to_str(heap_idx)), "_s_"), int_to_str(n));
            let mut val_str: String = "".to_string();
            if state.contains_key(&ekey) {
                val_str = state.get(&*ekey).cloned().unwrap_or_default()
            }            state.insert(nkey.to_string(), (n).to_string());
            let mut ns = (state.get(&"__nstr".to_string()).and_then(|v| v.parse::<i32>().ok()).unwrap_or(0));
            strings.insert(int_to_str(ns).to_string(), (val_str).to_string());
            state.insert("__nstr".to_string(), (ns + 1).to_string());
            bvm_push_str(state.clone(), int_to_str(ns).as_str())
        } else {
            let mut ns2 = (state.get(&"__nstr".to_string()).and_then(|v| v.parse::<i32>().ok()).unwrap_or(0));
            strings.insert(int_to_str(ns2).to_string(), ("").to_string());
            state.insert("__nstr".to_string(), (ns2 + 1).to_string());
            bvm_push_str(state.clone(), int_to_str(ns2).as_str())
        }
        return 1;
    }


    if op == 76 {
        let mut pool_key = bvm_pop_str_key(state.clone());
        let mut s: String = "".to_string();
        if strings.contains_key(&pool_key) {
            s = strings.get(&*pool_key).cloned().unwrap_or_default()
        }        bvm_push_int(state.clone(), (s.len() as i32));
        return 1;
    }


    if op == 77 {
        let mut idx = bvm_pop_int(state.clone());
        let mut pool_key = bvm_pop_str_key(state.clone());
        let mut s: String = "".to_string();
        if strings.contains_key(&pool_key) {
            s = strings.get(&*pool_key).cloned().unwrap_or_default()
        }        bvm_push_int(state.clone(), s.chars().nth(idx as usize).unwrap_or('\0') as i32);
        return 1;
    }


    if op == 78 {
        let mut end_idx = bvm_pop_int(state.clone());
        let mut start_idx = bvm_pop_int(state.clone());
        let mut pool_key = bvm_pop_str_key(state.clone());
        let mut s: String = "".to_string();
        if strings.contains_key(&pool_key) {
            s = strings.get(&*pool_key).cloned().unwrap_or_default()
        }        let mut result = a2r_std::str_substr(s.as_str(), start_idx, end_idx);
        let mut n = (state.get(&"__nstr".to_string()).and_then(|v| v.parse::<i32>().ok()).unwrap_or(0));
        strings.insert(int_to_str(n).to_string(), (result).to_string());
        state.insert("__nstr".to_string(), (n + 1).to_string());
        bvm_push_str(state.clone(), int_to_str(n).as_str());
        return 1;
    }


    return 0;
}

// === a2r ===
// a2r Standard Library (from crate)

fn a2r_find_lt(mut t: &str) -> i32 {
    let mut i: i32 = 0;
    let mut n = (t.len() as i32);
    while i < n {
        if t.chars().nth(i as usize).unwrap_or('\0') as i32 == 60 {
            return i;
        }
        i = i + 1;
    }
    return -1;
}

fn a2r_type(mut t: &str) -> String {
    if t == "" {
        return "".to_string();
    }
    if t == "int" {
        return "i32".to_string();
    }
    if t == "str" {
        return "String".to_string();
    }
    if t == "bool" {
        return "bool".to_string();
    }
    if t == "void" {
        return "".to_string();
    }
    if t == "uint" {
        return "u32".to_string();
    }
    if t == "i64" {
        return "i64".to_string();
    }
    if t == "f64" {
        return "f64".to_string();
    }
    if t == "f32" {
        return "f32".to_string();
    }
    if t == "byte" {
        return "u8".to_string();
    }
    if t == "List" {
        return "Vec<i32>".to_string();
    }
    if t == "Map" {
        return "std::collections::HashMap<String, i32>".to_string();
    }


    let mut lt = a2r_find_lt(&(t));
    if lt < 0 {
        return t.to_string();
    }
    let mut base = a2r_std::str_substr(t, 0, lt);
    let mut tl = (t.len() as i32);
    let mut inner = a2r_std::str_substr(t, lt + 1, tl - 1);
    if base == "List" {
        return format!("{}{}", format!("{}{}", "Vec<", a2r_type(inner.as_str())), ">");
    }
    if base == "Map" {
        

        let mut ci: i32 = 0;
        let mut cn = (inner.len() as i32);
        let mut comma_pos: i32 = -1;
        while ci < cn {
            if inner.chars().nth(ci as usize).unwrap_or('\0') as i32 == 44 {
                

                comma_pos = ci;
                ci = cn
            } else {
                ci = ci + 1
            }

        }
        if comma_pos >= 0 {
            let mut k = a2r_std::str_substr(inner.as_str(), 0, comma_pos);
            let mut v = a2r_std::str_substr(inner.as_str(), comma_pos + 2, cn);
            return format!("{}{}", format!("{}{}", format!("{}{}", format!("{}{}", "std::collections::HashMap<", a2r_type(k.as_str())), ", "), a2r_type(v.as_str())), ">");
        }        return format!("{}{}", format!("{}{}", "std::collections::HashMap<", a2r_type(inner.as_str())), ">");
    }
    return format!("{}{}", format!("{}{}", format!("{}{}", base, "<"), a2r_type(inner.as_str())), ">");
}

fn a2r_type_from_tag(mut ttag: i32) -> String {
    if ttag == 0 {
        return "i32".to_string();
    }
    if ttag == 1 {
        return "String".to_string();
    }
    if ttag == 2 {
        return "bool".to_string();
    }
    if ttag == 3 {
        return "".to_string();
    }
    return "i32".to_string();
}

fn a2r_indent_str(mut level: i32) -> String {
    let mut s: String = "".to_string();
    let mut i: i32 = 0;
    while i < level {
        s = format!("{}{}", s, "    ");
        i = i + 1;
    }
    return s;
}

fn a2r_expr(mut node: ASTNode, mut tenv: TypeEnv) -> String {
    let mut kind = node.kind;

    if kind == NodeKind::IntExpr {
        return node.name;
    }

    if kind == NodeKind::StrExpr {
        return format!("{}{}", format!("{}{}", "\"", node.name), "\".to_string()");
    }

    if kind == NodeKind::BoolExpr {
        return node.name;
    }

    if kind == NodeKind::IdentExpr {
        return node.name;
    }

    if kind == NodeKind::NilNode {
        return "None".to_string();
    }

    if kind == NodeKind::BinExpr {
        let mut op = node.op;
        


        if op == "=" {
            let mut lhs = a2r_expr(node.left[0 as usize].clone(), tenv.clone());
            let mut rhs = a2r_expr(node.right[0 as usize].clone(), tenv.clone());
            return format!("{}{}", format!("{}{}", lhs, " = "), rhs);
        }        


        if op == "+" {
            let mut lt = type_infer_expr(tenv, node.left[0 as usize].clone());
            if lt == 1 {
                let mut l = a2r_expr(node.left[0 as usize].clone(), tenv.clone());
                let mut r = a2r_expr(node.right[0 as usize].clone(), tenv.clone());
                return format!("{}{}", format!("{}{}", format!("{}{}", format!("{}{}", "format!(\"{}{}\", ", l), ", "), r), ")");
            }        }        


        if op == ".." {
            let mut l2 = a2r_expr(node.left[0 as usize].clone(), tenv.clone());
            let mut r2 = a2r_expr(node.right[0 as usize].clone(), tenv.clone());
            return format!("{}{}", format!("{}{}", l2, ".."), r2);
        }        if op == "..=" {
            let mut l3 = a2r_expr(node.left[0 as usize].clone(), tenv.clone());
            let mut r3 = a2r_expr(node.right[0 as usize].clone(), tenv.clone());
            return format!("{}{}", format!("{}{}", l3, "..="), r3);
        }        


        if op == "+=" || op == "-=" || op == "*=" || op == "/=" || op == "%=" {
            let mut la = a2r_expr(node.left[0 as usize].clone(), tenv.clone());
            let mut ra = a2r_expr(node.right[0 as usize].clone(), tenv.clone());
            return format!("{}{}", format!("{}{}", format!("{}{}", format!("{}{}", la, " "), op), " "), ra);
        }        


        if op == "[]" {
            let mut lo = a2r_expr(node.left[0 as usize].clone(), tenv.clone());
            let mut ro = a2r_expr(node.right[0 as usize].clone(), tenv.clone());
            return format!("{}{}", format!("{}{}", format!("{}{}", lo, "["), ro), "]");
        }        


        let mut le = a2r_expr(node.left[0 as usize].clone(), tenv.clone());
        let mut re = a2r_expr(node.right[0 as usize].clone(), tenv.clone());
        return format!("{}{}", format!("{}{}", format!("{}{}", format!("{}{}", format!("{}{}", format!("{}{}", "(", le), " "), op), " "), re), ")");
    }

    if kind == NodeKind::UnaryExpr {
        let mut operand = a2r_expr(node.left[0 as usize].clone(), tenv.clone());
        let mut uop = node.op;
        if uop == "-" {
            return format!("{}{}", format!("{}{}", "(-", operand), ")");
        }        if uop == "!" {
            return format!("{}{}", format!("{}{}", "(!", operand), ")");
        }        return format!("{}{}", format!("{}{}", format!("{}{}", "(", uop), operand), ")");
    }

    if kind == NodeKind::CallExpr {
        let mut callee = node.name;
        


        if callee == "print" {
            if (node.params.len() as i32) == 0 {
                return "println!()".to_string();
            }            if (node.params.len() as i32) == 1 {
                let mut arg = node.params[0 as usize].clone();
                let mut at = type_infer_expr(tenv, arg.clone());
                

                if arg.kind == NodeKind::StrExpr {
                    return format!("{}{}", format!("{}{}", "println!(\"", arg.name), "\")");
                }                let mut arg_str = a2r_expr(arg.clone(), tenv.clone());
                return format!("{}{}", format!("{}{}", "println!(\"{}\", ", arg_str), ")");
            }            

            let mut fmt: String = "".to_string();
            let mut args: String = "".to_string();
            let mut j: i32 = 0;
            let mut pn = (node.params.len() as i32);
            while j < pn {
                if j > 0 {
                    fmt = format!("{}{}", fmt, " ");
                    args = format!("{}{}", args, ", ")
                }
                let mut pa = node.params[j as usize].clone();
                let mut pat = type_infer_expr(tenv, pa.clone());
                if pa.kind == NodeKind::StrExpr {
                    fmt = format!("{}{}", fmt, a2r_expr(pa.clone(), tenv.clone()))
                } else {
                    fmt = format!("{}{}", fmt, "{}");
                    args = format!("{}{}", args, a2r_expr(pa.clone(), tenv.clone()))
                }

                j = j + 1;
            }
            return format!("{}{}", format!("{}{}", format!("{}{}", format!("{}{}", format!("{}{}", "println!(\"", fmt), "\""), ", "), args), ")");
        }        


        if tenv.struct_fields.contains_key(&callee) {
            return a2r_struct_init(callee.as_str(), node.params, tenv.clone());
        }        


        let mut call_args: String = "".to_string();
        let mut k: i32 = 0;
        let mut cn = (node.params.len() as i32);
        while k < cn {
            if k > 0 {
                call_args = format!("{}{}", call_args, ", ")
            }
            call_args = format!("{}{}", call_args, a2r_expr(node.params[k as usize].clone(), tenv.clone()));
            k = k + 1;
        }
        return format!("{}{}", format!("{}{}", format!("{}{}", callee, "("), call_args), ")");
    }

    if kind == NodeKind::DotExpr {
        let mut obj = a2r_expr(node.left[0 as usize].clone(), tenv.clone());
        return format!("{}{}", format!("{}{}", obj, "."), node.name);
    }

    if kind == NodeKind::FStrExpr {
        return a2r_fstr(node.clone(), tenv.clone());
    }

    if kind == NodeKind::ArrayExpr {
        let mut elems: String = "".to_string();
        let mut i: i32 = 0;
        let mut n = (node.params.len() as i32);
        while i < n {
            if i > 0 {
                elems = format!("{}{}", elems, ", ")
            }
            elems = format!("{}{}", elems, a2r_expr(node.params[i as usize].clone(), tenv.clone()));
            i = i + 1;
        }
        return format!("{}{}", format!("{}{}", "vec![", elems), "]");
    }

    if kind == NodeKind::ObjectExpr {
        let mut fields: String = "".to_string();
        let mut i: i32 = 0;
        let mut n = (node.children.len() as i32);
        while i < n {
            if i > 0 {
                fields = format!("{}{}", fields, ", ")
            }
            fields = format!("{}{}", fields, a2r_expr(node.children[i as usize].clone(), tenv.clone()));
            i = i + 1;
        }
        return format!("{}{}", format!("{}{}", "{ ", fields), " }");
    }

    if kind == NodeKind::PairExpr {
        let mut key = node.name;
        let mut val = a2r_expr(node.left[0 as usize].clone(), tenv.clone());
        return format!("{}{}", format!("{}{}", key, ": "), val);
    }

    if kind == NodeKind::ClosureExpr {
        let mut params: String = "".to_string();
        let mut i: i32 = 0;
        let mut pn = (node.params.len() as i32);
        while i < pn {
            if i > 0 {
                params = format!("{}{}", params, ", ")
            }
            params = format!("{}{}", params, node.params[i as usize].clone().name);
            i = i + 1;
        }
        let mut body = a2r_expr(node.left[0 as usize].clone(), tenv.clone());
        return format!("{}{}", format!("{}{}", format!("{}{}", "|", params), "| "), body);
    }

    if kind == NodeKind::ErrorPropagateExpr {
        let mut expr = a2r_expr(node.left[0 as usize].clone(), tenv.clone());
        return format!("{}{}", expr, "?");
    }


    if kind == NodeKind::ViewExpr {
        let mut inner = a2r_expr(node.left[0 as usize].clone(), tenv.clone());
        return format!("{}{}", "&", inner);
    }

    if kind == NodeKind::MutExpr {
        let mut inner2 = a2r_expr(node.left[0 as usize].clone(), tenv.clone());
        return format!("{}{}", "&mut ", inner2);
    }

    if kind == NodeKind::MoveExpr {
        

        return a2r_expr(node.left[0 as usize].clone(), tenv.clone());
    }

    return format!("{}{}", format!("{}{}", "/* expr:", int_to_str(kind)), " */");
}

fn a2r_body(mut stmts: Vec<ASTNode>, mut tenv: TypeEnv, mut indent: i32) -> String {
    let mut out: String = "".to_string();
    let mut i: i32 = 0;
    let mut n = (stmts.len() as i32);
    while i < n {
        out = format!("{}{}", out, a2r_stmt(stmts[i as usize].clone(), tenv.clone(), indent));
        i = i + 1;
    }
    return out;
}

fn a2r_stmt(mut node: ASTNode, mut tenv: TypeEnv, mut indent: i32) -> String {
    let mut kind = node.kind;
    let mut prefix = a2r_indent_str(indent);

    if kind == NodeKind::ExprStmt {
        if (node.left.len() as i32) > 0 {
            return format!("{}{}", prefix + a2r_expr(node.left[0 as usize].clone(), tenv.clone()), ";\n");
        }        return "".to_string();
    }

    if kind == NodeKind::ReturnStmt {
        if (node.left.len() as i32) > 0 {
            return format!("{}{}", format!("{}{}", format!("{}{}", prefix, "return "), a2r_expr(node.left[0 as usize].clone(), tenv.clone())), ";\n");
        }        return format!("{}{}", prefix, "return;\n");
    }

    if kind == NodeKind::LetStmt || kind == NodeKind::VarStmt {
        let mut type_str = a2r_type(node.type_name.as_str());
        let mut expr_str: String = "".to_string();
        if (node.left.len() as i32) > 0 {
            expr_str = a2r_expr(node.left[0 as usize].clone(), tenv.clone())
        } else {
            expr_str = "0".to_string()
        }
        

        if node.type_name == "" && (node.left.len() as i32) > 0 {
            let mut itag = type_infer_expr(tenv, node.left[0 as usize].clone());
            type_str = a2r_type_from_tag(itag)
        }        

        if type_str == "String" && (node.left.len() as i32) > 0 {
            let mut init = node.left[0 as usize].clone();
            if init.kind != NodeKind::StrExpr && init.kind != NodeKind::CallExpr {
                expr_str = format!("{}{}", a2r_expr(node.left[0 as usize].clone(), tenv.clone()), ".to_string()")
            }        }        let mut kw: String = "let".to_string();
        if kind == NodeKind::VarStmt {
            kw = "let mut".to_string()
        }        

        if type_str == "i32" && node.type_name == "" {
            return format!("{}{}", format!("{}{}", format!("{}{}", format!("{}{}", format!("{}{}", format!("{}{}", prefix, kw), " "), node.name), " = "), expr_str), ";\n");
        }        if type_str == "" {
            return format!("{}{}", format!("{}{}", format!("{}{}", format!("{}{}", format!("{}{}", format!("{}{}", prefix, kw), " "), node.name), " = "), expr_str), ";\n");
        }        return format!("{}{}", format!("{}{}", format!("{}{}", format!("{}{}", format!("{}{}", format!("{}{}", format!("{}{}", format!("{}{}", prefix, kw), " "), node.name), ": "), type_str), " = "), expr_str), ";\n");
    }

    if kind == NodeKind::IfStmt {
        let mut cond = a2r_expr(node.cond[0 as usize].clone(), tenv.clone());
        let mut out2: String = format!("{}{}", format!("{}{}", format!("{}{}", prefix, "if "), cond), " {\n");
        out2 = format!("{}{}", out2, a2r_body(node.children, tenv.clone(), indent + 1));
        if (node.else_body.len() as i32) > 0 {
            out2 = format!("{}{}", format!("{}{}", out2, prefix), "} else {\n");
            out2 = format!("{}{}", out2, a2r_body(node.else_body, tenv.clone(), indent + 1))
        }        out2 = format!("{}{}", format!("{}{}", out2, prefix), "}\n");
        return out2;
    }

    if kind == NodeKind::ForInStmt {
        let mut range = a2r_expr(node.left[0 as usize].clone(), tenv.clone());
        let mut out3: String = format!("{}{}", format!("{}{}", format!("{}{}", format!("{}{}", format!("{}{}", prefix, "for "), node.name), " in "), range), " {\n");
        out3 = format!("{}{}", out3, a2r_body(node.children, tenv.clone(), indent + 1));
        out3 = format!("{}{}", format!("{}{}", out3, prefix), "}\n");
        return out3;
    }

    if kind == NodeKind::ForStmt {
        let mut fcond = a2r_expr(node.left[0 as usize].clone(), tenv.clone());
        let mut out4: String = format!("{}{}", format!("{}{}", format!("{}{}", prefix, "while "), fcond), " {\n");
        out4 = format!("{}{}", out4, a2r_body(node.children, tenv.clone(), indent + 1));
        out4 = format!("{}{}", format!("{}{}", out4, prefix), "}\n");
        return out4;
    }

    if kind == NodeKind::BlockStmt {
        return a2r_body(node.children, tenv.clone(), indent);
    }

    if kind == NodeKind::FnStmt {
        return a2r_fn(node.clone(), tenv.clone(), indent);
    }

    if kind == NodeKind::TypeStmt {
        return a2r_type_decl(node.clone(), tenv.clone(), indent);
    }

    if kind == NodeKind::EnumStmt {
        return a2r_enum_decl(node.clone(), indent);
    }

    if kind == NodeKind::UseStmt {
        return a2r_use(node.clone());
    }

    if kind == NodeKind::IsStmt {
        return a2r_is(node.clone(), tenv.clone(), indent);
    }

    if kind == NodeKind::ExtStmt {
        return a2r_ext(node.clone(), tenv.clone(), indent);
    }

    if kind == NodeKind::SpecStmt {
        return a2r_spec(node.clone(), indent);
    }

    if kind == NodeKind::AliasStmt {
        let mut prefix = a2r_indent_str(indent);
        return format!("{}{}", format!("{}{}", format!("{}{}", format!("{}{}", format!("{}{}", prefix, "type "), node.name), " = "), a2r_type(node.type_name.as_str())), ";\n");
    }

    return format!("{}{}", format!("{}{}", format!("{}{}", prefix, "/* stmt:"), int_to_str(kind)), " */\n");
}

fn a2r_fn(mut node: ASTNode, mut tenv: TypeEnv, mut indent: i32) -> String {
    let mut prefix = a2r_indent_str(indent);


    let mut params_str: String = "".to_string();
    let mut i: i32 = 0;
    let mut pn = (node.params.len() as i32);
    while i < pn {
        if i > 0 {
            params_str = format!("{}{}", params_str, ", ")
        }
        let mut param = node.params[i as usize].clone();
        let mut pt = a2r_type(param.type_name.as_str());
        if pt == "" {
            pt = "i32".to_string()
        }
        if pt == "String" {
            params_str = format!("{}{}", format!("{}{}", params_str, param.name), ": &str")
        } else {
            params_str = format!("{}{}", format!("{}{}", format!("{}{}", params_str, param.name), ": "), pt)
        }

        i = i + 1;
    }


    let mut ret = a2r_type(node.type_name.as_str());
    let mut ret_part: String = "".to_string();
    if ret != "" {
        ret_part = format!("{}{}", " -> ", ret)
    }

    let mut out: String = format!("{}{}", format!("{}{}", format!("{}{}", format!("{}{}", format!("{}{}", format!("{}{}", format!("{}{}", prefix, "fn "), node.name), "("), params_str), ")"), ret_part), " {\n");
    out = format!("{}{}", out, a2r_body(node.children, tenv.clone(), indent + 1));
    out = format!("{}{}", format!("{}{}", out, prefix), "}\n");
    return out;
}

fn a2r_transpile(mut stmts: Vec<ASTNode>, mut tenv: TypeEnv) -> String {
    let mut out: String = "".to_string();
    let mut has_main: i32 = 0;
    let mut top_stmts: String = "".to_string();
    let mut i: i32 = 0;
    let mut n = (stmts.len() as i32);


    while i < n {
        let mut s = stmts[i as usize].clone();
        if s.kind == NodeKind::FnStmt {
            if s.name == "main" {
                has_main = 1
            }        }
        i = i + 1;
    }


    if has_main == 0 {
        

        let mut body_out: String = "".to_string();
        let mut j: i32 = 0;
        while j < n {
            let mut s2 = stmts[j as usize].clone();
            body_out = format!("{}{}", body_out, a2r_stmt(s2.clone(), tenv.clone(), 1));
            j = j + 1;
        }
        out = format!("{}{}", format!("{}{}", "fn main() {\n", body_out), "}\n")
    } else {
        

        let mut k: i32 = 0;
        while k < n {
            let mut s3 = stmts[k as usize].clone();
            out = format!("{}{}", out, a2r_stmt(s3.clone(), tenv.clone(), 0));
            k = k + 1;
        }
    }


    return out;
}

fn run_a2r(mut source: &str) -> String {
    let mut tokens = tokenize_list(&(source));
    let mut p = parser_new(tokens.clone());
    let mut stmts = parse_program(p);
    let mut tenv = typeenv_new();
    type_infer_program(tenv, stmts.clone());
    return a2r_transpile(stmts.clone(), tenv.clone());
}

fn a2r_path_to_rust(mut path: &str) -> String {
    return path.replace(".", "::");
}

fn a2r_type_decl(mut node: ASTNode, mut tenv: TypeEnv, mut indent: i32) -> String {
    let mut prefix = a2r_indent_str(indent);
    let mut out: String = format!("{}{}", prefix, "#[derive(Clone, Debug, PartialEq)]\n");
    out = format!("{}{}", format!("{}{}", format!("{}{}", format!("{}{}", out, prefix), "struct "), node.name), " {\n");
    let mut i: i32 = 0;
    let mut n = (node.params.len() as i32);
    while i < n {
        let mut field = node.params[i as usize].clone();
        let mut ft = a2r_type(field.type_name.as_str());
        if ft == "" {
            ft = "i32".to_string()
        }
        out = format!("{}{}", format!("{}{}", format!("{}{}", format!("{}{}", format!("{}{}", out, a2r_indent_str(indent + 1)), field.name), ": "), ft), ",\n");
        i = i + 1;
    }
    out = format!("{}{}", format!("{}{}", out, prefix), "}\n");
    return out;
}

fn a2r_enum_decl(mut node: ASTNode, mut indent: i32) -> String {
    let mut prefix = a2r_indent_str(indent);
    let mut out: String = format!("{}{}", prefix, "#[derive(Clone, Debug, PartialEq)]\n");
    let mut generic_part: String = "".to_string();
    if node.type_name != "" {
        generic_part = node.type_name
    }
    out = format!("{}{}", format!("{}{}", format!("{}{}", format!("{}{}", format!("{}{}", out, prefix), "enum "), node.name), generic_part), " {\n");
    let mut i: i32 = 0;
    let mut n = (node.params.len() as i32);
    while i < n {
        let mut variant = node.params[i as usize].clone();
        out = format!("{}{}", format!("{}{}", out, a2r_indent_str(indent + 1)), variant.name);
        if variant.type_name != "" {
            let mut val = variant.type_name;
            

            let mut vlen = (val.len() as i32);
            if vlen > 1 {
                out = format!("{}{}", format!("{}{}", out, " = "), a2r_std::str_substr(val.as_str(), 1, vlen))
            }        }
        out = format!("{}{}", out, ",\n");
        i = i + 1;
    }
    out = format!("{}{}", format!("{}{}", out, prefix), "}\n");
    return out;
}

fn a2r_use(mut node: ASTNode) -> String {
    let mut path = node.name;

    if path == "c" {
        return "// extern \"C\" { /* use.c */ }\n".to_string();
    }
    if path == "py" {
        return "// use python: ...\n".to_string();
    }

    let mut rust_path: String = "".to_string();
    if (path.len() as i32) >= 5 {
        if a2r_std::str_substr(path.as_str(), 0, 5) == "rust " {
            rust_path = a2r_path_to_rust(a2r_std::str_substr(path.as_str(), 5, (path.len() as i32)).as_str())
        }    }
    if rust_path == "" {
        

        rust_path = a2r_path_to_rust(path.as_str())
    }
    let mut items = node.params;
    if (items.len() as i32) > 0 {
        let mut item_names: String = "".to_string();
        let mut i: i32 = 0;
        let mut n = (items.len() as i32);
        while i < n {
            if i > 0 {
                item_names = format!("{}{}", item_names, ", ")
            }
            item_names = format!("{}{}", item_names, items[i as usize].clone().name);
            i = i + 1;
        }
        return format!("{}{}", format!("{}{}", format!("{}{}", format!("{}{}", "use ", rust_path), "::{"), item_names), "};\n");
    }
    return format!("{}{}", format!("{}{}", "use ", rust_path), ";\n");
}

fn a2r_is(mut node: ASTNode, mut tenv: TypeEnv, mut indent: i32) -> String {
    let mut prefix = a2r_indent_str(indent);
    let mut subj = a2r_expr(node.left[0 as usize].clone(), tenv.clone());
    let mut out: String = format!("{}{}", format!("{}{}", format!("{}{}", prefix, "match "), subj), " {\n");

    let mut i: i32 = 0;
    let mut n = (node.children.len() as i32);
    while i < n {
        let mut branch = node.children[i as usize].clone();
        let mut pattern = a2r_expr(branch.left[0 as usize].clone(), tenv.clone());
        let mut body_node = branch.right[0 as usize].clone();
        if body_node.kind == NodeKind::BlockStmt {
            let mut body_str = a2r_body(body_node.children, tenv.clone(), indent + 2);
            out = format!("{}{}", format!("{}{}", format!("{}{}", format!("{}{}", format!("{}{}", format!("{}{}", out, a2r_indent_str(indent + 1)), pattern), " => {\n"), body_str), a2r_indent_str(indent + 1)), "},\n")
        } else {
            let mut body = a2r_expr(body_node.clone(), tenv.clone());
            out = format!("{}{}", format!("{}{}", format!("{}{}", format!("{}{}", format!("{}{}", out, a2r_indent_str(indent + 1)), pattern), " => "), body), ",\n")
        }

        i = i + 1;
    }

    if (node.else_body.len() as i32) > 0 {
        let mut else_node = node.else_body[0 as usize].clone();
        if else_node.kind == NodeKind::BlockStmt {
            let mut else_str = a2r_body(else_node.children, tenv.clone(), indent + 2);
            out = format!("{}{}", format!("{}{}", format!("{}{}", format!("{}{}", format!("{}{}", out, a2r_indent_str(indent + 1)), "_ => {\n"), else_str), a2r_indent_str(indent + 1)), "},\n")
        } else {
            let mut else_body = a2r_expr(else_node.clone(), tenv.clone());
            out = format!("{}{}", format!("{}{}", format!("{}{}", format!("{}{}", out, a2r_indent_str(indent + 1)), "_ => "), else_body), ",\n")
        }
    }

    out = format!("{}{}", format!("{}{}", out, prefix), "}\n");
    return out;
}

fn a2r_method(mut node: ASTNode, mut tenv: TypeEnv, mut indent: i32) -> String {
    let mut prefix = a2r_indent_str(indent);


    let mut params_str: String = "&self".to_string();
    let mut i: i32 = 0;
    let mut pn = (node.params.len() as i32);
    while i < pn {
        let mut param = node.params[i as usize].clone();
        let mut pt = a2r_type(param.type_name.as_str());
        if pt == "" {
            pt = "i32".to_string()
        }
        if pt == "String" {
            params_str = format!("{}{}", format!("{}{}", format!("{}{}", params_str, ", "), param.name), ": &str")
        } else {
            params_str = format!("{}{}", format!("{}{}", format!("{}{}", format!("{}{}", params_str, ", "), param.name), ": "), pt)
        }

        i = i + 1;
    }


    let mut ret = a2r_type(node.type_name.as_str());
    let mut ret_part: String = "".to_string();
    if ret != "" {
        ret_part = format!("{}{}", " -> ", ret)
    }

    let mut out: String = format!("{}{}", format!("{}{}", format!("{}{}", format!("{}{}", format!("{}{}", format!("{}{}", format!("{}{}", prefix, "fn "), node.name), "("), params_str), ")"), ret_part), " {\n");
    out = format!("{}{}", out, a2r_body(node.children, tenv.clone(), indent + 1));
    out = format!("{}{}", format!("{}{}", out, prefix), "}\n");
    return out;
}

fn a2r_ext(mut node: ASTNode, mut tenv: TypeEnv, mut indent: i32) -> String {
    let mut prefix = a2r_indent_str(indent);
    let mut for_spec = node.op;
    let mut generic_part: String = "".to_string();
    if node.type_name != "" {
        generic_part = node.type_name
    }
    let mut out: String = "".to_string();
    if for_spec != "" {
        out = format!("{}{}", format!("{}{}", format!("{}{}", format!("{}{}", format!("{}{}", format!("{}{}", format!("{}{}", prefix, "impl"), generic_part), " "), for_spec), " for "), node.name), " {\n")
    } else {
        out = format!("{}{}", format!("{}{}", format!("{}{}", format!("{}{}", format!("{}{}", prefix, "impl"), generic_part), " "), node.name), " {\n")
    }


    let mut i: i32 = 0;
    let mut n = (node.children.len() as i32);
    while i < n {
        let mut method = node.children[i as usize].clone();
        out = format!("{}{}", out, a2r_method(method.clone(), tenv.clone(), indent + 1));
        i = i + 1;
    }

    out = format!("{}{}", format!("{}{}", out, prefix), "}\n");
    return out;
}

fn a2r_spec(mut node: ASTNode, mut indent: i32) -> String {
    let mut prefix = a2r_indent_str(indent);
    let mut generic_part: String = "".to_string();
    if node.type_name != "" {
        generic_part = node.type_name
    }
    let mut out: String = format!("{}{}", format!("{}{}", format!("{}{}", format!("{}{}", prefix, "trait "), node.name), generic_part), " {\n");
    let mut i: i32 = 0;
    let mut n = (node.params.len() as i32);
    while i < n {
        let mut method = node.params[i as usize].clone();
        out = format!("{}{}", format!("{}{}", format!("{}{}", format!("{}{}", out, a2r_indent_str(indent + 1)), "fn "), method.name), "(&self)");
        if method.type_name != "" {
            out = format!("{}{}", format!("{}{}", out, " -> "), a2r_type(method.type_name.as_str()))
        }
        out = format!("{}{}", out, ";\n");
        i = i + 1;
    }
    out = format!("{}{}", format!("{}{}", out, prefix), "}\n");
    return out;
}

fn a2r_fstr(mut node: ASTNode, mut tenv: TypeEnv) -> String {
    let mut fmt: String = "".to_string();
    let mut args: String = "".to_string();
    let mut i: i32 = 0;
    let mut n = (node.children.len() as i32);
    while i < n {
        let mut part = node.children[i as usize].clone();
        if part.kind == NodeKind::StrExpr {
            fmt = format!("{}{}", fmt, part.name)
        } else {
            fmt = format!("{}{}", fmt, "{}");
            if args != "" {
                args = format!("{}{}", args, ", ")
            }            args = format!("{}{}", args, a2r_expr(part.clone(), tenv.clone()))
        }

        i = i + 1;
    }
    if args == "" {
        return format!("{}{}", format!("{}{}", "format!(\"", fmt), "\")");
    }
    return format!("{}{}", format!("{}{}", format!("{}{}", format!("{}{}", "format!(\"", fmt), "\", "), args), ")");
}

fn a2r_struct_init(mut type_name: &str, mut args: Vec<ASTNode>, mut tenv: TypeEnv) -> String {
    let mut fields_str = tenv.struct_fields.get(&*type_name).cloned().unwrap_or_default();
    let mut out: String = format!("{}{}", type_name, " { ");
    let mut i: i32 = 0;
    let mut n = (args.len() as i32);
    while i < n {
        if i > 0 {
            out = format!("{}{}", out, ", ")
        }
        let mut field_name = str_get_part(&(fields_str), i);
        out = format!("{}{}", format!("{}{}", format!("{}{}", out, field_name), ": "), a2r_expr(args[i as usize].clone(), tenv.clone()));
        i = i + 1;
    }
    return format!("{}{}", out, " }");
}

// === eval ===
// a2r Standard Library (from crate)

#[derive(Clone, Debug, PartialEq)]
struct EvalEnv {
    pub globals: std::collections::HashMap<String, String>,
    pub scopes: Vec<String>,
    pub fn_defs: std::collections::HashMap<String, ASTNode>,
    pub output: String,
}

fn eval_new() -> EvalEnv {
    EvalEnv { globals: std::collections::HashMap::new(), scopes: Vec::new(), fn_defs: std::collections::HashMap::new(), output: "".to_string() }
}

fn eval_set_last_str(mut env: EvalEnv, mut s: &str) {
    env.globals.insert("__last_str__".to_string(), (s).to_string());
}

fn eval_get_last_str(mut env: EvalEnv) -> String {
    if env.globals.contains_key("__last_str__") {
        return env.globals.get("__last_str__").cloned().unwrap_or_default();
    }
    return "".to_string();
}

fn eval_set_last_type(mut env: EvalEnv, mut t: i32) {
    env.globals.insert("__last_type__".to_string(), (t).to_string());
}

fn eval_get_last_type(mut env: EvalEnv) -> i32 {
    if env.globals.contains_key("__last_type__") {
        return (env.globals.get(&"__last_type__".to_string()).and_then(|v| v.parse::<i32>().ok()).unwrap_or(0));
    }
    return 0;
}

fn eval_str_key(mut name: &str) -> String {
    return format!("{}{}", "__s__", name);
}

fn eval_is_str_var(mut env: EvalEnv, mut name: &str) -> i32 {
    let mut skey = eval_str_key(&(name));
    if env.globals.contains_key(&skey) {
        return 1;
    }
    return 0;
}

fn eval_push_scope(mut env: EvalEnv) {
    env.scopes.push(std::collections::HashMap::new());
}

fn eval_pop_scope(mut env: EvalEnv) {
    if (env.scopes.len() as i32) > 0 {
        let mut scope = env.scopes.pop();
        scope.take();
    }
}

fn eval_bind(mut env: EvalEnv, mut name: &str, mut value: i32) {
    env.globals.insert(name.to_string(), (value).to_string());
}

fn eval_bind_int(mut env: EvalEnv, mut name: &str, mut value: i32) {
    env.globals.insert(name.to_string(), (value).to_string());
}

fn eval_bind_str(mut env: EvalEnv, mut name: &str, mut value: &str) {
    let mut skey = eval_str_key(&(name));
    env.globals.insert(skey.to_string(), (value).to_string());
}

fn eval_lookup_int(mut env: EvalEnv, mut name: &str) -> i32 {
    if env.globals.contains_key(name) {
        return (env.globals.get(&name.to_string()).and_then(|v| v.parse::<i32>().ok()).unwrap_or(0));
    }
    return 0;
}

fn eval_lookup_str_var(mut env: EvalEnv, mut name: &str) -> String {
    let mut skey = eval_str_key(&(name));
    if env.globals.contains_key(&skey) {
        return env.globals.get(&*skey).cloned().unwrap_or_default();
    }
    return "".to_string();
}

fn eval_lookup_str(mut env: EvalEnv, mut name: &str) -> String {
    if env.globals.contains_key(name) {
        return env.globals.get(&*name).cloned().unwrap_or_default();
    }
    return "".to_string();
}

fn eval_str_cat(mut a: &str, mut b: &str) -> String {
    return format!("{}{}", a, b);
}

fn str_to_int(mut s: &str) -> i32 {
    let mut result: i32 = 0;
    let mut neg: i32 = 0;
    let mut i: i32 = 0;
    let mut n = (s.len() as i32);
    if n == 0 {
        return 0;
    }
    let mut ch = s.chars().nth(0 as usize).unwrap_or('\0') as i32;
    if ch == 45 {
        neg = 1;
        i = 1
    }
    while i < n {
        ch = s.chars().nth(i as usize).unwrap_or('\0') as i32;
        if ch >= 48 && ch <= 57 {
            result = format!("{}{}", result * 10, ch - 48)
        } else {
            break;
        }

        i = i + 1;
    }
    if neg == 1 {
        result = 0 - result
    }
    return result;
}

fn int_to_str(mut n: i32) -> String {
    if n == 0 {
        return "0".to_string();
    }
    let mut neg: i32 = 0;
    if n < 0 {
        neg = 1;
        n = 0 - n
    }
    let mut digits: String = "0123456789".to_string();
    let mut result: String = "".to_string();
    while n > 0 {
        let mut d = n % 10;
        result = format!("{}{}", a2r_std::str_substr(digits.as_str(), d, d + 1), result);
        n = n / 10;
    }
    if neg == 1 {
        result = format!("{}{}", "-", result)
    }
    return result;
}

fn str_get_part(mut s: &str, mut index: i32) -> String {
    let mut start: i32 = 0;
    let mut part: i32 = 0;
    let mut i: i32 = 0;
    let mut n = (s.len() as i32);
    while i <= n {
        let mut ch: i32 = 0;
        if i < n {
            ch = s.chars().nth(i as usize).unwrap_or('\0') as i32
        }
        if ch == 44 || i == n {
            if part == index {
                return a2r_std::str_substr(s, start, i);
            }            part = part + 1;
            start = i + 1
        }
        i = i + 1;
    }
    return "".to_string();
}

fn eval_program(mut env: EvalEnv, mut stmts: Vec<ASTNode>) {

    let mut i: i32 = 0;
    let mut n = (stmts.len() as i32);
    while i < n {
        let mut stmt = stmts[i as usize].clone();
        if stmt.kind == NodeKind::FnStmt {
            env.fn_defs.insert(stmt.name.to_string(), stmt.clone());
        }
        i = i + 1;
    }


    i = 0;
    while i < n {
        let mut stmt = stmts[i as usize].clone();
        eval_exec_stmt(env.clone(), stmt.clone());
        i = i + 1;
    }
}

fn run_eval(mut source: &str) -> String {
    let mut tokens = tokenize_list(&(source));
    let mut p = parser_new(tokens.clone());
    let mut stmts = parse_program(p);
    let mut env = eval_new();
    eval_program(env.clone(), stmts.clone());
    return env.output;
}

fn eval_exec_stmt(mut env: EvalEnv, mut node: ASTNode) -> i32 {
    let mut kind = node.kind;

    if kind == NodeKind::FnStmt {
        env.fn_defs.insert(node.name.to_string(), node.clone());
        return 0;
    }

    if kind == NodeKind::LetStmt || kind == NodeKind::VarStmt {
        let mut expr = node.left[0 as usize].clone();
        let mut val = eval_expr(env.clone(), expr.clone());
        if eval_get_last_type(env.clone()) == 1 {
            eval_bind_str(env.clone(), node.name.as_str(), eval_get_last_str(env.clone()).as_str())
        } else {
            eval_bind_int(env.clone(), node.name.as_str(), val)
        }
        return 0;
    }

    if kind == NodeKind::ReturnStmt {
        return 1;
    }

    if kind == NodeKind::IfStmt {
        return eval_if(env.clone(), node.clone());
    }

    if kind == NodeKind::ForStmt {
        return eval_for(env.clone(), node.clone());
    }

    if kind == NodeKind::ForInStmt {
        return eval_forin(env.clone(), node.clone());
    }

    if kind == NodeKind::ExprStmt {
        let mut val = eval_expr(env.clone(), node.left[0 as usize].clone());
        

        env.globals.insert("__last_expr__".to_string(), (val).to_string());
        return 0;
    }

    if kind == NodeKind::BlockStmt {
        return eval_block(env.clone(), node.clone());
    }

    return 0;
}

fn eval_exec_body(mut env: EvalEnv, mut body: Vec<ASTNode>) -> i32 {
    let mut i: i32 = 0;
    let mut n = (body.len() as i32);
    while i < n {
        let mut stmt = body[i as usize].clone();
        if stmt.kind == NodeKind::ReturnStmt {
            let mut val: i32 = 0;
            if (stmt.left.len() as i32) > 0 {
                val = eval_expr(env.clone(), stmt.left[0 as usize].clone())
            }            env.globals.insert("__ret__".to_string(), (val).to_string());
            env.globals.insert("__ret_type__".to_string(), (eval_get_last_type(env.clone())).to_string());
            if eval_get_last_type(env.clone()) == 1 {
                env.globals.insert("__ret_str__".to_string(), (eval_get_last_str(env.clone())).to_string());
            }            return 1;
        }
        let mut ret = eval_exec_stmt(env.clone(), stmt.clone());
        if ret == 1 {
            return 1;
        }
        

        if i == n - 1 {
            if stmt.kind == NodeKind::ExprStmt {
                


                if env.globals.contains_key("__last_expr__") {
                    env.globals.insert("__ret__".to_string(), ((env.globals.get(&"__last_expr__".to_string()).and_then(|v| v.parse::<i32>().ok()).unwrap_or(0))).to_string());
                }                if env.globals.contains_key("__last_type__") {
                    env.globals.insert("__ret_type__".to_string(), ((env.globals.get(&"__last_type__".to_string()).and_then(|v| v.parse::<i32>().ok()).unwrap_or(0))).to_string());
                }                if env.globals.contains_key("__last_str__") {
                    env.globals.insert("__ret_str__".to_string(), (env.globals.get("__last_str__").cloned().unwrap_or_default()).to_string());
                }            }        }
        i = i + 1;
    }
    return 0;
}

fn eval_expr(mut env: EvalEnv, mut node: ASTNode) -> i32 {
    let mut kind = node.kind;

    if kind == NodeKind::IntExpr {
        eval_set_last_type(env.clone(), 0);
        return str_to_int(node.name.as_str());
    }

    if kind == NodeKind::StrExpr {
        eval_set_last_str(env.clone(), node.name.as_str());
        eval_set_last_type(env.clone(), 1);
        return 0;
    }

    if kind == NodeKind::BoolExpr {
        if node.name == "true" {
            eval_set_last_type(env.clone(), 2);
            return 1;
        }        eval_set_last_type(env.clone(), 2);
        return 0;
    }

    if kind == NodeKind::IdentExpr {
        if eval_is_str_var(env.clone(), node.name.as_str()) == 1 {
            eval_set_last_str(env.clone(), eval_lookup_str_var(env.clone(), node.name.as_str()).as_str());
            eval_set_last_type(env.clone(), 1);
            return 0;
        }        eval_set_last_type(env.clone(), 0);
        return eval_lookup_int(env.clone(), node.name.as_str());
    }

    if kind == NodeKind::BinExpr {
        return eval_binop(env.clone(), node.clone());
    }

    if kind == NodeKind::UnaryExpr {
        return eval_unary(env.clone(), node.clone());
    }

    if kind == NodeKind::CallExpr {
        return eval_call(env.clone(), node.clone());
    }

    if kind == NodeKind::DotExpr {
        return eval_dot(env.clone(), node.clone());
    }

    if kind == NodeKind::FStrExpr {
        return 0;
    }

    if kind == NodeKind::NilNode {
        return 0;
    }

    return 0;
}

fn eval_binop(mut env: EvalEnv, mut node: ASTNode) -> i32 {
    let mut op = node.op;


    if op == "+" {
        let mut left_node = node.left[0 as usize].clone();
        let mut right_node = node.right[0 as usize].clone();
        

        if left_node.kind == NodeKind::StrExpr && right_node.kind == NodeKind::StrExpr {
            eval_set_last_str(env.clone(), eval_str_cat(left_node.name.as_str(), right_node.name.as_str()).as_str());
            eval_set_last_type(env.clone(), 1);
            return 0;
        }        

        if left_node.kind == NodeKind::StrExpr {
            let mut right_val = eval_expr(env.clone(), right_node.clone());
            if eval_get_last_type(env.clone()) == 1 {
                eval_set_last_str(env.clone(), eval_str_cat(left_node.name.as_str(), eval_get_last_str(env.clone()).as_str()).as_str())
            } else {
                eval_set_last_str(env.clone(), eval_str_cat(left_node.name.as_str(), int_to_str(right_val).as_str()).as_str())
            }
            eval_set_last_type(env.clone(), 1);
            return 0;
        }        

        if left_node.kind == NodeKind::IdentExpr {
            if eval_is_str_var(env.clone(), left_node.name.as_str()) == 1 {
                let mut left_str = eval_lookup_str_var(env.clone(), left_node.name.as_str());
                let mut right_val = eval_expr(env.clone(), right_node.clone());
                if eval_get_last_type(env.clone()) == 1 {
                    eval_set_last_str(env.clone(), eval_str_cat(left_str.as_str(), eval_get_last_str(env.clone()).as_str()).as_str())
                } else {
                    eval_set_last_str(env.clone(), eval_str_cat(left_str.as_str(), int_to_str(right_val).as_str()).as_str())
                }
                eval_set_last_type(env.clone(), 1);
                return 0;
            }        }        let mut left = eval_expr(env.clone(), left_node.clone());
        let mut right = eval_expr(env.clone(), right_node.clone());
        return left + right;
    }

    let mut left = eval_expr(env.clone(), node.left[0 as usize].clone());
    let mut right = eval_expr(env.clone(), node.right[0 as usize].clone());

    if op == "-" {
        return left - right;
    }
    if op == "*" {
        return left * right;
    }
    if op == "/" {
        return left / right;
    }
    if op == "%" {
        return left % right;
    }

    if op == "==" {
        return left == right;
    }
    if op == "!=" {
        return left != right;
    }
    if op == "<" {
        return left < right;
    }
    if op == ">" {
        return left > right;
    }
    if op == "<=" {
        return left <= right;
    }
    if op == ">=" {
        return left >= right;
    }

    if op == "||" {
        if left != 0 {
            return 1;
        }        if right != 0 {
            return 1;
        }        return 0;
    }
    if op == "&&" {
        if left == 0 {
            return 0;
        }        if right == 0 {
            return 0;
        }        return 1;
    }

    if op == ".." {
        return left;
    }
    if op == "..=" {
        return left;
    }

    if op == "=" {
        

        let mut left_node = node.left[0 as usize].clone();
        if eval_get_last_type(env.clone()) == 1 {
            eval_bind_str(env.clone(), left_node.name.as_str(), eval_get_last_str(env.clone()).as_str())
        } else {
            eval_bind_int(env.clone(), left_node.name.as_str(), right)
        }
        return right;
    }

    return 0;
}

fn eval_unary(mut env: EvalEnv, mut node: ASTNode) -> i32 {
    let mut op = node.op;
    let mut operand = eval_expr(env.clone(), node.left[0 as usize].clone());

    if op == "-" {
        return 0 - operand;
    }
    if op == "!" {
        if operand == 0 {
            return 1;
        }        return 0;
    }

    return operand;
}

fn eval_call(mut env: EvalEnv, mut node: ASTNode) -> i32 {
    let mut callee_name = node.name;


    if callee_name == "print" {
        if (node.params.len() as i32) == 0 {
            env.output = eval_str_cat(env.output.as_str(), "\n");
            return 0;
        }        

        let mut arg_node = node.params[0 as usize].clone();
        if arg_node.kind == NodeKind::BoolExpr {
            if arg_node.name == "true" {
                env.output = eval_str_cat(env.output.as_str(), "true\n")
            } else {
                env.output = eval_str_cat(env.output.as_str(), "false\n")
            }
            return 0;
        }        if arg_node.kind == NodeKind::StrExpr {
            env.output = eval_str_cat(env.output.as_str(), eval_str_cat(arg_node.name.as_str(), "\n").as_str());
            return 0;
        }        

        let mut result = eval_expr(env.clone(), arg_node.clone());
        if eval_get_last_type(env.clone()) == 1 {
            env.output = eval_str_cat(env.output.as_str(), eval_str_cat(eval_get_last_str(env.clone()).as_str(), "\n").as_str())
        } else {
            env.output = eval_str_cat(env.output.as_str(), eval_str_cat(int_to_str(result).as_str(), "\n").as_str())
        }
        return result;
    }


    if callee_name == "empty_list" {
        return 0;
    }


    if callee_name == "str_to_int" {
        return 0;
    }


    if env.fn_defs.contains_key(&callee_name) {
        return eval_fn_call(env.clone(), node.clone(), callee_name.as_str());
    }

    return 0;
}

fn eval_fn_call(mut env: EvalEnv, mut call_node: ASTNode, mut fn_name: &str) -> i32 {
    let mut fn_def = env.fn_defs.get(&*fn_name).cloned().unwrap_or(ASTNode { kind: NodeKind::NilNode, value: "".to_string(), name: "".to_string(), children: empty_list(), left: empty_list(), right: empty_list(), op: "".to_string(), params: empty_list(), type_name: "".to_string(), cond: empty_list(), else_body: empty_list() });

    let mut params = fn_def.params;
    let mut params_count = (params.len() as i32);


    let mut param_names = fn_def.op;


    eval_push_scope(env.clone());


    let mut i: i32 = 0;
    while i < params_count {
        if i < (call_node.params.len() as i32) {
            let mut arg_val = eval_expr(env.clone(), call_node.params[i as usize].clone());
            let mut lt = eval_get_last_type(env.clone());
            

            let mut pname = str_get_part(param_names.as_str(), i);
            if lt == 1 {
                eval_bind_str(env.clone(), pname.as_str(), eval_get_last_str(env.clone()).as_str())
            } else {
                eval_bind_int(env.clone(), pname.as_str(), arg_val)
            }
        }
        i = i + 1;
    }


    let mut body = fn_def.children;
    eval_exec_body(env.clone(), body.clone());


    let mut result: i32 = 0;
    if env.globals.contains_key("__ret_type__") {
        let mut ret_type = (env.globals.get(&"__ret_type__".to_string()).and_then(|v| v.parse::<i32>().ok()).unwrap_or(0));
        if ret_type == 1 && env.globals.contains_key("__ret_str__") {
            eval_set_last_str(env.clone(), env.globals.get("__ret_str__").cloned().unwrap_or_default().as_str());
            eval_set_last_type(env.clone(), 1);
        }    }
    if env.globals.contains_key("__ret__") {
        result = (env.globals.get(&"__ret__".to_string()).and_then(|v| v.parse::<i32>().ok()).unwrap_or(0))
    }

    eval_pop_scope(env.clone());
    return result;
}

fn eval_dot(mut env: EvalEnv, mut node: ASTNode) -> i32 {
    let mut obj = eval_expr(env.clone(), node.left[0 as usize].clone());
    let mut field = node.name;


    if field == "len" {
        return obj;
    }

    return 0;
}

fn eval_if(mut env: EvalEnv, mut node: ASTNode) -> i32 {
    let mut cond = eval_expr(env.clone(), node.cond[0 as usize].clone());
    if cond != 0 {
        return eval_exec_body(env.clone(), node.children);
    } else {
        if (node.else_body.len() as i32) > 0 {
            let mut first = node.else_body[0 as usize].clone();
            if first.kind == NodeKind::IfStmt {
                return eval_if(env.clone(), first.clone());
            }            return eval_exec_body(env.clone(), node.else_body);
        }    }

    return 0;
}

fn eval_for(mut env: EvalEnv, mut node: ASTNode) -> i32 {
    let mut cond_node = node.left[0 as usize].clone();
    let mut body = node.children;
    let mut limit: i32 = 10000;
    let mut count: i32 = 0;
    loop {
        let mut cond = eval_expr(env.clone(), cond_node.clone());
        if cond == 0 {
            break;
        }
        let mut ret = eval_exec_body(env.clone(), body.clone());
        if ret == 1 {
            return 1;
        }
        count = count + 1;
        if count >= limit {
            break;
        }
    }
    return 0;
}

fn eval_forin(mut env: EvalEnv, mut node: ASTNode) -> i32 {
    let mut range_node = node.left[0 as usize].clone();
    let mut body = node.children;
    let mut var_name = node.name;

    let mut start_val = eval_expr(env.clone(), range_node.left[0 as usize].clone());
    let mut end_val = eval_expr(env.clone(), range_node.right[0 as usize].clone());
    let mut inclusive: i32 = 0;
    if range_node.op == "..=" {
        inclusive = 1
    }

    let mut i = start_val;
    let mut limit = end_val;
    if inclusive == 1 {
        limit = end_val + 1
    }
    while i < limit {
        eval_bind(env.clone(), var_name.as_str(), i);
        let mut ret = eval_exec_body(env.clone(), body.clone());
        if ret == 1 {
            return 1;
        }
        i = i + 1;
    }
    return 0;
}

fn eval_block(mut env: EvalEnv, mut node: ASTNode) -> i32 {
    return eval_exec_body(env.clone(), node.children);
}

fn run_bytecode(mut source: &str) -> String {
    let mut tokens = tokenize_list(&(source));
    let mut p = parser_new(tokens.clone());
    let mut stmts = parse_program(p);

    let mut tenv = typeenv_new();
    type_infer_program(tenv, stmts.clone());

    let mut cg = codegen_new();
    codegen_compile(cg.clone(), stmts.clone(), tenv);

    let mut state = std::collections::HashMap::new();
    bvm_init(state.clone());
    state.insert("__nstr".to_string(), (cg.n_strings).to_string());
    return bvm_run(cg.code, cg.strings, state);
}


fn main() {}

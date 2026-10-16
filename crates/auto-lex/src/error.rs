//! auto-lex 错误与位置投影（PLAN-789 自 auto-lang/src/error.rs 切出）。
//!
//! 职责边界：本模块只承载词法层的**局部**错误与 span 助手——
//! `LexerError`（E0001..E0005 结构化五变体，逐字搬运，诊断 code/label/
//! 文案不变）与 `LexError`（词法面全集：结构化变体 + raw-block 字符串
//! 错误）。auto-lang 的错误聚合器（`AutoError` 及其 Name/Type/Runtime
//! 族）**不倒灌**入本 crate；上层经 `From<LexError>` 做等价投影
//! （结构化 → `AutoError::Lexer`，Raw → `AutoError::Msg`，与抽取前
//! 旧臂一致）。

use crate::token::Pos;
use miette::{Diagnostic, SourceSpan};
use thiserror::Error;

/// 词法层结果别名（对应抽取前 lexer 方法签名里的 `AutoResult`）。
pub type LexResult<T> = std::result::Result<T, LexError>;

/// Convert a `Pos` to a `SourceSpan` for use with miette diagnostics
///
/// # Example
///
/// ```rust
/// use auto_lex::error::pos_to_span;
/// use auto_lex::token::Pos;
///
/// let pos = Pos {
///     line: 5,
///     at: 10,
///     pos: 100,
///     len: 5,
/// };
/// let span = pos_to_span(pos);
/// assert_eq!(span.offset(), 100);
/// assert_eq!(span.len(), 5);
/// ```
pub fn pos_to_span(pos: Pos) -> SourceSpan {
    SourceSpan::new(pos.pos.into(), pos.len.into())
}

/// Create a span from absolute position and length
pub fn span_from(offset: usize, len: usize) -> SourceSpan {
    SourceSpan::new(offset.into(), len.into())
}

// ============================================================================
// Lexer Errors (E0001-E0099)
// ============================================================================

/// Lexer errors during tokenization
#[derive(Error, Diagnostic, Debug, Clone)]
pub enum LexerError {
    /// Unknown escape sequence in character literal
    #[error("unknown escape sequence")]
    #[diagnostic(
        code(auto_lexer_E0001),
        help("Valid escape sequences are: \\n, \\t, \\r, \\0, \\\\, \\'")
    )]
    UnknownEscapeSequence {
        sequence: String,
        #[label("unknown escape sequence")]
        span: SourceSpan,
    },

    /// Unterminated character literal
    #[error("unterminated character literal")]
    #[diagnostic(
        code(auto_lexer_E0002),
        help("Character literals must be enclosed in single quotes (')")
    )]
    UnterminatedChar {
        #[label("character literal not closed")]
        span: SourceSpan,
    },

    /// Empty character literal
    #[error("empty character literal")]
    #[diagnostic(
        code(auto_lexer_E0003),
        help("Character literals must contain exactly one character")
    )]
    EmptyChar {
        #[label("empty character literal")]
        span: SourceSpan,
    },

    /// Invalid identifier start
    #[error("invalid identifier")]
    #[diagnostic(
        code(auto_lexer_E0004),
        help("Identifiers must start with a letter or underscore")
    )]
    InvalidIdentifierStart {
        character: String,
        #[label("identifiers must start with a letter or underscore")]
        span: SourceSpan,
    },

    /// Unknown character
    #[error("unknown character")]
    #[diagnostic(
        code(auto_lexer_E0005),
        help("This character is not valid in AutoLang source code")
    )]
    UnknownCharacter {
        character: String,
        #[label("unknown character")]
        span: SourceSpan,
    },
}

/// 词法面全集错误：结构化五变体之外，`capture_raw_block`（widget `style { }`
/// 原始块捕获）的历史字符串错误在旧臂经 `String.into()` 落 `AutoError::Msg`
/// 投影，此处以 `Raw` 承载同一语义，上层 `From` 映射回 Msg，投影不变。
#[derive(Error, Debug, Clone)]
pub enum LexError {
    #[error(transparent)]
    Lexer(#[from] LexerError),

    /// raw-block 捕获的字符串错误（旧臂 Msg 投影，无诊断 code）。
    #[error("{0}")]
    Raw(String),
}

impl From<String> for LexError {
    fn from(msg: String) -> Self {
        LexError::Raw(msg)
    }
}

impl<'a> From<&'a str> for LexError {
    fn from(msg: &'a str) -> Self {
        LexError::Raw(msg.to_string())
    }
}

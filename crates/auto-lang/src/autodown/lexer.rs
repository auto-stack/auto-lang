//! AutoDown Lexer - Mode-Aware Tokenizer
//!
//! This lexer handles three modes:
//! - Text Mode: Default, for paragraphs,//! - Code Mode: After `$`, for Auto code
//! - Math Mode: Inside `%{ ... }`, for math expressions

use std::iter::Peekable;
use std::str::Chars;

use super::ast::{AdocBlock, AdocInline, AdocMath};
use super::error::{AdocError, AdocResult};

// ============================================================================
// Token Types
// ============================================================================

/// Lexer mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LexerMode {
    /// Default text mode (paragraphs, lists, etc.)
    Text,
    /// Code mode (after `$`)
    Code,
    /// Math mode (inside `%{ ... }`)
    Math,
}

/// Token produced by the lexer
#[derive(Debug, Clone)]
pub struct AdToken {
    /// Token kind
    pub kind: AdTokenKind,
    /// Token text
    pub text: String,
    /// Line number (1-based)
    pub line: usize,
    /// Column number (1-based)
    pub column: usize,
}

impl AdToken {
    /// Create a new token
    pub fn new(kind: AdTokenKind, text: impl Into<String>, line: usize, column: usize) -> Self {
        Self {
            kind,
            text: text.into(),
            line,
            column,
        }
    }
    
    /// Create a simple token
    pub fn simple(kind: AdTokenKind, line: usize, column: usize) -> Self {
        Self {
            kind,
            text: String::new(),
            line,
            column,
        }
    }
}

/// Token kinds
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdTokenKind {
    // Special
    EOF,
    Error,
    
    // Text tokens
    Text,           // Plain text content
    Newline,        // Line break
    BlankLine,      // Paragraph separator (empty line)
    
    // Headers
    Header { level: u8 },
    
    // Logic domain tokens (after $)
    Dollar,         // $
    LBrace,         // {
    RBrace,         // }
    LParen,         // (
    RParen,         // )
    LBracket,       // [
    RBracket,       // ]
    Colon,          // :
    Comma,          // ,
    Dot,            // .
    
    // Keywords
    If,
    Else,
    For,
    In,
    Let,
    Var,
    Fn,
    Return,
    True,
    False,
    Nil,
    
    // Literals
    Ident,          // identifier
    String,         // "string"
    FString,        // f"string"
    Number,         // 123 or 3.14
    
    // Operators
    Plus,           // +
    Minus,          // -
    Star,           // *
    Slash,          // /
    Percent,        // %
    Caret,          // ^
    Eq,             // ==
    Ne,             // !=
    Lt,             // <
    Le,             // <=
    Gt,             // >
    Ge,             // >=
    And,            // &&
    Or,             // ||
    Assign,         // =
    
    // Math domain
    MathStart,      // %{
    MathEnd,        // }
    MathContent,    // Raw math content
    
    // Interpolation
    InterpolateStart, // ${
    InterpolateEnd,   // }
    
    // Inline markup
    StarStar,        // ** (bold)
    Underscore,      // _ (italic)
    Underscore2,     // __ (underline - future)
    Backtick,        // ` (inline code)
    
    // Links
    LinkStart,       // [
    LinkText,        // link text
    LinkUrl,         // (url)
    ImageStart,      // ![
    
    // Lists
    ListItem,        // - or * or +
    NumberedList,    // 1. or 2)
    
    // Blocks
    CodeFence,       // ```
    Blockquote,      // >
    HorizontalRule,  // --- or ***
    
    // Code content (raw code)
    CodeContent,
}

// ============================================================================
// Lexer Implementation
// ============================================================================

/// AutoDown Lexer
pub struct AdocLexer<'a> {
    /// Source code being lexed
    source: &'a str,
    
    /// Character iterator with lookahead
    chars: Peekable<Chars<'a>>,
    
    /// Current lexer mode
    mode: LexerMode,
    
    /// Token buffer for multi-token emission
    buffer: VecDeque<AdToken>,
    
    /// Current line number (1-based)
    line: usize,
    
    /// Current column number (1-based)
    column: usize,
    
    /// Current character position
    pos: usize,
    
    /// Start position of current token
    token_start: usize,
    
    /// Brace level for nested structures
    brace_level: usize,
    
    /// Code fence language (if in code block)
    code_fence_lang: Option<String>,
    
    /// Code fence delimiter (``` or ~~~)
    code_fence_delim: String,
}

use std::collections::VecDeque;

impl<'a> AdocLexer<'a> {
    /// Create a new lexer
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            chars: source.chars().peekable(),
            mode: LexerMode::Text,
            buffer: VecDeque::new(),
            line: 1,
            column: 1,
            pos: 0,
            token_start: 0,
            brace_level: 0,
            code_fence_lang: None,
            code_fence_delim: String::new(),
        }
    }
    
    /// Get the next token
    pub fn next_token(&mut self) -> AdocResult<AdToken> {
        // If we have buffered tokens, return those first
        if let Some(token) = self.buffer.pop_front() {
            return Ok(token);
        }
        
        // Skip whitespace based on mode
        match self.mode {
            LexerMode::Text => self.skip_whitespace_text(),
            LexerMode::Code => self.skip_whitespace_code(),
            LexerMode::Math => {} // Don't skip in math mode
        }
        
        // Check for EOF
        if self.is_eof() {
            return Ok(AdToken::simple(AdTokenKind::EOF, self.line, self.column));
        }
        
        // Tokenize based on mode
        match self.mode {
            LexerMode::Text => self.tokenize_text(),
            LexerMode::Code => self.tokenize_code(),
            LexerMode::Math => self.tokenize_math(),
        }
    }
    
    /// Check if at end of source
    fn is_eof(&mut self) -> bool {
        self.chars.peek().is_none()
    }
    
    /// Peek at the next character without consuming
    fn peek(&mut self) -> Option<char> {
        self.chars.peek().copied()
    }
    
    /// Peek at the character after next
    fn peek_next(&mut self) -> Option<char> {
        // This is inefficient but works for now
        // A better approach would use a peekable buffer
        None // TODO: implement properly
    }
    
    /// Consume and return the next character
    fn advance(&mut self) -> Option<char> {
        let c = self.chars.next();
        if let Some(ch) = c {
            self.pos += ch.len_utf8();
            if ch == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
        }
        c
    }
    
    /// Skip whitespace in text mode (preserve newlines for structure)
    fn skip_whitespace_text(&mut self) {
        while let Some(&c) = self.chars.peek() {
            if c == ' ' || c == '\t' {
                self.advance();
            } else {
                break;
            }
        }
    }
    
    /// Skip whitespace in code mode
    fn skip_whitespace_code(&mut self) {
        while let Some(&c) = self.chars.peek() {
            if c == ' ' || c == '\t' || c == '\n' || c == '\r' {
                self.advance();
            } else {
                break;
            }
        }
    }
    
    /// Record token start position
    fn start_token(&mut self) {
        self.token_start = self.pos;
    }
    
    /// Get text from token start to current position
    fn token_text(&self) -> String {
        self.source[self.token_start..self.pos].to_string()
    }
    
    /// Tokenize in text mode
    fn tokenize_text(&mut self) -> AdocResult<AdToken> {
        let line = self.line;
        let column = self.column;
        
        match self.peek() {
            Some('\n') => {
                self.advance();
                // Check for blank line (two consecutive newlines)
                if self.peek() == Some('\n') {
                    self.advance();
                    return Ok(AdToken::simple(AdTokenKind::BlankLine, line, column));
                }
                return Ok(AdToken::simple(AdTokenKind::Newline, line, column));
            }
            
            Some('#') => {
                self.tokenize_header()
            }
            
            Some('$') => {
                self.advance();
                // Check for interpolation ${ ... }
                if self.peek() == Some('{') {
                    self.advance();
                    self.mode = LexerMode::Code;
                    return Ok(AdToken::simple(AdTokenKind::InterpolateStart, line, column));
                }
                // It's a code block start
                self.mode = LexerMode::Code;
                Ok(AdToken::simple(AdTokenKind::Dollar, line, column))
            }
            
            Some('%') => {
                // Check for math start %{
                if self.peek_next() == Some('{') {
                    self.advance(); // consume %
                    self.advance(); // consume {
                    self.mode = LexerMode::Math;
                    return Ok(AdToken::simple(AdTokenKind::MathStart, line, column));
                }
                // Just a percent sign in text
                self.start_token();
                self.advance();
                self.tokenize_text_content()
            }
            
            Some('*') => {
                // Check for bold ** or horizontal rule ***
                let line = self.line;
                let col = self.column;
                self.advance();
                if self.peek() == Some('*') {
                    self.advance();
                    // Check for horizontal rule ***
                    if self.peek() == Some('*') {
                        self.advance();
                        return Ok(AdToken::simple(AdTokenKind::HorizontalRule, line, col));
                    }
                    return Ok(AdToken::simple(AdTokenKind::StarStar, line, col));
                }
                // Just italic
                Ok(AdToken::simple(AdTokenKind::Underscore, line, col))
            }
            
            Some('_') => {
                let line = self.line;
                let col = self.column;
                self.advance();
                if self.peek() == Some('_') {
                    self.advance();
                    return Ok(AdToken::simple(AdTokenKind::Underscore2, line, col));
                }
                Ok(AdToken::simple(AdTokenKind::Underscore, line, col))
            }
            
            Some('`') => {
                // Check for code fence
                let line = self.line;
                let col = self.column;
                self.advance();
                if self.peek() == Some('`') {
                    self.advance();
                    if self.peek() == Some('`') {
                        self.advance();
                        return Ok(AdToken::simple(AdTokenKind::CodeFence, line, col));
                    }
                }
                // Inline code
                Ok(AdToken::simple(AdTokenKind::Backtick, line, col))
            }
            
            Some('-') => {
                self.tokenize_dash_or_list()
            }
            
            Some('>') => {
                let line = self.line;
                let col = self.column;
                self.advance();
                Ok(AdToken::simple(AdTokenKind::Blockquote, line, col))
            }
            
            Some('!') => {
                // Check for image ![...](...)
                let line = self.line;
                let col = self.column;
                self.advance();
                if self.peek() == Some('[') {
                    self.advance();
                    return Ok(AdToken::simple(AdTokenKind::ImageStart, line, col));
                }
                // Just a ! in text
                self.start_token();
                self.tokenize_text_content()
            }
            
            Some('[') => {
                let line = self.line;
                let col = self.column;
                self.advance();
                Ok(AdToken::simple(AdTokenKind::LinkStart, line, col))
            }
            
            Some(c) if c.is_ascii_digit() => {
                // Check for numbered list
                self.tokenize_number_or_list()
            }
            
            _ => {
                // Regular text content
                self.start_token();
                self.tokenize_text_content()
            }
        }
    }
    
    /// Tokenize header (# ## ### etc.)
    fn tokenize_header(&mut self) -> AdocResult<AdToken> {
        let line = self.line;
        let column = self.column;
        
        self.start_token();
        let mut level = 0u8;
        
        while self.peek() == Some('#') && level < 6 {
            self.advance();
            level += 1;
        }
        
        // Skip space after header
        if self.peek() == Some(' ') {
            self.advance();
        }
        
        Ok(AdToken::new(AdTokenKind::Header { level }, "#".repeat(level as usize), line, column))
    }
    
    /// Tokenize dash or list item
    fn tokenize_dash_or_list(&mut self) -> AdocResult<AdToken> {
        let line = self.line;
        let col = self.column;
        
        self.start_token();
        self.advance(); // consume -
        
        // Check for horizontal rule ---
        let mut dash_count = 1;
        while self.peek() == Some('-') {
            self.advance();
            dash_count += 1;
        }
        
        if dash_count >= 3 {
            return Ok(AdToken::simple(AdTokenKind::HorizontalRule, line, col));
        }
        
        // Check for list item (must be followed by space)
        if self.peek() == Some(' ') {
            self.advance();
            return Ok(AdToken::new(AdTokenKind::ListItem, "-", line, col));
        }
        
        // Just text
        self.tokenize_text_content()
    }
    
    /// Tokenize number or numbered list
    fn tokenize_number_or_list(&mut self) -> AdocResult<AdToken> {
        let line = self.line;
        let col = self.column;
        
        self.start_token();
        let mut num_str = String::new();
        
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() {
                num_str.push(c as char);
                self.advance();
            } else {
                break;
            }
        }
        
        // Check for numbered list (1. or 1))
        if self.peek() == Some('.') || self.peek() == Some(')') {
            let delim = self.peek().unwrap();
            self.advance();
            if self.peek() == Some(' ') {
                self.advance();
                return Ok(AdToken::new(AdTokenKind::NumberedList, num_str, line, col));
            }
        }
        
        // Just a number in text
        Ok(AdToken::new(AdTokenKind::Text, num_str, line, col))
    }
    
    /// Tokenize text content until special character
    fn tokenize_text_content(&mut self) -> AdocResult<AdToken> {
        let line = self.line;
        let column = self.column;
        
        self.start_token();
        let mut text = String::new();
        
        while let Some(c) = self.peek() {
            // Stop at special characters
            match c {
                '\n' | '#' | '$' | '%' | '*' | '_' | '`' | '-' 
                | '>' | '!' | '[' => break,
                ' ' | '\t' => {
                    text.push(c);
                    self.advance();
                }
                _ => {
                    text.push(c);
                    self.advance();
                }
            }
            
            if text.len() > 10000 {
                return Err(AdocError::lexer("Text content too long"));
            }
        }
        
        if text.is_empty() {
            // Single character token
            let c = self.advance().unwrap() as char;
            Ok(AdToken::new(AdTokenKind::Text, c.to_string(), line, column))
        } else {
            Ok(AdToken::new(AdTokenKind::Text, text, line, column))
        }
    }
    
    /// Tokenize in code mode (after $)
    fn tokenize_code(&mut self) -> AdocResult<AdToken> {
        let line = self.line;
        let column = self.column;
        
        match self.peek() {
            Some('{') => {
                self.advance();
                self.brace_level += 1;
                Ok(AdToken::simple(AdTokenKind::LBrace, line, column))
            }
            Some('}') => {
                self.advance();
                if self.brace_level > 0 {
                    self.brace_level -= 1;
                }
                // If brace level is 0, we might be exiting code mode
                if self.brace_level == 0 {
                    // Check if this is the end of interpolation
                    // The parser will handle mode switching
                }
                Ok(AdToken::simple(AdTokenKind::RBrace, line, column))
            }
            Some('(') => {
                self.advance();
                Ok(AdToken::simple(AdTokenKind::LParen, line, column))
            }
            Some(')') => {
                self.advance();
                Ok(AdToken::simple(AdTokenKind::RParen, line, column))
            }
            Some('[') => {
                self.advance();
                Ok(AdToken::simple(AdTokenKind::LBracket, line, column))
            }
            Some(']') => {
                self.advance();
                Ok(AdToken::simple(AdTokenKind::RBracket, line, column))
            }
            Some(':') => {
                self.advance();
                Ok(AdToken::simple(AdTokenKind::Colon, line, column))
            }
            Some(',') => {
                self.advance();
                Ok(AdToken::simple(AdTokenKind::Comma, line, column))
            }
            Some('.') => {
                self.advance();
                Ok(AdToken::simple(AdTokenKind::Dot, line, column))
            }
            Some('+') => {
                self.advance();
                Ok(AdToken::simple(AdTokenKind::Plus, line, column))
            }
            Some('-') => {
                self.advance();
                Ok(AdToken::simple(AdTokenKind::Minus, line, column))
            }
            Some('*') => {
                self.advance();
                Ok(AdToken::simple(AdTokenKind::Star, line, column))
            }
            Some('/') => {
                self.advance();
                Ok(AdToken::simple(AdTokenKind::Slash, line, column))
            }
            Some('%') => {
                self.advance();
                Ok(AdToken::simple(AdTokenKind::Percent, line, column))
            }
            Some('^') => {
                self.advance();
                Ok(AdToken::simple(AdTokenKind::Caret, line, column))
            }
            Some('=') => {
                self.advance();
                if self.peek() == Some('=') {
                    self.advance();
                    Ok(AdToken::simple(AdTokenKind::Eq, line, column))
                } else {
                    Ok(AdToken::simple(AdTokenKind::Assign, line, column))
                }
            }
            Some('!') => {
                self.advance();
                if self.peek() == Some('=') {
                    self.advance();
                    Ok(AdToken::simple(AdTokenKind::Ne, line, column))
                } else {
                    Err(AdocError::lexer("Unexpected '!' in code mode"))
                }
            }
            Some('<') => {
                self.advance();
                if self.peek() == Some('=') {
                    self.advance();
                    Ok(AdToken::simple(AdTokenKind::Le, line, column))
                } else {
                    Ok(AdToken::simple(AdTokenKind::Lt, line, column))
                }
            }
            Some('>') => {
                self.advance();
                if self.peek() == Some('=') {
                    self.advance();
                    Ok(AdToken::simple(AdTokenKind::Ge, line, column))
                } else {
                    Ok(AdToken::simple(AdTokenKind::Gt, line, column))
                }
            }
            Some('&') => {
                self.advance();
                if self.peek() == Some('&') {
                    self.advance();
                    Ok(AdToken::simple(AdTokenKind::And, line, column))
                } else {
                    Err(AdocError::lexer("Unexpected '&' in code mode"))
                }
            }
            Some('|') => {
                self.advance();
                if self.peek() == Some('|') {
                    self.advance();
                    Ok(AdToken::simple(AdTokenKind::Or, line, column))
                } else {
                    Err(AdocError::lexer("Unexpected '|' in code mode"))
                }
            }
            Some('"') => {
                self.tokenize_string()
            }
            Some(c) if c.is_ascii_digit() => {
                self.tokenize_number()
            }
            Some(c) if c.is_ascii_alphabetic() || c == '_' => {
                self.tokenize_identifier()
            }
            _ => Err(AdocError::unexpected_token(
                "valid code token",
                format!("'{}'", self.peek().unwrap_or('\0')),
            )),
        }
    }
    
    /// Tokenize string literal
    fn tokenize_string(&mut self) -> AdocResult<AdToken> {
        let line = self.line;
        let column = self.column;
        
        self.advance(); // consume opening "
        self.start_token();
        let mut content = String::new();
        
        loop {
            match self.peek() {
                Some('"') => {
                    self.advance();
                    return Ok(AdToken::new(AdTokenKind::String, content, line, column));
                }
                Some('\\') => {
                    self.advance();
                    if let Some(c) = self.advance() {
                        match c {
                            'n' => content.push('\n'),
                            't' => content.push('\t'),
                            'r' => content.push('\r'),
                            '\\' => content.push('\\'),
                            '"' => content.push('"'),
                            _ => content.push(c),
                        }
                    }
                }
                Some(c) => {
                    content.push(c as char);
                    self.advance();
                }
                None => {
                    return Err(AdocError::unterminated("string", line));
                }
            }
        }
    }
    
    /// Tokenize number literal
    fn tokenize_number(&mut self) -> AdocResult<AdToken> {
        let line = self.line;
        let column = self.column;
        
        self.start_token();
        let mut num_str = String::new();
        
        // Integer part
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() {
                num_str.push(c as char);
                self.advance();
            } else {
                break;
            }
        }
        
        // Decimal part
        if self.peek() == Some('.') {
            // Peek ahead to make sure it's not a method call
            num_str.push('.');
            self.advance();
            
            while let Some(c) = self.peek() {
                if c.is_ascii_digit() {
                    num_str.push(c as char);
                    self.advance();
                } else {
                    break;
                }
            }
        }
        
        Ok(AdToken::new(AdTokenKind::Number, num_str, line, column))
    }
    
    /// Tokenize identifier or keyword
    fn tokenize_identifier(&mut self) -> AdocResult<AdToken> {
        let line = self.line;
        let column = self.column;
        
        self.start_token();
        let mut ident = String::new();
        
        while let Some(c) = self.peek() {
            if c.is_ascii_alphanumeric() || c == '_' {
                ident.push(c as char);
                self.advance();
            } else {
                break;
            }
        }
        
        // Check for keywords
        let kind = match ident.as_str() {
            "if" => AdTokenKind::If,
            "else" => AdTokenKind::Else,
            "for" => AdTokenKind::For,
            "in" => AdTokenKind::In,
            "let" => AdTokenKind::Let,
            "var" => AdTokenKind::Var,
            "fn" => AdTokenKind::Fn,
            "return" => AdTokenKind::Return,
            "true" => AdTokenKind::True,
            "false" => AdTokenKind::False,
            "nil" => AdTokenKind::Nil,
            _ => AdTokenKind::Ident,
        };
        
        Ok(AdToken::new(kind, ident, line, column))
    }
    
    /// Tokenize in math mode (inside %{ ... })
    fn tokenize_math(&mut self) -> AdocResult<AdToken> {
        let line = self.line;
        let column = self.column;
        
        self.start_token();
        let mut content = String::new();
        let mut brace_level = 1;
        
        loop {
            match self.peek() {
                Some('}') => {
                    brace_level -= 1;
                    if brace_level == 0 {
                        self.advance();
                        self.mode = LexerMode::Text;
                        
                        // Return math content
                        if !content.is_empty() {
                            self.buffer.push_back(AdToken::new(
                                AdTokenKind::MathEnd,
                                "}",
                                self.line,
                                self.column
                            ));
                            return Ok(AdToken::new(AdTokenKind::MathContent, content, line, column));
                        }
                        return Ok(AdToken::simple(AdTokenKind::MathEnd, line, column));
                    } else {
                        content.push('}');
                        self.advance();
                    }
                }
                Some('{') => {
                    brace_level += 1;
                    content.push('{');
                    self.advance();
                }
                Some(c) => {
                    content.push(c as char);
                    self.advance();
                }
                None => {
                    return Err(AdocError::unterminated("math block", line));
                }
            }
        }
    }
    
    /// Switch to a different lexer mode
    pub fn set_mode(&mut self, mode: LexerMode) {
        self.mode = mode;
    }
    
    /// Get current lexer mode
    pub fn mode(&self) -> LexerMode {
        self.mode.clone()
    }
    
    /// Tokenize entire source into a vector
    pub fn tokenize_all(&mut self) -> AdocResult<Vec<AdToken>> {
        let mut tokens = Vec::new();
        
        loop {
            let token = self.next_token()?;
            if token.kind == AdTokenKind::EOF {
                tokens.push(token);
                break;
            }
            tokens.push(token);
        }
        
        Ok(tokens)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lexer_basic_text() {
        let source = "Hello, world!";
        let mut lexer = AdocLexer::new(source);
        
        let token = lexer.next_token().unwrap();
        assert_eq!(token.kind, AdTokenKind::Text);
        assert_eq!(token.text, "Hello,");
        
        let token = lexer.next_token().unwrap();
        assert_eq!(token.kind, AdTokenKind::Text);
        assert_eq!(token.text, "world!");
    }

    #[test]
    fn test_lexer_header() {
        let source = "# Title\n## Section";
        let mut lexer = AdocLexer::new(source);
        
        let token = lexer.next_token().unwrap();
        assert!(matches!(token.kind, AdTokenKind::Header { level: 1 }));
        
        let token = lexer.next_token().unwrap();
        assert_eq!(token.kind, AdTokenKind::Text);
        
        let token = lexer.next_token().unwrap();
        assert_eq!(token.kind, AdTokenKind::Newline);
        
        let token = lexer.next_token().unwrap();
        assert!(matches!(token.kind, AdTokenKind::Header { level: 2 }));
    }

    #[test]
    fn test_lexer_math() {
        let source = "Math: %{ E = mc^2 } here";
        let mut lexer = AdocLexer::new(source);
        
        // "Math:"
        let token = lexer.next_token().unwrap();
        assert_eq!(token.kind, AdTokenKind::Text);
        
        // MathStart
        let token = lexer.next_token().unwrap();
        assert_eq!(token.kind, AdTokenKind::MathStart);
        
        // MathContent
        let token = lexer.next_token().unwrap();
        assert_eq!(token.kind, AdTokenKind::MathContent);
        assert!(token.text.contains("E = mc^2"));
        
        // MathEnd
        let token = lexer.next_token().unwrap();
        assert_eq!(token.kind, AdTokenKind::MathEnd);
    }

    #[test]
    fn test_lexer_interpolation() {
        let source = "Hello, ${name}!";
        let mut lexer = AdocLexer::new(source);
        
        // "Hello,"
        let token = lexer.next_token().unwrap();
        assert_eq!(token.kind, AdTokenKind::Text);
        
        // ${
        let token = lexer.next_token().unwrap();
        assert_eq!(token.kind, AdTokenKind::InterpolateStart);
        
        // name (identifier in code mode)
        let token = lexer.next_token().unwrap();
        assert_eq!(token.kind, AdTokenKind::Ident);
        assert_eq!(token.text, "name");
        
        // }
        let token = lexer.next_token().unwrap();
        assert_eq!(token.kind, AdTokenKind::RBrace);
    }
}

/// Helper function: tokenize text content starting with a specific character already in buffer
fn tokenize_text_content_with(&mut self, start_char: char) -> AdocResult<AdToken> {
    let mut text = String::new();
    text.push(start_char);
    
    // Continue with normal text tokenization
    self.tokenize_text_content_with(start_char)
}

/// Helper function: tokenize text content starting with a specific character already in buffer
fn tokenize_text_content_with(&mut self, start_char: char) -> AdocResult<AdToken> {
    let line = self.line;
    let column = self.column;
    
    self.mode = LexerMode::Text;
    
    // Start with the initial character
    let mut text = String::new();
    text.push(start_char);
    
    loop {
        match self.peek() {
            // Stop at special characters
            '\n' | '#' | '$' | '%' | '*' | '_' | '`' | '-' 
            | '>' | '!' | '[' => break,
            ' ' | '\t' => {
                text.push(c);
                self.advance();
            }
            _ => {
                text.push(c);
                self.advance();
            }
        }
        
        if text.len() > 10000 {
            return Err(AdocError::lexer("Text content too long"));
        }
    }
    
    if text.is_empty() {
        // Single character token (should not happen here since we start)
        let c = self.advance().unwrap() as char;
        Ok(AdToken::new(AdTokenKind::Text, c.to_string(), line, column))
    } else {
        Ok(AdToken::new(AdTokenKind::Text, text, line, column))
    }
}

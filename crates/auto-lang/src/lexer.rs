use crate::error::{AutoResult, LexerError};
use crate::token::Pos;
use crate::token::{Token, TokenKind};
use std::collections::VecDeque;
use std::iter::Peekable;
use std::str::Chars;

pub struct Lexer<'a> {
    chars: Peekable<Chars<'a>>,
    line: usize,
    pos: usize,
    at: usize,
    buffer: VecDeque<Token>,
    last: Option<Token>,
    // special tokens
    fstr_note: char,
}

impl<'a> Lexer<'a> {
    pub fn new(code: &'a str) -> Self {
        Lexer {
            chars: code.chars().peekable(),
            line: 1,
            at: 0,
            pos: 0,
            buffer: VecDeque::new(),
            last: None,
            fstr_note: '$',
        }
    }

    pub fn set_fstr_note(&mut self, c: char) {
        self.fstr_note = c;
    }

    pub fn pos(&mut self, len: usize) -> Pos {
        let p = Pos {
            line: self.line,
            at: self.at,
            pos: self.pos,
            len,
        };
        self.pos += len;
        self.at += len;
        p
    }

    pub fn single(&mut self, kind: TokenKind, c: char) -> Token {
        let tok = Token::new(kind, self.pos(1), c.into());
        self.chars.next();
        tok
    }

    pub fn peek(&mut self, c: char) -> bool {
        if let Some(&nc) = self.chars.peek() {
            if nc == c {
                return true;
            }
        }
        false
    }

    pub fn peek_non_whitespace(&mut self, c: char) -> bool {
        let iter = self.chars.clone();
        for i in iter {
            if !i.is_whitespace() {
                return c == i;
            }
        }
        false
    }

    pub fn with_equal(&mut self, kind1: TokenKind, kind2: TokenKind, c: char) -> Token {
        self.chars.next(); // skip c
        if self.peek('=') {
            self.chars.next(); // skip =
            return Token::new(kind2, self.pos(2), format!("{}{}", c, '=').into());
        }
        Token::new(kind1, self.pos(1), c.into())
    }

    /// Push a token back to the front of the buffer.
    /// This is used for lookahead operations where tokens need to be restored.
    pub fn push_token(&mut self, token: Token) {
        self.buffer.push_front(token);
    }
}

// Lexer methods for various token types
impl<'a> Lexer<'a> {
    pub fn number(&mut self) -> Token {
        let mut text = String::new();
        let mut has_dot = false;
        let mut is_hex = false;
        if self.peek('0') {
            text.push('0');
            self.chars.next();

            if self.peek('x') {
                text.push('x');
                self.chars.next();
                is_hex = true;
            }
        }
        while let Some(&c) = self.chars.peek() {
            if c.is_digit(10) {
                text.push(c);
                self.chars.next();
            } else if c == '_' {
                self.chars.next();
            } else if c == '.' {
                let mut more = self.chars.clone();
                more.next();
                match more.peek() {
                    Some(c) => {
                        // floats that ends with a dot (like "10.") is not allowed because we want methods on int like `10.str()`
                        if !c.is_digit(10) {
                            break;
                        }
                    }
                    _ => {
                        break;
                    }
                }
                has_dot = true;
                text.push(c);
                self.chars.next();
            } else if is_hex && c.is_digit(16) {
                text.push(c);
                self.chars.next();
            } else {
                break;
            }
        }

        if has_dot {
            if self.peek('f') {
                self.chars.next();
                Token::float(self.pos(text.len()), text.into())
            } else if self.peek('d') {
                self.chars.next();
                Token::double(self.pos(text.len()), text.into())
            } else {
                Token::float(self.pos(text.len()), text.into())
            }
        } else {
            // check trailing character
            if self.peek('f') {
                self.chars.next();
                Token::float(self.pos(text.len()), text.into())
            } else if self.peek('d') {
                self.chars.next();
                Token::double(self.pos(text.len()), text.into())
            } else if self.peek('u') {
                self.chars.next();
                if self.peek('8') {
                    self.chars.next(); // skip 8
                    Token::u8(self.pos(text.len()), text.into())
                } else {
                    Token::uint(self.pos(text.len()), text.into())
                }
            } else if self.peek('i') {
                self.chars.next(); // skip i
                if self.peek('8') {
                    self.chars.next(); // skip 8
                    Token::i8(self.pos(text.len()), text.into())
                } else {
                    Token::int(self.pos(text.len()), text.into())
                }
            } else {
                Token::int(self.pos(text.len()), text.into())
            }
        }
    }

    fn char(&mut self) -> AutoResult<Token> {
        let start_pos = self.pos; // Record position BEFORE skipping '
        self.chars.next(); // skip '
        if let Some(&c) = self.chars.peek() {
            // deal with escapes
            if self.peek('\\') {
                self.chars.next(); // skip \
                if let Some(&esc_char) = self.chars.peek() {
                    match esc_char {
                        'n' => {
                            // \n
                            self.chars.next(); // skip char
                            self.chars.next(); // skip '
                            return Ok(Token::char(self.pos(1), '\n'.into()));
                        }
                        't' => {
                            // \t
                            self.chars.next(); // skip char
                            self.chars.next(); // skip '
                            return Ok(Token::char(self.pos(1), '\t'.into()));
                        }
                        'r' => {
                            // \r
                            self.chars.next(); // skip char
                            self.chars.next(); // skip '
                            return Ok(Token::char(self.pos(1), '\r'.into()));
                        }
                        '0' => {
                            // \0
                            self.chars.next(); // skip char
                            self.chars.next(); // skip '
                            return Ok(Token::char(self.pos(1), '\0'.into()));
                        }
                        _ => {
                            let seq = format!("\\{}", esc_char);
                            let span = crate::error::span_from(start_pos, seq.len());
                            return Err(LexerError::UnknownEscapeSequence {
                                sequence: seq,
                                span,
                            }
                            .into());
                        }
                    }
                } else {
                    // Unexpected end of input after backslash
                    let span = crate::error::span_from(start_pos, 1);
                    return Err(LexerError::UnknownEscapeSequence {
                        sequence: "\\".to_string(),
                        span,
                    }
                    .into());
                }
            } else {
                let tok = Token::char(self.pos(1), c.into());
                self.chars.next(); // skip char
                if self.peek('\'') {
                    self.chars.next(); // skip '
                    Ok(tok)
                } else {
                    let span = crate::error::span_from(start_pos, 1);
                    Err(LexerError::UnterminatedChar { span }.into())
                }
            }
        } else {
            let span = crate::error::span_from(start_pos, 1);
            Err(LexerError::EmptyChar { span }.into())
        }
    }

    pub fn str(&mut self) -> Token {
        let mut text = String::new();
        self.chars.next(); // skip opening "
        while let Some(&c) = self.chars.peek() {
            if c == '"' {
                self.chars.next();
                break;
            }
            if c == '\\' {
                self.chars.next(); // skip \
                if let Some(&esc) = self.chars.peek() {
                    match esc {
                        'n' => text.push('\n'),
                        't' => text.push('\t'),
                        'r' => text.push('\r'),
                        '0' => text.push('\0'),
                        '\\' => text.push('\\'),
                        '"' => text.push('"'),
                        _ => {
                            // Unknown escape, keep as-is
                            text.push('\\');
                            text.push(esc);
                        }
                    }
                    self.chars.next();
                    continue;
                }
            }
            text.push(c);
            self.chars.next();
        }
        Token::str(self.pos(text.len()), text.into())
    }

    fn cstr(&mut self) -> Token {
        self.chars.next(); // skip c
        self.chars.next(); // skip "

        let mut text = String::new();

        while let Some(&c) = self.chars.peek() {
            if c == '"' {
                self.chars.next();
                break;
            }
            text.push(c);
            self.chars.next();
        }
        Token::new(TokenKind::CStr, self.pos(text.len()), text.into())
    }

    fn fstr(&mut self) -> AutoResult<Token> {
        let mut endchar = '`';
        if self.peek('`') {
            let tk = self.single(TokenKind::FStrStart, '`');
            self.buffer.push_back(tk);
        } else {
            endchar = '"';
            self.chars.next(); // skip f
            self.chars.next(); // skip "
            let tk = Token::new(TokenKind::FStrStart, self.pos(2), "f\"".into());
            self.buffer.push_back(tk);
        }
        let mut text = String::new();
        while let Some(&c) = self.chars.peek() {
            if c == endchar {
                // got end
                if !text.is_empty() {
                    let tk = Token::fstr_part(self.pos(text.len()), text.into());
                    self.buffer.push_back(tk);
                }
                let tk = self.single(TokenKind::FStrEnd, endchar);
                self.buffer.push_back(tk);
                break;
            }
            if c == self.fstr_note {
                // text until $ is a string part
                let tk = Token::fstr_part(self.pos(text.len()), text.clone().into());
                self.buffer.push_back(tk);
                text.clear();
                // lex $
                let tk = self.single(TokenKind::FStrNote, c);
                self.buffer.push_back(tk);
                if let Some(&c) = self.chars.peek() {
                    if c == '{' {
                        self.fstr_expr()?;
                    } else {
                        // lex next data
                        let ident = self.identifier()?;
                        self.buffer.push_back(ident);
                    }
                }
            } else {
                text.push(c);
                self.chars.next();
            }
        }
        Ok(self.buffer.pop_front().unwrap())
    }

    fn fstr_expr(&mut self) -> AutoResult<()> {
        // push {
        let tk = self.single(TokenKind::LBrace, '{');
        self.buffer.push_back(tk);
        let mut level = 1;
        // tokens in the expression
        loop {
            let tk = self.next_step()?;
            let kind = tk.kind;
            self.buffer.push_back(tk);
            if kind == TokenKind::LBrace {
                level += 1;
            }
            if kind == TokenKind::RBrace {
                level -= 1;
                if level <= 0 {
                    break;
                }
            }
            if kind == TokenKind::EOF {
                break;
            }
        }
        Ok(())
        // if self.peek('}') {
        // let tk = self.single(TokenKind::RBrace, '}');
        // self.buffer.push_back(tk);
        // }
    }

    fn dot_or_range(&mut self) -> Token {
        self.chars.next(); // skip .
        if self.peek('.') {
            self.chars.next();
            if self.peek('=') {
                self.chars.next();
                return Token::new(TokenKind::RangeEq, self.pos(3), "..=".into());
            }
            return Token::new(TokenKind::Range, self.pos(2), "..".into());
        }

        // Check for ?. (error propagate operator - legacy May<T> style)
        if self.peek('?') {
            self.chars.next();
            return Token::new(TokenKind::DotQuestion, self.pos(2), "?.".into());
        }

        // Plan 120: Check for .? (error propagation operator - new Option/Result style)
        // This is handled by peeking ahead for '?' after the dot
        // We need to check if the next char is '?' (not peek, since we already consumed the dot)
        let iter_for_quest = self.chars.clone();
        if let Some('?') = iter_for_quest.clone().next() {
            self.chars.next(); // consume '?'
            return Token::new(TokenKind::DotQuest, self.pos(2), ".?".into());
        }

        // Check for property keywords: .view, .mut, .take
        // Clone the iterator to peek ahead without consuming
        let iter = self.chars.clone();
        let mut lookahead = String::new();

        for ch in iter {
            if ch.is_alphabetic() || ch == '_' || ch.is_numeric() {
                lookahead.push(ch);
            } else {
                break;
            }
        }

        // Check if it's a property keyword
        match lookahead.as_str() {
            "view" => {
                for _ in 0..4 {
                    self.chars.next();
                }
                return Token::new(TokenKind::DotView, self.pos(5), ".view ".into());
            }
            "mut" => {
                for _ in 0..3 {
                    self.chars.next();
                }
                return Token::new(TokenKind::DotMut, self.pos(4), ".mut ".into());
            }
            "move" => {
                for _ in 0..4 {
                    self.chars.next();
                }
                return Token::new(TokenKind::DotMove, self.pos(5), ".move ".into());
            }
            "take" => {
                for _ in 0..4 {
                    self.chars.next();
                }
                return Token::new(TokenKind::DotTake, self.pos(5), ".take ".into());
            }
            _ => {}
        }

        Token::new(TokenKind::Dot, self.pos(1), ".".into())
    }

    fn keyword_tok(&mut self, kind: TokenKind, text: &str) -> Option<Token> {
        Some(Token::new(kind, self.pos(text.len()), text.into()))
    }

    pub fn header_keyword(&mut self, text: String) -> Option<Token> {
        match text.as_str() {
            "on" => self.keyword_tok(TokenKind::On, &text),
            _ => None,
        }
    }

    pub fn keyword(&mut self, text: String) -> Option<Token> {
        match Token::keyword_kind(text.as_str()) {
            Some(kind) => self.keyword_tok(kind, &text),
            None => None,
        }
    }

    pub fn identifier_or_special_block(&mut self) -> AutoResult<Token> {
        let ident = self.identifier()?;
        // TODO: register special blocks dynamically
        if ident.text == "markdown" && self.peek_non_whitespace('{') {
            self.chars.next();
            let tk = self.single(TokenKind::LBrace, '{');
            self.buffer.push_back(tk);
            let mut code = String::new();
            while let Some(&c) = self.chars.peek() {
                if c == '}' {
                    self.chars.next();
                    break;
                }
                code.push(c);
                self.chars.next();
            }
            let code = Token::str(self.pos(code.len()), code.into());
            self.buffer.push_back(code);
            let tk = self.single(TokenKind::RBrace, '}');
            self.buffer.push_back(tk);
            Ok(ident)
        } else {
            Ok(ident)
        }
    }

    fn is_newstart(&mut self) -> bool {
        if let Some(last) = &self.last {
            return last.kind == TokenKind::Newline
                || last.kind == TokenKind::LBrace
                || last.kind == TokenKind::Semi;
        } else {
            true
        }
    }

    pub fn identifier(&mut self) -> AutoResult<Token> {
        let mut text = String::new();
        let start_pos = self.pos;
        // 第1个字符，必须是字母或下划线
        if let Some(&c) = self.chars.peek() {
            if !c.is_alphabetic() && c != '_' {
                let span = crate::error::span_from(start_pos, 1);
                return Err(LexerError::InvalidIdentifierStart {
                    character: c.to_string(),
                    span,
                }
                .into());
            } else {
                text.push(c);
                self.chars.next();
            }
        }
        while let Some(&c) = self.chars.peek() {
            if c.is_alphabetic() || c == '_' || c.is_digit(10) {
                text.push(c);
                self.chars.next();
            } else if c == '-' {
                // Hyphen: check if next char is a letter or underscore (not digit)
                // This allows "preview-card" but keeps "a-1" as "a - 1"
                let mut lookahead = self.chars.clone();
                lookahead.next(); // skip the '-'
                if let Some(&next) = lookahead.peek() {
                    if next.is_alphabetic() || next == '_' {
                        // Hyphen followed by letter/underscore, include in identifier
                        text.push(c);
                        self.chars.next();
                    } else {
                        // Hyphen followed by digit or other char, stop
                        break;
                    }
                } else {
                    // Hyphen at end of input, stop
                    break;
                }
            } else {
                break;
            }
        }
        // TODO: keyword detection should be dynamic, e.g. context dependent)
        if let Some(keyword) = self.keyword(text.clone()) {
            Ok(keyword)
        } else if self.is_newstart() {
            if let Some(header_keyword) = self.header_keyword(text.clone()) {
                Ok(header_keyword)
            } else {
                Ok(Token::ident(self.pos(text.len()), text.into()))
            }
        } else {
            Ok(Token::ident(self.pos(text.len()), text.into()))
        }
    }
}

impl<'a> Lexer<'a> {
    pub fn skip_whitespace(&mut self) {
        while let Some(&c) = self.chars.peek() {
            if c.is_whitespace() && c != '\n' {
                self.chars.next();
                // Update position tracking for accurate error reporting
                self.pos += 1;
                self.at += 1;
            } else {
                break;
            }
        }
    }
    pub fn next(&mut self) -> AutoResult<Token> {
        // skip whitespace
        self.skip_whitespace();
        if !self.buffer.is_empty() {
            let b = self.buffer.pop_front().unwrap();
            self.last = Some(b.clone());
            return Ok(b);
        }
        let t = self.next_step()?;
        self.last = Some(t.clone());
        Ok(t)
    }

    fn minus_or_arrow(&mut self, c: char) -> Token {
        self.chars.next(); // skip -
        if let Some(&next) = self.chars.peek() {
            if next == '>' {
                self.chars.next();
                Token::new(TokenKind::Arrow, self.pos(2), "->".into())
            } else if self.peek('=') {
                self.chars.next(); // skip =
                return Token::new(TokenKind::SubEq, self.pos(2), "-=".into());
            } else {
                Token::new(TokenKind::Sub, self.pos(1), c.into())
            }
        } else {
            Token::new(TokenKind::Sub, self.pos(1), c.into())
        }
    }

    fn equal_or_double_arrow(&mut self, c: char) -> Token {
        self.chars.next();
        if let Some(&next) = self.chars.peek() {
            if next == '>' {
                self.chars.next();
                Token::new(TokenKind::DoubleArrow, self.pos(2), "=>".into())
            } else if next == '=' {
                self.chars.next();
                Token::new(TokenKind::Eq, self.pos(2), "==".into())
            } else {
                Token::new(TokenKind::Asn, self.pos(1), c.into())
            }
        } else {
            Token::new(TokenKind::Asn, self.pos(1), c.into())
        }
    }

    fn question_or_operators(&mut self, c: char) -> Token {
        self.chars.next();
        if let Some(&next) = self.chars.peek() {
            if next == '?' {
                self.chars.next();
                Token::new(TokenKind::QuestionQuestion, self.pos(2), "??".into())
            } else {
                Token::new(TokenKind::Question, self.pos(1), c.into())
            }
        } else {
            Token::new(TokenKind::Question, self.pos(1), c.into())
        }
    }

    fn next_step(&mut self) -> AutoResult<Token> {
        self.skip_whitespace();
        while let Some(&c) = self.chars.peek() {
            match c {
                '(' => {
                    return Ok(self.single(TokenKind::LParen, c));
                }
                ')' => {
                    return Ok(self.single(TokenKind::RParen, c));
                }
                '[' => {
                    return Ok(self.single(TokenKind::LSquare, c));
                }
                ']' => {
                    return Ok(self.single(TokenKind::RSquare, c));
                }
                '{' => {
                    return Ok(self.single(TokenKind::LBrace, c));
                }
                '}' => {
                    return Ok(self.single(TokenKind::RBrace, c));
                }
                '\'' => {
                    return self.char();
                }
                '"' => {
                    return Ok(self.str());
                }
                '#' => {
                    // Plan 095: Check for comptime keywords (#if, #for, #is, #{)
                    // Clone iterator AFTER current char (#), so next() will give us the following char
                    let mut iter = self.chars.clone();
                    iter.next(); // skip the '#' that we're currently on

                    // Check for #if
                    if let Some('i') = iter.next() {
                        if let Some('f') = iter.next() {
                            let next_next = iter.clone().next();
                            if next_next.map(|c| !c.is_alphanumeric()).unwrap_or(true) {
                                self.chars.next(); // skip '#'
                                self.chars.next(); // skip 'i'
                                self.chars.next(); // skip 'f'
                                return Ok(Token::new(TokenKind::HashIf, self.pos(3), "#if".into()));
                            }
                        }
                        // Check for #is
                        let mut iter_s = self.chars.clone();
                        iter_s.next(); // skip '#'
                        iter_s.next(); // skip 'i'
                        if let Some('s') = iter_s.next() {
                            let next_next = iter_s.clone().next();
                            if next_next.map(|c| !c.is_alphanumeric()).unwrap_or(true) {
                                self.chars.next(); // skip '#'
                                self.chars.next(); // skip 'i'
                                self.chars.next(); // skip 's'
                                return Ok(Token::new(TokenKind::HashIs, self.pos(3), "#is".into()));
                            }
                        }
                    }

                    // Check for #for
                    let mut iter_for = self.chars.clone();
                    iter_for.next(); // skip '#'
                    if let Some('f') = iter_for.next() {
                        if let Some('o') = iter_for.next() {
                            if let Some('r') = iter_for.next() {
                                let next_next = iter_for.clone().next();
                                if next_next.map(|c| !c.is_alphanumeric()).unwrap_or(true) {
                                    self.chars.next(); // skip '#'
                                    self.chars.next(); // skip 'f'
                                    self.chars.next(); // skip 'o'
                                    self.chars.next(); // skip 'r'
                                    return Ok(Token::new(TokenKind::HashFor, self.pos(4), "#for".into()));
                                }
                            }
                        }
                    }

                    // Check for #{
                    let mut iter_brace = self.chars.clone();
                    iter_brace.next(); // skip '#'
                    if let Some('{') = iter_brace.next() {
                        self.chars.next(); // skip '#'
                        self.chars.next(); // skip '{'
                        return Ok(Token::new(TokenKind::HashBrace, self.pos(2), "#{".into()));
                    }

                    // Default: return Hash for #[...] annotations
                    return Ok(self.single(TokenKind::Hash, c));
                }
                'c' => {
                    let mut iter_copy = self.chars.clone();
                    iter_copy.next();
                    if let Some(next_char) = iter_copy.peek() {
                        if *next_char == '"' {
                            return Ok(self.cstr());
                        }
                    }
                    // If not followed by ", treat as identifier
                    return self.identifier();
                }
                'f' => {
                    let mut iter_copy = self.chars.clone();
                    iter_copy.next();
                    if let Some(next_char) = iter_copy.peek() {
                        if *next_char == '"' {
                            return self.fstr();
                        }
                    }
                    // If not followed by ", treat as identifier
                    return self.identifier();
                }
                ':' => {
                    return Ok(self.single(TokenKind::Colon, c));
                }
                ',' => {
                    return Ok(self.single(TokenKind::Comma, c));
                }
                ';' => {
                    return Ok(self.single(TokenKind::Semi, c));
                }
                '\n' => {
                    self.line += 1;
                    self.at = 0;
                    return Ok(self.single(TokenKind::Newline, c));
                }
                '+' => {
                    return Ok(self.with_equal(TokenKind::Add, TokenKind::AddEq, c));
                }
                '-' => {
                    return Ok(self.minus_or_arrow(c));
                }
                '*' => {
                    return Ok(self.with_equal(TokenKind::Star, TokenKind::MulEq, c));
                }
                '/' => {
                    return Ok(self.slash_or_comment());
                }
                '%' => {
                    return Ok(self.with_equal(TokenKind::Mod, TokenKind::ModEq, c));
                }
                '!' => {
                    return Ok(self.with_equal(TokenKind::Not, TokenKind::Neq, c));
                }
                '>' => {
                    return Ok(self.with_equal(TokenKind::Gt, TokenKind::Ge, c));
                }
                '<' => {
                    return Ok(self.with_equal(TokenKind::Lt, TokenKind::Le, c));
                }
                '=' => {
                    return Ok(self.equal_or_double_arrow(c));
                }
                '.' => {
                    return Ok(self.dot_or_range());
                }
                '|' => {
                    return Ok(self.single(TokenKind::VBar, c));
                }
                '`' => {
                    return self.fstr();
                }
                '?' => {
                    return Ok(self.question_or_operators(c));
                }
                '@' => {
                    return Ok(self.single(TokenKind::At, c));
                }
                '~' => {
                    return Ok(self.single(TokenKind::Tilde, c));
                }
                _ => {
                    if c == self.fstr_note {
                        return Ok(self.single(TokenKind::FStrNote, c));
                    }
                    if c.is_digit(10) {
                        return Ok(self.number());
                    }

                    if c.is_alphabetic() || c == '_' {
                        return self.identifier_or_special_block();
                    }

                    let span = crate::error::span_from(self.pos, 1);
                    return Err(LexerError::UnknownCharacter {
                        character: c.to_string(),
                        span,
                    }
                    .into());
                }
            }
        }
        Ok(Token::eof(self.pos(0)))
    }

    #[cfg(test)]
    fn tokens(&mut self) -> AutoResult<Vec<Token>> {
        let mut tokens = Vec::new();
        loop {
            let token = self.next()?;
            if token.kind == TokenKind::EOF {
                break;
            }
            tokens.push(token);
        }
        Ok(tokens)
    }

    #[cfg(test)]
    fn tokens_str(&mut self) -> AutoResult<String> {
        let tokens = self.tokens()?;
        Ok(tokens
            .iter()
            .map(|t| t.to_string())
            .collect::<Vec<String>>()
            .join(""))
    }

    fn slash_or_comment(&mut self) -> Token {
        self.chars.next();
        if self.peek('/') {
            // //
            let tok = Token::new(TokenKind::CommentLine, self.pos(2), "//".into());
            // content
            let mut text = String::new();
            while let Some(&c) = self.chars.peek() {
                if c == '\n' {
                    break;
                }
                text.push(c);
                self.chars.next();
            }
            let content = Token::new(TokenKind::CommentContent, self.pos(text.len()), text.into());
            self.buffer.push_back(content);
            tok
        } else if self.peek('*') {
            // /*
            let tok = Token::new(TokenKind::CommentStart, self.pos(2), "/*".into());
            // content
            let mut text = String::new();
            while let Some(&c) = self.chars.peek() {
                if c == '*' {
                    self.chars.next();
                    if self.peek('/') {
                        self.chars.next();
                        break;
                    }
                }
                text.push(c);
                self.chars.next();
            }
            let content = Token::new(TokenKind::CommentContent, self.pos(text.len()), text.into());
            self.buffer.push_back(content);
            // */
            let end = Token::new(TokenKind::CommentEnd, self.pos(2), "*/".into());
            self.buffer.push_back(end);
            tok
        } else if self.peek('=') {
            // /=
            self.chars.next(); // skip =
            Token::new(TokenKind::DivEq, self.pos(2), "/=".into())
        } else {
            Token::new(TokenKind::Div, self.pos(1), "/".into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_token_strings(code: &str) -> String {
        let mut lexer = Lexer::new(code);
        lexer.tokens_str().unwrap()
    }

    #[test]
    fn test_lexer() {
        let code = "(123)";
        let tokens = parse_token_strings(code);
        assert_eq!(tokens, "<(><int:123><)>");
    }

    #[test]
    fn test_str() {
        let code = "\"Hello, World!\"";
        let tokens = parse_token_strings(code);
        assert_eq!(tokens, "<str:Hello, World!>");
    }

    #[test]
    fn test_range() {
        let code = "1..5";
        let tokens = parse_token_strings(code);
        assert_eq!(tokens, "<int:1><..><int:5>");
    }

    #[test]
    fn test_pair() {
        let code = r#"a: 3
        b: 4"#;
        let tokens = parse_token_strings(code);
        assert_eq!(tokens, "<ident:a><:><int:3><nl><ident:b><:><int:4>");
    }

    #[test]
    fn test_fstr() {
        let code = r#"f"hello $you again""#;
        let tokens = parse_token_strings(code);
        assert_eq!(
            tokens,
            "<fstrs><fstrp:hello ><$><ident:you><fstrp: again><fstre>"
        );
    }

    #[test]
    fn test_fstr_expr() {
        let code = r#"f"hello ${2 + 1} again""#;
        let tokens = parse_token_strings(code);
        assert_eq!(
            tokens,
            "<fstrs><fstrp:hello ><$><{><int:2><+><int:1><}><fstrp: again><fstre>"
        );
    }

    #[test]
    fn test_tick_str() {
        let code = r#"`hello $you again`"#;
        let tokens = parse_token_strings(code);
        assert_eq!(
            tokens,
            "<fstrs><fstrp:hello ><$><ident:you><fstrp: again><fstre>"
        );
    }

    #[test]
    fn test_comment() {
        let code = r#"// this is a comment
        /* this is a multi-line comment */"#;
        let tokens = parse_token_strings(code);
        assert_eq!(tokens, "<//><comment:...><nl></*><comment:...><*/>");
    }

    #[test]
    fn test_fstr_multi() {
        let code = r#"`hello $name ${age}`"#;
        let tokens = parse_token_strings(code);
        assert_eq!(
            tokens,
            "<fstrs><fstrp:hello ><$><ident:name><fstrp: ><$><{><ident:age><}><fstre>"
        );
    }

    #[test]
    fn test_fstr_f() {
        let code = r#"f"hello $name""#;
        let tokens = parse_token_strings(code);
        assert_eq!(tokens, "<fstrs><fstrp:hello ><$><ident:name><fstre>");
    }

    #[test]
    fn test_uint() {
        let code = "125u";
        let tokens = parse_token_strings(code);
        assert_eq!(tokens, "<uint:125>");
    }

    #[test]
    fn test_u8() {
        let code = "125u8";
        let tokens = parse_token_strings(code);
        assert_eq!(tokens, "<u8:125>");
    }

    #[test]
    fn test_i8() {
        let code = "41i8";
        let tokens = parse_token_strings(code);
        assert_eq!(tokens, "<i8:41>");
    }

    #[test]
    fn test_path() {
        let code = "a.b.c: x, y";
        let tokens = parse_token_strings(code);
        assert_eq!(
            tokens,
            "<ident:a><.><ident:b><.><ident:c><:><ident:x><,><ident:y>"
        );
    }

    #[test]
    fn test_str_1() {
        let code = r#""Hello""#;
        let tokens = parse_token_strings(code);
        assert_eq!(tokens, "<str:Hello>");
    }

    #[test]
    fn test_str_escape_quote() {
        let code = r#""hello \"world\"""#;
        let tokens = parse_token_strings(code);
        assert_eq!(tokens, r#"<str:hello "world">"#);
    }

    #[test]
    fn test_str_escape_backslash() {
        let code = r#""path\\file""#;
        let tokens = parse_token_strings(code);
        assert_eq!(tokens, r#"<str:path\file>"#);
    }

    #[test]
    fn test_str_escape_newline() {
        let code = r#""line1\nline2""#;
        let tokens = parse_token_strings(code);
        assert_eq!(tokens, "<str:line1\nline2>");
    }

    #[test]
    fn test_str_escape_tab() {
        let code = r#""col1\tcol2""#;
        let tokens = parse_token_strings(code);
        assert_eq!(tokens, "<str:col1\tcol2>");
    }

    #[test]
    fn test_char() {
        let code = r#"'a'"#;
        let tokens = parse_token_strings(code);
        assert_eq!(tokens, "<'a'>");
    }

    #[test]
    fn test_fstr_with_note() {
        let fstr = r#"`hello #{2 + 1} again`"#;
        let mut lexer = Lexer::new(fstr);
        lexer.set_fstr_note('#');
        let tokens = lexer.tokens_str().unwrap();
        assert_eq!(
            tokens,
            "<fstrs><fstrp:hello ><#><{><int:2><+><int:1><}><fstrp: again><fstre>"
        );
    }

    #[test]
    fn test_markdown() {
        let code = r#"markdown {
        # hello
            This is a **test** for markdown
        }
        "#;
        let tokens = parse_token_strings(code);
        assert_eq!(tokens, "<ident:markdown><{><str:\n        # hello\n            This is a **test** for markdown\n        ><}>");
    }

    #[test]
    fn test_when() {
        let code = r#"is x {
            5 => print("x is 5")
            10 => print("x is 10")
            if x > 5 => print("x is greater than 5")
            else => print("x is else")
        }
        "#;
        let tokens = parse_token_strings(code);
        assert_eq!(
            tokens,
            format!(
                "{}{}{}{}{}",
                "<is><ident:x><{><nl>",
                "<int:5><=>><ident:print><(><str:x is 5><)><nl>",
                "<int:10><=>><ident:print><(><str:x is 10><)><nl>",
                "<if><ident:x><gt><int:5><=>><ident:print><(><str:x is greater than 5><)><nl>",
                "<else><=>><ident:print><(><str:x is else><)><nl><}><nl>"
            )
        );
    }

    #[test]
    fn test_fstr_lexer() {
        let code = r#"`${mid(){}}`"#;
        let tokens = parse_token_strings(code);
        assert_eq!(
            tokens,
            "<fstrs><fstrp:><$><{><ident:mid><(><)><{><}><}><fstre>"
        );
    }

    #[test]
    fn test_arrow() {
        let code = "5 -> 7";
        let tokens = parse_token_strings(code);
        assert_eq!(tokens, "<int:5><->><int:7>");
    }

    #[test]
    fn test_on() {
        let code = r#"on {
            5 -> 7
            6 -> 8 : 10
        }"#;
        let tokens = parse_token_strings(code);
        assert_eq!(
            tokens,
            "<on><{><nl><int:5><->><int:7><nl><int:6><->><int:8><:><int:10><nl><}>"
        )
    }

    #[test]
    fn test_on_in_fn() {
        let code = r#"fn on(ev str) {}"#;
        let tokens = parse_token_strings(code);
        assert_eq!(tokens, "<fn><ident:on><(><ident:ev><ident:str><)><{><}>")
    }

    #[test]
    fn test_use_c() {
        let code = "use c <stdio.h>";
        let tokens = parse_token_strings(code);
        assert_eq!(tokens, "<use><ident:c><lt><ident:stdio><.><ident:h><gt>");
    }

    #[test]
    fn test_cstr() {
        let code = "c\"hello\"";
        let tokens = parse_token_strings(code);
        assert_eq!(tokens, "<cstr:hello>");
    }

    #[test]
    fn test_at() {
        let code = "@int";
        let tokens = parse_token_strings(code);
        assert_eq!(tokens, "<@><ident:int>");
    }

    #[test]
    fn test_minus_one() {
        let code = "a-1";
        let tokens = parse_token_strings(code);
        assert_eq!(tokens, "<ident:a><-><int:1>");
    }

    #[test]
    fn test_router_keywords() {
        let code = "routes outlet link route nav";
        let tokens = parse_token_strings(code);
        assert_eq!(tokens, "<routes><outlet><link><route><nav>");
    }

    #[test]
    fn test_routes_token_text() {
        let code = "routes";
        let mut lexer = Lexer::new(code);
        let token = lexer.next().unwrap();

        assert_eq!(token.kind, TokenKind::Routes);
        assert_eq!(token.text.as_str(), "routes");
    }

    #[test]
    fn test_hyphenated_identifiers() {
        let code = "preview-card button-primary my-component";
        let tokens = parse_token_strings(code);
        assert_eq!(
            tokens,
            "<ident:preview-card><ident:button-primary><ident:my-component>"
        );
    }

    #[test]
    fn test_subtraction_with_spaces() {
        let code = "a - b";
        let tokens = parse_token_strings(code);
        assert_eq!(tokens, "<ident:a><-><ident:b>");
    }

    #[test]
    fn test_hyphen_identifier_vs_subtraction() {
        // a-b is identifier
        let tokens1 = parse_token_strings("a-b");
        assert_eq!(tokens1, "<ident:a-b>");

        // a - b is subtraction
        let tokens2 = parse_token_strings("a - b");
        assert_eq!(tokens2, "<ident:a><-><ident:b>");

        // a -b is a then unary minus
        let tokens3 = parse_token_strings("a -b");
        assert_eq!(tokens3, "<ident:a><-><ident:b>");
    }

    #[test]
    fn test_pac_keyword() {
        // Plan 131: Test that "pac" is recognized as a keyword
        let kind = Token::keyword_kind("pac");
        assert_eq!(kind, Some(TokenKind::Pac));

        // Test that "Pac" (capitalized) is NOT recognized as a keyword
        let kind = Token::keyword_kind("Pac");
        assert_eq!(kind, None);
    }

    #[test]
    fn test_super_keyword() {
        // Plan 131: Test that "super" is recognized as a keyword
        let kind = Token::keyword_kind("super");
        assert_eq!(kind, Some(TokenKind::Super));

        // Test that "Super" (capitalized) is NOT recognized as a keyword
        let kind = Token::keyword_kind("Super");
        assert_eq!(kind, None);
    }

    #[test]
    fn test_pac_keyword_in_use() {
        // Plan 131: Test pac keyword in use statement
        let code = "use pac.db";
        let tokens = parse_token_strings(code);
        assert_eq!(tokens, "<use><pac><.><ident:db>");
    }

    #[test]
    fn test_super_keyword_in_use() {
        // Plan 131: Test super keyword in use statement
        let code = "use super.db";
        let tokens = parse_token_strings(code);
        assert_eq!(tokens, "<use><super><.><ident:db>");
    }

    // Plan 095: Comptime token tests
    #[test]
    fn test_hash_if_token() {
        let code = "#if";
        let mut lexer = Lexer::new(code);
        let tok = lexer.next().unwrap();
        assert_eq!(tok.kind, TokenKind::HashIf);
        assert_eq!(tok.text.as_str(), "#if");
    }

    #[test]
    fn test_hash_for_token() {
        let code = "#for";
        let mut lexer = Lexer::new(code);
        let tok = lexer.next().unwrap();
        assert_eq!(tok.kind, TokenKind::HashFor);
        assert_eq!(tok.text.as_str(), "#for");
    }

    #[test]
    fn test_hash_is_token() {
        let code = "#is";
        let mut lexer = Lexer::new(code);
        let tok = lexer.next().unwrap();
        assert_eq!(tok.kind, TokenKind::HashIs);
        assert_eq!(tok.text.as_str(), "#is");
    }

    #[test]
    fn test_hash_brace_token() {
        let code = "#{";
        let mut lexer = Lexer::new(code);
        let tok = lexer.next().unwrap();
        assert_eq!(tok.kind, TokenKind::HashBrace);
        assert_eq!(tok.text.as_str(), "#{");
    }

    #[test]
    fn test_hash_alone_for_annotation() {
        // #[...] annotation syntax should still work
        let code = "#[";
        let mut lexer = Lexer::new(code);
        let tok = lexer.next().unwrap();
        assert_eq!(tok.kind, TokenKind::Hash);
    }

    #[test]
    fn test_hash_if_not_confused_with_identifier() {
        // #ifx should NOT be parsed as HashIf
        let code = "#ifx";
        let tokens = parse_token_strings(code);
        assert_eq!(tokens, "<#><ident:ifx>");
    }

    #[test]
    fn test_hash_for_not_confused_with_identifier() {
        // #form should NOT be parsed as HashFor
        let code = "#form";
        let tokens = parse_token_strings(code);
        assert_eq!(tokens, "<#><ident:form>");
    }
}

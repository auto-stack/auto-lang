use crate::token::Pos;
use crate::token::{Token, TokenKind};
use std::collections::VecDeque;
use std::iter::Peekable;
use std::str::Chars;

pub struct Lexer<'a> {
    chars: Peekable<Chars<'a>>,
    line: usize,
    pos: usize,
    buffer: VecDeque<Token>,
}

impl<'a> Lexer<'a> {
    pub fn new(code: &'a str) -> Self {
        Lexer {
            chars: code.chars().peekable(),
            line: 0,
            pos: 0,
            buffer: VecDeque::new(),
        }
    }

    pub fn pos(&mut self, len: usize) -> Pos {
        let p = Pos {
            line: self.line,
            pos: self.pos,
            len,
        };
        self.pos += len;
        p
    }

    pub fn single(&mut self, kind: TokenKind, c: char) -> Token {
        let tok = Token::new(kind, self.pos(1), c.to_string());
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

    pub fn with_equal(&mut self, kind1: TokenKind, kind2: TokenKind, c: char) -> Token {
        self.chars.next(); // skip c
        if self.peek('=') {
            self.chars.next(); // skip =
            return Token::new(kind2, self.pos(2), format!("{}{}", c, '='));
        }
        Token::new(kind1, self.pos(1), c.to_string())
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
            } else if c == '.' {
                let mut more = self.chars.clone();
                more.next();
                if more.peek() == Some(&'.') {
                    break;
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

    fn fstr(&mut self) -> Token {
        let mut text = String::new();
        if self.peek('`') {
            self.chars.next(); // skip `
        } else {
            self.chars.next(); // skip f
            self.chars.next(); // skip "
        }
        while let Some(&c) = self.chars.peek() {
            if c == '"' || c == '`' { // got end
                self.chars.next(); // skip ending " or `
                break;
            }
            if c == '$' {
                // lex fstr start
                let tk = Token::fstr_start(self.pos(text.len()), text.clone());
                self.buffer.push_back(tk);
                // lex $
                let tk = self.single(TokenKind::Dollar, '$');
                self.buffer.push_back(tk);
                if let Some(&c) = self.chars.peek() {
                    if c == '{' {
                        self.fstr_expr();
                    } else {
                        // lex next data
                        let ident = self.identifier();
                        self.buffer.push_back(ident);
                    }
                }
                // lex fstr end
                let fstr_end = self.fstr_end();
                self.buffer.push_back(fstr_end);
                break;
            }
            text.push(c);
            self.chars.next();
        }
        if self.buffer.is_empty() {
            Token::str(self.pos(text.len()), text)
        } else {
            self.buffer.pop_front().unwrap()
        }
    }

    fn fstr_end(&mut self) -> Token {
        let mut text = String::new();
        while let Some(&c) = self.chars.peek() {
            if c == '"' || c == '`' {
                self.chars.next();
                break;
            }
            text.push(c);
            self.chars.next();
        }
        Token::fstr_end(self.pos(text.len()), text)
    }

    fn fstr_expr(&mut self) {
        // push {
        let tk = self.single(TokenKind::LBrace, '{');
        self.buffer.push_back(tk);
        // tokens in the expression
        loop {
            let tk = self.next_step();
            let kind = tk.kind;
            self.buffer.push_back(tk);
            if kind == TokenKind::RBrace || kind == TokenKind::EOF {
                break;
            }
        }
    }

    fn dot_or_range(&mut self) -> Token {
        self.chars.next(); // skip .
        if self.peek('.') {
            self.chars.next();
            if self.peek('=') {
                self.chars.next();
                return Token::new(TokenKind::RangeEq, self.pos(3), "..=".to_string());
            }
            return Token::new(TokenKind::Range, self.pos(2), "..".to_string());
        }
        Token::new(TokenKind::Dot, self.pos(1), ".".to_string())
    }

    fn keyword_tok(&mut self, kind: TokenKind, text: &str) -> Option<Token> {
        Some(Token::new(kind, self.pos(text.len()), text.to_string()))
    }

    pub fn keyword(&mut self, text: String) -> Option<Token> {
        match text.as_str() {
            "true" => self.keyword_tok(TokenKind::True, &text),
            "false" => self.keyword_tok(TokenKind::False, &text),
            "nil" => self.keyword_tok(TokenKind::Nil, &text),
            "if" => self.keyword_tok(TokenKind::If, &text),
            "else" => self.keyword_tok(TokenKind::Else, &text),
            "for" => self.keyword_tok(TokenKind::For, &text),
            "var" => self.keyword_tok(TokenKind::Var, &text),
            "in" => self.keyword_tok(TokenKind::In, &text),
            "fn" => self.keyword_tok(TokenKind::Fn, &text),
            "type" => self.keyword_tok(TokenKind::Type, &text),
            _ => {
                // AutoUI Keywords
                // TODO: Add an Option to not check these keywords
                match text.as_str() {
                    "widget" => self.keyword_tok(TokenKind::Widget, &text),
                    "model" => self.keyword_tok(TokenKind::Model, &text),
                    "view" => self.keyword_tok(TokenKind::View, &text),
                    "style" => self.keyword_tok(TokenKind::Style, &text),
                    _ => None,
                }
            }
        }
    }

    pub fn identifier(&mut self) -> Token {
        let mut text = String::new();
        while let Some(&c) = self.chars.peek() {
            if c.is_alphabetic() || c == '_' {
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

impl<'a> Lexer<'a> {
    pub fn skip_whitespace(&mut self) {
        while let Some(&c) = self.chars.peek() {
            if c.is_whitespace() && c != '\n' {
                self.chars.next();
            } else {
                break;
            }
        }
    }
    pub fn next(&mut self) -> Token {
        // skip whitespace
        self.skip_whitespace();
        if !self.buffer.is_empty() {
            return self.buffer.pop_front().unwrap();
        }
        self.next_step()
    }

    fn next_step(&mut self) -> Token {
        self.skip_whitespace();
        while let Some(&c) = self.chars.peek() {
            match c {
                '(' => {
                    return self.single(TokenKind::LParen, c);
                }
                ')' => {
                    return self.single(TokenKind::RParen, c);
                }
                '[' => {
                    return self.single(TokenKind::LSquare, c);
                }
                ']' => {
                    return self.single(TokenKind::RSquare, c);
                }
                '{' => {
                    return self.single(TokenKind::LBrace, c);
                }
                '}' => {
                    return self.single(TokenKind::RBrace, c);
                }
                '"' => {
                    return self.str();
                }
                'f' => {
                    let mut iter_copy = self.chars.clone();
                    iter_copy.next();
                    if let Some(next_char) = iter_copy.peek() {
                        if *next_char == '"' {
                            return self.fstr();
                        } else {
                            return self.identifier();
                        }
                    }
                }
                ':' => {
                    return self.single(TokenKind::Colon, c);
                }
                ',' => {
                    return self.single(TokenKind::Comma, c);
                }
                ';' => {
                    return self.single(TokenKind::Semi, c);
                }
                '\n' => {
                    return self.single(TokenKind::Newline, c);
                }
                '+' => {
                    return self.single(TokenKind::Add, c);
                }
                '-' => {
                    return self.single(TokenKind::Sub, c);
                }
                '*' => {
                    return self.single(TokenKind::Mul, c);
                }
                '/' => {
                    return self.slash_or_comment();
                }
                '!' => {
                    return self.with_equal(TokenKind::Not, TokenKind::Neq, c);
                }
                '>' => {
                    return self.with_equal(TokenKind::Gt, TokenKind::Ge, c);
                }
                '<' => {
                    return self.with_equal(TokenKind::Lt, TokenKind::Le, c);
                }
                '=' => {
                    return self.with_equal(TokenKind::Asn, TokenKind::Eq, c);
                }
                '.' => {
                    return self.dot_or_range();
                }
                '|' => {
                    return self.single(TokenKind::VBar, c);
                }
                '$' => {
                    return self.single(TokenKind::Dollar, c);
                }
                '`' => {
                    return self.fstr();
                }
                _ => {
                    if c.is_digit(10) {
                        return self.number();
                    }

                    if c.is_alphabetic() {
                        return self.identifier();
                    }

                    panic!("unknown character: `{}`", c);
                    
                }
            }
        }
        Token::eof(self.pos(0))
    }

    fn tokens(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        loop {
            let token = self.next();
            if token.kind == TokenKind::EOF {
                break;
            }
            tokens.push(token);
        }
        tokens
    }

    fn tokens_str(&mut self) -> String {
        let tokens = self.tokens();
        tokens.iter().map(|t| t.to_string()).collect::<Vec<String>>().join("")
    }

    fn slash_or_comment(&mut self) -> Token {
        self.chars.next();
        if self.peek('/') {
            // //
            let tok = Token::new(TokenKind::CommentLine, self.pos(2), "//".to_string());
            // content
            let mut text = String::new();
            while let Some(&c) = self.chars.peek() {
                if c == '\n' {
                    break;
                }
                text.push(c);
                self.chars.next();
            }
            let content = Token::new(TokenKind::CommentContent, self.pos(text.len()), text);
            self.buffer.push_back(content);
            tok
        } else if self.peek('*') {
            // /*
            let tok = Token::new(TokenKind::CommentStart, self.pos(2), "/*".to_string());
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
            let content = Token::new(TokenKind::CommentContent, self.pos(text.len()), text);
            self.buffer.push_back(content);
            // */
            let end = Token::new(TokenKind::CommentEnd, self.pos(2), "*/".to_string());
            self.buffer.push_back(end);
            tok
        } else {
            self.single(TokenKind::Div, '/')
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_token_strings(code: &str) -> String {
        let mut lexer = Lexer::new(code);
        lexer.tokens_str()
    }

    #[test]
    fn test_lexer() {
        let code = "(123)";
        let tokens = parse_token_strings(code);
        assert_eq!(
            tokens,
            "<(><int:123><)>"
        );
    }

    #[test]
    fn test_str() {
        let code = "\"Hello, World!\"";
        let tokens = parse_token_strings(code);
        assert_eq!(
            tokens,
            "<str:Hello, World!>"
        );
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
        assert_eq!(tokens, "<fstrs:hello ><$><ident:you><fstre: again>");
    }

    #[test]
    fn test_fstr_expr() {
        let code = r#"f"hello ${2 + 1} again""#;
        let tokens = parse_token_strings(code);
        assert_eq!(tokens, "<fstrs:hello ><$><{><int:2><+><int:1><}><fstre: again>");
    }

    #[test]
    fn test_tick_str() {
        let code = r#"`hello $you again`"#;
        let tokens = parse_token_strings(code);
        assert_eq!(tokens, "<fstrs:hello ><$><ident:you><fstre: again>");
    }

    #[test]
    fn test_comment() {
        let code = r#"// this is a comment
        /* this is a multi-line comment */"#;
        let tokens = parse_token_strings(code);
        assert_eq!(tokens, "<//><comment:...><nl></*><comment:...><*/>");
    }
}

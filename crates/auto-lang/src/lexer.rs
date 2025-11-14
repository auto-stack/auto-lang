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

    fn char(&mut self) -> Token {
        self.chars.next(); // skip '
        if let Some(&c) = self.chars.peek() {
            let tok = Token::char(self.pos(1), c.into());
            self.chars.next(); // skip char
            if self.peek('\'') {
                self.chars.next(); // skip '
            } else {
                panic!("char must be ended by a '");
            }
            tok
        } else {
            panic!("char must be followed by a character");
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

    fn fstr(&mut self) -> Token {
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
                        self.fstr_expr();
                    } else {
                        // lex next data
                        let ident = self.identifier();
                        self.buffer.push_back(ident);
                    }
                }
            } else {
                text.push(c);
                self.chars.next();
            }
        }
        self.buffer.pop_front().unwrap()
    }

    fn fstr_expr(&mut self) {
        // push {
        let tk = self.single(TokenKind::LBrace, '{');
        self.buffer.push_back(tk);
        let mut level = 1;
        // tokens in the expression
        loop {
            let tk = self.next_step();
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

    pub fn identifier_or_special_block(&mut self) -> Token {
        let ident = self.identifier();
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
            ident
        } else {
            ident
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

    pub fn identifier(&mut self) -> Token {
        let mut text = String::new();
        // 第1个字符，必须是字母或下划线
        if let Some(&c) = self.chars.peek() {
            if !c.is_alphabetic() && c != '_' {
                panic!(
                    "identifier must start with a letter or underscore, got {}",
                    c
                );
            } else {
                text.push(c);
                self.chars.next();
            }
        }
        while let Some(&c) = self.chars.peek() {
            if c.is_alphabetic() || c == '_' || c.is_digit(10) {
                text.push(c);
                self.chars.next();
            } else {
                break;
            }
        }
        // TODO: keyword detection should be dynamic, e.g. context dependent)
        if let Some(keyword) = self.keyword(text.clone()) {
            keyword
        } else if self.is_newstart() {
            if let Some(header_keyword) = self.header_keyword(text.clone()) {
                header_keyword
            } else {
                Token::ident(self.pos(text.len()), text.into())
            }
        } else {
            Token::ident(self.pos(text.len()), text.into())
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
            let b = self.buffer.pop_front().unwrap();
            self.last = Some(b.clone());
            return b;
        }
        let t = self.next_step();
        self.last = Some(t.clone());
        t
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
                '\'' => {
                    return self.char();
                }
                '"' => {
                    return self.str();
                }
                'c' => {
                    let mut iter_copy = self.chars.clone();
                    iter_copy.next();
                    if let Some(next_char) = iter_copy.peek() {
                        if *next_char == '"' {
                            return self.cstr();
                        } else {
                            return self.identifier();
                        }
                    }
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
                    self.line += 1;
                    self.at = 0;
                    return self.single(TokenKind::Newline, c);
                }
                '+' => {
                    return self.with_equal(TokenKind::Add, TokenKind::AddEq, c);
                }
                '-' => {
                    return self.minus_or_arrow(c);
                }
                '*' => {
                    return self.with_equal(TokenKind::Star, TokenKind::MulEq, c);
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
                    return self.equal_or_double_arrow(c);
                }
                '.' => {
                    return self.dot_or_range();
                }
                '|' => {
                    return self.single(TokenKind::VBar, c);
                }
                '`' => {
                    return self.fstr();
                }
                '?' => {
                    return self.single(TokenKind::Question, c);
                }
                '@' => {
                    return self.single(TokenKind::At, c);
                }
                _ => {
                    if c == self.fstr_note {
                        return self.single(TokenKind::FStrNote, c);
                    }
                    if c.is_digit(10) {
                        return self.number();
                    }

                    if c.is_alphabetic() {
                        return self.identifier_or_special_block();
                    }

                    panic!("unknown character: `{}`", c);
                }
            }
        }
        Token::eof(self.pos(0))
    }

    #[cfg(test)]
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

    #[cfg(test)]
    fn tokens_str(&mut self) -> String {
        let tokens = self.tokens();
        tokens
            .iter()
            .map(|t| t.to_string())
            .collect::<Vec<String>>()
            .join("")
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
        } else {
            self.with_equal(TokenKind::Div, TokenKind::DivEq, '/')
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
        let tokens = lexer.tokens_str();
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
}

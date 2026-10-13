//! Strict Atom-subset parser for TestCase contracts (Plan 768 feasibility §7).
//!
//! Grammar: the static Atom subset accepted by `auto-atom::AtomParser`
//! (objects `{ key: value }`, arrays `[v, v]`, strings with `\n \t \r \\ \"`
//! escapes, int/float numbers with optional sign/fraction/exponent, `true` /
//! `false` / `null`, `//` and `/* */` comments, newlines as separators,
//! unquoted keys, nodes `name { key: value }`), with these strict additions
//! required by the contract layer:
//!
//! - duplicate object keys are a hard error (upstream `Obj::set` upserts and
//!   silently loses them — duplicates must die before information loss);
//! - node argument lists `name(args)` and bare child nodes inside bodies are
//!   rejected — contracts never use them;
//! - bare identifiers as values (other than `true`/`false`/`null`) are
//!   rejected;
//! - int/bool object keys are rejected (strings only).
//!
//! Parity with `auto-atom` on valid inputs is asserted by
//! `tests/atom_parity.rs`; where this parser is stricter it is by design.

/// A parsed contract value.
#[derive(Debug, Clone, PartialEq)]
pub enum CVal {
    Str(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    Null,
    List(Vec<CVal>),
    /// Object with ordered, duplicate-free `(key, value)` pairs.
    Obj(Vec<(String, CVal)>),
    /// Node `name { key: value }` with ordered, duplicate-free props.
    Node {
        name: String,
        props: Vec<(String, CVal)>,
    },
}

impl CVal {
    /// Object entries, or `None` for non-objects.
    pub fn as_obj(&self) -> Option<&[(String, CVal)]> {
        match self {
            CVal::Obj(pairs) => Some(pairs),
            _ => None,
        }
    }

    /// Node `(name, props)`, or `None` for non-nodes.
    pub fn as_node(&self) -> Option<(&str, &[(String, CVal)])> {
        match self {
            CVal::Node { name, props } => Some((name, props)),
            _ => None,
        }
    }

    pub fn type_name(&self) -> &'static str {
        match self {
            CVal::Str(_) => "string",
            CVal::Int(_) => "int",
            CVal::Float(_) => "float",
            CVal::Bool(_) => "bool",
            CVal::Null => "null",
            CVal::List(_) => "array",
            CVal::Obj(_) => "object",
            CVal::Node { .. } => "node",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AtomParseError {
    pub line: usize,
    pub col: usize,
    pub message: String,
}

impl std::fmt::Display for AtomParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "line {}, col {}: {}", self.line, self.col, self.message)
    }
}

impl std::error::Error for AtomParseError {}

struct Parser<'a> {
    input: &'a str,
    pos: usize,
    line: usize,
    col: usize,
}

/// Parse one contract value; trailing content is an error.
pub fn parse_contract(text: &str) -> Result<CVal, AtomParseError> {
    let mut p = Parser {
        input: text,
        pos: 0,
        line: 1,
        col: 1,
    };
    p.skip_ws_comments();
    let value = p.parse_value()?;
    p.skip_ws_comments();
    if p.peek().is_some() {
        return Err(p.error("unexpected trailing content after contract value"));
    }
    Ok(value)
}

impl<'a> Parser<'a> {
    fn error(&self, message: impl Into<String>) -> AtomParseError {
        AtomParseError {
            line: self.line,
            col: self.col,
            message: message.into(),
        }
    }

    fn peek(&self) -> Option<char> {
        self.input[self.pos..].chars().next()
    }

    fn advance(&mut self) -> Option<char> {
        let ch = self.peek()?;
        self.pos += ch.len_utf8();
        if ch == '\n' {
            self.line += 1;
            self.col = 1;
        } else {
            self.col += 1;
        }
        Some(ch)
    }

    fn starts_with(&self, s: &str) -> bool {
        self.input[self.pos..].starts_with(s)
    }

    fn skip_ws_comments(&mut self) {
        loop {
            while matches!(self.peek(), Some(ch) if ch.is_whitespace()) {
                self.advance();
            }
            if self.starts_with("//") {
                while let Some(ch) = self.advance() {
                    if ch == '\n' {
                        break;
                    }
                }
                continue;
            }
            if self.starts_with("/*") {
                self.advance();
                self.advance();
                while !self.starts_with("*/") && self.peek().is_some() {
                    self.advance();
                }
                if self.peek().is_none() {
                    return;
                }
                self.advance();
                self.advance();
                continue;
            }
            break;
        }
    }

    fn parse_value(&mut self) -> Result<CVal, AtomParseError> {
        self.skip_ws_comments();
        match self.peek() {
            Some('{') => self.parse_object(),
            Some('[') => self.parse_array(),
            Some('"') => Ok(CVal::Str(self.parse_string()?)),
            Some(ch) if ch.is_ascii_digit() || ch == '-' || ch == '+' => self.parse_number(),
            Some(ch) if is_ident_start(ch) => self.parse_ident_like(),
            Some(ch) => Err(self.error(format!("unexpected character '{}'", ch))),
            None => Err(self.error("unexpected end of input")),
        }
    }

    fn parse_string(&mut self) -> Result<String, AtomParseError> {
        // Caller guarantees a leading '"'.
        self.advance();
        let mut out = String::new();
        loop {
            match self.advance() {
                Some('"') => return Ok(out),
                Some('\\') => match self.advance() {
                    Some('n') => out.push('\n'),
                    Some('t') => out.push('\t'),
                    Some('r') => out.push('\r'),
                    Some('\\') => out.push('\\'),
                    Some('"') => out.push('"'),
                    Some(other) => out.push(other),
                    None => return Err(self.error("unterminated string escape")),
                },
                Some(ch) => out.push(ch),
                None => return Err(self.error("unterminated string literal")),
            }
        }
    }

    fn parse_number(&mut self) -> Result<CVal, AtomParseError> {
        let start = self.pos;
        let mut is_float = false;
        if matches!(self.peek(), Some('-') | Some('+')) {
            self.advance();
        }
        while matches!(self.peek(), Some(ch) if ch.is_ascii_digit()) {
            self.advance();
        }
        if self.peek() == Some('.') {
            is_float = true;
            self.advance();
            while matches!(self.peek(), Some(ch) if ch.is_ascii_digit()) {
                self.advance();
            }
        }
        if matches!(self.peek(), Some('e') | Some('E')) {
            is_float = true;
            self.advance();
            if matches!(self.peek(), Some('+') | Some('-')) {
                self.advance();
            }
            while matches!(self.peek(), Some(ch) if ch.is_ascii_digit()) {
                self.advance();
            }
        }
        let text = &self.input[start..self.pos];
        if text.is_empty() || text == "+" || text == "-" {
            return Err(self.error("invalid number"));
        }
        if is_float {
            text.parse::<f64>()
                .map(CVal::Float)
                .map_err(|_| self.error("invalid float"))
        } else {
            match text.parse::<i64>() {
                Ok(v) => Ok(CVal::Int(v)),
                Err(_) => Err(self.error("integer out of range (contracts use i64)")),
            }
        }
    }

    fn parse_ident_like(&mut self) -> Result<CVal, AtomParseError> {
        let ident = self.parse_identifier()?;
        match ident.as_str() {
            "true" => return Ok(CVal::Bool(true)),
            "false" => return Ok(CVal::Bool(false)),
            "null" => return Ok(CVal::Null),
            _ => {}
        }
        self.skip_ws_comments();
        match self.peek() {
            Some('(') => Err(self.error(format!(
                "node argument lists (`{}(…)`) are not part of the contract subset",
                ident
            ))),
            Some('{') => self
                .parse_node_body()
                .map(|props| CVal::Node { name: ident, props }),
            _ => Err(self.error(format!(
                "bare identifier `{}` is not a contract value (quote it as a string)",
                ident
            ))),
        }
    }

    fn parse_identifier(&mut self) -> Result<String, AtomParseError> {
        let start = self.pos;
        match self.peek() {
            Some(ch) if is_ident_start(ch) => self.advance(),
            Some(ch) => return Err(self.error(format!("expected identifier, found '{}'", ch))),
            None => return Err(self.error("expected identifier, reached end of input")),
        };
        while matches!(self.peek(), Some(ch) if is_ident_continue(ch)) {
            self.advance();
        }
        Ok(self.input[start..self.pos].to_string())
    }

    /// Parse `{ key: value … }` — node body or standalone object (caller has
    /// not consumed the `{`).
    fn parse_node_body(&mut self) -> Result<Vec<(String, CVal)>, AtomParseError> {
        self.skip_ws_comments();
        if self.peek() != Some('{') {
            return Err(self.error("expected '{'"));
        }
        self.advance();
        let mut pairs: Vec<(String, CVal)> = Vec::new();
        loop {
            self.skip_ws_comments();
            if self.peek() == Some('}') {
                break;
            }
            if self.peek().is_none() {
                return Err(self.error("unexpected end of input in object body"));
            }
            let key = if self.peek() == Some('"') {
                self.parse_string()?
            } else {
                self.parse_identifier()?
            };
            self.skip_ws_comments();
            if self.peek() != Some(':') {
                return Err(self.error(format!(
                    "expected ':' after key {:?} (bare child nodes and positional entries are not part of the contract subset)",
                    key
                )));
            }
            self.advance();
            let value = self.parse_value()?;
            if pairs.iter().any(|(k, _)| k == &key) {
                return Err(self.error(format!("duplicate object key {:?}", key)));
            }
            pairs.push((key, value));
            self.skip_ws_comments();
            if self.peek() == Some(',') {
                self.advance();
                continue;
            }
            if self.peek() == Some('}') {
                break;
            }
            // Newlines are separators; the next pair can start immediately.
            continue;
        }
        self.advance(); // '}'
        Ok(pairs)
    }

    fn parse_object(&mut self) -> Result<CVal, AtomParseError> {
        self.parse_node_body().map(CVal::Obj)
    }

    fn parse_array(&mut self) -> Result<CVal, AtomParseError> {
        self.advance(); // '['
        let mut items = Vec::new();
        loop {
            self.skip_ws_comments();
            if self.peek() == Some(']') {
                break;
            }
            if self.peek().is_none() {
                return Err(self.error("unexpected end of input in array"));
            }
            items.push(self.parse_value()?);
            self.skip_ws_comments();
            if self.peek() == Some(',') {
                self.advance();
                continue;
            }
            if self.peek() == Some(']') {
                break;
            }
            continue;
        }
        self.advance(); // ']'
        Ok(CVal::List(items))
    }
}

fn is_ident_start(ch: char) -> bool {
    ch.is_alphabetic() || ch == '_'
}

fn is_ident_continue(ch: char) -> bool {
    ch.is_alphanumeric() || ch == '_' || ch == '-'
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_primitives_and_containers() {
        let v = parse_contract(r#"case { a: 1, b: -2.5e2, c: "x\ny", d: true, e: null, f: [1, "two"], g: { h: "deep" } }"#).unwrap();
        let (name, props) = v.as_node().unwrap();
        assert_eq!(name, "case");
        assert_eq!(props.len(), 7);
        assert_eq!(props[0].1, CVal::Int(1));
        assert_eq!(props[1].1, CVal::Float(-250.0));
        assert_eq!(props[2].1, CVal::Str("x\ny".into()));
        assert_eq!(props[3].1, CVal::Bool(true));
        assert_eq!(props[4].1, CVal::Null);
        match &props[5].1 {
            CVal::List(items) => assert_eq!(items.len(), 2),
            other => panic!("expected list, got {:?}", other),
        }
    }

    #[test]
    fn comments_and_trailing_commas() {
        // A bare `{ … }` is an object; `name { … }` would be a NODE.
        let v = parse_contract("// lead\n{ /* mid */ k: \"v\", }\n").unwrap();
        assert_eq!(v.as_obj().unwrap().len(), 1);
    }

    #[test]
    fn duplicate_key_is_error() {
        let err = parse_contract(r#"obj { k: 1, k: 2 }"#).unwrap_err();
        assert!(err.message.contains("duplicate"), "{}", err.message);
    }

    #[test]
    fn unsupported_forms_are_clear_errors() {
        for (src, needle) in [
            ("n(a: 1) { }", "argument lists"),
            ("obj { child { k: 1 } }", "child nodes"),
            ("obj { k: bare }", "bare identifier"),
            ("obj { 1: 2 }", "identifier"), // int keys rejected
        ] {
            let err = parse_contract(src).unwrap_err();
            assert!(
                err.message.contains(needle),
                "{:?} -> {:?}",
                src,
                err.message
            );
        }
    }

    #[test]
    fn trailing_content_is_error() {
        assert!(parse_contract("obj { } extra").is_err());
    }
}

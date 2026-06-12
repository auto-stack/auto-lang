//! Static Atom parser.
//!
//! This parser reads the static subset of the Atom format (no variables,
//! functions, loops, or interpolation) and produces [`Atom`] values directly.
//! It is intentionally small and dependency-light so that downstream crates
//! such as AutoForge can parse Atom configuration without pulling in the full
//! AutoLang compiler.
//!
//! Supported features:
//! - Nodes: `name(args) { children }`
//! - Objects: `{ key: value }`
//! - Arrays: `[ value, value ]`
//! - Primitives: strings, integers, floats, booleans, null
//! - Unquoted object keys and node names
//! - `//` and `/* */` comments
//! - Newlines as item separators inside arrays, objects, and node bodies

use auto_val::{Array, AutoStr, Node, Obj, Value, ValueKey};
use crate::error::{AtomError, AtomResult};
use crate::Atom;

/// Parse static Atom text into an [`Atom`].
///
/// # Examples
///
/// ```rust
/// use auto_atom::AtomParser;
///
/// let atom = AtomParser::parse(r#"config { version: "1.0" }"#).unwrap();
/// assert!(atom.is_node());
/// ```
pub struct AtomParser;

impl AtomParser {
    /// Parse a complete Atom document.
    pub fn parse(input: &str) -> AtomResult<Atom> {
        let mut parser = Parser::new(input);
        parser.parse_document()
    }
}

struct Parser<'a> {
    input: &'a str,
    pos: usize,
    line: usize,
    col: usize,
}

impl<'a> Parser<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            input,
            pos: 0,
            line: 1,
            col: 1,
        }
    }

    fn error(&self, message: impl Into<String>) -> AtomError {
        AtomError::ParseError {
            line: self.line,
            column: self.col,
            message: message.into(),
        }
    }

    fn peek(&self) -> Option<char> {
        self.input[self.pos..].chars().next()
    }

    fn advance(&mut self) -> Option<char> {
        let ch = self.peek()?;
        let len = ch.len_utf8();
        self.pos += len;
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

    fn skip_whitespace_and_comments(&mut self) {
        loop {
            // Skip whitespace.
            while let Some(ch) = self.peek() {
                if ch.is_whitespace() {
                    self.advance();
                } else {
                    break;
                }
            }

            // Line comment.
            if self.starts_with("//") {
                while let Some(ch) = self.peek() {
                    self.advance();
                    if ch == '\n' {
                        break;
                    }
                }
                continue;
            }

            // Block comment.
            if self.starts_with("/*") {
                self.advance();
                self.advance();
                while !self.starts_with("*/") && self.peek().is_some() {
                    self.advance();
                }
                if self.peek().is_none() {
                    return;
                }
                self.advance(); // *
                self.advance(); // /
                continue;
            }

            break;
        }
    }

    fn expect_char(&mut self, expected: char) -> AtomResult<()> {
        self.skip_whitespace_and_comments();
        match self.peek() {
            Some(ch) if ch == expected => {
                self.advance();
                Ok(())
            }
            Some(ch) => Err(self.error(format!(
                "expected '{}' but found '{}'",
                expected, ch
            ))),
            None => Err(self.error(format!("expected '{}' but reached end of input", expected))),
        }
    }

    fn parse_document(&mut self) -> AtomResult<Atom> {
        self.skip_whitespace_and_comments();
        if self.peek().is_none() {
            return Ok(Atom::Empty);
        }
        let atom = self.parse_atom_value()?;
        self.skip_whitespace_and_comments();
        if self.peek().is_some() {
            return Err(self.error("unexpected trailing content after Atom value"));
        }
        Ok(atom)
    }

    /// Parse a top-level Atom value (node, object, array, or scalar).
    fn parse_atom_value(&mut self) -> AtomResult<Atom> {
        self.skip_whitespace_and_comments();
        match self.peek() {
            Some('{') => self.parse_object(),
            Some('[') => self.parse_array(),
            Some('"') => {
                let value = self.parse_string()?;
                Ok(Atom::new(value)?)
            }
            Some(ch) if ch.is_ascii_digit() || ch == '-' || ch == '+' => {
                let value = self.parse_number()?;
                Ok(Atom::new(value)?)
            }
            Some(ch) if is_ident_start(ch) => {
                let ident = self.parse_identifier()?;
                let ident_str = ident.as_str();
                match ident_str {
                    "true" | "false" | "null" => Ok(Atom::new(self.parse_identifier_value(ident)?)?),
                    _ => {
                        // Could be a node or a bare identifier value.
                        self.skip_whitespace_and_comments();
                        if self.peek() == Some('(') || self.peek() == Some('{') {
                            self.parse_node(ident)
                        } else {
                            Ok(Atom::new(self.parse_identifier_value(ident)?)?)
                        }
                    }
                }
            }
            Some(ch) => Err(self.error(format!("unexpected character '{}'", ch))),
            None => Err(self.error("unexpected end of input")),
        }
    }

    /// Parse a value usable as a property, array element, or argument.
    fn parse_value(&mut self) -> AtomResult<Value> {
        self.skip_whitespace_and_comments();
        match self.peek() {
            Some('{') => {
                let atom = self.parse_object()?;
                Ok(atom.to_value())
            }
            Some('[') => {
                let atom = self.parse_array()?;
                Ok(atom.to_value())
            }
            Some('"') => self.parse_string(),
            Some(ch) if ch.is_ascii_digit() || ch == '-' || ch == '+' => self.parse_number(),
            Some(ch) if is_ident_start(ch) => {
                let ident = self.parse_identifier()?;
                let ident_str = ident.as_str();
                match ident_str {
                    "true" | "false" | "null" | "nil" => self.parse_identifier_value(ident),
                    _ => {
                        self.skip_whitespace_and_comments();
                        if self.peek() == Some('(') || self.peek() == Some('{') {
                            let atom = self.parse_node(ident)?;
                            Ok(atom.to_value())
                        } else {
                            self.parse_identifier_value(ident)
                        }
                    }
                }
            }
            Some(ch) => Err(self.error(format!("unexpected character '{}'", ch))),
            None => Err(self.error("unexpected end of input")),
        }
    }

    fn parse_node(&mut self, name: AutoStr) -> AtomResult<Atom> {
        let mut node = Node::new(name);

        // Optional argument list: name(args)
        self.skip_whitespace_and_comments();
        if self.peek() == Some('(') {
            self.advance(); // '('
            self.parse_arg_list(&mut node)?;
            self.expect_char(')')?;
        }

        // Optional body: name { children }
        self.skip_whitespace_and_comments();
        if self.peek() == Some('{') {
            self.advance(); // '{'
            self.parse_node_body(&mut node)?;
            self.expect_char('}')?;
        }

        Ok(Atom::Node(node))
    }

    fn parse_arg_list(&mut self, node: &mut Node) -> AtomResult<()> {
        loop {
            self.skip_whitespace_and_comments();
            if self.peek() == Some(')') {
                break;
            }

            // Try to read a pair (key: value) or a positional value.
            let start = (self.line, self.col);
            let first = self.parse_value()?;

            self.skip_whitespace_and_comments();
            if self.peek() == Some(':') {
                // It was actually a key.
                let key = value_to_key(first, self, start)?;
                self.advance(); // ':'
                let value = self.parse_value()?;
                node.add_arg_unified(key, value);
            } else {
                // Positional argument.
                node.add_pos_arg_unified(first);
            }

            self.skip_whitespace_and_comments();
            if self.peek() == Some(',') {
                self.advance();
                continue;
            }
            if self.peek() == Some(')') {
                break;
            }
            // Newlines are whitespace; the next argument can start immediately.
            continue;
        }
        Ok(())
    }

    fn parse_node_body(&mut self, node: &mut Node) -> AtomResult<()> {
        loop {
            self.skip_whitespace_and_comments();
            if self.peek() == Some('}') {
                break;
            }
            if self.peek().is_none() {
                return Err(self.error("unexpected end of input in node body"));
            }

            // Try to parse as pair (key: value).
            let start = (self.line, self.col);
            let first = self.parse_value()?;

            self.skip_whitespace_and_comments();
            if self.peek() == Some(':') {
                let key = value_to_key(first, self, start)?;
                self.advance(); // ':'
                let value = self.parse_value()?;
                node.add_body_prop(key, value);
            } else if let Value::Node(child) = first {
                node.add_kid(child);
            } else {
                return Err(self.error(
                    "node body entries must be either pairs (key: value) or child nodes",
                ));
            }

            self.skip_whitespace_and_comments();
            if self.peek() == Some(',') {
                self.advance();
                continue;
            }
            if self.peek() == Some('}') {
                break;
            }
            // No explicit separator is required; newlines are whitespace, and
            // the next entry can start immediately.
            continue;
        }
        Ok(())
    }

    fn parse_object(&mut self) -> AtomResult<Atom> {
        self.expect_char('{')?;
        let mut obj = Obj::new();

        loop {
            self.skip_whitespace_and_comments();
            if self.peek() == Some('}') {
                break;
            }
            if self.peek().is_none() {
                return Err(self.error("unexpected end of input in object"));
            }

            let start = (self.line, self.col);
            let key_value = self.parse_value()?;
            let key = value_to_key(key_value, self, start)?;

            self.expect_char(':')?;
            let value = self.parse_value()?;
            obj.set(key, value);

            self.skip_whitespace_and_comments();
            if self.peek() == Some(',') {
                self.advance();
                continue;
            }
            if self.peek() == Some('}') {
                break;
            }
            // Newlines are whitespace; the next pair can start immediately.
            continue;
        }

        self.expect_char('}')?;
        Ok(Atom::Obj(obj))
    }

    fn parse_array(&mut self) -> AtomResult<Atom> {
        self.expect_char('[')?;
        let mut array = Array::new();

        loop {
            self.skip_whitespace_and_comments();
            if self.peek() == Some(']') {
                break;
            }
            if self.peek().is_none() {
                return Err(self.error("unexpected end of input in array"));
            }

            let value = self.parse_value()?;
            array.push(value);

            self.skip_whitespace_and_comments();
            if self.peek() == Some(',') {
                self.advance();
                continue;
            }
            if self.peek() == Some(']') {
                break;
            }
            // Newlines are whitespace; the next element can start immediately.
            continue;
        }

        self.expect_char(']')?;
        Ok(Atom::Array(array))
    }

    fn parse_string(&mut self) -> AtomResult<Value> {
        self.expect_char('"')?;
        let mut result = String::new();

        loop {
            match self.peek() {
                Some('"') => {
                    self.advance();
                    break;
                }
                Some('\\') => {
                    self.advance();
                    match self.advance() {
                        Some('n') => result.push('\n'),
                        Some('t') => result.push('\t'),
                        Some('r') => result.push('\r'),
                        Some('\\') => result.push('\\'),
                        Some('"') => result.push('"'),
                        Some(other) => result.push(other),
                        None => return Err(self.error("unterminated string escape")),
                    }
                }
                Some(ch) => {
                    self.advance();
                    result.push(ch);
                }
                None => return Err(self.error("unterminated string literal")),
            }
        }

        Ok(Value::Str(AutoStr::from(result.as_str())))
    }

    fn parse_number(&mut self) -> AtomResult<Value> {
        let start = self.pos;
        let mut is_float = false;

        // Optional sign.
        if let Some(ch) = self.peek() {
            if ch == '-' || ch == '+' {
                self.advance();
            }
        }

        // Integer part.
        while let Some(ch) = self.peek() {
            if ch.is_ascii_digit() {
                self.advance();
            } else {
                break;
            }
        }

        // Fractional part.
        if self.peek() == Some('.') {
            is_float = true;
            self.advance();
            while let Some(ch) = self.peek() {
                if ch.is_ascii_digit() {
                    self.advance();
                } else {
                    break;
                }
            }
        }

        // Exponent part.
        if let Some(ch) = self.peek() {
            if ch == 'e' || ch == 'E' {
                is_float = true;
                self.advance();
                if let Some(sign) = self.peek() {
                    if sign == '+' || sign == '-' {
                        self.advance();
                    }
                }
                while let Some(ch) = self.peek() {
                    if ch.is_ascii_digit() {
                        self.advance();
                    } else {
                        break;
                    }
                }
            }
        }

        let text = &self.input[start..self.pos];
        if is_float {
            text.parse::<f64>()
                .map(Value::Double)
                .map_err(|_| self.error(format!("invalid float literal: {}", text)))
        } else {
            text.parse::<i32>()
                .map(Value::Int)
                .map_err(|_| self.error(format!("invalid integer literal: {}", text)))
        }
    }

    fn parse_identifier(&mut self) -> AtomResult<AutoStr> {
        let start = self.pos;
        if let Some(ch) = self.peek() {
            if is_ident_start(ch) {
                self.advance();
            } else {
                return Err(self.error(format!("expected identifier start but found '{}'", ch)));
            }
        } else {
            return Err(self.error("expected identifier but reached end of input"));
        }

        while let Some(ch) = self.peek() {
            if is_ident_continue(ch) {
                self.advance();
            } else {
                break;
            }
        }

        Ok(AutoStr::from(&self.input[start..self.pos]))
    }

    fn parse_identifier_value(&mut self, ident: AutoStr) -> AtomResult<Value> {
        match ident.as_str() {
            "true" => Ok(Value::Bool(true)),
            "false" => Ok(Value::Bool(false)),
            "null" | "nil" => Ok(Value::Nil),
            _ => Ok(Value::Str(ident)),
        }
    }
}

fn is_ident_start(ch: char) -> bool {
    ch.is_alphabetic() || ch == '_'
}

fn is_ident_continue(ch: char) -> bool {
    ch.is_alphanumeric() || ch == '_' || ch == '-'
}

fn value_to_key(value: Value, _parser: &Parser, start: (usize, usize)) -> AtomResult<ValueKey> {
    match value {
        Value::Str(s) => Ok(ValueKey::Str(s)),
        Value::Int(i) => Ok(ValueKey::Int(i)),
        Value::Bool(b) => Ok(ValueKey::Bool(b)),
        _ => Err(AtomError::ParseError {
            line: start.0,
            column: start.1,
            message: "object key must be a string, integer, or boolean".to_string(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_empty() {
        assert!(AtomParser::parse("").unwrap().is_empty_atom());
        assert!(AtomParser::parse("   \n  ").unwrap().is_empty_atom());
    }

    #[test]
    fn parse_string() {
        let atom = AtomParser::parse(r#"["hello"]"#).unwrap();
        if let Atom::Array(arr) = atom {
            assert_eq!(arr.values[0], Value::Str("hello".into()));
        }
    }

    #[test]
    fn parse_number() {
        let atom = AtomParser::parse("[42, -3.14]").unwrap();
        if let Atom::Array(arr) = atom {
            assert_eq!(arr.values[0], Value::Int(42));
            assert_eq!(arr.values[1], Value::Double(-3.14));
        }
    }

    #[test]
    fn parse_bool_and_null() {
        let atom = AtomParser::parse("[true, false, null]").unwrap();
        if let Atom::Array(arr) = atom {
            assert_eq!(arr.values[0], Value::Bool(true));
            assert_eq!(arr.values[1], Value::Bool(false));
            assert_eq!(arr.values[2], Value::Nil);
        }
    }

    #[test]
    fn parse_array() {
        let atom = AtomParser::parse("[1, 2, 3]").unwrap();
        assert!(atom.is_array());
    }

    #[test]
    fn parse_object() {
        let atom = AtomParser::parse(r#"{ name: "Alice", age: 30 }"#).unwrap();
        assert!(atom.is_obj());
    }

    #[test]
    fn parse_node() {
        let atom = AtomParser::parse(r#"task_plan(id: "x") { title: "T" }"#).unwrap();
        assert!(atom.is_node());
    }

    #[test]
    fn parse_task_plan_example() {
        let input = r#"
        task_plan(id: "api_v2", version: 1) {
            title: "Build v2 API"
            phases: []
        }
        "#;
        let atom = AtomParser::parse(input).unwrap();
        assert!(atom.is_node());
    }

    #[test]
    fn comments_are_skipped() {
        let input = r#"
        // line comment
        task_plan { /* block */ title: "x" }
        "#;
        let atom = AtomParser::parse(input).unwrap();
        assert!(atom.is_node());
    }

    #[test]
    fn rejects_dynamic_syntax() {
        // We cannot easily detect all dynamic syntax, but interpolation is blocked
        // by the lexer (it never gets a chance to be evaluated).
        let result = AtomParser::parse(r#"{ name: #{var} }"#);
        assert!(result.is_err());
    }
}

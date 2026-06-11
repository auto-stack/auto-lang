//! from_json command - Parse JSON text into structured Value data

use crate::cmd::{Command, PipelineData, Signature};
use crate::cmd::parser::ParsedArgs;
use crate::shell::Shell;
use ash_core::pipeline::AtomPipeline;
use auto_val::{Array, Obj, Value};
use miette::{IntoDiagnostic, Result};

pub struct FromJsonCommand;

impl Command for FromJsonCommand {
    fn name(&self) -> &str {
        "from_json"
    }

    fn signature(&self) -> Signature {
        Signature::new("from_json", "Parse JSON string into structured Value")
    }

    fn run(
        &self,
        _args: &ParsedArgs,
        input: PipelineData,
        _shell: &mut Shell,
    ) -> Result<PipelineData> {
        let text = match input {
            PipelineData::Text(s) => s,
            PipelineData::Value(Value::Str(s)) => s.to_string(),
            _ => miette::bail!("from_json: input must be text"),
        };

        let value = parse_json(&text)?;
        Ok(PipelineData::from_value(value))
    }

    fn run_atom(
        &self,
        args: &ParsedArgs,
        input: AtomPipeline,
        shell: &mut Shell,
    ) -> Result<AtomPipeline> {
        let legacy_in = crate::cmd::pipeline_convert::atom_to_pipeline_data(input);
        let legacy_out = self.run(args, legacy_in, shell)?;
        Ok(crate::cmd::pipeline_convert::pipeline_data_to_atom(legacy_out))
    }
}

// ---------------------------------------------------------------------------
// Minimal recursive-descent JSON parser (no external crates)
// ---------------------------------------------------------------------------

struct JsonParser<'a> {
    chars: &'a [u8],
    pos: usize,
}

impl<'a> JsonParser<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            chars: input.as_bytes(),
            pos: 0,
        }
    }

    fn parse_value(&mut self) -> Result<Value> {
        self.skip_whitespace();
        if self.pos >= self.chars.len() {
            miette::bail!("from_json: unexpected end of input");
        }

        match self.chars[self.pos] {
            b'"' => self.parse_string().map(|s| Value::str(&s)),
            b'{' => self.parse_object(),
            b'[' => self.parse_array(),
            b't' => self.parse_literal("true", Value::Bool(true)),
            b'f' => self.parse_literal("false", Value::Bool(false)),
            b'n' => self.parse_literal("null", Value::Nil),
            b'-' | b'0'..=b'9' => self.parse_number(),
            c => miette::bail!(
                "from_json: unexpected character '{}' at position {}",
                c as char,
                self.pos
            ),
        }
    }

    fn parse_string(&mut self) -> Result<String> {
        self.expect(b'"')?;
        let mut result = String::new();

        while self.pos < self.chars.len() {
            let c = self.chars[self.pos];
            self.pos += 1;

            if c == b'"' {
                return Ok(result);
            }
            if c == b'\\' {
                if self.pos >= self.chars.len() {
                    miette::bail!("from_json: unterminated string escape");
                }
                let escaped = self.chars[self.pos];
                self.pos += 1;
                match escaped {
                    b'"' => result.push('"'),
                    b'\\' => result.push('\\'),
                    b'/' => result.push('/'),
                    b'n' => result.push('\n'),
                    b'r' => result.push('\r'),
                    b't' => result.push('\t'),
                    _ => result.push(escaped as char),
                }
            } else {
                result.push(c as char);
            }
        }

        miette::bail!("from_json: unterminated string")
    }

    fn parse_object(&mut self) -> Result<Value> {
        self.expect(b'{')?;
        let mut obj = Obj::new();

        self.skip_whitespace();
        if self.peek() == Some(b'}') {
            self.pos += 1;
            return Ok(Value::Obj(obj));
        }

        loop {
            self.skip_whitespace();
            let key = self.parse_string()?;

            self.skip_whitespace();
            self.expect(b':')?;

            let val = self.parse_value()?;
            obj.set(key.as_str(), val);

            self.skip_whitespace();
            match self.peek() {
                Some(b',') => {
                    self.pos += 1;
                }
                Some(b'}') => {
                    self.pos += 1;
                    break;
                }
                _ => miette::bail!("from_json: expected ',' or '}}' in object at pos {}", self.pos),
            }
        }

        Ok(Value::Obj(obj))
    }

    fn parse_array(&mut self) -> Result<Value> {
        self.expect(b'[')?;
        let mut arr = Array::new();

        self.skip_whitespace();
        if self.peek() == Some(b']') {
            self.pos += 1;
            return Ok(Value::Array(arr));
        }

        loop {
            let val = self.parse_value()?;
            arr.push(val);

            self.skip_whitespace();
            match self.peek() {
                Some(b',') => {
                    self.pos += 1;
                }
                Some(b']') => {
                    self.pos += 1;
                    break;
                }
                _ => miette::bail!("from_json: expected ',' or ']' in array at pos {}", self.pos),
            }
        }

        Ok(Value::Array(arr))
    }

    fn parse_number(&mut self) -> Result<Value> {
        let start = self.pos;

        if self.peek() == Some(b'-') {
            self.pos += 1;
        }

        while self.pos < self.chars.len() && self.chars[self.pos].is_ascii_digit() {
            self.pos += 1;
        }

        let is_float = self.pos < self.chars.len()
            && (self.chars[self.pos] == b'.' || self.chars[self.pos] == b'e' || self.chars[self.pos] == b'E');

        if is_float {
            if self.chars[self.pos] == b'.' {
                self.pos += 1;
                while self.pos < self.chars.len() && self.chars[self.pos].is_ascii_digit() {
                    self.pos += 1;
                }
            }
            if self.pos < self.chars.len() && (self.chars[self.pos] == b'e' || self.chars[self.pos] == b'E') {
                self.pos += 1;
                if self.pos < self.chars.len() && (self.chars[self.pos] == b'+' || self.chars[self.pos] == b'-') {
                    self.pos += 1;
                }
                while self.pos < self.chars.len() && self.chars[self.pos].is_ascii_digit() {
                    self.pos += 1;
                }
            }

            let s = std::str::from_utf8(&self.chars[start..self.pos]).into_diagnostic()?;
            let f: f64 = s.parse().into_diagnostic()?;
            Ok(Value::Float(f))
        } else {
            let s = std::str::from_utf8(&self.chars[start..self.pos]).into_diagnostic()?;
            let i: i64 = s.parse().into_diagnostic()?;
            Ok(Value::Int(i as i32))
        }
    }

    fn parse_literal(&mut self, expected: &str, value: Value) -> Result<Value> {
        let end = self.pos + expected.len();
        if end > self.chars.len() {
            miette::bail!("from_json: unexpected end of input, expected '{}'", expected);
        }
        if &self.chars[self.pos..end] != expected.as_bytes() {
            let found = std::str::from_utf8(&self.chars[self.pos..end]).unwrap_or("?");
            miette::bail!("from_json: expected '{}', found '{}'", expected, found);
        }
        self.pos = end;
        Ok(value)
    }

    fn skip_whitespace(&mut self) {
        while self.pos < self.chars.len() {
            match self.chars[self.pos] {
                b' ' | b'\t' | b'\n' | b'\r' => self.pos += 1,
                _ => break,
            }
        }
    }

    fn expect(&mut self, c: u8) -> Result<()> {
        self.skip_whitespace();
        if self.pos >= self.chars.len() || self.chars[self.pos] != c {
            miette::bail!(
                "from_json: expected '{}' at position {}",
                c as char,
                self.pos
            );
        }
        self.pos += 1;
        Ok(())
    }

    fn peek(&self) -> Option<u8> {
        self.chars.get(self.pos).copied()
    }
}

/// Parse a JSON string into a Value.
pub fn parse_json(input: &str) -> Result<Value> {
    let mut parser = JsonParser::new(input);
    let value = parser.parse_value()?;
    parser.skip_whitespace();
    if parser.pos != parser.chars.len() {
        miette::bail!(
            "from_json: trailing content at position {}",
            parser.pos
        );
    }
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_string() {
        let v = parse_json(r#""hello""#).unwrap();
        assert_eq!(v.as_str(), "hello");
    }

    #[test]
    fn test_parse_string_escapes() {
        let v = parse_json(r#""hello\nworld""#).unwrap();
        assert_eq!(v.as_str(), "hello\nworld");
    }

    #[test]
    fn test_parse_integer() {
        let v = parse_json("42").unwrap();
        assert_eq!(v, Value::Int(42));
    }

    #[test]
    fn test_parse_negative_integer() {
        let v = parse_json("-7").unwrap();
        assert_eq!(v, Value::Int(-7));
    }

    #[test]
    fn test_parse_float() {
        let v = parse_json("3.14").unwrap();
        assert_eq!(v, Value::Float(3.14));
    }

    #[test]
    fn test_parse_bool_true() {
        let v = parse_json("true").unwrap();
        assert_eq!(v, Value::Bool(true));
    }

    #[test]
    fn test_parse_bool_false() {
        let v = parse_json("false").unwrap();
        assert_eq!(v, Value::Bool(false));
    }

    #[test]
    fn test_parse_null() {
        let v = parse_json("null").unwrap();
        assert_eq!(v, Value::Nil);
    }

    #[test]
    fn test_parse_empty_array() {
        let v = parse_json("[]").unwrap();
        match v {
            Value::Array(arr) => assert_eq!(arr.len(), 0),
            _ => panic!("expected array"),
        }
    }

    #[test]
    fn test_parse_array() {
        let v = parse_json("[1, 2, 3]").unwrap();
        match v {
            Value::Array(arr) => {
                assert_eq!(arr.len(), 3);
                assert_eq!(arr[0], Value::Int(1));
                assert_eq!(arr[2], Value::Int(3));
            }
            _ => panic!("expected array"),
        }
    }

    #[test]
    fn test_parse_empty_object() {
        let v = parse_json("{}").unwrap();
        match v {
            Value::Obj(obj) => assert!(obj.get("x").is_none()),
            _ => panic!("expected object"),
        }
    }

    #[test]
    fn test_parse_object() {
        let v = parse_json(r#"{"name": "Alice", "age": 30}"#).unwrap();
        match v {
            Value::Obj(obj) => {
                assert_eq!(obj.get("name").unwrap().as_str(), "Alice");
                assert_eq!(obj.get("age").unwrap(), Value::Int(30));
            }
            _ => panic!("expected object"),
        }
    }

    #[test]
    fn test_parse_nested() {
        let v = parse_json(r#"{"users": [{"name": "Bob"}, {"name": "Eve"}]}"#).unwrap();
        match v {
            Value::Obj(obj) => {
                let users = obj.get("users").unwrap();
                match users {
                    Value::Array(arr) => assert_eq!(arr.len(), 2),
                    _ => panic!("expected array"),
                }
            }
            _ => panic!("expected object"),
        }
    }

    #[test]
    fn test_command_name() {
        let cmd = FromJsonCommand;
        assert_eq!(cmd.name(), "from_json");
    }
}

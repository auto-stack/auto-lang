//! `url-encode` command - URL encode/decode text
//!
//! Encodes or decodes text using percent-encoding for common characters.
//! Use `--decode` flag to decode instead of encode.

use crate::cmd::{Command, PipelineData, Signature};
use crate::cmd::parser::ParsedArgs;
use crate::shell::Shell;
use ash_core::pipeline::{Atom, AtomPipeline, AtomType};
use auto_val::Value;
use miette::Result;

pub struct UrlEncodeCommand;

impl Command for UrlEncodeCommand {
    fn name(&self) -> &str {
        "url-encode"
    }

    fn signature(&self) -> Signature {
        Signature::new("url-encode", "URL-encode or decode text")
            .required("text", "Text to encode or decode")
            .flag("decode", "Decode instead of encode")
    }

    fn run(
        &self,
        args: &ParsedArgs,
        input: PipelineData,
        _shell: &mut Shell,
    ) -> Result<PipelineData> {
        let text = get_text(args, &input)?;
        let result = if args.has_flag("decode") {
            percent_decode(&text)
        } else {
            percent_encode(&text)
        };
        Ok(PipelineData::from_text(result))
    }

    fn run_atom(
        &self,
        args: &ParsedArgs,
        input: AtomPipeline,
        _shell: &mut Shell,
    ) -> Result<AtomPipeline> {
        let legacy_in = crate::cmd::pipeline_convert::atom_to_pipeline_data(input);
        let text = get_text(args, &legacy_in)?;
        let result = if args.has_flag("decode") {
            percent_decode(&text)
        } else {
            percent_encode(&text)
        };
        Ok(AtomPipeline::from_atom(Atom::new(
            Value::str(&result),
            AtomType::Text,
        )))
    }
}

fn get_text(args: &ParsedArgs, input: &PipelineData) -> Result<String> {
    if let Some(text) = args.first() {
        return Ok(text.to_string());
    }
    match input {
        PipelineData::Text(s) if !s.is_empty() => Ok(s.clone()),
        PipelineData::Value(Value::Str(s)) => Ok(s.as_str().to_string()),
        _ => miette::bail!("url-encode: no text provided"),
    }
}

/// Percent-encode a string, encoding all non-unreserved characters.
/// Unreserved chars per RFC 3986: A-Z a-z 0-9 - _ . ~
pub fn percent_encode(input: &str) -> String {
    let mut out = String::with_capacity(input.len() * 3);
    for byte in input.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(byte as char);
            }
            _ => {
                out.push('%');
                out.push_str(&format!("{:02X}", byte));
            }
        }
    }
    out
}

/// Decode a percent-encoded string.
pub fn percent_decode(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut chars = input.bytes();

    while let Some(b) = chars.next() {
        if b == b'%' {
            let hi = chars.next();
            let lo = chars.next();
            match (hi, lo) {
                (Some(h), Some(l)) => {
                    if let (Some(hv), Some(lv)) = (hex_val(h), hex_val(l)) {
                        out.push(((hv << 4) | lv) as char);
                    } else {
                        out.push('%');
                        out.push(h as char);
                        out.push(l as char);
                    }
                }
                _ => {
                    out.push('%');
                    if let Some(h) = hi {
                        out.push(h as char);
                    }
                }
            }
        } else {
            out.push(b as char);
        }
    }
    out
}

fn hex_val(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_encode_command_name() {
        let cmd = UrlEncodeCommand;
        assert_eq!(cmd.name(), "url-encode");
    }

    #[test]
    fn test_percent_encode_basic() {
        assert_eq!(percent_encode("hello world"), "hello%20world");
    }

    #[test]
    fn test_percent_encode_unreserved() {
        assert_eq!(
            percent_encode("ABCxyz0123-_.~"),
            "ABCxyz0123-_.~"
        );
    }

    #[test]
    fn test_percent_encode_special() {
        assert_eq!(percent_encode("a=b&c=d"), "a%3Db%26c%3Dd");
    }

    #[test]
    fn test_percent_encode_unicode() {
        // UTF-8 bytes for "hi" (U+4F60): e4 bd a0
        assert_eq!(percent_encode("hi"), "hi");
    }

    #[test]
    fn test_percent_decode_basic() {
        assert_eq!(percent_decode("hello%20world"), "hello world");
    }

    #[test]
    fn test_percent_decode_unreserved() {
        assert_eq!(percent_decode("ABCxyz0123"), "ABCxyz0123");
    }

    #[test]
    fn test_percent_decode_special() {
        assert_eq!(percent_decode("a%3Db%26c%3Dd"), "a=b&c=d");
    }

    #[test]
    fn test_percent_decode_lowercase_hex() {
        assert_eq!(percent_decode("hello%20world"), "hello world");
        assert_eq!(percent_decode("a%3db"), "a=b");
    }

    #[test]
    fn test_percent_decode_incomplete() {
        assert_eq!(percent_decode("%2"), "%2");
        assert_eq!(percent_decode("%"), "%");
    }

    #[test]
    fn test_roundtrip() {
        let inputs = vec![
            "hello world",
            "foo=bar&baz=qux",
            "path/to/file?query=value&a=b",
            "",
            "simple",
        ];
        for input in inputs {
            assert_eq!(percent_decode(&percent_encode(input)), input);
        }
    }

    #[test]
    fn test_hex_val() {
        assert_eq!(hex_val(b'0'), Some(0));
        assert_eq!(hex_val(b'9'), Some(9));
        assert_eq!(hex_val(b'a'), Some(10));
        assert_eq!(hex_val(b'f'), Some(15));
        assert_eq!(hex_val(b'A'), Some(10));
        assert_eq!(hex_val(b'F'), Some(15));
        assert_eq!(hex_val(b'g'), None);
    }
}

//! from_xml command - Parse XML text into structured Value
//!
//! Converts XML to a nested Obj structure:
//! ```text
//! {
//!   tag: "element_name",
//!   attrs: { key: value, ... },
//!   children: [ ... ],
//!   text: "content"  (optional, for text nodes)
//! }
//! ```

use crate::cmd::{Command, PipelineData, Signature};
use crate::cmd::parser::ParsedArgs;
use crate::shell::Shell;
use ash_core::pipeline::AtomPipeline;
use auto_val::{Array, Obj, Value};
use miette::Result;

pub struct FromXmlCommand;

impl Command for FromXmlCommand {
    fn name(&self) -> &str {
        "from_xml"
    }

    fn signature(&self) -> Signature {
        Signature::new("from_xml", "Parse XML string into structured Value")
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
            _ => miette::bail!("from_xml: input must be text"),
        };

        let value = parse_xml(&text)?;
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
// Simple XML parser
// ---------------------------------------------------------------------------

struct XmlParser<'a> {
    chars: &'a [u8],
    pos: usize,
}

impl<'a> XmlParser<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            chars: input.as_bytes(),
            pos: 0,
        }
    }

    fn parse(&mut self) -> Result<Value> {
        self.skip_whitespace();

        // Skip XML declaration <?xml ...?>
        if self.starts_with("<?") {
            self.skip_until("?>")?;
            self.skip_whitespace();
        }

        // Skip comments <!-- ... -->
        while self.starts_with("<!--") {
            self.skip_until("-->")?;
            self.skip_whitespace();
        }

        // Parse root element
        let root = self.parse_element()?;
        Ok(root)
    }

    fn parse_element(&mut self) -> Result<Value> {
        self.expect_char('<')?;
        let tag = self.parse_name()?;

        // Parse attributes
        let mut attrs = Obj::new();
        loop {
            self.skip_whitespace();
            if self.starts_with("/>") {
                self.pos += 2;
                // Self-closing element
                let mut obj = Obj::new();
                obj.set("tag", Value::str(&tag));
                obj.set("attrs", Value::Obj(attrs));
                obj.set("children", Value::Array(Array::new()));
                return Ok(Value::Obj(obj));
            }
            if self.starts_with(">") {
                self.pos += 1;
                break;
            }
            // Parse attribute
            let attr_name = self.parse_name()?;
            self.skip_whitespace();
            self.expect_char('=')?;
            self.skip_whitespace();
            let attr_val = self.parse_attr_value()?;
            attrs.set(attr_name.as_str(), Value::str(&attr_val));
        }

        // Parse content (children + text)
        let mut children = Array::new();
        let mut text_content = String::new();

        loop {
            self.skip_whitespace();
            if self.pos >= self.chars.len() {
                miette::bail!("from_xml: unclosed element <{}>", tag);
            }

            if self.starts_with("</") {
                // Closing tag
                self.pos += 2;
                let closing_tag = self.parse_name()?;
                self.skip_whitespace();
                self.expect_char('>')?;

                if closing_tag != tag {
                    miette::bail!(
                        "from_xml: mismatched tags: <{}> closed by </{}>",
                        tag,
                        closing_tag
                    );
                }
                break;
            }

            if self.starts_with("<!--") {
                // Comment
                self.skip_until("-->")?;
                continue;
            }

            if self.chars[self.pos] == b'<' {
                // Child element
                let child = self.parse_element()?;
                children.push(child);
            } else {
                // Text content
                let text = self.parse_text()?;
                if !text.trim().is_empty() {
                    text_content.push_str(&text);
                }
            }
        }

        let mut obj = Obj::new();
        obj.set("tag", Value::str(&tag));
        obj.set("attrs", Value::Obj(attrs));
        obj.set("children", Value::Array(children));
        if !text_content.trim().is_empty() {
            obj.set("text", Value::str(text_content.trim()));
        }

        Ok(Value::Obj(obj))
    }

    fn parse_name(&mut self) -> Result<String> {
        let start = self.pos;
        while self.pos < self.chars.len() {
            let c = self.chars[self.pos];
            if c.is_ascii_alphanumeric() || c == b'-' || c == b'_' || c == b':' || c == b'.' {
                self.pos += 1;
            } else {
                break;
            }
        }
        if self.pos == start {
            miette::bail!("from_xml: expected element name at pos {}", self.pos);
        }
        String::from_utf8(self.chars[start..self.pos].to_vec())
            .map_err(|e| miette::miette!("from_xml: invalid UTF-8 in name: {}", e))
    }

    fn parse_attr_value(&mut self) -> Result<String> {
        let quote = self.chars[self.pos];
        if quote != b'"' && quote != b'\'' {
            miette::bail!("from_xml: expected quote at pos {}", self.pos);
        }
        self.pos += 1;
        let start = self.pos;
        while self.pos < self.chars.len() && self.chars[self.pos] != quote {
            self.pos += 1;
        }
        let val = String::from_utf8(self.chars[start..self.pos].to_vec())
            .map_err(|e| miette::miette!("from_xml: invalid UTF-8: {}", e))?;
        if self.pos < self.chars.len() {
            self.pos += 1; // skip closing quote
        }
        Ok(val)
    }

    fn parse_text(&mut self) -> Result<String> {
        let mut text = String::new();
        while self.pos < self.chars.len() && self.chars[self.pos] != b'<' {
            text.push(self.chars[self.pos] as char);
            self.pos += 1;
        }
        // Decode basic XML entities
        let decoded = text
            .replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&amp;", "&")
            .replace("&quot;", "\"")
            .replace("&apos;", "'");
        Ok(decoded)
    }

    fn skip_whitespace(&mut self) {
        while self.pos < self.chars.len() {
            match self.chars[self.pos] {
                b' ' | b'\t' | b'\n' | b'\r' => self.pos += 1,
                _ => break,
            }
        }
    }

    fn starts_with(&self, prefix: &str) -> bool {
        let end = self.pos + prefix.len();
        end <= self.chars.len() && &self.chars[self.pos..end] == prefix.as_bytes()
    }

    fn expect_char(&mut self, c: char) -> Result<()> {
        if self.pos >= self.chars.len() || self.chars[self.pos] != c as u8 {
            miette::bail!("from_xml: expected '{}' at pos {}", c, self.pos);
        }
        self.pos += 1;
        Ok(())
    }

    fn skip_until(&mut self, end: &str) -> Result<()> {
        let end_bytes = end.as_bytes();
        while self.pos + end_bytes.len() <= self.chars.len() {
            if &self.chars[self.pos..self.pos + end_bytes.len()] == end_bytes {
                self.pos += end_bytes.len();
                return Ok(());
            }
            self.pos += 1;
        }
        miette::bail!("from_xml: unclosed comment/declaration")
    }
}

/// Parse XML text into a Value.
pub fn parse_xml(text: &str) -> Result<Value> {
    let mut parser = XmlParser::new(text);
    parser.parse()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_element() {
        let xml = "<root>hello</root>";
        let val = parse_xml(xml).unwrap();
        match val {
            Value::Obj(obj) => {
                assert_eq!(obj.get("tag").unwrap().as_str(), "root");
                assert_eq!(obj.get("text").unwrap().as_str(), "hello");
            }
            _ => panic!("expected object"),
        }
    }

    #[test]
    fn test_self_closing() {
        let xml = "<br/>";
        let val = parse_xml(xml).unwrap();
        match val {
            Value::Obj(obj) => {
                assert_eq!(obj.get("tag").unwrap().as_str(), "br");
                match obj.get("children").unwrap() {
                    Value::Array(arr) => assert!(arr.is_empty()),
                    _ => panic!("expected array"),
                }
            }
            _ => panic!("expected object"),
        }
    }

    #[test]
    fn test_attributes() {
        let xml = r#"<div class="main" id="content">text</div>"#;
        let val = parse_xml(xml).unwrap();
        match val {
            Value::Obj(obj) => match obj.get("attrs").unwrap() {
                Value::Obj(attrs) => {
                    assert_eq!(attrs.get("class").unwrap().as_str(), "main");
                    assert_eq!(attrs.get("id").unwrap().as_str(), "content");
                }
                _ => panic!("expected object"),
            },
            _ => panic!("expected object"),
        }
    }

    #[test]
    fn test_nested_elements() {
        let xml = "<parent><child>text</child></parent>";
        let val = parse_xml(xml).unwrap();
        match val {
            Value::Obj(obj) => match obj.get("children").unwrap() {
                Value::Array(children) => {
                    assert_eq!(children.len(), 1);
                    match &children[0] {
                        Value::Obj(child) => {
                            assert_eq!(child.get("tag").unwrap().as_str(), "child");
                        }
                        _ => panic!("expected object"),
                    }
                }
                _ => panic!("expected array"),
            },
            _ => panic!("expected object"),
        }
    }

    #[test]
    fn test_xml_declaration() {
        let xml = "<?xml version=\"1.0\"?>\n<root>ok</root>";
        let val = parse_xml(xml).unwrap();
        match val {
            Value::Obj(obj) => {
                assert_eq!(obj.get("tag").unwrap().as_str(), "root");
            }
            _ => panic!("expected object"),
        }
    }

    #[test]
    fn test_comments_skipped() {
        let xml = "<root><!-- comment -->text</root>";
        let val = parse_xml(xml).unwrap();
        match val {
            Value::Obj(obj) => {
                assert_eq!(obj.get("text").unwrap().as_str(), "text");
            }
            _ => panic!("expected object"),
        }
    }

    #[test]
    fn test_xml_entities() {
        let xml = "<root>a &lt; b &amp; c</root>";
        let val = parse_xml(xml).unwrap();
        match val {
            Value::Obj(obj) => {
                assert_eq!(obj.get("text").unwrap().as_str(), "a < b & c");
            }
            _ => panic!("expected object"),
        }
    }

    #[test]
    fn test_empty_element() {
        let xml = "<root></root>";
        let val = parse_xml(xml).unwrap();
        match val {
            Value::Obj(obj) => {
                assert_eq!(obj.get("tag").unwrap().as_str(), "root");
                assert!(obj.get("text").is_none());
            }
            _ => panic!("expected object"),
        }
    }

    #[test]
    fn test_command_name() {
        let cmd = FromXmlCommand;
        assert_eq!(cmd.name(), "from_xml");
    }
}

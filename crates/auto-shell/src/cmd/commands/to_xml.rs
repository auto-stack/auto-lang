//! to_xml command - Convert structured Value to XML text
//!
//! Expects an Obj with {tag, attrs: {}, children: [], text: "..."} structure.
//! Flags: --root (root element name, default "root"), --indent (indent spaces, default 2)

use crate::cmd::{Command, PipelineData, Signature};
use crate::cmd::parser::ParsedArgs;
use crate::shell::Shell;
use ash_core::pipeline::AtomPipeline;
use auto_val::{Obj, Value};
use miette::Result;

pub struct ToXmlCommand;

impl Command for ToXmlCommand {
    fn name(&self) -> &str {
        "to_xml"
    }

    fn signature(&self) -> Signature {
        Signature::new("to_xml", "Convert structured Value to XML string")
            .optional("root", "Root element name (default: root)")
            .flag_with_short("root", 'r', "Root element name override")
            .flag_with_short("indent", 'i', "Indentation spaces (default: 2)")
    }

    fn run(
        &self,
        args: &ParsedArgs,
        input: PipelineData,
        _shell: &mut Shell,
    ) -> Result<PipelineData> {
        let value = match input {
            PipelineData::Value(v) => v,
            PipelineData::Text(s) => miette::bail!("to_xml: cannot convert text to XML; expected structured data"),
        };

        let indent: usize = if args.has_flag("indent") {
            args.positionals
                .iter()
                .find_map(|s| s.parse::<usize>().ok())
                .unwrap_or(2)
        } else {
            2
        };

        let root_name = if args.has_flag("root") {
            args.positionals
                .first()
                .map(|s| s.clone())
                .unwrap_or_else(|| "root".to_string())
        } else {
            "root".to_string()
        };

        let xml = value_to_xml(&value, &root_name, indent, 0);
        Ok(PipelineData::from_text(xml))
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
// Simple XML serializer
// ---------------------------------------------------------------------------

/// Convert a Value to XML text.
pub fn value_to_xml(value: &Value, root_name: &str, indent: usize, depth: usize) -> String {
    match value {
        Value::Obj(obj) => {
            // Check if it has the standard XML structure {tag, attrs, children}
            let tag = obj
                .get("tag")
                .map(|v| v.as_str().to_string())
                .unwrap_or_else(|| root_name.to_string());

            let attrs = obj
                .get("attrs")
                .and_then(|v| match v {
                    Value::Obj(o) => Some(o.clone()),
                    _ => None,
                })
                .unwrap_or_else(Obj::new);

            let children = obj
                .get("children")
                .and_then(|v| match v {
                    Value::Array(a) => Some(a.clone()),
                    _ => None,
                })
                .unwrap_or_default();

            let text = obj
                .get("text")
                .map(|v| v.as_str().to_string());

            element_to_xml(&tag, &attrs, &children, text.as_deref(), indent, depth)
        }
        _ => {
            // Wrap non-object value in root element
            let text = value.as_str().to_string();
            let prefix = " ".repeat(indent * depth);
            format!("{}<{}>{}</{}>", prefix, root_name, escape_xml(&text), root_name)
        }
    }
}

fn element_to_xml(
    tag: &str,
    attrs: &Obj,
    children: &auto_val::Array,
    text: Option<&str>,
    indent: usize,
    depth: usize,
) -> String {
    let prefix = " ".repeat(indent * depth);
    let attr_str = format_attrs(attrs);

    let has_children = !children.is_empty();
    let has_text = text.map_or(false, |t| !t.is_empty());

    if !has_children && !has_text {
        // Self-closing
        if attr_str.is_empty() {
            return format!("{}<{} />", prefix, tag);
        } else {
            return format!("{}<{}{} />", prefix, tag, attr_str);
        }
    }

    // Opening tag
    let open = if attr_str.is_empty() {
        format!("<{}>", tag)
    } else {
        format!("<{}{}>", tag, attr_str)
    };

    if !has_children && has_text {
        // Simple text content — keep on one line
        return format!("{}{}{}</{}>", prefix, open, escape_xml(text.unwrap_or("")), tag);
    }

    // Multi-line content
    let mut result = format!("{}{}\n", prefix, open);

    // Text content
    if let Some(t) = text {
        if !t.is_empty() {
            result.push_str(&format!(
                "{}{}\n",
                " ".repeat(indent * (depth + 1)),
                escape_xml(t)
            ));
        }
    }

    // Children
    for child in children.iter() {
        result.push_str(&value_to_xml(&child, tag, indent, depth + 1));
        result.push('\n');
    }

    // Closing tag
    result.push_str(&format!("{}</{}>", prefix, tag));

    result
}

fn format_attrs(attrs: &Obj) -> String {
    let entries: Vec<(String, Value)> = attrs.iter().map(|(k, v)| (k.to_string(), v.clone())).collect();
    if entries.is_empty() {
        return String::new();
    }

    let mut parts = Vec::new();
    for (k, v) in entries {
        parts.push(format!(" {}=\"{}\"", k, escape_xml(&v.as_str())));
    }
    parts.join("")
}

/// Escape text for XML content.
fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use auto_val::{Array, Obj};

    fn make_element(tag: &str, text: Option<&str>) -> Value {
        let mut obj = Obj::new();
        obj.set("tag", Value::str(tag));
        obj.set("attrs", Value::Obj(Obj::new()));
        obj.set("children", Value::Array(Array::new()));
        if let Some(t) = text {
            obj.set("text", Value::str(t));
        }
        Value::Obj(obj)
    }

    #[test]
    fn test_simple_element() {
        let val = make_element("root", Some("hello"));
        let xml = value_to_xml(&val, "root", 2, 0);
        assert_eq!(xml, "<root>hello</root>");
    }

    #[test]
    fn test_self_closing() {
        let val = make_element("br", None);
        let xml = value_to_xml(&val, "root", 2, 0);
        assert_eq!(xml, "<br />");
    }

    #[test]
    fn test_attributes() {
        let mut attrs = Obj::new();
        attrs.set("class", Value::str("main"));

        let mut obj = Obj::new();
        obj.set("tag", Value::str("div"));
        obj.set("attrs", Value::Obj(attrs));
        obj.set("children", Value::Array(Array::new()));
        obj.set("text", Value::str("content"));

        let xml = value_to_xml(&Value::Obj(obj), "root", 2, 0);
        assert!(xml.contains("class=\"main\""));
        assert!(xml.contains(">content</div>"));
    }

    #[test]
    fn test_nested() {
        let child = make_element("child", Some("inner"));
        let mut children = Array::new();
        children.push(child);

        let mut obj = Obj::new();
        obj.set("tag", Value::str("parent"));
        obj.set("attrs", Value::Obj(Obj::new()));
        obj.set("children", Value::Array(children));

        let xml = value_to_xml(&Value::Obj(obj), "root", 2, 0);
        assert!(xml.contains("<parent>"));
        assert!(xml.contains("  <child>inner</child>"));
        assert!(xml.contains("</parent>"));
    }

    #[test]
    fn test_xml_escaping() {
        assert_eq!(escape_xml("a < b & c"), "a &lt; b &amp; c");
        assert_eq!(escape_xml("\"quoted\""), "&quot;quoted&quot;");
    }

    #[test]
    fn test_plain_value_wrap() {
        let val = Value::str("hello");
        let xml = value_to_xml(&val, "root", 2, 0);
        assert_eq!(xml, "<root>hello</root>");
    }

    #[test]
    fn test_multiple_children() {
        let c1 = make_element("item", Some("a"));
        let c2 = make_element("item", Some("b"));
        let mut children = Array::new();
        children.push(c1);
        children.push(c2);

        let mut obj = Obj::new();
        obj.set("tag", Value::str("list"));
        obj.set("attrs", Value::Obj(Obj::new()));
        obj.set("children", Value::Array(children));

        let xml = value_to_xml(&Value::Obj(obj), "root", 2, 0);
        let lines: Vec<&str> = xml.lines().collect();
        assert!(lines[0].contains("<list>"));
        assert!(lines[1].contains("<item>a</item>"));
        assert!(lines[2].contains("<item>b</item>"));
        assert!(lines[3].contains("</list>"));
    }

    #[test]
    fn test_command_name() {
        let cmd = ToXmlCommand;
        assert_eq!(cmd.name(), "to_xml");
    }
}

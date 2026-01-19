use crate::error::AutoResult;
use auto_val::AutoStr;
use normalize_path::NormalizePath;
use roxmltree::{Document, NodeType};
use std::path::Path;
use std::path::PathBuf;

pub fn find_std_lib() -> AutoResult<AutoStr> {
    let mut search_dirs = Vec::new();

    // 1. Try project local stdlib (for development/testing)
    // Check if we're in a Cargo build (CARGO_MANIFEST_DIR is set)
    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        // From crates/auto-lang/, go up to project root (../../../)
        let project_root = PathBuf::from(manifest_dir)
            .join("../../../")
            .canonicalize();
        if let Ok(root) = project_root {
            let local_stdlib = root.join("stdlib");
            if let Some(path) = local_stdlib.to_str() {
                search_dirs.push(path.to_string());
            }
        }
    }

    // 2. Try user's local stdlib
    if let Some(home_dir) = dirs::home_dir() {
        let auto_std = home_dir.join(".auto/libs/");
        if let Some(path) = auto_std.to_str() {
            search_dirs.push(path.to_string());
        }
    }

    // 3. System-wide stdlib locations
    search_dirs.push("/usr/local/lib/auto".to_string());
    search_dirs.push("/usr/lib/auto".to_string());

    let std_lib_pat = "stdlib/auto";

    for dir in search_dirs {
        let std_path = PathBuf::from(dir).join(std_lib_pat);
        // println!("Checking {}", std_path.display()); // LSP: disabled
        if std_path.is_dir() {
            // println!("debug: std lib location: {}", std_path.to_str().unwrap()); // LSP: disabled
            return Ok(AutoStr::from(std_path.to_str().unwrap()));
        }
    }

    return Err("stdlib not found".into());
}

/// Get the file name from a path.
pub fn file_name(path: &str) -> &str {
    if let Some(pos) = path.rfind('/') {
        &path[pos + 1..]
    } else {
        if let Some(pos) = path.rfind('\\') {
            &path[pos + 1..]
        } else {
            path
        }
    }
}

pub trait PathExt {
    fn unified(&self) -> AutoStr;
}

impl PathExt for Path {
    fn unified(&self) -> AutoStr {
        let res = self.normalize().to_string_lossy().replace("\\", "/").into();
        if res == "" {
            ".".into()
        } else {
            res
        }
    }
}

/// Generate an error message with the file name and line number.
#[macro_export]
macro_rules! error_pos {
    ($msg: literal) => {
        Err(format!("{} at {}:{}", $msg, crate::util::file_name(file!()), line!()).into())
    };
    ($msg: literal, $($args:tt)*) => {
        Err(format!("{} at {}:{}", format!($msg, $($args)*), crate::util::file_name(file!()), line!()).into())
    };
}

/// Pretty print an s-expression with proper indentation.
///
/// Example:
/// ```text
/// (code
///   (stmt
///     (pair
///       (name name)
///       (str "hello")))
///   (stmt
///     (pair
///       (name version)
///       (str "0.1.0"))))
/// ```
pub fn pretty(text: &str) -> AutoStr {
    let mut result = String::new();
    let mut indent = 0;
    let mut in_str = false;

    for c in text.chars() {
        match c {
            '(' if !in_str => {
                if !result.is_empty() {
                    result.push('\n');
                    result.push_str(&"  ".repeat(indent));
                }
                result.push(c);
                indent += 1;
            }
            ')' if !in_str => {
                indent -= 1;
                result.push(c);
            }
            '"' => {
                in_str = !in_str;
                result.push(c);
            }
            ' ' if !in_str => {
                // result.push('\n');
                // result.push_str(&"  ".repeat(indent));
                result.push(c)
            }
            _ => result.push(c),
        }
    }
    result.into()
}

/// Compacts an XML string by parsing it and regenerating without extra whitespace
///
/// # Arguments
/// * `xml` - The XML string to compact
///
/// # Returns
/// A compacted version of the XML string, or an error message if parsing fails
pub fn compact_xml(xml: &str) -> Result<String, String> {
    // Parse the XML
    let doc = match Document::parse(xml) {
        Ok(doc) => doc,
        Err(e) => return Err(format!("XML parsing error: {}", e)),
    };

    let mut result = String::new();

    // Process the XML declaration if present
    // if let Some(decl) = doc.declaration() {
    //     result.push_str(&format!(
    //         "<?xml version=\"{}\" encoding=\"{}\"?>",
    //         decl.version().unwrap_or("1.0"),
    //         decl.encoding().unwrap_or("UTF-8")
    //     ));
    // }

    // Recursively process nodes
    fn process_node(node: roxmltree::Node, output: &mut String) {
        match node.node_type() {
            NodeType::Element => {
                // Open tag
                output.push('<');
                output.push_str(node.tag_name().name());

                // Attributes
                for attr in node.attributes() {
                    output.push(' ');
                    output.push_str(attr.name());
                    output.push_str("=\"");
                    output.push_str(&attr.value().replace("\"", "&quot;"));
                    output.push('"');
                }

                if !node.has_children() {
                    output.push_str("/>");
                } else {
                    output.push('>');

                    // Process children
                    for child in node.children() {
                        process_node(child, output);
                    }

                    // Close tag
                    output.push_str("</");
                    output.push_str(node.tag_name().name());
                    output.push('>');
                }
            }
            NodeType::Text => {
                // Compact whitespace in text, but preserve it
                let text = node.text().unwrap();
                let trimmed = text.trim();
                if !trimmed.is_empty() {
                    output.push_str(&text.split_whitespace().collect::<Vec<_>>().join(" "));
                }
            }
            NodeType::Comment => {
                // Preserve comments
                output.push_str("<!--");
                output.push_str(node.text().unwrap_or(""));
                output.push_str("-->");
            }
            _ => {}
        }
    }

    // Process all root-level nodes
    for child in doc.root().children() {
        process_node(child, &mut result);
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_name() {
        assert_eq!(file_name("src/util.rs"), "util.rs");
        assert_eq!(file_name("src/util.rs/"), "");
        assert_eq!(file_name("src\\util.rs"), "util.rs");
        assert_eq!(file_name("src\\util.rs\\"), "");
    }

    #[test]
    fn test_error() {
        let err: Result<(), String> = error_pos!("test error");
        let err_line = line!() - 1;
        assert_eq!(
            format!("{}", err.unwrap_err()),
            format!("test error at util.rs:{}", err_line)
        );

        let err: Result<(), String> = error_pos!("int error {}", "-1");
        let err_line = line!() - 1;
        assert_eq!(
            format!("{}", err.unwrap_err()),
            format!("int error -1 at util.rs:{}", err_line)
        );
    }

    #[test]
    fn test_pretty_print() {
        let text = "(code (stmt (pair (name name) (str \"hello\"))) (stmt (pair (name version) (str \"0.1.0\"))) (stmt (node (name exe) (args (str \"hello\")) (props (pair (name dir) (str \"src\")) (pair (name main) (str \"main.c\")))))";
        let pretty = pretty(text);
        println!("{}", pretty);
    }

    #[test]
    fn test_xml_compact() {
        let xml = r#"
            <root>
                <child attr="value">text</child>
            </root>
        "#;
        let compact = compact_xml(xml).unwrap();
        assert_eq!(compact, r#"<root><child attr="value">text</child></root>"#);
    }
}

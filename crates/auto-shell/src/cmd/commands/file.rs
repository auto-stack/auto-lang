//! file command - Determine file type
//!
//! Detects file type by extension and content inspection (text vs binary).

use crate::cmd::{Command, PipelineData, Signature};
use crate::cmd::parser::ParsedArgs;
use crate::shell::Shell;
use ash_core::pipeline::{Atom, AtomPipeline, AtomType};
use auto_val::{Value, Obj, Array};
use miette::Result;
use std::path::PathBuf;

pub struct FileCommand;

impl Command for FileCommand {
    fn name(&self) -> &str {
        "file"
    }

    fn signature(&self) -> Signature {
        Signature::new("file", "Determine file type")
            .required("path", "File(s) to inspect")
    }

    fn run(
        &self,
        args: &ParsedArgs,
        _input: PipelineData,
        shell: &mut Shell,
    ) -> Result<PipelineData> {
        if args.positionals.is_empty() {
            miette::bail!("file: missing file argument");
        }

        let mut results = Vec::new();

        for arg in &args.positionals {
            let path = if std::path::Path::new(arg).is_absolute() {
                PathBuf::from(arg)
            } else {
                shell.pwd().join(arg)
            };

            let (file_type, mime) = if !path.exists() {
                ("cannot open".to_string(), "application/x-error".to_string())
            } else if path.is_dir() {
                ("directory".to_string(), "inode/directory".to_string())
            } else if path.is_symlink() {
                ("symbolic link".to_string(), "inode/symlink".to_string())
            } else {
                detect_file_type(&path)
            };

            let mut obj = Obj::new();
            obj.set("path", Value::str(arg));
            obj.set("type", Value::str(&file_type));
            obj.set("mime", Value::str(&mime));
            results.push(Value::Obj(obj));
        }

        Ok(PipelineData::from_value(Value::Array(Array::from(results))))
    }

    fn run_atom(
        &self,
        args: &ParsedArgs,
        input: AtomPipeline,
        shell: &mut Shell,
    ) -> Result<AtomPipeline> {
        let legacy_in = crate::cmd::pipeline_convert::atom_to_pipeline_data(input);
        let legacy_out = self.run(args, legacy_in, shell)?;
        let value = match legacy_out {
            PipelineData::Value(v) => v,
            PipelineData::Text(s) => Value::str(&s),
        };
        Ok(AtomPipeline::from_atom(Atom::new(value, AtomType::Table)))
    }
}

/// Detect file type by reading content and examining extension.
fn detect_file_type(path: &std::path::Path) -> (String, String) {
    let ext = path.extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .unwrap_or_default();

    // Read a sample of the file to check text vs binary
    let content_sample = read_sample(path, 8192);
    let is_text = content_sample.as_ref().map(|s| is_text_data(s)).unwrap_or(true);

    // Determine type and MIME from extension
    match ext.as_str() {
        "rs" => ("Rust source".into(), "text/rust".into()),
        "c" | "h" => ("C source".into(), "text/c".into()),
        "cpp" | "cxx" | "cc" | "hpp" => ("C++ source".into(), "text/cpp".into()),
        "py" => ("Python source".into(), "text/python".into()),
        "js" | "mjs" => ("JavaScript source".into(), "text/javascript".into()),
        "ts" => ("TypeScript source".into(), "text/typescript".into()),
        "java" => ("Java source".into(), "text/java".into()),
        "html" | "htm" => ("HTML document".into(), "text/html".into()),
        "css" => ("CSS stylesheet".into(), "text/css".into()),
        "json" => ("JSON data".into(), "application/json".into()),
        "xml" => ("XML document".into(), "application/xml".into()),
        "yaml" | "yml" => ("YAML data".into(), "text/yaml".into()),
        "toml" => ("TOML data".into(), "text/toml".into()),
        "md" => ("Markdown document".into(), "text/markdown".into()),
        "txt" => ("plain text".into(), "text/plain".into()),
        "sh" | "bash" => ("shell script".into(), "text/x-shellscript".into()),
        "at" => ("AutoLang source".into(), "text/autolang".into()),
        "png" => ("PNG image".into(), "image/png".into()),
        "jpg" | "jpeg" => ("JPEG image".into(), "image/jpeg".into()),
        "gif" => ("GIF image".into(), "image/gif".into()),
        "svg" => ("SVG image".into(), "image/svg+xml".into()),
        "pdf" => ("PDF document".into(), "application/pdf".into()),
        "zip" => ("ZIP archive".into(), "application/zip".into()),
        "gz" | "tgz" => ("gzip archive".into(), "application/gzip".into()),
        "tar" => ("tar archive".into(), "application/x-tar".into()),
        _ => {
            if is_text {
                ("text file".into(), "text/plain".into())
            } else {
                ("binary file".into(), "application/octet-stream".into())
            }
        }
    }
}

/// Read the first N bytes of a file for content inspection.
fn read_sample(path: &std::path::Path, max_bytes: usize) -> Option<Vec<u8>> {
    use std::io::Read;
    let mut f = std::fs::File::open(path).ok()?;
    let mut buf = vec![0u8; max_bytes];
    let n = f.read(&mut buf).ok()?;
    buf.truncate(n);
    Some(buf)
}

/// Heuristic: determine if data appears to be text.
fn is_text_data(data: &[u8]) -> bool {
    // Check for null bytes (strong indicator of binary content)
    // and count control characters
    let text_chars = data.iter().filter(|&&b| {
        b == b'\n' || b == b'\r' || b == b'\t' || (b >= 0x20 && b < 0x7F) || b >= 0x80
    }).count();

    // If more than 85% of bytes look like text, treat as text
    let ratio = text_chars as f64 / data.len() as f64;
    ratio > 0.85
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_command_name() {
        let cmd = FileCommand;
        assert_eq!(cmd.name(), "file");
    }

    #[test]
    fn test_is_text_data_plain() {
        assert!(is_text_data(b"hello world\n"));
    }

    #[test]
    fn test_is_text_data_binary() {
        // Lots of null bytes
        let data = vec![0u8; 100];
        assert!(!is_text_data(&data));
    }

    #[test]
    fn test_is_text_data_mixed() {
        let data = b"some text with a few chars";
        assert!(is_text_data(data));
    }

    #[test]
    fn test_detect_by_ext() {
        let path = std::path::Path::new("test.rs");
        let (ft, mime) = detect_file_type(path);
        assert_eq!(ft, "Rust source");
        assert_eq!(mime, "text/rust");
    }
}

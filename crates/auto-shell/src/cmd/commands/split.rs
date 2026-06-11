use crate::cmd::{Command, PipelineData, Signature};
use crate::cmd::parser::ParsedArgs;
use crate::shell::Shell;
use ash_core::pipeline::{Atom, AtomPipeline, AtomType};
use auto_val::{Array, Obj, Value};
use miette::{IntoDiagnostic, Result};
use std::path::Path;

pub struct SplitCommand;

impl Command for SplitCommand {
    fn name(&self) -> &str {
        "split"
    }

    fn signature(&self) -> Signature {
        Signature::new("split", "Split text into chunks")
            .optional("file", "File to split (default: stdin)")
            .flag_with_short("lines", 'l', "Lines per chunk (default: 1000)")
            .flag_with_short("number", 'n', "Split into N chunks")
    }

    fn run(
        &self,
        args: &ParsedArgs,
        input: PipelineData,
        _shell: &mut Shell,
    ) -> Result<PipelineData> {
        let text = if let Some(path) = args.first() {
            std::fs::read_to_string(Path::new(path)).into_diagnostic()?
        } else {
            get_text(input)?
        };

        let lines_per_chunk = if args.has_flag("number") {
            // Split into N chunks - calculate lines per chunk
            let n: usize = args.positionals.iter()
                .find(|s| s.parse::<usize>().is_ok())
                .map(|s| s.parse::<usize>().unwrap())
                .unwrap_or(2);
            let total_lines = text.lines().count();
            if total_lines == 0 {
                1
            } else {
                (total_lines + n - 1) / n
            }
        } else if args.has_flag("lines") {
            args.positionals.iter()
                .find(|s| s.parse::<usize>().is_ok())
                .map(|s| s.parse::<usize>().unwrap())
                .unwrap_or(1000)
        } else {
            1000
        };

        let chunks = split_into_chunks(&text, lines_per_chunk);
        let arr = chunks_to_array(&chunks);
        Ok(PipelineData::from_value(Value::Array(arr)))
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

/// Extract text from PipelineData
fn get_text(input: PipelineData) -> Result<String> {
    match input {
        PipelineData::Text(s) => Ok(s),
        PipelineData::Value(Value::Str(s)) => Ok(s.to_string()),
        PipelineData::Value(Value::Array(arr)) => {
            let lines: Vec<String> = arr.iter().map(|v| v.as_str().to_string()).collect();
            Ok(lines.join("\n"))
        }
        _ => miette::bail!("split: input must be text"),
    }
}

/// Split text into chunks of N lines each
pub fn split_into_chunks(text: &str, lines_per_chunk: usize) -> Vec<Vec<&str>> {
    if lines_per_chunk == 0 {
        return vec![];
    }

    let lines: Vec<&str> = text.lines().collect();
    let mut chunks = Vec::new();

    for chunk_lines in lines.chunks(lines_per_chunk) {
        chunks.push(chunk_lines.to_vec());
    }

    chunks
}

/// Convert chunks to an Array of Objects with index, lines, content
fn chunks_to_array(chunks: &[Vec<&str>]) -> Array {
    let mut arr = Array::new();
    for (idx, chunk) in chunks.iter().enumerate() {
        let mut obj = Obj::new();
        obj.set("index", Value::Int(idx as i32));
        obj.set("lines", Value::Int(chunk.len() as i32));
        obj.set("content", Value::str(&chunk.join("\n")));
        arr.push(Value::Obj(obj));
    }
    arr
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_basic() {
        let text = "line1\nline2\nline3\nline4\nline5";
        let chunks = split_into_chunks(text, 2);
        assert_eq!(chunks.len(), 3);
        assert_eq!(chunks[0], vec!["line1", "line2"]);
        assert_eq!(chunks[1], vec!["line3", "line4"]);
        assert_eq!(chunks[2], vec!["line5"]);
    }

    #[test]
    fn test_split_exact() {
        let text = "a\nb\nc\nd";
        let chunks = split_into_chunks(text, 2);
        assert_eq!(chunks.len(), 2);
        assert_eq!(chunks[0], vec!["a", "b"]);
        assert_eq!(chunks[1], vec!["c", "d"]);
    }

    #[test]
    fn test_split_single_line() {
        let text = "only line";
        let chunks = split_into_chunks(text, 1000);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0], vec!["only line"]);
    }

    #[test]
    fn test_split_zero_lines() {
        let chunks = split_into_chunks("some text", 0);
        assert!(chunks.is_empty());
    }

    #[test]
    fn test_split_empty() {
        let chunks = split_into_chunks("", 10);
        assert!(chunks.is_empty());
    }

    #[test]
    fn test_chunks_to_array() {
        let chunks = vec![
            vec!["a", "b"],
            vec!["c"],
        ];
        let arr = chunks_to_array(&chunks);
        assert_eq!(arr.len(), 2);

        // Check first chunk
        if let Value::Obj(obj) = &arr.values[0] {
            assert_eq!(obj.get("index"), Some(Value::Int(0)));
            assert_eq!(obj.get("lines"), Some(Value::Int(2)));
        }
    }
}

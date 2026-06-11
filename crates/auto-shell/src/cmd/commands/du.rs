//! du command - Display disk usage
//!
//! Estimates file and directory space usage with optional human-readable
//! output and depth limiting.

use crate::cmd::{Command, PipelineData, Signature};
use crate::cmd::parser::ParsedArgs;
use crate::shell::Shell;
use ash_core::pipeline::{Atom, AtomPipeline, AtomType};
use auto_val::{Value, Obj, Array};
use miette::{IntoDiagnostic, Result};
use std::path::{Path, PathBuf};

pub struct DuCommand;

impl Command for DuCommand {
    fn name(&self) -> &str {
        "du"
    }

    fn signature(&self) -> Signature {
        Signature::new("du", "Display disk usage")
            .optional("path", "Path to measure (default: current directory)")
            .flag_with_short("summarize", 's', "Display only total")
            .flag_with_short("human-readable", 'h', "Print sizes in KB/MB/GB")
            .flag_with_short("depth", 'd', "Max display depth")
    }

    fn run(
        &self,
        args: &ParsedArgs,
        _input: PipelineData,
        shell: &mut Shell,
    ) -> Result<PipelineData> {
        let path_arg = args.first().unwrap_or(".");
        let summarize = args.has_flag("summarize");
        let human = args.has_flag("human-readable");

        let max_depth: Option<usize> = args.named.get("depth")
            .and_then(|s| s.parse().ok());

        let root = if Path::new(path_arg).is_absolute() {
            PathBuf::from(path_arg)
        } else {
            shell.pwd().join(path_arg)
        };

        if !root.exists() {
            miette::bail!("du: cannot access '{}': No such file or directory", path_arg);
        }

        let mut entries = Vec::new();
        compute_du(&root, &root, 0, summarize, max_depth, human, &mut entries)?;

        // Sort by size descending
        entries.sort_by(|a, b| {
            let sa = extract_size(a);
            let sb = extract_size(b);
            sb.cmp(&sa)
        });

        // Add total
        let total_bytes: u64 = entries.iter()
            .filter_map(|v| {
                if let Value::Obj(obj) = v {
                    if let Some(Value::Str(s)) = obj.get("bytes") {
                        s.parse::<u64>().ok()
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .sum();

        let mut total_obj = Obj::new();
        total_obj.set("path", Value::str("total"));
        total_obj.set("size", Value::str(&format_size(total_bytes, human)));
        total_obj.set("bytes", Value::Str(total_bytes.to_string().into()));
        entries.push(Value::Obj(total_obj));

        Ok(PipelineData::from_value(Value::Array(Array::from(entries))))
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

/// Extract the byte size from an entry Value.
fn extract_size(v: &Value) -> u64 {
    if let Value::Obj(obj) = v {
        if let Some(Value::Str(s)) = obj.get("bytes") {
            s.parse::<u64>().unwrap_or(0)
        } else {
            0
        }
    } else {
        0
    }
}

/// Recursively compute disk usage.
fn compute_du(
    root: &Path,
    current: &Path,
    depth: usize,
    summarize: bool,
    max_depth: Option<usize>,
    human: bool,
    entries: &mut Vec<Value>,
) -> Result<()> {
    if let Some(max) = max_depth {
        if depth > max {
            return Ok(());
        }
    }

    let mut total_bytes: u64 = 0;

    if current.is_dir() {
        let dir_entries = match std::fs::read_dir(current) {
            Ok(e) => e,
            Err(_) => return Ok(()),
        };

        for entry in dir_entries {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue,
            };
            let path = entry.path();

            let size = if path.is_dir() {
                let mut subdir_bytes: u64 = 0;
                compute_du_inner(&path, &mut subdir_bytes)?;
                subdir_bytes
            } else {
                entry.metadata().map(|m| m.len()).unwrap_or(0)
            };

            total_bytes += size;

            // Add subdirectory entry unless summarize mode
            if !summarize && path.is_dir() {
                let rel = pathdiff(root, &path);
                let mut obj = Obj::new();
                obj.set("path", Value::str(&rel));
                obj.set("size", Value::str(&format_size(size, human)));
                obj.set("bytes", Value::Str(size.to_string().into()));
                entries.push(Value::Obj(obj));
            }
        }
    } else {
        total_bytes = std::fs::metadata(current).map(|m| m.len()).unwrap_or(0);
    }

    // Add the current entry (root level)
    if depth == 0 {
        let rel = pathdiff(root, current);
        let mut obj = Obj::new();
        obj.set("path", Value::str(&rel));
        obj.set("size", Value::str(&format_size(total_bytes, human)));
        obj.set("bytes", Value::Str(total_bytes.to_string().into()));
        entries.push(Value::Obj(obj));
    }

    Ok(())
}

/// Inner helper: accumulate total bytes for a subtree.
fn compute_du_inner(path: &Path, total: &mut u64) -> Result<()> {
    if path.is_file() {
        *total += std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);
        return Ok(());
    }

    let entries = match std::fs::read_dir(path) {
        Ok(e) => e,
        Err(_) => return Ok(()),
    };

    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        let child = entry.path();
        if child.is_dir() {
            compute_du_inner(&child, total)?;
        } else {
            *total += entry.metadata().map(|m| m.len()).unwrap_or(0);
        }
    }

    Ok(())
}

/// Format bytes as human-readable size.
fn format_size(bytes: u64, human: bool) -> String {
    if !human {
        return bytes.to_string();
    }

    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.1}GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1}MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1}KB", bytes as f64 / KB as f64)
    } else {
        format!("{}B", bytes)
    }
}

/// Compute relative path string.
fn pathdiff(root: &Path, target: &Path) -> String {
    match target.strip_prefix(root) {
        Ok(rel) if rel.as_os_str().is_empty() => ".".to_string(),
        Ok(rel) => rel.to_string_lossy().to_string(),
        Err(_) => target.to_string_lossy().to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_du_command_name() {
        let cmd = DuCommand;
        assert_eq!(cmd.name(), "du");
    }

    #[test]
    fn test_format_size_plain() {
        assert_eq!(format_size(1024, false), "1024");
    }

    #[test]
    fn test_format_size_human() {
        assert_eq!(format_size(0, true), "0B");
        assert_eq!(format_size(512, true), "512B");
        assert!(format_size(1024, true).contains("KB"));
        assert!(format_size(1048576, true).contains("MB"));
        assert!(format_size(1073741824, true).contains("GB"));
    }
}

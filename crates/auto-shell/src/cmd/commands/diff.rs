use crate::cmd::{Command, PipelineData, Signature};
use crate::cmd::parser::ParsedArgs;
use crate::shell::Shell;
use ash_core::pipeline::{Atom, AtomPipeline, AtomType};
use auto_val::Value;
use miette::{IntoDiagnostic, Result};
use std::path::Path;

pub struct DiffCommand;

impl Command for DiffCommand {
    fn name(&self) -> &str {
        "diff"
    }

    fn signature(&self) -> Signature {
        Signature::new("diff", "Compare two files line by line")
            .required("file1", "First file to compare")
            .required("file2", "Second file to compare")
            .flag_with_short("unified", 'u', "Unified output format")
            .flag_with_short("brief", 'q', "Report only whether files differ")
    }

    fn run(
        &self,
        args: &ParsedArgs,
        _input: PipelineData,
        _shell: &mut Shell,
    ) -> Result<PipelineData> {
        let file1 = args.first()
            .ok_or_else(|| miette::miette!("diff: file1 argument required"))?;
        let file2 = args.second()
            .ok_or_else(|| miette::miette!("diff: file2 argument required"))?;

        let unified = args.has_flag("unified");
        let brief = args.has_flag("brief");

        let text1 = std::fs::read_to_string(Path::new(file1)).into_diagnostic()?;
        let text2 = std::fs::read_to_string(Path::new(file2)).into_diagnostic()?;

        let lines1: Vec<&str> = text1.lines().collect();
        let lines2: Vec<&str> = text2.lines().collect();

        let result = if brief {
            diff_brief(file1, file2, &lines1, &lines2)
        } else if unified {
            diff_unified(file1, file2, &lines1, &lines2)
        } else {
            diff_normal(file1, file2, &lines1, &lines2)
        };

        Ok(PipelineData::from_text(result))
    }

    fn run_atom(
        &self,
        args: &ParsedArgs,
        input: AtomPipeline,
        shell: &mut Shell,
    ) -> Result<AtomPipeline> {
        let legacy_in = crate::cmd::pipeline_convert::atom_to_pipeline_data(input);
        let legacy_out = self.run(args, legacy_in, shell)?;
        let text = legacy_out.into_text();
        Ok(AtomPipeline::from_atom(Atom::new(Value::str(&text), AtomType::Text)))
    }
}

/// Brief diff: only report if files differ
pub fn diff_brief(file1: &str, file2: &str, lines1: &[&str], lines2: &[&str]) -> String {
    if lines1 == lines2 {
        String::new()
    } else {
        format!("Files {} and {} differ", file1, file2)
    }
}

/// Normal diff: show added/deleted lines with + / - prefixes
pub fn diff_normal(file1: &str, file2: &str, lines1: &[&str], lines2: &[&str]) -> String {
    let ops = compute_diff_ops(lines1, lines2);
    let has_changes = ops.iter().any(|op| !matches!(op, DiffOp::Equal(_)));
    if !has_changes {
        return String::new();
    }

    let mut result = Vec::new();
    result.push(format!("--- {}", file1));
    result.push(format!("+++ {}", file2));

    for op in &ops {
        match op {
            DiffOp::Equal(line) => result.push(format!("  {}", line)),
            DiffOp::Delete(line) => result.push(format!("- {}", line)),
            DiffOp::Insert(line) => result.push(format!("+ {}", line)),
        }
    }

    result.join("\n")
}

/// Unified diff output
pub fn diff_unified(file1: &str, file2: &str, lines1: &[&str], lines2: &[&str]) -> String {
    let ops = compute_diff_ops(lines1, lines2);
    if ops.is_empty() {
        return String::new();
    }

    let mut result = Vec::new();
    result.push(format!("--- {}", file1));
    result.push(format!("+++ {}", file2));

    // Group into hunks
    let hunks = group_into_hunks(&ops);
    for hunk in &hunks {
        result.push(format!("@@ {} @@", hunk.range_str));
        for op in &hunk.ops {
            match op {
                DiffOp::Equal(line) => result.push(format!(" {}", line)),
                DiffOp::Delete(line) => result.push(format!("-{}", line)),
                DiffOp::Insert(line) => result.push(format!("+{}", line)),
            }
        }
    }

    result.join("\n")
}

/// Diff operation
#[derive(Debug, Clone, PartialEq)]
pub enum DiffOp {
    Equal(String),
    Delete(String),
    Insert(String),
}

/// A hunk of changes in unified diff
struct Hunk {
    range_str: String,
    ops: Vec<DiffOp>,
}

/// Group diff ops into hunks (groups of changes with 3 lines of context)
fn group_into_hunks(ops: &[DiffOp]) -> Vec<Hunk> {
    if ops.is_empty() {
        return Vec::new();
    }

    // Simple: put everything in one hunk
    let mut old_count = 0usize;
    let mut new_count = 0usize;
    for op in ops {
        match op {
            DiffOp::Equal(_) | DiffOp::Delete(_) => old_count += 1,
            DiffOp::Insert(_) => new_count += 1,
        }
    }
    for op in ops {
        if let DiffOp::Insert(_) = op {
            new_count += 0; // already counted
        }
    }

    let range_str = format!("-1,{} +1,{}", old_count, ops.len());
    vec![Hunk {
        range_str,
        ops: ops.to_vec(),
    }]
}

/// Compute diff operations using a simple LCS-based algorithm
pub fn compute_diff_ops(lines1: &[&str], lines2: &[&str]) -> Vec<DiffOp> {
    let m = lines1.len();
    let n = lines2.len();

    // Build LCS table
    let mut dp = vec![vec![0usize; n + 1]; m + 1];
    for i in 1..=m {
        for j in 1..=n {
            if lines1[i - 1] == lines2[j - 1] {
                dp[i][j] = dp[i - 1][j - 1] + 1;
            } else {
                dp[i][j] = dp[i - 1][j].max(dp[i][j - 1]);
            }
        }
    }

    // Backtrack to find diff
    let mut ops = Vec::new();
    let mut i = m;
    let mut j = n;

    while i > 0 || j > 0 {
        if i > 0 && j > 0 && lines1[i - 1] == lines2[j - 1] {
            ops.push(DiffOp::Equal(lines1[i - 1].to_string()));
            i -= 1;
            j -= 1;
        } else if j > 0 && (i == 0 || dp[i][j - 1] >= dp[i - 1][j]) {
            ops.push(DiffOp::Insert(lines2[j - 1].to_string()));
            j -= 1;
        } else {
            ops.push(DiffOp::Delete(lines1[i - 1].to_string()));
            i -= 1;
        }
    }

    ops.reverse();
    ops
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_diff_identical() {
        let ops = compute_diff_ops(&["a", "b", "c"], &["a", "b", "c"]);
        assert_eq!(ops, vec![
            DiffOp::Equal("a".into()),
            DiffOp::Equal("b".into()),
            DiffOp::Equal("c".into()),
        ]);
    }

    #[test]
    fn test_compute_diff_insert() {
        let ops = compute_diff_ops(&["a", "c"], &["a", "b", "c"]);
        assert_eq!(ops, vec![
            DiffOp::Equal("a".into()),
            DiffOp::Insert("b".into()),
            DiffOp::Equal("c".into()),
        ]);
    }

    #[test]
    fn test_compute_diff_delete() {
        let ops = compute_diff_ops(&["a", "b", "c"], &["a", "c"]);
        assert_eq!(ops, vec![
            DiffOp::Equal("a".into()),
            DiffOp::Delete("b".into()),
            DiffOp::Equal("c".into()),
        ]);
    }

    #[test]
    fn test_compute_diff_replace() {
        let ops = compute_diff_ops(&["a", "b", "c"], &["a", "x", "c"]);
        assert!(ops.contains(&DiffOp::Delete("b".into())));
        assert!(ops.contains(&DiffOp::Insert("x".into())));
    }

    #[test]
    fn test_diff_brief_same() {
        let result = diff_brief("f1", "f2", &["a", "b"], &["a", "b"]);
        assert!(result.is_empty());
    }

    #[test]
    fn test_diff_brief_different() {
        let result = diff_brief("f1", "f2", &["a"], &["b"]);
        assert_eq!(result, "Files f1 and f2 differ");
    }

    #[test]
    fn test_diff_normal() {
        let result = diff_normal("f1", "f2", &["a", "b"], &["a", "c"]);
        assert!(result.contains("- b"));
        assert!(result.contains("+ c"));
    }

    #[test]
    fn test_diff_normal_identical() {
        let result = diff_normal("f1", "f2", &["a", "b"], &["a", "b"]);
        assert!(result.is_empty());
    }
}

//! Pipeline and command chain parsing
//!
//! Handles parsing of:
//! - Pipe operator (|) for command chaining
//! - Logical AND (&&) and OR (||) for conditional execution

/// Operator connecting two command segments
#[derive(Debug, Clone, PartialEq)]
pub enum ChainOp {
    /// `|` — pipe stdout of left into stdin of right
    Pipe,
    /// `&&` — execute right only if left succeeded (exit code 0)
    And,
    /// `||` — execute right only if left failed (exit code != 0)
    Or,
}

/// A segment in a command chain: a command string + the operator that follows it.
///
/// The last segment always has `op: None`.
///
/// Example: `ls | grep foo && echo found`
/// → [Segment("ls", Pipe), Segment("grep foo", And), Segment("echo found", None)]
#[derive(Debug, Clone, PartialEq)]
pub struct ChainSegment {
    pub command: String,
    /// Operator connecting this segment to the *next* one; `None` for the last segment.
    pub op: Option<ChainOp>,
}

/// Parse a command line into chain segments (pipe, &&, ||).
///
/// Respects quotes and parentheses — operators inside `"..."` or `'...'` or `(...)` are ignored.
///
/// # Priority
///
/// `|` binds tighter than `&&` / `||`. So `a | b && c` is parsed as
/// `[a | b] && c`, which is correct bash behaviour.
pub fn parse_chain(input: &str) -> Vec<ChainSegment> {
    let mut segments = Vec::new();
    let mut current = String::new();
    let mut chars = input.chars().peekable();
    let mut in_single_quote = false;
    let mut in_double_quote = false;
    let mut paren_depth: i32 = 0;

    while let Some(c) = chars.next() {
        // Only look at operators when we're outside quotes and parens.
        let at_top_level = !in_single_quote && !in_double_quote && paren_depth == 0;

        match c {
            '\'' if !in_double_quote && paren_depth == 0 => {
                in_single_quote = !in_single_quote;
                current.push(c);
            }
            '"' if !in_single_quote && paren_depth == 0 => {
                in_double_quote = !in_double_quote;
                current.push(c);
            }
            '(' if !in_single_quote && !in_double_quote => {
                paren_depth += 1;
                current.push(c);
            }
            ')' if !in_single_quote && !in_double_quote => {
                paren_depth = paren_depth.saturating_sub(1);
                current.push(c);
            }
            '|' if at_top_level => {
                // Peek to see if this is `||` (logical OR) or `|` (pipe)
                if chars.peek() == Some(&'|') {
                    chars.next(); // consume second `|`
                    push_segment(&mut segments, &mut current, ChainOp::Or);
                } else {
                    push_segment(&mut segments, &mut current, ChainOp::Pipe);
                }
            }
            '&' if at_top_level => {
                // Peek to see if this is `&&` (logical AND) or lone `&` (background)
                if chars.peek() == Some(&'&') {
                    chars.next(); // consume second `&`
                    push_segment(&mut segments, &mut current, ChainOp::And);
                } else {
                    // Lone `&` — treat as part of the command (background operator
                    // is handled elsewhere in shell.rs)
                    current.push(c);
                }
            }
            _ => {
                current.push(c);
            }
        }
    }

    // Final segment (always has op: None)
    let cmd = current.trim().to_string();
    if !cmd.is_empty() {
        segments.push(ChainSegment { command: cmd, op: None });
    }

    // If nothing was parsed, return a single empty segment
    if segments.is_empty() {
        segments.push(ChainSegment {
            command: input.trim().to_string(),
            op: None,
        });
    }

    segments
}

/// Helper: push a completed segment with its operator, resetting `current`.
fn push_segment(segments: &mut Vec<ChainSegment>, current: &mut String, op: ChainOp) {
    let cmd = current.trim().to_string();
    if !cmd.is_empty() {
        segments.push(ChainSegment {
            command: cmd,
            op: Some(op),
        });
    }
    current.clear();
}

// ---------------------------------------------------------------------------
// Legacy API — kept for backward compatibility. Returns only the pipe-split
// command strings (ignores && / || semantics).
// ---------------------------------------------------------------------------

/// Parse a pipeline expression into individual commands (legacy).
///
/// Splits input by pipe operators (`|`), respecting quotes and parentheses.
/// `||` is treated as two separate pipes (use [`parse_chain`] for `||` support).
pub fn parse_pipeline(input: &str) -> Vec<String> {
    let result: Vec<String> = parse_chain(input)
        .into_iter()
        .filter_map(|seg| {
            if seg.command.is_empty() {
                None
            } else {
                Some(seg.command)
            }
        })
        .collect();

    // Match old behaviour: empty input → vec![""]
    if result.is_empty() {
        vec![input.trim().to_string()]
    } else {
        result
    }
}

// ---------------------------------------------------------------------------
// Helper for the executor: extract consecutive pipe-groups from a chain.
// ---------------------------------------------------------------------------

/// A "pipe group" is a sequence of segments connected by `Pipe`, terminated by
/// `And`, `Or`, or end-of-chain.
///
/// `parse_chain` returns flat segments. The executor needs pipe-groups so it
/// can execute a pipeline and then decide whether to continue based on `&&`/`||`.
///
/// Returns `Vec<(Vec<String>, Option<ChainOp>)>`:
/// each element is `(pipe_commands, next_operator)`.
pub fn group_pipe_segments(segments: Vec<ChainSegment>) -> Vec<(Vec<String>, Option<ChainOp>)> {
    let mut groups: Vec<(Vec<String>, Option<ChainOp>)> = Vec::new();
    let mut current_cmds: Vec<String> = Vec::new();

    for seg in segments {
        current_cmds.push(seg.command);
        match seg.op {
            Some(ChainOp::Pipe) => {
                // Continue accumulating pipe commands
            }
            Some(op @ ChainOp::And) | Some(op @ ChainOp::Or) => {
                // End of pipe group
                groups.push((std::mem::take(&mut current_cmds), Some(op)));
            }
            None => {
                // Last segment
                groups.push((std::mem::take(&mut current_cmds), None));
            }
        }
    }

    // Handle trailing case (shouldn't happen normally, but just in case)
    if !current_cmds.is_empty() {
        groups.push((current_cmds, None));
    }

    groups
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- Legacy parse_pipeline tests (unchanged behaviour) ----

    #[test]
    fn test_parse_single_command() {
        let pipeline = parse_pipeline("ls -la");
        assert_eq!(pipeline, vec!["ls -la"]);
    }

    #[test]
    fn test_parse_simple_pipeline() {
        let pipeline = parse_pipeline("ls | grep test");
        assert_eq!(pipeline, vec!["ls", "grep test"]);
    }

    #[test]
    fn test_parse_multiple_pipes() {
        let pipeline = parse_pipeline("ls | grep test | wc -l");
        assert_eq!(pipeline, vec!["ls", "grep test", "wc -l"]);
    }

    #[test]
    fn test_parse_pipeline_with_quotes() {
        let pipeline = parse_pipeline("echo \"hello | world\" | wc");
        assert_eq!(pipeline, vec!["echo \"hello | world\"", "wc"]);
    }

    #[test]
    fn test_parse_pipeline_with_single_quotes() {
        let pipeline = parse_pipeline("echo 'hello | world' | wc");
        assert_eq!(pipeline, vec!["echo 'hello | world'", "wc"]);
    }

    #[test]
    fn test_parse_pipeline_with_parens() {
        let pipeline = parse_pipeline("ls (echo | test) | grep foo");
        assert_eq!(pipeline, vec!["ls (echo | test)", "grep foo"]);
    }

    #[test]
    fn test_parse_empty_input() {
        let pipeline = parse_pipeline("");
        assert_eq!(pipeline, vec![""]);
    }

    #[test]
    fn test_parse_whitespace_only() {
        let pipeline = parse_pipeline("   ");
        assert_eq!(pipeline, vec![""]);
    }

    #[test]
    fn test_parse_pipe_at_start() {
        let pipeline = parse_pipeline("| grep test");
        assert_eq!(pipeline, vec!["grep test"]);
    }

    #[test]
    fn test_parse_pipe_at_end() {
        let pipeline = parse_pipeline("ls |");
        assert_eq!(pipeline, vec!["ls"]);
    }

    // ---- New parse_chain tests ----

    #[test]
    fn test_chain_and() {
        let segs = parse_chain("cargo build && cargo test");
        assert_eq!(segs.len(), 2);
        assert_eq!(segs[0].command, "cargo build");
        assert_eq!(segs[0].op, Some(ChainOp::And));
        assert_eq!(segs[1].command, "cargo test");
        assert_eq!(segs[1].op, None);
    }

    #[test]
    fn test_chain_or() {
        let segs = parse_chain("true || echo fallback");
        assert_eq!(segs.len(), 2);
        assert_eq!(segs[0].command, "true");
        assert_eq!(segs[0].op, Some(ChainOp::Or));
        assert_eq!(segs[1].command, "echo fallback");
        assert_eq!(segs[1].op, None);
    }

    #[test]
    fn test_chain_mixed_pipe_and() {
        // a | b && c → pipe group [a, b] then && c
        let segs = parse_chain("ls | grep foo && echo found");
        assert_eq!(segs.len(), 3);
        assert_eq!(&segs[0], &ChainSegment { command: "ls".into(), op: Some(ChainOp::Pipe) });
        assert_eq!(&segs[1], &ChainSegment { command: "grep foo".into(), op: Some(ChainOp::And) });
        assert_eq!(&segs[2], &ChainSegment { command: "echo found".into(), op: None });
    }

    #[test]
    fn test_chain_no_operators() {
        let segs = parse_chain("echo hello");
        assert_eq!(segs.len(), 1);
        assert_eq!(segs[0].command, "echo hello");
        assert_eq!(segs[0].op, None);
    }

    #[test]
    fn test_chain_and_in_quotes() {
        let segs = parse_chain("echo \"a && b\" && echo c");
        assert_eq!(segs.len(), 2);
        assert_eq!(segs[0].command, "echo \"a && b\"");
        assert_eq!(segs[0].op, Some(ChainOp::And));
        assert_eq!(segs[1].command, "echo c");
    }

    #[test]
    fn test_chain_background_ampersand() {
        // Lone & is NOT treated as an operator, it's part of the command
        let segs = parse_chain("sleep 1 &");
        assert_eq!(segs.len(), 1);
        assert_eq!(segs[0].command, "sleep 1 &");
        assert_eq!(segs[0].op, None);
    }

    #[test]
    fn test_chain_pipe_or_disambiguation() {
        // | vs || — ensure we correctly distinguish them
        let segs = parse_chain("a | b");
        assert_eq!(segs[0].op, Some(ChainOp::Pipe));

        let segs = parse_chain("a || b");
        assert_eq!(segs[0].op, Some(ChainOp::Or));
    }

    // ---- group_pipe_segments tests ----

    #[test]
    fn test_group_pipes_only() {
        let segs = parse_chain("ls | grep foo | wc -l");
        let groups = group_pipe_segments(segs);
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].0, vec!["ls", "grep foo", "wc -l"]);
        assert_eq!(groups[0].1, None);
    }

    #[test]
    fn test_group_mixed() {
        let segs = parse_chain("ls | grep foo && echo found || echo missing");
        let groups = group_pipe_segments(segs);
        assert_eq!(groups.len(), 3);
        assert_eq!(groups[0].0, vec!["ls", "grep foo"]);
        assert_eq!(groups[0].1, Some(ChainOp::And));
        assert_eq!(groups[1].0, vec!["echo found"]);
        assert_eq!(groups[1].1, Some(ChainOp::Or));
        assert_eq!(groups[2].0, vec!["echo missing"]);
        assert_eq!(groups[2].1, None);
    }
}

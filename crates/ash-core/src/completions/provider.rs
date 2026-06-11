//! Completion resolution engine
//!
//! Takes a `CompletionSpec` and a parsed command line, resolves the appropriate
//! completion candidates based on subcommand context, position, conditions, and
//! data sources.

use super::spec::*;
use super::{Completion, CompletionKind};
use std::collections::HashMap;
use std::path::Path;
use std::time::{Duration, Instant};

/// Context provided to the completion resolver at runtime.
/// Bridges the pure ash-core engine with auto-shell's state.
pub struct CompletionContext {
    /// Current working directory.
    pub current_dir: std::path::PathBuf,
    /// Function to execute an external command and capture its stdout.
    /// Injected because ash-core cannot call process::Command directly.
    pub command_executor: Box<dyn Fn(&str, &Path) -> Result<String, String>>,
}

/// The completion resolution engine.
pub struct CompletionProvider {
    /// Loaded completion specs, keyed by command name.
    specs: HashMap<String, CompletionSpec>,
    /// TTL cache for dynamic command sources.
    cache: HashMap<String, (Instant, Vec<String>)>,
    /// Cache TTL (default: 5 seconds).
    cache_ttl: Duration,
}

impl CompletionProvider {
    pub fn new() -> Self {
        Self {
            specs: HashMap::new(),
            cache: HashMap::new(),
            cache_ttl: Duration::from_secs(5),
        }
    }

    /// Register a completion spec for a command.
    pub fn register(&mut self, spec: CompletionSpec) {
        self.specs.insert(spec.command.clone(), spec);
    }

    /// Check if we have a spec for a command.
    pub fn has_spec(&self, command: &str) -> bool {
        self.specs.contains_key(command)
    }

    /// Main resolution method.
    ///
    /// - `parts`: tokenized command line (e.g., `["git", "checkout", "ma"]`)
    /// - `cursor_part`: index of the token the cursor is in
    /// - `prefix`: the text already typed in the current token
    /// - `ctx`: runtime context (cwd, command executor)
    pub fn resolve(
        &mut self,
        parts: &[&str],
        cursor_part: usize,
        prefix: &str,
        ctx: &CompletionContext,
    ) -> Vec<Completion> {
        if parts.is_empty() {
            return Vec::new();
        }

        let cmd_name = parts[0];
        let Some(spec) = self.specs.get(cmd_name) else {
            return Vec::new();
        };

        // Special case: cursor is on the command name itself
        if cursor_part == 0 {
            // Already typing the command, nothing to complete
            return Vec::new();
        }

        // Navigate subcommand chain
        let (node_flags, node_args, node_subcmds, _consumed) =
            navigate_subcommands(spec, parts, cursor_part);

        // Collect already-set flags from the entire line
        let present_flags = collect_flags_from_parts(parts);

        // Collect previous positional args (non-flag, non-subcommand tokens)
        let prev_positionals = collect_positionals(parts, &node_subcmds);

        // Determine what we're completing at the cursor
        let current_token = parts.get(cursor_part).copied().unwrap_or("");

        if current_token.starts_with('-') {
            // Flag completion
            return self.complete_flags_for_node(
                current_token, &node_flags, &present_flags,
            );
        }

        // Check if previous token was a flag that takes an argument
        if cursor_part > 0 {
            let prev = parts[cursor_part - 1];
            if let Some(flag_arg_name) = find_flag_arg(&node_flags, prev) {
                // The previous token is a flag expecting a value.
                // We could provide arg-specific completions here, but for now
                // fall through to file completion.
                let _ = flag_arg_name;
            }
        }

        // Try subcommand completion (if cursor is right after the command/subcommand)
        let subcommand_depth = count_subcommand_tokens(parts, spec);
        let positional_index = calculate_positional_index(parts, cursor_part, subcommand_depth);

        if positional_index == 0 && !node_subcmds.is_empty() && !current_token.starts_with('-') {
            let mut subcmd_completions = Vec::new();
            for sub in &node_subcmds {
                if sub.name.starts_with(prefix) || prefix.is_empty() {
                    subcmd_completions.push(Completion::with_description(
                        sub.name.clone(),
                        sub.name.clone(),
                        sub.desc.clone().unwrap_or_default(),
                        CompletionKind::Subcommand,
                    ));
                }
            }
            if !subcmd_completions.is_empty() {
                return subcmd_completions;
            }
        }

        // Try arg completion
        for arg in &node_args {
            let position_matches = arg.position == positional_index
                || (arg.position == ARG_ANY_POSITION && arg.repeat)
                || (arg.repeat && arg.position <= positional_index);
            if !position_matches {
                continue;
            }

            // Evaluate when condition
            if let Some(ref when) = arg.when {
                if !when.evaluate(&present_flags, &prev_positionals) {
                    continue;
                }
            }

            // Resolve source
            if let Some(ref source) = arg.source {
                let candidates = self.resolve_source(source, prefix, ctx);
                if !candidates.is_empty() {
                    return candidates;
                }
            }
        }

        // Fallback: if no arg matched, try file completion
        Vec::new() // Caller will fall back to file completion
    }

    fn complete_flags_for_node(
        &mut self,
        prefix: &str,
        flags: &[FlagSpec],
        already_set: &[String],
    ) -> Vec<Completion> {
        let mut completions = Vec::new();
        let is_long = prefix.starts_with("--");
        let is_short = !is_long && prefix.starts_with('-');

        for flag in flags {
            if is_long {
                if let Some(ref long) = flag.long {
                    let long_flag = format!("--{}", long);
                    if long_flag.starts_with(prefix)
                        && !already_set.contains(&long_flag)
                        && !already_set.contains(long)
                    {
                        completions.push(Completion::with_description(
                            long_flag.clone(),
                            long_flag,
                            flag.desc.as_deref().unwrap_or(""),
                            CompletionKind::Flag,
                        ));
                    }
                }
            } else if is_short {
                if let Some(ref short) = flag.short {
                    let short_flag = format!("-{}", short);
                    if short_flag.starts_with(prefix)
                        && !already_set.contains(&short_flag)
                        && !already_set.contains(short)
                    {
                        completions.push(Completion::with_description(
                            short_flag.clone(),
                            short_flag,
                            flag.desc.as_deref().unwrap_or(""),
                            CompletionKind::Flag,
                        ));
                    }
                }
                // Also show --long when user typed just `-`
                if prefix == "-" {
                    if let Some(ref long) = flag.long {
                        let long_flag = format!("--{}", long);
                        if !already_set.contains(&long_flag)
                            && !already_set.contains(long)
                        {
                            completions.push(Completion::with_description(
                                long_flag.clone(),
                                long_flag,
                                flag.desc.as_deref().unwrap_or(""),
                                CompletionKind::Flag,
                            ));
                        }
                    }
                }
            }
        }

        completions
    }

    fn resolve_source(
        &mut self,
        source: &CompletionSource,
        prefix: &str,
        ctx: &CompletionContext,
    ) -> Vec<Completion> {
        match source {
            CompletionSource::Static(items) => {
                items
                    .iter()
                    .filter(|item| item.starts_with(prefix) || prefix.is_empty())
                    .map(|item| {
                        Completion::with_kind(
                            item.clone(),
                            item.clone(),
                            CompletionKind::Subcommand,
                        )
                    })
                    .collect()
            }
            CompletionSource::Command { cmd, parse } => {
                let candidates = self.execute_cached(cmd, ctx);
                self.parse_and_filter(&candidates, parse, prefix)
            }
            CompletionSource::Files { .. } | CompletionSource::Directories => {
                // Handled by caller fallback
                Vec::new()
            }
            CompletionSource::Variables => Vec::new(),
        }
    }

    fn execute_cached(&mut self, cmd: &str, ctx: &CompletionContext) -> Vec<String> {
        // Check cache
        if let Some((instant, cached)) = self.cache.get(cmd) {
            if instant.elapsed() < self.cache_ttl {
                return cached.clone();
            }
        }

        // Execute
        let result = (ctx.command_executor)(cmd, &ctx.current_dir);
        let lines = match result {
            Ok(output) => output.lines().map(|l| l.to_string()).collect(),
            Err(_) => Vec::new(),
        };

        // Cache
        self.cache.insert(cmd.to_string(), (Instant::now(), lines.clone()));
        lines
    }

    fn parse_and_filter(
        &self,
        lines: &[String],
        parse: &ParseMode,
        prefix: &str,
    ) -> Vec<Completion> {
        let mut completions = Vec::new();

        for line in lines {
            let candidate = match parse {
                ParseMode::Line => {
                    // Strip leading whitespace and * marker
                    let trimmed = line.trim_start();
                    let trimmed = trimmed.strip_prefix("* ").unwrap_or(trimmed);
                    trimmed.trim().to_string()
                }
                ParseMode::Field(n) => {
                    let fields: Vec<&str> = line.split_whitespace().collect();
                    fields.get(*n).map(|s| s.to_string()).unwrap_or_default()
                }
            };

            if candidate.is_empty() {
                continue;
            }

            if candidate.starts_with(prefix) || prefix.is_empty() {
                completions.push(Completion::with_kind(
                    candidate.clone(),
                    candidate,
                    CompletionKind::Subcommand,
                ));
            }
        }

        completions
    }
}

// ── Helper functions ─────────────────────────────────────

/// Navigate subcommand chain to find the current node.
/// Returns (flags, args, subcommands, consumed_count).
fn navigate_subcommands<'a>(
    spec: &'a CompletionSpec,
    parts: &[&str],
    _cursor_part: usize,
) -> (Vec<FlagSpec>, Vec<ArgSpec>, Vec<SubcommandSpec>, usize) {
    // Start with root spec's flags, args, subcommands
    let mut flags = spec.flags.clone();
    let mut args = spec.args.clone();
    let mut subcmds = &spec.subcommands;
    let mut consumed = 0;

    // Walk parts[1..] matching subcommands
    for &part in &parts[1..] {
        if part.starts_with('-') {
            continue; // skip flags
        }
        if let Some(sub) = subcmds.iter().find(|s| s.name == part) {
            flags = sub.flags.clone();
            args = sub.args.clone();
            subcmds = &sub.subcommands;
            consumed += 1;
        } else {
            break;
        }
    }

    // Also include root-level flags that apply globally
    let mut all_flags = spec.flags.clone();
    all_flags.extend(flags);

    (all_flags, args, subcmds.clone(), consumed)
}

fn collect_flags_from_parts(parts: &[&str]) -> Vec<String> {
    let mut flags = Vec::new();
    for &part in &parts[1..] {
        if part.starts_with("--") {
            let name = part.trim_start_matches('-').split('=').next().unwrap_or("");
            if !name.is_empty() {
                flags.push(name.to_string());
            }
        } else if part.starts_with('-') && part.len() > 1 {
            for ch in part[1..].chars() {
                flags.push(ch.to_string());
            }
        }
    }
    flags
}

fn collect_positionals(parts: &[&str], _subcmds: &[SubcommandSpec]) -> Vec<String> {
    let mut positionals = Vec::new();
    for &part in &parts[1..] {
        if !part.starts_with('-') {
            positionals.push(part.to_string());
        }
    }
    positionals
}

fn count_subcommand_tokens(parts: &[&str], spec: &CompletionSpec) -> usize {
    let mut count = 0;
    let mut subcmds = &spec.subcommands;
    for &part in &parts[1..] {
        if part.starts_with('-') {
            continue;
        }
        if let Some(sub) = subcmds.iter().find(|s| s.name == part) {
            count += 1;
            subcmds = &sub.subcommands;
        } else {
            break;
        }
    }
    count
}

fn calculate_positional_index(parts: &[&str], cursor_part: usize, subcommand_depth: usize) -> usize {
    // Count non-flag tokens between the subcommand chain and cursor
    let start = 1 + subcommand_depth; // skip "git" + subcommands
    let mut positional_count = 0;
    for i in start..cursor_part {
        if let Some(&part) = parts.get(i) {
            if !part.starts_with('-') {
                positional_count += 1;
            }
        }
    }
    positional_count
}

fn find_flag_arg(flags: &[FlagSpec], token: &str) -> Option<String> {
    for flag in flags {
        if flag.arg.is_some() {
            if let Some(ref short) = flag.short {
                if token == format!("-{}", short) {
                    return flag.arg.clone();
                }
            }
            if let Some(ref long) = flag.long {
                if token == format!("--{}", long) {
                    return flag.arg.clone();
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn git_spec() -> CompletionSpec {
        CompletionSpec::new("git")
            .desc("Git version control")
            .subcommand(
                SubcommandSpec::new("checkout")
                    .desc("Switch branches")
                    .flag(FlagSpec::both("b", "branch").desc("Create new branch").takes_arg("name"))
                    .arg(
                        ArgSpec::new(0)
                            .desc("Branch to checkout")
                            .when(WhenCondition::flags_absent(&["b"]))
                            .source(CompletionSource::command("git branch --list")),
                    ),
            )
            .subcommand(
                SubcommandSpec::new("push")
                    .desc("Push to remote")
                    .flag(FlagSpec::both("f", "force").desc("Force push"))
                    .flag(FlagSpec::both("u", "set-upstream").desc("Set upstream"))
                    .arg(
                        ArgSpec::new(0)
                            .desc("Remote")
                            .source(CompletionSource::command("git remote")),
                    )
                    .arg(
                        ArgSpec::new(1)
                            .desc("Branch")
                            .source(CompletionSource::command("git branch --list")),
                    ),
            )
            .subcommand(
                SubcommandSpec::new("add")
                    .desc("Stage files")
                    .flag(FlagSpec::both("A", "all").desc("Stage all"))
                    .arg(
                        ArgSpec::any()
                            .repeat()
                            .desc("Files to stage")
                            .source(CompletionSource::command_field("git status --porcelain", 1)),
                    ),
            )
    }

    fn mock_ctx(output: &str) -> CompletionContext {
        let output = output.to_string();
        CompletionContext {
            current_dir: PathBuf::from("/tmp"),
            command_executor: Box::new(move |_cmd: &str, _dir: &Path| Ok(output.clone())),
        }
    }

    #[test]
    fn test_complete_subcommands() {
        let spec = git_spec();
        let mut provider = CompletionProvider::new();
        provider.register(spec);

        let ctx = mock_ctx("");
        let completions = provider.resolve(&["git", ""], 1, "", &ctx);

        let names: Vec<&str> = completions.iter().map(|c| c.replacement.as_str()).collect();
        assert!(names.contains(&"checkout"));
        assert!(names.contains(&"push"));
        assert!(names.contains(&"add"));
    }

    #[test]
    fn test_complete_flags() {
        let spec = git_spec();
        let mut provider = CompletionProvider::new();
        provider.register(spec);

        let ctx = mock_ctx("");
        let completions = provider.resolve(&["git", "push", "--"], 2, "--", &ctx);

        let values: Vec<&str> = completions.iter().map(|c| c.replacement.as_str()).collect();
        assert!(values.contains(&"--force"));
        assert!(values.contains(&"--set-upstream"));
    }

    #[test]
    fn test_complete_short_flags() {
        let spec = git_spec();
        let mut provider = CompletionProvider::new();
        provider.register(spec);

        let ctx = mock_ctx("");
        let completions = provider.resolve(&["git", "push", "-"], 2, "-", &ctx);

        let values: Vec<&str> = completions.iter().map(|c| c.replacement.as_str()).collect();
        assert!(values.contains(&"-f"));
        assert!(values.contains(&"-u"));
    }

    #[test]
    fn test_resolve_command_source() {
        let spec = git_spec();
        let mut provider = CompletionProvider::new();
        provider.register(spec);

        let ctx = mock_ctx("origin\nupstream\n");
        let completions = provider.resolve(&["git", "push", ""], 2, "", &ctx);

        let values: Vec<&str> = completions.iter().map(|c| c.replacement.as_str()).collect();
        assert!(values.contains(&"origin"));
        assert!(values.contains(&"upstream"));
    }

    #[test]
    fn test_when_condition_flags_absent() {
        let spec = git_spec();
        let mut provider = CompletionProvider::new();
        provider.register(spec);

        // Without -b: should complete branches
        let ctx = mock_ctx("* main\n  develop\n");
        let completions = provider.resolve(&["git", "checkout", ""], 2, "", &ctx);
        let values: Vec<&str> = completions.iter().map(|c| c.replacement.as_str()).collect();
        assert!(values.contains(&"main"));
        assert!(values.contains(&"develop"));

        // With -b: should NOT complete branches (condition flags_absent fails)
        let completions_with_b = provider.resolve(&["git", "checkout", "-b", ""], 3, "", &ctx);
        // No branch candidates because when condition fails
        assert!(completions_with_b.is_empty());
    }

    #[test]
    fn test_filter_by_prefix() {
        let spec = git_spec();
        let mut provider = CompletionProvider::new();
        provider.register(spec);

        let ctx = mock_ctx("origin\nupstream\n");
        let completions = provider.resolve(&["git", "push", "ori"], 2, "ori", &ctx);

        assert_eq!(completions.len(), 1);
        assert_eq!(completions[0].replacement, "origin");
    }

    #[test]
    fn test_parse_mode_line_strips_star() {
        let spec = git_spec();
        let mut provider = CompletionProvider::new();
        provider.register(spec);

        let ctx = mock_ctx("* main\n  develop\n  feature-x\n");
        let completions = provider.resolve(&["git", "checkout", ""], 2, "", &ctx);

        let values: Vec<&str> = completions.iter().map(|c| c.replacement.as_str()).collect();
        assert!(values.contains(&"main"));
        assert!(values.contains(&"develop"));
        assert!(values.contains(&"feature-x"));
    }
}

use miette::{IntoDiagnostic, Result};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Stdio;

use ash_core::parser::{
    parse_chain, parse_env_prefixes, group_pipe_segments, parse_redirect, ChainOp, Redirect,
    StderrRedirect,
};
use auto_lang::autovm_persistent::AutovmReplSession;
use auto_val::Value;
use ash_core::pipeline::AtomPipeline;

// Re-export vars from core
pub use crate::core::shell::vars;

use crate::bookmarks::BookmarkManager;
use crate::cmd::{commands, CommandRegistry};
use crate::job::JobManager;
use vars::ShellVars;

/// Help text for the `completions` builtin (Plan 315).
const COMPLETIONS_HELP: &str = "\
completions — manage three-tier completion specs (Plan 315)

USAGE:
  completions generate <cmd> [--refresh]   Probe `<cmd> --help` → write generated/<cmd>.at
  completions list                         List specs in user/generated/cache tiers
  completions clear <cmd>                  Delete the cache entry for <cmd>
  completions clear --cache                Delete the entire cache tier
  completions path                         Print the three tier directory paths

TIERS (highest priority first):
  user       ~/.config/ash/completions/<cmd>.at        (hand-written)
  generated  ~/.config/ash/completions/generated/...   (this command)
  cache      ~/.config/ash/completions/cache/...       (auto-probe on first Tab)

Format: Auto/Atom (.at). `generate` parses the command's --help output.
";

/// HSV to RGB conversion (h: 0-360°, s/v: 0.0-1.0) for the rainbow demo.
fn hsv_to_rgb(h: f64, s: f64, v: f64) -> (u8, u8, u8) {
    let c = v * s;
    let h6 = (h / 60.0) % 6.0;
    let x = c * (1.0 - (h6 % 2.0 - 1.0).abs());
    let m = v - c;
    let (r1, g1, b1) = match h6 as i32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    (
        ((r1 + m) * 255.0).round().clamp(0.0, 255.0) as u8,
        ((g1 + m) * 255.0).round().clamp(0.0, 255.0) as u8,
        ((b1 + m) * 255.0).round().clamp(0.0, 255.0) as u8,
    )
}

/// Shell state and context
pub struct Shell {
    current_dir: PathBuf,
    vars: ShellVars,
    session: AutovmReplSession, // Persistent AutoVM REPL session
    bookmarks: BookmarkManager,

    previous_dir: Option<PathBuf>,
    registry: CommandRegistry,
    last_exit_code: i32, // $? — exit code of the last command
    jobs: JobManager,    // Background/suspended job control
    aliases: HashMap<String, String>, // Plan 302 Step 1.4: alias map
    dir_stack: Vec<PathBuf>,           // Plan 302 Step 3.5: directory stack

    // Plan 304: Abbreviations (abbr) — like aliases but expanded in-line visually
    abbreviations: HashMap<String, String>,

    // Plan 304: Event hooks — function names to call on events
    hooks: HashMap<String, String>,     // event name → Auto function name

    // Plan 304: Special variables
    last_command_line: Option<String>,   // $_ / !! — last executed command line
    last_command_args: Vec<String>,      // $@ — args of last command
    last_command_last_arg: Option<String>, // !$ — last arg of last command
    temp_files_for_cleanup: Vec<std::path::PathBuf>, // process substitution temp files
    /// `ls` icon column style (from `~/.config/ash.at`). Plan 309 / ls UX.
    ls_icons: crate::config::IconStyle,
}

impl Shell {
    /// Create a new shell instance
    pub fn new() -> Self {
        // Create persistent session (AutoVM based)
        let session = AutovmReplSession::new();

        let registry = {
            let mut reg = CommandRegistry::new();
            reg.register(Box::new(commands::build::BuildCommand));
            reg.register(Box::new(commands::run::RunCommand));
            reg.register(Box::new(commands::ls::LsCommand));
            reg.register(Box::new(commands::cd::CdCommand));
            reg.register(Box::new(commands::pwd::PwdCommand));
            reg.register(Box::new(commands::echo::EchoCommand));
            reg.register(Box::new(commands::help::HelpCommand));
            reg.register(Box::new(commands::get::GetCommand));
            reg.register(Box::new(commands::r#where::WhereCommand));
            reg.register(Box::new(commands::select::SelectCommand));
            reg.register(Box::new(commands::wc::WcCommand));
            reg.register(Box::new(commands::grep::GrepCommand));
            reg.register(Box::new(commands::ps::PsCommand));
            reg.register(Box::new(commands::sys::SysCommand));
            reg.register(Box::new(commands::cp::CpCommand));
            reg.register(Box::new(commands::mv::MvCommand));
            reg.register(Box::new(commands::rm::RmCommand));
            reg.register(Box::new(commands::mkdir::MkdirCommand));
            // Batch 1: File operations
            reg.register(Box::new(commands::cat::CatCommand));
            reg.register(Box::new(commands::head::HeadCommand));
            reg.register(Box::new(commands::tail::TailCommand));
            reg.register(Box::new(commands::touch::TouchCommand));
            reg.register(Box::new(commands::find::FindCommand));
            reg.register(Box::new(commands::glob::GlobCommand));
            reg.register(Box::new(commands::stat::StatCommand));
            reg.register(Box::new(commands::du::DuCommand));
            reg.register(Box::new(commands::file::FileCommand));
            reg.register(Box::new(commands::tee::TeeCommand));
            reg.register(Box::new(commands::ln::LnCommand));
            // Batch 2: Text processing
            reg.register(Box::new(commands::sort::SortCommand));
            reg.register(Box::new(commands::uniq::UniqCommand));
            reg.register(Box::new(commands::cut::CutCommand));
            reg.register(Box::new(commands::paste::PasteCommand));
            reg.register(Box::new(commands::tr::TrCommand));
            reg.register(Box::new(commands::split::SplitCommand));
            reg.register(Box::new(commands::rev::RevCommand));
            reg.register(Box::new(commands::column::ColumnCommand));
            reg.register(Box::new(commands::fmt::FmtCommand));
            reg.register(Box::new(commands::diff::DiffCommand));
            // Batch 3: Data format conversion
            reg.register(Box::new(commands::from_json::FromJsonCommand));
            reg.register(Box::new(commands::to_json::ToJsonCommand));
            reg.register(Box::new(commands::from_csv::FromCsvCommand));
            reg.register(Box::new(commands::to_csv::ToCsvCommand));
            reg.register(Box::new(commands::from_toml::FromTomlCommand));
            reg.register(Box::new(commands::to_toml::ToTomlCommand));
            reg.register(Box::new(commands::from_yaml::FromYamlCommand));
            reg.register(Box::new(commands::to_yaml::ToYamlCommand));
            reg.register(Box::new(commands::from_xml::FromXmlCommand));
            reg.register(Box::new(commands::to_xml::ToXmlCommand));
            // Batch 4: String, math, data transformation
            reg.register(Box::new(commands::str_replace::StrReplaceCommand));
            reg.register(Box::new(commands::str_contains::StrContainsCommand));
            reg.register(Box::new(commands::str_split::StrSplitCommand));
            reg.register(Box::new(commands::str_join::StrJoinCommand));
            reg.register(Box::new(commands::str_trim::StrTrimCommand));
            reg.register(Box::new(commands::str_case::StrCaseCommand));
            reg.register(Box::new(commands::str_length::StrLengthCommand));
            reg.register(Box::new(commands::math_sum::MathSumCommand));
            reg.register(Box::new(commands::math_avg::MathAvgCommand));
            reg.register(Box::new(commands::math_min::MathMinCommand));
            reg.register(Box::new(commands::math_max::MathMaxCommand));
            reg.register(Box::new(commands::math_round::MathRoundCommand));
            reg.register(Box::new(commands::update::UpdateCommand));
            reg.register(Box::new(commands::insert::InsertCommand));
            reg.register(Box::new(commands::each::EachCommand));
            // Batch 5: HTTP, datetime, utilities
            reg.register(Box::new(commands::http_get::HttpGetCommand));
            reg.register(Box::new(commands::http_post::HttpPostCommand));
            reg.register(Box::new(commands::http_put::HttpPutCommand));
            reg.register(Box::new(commands::http_delete::HttpDeleteCommand));
            reg.register(Box::new(commands::http_head::HttpHeadCommand));
            reg.register(Box::new(commands::url_encode::UrlEncodeCommand));
            reg.register(Box::new(commands::date::DateCommand));
            reg.register(Box::new(commands::sleep::SleepCommand));
            reg.register(Box::new(commands::which::WhichCommand));
            reg.register(Box::new(commands::version::VersionCommand));
            reg
        };

        Self {
            current_dir: std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/")),
            vars: ShellVars::new(),
            session,
            bookmarks: BookmarkManager::new(),

            previous_dir: None,
            registry,
            last_exit_code: 0,
            jobs: JobManager::new(),
            aliases: HashMap::new(),
            dir_stack: Vec::new(),
            abbreviations: HashMap::new(),
            hooks: HashMap::new(),

            last_command_line: None,
            last_command_args: Vec::new(),
            last_command_last_arg: None,
            temp_files_for_cleanup: Vec::new(),
            ls_icons: crate::config::AshShellConfig::load().ls_icons,
        }
    }

    /// Execute a command or AutoLang expression.
    ///
    /// After execution, `$?` is updated with the exit code
    /// (0 for success, non-zero for failure).
    pub fn execute(&mut self, input: &str) -> Result<Option<String>> {
        self.last_exit_code = 0; // reset; execute_inner may override

        // Cleanup temp files from previous process substitution
        for path in self.temp_files_for_cleanup.drain(..) {
            let _ = std::fs::remove_file(path);
        }

        let _guard = crate::signal::CtrlCGuard::new();

        // Track special variables before execution
        let trimmed = input.trim();
        if !trimmed.is_empty() {
            self.last_command_line = Some(trimmed.to_string());
            // Parse args for $@, $#, !$ tracking
            let parts = crate::core::cmd::external::parse_command(trimmed);
            if !parts.is_empty() {
                self.last_command_args = parts[1..].to_vec();
                self.last_command_last_arg = parts.last().cloned();
            }
        }

        let result = self.execute_with_env_prefixes(input);
        if result.is_err() && self.last_exit_code == 0 {
            self.last_exit_code = 1;
        }
        // Fire precmd hook (after command execution, before returning)
        self.fire_hook("precmd", &[&self.last_exit_code.to_string()]);
        result
    }

    /// Plan 301 §3.4 / Plan 309 Task 1.2 — Phase 3 (execution side).
    ///
    /// Strips leading `KEY=VALUE` env prefixes from the command, then:
    /// - **Assignment-only** (`FOO=bar` with no command): sets the vars
    ///   persistently in the current shell (bash semantics).
    /// - **Prefix + command** (`FOO=bar cmd`): sets the vars in a temporary
    ///   scope, runs the command, then restores — so the change does not leak
    ///   past this command. The scope wraps `execute_inner` at its single call
    ///   site, so `pop_scope` runs on every path (ok or error).
    fn execute_with_env_prefixes(&mut self, input: &str) -> Result<Option<String>> {
        let (env_pairs, rest) = parse_env_prefixes(input.trim());

        // Assignment-only: persist, no command to run.
        if rest.is_empty() {
            for (k, v) in &env_pairs {
                self.vars.set_env(k.clone(), v.clone());
            }
            return Ok(None);
        }

        // Prefix + command: apply in a scope around execution.
        let scoped = !env_pairs.is_empty();
        if scoped {
            self.vars.push_scope();
            for (k, v) in &env_pairs {
                self.vars.set_env_scoped(k.clone(), v.clone());
            }
        }
        let result = self.execute_inner(&rest);
        if scoped {
            self.vars.pop_scope();
        }
        result
    }

    /// Get the exit code of the last executed command.
    pub fn last_exit_code(&self) -> i32 {
        self.last_exit_code
    }

    /// Internal: actual command dispatch.
    fn execute_inner(&mut self, input: &str) -> Result<Option<String>> {
        // Fire preexec hook (before command execution)
        self.fire_hook("preexec", &[input.trim()]);

        // Reap any finished background jobs and notify
        self.reap_jobs();

        // Plan 304: Process substitution — expand <(cmd) into temp file paths
        let processed = self.expand_process_substitution(input.trim());
        let trimmed = processed.trim();

        // Check for background execution suffix: `cmd &`
        if trimmed.ends_with('&') {
            // Strip the trailing `&` and any whitespace before it
            let cmd_part = trimmed.trim_end_matches('&').trim();
            if !cmd_part.is_empty() {
                return self.execute_background(cmd_part);
            }
        }

        // Handle job control builtins
        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if !parts.is_empty() {
            match parts[0] {
                "jobs" => return self.cmd_jobs(),
                "fg" => return self.cmd_fg(parts.get(1).and_then(|s| s.parse::<u32>().ok())),
                "bg" => return self.cmd_bg(parts.get(1).and_then(|s| s.parse::<u32>().ok())),
                "suspend" => return self.cmd_suspend(),
                "alias" => return self.cmd_alias(trimmed),
                "unalias" => return self.cmd_unalias(&parts),
                "source" | "." => return self.cmd_source(&parts),
                "pushd" => return self.cmd_pushd(&parts),
                "popd" => return self.cmd_popd(&parts),
                "dirs" => return self.cmd_dirs(&parts),
                "path" => return self.cmd_path(&parts),
                "env" | "env.path" => return self.cmd_env(&parts),
                "completions" => return self.cmd_completions(&parts),
                "color" => return self.cmd_color(&parts),
                "def" => return self.cmd_def(trimmed),
                "hook" => return self.cmd_hook(&parts),
                "abbr" => return self.cmd_abbr(&parts),
                "config" => return self.cmd_config(&parts),
                "bind" => return self.cmd_bind(&parts),
                _ => {}
            }
        }

        // Try to parse as AutoLang expression first
        if self.looks_like_auto_expr(input) {
            return self.execute_auto(input);
        }

        // Plan 302 Step 2.4: Expand command substitution ($() and backticks)
        let expanded = self.expand_command_substitution(input)?;
        // Expand variables in input
        let expanded = self.expand_variables(&expanded);

        // Plan 302 Step 1.4: Expand aliases (only first word of each segment)
        let expanded = self.expand_aliases(&expanded);

        // Plan 302 Step 2.2: Expand ~ and ~/path (outside quotes)
        let expanded = self.expand_tilde(&expanded);

        // Parse command chain (|, &&, ||) and group pipe segments
        let segments = parse_chain(&expanded);

        // Check if there are any && or || operators
        let has_logic_ops = segments.iter().any(|s| matches!(s.op, Some(ChainOp::And | ChainOp::Or)));

        if has_logic_ops {
            // Execute as a chain with short-circuit evaluation
            let groups = group_pipe_segments(segments);
            return self.execute_chain(&groups);
        }

        // Simple pipeline (no &&/||)
        let commands: Vec<String> = segments.into_iter().map(|s| s.command).collect();
        if commands.len() > 1 {
            return self.execute_pipeline_with_auto(&commands);
        }

        // Create cmd parts
        let parts: Vec<&str> = expanded.split_whitespace().collect();

        if !parts.is_empty() && (parts[0] == "set" || parts[0] == "export" || parts[0] == "unset") {
            // Handle variable management commands
            self.execute_var_command(&parts)
        } else if !parts.is_empty() && parts[0] == "use" {
            // Handle module import
            if parts.len() < 2 {
                miette::bail!("use: missing module name");
            }
            self.import_module(parts[1])
        } else if !parts.is_empty() && (parts[0] == "up" || parts[0] == "u") {
            // Handle up (u) command
            self.execute_up_command(&parts)
        } else if !parts.is_empty() && parts[0] == "b" {
            // Handle b (bookmark) command
            self.execute_bookmark_command(&parts)
        } else {
            // Otherwise, execute as single shell command
            self.execute_single_command(&expanded)
        }
    }

    /// Execute a single command (built-in, Auto function, or external)
    ///
    /// Handles I/O redirection (`>`, `>>`, `<`, `2>`) by stripping redirect
    /// operators from the command, executing the core command, then applying
    /// file I/O as needed.
    fn execute_single_command(&mut self, input: &str) -> Result<Option<String>> {
        use crate::cmd::{auto, builtin, external};
        use crate::parser::quote::parse_args;

        // Plan 302 Step 1.1: Parse and strip redirections
        let (clean_input, redirect) = parse_redirect(input);

        let mut parts = parse_args(&clean_input);
        if parts.is_empty() {
            return Ok(None);
        }

        // Plan 302 Step 2.1: Expand globs in arguments (skip command name)
        if parts.len() > 1 {
            parts = self.expand_globs(&parts);
        }

        let cmd_name = &parts[0];
        let args = &parts[1..];

        // If there are redirects AND it's an external command, use redirect-aware execution
        if let Some(ref redir) = redirect {
            // For registry/builtin/auto commands: execute normally, then write output to file
            if let Some(cmd) = self.registry.get(cmd_name) {
                let signature = cmd.signature();
                match crate::cmd::parser::parse_args(&signature, args) {
                    Ok(parsed_args) => {
                        // Auto --help generation
                        if parsed_args.help_requested {
                            return Ok(Some(signature.format_help()));
                        }
                        let atom_out = cmd.run_atom(&parsed_args, AtomPipeline::empty(), self)?;
                        let output = self.format_output(atom_out);
                        self.apply_output_redirect(&output, redir)?;
                        return Ok(None); // output went to file
                    }
                    Err(e) => return Err(e),
                }
            }

            if let Some(output) = builtin::execute_builtin(&clean_input, &self.current_dir)? {
                self.apply_output_redirect(&output, redir)?;
                return Ok(None);
            }

            if self.has_auto_function(cmd_name) {
                let result = auto::execute_auto_function(self, cmd_name, args, None)?;
                if let Some(ref output) = result {
                    self.apply_output_redirect(output, redir)?;
                }
                return Ok(None);
            }

            // External command with redirects
            return self.execute_external_with_redirect(&clean_input, redir);
        }

        // No redirects — normal execution path
        // Check registry first
        if let Some(cmd) = self.registry.get(cmd_name) {
            let signature = cmd.signature();
            match crate::cmd::parser::parse_args(&signature, args) {
                Ok(parsed_args) => {
                    // Auto --help generation
                    if parsed_args.help_requested {
                        return Ok(Some(signature.format_help()));
                    }
                    let atom_out = cmd.run_atom(&parsed_args, AtomPipeline::empty(), self)?;
                    return Ok(Some(self.format_output(atom_out)));
                }
                Err(e) => return Err(e),
            }
        }

        // Check for built-in commands first
        if let Some(output) = builtin::execute_builtin(input, &self.current_dir)? {
            return Ok(Some(output));
        }

        // Check if it's an Auto function
        if self.has_auto_function(cmd_name) {
            return auto::execute_auto_function(self, cmd_name, args, None);
        }

        // Plan 304: Auto-load from ~/.config/ash/functions/<name>.at (Fish-style)
        if let Some(loaded) = self.try_autoload(cmd_name)? {
            // Function was loaded, now execute it
            if self.has_auto_function(cmd_name) {
                return auto::execute_auto_function(self, cmd_name, args, None);
            }
            // The file was sourced but didn't define a matching function
            if let Some(output) = loaded {
                return Ok(Some(output));
            }
        }

        // Otherwise, execute as external command
        let result = external::execute_external(input, &self.current_dir, false);
        if let Err(ref e) = result {
            self.last_exit_code = extract_exit_code(&e.to_string());
            // Plan 304: "did you mean?" suggestion on failure
            if let Some(suggestion) = suggest_command(self, cmd_name) {
                eprintln!("  did you mean: {}?", suggestion);
            }
        }
        result
    }

    /// Write command output to a redirect target file.
    fn apply_output_redirect(&self, output: &str, redirect: &Redirect) -> Result<()> {
        use std::fs::File;
        use std::io::Write;

        if let Some(ref path) = redirect.stdout {
            let file = if redirect.append_stdout {
                std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(path)
                    .into_diagnostic()?
            } else {
                File::create(path).into_diagnostic()?
            };
            let mut writer = std::io::BufWriter::new(file);
            writer.write_all(output.as_bytes()).into_diagnostic()?;
            writer.write_all(b"\n").into_diagnostic()?;
        }

        Ok(())
    }

    /// Execute an external command with I/O redirection via std::process::Command.
    fn execute_external_with_redirect(
        &mut self,
        clean_input: &str,
        redirect: &Redirect,
    ) -> Result<Option<String>> {
        use std::fs::File;
        use std::process::Command;

        let parts = crate::parser::quote::parse_args(clean_input);
        if parts.is_empty() {
            return Ok(None);
        }

        let cmd_name = &parts[0];
        let args = &parts[1..];

        let mut cmd = Command::new(cmd_name);
        cmd.args(args).current_dir(&self.current_dir);

        // stdin redirect: < file
        if let Some(ref path) = redirect.stdin {
            let file = File::open(path).into_diagnostic()?;
            cmd.stdin(Stdio::from(file));
        }

        // stdout redirect: > file or >> file
        if let Some(ref path) = redirect.stdout {
            let file = if redirect.append_stdout {
                std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(path)
                    .into_diagnostic()?
            } else {
                File::create(path).into_diagnostic()?
            };
            cmd.stdout(Stdio::from(file));
        } else {
            cmd.stdout(Stdio::inherit());
        }

        // stderr redirect: 2> file, 2>> file, or 2>&1
        match &redirect.stderr {
            Some(StderrRedirect::File(path)) => {
                let file = File::create(path).into_diagnostic()?;
                cmd.stderr(Stdio::from(file));
            }
            Some(StderrRedirect::Append(path)) => {
                let file = std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(path)
                    .into_diagnostic()?;
                cmd.stderr(Stdio::from(file));
            }
            Some(StderrRedirect::ToStdout) => {
                // If stdout is also redirected, stderr goes to the same file
                cmd.stderr(Stdio::inherit());
            }
            None => {
                cmd.stderr(Stdio::inherit());
            }
        }

        let status = cmd.status().into_diagnostic()?;

        if status.success() {
            Ok(None) // output went to file
        } else {
            self.last_exit_code = status.code().unwrap_or(1);
            Err(miette::miette!(
                "Command failed with exit code: {}",
                status.code().unwrap_or(-1)
            ))
        }
    }

    /// Format an AtomPipeline for terminal display.
    ///
    /// Structured data (file lists, etc.) gets rendered as a ratatui table
    /// with borders. Everything else falls back to plain text.
    fn format_output(&self, pipeline: AtomPipeline) -> String {
        // Try ratatui table rendering for structured Atom data
        if let AtomPipeline::Atom(ref atom) = pipeline {
            if atom.is_structured() {
                let term_width = crossterm::terminal::size()
                    .map(|(w, _)| w)
                    .unwrap_or(80);
                if let Some(rendered) = crate::frontend::renderer::render_table_with(
                    &atom.value,
                    term_width,
                    self.ls_icons,
                ) {
                    return rendered;
                }
            }
        }

        // Fallback: plain text
        pipeline.into_text()
    }

    /// Execute a command chain with short-circuit `&&` / `||` evaluation.
    ///
    /// Each element is a `(pipe_commands, next_operator)` pair:
    /// - `pipe_commands` are executed as a pipeline (connected by `|`)
    /// - `next_operator` determines whether the *next* group runs based on exit code
    fn execute_chain(
        &mut self,
        groups: &[(Vec<String>, Option<ChainOp>)],
    ) -> Result<Option<String>> {
        let mut last_success = true;
        let mut final_output: Option<String> = None;

        for (pipe_cmds, next_op) in groups {
            match next_op {
                Some(ChainOp::And) if !last_success => {
                    // && but previous failed → skip this group
                    continue;
                }
                Some(ChainOp::Or) if last_success => {
                    // || but previous succeeded → skip this group
                    continue;
                }
                _ => {
                    // Execute this pipe group
                    if pipe_cmds.len() == 1 {
                        final_output = self.execute_single_command(&pipe_cmds[0])?;
                    } else {
                        final_output = self.execute_pipeline_with_auto(pipe_cmds)?;
                    }
                    last_success = self.last_exit_code == 0;
                }
            }
        }

        Ok(final_output)
    }

    /// Execute a pipeline with Auto function support
    fn execute_pipeline_with_auto(&mut self, commands: &[String]) -> Result<Option<String>> {
        use crate::cmd::{auto, builtin, external};

        use crate::parser::quote::parse_args;

        if commands.is_empty() {
            return Ok(None);
        }

        // Start with empty AtomPipeline
        let mut input_pipeline: Option<AtomPipeline> = None;

        for (i, cmd) in commands.iter().enumerate() {
            let is_last = i == commands.len() - 1;

            // Parse command into parts
            let parts = parse_args(cmd);
            if parts.is_empty() {
                continue;
            }

            // Plan 320: structured-pipeline DSL stage (filter/sort/select/...)?
            if let Some(op) = ash_core::parser::pipe_stages::parse_pipe_stage(cmd) {
                let input_val = match input_pipeline.take() {
                    Some(ash_core::pipeline::AtomPipeline::Atom(atom)) => atom.value,
                    _ => auto_val::Value::Array(auto_val::Array::new()),
                };
                let result_val = ash_core::pipeline::operators::apply(&op, &input_val);
                input_pipeline = Some(ash_core::pipeline::AtomPipeline::from_atom(
                    ash_core::pipeline::Atom::new(result_val, ash_core::pipeline::AtomType::Table),
                ));
                if is_last {
                    return Ok(input_pipeline.map(|p| self.format_output(p)));
                }
                continue;
            }

            let cmd_name = &parts[0];
            let args = &parts[1..];

            // ── Phase 1: Registered command (uses AtomPipeline via run_atom) ──
            if let Some(registered_cmd) = self.registry.get(cmd_name) {
                let signature = registered_cmd.signature();
                let input = input_pipeline.take().unwrap_or_else(AtomPipeline::empty);

                match crate::cmd::parser::parse_args(&signature, args) {
                    Ok(parsed_args) => {
                        // Auto --help generation (prints help, doesn't pipeline)
                        if parsed_args.help_requested {
                            return Ok(Some(signature.format_help()));
                        }
                        let output = registered_cmd.run_atom(&parsed_args, input, self)?;
                        input_pipeline = Some(output);
                    }
                    Err(e) => return Err(e),
                }
                if is_last {
                    return Ok(input_pipeline.map(|p| self.format_output(p)));
                }
                continue;
            }

            // ── Phase 2: OS pipe chaining (external → external) ──
            //
            // If the previous command produced an ExternalStream AND the current
            // command is neither a legacy builtin nor an Auto function, we can
            // connect them with a real OS pipe — no in-memory buffering needed.
            let prev_is_stream = input_pipeline
                .as_ref()
                .map_or(false, |p| p.is_external_stream());

            if prev_is_stream
                && !builtin::is_legacy_builtin(cmd_name)
                && !self.has_auto_function(cmd_name)
            {
                let stream = input_pipeline
                    .take()
                    .and_then(|p| p.into_external_stream())
                    .expect("just checked is_external_stream");
                let raw_stdout = stream.into_raw_stdout();
                let new_stream =
                    external::spawn_external_chained(cmd, &self.current_dir, raw_stdout)?;
                input_pipeline = Some(AtomPipeline::ExternalStream(new_stream));
                if is_last {
                    return Ok(input_pipeline.map(|p| self.format_output(p)));
                }
                continue;
            }

            // ── Phase 3: Text-based path (builtins, auto functions, first external) ──
            let input_str = input_pipeline.take().and_then(|p| {
                if p.is_empty() {
                    None
                } else {
                    Some(p.into_text())
                }
            });

            let output_pipeline = if let Some(input) = &input_str {
                // With pipeline input
                if let Some(output) =
                    builtin::execute_builtin_with_input(cmd, &self.current_dir, Some(input))?
                {
                    Some(AtomPipeline::text(output))
                } else if self.has_auto_function(cmd_name) {
                    let output =
                        auto::execute_auto_function(self, cmd_name, args, Some(input))?;
                    output.map(|s| AtomPipeline::text(s))
                } else {
                    // Spawn external command, piping upstream output to stdin
                    let stream = external::spawn_external_stream_with_input(
                        cmd,
                        &self.current_dir,
                        input,
                    )?;
                    Some(AtomPipeline::ExternalStream(stream))
                }
            } else {
                // No pipeline input
                if let Some(output) = builtin::execute_builtin(cmd, &self.current_dir)? {
                    Some(AtomPipeline::text(output))
                } else if self.has_auto_function(cmd_name) {
                    let output = auto::execute_auto_function(self, cmd_name, args, None)?;
                    output.map(|s| AtomPipeline::text(s))
                } else {
                    // Spawn external command with streaming output
                    let stream = external::spawn_external_stream(cmd, &self.current_dir)?;
                    Some(AtomPipeline::ExternalStream(stream))
                }
            };

            // Store AtomPipeline for next command
            input_pipeline = output_pipeline;

            // If this is the last command, return the final output as text
            if is_last {
                return Ok(input_pipeline.map(|p| self.format_output(p)));
            }
        }

        Ok(None)
    }

    /// Get the current working directory
    pub fn pwd(&self) -> PathBuf {
        self.current_dir.clone()
    }

    /// Change the current directory
    pub fn cd(&mut self, path: &str) -> Result<()> {
        let new_dir = if path == "-" {
            // Handle cd - (swap to previous dir)
            if let Some(prev) = &self.previous_dir {
                println!("{}", prev.display());
                prev.clone()
            } else {
                miette::bail!("cd: oldpwd not set");
            }
        } else if path.starts_with('/') {
            PathBuf::from(path)
        } else if path.starts_with('~') {
            // Expand ~ to home directory
            dirs::home_dir().unwrap_or_else(|| PathBuf::from("~"))
        } else {
            self.current_dir.join(path)
        };

        // Try to canonicalize the path
        let canonical = new_dir.canonicalize().into_diagnostic()?;

        if canonical.is_dir() {
            // Update internal state
            self.previous_dir = Some(self.current_dir.clone());
            let old_dir = self.current_dir.clone();
            self.current_dir = canonical.clone();
            // Update OS state (so Prompt and child processes see it)
            std::env::set_current_dir(&canonical).into_diagnostic()?;
            // Notify git cache: sync refresh + start filesystem watcher
            crate::prompt::context::on_directory_changed(canonical.clone());
            // Fire chdir hook
            self.fire_hook("chdir", &[
                &old_dir.to_string_lossy(),
                &canonical.to_string_lossy(),
            ]);
            Ok(())
        } else {
            miette::bail!("cd: {}: Not a directory", path);
        }
    }

    /// Get the command registry
    pub fn registry(&self) -> &CommandRegistry {
        &self.registry
    }

    /// Check if input looks like an AutoLang expression
    fn looks_like_auto_expr(&self, input: &str) -> bool {
        // Simple heuristic: if it starts with common Auto keywords/operators
        let trimmed = input.trim();

        if trimmed.is_empty() {
            return false;
        }

        let first_char = trimmed.chars().next().unwrap();

        // Auto expressions: numbers, strings with quotes, fn, let, mut, const, use
        trimmed.starts_with("fn ")
            || trimmed.starts_with("let ")
            || trimmed.starts_with("mut ")
            || trimmed.starts_with("const ")
            || trimmed.starts_with("use ")
            || first_char == '"'
            || first_char == 'f'
            || first_char == '['
            || first_char == '{'
            || first_char.is_ascii_digit()
            || first_char == '-'
            || first_char == '+'
            || first_char == '('
            || self.is_function_call(trimmed)
    }

    /// Check if input looks like a function call to an Auto function
    fn is_function_call(&self, input: &str) -> bool {
        // Check if it matches pattern: name(...)
        if let Some(paren_pos) = input.find('(') {
            if input.ends_with(')') {
                let func_name = &input[..paren_pos];
                // Check if this is a registered Auto function
                return self.has_auto_function(func_name);
            }
        }
        false
    }

    /// Execute an AutoLang expression using persistent interpreter
    fn execute_auto(&mut self, input: &str) -> Result<Option<String>> {
        match self.session.run(input) {
            Ok(_) => {
                let result = self.session.format_last_result().unwrap_or_default();
                Ok(Some(result))
            }
            Err(e) => Err(miette::miette!("{}", e)),
        }
    }

    /// Expand variables in input string
    /// Supports $name and ${name} syntax
    fn expand_variables(&self, input: &str) -> String {
        let mut result = String::new();
        let mut chars = input.chars().peekable();
        let mut in_var = false;
        let mut in_braced_var = false;
        let mut var_name = String::new();

        while let Some(c) = chars.next() {
            if c == '$' {
                if in_var {
                    // We were in a variable, now we see another $
                    // Finish the previous variable first
                    if let Some(value) = self.get_variable(&var_name) {
                        result.push_str(&value);
                    }
                    var_name.clear();
                }

                if let Some(&'{') = chars.peek() {
                    // Start of ${name} syntax
                    chars.next(); // consume '{'
                    in_braced_var = true;
                    in_var = false;
                    var_name.clear();
                } else if matches!(chars.peek(), Some('?' | '@' | '#' | '!')) {
                    // POSIX special parameters that are single non-alphanumeric
                    // chars (routed to get_variable, which already maps them).
                    // NOTE: `_` is intentionally excluded — it's a valid name char,
                    // so special-casing it would break names like `$_foo`.
                    let sc = chars.next().unwrap();
                    if let Some(value) = self.get_variable(&sc.to_string()) {
                        result.push_str(&value);
                    }
                } else {
                    // Start of $name syntax
                    in_var = true;
                    in_braced_var = false;
                    var_name.clear();
                }
            } else if in_braced_var {
                if c == '}' {
                    // End of ${name}
                    in_braced_var = false;
                    if let Some(value) = self.get_variable(&var_name) {
                        result.push_str(&value);
                    }
                    var_name.clear();
                } else {
                    var_name.push(c);
                }
            } else if in_var {
                // $name syntax - variable name ends at non-alphanumeric chars
                if !c.is_alphanumeric() && c != '_' {
                    in_var = false;
                    if let Some(value) = self.get_variable(&var_name) {
                        result.push_str(&value);
                    }
                    var_name.clear();
                    result.push(c);
                } else {
                    var_name.push(c);
                }
            } else {
                result.push(c);
            }
        }

        // Handle variable at end of string
        if in_var {
            if let Some(value) = self.get_variable(&var_name) {
                result.push_str(&value);
            }
        }

        result
    }

    // -----------------------------------------------------------------------
    // Plan 302 Step 2.4: Command substitution ($() and backticks)
    // -----------------------------------------------------------------------

    /// Expand command substitution: `$(cmd)` and `` `cmd` ``.
    ///
    /// - `echo "dir: $(pwd)"` → `echo "dir: /home/user"`
    /// - `` echo `whoami` `` → `echo user`
    /// - Supports nesting: `echo $(basename $(pwd))`
    /// - Trailing newlines stripped (bash behavior)
    /// - `$()` inside single quotes is NOT expanded
    fn expand_command_substitution(&mut self, input: &str) -> Result<String> {
        // First pass: convert backticks to $() syntax
        let input = Self::convert_backticks(input);

        let mut result = String::new();
        let mut chars = input.chars().peekable();
        let mut in_single_quote = false;

        while let Some(c) = chars.next() {
            if c == '\'' {
                in_single_quote = !in_single_quote;
                result.push(c);
                continue;
            }

            // Inside single quotes: no expansion
            if in_single_quote {
                result.push(c);
                continue;
            }

            // Outside single quotes: check for $(
            if c == '$' {
                if chars.peek() == Some(&'(') {
                    chars.next(); // consume (
                    let mut depth = 1;
                    let mut cmd = String::new();

                    loop {
                        match chars.next() {
                            None => {
                                // Unmatched $( — keep as-is
                                result.push_str("$(");
                                result.push_str(&cmd);
                                break;
                            }
                            Some('$') if chars.peek() == Some(&'(') => {
                                chars.next(); // consume (
                                depth += 1;
                                cmd.push_str("$(");
                            }
                            Some(')') => {
                                depth -= 1;
                                if depth > 0 {
                                    cmd.push(')');
                                } else {
                                    // Matching ) found — execute the command.
                                    // self.execute() → execute_inner() will recursively
                                    // expand any nested $() within `cmd`.
                                    let output = self.execute(&cmd)?;
                                    let trimmed: String = output
                                        .unwrap_or_default()
                                        .trim_end_matches('\n')
                                        .trim_end_matches('\r')
                                        .to_string();
                                    result.push_str(&trimmed);
                                    break;
                                }
                            }
                            Some(other) => {
                                cmd.push(other);
                            }
                        }
                    }
                } else {
                    result.push('$');
                }
            } else {
                result.push(c);
            }
        }

        Ok(result)
    }

    /// Convert backtick command substitution to `$()` syntax.
    ///
    /// `` `whoami` `` → `$(whoami)`, respects single quotes (no conversion inside).
    fn convert_backticks(input: &str) -> String {
        let mut result = String::with_capacity(input.len());
        let mut in_single_quote = false;
        let mut in_backtick = false;

        for c in input.chars() {
            if c == '\'' && !in_backtick {
                in_single_quote = !in_single_quote;
                result.push(c);
            } else if c == '`' && !in_single_quote {
                if in_backtick {
                    result.push(')');
                } else {
                    result.push_str("$(");
                }
                in_backtick = !in_backtick;
            } else {
                result.push(c);
            }
        }

        result
    }

    // -----------------------------------------------------------------------
    // Plan 302 Step 1.4: Alias system
    // -----------------------------------------------------------------------

    /// Expand aliases in the input. Only the first word of each pipe-segment
    /// is checked against the alias table. Supports multi-segment chains
    /// (e.g. `ll && ga` where both `ll` and `ga` are aliases).
    /// Expand process substitution `<(...)` into temp file paths.
    ///
    /// `diff <(sort file1) <(sort file2)` → `diff /tmp/ash_ps_0 /tmp/ash_ps_1`
    ///
    /// Each `<(...)` spawns the command, captures its stdout to a temp file,
    /// and replaces the expression with the file path.
    fn expand_process_substitution(&mut self, input: &str) -> String {
        let mut result = String::new();
        let mut chars = input.char_indices().peekable();
        let mut temp_files: Vec<std::path::PathBuf> = Vec::new();

        while let Some((i, c)) = chars.next() {
            // Detect <( pattern
            if c == '<' && chars.peek().map(|(_, c)| *c) == Some('(') {
                // Skip the '('
                chars.next();
                // Find matching closing )
                let mut depth = 1;
                let start = i + 2; // after <(
                let mut end = start;
                while let Some((j, ch)) = chars.next() {
                    if ch == '(' {
                        depth += 1;
                    } else if ch == ')' {
                        depth -= 1;
                        if depth == 0 {
                            end = j;
                            break;
                        }
                    }
                }
                if depth != 0 {
                    // Unmatched — leave as-is
                    result.push_str(&input[i..]);
                    break;
                }
                let cmd = &input[start..end];
                // Execute the command and capture output to temp file
                if let Ok(tmp_path) = self.capture_to_temp(cmd) {
                    result.push_str(&tmp_path.to_string_lossy());
                    temp_files.push(tmp_path);
                } else {
                    // On failure, leave the <(...) as-is
                    result.push_str("<(");
                    result.push_str(cmd);
                    result.push(')');
                }
            } else {
                result.push(c);
            }
        }

        // Store temp files for cleanup (they'll be cleaned on shell drop or next command)
        self.temp_files_for_cleanup = temp_files;

        result
    }

    /// Execute a command and capture its stdout to a temporary file.
    fn capture_to_temp(&mut self, cmd: &str) -> Result<std::path::PathBuf> {
        let tmp_dir = std::env::temp_dir();
        let tmp_name = format!("ash_ps_{}", std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .subsec_nanos());
        let tmp_path = tmp_dir.join(tmp_name);

        // Execute the command and capture output
        let output = self.execute(cmd)?;
        let content = output.unwrap_or_default();
        std::fs::write(&tmp_path, &content)
            .map_err(|e| miette::miette!("process substitution: {}", e))?;

        Ok(tmp_path)
    }

    ///
    /// Prevents infinite recursion with a max expansion depth of 10.
    fn expand_aliases(&self, input: &str) -> String {
        if self.aliases.is_empty() {
            return input.to_string();
        }

        // Split by pipe/chain operators, expand first word of each segment,
        // then reassemble. We reuse parse_chain for splitting.
        let segments = parse_chain(input);
        let mut result = String::new();

        for seg in &segments {
            // Expand the first word of this segment
            let expanded_cmd = self.expand_alias_first_word(&seg.command, 0);
            result.push_str(&expanded_cmd);
            if let Some(ref op) = seg.op {
                match op {
                    ChainOp::Pipe => result.push_str(" | "),
                    ChainOp::And => result.push_str(" && "),
                    ChainOp::Or => result.push_str(" || "),
                }
            }
        }

        result
    }

    /// Expand the first word of a single command via aliases, up to `max_depth`.
    fn expand_alias_first_word(&self, command: &str, depth: usize) -> String {
        if depth > 10 {
            return command.to_string(); // prevent infinite recursion
        }

        let trimmed = command.trim_start();
        if trimmed.is_empty() {
            return command.to_string();
        }

        // Find the first word
        let first_word_end = trimmed.find(|c: char| c.is_whitespace()).unwrap_or(trimmed.len());
        let first_word = &trimmed[..first_word_end];

        if let Some(expansion) = self.aliases.get(first_word) {
            let rest = &trimmed[first_word_end..];
            let expanded = format!("{}{}", expansion, rest);
            // Recursively expand in case the alias itself starts with an alias
            return self.expand_alias_first_word(&expanded, depth + 1);
        }

        command.to_string()
    }

    // -----------------------------------------------------------------------
    // Plan 302 Step 2.2: Tilde expansion
    // -----------------------------------------------------------------------

    /// Expand `~` and `~/path` to the user's home directory.
    ///
    /// Only expands `~` at word boundaries (after whitespace or at start of string).
    /// `~` inside quotes is NOT expanded.
    fn expand_tilde(&self, input: &str) -> String {
        let home = match dirs::home_dir() {
            Some(h) => h.to_string_lossy().to_string(),
            None => return input.to_string(),
        };

        let mut result = String::with_capacity(input.len() + 64);
        let mut chars = input.chars().peekable();
        let mut in_single_quote = false;
        let mut in_double_quote = false;
        let mut word_start = true;

        while let Some(c) = chars.next() {
            match c {
                '\'' if !in_double_quote => {
                    in_single_quote = !in_single_quote;
                    result.push(c);
                    word_start = false;
                }
                '"' if !in_single_quote => {
                    in_double_quote = !in_double_quote;
                    result.push(c);
                    word_start = false;
                }
                ' ' | '\t' | '|' | '&' | ';' if !in_single_quote && !in_double_quote => {
                    result.push(c);
                    word_start = true;
                }
                '~' if word_start && !in_single_quote && !in_double_quote => {
                    // Check what follows ~
                    match chars.peek() {
                        Some('/') => {
                            // ~/path → home + /path
                            chars.next(); // consume /
                            result.push_str(&home);
                            result.push('/');
                        }
                        Some(&' ') | Some(&'\t') | Some(&'|') | Some(&'&') | Some(&';') | None => {
                            // Lone ~ → home
                            result.push_str(&home);
                        }
                        _ => {
                            // ~something (like ~user) — on Windows, just expand to home
                            result.push_str(&home);
                        }
                    }
                    word_start = false;
                }
                _ => {
                    result.push(c);
                    word_start = false;
                }
            }
        }

        result
    }

    // -----------------------------------------------------------------------
    // Plan 302 Step 2.1: Glob expansion
    // -----------------------------------------------------------------------

    /// Expand glob patterns (`*`, `?`, `**/`) in command arguments.
    ///
    /// The first element (command name) is never expanded.
    /// If a pattern has no matches, the original pattern is kept (like bash).
    /// Results are sorted alphabetically (like bash).
    fn expand_globs(&self, parts: &[String]) -> Vec<String> {
        let mut expanded = Vec::with_capacity(parts.len());

        for (i, part) in parts.iter().enumerate() {
            if i == 0 {
                // Command name — never expand
                expanded.push(part.clone());
                continue;
            }

            // Only expand if the argument contains glob metacharacters
            if !part.contains('*') && !part.contains('?') {
                expanded.push(part.clone());
                continue;
            }

            // Build absolute pattern for glob crate
            let pattern = if part.starts_with('/') || part.starts_with('\\') {
                part.clone()
            } else {
                format!("{}/{}", self.current_dir.display(), part)
            };

            // On Windows, normalize path separators for glob
            #[cfg(windows)]
            let pattern = pattern.replace('\\', "/");

            match glob::glob(&pattern) {
                Ok(paths) => {
                    let mut matches: Vec<String> = paths
                        .filter_map(|p| p.ok())
                        .map(|p| {
                            // Convert back to relative path if it was originally relative
                            let abs = p.to_string_lossy().to_string();
                            #[cfg(windows)]
                            let abs = abs.replace('\\', "/");
                            if !part.starts_with('/') && !part.starts_with('\\') {
                                let prefix = format!("{}/", self.current_dir.display());
                                #[cfg(windows)]
                                let prefix = prefix.replace('\\', "/");
                                if let Some(rel) = abs.strip_prefix(&prefix) {
                                    return rel.to_string();
                                }
                            }
                            abs
                        })
                        .collect();
                    if matches.is_empty() {
                        // No matches — keep original pattern (bash behaviour)
                        expanded.push(part.clone());
                    } else {
                        matches.sort();
                        expanded.extend(matches);
                    }
                }
                Err(_) => {
                    // Invalid pattern — keep original
                    expanded.push(part.clone());
                }
            }
        }

        expanded
    }

    /// Handle the `alias` builtin command.
    ///
    /// - `alias` — list all aliases
    /// - `alias name=command` — define an alias (also supports `alias name command`)
    /// - `alias name='command with args'` — define alias with spaces
    fn cmd_alias(&mut self, input: &str) -> Result<Option<String>> {
        // Strip the "alias" keyword
        let rest = input.trim_start().strip_prefix("alias").unwrap_or("").trim();

        if rest.is_empty() {
            // List all aliases
            if self.aliases.is_empty() {
                return Ok(None);
            }
            let mut output = String::new();
            let mut names: Vec<&String> = self.aliases.keys().collect();
            names.sort();
            for name in names {
                let value = &self.aliases[name];
                output.push_str(&format!("alias {}='{}'\n", name, value));
            }
            return Ok(Some(output.trim_end().to_string()));
        }

        // Parse: alias name=value or alias name=value
        // Support: alias ll='ls -la', alias ll=ls, alias ll ls -la
        if let Some(eq_pos) = rest.find('=') {
            let name = rest[..eq_pos].trim().to_string();
            let value = rest[eq_pos + 1..].trim().to_string();
            let value = value.trim_matches('\'').trim_matches('"').to_string();
            if !name.is_empty() && !value.is_empty() {
                self.aliases.insert(name, value);
            }
        } else {
            // Space-separated: alias ll "ls -la"
            let mut parts = rest.splitn(2, |c: char| c.is_whitespace());
            if let (Some(name), Some(value)) = (parts.next(), parts.next()) {
                let name = name.to_string();
                let value = value.trim_matches('\'').trim_matches('"').to_string();
                if !name.is_empty() && !value.is_empty() {
                    self.aliases.insert(name, value);
                }
            }
        }

        Ok(None)
    }

    /// Set an alias programmatically (used by ash.toml config loader).
    pub fn set_alias(&mut self, name: &str, value: &str) {
        self.aliases.insert(name.to_string(), value.to_string());
    }

    /// Handle the `unalias` builtin command.
    fn cmd_unalias(&mut self, parts: &[&str]) -> Result<Option<String>> {
        if parts.len() < 2 {
            miette::bail!("unalias: missing alias name");
        }
        for name in &parts[1..] {
            if self.aliases.remove(*name).is_none() {
                eprintln!("unalias: {}: not found", name);
            }
        }
        Ok(None)
    }

    /// Manage abbreviations: `abbr -a name "expansion"` / `abbr -l` / `abbr -r name`
    fn cmd_abbr(&mut self, parts: &[&str]) -> Result<Option<String>> {
        if parts.len() < 2 {
            return Ok(Some(
                "abbr — manage abbreviations\n\nUSAGE:\n  abbr -a <name> <expansion>   Add abbreviation\n  abbr -r <name>               Remove abbreviation\n  abbr -l, abbr list           List all abbreviations\n\nAbbreviations expand in-line when you type them (like Fish).\n".to_string()
            ));
        }
        match parts[1] {
            "-a" | "--add" => {
                if parts.len() < 4 {
                    return Err(miette::miette!("abbr -a: requires name and expansion"));
                }
                let name = parts[2];
                let expansion = parts[3..].join(" ");
                let expansion = expansion.trim_matches('\'').trim_matches('"').to_string();
                self.abbreviations.insert(name.to_string(), expansion);
                Ok(None)
            }
            "-r" | "--remove" => {
                if parts.len() < 3 {
                    return Err(miette::miette!("abbr -r: requires name"));
                }
                if self.abbreviations.remove(parts[2]).is_none() {
                    eprintln!("abbr: {}: not found", parts[2]);
                }
                Ok(None)
            }
            "-l" | "--list" | "list" => {
                if self.abbreviations.is_empty() {
                    return Ok(Some("No abbreviations.\n".to_string()));
                }
                let mut out = String::new();
                let mut names: Vec<&String> = self.abbreviations.keys().collect();
                names.sort();
                for name in names {
                    out.push_str(&format!("abbr -a {} '{}'\n", name, self.abbreviations[name]));
                }
                Ok(Some(out))
            }
            _ => Err(miette::miette!("abbr: unknown option '{}'. Use -a, -r, or -l", parts[1])),
        }
    }

    /// Manage shell configuration: `config get <key>`, `config set <key> <value>`, `config list`
    fn cmd_config(&mut self, parts: &[&str]) -> Result<Option<String>> {
        if parts.len() < 2 {
            return Ok(Some(
                "config — manage shell configuration\n\nUSAGE:\n  config list             Show all settings\n  config get <key>        Get a setting value\n  config set <key> <val>  Set a setting value\n\nKEYS:\n  shell.history_size           History size (integer)\n  shell.autosuggestion         Fish-style autosuggestions (true/false)\n  shell.autosuggestion_min_chars  Min chars for autosuggestion (integer)\n  shell.edit_mode              \"emacs\" or \"vi\"\n  shell.syntax_highlighting    Syntax highlighting (true/false)\n  completion.case_sensitive    Case-sensitive completion (true/false)\n\nSettings are saved to ~/.config/ash.toml\n".to_string()
            ));
        }
        match parts[1] {
            "list" | "ls" | "show" => {
                let config = crate::config::AshShellConfig::load();
                let mut out = String::new();
                out.push_str(&format!("  shell.history_size = {}\n", config.history_size));
                out.push_str(&format!("  shell.autosuggestion = {}\n", config.autosuggestion));
                out.push_str(&format!("  shell.autosuggestion_min_chars = {}\n", config.autosuggestion_min_chars));
                out.push_str(&format!("  shell.edit_mode = \"{}\"\n", config.edit_mode));
                out.push_str(&format!("  shell.syntax_highlighting = {}\n", config.syntax_highlighting));
                out.push_str(&format!("  completion.case_sensitive = {}\n", config.completion_case_sensitive));
                if !config.aliases.is_empty() {
                    out.push_str("\n  [aliases]\n");
                    let mut names: Vec<&String> = config.aliases.keys().collect();
                    names.sort();
                    for name in names {
                        out.push_str(&format!("  {} = \"{}\"\n", name, config.aliases[name]));
                    }
                }
                Ok(Some(out))
            }
            "get" => {
                if parts.len() < 3 {
                    return Err(miette::miette!("config get: missing key"));
                }
                let config = crate::config::AshShellConfig::load();
                let value = match parts[2] {
                    "shell.history_size" => config.history_size.to_string(),
                    "shell.autosuggestion" => config.autosuggestion.to_string(),
                    "shell.autosuggestion_min_chars" => config.autosuggestion_min_chars.to_string(),
                    "shell.edit_mode" => config.edit_mode.clone(),
                    "shell.syntax_highlighting" => config.syntax_highlighting.to_string(),
                    "completion.case_sensitive" => config.completion_case_sensitive.to_string(),
                    _ => return Err(miette::miette!("config: unknown key '{}'", parts[2])),
                };
                Ok(Some(format!("{}\n", value)))
            }
            "set" => {
                if parts.len() < 4 {
                    return Err(miette::miette!("config set: requires key and value"));
                }
                let key = parts[2];
                let value = parts[3..].join(" ");
                self.config_set(key, &value)?;
                Ok(Some(format!("{} = {}\n", key, value)))
            }
            "migrate" => self.config_migrate(),
            _ => Err(miette::miette!("config: unknown subcommand '{}'. Use list, get, set, or migrate", parts[1])),
        }
    }

    /// Manage key bindings: `bind <key> <action>` or `bind list`
    /// Bindings are saved to ~/.config/ash/bindings.toml and loaded on next start.
    fn cmd_bind(&mut self, parts: &[&str]) -> Result<Option<String>> {
        if parts.len() < 2 {
            return Ok(Some(
                "bind — manage key bindings\n\nUSAGE:\n  bind list                    List current bindings\n  bind <key> <action>          Bind a key to an action\n\nKEYS (examples):\n  ctrl+r, ctrl+s, ctrl+e, ctrl+f\n  alt+left, alt+right\n  f1, f2, ... f12\n  enter, tab, escape, backspace\n\nACTIONS:\n  history-search    Search history\n  complete          Accept autosuggestion\n  edit-line         Open in $EDITOR\n  repaint           Repaint prompt\n\nBindings take effect on next shell start.\n".to_string()
            ));
        }

        match parts[1] {
            "list" | "ls" => {
                let bindings_path = dirs::config_dir()
                    .unwrap_or_else(|| std::path::PathBuf::from("."))
                    .join("ash")
                    .join("bindings.toml");

                if bindings_path.exists() {
                    let content = std::fs::read_to_string(&bindings_path)
                        .map_err(|e| miette::miette!("bind: {}", e))?;
                    Ok(Some(format!("Custom bindings ({}):\n{}\n", bindings_path.display(), content)))
                } else {
                    Ok(Some("No custom bindings configured.\nDefault bindings:\n  Ctrl+R  — history search\n  Ctrl+S  — forward search\n  Ctrl+E  — edit in $EDITOR\n  Ctrl+F  — accept autosuggestion\n  Ctrl+→  — accept next word\n  Tab     — completion\n\nUse 'bind <key> <action>' to add bindings.\n".to_string()))
                }
            }
            key => {
                if parts.len() < 3 {
                    return Err(miette::miette!("bind: missing action for key '{}'", key));
                }
                let action = parts[2];
                self.save_binding(key, action)?;
                Ok(Some(format!("bind: {} → {} (saved, takes effect on next start)\n", key, action)))
            }
        }
    }

    /// Save a key binding to ~/.config/ash/bindings.toml
    fn save_binding(&mut self, key: &str, action: &str) -> Result<()> {
        let bindings_dir = dirs::config_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("ash");
        let bindings_path = bindings_dir.join("bindings.toml");

        let _ = std::fs::create_dir_all(&bindings_dir);

        // Read existing or create new
        let mut doc = if bindings_path.exists() {
            let content = std::fs::read_to_string(&bindings_path).unwrap_or_default();
            content.parse::<toml_edit::DocumentMut>().unwrap_or_default()
        } else {
            toml_edit::DocumentMut::new()
        };

        doc[key] = toml_edit::Item::Value(toml_edit::Value::from(action));

        std::fs::write(&bindings_path, doc.to_string())
            .map_err(|e| miette::miette!("bind: failed to write: {}", e))
    }

    /// Write a config value to `~/.config/ash/config.at` (Plan 318 Auto/Atom).
    fn config_set(&mut self, key: &str, value: &str) -> Result<()> {
        // Load existing config.at (auto_config map).
        let mut cfg = crate::auto_config::load();

        // Parse key into block + field (e.g. "shell.edit_mode").
        let (block, field) = if let Some(dot) = key.find('.') {
            (&key[..dot], &key[dot + 1..])
        } else {
            ("shell", key)
        };
        cfg.entry(block.to_string())
            .or_default()
            .insert(field.to_string(), value.trim_matches('"').to_string());

        // Write config.at.
        let path = crate::auto_config::write_path("config.at")
            .ok_or_else(|| miette::miette!("config: no config dir"))?;
        let text = crate::auto_config::serialize(&cfg);
        std::fs::write(&path, text)
            .map_err(|e| miette::miette!("config: failed to write {}: {}", path.display(), e))?;
        Ok(())
    }

    /// Migrate `~/.config/ash.toml` → `~/.config/ash/config.at` (Plan 318).
    fn config_migrate(&mut self) -> Result<Option<String>> {
        let toml_path = dirs::config_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("ash.toml");
        if !toml_path.exists() {
            miette::bail!("config migrate: ash.toml not found at {}", toml_path.display());
        }
        let content = std::fs::read_to_string(&toml_path)
            .map_err(|e| miette::miette!("config migrate: {}", e))?;
        let old = crate::config::AshShellConfig::parse_toml(&content);

        // Build auto_config map from the parsed TOML config.
        use std::collections::HashMap;
        let mut cfg: HashMap<String, HashMap<String, String>> = HashMap::new();
        let mut shell = HashMap::new();
        shell.insert("history_size".into(), old.history_size.to_string());
        shell.insert("autosuggestion".into(), old.autosuggestion.to_string());
        shell.insert("autosuggestion_min_chars".into(), old.autosuggestion_min_chars.to_string());
        shell.insert("edit_mode".into(), old.edit_mode.clone());
        shell.insert("syntax_highlighting".into(), old.syntax_highlighting.to_string());
        cfg.insert("shell".into(), shell);
        if !old.aliases.is_empty() {
            cfg.insert("aliases".into(), old.aliases.clone());
        }
        if old.completion_case_sensitive {
            let mut comp = HashMap::new();
            comp.insert("case_sensitive".into(), "true".into());
            cfg.insert("completion".into(), comp);
        }

        let path = crate::auto_config::write_path("config.at")
            .ok_or_else(|| miette::miette!("config migrate: no config dir"))?;
        std::fs::write(&path, crate::auto_config::serialize(&cfg))
            .map_err(|e| miette::miette!("config migrate: write {}: {}", path.display(), e))?;
        Ok(Some(format!(
            "migrated ash.toml → {} (you can now delete the old ash.toml)",
            path.display()
        )))
    }

    /// Expand abbreviations in the input line (first word of each pipe segment).
    /// Returns the expanded line and whether any expansion occurred.
    pub fn expand_abbreviations(&self, line: &str) -> (String, bool) {
        if self.abbreviations.is_empty() {
            return (line.to_string(), false);
        }
        let segments = parse_chain(line);
        let mut result = String::new();
        let mut any_expanded = false;

        for seg in &segments {
            // Try to expand the first word
            let trimmed_cmd = seg.command.trim();
            if let Some(space_pos) = trimmed_cmd.find(|c: char| c.is_whitespace()) {
                let first_word = &trimmed_cmd[..space_pos];
                let rest = &trimmed_cmd[space_pos..];
                if let Some(expansion) = self.abbreviations.get(first_word) {
                    result.push_str(expansion);
                    result.push_str(rest);
                    any_expanded = true;
                } else {
                    result.push_str(trimmed_cmd);
                }
            } else {
                // Single word command
                if let Some(expansion) = self.abbreviations.get(trimmed_cmd) {
                    result.push_str(expansion);
                    any_expanded = true;
                } else {
                    result.push_str(trimmed_cmd);
                }
            }
            if let Some(ref op) = seg.op {
                match op {
                    ChainOp::Pipe => result.push_str(" | "),
                    ChainOp::And => result.push_str(" && "),
                    ChainOp::Or => result.push_str(" || "),
                }
            }
        }
        (result, any_expanded)
    }

    // -----------------------------------------------------------------------
    // Plan 302 Step 1.3 + 3.3: RC file / source
    // -----------------------------------------------------------------------

    /// Try to auto-load a command from ~/.config/ash/functions/<name>.at
    /// Returns Some(output) if the file was found and sourced, None if not found.
    /// This implements Fish-style function auto-loading: when a command is not
    /// found in the registry, builtins, or Auto functions, check if a file
    /// exists in the functions directory and source it.
    fn try_autoload(&mut self, name: &str) -> Result<Option<Option<String>>> {
        let func_dir = dirs::config_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("ash")
            .join("functions");

        let candidate = func_dir.join(format!("{}.at", name));
        if candidate.exists() {
            self.source_file(&candidate)?;
            Ok(Some(None)) // sourced successfully
        } else {
            Ok(None) // not found
        }
    }

    /// Execute each line of a file in the current shell context.
    ///
    /// Used by both `~/.ashrc` loading and the `source` builtin.
    /// Supports `>` prefix syntax for shell commands within mixed scripts.
    pub fn source_file(&mut self, path: &std::path::Path) -> Result<()> {
        let content = std::fs::read_to_string(path)
            .into_diagnostic()
            .map_err(|e| miette::miette!("source: {}: {}", path.display(), e))?;

        self.execute_script_content(&content)
    }

    // -----------------------------------------------------------------------
    // Plan 303 Step 2: Script execution with `>` shell syntax
    // -----------------------------------------------------------------------

    /// Execute a script file with `>` shell syntax support.
    ///
    /// Usage: `ash deploy.at` — runs the file, treating `>`-prefixed lines
    /// as shell commands and everything else as AutoLang code.
    pub fn execute_script_file(&mut self, path: &std::path::Path) -> Result<()> {
        let content = std::fs::read_to_string(path)
            .into_diagnostic()
            .map_err(|e| miette::miette!("ash: {}: {}", path.display(), e))?;

        self.execute_script_content(&content)
    }

    /// Execute script content with `>` shell syntax and heredoc support.
    ///
    /// Pre-processes lines into categories:
    /// 1. **Auto blocks** (non-`>` lines) — accumulated and sent to the VM as
    ///    a single block so multi-line constructs (`fn`, `for`, `if`) work.
    /// 2. **Shell lines** (`>` prefix) — interpolated and executed one-by-one.
    /// 3. **Here documents** (`<<MARKER`) — collect lines until marker, feed as stdin.
    pub fn execute_script_content(&mut self, content: &str) -> Result<()> {
        let mut auto_block = String::new();
        let mut lines = content.lines().peekable();

        while let Some(line) = lines.next() {
            let trimmed = line.trim();

            // ── Heredoc collection mode ──
            // If a previous shell command contained <<MARKER, we've already
            // extracted the marker. The loop below collects the heredoc body.
            // (Heredocs are handled inline within the shell command section.)

            // Skip empty lines
            if trimmed.is_empty() {
                if !auto_block.is_empty() {
                    auto_block.push('\n');
                }
                continue;
            }

            // Skip comments (both // and #)
            if trimmed.starts_with('#') || trimmed.starts_with("//") {
                if !auto_block.is_empty() {
                    auto_block.push('\n');
                }
                continue;
            }

            // Shell line: starts with >
            if trimmed.starts_with('>') {
                self.flush_auto_block(&mut auto_block)?;

                let cmd = trimmed[1..].trim();
                if cmd.is_empty() {
                    continue;
                }
                let cmd = self.interpolate_auto_vars(cmd);

                match self.execute(&cmd) {
                    Ok(Some(output)) => println!("{}", output),
                    Ok(None) => {}
                    Err(e) => eprintln!("Error: {}", e),
                }
                continue;
            }

            // Plan 303 Step 5: Assignment capture — let/var name = > cmd
            if let Some(captured) = self.try_capture_assignment(trimmed) {
                self.flush_auto_block(&mut auto_block)?;
                let _ = self.session.run(&captured);
                continue;
            }

            // ── Plan 304: Heredoc detection in shell/command lines ──
            // Lines containing <<MARKER (but not in Auto code) are shell heredocs.
            // Pattern: `command <<MARKER` or `command << MARKER` or `command <<-MARKER`
            if let Some(heredoc) = Self::parse_heredoc_start(trimmed) {
                self.flush_auto_block(&mut auto_block)?;

                // Collect heredoc body lines until the marker
                let mut body = String::new();
                let mut found_end = false;
                while let Some(body_line) = lines.next() {
                    let body_trimmed = if heredoc.strip_tabs {
                        body_line.trim_start_matches('\t')
                    } else {
                        body_line
                    };
                    if body_trimmed.trim() == heredoc.marker {
                        found_end = true;
                        break;
                    }
                    body.push_str(body_line);
                    body.push('\n');
                }
                if !found_end {
                    eprintln!("ash: warning: heredoc delimited by end-of-file (wanted '{}')", heredoc.marker);
                }

                // Execute the command with heredoc body as stdin
                let mut cmd = heredoc.command.clone();
                if heredoc.expand_vars {
                    cmd = self.interpolate_auto_vars(&cmd);
                    // Also expand vars in heredoc body
                    let expanded_body = self.interpolate_auto_vars(&body);
                    self.execute_with_stdin(&cmd, &expanded_body)?;
                } else {
                    self.execute_with_stdin(&cmd, &body)?;
                }
                continue;
            }

            // Regular Auto line: accumulate
            auto_block.push_str(line);
            auto_block.push('\n');
        }

        self.flush_auto_block(&mut auto_block)?;

        Ok(())
    }

    /// Parse a heredoc start: `command <<MARKER` or `command << MARKER` or `command <<-MARKER`
    ///
    /// Returns a `HeredocInfo` if the line contains a heredoc, or `None`.
    fn parse_heredoc_start(line: &str) -> Option<HeredocInfo> {
        // Find << that is NOT inside quotes
        let mut in_single = false;
        let mut in_double = false;
        let chars: Vec<char> = line.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            match chars[i] {
                '\'' if !in_double => in_single = !in_single,
                '"' if !in_single => in_double = !in_double,
                '<' if !in_single && !in_double && i + 1 < chars.len() && chars[i + 1] == '<' => {
                    // Found << — extract the rest
                    let strip_tabs = i + 2 < chars.len() && chars[i + 2] == '-';
                    let start = if strip_tabs { i + 3 } else { i + 2 };

                    // Skip whitespace
                    let mut j = start;
                    while j < chars.len() && chars[j] == ' ' { j += 1; }

                    // Read marker
                    let mut marker = String::new();
                    while j < chars.len() && !chars[j].is_whitespace() {
                        marker.push(chars[j]);
                        j += 1;
                    }

                    if marker.is_empty() {
                        return None; // << without a marker is invalid
                    }

                    // Check for quoted marker (no expansion)
                    let expand_vars = !marker.starts_with('\'') && !marker.starts_with('"');
                    // Strip quotes from marker
                    let clean_marker = marker.trim_matches('\'').trim_matches('"').to_string();

                    // Command is everything before <<
                    let command = line[..i].trim().to_string();
                    if command.is_empty() {
                        return None;
                    }

                    return Some(HeredocInfo {
                        command,
                        marker: clean_marker,
                        strip_tabs,
                        expand_vars,
                    });
                }
                _ => {}
            }
            i += 1;
        }
        None
    }

    /// Execute a command with the given string piped to stdin.
    fn execute_with_stdin(&mut self, command: &str, stdin_data: &str) -> Result<Option<String>> {
        use std::process::{Command, Stdio};
        use std::io::Write;

        let parts = crate::parser::quote::parse_args(command);
        if parts.is_empty() {
            return Ok(None);
        }

        // Check builtins first — for heredoc, feed stdin_data as pipeline input
        if let Some(output) = crate::cmd::builtin::execute_builtin_with_input(
            command, &self.current_dir, Some(stdin_data))?
        {
            println!("{}", output);
            return Ok(None);
        }

        // External command: pipe stdin_data
        let mut cmd = Command::new(&parts[0]);
        cmd.args(&parts[1..])
            .current_dir(&self.current_dir)
            .stdin(Stdio::piped())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit());

        let mut child = cmd.spawn().into_diagnostic()?;

        if let Some(mut stdin) = child.stdin.take() {
            let _ = stdin.write_all(stdin_data.as_bytes());
        }

        let status = child.wait().into_diagnostic()?;
        if !status.success() {
            self.last_exit_code = status.code().unwrap_or(1);
        }

        Ok(None)
    }

    /// Flush accumulated Auto code block to the VM.
    fn flush_auto_block(&mut self, block: &mut String) -> Result<()> {
        if block.trim().is_empty() {
            return Ok(());
        }
        let result = self.session.run(block);
        if let Err(e) = result {
            eprintln!("Error: {}", e);
        }
        block.clear();
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Plan 303 Step 5: Assignment capture (let x = > cmd)
    // -----------------------------------------------------------------------

    /// Try to parse an assignment-capture line: `let x = > cmd` or `var x = > cmd`.
    ///
    /// Returns `Some(auto_code)` if the pattern matches, where `auto_code` is
    /// an Auto `let`/`var` statement with the captured stdout as a string value.
    /// Returns `None` if this is a regular Auto line.
    fn try_capture_assignment(&mut self, line: &str) -> Option<String> {
        // Check for `let ` or `var ` prefix
        let rest = if let Some(r) = line.strip_prefix("let ") {
            r
        } else if let Some(r) = line.strip_prefix("var ") {
            r
        } else {
            return None;
        };

        // Look for `= >` separator
        let eq_pos = rest.find("= >")?;
        let var_name = rest[..eq_pos].trim();

        // Validate variable name
        if var_name.is_empty() || var_name.chars().next()?.is_ascii_digit() {
            return None;
        }

        // Extract the shell command after `= >`
        let cmd = rest[eq_pos + 3..].trim();
        if cmd.is_empty() {
            return None;
        }

        // Interpolate Auto variables in the command
        let cmd = self.interpolate_auto_vars(cmd);

        // Execute and capture stdout
        let output = match self.execute(&cmd) {
            Ok(Some(out)) => out,
            Ok(None) => String::new(),
            Err(e) => {
                eprintln!("Error: {}", e);
                String::new()
            }
        };

        // Escape the output for embedding in an Auto string literal
        let escaped = output
            .replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', "\\n")
            .replace('\r', "");

        // Build the Auto assignment
        let keyword = if line.starts_with("var") { "var" } else { "let" };
        Some(format!("{} {} = \"{}\"", keyword, var_name, escaped))
    }

    // -----------------------------------------------------------------------
    // Plan 303 Step 3: Auto variable interpolation
    // -----------------------------------------------------------------------

    /// Replace `$var` in shell command text with Auto VM variable values.
    ///
    /// Lookup priority: Auto VM locals → Shell local vars → Environment vars.
    fn interpolate_auto_vars(&self, cmd: &str) -> String {
        let mut result = String::with_capacity(cmd.len());
        let mut chars = cmd.chars().peekable();

        while let Some(c) = chars.next() {
            if c == '$' {
                // Read variable name
                let mut name = String::new();
                while let Some(&ch) = chars.peek() {
                    if ch.is_alphanumeric() || ch == '_' {
                        name.push(ch);
                        chars.next();
                    } else {
                        break;
                    }
                }
                if name.is_empty() {
                    result.push('$');
                    continue;
                }

                // Priority: Auto VM vars > Shell vars > Env vars
                let value = self.session.get_var_string(&name)
                    .or_else(|| self.vars.get_local(&name).cloned())
                    .or_else(|| self.vars.get_env(&name))
                    .unwrap_or_default();

                result.push_str(&value);
            } else {
                result.push(c);
            }
        }

        result
    }

    /// Handle the `source` and `.` builtin commands.
    fn cmd_source(&mut self, parts: &[&str]) -> Result<Option<String>> {
        if parts.len() < 2 {
            miette::bail!("source: missing file path");
        }
        let path = std::path::Path::new(parts[1]);
        // Resolve relative paths against cwd
        let path = if path.is_relative() {
            self.current_dir.join(path)
        } else {
            path.to_path_buf()
        };
        self.source_file(&path)?;
        Ok(None)
    }

    // -----------------------------------------------------------------------
    // Plan 302 Step 3.5: Directory stack (pushd / popd / dirs)
    // -----------------------------------------------------------------------

    /// `pushd [dir]` — push current dir onto stack and cd to `dir`.
    /// `pushd` (no arg) — swap top two entries (like bash).
    fn cmd_pushd(&mut self, parts: &[&str]) -> Result<Option<String>> {
        if parts.len() < 2 {
            // No arg: swap current dir with top of stack
            if self.dir_stack.is_empty() {
                miette::bail!("pushd: no other directory");
            }
            let old_current = self.current_dir.clone();
            let new_dir = self.dir_stack.pop().unwrap();
            self.cd(&new_dir.to_string_lossy())?;
            self.dir_stack.push(old_current);
        } else {
            let target = parts[1];
            // Push current dir, then cd to target
            self.dir_stack.push(self.current_dir.clone());
            if let Err(e) = self.cd(target) {
                // cd failed — restore stack
                self.dir_stack.pop();
                return Err(e);
            }
        }
        Ok(Some(self.format_dir_stack()))
    }

    /// `popd` — pop from stack and cd to it.
    /// `popd +N` — remove the Nth entry (0-indexed from top) without cd.
    fn cmd_popd(&mut self, parts: &[&str]) -> Result<Option<String>> {
        if self.dir_stack.is_empty() {
            miette::bail!("popd: directory stack empty");
        }

        if parts.len() > 1 && parts[1].starts_with('+') {
            // popd +N: remove Nth entry from stack (0 = top)
            if let Ok(n) = parts[1][1..].parse::<usize>() {
                if n < self.dir_stack.len() {
                    let idx = self.dir_stack.len() - 1 - n;
                    self.dir_stack.remove(idx);
                    return Ok(Some(self.format_dir_stack()));
                }
            }
            miette::bail!("popd: invalid index: {}", parts[1]);
        }

        let target = self.dir_stack.pop().unwrap();
        self.cd(&target.to_string_lossy())?;
        Ok(Some(self.format_dir_stack()))
    }

    /// `dirs` — display the directory stack.
    /// `dirs -c` — clear the stack.
    /// `dirs -v` — verbose (one per line with indices).
    fn cmd_dirs(&mut self, parts: &[&str]) -> Result<Option<String>> {
        if parts.len() > 1 {
            match parts[1] {
                "-c" => {
                    self.dir_stack.clear();
                    return Ok(None);
                }
                "-v" => {
                    let mut output = format!(" 0  {}\n", self.current_dir.display());
                    for (i, dir) in self.dir_stack.iter().rev().enumerate() {
                        output.push_str(&format!(" {}  {}\n", i + 1, dir.display()));
                    }
                    return Ok(Some(output.trim_end().to_string()));
                }
                _ => {}
            }
        }
        Ok(Some(self.format_dir_stack()))
    }

    /// Format the directory stack as a single line: current_dir dir1 dir2 ...
    fn format_dir_stack(&self) -> String {
        let mut parts = vec![self.current_dir.display().to_string()];
        for dir in self.dir_stack.iter().rev() {
            parts.push(dir.display().to_string());
        }
        parts.join(" ")
    }

    /// `path` command — manage the PATH environment variable as a list.
    ///
    /// Subcommands:
    ///   `path add <dir>`     — prepend a directory to PATH
    ///   `path append <dir>`  — append a directory to PATH
    ///   `path remove <dir>`  — remove a directory from PATH
    ///   `path list`          — show PATH entries, one per line
    ///   `path clear`         — clear PATH
    fn cmd_path(&mut self, parts: &[&str]) -> Result<Option<String>> {
        let subcmd = parts.get(1).copied().unwrap_or("list");
        let separator = if cfg!(windows) { ';' } else { ':' };

        match subcmd {
            "add" | "prepend" => {
                if parts.len() < 3 {
                    miette::bail!("path add: missing directory argument");
                }
                let dir = parts[2..].join(" ");
                let current = self.vars.get_env("PATH").unwrap_or_default();
                let new_path = if current.is_empty() {
                    dir
                } else {
                    format!("{}{}{}", dir, separator, current)
                };
                self.vars.set_env("PATH".to_string(), new_path);
                Ok(None)
            }
            "append" => {
                if parts.len() < 3 {
                    miette::bail!("path append: missing directory argument");
                }
                let dir = parts[2..].join(" ");
                let current = self.vars.get_env("PATH").unwrap_or_default();
                let new_path = if current.is_empty() {
                    dir
                } else {
                    format!("{}{}{}", current, separator, dir)
                };
                self.vars.set_env("PATH".to_string(), new_path);
                Ok(None)
            }
            "remove" | "rm" => {
                if parts.len() < 3 {
                    miette::bail!("path remove: missing directory argument");
                }
                let dir = parts[2..].join(" ");
                let current = self.vars.get_env("PATH").unwrap_or_default();
                let entries: Vec<&str> = current.split(separator).filter(|e| *e != dir).collect();
                let new_path = entries.join(&separator.to_string());
                self.vars.set_env("PATH".to_string(), new_path);
                Ok(None)
            }
            "list" | "ls" => {
                let current = self.vars.get_env("PATH").unwrap_or_default();
                let entries: Vec<&str> = current.split(separator).filter(|e| !e.is_empty()).collect();
                Ok(Some(entries.join("\n")))
            }
            "clear" => {
                self.vars.set_env("PATH".to_string(), String::new());
                Ok(None)
            }
            _ => {
                miette::bail!("path: unknown subcommand '{}'. Use: add, append, remove, list, clear", subcmd);
            }
        }
    }

    /// `env` / `env.path` commands — environment variable and PATH management.
    /// (Plan 301 / Plan 309 Task 1.2 — Phase 2)
    ///
    /// Subcommands use a **space** separator (works with the exact-match
    /// builtin dispatch); the dotted form `env.path.add` is a future alias.
    ///
    ///   `env`                → list all env vars (table)
    ///   `env NAME`           → query single var (empty string if absent)
    ///   `env NAME=val`       → set env var
    ///   `env -rm NAME`       → remove env var (PATH refused)
    ///   `env.path`           → list PATH entries (table: #, path, exists, dup)
    ///   `env.path add DIR`   → append to PATH
    ///   `env.path pre DIR`   → prepend to PATH
    ///   `env.path rm DIR|#N` → remove by path or index
    ///   `env.path dedup`     → deduplicate (case-insensitive)
    ///   `env.path clean`     → dedup + drop nonexistent dirs
    ///   `env.path move #N to #M` → reorder entry
    fn cmd_env(&mut self, parts: &[&str]) -> Result<Option<String>> {
        if parts[0] == "env.path" {
            return self.cmd_env_path(&parts[1..]);
        }

        // head == "env"
        let args = &parts[1..];
        if args.is_empty() {
            return Ok(Some(self.format_env_table()));
        }

        let first = args[0];
        if first == "-rm" {
            let name = args
                .get(1)
                .ok_or_else(|| miette::miette!("env -rm: missing NAME"))?;
            if name.eq_ignore_ascii_case("PATH") {
                miette::bail!("不能删除 PATH，请使用 env.path 命令操作");
            }
            Self::env_persist_remove(name); // Plan 309 Task 1.2 P4: also drop persisted line
            self.vars.unset_env(name);
            return Ok(None);
        }
        if first == "-save" {
            // Plan 301 §二 / Plan 309 Task 1.2 P4 — persist to ~/.config/ash/env.at
            let name = args
                .get(1)
                .ok_or_else(|| miette::miette!("env -save: missing NAME"))?;
            let val = args
                .get(2)
                .ok_or_else(|| miette::miette!("env -save: missing VALUE"))?;
            if name.eq_ignore_ascii_case("PATH") {
                miette::bail!("PATH 请使用 env.path 命令操作");
            }
            self.vars.set_env(name.to_string(), val.to_string());
            Self::env_persist_upsert(name, val)?;
            return Ok(None);
        }
        if first == "-load" {
            self.load_env_persistence();
            return Ok(None);
        }
        if first.starts_with('-') {
            miette::bail!("env: unknown flag '{}'", first);
        }
        if let Some((name, val)) = first.split_once('=') {
            if name.is_empty() {
                miette::bail!("env: empty variable name");
            }
            self.vars.set_env(name.to_string(), val.to_string());
            return Ok(None);
        }
        // Query single var: empty string if absent (no error), per Plan 301 §1.6.
        Ok(Some(self.vars.get_env(first).unwrap_or_default()))
    }

    // ── Plan 301 §二 / Plan 309 Task 1.2 P4 — env persistence ────────────
    //
    // ~/.config/ash/env.at is a plain list of shell command lines (one per line,
    // `//` comments allowed), e.g.:
    //   env EDITOR=vim
    //   env LANG=zh_CN.UTF-8
    //   env.path pre /usr/local/bin
    // `env -save` upserts an `env NAME=val` line; `env -load` (and startup)
    // execute each line. PATH lines etc. are executed too, so the file format is
    // forward-compatible with any persisted shell command.

    /// Best-effort: read `env.at` and execute each non-comment line.
    pub fn load_env_persistence(&mut self) {
        if let Some(path) = Self::env_file_path_for_read() {
            self.load_env_persistence_from(&path);
        }
    }

    /// Execute each non-comment line of an env-persistence file at `path`.
    fn load_env_persistence_from(&mut self, path: &std::path::Path) {
        if let Ok(content) = std::fs::read_to_string(path) {
            for line in content.lines() {
                let l = line.trim();
                if l.is_empty() || l.starts_with("//") {
                    continue;
                }
                let _ = self.execute(l);
            }
        }
    }

    /// Candidate `env.at` paths in priority order: `~/.config/ash/env.at`,
    /// then `<config_dir>/ash/env.at` (`%APPDATA%` on Windows).
    fn env_file_candidates() -> Vec<PathBuf> {
        let mut v = Vec::new();
        if let Some(home) = dirs::home_dir() {
            v.push(home.join(".config").join("ash").join("env.at"));
        }
        if let Some(cfg) = dirs::config_dir() {
            v.push(cfg.join("ash").join("env.at"));
        }
        v
    }

    fn env_file_path_for_read() -> Option<PathBuf> {
        Self::env_file_candidates().into_iter().find(|p| p.exists())
    }

    fn env_file_path_for_write() -> Option<PathBuf> {
        for cand in Self::env_file_candidates() {
            if let Some(parent) = cand.parent() {
                if std::fs::create_dir_all(parent).is_ok() {
                    return Some(cand);
                }
            }
        }
        None
    }

    /// Upsert a persisted `env NAME=val` line (replaces an existing line for NAME).
    fn env_persist_upsert(name: &str, val: &str) -> Result<()> {
        let path = Self::env_file_path_for_write()
            .ok_or_else(|| miette::miette!("env -save: no writable config dir"))?;
        Self::env_persist_upsert_at(&path, name, val)
    }

    /// Core upsert logic operating on a specific file path (testable).
    fn env_persist_upsert_at(path: &std::path::Path, name: &str, val: &str) -> Result<()> {
        let line = format!("env {}={}", name, val);
        let content = std::fs::read_to_string(path).unwrap_or_default();
        let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
        let prefix = format!("env {}=", name);
        if let Some(slot) = lines.iter_mut().find(|l| l.trim_start().starts_with(&prefix)) {
            *slot = line;
        } else {
            lines.push(line);
        }
        let mut out = lines.join("\n");
        if !out.is_empty() {
            out.push('\n');
        }
        std::fs::write(path, out).map_err(|e| miette::miette!("env -save: {}", e))?;
        Ok(())
    }

    /// Remove any persisted `env NAME=...` line from candidate files.
    fn env_persist_remove(name: &str) {
        for path in Self::env_file_candidates() {
            Self::env_persist_remove_at(&path, name);
        }
    }

    /// Core remove logic for a specific file path (testable).
    fn env_persist_remove_at(path: &std::path::Path, name: &str) {
        if !path.exists() {
            return;
        }
        let Ok(content) = std::fs::read_to_string(path) else {
            return;
        };
        let prefix = format!("env {}=", name);
        let kept: Vec<&str> = content
            .lines()
            .filter(|l| !l.trim_start().starts_with(&prefix))
            .collect();
        let new_content = kept.join("\n");
        let to_write = if new_content.is_empty() {
            new_content
        } else {
            format!("{}\n", new_content)
        };
        let _ = std::fs::write(path, to_write);
    }

    /// `env.path` subcommand dispatcher.
    fn cmd_env_path(&mut self, args: &[&str]) -> Result<Option<String>> {
        let sub = args.get(0).copied().unwrap_or("list");
        match sub {
            "list" | "show" => Ok(Some(self.format_path_table())),
            "add" | "append" => {
                let dir = args
                    .get(1)
                    .ok_or_else(|| miette::miette!("env.path add: missing DIR"))?;
                self.vars.path_add(dir);
                Ok(None)
            }
            "pre" | "prepend" => {
                let dir = args
                    .get(1)
                    .ok_or_else(|| miette::miette!("env.path pre: missing DIR"))?;
                self.vars.path_prepend(dir);
                Ok(None)
            }
            "rm" | "remove" => {
                let arg = args
                    .get(1)
                    .ok_or_else(|| miette::miette!("env.path rm: missing argument"))?;
                if let Some(n) = arg.strip_prefix('#').and_then(|s| s.parse::<usize>().ok()) {
                    self.vars
                        .path_remove_index(n)
                        .map_err(|e| miette::miette!("{}", e))?;
                } else {
                    self.vars.path_remove(arg);
                }
                Ok(None)
            }
            "dedup" => {
                self.vars.path_dedup();
                Ok(None)
            }
            "clean" => {
                self.vars.path_clean();
                Ok(None)
            }
            "move" => {
                let from = args
                    .get(1)
                    .and_then(|s| s.strip_prefix('#'))
                    .and_then(|s| s.parse::<usize>().ok())
                    .ok_or_else(|| miette::miette!("env.path move: usage: move #N to #M"))?;
                // layout: `move #N to #M`
                let to = args
                    .get(3)
                    .and_then(|s| s.strip_prefix('#'))
                    .and_then(|s| s.parse::<usize>().ok())
                    .ok_or_else(|| miette::miette!("env.path move: usage: move #N to #M"))?;
                self.vars
                    .path_move(from, to)
                    .map_err(|e| miette::miette!("{}", e))?;
                Ok(None)
            }
            other => miette::bail!("env.path: unknown subcommand '{}'", other),
        }
    }

    /// Render all environment variables as a two-column aligned table.
    fn format_env_table(&self) -> String {
        // Source of truth is the live process env (set_env keeps it in sync).
        let mut entries: Vec<(String, String)> = std::env::vars().collect();
        entries.sort_by(|a, b| a.0.to_lowercase().cmp(&b.0.to_lowercase()));

        let name_w = entries.iter().map(|(n, _)| n.chars().count()).max().unwrap_or(4).max(4);
        let mut out = String::new();
        out.push_str(&format!("{:<width$}  VALUE\n", "NAME", width = name_w));
        out.push_str(&format!("{}  ─────\n", "─".repeat(name_w)));
        for (name, value) in entries {
            // Show value on one line; collapse nothing.
            out.push_str(&format!("{:<width$}  {}\n", name, value, width = name_w));
        }
        out
    }

    /// Render PATH entries as a table with index / path / exists / duplicate.
    fn format_path_table(&self) -> String {
        let entries = self.vars.get_path_entries();
        let path_w = entries
            .iter()
            .map(|e| e.path.chars().count())
            .max()
            .unwrap_or(4)
            .max(4);
        let mut out = String::new();
        out.push_str(&format!("{:>2}  {:<width$}  EXISTS\n", "#", "PATH", width = path_w));
        out.push_str(&format!("{:─>2}  {}  ──────\n", "", "─".repeat(path_w)));
        for e in &entries {
            let mark = if e.exists { '✓' } else { '✗' };
            let dup = if e.duplicate { "  (duplicate)" } else { "" };
            out.push_str(&format!(
                "{:>2}  {:<width$}  {}{}\n",
                e.index, e.path, mark, dup, width = path_w
            ));
        }
        out
    }

    // ── Plan 315 Phase 2 — `completions` management builtin ──────────────

    /// `completions` — manage three-tier completion specs.
    ///
    ///   completions generate <cmd> [--refresh]   probe `<cmd> --help` → write generated/<cmd>.at
    ///   completions list                        list specs in user/generated/cache
    ///   completions clear <cmd>                 delete cache entry for <cmd>
    ///   completions clear --cache               delete the whole cache tier
    ///   completions path                        print the three tier dir paths
    fn cmd_completions(&mut self, parts: &[&str]) -> Result<Option<String>> {
        let sub = parts.get(1).copied().unwrap_or("");
        match sub {
            "" | "-h" | "--help" => Ok(Some(COMPLETIONS_HELP.to_string())),
            "generate" | "gen" => self.completions_generate(&parts[2..]),
            "list" | "ls" => Ok(Some(self.completions_list())),
            "clear" | "rm" => self.completions_clear(&parts[2..]),
            "path" | "paths" => Ok(Some(self.completions_path())),
            other => miette::bail!("completions: unknown subcommand '{}'", other),
        }
    }

    fn completions_generate(&self, args: &[&str]) -> Result<Option<String>> {
        let mut cmd: Option<&str> = None;
        let mut refresh = false;
        for &a in args {
            match a {
                "--refresh" | "-r" => refresh = true,
                "--man" => {
                    miette::bail!("--man (man-page parsing) is Phase 3, not yet supported")
                }
                other if !other.starts_with('-') => {
                    if cmd.is_some() {
                        miette::bail!("completions generate: only one command at a time");
                    }
                    cmd = Some(other);
                }
                other => miette::bail!("completions generate: unknown flag '{}'", other),
            }
        }
        let cmd = cmd.ok_or_else(|| miette::miette!("completions generate: missing command name"))?;

        let help = Self::capture_cmd_output(&format!("{} --help", cmd), &self.current_dir);
        if help.trim().is_empty() {
            miette::bail!(
                "completions generate: `{} --help` produced no output (is it on PATH?)",
                cmd
            );
        }
        let spec = crate::core::completions::help_parser::parse_help(cmd, &help);
        let dir = crate::completions::spec_tiers::generated_dir()
            .ok_or_else(|| miette::miette!("completions: no writable config dir"))?;
        std::fs::create_dir_all(&dir)
            .map_err(|e| miette::miette!("completions generate: {}", e))?;
        let path = dir.join(format!("{}.at", cmd));
        std::fs::write(&path, crate::core::completions::spec_format::serialize(&spec))
            .map_err(|e| miette::miette!("completions generate: {}", e))?;

        if refresh {
            // Drop any stale cache entry so the new generated spec is unambiguous.
            if let Some(cache) = crate::completions::spec_tiers::cache_dir() {
                let _ = std::fs::remove_file(cache.join(format!("{}.at", cmd)));
            }
        }
        Ok(Some(format!(
            "generated completion spec for '{}' → {} ({} flag{}, {} subcommand{})",
            cmd,
            path.display(),
            spec.flags.len(),
            if spec.flags.len() == 1 { "" } else { "s" },
            spec.subcommands.len(),
            if spec.subcommands.len() == 1 { "" } else { "s" },
        )))
    }

    fn completions_list(&self) -> String {
        let mut out = String::new();
        for (name, dir) in [
            ("user", crate::completions::spec_tiers::user_dir()),
            ("generated", crate::completions::spec_tiers::generated_dir()),
            ("cache", crate::completions::spec_tiers::cache_dir()),
        ] {
            match &dir {
                Some(d) => {
                    let specs: Vec<String> = std::fs::read_dir(d)
                        .into_iter()
                        .flatten()
                        .flatten()
                        .filter_map(|e| {
                            let p = e.path();
                            if p.extension().and_then(|x| x.to_str()) != Some("at") {
                                return None;
                            }
                            p.file_stem().map(|s| s.to_string_lossy().into_owned())
                        })
                        .collect();
                    out.push_str(&format!(
                        "{} ({}): {}\n",
                        name,
                        d.display(),
                        if specs.is_empty() { "(none)".to_string() } else { specs.join(", ") }
                    ));
                }
                None => out.push_str(&format!("{}: (no config dir)\n", name)),
            }
        }
        out
    }

    fn completions_clear(&self, args: &[&str]) -> Result<Option<String>> {
        let cache_dir = crate::completions::spec_tiers::cache_dir()
            .ok_or_else(|| miette::miette!("completions: no config dir"))?;
        let mut removed = 0u32;
        for &a in args {
            if a == "--cache" || a == "-a" {
                if cache_dir.exists() {
                    for entry in std::fs::read_dir(&cache_dir).into_iter().flatten().flatten() {
                        if entry.path().extension().and_then(|x| x.to_str()) == Some("at") {
                            if std::fs::remove_file(entry.path()).is_ok() {
                                removed += 1;
                            }
                        }
                    }
                }
            } else if !a.starts_with('-') {
                let p = cache_dir.join(format!("{}.at", a));
                if p.exists() && std::fs::remove_file(&p).is_ok() {
                    removed += 1;
                }
            }
        }
        Ok(Some(format!("removed {} cache entr{}", removed, if removed == 1 { "y" } else { "ies" })))
    }

    fn completions_path(&self) -> String {
        let mut out = String::new();
        for (name, dir) in [
            ("user", crate::completions::spec_tiers::user_dir()),
            ("generated", crate::completions::spec_tiers::generated_dir()),
            ("cache", crate::completions::spec_tiers::cache_dir()),
        ] {
            out.push_str(&format!(
                "{}: {}\n",
                name,
                dir.map(|d| d.display().to_string())
                    .unwrap_or_else(|| "(none)".to_string())
            ));
        }
        out
    }

    /// Run `cmd` via the platform shell, returning stdout regardless of exit code
    /// (some tools' `--help` exits non-zero while still printing usage to stdout).
    fn capture_cmd_output(cmd: &str, cwd: &std::path::Path) -> String {
        let result = if cfg!(windows) {
            std::process::Command::new("cmd")
                .args(["/C", cmd])
                .current_dir(cwd)
                .output()
        } else {
            std::process::Command::new("sh")
                .args(["-c", cmd])
                .current_dir(cwd)
                .output()
        };
        match result {
            Ok(o) => String::from_utf8_lossy(&o.stdout).to_string(),
            Err(_) => String::new(),
        }
    }

    // ── Plan 317 Phase 3 — `color` builtin (24-bit demo) ─────────────────

    /// `color` — print a 24-bit rainbow gradient / report terminal color depth.
    ///
    ///   color rainbow   → monospaced rainbow (each char a different hue)
    ///   color depth     → report the detected color depth + relevant env vars
    fn cmd_color(&mut self, parts: &[&str]) -> Result<Option<String>> {
        let sub = parts.get(1).copied().unwrap_or("depth");
        match sub {
            "rainbow" | "rb" => {
                let text = "Ash 24-bit Truecolor Rainbow!";
                let chars: Vec<char> = text.chars().collect();
                let n = chars.len();
                let mut out = String::new();
                for (i, ch) in chars.iter().enumerate() {
                    let hue = 360.0 * (i as f64) / (n as f64);
                    let (r, g, b) = hsv_to_rgb(hue, 1.0, 1.0);
                    let color = crate::frontend::term::color::resolve_fg(r, g, b);
                    out.push_str(&color.paint(&ch.to_string()).to_string());
                }
                Ok(Some(out))
            }
            "depth" | "info" => {
                let depth = crate::frontend::term::color::detect_color_depth();
                let label = match depth {
                    crate::frontend::term::color::ColorDepth::True24 => "24-bit truecolor",
                    crate::frontend::term::color::ColorDepth::Index256 => "256-color",
                    crate::frontend::term::color::ColorDepth::Index16 => "16-color",
                };
                let ct = std::env::var("COLORTERM").unwrap_or_else(|_| "(unset)".into());
                let term = std::env::var("TERM").unwrap_or_else(|_| "(unset)".into());
                Ok(Some(format!(
                    "Color depth: {} (COLORTERM={}, TERM={})",
                    label, ct, term
                )))
            }
            other => miette::bail!("color: unknown subcommand '{}'. Use: rainbow, depth", other),
        }
    }


    ///
    /// Translates to AutoLang `fn` syntax. Supports two forms:
    ///
    ///   `def ll [] { ls -la }`          →  `fn ll() { > ls -la }`
    ///   `def greet [name] { echo $name }` →  `fn greet(name) { > echo $name }`
    ///
    /// The body uses `>` prefix for shell commands (ASH convention).
    fn cmd_def(&mut self, input: &str) -> Result<Option<String>> {
        // Parse: def name [params] { body }
        // Strip leading "def "
        let rest = input.strip_prefix("def ").unwrap_or(input).trim();

        // Find the name (first word)
        let (name, remainder) = rest.split_once(|c: char| c.is_whitespace() || c == '[' || c == '{')
            .ok_or_else(|| miette::miette!("def: expected function name"))?;

        let name = name.trim();
        let remainder = remainder.trim();

        // Extract parameters (between [] or before {)
        let (params, body) = if remainder.starts_with('[') {
            // Fish-style: def name [param1 param2] { body }
            if let Some(end) = remainder.find(']') {
                let param_str = remainder[1..end].trim();
                let params: Vec<&str> = if param_str.is_empty() {
                    vec![]
                } else {
                    param_str.split_whitespace().collect()
                };
                let auto_params = params.iter().map(|p| format!("{} str", p)).collect::<Vec<_>>().join(", ");
                let after_bracket = remainder[end + 1..].trim();
                (auto_params, after_bracket)
            } else {
                (String::new(), remainder)
            }
        } else {
            (String::new(), remainder)
        };

        // Extract body (between { })
        let body = body.trim_start_matches('{').trim();
        let body = body.trim_end_matches('}').trim();

        // Convert body lines: shell commands get > prefix, Auto code stays as-is
        let mut auto_body = String::new();
        for line in body.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() { continue; }

            // If the line looks like a shell command (not Auto code), prefix with >
            if !trimmed.starts_with('>')
                && !trimmed.starts_with("let ")
                && !trimmed.starts_with("var ")
                && !trimmed.starts_with("for ")
                && !trimmed.starts_with("if ")
                && !trimmed.starts_with("fn ")
                && !trimmed.starts_with("return ")
                && !trimmed.starts_with("print")
            {
                auto_body.push_str("> ");
            }
            auto_body.push_str(trimmed);
            auto_body.push('\n');
        }

        // Generate AutoLang fn
        let auto_fn = if params.is_empty() {
            format!("fn {}() {{\n{}\n}}", name, auto_body.trim_end())
        } else {
            format!("fn {}({}) {{\n{}\n}}", name, params, auto_body.trim_end())
        };

        self.session.run(&auto_fn).map_err(|e| miette::miette!("def: {}", e))?;
        Ok(None)
    }

    /// Manage event hooks: `hook <event> <function_name>` or `hook list`
    fn cmd_hook(&mut self, parts: &[&str]) -> Result<Option<String>> {
        if parts.len() < 2 {
            return Ok(Some(
                "hook — manage event hooks\n\nUSAGE:\n  hook <event> <function>\n  hook list\n\nEVENTS:\n  chdir    — fired on directory change, args: (old_dir, new_dir)\n  preexec  — fired before command execution, args: (command_line)\n  precmd   — fired after command execution, args: (exit_code)\n".to_string()
            ));
        }
        match parts[1] {
            "list" | "ls" => {
                if self.hooks.is_empty() {
                    return Ok(Some("No hooks registered.\n".to_string()));
                }
                let mut out = String::from("Registered hooks:\n");
                for (event, func) in &self.hooks {
                    out.push_str(&format!("  {} → {}\n", event, func));
                }
                Ok(Some(out))
            }
            event @ ("chdir" | "preexec" | "precmd") => {
                if parts.len() < 3 {
                    return Err(miette::miette!("hook {}: missing function name", event));
                }
                let func_name = parts[2];
                self.register_hook(event, func_name);
                Ok(Some(format!("hook: {} → {}\n", event, func_name)))
            }
            _ => Err(miette::miette!("hook: unknown event '{}'. Valid: chdir, preexec, precmd", parts[1])),
        }
    }

    /// Register an event hook. `event` is one of: chdir, preexec, precmd.
    /// `func_name` is the name of an Auto function to call.
    fn register_hook(&mut self, event: &str, func_name: &str) {
        self.hooks.insert(event.to_string(), func_name.to_string());
    }

    /// Fire an event hook. Looks up the registered function and runs it.
    /// Errors are printed but do not propagate (hooks are best-effort).
    fn fire_hook(&mut self, event: &str, args: &[&str]) {
        if let Some(func_name) = self.hooks.get(event).cloned() {
            let args_vec: Vec<String> = args.iter().map(|s| format!("\"{}\"", s.replace('"', "\\\""))).collect();
            let call = format!("{}({})", func_name, args_vec.join(", "));
            if let Err(e) = self.session.run(&call) {
                eprintln!("hook {}:{}: {}", event, func_name, e);
            }
        }
    }

    fn get_variable(&self, name: &str) -> Option<String> {
        // Special variables (Plan 304)
        match name {
            "?" => return Some(self.last_exit_code.to_string()),
            "_" => return self.last_command_line.clone(),
            "!" => return self.last_command_last_arg.clone(),
            "@" => return Some(self.last_command_args.join(" ")),
            "#" => return Some(self.last_command_args.len().to_string()),
            "PWD" => return Some(self.current_dir.to_string_lossy().to_string()),
            "OLDPWD" => return self.previous_dir.as_ref().map(|p| p.to_string_lossy().to_string()),
            _ => {}
        }

        // First check local shell variables
        if let Some(value) = self.vars.get_local(name) {
            return Some(value.clone());
        }

        // Then check environment variables
        self.vars.get_env(name)
    }

    /// Check if a name is a registered Auto function
    pub fn has_auto_function(&self, name: &str) -> bool {
        self.session.functions().contains(&name.to_string())
    }

    /// Get Auto function by name
    pub fn get_auto_function(&self, name: &str) -> Option<Value> {
        // Note: AutovmReplSession doesn't easily expose the Value itself yet,
        // but we can check if it exists. For now return Nil if it exists.
        if self.has_auto_function(name) {
            Some(Value::Nil)
        } else {
            None
        }
    }

    /// Execute Auto function with arguments
    pub fn execute_auto_function(
        &mut self,
        func_name: &str,
        args: Vec<String>,
    ) -> Result<Option<String>> {
        // Build Auto function call expression
        let args_str = args.join(", ");
        let call = format!("{}({})", func_name, args_str);

        self.execute_auto(&call)
    }

    /// Import a stdlib module
    fn import_module(&mut self, module: &str) -> Result<Option<String>> {
        // Try to import from stdlib
        let module_path = format!("use auto:{}:*", module);
        match self.session.run(&module_path) {
            Ok(_) => Ok(None),
            Err(e) => Err(miette::miette!("{}", e)),
        }
    }

    /// Execute variable management commands
    fn execute_var_command(&mut self, parts: &[&str]) -> Result<Option<String>> {
        if parts.is_empty() {
            return Ok(None);
        }

        match parts[0] {
            "set" => {
                if parts.len() < 2 {
                    miette::bail!("set: missing variable name");
                }

                // Parse variable assignment: set name=value or set name value
                let arg = parts[1];
                if let Some(eq_pos) = arg.find('=') {
                    let name = arg[..eq_pos].to_string();
                    let value = arg[eq_pos + 1..].to_string();
                    self.vars.set_local(name, value);
                } else if parts.len() >= 3 {
                    let name = parts[1].to_string();
                    let value = parts[2..].join(" ");
                    self.vars.set_local(name, value);
                } else {
                    let name = parts[1].to_string();
                    self.vars.set_local(name, String::new());
                }

                Ok(None)
            }
            "export" => {
                if parts.len() < 2 {
                    miette::bail!("export: missing variable name");
                }

                // Parse variable assignment: export name=value or export name value
                let arg = parts[1];
                if let Some(eq_pos) = arg.find('=') {
                    let name = arg[..eq_pos].to_string();
                    let value = arg[eq_pos + 1..].to_string();
                    self.vars.set_env(name, value);
                } else if parts.len() >= 3 {
                    let name = parts[1].to_string();
                    let value = parts[2..].join(" ");
                    self.vars.set_env(name, value);
                } else {
                    // Export existing local variable
                    let name = parts[1].to_string();
                    if let Some(value) = self.vars.get_local(&name) {
                        self.vars.set_env(name.clone(), value.clone());
                    }
                }

                Ok(None)
            }
            "unset" => {
                if parts.len() < 2 {
                    miette::bail!("unset: missing variable name");
                }

                let name = parts[1];
                // Try to unset as local variable first
                if self.vars.get_local(name).is_some() {
                    self.vars.unset_local(name);
                } else {
                    // Otherwise unset as environment variable
                    self.vars.unset_env(name);
                }

                Ok(None)
            }
            _ => Ok(None),
        }
    }

    /// Execute 'u' (up) command
    fn execute_up_command(&mut self, parts: &[&str]) -> Result<Option<String>> {
        let n = if parts.len() > 1 {
            parts[1].parse::<usize>().unwrap_or(1)
        } else {
            1
        };

        let mut target = String::new();
        for i in 0..n {
            if i > 0 {
                target.push('/');
            }
            target.push_str("..");
        }

        match self.cd(&target) {
            Ok(()) => Ok(None),
            Err(e) => Err(e),
        }
    }

    /// Execute 'b' (bookmark) command
    fn execute_bookmark_command(&mut self, parts: &[&str]) -> Result<Option<String>> {
        use miette::IntoDiagnostic;

        if parts.len() < 2 {
            // List bookmarks
            return self.list_bookmarks();
        }

        match parts[1] {
            "add" => {
                let name = if parts.len() >= 3 {
                    parts[2].to_string()
                } else {
                    // Default to current dir name if no name provided? User said "b add <name>"
                    miette::bail!("b add: missing bookmark name");
                };

                let path = self.pwd();
                self.bookmarks.add(name, path).into_diagnostic()?;
                Ok(None)
            }
            "del" => {
                if parts.len() < 3 {
                    miette::bail!("b del: missing bookmark name");
                }
                let name = parts[2];
                if self.bookmarks.del(name).into_diagnostic()? {
                    Ok(Some(format!("Deleted bookmark '{}'", name)))
                } else {
                    miette::bail!("Bookmark '{}' not found", name);
                }
            }
            "list" => self.list_bookmarks(),
            name => {
                // Jump to bookmark
                if let Some(path) = self.bookmarks.get(name) {
                    let path_str = path.to_string_lossy().to_string();
                    match self.cd(&path_str) {
                        Ok(()) => Ok(None),
                        Err(e) => Err(e),
                    }
                } else {
                    miette::bail!("Bookmark '{}' not found", name);
                }
            }
        }
    }

    fn list_bookmarks(&self) -> Result<Option<String>> {
        let bookmarks = self.bookmarks.list();
        if bookmarks.is_empty() {
            return Ok(Some("No bookmarks found.".to_string()));
        }

        let mut output = String::new();
        output.push_str("Bookmarks:\n");
        for (name, path) in bookmarks {
            output.push_str(&format!("  {:<15} {}\n", name, path.display()));
        }
        Ok(Some(output))
    }

    // ── Job control ──────────────────────────────────────

    /// Public accessor for the job manager (used by builtins).
    pub fn jobs_mut(&mut self) -> &mut JobManager {
        &mut self.jobs
    }

    /// Reap finished background jobs and print notifications.
    fn reap_jobs(&mut self) {
        let finished = self.jobs.reap_finished();
        for (id, cmd, code) in &finished {
            if *code == 0 {
                eprintln!("[{}]  Done    {}", id, cmd);
            } else {
                eprintln!("[{}]  Exit {} {}", id, code, cmd);
            }
        }
    }

    /// Execute a command in the background (`cmd &`).
    fn execute_background(&mut self, input: &str) -> Result<Option<String>> {
        use crate::cmd::external;

        let expanded = self.expand_variables(input);

        let child = external::spawn_external_background(&expanded, &self.current_dir)?;
        let id = self.jobs.add(expanded, child);
        eprintln!("[{}]  Running in background", id);
        Ok(None)
    }

    /// `jobs` builtin — list background/suspended jobs.
    fn cmd_jobs(&mut self) -> Result<Option<String>> {
        Ok(Some(self.jobs.format_jobs()))
    }

    /// `fg [N]` builtin — bring job N (or the most recent) to the foreground.
    fn cmd_fg(&mut self, job_id: Option<u32>) -> Result<Option<String>> {
        let id = job_id
            .or_else(|| self.jobs.last_job_id())
            .ok_or_else(|| miette::miette!("fg: no current job"))?;

        // If stopped, resume it first
        let job = self.jobs.get_mut(id)
            .ok_or_else(|| miette::miette!("fg: job {} not found", id))?;

        if job.state == crate::job::JobState::Stopped {
            crate::job::JobManager::resume_job(&mut self.jobs, id)?;
        }

        // Remove from job manager so we can take ownership of the Child
        let mut job = self.jobs.remove(id)
            .ok_or_else(|| miette::miette!("fg: job {} not found", id))?;

        let cmd_str = job.command.clone();
        eprintln!("[{}]  Foreground  {}", id, cmd_str);

        // Wait for it to finish
        let _guard = crate::signal::CtrlCGuard::new();
        match job.child.wait() {
            Ok(status) => {
                self.last_exit_code = status.code().unwrap_or(-1);
                if !status.success() {
                    // Don't error — just set exit code
                }
                Ok(None)
            }
            Err(e) => {
                self.last_exit_code = 1;
                Err(miette::miette!("fg: wait failed: {}", e))
            }
        }
    }

    /// `bg [N]` builtin — resume a stopped job in the background.
    fn cmd_bg(&mut self, job_id: Option<u32>) -> Result<Option<String>> {
        let id = job_id
            .or_else(|| {
                // Find most recent stopped job
                let mut last_stopped = None;
                // Access jobs directly since we can't iterate pub fields
                for (jid, job) in self.jobs.jobs_raw() {
                    if job.state == crate::job::JobState::Stopped {
                        last_stopped = Some(*jid);
                    }
                }
                last_stopped.or_else(|| self.jobs.last_job_id())
            })
            .ok_or_else(|| miette::miette!("bg: no current job"))?;

        self.jobs.resume_job(id)?;
        let job = self.jobs.get_mut(id).unwrap();
        eprintln!("[{}]  Running  {}", id, job.command);
        Ok(None)
    }

    /// `suspend` builtin — no-op hint. Real suspend happens via Ctrl+Z in the
    /// frontend (TODO: REPL wait-loop with waitpid WUNTRACED on Unix, console
    /// input monitoring on Windows). The suspend/resume infrastructure is in
    /// `job.rs` (`JobManager::suspend_job` / `resume_job`).
    fn cmd_suspend(&mut self) -> Result<Option<String>> {
        Ok(Some("Use Ctrl+Z to suspend a running foreground command.".to_string()))
    }
}

/// Extract the exit code from an external command error message.
///
/// External command errors follow the pattern `"... exit code: N"`.
/// If no code can be parsed, returns 1 (generic failure).
fn extract_exit_code(error_msg: &str) -> i32 {
    // Look for "exit code: <number>" anywhere in the message
    if let Some(pos) = error_msg.rfind("exit code: ") {
        let rest = &error_msg[pos + "exit code: ".len()..];
        if let Ok(code) = rest.trim().parse::<i32>() {
            return code;
        }
    }
    1
}

/// Parsed heredoc start: `command <<MARKER`
struct HeredocInfo {
    command: String,
    marker: String,
    strip_tabs: bool,
    expand_vars: bool,
}

/// Suggest a similar command name when the user types an unknown command.
///
/// Uses simple edit-distance heuristics to find the closest match among:
/// registered commands, legacy builtins, aliases, and common shell commands.
///
/// Returns `None` if no good suggestion is found.
fn suggest_command(shell: &Shell, name: &str) -> Option<String> {
    if name.is_empty() || name.len() < 2 {
        return None;
    }

    // Collect all known command names
    let mut candidates: Vec<&str> = Vec::new();

    // Registered commands
    for cmd_name in shell.registry.names() {
        candidates.push(cmd_name);
    }

    // Legacy builtins not in registry
    const EXTRA_BUILTINS: &[&str] = &[
        "pwd", "echo", "help", "clear", "ls", "mkdir", "rm", "mv", "cp",
        "sort", "uniq", "head", "tail", "wc", "grep", "count", "first",
        "last", "genlines", "set", "export", "unset", "alias", "unalias",
        "source", "pushd", "popd", "dirs", "jobs", "fg", "bg", "exit",
    ];
    for &b in EXTRA_BUILTINS {
        if !candidates.contains(&b) {
            candidates.push(b);
        }
    }

    // Aliases
    for alias_name in shell.aliases.keys() {
        candidates.push(alias_name.as_str());
    }

    // Find the best match using simple edit distance
    let mut best: Option<(&str, usize)> = None;
    let threshold = if name.len() <= 3 { 1 } else { 2 };

    for candidate in &candidates {
        let dist = levenshtein_distance(name, candidate);
        if dist <= threshold {
            match best {
                None => best = Some((*candidate, dist)),
                Some((_, best_dist)) if dist < best_dist => best = Some((*candidate, dist)),
                _ => {}
            }
        }
    }

    best.map(|(name, _)| name.to_string())
}

/// Simple Levenshtein edit distance for short strings.
fn levenshtein_distance(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let (n, m) = (a.len(), b.len());

    if n == 0 { return m; }
    if m == 0 { return n; }

    let mut prev: Vec<usize> = (0..=m).collect();
    let mut curr: Vec<usize> = vec![0; m + 1];

    for i in 1..=n {
        curr[0] = i;
        for j in 1..=m {
            let cost = if a[i - 1] == b[j - 1] { 0 } else { 1 };
            curr[j] = (prev[j] + 1)
                .min(curr[j - 1] + 1)
                .min(prev[j - 1] + cost);
        }
        std::mem::swap(&mut prev, &mut curr);
    }

    prev[m]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shell_creation() {
        let shell = Shell::new();
        assert!(shell.pwd().is_absolute());
    }

    #[test]
    fn test_looks_like_auto_expr() {
        let shell = Shell::new();

        // Should be recognized as Auto expressions
        assert!(shell.looks_like_auto_expr("1 + 2"));
        assert!(shell.looks_like_auto_expr("let x = 1"));
        assert!(shell.looks_like_auto_expr("fn add() {}"));
        assert!(shell.looks_like_auto_expr("\"hello\""));
        assert!(shell.looks_like_auto_expr("[1, 2, 3]"));
        assert!(shell.looks_like_auto_expr("{key: value}"));

        // Should NOT be recognized as Auto expressions
        assert!(!shell.looks_like_auto_expr("ls"));
        assert!(!shell.looks_like_auto_expr("cargo build"));
        assert!(!shell.looks_like_auto_expr("echo hello"));
    }

    #[test]
    fn test_variable_expansion_simple() {
        let mut shell = Shell::new();
        shell
            .vars
            .set_local("name".to_string(), "world".to_string());

        let expanded = shell.expand_variables("echo $name");
        assert_eq!(expanded, "echo world");
    }

    #[test]
    fn test_variable_expansion_braced() {
        let mut shell = Shell::new();
        shell
            .vars
            .set_local("name".to_string(), "world".to_string());

        let expanded = shell.expand_variables("echo ${name}");
        assert_eq!(expanded, "echo world");
    }

    #[test]
    fn test_variable_expansion_multiple() {
        let mut shell = Shell::new();
        shell.vars.set_local("a".to_string(), "1".to_string());
        shell.vars.set_local("b".to_string(), "2".to_string());

        let expanded = shell.expand_variables("$a$b");
        assert_eq!(expanded, "12");
    }

    #[test]
    fn test_variable_expansion_in_middle() {
        let mut shell = Shell::new();
        shell
            .vars
            .set_local("name".to_string(), "world".to_string());

        let expanded = shell.expand_variables("hello $name!");
        assert_eq!(expanded, "hello world!");
    }

    #[test]
    fn test_variable_expansion_undefined() {
        let shell = Shell::new();
        let expanded = shell.expand_variables("echo $undefined");
        // Undefined variables should expand to empty string
        assert_eq!(expanded, "echo ");
    }

    #[test]
    fn test_set_command_equals() {
        let mut shell = Shell::new();
        shell.execute("set name=world").unwrap();

        assert_eq!(shell.vars.get_local("name"), Some(&"world".to_string()));
    }

    #[test]
    fn test_set_command_space() {
        let mut shell = Shell::new();
        shell.execute("set name world").unwrap();

        assert_eq!(shell.vars.get_local("name"), Some(&"world".to_string()));
    }

    #[test]
    fn test_export_command() {
        let mut shell = Shell::new();
        shell.execute("export MYVAR=test").unwrap();

        assert_eq!(shell.vars.get_env("MYVAR"), Some("test".to_string()));
    }

    #[test]
    fn test_unset_local() {
        let mut shell = Shell::new();
        shell
            .vars
            .set_local("name".to_string(), "value".to_string());
        assert!(shell.vars.get_local("name").is_some());

        shell.execute("unset name").unwrap();
        assert!(shell.vars.get_local("name").is_none());
    }

    #[test]
    fn test_variable_expansion_with_pipeline() {
        let mut shell = Shell::new();
        shell
            .vars
            .set_local("pattern".to_string(), "hello".to_string());

        let input = "genlines hello world | grep $pattern";
        let expanded = shell.expand_variables(input);
        assert_eq!(expanded, "genlines hello world | grep hello");
    }

    #[test]
    fn test_auto_expression_execution() {
        let mut shell = Shell::new();

        // Test basic arithmetic
        let result = shell.execute("1 + 2");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some("3".to_string()));

        // Test array literals - format_last_result now handles arrays
        let result = shell.execute("[1, 2, 3]");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some("[1, 2, 3]".to_string()));

        // Test object literals
        let result = shell.execute("{key: \"value\"}");
        assert!(result.is_ok());
    }

    #[test]
    fn test_auto_persistent_interpreter() {
        let mut shell = Shell::new();

        // Test that the interpreter persists across commands
        // We'll test with expressions that use the same scope
        let result1 = shell.execute("41 + 1");
        assert!(result1.is_ok());
        assert_eq!(result1.unwrap(), Some("42".to_string()));

        // Another expression should work in the same interpreter
        let result2 = shell.execute("10 * 5");
        assert!(result2.is_ok());
        assert_eq!(result2.unwrap(), Some("50".to_string()));
    }

    #[test]
    fn test_execute_auto_function_formatting() {
        let mut shell = Shell::new();

        // Test that function calls are formatted correctly
        let result = shell.execute_auto_function("test", vec!["1".to_string(), "2".to_string()]);
        // Will fail because function doesn't exist, but tests formatting
        assert!(result.is_err());
    }

    #[test]
    fn test_auto_function_call_detection() {
        let mut shell = Shell::new();

        // Even though the function doesn't exist, the call syntax should be recognized
        // We test this by checking that it doesn't try to execute it as an external command
        // We'll get a different error if it's parsed as Auto vs external
        let result = shell.execute("nonexistent_func(1, 2)");
        // Should be an error (function doesn't exist) but not "program not found"
        assert!(result.is_err());
    }

    #[test]
    fn test_exit_code_success() {
        let mut shell = Shell::new();
        let _ = shell.execute("1 + 2");
        assert_eq!(shell.last_exit_code(), 0);
    }

    #[test]
    fn test_exit_code_failure() {
        let mut shell = Shell::new();
        let _ = shell.execute("nonexistent_command_xyz");
        assert_ne!(shell.last_exit_code(), 0);
    }

    #[test]
    fn test_exit_code_variable() {
        let mut shell = Shell::new();
        let _ = shell.execute("1 + 2");
        assert_eq!(shell.last_exit_code(), 0);

        // $? should expand to "0"
        let expanded = shell.expand_variables("exit code was $?");
        assert_eq!(expanded, "exit code was 0");

        // After a failure, $? should be non-zero
        let _ = shell.execute("nonexistent_command_xyz");
        let expanded = shell.expand_variables("code: $?");
        assert_eq!(expanded, "code: 1");
    }

    #[test]
    fn test_extract_exit_code() {
        assert_eq!(extract_exit_code("Command failed with exit code: 42"), 42);
        assert_eq!(
            extract_exit_code("PowerShell command failed with exit code: 7"),
            7
        );
        assert_eq!(extract_exit_code("Command failed: something"), 1);
        assert_eq!(extract_exit_code("generic error"), 1);
    }

    // -----------------------------------------------------------------------
    // Plan 302 Step 2.4: Command substitution tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_convert_backticks_basic() {
        assert_eq!(Shell::convert_backticks("echo `whoami`"), "echo $(whoami)");
    }

    #[test]
    fn test_convert_backticks_multiple() {
        assert_eq!(
            Shell::convert_backticks("echo `a` and `b`"),
            "echo $(a) and $(b)"
        );
    }

    #[test]
    fn test_convert_backticks_in_single_quotes() {
        // Backticks inside single quotes should NOT be converted
        assert_eq!(
            Shell::convert_backticks("echo '`whoami`'"),
            "echo '`whoami`'"
        );
    }

    #[test]
    fn test_convert_backticks_no_backticks() {
        assert_eq!(Shell::convert_backticks("echo hello"), "echo hello");
    }

    #[test]
    fn test_command_substitution_pwd() {
        let mut shell = Shell::new();
        let result = shell.expand_command_substitution("echo $(pwd)").unwrap();
        // Should have replaced $(pwd) with actual path
        assert!(!result.contains("$("));
        assert!(result.starts_with("echo "));
        // The path should be absolute
        let path = &result[5..]; // after "echo "
        assert!(path.starts_with('/') || path.len() > 1);
    }

    #[test]
    fn test_command_substitution_no_subst() {
        let mut shell = Shell::new();
        let result = shell.expand_command_substitution("echo hello").unwrap();
        assert_eq!(result, "echo hello");
    }

    #[test]
    fn test_command_substitution_single_quote_no_expand() {
        let mut shell = Shell::new();
        let result = shell.expand_command_substitution("echo '$(pwd)'").unwrap();
        // $() inside single quotes should NOT be expanded
        assert_eq!(result, "echo '$(pwd)'");
    }

    #[test]
    fn test_command_substitution_trailing_newline_stripped() {
        let mut shell = Shell::new();
        // echo test produces "test\n", the trailing \n should be stripped
        let result = shell.expand_command_substitution("msg=$(echo test)").unwrap();
        assert_eq!(result, "msg=test");
    }

    #[test]
    fn test_command_substitution_multiple() {
        let mut shell = Shell::new();
        let result = shell.expand_command_substitution("echo $(pwd) and $(whoami)").unwrap();
        assert!(!result.contains("$("));
    }

    // ── Plan 309 Task 1.2 P4 — env persistence ──────────────────────────

    /// Unique temp path for a persistence test (per-test suffix avoids parallel races).
    fn env_persist_test_path(suffix: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!("ash_env_persist_{suffix}.at"))
    }

    #[test]
    fn test_env_persist_upsert_replaces_and_appends() {
        let path = env_persist_test_path("upsert");
        let _ = std::fs::remove_file(&path);

        Shell::env_persist_upsert_at(&path, "EDITOR", "vim").unwrap();
        Shell::env_persist_upsert_at(&path, "LANG", "en_US").unwrap();
        // Upserting EDITOR again replaces, not duplicates.
        Shell::env_persist_upsert_at(&path, "EDITOR", "nano").unwrap();

        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("env EDITOR=nano"));
        assert!(content.contains("env LANG=en_US"));
        assert!(!content.contains("vim"), "stale value not replaced:\n{content}");
        // Exactly one EDITOR line.
        assert_eq!(content.matches("env EDITOR=").count(), 1);

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_env_persist_remove_drops_only_named() {
        let path = env_persist_test_path("remove");
        let _ = std::fs::remove_file(&path);
        Shell::env_persist_upsert_at(&path, "KEEP_ME", "1").unwrap();
        Shell::env_persist_upsert_at(&path, "DROP_ME", "2").unwrap();

        Shell::env_persist_remove_at(&path, "DROP_ME");

        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("env KEEP_ME=1"));
        assert!(!content.contains("DROP_ME"), "named var not removed:\n{content}");

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_env_load_persistence_executes_lines() {
        let path = env_persist_test_path("load");
        std::fs::write(&path, "// comment\nenv ASH_TEST_PERSIST=hello\n\nenv ASH_TEST_PERSIST2=world\n")
            .unwrap();

        let mut shell = Shell::new();
        shell.load_env_persistence_from(&path);

        assert_eq!(shell.vars.get_env("ASH_TEST_PERSIST"), Some("hello".to_string()));
        assert_eq!(shell.vars.get_env("ASH_TEST_PERSIST2"), Some("world".to_string()));

        let _ = std::fs::remove_file(&path);
    }
}

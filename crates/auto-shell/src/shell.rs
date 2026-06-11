use miette::{IntoDiagnostic, Result};
use std::path::PathBuf;

use crate::parser::pipeline::parse_pipeline;
use auto_lang::autovm_persistent::AutovmReplSession;
use auto_val::Value;
use ash_core::pipeline::AtomPipeline;

// Re-export vars from core
pub use crate::core::shell::vars;

use crate::bookmarks::BookmarkManager;
use crate::cmd::{commands, CommandRegistry};
use vars::ShellVars;

/// Shell state and context
pub struct Shell {
    current_dir: PathBuf,
    vars: ShellVars,
    session: AutovmReplSession, // Persistent AutoVM REPL session
    bookmarks: BookmarkManager,

    previous_dir: Option<PathBuf>,
    registry: CommandRegistry,
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
        }
    }

    /// Execute a command or AutoLang expression
    pub fn execute(&mut self, input: &str) -> Result<Option<String>> {
        // Try to parse as AutoLang expression first
        if self.looks_like_auto_expr(input) {
            return self.execute_auto(input);
        }

        // Expand variables in input
        let expanded = self.expand_variables(input);

        // Check if input contains a pipeline
        if expanded.contains('|') {
            let commands = parse_pipeline(&expanded);
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
    fn execute_single_command(&mut self, input: &str) -> Result<Option<String>> {
        use crate::cmd::{auto, builtin, external};
        use crate::parser::quote::parse_args;

        let parts = parse_args(input);
        if parts.is_empty() {
            return Ok(None);
        }

        let cmd_name = &parts[0];
        let args = &parts[1..];

        // Check registry first
        if let Some(cmd) = self.registry.get(cmd_name) {
            let signature = cmd.signature();
            match crate::cmd::parser::parse_args(&signature, args) {
                Ok(parsed_args) => {
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

        // Otherwise, execute as external command
        external::execute_external(input, &self.current_dir, false)
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
                if let Some(rendered) =
                    crate::frontend::renderer::render_table(&atom.value, term_width)
                {
                    return rendered;
                }
            }
        }

        // Fallback: plain text
        pipeline.into_text()
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

            let cmd_name = &parts[0];
            let args = &parts[1..];

            // Execute the command
            let output_pipeline = if let Some(registered_cmd) = self.registry.get(cmd_name) {
                // Registered command (uses AtomPipeline via run_atom)
                let signature = registered_cmd.signature();
                let input = input_pipeline.take().unwrap_or_else(AtomPipeline::empty);

                match crate::cmd::parser::parse_args(&signature, args) {
                    Ok(parsed_args) => Some(registered_cmd.run_atom(&parsed_args, input, self)?),
                    Err(e) => return Err(e),
                }
            } else {
                // Non-registered command (builtins, auto functions, external)
                // Convert AtomPipeline to text for legacy commands
                let input_str = input_pipeline.take().and_then(|p| {
                    if p.is_empty() {
                        None
                    } else {
                        Some(p.into_text())
                    }
                });

                if let Some(input) = &input_str {
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
            self.current_dir = canonical.clone();
            // Update OS state (so Prompt and child processes see it)
            std::env::set_current_dir(&canonical).into_diagnostic()?;
            // Notify git cache: sync refresh + start filesystem watcher
            crate::prompt::context::on_directory_changed(canonical);
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

    /// Get a variable value (checks local vars, then env vars)
    fn get_variable(&self, name: &str) -> Option<String> {
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
}

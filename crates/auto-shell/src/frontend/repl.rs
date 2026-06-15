use miette::Result;
use reedline::{
    default_emacs_keybindings, default_vi_insert_keybindings, default_vi_normal_keybindings,
    CwdAwareHinter, EditCommand, Emacs, FileBackedHistory,
    KeyCode, KeyModifiers, Reedline, ReedlineEvent, ReedlineMenu, Signal, Vi,
};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::menu::{AshMenu, AshMenuConfig};
use crate::completions::CompletionSignature;
use crate::completions::definitions;
use crate::completions::reedline::{CompletionState, ShellCompleter};
use ash_core::completions::CompletionProvider;
use crate::{prompt::AshPrompt, shell::Shell};
use crate::frontend::term::highlight::AshHighlighter;

/// Read-Eval-Print Loop for AutoShell
pub struct Repl {
    shell: Shell,
    line_editor: Reedline,
    prompt: AshPrompt,
    /// Shared completion state — updated after each cd/pushd/etc.
    completion_state: Arc<Mutex<CompletionState>>,
}

impl Repl {
    /// Create a new REPL instance
    pub fn new() -> Result<Self> {
        let mut shell = Shell::new();

        // Plan 302 Step 4.2: Load ~/.config/ash.toml
        let shell_config = crate::config::AshShellConfig::load();

        // Apply aliases from config
        for (name, value) in &shell_config.aliases {
            shell.set_alias(name, value);
        }

        // Plan 302 Step 1.3: Load ~/.ashrc if it exists (can override config aliases)
        if let Some(home) = dirs::home_dir() {
            let rc_path = home.join(".ashrc");
            if rc_path.exists() {
                let _ = shell.source_file(&rc_path); // silently ignore errors
            }
        }

        // Set up history file (configurable size)
        let history_path = Self::get_history_path()?;
        let history = Box::new(
            FileBackedHistory::with_file(shell_config.history_size, history_path)
                .map_err(|e| miette::miette!("Failed to create history: {}", e))?,
        );

        // Create completer for Tab completion (with registry signatures)
        let completion_sigs: Vec<CompletionSignature> =
            shell.registry().params().into_iter().map(Into::into).collect();

        // Create CompletionProvider and register external command definitions
        let mut provider = CompletionProvider::new();
        definitions::register_all(&mut provider);

        // Shared state for completion (cwd, etc.)
        let completion_state = Arc::new(Mutex::new(CompletionState::new(shell.pwd().to_path_buf())));

        let completer = Box::new(ShellCompleter::new(
            completion_sigs,
            provider,
            Arc::clone(&completion_state),
        ));

        // Use AshMenu (adaptive completion menu replacing ColumnarMenu)
        let completion_menu = Box::new(AshMenu::new(AshMenuConfig {
            name: "completion_menu".to_string(),
            ..Default::default()
        }));

        // History candidate menu — a separate list sourced from history
        // (NOT the command-based Tab completions). Bound to Ctrl+R. This is
        // the fzf-history style "popup of all matching history entries".
        let history_menu = Box::new(AshMenu::new(AshMenuConfig {
            name: "history_menu".to_string(),
            ..Default::default()
        }));

        // Plan 302 Step 3.2: Determine edit mode (Vi or Emacs)
        // Priority: $ASH_EDIT_MODE env var > ash.toml edit_mode > ~/.ashrc
        let use_vi = std::env::var("ASH_EDIT_MODE").map(|v| v == "vi").unwrap_or_else(|_| {
            if shell_config.is_vi_mode() {
                return true;
            }
            // Fallback: check ~/.ashrc for "set editing-mode vi"
            if let Some(home) = dirs::home_dir() {
                let rc = home.join(".ashrc");
                if let Ok(content) = std::fs::read_to_string(&rc) {
                    return content.lines().any(|line| {
                        let line = line.trim();
                        line == "set editing-mode vi"
                    });
                }
            }
            false
        });

        // Helper to add common keybindings to any keybindings object
        fn add_common_keybindings(keybindings: &mut reedline::Keybindings) {
            keybindings.add_binding(
                KeyModifiers::NONE,
                KeyCode::Tab,
                ReedlineEvent::Multiple(vec![
                    ReedlineEvent::UntilFound(vec![
                        ReedlineEvent::Menu("completion_menu".to_string()),
                        ReedlineEvent::MenuNext,
                        ReedlineEvent::Edit(vec![EditCommand::Complete]),
                    ]),
                    ReedlineEvent::Repaint,
                ]),
            );
            // Plan 302: Ctrl+F accepts the full autosuggestion hint (Fish-style).
            // NOTE: must be `HistoryHintComplete`, NOT `EditCommand::Complete` —
            // the latter triggers the completion *menu* and never accepts a hint.
            keybindings.add_binding(
                KeyModifiers::CONTROL,
                KeyCode::Char('f'),
                ReedlineEvent::HistoryHintComplete,
            );
            // Plan 302 Step 3.4: Ctrl+→ accepts next word of autosuggestion
            keybindings.add_binding(
                KeyModifiers::CONTROL,
                KeyCode::Right,
                ReedlineEvent::HistoryHintWordComplete,
            );
            // Ctrl+R — pop up the history candidate menu (fzf-history style).
            // Separate from Tab (command-based) completions: this lists matching
            // entries from shell history. Supersedes the old inline SearchHistory.
            keybindings.add_binding(
                KeyModifiers::CONTROL,
                KeyCode::Char('r'),
                ReedlineEvent::Menu("history_menu".to_string()),
            );
            // Ctrl+S — forward history search (legacy inline search retained as a
            // non-popup fallback alongside the Ctrl+R menu).
            keybindings.add_binding(
                KeyModifiers::CONTROL,
                KeyCode::Char('s'),
                ReedlineEvent::SearchHistory,
            );
            // Plan 304: Ctrl+E — edit in $EDITOR (sends \x05 prefix)
            keybindings.add_binding(
                KeyModifiers::CONTROL,
                KeyCode::Char('e'),
                ReedlineEvent::Edit(vec![EditCommand::InsertString("\x05".to_string())]),
            );
        }

        let edit_mode: Box<dyn reedline::EditMode> = if use_vi {
            let mut insert_kb = default_vi_insert_keybindings();
            let normal_kb = default_vi_normal_keybindings();
            add_common_keybindings(&mut insert_kb);
            Box::new(Vi::new(insert_kb, normal_kb))
        } else {
            let mut keybindings = default_emacs_keybindings();
            add_common_keybindings(&mut keybindings);
            Box::new(Emacs::new(keybindings))
        };

        // Create modular prompt (AshPrompt)
        let prompt = AshPrompt::new(crate::prompt::AshConfig::load());

        // Plan 302: Fish-style autosuggestion hinter (configurable)
        // CwdAwareHinter prefers history items from the current working directory
        let hinter: Option<Box<CwdAwareHinter>> = if shell_config.autosuggestion {
            // Plan 302: Fish-style autosuggestion hinter.
            // Explicit dim style so the hint is clearly distinguishable from typed
            // text — reedline's default `LightGray` is too close to the terminal's
            // default foreground on Windows and reads as normal text.
            let hint_style = nu_ansi_term::Style::new()
                .fg(nu_ansi_term::Color::DarkGray)
                .italic();
            Some(Box::new(
                CwdAwareHinter::default()
                    .with_style(hint_style)
                    .with_min_chars(shell_config.autosuggestion_min_chars),
            ))
        } else {
            None
        };

        // Plan 302 Step 3.1: Syntax highlighting (configurable)
        let highlighter: Option<Box<AshHighlighter>> = if shell_config.syntax_highlighting {
            Some(Box::new(AshHighlighter::new()))
        } else {
            None
        };

        let mut line_editor = Reedline::create()
            .with_history(history)
            .with_completer(completer)
            .with_menu(ReedlineMenu::EngineCompleter(completion_menu))
            .with_menu(ReedlineMenu::HistoryMenu(history_menu))
            .with_quick_completions(true)
            .with_partial_completions(true)
            .with_edit_mode(edit_mode);

        if let Some(h) = highlighter {
            line_editor = line_editor.with_highlighter(h);
        }
        if let Some(h) = hinter {
            line_editor = line_editor.with_hinter(h);
        }

        Ok(Self { shell, line_editor, prompt, completion_state })
    }

    /// Get the path to the history file
    fn get_history_path() -> Result<PathBuf> {
        let mut history_path = dirs::home_dir()
            .ok_or_else(|| miette::miette!("Could not determine home directory"))?;

        history_path.push(".auto-shell-history");
        Ok(history_path)
    }

    /// Expand history references in the input line
    ///
    /// Supports: `!!` (last), `!n` (by number), `!-n` (relative),
    /// `!string` (prefix search), `!?string` (contains search).
    ///
    /// Returns Ok(true) if expansion occurred, Ok(false) if no expansion needed
    fn expand_line_history(&mut self, line: &mut String) -> Result<bool> {
        // Check if line contains history expansion character
        if !line.contains('!') {
            return Ok(false);
        }

        // Read history from file (reedline doesn't expose history via API)
        let history_path = Self::get_history_path()?;
        let history_strings = read_history_file(&history_path);

        if history_strings.is_empty() {
            return Ok(false);
        }

        struct FileHistory {
            strings: Vec<String>,
        }
        impl ash_core::parser::history::History for FileHistory {
            fn search(&self, _query: Option<&str>) -> Vec<String> {
                self.strings.clone()
            }
        }

        let file_history = FileHistory { strings: history_strings };
        let expanded = ash_core::parser::history::expand_history(line, &file_history)?;

        if &expanded != line {
            *line = expanded;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Update the shared completion state with the current working directory.
    fn sync_completion_state(&self) {
        if let Ok(mut state) = self.completion_state.lock() {
            state.current_dir = self.shell.pwd().to_path_buf();
        }
    }

    /// Open the current input line in $EDITOR (or vim/notepad) and return the result.
    /// Plan 304: Multi-line edit via Ctrl+E.
    fn edit_in_editor(&self, initial_content: &str) -> Result<String> {
        let tmp_dir = std::env::temp_dir();
        let tmp_file = tmp_dir.join("ash_edit_buffer.txt");
        std::fs::write(&tmp_file, initial_content)
            .map_err(|e| miette::miette!("editor: failed to write temp file: {}", e))?;

        // Determine editor: $VISUAL > $EDITOR > platform default
        let editor = std::env::var("VISUAL")
            .or_else(|_| std::env::var("EDITOR"))
            .unwrap_or_else(|_| {
                if cfg!(windows) { "notepad".to_string() } else { "vim".to_string() }
            });

        // Parse editor command (may have args like "code --wait")
        let parts: Vec<&str> = editor.split_whitespace().collect();
        let (cmd, extra_args) = match parts.split_first() {
            Some((c, args)) => (*c, args.to_vec()),
            None => ("vim", vec![]),
        };

        let mut command = std::process::Command::new(cmd);
        command.args(&extra_args).arg(&tmp_file);

        // Inherit terminal for the editor
        let status = command.status()
            .map_err(|e| miette::miette!("editor: failed to launch '{}': {}", cmd, e))?;

        if !status.success() {
            return Err(miette::miette!("editor: exited with status {}", status));
        }

        let content = std::fs::read_to_string(&tmp_file)
            .map_err(|e| miette::miette!("editor: failed to read temp file: {}", e))?;

        // Clean up temp file
        let _ = std::fs::remove_file(&tmp_file);

        Ok(content.trim().to_string())
    }

    /// Run the REPL loop
    pub fn run(&mut self) -> Result<()> {
        // One-time Ctrl+C handler init (protects ASH during commands)
        crate::signal::init();

        // Initial git cache: sync refresh + start filesystem watcher for cwd
        crate::prompt::context::on_directory_changed(self.shell.pwd());

        loop {
            // Read input
            let sig = self.line_editor.read_line(&self.prompt);

            match sig {
                Ok(Signal::Success(line)) => {
                    let mut line = line.trim().to_string();

                    // Plan 304: Ctrl+E — open line in editor
                    // If line starts with "\x05" (Ctrl+E character), edit in $EDITOR
                    if line.starts_with('\x05') {
                        line = line[1..].trim().to_string();
                        line = match self.edit_in_editor(&line) {
                            Ok(edited) => edited,
                            Err(e) => {
                                eprintln!("editor: {}", e);
                                continue;
                            }
                        };
                        if line.is_empty() {
                            continue;
                        }
                        println!("{}", line); // show edited command
                    }

                    // Plan 302 Step 2.3: Multi-line input handling
                    // Detect trailing backslash or unclosed quotes, then read continuation lines
                    loop {
                        let trimmed = line.trim_end_matches(' ');
                        if trimmed.ends_with('\\') && !trimmed.ends_with("\\\\") {
                            // Trailing backslash continuation
                            line = trimmed.to_string();
                            line.truncate(line.len() - 1); // remove the \
                            line.push(' ');
                            let cont = self.line_editor.read_line(&self.prompt);
                            match cont {
                                Ok(Signal::Success(next)) => {
                                    line.push_str(next.trim());
                                }
                                Ok(Signal::CtrlD) => break, // Ctrl-D accepts what we have
                                _ => break,
                            }
                        } else if has_unclosed_quote(&line) {
                            // Unclosed quote — read continuation
                            line.push('\n');
                            let cont = self.line_editor.read_line(&self.prompt);
                            match cont {
                                Ok(Signal::Success(next)) => {
                                    line.push_str(&next);
                                }
                                Ok(Signal::CtrlD) => break,
                                _ => break,
                            }
                        } else {
                            break;
                        }
                    }

                    // Skip empty lines
                    if line.is_empty() {
                        continue;
                    }

                    // Plan 304: Expand abbreviations (abbr) in-line
                    let (expanded, was_expanded) = self.shell.expand_abbreviations(&line);
                    if was_expanded {
                        println!("{}", expanded); // show the expanded form
                        line = expanded;
                    }

                    // Expand history references (!!, !n, etc.)
                    match self.expand_line_history(&mut line) {
                        Ok(true) => {
                            // History was expanded, show the expanded command
                            println!("{}", line);
                        }
                        Ok(false) => {
                            // No history expansion needed
                        }
                        Err(e) => {
                            eprintln!("History expansion error: {}", e);
                            continue;
                        }
                    }

                    // Handle exit command
                    if line == "exit" || line == "quit" || line == "q" {
                        println!("Goodbye!");
                        break;
                    }

                    // Handle interactive commands (vim, less, ssh, etc.)
                    // These need full terminal control — just execute directly
                    // with inherited stdio, bypassing the shell's pipeline system.
                    if ash_core::cmd::interactive::is_interactive_command(&line) {
                        let result = ash_core::cmd::external::execute_external(
                            &line,
                            &self.shell.pwd(),
                            false, // inherit stdio, not capture
                        );
                        if let Err(e) = result {
                            eprintln!("Error: {}", e);
                        }
                        // Refresh git info after interactive command
                        crate::prompt::context::refresh_git_info_async(
                            self.shell.pwd(),
                        );
                        self.sync_completion_state();
                        continue;
                    }

                    // Evaluate the line
                    match self.shell.execute(&line) {
                        Ok(output) => {
                            if let Some(s) = output {
                                println!("{}", s);
                            }
                            // After command execution, async-refresh git cache
                            // (most changes are caught by filesystem watcher,
                            //  but this covers edge cases like external git commands)
                            crate::prompt::context::refresh_git_info_async(
                                self.shell.pwd(),
                            );
                        }
                        Err(e) => {
                            eprintln!("Error: {}", e);
                        }
                    }

                    // Sync completion state (cwd may have changed after cd/pushd)
                    self.sync_completion_state();
                }
                Ok(Signal::CtrlD) => {
                    println!();
                    println!("Goodbye!");
                    break;
                }
                Ok(Signal::CtrlC) => {
                    // User pressed Ctrl+C, just show new prompt
                    continue;
                }
                Err(err) => {
                    eprintln!("Error: {}", err);
                    continue;
                }
            }
        }

        Ok(())
    }
}

/// Check if a line has unclosed quotes (single or double).
///
/// Counts quote characters outside of the other quote type.
/// An odd count means the quote is unclosed.
fn has_unclosed_quote(line: &str) -> bool {
    let mut single_count = 0;
    let mut double_count = 0;
    let mut chars = line.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '\\' => {
                // Skip escaped character
                chars.next();
            }
            '\'' if double_count % 2 == 0 => {
                single_count += 1;
            }
            '"' if single_count % 2 == 0 => {
                double_count += 1;
            }
            _ => {}
        }
    }

    single_count % 2 != 0 || double_count % 2 != 0
}

/// Read history entries from the reedline FileBackedHistory file.
///
/// The file format is simple: one command per line. Blank lines are skipped.
/// We deduplicate by keeping only the most recent occurrence of each command
/// (matches what users expect from `!!` — the last time they ran it).
fn read_history_file(path: &std::path::Path) -> Vec<String> {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    content
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.to_string())
        .collect()
}

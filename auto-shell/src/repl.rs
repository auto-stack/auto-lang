use miette::Result;
use reedline::{
    DefaultPrompt, DefaultPromptSegment, Reedline, Signal,
    FileBackedHistory,
};
use std::path::PathBuf;

use crate::shell::Shell;
use crate::parser::expand_history;

/// Mock history wrapper for expansion
struct MockHistory {
    strings: Vec<String>,
}

impl MockHistory {
    fn new(strings: Vec<String>) -> Self {
        Self { strings }
    }
}

impl crate::parser::History for MockHistory {
    fn search(&self, _query: Option<&str>) -> Vec<String> {
        self.strings.clone()
    }
}

/// Read-Eval-Print Loop for AutoShell
pub struct Repl {
    shell: Shell,
    line_editor: Reedline,
}

impl Repl {
    /// Create a new REPL instance
    pub fn new() -> Result<Self> {
        let shell = Shell::new();

        // Set up history file
        let history_path = Self::get_history_path()?;
        let history = Box::new(
            FileBackedHistory::with_file(1000, history_path)
                .map_err(|e| miette::miette!("Failed to create history: {}", e))?
        );
        let line_editor = Reedline::create()
            .with_history(history);

        Ok(Self { shell, line_editor })
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
    /// Returns Ok(true) if expansion occurred, Ok(false) if no expansion needed
    fn expand_line_history(&mut self, line: &mut String) -> Result<bool> {
        // Check if line contains history expansion character
        if !line.contains('!') {
            return Ok(false);
        }

        // Get history from reedline - use a simpler approach
        // We'll skip history expansion for now since reedline's API is complex
        // TODO: Implement proper history expansion once we understand reedline better
        Ok(false)
    }

    /// Run the REPL loop
    pub fn run(&mut self) -> Result<()> {
        let prompt = DefaultPrompt::new(
            DefaultPromptSegment::Empty,
            DefaultPromptSegment::Empty,
        );

        loop {
            // Read input
            let sig = self.line_editor.read_line(&prompt);

            match sig {
                Ok(Signal::Success(line)) => {
                    let mut line = line.trim().to_string();

                    // Skip empty lines
                    if line.is_empty() {
                        continue;
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
                    if line == "exit" || line == "quit" {
                        println!("Goodbye!");
                        break;
                    }

                    // Evaluate the line
                    match self.shell.execute(&line) {
                        Ok(output) => {
                            if let Some(s) = output {
                                println!("{}", s);
                            }
                        }
                        Err(e) => {
                            eprintln!("Error: {}", e);
                        }
                    }
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

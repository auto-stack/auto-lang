// BigVM REPL: Simple REPL using BigVM instead of TreeWalker Interpreter
//
// **Plan 068 Phase 9.5**: Default REPL now uses BigVM for better performance
//
// This is a simplified REPL that:
// - Uses BigVM for code execution (not Interpreter/Evaler)
// - Supports basic REPL commands (:help, :quit, :reset)
// - Does NOT maintain persistent scope across inputs (yet)
// - Can be compared against old-repl for validation

use crate::error::{AutoError, AutoResult};
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;

/// BigVM REPL session
///
/// This REPL uses BigVM (bytecode VM) for execution instead of
/// the deprecated TreeWalker Interpreter.
pub struct BigvmRepl {
    /// REPL history
    history_path: Option<String>,
}

impl BigvmRepl {
    /// Create a new BigVM REPL session
    pub fn new() -> Self {
        Self {
            history_path: None,
        }
    }

    /// Run the REPL main loop
    pub fn run(&mut self) -> AutoResult<()> {
        let mut editor = DefaultEditor::new().map_err(|e| {
            AutoError::Msg(format!("Failed to initialize REPL: {}", e))
        })?;

        // Load history if available
        if let Some(ref path) = self.history_path {
            let _ = editor.load_history(path);
        }

        println!("🟢 BigVM REPL (Plan 068 Phase 9)");
        println!("Type ':help' for commands, ':quit' to exit");
        println!();

        loop {
            let readline = editor.readline("BigVM> ");

            match readline {
                Ok(line) => {
                    let line = line.trim();

                    // Skip empty lines
                    if line.is_empty() {
                        continue;
                    }

                    // Add to history
                    editor.add_history_entry(line).ok();

                    // Handle REPL commands
                    if let Some(cmd) = Self::try_command(line, self) {
                        match cmd {
                            ReplCommand::Exit => break,
                            ReplCommand::Continue => continue,
                        }
                    }

                    // Execute code using BigVM
                    match crate::run_bigvm(line) {
                        Ok(result) => {
                            if !result.is_empty() {
                                println!("{}", result);
                            }
                        }
                        Err(e) => {
                            eprintln!("Error: {}", e);
                        }
                    }
                }
                Err(ReadlineError::Interrupted) => {
                    println!("Use ':quit' to exit");
                }
                Err(ReadlineError::Eof) => {
                    println!("Goodbye!");
                    break;
                }
                Err(err) => {
                    eprintln!("Error: {}", err);
                    break;
                }
            }
        }

        // Save history
        if let Some(ref path) = self.history_path {
            let _ = editor.save_history(path);
        }

        Ok(())
    }

    /// Try to handle a REPL command (starting with ':')
    fn try_command(line: &str, _repl: &mut BigvmRepl) -> Option<ReplCommand> {
        if !line.starts_with(':') {
            return None;
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            return None;
        }

        let cmd = parts[0];

        match cmd {
            ":help" => {
                println!("=== BigVM REPL Commands ===");
                println!("  :help     - Show this help message");
                println!("  :quit     - Exit the REPL");
                println!();
                println!("Note: This REPL uses BigVM (fast bytecode VM)");
                println!("For comparison, use 'auto.exe old-repl' to use the legacy Interpreter");
                Some(ReplCommand::Continue)
            }
            ":quit" | ":exit" | ":q" => {
                println!("Goodbye!");
                Some(ReplCommand::Exit)
            }
            _ => {
                println!("Unknown command: {}. Type ':help' for available commands.", cmd);
                Some(ReplCommand::Continue)
            }
        }
    }
}

impl Default for BigvmRepl {
    fn default() -> Self {
        Self::new()
    }
}

enum ReplCommand {
    Exit,
    Continue,
}

/// Main entry point for BigVM REPL
pub fn main_loop() -> AutoResult<()> {
    let mut repl = BigvmRepl::new();
    repl.run()
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bigvm_repl_create() {
        let repl = BigvmRepl::new();
        assert!(repl.history_path.is_none());
    }

    #[test]
    fn test_bigvm_repl_default() {
        let repl = BigvmRepl::default();
        assert!(repl.history_path.is_none());
    }
}

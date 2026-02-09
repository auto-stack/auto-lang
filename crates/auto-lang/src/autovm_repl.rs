// AutoVM REPL: Simple REPL using AutoVM instead of TreeWalker Interpreter
//
// **Plan 068 Phase 9.5**: Default REPL now uses AutoVM for better performance
// **Plan 068 Phase 9.6**: Added persistent session support
//
// This REPL:
// - Uses AutoVM for code execution (not Interpreter/Evaler)
// - Supports REPL commands (:help, :quit, :reset, :stats)
// - Maintains function definitions across inputs (via persistent session)
// - NOTE: Local variables don't persist (stack-based VM limitation)
// - Can be compared against old-repl for validation

use crate::autovm_persistent::AutovmReplSession;
use crate::error::{AutoError, AutoResult};
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;

/// AutoVM REPL session
///
/// This REPL uses AutoVM (bytecode VM) for execution instead of
/// the deprecated TreeWalker Interpreter.
pub struct AutovmRepl {
    /// REPL history
    history_path: Option<String>,

    /// Persistent session for function definitions
    session: AutovmReplSession,
}

impl AutovmRepl {
    /// Create a new AutoVM REPL session
    pub fn new() -> Self {
        // Use platform-specific history file location
        let history_path = if cfg!(windows) {
            // Windows: %APPDATA%\autolang\autovm_history.txt
            std::env::var("APPDATA")
                .ok()
                .map(|path| std::path::PathBuf::from(path).join("autolang").join("autovm_history.txt"))
                .and_then(|path| path.to_str().map(|s| s.to_string()))
        } else if cfg!(target_os = "macos") {
            // macOS: ~/Library/Application Support/autolang/autovm_history.txt
            std::env::var("HOME")
                .ok()
                .map(|path| std::path::PathBuf::from(path).join("Library").join("Application Support").join("autolang").join("autovm_history.txt"))
                .and_then(|path| path.to_str().map(|s| s.to_string()))
        } else {
            // Linux/Unix: ~/.cache/autolang/autovm_history.txt
            std::env::var("HOME")
                .ok()
                .map(|path| std::path::PathBuf::from(path).join(".cache").join("autolang").join("autovm_history.txt"))
                .and_then(|path| path.to_str().map(|s| s.to_string()))
        };

        Self {
            history_path,
            session: AutovmReplSession::new(),
        }
    }

    /// Run the REPL main loop
    pub fn run(&mut self) -> AutoResult<()> {
        let mut editor = DefaultEditor::new().map_err(|e| {
            AutoError::Msg(format!("Failed to initialize REPL: {}", e))
        })?;

        // Load history if available
        if let Some(ref path) = self.history_path {
            // Create parent directory if it doesn't exist
            if let Some(parent_dir) = std::path::Path::new(path).parent() {
                let _ = std::fs::create_dir_all(parent_dir);
            }
            let _ = editor.load_history(path);
        }

        println!("🟢 AutoVM REPL (Plan 068 Phase 9.6)");
        println!("Type ':help' for commands, 'quit' or ':quit' to exit");
        println!();

        loop {
            let readline = editor.readline("AutoVM> ");

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

                    // Handle bare quit/exit/q commands (without colon)
                    match line {
                        "quit" | "exit" | "q" => {
                            println!("Goodbye!");
                            break;
                        }
                        _ => {}
                    }

                    // Execute code using persistent AutoVM session
                    match self.session.run(line) {
                        Ok(_) => {
                            // Display formatted result (heap objects like lists are formatted properly)
                            if let Some(formatted) = self.session.format_last_result() {
                                println!("{}", formatted);
                            }
                        }
                        Err(e) => {
                            // Check if this is a MultipleErrors error and print all inner errors
                            if let crate::error::AutoError::MultipleErrors { errors, .. } = &e {
                                eprintln!("Error: {}", e);
                                for (i, err) in errors.iter().enumerate() {
                                    eprintln!("  Error {}:", i + 1);
                                    eprintln!("    {}", err);
                                }
                            } else {
                                eprintln!("Error: {}", e);
                            }
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
    fn try_command(line: &str, repl: &mut AutovmRepl) -> Option<ReplCommand> {
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
                println!("=== AutoVM REPL Commands ===");
                println!("  :help     - Show this help message");
                println!("  :stats    - Show session statistics");
                println!("  :reset    - Clear all state (functions, etc.)");
                println!("  :quit     - Exit the REPL (or: quit, exit, :q)");
                println!();
                println!("Note: This REPL uses AutoVM (fast bytecode VM)");
                println!("Function definitions persist across inputs.");
                println!("Local variables don't persist (stack-based VM).");
                println!("For comparison, use 'auto.exe old-repl' to use the legacy Interpreter");
                Some(ReplCommand::Continue)
            }
            ":stats" => {
                let stats = repl.session.stats();
                println!("=== AutoVM REPL Statistics ===");
                println!("  Functions: {}", stats.total_functions);
                println!("  Bytecode size: {} bytes", stats.bytecode_size);
                println!("  Strings: {}", stats.total_strings);
                Some(ReplCommand::Continue)
            }
            ":reset" => {
                repl.session.reset();
                println!("Session reset. All state cleared.");
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

impl Default for AutovmRepl {
    fn default() -> Self {
        Self::new()
    }
}

enum ReplCommand {
    Exit,
    Continue,
}

/// Main entry point for AutoVM REPL
pub fn main_loop() -> AutoResult<()> {
    let mut repl = AutovmRepl::new();
    repl.run()
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_autovm_repl_create() {
        let repl = AutovmRepl::new();
        assert!(repl.history_path.is_none());
    }

    #[test]
    fn test_autovm_repl_default() {
        let repl = AutovmRepl::default();
        assert!(repl.history_path.is_none());
    }
}

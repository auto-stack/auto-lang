use crate::compile::CompileSession;
use crate::error::{AutoError, AutoResult};
use crate::interp;
use crate::runtime::ExecutionEngine;
use crate::universe::{Universe, VmRefData};
use auto_val::{Shared, Type, Value};
use rustyline::error::ReadlineError;
use rustyline::{DefaultEditor, Result};
use std::rc::Rc;
use std::cell::RefCell;

/// Persistent REPL session with incremental compilation support
///
/// **Phase 1**: Basic structure with CompileSession persistence
///
/// This structure maintains a persistent CompileSession across multiple
/// REPL inputs, enabling incremental compilation and caching.
pub struct ReplSession {
    /// Compile-time data (persistent across inputs)
    pub session: CompileSession,

    /// Runtime execution engine (recreated or cleared per input)
    pub engine: Rc<RefCell<ExecutionEngine>>,
}

impl ReplSession {
    /// Create a new REPL session
    ///
    /// Initializes a new CompileSession with a fresh Database and
    /// creates a new ExecutionEngine for runtime execution.
    pub fn new() -> Self {
        let session = CompileSession::new();
        let engine = Rc::new(RefCell::new(ExecutionEngine::new()));

        Self {
            session,
            engine,
        }
    }

    /// Execute code with incremental compilation
    ///
    /// **Phase 2**: Execute code using persistent CompileSession
    ///
    /// This method uses the persistent CompileSession to enable incremental
    /// compilation across multiple REPL inputs.
    ///
    /// # Arguments
    ///
    /// * `code` - AutoLang source code to execute
    ///
    /// # Returns
    ///
    /// String representation of the result, or error message
    pub fn run(&mut self, code: &str) -> AutoResult<String> {
        // Use the run_with_session function for incremental compilation
        crate::run_with_session(&mut self.session, code)
    }

    /// Get session statistics
    pub fn stats(&self) -> ReplStats {
        let db = self.session.database().unwrap();
        ReplStats {
            total_files: db.get_files().len(),
            total_fragments: 0, // TODO: Implement fragment counting
            cache_entries: 0, // TODO: Phase 3 - QueryEngine cache stats
            dirty_files: 0, // TODO: Implement dirty file tracking
        }
    }

    /// Clear runtime state (keep compile-time data)
    pub fn reset_runtime(&mut self) {
        self.engine = Rc::new(RefCell::new(ExecutionEngine::new()));
    }
}

/// REPL session statistics
#[derive(Debug, Clone)]
pub struct ReplStats {
    pub total_files: usize,
    pub total_fragments: usize,
    pub cache_entries: usize,
    pub dirty_files: usize,
}

/// Format a value for display, with special handling for Lists
fn format_value(value: &Value, uni: &Shared<Universe>) -> String {
    // Check if this is a List instance
    if let Value::Instance(inst) = value {
        if let Type::User(ref_name) = &inst.ty {
            if ref_name == "List" {
                // Try to get the list contents from the VmRef
                if let Some(Value::USize(id)) = inst.fields.get("id") {
                    let uni_borrow = uni.borrow();
                    if let Some(vmref) = uni_borrow.get_vmref_ref(id) {
                        let ref_box = vmref.borrow();
                        if let VmRefData::List(list) = &*ref_box {
                            // Format as List[elem1, elem2, ...]
                            let elems: Vec<String> = list.elems.iter()
                                .map(|v| format!("{}", v))
                                .collect();
                            return format!("List[{}]", elems.join(", "));
                        }
                    }
                }
            }
        }
    }

    // Default formatting for non-List values
    format!("{}", value)
}

enum CmdResult {
    Exit,
    Continue,
}

/// Try special REPL commands (starting with ':')
fn try_repl_command(line: &str, repl_session: &mut ReplSession) -> CmdResult {
    let words = line.split_whitespace().collect::<Vec<&str>>();
    if words.is_empty() {
        return CmdResult::Continue;
    }

    let cmd = &words[0];
    match *cmd {
        ":stats" => {
            // Show session statistics
            let stats = repl_session.stats();
            println!("REPL Statistics:");
            println!("  Files: {}", stats.total_files);
            println!("  Fragments: {}", stats.total_fragments);
            println!("  Cache entries: {}", stats.cache_entries);
            println!("  Dirty files: {}", stats.dirty_files);
            CmdResult::Continue
        }
        ":reset" => {
            // Clear runtime state (keep compile-time data)
            repl_session.reset_runtime();
            println!("Runtime state cleared (compile-time data preserved)");
            CmdResult::Continue
        }
        ":help" => {
            println!("REPL Commands:");
            println!("  :stats  - Show session statistics");
            println!("  :reset  - Clear runtime state (keeps compiled code)");
            println!("  :help   - Show this help");
            println!("  quit    - Exit the REPL");
            println!();
            println!("You can also execute AutoLang code directly!");
            println!("Example: fn add(a int, b int) int {{ a + b }}");
            CmdResult::Continue
        }
        ":quit" | ":exit" => CmdResult::Exit,
        _ => {
            println!("Unknown command: {}. Try :help", cmd);
            CmdResult::Continue
        }
    }
}

/// Legacy command handler (for backward compatibility)
fn try_command(line: &str, interpreter: &mut interp::Interpreter) -> CmdResult {
    let words = line.split_whitespace().collect::<Vec<&str>>();
    if words.len() == 0 {
        return CmdResult::Continue;
    }
    let cmd = words[0];
    match cmd {
        "help" => {
            println!("help - show this help");
            CmdResult::Continue
        }
        "q" | "quit" => CmdResult::Exit,
        "load" => {
            if words.len() == 2 {
                let filename = words[1];
                println!("Loading file: {}", filename);
                match interpreter.load_file(filename) {
                    Ok(_) => CmdResult::Continue,
                    Err(error) => {
                        print_miette_error(error);
                        CmdResult::Continue
                    }
                }
            } else {
                eprintln!("Usage: load <filename>");
                CmdResult::Continue
            }
        }
        "load_config" => {
            if words.len() == 2 {
                let filename = words[1];
                println!("Loading config file: {}", filename);
                match interpreter.load_config(filename) {
                    Ok(_) => CmdResult::Continue,
                    Err(error) => {
                        print_miette_error(error);
                        CmdResult::Continue
                    }
                }
            } else {
                eprintln!("Usage: load_config <filename>");
                CmdResult::Continue
            }
        }
        "scope" => {
            // interpreter.dump_scope();
            CmdResult::Continue
        }
        _ => match interpreter.interpret(line) {
            Ok(_) => {
                let formatted = format_value(&interpreter.result, &interpreter.scope);
                println!("{}", formatted);
                CmdResult::Continue
            }
            Err(error) => {
                // Attach source code to the error for better display
                let error_with_source = crate::error::attach_source(
                    error,
                    "<repl>".to_string(),
                    line.to_string(),
                );
                print_miette_error(error_with_source);
                CmdResult::Continue
            }
        },
    }
}

fn print_miette_error(err: AutoError) {
    // Handle MultipleErrors by displaying each error separately
    if let crate::error::AutoError::MultipleErrors { errors, .. } = err {
        // Display each individual error
        for error in errors {
            eprintln!("{:?}", miette::Report::new(error));
        }
    } else {
        // Single error - just display it
        eprintln!("{:?}", miette::Report::new(err));
    }
}

pub fn main_loop() -> Result<()> {
    let mut rl = DefaultEditor::new()?;
    #[cfg(feature = "with-file-history")]
    if rl.load_history(".history.txt").is_err() {
        println!("No previous history");
    }

    // **Phase 4**: Use ReplSession for incremental compilation!
    let mut repl_session = ReplSession::new();
    println!("AutoLang REPL (with incremental compilation)");
    println!("Commands: :stats, :reset, help, quit");

    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                if rl.add_history_entry(line.as_str()).is_err() {
                    println!("Unable to add history");
                    break;
                }

                // Check for special REPL commands
                let trimmed = line.trim();
                if trimmed.starts_with(':') {
                    match try_repl_command(&trimmed, &mut repl_session) {
                        CmdResult::Exit => break,
                        CmdResult::Continue => continue,
                    }
                }

                // Execute code with incremental compilation
                match repl_session.run(&line) {
                    Ok(result) => println!("{}", result),
                    Err(error) => {
                        // Attach source code to the error for better display
                        let error_with_source = crate::error::attach_source(
                            error,
                            "<repl>".to_string(),
                            line.to_string(),
                        );
                        print_miette_error(error_with_source);
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
    #[cfg(feature = "with-file-history")]
    rl.save_history(".history.txt")?;
    Ok(())
}

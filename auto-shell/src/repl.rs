use miette::Result;
use reedline::{
    default_emacs_keybindings, ColumnarMenu, MenuBuilder, DefaultPrompt,
    Emacs, FileBackedHistory, KeyCode, KeyModifiers, Reedline, ReedlineEvent, ReedlineMenu, Signal,
};
use std::path::PathBuf;

use crate::{completions::reedline::ShellCompleter, shell::Shell};

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
                .map_err(|e| miette::miette!("Failed to create history: {}", e))?,
        );

        // Create completer for Tab completion
        let completer = Box::new(ShellCompleter::new());
        // // Test with DefaultCompleter
        // let commands = vec![
        //     "test".into(),
        //     "hello world".into(),
        //     "hello world reedline".into(),
        //     "this is the reedline crate".into(),
        // ];
        // let completer = Box::new(DefaultCompleter::new_with_wordlen(commands.clone(), 2));

        // Use the interactive menu to select options from the completer
        let completion_menu = Box::new(ColumnarMenu::default().with_name("completion_menu"));
        // Set up the required keybindings
        let mut keybindings = default_emacs_keybindings();
        keybindings.add_binding(
            KeyModifiers::NONE,
            KeyCode::Tab,
            ReedlineEvent::UntilFound(vec![
                ReedlineEvent::Menu("completion_menu".to_string()),
                ReedlineEvent::MenuNext,
            ]),
        );

        let edit_mode = Box::new(Emacs::new(keybindings));

        let line_editor = Reedline::create()
            .with_history(history)
            .with_completer(completer)
            .with_menu(ReedlineMenu::EngineCompleter(completion_menu))
            .with_edit_mode(edit_mode);

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
        // let prompt = DefaultPrompt::new(DefaultPromptSegment::Empty, DefaultPromptSegment::Empty);

        // // Use the interactive menu to select options from the completer
        // let completion_menu = Box::new(ColumnarMenu::default().with_name("completion_menu"));
        // // Set up the required keybindings
        // let mut keybindings = default_emacs_keybindings();
        // keybindings.add_binding(
        //     KeyModifiers::NONE,
        //     KeyCode::Tab,
        //     ReedlineEvent::UntilFound(vec![
        //         ReedlineEvent::Menu("completion_menu".to_string()),
        //         ReedlineEvent::MenuNext,
        //     ]),
        // );

        // let edit_mode = Box::new(Emacs::new(keybindings));

        // let mut line_editor = Reedline::create()
        //     .with_completer(completer)
        //     .with_menu(ReedlineMenu::EngineCompleter(completion_menu))
        //     .with_edit_mode(edit_mode);

        let prompt = DefaultPrompt::default();

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

use crate::eval;
use crate::scope;
use std::rc::Rc;
use std::cell::RefCell;
use rustyline::error::ReadlineError;
use rustyline::{DefaultEditor, Result};

enum CmdResult {
    Exit,
    Continue,
}

fn try_command(line: &str, evaler: &mut eval::Evaler) -> CmdResult {
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
        "q" | "quit" => {
            CmdResult::Exit
        }
        "load" => {
            if words.len() == 2 {
                let filename = words[1];
                match evaler.load_file(filename) {
                    Ok(_) => CmdResult::Continue,
                    Err(error) => {
                        eprintln!("Error: {}", error);
                        CmdResult::Continue
                    }
                }
            } else {
                eprintln!("Usage: load <filename>");
                CmdResult::Continue
            }
        }
        "scope" => {
            evaler.dump_scope();
            CmdResult::Continue
        }
        _ => {
            match evaler.interpret(line) {
                Ok(result) => {
                    println!("{}", result);
                    CmdResult::Continue
                }
                Err(error) => {
                    eprintln!("Error: {}", error);
                    CmdResult::Continue
                }
            }
        }
    }
}

pub fn main_loop() -> Result<()> {
    let mut rl = DefaultEditor::new()?;
    #[cfg(feature = "with-file-history")]
    if rl.load_history(".history.txt").is_err() {
        println!("No previous history");
    }
    // initialize evaler
    let scope = Rc::new(RefCell::new(scope::Universe::new()));
    let mut evaler = eval::Evaler::new(scope);
    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                if rl.add_history_entry(line.as_str()).is_err() {
                    println!("Unable to add history");
                    break;
                }
                // split first word and check if it's a command
                match try_command(&line, &mut evaler) {
                    CmdResult::Exit => break,
                    CmdResult::Continue => continue,
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

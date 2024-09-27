use crate::eval;
use crate::scope;
use rustyline::error::ReadlineError;
use rustyline::{DefaultEditor, Result};

pub fn main_loop() -> Result<()> {
    let mut rl = DefaultEditor::new()?;
    #[cfg(feature = "with-file-history")]
    if rl.load_history("history.txt").is_err() {
        println!("No previous history");
    }
    // initialize evaler
    let mut scope = scope::Universe::new();
    let mut evaler = eval::Evaler::new(&mut scope);
    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                if rl.add_history_entry(line.as_str()).is_err() {
                    println!("Unable to add history");
                    break;
                }
                if line == "q" || line == "quit" {
                    break;
                }
                match evaler.interpret(line.as_str()) {
                    Ok(result) => println!("{}", result),
                    Err(error) => eprintln!("Error: {}", error),
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
    rl.save_history("history.txt")?;
    Ok(())
}

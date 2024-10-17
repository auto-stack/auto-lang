use clap::{Parser, Subcommand};
use autolang::repl;
use std::error::Error;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    #[command(about = "Parse Auto to JSON")]
    Parse { code: String },
    Repl
}


fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Parse { code}) => {
            println!("Parsing Auto {} to JSON", code);
            let json = autolang::run(&code)?;
            println!("{}", json);
        }
        Some(Commands::Repl) => {
            repl::main_loop()?;
        }
        None => {
            repl::main_loop()?;
        }
    }

    Ok(())
}

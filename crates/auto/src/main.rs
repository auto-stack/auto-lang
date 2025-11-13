use auto_lang::repl;
use clap::{Parser, Subcommand};
use std::error::Error;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    #[command(about = "AutoLang REPL")]
    Repl,
    #[command(about = "Parse Auto to JSON")]
    Parse { code: String },
    #[command(about = "Run Auto Script")]
    Run { path: String },
    #[command(about = "Transpile Auto to C")]
    C { path: String },
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Parse { code }) => {
            println!("Parsing Auto {} to JSON", code);
            let json = auto_lang::run(&code)?;
            println!("{}", json);
        }
        Some(Commands::Run { path }) => {
            println!("Running Auto {} ", path);
            let result = auto_lang::run_file(&path)?;
            println!("{}", result);
        }
        Some(Commands::Repl) => {
            repl::main_loop()?;
        }
        Some(Commands::C { path }) => {
            let c = auto_lang::trans_c(path.as_str())?;
            println!("{}", c);
        }
        None => {
            repl::main_loop()?;
        }
    }

    Ok(())
}

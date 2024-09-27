use clap::{Parser, Subcommand};
use acl::repl;
use std::error::Error;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    #[command(about = "Parse ACL to JSON")]
    Parse { code: String },
    Repl
}


fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Parse { code}) => {
            println!("Parsing ACL {} to JSON", code);
            let json = acl::run(&code)?;
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

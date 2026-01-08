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
    #[command(about = "Evaluate Auto expression")]
    Eval { code: String },
    #[command(about= "Treat File as AutoConfig")]
    Config { path: String },
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
            println!("----------------------");
            println!("Running Auto {} ", path);
            println!("----------------------");
            let result = auto_lang::run_file(&path)?;
            println!("{}", result);
            println!();
        }
        Some(Commands::Eval { code }) => {
            let result = auto_lang::run(&code)?;
            println!("{}", result);
        }
        Some(Commands::Repl) => {
            repl::main_loop()?;
        }
        Some(Commands::Config { path }) => {
            let code = std::fs::read_to_string(path.as_str())?;
            let args = auto_val::Obj::new();
            let c = auto_lang::eval_config(code.as_str(), &args)?;
            println!("{}", c.result.repr());
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

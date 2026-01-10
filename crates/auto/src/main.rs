use auto_lang::error::AutoError;
use auto_lang::repl;
use clap::{Parser, Subcommand};
use miette::{MietteHandlerOpts, Result};

// Helper to convert AutoError to miette Report - this preserves all diagnostic info
fn to_miette_err(err: AutoError) -> miette::Report {
    miette::Report::new(err)
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Maximum number of errors to display before aborting (default: 20)
    #[arg(short, long, global = true, value_name = "N")]
    error_limit: Option<usize>,

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
    #[command(about = "Treat File as AutoConfig")]
    Config { path: String },
    #[command(about = "Transpile Auto to C")]
    C { path: String },
    #[command(about = "Transpile Auto to Rust")]
    Rust { path: String },
}

fn main() -> Result<()> {
    // Set up miette for beautiful error reporting with fancy colors
    miette::set_hook(Box::new(|_| {
        Box::new(MietteHandlerOpts::new().terminal_links(true).build())
    }))
    .ok();

    let cli = Cli::parse();

    // Set error limit from CLI if provided
    if let Some(limit) = cli.error_limit {
        auto_lang::set_error_limit(limit);
    }

    match cli.command {
        Some(Commands::Parse { code }) => {
            println!("Parsing Auto {} to JSON", code);
            let json = auto_lang::run(&code).map_err(to_miette_err)?;
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
            let result = auto_lang::run(&code).map_err(to_miette_err)?;
            println!("{}", result);
        }
        Some(Commands::Repl) => {
            repl::main_loop().map_err(|e| miette::miette!("{}", e))?;
        }
        Some(Commands::Config { path }) => {
            let code = std::fs::read_to_string(path.as_str())
                .map_err(|e| miette::miette!("Failed to read file: {}", e))?;
            let args = auto_val::Obj::new();
            let c = auto_lang::eval_config(code.as_str(), &args).map_err(to_miette_err)?;
            println!("{}", c.result.repr());
        }
        Some(Commands::C { path }) => {
            let c = auto_lang::trans_c(path.as_str()).map_err(to_miette_err)?;
            println!("{}", c);
        }
        Some(Commands::Rust { path }) => {
            let r = auto_lang::trans_rust(path.as_str()).map_err(to_miette_err)?;
            println!("{}", r);
        }
        None => {
            repl::main_loop().map_err(|e| miette::miette!("{}", e))?;
        }
    }

    Ok(())
}

use auto_lang::error::AutoError;
use auto_lang::repl;
use clap::{Parser, Subcommand, ValueEnum};
use miette::{Diagnostic, MietteHandlerOpts, Result};
use serde_json::{json, Value};

// Helper to convert AutoError to miette Report - this preserves all diagnostic info
fn to_miette_err(err: AutoError) -> miette::Report {
    miette::Report::new(err)
}

/// Format error as JSON for machine consumption
fn format_error_json(err: &AutoError) -> String {
    let mut error_obj: Value = json!({
        "message": err.to_string(),
    });

    // Add error code if available
    if let Some(code) = err.code() {
        error_obj["code"] = json!(code.to_string());
    }

    // Add severity level
    let severity = if matches!(err, AutoError::Warning(_)) {
        "warning"
    } else {
        "error"
    };
    error_obj["severity"] = json!(severity);

    // Try to get source span information from labels
    if let Some(labels) = err.labels() {
        let spans: Vec<Value> = labels
            .map(|label| {
                let mut span_obj = json!({
                    "offset": label.offset(),
                    "len": label.len(),
                });
                // Add label text if present
                if let Some(text) = label.label() {
                    span_obj["label"] = json!(text);
                }
                span_obj
            })
            .collect();
        error_obj["spans"] = json!(spans);
    }

    // Add help text if available
    if let Some(help) = err.help() {
        error_obj["help"] = json!(help.to_string());
    }

    // Add related errors (for MultipleErrors)
    if let Some(related) = err.related() {
        let related_errors: Vec<Value> = related
            .map(|diag| {
                json!({
                    "message": diag.to_string(),
                    "code": diag.code().map(|c| c.to_string()),
                })
            })
            .collect();
        if !related_errors.is_empty() {
            error_obj["related"] = json!(related_errors);
        }
    }

    error_obj.to_string()
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Maximum number of errors to display before aborting (default: 20)
    #[arg(short, long, global = true, value_name = "N")]
    error_limit: Option<usize>,

    /// Output format for errors and diagnostics
    #[arg(long, global = true, value_name = "FORMAT")]
    format: Option<OutputFormat>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Clone, Debug, ValueEnum)]
enum OutputFormat {
    /// Human-readable text output with colors (default)
    Text,
    /// Machine-readable JSON output for IDE integration
    Json,
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
    let cli = Cli::parse();

    // Set up miette handler based on format preference
    let format = cli.format.clone().unwrap_or(OutputFormat::Text);

    miette::set_hook(Box::new(move |_| {
        Box::new(MietteHandlerOpts::new().terminal_links(true).build())
    }))
    .ok();

    // Set error limit from CLI if provided
    if let Some(limit) = cli.error_limit {
        auto_lang::set_error_limit(limit);
    }

    match cli.command {
        Some(Commands::Parse { code }) => {
            // For JSON mode, suppress decorative output
            if matches!(format, OutputFormat::Text) {
                println!("Parsing Auto {} to JSON", code);
            }
            let json = auto_lang::run(&code).map_err(|e| {
                if matches!(format, OutputFormat::Json) {
                    // Print JSON error and exit
                    eprintln!("{}", format_error_json(&e));
                    std::process::exit(1);
                }
                to_miette_err(e)
            })?;
            println!("{}", json);
        }
        Some(Commands::Run { path }) => {
            // Only print decorative output for text format
            if matches!(format, OutputFormat::Text) {
                println!("----------------------");
                println!("Running Auto {} ", path);
                println!("----------------------");
            }
            let result = auto_lang::run_file(&path).map_err(|e| {
                if matches!(format, OutputFormat::Json) {
                    // Print JSON error and exit
                    eprintln!("{}", format_error_json(&e));
                    std::process::exit(1);
                }
                to_miette_err(e)
            })?;
            println!("{}", result);
            if matches!(format, OutputFormat::Text) {
                println!();
            }
        }
        Some(Commands::Eval { code }) => {
            let result = auto_lang::run(&code).map_err(|e| {
                if matches!(format, OutputFormat::Json) {
                    eprintln!("{}", format_error_json(&e));
                    std::process::exit(1);
                }
                to_miette_err(e)
            })?;
            println!("{}", result);
        }
        Some(Commands::Repl) => {
            repl::main_loop().map_err(|e| miette::miette!("{}", e))?;
        }
        Some(Commands::Config { path }) => {
            let code = std::fs::read_to_string(path.as_str())
                .map_err(|e| miette::miette!("Failed to read file: {}", e))?;
            let args = auto_val::Obj::new();
            let c = auto_lang::eval_config(code.as_str(), &args).map_err(|e| {
                if matches!(format, OutputFormat::Json) {
                    eprintln!("{}", format_error_json(&e));
                    std::process::exit(1);
                }
                to_miette_err(e)
            })?;
            println!("{}", c.result.repr());
        }
        Some(Commands::C { path }) => {
            let c = auto_lang::trans_c(path.as_str()).map_err(|e| {
                if matches!(format, OutputFormat::Json) {
                    eprintln!("{}", format_error_json(&e));
                    std::process::exit(1);
                }
                to_miette_err(e)
            })?;
            println!("{}", c);
        }
        Some(Commands::Rust { path }) => {
            let r = auto_lang::trans_rust(path.as_str()).map_err(|e| {
                if matches!(format, OutputFormat::Json) {
                    eprintln!("{}", format_error_json(&e));
                    std::process::exit(1);
                }
                to_miette_err(e)
            })?;
            println!("{}", r);
        }
        None => {
            repl::main_loop().map_err(|e| miette::miette!("{}", e))?;
        }
    }

    Ok(())
}

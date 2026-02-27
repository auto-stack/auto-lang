use auto_lang::error::AutoError;
use clap::{Parser, Subcommand, ValueEnum};
use miette::{Diagnostic, MietteHandlerOpts, Result};
use serde_json::{json, Value};
use colored::Colorize;
use log::info;

mod cmd_a2c_stdlib;

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

fn init_logger() {
    simplelog::CombinedLogger::init(vec![
        simplelog::TermLogger::new(
            simplelog::LevelFilter::Info,
            simplelog::Config::default(),
            simplelog::TerminalMode::Mixed,
            simplelog::ColorChoice::Auto,
        ),
    ])
    .unwrap();
}

fn load_am_config() -> Option<auto_man::AmConfig> {
    auto_man::load_am_config()
}

fn select_port(input: Option<String>, ports: &Vec<auto_val::AutoStr>) -> auto_val::AutoResult<auto_val::AutoStr> {
    auto_man::util::select_or_default_port(input, ports, "Which port do you want to build?")
}

#[derive(Subcommand, Debug)]
enum CacheCommands {
    #[command(about = "Show cache statistics")]
    Stats,
    #[command(about = "List all cached artifacts")]
    List {
        #[arg(long)]
        type_: Option<String>,
        #[arg(short, long, default_value = "50")]
        limit: usize,
    },
    #[command(about = "Run garbage collection")]
    Prune,
    #[command(about = "Clear all cached artifacts")]
    Clear,
    #[command(about = "Inspect a cache entry")]
    Inspect { name: String },
    #[command(about = "Verify cache integrity")]
    Verify,
}

#[derive(Subcommand, Debug)]
enum Commands {
    #[command(about = "AutoLang REPL (deprecated - uses TreeWalker Interpreter)")]
    OldRepl,
    #[command(about = "Parse Auto to JSON")]
    Parse { code: String },
    #[command(about = "Run Auto Script")]
    Run { path: String },
    #[command(about = "Evaluate Auto expression")]
    Eval { code: String },
    #[command(about = "Treat File as AutoConfig")]
    Config { path: String },
    #[command(about = "Transpile Auto to C")]
    C {
        path: String,
        #[arg(short, long, help = "Compilation target (mcu, pc, or auto)", global = false)]
        target: Option<String>,
    },
    #[command(about = "Transpile Auto to Rust")]
    Rust { path: String },
    #[command(about = "Transpile Auto to Python")]
    Python { path: String },
    #[command(about = "Transpile Auto to JavaScript")]
    JavaScript { path: String },
    #[command(about = "Transpile stdlib to C")]
    A2cStdlib,

    // ========== UI Commands ==========

    #[command(about = "Build UI components from Auto files")]
    Ui {
        /// Input file or directory
        path: String,

        /// Compilation scenario (core, ui, shell)
        #[arg(short, long, default_value = "ui")]
        scenario: String,

        /// Backend target (vue, rust, gpui)
        #[arg(short, long, default_value = "vue")]
        backend: String,

        /// Output directory
        #[arg(short, long)]
        output: Option<String>,
    },

    // ========== Build System Commands ==========

    #[command(about = "Create a new Auto application package", alias = "a")]
    App { name: String },

    #[command(about = "Create a new Auto library package", alias = "l")]
    Lib { name: String },

    #[command(about = "Create a new C application package")]
    Capp { name: String },

    #[command(about = "Create a new C library package")]
    Clib { name: String },

    #[command(about = "Scan project and download dependencies")]
    Scan,

    #[command(about = "Build the project", alias = "b")]
    Build {
        #[arg(short, long)]
        dir: Option<String>,
    },

    #[command(about = "Run the compiled executable", alias = "r")]
    RunExe {
        #[arg(allow_hyphen_values = true)]
        args: Vec<String>,
    },

    #[command(about = "Clean build artifacts")]
    Clean {
        #[arg(short, long)]
        dir: Option<String>,
    },

    #[command(about = "Show dependency tree")]
    Deps,

    #[command(about = "Show available devices")]
    Devices,

    #[command(about = "Manage AutoCache")]
    Cache {
        #[command(subcommand)]
        command: CacheCommands,
    },

    #[command(about = "Open project in IDE", alias = "o")]
    Open,

    #[command(about = "Show package or target information", alias = "i")]
    Info {
        #[arg(short, long)]
        target: Option<String>,
    },

    #[command(about = "Show or select build port")]
    Port,

    #[command(about = "Pull/download all dependencies")]
    Pull,

    #[command(about = "Reset AutoMan configuration and index")]
    Reset,

    #[command(about = "Install AutoMan configuration file")]
    Install { file: String },
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
        Some(Commands::OldRepl) => {
            // Plan 092: Use autovm_repl instead of old repl module
            auto_lang::autovm_repl::main_loop().map_err(|e| miette::miette!("{}", e))?;
        }
        Some(Commands::Config { path }) => {
            let code = std::fs::read_to_string(path.as_str())
                .map_err(|e| miette::miette!("Failed to read file: {}", e))?;
            let args = auto_val::Obj::new();
            let c = auto_lang::eval_config_with_vm(code.as_str(), &args).map_err(|e| {
                if matches!(format, OutputFormat::Json) {
                    eprintln!("{}", format_error_json(&e));
                    std::process::exit(1);
                }
                to_miette_err(e)
            })?;
            println!("{}", c.repr());
        }
        Some(Commands::C { path, target }) => {
            // Set target environment variable if specified
            if let Some(target_val) = target {
                std::env::set_var("AUTO_TARGET", target_val);
            }

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
        Some(Commands::Python { path }) => {
            let py = auto_lang::trans_python(path.as_str()).map_err(|e| {
                if matches!(format, OutputFormat::Json) {
                    eprintln!("{}", format_error_json(&e));
                    std::process::exit(1);
                }
                to_miette_err(e)
            })?;
            println!("{}", py);
        }
        Some(Commands::JavaScript { path }) => {
            let js = auto_lang::trans_javascript(path.as_str()).map_err(|e| {
                if matches!(format, OutputFormat::Json) {
                    eprintln!("{}", format_error_json(&e));
                    std::process::exit(1);
                }
                to_miette_err(e)
            })?;
            println!("{}", js);
        }
        Some(Commands::A2cStdlib) => {
            cmd_a2c_stdlib::run()?;
        }

        // ========== UI Commands ==========

        Some(Commands::Ui { path, scenario, backend, output }) => {
            init_logger();
            println!("{}", "---------------------------".bright_yellow().bold());
            println!("{}", "AURA UI Builder".bright_yellow().bold());
            println!("{}", "---------------------------".bright_yellow().bold());

            // Build UI components using AURA pipeline
            match auto_lang::ui_build(&path, &scenario, &backend, output.as_deref()) {
                Ok(code) => println!("{}", code),
                Err(e) => {
                    if matches!(format, OutputFormat::Json) {
                        eprintln!("{}", format_error_json(&e));
                        std::process::exit(1);
                    }
                    return Err(to_miette_err(e));
                }
            }
        }

        // ========== Build System Commands ==========

        Some(Commands::App { name }) => {
            init_logger();
            println!("{}", "---------------------------".bright_yellow().bold());
            println!("{}", "Hello, I'm Auto!".bright_yellow().bold());
            println!("{}", "---------------------------".bright_yellow().bold());
            auto_man::Automan::create_app(&name).map_err(|e| miette::miette!("{}", e))?;
        }

        Some(Commands::Lib { name }) => {
            init_logger();
            println!("{}", "---------------------------".bright_yellow().bold());
            println!("{}", "Hello, I'm Auto!".bright_yellow().bold());
            println!("{}", "---------------------------".bright_yellow().bold());
            auto_man::Automan::create_lib(&name).map_err(|e| miette::miette!("{}", e))?;
        }

        Some(Commands::Capp { name }) => {
            init_logger();
            println!("{}", "---------------------------".bright_yellow().bold());
            println!("{}", "Hello, I'm Auto!".bright_yellow().bold());
            println!("{}", "---------------------------".bright_yellow().bold());
            auto_man::Automan::create_capp(&name).map_err(|e| miette::miette!("{}", e))?;
        }

        Some(Commands::Clib { name }) => {
            init_logger();
            println!("{}", "---------------------------".bright_yellow().bold());
            println!("{}", "Hello, I'm Auto!".bright_yellow().bold());
            println!("{}", "---------------------------".bright_yellow().bold());
            auto_man::Automan::create_clib(&name).map_err(|e| miette::miette!("{}", e))?;
        }

        Some(Commands::Scan) => {
            init_logger();
            println!("{}", "---------------------------".bright_yellow().bold());
            println!("{}", "Hello, I'm Auto!".bright_yellow().bold());
            println!("{}", "---------------------------".bright_yellow().bold());
            let config = load_am_config().unwrap_or(auto_man::AmConfig::default());
            let mut am = auto_man::Automan::new(".", config).map_err(|e| miette::miette!("{}", e))?;
            am.scan().map_err(|e| miette::miette!("{}", e))?;
        }

        Some(Commands::Build { dir }) => {
            init_logger();
            println!("{}", "---------------------------".bright_yellow().bold());
            println!("{}", "Hello, I'm Auto!".bright_yellow().bold());
            println!("{}", "---------------------------".bright_yellow().bold());
            let dir = if let Some(dir) = dir { dir } else { ".".to_string() };
            let config = load_am_config().unwrap_or(auto_man::AmConfig::default());
            let mut am = auto_man::Automan::new(&dir, config).map_err(|e| miette::miette!("{}", e))?;
            am.scan().map_err(|e| miette::miette!("{}", e))?;
            am.build().map_err(|e| miette::miette!("{}", e))?;
        }

        Some(Commands::RunExe { args }) => {
            init_logger();
            println!("{}", "---------------------------".bright_yellow().bold());
            println!("{}", "Hello, I'm Auto!".bright_yellow().bold());
            println!("{}", "---------------------------".bright_yellow().bold());
            let config = load_am_config().unwrap_or(auto_man::AmConfig::default());
            let mut am = auto_man::Automan::new(".", config).map_err(|e| miette::miette!("{}", e))?;
            info!("Running app ...");
            println!();
            println!("------------ output ------------");
            am.run(args).map_err(|e| miette::miette!("{}", e))?;
            println!("------------- end --------------");
        }

        Some(Commands::Clean { dir }) => {
            init_logger();
            println!("{}", "---------------------------".bright_yellow().bold());
            println!("{}", "Hello, I'm Auto!".bright_yellow().bold());
            println!("{}", "---------------------------".bright_yellow().bold());
            let dir = if let Some(dir) = dir { dir } else { ".".to_string() };
            let config = load_am_config().unwrap_or(auto_man::AmConfig::default());
            let mut am = auto_man::Automan::new(&dir, config).map_err(|e| miette::miette!("{}", e))?;
            am.clean().map_err(|e| miette::miette!("{}", e))?;
        }

        Some(Commands::Deps) => {
            init_logger();
            println!("{}", "---------------------------".bright_yellow().bold());
            println!("{}", "Hello, I'm Auto!".bright_yellow().bold());
            println!("{}", "---------------------------".bright_yellow().bold());
            let config = load_am_config().unwrap_or(auto_man::AmConfig::default());
            auto_man::Automan::list_deps(&config).map_err(|e| miette::miette!("{}", e))?;
        }

        Some(Commands::Devices) => {
            init_logger();
            println!("{}", "---------------------------".bright_yellow().bold());
            println!("{}", "Hello, I'm Auto!".bright_yellow().bold());
            println!("{}", "---------------------------".bright_yellow().bold());
            let config = load_am_config().unwrap_or(auto_man::AmConfig::default());
            auto_man::Automan::list_devices(&config).map_err(|e| miette::miette!("{}", e))?;
        }

        Some(Commands::Open) => {
            init_logger();
            println!("{}", "---------------------------".bright_yellow().bold());
            println!("{}", "Hello, I'm Auto!".bright_yellow().bold());
            println!("{}", "---------------------------".bright_yellow().bold());
            let config = load_am_config().unwrap_or(auto_man::AmConfig::default());
            let mut am = auto_man::Automan::new(".", config).map_err(|e| miette::miette!("{}", e))?;
            am.open_ide().map_err(|e| miette::miette!("{}", e))?;
        }

        Some(Commands::Cache { command }) => {
            init_logger();
            println!("{}", "---------------------------".bright_yellow().bold());
            println!("{}", "Hello, I'm Auto!".bright_yellow().bold());
            println!("{}", "---------------------------".bright_yellow().bold());
            let config = load_am_config().unwrap_or(auto_man::AmConfig::default());
            let mut am = auto_man::Automan::new(".", config).map_err(|e| miette::miette!("{}", e))?;

            match command {
                CacheCommands::Stats => {
                    am.cache_stats().map_err(|e| miette::miette!("{}", e))?;
                }
                CacheCommands::List { type_, limit } => {
                    am.cache_list(type_, limit).map_err(|e| miette::miette!("{}", e))?;
                }
                CacheCommands::Prune => {
                    am.cache_prune().map_err(|e| miette::miette!("{}", e))?;
                }
                CacheCommands::Clear => {
                    am.cache_clear().map_err(|e| miette::miette!("{}", e))?;
                }
                CacheCommands::Inspect { name } => {
                    am.cache_inspect(&name).map_err(|e| miette::miette!("{}", e))?;
                }
                CacheCommands::Verify => {
                    am.cache_verify().map_err(|e| miette::miette!("{}", e))?;
                }
            }
        }

        Some(Commands::Info { target }) => {
            init_logger();
            println!("{}", "---------------------------".bright_yellow().bold());
            println!("{}", "Hello, I'm Auto!".bright_yellow().bold());
            println!("{}", "---------------------------".bright_yellow().bold());
            let config = load_am_config().unwrap_or(auto_man::AmConfig::default());
            let am = auto_man::Automan::new(".", config).map_err(|e| miette::miette!("{}", e))?;
            am.info(target).map_err(|e| miette::miette!("{}", e))?;
        }

        Some(Commands::Port) => {
            init_logger();
            println!("{}", "---------------------------".bright_yellow().bold());
            println!("{}", "Hello, I'm Auto!".bright_yellow().bold());
            println!("{}", "---------------------------".bright_yellow().bold());
            let config = load_am_config().unwrap_or(auto_man::AmConfig::default());
            let mut am = auto_man::Automan::new(".", config).map_err(|e| miette::miette!("{}", e))?;
            let port = select_port(None, &am.list_port_names()).map_err(|e| miette::miette!("{}", e))?;
            am.set_port(port.clone()).map_err(|e| miette::miette!("{}", e))?;
            info!("port \"{}\" written to .am/state.at", port)
        }

        Some(Commands::Pull) => {
            init_logger();
            println!("{}", "---------------------------".bright_yellow().bold());
            println!("{}", "Hello, I'm Auto!".bright_yellow().bold());
            println!("{}", "---------------------------".bright_yellow().bold());
            let config = load_am_config().unwrap_or(auto_man::AmConfig::default());
            let mut am = auto_man::Automan::new(".", config).map_err(|e| miette::miette!("{}", e))?;
            am.pull().map_err(|e| miette::miette!("{}", e))?;
        }

        Some(Commands::Reset) => {
            init_logger();
            println!("{}", "---------------------------".bright_yellow().bold());
            println!("{}", "Hello, I'm Auto!".bright_yellow().bold());
            println!("{}", "---------------------------".bright_yellow().bold());
            auto_man::Automan::reset_index().map_err(|e| miette::miette!("{}", e))?;
        }

        Some(Commands::Install { file }) => {
            init_logger();
            println!("{}", "---------------------------".bright_yellow().bold());
            println!("{}", "Hello, I'm Auto!".bright_yellow().bold());
            println!("{}", "---------------------------".bright_yellow().bold());
            auto_man::Automan::install_config(&file).map_err(|e| miette::miette!("{}", e))?;
        }

        None => {
            // Default: Use BigVM REPL (Plan 068 Phase 9.5)
            auto_lang::autovm_repl::main_loop().map_err(|e| miette::miette!("{}", e))?;
        }
    }

    Ok(())
}

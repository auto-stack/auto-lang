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
#[command(
    name = "auto",
    version,
    about = "AutoNexus / Auto CLI\nThe Universal Build Coordinator & Language Environment",
    long_about = None
)]
struct Cli {
    /// Maximum number of errors to display before aborting (default: 20)
    #[arg(short, long, global = true, value_name = "N")]
    error_limit: Option<usize>,

    /// Output format for errors and diagnostics
    #[arg(long, global = true, value_name = "FORMAT")]
    format: Option<OutputFormat>,

    /// Run an Auto script directly via AutoVM
    #[arg(index = 1)]
    file: Option<String>,

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

fn println_logo() {
    println!("{}", "---------------------------".bright_yellow().bold());
    println!("{}", "AutoNexus / Auto CLI".bright_yellow().bold());
    println!("{}", "---------------------------".bright_yellow().bold());
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
enum DeviceAction {
    #[command(about = "List connected hardware devices and ports")]
    List,
    #[command(about = "Select a specific port for deployment")]
    Select { port: String },
}

#[derive(Subcommand, Debug)]
enum EnvAction {
    #[command(about = "Reset AutoMan configuration and index")]
    Reset,
    #[command(about = "Install a custom am.at configuration file")]
    Install { file: String },
    #[command(about = "Manage AutoCache")]
    Cache {
        #[command(subcommand)]
        command: CacheCommands,
    },
}

#[derive(Subcommand, Debug)]
enum Commands {
    // ========== Project Creation ==========
    #[command(about = "Create a new Auto project (app, lib, gear, gadget)")]
    New {
        name: String,
        #[arg(short, long, help = "Project template (e.g. c-app, rs-app, vue-app)")]
        template: Option<String>,
    },
    #[command(about = "Initialize an Auto project in the current directory")]
    Init,

    // ========== Build & Run ==========
    #[command(about = "Compile the project based on pac.at backend", alias = "b")]
    Build {
        #[arg(short, long)]
        dir: Option<String>,
    },
    #[command(about = "Build and run the executable/dev-server", alias = "r")]
    Run {
        #[arg(allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(about = "Remove the .auto/build directory and artifacts")]
    Clean {
        #[arg(short, long)]
        dir: Option<String>,
    },

    // ========== Dependencies ==========
    #[command(about = "Add a dependency to pac.at")]
    Add { package: String },
    #[command(about = "Fetch and resolve all dependencies (Replaces scan/pull)")]
    Fetch,
    #[command(about = "Show the dependency graph")]
    Deps,

    // ========== Hardware & Embedded ==========
    #[command(about = "Manage connected hardware devices and ports")]
    Device {
        #[command(subcommand)]
        action: DeviceAction,
    },

    // ========== Project Utils ==========
    #[command(about = "Show package, backend, and target information", alias = "i")]
    Info {
        #[arg(short, long)]
        target: Option<String>,
    },
    #[command(about = "Open the current project in the default IDE", alias = "o")]
    Open,

    // ========== Environment ==========
    #[command(about = "Upgrade auto.exe toolchain to the latest version")]
    Upgrade,
    #[command(about = "Manage global AutoMan configurations and cache")]
    Env {
        #[command(subcommand)]
        action: EnvAction,
    },

    // ========== Legacy / Dev Tools ==========
    #[command(about = "AutoLang REPL (deprecated - uses TreeWalker Interpreter)", hide = true)]
    OldRepl,
    #[command(about = "Parse Auto to JSON", hide = true)]
    Parse { code: String },
    #[command(about = "Evaluate Auto expression", hide = true)]
    Eval { code: String },
    #[command(about = "Treat File as AutoConfig", hide = true)]
    Config { path: String },
    #[command(about = "Transpile Auto to C", hide = true)]
    C {
        path: String,
        #[arg(short, long, help = "Compilation target", global = false)]
        target: Option<String>,
    },
    #[command(about = "Transpile Auto to Rust", hide = true)]
    Rust { path: String },
    #[command(about = "Transpile Auto to Python", hide = true)]
    Python { path: String },
    #[command(about = "Transpile Auto to JavaScript", hide = true)]
    JavaScript { path: String },
    #[command(about = "Transpile stdlib to C", hide = true)]
    A2cStdlib,
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

    // Execution: Run an Auto script directly via AutoVM
    if let Some(path) = cli.file {
        if matches!(format, OutputFormat::Text) {
            println!("----------------------");
            println!("Running Auto {} ", path);
            println!("----------------------");
        }
        let result = auto_lang::run_file(&path).map_err(|e| {
            if matches!(format, OutputFormat::Json) {
                eprintln!("{}", format_error_json(&e));
                std::process::exit(1);
            }
            to_miette_err(e)
        })?;
        println!("{}", result);
        if matches!(format, OutputFormat::Text) {
            println!();
        }
        return Ok(());
    }

    match cli.command {
        // ========== Project Creation ==========
        Some(Commands::New { name, template }) => {
            init_logger();
            println_logo();
            info!("Creating new project: {}", name);
            if let Some(t) = template {
                auto_man::Automan::create_by_template(&name, &t).map_err(|e| miette::miette!("{}", e))?;
            } else {
                auto_man::Automan::create_app(&name).map_err(|e| miette::miette!("{}", e))?;
            }
        }
        Some(Commands::Init) => {
            init_logger();
            println_logo();
            info!("Initializing Auto project in current directory");
            // For now, we use a default app template for init
            auto_man::Automan::create_app(".").map_err(|e| miette::miette!("{}", e))?;
        }

        // ========== Build & Run ==========
        Some(Commands::Build { dir }) => {
            init_logger();
            println_logo();
            let dir = dir.unwrap_or_else(|| ".".to_string());
            let config = load_am_config().unwrap_or_default();
            let mut am = auto_man::Automan::new(&dir, config).map_err(|e| miette::miette!("{}", e))?;
            am.scan().map_err(|e| miette::miette!("{}", e))?;
            am.build().map_err(|e| miette::miette!("{}", e))?;
        }
        Some(Commands::Run { args }) => {
            init_logger();
            println_logo();
            let config = load_am_config().unwrap_or_default();
            let mut am = auto_man::Automan::new(".", config).map_err(|e| miette::miette!("{}", e))?;
            info!("Running project ...");
            println!();
            println!("------------ output ------------");
            am.run(args).map_err(|e| miette::miette!("{}", e))?;
            println!("------------- end --------------");
        }
        Some(Commands::Clean { dir }) => {
            init_logger();
            println_logo();
            let dir = dir.unwrap_or_else(|| ".".to_string());
            let config = load_am_config().unwrap_or_default();
            let mut am = auto_man::Automan::new(&dir, config).map_err(|e| miette::miette!("{}", e))?;
            am.clean().map_err(|e| miette::miette!("{}", e))?;
        }

        // ========== Dependencies ==========
        Some(Commands::Add { package }) => {
            init_logger();
            println_logo();
            info!("Adding dependency: {}", package);
            // TODO: Implement Automan::add_dependency
            miette::bail!("'add' command is not yet implemented in the library");
        }
        Some(Commands::Fetch) => {
            init_logger();
            println_logo();
            info!("Fetching dependencies...");
            let config = load_am_config().unwrap_or_default();
            let mut am = auto_man::Automan::new(".", config).map_err(|e| miette::miette!("{}", e))?;
            am.pull().map_err(|e| miette::miette!("{}", e))?;
            am.scan().map_err(|e| miette::miette!("{}", e))?;
        }
        Some(Commands::Deps) => {
            init_logger();
            println_logo();
            let config = load_am_config().unwrap_or_default();
            auto_man::Automan::list_deps(&config).map_err(|e| miette::miette!("{}", e))?;
        }

        // ========== Hardware & Embedded ==========
        Some(Commands::Device { action }) => {
            init_logger();
            println_logo();
            let config = load_am_config().unwrap_or_default();
            match action {
                DeviceAction::List => {
                    auto_man::Automan::list_devices(&config).map_err(|e| miette::miette!("{}", e))?;
                }
                DeviceAction::Select { port } => {
                    let mut am = auto_man::Automan::new(".", config).map_err(|e| miette::miette!("{}", e))?;
                    am.set_port(port.into()).map_err(|e| miette::miette!("{}", e))?;
                    info!("Port updated successfully");
                }
            }
        }

        // ========== Project Utils ==========
        Some(Commands::Info { target }) => {
            init_logger();
            println_logo();
            let config = load_am_config().unwrap_or_default();
            let am = auto_man::Automan::new(".", config).map_err(|e| miette::miette!("{}", e))?;
            am.info(target).map_err(|e| miette::miette!("{}", e))?;
        }
        Some(Commands::Open) => {
            init_logger();
            println_logo();
            let config = load_am_config().unwrap_or_default();
            let mut am = auto_man::Automan::new(".", config).map_err(|e| miette::miette!("{}", e))?;
            am.open_ide().map_err(|e| miette::miette!("{}", e))?;
        }

        // ========== Environment ==========
        Some(Commands::Upgrade) => {
            init_logger();
            println_logo();
            auto_man::upgrade().map_err(|e| miette::miette!("{}", e))?;
        }
        Some(Commands::Env { action }) => {
            init_logger();
            println_logo();
            match action {
                EnvAction::Reset => {
                    auto_man::Automan::reset_index().map_err(|e| miette::miette!("{}", e))?;
                }
                EnvAction::Install { file } => {
                    auto_man::Automan::install_config(&file).map_err(|e| miette::miette!("{}", e))?;
                }
                EnvAction::Cache { command } => {
                    let config = load_am_config().unwrap_or_default();
                    let mut am = auto_man::Automan::new(".", config).map_err(|e| miette::miette!("{}", e))?;
                    match command {
                        CacheCommands::Stats => am.cache_stats().map_err(|e| miette::miette!("{}", e))?,
                        CacheCommands::List { type_, limit } => am.cache_list(type_, limit).map_err(|e| miette::miette!("{}", e))?,
                        CacheCommands::Prune => am.cache_prune().map_err(|e| miette::miette!("{}", e))?,
                        CacheCommands::Clear => am.cache_clear().map_err(|e| miette::miette!("{}", e))?,
                        CacheCommands::Inspect { name } => am.cache_inspect(&name).map_err(|e| miette::miette!("{}", e))?,
                        CacheCommands::Verify => am.cache_verify().map_err(|e| miette::miette!("{}", e))?,
                    }
                }
            }
        }

        // ========== Legacy / Dev Tools ==========
        Some(Commands::Parse { code }) => {
            if matches!(format, OutputFormat::Text) {
                println!("Parsing Auto {} to JSON", code);
            }
            let json = auto_lang::run(&code).map_err(|e| {
                if matches!(format, OutputFormat::Json) {
                    eprintln!("{}", format_error_json(&e));
                    std::process::exit(1);
                }
                to_miette_err(e)
            })?;
            println!("{}", json);
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

        None => {
            // Default: Use BigVM REPL
            auto_lang::autovm_repl::main_loop().map_err(|e| miette::miette!("{}", e))?;
        }
    }

    Ok(())
}

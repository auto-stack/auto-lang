use auto_lang::error::AutoError;
use clap::{Parser, Subcommand, ValueEnum};
use miette::{Diagnostic, MietteHandlerOpts, Result};
use serde_json::{json, Value};
use colored::Colorize;
use log::info;

mod cmd_a2c_stdlib;
mod cmd_block;
mod cmd_ui;

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

/// Format success result as JSON for AI consumption
fn format_success_json<T: serde::Serialize>(result: T) -> String {
    json!({
        "status": "success",
        "result": result
    }).to_string()
}

/// Output success result in appropriate format based on AI mode
fn output_success(ai_mode: bool, result: &str) {
    if ai_mode {
        println!("{}", format_success_json(result));
    } else {
        println!("{}", result);
    }
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

    /// AI-friendly output mode: JSON structured output, suppress human-readable info
    /// Equivalent to --format json with additional output suppression
    #[arg(long = "ai", global = true)]
    ai: bool,

    /// Enable VM debug logging (shows task spawning, message handling, replies)
    #[arg(short = 'D', long = "debug", global = true)]
    debug: bool,

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
enum TransTarget {
    Ts {
        #[arg(short, long, help = "Output file path (default: same name with .ts extension)")]
        output: Option<String>,
    },
    C {
        #[arg(short, long, help = "Output file path (default: same name with .c extension)")]
        output: Option<String>,
    },
    Rust {
        #[arg(short, long, help = "Output file path (default: same name with .rs extension)")]
        output: Option<String>,
        #[arg(short, long, help = "Merge all discovered modules into a single .rs file")]
        merge: bool,
    },
    Python {
        #[arg(short, long, help = "Output file path (default: same name with .py extension)")]
        output: Option<String>,
    },
    Js {
        #[arg(short, long, help = "Output file path (default: same name with .js extension)")]
        output: Option<String>,
    },
    Gd {
        #[arg(short, long, help = "Output file path (default: same name with .gd extension)")]
        output: Option<String>,
    },
    Tscn {
        #[arg(short, long, help = "Output file path (default: same name with .tscn extension)")]
        output: Option<String>,
    },
    /// Emit both .tscn (from any `scene`) and .gd (from functions) for one .at file.
    Godot {
        #[arg(short, long, help = "Output base name (default: source name; writes <base>.tscn + <base>.gd)")]
        output: Option<String>,
    },
}

#[derive(Subcommand, Debug)]
enum UiAction {
    /// Generate self-contained widget SFCs into an output directory
    Build {
        #[arg(long, help = "Target framework (default: vue)", default_value = "vue")]
        target: String,
        #[arg(long, help = "Output directory (default: packages/widgets/registry)", default_value = "packages/widgets/registry")]
        out: String,
        #[arg(long, value_delimiter = ',', help = "Comma-separated widget names (default: all registered)")]
        widgets: Vec<String>,
    },
    /// List registered library widgets
    List,
}

#[derive(Subcommand, Debug)]
enum BlockAction {
    /// List block packages (grouped by kind)
    List,
    /// Print a block's spec + variants + gotchas (the agent skill interface)
    Show {
        /// `kind/name` of the block (e.g. `form/login`)
        key: String,
    },
    /// Copy a reference implementation into a consumer project (adopt-and-edit)
    Add {
        /// `kind/name` of the block (e.g. `form/login`)
        key: String,
        /// Variant to copy (default: the package's first variant)
        #[arg(long)]
        reference: Option<String>,
        /// Output directory (default: src/front/blocks)
        #[arg(long, default_value = "src/front/blocks")]
        out: String,
    },
    /// Static acceptance check on a generated/copied .at (agent repair-loop gate)
    Check {
        /// Path to the .at file to check
        file: String,
        /// Optional `kind/name` spec to check extension-point EDIT markers against
        #[arg(long)]
        spec: Option<String>,
    },
}

#[derive(Subcommand, Debug)]
enum Commands {
    // ========== Project Creation ==========
    #[command(about = "Create a new Auto project (app, lib, gear, gadget)")]
    New {
        name: String,
        #[arg(short, long, help = "Project template (app, jet, capp, lib, clib)")]
        template: Option<String>,
    },
    #[command(about = "Initialize an Auto project in the current directory")]
    Init {
        #[arg(short, long, help = "Project template (e.g. app, jet)")]
        template: Option<String>,
    },

    // ========== Build & Run ==========
    #[command(about = "Compile the project based on pac.at render target", alias = "b")]
    Build {
        #[arg(short, long)]
        dir: Option<String>,
        #[arg(short, long)]
        port: Option<String>,
        #[arg(short = 'B', long = "back-port", help = "Backend HTTP API server port (default 8080)")]
        back_port: Option<String>,
        #[arg(short = 'F', long = "front-port", help = "Frontend dev server port (default 3000)")]
        front_port: Option<String>,
        #[arg(short, long, help = "Render target to use (vue, rust, vm, jet, arkts, tauri)")]
        render: Option<String>,
    },
    #[command(about = "Build and run the executable/dev-server", alias = "r")]
    Run {
        #[arg(short, long)]
        dir: Option<String>,
        #[arg(short, long)]
        port: Option<String>,
        #[arg(short = 'B', long = "back-port", help = "Backend HTTP API server port (default 8080)")]
        back_port: Option<String>,
        #[arg(short = 'F', long = "front-port", help = "Frontend dev server port (default 3000)")]
        front_port: Option<String>,
        #[arg(short, long, help = "Render target to use (vue, rust, vm, jet, arkts, tauri)")]
        render: Option<String>,
        #[arg(long, help = "Backend server mode: vm (AutoVM HTTP) or rust (a2r, default)")]
        server: Option<String>,
        #[arg(long, help = "Plan 340: merge frontend+backend VM in-process (default true). --no-merge uses HTTP between VMs")]
        #[arg(long = "no-merge", action = clap::ArgAction::SetTrue, help = "Plan 340: use HTTP between VMs (split mode)")]
        no_merge: bool,
        #[arg(allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(about = "Run all #[test] functions in the project", alias = "t")]
    Test {
        #[arg(short, long, help = "Source file or directory to test")]
        dir: Option<String>,
        #[arg(short, long, help = "Run only tests matching this filter")]
        filter: Option<String>,
        #[arg(short = 'v', long, help = "Show test output (print statements)")]
        verbose: bool,
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
    #[command(about = "Export the project to a specific format (cmake, iar, ghs)")]
    Export {
        #[arg(short, long, help = "Name of the port to export")]
        port: String,
        #[arg(short, long, help = "Format to export to (cmake, iar, ghs)")]
        format: String,
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
    // #[command(about = "Upgrade auto.exe toolchain to the latest version")]
    // Upgrade,  // NOTE: disabled — zip dependency removed
    #[command(about = "Manage global AutoMan configurations and cache")]
    Env {
        #[command(subcommand)]
        action: EnvAction,
    },

    // ========== Code Generation ==========
    #[command(about = "Generate code from .at files (kotlin for jet backend)")]
    Gen {
        #[arg(short, long, help = "Output directory (default: dist for vue, current dir for jet)")]
        output: Option<String>,
        #[arg(short, long, help = "Generate full project structure")]
        project: bool,
    },

    // ========== AutoUI Widget Library (Plan 331) ==========
    #[command(about = "AutoUI widget library commands")]
    Ui {
        #[command(subcommand)]
        action: UiAction,
    },

    // ========== AutoUI Blocks (Plan 343, Design 17) ==========
    #[command(about = "AutoUI block catalog commands (Skill-tier)")]
    Block {
        #[command(subcommand)]
        action: BlockAction,
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
    // ========== Transpile (single file) ==========
    #[command(about = "Transpile a single .at file to a target language")]
    Trans {
        /// Input .at file to transpile
        #[arg(short, long, help = "Input .at file to transpile")]
        path: String,
        #[command(subcommand)]
        target: TransTarget,
    },
    #[command(about = "Transpile Auto to C", hide = true)]
    C {
        path: String,
        #[arg(short, long, help = "Compilation target", global = false)]
        target: Option<String>,
    },
    #[command(about = "Transpile Auto to Rust", hide = true)]
    Rust { path: String },
    #[command(about = "Transpile Rust to AutoLang (r2a)", hide = true)]
    R2a {
        path: String,
        #[arg(short, long, help = "Output file path")]
        output: Option<String>,
    },
    #[command(about = "Transpile Auto to Python", hide = true)]
    Python { path: String },
    #[command(about = "Transpile Auto to JavaScript", hide = true)]
    JavaScript { path: String },
    #[command(about = "Transpile Auto to GDScript (Godot 4.x)", hide = true)]
    GDScript { path: String },
    #[command(about = "Transpile stdlib to C", hide = true)]
    A2cStdlib,

    // ========== Debug (Plan 199) ==========
    #[command(about = "Debug an Auto program with interactive debugger", alias = "dbg")]
    Debug {
        /// Path to the .at file to debug
        file: String,
        /// Agent-friendly JSON mode: each pause emits JSON state, read commands from stdin
        #[arg(long = "agent", short = 'a')]
        agent: bool,
    },

    // ========== C FFI Bindgen (Plan 216) ==========
    #[command(about = "List available C FFI bindings from manifests")]
    Cffi {
        /// Show functions for a specific header (e.g., "string.h", "math.h")
        #[arg(long)]
        header: Option<String>,
    },

    // ========== MCP Server (Plan 265) ==========
    #[command(about = "Start AutoVM MCP server (stdio transport for AI agents)")]
    Mcp,

    // ========== Daemon + CLI (Plan 269) ==========
    #[command(about = "Start AutoVM daemon server (stateful VM, persistent sessions)")]
    Serve {
        /// Run in foreground (don't daemonize)
        #[arg(long)]
        foreground: bool,
        /// Run in stdio mode (used internally by auto req)
        #[arg(long)]
        stdio: bool,
        /// Named pipe name (Windows) or socket name (Unix)
        #[arg(short, long, default_value = "autovm")]
        pipe_name: String,
        /// Maximum number of concurrent sessions
        #[arg(long, default_value_t = 20)]
        max_sessions: usize,
        /// Session idle timeout in seconds (0 = no timeout)
        #[arg(long, default_value_t = 1800)]
        timeout: u64,
    },

    #[command(about = "Send request to AutoVM daemon (eval code, inspect sessions)")]
    Req {
        /// Session ID to use
        #[arg(short, long)]
        session: Option<String>,
        /// Named pipe name (must match daemon)
        #[arg(short = 'p', long, default_value = "autovm")]
        pipe_name: String,
        /// Create a new session and print its ID
        #[arg(long)]
        new_session: bool,
        /// Inspect session state
        #[arg(long)]
        inspect: bool,
        /// Reset session
        #[arg(long)]
        reset: bool,
        /// Delete session
        #[arg(long)]
        delete: bool,
        /// Export session as .at source
        #[arg(long)]
        snapshot: bool,
        /// List active sessions
        #[arg(long)]
        list: bool,
        /// Output as JSON (machine-readable)
        #[arg(long)]
        json: bool,
        /// Auto code to evaluate
        code: Option<String>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // The binary is linked with a 4MB stack (see build.rs) so the main thread
    // has enough headroom for both UI (iced needs the main thread) and the
    // parser's deep recursion on complex UI files.
    real_main(cli)
}

fn real_main(cli: Cli) -> Result<()> {
    // Determine AI mode: -ai flag or --format json
    let ai_mode = cli.ai || matches!(cli.format, Some(OutputFormat::Json));

    miette::set_hook(Box::new(move |_| {
        Box::new(MietteHandlerOpts::new().terminal_links(true).build())
    }))
    .ok();

    // Set error limit from CLI if provided
    if let Some(limit) = cli.error_limit {
        auto_lang::set_error_limit(limit);
    }

    // Enable VM debug logging if requested
    if cli.debug {
        auto_lang::set_vm_debug(true);
    }

    // Execution: Run an Auto script directly via AutoVM
    if let Some(path) = cli.file {
        if !ai_mode {
            println!("----------------------");
            println!("Running Auto {} ", path);
            println!("----------------------");
        }
        let result = auto_lang::run_file(&path).map_err(|e| {
            if ai_mode {
                eprintln!("{}", format_error_json(&e));
                std::process::exit(1);
            }
            to_miette_err(e)
        })?;
        output_success(ai_mode, &result);
        if !ai_mode {
            println!();
        }
        return Ok(());
    }

    match cli.command {
        // ========== Project Creation ==========
        Some(Commands::New { name, template }) => {
            if !ai_mode {
                init_logger();
                println_logo();
                info!("Creating new project: {}", name);
            }
            if let Some(t) = template {
                auto_man::Automan::create_by_template(&name, &t).map_err(|e| {
                    if ai_mode {
                        eprintln!("{}", format_error_json(&AutoError::Msg(e.to_string())));
                        std::process::exit(1);
                    }
                    miette::miette!("{}", e)
                })?;
            } else {
                auto_man::Automan::create_app(&name).map_err(|e| {
                    if ai_mode {
                        eprintln!("{}", format_error_json(&AutoError::Msg(e.to_string())));
                        std::process::exit(1);
                    }
                    miette::miette!("{}", e)
                })?;
            }
            if ai_mode {
                println!("{}", format_success_json(json!({"message": "Project created", "path": name})));
            }
        }
        Some(Commands::Init { template }) => {
            if !ai_mode {
                init_logger();
                println_logo();
                info!("Initializing Auto project in current directory");
            }
            if let Some(t) = template {
                auto_man::Automan::create_by_template(".", &t).map_err(|e| {
                    if ai_mode {
                        eprintln!("{}", format_error_json(&AutoError::Msg(e.to_string())));
                        std::process::exit(1);
                    }
                    miette::miette!("{}", e)
                })?;
            } else {
                auto_man::Automan::create_app(".").map_err(|e| {
                    if ai_mode {
                        eprintln!("{}", format_error_json(&AutoError::Msg(e.to_string())));
                        std::process::exit(1);
                    }
                    miette::miette!("{}", e)
                })?;
            }
            if ai_mode {
                println!("{}", format_success_json(json!({"message": "Project initialized"})));
            }
        }

        // ========== Build & Run ==========
        Some(Commands::Build { dir, port, back_port, front_port, render }) => {
            if !ai_mode {
                init_logger();
                println_logo();
            }
            let dir = dir.unwrap_or_else(|| ".".to_string());
            let config = load_am_config().unwrap_or_default();
            let mut am = auto_man::Automan::new(&dir, config).map_err(|e| {
                if ai_mode {
                    eprintln!("{}", format_error_json(&AutoError::Msg(e.to_string())));
                    std::process::exit(1);
                }
                miette::miette!("{}", e)
            })?;
            if let Some(p) = port {
                am.set_port(p.into()).map_err(|e| {
                    if ai_mode {
                        eprintln!("{}", format_error_json(&AutoError::Msg(e.to_string())));
                        std::process::exit(1);
                    }
                    miette::miette!("{}", e)
                })?;
            }
            if let Some(b) = render {
                am.set_render(b);
            }
            // Plan 330: bake ports into generated artifacts the same way `run`
            // does, for symmetry (so `build` then manual run matches).
            if let Some(p) = &back_port {
                if p.trim().parse::<u16>().is_err() {
                    return Err(miette::miette!(
                        "Invalid backend port '{}': must be a number 0-65535", p
                    ));
                }
                std::env::set_var("AUTO_HTTP_PORT", p.trim());
                println!("  Backend API server port: {}", p.trim());
            }
            if let Some(p) = &front_port {
                if p.trim().parse::<u16>().is_err() {
                    return Err(miette::miette!(
                        "Invalid frontend port '{}': must be a number 0-65535", p
                    ));
                }
                std::env::set_var("AUTO_FRONT_PORT", p.trim());
                println!("  Frontend dev server port: {}", p.trim());
            }
            am.scan().map_err(|e| {
                if ai_mode {
                    eprintln!("{}", format_error_json(&AutoError::Msg(e.to_string())));
                    std::process::exit(1);
                }
                miette::miette!("{}", e)
            })?;
            am.build().map_err(|e| {
                if ai_mode {
                    eprintln!("{}", format_error_json(&AutoError::Msg(e.to_string())));
                    std::process::exit(1);
                }
                miette::miette!("{}", e)
            })?;
            if ai_mode {
                println!("{}", format_success_json(json!({"message": "Build completed"})));
            }
        }
        Some(Commands::Run { dir, port, back_port, front_port, render, server, no_merge, args }) => {
            if !ai_mode {
                init_logger();
                println_logo();
            }
            let dir = dir.unwrap_or_else(|| ".".to_string());
            let config = load_am_config().unwrap_or_default();
            let mut am = auto_man::Automan::new(&dir, config).map_err(|e| {
                if ai_mode {
                    eprintln!("{}", format_error_json(&AutoError::Msg(e.to_string())));
                    std::process::exit(1);
                }
                miette::miette!("{}", e)
            })?;
            if let Some(p) = port {
                am.set_port(p.into()).map_err(|e| {
                    if ai_mode {
                        eprintln!("{}", format_error_json(&AutoError::Msg(e.to_string())));
                        std::process::exit(1);
                    }
                    miette::miette!("{}", e)
                })?;
            }
            if let Some(b) = render {
                am.set_render(b);
            }
            // Plan 327: --server=vm uses AutoVM HTTP server (with module
            // flattening for use db), --server=rust (default) uses a2r.
            let vm_server_mode = server.as_deref() == Some("vm");
            if vm_server_mode {
                am.set_vm_server_mode(true);
            }
            // Plan 340: --merge/--no-merge controls VM+VM in-process merging.
            // Default (no --no-merge) keeps the existing fast in-process path.
            // --no-merge sets AUTO_VM_MERGE=0 so run_file_dynamic_ui rewrites
            // #[api] calls to HTTP requests against the separate backend.
            let merge_mode = !no_merge;
            std::env::set_var("AUTO_VM_MERGE", if merge_mode { "1" } else { "0" });
            if !merge_mode {
                println!("  {} split mode: frontend↔backend over HTTP", "→".bright_cyan());
            }
            // Plan 330: `--back-port`/`-B` selects the backend HTTP API port.
            // We inject it into AUTO_HTTP_PORT so all three port consumers stay
            // in sync: the generated backend main.rs (reads it at runtime, via
            // inherited child env), the vite proxy, and the rust-ui client (both
            // read crate::util::http_port() at generation time, which happens
            // inside am.run below). Falls back to 8080 when unset.
            if let Some(p) = &back_port {
                if p.trim().parse::<u16>().is_err() {
                    return Err(miette::miette!(
                        "Invalid backend port '{}': must be a number 0-65535", p
                    ));
                }
                std::env::set_var("AUTO_HTTP_PORT", p.trim());
                println!("  Backend API server port: {}", p.trim());
            }
            // Plan 330: `--front-port`/`-F` selects the frontend dev server port.
            // Injected into AUTO_FRONT_PORT, which the generated vite config reads
            // at dev-server start. Falls back to 3000 when unset.
            if let Some(p) = &front_port {
                if p.trim().parse::<u16>().is_err() {
                    return Err(miette::miette!(
                        "Invalid frontend port '{}': must be a number 0-65535", p
                    ));
                }
                std::env::set_var("AUTO_FRONT_PORT", p.trim());
                println!("  Frontend dev server port: {}", p.trim());
            }
            if !ai_mode {
                info!("Running project ...");
                println!();
                println!("------------ output ------------");
            }
            am.run(args).map_err(|e| {
                if ai_mode {
                    eprintln!("{}", format_error_json(&AutoError::Msg(e.to_string())));
                    std::process::exit(1);
                }
                miette::miette!("{}", e)
            })?;
            if !ai_mode {
                println!("------------- end --------------");
            }
        }
        Some(Commands::Test { dir, filter, verbose }) => {
            let target = dir.unwrap_or_else(|| ".".to_string());

            // Collect .at files to test
            let path = std::path::Path::new(&target);
            let files: Vec<String> = if path.is_file() {
                // File mode: test a single file
                vec![target.clone()]
            } else if path.is_dir() {
                // Directory mode: discover all .at files recursively
                let mut found = Vec::new();
                fn collect_at_files(dir: &std::path::Path, out: &mut Vec<String>) {
                    if let Ok(entries) = std::fs::read_dir(dir) {
                        for entry in entries.flatten() {
                            let p = entry.path();
                            if p.is_dir() {
                                collect_at_files(&p, out);
                            } else if p.extension().map_or(false, |e| e == "at") {
                                out.push(p.to_string_lossy().to_string());
                            }
                        }
                    }
                }
                collect_at_files(path, &mut found);
                found.sort();
                if found.is_empty() {
                    eprintln!("error: no .at files found in '{}'", target);
                    std::process::exit(1);
                }
                found
            } else {
                eprintln!("error: '{}' not found", target);
                std::process::exit(1);
            };

            let start = std::time::Instant::now();
            let mut all_results = auto_lang::test_runner::TestResult::default();
            let mut test_files = 0;
            let mut failed_files = 0;
            let multi_file = files.len() > 1;

            for file in &files {
                match auto_lang::test_file(file) {
                    Ok(result) => {
                        // Skip files with no tests
                        if result.reports.is_empty() {
                            continue;
                        }
                        test_files += 1;
                        let mut file_failed = 0;
                        if multi_file {
                            let file_name = std::path::Path::new(file)
                                .file_name().unwrap_or_default().to_string_lossy();
                            println!("\nRunning {} ({} tests):", file_name, result.reports.len());
                        }
                        for report in &result.reports {
                            if let Some(f) = &filter {
                                if !report.qualified_name.contains(f.as_str()) {
                                    continue;
                                }
                            }
                            if verbose && !report.stdout.is_empty() {
                                println!("--- {} stdout ---", report.qualified_name);
                                println!("{}", report.stdout);
                            }
                            match &report.outcome {
                                auto_lang::test_runner::TestOutcome::Passed => {}
                                auto_lang::test_runner::TestOutcome::Failed(_) => file_failed += 1,
                            }
                            all_results.reports.push(report.clone());
                        }
                        if file_failed > 0 {
                            failed_files += 1;
                        }
                    }
                    Err(_) => {
                        // Compile errors are expected for non-test files (stdlib, examples, etc.)
                        // Only report errors in single-file mode
                        if !multi_file {
                            eprintln!("error: failed to compile {}", file);
                            std::process::exit(1);
                        }
                    }
                }
            }

            let elapsed = start.elapsed().as_millis();
            print!("{}", auto_lang::test_runner::format_test_report(&all_results, elapsed));

            if multi_file {
                println!("{} test file(s), {} file(s) had failures", test_files, failed_files);
            }

            // File-based tests (VM, A2R, A2C, A2TS) now discovered via tests/*.at files
            // Plan 263 Phase 2-3: tests/a2r_tests.at, tests/vm_tests.at, tests/a2c_tests.at, tests/a2ts_tests.at

            if all_results.has_failures() {
                std::process::exit(1);
            }
        }
        Some(Commands::Clean { dir }) => {
            if !ai_mode {
                init_logger();
                println_logo();
            }
            let dir = dir.unwrap_or_else(|| ".".to_string());
            let config = load_am_config().unwrap_or_default();
            let mut am = auto_man::Automan::new(&dir, config).map_err(|e| {
                if ai_mode {
                    eprintln!("{}", format_error_json(&AutoError::Msg(e.to_string())));
                    std::process::exit(1);
                }
                miette::miette!("{}", e)
            })?;
            am.clean().map_err(|e| {
                if ai_mode {
                    eprintln!("{}", format_error_json(&AutoError::Msg(e.to_string())));
                    std::process::exit(1);
                }
                miette::miette!("{}", e)
            })?;
            if ai_mode {
                println!("{}", format_success_json(json!({"message": "Clean completed"})));
            }
        }

        // ========== Dependencies ==========
        Some(Commands::Add { package }) => {
            if !ai_mode {
                init_logger();
                println_logo();
                info!("Adding dependency: {}", package);
            }
            // TODO: Implement Automan::add_dependency
            if ai_mode {
                eprintln!("{}", format_error_json(&AutoError::Msg("'add' command is not yet implemented".to_string())));
                std::process::exit(1);
            }
            miette::bail!("'add' command is not yet implemented in the library");
        }
        Some(Commands::Fetch) => {
            if !ai_mode {
                init_logger();
                println_logo();
                info!("Fetching dependencies...");
            }
            let config = load_am_config().unwrap_or_default();
            let mut am = auto_man::Automan::new(".", config).map_err(|e| {
                if ai_mode {
                    eprintln!("{}", format_error_json(&AutoError::Msg(e.to_string())));
                    std::process::exit(1);
                }
                miette::miette!("{}", e)
            })?;
            am.pull().map_err(|e| {
                if ai_mode {
                    eprintln!("{}", format_error_json(&AutoError::Msg(e.to_string())));
                    std::process::exit(1);
                }
                miette::miette!("{}", e)
            })?;
            am.scan().map_err(|e| {
                if ai_mode {
                    eprintln!("{}", format_error_json(&AutoError::Msg(e.to_string())));
                    std::process::exit(1);
                }
                miette::miette!("{}", e)
            })?;
            if ai_mode {
                println!("{}", format_success_json(json!({"message": "Dependencies fetched"})));
            }
        }
        Some(Commands::Deps) => {
            if !ai_mode {
                init_logger();
                println_logo();
            }
            let config = load_am_config().unwrap_or_default();
            auto_man::Automan::list_deps(&config).map_err(|e| {
                if ai_mode {
                    eprintln!("{}", format_error_json(&AutoError::Msg(e.to_string())));
                    std::process::exit(1);
                }
                miette::miette!("{}", e)
            })?;
        }

        // ========== Hardware & Embedded ==========
        Some(Commands::Device { action }) => {
            if !ai_mode {
                init_logger();
                println_logo();
            }
            let config = load_am_config().unwrap_or_default();
            match action {
                DeviceAction::List => {
                    auto_man::Automan::list_devices(&config).map_err(|e| {
                        if ai_mode {
                            eprintln!("{}", format_error_json(&AutoError::Msg(e.to_string())));
                            std::process::exit(1);
                        }
                        miette::miette!("{}", e)
                    })?;
                }
                DeviceAction::Select { port } => {
                    let mut am = auto_man::Automan::new(".", config).map_err(|e| {
                        if ai_mode {
                            eprintln!("{}", format_error_json(&AutoError::Msg(e.to_string())));
                            std::process::exit(1);
                        }
                        miette::miette!("{}", e)
                    })?;
                    am.set_port(port.clone().into()).map_err(|e| {
                        if ai_mode {
                            eprintln!("{}", format_error_json(&AutoError::Msg(e.to_string())));
                            std::process::exit(1);
                        }
                        miette::miette!("{}", e)
                    })?;
                    if !ai_mode {
                        info!("Port updated successfully");
                    }
                    if ai_mode {
                        println!("{}", format_success_json(json!({"message": "Port selected", "port": port})));
                    }
                }
            }
        }
        Some(Commands::Export { port, format }) => {
            if !ai_mode {
                init_logger();
                println_logo();
            }
            let config = load_am_config().unwrap_or_default();
            let mut am = auto_man::Automan::new(".", config).map_err(|e| {
                if ai_mode {
                    eprintln!("{}", format_error_json(&AutoError::Msg(e.to_string())));
                    std::process::exit(1);
                }
                miette::miette!("{}", e)
            })?;
            am.export(port.clone(), format.clone()).map_err(|e| {
                if ai_mode {
                    eprintln!("{}", format_error_json(&AutoError::Msg(e.to_string())));
                    std::process::exit(1);
                }
                miette::miette!("{}", e)
            })?;
            if ai_mode {
                println!("{}", format_success_json(json!({"message": "Export completed", "port": port, "format": format})));
            }
        }

        // ========== Project Utils ==========
        Some(Commands::Info { target }) => {
            if !ai_mode {
                init_logger();
                println_logo();
            }
            let config = load_am_config().unwrap_or_default();
            let am = auto_man::Automan::new(".", config).map_err(|e| {
                if ai_mode {
                    eprintln!("{}", format_error_json(&AutoError::Msg(e.to_string())));
                    std::process::exit(1);
                }
                miette::miette!("{}", e)
            })?;
            am.info(target).map_err(|e| {
                if ai_mode {
                    eprintln!("{}", format_error_json(&AutoError::Msg(e.to_string())));
                    std::process::exit(1);
                }
                miette::miette!("{}", e)
            })?;
        }
        Some(Commands::Open) => {
            if !ai_mode {
                init_logger();
                println_logo();
            }
            let config = load_am_config().unwrap_or_default();
            let mut am = auto_man::Automan::new(".", config).map_err(|e| {
                if ai_mode {
                    eprintln!("{}", format_error_json(&AutoError::Msg(e.to_string())));
                    std::process::exit(1);
                }
                miette::miette!("{}", e)
            })?;
            am.open_ide().map_err(|e| {
                if ai_mode {
                    eprintln!("{}", format_error_json(&AutoError::Msg(e.to_string())));
                    std::process::exit(1);
                }
                miette::miette!("{}", e)
            })?;
            if ai_mode {
                println!("{}", format_success_json(json!({"message": "IDE opened"})));
            }
        }

        // ========== Environment ==========
        // NOTE: Upgrade disabled — zip dependency removed
        // Some(Commands::Upgrade) => {
        //     if !ai_mode {
        //         init_logger();
        //         println_logo();
        //     }
        //     auto_man::upgrade().map_err(|e| {
        //         if ai_mode {
        //             eprintln!("{}", format_error_json(&AutoError::Msg(e.to_string())));
        //             std::process::exit(1);
        //         }
        //         miette::miette!("{}", e)
        //     })?;
        //     if ai_mode {
        //         println!("{}", format_success_json(json!({"message": "Upgrade completed"})));
        //     }
        // }
        Some(Commands::Env { action }) => {
            if !ai_mode {
                init_logger();
                println_logo();
            }
            match action {
                EnvAction::Reset => {
                    auto_man::Automan::reset_index().map_err(|e| {
                        if ai_mode {
                            eprintln!("{}", format_error_json(&AutoError::Msg(e.to_string())));
                            std::process::exit(1);
                        }
                        miette::miette!("{}", e)
                    })?;
                    if ai_mode {
                        println!("{}", format_success_json(json!({"message": "Index reset"})));
                    }
                }
                EnvAction::Install { file } => {
                    auto_man::Automan::install_config(&file).map_err(|e| {
                        if ai_mode {
                            eprintln!("{}", format_error_json(&AutoError::Msg(e.to_string())));
                            std::process::exit(1);
                        }
                        miette::miette!("{}", e)
                    })?;
                    if ai_mode {
                        println!("{}", format_success_json(json!({"message": "Config installed", "file": file})));
                    }
                }
                EnvAction::Cache { command } => {
                    let config = load_am_config().unwrap_or_default();
                    let mut am = auto_man::Automan::new(".", config).map_err(|e| {
                        if ai_mode {
                            eprintln!("{}", format_error_json(&AutoError::Msg(e.to_string())));
                            std::process::exit(1);
                        }
                        miette::miette!("{}", e)
                    })?;
                    match command {
                        CacheCommands::Stats => {
                            am.cache_stats().map_err(|e| {
                                if ai_mode {
                                    eprintln!("{}", format_error_json(&AutoError::Msg(e.to_string())));
                                    std::process::exit(1);
                                }
                                miette::miette!("{}", e)
                            })?;
                        }
                        CacheCommands::List { type_, limit } => {
                            am.cache_list(type_, limit).map_err(|e| {
                                if ai_mode {
                                    eprintln!("{}", format_error_json(&AutoError::Msg(e.to_string())));
                                    std::process::exit(1);
                                }
                                miette::miette!("{}", e)
                            })?;
                        }
                        CacheCommands::Prune => {
                            am.cache_prune().map_err(|e| {
                                if ai_mode {
                                    eprintln!("{}", format_error_json(&AutoError::Msg(e.to_string())));
                                    std::process::exit(1);
                                }
                                miette::miette!("{}", e)
                            })?;
                            if ai_mode {
                                println!("{}", format_success_json(json!({"message": "Cache pruned"})));
                            }
                        }
                        CacheCommands::Clear => {
                            am.cache_clear().map_err(|e| {
                                if ai_mode {
                                    eprintln!("{}", format_error_json(&AutoError::Msg(e.to_string())));
                                    std::process::exit(1);
                                }
                                miette::miette!("{}", e)
                            })?;
                            if ai_mode {
                                println!("{}", format_success_json(json!({"message": "Cache cleared"})));
                            }
                        }
                        CacheCommands::Inspect { name } => {
                            am.cache_inspect(&name).map_err(|e| {
                                if ai_mode {
                                    eprintln!("{}", format_error_json(&AutoError::Msg(e.to_string())));
                                    std::process::exit(1);
                                }
                                miette::miette!("{}", e)
                            })?;
                        }
                        CacheCommands::Verify => {
                            am.cache_verify().map_err(|e| {
                                if ai_mode {
                                    eprintln!("{}", format_error_json(&AutoError::Msg(e.to_string())));
                                    std::process::exit(1);
                                }
                                miette::miette!("{}", e)
                            })?;
                            if ai_mode {
                                println!("{}", format_success_json(json!({"message": "Cache verified"})));
                            }
                        }
                    }
                }
            }
        }

        // ========== Code Generation ==========
        Some(Commands::Gen { output, project }) => {
            if !ai_mode {
                init_logger();
                println_logo();
            }
            let config = load_am_config().unwrap_or_default();
            let am = auto_man::Automan::new(".", config).map_err(|e| {
                if ai_mode {
                    eprintln!("{}", format_error_json(&AutoError::Msg(e.to_string())));
                    std::process::exit(1);
                }
                miette::miette!("{}", e)
            })?;
            am.gen(output, project).map_err(|e| {
                if ai_mode {
                    eprintln!("{}", format_error_json(&AutoError::Msg(e.to_string())));
                    std::process::exit(1);
                }
                miette::miette!("{}", e)
            })?;
            if ai_mode {
                println!("{}", format_success_json(json!({"message": "Code generated"})));
            }
        }

        // ========== AutoUI Widget Library (Plan 331) ==========
        Some(Commands::Ui { action }) => {
            cmd_ui::run(action)?;
        }

        // ========== AutoUI Blocks (Plan 343, Design 17) ==========
        Some(Commands::Block { action }) => {
            cmd_block::run(action)?;
        }

        // ========== Legacy / Dev Tools ==========
        Some(Commands::Parse { code }) => {
            if !ai_mode {
                println!("Parsing Auto {} to JSON", code);
            }
            let json = auto_lang::run(&code).map_err(|e| {
                if ai_mode {
                    eprintln!("{}", format_error_json(&e));
                    std::process::exit(1);
                }
                to_miette_err(e)
            })?;
            output_success(ai_mode, &json);
        }
        Some(Commands::Eval { code }) => {
            let result = auto_lang::run(&code).map_err(|e| {
                if ai_mode {
                    eprintln!("{}", format_error_json(&e));
                    std::process::exit(1);
                }
                to_miette_err(e)
            })?;
            output_success(ai_mode, &result);
        }
        Some(Commands::OldRepl) => {
            auto_lang::autovm_repl::main_loop().map_err(|e| miette::miette!("{}", e))?;
        }
        Some(Commands::Config { path }) => {
            let code = std::fs::read_to_string(path.as_str())
                .map_err(|e| {
                    if ai_mode {
                        eprintln!("{}", format_error_json(&AutoError::Io(format!("Failed to read file: {}", e))));
                        std::process::exit(1);
                    }
                    miette::miette!("Failed to read file: {}", e)
                })?;
            let args = auto_val::Obj::new();
            let c = auto_lang::eval_config_with_vm(code.as_str(), &args).map_err(|e| {
                if ai_mode {
                    eprintln!("{}", format_error_json(&e));
                    std::process::exit(1);
                }
                to_miette_err(e)
            })?;
            output_success(ai_mode, &c.repr());
        }
        Some(Commands::C { path, target }) => {
            if let Some(target_val) = target {
                // SAFETY: set_var is safe in single-threaded context during CLI startup
                #[allow(deprecated)]
                unsafe {
                    std::env::set_var("AUTO_TARGET", target_val);
                }
            }
            let c = auto_lang::trans_c(path.as_str()).map_err(|e| {
                if ai_mode {
                    eprintln!("{}", format_error_json(&e));
                    std::process::exit(1);
                }
                to_miette_err(e)
            })?;
            output_success(ai_mode, &c);
        }
        Some(Commands::Rust { path }) => {
            let r = auto_lang::trans_rust(path.as_str()).map_err(|e| {
                if ai_mode {
                    eprintln!("{}", format_error_json(&e));
                    std::process::exit(1);
                }
                to_miette_err(e)
            })?;
            output_success(ai_mode, &r);
        }
        Some(Commands::Python { path }) => {
            let py = auto_lang::trans_python(path.as_str()).map_err(|e| {
                if ai_mode {
                    eprintln!("{}", format_error_json(&e));
                    std::process::exit(1);
                }
                to_miette_err(e)
            })?;
            output_success(ai_mode, &py);
        }
        Some(Commands::JavaScript { path }) => {
            let js = auto_lang::trans_javascript(path.as_str()).map_err(|e| {
                if ai_mode {
                    eprintln!("{}", format_error_json(&e));
                    std::process::exit(1);
                }
                to_miette_err(e)
            })?;
            output_success(ai_mode, &js);
        }
        Some(Commands::GDScript { path }) => {
            let gd = auto_lang::trans_gdscript(path.as_str()).map_err(|e| {
                if ai_mode {
                    eprintln!("{}", format_error_json(&e));
                    std::process::exit(1);
                }
                to_miette_err(e)
            })?;
            output_success(ai_mode, &gd);
        }
        // ========== Trans (single file) ==========
        Some(Commands::Trans { path, target }) => match target {
            TransTarget::Ts { output } => {
                let out_path = output.unwrap_or_else(|| {
                    std::path::Path::new(&path)
                        .with_extension("ts")
                        .to_string_lossy()
                        .into_owned()
                });
                let msg = auto_lang::trans_typescript_to(&path, &out_path).map_err(|e| {
                    if ai_mode { eprintln!("{}", format_error_json(&e)); std::process::exit(1); }
                    to_miette_err(e)
                })?;
                println!("{}", msg);
            }
            TransTarget::C { output } => {
                let c = auto_lang::trans_c(path.as_str()).map_err(|e| {
                    if ai_mode { eprintln!("{}", format_error_json(&e)); std::process::exit(1); }
                    to_miette_err(e)
                })?;
                if let Some(out) = output {
                    std::fs::write(&out, &c).map_err(|e| miette::miette!("Failed to write: {}", e))?;
                    println!("[trans] {} -> {}", path, out);
                } else {
                    output_success(ai_mode, &c);
                }
            }
            TransTarget::Rust { output, merge } => {
                if merge {
                    let merged = auto_lang::trans_rust_merged(path.as_str()).map_err(|e| {
                        if ai_mode { eprintln!("{}", format_error_json(&e)); std::process::exit(1); }
                        to_miette_err(e)
                    })?;
                    let content = String::from_utf8_lossy(&merged);
                    if let Some(out) = output {
                        std::fs::write(&out, &*merged).map_err(|e| miette::miette!("Failed to write: {}", e))?;
                        println!("[trans] {} -> {} (merged)", path, out);
                    } else {
                        output_success(ai_mode, &content);
                    }
                } else {
                    let r = auto_lang::trans_rust(path.as_str()).map_err(|e| {
                        if ai_mode { eprintln!("{}", format_error_json(&e)); std::process::exit(1); }
                        to_miette_err(e)
                    })?;
                    if let Some(_out) = output {
                        println!("{}", r);
                    } else {
                        output_success(ai_mode, &r);
                    }
                }
            }
            TransTarget::Python { output } => {
                let py = auto_lang::trans_python(path.as_str()).map_err(|e| {
                    if ai_mode { eprintln!("{}", format_error_json(&e)); std::process::exit(1); }
                    to_miette_err(e)
                })?;
                if let Some(out) = output {
                    std::fs::write(&out, &py).map_err(|e| miette::miette!("Failed to write: {}", e))?;
                    println!("[trans] {} -> {}", path, out);
                } else {
                    output_success(ai_mode, &py);
                }
            }
            TransTarget::Js { output } => {
                let js = auto_lang::trans_javascript(path.as_str()).map_err(|e| {
                    if ai_mode { eprintln!("{}", format_error_json(&e)); std::process::exit(1); }
                    to_miette_err(e)
                })?;
                if let Some(out) = output {
                    std::fs::write(&out, &js).map_err(|e| miette::miette!("Failed to write: {}", e))?;
                    println!("[trans] {} -> {}", path, out);
                } else {
                    output_success(ai_mode, &js);
                }
            }
            TransTarget::Gd { output } => {
                let gd = auto_lang::trans_gdscript(path.as_str()).map_err(|e| {
                    if ai_mode { eprintln!("{}", format_error_json(&e)); std::process::exit(1); }
                    to_miette_err(e)
                })?;
                if let Some(out) = output {
                    std::fs::write(&out, &gd).map_err(|e| miette::miette!("Failed to write: {}", e))?;
                    println!("[trans] {} -> {}", path, out);
                } else {
                    output_success(ai_mode, &gd);
                }
            }
            TransTarget::Tscn { output } => {
                let tscn = auto_lang::trans_tscn(path.as_str()).map_err(|e| {
                    if ai_mode { eprintln!("{}", format_error_json(&e)); std::process::exit(1); }
                    to_miette_err(e)
                })?;
                if let Some(out) = output {
                    std::fs::write(&out, &tscn).map_err(|e| miette::miette!("Failed to write: {}", e))?;
                    println!("[trans] {} -> {}", path, out);
                } else {
                    output_success(ai_mode, &tscn);
                }
            }
            TransTarget::Godot { output: _ } => {
                // trans_godot writes <base>.tscn and <base>.gd next to the source.
                let msg = auto_lang::trans_godot(path.as_str()).map_err(|e| {
                    if ai_mode { eprintln!("{}", format_error_json(&e)); std::process::exit(1); }
                    to_miette_err(e)
                })?;
                output_success(ai_mode, &msg);
            }
        }
        Some(Commands::R2a { path, output }) => {
            let r = auto_lang::transpile_r2a_file(path.as_str()).map_err(|e| {
                if ai_mode { eprintln!("{}", format_error_json(&e)); std::process::exit(1); }
                to_miette_err(e)
            })?;
            if let Some(out) = output {
                std::fs::write(&out, &r).map_err(|e| miette::miette!("Failed to write: {}", e))?;
                println!("[r2a] {} -> {}", path, out);
            } else {
                output_success(ai_mode, &r);
            }
        }
        Some(Commands::A2cStdlib) => {
            cmd_a2c_stdlib::run()?;
        }

        // Plan 216 Phase 4: C FFI Bindgen
        Some(Commands::Cffi { header }) => {
            let headers = ["string.h", "math.h", "stdio.h", "stdlib.h", "time.h"];
            if let Some(h) = header {
                // Show functions for a specific header
                match auto_lang::vm::ffi::c_ffi::load_builtin_manifest(&h) {
                    Some(manifest) => {
                        println!("=== C FFI: {} ===", manifest.header);
                        println!("Library: {}", manifest.library);
                        println!();
                        for func in &manifest.functions {
                            let variadic = if func.variadic { " (variadic)" } else { "" };
                            println!("  {}{} — {:?}({})",
                                func.name,
                                variadic,
                                func.return_type,
                                func.params.iter()
                                    .map(|p| format!("{:?} {}", p.ty, p.name))
                                    .collect::<Vec<_>>()
                                    .join(", ")
                            );
                        }
                    }
                    None => {
                        eprintln!("No manifest found for: {}", h);
                        eprintln!("Available headers: {}", headers.join(", "));
                    }
                }
            } else {
                // List all available headers
                println!("=== C FFI Bindings (Plan 216) ===");
                println!();
                for h in &headers {
                    match auto_lang::vm::ffi::c_ffi::load_builtin_manifest(h) {
                        Some(manifest) => {
                            println!("  {} — {} functions", h, manifest.functions.len());
                        }
                        None => {}
                    }
                }
                println!();
                println!("Use --header <name> to see functions for a specific header.");
            }
        }

        // ========== MCP Server (Plan 265) ==========
        Some(Commands::Mcp) => {
            let mut server = auto_lang::mcp::McpServer::new();
            server.run();
        }

        // ========== AutoVM Daemon (Plan 269) ==========
        Some(Commands::Serve { foreground, stdio, pipe_name, max_sessions, timeout }) => {
            let mut daemon = auto_lang::autovm_daemon::AutovmDaemon::new_with_config(max_sessions, timeout);
            if stdio {
                daemon.run_stdio();
            } else if foreground {
                daemon.run_pipe(&pipe_name);
            } else {
                // Background mode: spawn self with --foreground to listen on named pipe
                let exe = std::env::current_exe().map_err(|e| miette::miette!("Cannot find auto executable: {}", e))?;
                #[cfg(target_family = "windows")]
                {
                    use std::os::windows::process::CommandExt;
                    std::process::Command::new(exe)
                        .args(["serve", "--foreground", "--pipe-name", &pipe_name])
                        .creation_flags(0x08000000) // CREATE_NO_WINDOW
                        .spawn()
                        .map_err(|e| miette::miette!("Failed to start daemon: {}", e))?;
                }
                #[cfg(target_family = "unix")]
                std::process::Command::new(exe)
                    .args(["serve", "--foreground", "--pipe-name", &pipe_name])
                    .spawn()
                    .map_err(|e| miette::miette!("Failed to start daemon: {}", e))?;
                println!("AutoVM daemon started (pipe: {})", pipe_name);
            }
        }

        // ========== AutoVM Client (Plan 269) ==========
        Some(Commands::Req { session, pipe_name, new_session, inspect, reset, delete, snapshot, list, json, code }) => {
            use auto_lang::autovm_client::AutovmClient;

            let mut client = AutovmClient::connect(&pipe_name).map_err(|e| miette::miette!("{}", e))?;

            if list {
                let resp = client.list().map_err(|e| miette::miette!("{}", e))?;
                if json {
                    println!("{}", serde_json::to_string(&resp).unwrap());
                } else {
                    match &resp.data {
                        Some(d) => println!("Sessions: {}", d),
                        None => println!("No sessions"),
                    }
                }
                return Ok(());
            }

            if new_session {
                let ses_id = client.new_session().map_err(|e| miette::miette!("{}", e))?;
                println!("{}", ses_id);
                return Ok(());
            }

            let ses_id = match &session {
                Some(s) => s.clone(),
                None => {
                    // Anonymous mode: create temp session, eval, delete
                    if let Some(code) = &code {
                        let (value, ok) = auto_lang::autovm_client::eval_one_shot(code);
                        if json {
                            let status = if ok { "ok" } else { "error" };
                            println!("{}", serde_json::json!({"status": status, "value": value}));
                        } else {
                            if ok { println!("{}", value); } else { eprintln!("{}", value); }
                        }
                        return Ok(());
                    }
                    eprintln!("Error: provide --session <id>, --new-session, or code to evaluate");
                    std::process::exit(1);
                }
            };

            if inspect {
                let resp = client.inspect(&ses_id).map_err(|e| miette::miette!("{}", e))?;
                if json {
                    println!("{}", serde_json::to_string(&resp).unwrap());
                } else {
                    if resp.status == "ok" {
                        if let Some(data) = &resp.data {
                            println!("{}", serde_json::to_string_pretty(data).unwrap());
                        }
                    } else {
                        eprintln!("Error: {}", resp.message.unwrap_or_default());
                    }
                }
            } else if reset {
                let resp = client.reset(&ses_id).map_err(|e| miette::miette!("{}", e))?;
                if json { println!("{}", serde_json::to_string(&resp).unwrap()); }
                else { println!("Session {} reset", ses_id); }
            } else if delete {
                let resp = client.delete(&ses_id).map_err(|e| miette::miette!("{}", e))?;
                if json { println!("{}", serde_json::to_string(&resp).unwrap()); }
                else { println!("Session {} deleted", ses_id); }
            } else if snapshot {
                let resp = client.snapshot(&ses_id).map_err(|e| miette::miette!("{}", e))?;
                if json {
                    println!("{}", serde_json::to_string(&resp).unwrap());
                } else {
                    match &resp.value {
                        Some(src) => println!("{}", src),
                        None => eprintln!("Error: {}", resp.message.unwrap_or_default()),
                    }
                }
            } else if let Some(code) = &code {
                let resp = client.eval(&ses_id, code).map_err(|e| miette::miette!("{}", e))?;
                if json {
                    println!("{}", serde_json::to_string(&resp).unwrap());
                } else {
                    if resp.status == "ok" {
                        if let Some(val) = &resp.value {
                            println!("{}", val);
                        }
                    } else {
                        eprintln!("Error: {}", resp.message.unwrap_or_default());
                    }
                }
            } else {
                eprintln!("Error: provide code to evaluate or an action flag (--inspect, --reset, --delete, --snapshot)");
                std::process::exit(1);
            }
        }

        // ========== Debug (Plan 199) ==========
        Some(Commands::Debug { file, agent }) => {
            if agent {
                auto_lang::debug_file_agent(&file).map_err(to_miette_err)?;
            } else {
                println!("----------------------");
                println!("Debugging Auto {}", file);
                println!("----------------------");
                auto_lang::debug_file(&file).map_err(to_miette_err)?;
            }
        }

        None => {
            // Default: Use BigVM REPL
            auto_lang::autovm_repl::main_loop().map_err(|e| miette::miette!("{}", e))?;
        }
    }

    Ok(())
}

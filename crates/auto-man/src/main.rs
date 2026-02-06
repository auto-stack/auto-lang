use auto_man::*;
use auto_val::AutoStr;
use clap::{CommandFactory, Parser, Subcommand};
use colored::Colorize;
use log::*;
use simplelog::*;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None, subcommand_required = true)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    #[command(
        about = "Create a new Auto application package",
        long_about = "Create a new Auto application package with the given name.

This creates a new directory with:
  - pac.at - Package configuration file
  - src/    - Source directory (with main.at)
  - build/  - Build output directory

EXAMPLES:
  am app myproject
  am app hello-world",
        alias = "a"
    )]
    App { name: String },

    #[command(
        about = "Create a new Auto library package",
        long_about = "Create a new Auto library package with the given name.

This creates a new directory with:
  - pac.at - Package configuration file
  - src/    - Source directory for library code
  - build/  - Build output directory

EXAMPLES:
  am lib mylib
  am lib utils",
        alias = "l"
    )]
    Lib { name: String },

    #[command(
        about = "Create a new C application package",
        long_about = "Create a new C application package (no Auto transpilation).

This creates a new directory with:
  - pac.at - Package configuration file
  - src/    - Source directory (with main.c)
  - build/  - Build output directory

EXAMPLES:
  am capp mycproject",
    )]
    Capp { name: String },

    #[command(
        about = "Create a new C library package",
        long_about = "Create a new C library package (no Auto transpilation).

This creates a new directory with:
  - pac.at - Package configuration file
  - src/    - Source directory for library code
  - build/  - Build output directory

EXAMPLES:
  am clib myclib",
    )]
    Clib { name: String },

    #[command(
        about = "Scan project and download dependencies",
        long_about = "Scan the project, discover sources, and download dependencies.

This command:
  1. Parses pac.at configuration
  2. Downloads missing dependencies from git
  3. Scans for Auto (.at) and C source files
  4. Discovers include directories

EXAMPLES:
  am scan
  am scan          # Run from project root"
    )]
    Scan,

    #[command(
        about = "Build the project",
        long_about = "Build the project using the configured port.

This will:
  1. Scan the project and download dependencies
  2. Transpile Auto files to C
  3. Generate build configuration (CMake, IAR, etc.)
  4. Compile the project

OPTIONS:
  --dir <DIR>  Project directory (default: current directory)

EXAMPLES:
  am build
  am b              # Using alias
  am build --dir ../myproject",
        alias = "b"
    )]
    Build {
        /// Project directory (default: current directory)
        #[arg(short, long)]
        dir: Option<String>,
    },

    #[command(
        about = "Run the compiled executable",
        long_about = "Run the compiled executable with optional arguments.

The executable is located in the build directory (e.g., build/Debug/).

EXAMPLES:
  am run
  am r              # Using alias
  am run -- --help --verbose",
        alias = "r"
    )]
    Run {
        /// Arguments to pass to the executable
        #[arg(allow_hyphen_values = true)]
        args: Vec<String>,
    },

    #[command(
        about = "Clean build artifacts",
        long_about = "Remove build artifacts and temporary files.

This removes:
  - build/ directory
  - Transpiled C files (.c, .h from .at)
  - Log files

OPTIONS:
  --dir <DIR>  Project directory (default: current directory)

EXAMPLES:
  am clean
  am clean --dir ../myproject"
    )]
    Clean {
        /// Project directory (default: current directory)
        #[arg(short, long)]
        dir: Option<String>,
    },

    #[command(
        about = "Show dependency tree",
        long_about = "Display all dependencies and their relationships.

Shows:
  - Direct dependencies from pac.at
  - Transitive dependencies
  - Dependency versions

EXAMPLES:
  am deps"
    )]
    Deps,

    #[command(
        about = "Show available devices",
        long_about = "List all available devices from the device index.

EXAMPLES:
  am devices"
    )]
    Devices,

    #[command(
        about = "Open project in IDE",
        long_about = "Open the project in the configured IDE.

The IDE is determined by the port configuration.

EXAMPLES:
  am open
  am o              # Using alias",
        alias = "o"
    )]
    Open,

    #[command(
        about = "Show package or target information",
        long_about = "Display detailed information about the package or a specific target.

Without arguments, shows package information.
With target name, shows specific target details.

OPTIONS:
  --target <NAME>  Target name to inspect

EXAMPLES:
  am info
  am i              # Using alias
  am info --target myapp",
        alias = "i"
    )]
    Info {
        /// Target name to inspect
        #[arg(short, long)]
        target: Option<String>,
    },

    #[command(
        about = "Show or select build port",
        long_about = "Display the current port or select a different port.

Ports define the platform, toolchain, and builder to use.

EXAMPLES:
  am port           # Show current port
  am port           # Select from available ports"
    )]
    Port,

    #[command(
        about = "Upgrade AutoMan to latest version",
        long_about = "Upgrade AutoMan to the latest version from the repository.

EXAMPLES:
  am upgrade"
    )]
    Upgrade,

    #[command(
        about = "Pull/download all dependencies",
        long_about = "Download all dependencies from git repositories.

This command:
  1. Reads pac.at for dependency declarations
  2. Downloads each dependency from git
  3. Places them in the deps/ directory

EXAMPLES:
  am pull"
    )]
    Pull,

    #[command(
        about = "Reset AutoMan configuration and index",
        long_about = "Reset AutoMan to default state.

This removes:
  - User configuration (~/.auto/auto-man/am.at)
  - Package index cache
  - Device index cache

EXAMPLES:
  am reset"
    )]
    Reset,

    #[command(
        about = "Install AutoMan configuration file",
        long_about = "Install a custom am.at configuration file.

OPTIONS:
  <FILE>  Path to configuration file

EXAMPLES:
  am install my-config.at
  am install ~/.auto/am.at"
    )]
    Install {
        /// Configuration file to install
        file: String,
    },
}

fn init_logger() {
    CombinedLogger::init(vec![
        TermLogger::new(
            LevelFilter::Info,
            Config::default(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        ),
        // WriteLogger::new(LevelFilter::Info, Config::default(), File::create("automan.log").unwrap()),
    ])
    .unwrap();
}

fn select_port(input: Option<String>, ports: &Vec<AutoStr>) -> AutoResult<AutoStr> {
    crate::util::select_or_default_port(input, ports, "Which port do you want to build?")
}

fn do_upgrade() -> AutoResult<()> {
    info!("Upgrading Automan");
    // Implement upgrade logic here
    use crate::up::*;
    upgrade()?;
    Ok(())
}

fn main() -> Result<(), AutoError> {
    init_logger();
    let logo = format!(
        r#"---------------------------
Hello, I'm Automan {}!
---------------------------"#,
        AUTOMAN_VERSION,
    );
    println!("{}", logo.bright_yellow().bold());
    let cli = Cli::parse();

    let config = match load_am_config() {
        Some(config) => config,
        _ => AmConfig::default(),
    };
    // try to load base config
    match cli.command {
        Some(Commands::App { name }) => {
            info!("Creating new app package {}", name);
            Automan::create_app(&name)?;
        }
        Some(Commands::Lib { name }) => {
            info!("Creating new library package {}", name);
            Automan::create_lib(&name)?;
        }
        Some(Commands::Capp { name }) => {
            info!("Creating new C app package {}", name);
            Automan::create_capp(&name)?;
        }
        Some(Commands::Clib { name }) => {
            info!("Creating new C library package {}", name);
            Automan::create_clib(&name)?;
        }
        Some(Commands::Scan) => {
            info!("Scanning dependencies");
            let mut am = Automan::new(".", config)?;
            am.scan()?;
        }
        Some(Commands::Build { dir }) => {
            let dir = if let Some(dir) = dir {
                dir
            } else {
                ".".to_string()
            };
            let mut am = Automan::new(&dir, config)?;
            am.scan()?;
            am.build()?;
        }
        Some(Commands::Run { args }) => {
            let mut am = Automan::new(".", config)?;
            // TODO: add build process before running when user specified a '-b' flag
            info!("Running app ...");
            println!();
            println!("------------ output ------------");
            am.run(args)?;
            println!("------------- end --------------");
        }
        Some(Commands::Clean { dir }) => {
            let dir = if let Some(dir) = dir {
                dir
            } else {
                ".".to_string()
            };
            let mut am = Automan::new(&dir, config)?;
            am.clean()?;
        }
        Some(Commands::Deps) => {
            Automan::list_deps(&config)?;
        }
        Some(Commands::Devices) => {
            Automan::list_devices(&config)?;
        }
        Some(Commands::Open) => {
            // std::process::Command::new("explorer.exe")
            // .arg("iar\\hello.eww")
            // .spawn()?;
            let mut am = Automan::new(".", config)?;
            am.open_ide()?;
        }
        Some(Commands::Info { target }) => {
            let am = Automan::new(".", config)?;
            am.info(target)?;
        }
        Some(Commands::Port) => {
            let mut am = Automan::new(".", config)?;
            let port = select_port(None, &am.list_port_names())?;
            am.set_port(port.clone())?;
            info!("port \"{}\" written to .am/state.at", port)
        }
        Some(Commands::Upgrade) => {
            do_upgrade()?;
        }
        Some(Commands::Pull) => {
            let mut am = Automan::new(".", config)?;
            am.pull()?;
        }
        Some(Commands::Reset) => {
            Automan::reset_index()?;
        }
        Some(Commands::Install { file }) => {
            Automan::install_config(&file)?;
        }
        None => {
            Cli::command().print_help()?;
        }
    }
    Ok(())
}

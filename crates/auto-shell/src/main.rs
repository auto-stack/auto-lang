use miette::Result;

fn main() -> Result<()> {
    // Set up miette for beautiful error reporting
    miette::set_hook(Box::new(|_| {
        Box::new(
            miette::MietteHandlerOpts::new()
                .terminal_links(true)
                .build(),
        )
    }))
    .ok();

    let args: Vec<String> = std::env::args().collect();

    // ── CLI argument handling ──────────────────────
    //
    // ash               → interactive REPL
    // ash script.at     → execute script file (Plan 303)
    // ash -c "cmd"      → execute single command (Plan 304)
    // ash -s            → read script from stdin (Plan 304)
    // ash -l / --login  → login shell mode (Plan 304)
    // ash -h / --help   → help text
    // ash -v / --version → version

    let mut i = 1;
    let mut login_mode = false;

    while i < args.len() {
        let arg = &args[i];
        match arg.as_str() {
            "-c" => {
                // Execute a single command string
                if i + 1 >= args.len() {
                    eprintln!("ash -c: option requires an argument");
                    std::process::exit(1);
                }
                let command = &args[i + 1];
                let mut shell = auto_shell::Shell::new();
                shell.load_env_persistence(); // Plan 309 Task 1.2 P4: apply ~/.config/ash/env.at
                match shell.execute(command) {
                    Ok(output) => {
                        if let Some(s) = output {
                            println!("{}", s);
                        }
                    }
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        std::process::exit(1);
                    }
                }
                return Ok(());
            }
            "-s" => {
                // Read script from stdin
                let mut input = String::new();
                if let Err(e) = std::io::Read::read_to_string(&mut std::io::stdin(), &mut input) {
                    eprintln!("ash: failed to read stdin: {}", e);
                    std::process::exit(1);
                }
                let mut shell = auto_shell::Shell::new();
                shell.load_env_persistence();
                shell.execute_script_content(&input)?;
                return Ok(());
            }
            "-l" | "--login" => {
                login_mode = true;
                i += 1;
                continue;
            }
            "-h" | "--help" => {
                println!("ash — AutoShell v0.1.0");
                println!();
                println!("USAGE:");
                println!("  ash               Start interactive REPL");
                println!("  ash <script.at>   Execute a script file");
                println!("  ash -c <cmd>      Execute a single command");
                println!("  ash -s            Read script from stdin");
                println!("  ash -l, --login   Start as login shell");
                println!("  ash -h, --help    Show this help");
                println!("  ash -v, --version Show version");
                return Ok(());
            }
            "-v" | "--version" => {
                println!("ash (AutoShell) v0.1.0");
                return Ok(());
            }
            _ => {
                // Not a flag — treat as script file path
                break;
            }
        }
    }

    // Determine if we're running a script or entering REPL
    let script_arg = args.get(i).map(|s| s.as_str()).unwrap_or("");

    if !script_arg.is_empty() && !script_arg.starts_with('-') {
        // Script execution mode: ash hello.at [args...]
        let path = std::path::Path::new(script_arg);

        if !path.exists() {
            eprintln!("ash: {}: No such file", script_arg);
            std::process::exit(1);
        }

        let mut shell = auto_shell::Shell::new();
        shell.load_env_persistence();
        shell.execute_script_file(path)?;
        return Ok(());
    }

    // Interactive REPL (with optional login mode)
    if login_mode {
        // Login shell: source /etc/profile, then ~/.ash_profile or ~/.ashrc
        #[cfg(unix)]
        {
            let etc_profile = std::path::Path::new("/etc/profile");
            if etc_profile.exists() {
                // Best-effort: source /etc/profile via external shell
                let _ = std::process::Command::new("sh")
                    .arg("-c")
                    .arg("source /etc/profile 2>/dev/null && env")
                    .output()
                    .ok();
            }
        }
    }

    println!("AutoShell v0.1.0");
    println!("Type 'exit' or press Ctrl+D to exit");
    println!();

    let mut repl = auto_shell::Repl::new()?;
    repl.run()?;

    Ok(())
}

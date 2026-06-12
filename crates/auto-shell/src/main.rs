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

    if args.len() > 1 {
        // Plan 303 Step 1: Script execution mode — ash hello.at [args...]
        let script_path = &args[1];
        let path = std::path::Path::new(script_path);

        if !path.exists() {
            eprintln!("ash: {}: No such file", script_path);
            std::process::exit(1);
        }

        let mut shell = auto_shell::Shell::new();
        shell.execute_script_file(path)?;
        return Ok(());
    }

    // Default: interactive REPL
    println!("AutoShell v0.1.0");
    println!("Type 'exit' or press Ctrl+D to exit");
    println!();

    let mut repl = auto_shell::Repl::new()?;
    repl.run()?;

    Ok(())
}

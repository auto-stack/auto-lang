use std::process::{Command, ExitCode};

fn main() -> ExitCode {
    let arguments: Vec<_> = std::env::args_os().skip(1).collect();
    let modules = arguments.first().is_some_and(|arg| arg == "tm");
    let script = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join(if modules {
            "test-governance/module_command.py"
        } else {
            "run-full-regression.py"
        });
    let python = std::env::var_os("PYTHON").unwrap_or_else(|| {
        if cfg!(windows) {
            "python".into()
        } else {
            "python3".into()
        }
    });
    match Command::new(python)
        .arg(script)
        .args(arguments.iter().skip(usize::from(modules)))
        .status()
    {
        Ok(status) => ExitCode::from(status.code().unwrap_or(130).clamp(0, 255) as u8),
        Err(error) => {
            eprintln!("full regression launcher: {error}");
            ExitCode::FAILURE
        }
    }
}

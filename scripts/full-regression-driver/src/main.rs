use std::process::{Command, ExitCode};

fn main() -> ExitCode {
    let arguments: Vec<_> = std::env::args_os().skip(1).collect();
    let modules = arguments.first().is_some_and(|arg| arg == "tm");
    let fixed = arguments.first().is_some_and(|arg| arg == "test");
    if modules {
        eprintln!(
            "cargo tm is retired (Plan 896). Use cargo t --list, cargo t <group>, or cargo t all."
        );
        return ExitCode::from(2);
    }
    let script = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join(if fixed {
            "fixed-tests.py"
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
        .args(arguments.iter().skip(usize::from(fixed)))
        .status()
    {
        Ok(status) => ExitCode::from(status.code().unwrap_or(130).clamp(0, 255) as u8),
        Err(error) => {
            eprintln!("full regression launcher: {error}");
            ExitCode::FAILURE
        }
    }
}

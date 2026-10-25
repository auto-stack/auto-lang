use std::process::{Command, ExitCode};

fn main() -> ExitCode {
    let script = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("run-full-regression.py");
    let python = std::env::var_os("PYTHON").unwrap_or_else(|| {
        if cfg!(windows) {
            "python".into()
        } else {
            "python3".into()
        }
    });
    match Command::new(python)
        .arg(script)
        .args(std::env::args_os().skip(1))
        .status()
    {
        Ok(status) => ExitCode::from(status.code().unwrap_or(130).clamp(0, 255) as u8),
        Err(error) => {
            eprintln!("full regression launcher: {error}");
            ExitCode::FAILURE
        }
    }
}

use std::process::{Command, ExitCode};

// PLAN-898: single fixed-group launcher. The retired `tm`/tier dispatch
// (tf/taa/ta/t3 → run-full-regression.py) is gone; `cargo t ...` passes
// straight through to scripts/fixed-tests.py (PLAN-896 fixed groups).
fn main() -> ExitCode {
    let arguments: Vec<_> = std::env::args_os().skip(1).collect();
    let script = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("fixed-tests.py");
    let python = std::env::var_os("PYTHON").unwrap_or_else(|| {
        if cfg!(windows) {
            "python".into()
        } else {
            "python3".into()
        }
    });
    let skip_test = usize::from(arguments.first().is_some_and(|arg| arg == "test"));
    let args = arguments.iter().skip(skip_test);
    match Command::new(python).arg(script).args(args).status() {
        Ok(status) => ExitCode::from(status.code().unwrap_or(130).clamp(0, 255) as u8),
        Err(error) => {
            eprintln!("fixed-group launcher: {error}");
            ExitCode::FAILURE
        }
    }
}

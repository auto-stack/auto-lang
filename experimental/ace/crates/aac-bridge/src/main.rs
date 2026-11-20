use ace_aac_bridge::{Client, Failure, FailureKind};
use std::path::Path;
use std::process::ExitCode;

fn run(args: &[std::ffi::OsString]) -> Result<(), Failure> {
    match args {
        [mode] if mode == "--worker" => {
            ace_aac_bridge::serve(std::io::stdin().lock(), std::io::stdout().lock())
        }
        [mode, input, target] if mode == "--exchange" => {
            let executable = std::env::current_exe().map_err(|error| Failure {
                kind: FailureKind::Tool,
                code: "bridge.executable".into(),
                message: error.to_string(),
                span: None,
            })?;
            Client::new(&executable)?.exchange_file(Path::new(input), Path::new(target))
        }
        _ => Err(Failure {
            kind: FailureKind::Admission,
            code: "bridge.cli".into(),
            message: "expected --worker or --exchange <absolute-request> <absolute-response>"
                .into(),
            span: None,
        }),
    }
}

fn main() -> ExitCode {
    let args = std::env::args_os().skip(1).collect::<Vec<_>>();
    match run(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::from(match error.kind {
                FailureKind::Admission | FailureKind::Language => 2,
                FailureKind::Resource => 3,
                FailureKind::Cancelled => 4,
                FailureKind::Tool => 5,
            })
        }
    }
}

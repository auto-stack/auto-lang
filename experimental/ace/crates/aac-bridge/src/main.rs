use ace_aac_bridge::{Client, Failure, FailureKind, FOREIGN_CONFIG_ENV, FOREIGN_CONFIG_HASH_ENV};
use std::path::Path;
use std::process::ExitCode;

fn config_hash(expected: &std::ffi::OsStr) -> Result<[u8; 32], Failure> {
    expected
        .to_str()
        .and_then(|hex| blake3::Hash::from_hex(hex).ok())
        .map(|hash| *hash.as_bytes())
        .ok_or(Failure {
            kind: FailureKind::Admission,
            code: "bridge.foreign-config-identity".into(),
            message: "expected the SDK configuration digest".into(),
            span: None,
        })
}

fn sdk_client(executable: &Path) -> Result<Client, Failure> {
    let client = Client::new(executable)?;
    // The SDK launcher supplies both values independently of the request.
    // A digest in the request can select only this already approved provider.
    match (
        std::env::var_os(FOREIGN_CONFIG_ENV),
        std::env::var_os(FOREIGN_CONFIG_HASH_ENV),
    ) {
        (None, None) => Ok(client),
        (Some(config), Some(expected)) => {
            client.with_foreign_provider(Path::new(&config), config_hash(&expected)?)
        }
        _ => Err(Failure {
            kind: FailureKind::Admission,
            code: "bridge.foreign-config-identity".into(),
            message: "the SDK launcher must supply both configuration path and digest".into(),
            span: None,
        }),
    }
}

fn run(args: &[std::ffi::OsString]) -> Result<(), Failure> {
    match args {
        [mode] if mode == "--worker" => {
            ace_aac_bridge::serve(std::io::stdin().lock(), std::io::stdout().lock())
        }
        [mode, config, expected] if mode == "--worker-foreign" => {
            let permit =
                ace_aac_bridge::ForeignPermit::read(Path::new(config), config_hash(expected)?)?;
            ace_aac_bridge::serve_with_foreign(
                std::io::stdin().lock(),
                std::io::stdout().lock(),
                &permit,
            )
        }
        [mode, input, target] if mode == "--exchange" => {
            let executable = std::env::current_exe().map_err(|error| Failure {
                kind: FailureKind::Tool,
                code: "bridge.executable".into(),
                message: error.to_string(),
                span: None,
            })?;
            sdk_client(&executable)?.exchange_file(Path::new(input), Path::new(target))
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

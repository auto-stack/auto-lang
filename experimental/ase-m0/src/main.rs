//! One-shot experiment CLI (PLAN-1021 §5): resolve one registered app
//! identity and execute it headlessly through the public ACE packages
//! chain, printing the result, phase identity of the consumed sources and
//! the dispatch count. Rejections print a named error with the dispatch
//! count observed at the real adapter boundary (0 for URI/version refusals).

use ase_m0::{load_registry, run_app, AseError, RunOptions};
use std::path::PathBuf;
use std::process::ExitCode;

const USAGE: &str = "usage: ase-m0 --app app://auto.demo.a/ --artifact-root <dir> \
[--registry <dir>] [--version <v>]";

fn default_registry() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures")
}

fn parse_args(args: &[String]) -> Result<(String, String, Option<String>, String), String> {
    let mut app = None;
    let mut registry = None;
    let mut version = None;
    let mut artifact_root = None;
    let mut index = 0;
    while index < args.len() {
        let arg = args[index].as_str();
        let mut value = |name: &str| -> Result<String, String> {
            index += 1;
            args.get(index)
                .cloned()
                .ok_or_else(|| format!("missing value for {name}"))
        };
        match arg {
            "--app" => app = Some(value("--app")?),
            "--registry" => registry = Some(value("--registry")?),
            "--version" => version = Some(value("--version")?),
            "--artifact-root" => artifact_root = Some(value("--artifact-root")?),
            other => return Err(format!("unknown argument {other:?}; {USAGE}")),
        }
        index += 1;
    }
    Ok((
        app.ok_or_else(|| format!("--app is required; {USAGE}"))?,
        registry.unwrap_or_else(|| default_registry().to_string_lossy().into_owned()),
        version,
        artifact_root.ok_or_else(|| format!("--artifact-root is required; {USAGE}"))?,
    ))
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (app, registry_dir, version, artifact_root) = match parse_args(&args) {
        Ok(parsed) => parsed,
        Err(message) => {
            eprintln!("error phase=resolve kind=uri dispatch=0: {message}");
            return ExitCode::FAILURE;
        }
    };
    let registry = match load_registry(&PathBuf::from(&registry_dir)) {
        Ok(registry) => registry,
        Err(e) => return report(e),
    };
    let store_dir = PathBuf::from(&artifact_root).join("store");
    let options = RunOptions {
        version,
        budget: None,
        store_dir,
    };
    match run_app(&registry, &app, &options) {
        Ok(report) => {
            println!(
                "app={} version={} package={} entry={}",
                report.identity, report.version, report.package, report.entry
            );
            for (module, file, sha256_16) in &report.sources {
                println!("source module={module} file={file} sha256={sha256_16}");
            }
            println!("input_hash={}", report.input_hash);
            println!("lowered={}", report.lowered);
            println!("value={}", report.value);
            println!("dispatch={}", report.dispatches);
            ExitCode::SUCCESS
        }
        Err(e) => report(e),
    }
}

fn report(error: AseError) -> ExitCode {
    eprintln!("error {error}");
    ExitCode::FAILURE
}

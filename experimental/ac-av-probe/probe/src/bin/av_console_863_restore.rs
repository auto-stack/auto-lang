//! PLAN-863 fresh-process portable restore oracle. No product entrypoint uses this binary.
use auto_av_core::av::{CancelFlag, ExecLimits};
use auto_av_probe::console_host::{restore, ConsoleArtifactRef, ConsoleCapture};
use base64::Engine;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let result = (|| -> Result<serde_json::Value, String> {
        if args.len() != 4 {
            return Err("usage: av_console_863_restore STORE REFERENCE GRANTED|WITHDRAWN".into());
        }
        let reference: ConsoleArtifactRef =
            serde_json::from_slice(&std::fs::read(&args[2]).map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())?;
        let program = restore(std::path::Path::new(&args[1]), &reference)
            .map_err(|e| format!("{}: {}", e.code, e.message))?;
        let grants = match args[3].as_str() {
            "GRANTED" => vec!["console.stdout".into(), "console.stderr".into()],
            "WITHDRAWN" => vec![],
            _ => return Err("invalid grant mode".into()),
        };
        let mut capture = ConsoleCapture::default();
        let run = program.run(
            &grants,
            &ExecLimits::default(),
            &CancelFlag::new(false),
            &mut capture,
        );
        let (outcome, code) = match run {
            Ok((out, _)) => (format!("{out:?}"), "ok".to_string()),
            Err(e) => (e.message, e.code),
        };
        Ok(serde_json::json!({"code": code, "outcome": outcome,
            "stdout_b64": base64::engine::general_purpose::STANDARD.encode(capture.stdout),
            "stderr_b64": base64::engine::general_purpose::STANDARD.encode(capture.stderr),
            "dispatches": capture.dispatches}))
    })();
    match result {
        Ok(response) => println!("{response}"),
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(2);
        }
    }
}

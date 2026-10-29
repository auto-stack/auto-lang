//! PLAN-863 console recipe producer. Reports observations, never expected values or PASS.
use auto_av_core::av::{CancelFlag, ExecLimits};
use auto_av_core::contract::{fnv1a64, ExecOutcome, Value, AV_HOST_CONSOLE_PROFILE};
use base64::Engine;
use serde_json::{json, Value as Json};

use crate::console_host::{compile, restore, ConsoleCapture};

pub const RECIPE: &str = "ac-av/console-run-v1";

pub fn serve(request: &Json) -> Result<Json, String> {
    let object = request
        .as_object()
        .ok_or("console request must be an object")?;
    let allowed = [
        "recipe",
        "nonce",
        "request_hash",
        "source_b64",
        "entry",
        "caps",
        "store_root",
        "steps",
    ];
    if object.keys().any(|k| !allowed.contains(&k.as_str())) {
        return Err("unknown console request field".into());
    }
    if request["recipe"].as_str() != Some(RECIPE) || request["entry"].as_str() != Some("d_main") {
        return Err("console recipe requires d_main".into());
    }
    let steps = request["steps"].as_array().ok_or("missing console steps")?;
    if steps.len() != 1 || steps[0] != json!({"op":"run"}) {
        return Err("console requires exactly one run step".into());
    }
    let nonce = request["nonce"].as_str().ok_or("missing nonce")?;
    if nonce.len() != 64 || !nonce.bytes().all(|c| c.is_ascii_hexdigit()) {
        return Err("invalid console nonce".into());
    }
    let source = base64::engine::general_purpose::STANDARD
        .decode(request["source_b64"].as_str().ok_or("missing source")?)
        .map_err(|e| e.to_string())?;
    let source = String::from_utf8(source).map_err(|e| e.to_string())?;
    let caps: Vec<String> = request["caps"]
        .as_array()
        .ok_or("missing caps")?
        .iter()
        .map(|c| c.as_str().map(String::from).ok_or("invalid capability"))
        .collect::<Result<_, _>>()?;
    let root = request["store_root"]
        .as_str()
        .filter(|s| !s.is_empty())
        .ok_or("missing store root")?;
    // The parent owns the validated artifact directory. This newly named store is private to this request.
    let store = std::path::Path::new(root).join(format!("console-{nonce}"));
    let started = std::time::Instant::now();
    let mut capture = ConsoleCapture::default();
    let mut step = json!({"op":"run", "outcome":"infra", "value":null, "trap":null, "diag":null,
        "source_hash":format!("{:016x}",fnv1a64(source.as_bytes())), "wall_ms":0.0});
    let mut stored = false;
    match compile(&source, "d_main") {
        Err(error) => {
            let resource =
                error.code.starts_with("av.resource.") || error.code.starts_with("resource.");
            let frontend = ["source.", "verify.", "bind.", "capability."]
                .iter()
                .any(|prefix| error.code.starts_with(prefix));
            step["outcome"] = json!(if resource {
                "resource"
            } else if frontend {
                "diag"
            } else {
                "infra"
            });
            step["diag"] = json!({"stage":error.stage,"code":error.code,"message":error.message});
        }
        Ok(program) => {
            program
                .persist(&store, "formal-console")
                .map_err(|e| format!("{}: {}", e.code, e.message))?;
            stored = true;
            let reference = program.reference.clone();
            drop(program);
            let restored =
                restore(&store, &reference).map_err(|e| format!("{}: {}", e.code, e.message))?;
            match restored.run(
                &caps,
                &ExecLimits::default(),
                &CancelFlag::new(false),
                &mut capture,
            ) {
                Err(error) => {
                    step["outcome"] = json!(if error.code == "host.unauthorized" {
                        "diag"
                    } else {
                        "infra"
                    });
                    step["diag"] =
                        json!({"stage":error.stage,"code":error.code,"message":error.message});
                }
                Ok((outcome, _trace)) => match outcome {
                    ExecOutcome::Success {
                        value: Value::I32(n),
                        ..
                    } => {
                        step["outcome"] = json!("success");
                        step["channel"] = json!("value");
                        step["value"] = json!(n);
                    }
                    ExecOutcome::Success {
                        value: Value::Unit, ..
                    } => {
                        step["outcome"] = json!("success");
                        step["channel"] = json!("unit");
                    }
                    ExecOutcome::Success { .. } => {
                        return Err("console returned a non-entry result type".into())
                    }
                    ExecOutcome::Trap { kind, .. } => {
                        step["outcome"] = json!("trap");
                        step["trap"] = json!(kind.code());
                    }
                    ExecOutcome::Resource { kind, message } => {
                        step["outcome"] = json!("resource");
                        step["diag"] = json!({"stage":"run","code":kind.code(),"message":message});
                    }
                    ExecOutcome::Cancelled => {
                        step["outcome"] = json!("cancelled");
                        step["diag"] = json!({"stage":"run","code":"cancelled","message":"execution cancelled"});
                    }
                    ExecOutcome::Infra(message) => {
                        step["outcome"] = json!("infra");
                        step["diag"] = json!({"stage":"run","code":"host.infra","message":message});
                    }
                },
            }
        }
    }
    step["console"] = json!({"schema":"av.console-capture/1", "mir_lowering":auto_mir_lower::MIR_LOWERING_VERSION,
        "stdout_present":true,"stderr_present":true,"stdout_truncated":false,"stderr_truncated":false,
        "stdout_b64":base64::engine::general_purpose::STANDARD.encode(&capture.stdout),
        "stderr_b64":base64::engine::general_purpose::STANDARD.encode(&capture.stderr)});
    step["counters"] =
        json!({"host_dispatches":capture.dispatches,"store_writes":u64::from(stored)});
    step["wall_ms"] = json!(started.elapsed().as_secs_f64() * 1000.0);
    Ok(
        json!({"recipe":RECIPE,"steps":[step],"worker_version":"avworker/1.0","av_core_version":AV_HOST_CONSOLE_PROFILE}),
    )
}

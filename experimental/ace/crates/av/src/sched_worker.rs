//! PLAN-968 scheduler recipe producer: task actor sources through the
//! full chain (scheduler compile → closures-family lower → MIR verify →
//! ABC admit → the deterministic cooperative scheduler) with console
//! observation. Reports observations, never expected values or PASS.
use ace_av_core::abc::encode_abc;
use ace_av_core::av::{execute_sched, CancelFlag, ExecLimits};
use ace_av_core::contract::{fnv1a64, ExecOutcome, Value};
use ace_av_core::verify_mir;
use base64::Engine;
use serde_json::{json, Value as Json};

use crate::console_host::ConsoleCapture;

pub const RECIPE: &str = "ac-av/scheduler-source-run-v1";
/// The execution identity rides the closures-superset profile family
/// (SCHED opcodes are additive on it).
pub const AV_SCHEDULER_PROFILE: &str = "av.closures/1";

pub fn serve(request: &Json) -> Result<Json, String> {
    let object = request
        .as_object()
        .ok_or("scheduler request must be an object")?;
    let allowed = [
        "recipe",
        "nonce",
        "request_hash",
        "source_b64",
        "entry",
        "steps",
        // Present-but-inert optional fields of the shared request builder.
        "caps",
        "limits",
        "cancel_after_ms",
        "store_root",
    ];
    if let Some(bad) = object.keys().find(|k| !allowed.contains(&k.as_str())) {
        return Err(format!("unknown scheduler request field: {bad}"));
    }
    if request["recipe"].as_str() != Some(RECIPE) || request["entry"].as_str() != Some("d_main") {
        return Err("scheduler recipe requires d_main".into());
    }
    let steps = request["steps"]
        .as_array()
        .ok_or("missing scheduler steps")?;
    if steps.len() != 1 || steps[0] != json!({"op":"run"}) {
        return Err("scheduler requires exactly one run step".into());
    }
    let nonce = request["nonce"].as_str().ok_or("missing nonce")?;
    if nonce.len() != 64 || !nonce.bytes().all(|c| c.is_ascii_hexdigit()) {
        return Err("invalid scheduler nonce".into());
    }
    let source = base64::engine::general_purpose::STANDARD
        .decode(request["source_b64"].as_str().ok_or("missing source")?)
        .map_err(|e| e.to_string())?;
    let source = String::from_utf8(source).map_err(|e| e.to_string())?;
    let started = std::time::Instant::now();
    let mut step = json!({"op":"run", "outcome":"infra", "value":null, "trap":null, "diag":null,
        "source_hash":format!("{:016x}",fnv1a64(source.as_bytes())), "wall_ms":0.0});

    let (result, capture) = run_chain(&source);
    match result {
        Err(diag) => {
            step["outcome"] = json!("diag");
            step["diag"] = diag;
        }
        Ok(outcome) => match outcome {
            ExecOutcome::Success {
                value: Value::I32(n),
                ..
            } => {
                step["outcome"] = json!("success");
                step["channel"] = json!("value");
                step["value"] = json!(n);
            }
            ExecOutcome::Success { .. } => {
                return Err("scheduler returned a non-entry result type".into())
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
                step["diag"] =
                    json!({"stage":"run","code":"cancelled","message":"execution cancelled"});
            }
            ExecOutcome::Error { message, .. } => {
                // The judge's error completions are run-stage diag steps
                // (the 965 lang-error precedent).
                step["outcome"] = json!("diag");
                step["diag"] = json!({"stage":"run","code":"lang.error","message":message});
            }
            ExecOutcome::Infra(message) => {
                step["outcome"] = json!("infra");
                step["diag"] = json!({"stage":"run","code":"host.infra","message":message});
            }
            // PLAN-973: a guest exit request is a normal named outcome
            // (the worker process itself keeps running).
            ExecOutcome::Exit { code } => {
                step["outcome"] = json!("exit");
                step["channel"] = json!("value");
                step["value"] = json!(code);
            }
        },
    }
    step["console"] = json!({"schema":"av.console-capture/1", "mir_lowering":ace_lowering::MIR_LOWERING_VERSION,
        "stdout_present":true,"stderr_present":true,"stdout_truncated":false,"stderr_truncated":false,
        "stdout_b64":base64::engine::general_purpose::STANDARD.encode(&capture.stdout),
        "stderr_b64":base64::engine::general_purpose::STANDARD.encode(&capture.stderr)});
    step["counters"] = json!({"host_dispatches":capture.dispatches,"store_writes":0});
    step["wall_ms"] = json!(started.elapsed().as_secs_f64() * 1000.0);
    Ok(
        json!({"recipe":RECIPE,"steps":[step],"worker_version":"avworker/1.0","av_core_version":AV_SCHEDULER_PROFILE}),
    )
}

/// The full producer→consumer chain; compile-stage diagnostics map to the
/// diag outcome (frontend codes) exactly like the console producer.
fn run_chain(source: &str) -> (Result<ExecOutcome, Json>, ConsoleCapture) {
    let mut capture = ConsoleCapture::default();
    let outcome = (|| -> Result<ExecOutcome, Json> {
        let compiled = ace_core::source::compile_scheduler_source(source, "probe.at", &[])
            .map_err(|ds| {
                let d = ds.first();
                json!({
                    "stage": d.map(|x| format!("{:?}", x.stage).to_lowercase()).unwrap_or_default(),
                    "code": d.map(|x| x.code.clone()).unwrap_or_default(),
                    "message": d.map(|x| x.message.clone()).unwrap_or_default()
                })
            })?;
        let mir = ace_lowering::lower_checked_closures(&compiled.checked, "d_main", 0)
            .map_err(|e| json!({"stage":"lower","code":"lower","message":format!("{e:?}")}))?;
        verify_mir(&mir)
            .map_err(|e| json!({"stage":"verify","code":"mir","message":format!("{e:?}")}))?;
        let abc = encode_abc(&mir)
            .map_err(|e| json!({"stage":"encode","code":"abc","message":format!("{e:?}")}))?;
        let flat = abc.to_flat();
        let loaded = ace_av_core::abc_verify::verify_abc_bytes(&flat, &mir.owner)
            .map_err(|e| json!({"stage":"admit","code":"abc","message":e.to_string()}))?;
        let (out, _trace) = execute_sched(
            &loaded,
            &ExecLimits::default(),
            &CancelFlag::new(false),
            &mut capture,
        );
        Ok(out)
    })();
    (outcome, capture)
}

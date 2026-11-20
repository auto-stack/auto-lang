//! PLAN-1078 language-source recipe producer: ordinary AV language sources
//! through the shared front door (Auto profile; first-batch semantics equal
//! the aggregates surface) onto the closures-family lowering/runtime with
//! console observation. Reports observations, never expected values or PASS.
use ace_av_core::abc::encode_abc;
use ace_av_core::av::{execute_sched, CancelFlag, ExecLimits};
use ace_av_core::contract::{fnv1a64, ExecOutcome, ObjectKey, Value};
use ace_av_core::verify_mir;
use base64::Engine;
use serde_json::{json, Value as Json};

use crate::console_host::ConsoleCapture;

pub const RECIPE: &str = "ac-av/language-source-run-v1";
/// The ordinary language source lowers through the same closures-superset
/// wire family as the aggregates face (no new MIR/ABC generation).
pub const AV_LANGUAGE_PROFILE: &str = "av.closures/1";

pub fn serve(request: &Json) -> Result<Json, String> {
    let object = request
        .as_object()
        .ok_or("language-source request must be an object")?;
    let allowed = [
        "recipe",
        "nonce",
        "request_hash",
        "source_b64",
        "source_path",
        "entry",
        "steps",
        // Present-but-inert optional fields of the shared request builder.
        "caps",
        "limits",
        "cancel_after_ms",
        "store_root",
    ];
    if let Some(bad) = object.keys().find(|k| !allowed.contains(&k.as_str())) {
        return Err(format!("unknown language-source request field: {bad}"));
    }
    if request["recipe"].as_str() != Some(RECIPE) || request["entry"].as_str() != Some("d_main") {
        return Err("language-source recipe requires d_main".into());
    }
    let steps = request["steps"]
        .as_array()
        .ok_or("missing language-source steps")?;
    if steps.len() != 1 || steps[0] != json!({"op":"run"}) {
        return Err("language-source requires exactly one run step".into());
    }
    let nonce = request["nonce"].as_str().ok_or("missing nonce")?;
    if nonce.len() != 64 || !nonce.bytes().all(|c| c.is_ascii_hexdigit()) {
        return Err("invalid language-source nonce".into());
    }
    let source = base64::engine::general_purpose::STANDARD
        .decode(request["source_b64"].as_str().ok_or("missing source")?)
        .map_err(|e| e.to_string())?;
    let source = String::from_utf8(source).map_err(|e| e.to_string())?;
    let path = match object.get("source_path") {
        None => "probe.at",
        Some(path) => path
            .as_str()
            .filter(|path| {
                !path.is_empty() && path.len() <= 4096 && !path.chars().any(char::is_control)
            })
            .ok_or("invalid language-source path")?,
    };
    // PLAN-1078: the declared caps enter the source admission/identity axis
    // (e.g. `ac.source.language.av/v1`); unknown/unadmitted capabilities are
    // rejected by the compile front door, never silently dropped.
    let caps: Vec<String> = request["caps"]
        .as_array()
        .map(|a| {
            a.iter()
                .filter_map(|c| c.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();
    let started = std::time::Instant::now();
    let mut step = json!({"op":"run", "outcome":"infra", "value":null, "trap":null, "diag":null,
        "source_hash":format!("{:016x}",fnv1a64(source.as_bytes())), "wall_ms":0.0});

    let (result, capture) = run_chain(&source, path, &caps);
    match result {
        Err(diag) => {
            step["outcome"] = json!("diag");
            if let Some(span) = diag.get("span") {
                step["source_span"] = span.clone();
            }
            step["diag"] = diag;
        }
        Ok(outcome) => match outcome {
            ExecOutcome::Success { value, .. } => {
                let value = match &value {
                    Value::Dyn(dynamic) => dynamic.as_value(),
                    other => other,
                };
                step["outcome"] = json!("success");
                match value {
                    Value::I32(value) => {
                        step["channel"] = json!("value");
                        step["value"] = json!(value);
                    }
                    Value::Unit => step["channel"] = json!("unit"),
                    Value::Str(_) | Value::Object(_) => match display_result(value) {
                        Some(text) => {
                            step["channel"] = json!(if matches!(value, Value::Str(_)) {
                                "text"
                            } else {
                                "object"
                            });
                            step["value_text"] = json!(text);
                        }
                        None => {
                            step["outcome"] = json!("resource");
                            step["diag"] = json!({"stage":"run", "code":"av.resource.result-display", "message":"result display exceeds its bounded value domain"});
                        }
                    },
                    other => {
                        return Err(format!(
                            "language-source returned unsupported result {}",
                            other.type_name()
                        ))
                    }
                }
            }
            ExecOutcome::Trap { kind, span } => {
                step["outcome"] = json!("trap");
                step["trap"] = json!(kind.code());
                if let Some(span) = span {
                    step["source_span"] = json!({"path":path, "start":span.start, "end":span.end});
                }
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
            ExecOutcome::Error { message, span } => {
                step["outcome"] = json!("diag");
                step["diag"] = json!({"stage":"run","code":"lang.error","message":message});
                if let Some(span) = span {
                    step["source_span"] = json!({"path":path, "start":span.start, "end":span.end});
                }
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
        json!({"recipe":RECIPE,"steps":[step],"worker_version":"avworker/1.0","av_core_version":AV_LANGUAGE_PROFILE}),
    )
}

fn display_result(value: &Value) -> Option<String> {
    enum Part {
        Value(Value),
        Key(ObjectKey),
        Text(&'static str),
    }
    const MAX_BYTES: usize = 1 << 20;
    let mut result = String::new();
    let mut stack = vec![Part::Value(value.clone())];
    let mut visited = 0usize;
    while let Some(part) = stack.pop() {
        visited += 1;
        if visited + stack.len() > ace_av_core::dynamic::MAX_DYN_NODES as usize {
            return None;
        }
        let mut append = |text: &str| {
            if result.len().saturating_add(text.len()) > MAX_BYTES {
                return None;
            }
            result.push_str(text);
            Some(())
        };
        match part {
            Part::Text(text) => append(text)?,
            Part::Key(ObjectKey::Text(text)) => append(std::str::from_utf8(text.bytes()).ok()?)?,
            Part::Key(ObjectKey::Int(value)) | Part::Value(Value::I32(value)) => {
                append(&value.to_string())?
            }
            Part::Key(ObjectKey::Bool(value)) | Part::Value(Value::Bool(value)) => {
                append(if value { "true" } else { "false" })?
            }
            Part::Value(Value::Nil) => append("None")?,
            Part::Value(Value::Str(text)) => append(std::str::from_utf8(text.bytes()).ok()?)?,
            Part::Value(Value::Dyn(value)) => stack.push(Part::Value(value.as_value().clone())),
            Part::Value(Value::Object(cell)) => {
                let cell = cell.borrow();
                if cell.entries.len().saturating_mul(4) + stack.len() + visited
                    > ace_av_core::dynamic::MAX_DYN_NODES as usize
                {
                    return None;
                }
                append("{")?;
                stack.push(Part::Text("}"));
                for (index, (key, value)) in cell.entries.iter().enumerate().rev() {
                    stack.push(Part::Value(value.clone()));
                    stack.push(Part::Text(": "));
                    stack.push(Part::Key(key.clone()));
                    if index > 0 {
                        stack.push(Part::Text(", "));
                    }
                }
            }
            Part::Value(_) => return None,
        }
    }
    Some(result)
}

/// The full producer→consumer chain: Auto-profile compile → closures
/// lowering → MIR verify → ABC admit → AV execute with console capture.
/// Compile-stage diagnostics map to the diag outcome (frontend codes)
/// exactly like the other source producers.
fn run_chain(
    source: &str,
    path: &str,
    caps: &[String],
) -> (Result<ExecOutcome, Json>, ConsoleCapture) {
    let mut capture = ConsoleCapture::default();
    let outcome = (|| -> Result<ExecOutcome, Json> {
        let compiled =
            ace_core::source::compile_language_source(source, path, caps).map_err(|ds| {
                let d = ds.first();
                json!({
                    "stage": d.map(|x| format!("{:?}", x.stage).to_lowercase()).unwrap_or_default(),
                    "code": d.map(|x| x.code.clone()).unwrap_or_default(),
                    "message": d.map(|x| x.message.clone()).unwrap_or_default(),
                    "span": d.map(|x| json!({"path":path, "start":x.span.start, "end":x.span.end})),
                })
            })?;
        // PLAN-1080: the ordinary language face lowers through the same
        // frozen closures-family MIR with the record-method def admission.
        let mir = ace_lowering::lower_checked_auto_language(&compiled.checked, "d_main", 0)
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

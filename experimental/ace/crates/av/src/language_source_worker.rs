//! PLAN-1078 language-source recipe producer: ordinary AV language sources
//! through the shared front door (Auto profile; first-batch semantics equal
//! the aggregates surface) onto the closures-family lowering/runtime with
//! console observation. Reports observations, never expected values or PASS.
use ace_av_core::abc::encode_abc;
use ace_av_core::av::{execute_sched, CancelFlag, ExecLimits, ExecTrace};
use ace_av_core::contract::{fnv1a64, ExecOutcome, LanguageErrorKind, ObjectKey, SpanInfo, Value};
use ace_av_core::mir_verify::MirVerifyError;
use ace_av_core::verify_mir;
use ace_core::atom_text::Diagnostic;
use ace_core::runtime_contract::sha256_hex;
use ace_core::source::cst::FrontBudget;
use ace_core::source::packages::{unit_key, PackageModuleInput};
use base64::Engine;
use serde::Deserialize;
use serde_json::{json, Value as Json};
use std::collections::BTreeMap;
use std::path::Path;

use crate::console_host::ConsoleCapture;

pub const RECIPE: &str = "ac-av/language-source-run-v1";
/// The ordinary language source lowers through the same closures-superset
/// wire family as the aggregates face, with capability-gated operations.
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
        "project",
        "entry",
        "steps",
        "caps",
        "store_root",
        // The shared request builder reserves these for the resource recipe.
        "limits",
        "cancel_after_ms",
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

    let mut store_writes = Some(0);
    let mut handled_errors = 0;
    let (result, capture) = if let Some(project) = object.get("project") {
        store_writes = None;
        let project: SourceProject =
            serde_json::from_value(project.clone()).map_err(|e| e.to_string())?;
        let store = request["store_root"]
            .as_str()
            .ok_or("missing project store root")?;
        let mut files = BTreeMap::from([(project.entry_path.clone(), source)]);
        for file in project.files {
            let bytes = base64::engine::general_purpose::STANDARD
                .decode(file.source_b64)
                .map_err(|e| format!("invalid project file {}: {e}", file.path))?;
            let source = String::from_utf8(bytes)
                .map_err(|e| format!("invalid UTF-8 project file {}: {e}", file.path))?;
            if files.insert(file.path, source).is_some() {
                return Err("duplicate project path".into());
            }
        }
        step["source_hash"] = json!(project_source_hash(&files));
        let mut capture = ConsoleCapture::default();
        let result = run_project(
            &project.entry_path,
            files,
            &caps,
            Path::new(store),
            &mut capture,
        );
        match result {
            Ok(observation) => {
                store_writes = Some(observation.store_writes);
                step["trace"] = json!(observation.trace);
                handled_errors = observation.handled_errors;
                if let Some(error) = observation.language_error {
                    step["language_error"] = error;
                }
                if let Some(span) = observation.source_span {
                    step["source_span"] = span;
                }
                (Ok(observation.outcome), capture)
            }
            Err(diag) => (Err(diag), capture),
        }
    } else {
        let mut capture = ConsoleCapture::default();
        let result = run_chain(&source, path, &caps, &mut capture).map(|(outcome, trace)| {
            handled_errors = trace.handled_language_errors;
            let locate = |span: SpanInfo| json!({"path":path,"start":span.start,"end":span.end});
            if let Some(error) = observed_language_error(&trace, locate) {
                step["language_error"] = error;
            }
            outcome
        });
        (result, capture)
    };
    match result {
        Err(diag) => {
            step["outcome"] = json!("diag");
            if let (Some(path), Some(span)) = (diag.get("path"), diag.get("span")) {
                step["source_span"] =
                    json!({"path":path, "start":span["start"], "end":span["end"]});
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
                    Value::Str(_) | Value::List(_) | Value::Object(_) => {
                        match display_result(value) {
                            Some(text) => {
                                step["channel"] =
                                    json!(if matches!(value, Value::Str(_) | Value::List(_)) {
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
                        }
                    }
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
                if let Some(span) = span.filter(|_| step["source_span"].is_null()) {
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
                if let Some(span) = span.filter(|_| step["source_span"].is_null()) {
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
    step["console"] = json!({"schema":"av.console-capture/1", "mir_lowering":ace_lowering::AUTO_LANGUAGE_MIR_LOWERING_VERSION,
        "stdout_present":true,"stderr_present":true,"stdout_truncated":false,"stderr_truncated":false,
        "stdout_b64":base64::engine::general_purpose::STANDARD.encode(&capture.stdout),
        "stderr_b64":base64::engine::general_purpose::STANDARD.encode(&capture.stderr)});
    step["counters"] = json!({"host_dispatches":capture.dispatches});
    step["counters"]["handled_language_errors"] = json!(handled_errors);
    if let Some(writes) = store_writes {
        step["counters"]["store_writes"] = json!(writes);
    }
    step["wall_ms"] = json!(started.elapsed().as_secs_f64() * 1000.0);
    Ok(
        json!({"recipe":RECIPE,"steps":[step],"worker_version":"avworker/1.0","av_core_version":AV_LANGUAGE_PROFILE}),
    )
}

fn display_result(value: &Value) -> Option<String> {
    enum Part {
        Value(Value),
        ListMember(Value),
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
            Part::ListMember(Value::Dyn(value)) => {
                stack.push(Part::ListMember(value.as_value().clone()));
            }
            Part::ListMember(Value::Str(text)) => {
                let text = std::str::from_utf8(text.bytes()).ok()?;
                if text.len() > MAX_BYTES {
                    return None;
                }
                append(&serde_json::to_string(text).ok()?)?;
            }
            Part::ListMember(value) => stack.push(Part::Value(value)),
            Part::Value(Value::List(list)) => {
                let list = list.rc.borrow();
                if list.elems.len().saturating_mul(2) + stack.len() + visited
                    > ace_av_core::dynamic::MAX_DYN_NODES as usize
                {
                    return None;
                }
                append("[")?;
                stack.push(Part::Text("]"));
                for (index, value) in list.elems.iter().enumerate().rev() {
                    stack.push(Part::ListMember(value.clone()));
                    if index > 0 {
                        stack.push(Part::Text(", "));
                    }
                }
            }
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

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SourceProject {
    entry_path: String,
    files: Vec<ProjectFile>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ProjectFile {
    path: String,
    source_b64: String,
}

fn diagnostic(diag: &Diagnostic, path: &str) -> Json {
    json!({
        "stage": format!("{:?}", diag.stage).to_lowercase(),
        "code": diag.code,
        "message": diag.message,
        "path": path,
        "span": {"start": diag.span.start, "end": diag.span.end}
    })
}

/// Resolve virtual Auto project paths to the package graph. The unqualified
/// project root is package `auto` (including legacy `use auto.mod: a, b`).
/// A single directory component in a project path names a dependency package.
fn project_module(path: &str, source: String) -> Option<PackageModuleInput> {
    let (package, file) = path.split_once('/').unwrap_or(("auto", path));
    let id = file.strip_suffix(".at")?;
    let valid_segment = |part: &str| {
        part.bytes()
            .next()
            .is_some_and(|b| b.is_ascii_alphabetic() || b == b'_')
            && part.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
    };
    if !valid_segment(package) || !valid_segment(id) {
        return None;
    }
    Some(PackageModuleInput {
        package: package.into(),
        version: "0".into(),
        id: id.into(),
        source,
        provider: None,
    })
}

// Match the parent Case IR's identity over the admitted virtual paths and bytes.
fn project_source_hash(files: &BTreeMap<String, String>) -> String {
    let mut combined = String::new();
    for (path, source) in files {
        combined.push_str(path);
        combined.push('\0');
        combined.push_str(&sha256_hex(source.as_bytes()));
        combined.push('\0');
    }
    sha256_hex(combined.as_bytes())
}

struct ProjectObservation {
    outcome: ExecOutcome,
    trace: Vec<&'static str>,
    source_span: Option<Json>,
    store_writes: u64,
    handled_errors: u64,
    language_error: Option<Json>,
}

fn observed_language_error(trace: &ExecTrace, locate: impl Fn(SpanInfo) -> Json) -> Option<Json> {
    let origin = trace.last_language_error?;
    let kind = match origin.kind {
        LanguageErrorKind::Explicit => "explicit".to_string(),
        LanguageErrorKind::ListIndex => "list-index".to_string(),
        LanguageErrorKind::Python(class) => format!("python.{class}"),
    };
    Some(json!({"kind":kind,"span":origin.span.map(locate)}))
}

fn run_project(
    entry_path: &str,
    files: BTreeMap<String, String>,
    caps: &[String],
    store: &Path,
    capture: &mut ConsoleCapture,
) -> Result<ProjectObservation, Json> {
    let path_diagnostic = |path: &str| {
        json!({
            "stage": "bind", "code": "source.module-path",
            "message": "project files must use module.at or package/module.at",
            "path": path, "span": {"start": 0, "end": 0}
        })
    };
    let mut paths = BTreeMap::new();
    let mut inputs = Vec::with_capacity(files.len());
    let mut entry = None;
    for (path, source) in files {
        let module = project_module(&path, source).ok_or_else(|| path_diagnostic(&path))?;
        let key = unit_key(&module.package, &module.id);
        if path == entry_path {
            entry = Some(key.clone());
        }
        if paths.insert(key.clone(), path.clone()).is_some() {
            return Err(json!({"stage":"bind","code":"source.module-identity",
                "message":format!("duplicate package module {key}"),"path":path,
                "span":{"start":0,"end":0}}));
        }
        inputs.push(module);
    }
    let entry = entry.ok_or_else(|| path_diagnostic(entry_path))?;
    let compiled = ace_source_query::modules::packages::compile_package_product_auto(
        &inputs,
        &entry,
        caps,
        FrontBudget::WORK_OWNED_64MIB,
    )
    .map_err(|ds| {
        let d = &ds[0];
        let path = paths.get(&d.module).map_or(entry_path, String::as_str);
        diagnostic(&d.diagnostic, path)
    })?;
    let target = crate::modules::packages::PackageAvTarget::cold(store, &compiled)
        .map_err(|message| json!({"stage":"admit","code":"av.package","message":message}))?;
    // The ordinary Case worker is a console-only provider. Do not turn a
    // source capability declaration into ambient platform I/O authority.
    let grants: Vec<_> = caps
        .iter()
        .filter(|cap| {
            matches!(
                cap.as_str(),
                ace_av_core::host::STDOUT | ace_av_core::host::STDERR
            )
        })
        .cloned()
        .collect();
    let mut backend = ace_av_core::platform_io::CapturePlatformIo::default();
    let (outcome, trace, _) = ace_av_core::av::execute_program_platform(
        &target.current,
        &ExecLimits::default(),
        &CancelFlag::default(),
        &grants,
        Some(capture),
        &mut backend,
    )
    .map_err(|e| json!({"stage":"admit","code":e.code,"message":e.message}))?;
    let marks = trace
        .events
        .iter()
        .map(|event| match event {
            ace_av_core::mir::TraceEvent::MarkA => "a",
            ace_av_core::mir::TraceEvent::MarkB => "b",
        })
        .collect();
    let locate = |span: SpanInfo| {
        use ace_core::atom_text::{Span, Stage};
        let located = compiled.bound.locate(Diagnostic::new(
            Stage::Text,
            "",
            "",
            Span {
                start: span.start as usize,
                end: span.end as usize,
            },
        ));
        let path = paths
            .get(&located.module)
            .map_or(entry_path, String::as_str);
        let span = located.diagnostic.span;
        json!({"path":path, "start":span.start, "end":span.end})
    };
    let source_span = match &outcome {
        ExecOutcome::Trap { span, .. } | ExecOutcome::Error { span, .. } => span.map(locate),
        _ => None,
    };
    Ok(ProjectObservation {
        outcome,
        trace: marks,
        source_span,
        store_writes: target.lowered,
        handled_errors: trace.handled_language_errors,
        language_error: observed_language_error(&trace, locate),
    })
}

/// The full producer→consumer chain: Auto-profile compile → closures
/// lowering → MIR verify → ABC admit → AV execute with console capture.
/// Compile-stage diagnostics map to the diag outcome (frontend codes)
/// exactly like the other source producers.
fn run_chain(
    source: &str,
    path: &str,
    caps: &[String],
    capture: &mut ConsoleCapture,
) -> Result<(ExecOutcome, ExecTrace), Json> {
        let compiled = ace_core::source::adapter::compile_language_source_with(
            source,
            path,
            caps,
            |program| ace_source_query::instantiate_auto_program(program, path, caps),
        )
        .map_err(|ds| diagnostic(&ds[0], path))?;
        // PLAN-1080: the ordinary language face lowers through the same
        // frozen closures-family MIR with the record-method def admission.
        let mir = ace_lowering::lower_checked_auto_language(&compiled.checked, "d_main", 0)
            .map_err(|e| json!({"stage":"lower","code":"lower","message":format!("{e:?}")}))?;
        verify_mir(&mir).map_err(|error| {
            let mut diagnostic = json!({
                "stage": "verify",
                "code": error.code(),
                "message": format!("{error:?}"),
            });
            if let MirVerifyError::Invalid {
                span: Some(span), ..
            }
            | MirVerifyError::Unsupported {
                span: Some(span), ..
            } = error
            {
                diagnostic["path"] = json!(path);
                diagnostic["span"] = json!({"start": span.start, "end": span.end});
            }
            diagnostic
        })?;
        let abc = encode_abc(&mir)
            .map_err(|e| json!({"stage":"encode","code":"abc","message":format!("{e:?}")}))?;
        let flat = abc.to_flat();
        let loaded = ace_av_core::abc_verify::verify_abc_bytes(&flat, &mir.owner)
            .map_err(|e| json!({"stage":"admit","code":"abc","message":e.to_string()}))?;
    Ok(execute_sched(
        &loaded,
        &ExecLimits::default(),
        &CancelFlag::new(false),
        capture,
    ))
}

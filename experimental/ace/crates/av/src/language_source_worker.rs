//! PLAN-1078 language-source recipe producer: ordinary AV language sources
//! through the shared front door (Auto profile; first-batch semantics equal
//! the aggregates surface) onto the closures-family lowering/runtime with
//! console observation. Reports observations, never expected values or PASS.
use ace_av_core::abc::encode_abc;
use ace_av_core::av::{execute_sched, CancelFlag, ExecLimits, ExecTrace};
use ace_av_core::contract::{fnv1a64, ExecOutcome, LanguageErrorKind, ObjectKey, SpanInfo, Value};
use ace_av_core::mir::{MirInstr, MirModule};
use ace_av_core::mir_verify::MirVerifyError;
use ace_av_core::verify_mir;
use ace_core::atom_text::Diagnostic;
use ace_core::runtime_contract::sha256_hex;
use ace_core::source::cst::{FrontBudget, SourceMode};
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
        // PLAN-1093: an optional prepared foreign provider binding.
        "foreign",
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
    let source_len = source.len();
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

    // PLAN-1095: the node CPU observation (979 producer on the same source)
    // is computed before dispatch — the project lane moves `source`.
    let node_obs = observe_node_facts(&source, path);

    let mut store_writes = Some(0);
    let mut handled_errors = 0;
    let mut runtime_calls = 0;
    let mut script_sources = 0;
    let mut ct_counters = ace_source_cte::CtCounters::default();
    let store_path = request["store_root"].as_str().map(Path::new);
    // The single-file program with its graft, kept to place a trap or an
    // error raised inside grafted standard-library text.
    let mut lowered: Option<MirModule> = None;
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
            &mut ct_counters,
        );
        match result {
            Ok(observation) => {
                store_writes = Some(observation.store_writes);
                step["trace"] = json!(observation.trace);
                handled_errors = observation.handled_errors;
                runtime_calls = observation.runtime_calls;
                script_sources = observation.script_sources;
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
    } else if let Some(foreign_value) = object.get("foreign") {
        // PLAN-1093: a prepared foreign provider rides the request; the
        // chain compiles through the shared admitted-foreign front door
        // and executes with per-run stub admission.
        let foreign: crate::foreign_source::ForeignSpec = serde_json::from_value(
            foreign_value.clone(),
        )
        .map_err(|e| format!("invalid foreign binding: {e}"))?;
        let root = Path::new(&foreign.artifact_root);
        if !root.is_dir() {
            return Err(format!(
                "prepared provider root missing: {}",
                root.display()
            ));
        }
        let mut capture = ConsoleCapture::default();
        let result =
            crate::foreign_source::run_foreign_chain(&source, path, &caps, &foreign, &mut capture)
                .map(|(outcome, trace)| {
                    handled_errors = trace.handled_language_errors;
                    runtime_calls = trace.call_frame_commits;
                    script_sources = 0;
                    let locate = |span: SpanInfo| {
                        in_submitted_source(span, source_len)
                            .then(|| json!({"path":path,"start":span.start,"end":span.end}))
                    };
                    if let Some(error) = observed_language_error(&trace, locate) {
                        step["language_error"] = error;
                    }
                    outcome
                });
        (result, capture)
    } else {
        let mut capture = ConsoleCapture::default();
        let result = run_chain(
            &source,
            path,
            &caps,
            store_path,
            &mut capture,
            &mut ct_counters,
        )
        .map(|(outcome, trace, mode, mir)| {
            handled_errors = trace.handled_language_errors;
            runtime_calls = trace.call_frame_commits;
            script_sources = u64::from(mode == SourceMode::Script);
            lowered = Some(mir);
            let locate = |span: SpanInfo| {
                in_submitted_source(span, source_len)
                    .then(|| json!({"path":path,"start":span.start,"end":span.end}))
            };
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
                // A span inside grafted standard-library text does not address
                // the submitted source; the report then names the program's
                // call that reached the raising function.
                if let Some(span) = reported_span(span, lowered.as_ref(), source_len)
                    .filter(|_| step["source_span"].is_null())
                {
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
                // As for traps: an uncaught error raised by grafted library
                // code is reported at the program's call that reached it.
                if let Some(span) = reported_span(span, lowered.as_ref(), source_len)
                    .filter(|_| step["source_span"].is_null())
                {
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
    // PLAN-1095: the node CPU observation. When the ordinary source carries
    // an AGE node declaration, the 979 producer observes the same source and
    // the shared execution chain produces the node's view text values — no
    // renderer, no event/scene lanes. Absent nodes → the field is absent.
    if let Some(nodes) = node_obs {
        step["nodes"] = json!(nodes);
    }
    step["counters"] = json!({"host_dispatches":capture.dispatches});
    step["counters"]["handled_language_errors"] = json!(handled_errors);
    step["counters"]["ct_execute"] = json!(ct_counters.ct_execute);
    step["counters"]["ct_hits"] = json!(ct_counters.ct_hits);
    step["counters"]["runtime_calls"] = json!(runtime_calls);
    step["counters"]["script_sources"] = json!(script_sources);
    if let Some(writes) = store_writes {
        step["counters"]["store_writes"] = json!(writes);
    }
    step["wall_ms"] = json!(started.elapsed().as_secs_f64() * 1000.0);
    Ok(
        json!({"recipe":RECIPE,"steps":[step],"worker_version":"avworker/1.0","av_core_version":AV_LANGUAGE_PROFILE}),
    )
}

/// PLAN-1095: observe the AGE node CPU facts of an ordinary source via the
/// 979 producer (`compile_age_h0` + the shared execution chain). Only runs
/// when the ordinary parse admits a node declaration; a non-H0 node body or
/// any other failure skips the observation (never fails the request).
fn observe_node_facts(source: &str, path: &str) -> Option<Vec<Json>> {
    use ace_core::source::{parse_program_with, SourceProfile};
    let program = parse_program_with(source, SourceProfile::Auto).ok()?;
    let has_node = program
        .items
        .iter()
        .any(|item| matches!(item, ace_core::source::ast::Item::Node(_)));
    if !has_node {
        return None;
    }
    let observed = crate::age_h0::observe_age_h0(
        source,
        path,
        ace_core::source::age_h0::LegacyWidgetAlias::Reject,
    )
    .ok()?;
    if observed.nodes.is_empty() {
        return None;
    }
    Some(
        observed
            .nodes
            .iter()
            .map(|n| {
                json!({
                    "name": n.name,
                    "head_role": n.head_role,
                    "view": {
                        "role": n.view.role,
                        "texts": n.view.texts.iter().map(|t| json!({
                            "role": t.role,
                            "value": t.value,
                            "kind": format!("{:?}", t.kind),
                        })).collect::<Vec<_>>(),
                    },
                })
            })
            .collect(),
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
    let id = file.strip_suffix(".at").or_else(|| file.strip_suffix(".as"))?;
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

/// A span that addresses the submitted source itself (the single-file graft
/// appends standard-library text after it).
fn in_submitted_source(span: SpanInfo, source_len: usize) -> bool {
    span.start <= span.end && (span.end as usize) <= source_len
}

struct ProjectObservation {
    outcome: ExecOutcome,
    trace: Vec<&'static str>,
    source_span: Option<Json>,
    store_writes: u64,
    handled_errors: u64,
    runtime_calls: u64,
    script_sources: u64,
    language_error: Option<Json>,
}

fn observed_language_error(
    trace: &ExecTrace,
    locate: impl Fn(SpanInfo) -> Option<Json>,
) -> Option<Json> {
    let origin = trace.last_language_error?;
    let kind = match origin.kind {
        LanguageErrorKind::Explicit => "explicit".to_string(),
        LanguageErrorKind::ListIndex => "list-index".to_string(),
        LanguageErrorKind::Http => "http.client".to_string(),
        LanguageErrorKind::Python(class) => format!("python.{class}"),
    };
    Some(json!({"kind":kind,"span":origin.span.and_then(locate)}))
}

fn run_project(
    entry_path: &str,
    files: BTreeMap<String, String>,
    caps: &[String],
    store: &Path,
    capture: &mut ConsoleCapture,
    ct_counters: &mut ace_source_cte::CtCounters,
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
    let compiled = ace_source_query::modules::packages::compile_package_product_auto_prepared(
        &inputs,
        &entry,
        caps,
        FrontBudget::WORK_OWNED_64MIB,
        |bound| {
            if !bound
                .segments
                .iter()
                .any(|segment| segment.source.contains('#'))
            {
                return Ok(());
            }
            // The checked source maps use the binding layer's flat byte
            // coordinates. This text is only consumed by emit, not parsed.
            let mut source = String::with_capacity(bound.source_len());
            for segment in &bound.segments {
                while source.len() < segment.start {
                    source.push('\n');
                }
                source.push_str(&segment.source);
            }
            prepare_ct(
                &mut bound.program,
                &source,
                &entry,
                &bound.capabilities,
                Some(store),
                Some(bound.input_hash.as_str()),
                ct_counters,
            )
        },
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
        runtime_calls: trace.call_frame_commits,
        script_sources: compiled
            .bound
            .units
            .iter()
            .filter(|(key, unit)| {
                paths.get(*key).is_some_and(|path| {
                    unit.source_metadata.pragmas.mode(path) == SourceMode::Script
                })
            })
            .count() as u64,
        language_error: observed_language_error(&trace, |span| {
            let located = compiled.bound.locate(Diagnostic::new(
                ace_core::atom_text::Stage::Text,
                "",
                "",
                ace_core::atom_text::Span {
                    start: span.start as usize,
                    end: span.end as usize,
                },
            ));
            paths.get(&located.module).map(|path| {
                let lspan = located.diagnostic.span;
                json!({"path": path, "start": lspan.start, "end": lspan.end})
            })
        }),
    })
}

fn prepare_ct(
    program: &mut ace_core::source::ast::Program,
    source: &str,
    path: &str,
    caps: &[String],
    store: Option<&Path>,
    package_identity: Option<&str>,
    counters: &mut ace_source_cte::CtCounters,
) -> Result<(), Vec<Diagnostic>> {
    use ace_core::atom_text::Stage;
    let sites = ace_source_cte::schedule::all_hash_sites(program);
    let Some((_, span)) = sites.first() else {
        return Ok(());
    };
    if !caps
        .iter()
        .any(|cap| cap == ace_core::source::cst::CT_CAPABILITY)
    {
        return Err(vec![Diagnostic::new(
            Stage::Bind,
            "source.cte.capability",
            "a compile-time expression requires ac.source.cte.scalar/v1",
            *span,
        )]);
    }
    let mut axes = caps.to_vec();
    let context = ace_core::source::packages::PackageAssemblyContext::local_av();
    axes.push(context.identity());
    axes.push(format!("{:?}", context.sys_facts()));
    axes.push(ace_lowering::AUTO_LANGUAGE_MIR_LOWERING_VERSION.into());
    let identity = package_identity.map_or_else(|| sha256_hex(source.as_bytes()), str::to_owned);
    if let Some(store) = store {
        let sink = ace_source_cte::store::ManagedEnvelopeSink::open(store, None).map_err(|e| {
            vec![Diagnostic::new(
                Stage::Bind,
                "cte.infra/store",
                e.to_string(),
                *span,
            )]
        })?;
        let mut service = ace_source_cte::CtService::for_auto(sink, axes);
        service.bind_source_closure(identity);
        prepare_ct_with_service(&mut service, program, source, path, caps, counters)
    } else {
        let mut service =
            ace_source_cte::CtService::for_auto(ace_source_cte::MemoryEnvelopeSink::new(), axes);
        service.bind_source_closure(identity);
        prepare_ct_with_service(&mut service, program, source, path, caps, counters)
    }
}

fn prepare_ct_with_service<S: ace_source_cte::CtEnvelopeSink>(
    service: &mut ace_source_cte::CtService<S>,
    program: &mut ace_core::source::ast::Program,
    source: &str,
    path: &str,
    caps: &[String],
    counters: &mut ace_source_cte::CtCounters,
) -> Result<(), Vec<Diagnostic>> {
    let result = ace_source_cte::product::prepare_auto_ct(service, program, source, path, caps);
    *counters = *service.counters();
    result
}

/// The full producer→consumer chain: Auto-profile compile → closures
/// lowering → MIR verify → ABC admit → AV execute with console capture.
/// Compile-stage diagnostics map to the diag outcome (frontend codes)
/// exactly like the other source producers.
fn run_chain(
    source: &str,
    path: &str,
    caps: &[String],
    store: Option<&Path>,
    capture: &mut ConsoleCapture,
    ct_counters: &mut ace_source_cte::CtCounters,
) -> Result<(ExecOutcome, ExecTrace, SourceMode, MirModule), Json> {
    let (compiled, metadata) = ace_core::source::adapter::compile_language_source_with_metadata(
        source,
        path,
        caps,
        |program, projected_source, has_instances| {
            if has_instances {
                ace_source_query::instantiate_auto_program(program, path, caps)?;
            }
            if source.contains('#') {
                prepare_ct(
                    program,
                    projected_source,
                    path,
                    caps,
                    store,
                    None,
                    ct_counters,
                )?;
            }
            Ok(())
        },
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
        authorize_console_calls(&mir, caps, path)?;
        let abc = encode_abc(&mir)
            .map_err(|e| json!({"stage":"encode","code":"abc","message":format!("{e:?}")}))?;
        let flat = abc.to_flat();
        let loaded = ace_av_core::abc_verify::verify_abc_bytes(&flat, &mir.owner)
            .map_err(|e| json!({"stage":"admit","code":"abc","message":e.to_string()}))?;
    let (outcome, trace) = execute_sched(
        &loaded,
        &ExecLimits::default(),
        &CancelFlag::new(false),
        capture,
    );
    Ok((outcome, trace, metadata.pragmas.mode(path), mir))
}

/// The span to report for a trap or an uncaught error. A span inside grafted
/// standard-library text does not address the submitted source: the report
/// then names the program's call that reached the raising function.
fn reported_span(
    span: Option<SpanInfo>,
    lowered: Option<&MirModule>,
    source_len: usize,
) -> Option<SpanInfo> {
    let span = span?;
    if in_submitted_source(span, source_len) {
        return Some(span);
    }
    library_call_site(lowered?, source_len, span.func)
}

/// The first call in the submitted source, in source order, to a function
/// that can reach `raised` — for a trap or an error raised inside grafted
/// standard-library text. Only a program that calls into the library can
/// get there, so some such call exists whenever the library ran.
fn library_call_site(lowered: &MirModule, source_len: usize, raised: u32) -> Option<SpanInfo> {
    let function_count = lowered.funcs.len();
    let raised = usize::try_from(raised)
        .ok()
        .filter(|f| *f < function_count)?;
    let in_source = |function: &ace_av_core::mir::MirFunction| {
        function
            .spans
            .iter()
            .any(|(_, span)| (span.start as usize) < source_len)
    };
    // Who calls whom, and where the submitted source calls.
    let mut callers: Vec<Vec<u32>> = vec![Vec::new(); function_count];
    let mut sites: Vec<(usize, SpanInfo)> = Vec::new();
    for (index, function) in lowered.funcs.iter().enumerate() {
        let own = in_source(function);
        let mut flat = 0u32;
        for block in &function.blocks {
            for instr in &block.instrs {
                if let MirInstr::Call {
                    callee: ace_av_core::mir::CalleeRef::Func(callee),
                    ..
                } = instr
                {
                    let callee = *callee as usize;
                    callers[callee].push(index as u32);
                    if own {
                        if let Ok(at) = function.spans.binary_search_by_key(&flat, |(at, _)| *at) {
                            sites.push((callee, function.spans[at].1));
                        }
                    }
                }
                flat += 1;
            }
        }
    }
    // Everything that can reach `raised`, by calls from the program.
    let mut reaching = vec![false; function_count];
    reaching[raised] = true;
    let mut work = vec![raised];
    while let Some(function) = work.pop() {
        for caller in std::mem::take(&mut callers[function]) {
            let caller = caller as usize;
            if !reaching[caller] {
                reaching[caller] = true;
                work.push(caller);
            }
        }
    }
    sites
        .into_iter()
        .filter(|(callee, _)| reaching[*callee])
        .map(|(_, span)| span)
        .min_by_key(|span| span.start)
}

#[cfg(test)]
mod compile_time_tests {
    use super::*;
    use ace_core::source::{cst::CT_CAPABILITY, AUTO_SOURCE_ADMISSION_CAP};
    use std::sync::atomic::Ordering;

    struct Scratch(std::path::PathBuf);

    impl Scratch {
        fn new() -> Self {
            static NEXT_DIR: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
            let root = std::env::var_os("AUTO_LANG_TEST_SCRATCH")
                .map(std::path::PathBuf::from)
                .unwrap_or_else(std::env::temp_dir);
            Self(root.join(format!(
                "ace-language-ct-cache-{}-{}",
                std::process::id(),
                NEXT_DIR.fetch_add(1, Ordering::Relaxed)
            )))
        }
    }

    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn request(source: &str, caps: &[&str], store: Option<&Path>) -> Json {
        request_with_path(source, "main.at", caps, store)
    }

    fn request_with_path(source: &str, path: &str, caps: &[&str], store: Option<&Path>) -> Json {
        let mut request = json!({
            "recipe": RECIPE,
            "nonce": "0".repeat(64),
            "source_b64": base64::engine::general_purpose::STANDARD.encode(source),
            "steps": [{"op": "run"}],
            "source_path": path,
            "entry": "d_main",
            "caps": caps,
        });
        if let Some(store) = store {
            request["store_root"] = json!(store);
        }
        serve(&request).expect("ordinary worker response")["steps"][0].clone()
    }

    #[test]
    fn plan1093_c_header_spelling_parses() {
        // PLAN-1093: `use.c <string.h>` — the C header import spelling
        // projects the header name; the ident form stays unchanged.
        use ace_core::source::{parse_program_with, SourceProfile, ast::Item};
        for src in ["use.c <x>\nfn main() int { return 1 }", "use.c <string.h>\nfn main() int { return 1 }", "use.c parity1093_c\nfn main() int { return 1 }"] {
            let program = parse_program_with(src, SourceProfile::Auto).expect("header spelling parses");
            let uses: Vec<_> = program.items.iter().filter_map(|i| match i {
                Item::UseRs(u) => Some(u),
                _ => None,
            }).collect();
            assert_eq!(uses.len(), 1, "{src}");
            let u = &uses[0];
            if src.contains('<') {
                // the header spelling projects the header name (no brackets).
                assert!(
                    !u.provider.name.contains('<') && !u.provider.name.contains('>'),
                    "{src}: {u:?}"
                );
            } else {
                assert_eq!(u.provider.name, "parity1093_c", "{src}");
            }
        }
    }
    /// PLAN-1124: an uncaught error raised inside grafted standard-library
    /// text is reported at the program's call that reached the library, so
    /// the checker judges it instead of refusing a span outside the
    /// submitted source; a failing call in the program itself keeps its own
    /// span.
    #[test]
    fn plan1124_uncaught_library_error_is_reported_at_the_call() {
        let caps = &[AUTO_SOURCE_ADMISSION_CAP];
        let in_source = |step: &Json, source: &str| {
            step["source_span"].as_object().is_none_or(|span| {
                span["end"]
                    .as_u64()
                    .is_some_and(|end| end as usize <= source.len())
            })
        };
        let library = "fn main() int {\n    let _a = csv_parse(\"\\\"unclosed quoted field\")\n    return 0\n}\n";
        let step = request(library, caps, None);
        assert_eq!(step["outcome"], "diag", "{step}");
        assert_eq!(step["diag"]["code"], "lang.error", "{step}");
        assert!(
            step["diag"]["message"]
                .as_str()
                .is_some_and(|message| message.contains("CsvError")),
            "{step}"
        );
        assert!(in_source(&step, library), "{step}");
        // The report names the program's call that reached the library —
        // the first such call in source order — never a span in the graft.
        let call = "csv_parse(\"\\\"unclosed quoted field\")";
        let call_at = library.find(call).expect("the call is in the program");
        let (start, end) = (
            step["source_span"]["start"].as_u64().expect("a start") as usize,
            step["source_span"]["end"].as_u64().expect("an end") as usize,
        );
        assert!(
            call_at <= start && start < end && end <= call_at + call.len(),
            "{step}"
        );

        let twice = "fn main() int {\n    let _a = csv_parse(\"a,b\")\n    let _b = csv_parse(\"\\\"unclosed quoted field\")\n    return 0\n}\n";
        let step = request(twice, caps, None);
        assert_eq!(step["diag"]["code"], "lang.error", "{step}");
        let first = twice.find("csv_parse(\"a,b\")").expect("the first call");
        assert!(
            step["source_span"]["start"].as_u64().expect("a start") as usize >= first
                && step["source_span"]["end"].as_u64().expect("an end") as usize
                    <= first + "csv_parse(\"a,b\")".len(),
            "{step}"
        );

        let own = "fn boom(msg str) int {\n    fail(msg)\n}\nfn main() int {\n    boom(\"mine\")\n    return 0\n}\n";
        let step = request(own, caps, None);
        assert_eq!(step["diag"]["message"], "mine", "{step}");
        assert!(step["source_span"].is_object(), "{step}");
        assert!(in_source(&step, own), "{step}");
    }

    #[test]
    fn ordinary_file_pragmas_use_the_original_path_and_rust_precedence() {
        let caps = &[AUTO_SOURCE_ADMISSION_CAP];
        for (path, markers, expected) in [
            ("main.at", "", 0),
            ("main.as", "", 1),
            ("main.at", "#[script]\n", 1),
            ("main.as", "#[rust]\n", 0),
            ("main.at", "#[script]\n#[rust]\n", 0),
            ("main.as", "#[rust]\n#[script]\n", 0),
        ] {
            let source = format!("{markers}fn main() int {{ return 42 }}");
            let step = request_with_path(&source, path, caps, None);
            assert_eq!(step["value"], 42, "{step}");
            assert_eq!(step["counters"]["script_sources"], expected, "{step}");
        }
        let duplicate = request(
            "#[script]\n#[script]\nfn main() int { return 42 }",
            caps,
            None,
        );
        assert_eq!(duplicate["diag"]["code"], "source.attribute-duplicate");
        assert_eq!(duplicate["source_span"]["start"], 12);
        assert_eq!(duplicate["source_span"]["end"], 18);
    }

    #[test]
    fn ordinary_annotations_keep_route_abi_and_rust_field_arguments() {
        use ace_core::source::cst::SourceAnnotationKind;

        let source = concat!(
            "fn label(n int) Str { return f\"the number $n\" }\n",
            "#[derive(Debug, Clone)]\n",
            "type Input { #[serde(rename = \"payload\")] value int }\n",
            "#[api(path = \"/item/:id\", method = \"POST\")]\n",
            "#[export(stdcall)]\n",
            "pub fn answer() int { return Input(42).value }\n",
            "fn main() int { return answer() }\n",
        );
        let caps = [AUTO_SOURCE_ADMISSION_CAP.to_string()];
        let (_, meta) = ace_core::source::adapter::compile_language_source_with_metadata(
            source,
            "main.at",
            &caps,
            |_, _, _| Ok(()),
        )
        .expect("ordinary checked source and annotation facts");
        let find = |decl, name| {
            meta.annotations
                .iter()
                .find(|fact| {
                    fact.declaration == decl && &source[fact.span.start..fact.span.end] == name
                })
                .expect("annotated source declaration")
        };
        let derive = find("Input", "derive");
        assert!(
            matches!(derive.kind, SourceAnnotationKind::Rust { ref name, .. } if name == "derive")
        );
        let serde = find("value", "serde");
        let SourceAnnotationKind::Rust {
            arguments: Some(span),
            ..
        } = serde.kind
        else {
            panic!("missing original Rust field arguments");
        };
        assert_eq!(&source[span.start..span.end], "(rename = \"payload\")");
        let api = find("answer", "api");
        assert!(matches!(
            api.kind,
            SourceAnnotationKind::Api { ref method, ref path }
                if method == "POST" && path == "/item/:id"
        ));
        assert!(source[api.owner_span.start..api.owner_span.end].starts_with("fn answer"));
        assert!(matches!(
            find("answer", "export").kind,
            SourceAnnotationKind::Export { ref abi } if abi == "system"
        ));
        let step = request(source, &[AUTO_SOURCE_ADMISSION_CAP], None);
        assert_eq!(step["value"], 42, "{step}");
        assert_eq!(step["counters"]["host_dispatches"], 0, "{step}");
    }

    #[test]
    fn package_field_edits_invalidate_metadata_and_reuse_an_unrelated_ct_value() {
        let store = Scratch::new();
        let source = concat!(
            "use lib.Record as Local\n",
            "fn unchanged(n int) int { return n + 39 }\n",
            "fn main() int { let base = #{ unchanged(1) }; ",
            "if #{ ct_has_field(Local, \"flag\") } { return base + #{ ct_field_count(Local) } }; ",
            "return 1 }",
        );
        let run = |library: &str| {
            let b64 = |text: &str| base64::engine::general_purpose::STANDARD.encode(text);
            let request = json!({
                "recipe": RECIPE,
                "nonce": "0".repeat(64),
                "source_b64": b64(source),
                "steps": [{"op": "run"}],
                "source_path": "main.at",
                "entry": "d_main",
                "caps": [AUTO_SOURCE_ADMISSION_CAP, CT_CAPABILITY, ace_core::source::META_CAPABILITY],
                "project": {
                    "entry_path": "main.at",
                    "files": [{"path": "lib.at", "source_b64": b64(library)}],
                },
                "store_root": &store.0,
            });
            serve(&request).expect("ordinary package AV response")["steps"][0].clone()
        };
        let initial = "pub type Record { flag bool; value int }";
        let cold = run(initial);
        assert_eq!(cold["value"], 42, "{cold}");
        assert_eq!(cold["counters"]["ct_execute"], 1, "{cold}");
        let extra = run("pub type Record { flag bool; value int; extra int }");
        let renamed = run("pub type Record { renamed bool; value int; extra int }");
        let restored = run(initial);
        for (after, value) in [(&extra, 43), (&renamed, 1), (&restored, 42)] {
            assert_eq!(after["value"], value, "{after}");
            assert_eq!(after["counters"]["ct_execute"], 0, "{after}");
            assert_eq!(after["counters"]["ct_hits"], 1, "{after}");
            assert_eq!(after["counters"]["runtime_calls"], 0, "{after}");
        }
        assert_ne!(extra["source_hash"], cold["source_hash"]);
        assert_ne!(extra["source_hash"], renamed["source_hash"]);
        assert_eq!(restored["source_hash"], cold["source_hash"]);
    }

    #[test]
    fn ct_dispatches_once_and_leaves_no_runtime_call() {
        let source = "fn twice(n int) int { return n * 2 }
fn main() int { return #{ twice(21) } }";
        let ct = request(source, &[AUTO_SOURCE_ADMISSION_CAP, CT_CAPABILITY], None);
        assert_eq!(ct["value"], 42, "{ct}");
        assert_eq!(ct["counters"]["ct_execute"], 1, "{ct}");
        assert_eq!(ct["counters"]["runtime_calls"], 0, "{ct}");
        let dynamic = request(
            &source.replace("#{ twice(21) }", "twice(21)"),
            &[AUTO_SOURCE_ADMISSION_CAP],
            None,
        );
        assert_eq!(dynamic["value"], 42, "{dynamic}");
        assert_eq!(dynamic["counters"]["ct_execute"], 0, "{dynamic}");
        assert_eq!(dynamic["counters"]["runtime_calls"], 1, "{dynamic}");
    }

    #[test]
    fn ct_cancel_reaches_the_av_and_preserves_the_demand_span() {
        let caps = vec![
            AUTO_SOURCE_ADMISSION_CAP.to_string(),
            CT_CAPABILITY.to_string(),
        ];
        let mut service = ace_source_cte::CtService::for_auto(
            ace_source_cte::MemoryEnvelopeSink::new(),
            caps.clone(),
        );
        service.cancel().store(true, Ordering::Relaxed);
        let source = "fn count(n int) int { var i = 0; for i < n { i = i + 1 }; return i }\nfn main() int { return #{ count(100000) } }";
        let result = ace_core::source::adapter::compile_language_source_prepared(
            source,
            "main.at",
            &caps,
            |program, source, _| {
                ace_source_cte::product::prepare_auto_ct(
                    &mut service,
                    program,
                    source,
                    "main.at",
                    &caps,
                )
            },
        );
        let error = result.err().expect("cancelled compilation");
        assert_eq!(error[0].code, "cte.cancelled");
        assert_eq!(
            &source[error[0].span.start..error[0].span.end],
            "#{ count(100000) }"
        );
        assert_eq!(service.counters().ct_execute, 1);
        assert_eq!(service.counters().ct_freeze_publishes, 0);
    }

    #[test]
    fn explicit_package_target_changes_ct_selection_and_identity() {
        use ace_core::source::ast::{ExprKind, Item, Stmt};
        use ace_core::source::packages::{
            bind_packages_auto_with_context, PackageAssemblyContext, TargetOs,
        };
        let source = "#if sys.current == sys.host { fn main() int { return 1 } } else { fn main() int { return 42 } }";
        let input = PackageModuleInput {
            package: "auto".into(),
            id: "main".into(),
            version: "0".into(),
            source: source.into(),
            provider: None,
        };
        let caps = [
            AUTO_SOURCE_ADMISSION_CAP.to_string(),
            ace_core::source::packages::PACKAGES_CAPABILITY.to_string(),
        ];
        let local = PackageAssemblyContext::local_av();
        let mut other = local.clone();
        other.target = if local.target == TargetOs::Macos {
            TargetOs::Windows
        } else {
            TargetOs::Macos
        };
        let bind = |context| {
            bind_packages_auto_with_context(
                std::slice::from_ref(&input),
                "auto::main",
                &caps,
                FrontBudget::WORK_OWNED_64MIB,
                context,
            )
            .expect("selected Auto package")
        };
        let local = bind(&local);
        let other = bind(&other);
        assert_ne!(local.input_hash, other.input_hash);
        for (bound, expected) in [(local, 1), (other, 42)] {
            let [Item::Fn(entry)] = &bound.program.items[..] else {
                panic!("expected one selected main")
            };
            let [Stmt::Return { value, .. }] = &entry.body.stmts[..] else {
                panic!("expected a selected return")
            };
            assert!(matches!(value.kind, ExprKind::IntLit(actual) if actual == expected));
        }
    }

    #[test]
    fn ct_cache_reopens_after_failed_edit_and_uses_current_source_and_grants() {
        let store = Scratch::new();
        let caps = [AUTO_SOURCE_ADMISSION_CAP, CT_CAPABILITY];
        let source = "fn f(n int) int { return n + 2 }
fn main() int { return #{ f(40) } }";
        let run = |text: &str, grants: &[&str]| request(text, grants, Some(&store.0));
        let first = run(source, &caps);
        assert_eq!(first["value"], 42, "{first}");
        assert_eq!(first["counters"]["ct_execute"], 1, "{first}");

        let bad = run(&source.replace("n + 2", "n / 0"), &caps);
        assert_eq!(bad["outcome"], "diag", "{bad}");
        assert!(
            bad["diag"]["code"]
                .as_str()
                .expect("trap code")
                .starts_with("cte.trap/"),
            "{bad}"
        );
        assert_eq!(bad["counters"]["ct_execute"], 1, "{bad}");

        let restored = run(source, &caps);
        assert_eq!(restored["value"], 42, "{restored}");
        assert_eq!(restored["counters"]["ct_execute"], 0, "{restored}");
        assert_eq!(restored["counters"]["ct_hits"], 1, "{restored}");

        let changed = source.replace("n + 2", "n + 3");
        let edit = run(&changed, &caps);
        assert_eq!(edit["value"], 43, "{edit}");
        assert_eq!(edit["counters"]["ct_execute"], 1, "{edit}");
        assert_eq!(edit["value"], request(&changed, &caps, None)["value"]);

        let new_grants = run(
            source,
            &[AUTO_SOURCE_ADMISSION_CAP, CT_CAPABILITY, "console.stdout"],
        );
        assert_eq!(new_grants["value"], 42, "{new_grants}");
        assert_eq!(new_grants["counters"]["ct_execute"], 1, "{new_grants}");
        assert_eq!(new_grants["counters"]["ct_hits"], 0, "{new_grants}");

        let revoked = run(source, &[AUTO_SOURCE_ADMISSION_CAP]);
        assert_eq!(
            revoked["diag"]["code"], "source.cte.capability",
            "{revoked}"
        );
        assert_eq!(revoked["counters"]["ct_hits"], 0, "{revoked}");
        assert_eq!(revoked["counters"]["ct_execute"], 0, "{revoked}");
    }
}

pub(crate) fn authorize_console_calls(mir: &MirModule, grants: &[String], path: &str) -> Result<(), Json> {
    for function in &mir.funcs {
        let instructions = function.blocks.iter().flat_map(|block| &block.instrs);
        for (index, instruction) in instructions.enumerate() {
            let MirInstr::HostCall { host, .. } = instruction else {
                continue;
            };
            let import = &mir.host_imports[*host as usize];
            if matches!(
                import.capability.as_str(),
                ace_av_core::host::STDOUT | ace_av_core::host::STDERR
            ) && grants.contains(&import.capability)
            {
                continue;
            }
            let message = format!("console capability `{}` is not granted", import.capability);
            let mut diagnostic =
                json!({"stage":"verify", "code":"host.unauthorized", "message":message});
            if let Ok(entry) = function.spans.binary_search_by_key(&(index as u32), |(instr, _)| *instr)
            {
                let (_, span) = function.spans[entry];
                diagnostic["path"] = json!(path);
                diagnostic["span"] = json!({"start":span.start, "end":span.end});
            }
            return Err(diagnostic);
        }
    }
    Ok(())
}

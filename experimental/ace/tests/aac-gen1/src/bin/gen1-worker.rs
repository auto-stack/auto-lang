//! `gen1-worker` — the bounded Gen1 corpus worker (PLAN-1053 T-03).
//!
//! One request on stdin, one JSON response on stdout, then exit — the same
//! single-exchange discipline as the mn-worker (`mnworker/1` precedent):
//! the AutoDown runner (auto-lang-core `ac_gen1`) spawns this worker per
//! case under Job containment and judges PARENT-SIDE; the worker never
//! sees the expected outcomes.
//!
//! Request (one JSON line):
//!   { "schema": "gen1worker/1",
//!     "prep_dir": "<absolute AAC1053_GEN1_DIR with the prepared exes>",
//!     "source_path": "<absolute .at source file>",
//!     "entry": "d_main",
//!     "budget_ms": 8000 }
//!
//! Response (one JSON line):
//!   { "schema": "gen1worker/1", "nonce": <echoed>,
//!     "outcome": { "kind": "exit", "code": 14 }
//!                | { "kind": "refused", "report_head": "ERROR bind.unresolved 0 1..5" }
//!                | { "kind": "failure", "code": "<bridge/link/run failure class>",
//!                  "detail": "<text>" } }
//!
//! The chain: aac1053-compile (the Gen1 Auto compiler, mir) -> ABC ->
//! ace-aac-bridge (the 1051 machine-backend service) -> COFF objects ->
//! parent-side link -> real PE run. Every tool identity is admitted from
//! the preparation manifests before use.

use std::io::{BufRead, Write};
use std::path::Path;
use std::process::Command;

#[macro_use]
extern crate serde_json;

fn main() {
    let stdin = std::io::stdin();
    let line = stdin
        .lock()
        .lines()
        .next()
        .and_then(|l| l.ok())
        .unwrap_or_default();
    let response = run_request(&line);
    let mut stdout = std::io::stdout().lock();
    let _ = stdout.write_all(response.as_bytes());
    let _ = stdout.write_all(b"\n");
}

fn run_request(line: &str) -> String {
    let Ok(req) = serde_json::from_str::<serde_json::Value>(line) else {
        return json!({"schema": "gen1worker/1", "outcome": {"kind": "failure", "code": "gen1.request", "detail": "malformed request line"}}).to_string();
    };
    let nonce = req.get("nonce").cloned().unwrap_or(serde_json::Value::Null);
    let respond = |outcome: serde_json::Value| {
        json!({"schema": "gen1worker/1", "nonce": nonce, "outcome": outcome}).to_string()
    };
    let Some(prep_dir) = req.get("prep_dir").and_then(|v| v.as_str()) else {
        return respond(json!({"kind": "failure", "code": "gen1.request", "detail": "prep_dir missing"}));
    };
    let Some(source_path) = req.get("source_path").and_then(|v| v.as_str()) else {
        return respond(json!({"kind": "failure", "code": "gen1.request", "detail": "source_path missing"}));
    };
    if !Path::new(prep_dir).is_absolute() || !Path::new(source_path).is_absolute() {
        return respond(json!({"kind": "failure", "code": "gen1.request", "detail": "prep_dir and source_path must be absolute"}));
    }
    let budget_ms = req
        .get("budget_ms")
        .and_then(|v| v.as_u64())
        .unwrap_or(8000)
        .min(15000) as u32;
    match run_chain(Path::new(prep_dir), Path::new(source_path), budget_ms) {
        Ok(Chain::Ran(code)) => respond(json!({"kind": "exit", "code": code})),
        Ok(Chain::Refused(head)) => respond(json!({"kind": "refused", "report_head": head})),
        Err(e) => respond(json!({"kind": "failure", "code": e.0, "detail": e.1})),
    }
}

enum Chain {
    Ran(i32),
    Refused(String),
}

fn run_chain(prep: &Path, source: &Path, budget_ms: u32) -> Result<Chain, (&'static str, String)> {
    use auto_tool_execution::{run_command_capture, ExitKind, TreeScope};
    let started = std::time::Instant::now();
    let compile = prep.join(ace_aac_gen1_conformance::COMPILE_EXE);
    let bridge_exe = prep.join(ace_aac_gen1_conformance::BRIDGE_EXE);
    for (name, exe) in [("compile", &compile), ("bridge", &bridge_exe)] {
        if !exe.is_file() {
            return Err(("gen1.preparation", format!("{name} exe missing from {prep:?}")));
        }
    }
    let admit = |exe_name: &str, manifest_name: &str| -> Result<(), (&'static str, String)> {
        let manifest = prep.join(manifest_name);
        let text = std::fs::read_to_string(&manifest)
            .map_err(|e| ("gen1.preparation", format!("read {manifest:?}: {e}")))?;
        let declared = text.lines().nth(1).unwrap_or("");
        let exe = prep.join(exe_name);
        let bytes = std::fs::read(&exe)
            .map_err(|e| ("gen1.preparation", format!("read {exe:?}: {e}")))?;
        if declared != blake3::hash(&bytes).to_hex().as_str() {
            return Err(("gen1.preparation", format!("{exe:?} digest differs from {manifest:?}")));
        }
        Ok(())
    };
    admit(
        ace_aac_gen1_conformance::COMPILE_EXE,
        ace_aac_gen1_conformance::COMPILE_MANIFEST,
    )?;
    admit(
        ace_aac_gen1_conformance::BRIDGE_EXE,
        ace_aac_gen1_conformance::TOOLS_MANIFEST,
    )?;

    let work = std::env::temp_dir().join(format!(
        "aac1053-gen1worker-{}-{}",
        std::process::id(),
        started.elapsed().as_nanos()
    ));
    std::fs::create_dir_all(&work).map_err(|e| ("gen1.io", e.to_string()))?;
    let output = work.join("report.txt");
    let abc_path = work.join("closure.abc");

    // 1. Gen1 compile (mir) — the Auto pipeline is the authority.
    let mut cmd = Command::new(&compile);
    cmd.current_dir(&work)
        .env("A2X_RT_BUDGET", "33554432")
        .env("AAC1047_INPUT", source)
        .env("AAC1047_OUTPUT", &output)
        .env("AAC1047_TYPED", "1")
        .env("AAC1047_HIR", "mir")
        .env_remove("AAC1047_PROJECT")
        .env_remove("AAC1047_STEPS")
        .env_remove("AAC1047_ARTIFACT")
        .env_remove("AAC1047_STAGED")
        .env_remove("AAC1047_FOREIGN_PATH")
        .env_remove("AAC1047_FOREIGN_AUTHORITY")
        .env_remove("AAC1055_ABC")
        .env_remove("AAC1049_RESOURCES")
        .env("AAC1055_ABC", &abc_path);
    let run = run_command_capture(cmd, TreeScope::ProcessTree, Some(std::time::Duration::from_millis(u64::from(budget_ms))))
        .map_err(|e| ("gen1.compile", e.to_string()))?;
    if !matches!(run.kind, ExitKind::Success) {
        return Err(("gen1.compile", format!("compile exited {:?}: {}", run.kind, String::from_utf8_lossy(run.stderr.as_deref().unwrap_or_default()))));
    }
    let report = std::fs::read_to_string(&output).map_err(|e| ("gen1.io", e.to_string()))?;
    let abc = match std::fs::read(&abc_path) {
        Ok(bytes) => Some(bytes),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
        Err(e) => return Err(("gen1.io", e.to_string())),
    };
    let Some(abc) = abc else {
        let _ = std::fs::remove_dir_all(&work);
        return Ok(Chain::Refused(report.trim_end().to_string()));
    };

    // 2. Bridge exchange — the 1051 machine-backend service.
    let decoded = ace_av_core::abc::decode_abc(&abc).map_err(|e| ("gen1.abc", format!("{e:?}")))?;
    let module = decoded.module;
    use ace_av_core::contract::*;
    let expected_schema: &str = match module.profile.as_str() {
        AV_PROFILE => MIR_SCHEMA,
        AV_DATA_PROFILE => MIR_DATA_SCHEMA,
        AV_OWNED_PROFILE => MIR_OWNED_SCHEMA,
        AV_COLLECTIONS_PROFILE => MIR_COLLECTIONS_SCHEMA,
        AV_CLOSURES_PROFILE => MIR_CLOSURES_SCHEMA,
        AV_DYNAMIC_PROFILE => MIR_DYNAMIC_SCHEMA,
        AV_ENTRY_UNIT_PROFILE => MIR_ENTRY_UNIT_SCHEMA,
        AV_HOST_CONSOLE_PROFILE => MIR_HOST_CONSOLE_SCHEMA,
        other => return Err(("gen1.abc", format!("unsupported ABC family: {other}"))),
    };
    let mode = ace_aac_bridge::Mode::NativeObjects;
    let source_bytes = std::fs::read(source).map_err(|e| ("gen1.io", e.to_string()))?;
    let request = ace_aac_bridge::Request {
        id: blake3::hash(&source_bytes).as_bytes()[..16]
            .try_into()
            .expect("16-byte request id"),
        mode,
        domains: ace_aac_bridge::Domains::windows(mode),
        inputs: ace_aac_bridge::InputIdentity {
            source: *blake3::hash(&source_bytes).as_bytes(),
            grammar: *blake3::hash(expected_schema.as_bytes()).as_bytes(),
            passes: [3; 32],
            stdlib: [4; 32],
            dependencies: [5; 32],
        },
        owner: if module.owner.module.is_empty() {
            "aac1053.gen1.worker".to_string()
        } else {
            module.owner.module.clone()
        },
        mir_schema: expected_schema.to_string(),
        options: ace_aac_bridge::Options { remaining_ms: budget_ms },
        capabilities: module.capabilities.clone(),
        external_provider: None,
        globals: module
            .globals
            .iter()
            .map(|global| ace_aac_bridge::WireGlobal {
                name: global.name.clone(),
                ty: global.ty,
                mutable: global.mutable,
            })
            .collect(),
        initializers: module.init.clone(),
    };
    let bridge = ace_aac_bridge::Client::new(&bridge_exe).map_err(|e| ("gen1.bridge", e.to_string()))?;
    let (response, bytes) = bridge
        .run(&request, &abc)
        .map_err(|e| ("gen1.bridge", e.to_string()))?;
    let ace_aac_bridge::Outcome::Objects { units, wrapper, .. } = response.outcome else {
        return Err(("gen1.bridge", format!("not an object bundle: {:?}", response.outcome)));
    };
    let mut remaining = bytes.as_slice();
    let mut objects = Vec::new();
    for object in units {
        let Some((unit, rest)) = remaining.split_at_checked(object.len as usize) else {
            return Err(("gen1.bridge", "truncated object bundle".into()));
        };
        objects.push(ace_native::link_driver::ObjectInput {
            name: object.symbol,
            bytes: unit.to_vec(),
        });
        remaining = rest;
    }

    // 3. Link + 4. run the real PE.
    let image = ace_native::link_driver::link_only(
        &objects,
        remaining,
        &wrapper.symbol,
        &work.join("link"),
        &[],
        "aac1053-gen1worker",
    )
    .map_err(|e| ("gen1.link", format!("{e:?}")))?;
    let mut spec = auto_tool_execution::ExecutionSpec::new([std::ffi::OsStr::new(&image.exe_path)]);
    spec.tree_scope = TreeScope::ProcessTree;
    let result = auto_tool_execution::run_bounded(
        &spec,
        &auto_tool_execution::BoundedPolicy {
            exec_deadline: std::time::Duration::from_millis(u64::from(budget_ms)),
            cleanup_deadline: std::time::Duration::from_secs(1),
            stdout_cap: Some(4096),
            stderr_cap: Some(4096),
        },
    )
    .map_err(|e| ("gen1.run", e.to_string()))?;
    let _ = std::fs::remove_dir_all(&work);
    match result.kind {
        ExitKind::Success => Ok(Chain::Ran(0)),
        ExitKind::Failure(code) => Ok(Chain::Ran(code)),
        other => Err(("gen1.run", format!("unexpected exit kind {other:?}"))),
    }
}

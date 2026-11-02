//! PLAN-913 T-02/T-05 probe: the FRESH-PROCESS source-bound restore oracle.
//!
//! No product entrypoint uses this binary. It receives NO expected result and
//! NO oracle value — only the store root, the parent's expected manifest id
//! and digest, the source text and the request's configuration facts. It
//! restores + deep-verifies + installs + links + executes, then (optionally)
//! takes ONE more single-body edit through the restored declaration context
//! and reports what that edit produced. It prints one JSON line including its
//! own PID so the caller can prove a different process ran the restore.
//!
//! usage: av_closures_913_restore STORE_ROOT MANIFEST_ID EXPECTED_DIGEST \
//!        SOURCE_PATH OWNER ENTRY PROFILE [CAPS_CSV] [EDIT_PATH]
//!
//! Exit codes: 0 = restored (+ edit) succeeded; 3 = controlled rejection;
//! 2 = usage.

use std::path::Path;
use std::sync::Arc;

use ace_av::closures_source_recovery::{restore_source_session, RestoreRequest};
use ace_av_core::function_code::FunctionCode;
use ace_core::body_work::Stage;
use serde_json::json;

const STAGES: &[Stage] = &[
    Stage::ParseProject,
    Stage::HirVerify,
    Stage::MirLower,
    Stage::MirVerify,
    Stage::AbcEncode,
    Stage::DecodeAdmit,
    Stage::IndexBuild,
    Stage::ArtifactWrite,
    Stage::RestartDecode,
    Stage::RestartDeepVerify,
];

fn reject(code: &str, message: impl Into<String>) -> ! {
    println!(
        "{}",
        json!({
            "ok": false,
            "code": code,
            "message": message.into(),
            "pid": std::process::id(),
        })
    );
    std::process::exit(3);
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() < 7 || args.len() > 9 {
        eprintln!(
            "usage: av_closures_913_restore STORE_ROOT MANIFEST_ID EXPECTED_DIGEST SOURCE_PATH OWNER ENTRY PROFILE [CAPS_CSV] [EDIT_PATH]"
        );
        std::process::exit(2);
    }
    let store_root = Path::new(&args[0]);
    let manifest = &args[1];
    let digest = &args[2];
    let source_path = &args[3];
    let owner = &args[4];
    let entry = &args[5];
    let profile = &args[6];
    let caps: Vec<String> = match args.get(7) {
        Some(list) if !list.is_empty() => list.split(',').map(str::to_string).collect(),
        _ => Vec::new(),
    };
    let edit_source = args.get(8).map(|p| {
        std::fs::read_to_string(p).unwrap_or_else(|e| {
            eprintln!("read edit source: {e}");
            std::process::exit(2);
        })
    });
    let source = match std::fs::read_to_string(source_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("read source: {e}");
            std::process::exit(2);
        }
    };
    let request = RestoreRequest {
        expected_manifest: manifest,
        expected_digest: digest,
        source: &source,
        logical_owner: owner,
        entry,
        profile,
        capabilities: &caps,
        read_only: true,
    };
    let restored = match restore_source_session(store_root, &request) {
        Ok(r) => r,
        Err(e) => reject(e.code, e.message),
    };
    let counters = restored.counters;
    let manifest_info = &restored.manifest;
    let base = json!({
        "ok": true,
        "pid": std::process::id(),
        "restore_value": restored.restore_value,
        "units": manifest_info.units.len(),
        "manifest_bytes": counters.manifest_bytes,
        "wire_bytes": counters.wire_bytes,
        "r_manifest_reads": counters.manifest_reads,
        "r_unit_reads": counters.unit_reads,
        "r_deep_verifies": counters.unit_deep_verifies,
        "r_admits": counters.unit_admits,
        "r_context_reads": counters.context_reads,
        "p_axis_attempts": counters.p_axis_attempts,
        "source_sha256": manifest_info.source_sha256,
        "context_revision": manifest_info.context_revision,
        "accepted_generation": manifest_info.accepted_generation,
    });
    let Some(edit_source) = edit_source else {
        println!("{base}");
        return;
    };
    // ONE more single-body edit through the restored context.
    let mut session = restored.session;
    let before: Vec<(String, Arc<FunctionCode>)> = session
        .unit_names()
        .into_iter()
        .filter_map(|name| session.unit(&name).cloned().map(|code| (name, code)))
        .collect();
    let run = match session.apply_source(&edit_source) {
        Ok(run) => run,
        Err(e) => reject("recovery.edit", format!("{e:?}")),
    };
    let mut unchanged_same_object = 0usize;
    let mut unchanged_total = 0usize;
    let mut changed_new_object = false;
    let mut changed_total = 0usize;
    for (name, code) in &before {
        let now = session.unit(name).expect("installed unit");
        if run.submitted_unit_names.iter().any(|s| s == name) {
            changed_total += 1;
            if !Arc::ptr_eq(now, code) {
                changed_new_object = true;
            }
        } else {
            unchanged_total += 1;
            if Arc::ptr_eq(now, code) {
                unchanged_same_object += 1;
            }
        }
    }
    let mut unchanged_production_attempts = 0u64;
    let changed_name = run
        .submitted_unit_names
        .first()
        .cloned()
        .unwrap_or_default();
    for stage in STAGES.iter().filter(|s| s.is_production()) {
        for name in session.unit_names() {
            if name == changed_name {
                continue;
            }
            unchanged_production_attempts += run.stages.of(&name, *stage);
        }
    }
    let stages: serde_json::Value = STAGES
        .iter()
        .map(|stage| (stage.as_str().to_string(), json!(run.stages.total(*stage))))
        .collect::<serde_json::Map<_, _>>()
        .into();
    let body: serde_json::Value = run
        .body
        .iter()
        .map(|(unit, work)| {
            (
                unit.clone(),
                json!({
                    "resolve": work.resolve,
                    "typecheck": work.typecheck,
                    "plan": work.plan,
                    "bind": work.bind,
                }),
            )
        })
        .collect::<serde_json::Map<_, _>>()
        .into();
    println!(
        "{}",
        json!({
            "base": base,
            "edit": {
                "route": run.route.as_str(),
                "route_detail": format!("{:?}", run.route),
                "value": run.value,
                "submitted": run.submitted_units,
                "submitted_names": run.submitted_unit_names,
                "whole_total": run.stages.whole_total(),
                "stages": stages,
                "body": body,
                "warm_gate_pass": run.warm_gate.as_ref().map(|g| g.pass()),
                "warm_gate_violations": run.warm_gate.as_ref().map(|g| g.violations.clone()),
                "unchanged_total": unchanged_total,
                "unchanged_same_object": unchanged_same_object,
                "unchanged_production_attempts": unchanged_production_attempts,
                "changed_total": changed_total,
                "changed_new_object": changed_new_object,
                "store_writes": run.production.store_writes,
                "p_verified": run.production.verified,
                "p_encoded": run.production.encoded,
                "p_admitted": run.production.admitted,
                "live_after_release": run.live_after_release,
                "allocs": run.allocs,
                "frees": run.frees,
            }
        })
    );
}

//! PLAN-913 T-02 probe: the FRESH-PROCESS source-bound restore oracle.
//!
//! No product entrypoint uses this binary. It receives NO expected result and
//! NO oracle value — only the store root, the parent's expected manifest id
//! and digest, the source text and the request's configuration facts. It
//! restores + deep-verifies + installs + links + executes, then prints one
//! JSON line (including its own PID so the caller can prove a different
//! process ran the restore).
//!
//! usage: av_closures_913_restore STORE_ROOT MANIFEST_ID EXPECTED_DIGEST SOURCE_PATH OWNER ENTRY PROFILE [CAPS_CSV]
//!
//! Exit codes: 0 = restored + executed; 3 = controlled recovery rejection;
//! 2 = usage.

use std::path::Path;

use ace_av::closures_session::ClosuresUpdateSession;
use ace_av::closures_source_recovery::{restore_source_recovery, RestoreRequest};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() < 7 || args.len() > 8 {
        eprintln!(
            "usage: av_closures_913_restore STORE_ROOT MANIFEST_ID EXPECTED_DIGEST SOURCE_PATH OWNER ENTRY PROFILE [CAPS_CSV]"
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
    let restored = match restore_source_recovery(store_root, &request) {
        Ok(r) => r,
        Err(e) => {
            println!(
                "{}",
                serde_json::json!({
                    "ok": false,
                    "code": e.code,
                    "message": e.message,
                    "pid": std::process::id(),
                })
            );
            std::process::exit(3);
        }
    };
    let counters = restored.counters;
    let mut session = ClosuresUpdateSession::new(None, entry);
    let outcome = match session.restore_install(restored.units) {
        Ok(o) => o,
        Err(e) => {
            println!(
                "{}",
                serde_json::json!({
                    "ok": false,
                    "code": "recovery.install",
                    "message": e,
                    "pid": std::process::id(),
                })
            );
            std::process::exit(3);
        }
    };
    println!(
        "{}",
        serde_json::json!({
            "ok": true,
            "pid": std::process::id(),
            "value": outcome.value,
            "live_after_release": outcome.live_after_release,
            "allocs": outcome.allocs,
            "frees": outcome.frees,
            "units": restored.manifest.units.len(),
            "manifest_bytes": counters.manifest_bytes,
            "wire_bytes": counters.wire_bytes,
            "r_manifest_reads": counters.manifest_reads,
            "r_unit_reads": counters.unit_reads,
            "r_deep_verifies": counters.unit_deep_verifies,
            "r_admits": counters.unit_admits,
            "p_axis_attempts": counters.p_axis_attempts,
            "p_verified": outcome.production.verified,
            "p_encoded": outcome.production.encoded,
            "p_admitted_installed": outcome.production.admitted,
            "store_writes": outcome.production.store_writes,
            "source_sha256": restored.manifest.source_sha256,
            "context_revision": restored.manifest.context_revision,
            "accepted_generation": restored.accepted_generation,
        })
    );
}

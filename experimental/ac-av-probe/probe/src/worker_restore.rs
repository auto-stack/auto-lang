//! Cross-process restore + re-verify + execute (PLAN-798 T-01/T-04).
//!
//! This module implements the restore path the worker binary exposes. It
//! receives NO source: the manifest artifact id (printed by the driver's
//! store round) locates the module closure, units load by artifact id, and
//! every layer re-validates (envelope identity → decode → verify) before
//! assembly and execution. A restored artifact set is data, never trust.

use crate::cache::{
    assemble_closure, decode_sidecar, get_by_artifact_id, merge_spans,
    EnvelopeIdentity,
};
use auto_av_core::mir::OwnerScope;
use auto_av_core::{abc_verify, decode_abc, execute, CancelFlag, ExecLimits};
use ac_artifact_probe::store::ManagedStore;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct RestoreOutcome {
    pub executed_ok: bool,
    pub value: Option<i32>,
    pub trap: Option<String>,
    pub error: Option<String>,
    pub units_loaded: usize,
    pub units_verified: usize,
    pub steps: Option<u64>,
}

/// Restore a module closure by manifest artifact id and execute its entry.
pub fn restore_and_run(
    store_dir: &std::path::Path,
    owner: &str,
    entry: &str,
    manifest_artifact: &str,
    snapshot: u64,
) -> RestoreOutcome {
    let store = match ManagedStore::open_existing(store_dir) {
        Ok(s) => s,
        Err(e) => {
            return RestoreOutcome {
                executed_ok: false,
                value: None,
                trap: None,
                error: Some(format!("open store: {}", e)),
                units_loaded: 0,
                units_verified: 0,
                steps: None,
            }
        }
    };
    // Manifest: envelope (code identity, snapshot 0) → JSON.
    let m_expect = EnvelopeIdentity::manifest(owner, 0);
    let m_bytes = match get_by_artifact_id(&store, manifest_artifact, &m_expect) {
        Ok(b) => b,
        Err(e) => return fail(format!("manifest load: {}", e)),
    };
    let manifest: crate::cache::ModuleManifest = match serde_json::from_slice(&m_bytes) {
        Ok(m) => m,
        Err(e) => return fail(format!("manifest decode: {}", e)),
    };
    if manifest.owner != owner || manifest.entry != entry {
        return fail(format!(
            "manifest identity mismatch: {}/{} vs expected {}/{}",
            manifest.owner, manifest.entry, owner, entry
        ));
    }
    // Units: per-artifact envelope + decode + verify; sidecar re-binds spans.
    let owner_scope = OwnerScope {
        schema: auto_av_core::contract::ABC_SCHEMA.to_string(),
        module: owner.to_string(),
    };
    let mut units = Vec::new();
    let mut verified = 0usize;
    for link in &manifest.units {
        let expect = EnvelopeIdentity::abc(owner);
        let payload = match get_by_artifact_id(&store, &link.artifact, &expect) {
            Ok(b) => b,
            Err(e) => return fail(format!("unit `{}` load: {}", link.name, e)),
        };
        let decoded = match decode_abc(&payload) {
            Ok(d) => d,
            Err(e) => return fail(format!("unit `{}` decode: {}", link.name, e.code())),
        };
        let mut module = decoded.module;
        module.owner = owner_scope.clone();
        for f in module.funcs.iter_mut() {
            f.owner = owner_scope.clone();
        }
        match abc_verify::verify_abc(
            auto_av_core::DecodedAbc {
                version: decoded.version,
                profile: decoded.profile,
                module: module.clone(),
            },
            &owner_scope,
        ) {
            Ok(_) => verified += 1,
            Err(e) => return fail(format!("unit `{}` re-verify: {}", link.name, e)),
        }
        // Diagnostics sidecar (best-effort; code content never depends on it).
        let skey = crate::cache::sidecar_key(owner, &link.name, snapshot);
        let sexpect = EnvelopeIdentity::sidecar(owner, snapshot);
        if let Ok(Some(sp)) =
            crate::cache::get_enveloped_opt(&store, &skey, "A08", &sexpect)
        {
            if let Ok(spans) = decode_sidecar(&sp) {
                merge_spans(&mut module, &spans);
            }
        }
        units.push(module);
    }
    let loaded = units.len();
    let merged = match assemble_closure(&units, owner, entry) {
        Ok(m) => m,
        Err(e) => return fail(format!("closure: {}", e)),
    };
    let bytes = match auto_av_core::encode_abc(&merged) {
        Ok(b) => b,
        Err(e) => return fail(format!("closure encode: {}", e.code())),
    };
    let decoded = match decode_abc(&bytes.to_flat()) {
        Ok(d) => d,
        Err(e) => return fail(format!("closure decode: {}", e.code())),
    };
    let loaded_module = match abc_verify::verify_abc(decoded, &owner_scope) {
        Ok(l) => l,
        Err(e) => return fail(format!("closure verify: {}", e)),
    };
    let cancel = CancelFlag::new(false);
    let (outcome, _trace) = execute(&loaded_module, &ExecLimits::default(), &cancel);
    match outcome {
        auto_av_core::contract::ExecOutcome::Success { value, steps } => RestoreOutcome {
            executed_ok: true,
            value: value.as_i32(),
            trap: None,
            error: None,
            units_loaded: loaded,
            units_verified: verified,
            steps: Some(steps),
        },
        auto_av_core::contract::ExecOutcome::Trap { kind, span } => RestoreOutcome {
            executed_ok: true,
            value: None,
            trap: Some(format!("{}@{}", kind.code(), span.map(|s| s.to_string()).unwrap_or_default())),
            error: None,
            units_loaded: loaded,
            units_verified: verified,
            steps: None,
        },
        other => fail(format!("execution state: {:?}", other)),
    }
}

fn fail(msg: String) -> RestoreOutcome {
    RestoreOutcome {
        executed_ok: false,
        value: None,
        trap: None,
        error: Some(msg),
        units_loaded: 0,
        units_verified: 0,
        steps: None,
    }
}

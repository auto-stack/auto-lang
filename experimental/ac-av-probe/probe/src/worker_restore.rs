//! Cross-process restore + re-verify + execute (PLAN-798 T-01/T-04).
//!
//! This module implements the restore path the worker binary exposes. It
//! receives NO source: the manifest artifact id (printed by the driver's
//! store round) locates the module closure, units load by artifact id, and
//! every layer re-validates (envelope identity → decode → verify) before
//! assembly and execution. A restored artifact set is data, never trust.
//!
//! F798-R3-02: restore admission is now CLOSURE-IDENTITY-COMPLETE — the
//! manifest's profile/av_sem, every link's re-derived logical id, its
//! signature rendered from the DECODED payload, and the persisted sema-ref
//! envelope's payload binding are all checked against the CURRENT request
//! (including the caller's live capability authorization) BEFORE anything
//! executes. A self-consistent envelope over a tampered inner layer is a
//! controlled rejection, never a green run.

use crate::cache::{
    assemble_closure, decode_sidecar, get_by_artifact_id, merge_spans, render_unit_sig,
    EnvelopeIdentity,
};
use ac_artifact_probe::store::ManagedStore;
use auto_av_core::contract::{ExecOutcome, TRACE_CAPABILITY};
use auto_av_core::mir::OwnerScope;
use auto_av_core::{abc_verify, decode_abc, execute, CancelFlag, ExecLimits, ExecTrace};
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
    pub av_peak_slot_bytes: u64,
}

/// Restore a module closure by manifest artifact id and execute its entry.
///
/// `caps` is the CURRENT request's capability authorization — persisted
/// "already granted" bits are never inherited (plan §5.4): a trace-using
/// module restored without a live trace authorization is rejected before
/// execution, not executed on the strength of its own claim.
#[allow(clippy::too_many_arguments)]
pub fn restore_and_run(
    store_dir: &std::path::Path,
    owner: &str,
    entry: &str,
    manifest_artifact: &str,
    snapshot: u64,
    caps: &[String],
) -> RestoreOutcome {
    match restore_and_run_inner(store_dir, owner, entry, manifest_artifact, snapshot, caps) {
        Ok(r) => r,
        Err(e) => fail(e),
    }
}

fn restore_and_run_inner(
    store_dir: &std::path::Path,
    owner: &str,
    entry: &str,
    manifest_artifact: &str,
    snapshot: u64,
    caps: &[String],
) -> Result<RestoreOutcome, String> {
    let store = ManagedStore::open_existing(store_dir).map_err(|e| format!("open store: {}", e))?;
    // Pin the manifest BEFORE reading: its required-data edges retain every
    // unit and current snapshot sidecar through validation and execution.
    let _manifest_lease = store
        .acquire_lease("plan798-restore-manifest", 300, Some(manifest_artifact))
        .map_err(|e| format!("lease manifest: {}", e))?;
    // The envelope stays position-free; the versioned recovery payload
    // and key bind the exact diagnostics snapshot, unlike reusable unit code.
    let m_expect = EnvelopeIdentity::manifest(owner, 0);
    let m_bytes = get_by_artifact_id(&store, manifest_artifact, &m_expect)
        .map_err(|e| format!("manifest load: {}", e))?;
    let manifest: crate::cache::ModuleManifest =
        serde_json::from_slice(&m_bytes).map_err(|e| format!("manifest decode: {}", e))?;
    if manifest.format != 2 {
        return Err(format!("unsupported manifest format {}", manifest.format));
    }
    if manifest.diagnostics_snapshot != snapshot {
        return Err(format!(
            "manifest snapshot mismatch: {:016x} vs requested {:016x}",
            manifest.diagnostics_snapshot, snapshot
        ));
    }
    if manifest.owner != owner || manifest.entry != entry {
        return Err(format!(
            "manifest identity mismatch: {}/{} vs expected {}/{}",
            manifest.owner, manifest.entry, owner, entry
        ));
    }
    // F798-R3-02: the CURRENT profile/semantics axes — not whatever the
    // persisted manifest claims.
    if manifest.profile != auto_av_core::contract::AV_PROFILE {
        return Err(format!(
            "manifest profile `{}` is not the current `{}`",
            manifest.profile,
            auto_av_core::contract::AV_PROFILE
        ));
    }
    if manifest.av_sem != auto_av_core::contract::AV_SEMANTICS.0 {
        return Err(format!(
            "manifest av semantics {} is not the current {}",
            manifest.av_sem,
            auto_av_core::contract::AV_SEMANTICS.0
        ));
    }
    // Units: per-artifact envelope + decode + verify; sidecar re-binds spans.
    let owner_scope = OwnerScope {
        schema: auto_av_core::contract::ABC_SCHEMA.to_string(),
        module: owner.to_string(),
    };
    let package = crate::semrefs::av_package();
    let mut units = Vec::new();
    // F798-R15-04: consumption leases — every unit artifact (and the
    // manifest itself) is held live for the whole restore+execute window;
    // dropped (released) at return, so concurrent GC cannot delete the
    // closure mid-consumption.
    let mut _held_leases: Vec<ac_artifact_probe::store::Lease> = Vec::new();
    let mut verified = 0usize;
    for link in &manifest.units {
        if link.body_fp.is_empty() {
            return Err(format!(
                "unit `{}` carries no body-fingerprint binding (legacy/unbound manifest)",
                link.name
            ));
        }
        // F798-R6-01: the artifact's envelope must declare the link's
        // (owner, unit, body fingerprint) — a self-consistent manifest
        // pointing at ANOTHER declaration's artifact fails here.
        let expect = EnvelopeIdentity::abc_bound(owner, &link.name, &link.body_fp);
        match store.acquire_lease("plan798-restore-run", 300, Some(&link.artifact)) {
            Ok(l) => _held_leases.push(l),
            Err(e) => return Err(format!("lease unit `{}`: {}", link.name, e)),
        }
        let payload = get_by_artifact_id(&store, &link.artifact, &expect)
            .map_err(|e| format!("unit `{}` load: {}", link.name, e))?;
        // F798-R3-02: payload binding — the sema-ref envelope must name the
        // EXACT bytes this restore is about to execute, and the link's
        // logical id must re-derive from the current owner/name.
        let payload_fp = crate::cache::payload_fingerprint(&payload);
        let logical = crate::semrefs::unit_logical_id(&package, owner, &link.name);
        if link.logical != logical {
            return Err(format!(
                "unit `{}` logical id `{}` does not re-derive to `{}` for owner `{}`",
                link.name, link.logical, logical, owner
            ));
        }
        let sema = link.sema.as_ref().ok_or_else(|| {
            format!(
                "unit `{}` carries no sema-ref envelope (legacy/unbound manifest)",
                link.name
            )
        })?;
        crate::semrefs::admit_restored_unit(sema, &payload_fp)
            .map_err(|e| format!("unit `{}` sema-ref admission failed: {}", link.name, e))?;
        if link.payload_fp != payload_fp {
            return Err(format!(
                "unit `{}` payload fingerprint `{}` does not match loaded bytes `{}`",
                link.name, link.payload_fp, payload_fp
            ));
        }
        let decoded = decode_abc(&payload)
            .map_err(|e| format!("unit `{}` decode: {}", link.name, e.code()))?;
        // F798-R15-02: the manifest's unit link (and the unit envelope it
        // was published under) declares a UNIT — a complete-module payload
        // is a producer/payload category conflict, refused here too.
        if !decoded.module.is_unit {
            return Err(format!(
                "unit `{}` payload declares complete-module kind under a unit link — category conflict",
                link.name
            ));
        }
        // F798-R6-01: the decoded payload's actual function must BE the
        // declaration the link names — a renamed link over another unit's
        // artifact is rejected, not executed under the wrong name.
        let actual_name = decoded
            .module
            .funcs
            .first()
            .map(|f| f.name.clone())
            .unwrap_or_default();
        if actual_name != link.name {
            return Err(format!(
                "unit `{}` payload actually contains function `{}` — declaration/payload mismatch",
                link.name, actual_name
            ));
        }
        let mut module = decoded.module;
        module.owner = owner_scope.clone();
        for f in module.funcs.iter_mut() {
            f.owner = owner_scope.clone();
        }
        // F798-R3-02: the claimed signature must match the DECODED payload.
        let actual_sig = render_unit_sig(&module);
        if link.sig != actual_sig {
            return Err(format!(
                "unit `{}` claimed signature `{}` but payload renders `{}`",
                link.name, link.sig, actual_sig
            ));
        }
        // F798-R3-02: capability re-admission against the CURRENT request —
        // a trace-requiring unit restores only under a live grant.
        for c in &module.capabilities {
            if c == TRACE_CAPABILITY && !caps.iter().any(|x| x == c) {
                return Err(format!(
                    "unit `{}` requires capability `{}` not granted by the current request",
                    link.name, c
                ));
            }
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
            Err(e) => return Err(format!("unit `{}` re-verify: {}", link.name, e)),
        }
        // Diagnostics sidecar is REQUIRED for the current snapshot
        // (F798-R6-03): a source-less restore cannot re-derive the source
        // mapping, so a missing/undecodable sidecar is a controlled
        // UNAVAILABLE rejection — never a successful restore that publishes
        // source-less traps.
        let skey = crate::cache::sidecar_key(owner, &link.name, snapshot);
        let sexpect = EnvelopeIdentity::sidecar(owner, snapshot);
        match crate::cache::get_enveloped_opt(&store, &skey, "A08", &sexpect) {
            Ok(Some(sp)) => match decode_sidecar(&sp) {
                Ok(spans) => {
                    merge_spans(&mut module, &spans);
                    // F798-R7-03/F798-R15-03: decodable is not APPLICABLE —
                    // the shared trap-coverage check (inverted spans and a
                    // span for every trap-capable instruction) decides; a
                    // non-covering map is refused exactly like a missing
                    // one (source-less restore cannot re-derive).
                    if let Err(e) = crate::cache::sidecar_applicable(&module.funcs[0]) {
                        return Err(format!(
                            "unit `{}` sidecar for snapshot {:016x} {} — diagnostics provenance unavailable; restore refused",
                            link.name, snapshot, e
                        ));
                    }
                }
                Err(e) => {
                    return Err(format!(
                        "unit `{}` sidecar for snapshot {:016x} is invalid: {}",
                        link.name, snapshot, e
                    ))
                }
            },
            Ok(None) => {
                return Err(format!(
                    "unit `{}` has no source sidecar for the current snapshot {:016x} (diagnostics provenance unavailable; restore refused)",
                    link.name, snapshot
                ))
            }
            Err(e) => {
                return Err(format!(
                    "unit `{}` sidecar for snapshot {:016x}: {}",
                    link.name, snapshot, e
                ))
            }
        }
        units.push(module);
    }
    let loaded = units.len();
    let merged = assemble_closure(&units, owner, entry).map_err(|e| format!("closure: {}", e))?;
    let bytes =
        auto_av_core::encode_abc(&merged).map_err(|e| format!("closure encode: {}", e.code()))?;
    let decoded =
        decode_abc(&bytes.to_flat()).map_err(|e| format!("closure decode: {}", e.code()))?;
    let loaded_module = abc_verify::verify_abc(decoded, &owner_scope)
        .map_err(|e| format!("closure verify: {}", e))?;
    let cancel = CancelFlag::new(false);
    let (outcome, trace): (ExecOutcome, ExecTrace) =
        execute(&loaded_module, &ExecLimits::default(), &cancel);
    match outcome {
        ExecOutcome::Success { value, steps } => Ok(RestoreOutcome {
            executed_ok: true,
            value: value.as_i32(),
            trap: None,
            error: None,
            units_loaded: loaded,
            units_verified: verified,
            steps: Some(steps),
            av_peak_slot_bytes: trace.peak_slot_bytes,
        }),
        ExecOutcome::Trap { kind, span } => Ok(RestoreOutcome {
            executed_ok: true,
            value: None,
            trap: Some(format!(
                "{}@{}",
                kind.code(),
                span.map(|s| s.to_string()).unwrap_or_default()
            )),
            error: None,
            units_loaded: loaded,
            units_verified: verified,
            steps: None,
            av_peak_slot_bytes: trace.peak_slot_bytes,
        }),
        other => Err(format!("execution state: {:?}", other)),
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
        av_peak_slot_bytes: 0,
    }
}

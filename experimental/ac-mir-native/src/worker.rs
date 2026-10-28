//! Worker entry: cross-process restore of an A09'/A10' closure — a NEW
//! process opens the store, re-admits every object from its bytes
//! (envelope identity + content hash + structural COFF check), re-emits
//! the wrapper, really links and really runs. The worker NEVER lowers
//! MIR — `codegen_emit_units=0` in its output is structural, and a hit
//! that secretly recompiled cannot happen here (there is no codegen path
//! in this binary at all beyond the tiny wrapper object).

use crate::cache::{self, CoffMeta, LinkManifest};
use crate::config::*;
use crate::link_driver::{self, ObjectInput};
use crate::{emit, MnError};
use ac_artifact_probe::store::ManagedStore;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct RestoreRunOutcome {
    pub exit_code: Option<i32>,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub units_admitted: usize,
    pub manifest_fp: String,
    pub workdir: PathBuf,
}

/// Restore by manifest KEY axes (owner module + entry + unit fps + link
/// inputs digest) — the same axes the driver computed, recomputed here
/// from CLI args so the worker never receives a raw artifact id it could
/// trust blindly: the key must resolve, and every unit still re-admits
/// from bytes.
#[allow(clippy::too_many_arguments)]
pub fn restore_run_by_key(
    store_root: &Path,
    owner_module: &str,
    entry: &str,
    unit_fps: &[(String, String)],
    link_inputs: &str,
    workdir: &Path,
    trace_lib: Option<&Path>,
    text_lib_sha: Option<&str>,
) -> Result<RestoreRunOutcome, MnError> {
    let store =
        ManagedStore::open(store_root).map_err(|e| MnError::Infra(format!("store open: {e}")))?;
    let owner = auto_av_core::mir::OwnerScope {
        schema: MIR_SCHEMA_AXIS.to_string(),
        module: owner_module.to_string(),
    };
    let Some(manifest) =
        cache::get_link_manifest_for(&store, &owner, entry, unit_fps, link_inputs, text_lib_sha)?
    else {
        return Err(MnError::Admit(
            "manifest",
            "no link manifest under the recomputed key (clean miss is not restore-run's job)"
                .to_string(),
        ));
    };
    run_manifest(&store, &manifest, workdir, trace_lib)
}

/// Restore by a manifest artifact id (the driver echoes it on stdout).
pub fn restore_run_by_id(
    store_root: &Path,
    manifest_artifact_id: &str,
    workdir: &Path,
    trace_lib: Option<&Path>,
) -> Result<RestoreRunOutcome, MnError> {
    let store =
        ManagedStore::open(store_root).map_err(|e| MnError::Infra(format!("store open: {e}")))?;
    let art = store
        .get_artifact_by_id(manifest_artifact_id)
        .map_err(|e| MnError::Infra(format!("manifest get: {e}")))?
        .ok_or_else(|| {
            MnError::Admit(
                "manifest",
                format!("manifest artifact {manifest_artifact_id} missing"),
            )
        })?;
    if art.kind != ac_artifact_probe::codec::ChunkKind::LayoutData {
        return Err(MnError::Admit(
            "manifest",
            format!(
                "manifest carries kind {:?}, not the LayoutData carrier",
                art.kind
            ),
        ));
    }
    let manifest = cache::parse_link_manifest(&art.payload)?;
    run_manifest(&store, &manifest, workdir, trace_lib)
}

fn run_manifest(
    store: &ManagedStore,
    manifest: &LinkManifest,
    workdir: &Path,
    trace_lib: Option<&Path>,
) -> Result<RestoreRunOutcome, MnError> {
    if manifest.abi != MN_ABI || manifest.target != MN_TARGET {
        return Err(MnError::Admit(
            "manifest",
            "manifest ABI/target axes are not the admitted pair".to_string(),
        ));
    }
    let mut objects: Vec<ObjectInput> = Vec::with_capacity(manifest.units.len());
    for u in &manifest.units {
        // The claimed symbol must equal the recomputed logical symbol
        // before any byte is read (self-consistent wrong address fails).
        let want = crate::symbols::function_symbol(&u.meta.owner, &u.meta.unit);
        if want != u.meta.symbol {
            return Err(MnError::Admit(
                "address",
                format!("manifest symbol `{}` != logical `{want}`", u.meta.symbol),
            ));
        }
        let bytes = cache::get_unit_coff_by_id(store, &u.meta, &u.artifact_id)?;
        let sha = cache::content_hash(&bytes);
        if sha != u.sha256 || bytes.len() != u.len {
            return Err(MnError::Admit(
                "sha256",
                format!(
                    "restored `{}` does not bind to the manifest claim",
                    u.meta.unit
                ),
            ));
        }
        objects.push(ObjectInput {
            name: u.meta.unit.clone(),
            bytes,
        });
    }
    let entry_unit = manifest
        .units
        .iter()
        .find(|u| u.meta.unit == manifest.entry)
        .ok_or_else(|| {
            MnError::Admit(
                "manifest",
                format!("entry unit `{}` absent from the manifest", manifest.entry),
            )
        })?;
    let wrapper = emit::emit_wrapper_coff(&entry_unit.meta.symbol)?;
    // PLAN-870: a manifest carrying the text-lib axis binds its identity —
    // the located runtime must hash to the recorded value before it may
    // join the link line.
    let mut support_libs: Vec<PathBuf> = Vec::new();
    if let Some(t) = trace_lib {
        support_libs.push(t.to_path_buf());
    }
    if let Some(want) = &manifest.text_lib_sha {
        let lib = auto_av_probe::native_reference::text_runtime_lib().map_err(MnError::Infra)?;
        let got = link_driver::file_identity(&lib)?;
        if got != *want {
            return Err(MnError::Admit(
                "textlib",
                format!("located text runtime identity `{got}` != manifest `{want}`"),
            ));
        }
        support_libs.push(lib);
    }
    let run = link_driver::link_and_run(
        &objects,
        &wrapper,
        START_SYMBOL,
        workdir,
        &support_libs,
        &format!(
            "restored-manifest {}\nunits {}\n",
            manifest.fingerprint(),
            manifest.units.len()
        ),
    )?;
    Ok(RestoreRunOutcome {
        exit_code: run.exit_code,
        stdout: run.stdout,
        stderr: run.stderr,
        units_admitted: manifest.units.len(),
        manifest_fp: manifest.fingerprint(),
        workdir: workdir.to_path_buf(),
    })
}

/// The unit meta the driver records per unit for worker handoff (already
/// inside the manifest — this helper recomputes it from a unit for the
/// key-axes form).
pub fn meta_of(unit: &auto_av_core::mir::MirModule) -> Result<CoffMeta, MnError> {
    CoffMeta::for_unit(unit, cache::unit_fp(unit))
}

//! Request driver: source → (798 front door) Checked HIR → MIR →
//! per-function units → A09' cache (miss = NEW codegen, hit = bytes only,
//! `codegen_emit_units` stays 0) → A10' manifest → real multi-object
//! link → real PE run. Every request links and runs — a cached PASS is
//! never a substitute for execution (contract-decision §9.4).

use crate::cache::{self, CoffMeta, LinkManifest, LinkUnitEntry};
use crate::config::*;
use crate::link_driver::{self, ObjectInput};
use crate::observe::Obs;
use crate::resources::Partition;
use crate::{emit, MnError};
use ac_artifact_probe::store::ManagedStore;
use auto_ac_prototype::source::compile_scalar_source;
use auto_av_core::mir_verify::{verify_mir, MirVerifyError};
use std::path::{Path, PathBuf};

pub struct MnDriver {
    store: Option<ManagedStore>,
    project: Option<String>,
    /// Fault seam for the hit-path counter control: when set, ANY attempt
    /// to lower a unit aborts — a "hit" that secretly recompiled cannot
    /// pass as a hit.
    pub forbid_codegen: bool,
    /// Current publication per (project, owner, entry): publishing a new
    /// manifest revision auto-roots it (783 soft root on a project-tagged
    /// put) and the SUPERSEDED revision's soft root is dropped right after
    /// (ordered publication, PLAN-787 R3-03) — the store's live set is the
    /// current publications, not every historical revision. Unit COFFs are
    /// cache entries (put WITHOUT a project tag): they stay live through
    /// the manifest `required` edges, so watermark GC can reclaim stale
    /// revisions without touching the current closure.
    last_publication: std::collections::HashMap<(String, String, String), String>,
}

pub struct MnRequest {
    pub ok: bool,
    pub error: Option<MnError>,
    pub exit_code: Option<i32>,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub obs: Obs,
    pub manifest_fp: Option<String>,
    /// A10' artifact id when a store was attached (worker handoff).
    pub manifest_artifact_id: Option<String>,
    pub workdir: Option<PathBuf>,
}

impl MnDriver {
    pub fn ephemeral() -> Self {
        MnDriver {
            store: None,
            project: None,
            forbid_codegen: false,
            last_publication: std::collections::HashMap::new(),
        }
    }

    pub fn with_store(root: &Path, project: &str) -> Result<Self, MnError> {
        Ok(MnDriver {
            store: Some(
                ManagedStore::open(root).map_err(|e| MnError::Infra(format!("store open: {e}")))?,
            ),
            project: Some(project.to_string()),
            forbid_codegen: false,
            last_publication: std::collections::HashMap::new(),
        })
    }

    /// One full request. `entry` is the HIR def id of the entry function
    /// (e.g. `d_main`); `workdir` receives the staged build products.
    pub fn request(
        &mut self,
        src: &str,
        entry: &str,
        caps: &[String],
        workdir: &Path,
        trace_lib: Option<&Path>,
    ) -> MnRequest {
        let mut obs = Obs::default();
        match self.request_inner(src, entry, caps, workdir, trace_lib, &mut obs) {
            Ok((exit, stdout, stderr, fp, art_id)) => MnRequest {
                ok: true,
                error: None,
                exit_code: exit,
                stdout,
                stderr,
                obs,
                manifest_fp: Some(fp),
                manifest_artifact_id: art_id,
                workdir: Some(workdir.to_path_buf()),
            },
            Err(e) => MnRequest {
                ok: false,
                error: Some(e),
                exit_code: None,
                stdout: vec![],
                stderr: vec![],
                obs,
                manifest_fp: None,
                manifest_artifact_id: None,
                workdir: Some(workdir.to_path_buf()),
            },
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn request_inner(
        &mut self,
        src: &str,
        entry: &str,
        caps: &[String],
        workdir: &Path,
        trace_lib: Option<&Path>,
        obs: &mut Obs,
    ) -> Result<(Option<i32>, Vec<u8>, Vec<u8>, String, Option<String>), MnError> {
        // ---- front-door admission + frontend (798 delivered surfaces) ---
        if src.len() > MAX_INPUT_BYTES {
            return Err(MnError::Unsupported(
                "input-size",
                format!("source is {} bytes, above the frozen input cap", src.len()),
            ));
        }
        let compiled = obs.span("frontend", |o| {
            compile_scalar_source(src, "mn.at", caps).map_err(|ds| {
                o.counter("frontend_reject", 1);
                MnError::Unsupported(
                    "frontend",
                    ds.first()
                        .map(|d| d.message.clone())
                        .unwrap_or_else(|| "frontend failed".into()),
                )
            })
        })?;
        let checked = &compiled.checked;

        // ---- HIR → MIR (798 public adapter) + verify ---------------------
        let snapshot = auto_av_core::contract::fnv1a64(src.as_bytes());
        let module = obs.span("mir_lower", |o| {
            auto_av_probe::hir_adapter::lower_checked(checked, entry, snapshot).map_err(|e| {
                o.counter("mir_reject", 1);
                let msg = match &e {
                    auto_av_probe::hir_adapter::AdapterError::Unsupported { message, .. }
                    | auto_av_probe::hir_adapter::AdapterError::Invalid { message, .. } => {
                        message.clone()
                    }
                };
                MnError::Unsupported("adapter", msg)
            })
        })?;
        let checked_mir = obs.span("mir_verify", |o| {
            verify_mir(&module).map_err(|e| {
                o.counter("mir_verify_reject", 1);
                let code = match &e {
                    MirVerifyError::Unsupported { code, .. } => format!("mir.{code}"),
                    MirVerifyError::Invalid { code, .. } => format!("mir.{code}"),
                    MirVerifyError::Resource { kind, .. } => kind.code().to_string(),
                };
                MnError::Unsupported("mir", format!("{code}: {e:?}"))
            })
        })?;

        // ---- split into per-function units (798 delivered helper) --------
        let units = obs.span("mir_split", |_| {
            auto_av_probe::cache::split_units(checked_mir.module())
                .map_err(|e| MnError::Backend(format!("unit split: {e}")))
        })?;
        obs.counter("mir_units_split", units.len() as u64);
        // The units stay resident for the whole request: their structural
        // heap estimate is the MIR/symbol-hot partition charge (T-05 §11).
        for u in &units {
            obs.ledger
                .charge(Partition::MirSymbol, u.heap_estimate() as u64);
        }

        // ---- per-unit: A09' lookup → hit(admit) | miss(codegen+persist) --
        let mut objects: Vec<ObjectInput> = Vec::with_capacity(units.len());
        let mut entries: Vec<LinkUnitEntry> = Vec::with_capacity(units.len());
        let mut emitted_total = 0usize;
        for unit in &units {
            // The fingerprint render is transient codec staging.
            let fp = obs
                .ledger
                .with_charge(Partition::Staging, 4096, |_| cache::unit_fp(unit));
            let meta = obs.span("unit_fp", |_| CoffMeta::for_unit(unit, fp))?;
            let entry_row: Option<LinkUnitEntry>;
            let bytes: Vec<u8>;
            match self.store.as_ref() {
                Some(store) => {
                    match obs.span("cache_lookup", |_| cache::get_unit_coff(store, &meta))? {
                        Some(b) => {
                            obs.counter("cache_hit_coff", 1);
                            // Restored object bytes: I/O read buffers held
                            // through the link (T-05 §11 partition).
                            obs.ledger.charge(Partition::Io, b.len() as u64);
                            // The manifest binds the concrete artifact id.
                            let id = store
                                .get_artifact(COFF_LAYER.id, &cache::coff_unit_key(&meta))
                                .map_err(|e| MnError::Infra(format!("store id lookup: {e}")))?
                                .map(|a| a.artifact_id)
                                .ok_or_else(|| {
                                    MnError::Admit(
                                        "address",
                                        format!("admitted unit `{}` has no artifact id", meta.unit),
                                    )
                                })?;
                            entry_row = Some(LinkUnitEntry {
                                sha256: cache::content_hash(&b),
                                len: b.len(),
                                artifact_id: id,
                                extra_page_ids: vec![],
                                meta: meta.clone(),
                            });
                            bytes = b;
                        }
                        None => {
                            if self.forbid_codegen {
                                return Err(MnError::Admit(
                                    "forbidden-codegen",
                                    "hit path attempted to compile (forbid_codegen seam)".into(),
                                ));
                            }
                            let obj = obs.span("codegen", |o| {
                                o.counter("codegen_emit_units", 1);
                                emit::emit_unit_coff(unit)
                            })?;
                            emitted_total += obj.bytes.len();
                            // Emitted object bytes held for the link:
                            // lowering/codec staging (T-05 §11).
                            obs.ledger
                                .charge(Partition::Staging, obj.bytes.len() as u64);
                            let (id, extra) = obs.span("coff_persist", |o| {
                                o.ledger.with_charge(
                                    Partition::Io,
                                    obj.bytes.len() as u64 + 512,
                                    |_| {
                                        // Unit COFFs are CACHE entries, not
                                        // publications: no project tag (a
                                        // tagged put would auto-soft-root
                                        // them and pin every revision).
                                        cache::put_unit_coff(
                                            store,
                                            None,
                                            &meta,
                                            &obj.bytes,
                                        )
                                    },
                                )
                            })?;
                            obs.counter("coff_objects_persisted", 1);
                            entry_row = Some(LinkUnitEntry {
                                sha256: cache::content_hash(&obj.bytes),
                                len: obj.bytes.len(),
                                artifact_id: id,
                                extra_page_ids: extra,
                                meta: meta.clone(),
                            });
                            bytes = obj.bytes;
                        }
                    }
                }
                None => {
                    // Ephemeral: no store — the pure new-codegen path.
                    if self.forbid_codegen {
                        return Err(MnError::Admit(
                            "forbidden-codegen",
                            "hit path attempted to compile (forbid_codegen seam)".into(),
                        ));
                    }
                    let obj = obs.span("codegen", |o| {
                        o.counter("codegen_emit_units", 1);
                        emit::emit_unit_coff(unit)
                    })?;
                    emitted_total += obj.bytes.len();
                    obs.ledger.charge(Partition::Staging, obj.bytes.len() as u64);
                    entry_row = None;
                    bytes = obj.bytes;
                }
            }
            objects.push(ObjectInput {
                name: unit.funcs[0].name.clone(),
                bytes,
            });
            if let Some(e) = entry_row {
                entries.push(e);
            }
        }
        if emitted_total > MAX_REQUEST_OBJECTS_BYTES {
            return Err(MnError::Resource(
                "request-objects",
                format!("emitted objects total {emitted_total} bytes"),
            ));
        }

        // ---- link identities + manifest -----------------------------------
        let entry_unit = units
            .iter()
            .find(|u| u.funcs[0].name == entry)
            .ok_or_else(|| {
                MnError::Unsupported(
                    "entry",
                    format!("entry unit `{entry}` not among the split units"),
                )
            })?;
        let entry_meta = CoffMeta::for_unit(entry_unit, cache::unit_fp(entry_unit))?;
        let (lld_sha, sdk_tag, k32_sha) =
            obs.span("link_identity", |_| link_driver::link_identities())?;
        let trace_lib_sha = match trace_lib {
            Some(p) => Some(link_driver::file_identity(p)?),
            None => None,
        };
        // Deterministic manifest unit order (declaration identity, not
        // link order — the symbol scheme is order-independent).
        entries.sort_by(|a, b| a.meta.unit.cmp(&b.meta.unit));
        let manifest = LinkManifest {
            owner: checked_mir.module().owner.clone(),
            entry: entry.to_string(),
            entry_symbol: entry_meta.symbol.clone(),
            abi: MN_ABI.to_string(),
            target: MN_TARGET.to_string(),
            units: entries,
            lld_sha,
            sdk_tag,
            kernel32_sha: k32_sha,
            trace_lib_sha,
        };
        // Manifest render: index/codec staging for the closure record.
        obs.ledger
            .charge(Partition::Staging, manifest.render().len() as u64);
        let fp = manifest.fingerprint();
        let mut manifest_artifact_id = None;
        if let Some(store) = &self.store {
            let id = obs.span("manifest_persist", |o| {
                o.counter("link_manifests_persisted", 1);
                cache::put_link_manifest(store, self.project.as_deref(), &manifest)
            })?;
            // Ordered publication (787 R3-03): the new revision is rooted
            // by its put; the superseded revision's soft root drops now.
            // The current closure stays live via the manifest `required`
            // edges; stale revisions become GC-reclaimable.
            if let Some(project) = self.project.as_deref() {
                let key = (
                    project.to_string(),
                    manifest.owner.module.clone(),
                    manifest.entry.clone(),
                );
                if let Some(prev) = self.last_publication.insert(key, id.clone()) {
                    let _ = store.remove_root(project, "soft", &prev);
                }
            }
            manifest_artifact_id = Some(id);
        }

        // ---- wrapper + real multi-object link + real run ------------------
        let wrapper = obs.span("codegen", |o| {
            o.counter("codegen_emit_wrappers", 1);
            emit::emit_wrapper_coff(&entry_meta.symbol)
        })?;
        obs.ledger.charge(Partition::Staging, wrapper.len() as u64);
        let run = obs.span("link_run", |o| {
            o.counter("link_invocations", 1);
            o.counter("run_invocations", 1);
            link_driver::link_and_run(
                &objects,
                &wrapper,
                START_SYMBOL,
                workdir,
                trace_lib,
                &format!("manifest-fp {fp}\nunits {}\n", manifest.units.len()),
            )
        })?;
        obs.counter("mn_runs", 1);
        // Ledger peaks enter the observable trace (T-05 evidence).
        let peaks = obs.ledger.peaks();
        let peak_total = obs.ledger.peak_total();
        for (i, name) in crate::resources::PARTITION_NAMES.iter().enumerate() {
            obs.counter(&format!("ledger_peak_{name}"), peaks[i] as u64);
        }
        obs.counter("ledger_peak_total", peak_total as u64);
        Ok((
            run.exit_code,
            run.stdout,
            run.stderr,
            fp,
            manifest_artifact_id,
        ))
    }
}

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
use auto_ac_prototype::link::CaptureCaps;
use auto_av_core::mir::MirType;
use auto_av_core::mir_verify::{verify_mir, MirVerifyError};
use std::path::{Path, PathBuf};

/// The narrowest source domain that admits a request (PLAN-870 freeze §4):
/// scalar → record → text-record escalation picks the narrowest compiling
/// profile, so B* sources keep their `av.core-i32/1` unit identity while
/// T*/R* sources reach `av.owned/1`/`av.data/1` through the SAME shared
/// frontend stack AV uses (diagnostics included).
enum Domain {
    Scalar(auto_ac_prototype::source::CompiledSource),
    Data(auto_ac_prototype::source::CompiledSource),
    Owned(auto_ac_prototype::source::CompiledSource),
}

/// A profile-surface rejection (`source.unsupported` out of the Text
/// stage) — the only failure class that escalates to the next wider
/// domain; every other diagnostic (parse, bind, verify) is terminal and
/// reports from the domain that produced it, exactly like AV's explicit
/// per-domain drivers.
fn is_surface_rejection(ds: &[auto_ac_prototype::atom_text::Diagnostic]) -> bool {
    ds.iter().all(|d| {
        d.code == "source.unsupported" && d.stage == auto_ac_prototype::atom_text::Stage::Text
    })
}

fn compile_escalating(
    src: &str,
    caps: &[String],
) -> std::result::Result<Domain, Vec<auto_ac_prototype::atom_text::Diagnostic>> {
    match auto_ac_prototype::source::compile_scalar_source(src, "mn.at", caps) {
        Ok(c) => return Ok(Domain::Scalar(c)),
        Err(ds) if is_surface_rejection(&ds) => {}
        Err(ds) => return Err(ds),
    }
    match auto_ac_prototype::source::compile_record_source(src, "mn.at", caps) {
        Ok(c) => return Ok(Domain::Data(c)),
        Err(ds) if is_surface_rejection(&ds) => {}
        Err(ds) => return Err(ds),
    }
    auto_ac_prototype::source::compile_text_record_source(src, "mn.at", caps).map(Domain::Owned)
}

impl Domain {
    fn arm(&self) -> Arm {
        match self {
            Domain::Scalar(_) => Arm::Scalar,
            Domain::Data(_) => Arm::Data,
            Domain::Owned(_) => Arm::Owned,
        }
    }
    fn checked(&self) -> &auto_ac_prototype::verify::CheckedModule {
        match self {
            Domain::Scalar(c) | Domain::Data(c) | Domain::Owned(c) => &c.checked,
        }
    }
}

/// Which shared-lowering arm a module's HIR domain selects (PLAN-871): the
/// backend entry reuses the same arm the escalating source front door
/// picked — Core/Scalar ride `lower_checked`, Record `lower_checked_data`,
/// Text/TextRecord `lower_checked_owned`.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Arm {
    Scalar,
    Data,
    Owned,
}

fn arm_of_domain(d: auto_ac_prototype::hir::HirDomain) -> Option<Arm> {
    use auto_ac_prototype::hir::HirDomain;
    Some(match d {
        HirDomain::Core | HirDomain::Scalar => Arm::Scalar,
        HirDomain::Record => Arm::Data,
        HirDomain::Text | HirDomain::TextRecord => Arm::Owned,
        _ => return None,
    })
}

/// How a request may use the A09'/A10' store (PLAN-871 T-05, mirroring the
/// ac-case contract's cache modes).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheMode {
    /// Read hits, recompute misses, publish both (the default).
    Reuse,
    /// Never read old results (forced recompute); publishing stays allowed.
    Fresh,
    /// Read hits, NEVER publish; a miss is recomputed and linked for real
    /// but leaves the store untouched (a later reuse still misses).
    ReadOnly,
}

pub struct MnDriver {
    store: Option<ManagedStore>,
    project: Option<String>,
    /// Fault seam for the hit-path counter control: when set, ANY attempt
    /// to lower a unit aborts — a "hit" that secretly recompiled cannot
    /// pass as a hit.
    pub forbid_codegen: bool,
    /// Read/publish discipline of the store for this request (T-05).
    pub cache_mode: CacheMode,
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

impl MnRequest {
    fn of(
        result: Result<(Option<i32>, Vec<u8>, Vec<u8>, String, Option<String>), MnError>,
        obs: Obs,
        workdir: &Path,
    ) -> MnRequest {
        match result {
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
}

impl MnDriver {
    pub fn ephemeral() -> Self {
        MnDriver {
            store: None,
            project: None,
            forbid_codegen: false,
            cache_mode: CacheMode::Reuse,
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
            cache_mode: CacheMode::Reuse,
            last_publication: std::collections::HashMap::new(),
        })
    }

    /// Attach an ALREADY-OPENED store (the caller owns the open policy —
    /// e.g. `read-only` requires `open_existing` semantics so a missing
    /// root is a typed refusal, never a fresh store).
    pub fn with_managed_store(store: ManagedStore, project: &str) -> Self {
        MnDriver {
            store: Some(store),
            project: Some(project.to_string()),
            forbid_codegen: false,
            cache_mode: CacheMode::Reuse,
            last_publication: std::collections::HashMap::new(),
        }
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
        let result = if src.len() > MAX_INPUT_BYTES {
            Err(MnError::Unsupported(
                "input-size",
                format!("source is {} bytes, above the frozen input cap", src.len()),
            ))
        } else {
            let compiled = obs.span("frontend", |o| {
                compile_escalating(src, caps).map_err(|ds| {
                    o.counter("frontend_reject", 1);
                    MnError::Unsupported(
                        "frontend",
                        ds.first()
                            .map(|d| format!("{:?}/{}: {}", d.stage, d.code, d.message))
                            .unwrap_or_else(|| "frontend failed".into()),
                    )
                })
            });
            compiled.and_then(|compiled| {
                let snapshot = auto_av_core::contract::fnv1a64(src.as_bytes());
                self.run_arm(
                    compiled.arm(),
                    compiled.checked(),
                    entry,
                    workdir,
                    trace_lib,
                    snapshot,
                    &mut obs,
                )
            })
        };
        MnRequest::of(result, obs, workdir)
    }

    /// PLAN-871 backend entry: run the SAME mn pipeline from a
    /// pre-checked module (the caller owns the frontend — the ac-case
    /// worker's direct/backend arms, the ac-native CLI and the shadow gate
    /// all arrive here with their own Checked HIR). `snapshot` is the
    /// caller's content identity (source bytes or canonical atom text).
    pub fn request_checked(
        &mut self,
        checked: &auto_ac_prototype::verify::CheckedModule,
        entry: &str,
        workdir: &Path,
        trace_lib: Option<&Path>,
        snapshot: u64,
    ) -> MnRequest {
        let mut obs = Obs::default();
        let arm = arm_of_domain(checked.bundle().domain);
        let result = match arm {
            Some(arm) => self.run_arm(arm, checked, entry, workdir, trace_lib, snapshot, &mut obs),
            None => Err(MnError::Unsupported(
                "domain",
                format!(
                    "HIR domain `{}` is outside the mn lowering arms",
                    checked.bundle().domain.as_str()
                ),
            )),
        };
        MnRequest::of(result, obs, workdir)
    }

    /// Build + contained run (the source front door and the shadow gate):
    /// [`MnDriver::build_arm`] then the same contained executor the link
    /// driver uses — a run failure is INFRA, the published evidence stays.
    #[allow(clippy::too_many_arguments)]
    fn run_arm(
        &mut self,
        arm: Arm,
        checked: &auto_ac_prototype::verify::CheckedModule,
        entry: &str,
        workdir: &Path,
        trace_lib: Option<&Path>,
        snapshot: u64,
        obs: &mut Obs,
    ) -> Result<(Option<i32>, Vec<u8>, Vec<u8>, String, Option<String>), MnError> {
        let build = self.build_arm(arm, checked, entry, workdir, trace_lib, None, snapshot, obs)?;
        let run = obs.span("run", |o| {
            o.counter("run_invocations", 1);
            link_driver::run_pe(
                &build.exe_path,
                RUN_BUDGET,
                CaptureCaps {
                    stdout_max: STREAM_CAP,
                    stderr_max: STREAM_CAP,
                },
            )
        })?;
        obs.counter("mn_runs", 1);
        Ok((
            run.exit_code,
            run.stdout,
            run.stderr,
            build.manifest_fp,
            build.manifest_artifact_id,
        ))
    }

    /// The backend pipeline shared by the source front door and
    /// [`MnDriver::request_checked`]: lower arm → MIR verify → split →
    /// A09' cache → A10' manifest → wrapper + real link (no run).
    #[allow(clippy::too_many_arguments)]
    fn build_arm(
        &mut self,
        arm: Arm,
        checked: &auto_ac_prototype::verify::CheckedModule,
        entry: &str,
        workdir: &Path,
        trace_lib: Option<&Path>,
        support_override: Option<&Path>,
        snapshot: u64,
        obs: &mut Obs,
    ) -> Result<MnBuild, MnError> {
        let needs_text_runtime = matches!(arm, Arm::Owned);
        // An explicit `--support-lib` means the DOMAIN runtime library on
        // owned (text/text-record) rows and the trace support library on
        // every other arm — the same split the retiring CLI had (PLAN-871
        // T-02 F4 item 4). Text runtime stays auto-located when absent.
        let (trace_lib, text_override) = if needs_text_runtime {
            (trace_lib, support_override)
        } else {
            (support_override.or(trace_lib), None)
        };

        // ---- HIR → MIR (798 public adapter) + verify ---------------------
        let module = obs.span("mir_lower", |o| {
            let r = match arm {
                Arm::Scalar => auto_mir_lower::lower_checked(checked, entry, snapshot),
                Arm::Data => auto_mir_lower::lower_checked_data(checked, entry, snapshot),
                Arm::Owned => auto_mir_lower::lower_checked_owned(checked, entry, snapshot),
            };
            r.map_err(|e| {
                o.counter("mir_reject", 1);
                let msg = match &e {
                    auto_mir_lower::AdapterError::Unsupported { message, .. }
                    | auto_mir_lower::AdapterError::Invalid { message, .. } => message.clone(),
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
        // PLAN-871 T-05 cache modes: `fresh` never READS old units (it may
        // still publish), `read-only` reads but never publishes — a miss is
        // recomputed for real and leaves the store untouched.
        let read_cache = self.cache_mode != CacheMode::Fresh;
        let persist = self.cache_mode != CacheMode::ReadOnly;
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
                    let found = if read_cache {
                        obs.span("cache_lookup", |_| {
                            cache::get_unit_coff_with_id(store, &meta)
                        })?
                    } else {
                        None
                    };
                    match found {
                        Some((id, b)) => {
                            obs.counter("cache_hit_coff", 1);
                            // Restored object bytes: I/O read buffers held
                            // through the link (T-05 §11 partition).
                            obs.ledger.charge(Partition::Io, b.len() as u64);
                            // The manifest binds the concrete artifact id.
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
                            let persisted = if persist {
                                let (id, extra) = obs.span("coff_persist", |o| {
                                    o.ledger.with_charge(
                                        Partition::Io,
                                        obj.bytes.len() as u64 + 512,
                                        |_| {
                                            // Unit COFFs are CACHE entries, not
                                            // publications: no project tag (a
                                            // tagged put would auto-soft-root
                                            // them and pin every revision).
                                            cache::put_unit_coff(store, None, &meta, &obj.bytes)
                                        },
                                    )
                                })?;
                                obs.counter("coff_objects_persisted", 1);
                                obs.counter("store_puts", 1);
                                Some((id, extra))
                            } else {
                                // read-only miss: real recompute, no publish.
                                obs.counter("read_only_miss", 1);
                                None
                            };
                            entry_row = persisted.map(|(id, extra)| LinkUnitEntry {
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
                    obs.ledger
                        .charge(Partition::Staging, obj.bytes.len() as u64);
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
        // The frozen wrapper contract: the entry function is zero-param
        // with an I32-family result (freeze §3). Anything else rejects by
        // name before any object is emitted.
        {
            let sig = entry_unit.sig_of(entry_unit.funcs[0].sig).ok_or_else(|| {
                MnError::Unsupported("entry", format!("entry `{entry}` has no signature"))
            })?;
            if !sig.params.is_empty()
                || !matches!(
                    entry_unit.types.get(sig.result as usize),
                    Some(MirType::I32) | Some(MirType::Bool) | Some(MirType::Char)
                )
            {
                return Err(MnError::Unsupported(
                    "entry",
                    "entry must be zero-param with an i32-family result (wrapper contract)"
                        .to_string(),
                ));
            }
        }
        let (lld_sha, sdk_tag, k32_sha) =
            obs.span("link_identity", |_| link_driver::link_identities())?;
        let trace_lib_sha = match trace_lib {
            Some(p) => Some(link_driver::file_identity(p)?),
            None => None,
        };
        // PLAN-870: the owned domain links the landed text runtime
        // (ac.runtime.text.v1) — located/built through the shared probe
        // seam, memoized per process; data/scalar domains never do.
        let text_lib: Option<std::path::PathBuf> = if needs_text_runtime {
            match text_override {
                Some(p) => Some(p.to_path_buf()),
                None => Some(
                    auto_av_probe::text_runtime::text_runtime_lib().map_err(MnError::Infra)?,
                ),
            }
        } else {
            None
        };
        let text_lib_sha = match &text_lib {
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
            text_lib_sha,
        };
        // Manifest render: index/codec staging for the closure record.
        obs.ledger
            .charge(Partition::Staging, manifest.render().len() as u64);
        let fp = manifest.fingerprint();
        let mut manifest_artifact_id = None;
        // read-only: the closure is recomputed and linked, never published
        // (T-02 F5: read-only consumes hits, publishes nothing).
        if let (Some(store), true) = (&self.store, persist) {
            let id = obs.span("manifest_persist", |o| {
                o.counter("link_manifests_persisted", 1);
                o.counter("store_puts", 1);
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

        // ---- wrapper + real multi-object link -----------------------------
        let wrapper = obs.span("codegen", |o| {
            o.counter("codegen_emit_wrappers", 1);
            emit::emit_wrapper_coff(&entry_meta.symbol)
        })?;
        obs.ledger.charge(Partition::Staging, wrapper.len() as u64);
        let mut support_libs: Vec<std::path::PathBuf> = Vec::new();
        if let Some(t) = trace_lib {
            support_libs.push(t.to_path_buf());
        }
        if let Some(t) = &text_lib {
            support_libs.push(t.clone());
        }
        let linked = obs.span("link", |o| {
            o.counter("link_invocations", 1);
            link_driver::link_only(
                &objects,
                &wrapper,
                START_SYMBOL,
                workdir,
                &support_libs,
                &format!("manifest-fp {fp}\nunits {}\n", manifest.units.len()),
            )
        })?;
        // Ledger peaks enter the observable trace (T-05 evidence).
        let peaks = obs.ledger.peaks();
        let peak_total = obs.ledger.peak_total();
        for (i, name) in crate::resources::PARTITION_NAMES.iter().enumerate() {
            obs.counter(&format!("ledger_peak_{name}"), peaks[i] as u64);
        }
        obs.counter("ledger_peak_total", peak_total as u64);
        Ok(MnBuild {
            exe_path: linked.exe_path,
            receipt_path: linked.receipt_path,
            manifest_fp: fp,
            manifest_artifact_id,
            // Linked units, not manifest entries: an ephemeral (no-store)
            // build has no A10 unit rows yet every unit was still linked.
            units: objects.len(),
            unit_sha256: objects
                .iter()
                .map(|o| (o.name.clone(), sha256_hex(&o.bytes)))
                .collect(),
            obs: obs.clone(),
        })
    }

    /// Build-only entry: lower through the shared MIR chain, emit per-unit
    /// COFFs, link for real, publish exe + link receipt — and STOP. The
    /// caller keeps execution (its budgets, caps and containment), which is
    /// why the worker's run leg and `ac-native build` can both use this.
    pub fn build_checked(
        &mut self,
        checked: &auto_ac_prototype::verify::CheckedModule,
        entry: &str,
        workdir: &Path,
        trace_lib: Option<&Path>,
        support_override: Option<&Path>,
        snapshot: u64,
    ) -> Result<MnBuild, MnError> {
        let mut obs = Obs::default();
        let arm = arm_of_domain(checked.bundle().domain).ok_or_else(|| {
            MnError::Unsupported(
                "domain",
                format!(
                    "HIR domain `{}` is outside the mn lowering arms",
                    checked.bundle().domain.as_str()
                ),
            )
        })?;
        self.build_arm(
            arm,
            checked,
            entry,
            workdir,
            trace_lib,
            support_override,
            snapshot,
            &mut obs,
        )
    }
}

/// The committed products of one mn build: the exe, its link receipt and
/// the A10' manifest identity (fingerprint + optional store artifact id).
pub struct MnBuild {
    pub exe_path: std::path::PathBuf,
    pub receipt_path: std::path::PathBuf,
    pub manifest_fp: String,
    pub manifest_artifact_id: Option<String>,
    pub units: usize,
    /// (unit name, SHA-256 of its COFF bytes) — the receipt's per-unit
    /// identity needs (PLAN-871 T-02 F4 item 2). SHA-256 here, not the
    /// internal blake3 key: the receipt is an external document.
    pub unit_sha256: Vec<(String, String)>,
    /// The build's own observation (spans/counters/ledger peaks) so a
    /// driver-layer consumer (ac-case worker, CLI receipts) reports mn's
    /// real counters instead of inventing them.
    pub obs: Obs,
}

/// Content identity seed for a lowering request (FNV-1a 64), shared with
/// the source front door so callers that only hold canonical HIR bytes
/// derive the SAME kind of snapshot.
pub fn content_snapshot(bytes: &[u8]) -> u64 {
    auto_av_core::contract::fnv1a64(bytes)
}

fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut h = Sha256::new();
    h.update(bytes);
    format!("{:x}", h.finalize())
}

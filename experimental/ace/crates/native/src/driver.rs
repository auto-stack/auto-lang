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
use crate::source_diagnostics as sd;
use crate::{emit, MnError};
use ace_artifacts::store::ManagedStore;
use ace_av_core::mir::MirType;
use ace_av_core::mir_verify::{verify_mir, MirVerifyError};
use ace_core::link::CaptureCaps;
use std::path::{Path, PathBuf};

#[path = "closures_driver.rs"]
pub(crate) mod closures;
#[path = "collections_driver.rs"]
pub(crate) mod collections;

/// The narrowest source domain that admits a request (PLAN-870 freeze §4):
/// scalar → record → text-record escalation picks the narrowest compiling
/// profile, so B* sources keep their `av.core-i32/1` unit identity while
/// T*/R* sources reach `av.owned/1`/`av.data/1` through the SAME shared
/// frontend stack AV uses (diagnostics included).
enum Domain {
    Scalar(ace_core::source::CompiledSource),
    /// PLAN-999 (T-03): the core-i32 fact-lane compile (`while`/`loop`
    /// sources); its Core-domain bundle lowers through the scalar arm.
    Core(ace_core::source::CompiledSource),
    Data(ace_core::source::CompiledSource),
    Owned(ace_core::source::CompiledSource),
    /// PLAN-961: the numeric source surface (closures-family lowering).
    Numeric(ace_core::source::CompiledSource),
    /// PLAN-962: the aggregates surface (numeric superset + composites;
    // rides the same closures-family arm — the plan added ZERO new MIR
    // instructions).
    Aggregates(ace_core::source::CompiledSource),
}

/// A profile-surface rejection (`source.unsupported` out of the Text
/// stage) — the only failure class that escalates to the next wider
/// domain; every other diagnostic (parse, bind, verify) is terminal and
/// reports from the domain that produced it, exactly like AV's explicit
/// per-domain drivers.
fn is_surface_rejection(ds: &[ace_core::atom_text::Diagnostic]) -> bool {
    ds.iter().all(|d| {
        d.code == "source.unsupported" && d.stage == ace_core::atom_text::Stage::Text || {
            // PLAN-961: the numeric-only operator characters (& | ^)
            // lex as unknown characters on every narrower profile —
            // that lex failure is a surface fact (the numeric profile
            // owns the characters), so escalation continues.
            d.code == "source.lex"
                && d.message.contains("unknown character")
                && (d.message.contains("`&`")
                    || d.message.contains("`|`")
                    || d.message.contains("`^`"))
                || {
                    // PLAN-962: the aggregates-owned token spellings (`[`,
                    // `..`, `..=`) surface as narrow-face syntax errors —
                    // climb to the widest rung exactly like the numeric-only
                    // operator characters (961 precedent).
                    d.code == "source.syntax"
                        && (d.message.contains("found `[`")
                        || d.message.contains("found `..`")
                        // PLAN-962: the tuple literal (comma inside the
                        // paren group) and the named ctor (ident followed
                        // by `{ field: v }`) are aggregates-owned shapes.
                        || d.message.contains("expected operator, found `,`")
                        || d.message
                            .contains("expected `;`, newline or `}`, found `{`"))
                }
        }
    })
}

/// The escalating source front door, PLAN-911 shape: a failure names the
/// profile that TERMINATED the chain, the profiles actually attempted and
/// the surface rejections consumed only to escalate — the terminal
/// diagnostics are never a surface rejection re-read from an earlier
/// domain, and nothing here re-runs a compile to learn "where it failed".
fn compile_escalating(
    src: &str,
    caps: &[String],
) -> std::result::Result<Domain, sd::CompileFailure> {
    let mut attempted = Vec::with_capacity(4);
    let mut surface = Vec::new();
    attempted.push(sd::SourceProfile::Scalar);
    match ace_av::query_entry::compile_scalar_source_via_facts(src, "mn.at", caps) {
        Ok(c) => return Ok(Domain::Scalar(c)),
        Err(ds) if is_surface_rejection(&ds) => surface.push(sd::SurfaceRejection {
            profile: sd::SourceProfile::Scalar,
            codes: ds.iter().map(|d| d.code.clone()).collect(),
        }),
        Err(ds) => {
            return Err(sd::CompileFailure {
                profile: sd::SourceProfile::Scalar,
                attempted,
                surface_rejections: surface,
                diagnostics: ds,
            });
        }
    }
    // PLAN-999 (T-03, T-09 repair): the CORE fact lane answers the
    // constructs the scalar face surface-rejects (`while`/`loop`); the SAME
    // shared fact producer and the Core-domain verifier. Core is a SIDE
    // face, not a monotone rung: it owns `while`/`loop` but is NARROWER
    // than scalar in operators — constructs that are syntax errors to Core
    // (numeric literals/operators) can be legal surface in the wider faces.
    // A Core failure therefore NEVER terminates this ladder: only success
    // returns here; every failure keeps climbing to the unchanged
    // record/text-record/numeric rungs (their terminal rules are intact).
    attempted.push(sd::SourceProfile::Core);
    if let Ok(c) = ace_av::query_entry::compile_core_source_via_facts(src, "mn.at", caps) {
        return Ok(Domain::Core(c));
    }
    attempted.push(sd::SourceProfile::Record);
    match ace_core::source::compile_record_source(src, "mn.at", caps) {
        Ok(c) => return Ok(Domain::Data(c)),
        Err(ds) if is_surface_rejection(&ds) => surface.push(sd::SurfaceRejection {
            profile: sd::SourceProfile::Record,
            codes: ds.iter().map(|d| d.code.clone()).collect(),
        }),
        Err(ds) => {
            return Err(sd::CompileFailure {
                profile: sd::SourceProfile::Record,
                attempted,
                surface_rejections: surface,
                diagnostics: ds,
            });
        }
    }
    attempted.push(sd::SourceProfile::TextRecord);
    let owned = match ace_core::source::compile_text_record_source(src, "mn.at", caps) {
        Ok(c) => return Ok(Domain::Owned(c)),
        Err(ds) if is_surface_rejection(&ds) => surface.push(sd::SurfaceRejection {
            profile: sd::SourceProfile::TextRecord,
            codes: ds.iter().map(|d| d.code.clone()).collect(),
        }),
        Err(ds) => {
            return Err(sd::CompileFailure {
                profile: sd::SourceProfile::TextRecord,
                attempted,
                surface_rejections: surface,
                diagnostics: ds,
            });
        }
    };
    let _ = owned;
    // PLAN-961: the numeric surface attempts LAST (a strict superset of the
    // closures vocabulary; earlier profiles keep their frozen precedence).
    attempted.push(sd::SourceProfile::Numeric);
    match ace_core::source::compile_numeric_source(src, "mn.at", caps) {
        Ok(c) => return Ok(Domain::Numeric(c)),
        Err(ds) if is_surface_rejection(&ds) => surface.push(sd::SurfaceRejection {
            profile: sd::SourceProfile::Numeric,
            codes: ds.iter().map(|d| d.code.clone()).collect(),
        }),
        Err(ds) => {
            return Err(sd::CompileFailure {
                profile: sd::SourceProfile::Numeric,
                attempted,
                surface_rejections: surface,
                diagnostics: ds,
            });
        }
    }
    // PLAN-962: the aggregates surface attempts truly LAST (numeric
    // superset + composites; the escalation ladder keeps every frozen
    // profile's precedence).
    attempted.push(sd::SourceProfile::Aggregates);
    ace_core::source::compile_aggregates_source(src, "mn.at", caps)
        .map(Domain::Aggregates)
        .map_err(|ds| sd::CompileFailure {
            profile: sd::SourceProfile::Aggregates,
            attempted,
            surface_rejections: surface,
            diagnostics: ds,
        })
}

impl Domain {
    fn arm(&self) -> Arm {
        match self {
            Domain::Scalar(_) => Arm::Scalar,
            // PLAN-999 (T-03): Core-domain bundles lower through the scalar
            // arm (`arm_of_domain` already maps `HirDomain::Core` there).
            Domain::Core(_) => Arm::Scalar,
            Domain::Data(_) => Arm::Data,
            Domain::Owned(_) => Arm::Owned,
            // PLAN-961: numeric compiles ride the closures-family arm
            // regardless of the binder-stamped domain.
            Domain::Numeric(_) => Arm::Numeric,
            // PLAN-962: aggregates likewise (zero new MIR instructions).
            Domain::Aggregates(_) => Arm::Numeric,
        }
    }
    fn checked(&self) -> &ace_core::verify::CheckedModule {
        match self {
            Domain::Scalar(c)
            | Domain::Core(c)
            | Domain::Data(c)
            | Domain::Owned(c)
            | Domain::Numeric(c)
            | Domain::Aggregates(c) => &c.checked,
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
    /// PLAN-961: the numeric surface (collections/closures family lowering).
    Numeric,
}

fn arm_of_domain(d: ace_core::hir::HirDomain) -> Option<Arm> {
    use ace_core::hir::HirDomain;
    Some(match d {
        HirDomain::Core | HirDomain::Scalar => Arm::Scalar,
        HirDomain::Record => Arm::Data,
        HirDomain::Text | HirDomain::TextRecord => Arm::Owned,
        // PLAN-961: the numeric source surface rides the closures-family
        // shared lowering arm.
        HirDomain::Collections | HirDomain::Closures => Arm::Numeric,
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
    /// PLAN-919: the R06 exit knobs + observation switch for the NEXT
    /// collections wrapper emission (resource-events requests only; the
    /// default is the historical all-off wrapper).
    pub resource_knobs: crate::collections_lower::WrapperKnobs,
    /// Fault seam for the hit-path counter control: when set, ANY attempt
    /// to lower a unit aborts — a "hit" that secretly recompiled cannot
    /// pass as a hit.
    pub forbid_codegen: bool,
    /// Negative control: perform a real link without its required collections
    /// runtime. Never set by the source language or ordinary request protocol.
    pub test_omit_collections_runtime: bool,
    last_collections_run: Option<link_driver::PeRun>,
    /// PLAN-924: the retained failed-run evidence for the closures family
    /// (same contract as `last_collections_run`).
    pub(crate) last_closures_run: Option<link_driver::PeRun>,
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
            test_omit_collections_runtime: false,
            resource_knobs: Default::default(),
            last_collections_run: None,
            last_closures_run: None,
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
            test_omit_collections_runtime: false,
            resource_knobs: Default::default(),
            last_collections_run: None,
            last_closures_run: None,
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
            test_omit_collections_runtime: false,
            resource_knobs: Default::default(),
            last_collections_run: None,
            last_closures_run: None,
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
            // PLAN-1017: the A17 source memo first — a fully-hot request
            // (identical source/entry/caps under the current pipeline axes)
            // rebuilds from the recorded manifest instead of re-running the
            // whole-source frontend. `None` is a clean miss: the full
            // pipeline below is the only authority on a miss.
            match self.try_source_memo(src, entry, caps, workdir, trace_lib, &mut obs) {
                Ok(Some(hit)) => Ok(hit),
                Ok(None) => {
                    let compiled = obs.span("frontend", |o| {
                        compile_escalating(src, caps).map_err(|failure| {
                            o.counter("frontend_reject", 1);
                            // PLAN-911: the operation's own typed snapshot
                            // rides the error; `code()`/`message()` stay the
                            // legacy projection.
                            sd::mn_error_from_compile_failure(
                                src,
                                "mn.at",
                                failure,
                                sd::Operation::cold(src),
                            )
                        })
                    });
                    let ran = compiled.and_then(|compiled| {
                        let snapshot = ace_av_core::contract::fnv1a64(src.as_bytes());
                        self.run_arm(
                            compiled.arm(),
                            compiled.checked(),
                            entry,
                            workdir,
                            trace_lib,
                            snapshot,
                            &mut obs,
                        )
                    });
                    // PLAN-1017: remember source→manifest for future hot
                    // rounds (content-addressed; a repeated request is
                    // byte-identical by construction). The memo is a pure
                    // accelerator: a persist failure is OBSERVED (counter),
                    // never fails the already-successful request.
                    if ran.is_ok() {
                        if let (Some(store), true, Some(man_id)) = (
                            self.store.as_ref(),
                            self.cache_mode != CacheMode::ReadOnly,
                            ran.as_ref().unwrap().4.clone(),
                        ) {
                            match obs.span("source_memo_persist", |_| {
                                cache::put_source_memo(
                                    store,
                                    self.project.as_deref(),
                                    &sha256_hex(src.as_bytes()),
                                    entry,
                                    caps,
                                    &man_id,
                                )
                            }) {
                                Ok(_) => {
                                    obs.counter("store_puts", 1);
                                }
                                Err(_) => {
                                    obs.counter("source_memo_persist_failures", 1);
                                }
                            }
                        }
                    }
                    ran
                }
                Err(e) => Err(e),
            }
        };
        MnRequest::of(result, obs, workdir)
    }

    /// PLAN-1017: serve one request from the A17 source memo. On a hit the
    /// whole-source frontend (the measured dominant cost of fully-hot
    /// turnaround legs — ~1.9s of a ~3.6s hit request) is skipped; the
    /// recorded link manifest is restored and every unit re-admits from
    /// its bytes through the worker-restore discipline (symbol recompute,
    /// sha/len binding, envelope admission), then the wrapper is re-emitted
    /// and the closure really links and really runs. Any miss signal — no
    /// memo, foreign carrier, toolchain drift (manifest link axes vs the
    /// current identities), missing/foreign unit — returns `Ok(None)` and
    /// the caller runs the full pipeline, which is always correct.
    #[allow(clippy::too_many_arguments)]
    fn try_source_memo(
        &mut self,
        src: &str,
        entry: &str,
        caps: &[String],
        workdir: &Path,
        trace_lib: Option<&Path>,
        obs: &mut Obs,
    ) -> Result<Option<(Option<i32>, Vec<u8>, Vec<u8>, String, Option<String>)>, MnError> {
        let Some(store) = self.store.as_ref() else {
            return Ok(None);
        };
        if self.cache_mode == CacheMode::Fresh {
            // `fresh` never READS old state — the memo included.
            return Ok(None);
        }
        let src_sha = sha256_hex(src.as_bytes());
        let Some(manifest_id) = cache::get_source_memo(store, &src_sha, entry, caps)? else {
            return Ok(None);
        };
        let Some(art) = store
            .get_artifact_by_id(&manifest_id)
            .map_err(|e| MnError::Infra(format!("memo manifest get: {e}")))?
        else {
            return Ok(None);
        };
        let manifest = match cache::parse_link_manifest(&art.payload) {
            Ok(m) => m,
            Err(_) => return Ok(None),
        };
        if manifest.abi != MN_ABI || manifest.target != MN_TARGET {
            return Ok(None);
        }
        // Toolchain drift: the manifest's link axes must equal the CURRENT
        // identities — a memo from another toolchain is a miss, not a serve.
        let (lld_sha, sdk_tag, k32_sha) =
            obs.span("link_identity", |_| link_driver::link_identities())?;
        if manifest.lld_sha != lld_sha || manifest.sdk_tag != sdk_tag || manifest.kernel32_sha != k32_sha
        {
            obs.counter("source_memo_drift", 1);
            return Ok(None);
        }
        let trace_lib_sha = match trace_lib {
            Some(p) => Some(link_driver::file_identity(p)?),
            None => None,
        };
        if manifest.trace_lib_sha != trace_lib_sha {
            return Ok(None);
        }
        let needs_text_runtime = manifest.text_lib_sha.is_some();
        let text_lib: Option<std::path::PathBuf> = if needs_text_runtime {
            Some(ace_av::text_runtime::text_runtime_lib().map_err(MnError::Infra)?)
        } else {
            None
        };
        let text_lib_sha = match &text_lib {
            Some(p) => Some(link_driver::file_identity(p)?),
            None => None,
        };
        if manifest.text_lib_sha != text_lib_sha {
            return Ok(None);
        }

        // Re-admit every unit from its bytes (the worker-restore
        // discipline). Any admission failure is a miss back to the full
        // pipeline — a memo hit never bypasses admission.
        let mut objects: Vec<ObjectInput> = Vec::with_capacity(manifest.units.len());
        for u in &manifest.units {
            let want = crate::symbols::function_symbol(&u.meta.owner, &u.meta.unit);
            if want != u.meta.symbol {
                return Ok(None);
            }
            let bytes = match cache::get_unit_coff_by_id(store, &u.meta, &u.artifact_id) {
                Ok(b) => b,
                Err(_) => return Ok(None),
            };
            if cache::content_hash(&bytes) != u.sha256 || bytes.len() != u.len {
                return Ok(None);
            }
            obs.counter("cache_hit_coff", 1);
            obs.ledger.charge(Partition::Io, bytes.len() as u64);
            objects.push(ObjectInput {
                name: u.meta.unit.clone(),
                bytes,
            });
        }

        // Re-publish the manifest idempotently + the ordered-publication
        // bookkeeping, mirroring build_arm exactly (same key → same
        // artifact id; a repeat request keeps the same publication state).
        let persist = self.cache_mode != CacheMode::ReadOnly;
        let mut manifest_artifact_id = None;
        if persist {
            obs.ledger
                .charge(Partition::Staging, manifest.render().len() as u64);
            let id = obs.span("manifest_persist", |o| {
                o.counter("link_manifests_persisted", 1);
                o.counter("store_puts", 1);
                cache::put_link_manifest(store, self.project.as_deref(), &manifest)
            })?;
            if let Some(project) = self.project.as_deref() {
                let key = (
                    project.to_string(),
                    manifest.owner.module.clone(),
                    manifest.entry.clone(),
                );
                if let Some(prev) = self.last_publication.insert(key.clone(), id.clone()) {
                    let _ = store.remove_root(project, "soft", &prev);
                }
            }
            manifest_artifact_id = Some(id);
        }
        let fp = manifest.fingerprint();

        // Wrapper + real link (every memo hit really links and really runs —
        // a cached PASS is never a substitute for execution, §9.4).
        let wrapper = obs.span("codegen", |o| {
            o.counter("codegen_emit_wrappers", 1);
            emit::emit_wrapper_coff(&manifest.entry_symbol)
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
        // Ledger peaks enter the observable trace (mirrors build_arm).
        let peaks = obs.ledger.peaks();
        let peak_total = obs.ledger.peak_total();
        for (i, name) in crate::resources::PARTITION_NAMES.iter().enumerate() {
            obs.counter(&format!("ledger_peak_{name}"), peaks[i] as u64);
        }
        obs.counter("ledger_peak_total", peak_total as u64);
        let run = obs.span("run", |o| {
            o.counter("run_invocations", 1);
            link_driver::run_pe(
                &linked.exe_path,
                RUN_BUDGET,
                CaptureCaps {
                    stdout_max: STREAM_CAP,
                    stderr_max: STREAM_CAP,
                },
            )
        })?;
        obs.counter("mn_runs", 1);
        obs.counter("source_memo_hits", 1);
        Ok(Some((
            run.exit_code,
            run.stdout,
            run.stderr,
            fp,
            manifest_artifact_id,
        )))
    }

    /// PLAN-911: the frontend gate of a whole-request candidate — the SAME
    /// escalating source compile `request()` runs, and NOTHING else (zero
    /// codegen/link/run, zero guest production). Used by the worker's
    /// strict-mode per-candidate fallback gate so a materialized edit's
    /// real semantic failure stops the sequence at ITS candidate instead
    /// of being discovered after the whole tail was applied. This is not
    /// a second compile to "recover" a cause: each candidate is compiled
    /// exactly once, here or by the request that runs it.
    pub fn check_source(&mut self, src: &str, caps: &[String]) -> Result<(), MnError> {
        if src.len() > MAX_INPUT_BYTES {
            return Err(MnError::Unsupported(
                "input-size",
                format!("source is {} bytes, above the frozen input cap", src.len()),
            ));
        }
        compile_escalating(src, caps)
            .map(|_| ())
            .map_err(|failure| {
                sd::mn_error_from_compile_failure(src, "mn.at", failure, sd::Operation::cold(src))
            })
    }

    /// PLAN-1000 F-1000-01 (owner option a): a response-side NAMED trap
    /// observation for a production run that ended in the frozen
    /// unlabelled exit 70 with empty streams. The SAME source re-runs the
    /// identical escalating frontend / MIR verify / unit-split pipeline,
    /// then lowers in `LowerMode::TrapProbe` (the production `Numeric`
    /// value surface with reason-carrying trap branches into the typed
    /// runtime's `mn_typed_trap`) and links + runs ONCE in `workdir`
    /// against the typed-entry runtime. The probe build is a distinct
    /// observation object: it never enters the A09'/A10' caches or any
    /// manifest, and the production objects keep their frozen bytes. Only
    /// the pure-value arms (Scalar/Core via `Arm::Scalar`, Numeric) are
    /// probeable; data/owned arms honestly return no observation.
    pub fn trap_probe(
        &mut self,
        src: &str,
        entry: &str,
        caps: &[String],
        workdir: &Path,
        trace_lib: Option<&Path>,
    ) -> Result<Option<u32>, MnError> {
        let compiled = compile_escalating(src, caps).map_err(|failure| {
            sd::mn_error_from_compile_failure(src, "mn.at", failure, sd::Operation::cold(src))
        })?;
        let arm = compiled.arm();
        if !matches!(arm, Arm::Scalar | Arm::Numeric) {
            return Ok(None);
        }
        let checked = compiled.checked();
        let snapshot = ace_av_core::contract::fnv1a64(src.as_bytes());
        let module = match arm {
            Arm::Scalar => ace_lowering::lower_checked(checked, entry, snapshot),
            Arm::Numeric => ace_lowering::lower_checked_closures(checked, entry, snapshot),
            _ => unreachable!("checked above"),
        }
        .map_err(|e| match e {
            ace_lowering::AdapterError::Unsupported { message, .. }
            | ace_lowering::AdapterError::Invalid { message, .. } => {
                MnError::Unsupported("trap-probe", message)
            }
            ace_lowering::AdapterError::Resource { message, .. } => {
                MnError::Resource("trap-probe", message)
            }
        })?;
        let checked_mir = verify_mir(&module)
            .map_err(|e| MnError::Unsupported("trap-probe", format!("mir: {e:?}")))?;
        let units = ace_av::cache::split_units(checked_mir.module())
            .map_err(|e| MnError::Backend(format!("trap-probe split: {e}")))?;
        let mut objects = Vec::with_capacity(units.len());
        for unit in &units {
            let object = emit::emit_unit_coff_trap_probe(unit)?;
            objects.push(ObjectInput {
                name: object.symbol,
                bytes: object.bytes,
            });
        }
        let entry_unit = units
            .iter()
            .find(|u| u.funcs[0].name == entry)
            .ok_or_else(|| {
                MnError::Unsupported("trap-probe", format!("entry unit `{entry}` absent"))
            })?;
        let entry_meta = cache::CoffMeta::for_unit(entry_unit, cache::unit_fp(entry_unit))?;
        let entry_symbol = entry_meta.symbol;
        let wrapper = emit::emit_wrapper_coff(&entry_symbol)?;
        let runtime = crate::typed_runtime::prepare(None)?;
        let mut support = vec![runtime.library];
        if let Some(t) = trace_lib {
            support.push(t.to_path_buf());
        }
        let linked = link_driver::link_only(
            &objects,
            &wrapper,
            START_SYMBOL,
            workdir,
            &support,
            "plan1000-trap-probe/1",
        )?;
        let run = link_driver::run_pe(
            &linked.exe_path,
            RUN_BUDGET,
            CaptureCaps {
                stdout_max: STREAM_CAP,
                stderr_max: STREAM_CAP,
            },
        )?;
        // The probe PE never read a request frame, so the response frame
        // echoes zero sig/nonce; only STATUS+reason matter here. Any other
        // outcome (value frame, resource/error frame, bad exit) is the
        // honest absence of a named trap.
        match crate::typed_entry::admit_frame_and_exit(&run.stdout, run.exit_code) {
            Ok((_, _, crate::typed_entry::TypedOutcome::Trap { reason }))
                if crate::typed_entry::trap_label(reason).is_some() =>
            {
                Ok(Some(reason))
            }
            _ => Ok(None),
        }
    }

    /// PLAN-871 backend entry: run the SAME mn pipeline from a
    /// pre-checked module (the caller owns the frontend — the ac-case
    /// worker's direct/backend arms, the ac-native CLI and the shadow gate
    /// all arrive here with their own Checked HIR). `snapshot` is the
    /// caller's content identity (source bytes or canonical atom text).
    pub fn request_checked(
        &mut self,
        checked: &ace_core::verify::CheckedModule,
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
        checked: &ace_core::verify::CheckedModule,
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
        checked: &ace_core::verify::CheckedModule,
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
                Arm::Scalar => ace_lowering::lower_checked(checked, entry, snapshot),
                Arm::Data => ace_lowering::lower_checked_data(checked, entry, snapshot),
                Arm::Owned => ace_lowering::lower_checked_owned(checked, entry, snapshot),
                Arm::Numeric => ace_lowering::lower_checked_closures(checked, entry, snapshot),
            };
            r.map_err(|e| {
                o.counter("mir_reject", 1);
                match e {
                    ace_lowering::AdapterError::Unsupported { message, .. }
                    | ace_lowering::AdapterError::Invalid { message, .. } => {
                        MnError::Unsupported("adapter", message)
                    }
                    ace_lowering::AdapterError::Resource { message, .. } => {
                        MnError::Resource("adapter", message)
                    }
                }
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
            ace_av::cache::split_units(checked_mir.module())
                .map_err(|e| MnError::Backend(format!("unit split: {e}")))
        })?;
        obs.counter("mir_units_split", units.len() as u64);
        // The units stay resident for the whole request: their structural
        // heap estimate is the MIR/symbol-hot partition charge (T-05 §11).
        for u in &units {
            obs.ledger
                .charge(Partition::MirSymbol, u.heap_estimate() as u64);
        }

        let persist = self.cache_mode != CacheMode::ReadOnly;
        let (objects, mut entries) = self.collect_objects(
            &units,
            &CoffMeta::for_unit,
            &emit::emit_unit_coff,
            obs,
            None,
        )?;

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
                None => Some(ace_av::text_runtime::text_runtime_lib().map_err(MnError::Infra)?),
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

    /// Shared A09 lookup/admit/emit discipline for every native source family.
    pub(crate) fn collect_objects(
        &mut self,
        units: &[ace_av_core::mir::MirModule],
        meta_for: &dyn Fn(&ace_av_core::mir::MirModule, String) -> Result<CoffMeta, MnError>,
        emit_for: &dyn Fn(&ace_av_core::mir::MirModule) -> Result<emit::UnitObject, MnError>,
        obs: &mut Obs,
        known_fp: Option<&str>,
    ) -> Result<(Vec<ObjectInput>, Vec<LinkUnitEntry>), MnError> {
        if self.store.is_some()
            && crate::collections_lower::test_frame_free_omitted()
            && units
                .iter()
                .any(|unit| unit.profile == ace_av_core::contract::AV_COLLECTIONS_PROFILE)
        {
            return Err(MnError::Admit(
                "test-control-persistence",
                "intentionally incorrect collections objects require an ephemeral driver".into(),
            ));
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
        for unit in units {
            // The fingerprint render is transient codec staging.
            let fp = if let Some(fp) = known_fp {
                if units.len() != 1 {
                    return Err(MnError::Admit(
                        "live-unit-identity",
                        "a checked fingerprint belongs to one immutable unit".into(),
                    ));
                }
                fp.to_string()
            } else if unit.profile == ace_av_core::contract::AV_COLLECTIONS_PROFILE {
                obs.span("unit_fp_render", |o| {
                    o.ledger
                        .with_charge(Partition::Staging, 4096, |_| cache::unit_fp(unit))
                })
            } else if unit.profile == ace_av_core::contract::AV_CLOSURES_PROFILE {
                // PLAN-924: the closures family's unit identity is the
                // spans-cleared ABC wire bytes (the same identity the AV
                // closures sessions compare) — the scalar render never
                // widens to closure instructions.
                obs.span("unit_fp_render", |o| {
                    o.ledger.with_charge(Partition::Staging, 4096, |_| {
                        crate::closures_layout::unit_fp(unit)
                            .expect("verified closures unit encodes")
                    })
                })
            } else {
                obs.ledger
                    .with_charge(Partition::Staging, 4096, |_| cache::unit_fp(unit))
            };
            let meta = obs.span("unit_fp", |_| meta_for(unit, fp))?;
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
                            let extra_page_ids = if unit.profile
                                == ace_av_core::contract::AV_COLLECTIONS_PROFILE
                                || unit.profile == ace_av_core::contract::AV_CLOSURES_PROFILE
                            {
                                cache::unit_coff_page_ids(store, &meta, b.len())?
                            } else {
                                vec![]
                            };
                            entry_row = Some(LinkUnitEntry {
                                sha256: cache::content_hash(&b),
                                len: b.len(),
                                artifact_id: id,
                                extra_page_ids,
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
                                emit_for(unit)
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
                        emit_for(unit)
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

        Ok((objects, entries))
    }

    /// Build-only entry: lower through the shared MIR chain, emit per-unit
    /// COFFs, link for real, publish exe + link receipt — and STOP. The
    /// caller keeps execution (its budgets, caps and containment), which is
    /// why the worker's run leg and `ac-native build` can both use this.
    pub fn build_checked(
        &mut self,
        checked: &ace_core::verify::CheckedModule,
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
    ace_av_core::contract::fnv1a64(bytes)
}

fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut h = Sha256::new();
    h.update(bytes);
    format!("{:x}", h.finalize())
}

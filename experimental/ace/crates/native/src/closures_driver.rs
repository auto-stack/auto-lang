//! PLAN-924: the closures native driver arm — cold source → shared MIR
//! units → native admit/CLIF/COFF (the closures family identities) → the
//! SAME no_std runtime library (runtime/4 with the closure-environment
//! surface) → PE link/run/observe. Mirrors the collections driver's
//! discipline (candidate transaction, publication, failed-run evidence)
//! with the closure family's own identities and cross-unit interface
//! checks; restore-to-session lands with T-08.

use super::*;
use crate::{closures_layout as layout, closures_lower as lower};
use crate::{collections_artifacts as art, collections_runtime as runtime};
use crate::cache::CoffMeta;
use super::collections::Entry;
use ace_av_core::mir::{MirModule, MirType};
use std::path::Path;

fn bad(message: impl Into<String>) -> crate::MnError {
    crate::MnError::Admit("closures", message.into())
}

struct ClosureInterface {
    owner: ace_av_core::mir::OwnerScope,
    name: String,
    signature: layout::Descriptor,
    env: Option<Vec<layout::Descriptor>>,
    imports: Vec<(String, layout::Descriptor, Option<Vec<layout::Descriptor>>)>,
    entry_eligible: bool,
}
fn closures_interface_parts(
    unit: &MirModule,
    shapes: &[layout::Descriptor],
) -> Result<ClosureInterface, crate::MnError> {
    let function = &unit.funcs[0];
    let mut imports = Vec::with_capacity(unit.imports.len());
    for import in &unit.imports {
        let name = import
            .symbol
            .strip_prefix("fn:")
            .ok_or_else(|| bad("non-function closure import"))?;
        let env = import
            .env
            .as_ref()
            .map(|layout| layout.iter().map(|ty| shapes[*ty as usize].clone()).collect());
        imports.push((name.into(), shapes[import.sig as usize].clone(), env));
    }
    let entry_eligible = matches!(&unit.types[function.sig as usize], MirType::Fn(sig)
        if sig.params.is_empty() && matches!(unit.types[sig.result as usize], MirType::I32) && function.env.is_none());
    Ok(ClosureInterface {
        owner: unit.owner.clone(),
        name: function.name.clone(),
        signature: shapes[function.sig as usize].clone(),
        env: function
            .env
            .as_ref()
            .map(|layout| layout.iter().map(|ty| shapes[*ty as usize].clone()).collect()),
        imports,
        entry_eligible,
    })
}
/// Cross-unit closure link admission: one owner, unique names, every
/// import's signature AND env declaration byte-equal to the target body's,
/// and exactly one eligible entry. CreateClosure sites were already checked
/// against the callee's declared env by shared verify; this pins the units
/// actually being linked.
fn check_closures_interfaces(
    units: &[MirModule],
    entry: &str,
) -> Result<Entry, crate::MnError> {
    let mut functions = std::collections::HashMap::new();
    let mut owner = None;
    for unit in units {
        let shapes = lower::admit_layouts(unit)?;
        let interface = closures_interface_parts(unit, &shapes)?;
        if owner.as_ref().is_some_and(|o| o != &interface.owner)
            || functions
                .insert(interface.name.clone(), interface)
                .is_some()
        {
            return Err(bad("mixed owner or duplicate function"));
        }
        owner = Some(unit.owner.clone());
    }
    for interface in functions.values() {
        for (name, signature, env) in &interface.imports {
            let target = functions
                .get(name)
                .ok_or_else(|| bad(format!("missing callee {name}")))?;
            if &target.signature != signature {
                return Err(bad(format!("callee signature mismatch: {name}")));
            }
            if &target.env != env {
                return Err(bad(format!(
                    "callee environment declaration mismatch: {name}"
                )));
            }
        }
    }
    let by_name = format!("d_{entry}");
    let hits: Vec<_> = functions
        .values()
        .filter(|i| i.name == entry || i.name == by_name)
        .collect::<Vec<_>>();
    if hits.len() != 1 {
        return Err(bad("entry is missing or ambiguous"));
    }
    let entry = hits[0];
    if !entry.entry_eligible {
        return Err(bad(
            "the closures entry must be an ordinary zero-argument int function",
        ));
    }
    Ok(Entry {
        owner: entry.owner.clone(),
        unit: entry.name.clone(),
        symbol: crate::symbols::function_symbol(&entry.owner, &entry.name),
    })
}

impl MnDriver {
    pub fn build_closures(
        &mut self,
        source: &str,
        entry: &str,
        workdir: &Path,
        support: Option<&Path>,
    ) -> Result<MnBuild, crate::MnError> {
        self.last_closures_run = None;
        let mut obs = Obs::default();
        let cold = obs.span("frontend", |o| {
            let cold = crate::closures_frontend::cold(source, "mn.at").map_err(|e| match e {
                crate::closures_frontend::ColdError::Source(ds) => {
                    crate::MnError::Unsupported("frontend", format!("{ds:?}"))
                }
                // Native admission failures keep their NAMED face.
                crate::closures_frontend::ColdError::Admit(e) => e,
            })?;
            o.counter("frontend_real_body_plans", cold.real_body_plans as u64);
            Ok::<_, crate::MnError>(cold)
        })?;
        let units = cold.units;
        self.closures_units_inner(&units, source, entry, workdir, support, &mut obs, false)
            .map(|r| r.0)
    }

    /// PLAN-924: the closures resource request — the SAME closures pipeline
    /// with the 919/923 observation and fault knobs installed by the entry
    /// wrapper (seam 9 = the closure-environment allocation; operations and
    /// budgets only — the parent judges the stderr facts).
    pub fn request_closures_resource(
        &mut self,
        source: &str,
        entry: &str,
        workdir: &Path,
        support: Option<&Path>,
        knobs: crate::collections_lower::WrapperKnobs,
    ) -> MnRequest {
        self.resource_knobs = knobs;
        let r = self.request_closures(source, entry, workdir, support);
        self.resource_knobs = Default::default();
        r
    }

    pub fn request_closures(
        &mut self,
        source: &str,
        entry: &str,
        workdir: &Path,
        support: Option<&Path>,
    ) -> MnRequest {
        self.last_closures_run = None;
        let mut obs = Obs::default();
        let result = (|| {
            let cold = obs.span("frontend", |o| {
                let cold = crate::closures_frontend::cold(source, "mn.at").map_err(|e| match e {
                    crate::closures_frontend::ColdError::Source(ds) => {
                        crate::MnError::Unsupported("frontend", format!("{ds:?}"))
                    }
                    crate::closures_frontend::ColdError::Admit(e) => e,
                })?;
                o.counter("frontend_real_body_plans", cold.real_body_plans as u64);
                Ok::<_, crate::MnError>(cold)
            })?;
            let units = cold.units;
            let (build, run) = self.closures_units_inner(
                &units,
                source,
                entry,
                workdir,
                support,
                &mut obs,
                true,
            )?;
            let run = run.ok_or_else(|| bad("missing candidate execution"))?;
            Ok((
                run.exit_code,
                run.stdout,
                run.stderr,
                build.manifest_fp,
                build.manifest_artifact_id,
            ))
        })();
        let mut response = MnRequest::of(result, obs, workdir);
        if !response.ok {
            if let Some(run) = self.last_closures_run.take() {
                response.exit_code = run.exit_code;
                response.stdout = run.stdout;
                response.stderr = run.stderr;
            }
        }
        response
    }

    fn closures_units_inner(
        &mut self,
        units: &[MirModule],
        source: &str,
        entry: &str,
        workdir: &Path,
        support: Option<&Path>,
        obs: &mut Obs,
        execute: bool,
    ) -> Result<(MnBuild, Option<link_driver::PeRun>), crate::MnError> {
        let entry_meta = check_closures_interfaces(units, entry)?;
        let mir_bytes: u64 = units.iter().map(|u| u.heap_estimate() as u64).sum();
        if mir_bytes > crate::resources::PARTITION_CAPS[0] {
            return Err(crate::MnError::Resource(
                "mir-symbol",
                format!("closures program uses {mir_bytes} bytes"),
            ));
        }
        obs.ledger.charge(Partition::MirSymbol, mir_bytes);
        let runtime = obs.span("runtime_prepare", |_| runtime::prepare(support))?;
        obs.counter("runtime_cargo_invocations", runtime.cargo_invocations);
        let identity = runtime.identity.clone();
        let (objects, entries) = self.collect_objects(
            units,
            &|u, fp| CoffMeta::for_closures_unit(u, fp, &identity),
            &lower::emit_unit_coff,
            obs,
            None,
        )?;
        let (lld, sdk, kernel32) = obs.span("link_identity", |_| {
            link_driver::collections_link_identities()
        })?;
        let persist = self.store.is_some() && self.cache_mode != CacheMode::ReadOnly;
        let mut manifest_id = None;
        let mut manifest = art::Manifest {
            schema: art::MANIFEST_LAYER.schema_id.into(),
            profile: layout::PROFILE.into(),
            abi: lower::ABI.into(),
            target: MN_TARGET.into(),
            codegen_build: layout::codegen_build().into(),
            owner_module: entry_meta.owner.module.clone(),
            entry: entry_meta.unit.clone(),
            entry_symbol: entry_meta.symbol.clone(),
            source_hash: crate::cache::content_hash(source.as_bytes()),
            source: art::Blob {
                kind: "source".into(),
                hash: crate::cache::content_hash(source.as_bytes()),
                len: source.len(),
                pages: vec![],
                slice_offset: None,
            },
            coordinates: units
                .iter()
                .map(|unit| art::Coordinate::identity(unit, source))
                .collect(),
            runtime: identity,
            runtime_blob: art::Blob {
                kind: "runtime".into(),
                hash: String::new(),
                len: 0,
                pages: vec![],
                slice_offset: None,
            },
            units: vec![],
            lld,
            sdk,
            kernel32,
        };
        if persist {
            let store = self.store.as_ref().unwrap();
            let bytes = std::fs::read(&runtime.library)
                .map_err(|e| crate::MnError::Infra(e.to_string()))?;
            obs.ledger.charge(Partition::Io, bytes.len() as u64);
            manifest.runtime_blob = art::put_blob(store, "runtime", &bytes)?;
            if manifest.runtime_blob.hash != manifest.runtime.content_hash {
                return Err(bad("runtime library changed after preparation"));
            }
            obs.ledger.discharge(Partition::Io, bytes.len() as u64);
            drop(bytes);
            manifest.source = art::put_blob(store, "source", source.as_bytes())?;
            if entries.len() != units.len() {
                return Err(bad("missing persisted COFF row"));
            }
            for (unit, row) in units.iter().zip(entries) {
                let bytes = crate::closures_layout::unit_wire_bytes(unit)?;
                obs.ledger.charge(Partition::Staging, bytes.len() as u64);
                let mir = art::put_blob(store, "mir", &bytes)?;
                obs.ledger.discharge(Partition::Staging, bytes.len() as u64);
                manifest.units.push(art::Unit {
                    meta: (&row.meta).into(),
                    mir,
                    coff_id: row.artifact_id,
                    coff_hash: row.sha256,
                    coff_len: row.len,
                    extra_coff_pages: row.extra_page_ids,
                });
            }
            manifest.units.sort_by(|a, b| a.meta.unit.cmp(&b.meta.unit));
            manifest_id = Some(art::put_manifest(store, &manifest)?);
            obs.counter("link_manifests_persisted", 1);
        } else {
            for (unit, object) in units.iter().zip(&objects) {
                let meta = CoffMeta::for_closures_unit(
                    unit,
                    crate::closures_layout::unit_fp(unit)?,
                    &manifest.runtime,
                )?;
                manifest.units.push(art::Unit {
                    meta: (&meta).into(),
                    mir: art::Blob {
                        kind: "mir".into(),
                        hash: crate::closures_layout::unit_fp(unit)?,
                        len: 0,
                        pages: vec![],
                        slice_offset: None,
                    },
                    coff_id: String::new(),
                    coff_hash: crate::cache::content_hash(&object.bytes),
                    coff_len: object.bytes.len(),
                    extra_coff_pages: vec![],
                });
            }
            manifest.runtime_blob.hash = manifest.runtime.content_hash.clone();
        }
        let fp = manifest.fingerprint()?;
        self.finish_closures(
            objects,
            &entry_meta.symbol,
            &manifest.owner_module,
            &entry_meta.unit,
            &runtime.library,
            workdir,
            fp,
            manifest_id,
            obs,
            execute,
        )
    }

    fn finish_closures(
        &mut self,
        objects: Vec<link_driver::ObjectInput>,
        symbol: &str,
        owner: &str,
        entry: &str,
        library: &Path,
        workdir: &Path,
        fp: String,
        manifest_id: Option<String>,
        obs: &mut Obs,
        execute: bool,
    ) -> Result<(MnBuild, Option<link_driver::PeRun>), crate::MnError> {
        self.last_closures_run = None;
        let knobs = self.resource_knobs;
        let wrapper = obs.span("codegen", |o| {
            o.counter("codegen_emit_wrappers", 1);
            crate::collections_lower::emit_wrapper_coff(symbol, &knobs)
        })?;
        obs.ledger.charge(Partition::Staging, wrapper.len() as u64);
        if !obs.ledger.violations().is_empty() {
            return Err(crate::MnError::Resource(
                "managed-memory",
                obs.ledger.violations().join("; "),
            ));
        }
        std::fs::create_dir_all(workdir).map_err(|e| crate::MnError::Infra(e.to_string()))?;
        let candidate_dir = workdir.join(format!(
            ".closures-candidate-{}-{}",
            std::process::id(),
            super::collections::next_candidate()
        ));
        std::fs::create_dir(&candidate_dir).map_err(|e| crate::MnError::Infra(e.to_string()))?;
        let mut created_products = None;
        let result = (|| {
            let linked = obs.span("link", |o| {
                o.counter("link_invocations", 1);
                link_driver::link_only(
                    &objects,
                    &wrapper,
                    START_SYMBOL,
                    &candidate_dir,
                    &if self.test_omit_collections_runtime {
                        Vec::new()
                    } else {
                        vec![library.to_path_buf()]
                    },
                    &format!("closures manifest {fp}\n"),
                )
            })?;
            created_products = Some((linked.exe_path.clone(), linked.receipt_path.clone()));
            let run = if execute {
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
                let observed = match crate::collections_observe::decode(run.exit_code, &run.stderr)
                {
                    Ok(observed) => observed,
                    Err(error) => {
                        self.last_closures_run = Some(run);
                        return Err(error);
                    }
                };
                use crate::collections_observe::Outcome;
                let error = match observed.outcome {
                    Outcome::Trap(22) => Some(crate::MnError::Resource(
                        "execution-budget",
                        "native closure allocation/budget refused".into(),
                    )),
                    Outcome::Trap(23) => Some(crate::MnError::Admit(
                        "runtime-value",
                        "invalid native closure value/layout".into(),
                    )),
                    Outcome::Trap(25) => Some(crate::MnError::Resource(
                        "allocation-injected",
                        "native allocation fault seam fired".into(),
                    )),
                    Outcome::Trap(26) => Some(crate::MnError::Resource(
                        "cancelled",
                        "native closure operation cancelled before commit".into(),
                    )),
                    _ => None,
                };
                if let Some(error) = error {
                    self.last_closures_run = Some(run);
                    return Err(error);
                }
                Some(run)
            } else {
                None
            };
            crate::collections_memory::check()?;
            let mut already_rooted = false;
            let mut superseded = Vec::new();
            if let (Some(store), Some(project), Some(id)) =
                (&self.store, &self.project, &manifest_id)
            {
                let roots = store
                    .root_artifacts(project, "soft")
                    .map_err(|e| crate::MnError::Infra(format!("read roots: {e}")))?;
                already_rooted = roots.contains(id);
                // Only THIS family's roots are superseded; collections /
                // typed publications belong to their own manifest profiles.
                for old in roots.into_iter().filter(|old| old != id) {
                    if let Ok(previous) = art::get_manifest(store, &old) {
                        if previous.schema == art::MANIFEST_LAYER.schema_id
                            && previous.profile == layout::PROFILE
                            && previous.owner_module == owner
                            && previous.entry == entry
                        {
                            superseded.push(old);
                        }
                    }
                }
                store
                    .register_root(project, id, "soft")
                    .map_err(|e| crate::MnError::Infra(format!("publish root: {e}")))?;
            }
            let published = link_driver::publish_collections(&linked, workdir);
            if published.is_err() {
                if let (Some(store), Some(project), Some(id)) =
                    (&self.store, &self.project, &manifest_id)
                {
                    if !already_rooted {
                        store
                            .remove_root(project, "soft", id)
                            .map_err(|e| crate::MnError::Infra(format!("rollback new root: {e}")))?;
                    }
                }
            }
            let published = published?;
            if let (Some(store), Some(project)) = (&self.store, &self.project) {
                for old in superseded {
                    if store.remove_root(project, "soft", &old).is_err() {
                        obs.counter("stale_publication_root_retained", 1);
                    }
                }
            }
            for (i, name) in crate::resources::PARTITION_NAMES.iter().enumerate() {
                obs.counter(&format!("ledger_peak_{name}"), obs.ledger.peaks()[i]);
            }
            obs.counter("ledger_peak_total", obs.ledger.peak_total());
            let build = MnBuild {
                exe_path: published.exe_path,
                receipt_path: published.receipt_path,
                manifest_fp: fp,
                manifest_artifact_id: manifest_id,
                units: objects.len(),
                unit_sha256: objects
                    .iter()
                    .map(|o| (o.name.clone(), sha256_hex(&o.bytes)))
                    .collect(),
                obs: obs.clone(),
            };
            Ok((build, run))
        })();
        if let Some((exe, receipt)) = created_products {
            for path in [exe, receipt] {
                if let Some(error) = super::collections::reclaim_candidate(&path) {
                    eprintln!("closures candidate retained: {error}");
                }
            }
        }
        let _ = std::fs::remove_dir(candidate_dir);
        result
    }
}

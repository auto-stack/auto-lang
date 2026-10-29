//! Explicit collections requests share the driver's A09 machinery and native
//! linker. Persisted MIR is data, never a restored verification credential.
use super::*;
use crate::{collections_artifacts as art, collections_layout as layout};
use crate::{collections_lower as lower, collections_runtime as runtime};
use auto_av_core::mir::MirModule;
use std::collections::HashMap;

fn bad(message: impl Into<String>) -> MnError {
    MnError::Admit("collections-closure", message.into())
}
struct Entry {
    owner: auto_av_core::mir::OwnerScope,
    unit: String,
    symbol: String,
}
fn check_closure(units: &[MirModule], entry: &str) -> Result<Entry, MnError> {
    // No object producer is entered until the complete call/entry closure has
    // been admitted. Type-table indices have meaning only inside their unit.
    let mut functions = HashMap::new();
    let owner = units
        .first()
        .ok_or_else(|| bad("empty closure"))?
        .owner
        .clone();
    let mut shapes = Vec::with_capacity(units.len());
    for (i, unit) in units.iter().enumerate() {
        lower::admit_unit(unit)?;
        if unit.owner != owner || functions.insert(unit.funcs[0].name.clone(), i).is_some() {
            return Err(bad("mixed owner or duplicate function"));
        }
        shapes.push(layout::descriptors(unit)?);
    }
    for (i, unit) in units.iter().enumerate() {
        for import in &unit.imports {
            let name = import
                .symbol
                .strip_prefix("fn:")
                .ok_or_else(|| bad("non-function import"))?;
            let target = *functions
                .get(name)
                .ok_or_else(|| bad(format!("missing callee {name}")))?;
            let sig = units[target].funcs[0].sig as usize;
            if import.env.is_some() || shapes[i][import.sig as usize] != shapes[target][sig] {
                return Err(bad(format!("callee signature mismatch: {name}")));
            }
        }
    }
    let hits: Vec<_> = functions
        .iter()
        .filter(|(name, _)| name.as_str() == entry || name.as_str() == format!("d_{entry}"))
        .collect();
    if hits.len() != 1 {
        return Err(bad("entry is missing or ambiguous"));
    }
    let index = *hits[0].1;
    let unit = &units[index];
    match &unit.types[unit.funcs[0].sig as usize] {
        MirType::Fn(sig)
            if sig.params.is_empty()
                && matches!(
                    unit.types[sig.result as usize],
                    MirType::I32 | MirType::Bool
                ) => {}
        _ => return Err(bad("entry must take no arguments and return int or bool")),
    }
    Ok(Entry {
        owner,
        unit: unit.funcs[0].name.clone(),
        symbol: crate::symbols::function_symbol(&unit.owner, &unit.funcs[0].name),
    })
}

impl MnDriver {
    /// Source-only explicit profile. Preparation, shared frontend, cache and
    /// linking all belong to this call's measured request window.
    pub fn build_collections(
        &mut self,
        source: &str,
        entry: &str,
        workdir: &Path,
        support: Option<&Path>,
    ) -> Result<MnBuild, MnError> {
        let mut obs = Obs::default();
        let cold = obs.span("frontend", |o| {
            let cold = crate::collections_frontend::cold(source, "mn.at")
                .map_err(|ds| MnError::Unsupported("frontend", format!("{ds:?}")))?;
            o.counter("frontend_real_body_plans", cold.real_body_plans as u64);
            o.counter(
                "frontend_scaffold_declarations",
                cold.scaffold_declarations as u64,
            );
            Ok::<_, MnError>(cold)
        })?;
        let units = cold.units;
        drop(cold.env);
        self.collections_units(&units, source, entry, workdir, support, &mut obs, false)
    }

    /// Run candidates before publishing either their manifest root or PE. An
    /// allocation/cancellation/runtime invariant failure retains the old product.
    pub fn request_collections(
        &mut self,
        source: &str,
        entry: &str,
        workdir: &Path,
        support: Option<&Path>,
    ) -> MnRequest {
        let mut obs = Obs::default();
        let result = (|| {
            let cold = obs.span("frontend", |o| {
                let cold = crate::collections_frontend::cold(source, "mn.at")
                    .map_err(|ds| MnError::Unsupported("frontend", format!("{ds:?}")))?;
                o.counter("frontend_real_body_plans", cold.real_body_plans as u64);
                Ok::<_, MnError>(cold)
            })?;
            let units = cold.units;
            drop(cold.env);
            let (build, run) = self
                .collections_units_inner(&units, source, entry, workdir, support, &mut obs, true)?;
            let run = run.ok_or_else(|| bad("missing candidate execution"))?;
            Ok((
                run.exit_code,
                run.stdout,
                run.stderr,
                build.manifest_fp,
                build.manifest_artifact_id,
            ))
        })();
        MnRequest::of(result, obs, workdir)
    }

    fn collections_units(
        &mut self,
        units: &[MirModule],
        source: &str,
        entry: &str,
        workdir: &Path,
        support: Option<&Path>,
        obs: &mut Obs,
        execute: bool,
    ) -> Result<MnBuild, MnError> {
        self.collections_units_inner(units, source, entry, workdir, support, obs, execute)
            .map(|r| r.0)
    }

    fn collections_units_inner(
        &mut self,
        units: &[MirModule],
        source: &str,
        entry: &str,
        workdir: &Path,
        support: Option<&Path>,
        obs: &mut Obs,
        execute: bool,
    ) -> Result<(MnBuild, Option<link_driver::PeRun>), MnError> {
        let entry_meta = check_closure(units, entry)?;
        let mir_bytes: u64 = units.iter().map(|u| u.heap_estimate() as u64).sum();
        if mir_bytes > crate::resources::PARTITION_CAPS[0] {
            return Err(MnError::Resource(
                "mir-symbol",
                format!("closure uses {mir_bytes} bytes"),
            ));
        }
        obs.ledger.charge(Partition::MirSymbol, mir_bytes);
        let runtime = obs.span("runtime_prepare", |_| runtime::prepare(support))?;
        obs.counter("runtime_cargo_invocations", runtime.cargo_invocations);
        let identity = runtime.identity.clone();
        let (objects, entries) = self.collect_objects(
            units,
            &|u, fp| CoffMeta::for_collections_unit(u, fp, &identity),
            lower::emit_unit_coff,
            obs,
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
            source_hash: cache::content_hash(source.as_bytes()),
            runtime: identity,
            runtime_blob: art::Blob {
                kind: "runtime".into(),
                hash: String::new(),
                len: 0,
                pages: vec![],
            },
            units: vec![],
            lld,
            sdk,
            kernel32,
        };
        if persist {
            let store = self.store.as_ref().unwrap();
            let bytes =
                std::fs::read(&runtime.library).map_err(|e| MnError::Infra(e.to_string()))?;
            obs.ledger.charge(Partition::Io, bytes.len() as u64);
            manifest.runtime_blob = art::put_blob(store, "runtime", &bytes)?;
            if manifest.runtime_blob.hash != manifest.runtime.content_hash {
                return Err(bad("runtime library changed after preparation"));
            }
            obs.ledger.discharge(Partition::Io, bytes.len() as u64);
            drop(bytes);
            if entries.len() != units.len() {
                return Err(bad("missing persisted COFF row"));
            }
            for (unit, row) in units.iter().zip(entries) {
                let bytes = auto_av_core::encode_abc(unit)
                    .map_err(|e| bad(format!("encode MIR: {e:?}")))?
                    .to_flat();
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
            // The in-memory identity still describes every admitted unit. It
            // cannot be restored: no store id is emitted for an ephemeral build.
            for (unit, object) in units.iter().zip(&objects) {
                let meta =
                    CoffMeta::for_collections_unit(unit, cache::unit_fp(unit), &manifest.runtime)?;
                manifest.units.push(art::Unit {
                    meta: (&meta).into(),
                    mir: art::Blob {
                        kind: "mir".into(),
                        hash: cache::unit_fp(unit),
                        len: 0,
                        pages: vec![],
                    },
                    coff_id: String::new(),
                    coff_hash: cache::content_hash(&object.bytes),
                    coff_len: object.bytes.len(),
                    extra_coff_pages: vec![],
                });
            }
            manifest.runtime_blob.hash = manifest.runtime.content_hash.clone();
        }
        let fp = manifest.fingerprint()?;
        self.finish_collections(
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

    /// Independent restart entry. All claimed schemas/producer/runtime/layout
    /// axes are compared with the current producer, then MIR and COFF bytes are
    /// decoded and admitted again before any wrapper or link is produced.
    pub fn restore_collections(
        &mut self,
        id: &str,
        workdir: &Path,
        support: Option<&Path>,
    ) -> Result<MnBuild, MnError> {
        let mut obs = Obs::default();
        self.restore_collections_inner(id, workdir, support, &mut obs, false)
            .map(|r| r.0)
    }
    pub fn request_restored_collections(
        &mut self,
        id: &str,
        workdir: &Path,
        support: Option<&Path>,
    ) -> MnRequest {
        let mut obs = Obs::default();
        let result = self
            .restore_collections_inner(id, workdir, support, &mut obs, true)
            .and_then(|(build, run)| {
                let run = run.ok_or_else(|| bad("missing restored execution"))?;
                Ok((
                    run.exit_code,
                    run.stdout,
                    run.stderr,
                    build.manifest_fp,
                    build.manifest_artifact_id,
                ))
            });
        MnRequest::of(result, obs, workdir)
    }
    fn restore_collections_inner(
        &mut self,
        id: &str,
        workdir: &Path,
        support: Option<&Path>,
        obs: &mut Obs,
        execute: bool,
    ) -> Result<(MnBuild, Option<link_driver::PeRun>), MnError> {
        let store = self
            .store
            .as_ref()
            .ok_or_else(|| bad("restore requires an existing store"))?;
        let manifest = obs.span("manifest_restore", |_| art::get_manifest(store, id))?;
        if manifest.schema != art::MANIFEST_LAYER.schema_id
            || manifest.profile != layout::PROFILE
            || manifest.abi != lower::ABI
            || manifest.target != MN_TARGET
            || manifest.codegen_build != layout::codegen_build()
            || manifest.units.is_empty()
            || manifest.source_hash.len() != 64
            || !manifest.source_hash.bytes().all(|b| b.is_ascii_hexdigit())
        {
            return Err(bad(
                "manifest schema/profile/ABI/target/producer/source identity",
            ));
        }
        let runtime = obs.span("runtime_prepare", |_| runtime::prepare(support))?;
        obs.counter("runtime_cargo_invocations", runtime.cargo_invocations);
        if runtime.identity != manifest.runtime
            || manifest.runtime_blob.hash != runtime.identity.content_hash
        {
            return Err(bad("runtime build/content identity changed"));
        }
        obs.span("runtime_restore", |_| {
            art::verify_runtime_blob(store, &manifest.runtime_blob)
        })?;
        let current_tools = obs.span("link_identity", |_| {
            link_driver::collections_link_identities()
        })?;
        if current_tools
            != (
                manifest.lld.clone(),
                manifest.sdk.clone(),
                manifest.kernel32.clone(),
            )
        {
            return Err(bad("linker/SDK identity changed"));
        }
        let mut units = Vec::with_capacity(manifest.units.len());
        let mut objects = Vec::with_capacity(manifest.units.len());
        for row in &manifest.units {
            let wire = obs.span("mir_restore", |_| art::get_blob(store, &row.mir, "mir"))?;
            obs.ledger.charge(Partition::Io, wire.len() as u64);
            if row.meta.owner_schema != auto_av_core::contract::MIR_COLLECTIONS_SCHEMA {
                return Err(bad("unexpected MIR owner schema"));
            }
            let unit = auto_av_core::verify_abc_bytes(&wire, &row.meta.to_meta().owner)
                .map_err(|e| bad(format!("MIR wire admission: {e:?}")))?
                .module()
                .clone();
            obs.ledger.discharge(Partition::Io, wire.len() as u64);
            drop(wire);
            lower::admit_unit(&unit)?;
            obs.counter("restored_mir_units_readmitted", 1);
            let meta =
                CoffMeta::for_collections_unit(&unit, cache::unit_fp(&unit), &runtime.identity)?;
            if meta != row.meta.to_meta() || meta.owner.module != manifest.owner_module {
                return Err(bad(
                    "restored MIR does not match full claimed COFF identity",
                ));
            }
            let bytes = obs.span("coff_restore", |_| {
                cache::get_unit_coff_by_id(store, &meta, &row.coff_id)
            })?;
            if bytes.len() != row.coff_len
                || cache::content_hash(&bytes) != row.coff_hash
                || cache::unit_coff_page_ids(store, &meta, bytes.len())? != row.extra_coff_pages
            {
                return Err(bad("COFF hash/length/required page mismatch"));
            }
            obs.counter("cache_hit_coff", 1);
            obs.ledger.charge(Partition::Io, bytes.len() as u64);
            obs.ledger
                .charge(Partition::MirSymbol, unit.heap_estimate() as u64);
            if !obs.ledger.violations().is_empty() {
                return Err(MnError::Resource(
                    "managed-memory",
                    obs.ledger.violations().join("; "),
                ));
            }
            objects.push(ObjectInput {
                name: meta.unit,
                bytes,
            });
            units.push(unit);
        }
        let entry = check_closure(&units, &manifest.entry)?;
        if entry.symbol != manifest.entry_symbol || entry.unit != manifest.entry {
            return Err(bad("entry identity/symbol mismatch"));
        }
        let fp = manifest.fingerprint()?;
        let manifest_id = if self.cache_mode == CacheMode::ReadOnly {
            None
        } else {
            Some(id.to_string())
        };
        self.finish_collections(
            objects,
            &entry.symbol,
            &manifest.owner_module,
            &manifest.entry,
            &runtime.library,
            workdir,
            fp,
            manifest_id,
            obs,
            execute,
        )
    }

    fn finish_collections(
        &mut self,
        objects: Vec<ObjectInput>,
        symbol: &str,
        owner: &str,
        entry: &str,
        library: &Path,
        workdir: &Path,
        fp: String,
        manifest_id: Option<String>,
        obs: &mut Obs,
        execute: bool,
    ) -> Result<(MnBuild, Option<link_driver::PeRun>), MnError> {
        let wrapper = obs.span("codegen", |o| {
            o.counter("codegen_emit_wrappers", 1);
            lower::emit_wrapper_coff(symbol)
        })?;
        obs.ledger.charge(Partition::Staging, wrapper.len() as u64);
        if !obs.ledger.violations().is_empty() {
            return Err(MnError::Resource(
                "managed-memory",
                obs.ledger.violations().join("; "),
            ));
        }
        std::fs::create_dir_all(workdir).map_err(|e| MnError::Infra(e.to_string()))?;
        let candidate_dir = workdir.join(format!(
            ".collections-candidate-{}-{}",
            std::process::id(),
            next_candidate()
        ));
        std::fs::create_dir(&candidate_dir).map_err(|e| MnError::Infra(e.to_string()))?;
        let mut created_products = None;
        let result = (|| {
            let linked = obs.span("link", |o| {
                o.counter("link_invocations", 1);
                link_driver::link_only(
                    &objects,
                    &wrapper,
                    START_SYMBOL,
                    &candidate_dir,
                    &[library.to_path_buf()],
                    &format!("collections manifest {fp}\n"),
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
                let observed = crate::collections_observe::decode(run.exit_code, &run.stderr)?;
                use crate::collections_observe::Outcome;
                match observed.outcome {
                    Outcome::Trap(22) => {
                        return Err(MnError::Resource(
                            "execution-budget",
                            "native collection allocation/budget refused".into(),
                        ))
                    }
                    Outcome::Trap(23) => {
                        return Err(MnError::Admit(
                            "runtime-value",
                            "invalid native collection value/layout".into(),
                        ))
                    }
                    Outcome::Trap(25) => {
                        return Err(MnError::Resource(
                            "allocation-injected",
                            "native allocation fault seam fired".into(),
                        ))
                    }
                    Outcome::Trap(26) => {
                        return Err(MnError::Resource(
                            "cancelled",
                            "native collection operation cancelled before commit".into(),
                        ))
                    }
                    _ => {}
                }
                Some(run)
            } else {
                None
            };
            let mut already_rooted = false;
            let mut superseded = Vec::new();
            if let (Some(store), Some(project), Some(id)) =
                (&self.store, &self.project, &manifest_id)
            {
                let roots = store
                    .root_artifacts(project, "soft")
                    .map_err(|e| MnError::Infra(format!("read roots: {e}")))?;
                already_rooted = roots.contains(id);
                // Reconstruct this profile's publication set after a process
                // restart too. Old five-profile roots belong to their own
                // manifest schema and must never be consumed or removed here.
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
                    .map_err(|e| MnError::Infra(format!("publish root: {e}")))?;
            }
            let published = link_driver::publish_collections(&linked, workdir);
            if published.is_err() {
                if let (Some(store), Some(project), Some(id)) =
                    (&self.store, &self.project, &manifest_id)
                {
                    // Reused current roots must not disappear on failed publish.
                    if !already_rooted {
                        store
                            .remove_root(project, "soft", id)
                            .map_err(|e| MnError::Infra(format!("rollback new root: {e}")))?;
                    }
                }
            }
            let published = published?;
            if let (Some(store), Some(project)) = (&self.store, &self.project) {
                for old in superseded {
                    if store.remove_root(project, "soft", &old).is_err() {
                        // The committed closure is safe; retain and report a
                        // stale root rather than report a failed transaction.
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
        // Reclaim only this request's known product paths. No recursive removal
        // and no traversal through a worktree link is involved.
        if let Some((exe, receipt)) = created_products {
            for path in [exe, receipt] {
                if let Some(error) = reclaim_candidate(&path) {
                    eprintln!("collections candidate retained: {error}");
                }
            }
        }
        let _ = std::fs::remove_dir(candidate_dir);
        result
    }
}
fn reclaim_candidate(path: &Path) -> Option<String> {
    match std::fs::remove_file(path) {
        Ok(()) => None,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
        Err(e) => Some(format!("{}: {e}", path.display())),
    }
}
fn next_candidate() -> u64 {
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn closure_admission_checks_cross_unit_signatures_and_entry_ambiguity() {
        let first = crate::collections_frontend::cold(
            "fn value() int { return 42 }\nfn main() int { return value() }",
            "mn.at",
        )
        .unwrap();
        let second=crate::collections_frontend::cold("fn value() bool { return true }\nfn main() int { if value() { return 42 }\n return 0 }","mn.at").unwrap();
        let mut units = first.units;
        let foreign = second
            .units
            .into_iter()
            .find(|u| u.funcs[0].name == "d_value")
            .unwrap();
        let slot = units
            .iter()
            .position(|u| u.funcs[0].name == "d_value")
            .unwrap();
        units[slot] = foreign;
        assert!(check_closure(&units, "main")
            .err()
            .unwrap()
            .message()
            .contains("signature mismatch"));
        let ambiguous = crate::collections_frontend::cold(
            "fn main() int { return 42 }\nfn d_main() int { return 43 }",
            "mn.at",
        )
        .unwrap();
        assert!(check_closure(&ambiguous.units, "d_main").is_err());
    }
}

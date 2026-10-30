//! Native consumer of the shared retained source environment and 828 candidate
//! transaction. Pure body edits never rebuild another function's producer.
use crate::driver::collections::{
    check_resident_interfaces, checked_collection_interface, ResidentCollectionUnit,
};
use crate::driver::{MnBuild, MnDriver};
use crate::{collections_artifacts as art, collections_runtime as runtime};
use crate::{observe::Obs, resources::Partition, MnError};
use auto_ac_prototype::{body_work::BodyWork, source as ac};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    rc::Rc,
    time::Instant,
};

#[derive(Clone, Debug, Default)]
pub struct CollectionWork {
    pub syntax: ac::parser::SyntaxWork,
    pub bodies: BTreeMap<String, BodyWork>,
    pub obs: Obs,
    pub elapsed_ms: u128,
}
pub struct CollectionRun {
    pub build: MnBuild,
    pub exit_code: Option<i32>,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub work: CollectionWork,
}
/// Actual degenerations used by counter-controls, never source-language flags.
#[derive(Default)]
pub struct CollectionNegativeControls {
    pub whole_frontend: bool,
    pub whole_lower: bool,
    pub copy_unchanged_code: bool,
}
pub struct PreparedCollectionsEdit {
    session_id: u64,
    revision: u64,
    source: String,
    regions: Vec<ac::resolve::FunctionRegion>,
    mint_origins: Vec<usize>,
    candidate: Option<ac::FnCandidate>,
    environment: Option<ac::CheckedEnv>,
    units: Vec<Rc<ResidentCollectionUnit>>,
    runtime: runtime::Prepared,
    changed: Option<usize>,
    invalidation: bool,
    pub work: CollectionWork,
}
impl PreparedCollectionsEdit {
    pub fn changed_function(&self) -> Option<&str> {
        self.changed.map(|i| self.regions[i].name.as_str())
    }
    /// No environment reference is taken: cancelling cannot change current.
    pub fn cancel(self) {}
    pub fn is_declaration_invalidation(&self) -> bool {
        self.invalidation
    }
}
pub struct CollectionsIncrementalSession {
    id: u64,
    revision: u64,
    env: ac::CheckedEnv,
    source: String,
    regions: Vec<ac::resolve::FunctionRegion>,
    mint_origins: Vec<usize>,
    units: Vec<Rc<ResidentCollectionUnit>>,
    driver: MnDriver,
    runtime: Option<runtime::Identity>,
    runtime_blob: Option<art::Blob>,
    link_identity: Option<[String; 3]>,
    support: Option<PathBuf>,
    entry: String,
    workdir: PathBuf,
    manifest_id: Option<String>,
    manifest_fp: String,
    last_work: CollectionWork,
    last_failure_run: Option<crate::link_driver::PeRun>,
    closed: bool,
    pub negative_controls: CollectionNegativeControls,
}
fn frontend(errors: Vec<auto_ac_prototype::atom_text::Diagnostic>) -> MnError {
    MnError::Unsupported("frontend", format!("{errors:?}"))
}
fn invalid(message: impl Into<String>) -> MnError {
    MnError::Admit("collections-session", message.into())
}
fn check_ledger(obs: &Obs) -> Result<(), MnError> {
    crate::collections_memory::check()?;
    let violations = obs.ledger.violations();
    if violations.is_empty() {
        Ok(())
    } else {
        Err(MnError::Resource("managed-memory", violations.join("; ")))
    }
}
impl CollectionsIncrementalSession {
    pub fn new(
        source: &str,
        entry: &str,
        mut driver: MnDriver,
        workdir: &Path,
        support: Option<&Path>,
    ) -> Result<(Self, CollectionRun), MnError> {
        let started = Instant::now();
        let mut obs = Obs::default();
        let ((cold, bodies), syntax) = ac::parser::capture_syntax_work(|| {
            auto_ac_prototype::body_work::capture_body_work(|| {
                obs.span("frontend", |_| {
                    crate::collections_frontend::cold_admitted(source, "mn.at").map_err(frontend)
                })
            })
        });
        let cold = cold?;
        let regions = ac::env_fn_regions(&cold.env);
        let interfaces = obs.span("cold_closure_admit", |_| {
            cold.units
                .iter()
                .map(checked_collection_interface)
                .collect::<Result<Vec<_>, _>>()
        })?;
        check_resident_interfaces(&interfaces, entry)?;
        let prepared = obs.span("runtime_prepare", |_| runtime::prepare(support))?;
        obs.counter("runtime_cargo_invocations", prepared.cargo_invocations);
        let mut units = Vec::with_capacity(cold.units.len());
        let mut packer = art::MirPagePacker::default();
        obs.ledger
            .charge(Partition::Staging, crate::config::PAGE_BYTES as u64);
        obs.ledger.charge(Partition::Staging, source.len() as u64);
        for checked in cold.units {
            obs.ledger.charge(
                Partition::MirSymbol,
                checked.module().heap_estimate() as u64,
            );
            check_ledger(&obs)?;
            units.push(Rc::new(driver.prepare_packed_collection_unit(
                checked,
                &prepared.identity,
                &mut obs,
                &mut packer,
            )?));
        }
        driver.finish_collection_pack(packer, &mut units, &mut obs)?;
        obs.ledger
            .discharge(Partition::Staging, crate::config::PAGE_BYTES as u64);
        let mint_origins: Vec<_> = regions.iter().map(|region| region.item.start).collect();
        let coordinates = coordinate_records(&units, &regions, &mint_origins, source)?;
        let (build, run, runtime_blob, link_identity) = driver.run_resident_collections(
            &units,
            source,
            entry,
            &prepared,
            None,
            &coordinates,
            workdir,
            &mut obs,
        )?;
        let work = CollectionWork {
            syntax,
            bodies,
            obs,
            elapsed_ms: started.elapsed().as_millis(),
        };
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
        let session = Self {
            id: NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
            revision: 0,
            env: cold.env,
            source: source.into(),
            mint_origins,
            regions,
            units,
            driver,
            runtime: Some(prepared.identity),
            runtime_blob,
            link_identity: Some(link_identity),
            support: support.map(Path::to_path_buf),
            entry: entry.into(),
            workdir: workdir.into(),
            manifest_id: build.manifest_artifact_id.clone(),
            manifest_fp: build.manifest_fp.clone(),
            last_work: work.clone(),
            last_failure_run: None,
            closed: false,
            negative_controls: CollectionNegativeControls::default(),
        };
        Ok((
            session,
            CollectionRun {
                build,
                exit_code: run.exit_code,
                stdout: run.stdout,
                stderr: run.stderr,
                work,
            },
        ))
    }
    pub fn source(&self) -> &str {
        &self.source
    }
    pub fn revision(&self) -> u64 {
        self.revision
    }
    pub fn runtime_identity(&self) -> Option<&runtime::Identity> {
        self.runtime.as_ref()
    }
    pub fn link_identity(&self) -> Option<&[String; 3]> {
        self.link_identity.as_ref()
    }
    pub fn manifest_id(&self) -> Option<&str> {
        self.manifest_id.as_deref()
    }
    pub fn manifest_fingerprint(&self) -> &str {
        &self.manifest_fp
    }
    pub fn last_work(&self) -> &CollectionWork {
        &self.last_work
    }
    pub fn last_failure_run(&self) -> Option<&crate::link_driver::PeRun> {
        self.last_failure_run.as_ref()
    }
    pub fn test_omit_runtime(&mut self, omit: bool) {
        self.driver.test_omit_collections_runtime = omit;
    }
    /// Actual retained source/MIR/code allocations, for structural evidence.
    pub fn identities(&self) -> BTreeMap<String, (usize, usize, usize)> {
        self.units
            .iter()
            .enumerate()
            .map(|(i, u)| {
                (
                    self.regions[i].name.clone(),
                    (
                        ac::env_fn_body_identity(&self.env, i).unwrap_or(0),
                        Rc::as_ptr(&u.mir) as usize,
                        Rc::as_ptr(&u.coff) as usize,
                    ),
                )
            })
            .collect()
    }
    pub fn unit_fingerprints(&self) -> BTreeMap<String, String> {
        self.units
            .iter()
            .map(|u| (u.meta.unit.clone(), u.meta.fp.clone()))
            .collect()
    }
    /// Re-anchor the retained unit's diagnostic spans with one origin delta.
    /// Changed bodies always have freshly lowered MIR (including history COFF
    /// hits); unchanged function text only moves as one intact source range.
    pub fn mapped_spans(
        &self,
        function: &str,
    ) -> Result<Vec<(u32, auto_av_core::contract::SpanInfo)>, MnError> {
        let index = self
            .regions
            .iter()
            .position(|r| r.name == function)
            .ok_or_else(|| invalid("unknown function"))?;
        let delta = self.regions[index].item.start as isize - self.mint_origins[index] as isize;
        self.units[index].mir.funcs[0]
            .spans
            .iter()
            .map(|(instruction, span)| {
                let start = (span.start as usize)
                    .checked_add_signed(delta)
                    .ok_or_else(|| invalid("span start overflow"))?;
                let end = (span.end as usize)
                    .checked_add_signed(delta)
                    .ok_or_else(|| invalid("span end overflow"))?;
                if start > end
                    || end > self.source.len()
                    || !self.source.is_char_boundary(start)
                    || !self.source.is_char_boundary(end)
                {
                    return Err(invalid("mapped span is outside the current UTF-8 source"));
                }
                Ok((
                    *instruction,
                    auto_av_core::contract::SpanInfo {
                        func: span.func,
                        start: start as u32,
                        end: end as u32,
                    },
                ))
            })
            .collect()
    }
    fn request_ledger(&self, obs: &mut Obs) {
        obs.ledger.charge(
            Partition::MirSymbol,
            self.units.iter().map(|u| u.mir_bytes as u64).sum(),
        );
        obs.ledger.charge(
            Partition::Staging,
            self.source.capacity() as u64
                + self
                    .units
                    .iter()
                    .map(|u| u.coff.capacity() as u64)
                    .sum::<u64>(),
        );
    }
    pub fn prepare_source(&mut self, source: &str) -> Result<PreparedCollectionsEdit, MnError> {
        self.last_failure_run = None;
        let started = Instant::now();
        let mut obs = Obs::default();
        self.request_ledger(&mut obs);
        let ((result, bodies), syntax) = ac::parser::capture_syntax_work(|| {
            auto_ac_prototype::body_work::capture_body_work(|| self.prepare_body(source, &mut obs))
        });
        let work = CollectionWork {
            syntax,
            bodies,
            obs,
            elapsed_ms: started.elapsed().as_millis(),
        };
        self.last_work = work.clone();
        let mut prepared = result?;
        prepared.work = work;
        // Producer-side evidence and actual allocation identity are enforced,
        // rather than inferred from equal content or a caller's "carry" flag.
        for (i, (old, new)) in self.units.iter().zip(&prepared.units).enumerate() {
            if Some(i) != prepared.changed
                && (!Rc::ptr_eq(&old.mir, &new.mir) || !Rc::ptr_eq(&old.coff, &new.coff))
            {
                return Err(invalid(format!(
                    "unchanged unit {} lost producer identity",
                    old.meta.unit
                )));
            }
        }
        if let Some(index) = prepared.changed {
            let name = &self.regions[index].name;
            if prepared.work.syntax.parsed_functions.len() != 1
                || prepared.work.syntax.parsed_functions.get(name) != Some(&1)
                || prepared.work.bodies.keys().any(|function| function != name)
                || prepared.work.bodies.get(name)
                    != Some(&BodyWork {
                        resolve: 1,
                        typecheck: 1,
                        plan: 1,
                        bind: 1,
                    })
            {
                return Err(invalid(
                    "pure body edit performed work on an unchanged function",
                ));
            }
        }
        Ok(prepared)
    }
    fn prepare_body(
        &mut self,
        source: &str,
        obs: &mut Obs,
    ) -> Result<PreparedCollectionsEdit, MnError> {
        if self.closed {
            return Err(invalid("session is closed"));
        }
        if source.len() > crate::config::MAX_INPUT_BYTES {
            return Err(MnError::Resource(
                "input-size",
                "source exceeds native input budget".into(),
            ));
        }
        let prepared = obs.span("runtime_prepare", |_| {
            runtime::prepare(self.support.as_deref())
        })?;
        obs.counter("runtime_cargo_invocations", prepared.cargo_invocations);
        if Some(&prepared.identity) != self.runtime.as_ref() {
            return Err(invalid(
                "runtime changed; explicit dependency invalidation is required",
            ));
        }
        let mut regions = self.regions.clone();
        let mut mint_origins = self.mint_origins.clone();
        let mut units = self.units.clone();
        let mut candidate = None;
        let mut changed = None;
        if source != self.source {
            let (start, old_end, new_end) = difference(&self.source, source);
            let index = regions
                .iter()
                .position(|r| start > r.body.start && old_end < r.body.end)
                .ok_or_else(|| {
                    MnError::Unsupported(
                        "declaration-invalidation",
                        "edit leaves one function body; use explicit declaration invalidation"
                            .into(),
                    )
                })?;
            let delta = new_end as isize - old_end as isize;
            let end = regions[index]
                .item
                .end
                .checked_add_signed(delta)
                .ok_or_else(|| invalid("edited range overflow"))?;
            if self.negative_controls.whole_frontend {
                obs.span("negative_whole_frontend", |o| {
                    o.counter("negative_control_whole_frontend", 1);
                    crate::collections_frontend::cold(source, "mn.at").map_err(frontend)
                })?;
            }
            if self.negative_controls.whole_lower {
                obs.span("negative_whole_lower", |o| {
                    for region in &self.regions {
                        let (parts, _) =
                            ac::emit_function_body(&self.env, region.index, &self.source, "mn.at")
                                .map_err(frontend)?;
                        auto_mir_lower::hir_adapter::lower_function_collections_unit(
                            &parts.bundle,
                            &format!("d_{}", region.name),
                            &[],
                            auto_av_core::contract::fnv1a64(self.source.as_bytes()),
                        )
                        .map_err(|e| invalid(format!("forced lower: {e:?}")))?;
                        o.counter(&format!("negative_control_lower:{}", region.name), 1);
                    }
                    Ok::<_, MnError>(())
                })?;
            }
            let item = obs.span("syntax_fragment", |_| {
                ac::parse_function_fragment(
                    source,
                    regions[index].item.start..end,
                    ac::SourceProfile::Collections,
                )
                .map_err(|error| frontend(vec![error]))
            })?;
            let document = obs.span("frontend_body", |_| {
                ac::prepare_candidate_document(&mut self.env, &item, index, source, "mn.at")
                    .map_err(frontend)
            })?;
            let name = format!("d_{}", regions[index].name);
            let bundle = &document.parts().bundle;
            let def = bundle.modules[0]
                .defs
                .iter()
                .position(|d| d.id_text == name)
                .ok_or_else(|| invalid("candidate declaration missing"))?;
            obs.span("verify_hir", |_| {
                auto_ac_prototype::verify::verify_function_ref(bundle, def).map_err(frontend)
            })?;
            let mir = crate::collections_memory::scoped(Partition::MirSymbol, || {
                obs.span("mir_lower", |o| {
                    o.counter(&format!("mir_lower:{name}"), 1);
                    auto_mir_lower::hir_adapter::lower_function_collections_unit(
                        bundle,
                        &name,
                        &[],
                        auto_av_core::contract::fnv1a64(source.as_bytes()),
                    )
                    .map_err(|e| MnError::Unsupported("adapter", format!("{e:?}")))
                })
            })?;
            let checked = crate::collections_memory::scoped(Partition::MirSymbol, || {
                crate::collections_lower::CheckedCollectionUnit::admit(Rc::new(mir))
            })?;
            let interface = checked_collection_interface(&checked)?;
            check_resident_interfaces(
                units
                    .iter()
                    .enumerate()
                    .map(|(i, u)| if i == index { &interface } else { &u.interface }),
                &self.entry,
            )?;
            obs.ledger.charge(
                Partition::MirSymbol,
                checked.module().heap_estimate() as u64,
            );
            check_ledger(obs)?;
            units[index] = Rc::new(self.driver.prepare_collection_unit(
                checked,
                &prepared.identity,
                obs,
            )?);
            if self.negative_controls.copy_unchanged_code {
                for (i, unit) in units.iter_mut().enumerate() {
                    if i != index {
                        let mut replacement = unit.as_ref().clone();
                        replacement.coff = Rc::new(unit.coff.as_ref().clone());
                        *unit = Rc::new(replacement);
                        obs.counter("negative_control_same_content_copy", 1);
                    }
                }
            }
            for region in &mut regions {
                if region.index == index {
                    region.item = item.span;
                    region.body = item.body.span;
                } else if region.item.start >= self.regions[index].item.end {
                    for value in [
                        &mut region.item.start,
                        &mut region.item.end,
                        &mut region.body.start,
                        &mut region.body.end,
                    ] {
                        *value = value
                            .checked_add_signed(delta)
                            .ok_or_else(|| invalid("source origin overflow"))?;
                    }
                }
            }
            mint_origins[index] = regions[index].item.start;
            candidate = Some(document);
            changed = Some(index);
        }
        obs.ledger.charge(Partition::Staging, source.len() as u64);
        check_ledger(obs)?;
        Ok(PreparedCollectionsEdit {
            session_id: self.id,
            revision: self.revision,
            source: source.into(),
            regions,
            mint_origins,
            candidate,
            environment: None,
            units,
            runtime: prepared,
            changed,
            invalidation: false,
            work: CollectionWork::default(),
        })
    }
    /// Explicit declaration/layout/signature/configuration invalidation. This
    /// route truthfully records its cold frontend work; byte-identical units
    /// keep the actual resident MIR/code objects, and only incompatible native
    /// identities enter the object producer. Pure body edits use prepare_source.
    pub fn prepare_declaration_change(
        &mut self,
        source: &str,
    ) -> Result<PreparedCollectionsEdit, MnError> {
        self.last_failure_run = None;
        let started = Instant::now();
        let mut obs = Obs::default();
        self.request_ledger(&mut obs);
        let ((result, bodies), syntax) = ac::parser::capture_syntax_work(|| {
            auto_ac_prototype::body_work::capture_body_work(|| {
                if self.closed {
                    return Err(invalid("session is closed"));
                }
                let cold = obs.span("declaration_frontend", |o| {
                    o.counter("declaration_frontend_invalidations", 1);
                    crate::collections_frontend::cold_admitted(source, "mn.at").map_err(frontend)
                })?;
                let interfaces = cold
                    .units
                    .iter()
                    .map(checked_collection_interface)
                    .collect::<Result<Vec<_>, _>>()?;
                check_resident_interfaces(&interfaces, &self.entry)?;
                let prepared = obs.span("runtime_prepare", |_| {
                    runtime::prepare(self.support.as_deref())
                })?;
                obs.counter("runtime_cargo_invocations", prepared.cargo_invocations);
                let regions = ac::env_fn_regions(&cold.env);
                // The new cold MIR set coexists with the current publication
                // until commit. Charge every temporary unit now, not just the
                // incompatible units eventually retained by the candidate.
                obs.ledger.charge(
                    Partition::MirSymbol,
                    cold.units
                        .iter()
                        .map(|unit| unit.module().heap_estimate() as u64)
                        .sum(),
                );
                check_ledger(&obs)?;
                let mut mint_origins = Vec::with_capacity(regions.len());
                let mut units = Vec::with_capacity(regions.len());
                for (index, checked) in cold.units.into_iter().enumerate() {
                    let meta = crate::cache::CoffMeta::for_checked_collections(
                        &checked,
                        &prepared.identity,
                    )?;
                    if let Some((old_index, old)) =
                        self.units.iter().enumerate().find(|(_, u)| u.meta == meta)
                    {
                        let old_range = &self.regions[old_index].item;
                        let new_range = &regions[index].item;
                        // Equal code can come from differently spelled source. A
                        // full invalidation remints MIR for changed spelling so its
                        // spans cannot accidentally adopt an old coordinate frame.
                        if self.source.get(old_range.start..old_range.end)
                            == source.get(new_range.start..new_range.end)
                        {
                            units.push(old.clone());
                            mint_origins.push(self.mint_origins[old_index]);
                            obs.ledger.discharge(
                                Partition::MirSymbol,
                                checked.module().heap_estimate() as u64,
                            );
                            continue;
                        }
                    }
                    units.push(Rc::new(self.driver.prepare_collection_unit(
                        checked,
                        &prepared.identity,
                        &mut obs,
                    )?));
                    mint_origins.push(regions[index].item.start);
                }
                obs.ledger.charge(Partition::Staging, source.len() as u64);
                check_ledger(&obs)?;
                Ok(PreparedCollectionsEdit {
                    session_id: self.id,
                    revision: self.revision,
                    source: source.into(),
                    regions,
                    mint_origins,
                    candidate: None,
                    environment: Some(cold.env),
                    units,
                    runtime: prepared,
                    changed: None,
                    invalidation: true,
                    work: CollectionWork::default(),
                })
            })
        });
        let work = CollectionWork {
            syntax,
            bodies,
            obs,
            elapsed_ms: started.elapsed().as_millis(),
        };
        self.last_work = work.clone();
        let mut prepared = result?;
        prepared.work = work;
        Ok(prepared)
    }
    pub fn commit_prepared(
        &mut self,
        prepared: PreparedCollectionsEdit,
    ) -> Result<CollectionRun, MnError> {
        self.last_failure_run = None;
        self.last_work = prepared.work.clone();
        if self.closed || prepared.session_id != self.id || prepared.revision != self.revision {
            return Err(invalid("closed, foreign or stale prepared edit"));
        }
        if let Some(candidate) = &prepared.candidate {
            candidate.validate(&self.env).map_err(frontend)?;
        }
        let started = Instant::now();
        let mut work = prepared.work;
        let current_runtime = work.obs.span("runtime_commit_check", |o| {
            let runtime = runtime::prepare(self.support.as_deref())?;
            o.counter("runtime_cargo_invocations", runtime.cargo_invocations);
            if runtime.identity != prepared.runtime.identity
                || runtime.library != prepared.runtime.library
            {
                return Err(invalid("prepared runtime/configuration is stale"));
            }
            Ok(())
        });
        if let Err(error) = current_runtime {
            work.elapsed_ms += started.elapsed().as_millis();
            self.last_work = work;
            return Err(error);
        }
        let prior_blob = if self.runtime.as_ref() == Some(&prepared.runtime.identity) {
            self.runtime_blob.as_ref()
        } else {
            None
        };
        let coordinates = coordinate_records(
            &prepared.units,
            &prepared.regions,
            &prepared.mint_origins,
            &prepared.source,
        )?;
        let result = self.driver.run_resident_collections(
            &prepared.units,
            &prepared.source,
            &self.entry,
            &prepared.runtime,
            prior_blob,
            &coordinates,
            &self.workdir,
            &mut work.obs,
        );
        work.elapsed_ms += started.elapsed().as_millis();
        self.last_work = work.clone();
        let (build, run, blob, link_identity) = match result {
            Ok(result) => result,
            Err(error) => {
                self.last_failure_run = self.driver.take_collections_run();
                return Err(error);
            }
        };
        // No fallible producer work follows native publication. Exclusive
        // access and the validation above keep owner/base/slot unchanged.
        if let Some(candidate) = prepared.candidate {
            candidate.commit(&mut self.env).map_err(frontend)?;
        }
        if self.source != prepared.source || prepared.invalidation {
            self.revision += 1;
        }
        if let Some(environment) = prepared.environment {
            self.env = environment;
        }
        self.source = prepared.source;
        self.regions = prepared.regions;
        self.mint_origins = prepared.mint_origins;
        self.units = prepared.units;
        self.runtime = Some(prepared.runtime.identity);
        self.runtime_blob = blob;
        self.link_identity = Some(link_identity);
        self.manifest_id = build.manifest_artifact_id.clone();
        self.manifest_fp = build.manifest_fp.clone();
        Ok(CollectionRun {
            build,
            exit_code: run.exit_code,
            stdout: run.stdout,
            stderr: run.stderr,
            work,
        })
    }
    pub fn apply_source(&mut self, source: &str) -> Result<CollectionRun, MnError> {
        let prepared = self.prepare_source(source)?;
        self.commit_prepared(prepared)
    }
    pub fn close(&mut self) {
        self.env.close();
        self.source = String::new();
        self.regions = Vec::new();
        self.mint_origins = Vec::new();
        self.units = Vec::new();
        self.runtime = None;
        self.last_failure_run = None;
        self.runtime_blob = None;
        self.link_identity = None;
        self.support = None;
        self.driver = MnDriver::ephemeral();
        self.closed = true;
    }
}
fn difference(old: &str, new: &str) -> (usize, usize, usize) {
    let mut start = 0;
    while start < old.len().min(new.len()) && old.as_bytes()[start] == new.as_bytes()[start] {
        start += 1;
    }
    while !old.is_char_boundary(start) || !new.is_char_boundary(start) {
        start -= 1;
    }
    let mut suffix = 0;
    while suffix < old.len() - start
        && suffix < new.len() - start
        && old.as_bytes()[old.len() - suffix - 1] == new.as_bytes()[new.len() - suffix - 1]
    {
        suffix += 1;
    }
    while !old.is_char_boundary(old.len() - suffix) || !new.is_char_boundary(new.len() - suffix) {
        suffix -= 1;
    }
    (start, old.len() - suffix, new.len() - suffix)
}

fn coordinate_records(
    units: &[Rc<ResidentCollectionUnit>],
    regions: &[ac::resolve::FunctionRegion],
    origins: &[usize],
    source: &str,
) -> Result<Vec<art::Coordinate>, MnError> {
    if units.len() != regions.len() || units.len() != origins.len() {
        return Err(invalid("source coordinate directory shape"));
    }
    units
        .iter()
        .zip(regions)
        .zip(origins)
        .map(|((unit, region), origin)| {
            let text = source
                .get(region.item.start..region.item.end)
                .ok_or_else(|| invalid("current function coordinate range"))?;
            let mint_end = origin
                .checked_add(text.len())
                .ok_or_else(|| invalid("mint source coordinate overflow"))?;
            Ok(art::Coordinate {
                unit: unit.meta.unit.clone(),
                mint_start: *origin,
                mint_end,
                current_start: region.item.start,
                current_end: region.item.end,
                mint_snapshot: unit.mir.funcs[0].source_snapshot,
                text_hash: crate::cache::content_hash(text.as_bytes()),
            })
        })
        .collect()
}

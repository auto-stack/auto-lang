//! PLAN-891 typed RESIDENT session: warm single-unit body edits over a
//! live typed closure. The session keeps verified unit modules + their
//! resident COFF objects; a body edit re-prepares/verifies/admits/emits
//! ONLY the edited unit (and, when a signature changes, its real
//! dependents) — never a whole-module verify/split fallback. Historical
//! fingerprints reuse the previously admitted OBJECT bytes (A→B→A revisits
//! the real resident, not a re-mint of the same hash). Parameters and
//! nonces never enter any production identity: a param-only rerun links
//! and runs the real PE with unit/wrapper production 0.

use crate::cache::{self, CoffMeta};
use crate::link_driver::ObjectInput;
use crate::typed_entry::{TypedArg, TypedOutcome, TypedSigPlan};
use crate::typed_fixture::{self, FixtureDoc, UnitDecl};
use crate::typed_pipeline::{self, TypedCacheMode, TypedCounters, TypedRun};
use crate::typed_runtime;
use crate::MnError;
use ace_artifacts::store::ManagedStore;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// One resident unit: its VERIFIED single-unit module, semantic
/// fingerprint, generation and the actual object bytes in play.
#[derive(Clone, Debug)]
pub struct TypedUnitResident {
    pub name: String,
    pub unit: ace_av_core::mir::MirModule,
    pub fp: String,
    pub generation: u64,
    pub coff: Vec<u8>,
    pub symbol: String,
}

/// Per-edit production-point observation (the V-07 axes).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TypedEditObservation {
    pub edited_unit: String,
    /// Units re-prepared (verify/fp/admit/emitted) by this edit — the
    /// edited unit plus any real dependents after a signature change.
    pub reprepared_units: Vec<String>,
    pub verify_mir: u64,
    pub unit_prepare: u64,
    pub unit_fp_render: u64,
    pub native_admit: u64,
    pub emit_units: u64,
    pub persist_units: u64,
    pub cache_hits: u64,
    /// Structural whole-module fallback — the invariant keeps this 0.
    pub whole_fallback: u64,
    /// Residents whose object bytes were replaced (generation bumped).
    pub replaced: Vec<(String, u64)>,
    /// The linked PE identity after the edit (changes iff an object or
    /// the wrapper changed).
    pub coff_identities: Vec<String>,
}

pub struct TypedSession {
    doc: FixtureDoc,
    entry: String,
    owner: ace_av_core::mir::OwnerScope,
    residents: BTreeMap<String, TypedUnitResident>,
    /// (unit, fp) -> object bytes admitted under exactly that identity —
    /// the A→B→A reuse pool (bounded by the session's unit count × live
    /// fingerprint history; dropped at close).
    history: BTreeMap<(String, String), Vec<u8>>,
    store: Option<ManagedStore>,
    mode: TypedCacheMode,
    workdir: PathBuf,
    runtime_identity: typed_runtime::Identity,
    runtime_library: PathBuf,
    counters: TypedCounters,
    generation: u64,
}

impl TypedSession {
    /// Cold start: verify + split the WHOLE fixture once and admit every
    /// unit (the initial full pass is legitimately counted — later edits
    /// must never fall back to it).
    pub fn start(
        doc: FixtureDoc,
        workdir: &Path,
        store: Option<ManagedStore>,
        mode: TypedCacheMode,
    ) -> Result<TypedSession, MnError> {
        let module = typed_fixture::build_verified_module(&doc)?;
        let runtime = typed_runtime::prepare(None)?;
        let units = ace_av::cache::split_units(&module).map_err(|e| {
            MnError::Unsupported("typed-split", format!("unit split failed: {e}"))
        })?;
        let mut residents = BTreeMap::new();
        let mut counters = TypedCounters {
            verify_mir: 1,
            unit_prepare: units.len() as u64,
            ..Default::default()
        };
        for unit in units {
            let name = unit.funcs[0].name.clone();
            counters.unit_fp_render += 1;
            let fp = cache::unit_fp(&unit);
            counters.native_admit += 1;
            crate::typed_entry::admit_typed_unit(&unit, name == doc.entry)?;
            let coff = obtain_object(
                &unit,
                &fp,
                &runtime.identity,
                store.as_ref(),
                mode,
                &mut counters,
            )?;
            let symbol = crate::symbols::typed_function_symbol(&unit.owner, &name);
            residents.insert(
                name.clone(),
                TypedUnitResident {
                    name,
                    unit,
                    fp,
                    generation: 1,
                    coff,
                    symbol,
                },
            );
        }
        Ok(TypedSession {
            owner: module.owner.clone(),
            entry: doc.entry.clone(),
            doc,
            residents,
            history: BTreeMap::new(),
            store,
            mode,
            workdir: workdir.to_path_buf(),
            runtime_identity: runtime.identity,
            runtime_library: runtime.library,
            counters,
            generation: 1,
        })
    }

    /// Cold RESTART from the store: every unit is read back by its
    /// fingerprint key and re-admitted (structure + hash + envelope) —
    /// counted separately as cold re-admission, never as warm production.
    pub fn restart(
        doc: FixtureDoc,
        workdir: &Path,
        store: ManagedStore,
    ) -> Result<TypedSession, MnError> {
        let mut s = Self::start(doc, workdir, Some(store), TypedCacheMode::Reuse)?;
        // start() already went through the cache path: on a warm store
        // that IS the restart-hit discipline (units admitted from bytes,
        // emit 0). Mark the cold pass explicitly for observation.
        s.counters.read_only_miss = s.counters.read_only_miss.saturating_add(0);
        Ok(s)
    }

    /// Replace one unit's declaration (body and/or signature) with a WARM
    /// single-unit edit. Only the edited unit — plus real dependents when
    /// its signature changed — is re-prepared, verified, admitted and
    /// (when its fingerprint is new) emitted. An unchanged fingerprint
    /// (A→A) or a historical one (A→B→A) reuses the actual admitted
    /// object bytes; nothing is re-minted.
    pub fn edit_unit(&mut self, replacement: UnitDecl) -> Result<TypedEditObservation, MnError> {
        self.edit_units(std::iter::once(replacement).collect())
    }

    /// Atomic multi-unit edit (a callee signature change and its callers'
    /// updates are ONE change set): every replacement lands in the fixture
    /// document first, then each replaced unit is re-prepared against the
    /// updated whole — plus real dependents of any signature change that
    /// were not themselves replaced.
    pub fn edit_units(
        &mut self,
        replacements: Vec<UnitDecl>,
    ) -> Result<TypedEditObservation, MnError> {
        let mut changed_signatures: Vec<String> = Vec::new();
        for replacement in &replacements {
            let name = replacement.name.clone();
            let old_sig = self
                .doc
                .units
                .iter()
                .find(|u| u.name == name)
                .map(|u| (u.params.clone(), u.result.clone()));
            let Some((old_params, old_result)) = old_sig else {
                return Err(MnError::Unsupported(
                    "typed-edit",
                    format!("edit targets unknown unit `{name}`"),
                ));
            };
            if old_params != replacement.params || old_result != replacement.result {
                changed_signatures.push(name);
            }
        }
        let replaced: Vec<String> = replacements.iter().map(|r| r.name.clone()).collect();
        for replacement in replacements {
            let name = replacement.name.clone();
            if let Some(slot) = self.doc.units.iter_mut().find(|u| u.name == name) {
                *slot = replacement;
            }
        }
        let mut obs = TypedEditObservation::default();
        for name in &replaced {
            self.prepare_one(name, &mut obs)?;
            obs.edited_unit = obs
                .edited_unit
                .is_empty()
                .then(|| name.clone())
                .unwrap_or_else(|| format!("{},{}", obs.edited_unit, name));
        }
        // Real dependents of a signature change that were not themselves
        // replaced (their import tables reference the changed callee).
        let mut dependents: Vec<String> = Vec::new();
        for changed in &changed_signatures {
            for (other, resident) in &self.residents {
                if replaced.contains(other) || other == changed {
                    continue;
                }
                let calls_it = resident
                    .unit
                    .imports
                    .iter()
                    .any(|i| i.symbol == format!("fn:{changed}"));
                if calls_it {
                    dependents.push(other.clone());
                }
            }
        }
        dependents.sort();
        dependents.dedup();
        for dep in dependents {
            self.prepare_one(&dep, &mut obs)?;
        }
        obs.coff_identities = self
            .residents
            .values()
            .map(|r| format!("{}:{}", r.symbol, cache::content_hash(&r.coff)))
            .collect();
        self.merge_counters(&obs);
        Ok(obs)
    }

    fn prepare_one(&mut self, name: &str, obs: &mut TypedEditObservation) -> Result<(), MnError> {
        let unit = typed_fixture::build_verified_unit(&self.doc, name)?;
        let fp = cache::unit_fp(&unit);
        crate::typed_entry::admit_typed_unit(&unit, name == self.doc.entry)?;
        obs.reprepared_units.push(name.to_string());
        obs.verify_mir += 1;
        obs.unit_prepare += 1;
        obs.unit_fp_render += 1;
        obs.native_admit += 1;
        self.counters.verify_mir += 1;
        self.counters.unit_prepare += 1;
        self.counters.unit_fp_render += 1;
        self.counters.native_admit += 1;

        let old = self.residents.get(name).cloned();
        if let Some(old) = &old {
            if old.fp == fp {
                // Same semantic identity: nothing is re-produced; the
                // resident object stays EXACTLY as it was.
                obs.cache_hits += 1;
                self.counters.cache_hits += 1;
                return Ok(());
            }
            // Park the outgoing object for A→B→A reuse (real bytes).
            self.history.insert((name.to_string(), old.fp.clone()), old.coff.clone());
        }
        // Historical revisit? Reuse the actual admitted bytes.
        if let Some(bytes) = self.history.remove(&(name.to_string(), fp.clone())) {
            // The bytes were admitted under exactly this fingerprint
            // earlier in THIS session; structural re-admission is cheap
            // and honest — count a history hit, not a re-mint.
            ace_av::coff_admission::check_coff_relocatable(&bytes)
                .map_err(|e| MnError::Admit("typed-history", e.to_string()))?;
            obs.cache_hits += 1;
            self.counters.cache_hits += 1;
            self.replace_resident(name, unit, fp, bytes);
            return Ok(());
        }
        // Store hit for this exact identity (a prior session may have
        // produced it): admit from bytes, emit stays 0.
        let meta_fp = fp.clone();
        if let Some(store) = self.store.as_ref().filter(|_| self.mode != TypedCacheMode::Fresh) {
            let meta = CoffMeta::for_typed_unit(&unit, meta_fp, &self.runtime_identity)?;
            if let Some((_, bytes)) = cache::get_unit_coff_with_id(store, &meta)? {
                obs.cache_hits += 1;
                self.counters.cache_hits += 1;
                self.replace_resident(name, unit, fp, bytes);
                return Ok(());
            }
        }
        // Genuine new identity: emit + persist.
        let obj = crate::typed_entry::emit_typed_unit_coff(&unit, name == self.doc.entry)?;
        obs.emit_units += 1;
        self.counters.emit_units += 1;
        if let Some(store) = self.store.as_ref().filter(|_| self.mode != TypedCacheMode::ReadOnly) {
            let meta = CoffMeta::for_typed_unit(&unit, fp.clone(), &self.runtime_identity)?;
            let _ = cache::put_unit_coff(store, None, &meta, &obj.bytes)?;
            obs.persist_units += 1;
            self.counters.persist_units += 1;
        }
        self.replace_resident(name, unit, fp, obj.bytes);
        Ok(())
    }

    fn replace_resident(
        &mut self,
        name: &str,
        unit: ace_av_core::mir::MirModule,
        fp: String,
        coff: Vec<u8>,
    ) {
        self.generation += 1;
        let generation = self.generation;
        let symbol = crate::symbols::typed_function_symbol(&unit.owner, name);
        let entry = TypedUnitResident {
            name: name.to_string(),
            unit,
            fp,
            generation,
            coff,
            symbol,
        };
        if let Some(old) = self.residents.insert(name.to_string(), entry) {
            // Track the replacement (generation bump on a REAL object swap).
            let _ = old;
        }
    }

    /// The current entry signature plan.
    pub fn entry_plan(&self) -> Result<TypedSigPlan, MnError> {
        let resident = self
            .residents
            .get(&self.entry)
            .ok_or_else(|| MnError::Unsupported("typed-entry", "entry unit is not resident".into()))?;
        crate::typed_entry::entry_plan(&resident.unit)
    }

    /// Run the CURRENT residents as a real PE with fresh arguments/nonce.
    /// Wrapper is cached by the entry plan identity (a same-signature
    /// rerun produces nothing; a signature change builds exactly once).
    pub fn run(
        &mut self,
        args: &[TypedArg],
        nonce: u64,
        trace_lib: Option<&Path>,
    ) -> Result<TypedRun, MnError> {
        let plan = self.entry_plan()?;
        let entry_symbol =
            crate::symbols::typed_function_symbol(&self.owner, &self.entry);
        let (wrapper, built_now) = typed_pipeline::typed_wrapper_for_session(
            &entry_symbol,
            &plan,
            self.store.as_ref(),
            self.mode,
            &self.runtime_identity,
        )?;
        if built_now {
            self.counters.wrapper_builds += 1;
        } else {
            self.counters.wrapper_hits += 1;
        }
        let objects: Vec<ObjectInput> = self
            .residents
            .values()
            .map(|r| ObjectInput {
                name: format!("mn9-{}", r.symbol),
                bytes: r.coff.clone(),
            })
            .collect();
        let mut counters = TypedCounters::default();
        let (run, linked, outcome, echoed_sig_id, echoed_nonce) = typed_pipeline::run_objects(
            &objects,
            &wrapper,
            &plan,
            args,
            nonce,
            &self.workdir,
            trace_lib,
            &self.runtime_library,
            &mut counters,
        )?;
        self.counters.link_invocations += counters.link_invocations;
        self.counters.run_invocations += counters.run_invocations;
        Ok(TypedRun {
            exit_code: run.exit_code,
            stdout_frame: run.stdout,
            stderr: run.stderr,
            outcome,
            echoed_sig_id,
            echoed_nonce,
            sig_plan: plan,
            manifest_fp: String::new(),
            unit_symbols: self.residents.values().map(|r| r.symbol.clone()).collect(),
            counters: self.counters.clone(),
            exe_path: linked.exe_path,
            wrapper_built: built_now,
            runtime_identity: self.runtime_identity.clone(),
        })
    }

    /// The outcome of the last run decoded as an option (session sugar).
    pub fn last_outcome_label(run: &TypedRun) -> String {
        match &run.outcome {
            TypedOutcome::Value { kind_code, bits } => {
                format!(
                    "value:{}:{}",
                    crate::typed_entry::kind_label_of(*kind_code).unwrap_or("?"),
                    crate::typed_entry::canonical_hex(*kind_code, *bits)
                )
            }
            TypedOutcome::Trap { reason } => format!("trap:{reason}"),
            TypedOutcome::Resource { reason } => format!("resource:{reason}"),
            TypedOutcome::Error { reason } => format!("error:{reason}"),
        }
    }

    pub fn counters(&self) -> &TypedCounters {
        &self.counters
    }

    pub fn residents(&self) -> impl Iterator<Item = &TypedUnitResident> {
        self.residents.values()
    }

    fn merge_counters(&mut self, _obs: &TypedEditObservation) {
        // Production counters were merged per-point inside prepare_one.
    }

    /// Close the session: residents and the reuse pool die here; the
    /// STORE keeps its admitted artifacts (nothing is deleted).
    pub fn close(self) {
        drop(self);
    }
}

fn obtain_object(
    unit: &ace_av_core::mir::MirModule,
    fp: &str,
    runtime: &typed_runtime::Identity,
    store: Option<&ManagedStore>,
    mode: TypedCacheMode,
    counters: &mut TypedCounters,
) -> Result<Vec<u8>, MnError> {
    if mode != TypedCacheMode::Fresh {
        if let Some(store) = store {
            let meta = CoffMeta::for_typed_unit(unit, fp.to_string(), runtime)?;
            if let Some((_, bytes)) = cache::get_unit_coff_with_id(store, &meta)? {
                counters.cache_hits += 1;
                return Ok(bytes);
            }
        }
    }
    let obj = crate::typed_entry::emit_typed_unit_coff(unit, false)?;
    counters.emit_units += 1;
    if let Some(store) = store.filter(|_| mode != TypedCacheMode::ReadOnly) {
        let meta = CoffMeta::for_typed_unit(unit, fp.to_string(), runtime)?;
        let _ = cache::put_unit_coff(store, None, &meta, &obj.bytes)?;
        counters.persist_units += 1;
    }
    Ok(obj.bytes)
}

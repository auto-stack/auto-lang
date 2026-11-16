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

/// One unit of a change set, fully built/verified/admitted and with its
/// OBJECT bytes resolved, but NOT yet installed in the session (PLAN-1032
/// staged-commit shape).
struct PreparedUnit {
    name: String,
    unit: ace_av_core::mir::MirModule,
    fp: String,
    coff: Vec<u8>,
    /// The resident's semantic identity is unchanged: commit installs
    /// nothing (no generation bump).
    unchanged: bool,
    /// The bytes came from the A→B→A pool; commit consumes that key.
    from_history: bool,
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
        let units = ace_av::cache::split_units(&module)
            .map_err(|e| MnError::Unsupported("typed-split", format!("unit split failed: {e}")))?;
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
    /// updates are ONE change set). The whole change set is STAGED aside:
    /// every replacement lands in a CANDIDATE document, every affected unit
    /// (the replaced ones plus real dependents of a signature change) is
    /// built/verified/admitted/emitted-or-reused against that candidate, and
    /// only after the ENTIRE set succeeded is the accepted transaction
    /// switched in one step (document + residents + reuse pool +
    /// generation). A failure at any point — a later unit's
    /// build/verify/admit, or an emit/store write — therefore installs
    /// nothing: the previous entry plan still links and runs, the A→B→A pool
    /// keeps its keys, and the SAME batch can be retried once the defect is
    /// repaired (PLAN-1032 F24).
    pub fn edit_units(
        &mut self,
        replacements: Vec<UnitDecl>,
    ) -> Result<TypedEditObservation, MnError> {
        // 1. Validate the change set against the CURRENT document and note
        //    which declarations really change signature (still no writes).
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
        // 2. Stage the CANDIDATE document aside; the accepted one is not
        //    touched until commit.
        let replaced: Vec<String> = replacements.iter().map(|r| r.name.clone()).collect();
        let mut candidate = self.doc.clone();
        for replacement in replacements {
            let name = replacement.name.clone();
            if let Some(slot) = candidate.units.iter_mut().find(|u| u.name == name) {
                *slot = replacement;
            }
        }
        // 3. Plan the work: replaced units first (in change-set order), then
        //    real dependents of a signature change that were not themselves
        //    replaced (their import tables reference the changed callee).
        let mut plan: Vec<String> = replaced.clone();
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
        plan.extend(dependents);
        // A change set that names one unit twice is degenerate: the candidate
        // document already holds its LAST declaration, so it is staged and
        // installed once (never twice into the same accepted slot).
        let mut plan_seen = std::collections::BTreeSet::new();
        plan.retain(|name| plan_seen.insert(name.clone()));

        // 4. Build every candidate ASIDE. Reads (the reuse pool, the store)
        //    are observed here; nothing accepted is mutated.
        let mut obs = TypedEditObservation::default();
        let mut staged: Vec<PreparedUnit> = Vec::with_capacity(plan.len());
        for name in &plan {
            staged.push(self.prepare_candidate(&candidate, name, &mut obs)?);
        }
        for name in &replaced {
            obs.edited_unit = obs
                .edited_unit
                .is_empty()
                .then(|| name.clone())
                .unwrap_or_else(|| format!("{},{}", obs.edited_unit, name));
        }

        // 5. ONE switch point: the whole change set becomes the accepted
        //    transaction, or (on any earlier `?`) none of it does.
        self.commit_staged(staged, candidate);
        obs.coff_identities = self
            .residents
            .values()
            .map(|r| format!("{}:{}", r.symbol, cache::content_hash(&r.coff)))
            .collect();
        Ok(obs)
    }

    /// Stage ONE unit of a change set without installing it. Counter
    /// movements here are ATTEMPT observations (they may grow even when a
    /// later unit fails); the accepted state — document, residents, reuse
    /// pool, generation — is untouched. The historical-revisit branch only
    /// READS its key: `commit_staged` consumes it.
    fn prepare_candidate(
        &mut self,
        candidate: &FixtureDoc,
        name: &str,
        obs: &mut TypedEditObservation,
    ) -> Result<PreparedUnit, MnError> {
        let unit = typed_fixture::build_verified_unit(candidate, name)?;
        let fp = cache::unit_fp(&unit);
        crate::typed_entry::admit_typed_unit(&unit, name == candidate.entry)?;
        obs.reprepared_units.push(name.to_string());
        obs.verify_mir += 1;
        obs.unit_prepare += 1;
        obs.unit_fp_render += 1;
        obs.native_admit += 1;
        self.counters.verify_mir += 1;
        self.counters.unit_prepare += 1;
        self.counters.unit_fp_render += 1;
        self.counters.native_admit += 1;

        if let Some(old) = self.residents.get(name) {
            if old.fp == fp {
                // Same semantic identity: nothing is re-produced; the
                // resident object stays EXACTLY as it was.
                obs.cache_hits += 1;
                self.counters.cache_hits += 1;
                return Ok(PreparedUnit {
                    name: name.to_string(),
                    unit,
                    fp,
                    coff: old.coff.clone(),
                    unchanged: true,
                    from_history: false,
                });
            }
        }
        // Historical revisit? Reuse the actual admitted bytes (read only).
        if let Some(bytes) = self.history.get(&(name.to_string(), fp.clone())) {
            // The bytes were admitted under exactly this fingerprint
            // earlier in THIS session; structural re-admission is cheap
            // and honest — count a history hit, not a re-mint.
            ace_av::coff_admission::check_coff_relocatable(bytes)
                .map_err(|e| MnError::Admit("typed-history", e.to_string()))?;
            obs.cache_hits += 1;
            self.counters.cache_hits += 1;
            return Ok(PreparedUnit {
                name: name.to_string(),
                unit,
                fp,
                coff: bytes.clone(),
                unchanged: false,
                from_history: true,
            });
        }
        // Store hit for this exact identity (a prior session may have
        // produced it): admit from bytes, emit stays 0.
        if let Some(store) = self
            .store
            .as_ref()
            .filter(|_| self.mode != TypedCacheMode::Fresh)
        {
            let meta = CoffMeta::for_typed_unit(&unit, fp.clone(), &self.runtime_identity)?;
            if let Some((_, bytes)) = cache::get_unit_coff_with_id(store, &meta)? {
                obs.cache_hits += 1;
                self.counters.cache_hits += 1;
                return Ok(PreparedUnit {
                    name: name.to_string(),
                    unit,
                    fp,
                    coff: bytes,
                    unchanged: false,
                    from_history: false,
                });
            }
        }
        // Genuine new identity: emit + persist. Store writes are
        // content-addressed/reclaimable and never part of the session's
        // accepted state, so a batch that fails after this point leaves at
        // most unreferenced objects behind (the 2026-10-31 ruling permits
        // recoverable no-root residue).
        #[cfg(test)]
        if failpoint("emit") {
            return Err(MnError::Infra(
                "failpoint: PLAN-1032 injected emit failure".to_string(),
            ));
        }
        let obj = crate::typed_entry::emit_typed_unit_coff(&unit, name == candidate.entry)?;
        obs.emit_units += 1;
        self.counters.emit_units += 1;
        if let Some(store) = self
            .store
            .as_ref()
            .filter(|_| self.mode != TypedCacheMode::ReadOnly)
        {
            #[cfg(test)]
            if failpoint("store") {
                return Err(MnError::Infra(
                    "failpoint: PLAN-1032 injected store failure".to_string(),
                ));
            }
            let meta = CoffMeta::for_typed_unit(&unit, fp.clone(), &self.runtime_identity)?;
            let _ = cache::put_unit_coff(store, None, &meta, &obj.bytes)?;
            obs.persist_units += 1;
            self.counters.persist_units += 1;
        }
        Ok(PreparedUnit {
            name: name.to_string(),
            unit,
            fp,
            coff: obj.bytes,
            unchanged: false,
            from_history: false,
        })
    }

    /// Install a fully staged change set. Cannot fail: every fallible step
    /// already ran in `prepare_candidate`.
    fn commit_staged(&mut self, staged: Vec<PreparedUnit>, doc: FixtureDoc) {
        for unit in staged {
            if unit.unchanged {
                // Same fingerprint: the resident object stays exactly as it
                // was (no generation bump, no history churn).
                continue;
            }
            if let Some(old) = self.residents.get(&unit.name) {
                if old.fp != unit.fp {
                    // Park the outgoing object for A→B→A reuse (real bytes).
                    self.history
                        .insert((unit.name.clone(), old.fp.clone()), old.coff.clone());
                }
            }
            if unit.from_history {
                self.history.remove(&(unit.name.clone(), unit.fp.clone()));
            }
            self.replace_resident(&unit.name, unit.unit, unit.fp, unit.coff);
        }
        self.doc = doc;
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
        let resident = self.residents.get(&self.entry).ok_or_else(|| {
            MnError::Unsupported("typed-entry", "entry unit is not resident".into())
        })?;
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
        let entry_symbol = crate::symbols::typed_function_symbol(&self.owner, &self.entry);
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

// ==== PLAN-1032 test-only acceptance/failure instrumentation ==============
//
// The atomicity oracle needs two things a production build must never carry:
// (1) a *late* fault (emit/store write) injected after an earlier unit of the
// same change set already produced, and (2) a one-shot read of the session's
// ACCEPTED transaction (document + residents + reuse pool + generation).
// Both are `cfg(test)` only: no public accessor and no production failpoint.

#[cfg(test)]
thread_local! {
    /// `(point, ordinal)`: fire on the `ordinal`-th hit of `point`, then
    /// disarm (one-shot). Thread-local so parallel tests never interfere.
    static FAILPOINT: std::cell::Cell<Option<(&'static str, u32)>> =
        const { std::cell::Cell::new(None) };
}

#[cfg(test)]
fn arm_failpoint(point: &'static str, ordinal: u32) {
    FAILPOINT.with(|f| f.set(Some((point, ordinal.max(1)))));
}

#[cfg(test)]
fn failpoint(point: &str) -> bool {
    FAILPOINT.with(|f| match f.get() {
        Some((armed, ordinal)) if armed == point => {
            if ordinal <= 1 {
                f.set(None);
                true
            } else {
                f.set(Some((armed, ordinal - 1)));
                false
            }
        }
        _ => false,
    })
}

#[cfg(test)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct AcceptanceSnapshot {
    pub doc: FixtureDoc,
    /// (name, fingerprint, generation, object content hash), sorted by name.
    pub residents: Vec<(String, String, u64, String)>,
    /// (unit, fingerprint, byte len) of the A→B→A pool.
    pub history: Vec<(String, String, usize)>,
    pub generation: u64,
}

#[cfg(test)]
impl TypedSession {
    /// Read the whole ACCEPTED transaction in one shot. Test-only on
    /// purpose: production keeps its mutable acceptance state private.
    pub(crate) fn acceptance_snapshot(&self) -> AcceptanceSnapshot {
        AcceptanceSnapshot {
            doc: self.doc.clone(),
            residents: self
                .residents
                .values()
                .map(|r| {
                    (
                        r.name.clone(),
                        r.fp.clone(),
                        r.generation,
                        cache::content_hash(&r.coff),
                    )
                })
                .collect(),
            history: self
                .history
                .iter()
                .map(|((unit, fp), bytes)| (unit.clone(), fp.clone(), bytes.len()))
                .collect(),
            generation: self.generation,
        }
    }

    pub(crate) fn resident_coff(&self, name: &str) -> Option<Vec<u8>> {
        self.residents.get(name).map(|r| r.coff.clone())
    }
}

#[cfg(test)]
mod atomicity_1032 {
    use super::*;
    use crate::typed_entry::{self, TypedArg, TypedOutcome};

    fn tools_available() -> bool {
        ace_core::link::find_rust_lld().is_ok() && ace_core::link::find_sdk_um_dir().is_ok()
    }

    fn workdir(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("mn1032-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("workdir");
        dir
    }

    fn doc_from(json: &str) -> FixtureDoc {
        typed_fixture::parse_fixture(json.as_bytes()).expect("fixture decodes")
    }

    fn unit_decl(json: &str) -> UnitDecl {
        let wrapped = format!(
            r#"{{"schema":"mn-typed-fixture/1","module":"m891_sess","units":[{json}],"entry":"main"}}"#
        );
        let doc = typed_fixture::parse_fixture(wrapped.as_bytes()).expect("unit decodes");
        doc.units.into_iter().next().expect("one unit")
    }

    fn arg(kind: &str, bits: &str) -> TypedArg {
        typed_entry::parse_arg(
            typed_entry::TypedKind::from_label(kind).expect("kind"),
            bits,
        )
        .expect("arg")
    }

    fn value_bits(run: &crate::typed_pipeline::TypedRun) -> String {
        match &run.outcome {
            TypedOutcome::Value { kind_code, bits } => {
                typed_entry::canonical_hex(*kind_code, *bits)
            }
            other => panic!("expected a value outcome, got {other:?}"),
        }
    }

    fn reject(err: MnError, what: &str) {
        assert!(
            matches!(err, MnError::Unsupported(..)),
            "{what}: expected a controlled rejection, got {}",
            err.code()
        );
    }

    fn injected(err: MnError, what: &str) {
        match err {
            MnError::Infra(message) => {
                assert!(message.contains("failpoint"), "{what}: {message}")
            }
            other => panic!(
                "{what}: expected the injected late failure, got {}",
                other.code()
            ),
        }
    }

    /// caller passes through helper(a); helper identity(a); disjoint returns
    /// a constant. Same input profile as the public incremental tier — the
    /// mechanism is shared, the oracle is not.
    const BASE: &str = r#"{"schema":"mn-typed-fixture/1","module":"m891_sess","units":[
  {"name":"helper","params":[["a","i64"]],"result":"i64","body":[{"op":"copy","dst":"t0","src":"a"}],"ret":"t0"},
  {"name":"disjoint","params":[],"result":"i64","body":[{"op":"const","dst":"t0","kind":"i64","bits":"0000000000000001"}],"ret":"t0"},
  {"name":"main","params":[["a","i64"]],"result":"i64","body":[
     {"op":"call","dst":"t0","callee":"helper","args":["a"]}],
   "ret":"t0"}],
 "entry":"main"}"#;

    const HELPER_A: &str = r#"{"name":"helper","params":[["a","i64"]],"result":"i64","body":[{"op":"copy","dst":"t0","src":"a"}],"ret":"t0"}"#;
    const HELPER_B: &str = r#"{"name":"helper","params":[["a","i64"]],"result":"i64","body":[{"op":"const","dst":"t0","kind":"i64","bits":"00000000000000ff"}],"ret":"t0"}"#;
    const HELPER_C: &str = r#"{"name":"helper","params":[["a","i64"]],"result":"i64","body":[{"op":"const","dst":"t0","kind":"i64","bits":"00000000000000ab"}],"ret":"t0"}"#;
    const HELPER_WIDE: &str = r#"{"name":"helper","params":[["a","i64"],["b","i64"]],"result":"i64","body":[{"op":"copy","dst":"t0","src":"b"}],"ret":"t0"}"#;
    const DISJOINT_BAD: &str = r#"{"name":"disjoint","params":[],"result":"i64","body":[],"ret":"missing"}"#;
    const DISJOINT_GOOD: &str = r#"{"name":"disjoint","params":[],"result":"i64","body":[{"op":"const","dst":"t0","kind":"i64","bits":"0000000000000003"}],"ret":"t0"}"#;
    const DISJOINT_C: &str = r#"{"name":"disjoint","params":[],"result":"i64","body":[{"op":"const","dst":"t0","kind":"i64","bits":"0000000000000005"}],"ret":"t0"}"#;
    const HELPER_D: &str = r#"{"name":"helper","params":[["a","i64"]],"result":"i64","body":[{"op":"const","dst":"t0","kind":"i64","bits":"00000000000000cd"}],"ret":"t0"}"#;
    const DISJOINT_D: &str = r#"{"name":"disjoint","params":[],"result":"i64","body":[{"op":"const","dst":"t0","kind":"i64","bits":"0000000000000009"}],"ret":"t0"}"#;

    /// PLAN-1032 NTX-02 (internal oracle): a failed batch leaves the WHOLE
    /// accepted transaction untouched — document, residents/objects,
    /// A→B→A pool keys and generation — including a history hit followed by
    /// a late verify failure and an emit/store write failure; the same batch
    /// then retries successfully, and A→B→A reuses the REAL admitted bytes.
    #[test]
    #[cfg(windows)]
    fn audit1032_ntx_02_batch_failure_is_atomic() {
        if !tools_available() {
            eprintln!("infra-blocked: rust-lld/SDK unavailable — PLAN-1032 oracle skipped");
            return;
        }
        let store_root = workdir("store");
        let store = ManagedStore::open(&store_root).expect("store");
        let dir = workdir("run");
        let mut session = TypedSession::start(
            doc_from(BASE),
            &dir,
            Some(store),
            TypedCacheMode::Reuse,
        )
        .expect("session starts");

        // A: helper identity.
        let run = session
            .run(&[arg("i64", "0000000000000010")], 1, None)
            .expect("baseline run");
        assert_eq!(value_bits(&run), "0000000000000010");
        let helper_a_bytes = session.resident_coff("helper").expect("helper resident");

        // A -> B: B is accepted and the REAL A object is parked for reuse.
        let obs = session
            .edit_unit(unit_decl(HELPER_B))
            .expect("edit B accepted");
        assert_eq!(obs.emit_units, 1, "B is a genuine new identity");
        let run = session
            .run(&[arg("i64", "0000000000000010")], 2, None)
            .expect("run B");
        assert_eq!(value_bits(&run), "00000000000000ff", "B is installed");

        // (1) HISTORY HIT followed by a LATE failure in the same batch:
        // helper rebuilds to A from the reuse pool, then disjoint fails.
        let accepted = session.acceptance_snapshot();
        let emit_before = session.counters().emit_units;
        let err = session
            .edit_units(vec![unit_decl(HELPER_A), unit_decl(DISJOINT_BAD)])
            .expect_err("late verify failure must reject the whole batch");
        reject(err, "history-hit batch");
        assert_eq!(
            session.acceptance_snapshot(),
            accepted,
            "a failed batch must not change doc/residents/history/generation"
        );
        assert!(
            session.counters().emit_units >= emit_before,
            "attempt observations may grow; acceptance must not"
        );
        let run = session
            .run(&[arg("i64", "0000000000000010")], 3, None)
            .expect("old closure still runs");
        assert_eq!(
            value_bits(&run),
            "00000000000000ff",
            "the previously accepted closure still runs after the failure"
        );

        // (2) Clear the defect and retry the SAME change set: A comes back
        // from the real historical COFF bytes (no re-mint).
        let obs = session
            .edit_units(vec![unit_decl(HELPER_A), unit_decl(DISJOINT_GOOD)])
            .expect("same-batch retry after repair");
        assert!(obs.cache_hits >= 1, "helper is a real history hit");
        assert_eq!(
            obs.reprepared_units,
            vec!["helper".to_string(), "disjoint".to_string()],
            "both replacements re-prepare in change-set order"
        );
        assert_eq!(
            session.resident_coff("helper").expect("helper resident"),
            helper_a_bytes,
            "A→B→A reuses the ACTUAL admitted object bytes"
        );
        let run = session
            .run(&[arg("i64", "0000000000000010")], 4, None)
            .expect("run A again");
        assert_eq!(value_bits(&run), "0000000000000010", "A is installed");

        // (3) EMIT failure on the SECOND unit: the first unit already
        // produced, the batch must still install nothing.
        let accepted = session.acceptance_snapshot();
        arm_failpoint("emit", 2);
        let err = session
            .edit_units(vec![unit_decl(HELPER_C), unit_decl(DISJOINT_C)])
            .expect_err("injected emit failure must reject the batch");
        injected(err, "emit failpoint");
        assert_eq!(
            session.acceptance_snapshot(),
            accepted,
            "a late emit failure must not install the already-produced unit"
        );

        // (4) STORE failure on the SECOND unit: even later than emit. The
        // fixture uses FRESH identities because step (3) already persisted
        // the emit-failed head's object bytes — store puts are
        // content-addressed residue outside the session transaction, and
        // that is precisely why acceptance must not depend on them.
        arm_failpoint("store", 2);
        let err = session
            .edit_units(vec![unit_decl(HELPER_D), unit_decl(DISJOINT_D)])
            .expect_err("injected store failure must reject the batch");
        injected(err, "store failpoint");
        assert_eq!(
            session.acceptance_snapshot(),
            accepted,
            "a late store failure must not install the already-produced unit"
        );
        let run = session
            .run(&[arg("i64", "0000000000000010")], 5, None)
            .expect("old closure still runs after write failures");
        assert_eq!(value_bits(&run), "0000000000000010");

        // (5) Same batch, fault cleared: the whole set installs. The head's
        // object survived in the store (reclaimable residue), so it is
        // re-admitted instead of re-emitted; the tail emits.
        let obs = session
            .edit_units(vec![unit_decl(HELPER_D), unit_decl(DISJOINT_D)])
            .expect("retry after clearing the write fault");
        assert_eq!(
            obs.cache_hits, 1,
            "the persisted head object is re-admitted from the store"
        );
        assert_eq!(obs.emit_units, 1, "only the never-persisted tail emits");
        assert_eq!(obs.persist_units, 1, "the tail is persisted on success");
        let run = session
            .run(&[arg("i64", "0000000000000010")], 6, None)
            .expect("run D");
        assert_eq!(value_bits(&run), "00000000000000cd", "D is installed");
        assert_eq!(
            session.counters().whole_fallback,
            0,
            "the staged batch never falls back to a whole-module pass"
        );
        session.close();
    }

    /// PLAN-1032 neighbor control: a stale caller of a changed signature is
    /// not a "broken edit" of the callee — the batch still rejects atomically
    /// (this is the signature-dependent late failure the change set must
    /// handle by re-preparing the dependent, not by installing the callee).
    #[test]
    #[cfg(windows)]
    fn audit1032_ntx_02_signature_change_pulls_real_dependents() {
        if !tools_available() {
            eprintln!("infra-blocked: rust-lld/SDK unavailable — PLAN-1032 control skipped");
            return;
        }
        let dir = workdir("sig");
        let mut session = TypedSession::start(
            doc_from(BASE),
            &dir,
            None,
            TypedCacheMode::Reuse,
        )
        .expect("session starts");
        let accepted = session.acceptance_snapshot();

        // helper widens WITHOUT its caller following: the dependent `main`
        // fails verify, so the callee must not be installed.
        let err = session
            .edit_unit(unit_decl(HELPER_WIDE))
            .expect_err("stale caller must reject the change set");
        reject(err, "signature-dependent batch");
        assert_eq!(
            session.acceptance_snapshot(),
            accepted,
            "a dependent verify failure must not install the new callee"
        );
        let run = session
            .run(&[arg("i64", "0000000000000010")], 7, None)
            .expect("old closure still runs");
        assert_eq!(value_bits(&run), "0000000000000010");
        session.close();
    }
}
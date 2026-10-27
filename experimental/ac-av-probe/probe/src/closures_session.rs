//! PLAN-822 F-822-R1-02: the REAL closures update consumer.
//!
//! A session over the D828 function-code/snapshot seam: units enter ONLY
//! through the real gates (verify → encode → abc-admit → unit-admit,
//! optional store persist); a revision rebuilds the SUBMITTED functions
//! through those gates while the SESSION carries the rest (the caller
//! never hands it an Arc); link resolves `fn:<name>` imports with
//! signature equivalence; the executed face reports semantics + cleanup;
//! the production counters prove un-submitted units produced nothing; and
//! the structural gate is a PRODUCTION predicate over the content-
//! fingerprint domain — re-submitting an identical module is the
//! forced-whole shape: the fresh object under the same fingerprint reads
//! degraded-unchanged (non-zero → gate FAILS), an honest revision passes.

use ac_artifact_probe::codec::ChunkKind;
use ac_artifact_probe::store::{ManagedStore, PutRequest};
use auto_av_core::abc::encode_abc;
use auto_av_core::abc_verify::verify_abc_bytes;
use auto_av_core::av::{execute_program, CancelFlag, ExecLimits};
use auto_av_core::contract::ExecOutcome;
use auto_av_core::function_code::FunctionCode;
use auto_av_core::mir::MirModule;
use auto_av_core::mir_verify::verify_mir;
use auto_av_core::program_snapshot::ProgramSnapshot;
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use crate::cache::{envelope_wrap, EnvelopeIdentity, EnvelopeKind};
use ac_artifact_probe::identity::QueryKey;

/// The production counters of one revision: how many units went through
/// each REAL production stage. An un-submitted unit produces NOTHING
/// (V-13 sub-case 2: 未改生产阶段/IO=0).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ProductionCounters {
    pub verified: usize,
    pub encoded: usize,
    pub admitted: usize,
    pub store_writes: usize,
}

/// The production structural gate over one revision. A unit is
/// UNCHANGED when its admitted wire's content fingerprint did not
/// change; it is DEGRADED when that unchanged content arrived in a
/// DIFFERENT object anyway (the forced-whole-recompute signature — "only
/// the hash kept, the object/generation swapped"). An honest revision
/// passes; a forced rebuild fails with a non-zero count.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct GateReport {
    pub rebuilt_units: usize,
    pub unchanged_degraded: usize,
}

impl GateReport {
    pub fn pass(&self) -> bool {
        self.unchanged_degraded == 0
    }
}

/// One revision's observed outcome.
#[derive(Clone, Debug)]
pub struct ApplyOutcome {
    pub revision: u64,
    /// The executed result of the linked snapshot (i32 face of the entry).
    pub value: Option<i32>,
    /// The post-release arena ledger axes.
    pub live_after_release: u64,
    pub allocs: usize,
    pub frees: usize,
    pub gate: GateReport,
    pub production: ProductionCounters,
}

// The typed consumer retains a bounded history of admitted units. Canonical
// bytes and owner both participate in a hit; a hash alone is never authority.
const HISTORY_PER_UNIT: usize = 32;
const HISTORY_HOT_LIMIT: usize = 8 * 1024 * 1024;

#[derive(Clone)]
struct HistoryEntry {
    wire: Arc<[u8]>,
    code: Arc<FunctionCode>,
    fingerprint: String,
    artifact: Option<String>,
}

static SESSION_SEQUENCE: AtomicU64 = AtomicU64::new(1);

// Staged artifacts are protected by OS-backed leases, not persistent
// roots: a killed process cannot leave an unpublished candidate pinned.
struct StagedStore {
    store: ManagedStore,
    leases: RefCell<Vec<ac_artifact_probe::store::Lease>>,
}
impl StagedStore {
    fn protect(&self, id: &str) -> Result<(), String> {
        let lease = self
            .store
            .acquire_lease("plan823-candidate", 300, Some(id))
            .map_err(|e| format!("protect candidate {id}: {e}"))?;
        self.leases.borrow_mut().push(lease);
        Ok(())
    }
}

fn history_charge(wire: &[u8], code: &FunctionCode) -> usize {
    wire.len()
        .saturating_add(code.unit().module().heap_estimate())
        .saturating_add(std::mem::size_of::<HistoryEntry>())
        .saturating_add(code.name().len())
        .saturating_add(128) // map, deque and fingerprint metadata
}

pub struct ClosuresUpdateSession {
    store_root: Option<PathBuf>,
    store_project: String,
    current_artifacts: BTreeMap<String, String>,
    owned_roots: BTreeSet<String>,
    closed: bool,
    units: BTreeMap<String, Arc<FunctionCode>>,
    /// Per unit: (code object, content fingerprint of the admitted wire).
    identity: BTreeMap<String, (Arc<FunctionCode>, String)>,
    revision: u64,
    current_wire: BTreeMap<String, Arc<[u8]>>,
    history: BTreeMap<String, VecDeque<HistoryEntry>>,
    history_hot_bytes: usize,
    counters: ProductionCounters,
    entry: String,
    /// F822-R17-01: the CURRENT host grants. A submitted module's
    /// declared capabilities (the input's REQUIRED capabilities) are
    /// admitted only against this set — required and granted stay
    /// SEPARATE facts (the F798-R3-02 restore pattern, applied at the
    /// revision seam): an ungranted capability rejects the revision
    /// BEFORE any production stage or installation.
    host_grants: Vec<String>,
}

impl Drop for ClosuresUpdateSession {
    fn drop(&mut self) {
        // Callers that need to observe cleanup failures should call close
        // explicitly; Drop makes a best-effort fallback for abandoned sessions.
        let _ = self.close();
    }
}

impl ClosuresUpdateSession {
    pub fn new(store_root: Option<&Path>, entry: &str) -> Self {
        ClosuresUpdateSession {
            store_root: store_root.map(PathBuf::from),
            store_project: format!(
                "closures-update-session-{}-{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_nanos(),
                SESSION_SEQUENCE.fetch_add(1, Ordering::Relaxed)
            ),
            current_artifacts: BTreeMap::new(),
            owned_roots: BTreeSet::new(),
            closed: false,
            units: BTreeMap::new(),
            identity: BTreeMap::new(),
            revision: 0,
            current_wire: BTreeMap::new(),
            history: BTreeMap::new(),
            history_hot_bytes: 0,
            counters: ProductionCounters::default(),
            entry: entry.to_string(),
            host_grants: Vec::new(),
        }
    }

    /// Project whose soft roots protect this session's published units.
    pub fn store_project(&self) -> &str {
        &self.store_project
    }

    /// Release this session's roots and resident references. External
    /// snapshots own their code independently and can continue to run.
    pub fn close(&mut self) -> Result<(), String> {
        if self.closed {
            return Ok(());
        }
        if let Some(root) = &self.store_root {
            let store =
                ManagedStore::open(root).map_err(|e| format!("open store for close: {e}"))?;
            store
                .remove_project_roots(&self.store_project)
                .map_err(|e| format!("remove session roots: {e}"))?;
        }
        self.units.clear();
        self.identity.clear();
        self.current_wire.clear();
        self.history.clear();
        self.current_artifacts.clear();
        self.owned_roots.clear();
        self.history_hot_bytes = 0;
        self.closed = true;
        Ok(())
    }

    /// F822-R17-01: set the host's CURRENT capability grants (empty by
    /// default — a capability-requiring unit is never admitted unless the
    /// host explicitly grants it).
    pub fn set_host_grants(&mut self, grants: Vec<String>) {
        self.host_grants = grants;
    }

    /// F822-R18-01: the production counters of the LATEST revision
    /// ATTEMPT — including a REJECTED one (the authorization preflight
    /// performs none; a shortfall leaves every stage at zero, observable
    /// here).
    pub fn production_counters(&self) -> ProductionCounters {
        self.counters
    }

    /// The linked snapshot of the current units (executable as-is).
    pub fn snapshot(&self) -> Result<ProgramSnapshot, String> {
        if self.closed {
            return Err("session is closed".into());
        }
        let code = self.units.values().cloned().collect();
        ProgramSnapshot::link(code, &self.entry, self.revision)
    }

    /// The code object the session currently holds for `name` (the
    /// identity observation seam).
    pub fn unit(&self, name: &str) -> Option<&Arc<FunctionCode>> {
        self.units.get(name)
    }

    /// Produce ONE unit through the real gates (counted). Returns the
    /// code object and the content fingerprint of its admitted wire.
    fn produce_unit(
        &mut self,
        m: &MirModule,
        staged: Option<&StagedStore>,
    ) -> Result<(Arc<FunctionCode>, String, Arc<[u8]>, Option<String>), String> {
        let name = m
            .funcs
            .first()
            .ok_or("submitted unit has no function")?
            .name
            .clone();
        verify_mir(m).map_err(|e| format!("verify {name}: {e:?}"))?;
        self.counters.verified += 1;
        let wire = encode_abc(m).map_err(|e| format!("encode {name}: {}", e.code()))?;
        self.counters.encoded += 1;
        let flat: Arc<[u8]> = wire.to_flat().into();
        // A repeated CURRENT submission remains the deliberate degradation
        // control. Only a genuine return to older content can hit history.
        if self
            .current_wire
            .get(&name)
            .is_some_and(|current| current != &flat)
        {
            if let Some(entry) = self.history.get(&name).and_then(|entries| {
                entries
                    .iter()
                    .find(|entry| entry.wire == flat && entry.code.unit().module().owner == m.owner)
            }) {
                return Ok((
                    entry.code.clone(),
                    entry.fingerprint.clone(),
                    flat,
                    entry.artifact.clone(),
                ));
            }
        }
        let loaded =
            verify_abc_bytes(&flat, &m.owner).map_err(|e| format!("abc-admit {name}: {e:?}"))?;
        let fc = FunctionCode::admit_unit(loaded, self.revision + 1)
            .map_err(|e| format!("unit-admit {name}: {e}"))?;
        self.counters.admitted += 1;
        let mut artifact = None;
        if let Some(staged) = staged {
            let store = &staged.store;
            let owner = &m.owner.module;
            let env = envelope_wrap(
                &wire.to_flat(),
                &EnvelopeIdentity {
                    kind: EnvelopeKind::AbcUnit,
                    producer: auto_av_core::contract::CLOSURES_PRODUCER_ID,
                    schema: auto_av_core::contract::ABC_CLOSURES_SCHEMA,
                    owner_module: owner,
                    stage: 2,
                    mir_sem: auto_av_core::contract::MIR_CLOSURES_SEMANTICS.0,
                    abc_sem: auto_av_core::contract::ABC_CLOSURES_SEMANTICS.0,
                    av_sem: auto_av_core::contract::AV_CLOSURES_SEMANTICS.0,
                    snapshot: m.funcs[0].source_snapshot,
                    unit: &name,
                    body_fp: &format!("{:016x}", auto_av_core::contract::fnv1a64(&wire.to_flat())),
                },
            );
            let key = QueryKey::new(
                &crate::closures_fixture::CLOSURES_ABC_LAYER,
                vec![
                    (
                        "producer".into(),
                        auto_av_core::contract::CLOSURES_PRODUCER_ID.into(),
                    ),
                    ("owner-module".into(), owner.clone()),
                    ("entry".into(), name.clone()),
                    (
                        "profile".into(),
                        auto_av_core::contract::AV_CLOSURES_PROFILE.into(),
                    ),
                    (
                        "source-snapshot".into(),
                        format!("{:016x}", m.funcs[0].source_snapshot),
                    ),
                    ("unit-binding".into(), name.clone()),
                    // Distinct body revisions must never contend for one
                    // immutable store key. Keep the hash domain separate
                    // from other artifacts; the envelope still binds its
                    // independently verified unit/body identity.
                    (
                        "unit-content".into(),
                        ac_artifact_probe::identity::domain_hash("av-closures-unit/v1", &[&flat]),
                    ),
                    (
                        "abc-codec".into(),
                        auto_av_core::contract::ABC_CLOSURES_VERSION.to_string(),
                    ),
                    (
                        "mir-semantics".into(),
                        auto_av_core::contract::MIR_CLOSURES_SEMANTICS.0.to_string(),
                    ),
                    (
                        "abc-semantics".into(),
                        auto_av_core::contract::ABC_CLOSURES_SEMANTICS.0.to_string(),
                    ),
                    (
                        "av-semantics".into(),
                        auto_av_core::contract::AV_CLOSURES_SEMANTICS.0.to_string(),
                    ),
                ],
            );
            let id = store
                .put_artifact(PutRequest {
                    layer: crate::closures_fixture::CLOSURES_ABC_LAYER.id,
                    key,
                    kind: ChunkKind::LayoutData,
                    payload: env,
                    edges: vec![],
                    project: None,
                })
                .map_err(|e| format!("persist {name}: {e}"))?;
            // If concurrent GC wins the put-to-lease race, protection
            // returns a controlled error and the candidate is not installed.
            staged.protect(&id)?;
            artifact = Some(id);
            self.counters.store_writes += 1;
        }
        let fp = format!("{:016x}", auto_av_core::contract::fnv1a64(&wire.to_flat()));
        Ok((Arc::new(fc), fp, flat, artifact))
    }

    /// Apply one revision: rebuild the SUBMITTED functions through the
    /// real gates, carry the un-submitted units from the session, link,
    /// execute, and report the production gate. Re-submitting a module
    /// whose content equals an installed unit is the FORCED-WHOLE shape —
    /// the fresh object under the same fingerprint reads
    /// degraded-unchanged (non-zero → gate FAILS). A link failure (a real
    /// dependency change) is an Err; the session keeps its prior state.
    pub fn apply_revision(
        &mut self,
        submitted: &[(&str, MirModule)],
    ) -> Result<ApplyOutcome, String> {
        if self.closed {
            return Err("session is closed".into());
        }
        let staged = match &self.store_root {
            Some(root) => Some(StagedStore {
                store: ManagedStore::open(root).map_err(|e| format!("open store: {e}"))?,
                leases: RefCell::new(Vec::new()),
            }),
            None => None,
        };
        self.apply_revision_inner(submitted, staged.as_ref())
    }

    fn apply_revision_inner(
        &mut self,
        submitted: &[(&str, MirModule)],
        staged: Option<&StagedStore>,
    ) -> Result<ApplyOutcome, String> {
        let prev_units = self.units.clone();
        self.counters = ProductionCounters::default();
        let mut next: BTreeMap<String, (Arc<FunctionCode>, String, Arc<[u8]>, Option<String>)> =
            BTreeMap::new();
        let mut order: Vec<String> = self.units.keys().cloned().collect();
        for (name, _) in submitted {
            if !order.iter().any(|n| n == name) {
                order.push(name.to_string());
            }
        }
        // F822-R18-01: authorize the COMPLETE candidate closure against
        // the CURRENT host grants BEFORE any production stage. A carried
        // unit's code is reused, not re-derived — its required
        // capabilities still bind every NEW revision that would execute
        // it; a submitted replacement supplies its OWN requirements (it
        // does not inherit the carried unit's). This is a decision about
        // the new candidate only — snapshots already published under an
        // earlier, live grant fact keep executing (authorization binds at
        // revision time, not retroactively).
        for name in &order {
            let required: &[String] = match submitted.iter().find(|(n, _)| *n == name.as_str()) {
                Some((_, m)) => &m.capabilities,
                None => {
                    &self
                        .units
                        .get(name)
                        .expect("carried name came from the session")
                        .unit()
                        .module()
                        .capabilities
                }
            };
            for c in required {
                if !self.host_grants.iter().any(|g| g == c) {
                    return Err(format!(
                        "unit `{name}` requires capability `{c}` not granted by the current host grants"
                    ));
                }
            }
        }
        for name in &order {
            match submitted.iter().find(|(n, _)| *n == name.as_str()) {
                Some((_, m)) => {
                    if m.funcs.first().map(|f| f.name.as_str()) != Some(name.as_str()) {
                        return Err(format!("submitted unit name does not match `{name}`"));
                    }
                    let (fc, fp, wire, artifact) = self.produce_unit(m, staged)?;
                    next.insert(name.clone(), (fc, fp, wire, artifact));
                }
                None => {
                    let fc = self
                        .units
                        .get(name)
                        .cloned()
                        .ok_or_else(|| format!("unknown unit `{name}`"))?;
                    let fp = self
                        .identity
                        .get(name)
                        .map(|(_, fp)| fp.clone())
                        .unwrap_or_default();
                    let wire = self
                        .current_wire
                        .get(name)
                        .cloned()
                        .ok_or("missing current wire")?;
                    let artifact = self.current_artifacts.get(name).cloned();
                    next.insert(name.clone(), (fc, fp, wire, artifact));
                }
            }
        }
        // The production structural gate over the fingerprint domain: an
        // unchanged unit (same fingerprint) must keep its object.
        let mut gate = GateReport {
            rebuilt_units: self.counters.admitted,
            unchanged_degraded: 0,
        };
        for (name, (fc, _, wire, _)) in &next {
            if self.current_wire.get(name).is_some_and(|old| old == wire) {
                let same_object = prev_units
                    .get(name)
                    .map(|old| Arc::ptr_eq(old, fc))
                    .unwrap_or(false);
                if !same_object {
                    gate.unchanged_degraded += 1;
                }
            }
        }
        let current_hot = next.values().fold(0usize, |sum, (code, _, wire, _)| {
            sum.saturating_add(history_charge(wire, code))
        });
        if current_hot > HISTORY_HOT_LIMIT {
            return Err(format!(
                "current code exceeds hot budget: {current_hot} > {HISTORY_HOT_LIMIT}"
            ));
        }
        // F822-R2-01: BUILD-ASIDE + ATOMIC COMMIT (resident-function-install
        // transaction): the candidate set links and executes BEFORE any
        // session state changes; a rejected candidate (link refusal or
        // execution failure) leaves the prior revision/units/identity
        // untouched and the prior snapshot stays usable.
        let candidate_units = next
            .iter()
            .map(|(k, (fc, _, _, _))| (k.clone(), fc.clone()))
            .collect::<Vec<_>>();
        let candidate_rev = self.revision + 1;
        let snapshot = ProgramSnapshot::link(
            candidate_units.iter().map(|(_, fc)| fc.clone()).collect(),
            &self.entry,
            candidate_rev,
        )?;
        let (out, trace) =
            execute_program(&snapshot, &ExecLimits::default(), &CancelFlag::new(false));
        let arena = trace.arena.clone().expect("execute attaches the ledger");
        let value = match out {
            ExecOutcome::Success { value, .. } => match value {
                auto_av_core::contract::Value::I32(v) => {
                    // Host release before the ledger read (live=0 ordering).
                    drop(value);
                    Some(v)
                }
                other => {
                    return Err(format!(
                        "entry returned {} (admission violation)",
                        other.type_name()
                    ))
                }
            },
            other => return Err(format!("revision did not execute: {other:?}")),
        };
        // COMMIT: only a fully verified+executed candidate installs.
        // History changes only after link and execution have succeeded.
        // The history shares the existing 8 MiB hot budget with current
        // code. Charge both wire and the decoded module's established heap
        // estimate; an entry too large for the remaining budget is not cached.
        let history_limit = HISTORY_HOT_LIMIT - current_hot;
        let mut history = self.history.clone();
        let mut history_hot_bytes = self.history_hot_bytes;
        for (name, (code, _, wire, _)) in &next {
            if let Some(entries) = history.get_mut(name) {
                if let Some(pos) = entries
                    .iter()
                    .position(|entry| entry.wire == *wire && Arc::ptr_eq(&entry.code, code))
                {
                    let entry = entries.remove(pos).expect("position exists");
                    history_hot_bytes -= history_charge(&entry.wire, &entry.code);
                }
            }
            if let (Some(old), Some(old_wire), Some((_, old_fp))) = (
                prev_units.get(name),
                self.current_wire.get(name),
                self.identity.get(name),
            ) {
                let old_charge = history_charge(old_wire, old);
                if !Arc::ptr_eq(old, code) && old_charge <= history_limit {
                    let entries = history.entry(name.clone()).or_default();
                    while entries.len() >= HISTORY_PER_UNIT {
                        let evicted = entries.pop_front().expect("nonempty");
                        history_hot_bytes -= history_charge(&evicted.wire, &evicted.code);
                    }
                    history_hot_bytes = history_hot_bytes.saturating_add(old_charge);
                    entries.push_back(HistoryEntry {
                        wire: old_wire.clone(),
                        code: old.clone(),
                        fingerprint: old_fp.clone(),
                        artifact: self.current_artifacts.get(name).cloned(),
                    });
                }
            }
        }
        while history_hot_bytes > history_limit {
            let name = history
                .iter()
                .find(|(_, entries)| !entries.is_empty())
                .map(|(name, _)| name.clone())
                .expect("history bytes imply entries");
            let evicted = history.get_mut(&name).unwrap().pop_front().unwrap();
            history_hot_bytes -= history_charge(&evicted.wire, &evicted.code);
        }
        let current_artifacts: BTreeMap<String, String> = next
            .iter()
            .filter_map(|(name, (_, _, _, artifact))| {
                artifact.as_ref().map(|id| (name.clone(), id.clone()))
            })
            .collect();
        let desired: BTreeSet<String> = current_artifacts
            .values()
            .cloned()
            .chain(
                history
                    .values()
                    .flat_map(|entries| entries.iter().filter_map(|entry| entry.artifact.clone())),
            )
            .collect();
        if let Some(stage) = staged {
            // Protect old and new roots during the multi-step root update.
            for id in self.owned_roots.difference(&desired) {
                stage.protect(id)?;
            }
            let mut added = Vec::new();
            let mut removed = Vec::new();
            let operation = (|| -> Result<(), String> {
                for id in desired.difference(&self.owned_roots) {
                    stage
                        .store
                        .register_root(&self.store_project, id, "soft")
                        .map_err(|e| format!("publish root {id}: {e}"))?;
                    added.push(id.clone());
                }
                for id in self.owned_roots.difference(&desired) {
                    stage
                        .store
                        .remove_root(&self.store_project, "soft", id)
                        .map_err(|e| format!("retire root {id}: {e}"))?;
                    removed.push(id.clone());
                }
                Ok(())
            })();
            if let Err(error) = operation {
                let mut rollback = Vec::new();
                for id in removed {
                    if let Err(e) = stage.store.register_root(&self.store_project, &id, "soft") {
                        rollback.push(format!("restore {id}: {e}"));
                    }
                }
                for id in added {
                    if let Err(e) = stage.store.remove_root(&self.store_project, "soft", &id) {
                        rollback.push(format!("remove {id}: {e}"));
                    }
                }
                return Err(format!("{error}; root rollback issues: {rollback:?}"));
            }
        }
        self.history = history;
        self.history_hot_bytes = history_hot_bytes;
        self.owned_roots = desired;
        self.current_artifacts = current_artifacts;
        self.revision = candidate_rev;
        self.units = candidate_units.into_iter().collect();
        self.current_wire = next
            .iter()
            .map(|(name, (_, _, wire, _))| (name.clone(), wire.clone()))
            .collect();
        self.identity = next
            .into_iter()
            .map(|(name, (code, fp, _, _))| (name, (code, fp)))
            .collect();
        Ok(ApplyOutcome {
            revision: self.revision,
            value,
            live_after_release: arena.live() as u64,
            allocs: arena.alloc_events().len(),
            frees: arena.free_events().len(),
            gate,
            production: self.counters,
        })
    }
}

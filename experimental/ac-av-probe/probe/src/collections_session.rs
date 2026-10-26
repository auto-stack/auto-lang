//! PLAN-822 F-822-R1-02: the REAL collections update consumer.
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
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
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

pub struct CollectionsUpdateSession {
    store_root: Option<PathBuf>,
    units: BTreeMap<String, Arc<FunctionCode>>,
    /// Per unit: (code object, content fingerprint of the admitted wire).
    identity: BTreeMap<String, (Arc<FunctionCode>, String)>,
    revision: u64,
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

impl CollectionsUpdateSession {
    pub fn new(store_root: Option<&Path>, entry: &str) -> Self {
        CollectionsUpdateSession {
            store_root: store_root.map(PathBuf::from),
            units: BTreeMap::new(),
            identity: BTreeMap::new(),
            revision: 0,
            counters: ProductionCounters::default(),
            entry: entry.to_string(),
            host_grants: Vec::new(),
        }
    }

    /// F822-R17-01: set the host's CURRENT capability grants (empty by
    /// default — a capability-requiring unit is never admitted unless the
    /// host explicitly grants it).
    pub fn set_host_grants(&mut self, grants: Vec<String>) {
        self.host_grants = grants;
    }

    /// The linked snapshot of the current units (executable as-is).
    pub fn snapshot(&self) -> Result<ProgramSnapshot, String> {
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
    fn produce_unit(&mut self, m: &MirModule) -> Result<(Arc<FunctionCode>, String), String> {
        let name = m.funcs[0].name.clone();
        verify_mir(m).map_err(|e| format!("verify {name}: {e:?}"))?;
        self.counters.verified += 1;
        let wire = encode_abc(m).map_err(|e| format!("encode {name}: {}", e.code()))?;
        self.counters.encoded += 1;
        let loaded = verify_abc_bytes(&wire.to_flat(), &m.owner)
            .map_err(|e| format!("abc-admit {name}: {e:?}"))?;
        let fc = FunctionCode::admit_unit(loaded, self.revision + 1)
            .map_err(|e| format!("unit-admit {name}: {e}"))?;
        self.counters.admitted += 1;
        if let Some(root) = &self.store_root {
            let store = ManagedStore::open(root).map_err(|e| format!("open store: {e}"))?;
            let owner = &m.owner.module;
            let env = envelope_wrap(
                &wire.to_flat(),
                &EnvelopeIdentity {
                    kind: EnvelopeKind::AbcUnit,
                    producer: auto_av_core::contract::COLLECTIONS_PRODUCER_ID,
                    schema: auto_av_core::contract::ABC_COLLECTIONS_SCHEMA,
                    owner_module: owner,
                    stage: 2,
                    mir_sem: auto_av_core::contract::MIR_COLLECTIONS_SEMANTICS.0,
                    abc_sem: auto_av_core::contract::ABC_COLLECTIONS_SEMANTICS.0,
                    av_sem: auto_av_core::contract::AV_COLLECTIONS_SEMANTICS.0,
                    snapshot: m.funcs[0].source_snapshot,
                    unit: &name,
                    body_fp: &format!("{:016x}", auto_av_core::contract::fnv1a64(&wire.to_flat())),
                },
            );
            let key = QueryKey::new(
                &crate::collections_fixture::COLLECTIONS_ABC_LAYER,
                vec![
                    (
                        "producer".into(),
                        auto_av_core::contract::COLLECTIONS_PRODUCER_ID.into(),
                    ),
                    ("owner-module".into(), owner.clone()),
                    ("entry".into(), name.clone()),
                    (
                        "profile".into(),
                        auto_av_core::contract::AV_COLLECTIONS_PROFILE.into(),
                    ),
                    (
                        "source-snapshot".into(),
                        format!("{:016x}", m.funcs[0].source_snapshot),
                    ),
                    ("unit-binding".into(), name.clone()),
                ],
            );
            store
                .put_artifact(PutRequest {
                    layer: crate::collections_fixture::COLLECTIONS_ABC_LAYER.id,
                    key,
                    kind: ChunkKind::LayoutData,
                    payload: env,
                    edges: vec![],
                    project: Some("collections-update-session".into()),
                })
                .map_err(|e| format!("persist {name}: {e}"))?;
            self.counters.store_writes += 1;
        }
        let fp = format!("{:016x}", auto_av_core::contract::fnv1a64(&wire.to_flat()));
        Ok((Arc::new(fc), fp))
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
        let prev_units = self.units.clone();
        let prev_fp: BTreeMap<String, String> = self
            .identity
            .iter()
            .map(|(k, (_, fp))| (k.clone(), fp.clone()))
            .collect();
        self.counters = ProductionCounters::default();
        let mut next: BTreeMap<String, (Arc<FunctionCode>, String)> = BTreeMap::new();
        let mut order: Vec<String> = self.units.keys().cloned().collect();
        for (name, _) in submitted {
            if !order.iter().any(|n| n == name) {
                order.push(name.to_string());
            }
        }
        for name in &order {
            match submitted.iter().find(|(n, _)| *n == name.as_str()) {
                Some((_, m)) => {
                    // F822-R17-01: capability admission — every capability
                    // the INPUT requires must be granted by the CURRENT
                    // host grant set; a shortfall rejects the revision
                    // BEFORE any production stage, link or installation
                    // (the session keeps its prior state).
                    for c in m.capabilities.iter() {
                        if !self.host_grants.iter().any(|g| g == c) {
                            return Err(format!(
                                "unit `{name}` requires capability `{c}` not granted by the current host grants"
                            ));
                        }
                    }
                    let (fc, fp) = self.produce_unit(m)?;
                    next.insert(name.clone(), (fc, fp));
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
                    next.insert(name.clone(), (fc, fp));
                }
            }
        }
        // The production structural gate over the fingerprint domain: an
        // unchanged unit (same fingerprint) must keep its object.
        let mut gate = GateReport {
            rebuilt_units: self.counters.admitted,
            unchanged_degraded: 0,
        };
        for (name, (fc, fp)) in next.iter() {
            if let Some(prev_fp) = prev_fp.get(name) {
                let same_content = prev_fp == fp;
                let same_object = prev_units
                    .get(name)
                    .map(|old| Arc::ptr_eq(old, fc))
                    .unwrap_or(false);
                if same_content && !same_object {
                    gate.unchanged_degraded += 1;
                }
            }
        }
        // F822-R2-01: BUILD-ASIDE + ATOMIC COMMIT (resident-function-install
        // transaction): the candidate set links and executes BEFORE any
        // session state changes; a rejected candidate (link refusal or
        // execution failure) leaves the prior revision/units/identity
        // untouched and the prior snapshot stays usable.
        let candidate_units = next
            .iter()
            .map(|(k, (fc, _))| (k.clone(), fc.clone()))
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
        self.revision = candidate_rev;
        self.units = candidate_units.into_iter().collect();
        self.identity = next;
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

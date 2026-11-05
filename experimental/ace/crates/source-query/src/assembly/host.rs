//! Backend-free coordinator over the existing module root and store CAS.
//! Preparing never installs. One source root protects both target manifests and
//! all required payloads; there is no second CURRENT or parallel store.
use super::*;
use crate::modules::{ModuleReceipt, ModuleSession, ModuleStore, PreparedModuleBatch};
use ace_core::source::modules::edit_module_function;
use std::collections::BTreeSet;
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};
static NEXT_HOST: AtomicU64 = AtomicU64::new(1);

/// Consumer implementations retain their own admission/producer/cache. A
/// successful validation under this host's exclusive borrow must make commit
/// deterministic and free of external I/O; preparation owns fallible work.
pub trait AssemblyTarget {
    type Candidate;
    fn label(&self) -> &'static str;
    fn current_manifest(&self) -> Result<&Manifest, String>;
    fn prepare_assembly(
        &mut self,
        batch: &PreparedModuleBatch,
        manifest: &Manifest,
    ) -> Result<Self::Candidate, String>;
    fn validate_assembly(&self, candidate: &Self::Candidate) -> Result<(), String>;
    fn commit_assembly(&mut self, candidate: Self::Candidate) -> Result<(), String>;
    fn current_receipt(&mut self) -> Result<ModuleReceipt, String>;
    fn current_digest(&self) -> Result<String, String>;
    fn candidate_receipt(&mut self, candidate: &Self::Candidate) -> Result<ModuleReceipt, String>;
}
pub struct AssemblyHost<A: AssemblyTarget, B: AssemblyTarget> {
    id: u64,
    source: ModuleSession,
    store: ModuleStore,
    first: A,
    second: B,
    current: ModuleReceipt,
}
pub struct PreparedAssembly<A: AssemblyTarget, B: AssemblyTarget> {
    owner: u64,
    base: ModuleReceipt,
    source: PreparedModuleBatch,
    first: A::Candidate,
    second: B::Candidate,
    receipt: ModuleReceipt,
}
impl<A: AssemblyTarget, B: AssemblyTarget> PreparedAssembly<A, B> {
    pub fn receipt(&self) -> &ModuleReceipt {
        &self.receipt
    }
}
fn pair_contexts(
    bound: &BoundModules,
    a: &Manifest,
    b: &Manifest,
) -> Result<BTreeMap<String, String>, String> {
    if a.fingerprint() != b.fingerprint() {
        return Err("assembly.common-identity-mismatch".into());
    }
    let first = manifest_contexts(bound, a)?;
    if manifest_contexts(bound, b)? != first {
        return Err("assembly.declaration-inputs-mismatch".into());
    }
    Ok(first)
}
impl<A: AssemblyTarget, B: AssemblyTarget> AssemblyHost<A, B> {
    fn check_source(source: &ModuleSession, first: &A, second: &B) -> Result<(), String> {
        if first.label() == second.label() {
            return Err("assembly.duplicate-consumer".into());
        }
        let contexts = pair_contexts(
            &source.bound,
            first.current_manifest()?,
            second.current_manifest()?,
        )?;
        if source.query.declaration_contexts() != &contexts {
            return Err("assembly.query-context-mismatch".into());
        }
        Ok(())
    }
    pub fn publish_initial(
        mut source: ModuleSession,
        mut store: ModuleStore,
        mut first: A,
        mut second: B,
    ) -> Result<Self, String> {
        Self::check_source(&source, &first, &second)?;
        let targets = BTreeMap::from([
            (first.label().into(), first.current_receipt()?),
            (second.label().into(), second.current_receipt()?),
        ]);
        let current = source.prepare_current_receipt(&mut store, &targets)?;
        store.publish(&current, None, false)?;
        source.targets = targets;
        Ok(Self {
            id: NEXT_HOST.fetch_add(1, Ordering::Relaxed),
            source,
            store,
            first,
            second,
            current,
        })
    }
    pub fn from_recovery(
        source: ModuleSession,
        store: ModuleStore,
        first: A,
        second: B,
        current: ModuleReceipt,
    ) -> Result<Self, String> {
        Self::check_source(&source, &first, &second)?;
        let targets = source.targets.clone();
        if targets.len() != 2
            || targets.get(first.label()).map(|r| r.digest.as_str())
                != Some(first.current_digest()?.as_str())
            || targets.get(second.label()).map(|r| r.digest.as_str())
                != Some(second.current_digest()?.as_str())
        {
            return Err("assembly.recovery-consumer-receipts".into());
        }
        store.verify_snapshot_inputs(
            &current,
            &source.bound,
            source.query.declaration_contexts(),
            &targets,
            source.query.package(),
            source.query.module(),
        )?;
        Ok(Self {
            id: NEXT_HOST.fetch_add(1, Ordering::Relaxed),
            source,
            store,
            first,
            second,
            current,
        })
    }
    pub fn source(&self) -> &ModuleSession {
        &self.source
    }
    pub fn targets(&self) -> (&A, &B) {
        (&self.first, &self.second)
    }
    pub fn receipt(&self) -> &ModuleReceipt {
        &self.current
    }

    /// The factory reads the once-edited, actually bound source graph. It may
    /// collect current operation/provider facts but cannot alter live targets.
    pub fn prepare(
        &mut self,
        edit: Option<(&str, &str, &str)>,
        budget: FrontBudget,
        manifests: impl FnOnce(&BoundModules) -> Result<(Manifest, Manifest), String>,
    ) -> Result<PreparedAssembly<A, B>, String> {
        let bound = match edit {
            Some((module, name, fragment)) => Rc::new(
                edit_module_function(&self.source.bound, module, name, fragment, budget)
                    .map_err(|e| format!("{e:?}"))?,
            ),
            None => self.source.bound.clone(),
        };
        let edited = edit
            .map(|(module, name, _)| {
                bound
                    .origins
                    .iter()
                    .find(|(_, o)| o.0 == module && o.1 == name)
                    .map(|(n, _)| n.clone())
                    .ok_or("assembly.edit-owner")
            })
            .transpose()?;
        let (a, b) = manifests(&bound)?;
        let contexts = pair_contexts(&bound, &a, &b)?;
        let a_versions = function_input_versions(&bound, &a)?;
        let b_versions = function_input_versions(&bound, &b)?;
        let previous_a =
            function_input_versions(&self.source.bound, self.first.current_manifest()?)?;
        let previous_b =
            function_input_versions(&self.source.bound, self.second.current_manifest()?)?;
        let mut physical = BTreeSet::new();
        for owner in bound.origins.keys() {
            if a_versions.get(owner) != previous_a.get(owner)
                || b_versions.get(owner) != previous_b.get(owner)
            {
                physical.insert(owner.clone());
            }
        }
        let mut source = self
            .source
            .prepare_bound_batch(bound, edited, contexts, &physical)?;
        let first = self.first.prepare_assembly(&source, &a)?;
        let second = self.second.prepare_assembly(&source, &b)?;
        let targets = BTreeMap::from([
            (
                self.first.label().into(),
                self.first.candidate_receipt(&first)?,
            ),
            (
                self.second.label().into(),
                self.second.candidate_receipt(&second)?,
            ),
        ]);
        let receipt = self
            .source
            .prepare_batch_receipt(&mut self.store, &mut source, targets)?;
        Ok(PreparedAssembly {
            owner: self.id,
            base: self.current.clone(),
            source,
            first,
            second,
            receipt,
        })
    }
    /// All validation precedes the single durable CAS. No implementation writer
    /// can enter between validation and the deterministic in-memory installs.
    pub fn publish(
        &mut self,
        candidate: PreparedAssembly<A, B>,
        cancelled: bool,
    ) -> Result<(), String> {
        if candidate.owner != self.id {
            return Err("assembly.foreign-publication".into());
        }
        if candidate.base != self.current {
            return Err("assembly.stale-publication".into());
        }
        self.source.validate_batch(&candidate.source)?;
        self.first.validate_assembly(&candidate.first)?;
        self.second.validate_assembly(&candidate.second)?;
        self.store
            .publish(&candidate.receipt, Some(&self.current), cancelled)?;
        self.source.commit_batch(candidate.source)?;
        self.first.commit_assembly(candidate.first)?;
        self.second.commit_assembly(candidate.second)?;
        self.current = candidate.receipt;
        Ok(())
    }
}

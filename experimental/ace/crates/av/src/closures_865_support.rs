//! PLAN-865 source path: closures-profile source → per-function units →
//! the existing ClosuresUpdateSession/link/runtime contract.

use crate::closures_session::{ApplyOutcome, ClosuresUpdateSession, ProductionCounters};
use ace_av_core::abc::encode_abc;
use ace_av_core::mir::MirModule;
use ace_core::hir::DefKind;
use ace_core::source::{
    compile_closures_source_incremental, ClosuresFrontendLedger, ClosuresOwnerCache,
};
use ace_lowering::lower_function_closures_unit;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

#[derive(Debug)]
pub enum CompileAndRunError {
    Diagnostics(Vec<ace_core::atom_text::Diagnostic>),
    Pipeline(String),
}

pub struct ClosuresSourceRun {
    pub value: i32,
    pub live_after_release: u64,
    pub allocs: usize,
    pub frees: usize,
    pub submitted_units: usize,
    pub submitted_unit_names: Vec<String>,
    pub production: ProductionCounters,
    pub current_hot_bytes: usize,
    pub history_hot_bytes: usize,
    /// PLAN-909 §5.5: the observed frontend stages of THIS apply — real
    /// body-work counts (a reused owner has no resolve/typecheck entry),
    /// the whole-module plan/bind walks reported as they run, and the
    /// lowered/encoded unit names (unchanged owners are absent: their
    /// lower/encode/verify/admit/artifact-write production is zero).
    pub frontend: ClosuresFrontendLedger,
    pub lowered_units: Vec<String>,
}

/// A source-facing owner for one bounded update session. It keeps per-unit
/// body identities so a source edit submits only changed executable bodies;
/// diagnostics-only MIR spans do not create production work for an unchanged
/// function, which stays installed with its existing code object.
pub struct ClosuresSourceSession {
    update: ClosuresUpdateSession,
    current_wires: BTreeMap<String, Vec<u8>>,
    /// PLAN-909 §5.5: the last SUCCESSFUL apply's owner cache. A later
    /// apply reuses owners whose item text and the declaration context
    /// are unchanged — no resolve/typecheck walk, no lower/encode for
    /// their units (the wire identity decides submission as before).
    frontend: Option<ClosuresOwnerCache>,
}

impl ClosuresSourceSession {
    pub fn new(store_root: &Path) -> Self {
        Self {
            update: ClosuresUpdateSession::new(Some(store_root), "d_main"),
            current_wires: BTreeMap::new(),
            frontend: None,
        }
    }

    pub fn apply_source(
        &mut self,
        source: &str,
        source_path: &str,
    ) -> Result<ClosuresSourceRun, CompileAndRunError> {
        // PLAN-909 §5.5: the warm path. The frontend compiles with the
        // previous successful apply's owner cache; owners whose text and
        // the declaration context are unchanged are reused verbatim (zero
        // resolve/typecheck walks — observed in `run.frontend`) and their
        // units skip lowering and encoding entirely (the installed code
        // object, wire and generation stay; the structural gate still owns
        // submission through the wire identity).
        let (compiled, frontend, ledger) =
            compile_closures_source_incremental(self.frontend.as_ref(), source, source_path, &[])
                .map_err(CompileAndRunError::Diagnostics)?;
        // Units owned by reused owners: skip lower/encode when their wire
        // is already installed (a cache hit without an installed wire — a
        // failed previous apply — still goes through the full path).
        let mut skip_units: BTreeSet<String> = BTreeSet::new();
        for owner in &ledger.reused_owners {
            let Some(entry) = frontend.owners.get(owner) else {
                continue;
            };
            for name in std::iter::once(&entry.resolved).chain(entry.lambdas.iter()) {
                let unit = format!("d_{}", name.name);
                if self.current_wires.contains_key(&unit) {
                    skip_units.insert(unit);
                }
            }
        }
        let bundle = compiled.checked.bundle();
        let module = &bundle.modules[0];
        // The snapshot identifies the source unit's stable logical origin.
        // Body bytes remain a separate per-function content axis so editing
        // one body does not invalidate unchanged sibling units.
        let source_snapshot = ace_av_core::contract::fnv1a64(source_path.as_bytes());
        let mut all_units = BTreeMap::<String, (MirModule, Vec<u8>)>::new();
        let mut lowered_units = Vec::new();
        for def in module
            .defs
            .iter()
            .filter(|def| matches!(def.kind, DefKind::Function { .. }))
        {
            // def.id_text already carries the unit prefix ().
            let name_probe = def.id_text.clone();
            if skip_units.contains(&name_probe) {
                // Reused owner, installed wire: zero lower/encode work; the
                // wire comparison below never sees (or needs) this unit.
                continue;
            }
            let unit = lower_function_closures_unit(bundle, &def.id_text, &[], source_snapshot)
                .map_err(|error| {
                    CompileAndRunError::Pipeline(format!("lower {}: {error:?}", def.id_text))
                })?;
            let name = unit.funcs[0].name.clone();
            let mut identity = unit.clone();
            for function in &mut identity.funcs {
                function.spans.clear();
            }
            let bytes = encode_abc(&identity)
                .map_err(|error| CompileAndRunError::Pipeline(format!("encode {name}: {error:?}")))?
                .to_flat();
            lowered_units.push(name.clone());
            all_units.insert(name, (unit, bytes));
        }
        if all_units.is_empty() && self.current_wires.is_empty() {
            // A fully-reused apply (identical source) has empty lowered
            // output but installed wires — that is a zero-production hit,
            // not a failure.
            return Err(CompileAndRunError::Pipeline(
                "closures source produced no function units".into(),
            ));
        }
        let submitted = all_units
            .iter()
            .filter(|(name, (_, bytes))| self.current_wires.get(*name) != Some(bytes))
            .map(|(name, (unit, _))| (name.clone(), unit.clone()))
            .collect::<Vec<_>>();
        let submitted_units = submitted.len();
        let submitted_unit_names = submitted
            .iter()
            .map(|(name, _)| name.clone())
            .collect::<Vec<_>>();
        let submitted_refs = submitted
            .iter()
            .map(|(name, unit)| (name.as_str(), unit.clone()))
            .collect::<Vec<_>>();
        // Unchanged owners keep their installed wires; only lowered units
        // refresh the wire map.
        let mut next_wires = self.current_wires.clone();
        for (name, (_, bytes)) in &all_units {
            next_wires.insert(name.clone(), bytes.clone());
        }
        let outcome = self
            .update
            .apply_revision(&submitted_refs)
            .map_err(CompileAndRunError::Pipeline)?;
        if !outcome.gate.pass() {
            return Err(CompileAndRunError::Pipeline(format!(
                "source revision failed the closures structural gate: {:?}",
                outcome.gate
            )));
        }
        self.current_wires = next_wires;
        self.frontend = Some(frontend);
        Self::run_face(
            outcome,
            submitted_units,
            submitted_unit_names,
            ledger,
            lowered_units,
        )
    }

    pub fn close(&mut self) -> Result<(), String> {
        self.update.close()
    }

    #[allow(clippy::too_many_arguments)]
    fn run_face(
        outcome: ApplyOutcome,
        submitted_units: usize,
        submitted_unit_names: Vec<String>,
        frontend: ClosuresFrontendLedger,
        lowered_units: Vec<String>,
    ) -> Result<ClosuresSourceRun, CompileAndRunError> {
        let value = outcome.value.ok_or_else(|| {
            CompileAndRunError::Pipeline("source entry did not produce an i32 value".into())
        })?;
        Ok(ClosuresSourceRun {
            value,
            live_after_release: outcome.live_after_release,
            allocs: outcome.allocs,
            frees: outcome.frees,
            submitted_units,
            submitted_unit_names,
            production: outcome.production,
            current_hot_bytes: outcome.current_hot_bytes,
            history_hot_bytes: outcome.history_hot_bytes,
            frontend,
            lowered_units,
        })
    }
}

pub fn compile_and_execute(
    source: &str,
    source_path: &str,
    store_root: &Path,
) -> Result<ClosuresSourceRun, CompileAndRunError> {
    let mut session = ClosuresSourceSession::new(store_root);
    let outcome = session.apply_source(source, source_path)?;
    session.close().map_err(CompileAndRunError::Pipeline)?;
    Ok(outcome)
}

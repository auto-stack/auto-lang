//! PLAN-865 source path: closures-profile source → per-function units →
//! the existing ClosuresUpdateSession/link/runtime contract.

use auto_ac_prototype::hir::DefKind;
use auto_ac_prototype::source::compile_closures_source;
use auto_av_core::abc::encode_abc;
use crate::closures_session::{ApplyOutcome, ClosuresUpdateSession, ProductionCounters};
use auto_av_core::mir::MirModule;
use auto_mir_lower::lower_function_closures_unit;
use std::collections::BTreeMap;
use std::path::Path;

#[derive(Debug)]
pub enum CompileAndRunError {
    Diagnostics(Vec<auto_ac_prototype::atom_text::Diagnostic>),
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
}

/// A source-facing owner for one bounded update session. It keeps per-unit
/// body identities so a source edit submits only changed executable bodies;
/// diagnostics-only MIR spans do not create production work for an unchanged
/// function, which stays installed with its existing code object.
pub struct ClosuresSourceSession {
    update: ClosuresUpdateSession,
    current_wires: BTreeMap<String, Vec<u8>>,
}

impl ClosuresSourceSession {
    pub fn new(store_root: &Path) -> Self {
        Self {
            update: ClosuresUpdateSession::new(Some(store_root), "d_main"),
            current_wires: BTreeMap::new(),
        }
    }

    pub fn apply_source(
        &mut self,
        source: &str,
        source_path: &str,
    ) -> Result<ClosuresSourceRun, CompileAndRunError> {
        let compiled = compile_closures_source(source, source_path, &[])
            .map_err(CompileAndRunError::Diagnostics)?;
        let bundle = compiled.checked.bundle();
        let module = &bundle.modules[0];
        // The snapshot identifies the source unit's stable logical origin.
        // Body bytes remain a separate per-function content axis so editing
        // one body does not invalidate unchanged sibling units.
        let source_snapshot = auto_av_core::contract::fnv1a64(source_path.as_bytes());
        let mut all_units = BTreeMap::<String, (MirModule, Vec<u8>)>::new();
        for def in module
            .defs
            .iter()
            .filter(|def| matches!(def.kind, DefKind::Function { .. }))
        {
            let unit = lower_function_closures_unit(
                bundle,
                &def.id_text,
                &[],
                source_snapshot,
            )
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
            all_units.insert(name, (unit, bytes));
        }
        if all_units.is_empty() {
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
        let next_wires = all_units
            .into_iter()
            .map(|(name, (_, bytes))| (name, bytes))
            .collect::<BTreeMap<_, _>>();
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
        Self::run_face(outcome, submitted_units, submitted_unit_names)
    }

    pub fn close(&mut self) -> Result<(), String> {
        self.update.close()
    }

    fn run_face(
        outcome: ApplyOutcome,
        submitted_units: usize,
        submitted_unit_names: Vec<String>,
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

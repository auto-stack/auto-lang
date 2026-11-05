//! PLAN-950 module graph consumer seam. Only the existing fact producer,
//! resolver, emitter and HIR verifier may construct its checked artifact.
use crate::{bind_with_facts, AppliedFn, BodyTypeSupply, SourceQuerySession};
use ace_core::atom_text::{Diagnostic, Span, Stage};
use ace_core::source::cst::FrontBudget;
use ace_core::source::modules::{bind_modules, BoundModules, ModuleDiagnostic, ModuleInput};
use ace_core::source::{emit_resolved_module_parts, CompiledSource, SourceProfile};

pub struct ModuleProduct {
    pub compiled: CompiledSource,
    pub bound: BoundModules,
    pub applied: Vec<AppliedFn>,
    pub supply: BodyTypeSupply,
}

pub fn compile_module_product(
    inputs: &[ModuleInput],
    entry: &str,
    caps: &[String],
    session: &mut SourceQuerySession,
    budget: FrontBudget,
) -> Result<ModuleProduct, Vec<ModuleDiagnostic>> {
    let bound = bind_modules(inputs, entry, caps, budget).map_err(|e| vec![e])?;
    compile_bound_modules(bound, caps, session)
}

/// Shared by the cold module lane and retained/restored module graphs. A
/// caller must retain the graph's visibility/identity admission, never a bare
/// Program manufactured by stripping public/import metadata.
pub(crate) fn compile_bound_modules(
    bound: BoundModules,
    caps: &[String],
    session: &mut SourceQuerySession,
) -> Result<ModuleProduct, Vec<ModuleDiagnostic>> {
    session.set_capabilities(caps.iter().cloned());
    let (resolved, applied, supply) = bind_with_facts(
        &bound.program,
        caps,
        session.package().to_owned().as_str(),
        session.module().to_owned().as_str(),
        session,
        SourceProfile::Scalar,
    )
    .map_err(|e| {
        let ds = match e {
            crate::BindFactsError::Resolve(ds) => ds,
            other => vec![Diagnostic::new(
                Stage::Bind,
                "source.module-facts",
                format!("{other}"),
                Span { start: 0, end: 0 },
            )],
        };
        ds.into_iter().map(|d| bound.locate(d)).collect::<Vec<_>>()
    })?;
    let parts = emit_resolved_module_parts(&resolved, &bound, "modules")
        .map_err(|ds| ds.into_iter().map(|d| bound.locate(d)).collect::<Vec<_>>())?;
    let checked = ace_core::verify::verify_scalar(parts.bundle)
        .map_err(|ds| ds.into_iter().map(|d| bound.locate(d)).collect::<Vec<_>>())?;
    Ok(ModuleProduct {
        compiled: CompiledSource {
            profile: parts.profile,
            input_hash: parts.input_hash,
            atom_text: parts.atom_text,
            checked,
        },
        bound,
        applied,
        supply,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use ace_core::source::modules::MODULES_CAPABILITY;
    #[test]
    fn finite_module_graph_reaches_checked_hir_through_query_facts() {
        let inputs=[ModuleInput {id:"std".into(),source:"pub fn answer() int { return 42 }".into()},
            ModuleInput {id:"app".into(),source:"use std.answer\nfn answer() int { return 7 } fn main() int { return std.answer() + answer() }".into()}];
        let mut session = SourceQuerySession::new("modules", "app");
        let product = compile_module_product(
            &inputs,
            "app",
            &[MODULES_CAPABILITY.into()],
            &mut session,
            FrontBudget::WORK_OWNED_64MIB,
        )
        .unwrap();
        assert_eq!(product.applied.len(), 3);
        let before = session.counters();
        let _ = compile_module_product(
            &inputs,
            "app",
            &[MODULES_CAPABILITY.into()],
            &mut session,
            FrontBudget::WORK_OWNED_64MIB,
        )
        .unwrap();
        assert_eq!(session.counters().solver_runs, before.solver_runs);
    }
}

mod persistence;
pub use persistence::{ModuleReceipt, ModuleRecovery, ModuleStore, MODULE_ROOT_SCHEMA};

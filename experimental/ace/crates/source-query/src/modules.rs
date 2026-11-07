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

fn fact_diagnostics(
    bound: &BoundModules,
    body: &ace_core::source::ast::Block,
    facts: &crate::facts::InferBodyFacts,
) -> Vec<ModuleDiagnostic> {
    let Ok(walk) = crate::walk::walk_body(body) else {
        return Vec::new();
    };
    facts
        .diags
        .iter()
        .map(|d| {
            let span = d
                .node
                .and_then(|id| walk.node(id))
                .map(|n| n.span)
                .unwrap_or(body.span);
            bound.locate(Diagnostic::new(Stage::Bind, &d.code, &d.message, span))
        })
        .collect()
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
        if !matches!(e, crate::BindFactsError::Resolve(_)) {
            let ds: Vec<_> = bound
                .program
                .items
                .iter()
                .filter_map(|item| match item {
                    ace_core::source::ast::Item::Fn(f) => session
                        .cached_body_facts(&bound.program, &f.name.name)
                        .map(|facts| fact_diagnostics(&bound, &f.body, facts)),
                    _ => None,
                })
                .flatten()
                .collect();
            if !ds.is_empty() {
                return ds;
            }
        }
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
    fn module_type_rejections_keep_the_actual_source_location() {
        let caps = [MODULES_CAPABILITY.into()];
        let good = "pub fn accept_bool(x bool) int { return 42 } pub fn answer() int { return accept_bool(true) }";
        let bad = good.replace("accept_bool(true)", "accept_bool(1)");
        let mut inputs = [
            ModuleInput {
                id: "std".into(),
                source: good.into(),
            },
            ModuleInput {
                id: "app".into(),
                source: "fn main() int { return std.answer() }".into(),
            },
        ];
        let mut query = SourceQuerySession::new("pkg", "app");
        let product = compile_module_product(
            &inputs,
            "app",
            &caps,
            &mut query,
            FrontBudget::WORK_OWNED_64MIB,
        )
        .unwrap();
        let mut retained = ModuleSession::from_product(&product, query).unwrap();
        let err = retained
            .prepare(
                "std",
                "answer",
                "pub fn answer() int { return accept_bool(1) }",
                FrontBudget::WORK_OWNED_64MIB,
            )
            .err()
            .unwrap();
        assert!(err.contains("std"), "{err}");
        inputs[0].source = bad.clone();
        let mut query = SourceQuerySession::new("pkg", "app");
        let ds = compile_module_product(
            &inputs,
            "app",
            &caps,
            &mut query,
            FrontBudget::WORK_OWNED_64MIB,
        )
        .err()
        .unwrap();
        assert_eq!(ds[0].module, "std", "{ds:?}");
        assert!(
            ds[0].diagnostic.span.start >= bad.find("accept_bool(1)").unwrap(),
            "{ds:?}"
        );
        assert!(ds[0].diagnostic.span.end <= bad.len(), "{ds:?}");
        assert!(
            err.contains(&format!("start: {}", ds[0].diagnostic.span.start)),
            "{err}; cold={ds:?}"
        );
        let parse_err = ace_core::source::modules::edit_module_function(
            &product.bound,
            "std",
            "answer",
            "pub fn answer() int { return @ }",
            FrontBudget::WORK_OWNED_64MIB,
        )
        .err()
        .unwrap();
        assert_eq!(parse_err.module, "std");
        assert!(
            parse_err.diagnostic.span.start > good.find("pub fn answer").unwrap(),
            "{parse_err:?}"
        );
    }
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
    #[test]
    fn missing_export_rechecks_current_declarations_and_keeps_unrelated_facts() {
        let caps = [MODULES_CAPABILITY.into()];
        let mut inputs = [
            ModuleInput {
                id: "std".into(),
                source: "pub fn answer() int { return 42 }".into(),
            },
            ModuleInput {
                id: "app".into(),
                source: "fn local() int { return 7 } fn main() int { return std.answer() }".into(),
            },
        ];
        let mut session = SourceQuerySession::new("pkg", "app");
        compile_module_product(
            &inputs,
            "app",
            &caps,
            &mut session,
            FrontBudget::WORK_OWNED_64MIB,
        )
        .unwrap();
        let before = session.counters().solver_runs;
        inputs[1].source =
            "fn local() int { return 7 } fn main() int { return std.missing() }".into();
        let bad = compile_module_product(
            &inputs,
            "app",
            &caps,
            &mut session,
            FrontBudget::WORK_OWNED_64MIB,
        )
        .err()
        .unwrap();
        assert_eq!(bad[0].diagnostic.code, "source.module-symbol");
        assert_eq!(
            session.counters().solver_runs,
            before,
            "missing binding cannot reach a type/target producer"
        );
        inputs[0]
            .source
            .push_str(" pub fn missing() int { return 99 }");
        let p = compile_module_product(
            &inputs,
            "app",
            &caps,
            &mut session,
            FrontBudget::WORK_OWNED_64MIB,
        )
        .unwrap();
        assert!(
            p.applied
                .iter()
                .find(|f| f.name == "m3_app_local")
                .unwrap()
                .body_hit
        );
        assert!(
            p.applied
                .iter()
                .find(|f| f.name == "m3_std_answer")
                .unwrap()
                .body_hit
        );
        assert!(
            !p.applied
                .iter()
                .find(|f| f.name == "main")
                .unwrap()
                .body_hit
        );
        assert_eq!(session.counters().solver_runs - before, 2);
    }
    #[test]
    fn signature_read_change_reproduces_its_reader_but_not_an_unrelated_body() {
        let caps = [MODULES_CAPABILITY.into()];
        let mut inputs = [
            ModuleInput {
                id: "std".into(),
                source: "pub fn choice() int { return 1 }".into(),
            },
            ModuleInput {
                id: "app".into(),
                source:
                    "fn local() int { return 7 } fn main() int { let x = std.choice()\n return 7 }"
                        .into(),
            },
        ];
        let mut session = SourceQuerySession::new("pkg", "app");
        let old = compile_module_product(
            &inputs,
            "app",
            &caps,
            &mut session,
            FrontBudget::WORK_OWNED_64MIB,
        )
        .unwrap();
        let key = session.body_key(&old.bound.program, "main").unwrap();
        let before = session.counters().solver_runs;
        inputs[0].source = "pub fn choice() bool { return true }".into();
        let p = compile_module_product(
            &inputs,
            "app",
            &caps,
            &mut session,
            FrontBudget::WORK_OWNED_64MIB,
        )
        .unwrap();
        assert_eq!(
            key,
            session.body_key(&p.bound.program, "main").unwrap(),
            "the reader body itself did not change"
        );
        assert!(
            p.applied
                .iter()
                .find(|f| f.name == "m3_app_local")
                .unwrap()
                .body_hit
        );
        assert!(
            !p.applied
                .iter()
                .find(|f| f.name == "main")
                .unwrap()
                .body_hit
        );
        assert_eq!(session.counters().solver_runs - before, 2);
        let facts = session.infer_body(&p.bound.program, "main").unwrap();
        assert_eq!(facts.facts.bindings[0].ty, crate::facts::PortableTy::Bool);
    }
}

pub mod packages;
pub use packages::{compile_bound_packages, compile_package_product, PackageProduct};

mod persistence;
pub use persistence::{
    ModuleReceipt, ModuleRecovery, ModuleStore, CONTEXT_ROOT_SCHEMA, MODULE_ROOT_SCHEMA,
};

mod session;
pub use session::{ModuleSession, PreparedModuleBatch, PreparedModuleBody, PreparedModuleEdit};

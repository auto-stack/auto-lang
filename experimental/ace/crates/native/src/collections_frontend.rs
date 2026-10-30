//! Bounded cold collections producer, using the existing retained environment
//! and exactly the same source emitter, HIR verifier and shared MIR lowerer.
use ace_av_core::mir::MirModule;
use ace_core::{
    atom_text::{Diagnostic, Span, Stage},
    source::{self, CheckedEnv, SourceProfile},
};

pub struct ColdCollections<T = MirModule> {
    pub env: CheckedEnv,
    pub units: Vec<T>,
    pub real_body_plans: usize,
    pub scaffold_declarations: usize,
}
fn backend(error: impl std::fmt::Display) -> Vec<Diagnostic> {
    vec![Diagnostic::new(
        Stage::Backend,
        "native.collections-lower",
        error.to_string(),
        Span { start: 0, end: 0 },
    )]
}
pub fn cold(source: &str, module_path: &str) -> Result<ColdCollections, Vec<Diagnostic>> {
    cold_produce(source, module_path, |unit| {
        crate::collections_lower::admit_unit(&unit).map_err(backend)?;
        Ok(unit)
    })
}
pub(crate) fn cold_admitted(
    source: &str,
    module_path: &str,
) -> Result<ColdCollections<crate::collections_lower::CheckedCollectionUnit>, Vec<Diagnostic>> {
    cold_produce(source, module_path, |unit| {
        crate::collections_lower::CheckedCollectionUnit::admit(std::rc::Rc::new(unit))
            .map_err(backend)
    })
}
fn cold_produce<T>(
    source: &str,
    module_path: &str,
    mut admit: impl FnMut(MirModule) -> Result<T, Vec<Diagnostic>>,
) -> Result<ColdCollections<T>, Vec<Diagnostic>> {
    if source.len() > crate::config::MAX_INPUT_BYTES {
        return Err(vec![Diagnostic::new(
            Stage::Backend,
            "mn.resource.input-size",
            "collections source exceeds the existing native input budget",
            Span { start: 0, end: 0 },
        )]);
    }
    let program =
        source::parse_program_with(source, SourceProfile::Collections).map_err(|d| vec![d])?;
    let mut env = source::resolve_env(&program, &[], SourceProfile::Collections)?;
    drop(program);
    env.typecheck_all()?;
    let names: Vec<_> = source::env_fn_names(&env)
        .into_iter()
        .map(|name| format!("d_{name}"))
        .collect();
    let mut units = Vec::with_capacity(names.len());
    let mut real_body_plans = 0;
    let mut scaffold_declarations = 0;
    let snapshot = ace_av_core::contract::fnv1a64(source.as_bytes());
    let regions = source::env_fn_regions(&env);
    let mut start = 0;
    while start < names.len() {
        // A large body always keeps the old single-body path. This cap only
        // chooses batching; it does not reject any legal source function.
        let mut end = start + 1;
        let mut bytes = regions[start].body.end - regions[start].body.start;
        while end < names.len() && end - start < 4 {
            let next = regions[end].body.end - regions[end].body.start;
            if bytes.saturating_add(next) > 16 * 1024 {
                break;
            }
            bytes += next;
            end += 1;
        }
        let (parts, receipt) = if end == start + 1 {
            source::emit_function_body(&env, start, source, module_path)?
        } else {
            source::emit_collections_function_batch(
                &env,
                &(start..end).collect::<Vec<_>>(),
                source,
                module_path,
            )?
        };
        for name in &names[start..end] {
            let module = &parts.bundle.modules[0];
            let def = module
                .defs
                .iter()
                .position(|d| &d.id_text == name)
                .ok_or_else(|| backend("missing scoped declaration"))?;
            let credential = ace_core::verify::verify_function_ref(&parts.bundle, def)?;
            debug_assert_eq!(credential.def_id_text(), name);
            let unit =
                crate::collections_memory::scoped(crate::resources::Partition::MirSymbol, || {
                    ace_lowering::hir_adapter::lower_function_collections_unit(
                        &parts.bundle,
                        name,
                        &[],
                        snapshot,
                    )
                    .map_err(|e| backend(format!("{e:?}")))
                })?;
            units.push(crate::collections_memory::scoped(
                crate::resources::Partition::MirSymbol,
                || admit(unit),
            )?);
        }
        real_body_plans += receipt.real_body_plans;
        scaffold_declarations += receipt.scaffold_fns;
        start = end;
        // parts, Atom text and checked body die here; no whole-module Atom/HIR
        // copy is ever retained alongside all the source and MIR bodies.
    }
    Ok(ColdCollections {
        env,
        units,
        real_body_plans,
        scaffold_declarations,
    })
}

//! Bounded cold collections producer, using the existing retained environment
//! and exactly the same source emitter, HIR verifier and shared MIR lowerer.
use auto_ac_prototype::{
    atom_text::{Diagnostic, Span, Stage},
    source::{self, CheckedEnv, SourceProfile},
};
use auto_av_core::mir::MirModule;

pub struct ColdCollections {
    pub env: CheckedEnv,
    pub units: Vec<MirModule>,
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
    for (index, name) in names.iter().enumerate() {
        let (parts, receipt) = source::emit_function_body(&env, index, source, module_path)?;
        let module = &parts.bundle.modules[0];
        let def = module
            .defs
            .iter()
            .position(|d| &d.id_text == name)
            .ok_or_else(|| backend("missing scoped declaration"))?;
        let credential = auto_ac_prototype::verify::verify_function(parts.bundle.clone(), def)?;
        debug_assert_eq!(credential.def_id_text(), name);
        let unit = auto_mir_lower::hir_adapter::lower_function_collections_unit(
            &parts.bundle,
            name,
            &[],
            auto_av_core::contract::fnv1a64(source.as_bytes()),
        )
        .map_err(|e| backend(format!("{e:?}")))?;
        auto_av_core::verify_mir(&unit).map_err(|e| backend(format!("{e:?}")))?;
        crate::collections_lower::admit_unit(&unit).map_err(backend)?;
        units.push(unit);
        real_body_plans += receipt.real_body_plans;
        scaffold_declarations += receipt.scaffold_fns;
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

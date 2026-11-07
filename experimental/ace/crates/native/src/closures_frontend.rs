//! PLAN-924: bounded cold closures producer. The SAME retained source
//! frontend the AV closures sessions use (`compile_closures_source` truth
//! path); every function def — owners and synthetic lambda bodies alike —
//! lowers to its own `av.closures/1` unit through the shared per-function
//! adapter, then passes the native limited-domain admission. No AV
//! executor, no ABC interpretation, no whole-module native lowering.

use ace_av_core::mir::MirModule;
use ace_core::{
    atom_text::{Diagnostic, Span, Stage},
    hir::DefKind,
};

pub struct ColdClosures<T = MirModule> {
    pub units: Vec<T>,
    pub real_body_plans: usize,
    pub scaffold_declarations: usize,
}
/// The cold-produce error faces: source diagnostics keep the shared
/// frontend's report; native admission failures pass through UNMASKED so
/// the named rejection (type/capture/instruction face) reaches the parent.
pub enum ColdError {
    Source(Vec<Diagnostic>),
    Admit(crate::MnError),
}
pub fn cold(source: &str, module_path: &str) -> Result<ColdClosures, ColdError> {
    cold_produce(source, module_path, |unit| {
        crate::closures_lower::admit_layouts(&unit).map_err(ColdError::Admit)?;
        Ok(unit)
    })
    .map_err(|e| match e {
        ColdProduceError::Source(ds) => ColdError::Source(ds),
        ColdProduceError::Admit(e) => ColdError::Admit(e),
    })
}
enum ColdProduceError {
    Source(Vec<Diagnostic>),
    Admit(crate::MnError),
}
fn cold_produce<T>(
    source: &str,
    module_path: &str,
    mut admit: impl FnMut(MirModule) -> Result<T, ColdError>,
) -> Result<ColdClosures<T>, ColdProduceError> {
    if source.len() > crate::config::MAX_INPUT_BYTES {
        return Err(ColdProduceError::Source(vec![Diagnostic::new(
            Stage::Backend,
            "mn.resource.input-size",
            "closures source exceeds the existing native input budget",
            Span { start: 0, end: 0 },
        )]));
    }
    let compiled = ace_core::source::compile_closures_source(source, module_path, &[])
        .map_err(ColdProduceError::Source)?;
    let bundle = compiled.checked.bundle();
    let module = bundle.modules.first().ok_or_else(|| {
        ColdProduceError::Admit(crate::MnError::Admit(
            "closures",
            "bundle carries no module".into(),
        ))
    })?;
    let snapshot = ace_av_core::contract::fnv1a64(source.as_bytes());
    let defs: Vec<String> = module
        .defs
        .iter()
        .filter(|def| matches!(def.kind, DefKind::Function { .. }))
        .map(|def| def.id_text.clone())
        .collect();
    let mut units = Vec::with_capacity(defs.len());
    let real_body_plans = defs.len();
    for def in &defs {
        let unit =
            crate::collections_memory::scoped(crate::resources::Partition::MirSymbol, || {
                ace_lowering::hir_adapter::lower_function_closures_unit(bundle, def, &[], snapshot)
                    .map_err(|e| ColdProduceError::Admit(crate::MnError::Backend(format!("{e:?}"))))
            })?;
        units.push(crate::collections_memory::scoped(
            crate::resources::Partition::MirSymbol,
            || {
                admit(unit).map_err(|e| match e {
                    ColdError::Source(ds) => ColdProduceError::Source(ds),
                    ColdError::Admit(e) => ColdProduceError::Admit(e),
                })
            },
        )?);
    }
    // The bundle and checked module die here; the resident/incremental
    // paths keep their own bounded frontend state (T-07).
    Ok(ColdClosures {
        units,
        real_body_plans,
        scaffold_declarations: 0,
    })
}

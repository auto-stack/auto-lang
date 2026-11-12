//! PLAN-934 T-08: the product seam for the fact-driven scalar source entry.
//!
//! Every AV/native/A2X host compiles ordinary scalar source through
//! [`compile_scalar_source_product`]: per-body query facts → fact-supplied
//! bind → emitted HIR → real verification, with the query layer as the
//! type authority. The emitted HIR is byte-identical to the legacy
//! `compile_scalar_source` result for the finite surface (crate-level
//! corpus parity), so a consumer that switches to this seam keeps the same
//! executable artifact while the legacy typecheck/synthesis path no longer
//! produces its types.
//!
//! The entry also returns the admitted fact SUPPLY, so a host that retains
//! a checked environment (AV warm session, native session, A2X cold store)
//! can seed it from the same facts instead of re-deriving types with the
//! legacy rules.

use ace_core::atom_text::{Diagnostic, Stage};
use ace_core::source::CompiledSource;
use ace_core::verify::CheckedModule;
use ace_source_query::{
    compile_core_product, compile_scalar_product, CompileLane, ProductCompile, SourceQuerySession,
};

/// Compile one ordinary scalar source for a product consumer.
///
/// A language rejection is reported as the frontend's own diagnostics
/// (stage/code/message/span unchanged); a construct outside the finite
/// query surface is a controlled `source.unsupported` rejection, never a
/// silent re-route to the legacy type engine.
pub fn compile_scalar_source_product(
    src: &str,
    module_path: &str,
    capabilities: &[String],
    package: &str,
) -> Result<ProductCompile, Vec<Diagnostic>> {
    // PLAN-936（T-11）：CT 需求的产品接线——携带 `ac.source.cte.scalar/v1`
    // 能力的请求经 source-cte 的 CT 单门（服务/回灌/事实车道），产物与
    // 普通车道同形（CompiledSource/instances/program）。服务在此组合根
    // 注入：每请求 owned store（冷真实执行；持久 store 由宿主注入是
    // T-09 恢复面的职责）。
    if capabilities
        .iter()
        .any(|c| c == ace_core::source::cst::CT_CAPABILITY)
    {
        return compile_scalar_source_ct(src, module_path, capabilities, package);
    }
    let mut session = SourceQuerySession::new(package, module_path)
        .with_capabilities(capabilities.iter().cloned());
    compile_scalar_product(
        src,
        module_path,
        capabilities,
        package,
        module_path,
        &mut session,
    )
}

/// Convenience form for consumers that only need the compiled artifact.
pub fn compile_scalar_source_via_facts(
    src: &str,
    module_path: &str,
    capabilities: &[String],
) -> Result<CompiledSource, Vec<Diagnostic>> {
    compile_scalar_source_product(src, module_path, capabilities, "ace-av").map(|p| p.compiled)
}

/// PLAN-999 (T-03): the Core-compatible shared selection gate for the
/// ordinary Auto source lane. The explicit scalar entry above keeps its
/// frozen old meaning (it refuses `while`); this gate first runs that exact
/// scalar product, and ONLY when the scalar face reports a profile-SURFACE
/// rejection (`source.unsupported` at the Text stage — the projector's
/// "not part of this source profile" class) does it re-attempt the same
/// source through the CORE fact entry (same producer, Core verifier). A
/// scalar success keeps its original identity and bytes untouched; real
/// syntax/type/resource/cancel/infra verdicts pass through unchanged (no
/// retry); if the Core face also fails, ITS diagnostics are the answer —
/// the face that owns the rejected construct reports the next real problem.
pub fn compile_scalar_or_core_source_product(
    src: &str,
    module_path: &str,
    capabilities: &[String],
    package: &str,
) -> Result<ProductCompile, Vec<Diagnostic>> {
    // The CT lane keeps its single-door contract (unchanged by this gate).
    if capabilities
        .iter()
        .any(|c| c == ace_core::source::cst::CT_CAPABILITY)
    {
        return compile_scalar_source_product(src, module_path, capabilities, package);
    }
    match compile_scalar_source_product(src, module_path, capabilities, package) {
        Ok(p) => Ok(p),
        Err(ds) if is_profile_surface_rejection(&ds) => {
            let mut session = SourceQuerySession::new(package, module_path)
                .with_capabilities(capabilities.iter().cloned());
            compile_core_product(
                src,
                module_path,
                capabilities,
                package,
                module_path,
                &mut session,
            )
        }
        Err(ds) => Err(ds),
    }
}

/// A profile-surface rejection: every diagnostic is the projector's
/// `source.unsupported` at the Text stage. Mirrors the native driver's
/// escalation rule — anything else (parse, bind, verify, resource, cancel)
/// is terminal and never re-routed.
fn is_profile_surface_rejection(ds: &[Diagnostic]) -> bool {
    ds.iter()
        .all(|d| d.code == "source.unsupported" && d.stage == Stage::Text)
}

/// Convenience form of the shared selection gate (compiled artifact only).
pub fn compile_scalar_or_core_source_via_facts(
    src: &str,
    module_path: &str,
    capabilities: &[String],
) -> Result<CompiledSource, Vec<Diagnostic>> {
    compile_scalar_or_core_source_product(src, module_path, capabilities, "ace-av")
        .map(|p| p.compiled)
}

/// PLAN-999 (T-03): the CORE fact entry as a compiled-artifact convenience —
/// the native driver's escalation ladder calls this for its Core step.
pub fn compile_core_source_via_facts(
    src: &str,
    module_path: &str,
    capabilities: &[String],
) -> Result<CompiledSource, Vec<Diagnostic>> {
    let mut session = SourceQuerySession::new("ace-av", module_path)
        .with_capabilities(capabilities.iter().cloned());
    compile_core_product(
        src,
        module_path,
        capabilities,
        "ace-av",
        module_path,
        &mut session,
    )
    .map(|p| p.compiled)
}

/// The fact-driven entry plus its provenance (used by tests and by hosts
/// that keep their own `SourceQuerySession` across edits).
pub fn compile_scalar_source_via_facts_reported(
    src: &str,
    module_path: &str,
    capabilities: &[String],
    session: &mut SourceQuerySession,
    package: &str,
) -> Result<ProductCompile, Vec<Diagnostic>> {
    compile_scalar_product(
        src,
        module_path,
        capabilities,
        package,
        module_path,
        session,
    )
}

/// The verified checked module only (tiny helper for execution tests).
pub fn compile_scalar_checked_via_facts(
    src: &str,
    module_path: &str,
    capabilities: &[String],
) -> Result<CheckedModule, String> {
    compile_scalar_source_via_facts(src, module_path, capabilities)
        .map(|c| c.checked)
        .map_err(|ds| {
            ds.first()
                .map(|d| format!("{}: {}", d.code, d.message))
                .unwrap_or_else(|| "frontend failed".into())
        })
}

/// PLAN-936（T-11）：the CT lane of the product entry — the source-cte
/// single door (serve every `#{ ... }` demand for real → structural
/// backfill → the fact lane) wrapped into the ProductCompile shape every
/// consumer consumes unchanged.
fn compile_scalar_source_ct(
    src: &str,
    module_path: &str,
    capabilities: &[String],
    package: &str,
) -> Result<ProductCompile, Vec<Diagnostic>> {
    let mut svc = ace_source_cte::CtService::new(ace_source_cte::MemoryEnvelopeSink::new());
    let out = ace_source_cte::compile_scalar_ct(
        &mut svc,
        src,
        module_path,
        capabilities,
        package,
        module_path,
    )
    .map_err(|e| match e {
        ace_source_cte::CtProductError::Diagnostics(ds) => ds,
        ace_source_cte::CtProductError::Plan(p) => vec![Diagnostic::new(
            ace_core::atom_text::Stage::Text,
            "source.cte.plan",
            format!("{p}"),
            ace_core::atom_text::Span { start: 0, end: 0 },
        )],
    })?;
    let compiled = CompiledSource {
        profile: "scalar",
        input_hash: out.input_hash,
        atom_text: out.atom_text,
        checked: out.checked,
    };
    Ok(ProductCompile {
        lane: CompileLane::Facts,
        applied: Vec::new(),
        supply: out.supply,
        instances: out.instances,
        program: out.program,
        compiled,
    })
}

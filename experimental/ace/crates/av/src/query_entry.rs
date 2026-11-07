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

use ace_core::atom_text::Diagnostic;
use ace_core::source::CompiledSource;
use ace_core::verify::CheckedModule;
use ace_source_query::{compile_scalar_product, CompileLane, ProductCompile, SourceQuerySession};

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

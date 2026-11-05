//! PLAN-934 T-08: the product seam for the fact-driven scalar source entry.
//!
//! The AV probe's scalar source compilation can go through the shared SQ-A
//! entry (`compile_via_facts`): per-body query facts → fact-supplied bind →
//! emitted HIR → real verification. The emitted HIR is byte-identical to
//! the legacy `compile_scalar_source` result for the finite surface, so a
//! consumer that switches to this seam keeps the same executable artifact
//! while the legacy typecheck/synthesis path no longer produces its types.

use ace_core::source::SourceProfile;
use ace_core::verify::CheckedModule;
use ace_source_query::{compile_via_facts, SourceQuerySession};

/// Compile one ordinary scalar source through the SQ-A fact layer.
///
/// Each call owns a fresh query session (cold). A warm/restore host keeps
/// its `SourceQuerySession` and calls [`compile_via_facts`] directly.
pub fn compile_scalar_source_via_facts(
    src: &str,
    module_path: &str,
    capabilities: &[String],
) -> Result<CheckedModule, String> {
    let mut session =
        SourceQuerySession::new("ace-av", module_path).with_capabilities(capabilities.iter().cloned());
    let compiled = compile_via_facts(
        src,
        module_path,
        capabilities,
        "ace-av",
        module_path,
        &mut session,
        SourceProfile::Scalar,
    )
    .map_err(|e| e.to_string())?;
    Ok(compiled.checked)
}

/// The fact-driven entry plus its producer-side report (used by tests and
/// by hosts that need the hit/miss provenance).
pub fn compile_scalar_source_via_facts_reported(
    src: &str,
    module_path: &str,
    capabilities: &[String],
    session: &mut SourceQuerySession,
) -> Result<ace_source_query::FactCompiled, String> {
    compile_via_facts(
        src,
        module_path,
        capabilities,
        "ace-av",
        module_path,
        session,
        SourceProfile::Scalar,
    )
    .map_err(|e| e.to_string())
}
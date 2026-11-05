//! PLAN-934 T-08: the shared ordinary-source consumer entry.
//!
//! Every consumer end (AV execution, native source worker, A2X store and
//! the Rust/C emitters) needs the same thing from the frontend: one
//! verified `CheckedModule` whose types came from the query facts. This
//! entry is that single door:
//!
//! ```text
//! parse → query facts per body → fact-supplied bind → emit_resolved_parts
//!       → verify (real HIR verification) → CheckedModule
//! ```
//!
//! No consumer invents its own compile path, and the HIR is the SAME
//! artifact the legacy entry produces (byte-identical for the finite
//! surface, which is what lets every downstream end share it).

use crate::apply::{bind_with_facts, AppliedFn, BindFactsError, BodyTypeSupply};
use ace_core::atom_text::Diagnostic;
use ace_core::source::{emit_resolved_parts, parse_program_with, SourceParts, SourceProfile};
use ace_core::verify::{verify_scalar, CheckedModule};

/// What the shared entry produced.
pub struct FactCompiled {
    pub profile: &'static str,
    pub input_hash: String,
    pub atom_text: String,
    pub checked: CheckedModule,
    pub applied: Vec<AppliedFn>,
    pub supply: BodyTypeSupply,
}

/// Why the shared entry failed. Every variant is a core/facts verdict, not
/// a consumer workaround.
#[derive(Debug)]
pub enum ConsumerError {
    Parse(String),
    Bind(BindFactsError),
    Emit(Vec<Diagnostic>),
    Verify(Vec<Diagnostic>),
}

impl std::fmt::Display for ConsumerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConsumerError::Parse(m) => write!(f, "parse: {m}"),
            ConsumerError::Bind(e) => write!(f, "bind: {e}"),
            ConsumerError::Emit(ds) => write!(f, "emit: {}", first(ds)),
            ConsumerError::Verify(ds) => write!(f, "verify: {}", first(ds)),
        }
    }
}

fn first(ds: &[Diagnostic]) -> String {
    ds.first()
        .map(|d| format!("{}: {}", d.code, d.message))
        .unwrap_or_else(|| "unknown".into())
}

/// Compile one ordinary scalar source through the fact layer.
pub fn compile_via_facts(
    src: &str,
    module_path: &str,
    capabilities: &[String],
    package: &str,
    module: &str,
    session: &mut crate::session::SourceQuerySession,
    profile: SourceProfile,
) -> Result<FactCompiled, ConsumerError> {
    let program = parse_program_with(src, profile).map_err(|d| ConsumerError::Parse(d.message))?;
    let (resolved, applied, supply) =
        bind_with_facts(&program, capabilities, package, module, session, profile)
            .map_err(ConsumerError::Bind)?;
    let parts =
        emit_resolved_parts(&resolved, src, module_path, profile).map_err(ConsumerError::Emit)?;
    let SourceParts {
        profile: prof,
        input_hash,
        atom_text,
        bundle,
        ..
    } = parts;
    let checked = verify_scalar(bundle).map_err(ConsumerError::Verify)?;
    Ok(FactCompiled {
        profile: prof,
        input_hash,
        atom_text,
        checked,
        applied,
        supply,
    })
}

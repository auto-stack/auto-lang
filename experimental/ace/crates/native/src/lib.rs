//! ac-mn — PLAN-818 MIR-native backend probe.
//!
//! Consumes the PLAN-798 delivered surface read-only (Checked HIR front
//! door, HIR→MIR adapter, `verify_mir`, per-function unit split/cache
//! helpers), the PLAN-783 ManagedStore, and the landed ac-core link
//! facilities. Owns the NEW `CheckedMir → CLIF → per-function COFF →
//! multi-object PE` path, its A09'/A10' unit cache and the parity loop
//! against AV / the old HIR-native reference / independent goldens.
//! Nothing here feeds product paths; the private test ABI is NOT the
//! public AutoABI of PLAN-797.

pub mod backend;
pub mod cache;
pub mod collections_artifacts;
pub mod collections_frontend;
pub mod collections_incremental;
pub mod collections_layout;
pub mod collections_lower;
pub mod collections_memory;
pub mod collections_observe;
pub mod collections_runtime;
pub mod collections_workload;
pub mod config;
pub mod driver;
pub mod emit;
pub mod entry;
pub mod fingerprint;
pub mod incremental;
pub mod layout;
pub mod link_driver;
pub mod lower;
pub mod observe;
pub mod owned_source_incremental;
pub mod resources;
pub mod source_diagnostics;
pub mod symbols;
pub mod typed_entry;
pub mod typed_fixture;
pub mod typed_pipeline;
pub mod typed_runtime;
pub mod typed_session;
pub mod worker;
pub mod workload;

/// Error taxonomy of the MIR-native backend (contract-decision §7.3).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MnError {
    /// Out-of-surface content: unknown MIR version/instruction/profile/
    /// capability/import. Rejected with codegen count 0.
    Unsupported(&'static str, String),
    /// Controlled restore-admission rejection (bad owner/target/profile/
    /// abi/callee-sig/address/manifest) — never a miss, never a rebuild.
    Admit(&'static str, String),
    /// Cranelift emission/verifier failure — a backend error class.
    Backend(String),
    /// Frozen native-side budget breach (`mn.resource.<kind>`).
    Resource(&'static str, String),
    /// Toolchain/executor failures (missing tools, link infra, crashes).
    Infra(String),
    /// PLAN-911: a REAL shared-frontend/verify rejection, carried as the
    /// operation's immutable typed snapshot. Its `code()`/`message()` are
    /// the exact legacy `mn.unsupported.frontend` projection, so default
    /// consumers see no change; `source-v1` consumers read the fact.
    Source(Box<source_diagnostics::SourceDiagnosticSnapshot>),
}

impl MnError {
    pub fn code(&self) -> String {
        match self {
            MnError::Unsupported(k, _) => format!("mn.unsupported.{k}"),
            MnError::Admit(k, _) => format!("mn.admit.{k}"),
            MnError::Backend(_) => "mn.backend.lowering".to_string(),
            MnError::Resource(k, _) => format!("mn.resource.{k}"),
            MnError::Infra(_) => "mn.infra".to_string(),
            MnError::Source(_) => "mn.unsupported.frontend".to_string(),
        }
    }

    pub fn message(&self) -> String {
        match self {
            MnError::Unsupported(_, m) | MnError::Admit(_, m) => m.clone(),
            MnError::Backend(m) | MnError::Infra(m) => m.clone(),
            MnError::Resource(_, m) => m.clone(),
            MnError::Source(s) => s.legacy_message(),
        }
    }

    /// The typed source-diagnostic fact when this error IS one (PLAN-911).
    pub fn source_diagnostic(&self) -> Option<&source_diagnostics::SourceDiagnosticSnapshot> {
        match self {
            MnError::Source(s) => Some(s),
            _ => None,
        }
    }
}

impl std::fmt::Display for MnError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code(), self.message())
    }
}

impl std::error::Error for MnError {}

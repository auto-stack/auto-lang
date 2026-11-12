//! PLAN-911: the native source-diagnostic fact channel (`mn.source-diagnostic/1`).
//!
//! One operation (a cold request, a warm single-body edit, or a fallback cold
//! compile) that the SHARED frontend/verify rejects owns exactly one immutable
//! typed snapshot built at its exit point from the real `Vec<Diagnostic>` the
//! compiler returned — raw stage/code/message/span in producer order, the
//! terminating source profile, the profiles actually attempted and the surface
//! rejections consumed only to escalate. Nothing here re-runs a compiler,
//! re-parses a rendered string, reads a session's "last errors", or picks a
//! cause by what some oracle expected: `primary` is the compiler's first
//! diagnostic, always index 0.
//!
//! The legacy `mn.unsupported.frontend` display is a PROJECTION of this same
//! fact (`legacy_message`), byte-identical to the pre-911 string, so default
//! (non `source-v1`) consumers observe no change.

use crate::MnError;
use ace_core::atom_text::Diagnostic;
use serde::{Deserialize, Serialize};

pub const SD_SCHEMA: &str = "mn.source-diagnostic/1";
pub const SD_MODE: &str = "source-v1";
pub const SD_PRODUCER: &str = "ace-core.source";
pub const SD_SPAN_BASIS: &str = "utf8-byte";
/// Frozen diagnostic partition caps (Plan 911 §5.6): the compiler's real
/// list is never truncated — a breach is a typed `mn.resource.source-diagnostic`.
pub const SD_MAX_CAUSES: usize = 64;
pub const SD_MAX_MESSAGE_UTF8: usize = 4096;
pub const SD_MAX_PAYLOAD_UTF8: usize = 64 * 1024;

/// The source domain whose compile produced the terminal diagnostics.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SourceProfile {
    Scalar,
    /// PLAN-999 (T-03): the core-i32 surface — attempted after scalar
    /// surface-rejects (it owns `while`/`loop`); Core-domain bundles lower
    /// through the scalar arm.
    Core,
    Record,
    TextRecord,
    /// PLAN-961: the numeric source surface (attempted last).
    Numeric,
    /// PLAN-962: the aggregates source surface (numeric superset +
    /// composites; attempted truly last).
    Aggregates,
}

impl SourceProfile {
    pub fn as_str(&self) -> &'static str {
        match self {
            SourceProfile::Scalar => "scalar",
            SourceProfile::Core => "core",
            SourceProfile::Record => "record",
            SourceProfile::TextRecord => "text-record",
            SourceProfile::Numeric => "numeric",
            SourceProfile::Aggregates => "aggregates",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RawSpan {
    pub start: usize,
    pub end: usize,
}

/// One compiler diagnostic, verbatim. `stage` is `Stage::as_str()`
/// (lowercase); the legacy Debug capitalisation is derived for the
/// projection only.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RawCause {
    pub stage: String,
    pub code: String,
    pub message: String,
    pub span: RawSpan,
}

impl RawCause {
    pub fn from_diagnostic(d: &Diagnostic) -> Self {
        RawCause {
            stage: d.stage.as_str().to_string(),
            code: d.code.clone(),
            message: d.message.clone(),
            span: RawSpan {
                start: d.span.start,
                end: d.span.end,
            },
        }
    }

    /// `format!("{:?}", Stage)` of the producer — the pre-911 display used
    /// the Debug name, so the projection must too.
    fn stage_debug_name(&self) -> &str {
        match self.stage.as_str() {
            "text" => "Text",
            "bind" => "Bind",
            "verify" => "Verify",
            "capability" => "Capability",
            "backend" => "Backend",
            "link" => "Link",
            "run" => "Run",
            other => other,
        }
    }

    /// The exact legacy `mn.unsupported.frontend` message for this cause.
    pub fn legacy_projection(&self) -> String {
        format!(
            "{}/{}: {}",
            self.stage_debug_name(),
            self.code,
            self.message
        )
    }
}

/// A profile that rejected on its surface (`source.unsupported` out of the
/// Text stage) and was therefore escalated past — recorded, never a primary.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SurfaceRejection {
    pub profile: SourceProfile,
    pub codes: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum OperationKind {
    Cold,
    WarmEdit,
    FallbackCold,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditRegion {
    pub start: u64,
    pub end: u64,
    pub insert_len: u64,
}

/// What this snapshot is bound to: the request/step/candidate that failed.
/// A warm edit names its base (the session's accepted prefix) and the
/// candidate the compiler actually saw.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Operation {
    pub kind: OperationKind,
    pub step: Option<u64>,
    pub base_sha256: Option<String>,
    pub candidate_sha256: String,
    pub session_revision: Option<u64>,
    pub edit: Option<EditRegion>,
}

impl Operation {
    pub fn cold(src: &str) -> Self {
        Operation {
            kind: OperationKind::Cold,
            step: None,
            base_sha256: None,
            candidate_sha256: sha256_hex(src.as_bytes()),
            session_revision: None,
            edit: None,
        }
    }

    pub fn warm_edit(
        base_src: &str,
        candidate_src: &str,
        session_revision: u64,
        edit: EditRegion,
    ) -> Self {
        Operation {
            kind: OperationKind::WarmEdit,
            step: None,
            base_sha256: Some(sha256_hex(base_src.as_bytes())),
            candidate_sha256: sha256_hex(candidate_src.as_bytes()),
            session_revision: Some(session_revision),
            edit: Some(edit),
        }
    }

    pub fn fallback_cold(base_src: &str, candidate_src: &str) -> Self {
        Operation {
            kind: OperationKind::FallbackCold,
            step: None,
            base_sha256: Some(sha256_hex(base_src.as_bytes())),
            candidate_sha256: sha256_hex(candidate_src.as_bytes()),
            session_revision: None,
            edit: None,
        }
    }

    pub fn with_step(mut self, step: u64) -> Self {
        self.step = Some(step);
        self
    }
}

/// The real compile failure as the escalating front door observed it.
#[derive(Clone, Debug)]
pub struct CompileFailure {
    pub profile: SourceProfile,
    pub attempted: Vec<SourceProfile>,
    pub surface_rejections: Vec<SurfaceRejection>,
    pub diagnostics: Vec<Diagnostic>,
}

/// The immutable, operation-owned snapshot (`mn.source-diagnostic/1`).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceDiagnosticSnapshot {
    pub schema: String,
    pub mode: String,
    pub producer: String,
    pub profile: SourceProfile,
    pub attempted: Vec<SourceProfile>,
    pub surface_rejections: Vec<SurfaceRejection>,
    /// The path the PRODUCER received (`mn.at` / `probe.at`), not the case's
    /// logical path — the two are related by the source hash, never renamed.
    pub path: String,
    pub source_sha256: String,
    pub source_len: usize,
    pub span_basis: String,
    pub causes: Vec<RawCause>,
    /// Always 0: the compiler's own order decides, never an oracle.
    pub primary: usize,
    pub operation: Operation,
}

impl SourceDiagnosticSnapshot {
    /// Build the snapshot at the operation's exit from the compiler's real
    /// result. `Err` is the typed budget breach (never a truncated PASS);
    /// an empty diagnostic list is not a snapshot (the caller keeps its
    /// legacy generic error for that impossible edge).
    pub fn capture(
        src: &str,
        path: &str,
        failure: CompileFailure,
        operation: Operation,
    ) -> Result<Self, MnError> {
        if failure.diagnostics.is_empty() {
            return Err(MnError::Unsupported(
                "frontend",
                "frontend failed".to_string(),
            ));
        }
        if failure.diagnostics.len() > SD_MAX_CAUSES {
            return Err(MnError::Resource(
                "source-diagnostic",
                format!(
                    "{} diagnostics exceed the frozen {SD_MAX_CAUSES}-cause cap",
                    failure.diagnostics.len()
                ),
            ));
        }
        for (i, d) in failure.diagnostics.iter().enumerate() {
            if d.message.len() > SD_MAX_MESSAGE_UTF8 {
                return Err(MnError::Resource(
                    "source-diagnostic",
                    format!(
                        "diagnostic {i} message is {} UTF-8 bytes, above the {SD_MAX_MESSAGE_UTF8}-byte cap",
                        d.message.len()
                    ),
                ));
            }
        }
        let snap = SourceDiagnosticSnapshot {
            schema: SD_SCHEMA.to_string(),
            mode: SD_MODE.to_string(),
            producer: SD_PRODUCER.to_string(),
            profile: failure.profile,
            attempted: failure.attempted,
            surface_rejections: failure.surface_rejections,
            path: path.to_string(),
            source_sha256: sha256_hex(src.as_bytes()),
            source_len: src.len(),
            span_basis: SD_SPAN_BASIS.to_string(),
            causes: failure
                .diagnostics
                .iter()
                .map(RawCause::from_diagnostic)
                .collect(),
            primary: 0,
            operation,
        };
        let payload = snap.to_json_string();
        if payload.len() > SD_MAX_PAYLOAD_UTF8 {
            return Err(MnError::Resource(
                "source-diagnostic",
                format!(
                    "snapshot payload is {} UTF-8 bytes, above the {SD_MAX_PAYLOAD_UTF8}-byte cap",
                    payload.len()
                ),
            ));
        }
        Ok(snap)
    }

    pub fn primary_cause(&self) -> &RawCause {
        &self.causes[self.primary]
    }

    /// The legacy `mn.unsupported.frontend` message: the projection of the
    /// primary cause, byte-identical to the pre-911 `request()` string.
    pub fn legacy_message(&self) -> String {
        self.primary_cause().legacy_projection()
    }

    pub fn to_json(&self) -> serde_json::Value {
        serde_json::to_value(self).expect("snapshot serializes")
    }

    pub fn to_json_string(&self) -> String {
        serde_json::to_string(self).expect("snapshot serializes")
    }
}

/// The single conversion entry every native exit uses for a real frontend
/// failure: snapshot on success, the typed budget/legacy error otherwise.
/// Branches cannot drop the cause because it rides inside the `MnError`.
pub fn mn_error_from_compile_failure(
    src: &str,
    path: &str,
    failure: CompileFailure,
    operation: Operation,
) -> MnError {
    match SourceDiagnosticSnapshot::capture(src, path, failure, operation) {
        Ok(snap) => MnError::Source(Box::new(snap)),
        Err(e) => e,
    }
}

/// Why a warm edit was refused — the Rust enum, serialized directly;
/// never derived from the rendered reason text.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RefusalKind {
    /// Real language diagnostics from the shared frontend/verify: terminal.
    LanguageDiagnostics,
    /// The syntax increment engine could not converge (legal edit, engine
    /// limit) — the only kind the contract lets fall back.
    SyntaxNotConverged,
    /// Interface / declaration-context drift (the retained environment's
    /// interface gate).
    SignatureOrContextDrift,
    /// Edit shape outside the single-body fast path.
    InvalidEdit,
    /// Candidate / session identity (closed, stale base, cross-session).
    SessionOrCandidate,
    /// Post-frontend MIR lower/admit rejection (already rendered upstream).
    LowerAdmit,
    /// The producer returned no typed kind — impossible on the scalar
    /// session after 911; strict consumers treat it as an error, never as
    /// a language family.
    Untyped,
    /// A frozen budget breach while capturing the fact.
    Resource,
}

impl From<ace_av::session::RefusalKind> for RefusalKind {
    fn from(k: ace_av::session::RefusalKind) -> Self {
        use ace_av::session::RefusalKind as K;
        match k {
            K::LanguageDiagnostics => RefusalKind::LanguageDiagnostics,
            K::SyntaxNotConverged => RefusalKind::SyntaxNotConverged,
            K::SignatureOrContextDrift => RefusalKind::SignatureOrContextDrift,
            K::InvalidEdit => RefusalKind::InvalidEdit,
            K::SessionOrCandidate => RefusalKind::SessionOrCandidate,
            K::LowerAdmit => RefusalKind::LowerAdmit,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RejectPhase {
    Frontend,
    Verify,
    Commit,
}

impl From<ace_av::session::RejectPhase> for RejectPhase {
    fn from(p: ace_av::session::RejectPhase) -> Self {
        use ace_av::session::RejectPhase as P;
        match p {
            P::Frontend => RejectPhase::Frontend,
            P::Verify => RejectPhase::Verify,
            P::Commit => RejectPhase::Commit,
        }
    }
}

/// The typed refusal of one warm edit (rides `EditObservation.refusal`).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditRefusal {
    pub kind: RefusalKind,
    pub phase: Option<RejectPhase>,
    /// Present only for `LanguageDiagnostics` (and then always Some).
    pub source_error: Option<SourceDiagnosticSnapshot>,
    /// The legacy `refused_reason` text (display projection, unchanged).
    pub reason: String,
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut h = Sha256::new();
    h.update(bytes);
    format!("{:x}", h.finalize())
}

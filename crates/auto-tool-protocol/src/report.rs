//! `validate_project` report shape (PLAN-815 §5.5). The report is honest
//! about scope: `summary.checked`/`not_checked` state exactly what was and
//! was not verified — a passing report never implies language parsing,
//! dynamic evaluation, generation, or a build succeeded.

use serde::{Deserialize, Serialize};

/// What a validation finding claims.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    /// Informational observation (no action needed).
    Info,
    /// Suspicious but possibly intentional; validate does not fail.
    Warning,
    /// Request constraint violated; `ok` is false.
    Error,
}

/// One validation finding.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Finding {
    pub severity: Severity,
    /// Stable machine-readable code (e.g. `project-root-missing`).
    pub code: String,
    /// Request field the finding is about, when nameable.
    pub field: Option<String>,
    pub message: String,
}

impl Finding {
    pub fn new(severity: Severity, code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            severity,
            code: code.into(),
            field: None,
            message: message.into(),
        }
    }

    pub fn with_field(mut self, field: impl Into<String>) -> Self {
        self.field = Some(field.into());
        self
    }
}

/// Loss-sensitive echo of the validated request: counts and identities that
/// would change if any field were dropped or mangled crossing the wire.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RequestSummary {
    pub name: Option<String>,
    /// "static" | "dynamic" — request origin arm.
    pub origin: String,
    /// [`GenerationRequest::is_evaluated`] at the host.
    pub is_evaluated: bool,
    /// `frontend_names()` result.
    pub frontends: Vec<String>,
    pub mode: String,
    pub window: Option<(f32, f32)>,
    pub members: usize,
    pub dep_roots: usize,
    pub raw_props: usize,
    pub raw_nodes: usize,
    pub input_sources: usize,
    pub has_output_root: bool,
    pub has_theme_decl: bool,
    /// npm_deps/styles/i18n presence flags.
    pub package_assets_present: [bool; 3],
    /// Exactly what this validation covered.
    pub checked: Vec<String>,
    /// Exactly what it did NOT cover — never empty for honesty.
    pub not_checked: Vec<String>,
}

/// The `validate_project` answer.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ValidateProjectReport {
    /// False when any finding has [`Severity::Error`].
    pub ok: bool,
    pub findings: Vec<Finding>,
    pub summary: RequestSummary,
}

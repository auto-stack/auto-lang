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

/// The `generate_ui` answer (PLAN-815 T-05): what the real generation
/// produced on disk. Counts are measured by walking the output root after
/// generation — never the generator's self-report alone.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GenerationReport {
    /// Absolute output root the artifacts landed under.
    pub output_root: String,
    /// Frontends actually generated (subset of the request's declared
    /// frontends this build serves).
    pub frontends: Vec<String>,
    /// Files written under the output root (post-generation walk).
    pub files_written: u64,
    pub duration_ms: u64,
}

/// How a `run_ui` operation ended (PLAN-815 T-05).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RunOutcome {
    /// The run path completed on its own (service exited / VM UI window
    /// closed / bounded path returned).
    Completed,
    /// A `cancel` (or progress-budget abort behaving as cancellation)
    /// stopped the operation; owned child processes were killed.
    Cancelled,
    /// The run path failed with a real error.
    Failed,
}

impl RunOutcome {
    pub fn as_str(&self) -> &'static str {
        match self {
            RunOutcome::Completed => "completed",
            RunOutcome::Cancelled => "cancelled",
            RunOutcome::Failed => "failed",
        }
    }
}

/// The `build_ui` answer (PLAN-815 T-06): same shape as a run report —
/// a build is a bounded execution with a profile, outcome, duration and
/// possible error.
pub type BuildReport = RunReport;

/// The `run_ui` answer (PLAN-815 T-05): the real run's session result.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RunReport {
    /// Render profile the request resolved to (e.g. `vue`, `vm`).
    pub profile: String,
    pub outcome: RunOutcome,
    pub duration_ms: u64,
    /// Last announced service URL (the [`crate::message::ToolEvent::Launched`]
    /// value), when a foreground service came up.
    pub launch: Option<String>,
    /// Real error text when `outcome` is `failed`.
    pub error: Option<String>,
}

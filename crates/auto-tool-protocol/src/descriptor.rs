//! Tool identity, capabilities, and lifecycle (PLAN-815 §5.3/§5.5).

use serde::{Deserialize, Serialize};

/// Exit code: stdin reached EOF after a clean shutdown.
pub const EXIT_CODE_OK: i32 = 0;
/// Exit code: host-internal failure.
pub const EXIT_CODE_INTERNAL: i32 = 1;
/// Exit code: stream-level fatal failure (bad/oversized frame) — framing
/// sync is lost, the host must stop reading.
pub const EXIT_CODE_FATAL_STREAM: i32 = 2;

/// One operation the tool offers, with its lifecycle semantics declared up
/// front — clients never have to probe for them.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Capability {
    /// Operation name as it appears in
    /// [`ClientBody`](crate::message::ClientBody).
    pub operation: String,
    /// Whether the operation honors `cancel` while in flight.
    pub cancellable: bool,
    /// Whether the operation may read interactive input from stdin beyond
    /// framed protocol messages.
    pub interactive: bool,
}

impl Capability {
    pub fn non_interactive(operation: impl Into<String>, cancellable: bool) -> Self {
        Self {
            operation: operation.into(),
            cancellable,
            interactive: false,
        }
    }
}

/// Lifecycle policy of the tool host.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LifecyclePolicy {
    /// The host shuts down cleanly (exit 0) when stdin reaches EOF.
    pub shutdown_on_eof: bool,
    /// Logs (free-form diagnostics) are written to stderr only; the control
    /// stream (stdout) carries framed protocol messages exclusively.
    pub logs_on_stderr_only: bool,
    /// Exit codes as [code, meaning] pairs, so clients can render them.
    pub exit_codes: Vec<(i32, String)>,
}

impl LifecyclePolicy {
    pub fn contract() -> Self {
        Self {
            shutdown_on_eof: true,
            logs_on_stderr_only: true,
            exit_codes: vec![
                (EXIT_CODE_OK, "clean EOF shutdown".into()),
                (EXIT_CODE_INTERNAL, "internal failure".into()),
                (EXIT_CODE_FATAL_STREAM, "fatal stream/protocol violation".into()),
            ],
        }
    }
}

/// The `describe` answer: identity + version + capability/lifecycle truth.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolDescriptor {
    /// Tool identity, e.g. `auto-ui-tool`.
    pub tool_name: String,
    /// Tool implementation version.
    pub tool_version: String,
    /// Protocol version this host speaks.
    pub protocol_version: u32,
    /// Offered operations with lifecycle semantics.
    pub capabilities: Vec<Capability>,
    /// Host lifecycle contract.
    pub lifecycle: LifecyclePolicy,
}

impl ToolDescriptor {
    /// True when `operation` is listed in the capabilities.
    pub fn offers(&self, operation: &str) -> bool {
        self.capabilities.iter().any(|c| c.operation == operation)
    }
}

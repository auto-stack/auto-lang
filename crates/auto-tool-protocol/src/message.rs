//! Message envelopes (PLAN-815 §5.3). Client → host on the tool's stdin,
//! host → client on its stdout; both framed by [`crate::framing`].

use crate::descriptor::ToolDescriptor;
use crate::error::ToolError;
use crate::report::{GenerationReport, RunReport, ValidateProjectReport};
use auto_project_model::GenerationRequest;
use serde::de::Error as DeError;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value as Json;

pub use crate::report::{Finding, RequestSummary, Severity};

/// Newtype carrying the one-source I-PROJECT request through serde via the
/// data owner's wire codec — the only (de)serialization path for requests;
/// no parallel structure exists in this crate.
#[derive(Debug, Clone, PartialEq)]
pub struct WireRequest(pub GenerationRequest);

impl Serialize for WireRequest {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match auto_project_model::wire::request_to_wire(&self.0) {
            Ok(json) => json.serialize(serializer),
            Err(e) => Err(serde::ser::Error::custom(format!(
                "request not wireable: {e}"
            ))),
        }
    }
}

impl<'de> Deserialize<'de> for WireRequest {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let json = Json::deserialize(deserializer)?;
        match auto_project_model::wire::request_from_wire(&json) {
            Ok(request) => Ok(WireRequest(request)),
            Err(e) => Err(D::Error::custom(format!("malformed request payload: {e}"))),
        }
    }
}

/// `validate_project` request payload.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ValidateProjectRequest {
    pub request: WireRequest,
}

/// Client → host body.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum ClientBody {
    /// Ask for the [`ToolDescriptor`].
    Describe,
    /// Validate I-PROJECT metadata / paths / request constraints.
    ValidateProject { request: WireRequest },
    /// Generate the UI project for the request into its declared output
    /// root (PLAN-815 T-05). Bounded operation: runs to completion.
    GenerateUi { request: WireRequest },
    /// Run the UI project for the request through the real run path
    /// (PLAN-815 T-05): generation steps, owned child processes
    /// (package manager / dev server / API server) or the in-process
    /// VM UI on the OS main thread. Long-running and cancellable;
    /// the dev-server URL is announced via
    /// [`ToolEvent::Launched`] once the foreground child is up.
    RunUi { request: WireRequest },
    /// Forward raw bytes to the stdin of the operation's foreground owned
    /// child (PLAN-815 T-05 interactive channel). The Vite dev server
    /// genuinely consumes this — `q` + Enter terminates it. Protocol-level
    /// control message like `cancel`, not a capability-listed operation;
    /// operations without an interactive child answer with a named error.
    Input { target_id: u64, data: Vec<u8> },
    /// Cancel an in-flight operation (honest answer for non-cancellable
    /// operations is [`ToolError::not_cancellable`]).
    Cancel { target_id: u64 },
}

impl ClientBody {
    /// The operation name this body carries.
    pub fn operation(&self) -> &'static str {
        match self {
            ClientBody::Describe => "describe",
            ClientBody::ValidateProject { .. } => "validate_project",
            ClientBody::GenerateUi { .. } => "generate_ui",
            ClientBody::RunUi { .. } => "run_ui",
            ClientBody::Input { .. } => "input",
            ClientBody::Cancel { .. } => "cancel",
        }
    }
}

/// Client → host envelope.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClientEnvelope {
    pub protocol_version: u32,
    pub id: u64,
    pub body: ClientBody,
}

/// A structured event on the control stream (progress). Free-form logs go
/// to stderr — they never appear here.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum ToolEvent {
    /// Cumulative progress: `done` of `total` (unknown total = None).
    Progress {
        message: String,
        done: u64,
        total: Option<u64>,
    },
    /// The operation's foreground service is up at `url` (PLAN-815 T-05
    /// LaunchSpec signal, e.g. the Vite dev server). The operation stays
    /// in flight until the service exits or is cancelled.
    Launched { url: String },
}

/// Host → client body.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerBody {
    DescribeOk {
        descriptor: ToolDescriptor,
    },
    ValidateOk {
        report: ValidateProjectReport,
    },
    GenerateOk {
        report: GenerationReport,
    },
    RunOk {
        report: RunReport,
    },
    /// Receipt for an accepted `cancel` of an in-flight operation; the
    /// operation itself completes with its own final envelope
    /// (e.g. `RunOk` with `outcome: "cancelled"`).
    Cancelled {
        target_id: u64,
    },
    /// Receipt for accepted `input` bytes forwarded to the target
    /// operation's foreground child stdin.
    InputAck {
        target_id: u64,
        bytes: u64,
    },
    Event {
        event: ToolEvent,
    },
    Error {
        error: ToolError,
    },
}

/// Host → client envelope; `id` mirrors the request id (0 when the request
/// could not be identified at all).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ServerEnvelope {
    pub id: u64,
    pub body: ServerBody,
}

impl ServerEnvelope {
    pub fn error(id: u64, error: ToolError) -> Self {
        Self {
            id,
            body: ServerBody::Error { error },
        }
    }
}

//! Message envelopes (PLAN-815 §5.3). Client → host on the tool's stdin,
//! host → client on its stdout; both framed by [`crate::framing`].

use crate::descriptor::ToolDescriptor;
use crate::error::ToolError;
use crate::report::ValidateProjectReport;
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
}

/// Host → client body.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerBody {
    DescribeOk { descriptor: ToolDescriptor },
    ValidateOk { report: ValidateProjectReport },
    Event { event: ToolEvent },
    Error { error: ToolError },
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

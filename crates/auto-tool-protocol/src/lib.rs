//! # auto-tool-protocol
//!
//! I-TOOL — the light hand-off protocol between a tool host process and its
//! clients (PLAN-815 §5.3). Pure data: protocol version, operation and
//! capability descriptors, budgets, the error taxonomy, message envelopes,
//! and the single framing implementation. No AST, VM, renderer, or Man
//! types cross this boundary; the project payload is the one-source
//! [`auto_project_model::GenerationRequest`] serialized through the data
//! owner's wire codec — this crate never re-declares it.
//!
//! Contract highlights (frozen in `docs/plans/attachments/815/
//! contract-baseline.md` §3–4):
//!
//! - **Control stream vs logs**: stdout of the tool host carries only framed
//!   JSON messages; free-form logs go to stderr. Clients must treat any
//!   non-frame byte on the control stream as a protocol violation.
//! - **Version/capability rejection before side effects**: a request with an
//!   unknown protocol version or an unsupported operation is answered with a
//!   structured error and performs no work.
//! - **Budgets**: frames are bounded by [`budget::Budgets`]; violating the
//!   frame budget is fatal for the stream (sync is lost), while semantic
//!   over-limits surface as [`error::ErrorKind::BudgetExceeded`].
//! - **Exit codes**: 0 = EOF shutdown, 1 = internal error, 2 = stream-level
//!   fatal failure (see [`descriptor::EXIT_CODE_*`]).

pub mod budget;
pub mod descriptor;
pub mod error;
pub mod framing;
pub mod message;
pub mod report;
pub mod version;

pub use budget::{Budgets, DEFAULT_BUDGETS};
pub use descriptor::{
    op_options, Capability, LifecyclePolicy, ToolDescriptor, EXIT_CODE_FATAL_STREAM,
    EXIT_CODE_INTERNAL, EXIT_CODE_OK,
};
pub use error::{ErrorKind, ToolError};
pub use framing::{read_frame, write_frame, FrameError};
pub use message::{
    ClientBody, ClientEnvelope, ServerBody, ServerEnvelope, ToolEvent, ValidateProjectRequest,
    WireRequest,
};
pub use report::{
    Finding, GenerationReport, RequestSummary, RunOutcome, RunReport, Severity,
    ValidateProjectReport,
};
pub use version::PROTOCOL_VERSION;

#[cfg(test)]
mod tests;

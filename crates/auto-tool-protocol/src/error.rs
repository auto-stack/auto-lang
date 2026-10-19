//! Tool error taxonomy (PLAN-815 §5.3): every failure a client can see is a
//! named kind — no stringly-typed protocol errors, no hidden internals.

use serde::{Deserialize, Serialize};

/// Structured error kinds. Version/capability rejections happen before any
/// side effect; `BudgetExceeded` names the violated budget and its limit.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ErrorKind {
    /// The request's protocol version is not spoken by this host.
    UnknownProtocolVersion { client: u32, server: u32 },
    /// The operation name is not in the host's capability list.
    UnknownOperation { operation: String },
    /// A required field is absent from the request payload.
    MissingField { field: String },
    /// A field has the right name but an invalid value.
    InvalidField { field: String, reason: String },
    /// A named budget was exceeded.
    BudgetExceeded {
        budget: String,
        limit: u64,
        observed: u64,
    },
    /// The operation is implemented but not cancellable (see the
    /// capability's `cancellable` flag — this error is the honest answer
    /// to a `cancel` for such operations).
    NotCancellable { operation: String },
    /// Host-internal failure; the message carries no secrets.
    Internal,
}

/// A serializable tool error.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolError {
    pub kind: ErrorKind,
    pub message: String,
}

impl ToolError {
    pub fn new(kind: ErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }

    pub fn unknown_protocol_version(client: u32, server: u32) -> Self {
        Self::new(
            ErrorKind::UnknownProtocolVersion { client, server },
            format!("protocol version {client} is not spoken by this host (server: {server})"),
        )
    }

    pub fn unknown_operation(operation: impl Into<String>) -> Self {
        let operation = operation.into();
        Self::new(
            ErrorKind::UnknownOperation { operation: operation.clone() },
            format!("operation '{operation}' is not offered by this tool"),
        )
    }

    pub fn missing_field(field: impl Into<String>) -> Self {
        let field = field.into();
        Self::new(
            ErrorKind::MissingField { field: field.clone() },
            format!("required field '{field}' is missing"),
        )
    }

    pub fn invalid_field(field: impl Into<String>, reason: impl Into<String>) -> Self {
        let field = field.into();
        Self::new(
            ErrorKind::InvalidField {
                field: field.clone(),
                reason: reason.into(),
            },
            format!("field '{field}' is invalid"),
        )
    }

    pub fn budget_exceeded(budget: impl Into<String>, limit: u64, observed: u64) -> Self {
        let budget = budget.into();
        Self::new(
            ErrorKind::BudgetExceeded {
                budget: budget.clone(),
                limit,
                observed,
            },
            format!("budget '{budget}' exceeded: {observed} > {limit}"),
        )
    }

    pub fn not_cancellable(operation: impl Into<String>) -> Self {
        let operation = operation.into();
        Self::new(
            ErrorKind::NotCancellable { operation: operation.clone() },
            format!("operation '{operation}' is not cancellable"),
        )
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::Internal, message)
    }
}

impl std::fmt::Display for ToolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", kind_label(&self.kind), self.message)
    }
}

impl std::error::Error for ToolError {}

fn kind_label(kind: &ErrorKind) -> &'static str {
    match kind {
        ErrorKind::UnknownProtocolVersion { .. } => "unknown-protocol-version",
        ErrorKind::UnknownOperation { .. } => "unknown-operation",
        ErrorKind::MissingField { .. } => "missing-field",
        ErrorKind::InvalidField { .. } => "invalid-field",
        ErrorKind::BudgetExceeded { .. } => "budget-exceeded",
        ErrorKind::NotCancellable { .. } => "not-cancellable",
        ErrorKind::Internal => "internal",
    }
}

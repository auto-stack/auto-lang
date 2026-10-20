//! # auto-ui-tools
//!
//! UI engineering tool package (PLAN-815). **M815-A scope** (this revision):
//! the real `auto-ui-tool` process hosts the light I-TOOL protocol with two
//! narrow operations — `describe` (identity/version/capabilities/lifecycle)
//! and `validate_project` (I-PROJECT metadata / path / request-constraint
//! checks). It deliberately does *not* claim generation, build, or run
//! support: requests for unimplemented operations are rejected before any
//! side effect, and validation reports list exactly what was not verified
//! (§5.5 honesty contract). The generation/build/run closure lands with
//! M815-B on top of the M814-A SDK.
//!
//! Closure discipline: this crate depends only on the protocol crate and
//! the one-source project model — no auto-man, no language/VM/UI stack.
//! Clients (PLAN-816 CLI) talk to the binary over framed stdio; they must
//! not link this crate as a library dependency.

pub mod resources;
pub mod validate;

use auto_tool_protocol::message::{ClientBody, ClientEnvelope, ServerBody, ServerEnvelope};
use auto_tool_protocol::{
    Capability, LifecyclePolicy, ToolDescriptor, ToolError, EXIT_CODE_FATAL_STREAM,
    EXIT_CODE_INTERNAL, EXIT_CODE_OK, PROTOCOL_VERSION,
};
use std::io::{Read, Write};

/// Tool identity on the wire.
pub const TOOL_NAME: &str = "auto-ui-tool";

/// The A-stage capability list: narrow, honest, non-cancellable,
/// non-interactive. Generation/build/run are NOT listed — requesting them
/// yields `unknown-operation` with zero side effects.
pub fn tool_descriptor() -> ToolDescriptor {
    ToolDescriptor {
        tool_name: TOOL_NAME.to_string(),
        tool_version: env!("CARGO_PKG_VERSION").to_string(),
        protocol_version: PROTOCOL_VERSION,
        capabilities: vec![
            Capability::non_interactive("describe", false),
            Capability::non_interactive("validate_project", false),
        ],
        lifecycle: LifecyclePolicy::contract(),
    }
}

/// Run the tool host over framed stdio until EOF.
///
/// - control stream (`control`) carries framed JSON only;
/// - `log` receives free-form human diagnostics (stderr in the binary);
/// - exit codes follow the frozen contract (0 EOF / 1 internal / 2 stream).
///
/// Message-level failures (unknown version/operation, malformed payload)
/// answer with an error envelope and keep serving; framing failures are
/// fatal because stream sync is lost.
pub fn run_tool<R: Read, W: Write, L: Write>(
    mut input: R,
    mut control: W,
    mut log: L,
    max_frame_bytes: usize,
) -> i32 {
    // Served-request registry: id -> operation. The A stage dispatches
    // synchronously, so `cancel` can only ever target an already-completed
    // operation — answered honestly per operation instead of pretending
    // cancellation is unsupported-by-omission.
    let mut served: std::collections::HashMap<u64, &'static str> = std::collections::HashMap::new();
    loop {
        let frame = match auto_tool_protocol::framing::read_frame(&mut input, max_frame_bytes) {
            Ok(Some(frame)) => frame,
            Ok(None) => return EXIT_CODE_OK,
            Err(e) => {
                let _ = writeln!(log, "fatal: stream violation: {e}");
                return EXIT_CODE_FATAL_STREAM;
            }
        };
        let (id, body) = match classify_and_parse(&frame, &mut log) {
            Ok(parsed) => parsed,
            Err(error) => {
                let id = frame.get("id").and_then(|v| v.as_u64()).unwrap_or(0);
                if let Err(e) = respond(
                    &mut control,
                    &ServerEnvelope::error(id, error),
                    max_frame_bytes,
                    &mut log,
                ) {
                    return e;
                }
                continue;
            }
        };
        let _ = writeln!(log, "accepted request id={id} op={}", body.operation());
        let response = match body {
            ClientBody::Describe => {
                served.insert(id, "describe");
                ServerEnvelope {
                    id,
                    body: ServerBody::DescribeOk {
                        descriptor: tool_descriptor(),
                    },
                }
            }
            ClientBody::ValidateProject { request } => {
                served.insert(id, "validate_project");
                let report = validate::validate_request(&request.0);
                ServerEnvelope {
                    id,
                    body: ServerBody::ValidateOk { report },
                }
            }
            ClientBody::Cancel { target_id } => match served.get(&target_id) {
                Some(op) => ServerEnvelope::error(id, ToolError::not_cancellable(*op)),
                None => ServerEnvelope::error(
                    id,
                    ToolError::invalid_field(
                        "target_id",
                        format!("no in-flight or completed operation with id {target_id}"),
                    ),
                ),
            },
        };
        if let Err(e) = respond(&mut control, &response, max_frame_bytes, &mut log) {
            return e;
        }
    }
}

/// Pre-classify a raw frame into precise error kinds before the typed parse:
/// missing envelope fields, unknown protocol version, then full serde parse
/// (residual failures map to `invalid-field`).
fn classify_and_parse(
    frame: &serde_json::Value,
    log: &mut impl Write,
) -> Result<(u64, ClientBody), ToolError> {
    let object = frame
        .as_object()
        .ok_or_else(|| ToolError::invalid_field("envelope", "frame must be a JSON object"))?;
    for field in ["protocol_version", "id", "body"] {
        if !object.contains_key(field) {
            return Err(ToolError::missing_field(field));
        }
    }
    let version = frame["protocol_version"]
        .as_u64()
        .ok_or_else(|| ToolError::invalid_field("protocol_version", "must be a number"))?;
    if version != PROTOCOL_VERSION as u64 {
        let _ = writeln!(
            log,
            "rejected request: protocol version {version} (server: {PROTOCOL_VERSION})"
        );
        return Err(ToolError::unknown_protocol_version(
            version,
            PROTOCOL_VERSION,
        ));
    }
    let body = frame["body"]
        .as_object()
        .ok_or_else(|| ToolError::invalid_field("body", "must be an object"))?;
    if !body.contains_key("op") {
        return Err(ToolError::missing_field("op"));
    }
    let op = frame["body"]["op"]
        .as_str()
        .ok_or_else(|| ToolError::invalid_field("op", "must be a string"))?;
    // Version/capability rejection BEFORE any work or side effect.
    // `cancel` is a protocol-level control message, not a work operation:
    // it always parses and is answered per its target (NotCancellable /
    // invalid target) — the work operations are gated by the capability
    // list.
    if op != "cancel" && !tool_descriptor().offers(op) {
        return Err(ToolError::unknown_operation(op));
    }
    let envelope: ClientEnvelope = serde_json::from_value(frame.clone()).map_err(|e| {
        ToolError::invalid_field(
            "envelope",
            format!("payload does not match the protocol: {e}"),
        )
    })?;
    Ok((envelope.id, envelope.body))
}

fn respond(
    control: &mut impl Write,
    envelope: &ServerEnvelope,
    max_frame_bytes: usize,
    log: &mut impl Write,
) -> Result<(), i32> {
    let json = serde_json::to_value(envelope).map_err(|e| {
        let _ = writeln!(log, "fatal: response serialization failed: {e}");
        EXIT_CODE_INTERNAL
    })?;
    auto_tool_protocol::framing::write_frame(control, &json, max_frame_bytes).map_err(|e| {
        let _ = writeln!(log, "fatal: control stream write failed: {e}");
        EXIT_CODE_INTERNAL
    })
}

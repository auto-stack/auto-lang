//! PLAN-815 T-07: pure protocol consumer — proves the data crate is
//! consumable independently of any UI/runtime/compiler/Man crate, with the
//! full v1 message surface round-tripping losslessly and the framing
//! boundaries enforcing named rejections on the consumer side.

use auto_project_model::descriptor::SourceInfo;
use auto_project_model::{parse_document, GenerationRequest, RequestMode};
use auto_tool_protocol::budget::Budgets;
use auto_tool_protocol::framing::{parse_frame, write_frame, FrameParse};
use auto_tool_protocol::message::{
    ClientBody, ClientEnvelope, ServerBody, ServerEnvelope, ToolEvent, WireRequest,
};
use auto_tool_protocol::report::{RunOutcome, Severity};
use auto_tool_protocol::{Capability, ErrorKind, ToolError, PROTOCOL_VERSION};

/// Compile-time face: the descriptor type is buildable from the protocol
/// crate alone — no UI dependency for the capability surface.
#[test]
fn consumer_builds_descriptor_from_protocol() {
    let caps = vec![
        Capability::non_interactive("describe", false),
        Capability::non_interactive("validate_project", false),
        Capability::non_interactive("generate_ui", false),
        Capability {
            operation: "build_ui".to_string(),
            cancellable: true,
            interactive: false,
        },
        Capability {
            operation: "run_ui".to_string(),
            cancellable: true,
            interactive: true,
        },
    ];
    assert_eq!(caps.len(), 5);
    assert!(caps.iter().any(|c| c.operation == "build_ui" && c.cancellable));
}

/// Full v1 client message surface round-trips losslessly: every operation
/// body serializes and deserializes with the operation name preserved.
#[test]
fn consumer_full_client_surface_round_trips() {
    let request = test_request();
    let bodies = vec![
        (ClientBody::Describe, "describe"),
        (
            ClientBody::ValidateProject {
                request: WireRequest(request.clone()),
            },
            "validate_project",
        ),
        (
            ClientBody::GenerateUi {
                request: WireRequest(request.clone()),
            },
            "generate_ui",
        ),
        (
            ClientBody::BuildUi {
                request: WireRequest(request.clone()),
            },
            "build_ui",
        ),
        (
            ClientBody::RunUi {
                request: WireRequest(request),
            },
            "run_ui",
        ),
        (ClientBody::Input { target_id: 1, data: vec![113, 10] }, "input"),
        (ClientBody::Cancel { target_id: 1 }, "cancel"),
    ];
    for (body, op_name) in bodies {
        let envelope = ClientEnvelope {
            protocol_version: PROTOCOL_VERSION,
            id: 42,
            body,
        };
        let json = serde_json::to_value(&envelope).expect("serialize");
        assert_eq!(json["body"]["op"], op_name, "op name for {op_name}");
        let back: ClientEnvelope =
            serde_json::from_value(json).expect("deserialize");
        assert_eq!(back, envelope);
        assert_eq!(back.body.operation(), op_name);
    }
}

/// Full v1 server message surface round-trips: every receipt, event, and
/// report type.
#[test]
fn consumer_full_server_surface_round_trips() {
    let envelopes: Vec<ServerEnvelope> = vec![
        ServerEnvelope {
            id: 1,
            body: ServerBody::Cancelled { target_id: 7 },
        },
        ServerEnvelope {
            id: 2,
            body: ServerBody::InputAck {
                target_id: 7,
                bytes: 2,
            },
        },
        ServerEnvelope {
            id: 3,
            body: ServerBody::Event {
                event: ToolEvent::Progress {
                    message: "install".into(),
                    done: 4,
                    total: Some(6),
                },
            },
        },
        ServerEnvelope {
            id: 4,
            body: ServerBody::Event {
                event: ToolEvent::Launched {
                    url: "http://127.0.0.1:5173".into(),
                },
            },
        },
        ServerEnvelope {
            id: 5,
            body: ServerBody::RunOk {
                report: auto_tool_protocol::RunReport {
                    profile: "vm".into(),
                    outcome: RunOutcome::Completed,
                    duration_ms: 970,
                    launch: Some("http://127.0.0.1:9247".into()),
                    error: None,
                },
            },
        },
        ServerEnvelope {
            id: 6,
            body: ServerBody::Error {
                error: ToolError::budget_exceeded("max_progress_bytes_per_op", 1048576, 1048577),
            },
        },
    ];
    for envelope in envelopes {
        let json = serde_json::to_value(&envelope).expect("serialize");
        let back: ServerEnvelope =
            serde_json::from_value(json).expect("deserialize");
        assert_eq!(back, envelope);
    }
}

/// Consumer-side framing: a valid message frames and parses back; an
/// oversized frame is rejected with a named `TooLarge` (not a panic).
#[test]
fn consumer_framing_boundaries() {
    let envelope = ClientEnvelope {
        protocol_version: PROTOCOL_VERSION,
        id: 1,
        body: ClientBody::Describe,
    };
    let mut buf = Vec::new();
    write_frame(&mut buf, &serde_json::to_value(&envelope).unwrap(), 8 << 20)
        .expect("frameable");
    match parse_frame(&buf, 8 << 20) {
        FrameParse::Complete { message, consumed } => {
            assert_eq!(consumed, buf.len());
            assert_eq!(message["id"], 1);
        }
        other => panic!("expected Complete, got {other:?}"),
    }

    // Oversized: the payload exceeds the budget → named TooLarge.
    let mut big = serde_json::json!({"id": 2, "pad": "x".repeat(9 << 20)});
    big["protocol_version"] = serde_json::json!(PROTOCOL_VERSION);
    let mut big_buf = Vec::new();
    write_frame(&mut big_buf, &big, 16 << 20).expect("locally frameable");
    match parse_frame(&big_buf, 8 << 20) {
        FrameParse::TooLarge { .. } => {} // named rejection
        other => panic!("expected TooLarge, got {other:?}"),
    }
}

fn test_request() -> GenerationRequest {
    GenerationRequest::from_descriptor(
        &parse_document(
            "name: \"consumer\"\nrender: \"vue\"\n",
            SourceInfo::inline(),
        )
        .expect("parse"),
    )
}

/// The `Severity` and error taxonomy types are constructible from the
/// protocol crate alone (consumer-side error rendering).
#[test]
fn consumer_error_taxonomy() {
    let err = ToolError::unknown_operation("teleport");
    let json = serde_json::to_value(&err).expect("serialize");
    assert!(json["kind"]["kind"].is_string());
    let errors = [
        ToolError::not_cancellable("describe"),
        ToolError::invalid_field("data", "no interactive child"),
        ToolError::internal("boom"),
    ];
    for e in &errors {
        let round: ToolError =
            serde_json::from_value(serde_json::to_value(e).unwrap()).unwrap();
        assert_eq!(round, *e);
    }
    let _ = Severity::Warning; // Severity is importable + usable
}

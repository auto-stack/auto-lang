//! Protocol contract tests (PLAN-815 T-03): framing round-trip and
//! failure classification, envelope serde round-trip through the
//! one-source request codec, error taxonomy serialization, budget sanity.

use crate::budget::Budgets;
use crate::error::{ErrorKind, ToolError};
use crate::framing::{read_frame, write_frame, FrameError};
use crate::message::{ClientBody, ClientEnvelope, ServerBody, ServerEnvelope, WireRequest};
use crate::report::{Finding, RequestSummary, Severity, ValidateProjectReport};
use crate::{Capability, LifecyclePolicy, ToolDescriptor, PROTOCOL_VERSION};
use auto_project_model::descriptor::SourceInfo;
use auto_project_model::{parse_document, GenerationRequest};
use serde_json::{json, Value as Json};
use std::io::Cursor;

fn sample_request() -> GenerationRequest {
    let descriptor = parse_document(
        "name: \"demo\"\nrender: \"vue\"\nwindow: \"fit\"\n",
        SourceInfo::from_path("proj/pac.at"),
    )
    .expect("parse");
    GenerationRequest::from_descriptor(&descriptor)
}

// ---------------------------------------------------------------------------
// framing
// ---------------------------------------------------------------------------

#[test]
fn frame_round_trip_preserves_message() {
    let message = json!({"op": "describe", "id": 3, "unicode": "中文/emoji-🪟"});
    let mut buffer = Vec::new();
    write_frame(&mut buffer, &message, 1024).expect("write");
    // exactly one frame's bytes were produced
    let mut reader = Cursor::new(buffer);
    let back = read_frame(&mut reader, 1024).expect("read").expect("frame");
    assert_eq!(back, message);
    // stream is exhausted afterwards -> clean EOF
    assert!(read_frame(&mut reader, 1024).expect("read").is_none());
}

#[test]
fn frames_sequence_in_order() {
    let mut buffer = Vec::new();
    for id in 0..3u64 {
        write_frame(&mut buffer, &json!({"id": id}), 1024).unwrap();
    }
    let mut reader = Cursor::new(buffer);
    for id in 0..3u64 {
        let frame = read_frame(&mut reader, 1024).unwrap().expect("frame");
        assert_eq!(frame["id"].as_u64(), Some(id));
    }
    assert!(read_frame(&mut reader, 1024).unwrap().is_none());
}

#[test]
fn buffered_parse_reports_incomplete_not_error() {
    use crate::framing::{parse_frame, FrameParse};
    let mut buffer = Vec::new();
    write_frame(&mut buffer, &json!({"op": "describe"}), 1024).unwrap();
    // feed byte-by-byte: every prefix length must be Incomplete
    for take in 0..buffer.len() {
        match parse_frame(&buffer[..take], 1024) {
            FrameParse::Incomplete { needed } => assert!(needed > take),
            other => panic!("byte {take}: expected Incomplete, got {other:?}"),
        }
    }
    match parse_frame(&buffer, 1024) {
        FrameParse::Complete { message, consumed } => {
            assert_eq!(consumed, buffer.len());
            assert_eq!(message["op"], "describe");
        }
        other => panic!("expected Complete, got {other:?}"),
    }
    // budget applies to the declared length before payload exists
    let over = (7000u32).to_be_bytes();
    match parse_frame(&over, 1024) {
        FrameParse::TooLarge {
            limit: 1024,
            observed: 7000,
        } => {}
        other => panic!("expected TooLarge, got {other:?}"),
    }
}

#[test]
fn oversize_frame_is_rejected_in_both_directions() {
    let big = json!({"blob": "x".repeat(512)});
    let mut buffer = Vec::new();
    assert!(matches!(
        write_frame(&mut buffer, &big, 64),
        Err(FrameError::TooLarge { limit: 64, .. })
    ));

    // hand-build an over-limit prefix: payload never needs to exist
    let mut stream = Vec::new();
    stream.extend_from_slice(&1_000_000u32.to_be_bytes());
    let mut reader = Cursor::new(stream);
    assert!(matches!(
        read_frame(&mut reader, 64).unwrap_err(),
        FrameError::TooLarge {
            limit: 64,
            observed: 1_000_000
        }
    ));
}

#[test]
fn garbage_stream_is_malformed_not_panicking() {
    // truncated prefix
    let mut reader = Cursor::new(vec![0u8, 0]);
    assert!(matches!(
        read_frame(&mut reader, 1024).unwrap_err(),
        FrameError::Malformed { .. }
    ));
    // non-JSON payload
    let payload = b"not json at all";
    let mut stream = vec![];
    stream.extend_from_slice(&(payload.len() as u32).to_be_bytes());
    stream.extend_from_slice(payload);
    let mut reader = Cursor::new(stream);
    assert!(matches!(
        read_frame(&mut reader, 1024).unwrap_err(),
        FrameError::Malformed { .. }
    ));
    // truncated payload (prefix promises more than the stream holds)
    let mut stream = vec![];
    stream.extend_from_slice(&100u32.to_be_bytes());
    stream.extend_from_slice(b"{");
    let mut reader = Cursor::new(stream);
    assert!(matches!(
        read_frame(&mut reader, 1024).unwrap_err(),
        FrameError::Malformed { .. }
    ));
}

// ---------------------------------------------------------------------------
// envelopes + one-source request codec
// ---------------------------------------------------------------------------

#[test]
fn client_envelope_round_trip_with_request() {
    let envelope = ClientEnvelope {
        protocol_version: PROTOCOL_VERSION,
        id: 42,
        body: ClientBody::ValidateProject {
            request: WireRequest(sample_request()),
        },
    };
    let json = serde_json::to_value(&envelope).expect("serialize");
    assert_eq!(json["body"]["op"], "validate_project");
    let back: ClientEnvelope = serde_json::from_value(json).expect("deserialize");
    assert_eq!(back, envelope);
    assert_eq!(back.body.operation(), "validate_project");
}

#[test]
fn client_envelope_all_bodies_round_trip() {
    let cases = vec![
        ClientEnvelope {
            protocol_version: 1,
            id: 1,
            body: ClientBody::Describe,
        },
        ClientEnvelope {
            protocol_version: 1,
            id: 2,
            body: ClientBody::Cancel { target_id: 7 },
        },
    ];
    for envelope in cases {
        let back: ClientEnvelope =
            serde_json::from_value(serde_json::to_value(&envelope).unwrap()).unwrap();
        assert_eq!(back, envelope);
    }
}

#[test]
fn server_envelope_error_round_trip_keeps_kind() {
    let envelope = ServerEnvelope::error(9, ToolError::unknown_protocol_version(99, 1));
    let json = serde_json::to_value(&envelope).expect("serialize");
    assert_eq!(json["body"]["type"], "error");
    assert_eq!(
        json["body"]["error"]["kind"]["kind"],
        "unknown_protocol_version"
    );
    let back: ServerEnvelope = serde_json::from_value(json).expect("deserialize");
    assert_eq!(
        back.body,
        ServerBody::Error {
            error: ToolError::new(
                ErrorKind::UnknownProtocolVersion {
                    client: 99,
                    server: 1
                },
                "protocol version 99 is not spoken by this host (server: 1)"
            )
        }
    );
}

#[test]
fn server_envelope_report_round_trip() {
    let envelope = ServerEnvelope {
        id: 5,
        body: ServerBody::ValidateOk {
            report: ValidateProjectReport {
                ok: false,
                findings: vec![Finding::new(
                    Severity::Warning,
                    "static-unsupported",
                    "request carries 2 unsupported constructs",
                )
                .with_field("origin")],
                summary: RequestSummary {
                    name: Some("demo".into()),
                    origin: "static".into(),
                    is_evaluated: false,
                    frontends: vec!["vue".into()],
                    mode: "generate".into(),
                    window: None,
                    members: 0,
                    dep_roots: 0,
                    raw_props: 0,
                    raw_nodes: 0,
                    input_sources: 0,
                    has_output_root: false,
                    has_theme_decl: false,
                    package_assets_present: [false, false, false],
                    checked: vec!["metadata".into(), "paths".into()],
                    not_checked: vec!["language-parse".into(), "generation".into()],
                },
            },
        },
    };
    let back: ServerEnvelope =
        serde_json::from_value(serde_json::to_value(&envelope).unwrap()).unwrap();
    assert_eq!(back, envelope);
}

#[test]
fn request_with_unwireable_value_fails_serialization_loudly() {
    let mut request = GenerationRequest::dynamic(SourceInfo::inline());
    request.package_assets.npm_deps = Some(auto_val::Value::Lambda("x".into()));
    let envelope = ClientEnvelope {
        protocol_version: 1,
        id: 1,
        body: ClientBody::ValidateProject {
            request: WireRequest(request),
        },
    };
    let err = serde_json::to_value(&envelope).unwrap_err();
    assert!(err.to_string().contains("not wireable"), "{err}");
}

// ---------------------------------------------------------------------------
// descriptor / budget contract
// ---------------------------------------------------------------------------

#[test]
fn descriptor_offers_and_lifecycle_contract() {
    let descriptor = ToolDescriptor {
        tool_name: "auto-ui-tool".into(),
        tool_version: "0.1.0".into(),
        protocol_version: PROTOCOL_VERSION,
        capabilities: vec![
            Capability::non_interactive("describe", false),
            Capability::non_interactive("validate_project", false),
        ],
        lifecycle: LifecyclePolicy::contract(),
    };
    assert!(descriptor.offers("describe") && descriptor.offers("validate_project"));
    assert!(!descriptor.offers("generate"));
    let lifecycle = descriptor.lifecycle;
    assert!(lifecycle.shutdown_on_eof && lifecycle.logs_on_stderr_only);
    assert_eq!(lifecycle.exit_codes.len(), 3);
}

#[test]
fn budget_contract_values_are_sane_and_enforced() {
    let budgets = Budgets::contract();
    assert_eq!(budgets.max_frame_bytes, 8 * 1024 * 1024);
    assert_eq!(budgets.max_progress_bytes_per_op, 1024 * 1024);
    assert_eq!(budgets.max_queue_depth, 128);
    assert!(budgets.check_frame(budgets.max_frame_bytes).is_ok());
    assert_eq!(
        budgets.check_frame(budgets.max_frame_bytes + 1),
        Err(budgets.max_frame_bytes + 1)
    );
}

#[test]
fn error_display_is_stable() {
    assert_eq!(
        ToolError::missing_field("op").to_string(),
        "missing-field: required field 'op' is missing"
    );
    assert_eq!(
        ToolError::budget_exceeded("max_frame_bytes", 8, 9).kind,
        ErrorKind::BudgetExceeded {
            budget: "max_frame_bytes".into(),
            limit: 8,
            observed: 9
        }
    );
}

/// Review R1 F-05: the writer enforces the same nesting envelope the shared
/// reader can parse — a frame this writer emits is always readable.
#[test]
fn r1_writer_enforces_the_transmittable_nesting_envelope() {
    fn nested_arrays(depth: usize) -> Json {
        let mut value = json!(1);
        for _ in 0..depth {
            value = json!([value]);
        }
        value
    }
    let limit = crate::framing::MAX_FRAME_JSON_DEPTH;
    // within: write + read round-trip
    let mut buffer = Vec::new();
    write_frame(&mut buffer, &nested_arrays(limit - 5), 1024).expect("within envelope");
    let mut reader = Cursor::new(buffer);
    let back = read_frame(&mut reader, 1024)
        .expect("readable")
        .expect("frame");
    assert_eq!(back, nested_arrays(limit - 5));
    // beyond: the writer refuses with a named reason
    let mut buffer = Vec::new();
    let err = write_frame(&mut buffer, &nested_arrays(limit + 5), 1024).unwrap_err();
    assert!(
        matches!(err, FrameError::Malformed { ref reason } if reason.contains("envelope")),
        "{err}"
    );
}

/// Review R1 F-03: the version-rejection payload carries the actual client
/// number — u64, never truncated.
#[test]
fn r1_unknown_version_error_carries_the_actual_number() {
    let error = ToolError::unknown_protocol_version(4294967297, 1);
    let json = serde_json::to_value(&error).expect("serialize");
    assert_eq!(json["kind"]["client"], 4294967297u64);
    let back: ToolError = serde_json::from_value(json).expect("deserialize");
    assert_eq!(
        back.kind,
        ErrorKind::UnknownProtocolVersion {
            client: 4294967297,
            server: 1
        }
    );
}

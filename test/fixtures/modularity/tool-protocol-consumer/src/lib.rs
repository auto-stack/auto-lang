//! PLAN-815 T-07: pure protocol consumer — proves the data crate is
//! consumable independently of any UI/runtime/compiler/Man crate, with the
//! full v1 message surface round-tripping losslessly and the framing
//! boundaries enforcing named rejections on the consumer side.

// All code here is test-only (the fixture proves CONSUMABILITY, not a
// product API) — imports stay under cfg(test) so the lib target is
// warning-free (PLAN-815 T-06..08 R1 / F-815-T07-R1-04).
#[cfg(test)]
use auto_project_model::descriptor::SourceInfo;
#[cfg(test)]
use auto_project_model::{parse_document, GenerationRequest};
#[cfg(test)]
use auto_tool_protocol::framing::{parse_frame, write_frame, FrameParse};
#[cfg(test)]
use auto_tool_protocol::message::{
    ClientBody, ClientEnvelope, ServerBody, ServerEnvelope, ToolEvent, WireRequest,
};
#[cfg(test)]
use auto_tool_protocol::report::{RunOutcome, Severity};
#[cfg(test)]
use auto_tool_protocol::Budgets;
#[cfg(test)]
use auto_tool_protocol::{Capability, ToolError, PROTOCOL_VERSION};
#[cfg(test)]
use serde_json::Value as Json;

/// A minimal valid request for the wire round-trips.
#[cfg(test)]
fn test_request() -> GenerationRequest {
    GenerationRequest::from_descriptor(
        &parse_document(
            "name: \"consumer\"
render: \"vue\"
",
            SourceInfo::inline(),
        )
        .expect("parse"),
    )
}

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
    assert!(caps
        .iter()
        .any(|c| c.operation == "build_ui" && c.cancellable));
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
                output: None,
                project: None,
                delivery_root: None,
                strict: None,
            },
            "generate_ui",
        ),
        (
            ClientBody::BuildUi {
                request: WireRequest(request.clone()),
                strict: None,
            },
            "build_ui",
        ),
        (
            ClientBody::RunUi {
                request: WireRequest(request),
                args: Vec::new(),
            },
            "run_ui",
        ),
        (
            ClientBody::Input {
                target_id: 1,
                data: vec![113, 10],
            },
            "input",
        ),
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
        let back: ClientEnvelope = serde_json::from_value(json).expect("deserialize");
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
        let back: ServerEnvelope = serde_json::from_value(json).expect("deserialize");
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
    write_frame(&mut buf, &serde_json::to_value(&envelope).unwrap(), 8 << 20).expect("frameable");
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
        let round: ToolError = serde_json::from_value(serde_json::to_value(e).unwrap()).unwrap();
        assert_eq!(round, *e);
    }
    let _ = Severity::Warning; // Severity is importable + usable
}

/// PLAN-815 T-07（R2 审计缺项）：四种工作回执 —— DescribeOk /
/// ValidateOk / GenerateOk / BuildOk —— 的 wire 往返（此前服务端面只
/// 覆盖五种事件/错误回执）。
#[test]
fn consumer_server_receipts_round_trips() {
    let summary = auto_tool_protocol::report::RequestSummary {
        name: Some("receipt-demo".into()),
        origin: "static".into(),
        is_evaluated: false,
        frontends: vec!["vue".into()],
        mode: "run".into(),
        window: Some((640.0, 480.0)),
        members: 0,
        dep_roots: 1,
        raw_props: 2,
        raw_nodes: 1,
        input_sources: 0,
        has_output_root: true,
        has_theme_decl: false,
        package_assets_present: [false, false, false],
        checked: vec!["metadata".into(), "paths".into()],
        not_checked: vec!["generation".into()],
    };
    let receipts: Vec<ServerEnvelope> = vec![
        ServerEnvelope {
            id: 10,
            body: ServerBody::DescribeOk {
                descriptor: auto_tool_protocol::ToolDescriptor {
                    tool_name: "auto-ui-tool".into(),
                    tool_version: "0.1.0".into(),
                    protocol_version: PROTOCOL_VERSION,
                    capabilities: vec![
                        Capability::non_interactive("describe", false),
                        Capability {
                            operation: "build_ui".into(),
                            cancellable: true,
                            interactive: false,
                        },
                    ],
                    lifecycle: auto_tool_protocol::LifecyclePolicy::contract(),
                },
            },
        },
        ServerEnvelope {
            id: 11,
            body: ServerBody::ValidateOk {
                report: auto_tool_protocol::report::ValidateProjectReport {
                    ok: false,
                    findings: vec![auto_tool_protocol::report::Finding::new(
                        Severity::Error,
                        "project-root-missing",
                        "work operations require an absolute project_root",
                    )],
                    summary: summary.clone(),
                },
            },
        },
        ServerEnvelope {
            id: 12,
            body: ServerBody::GenerateOk {
                report: auto_tool_protocol::report::GenerationReport {
                    output_root: "D:/proj/gen/front".into(),
                    frontends: vec!["vue".into(), "jet".into()],
                    files_written: 41,
                    duration_ms: 267,
                },
            },
        },
        ServerEnvelope {
            id: 13,
            body: ServerBody::BuildOk {
                report: auto_tool_protocol::report::BuildReport {
                    profile: "jet".into(),
                    outcome: RunOutcome::Failed,
                    duration_ms: 218,
                    launch: None,
                    error: Some("jet build failed (exited with 7)".into()),
                },
            },
        },
    ];
    for envelope in receipts {
        let json = serde_json::to_value(&envelope).expect("serialize");
        let back: ServerEnvelope = serde_json::from_value(json).expect("deserialize");
        assert_eq!(back, envelope);
    }
}

/// PLAN-815 T-07（R2 审计缺项）：帧深度边界对照 —— 恰好在可传输包络
/// （`MAX_FRAME_JSON_DEPTH`）内的最深消息可写可读；深一级被具名
/// `Malformed`（"nests deeper than the transmittable envelope"）拒绝，
/// 不是 panic 或静默截断。
#[test]
fn consumer_frame_depth_boundaries() {
    fn nested_arrays(wrappers: usize) -> Json {
        //  single-element arrays around a scalar atom —
        // json_depth counts the atom as level 1, so the result depth is
        // wrappers + 1.
        let mut value = Json::Bool(true);
        for _ in 0..wrappers {
            value = Json::Array(vec![value]);
        }
        value
    }
    let limit = auto_tool_protocol::framing::MAX_FRAME_JSON_DEPTH;
    let budget = Budgets::contract().max_frame_bytes;

    // At the exact limit: writes and parses back losslessly.
    let at_limit = nested_arrays(limit - 1);
    let mut buffer = Vec::new();
    write_frame(&mut buffer, &at_limit, budget).expect("depth == limit is transmittable");
    match parse_frame(&buffer, budget) {
        FrameParse::Complete { message, .. } => assert_eq!(message, at_limit),
        other => panic!("boundary frame must parse: {other:?}"),
    }

    // One level deeper: named Malformed rejection (never a panic).
    let too_deep = nested_arrays(limit);
    let mut buffer = Vec::new();
    match write_frame(&mut buffer, &too_deep, budget) {
        Err(auto_tool_protocol::FrameError::Malformed { reason }) => {
            assert!(
                reason.contains("nests deeper than the transmittable envelope"),
                "named depth rejection: {reason}"
            );
        }
        other => panic!("expected Malformed depth rejection, got {other:?}"),
    }
}

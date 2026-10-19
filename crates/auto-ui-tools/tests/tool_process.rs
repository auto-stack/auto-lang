//! Real-process consumer tests (PLAN-815 T-12 / V-05): spawn the actual
//! `auto-ui-tool` binary, speak framed I-TOOL over its stdio, and verify —
//! describe identity, validate honesty, unknown version/operation rejection
//! with zero side effects, missing fields, cancel semantics, budget
//! enforcement, log/control separation, and loss-sensitive request echo.

use auto_project_model::descriptor::SourceInfo;
use auto_project_model::{parse_document, GenerationRequest, RequestMode};
use auto_tool_protocol::framing::{read_frame, write_frame};
use auto_tool_protocol::message::{ClientBody, ClientEnvelope, ServerBody, ServerEnvelope, WireRequest};
use auto_tool_protocol::report::Severity;
use auto_tool_protocol::{Budgets, ErrorKind};
use serde_json::{json, Value as Json};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};

const BIN: &str = env!("CARGO_BIN_EXE_auto-ui-tool");

struct ToolProcess {
    child: Child,
    stdout_buffer: Vec<u8>,
    read_pos: usize,
    stderr_text: String,
}

impl ToolProcess {
    fn spawn() -> Self {
        let child = Command::new(BIN)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("spawn auto-ui-tool");
        Self {
            child,
            stdout_buffer: Vec::new(),
            read_pos: 0,
            stderr_text: String::new(),
        }
    }

    fn send(&mut self, message: &Json) {
        let stdin = self.child.stdin.as_mut().expect("stdin");
        write_frame(stdin, message, Budgets::contract().max_frame_bytes).expect("write frame");
    }

    fn send_raw(&mut self, bytes: &[u8]) {
        // A stream-fatal frame makes the tool exit while we may still be
        // writing; a broken pipe here is the expected consequence, the
        // assertions live on the exit code and stderr.
        let _ = self
            .child
            .stdin
            .as_mut()
            .expect("stdin")
            .write_all(bytes);
    }

    /// Read exactly one framed response from stdout (blocking,
    /// incremental: partial buffer contents are not stream end).
    fn next_response(&mut self) -> Json {
        let limit = Budgets::contract().max_frame_bytes;
        loop {
            let buffer = self.stdout_buffer[self.read_pos..].to_vec();
            match auto_tool_protocol::framing::parse_frame(&buffer, limit) {
                auto_tool_protocol::framing::FrameParse::Complete { message, consumed } => {
                    self.read_pos += consumed;
                    return message;
                }
                auto_tool_protocol::framing::FrameParse::Malformed { reason } => {
                    panic!("control stream is not framed JSON: {reason}")
                }
                auto_tool_protocol::framing::FrameParse::TooLarge { .. } => {
                    panic!("tool emitted an over-budget frame")
                }
                auto_tool_protocol::framing::FrameParse::Incomplete { .. } => {}
            }
            // Not enough bytes yet: pull more from the pipe.
            let mut chunk = [0u8; 8192];
            let n = self
                .child
                .stdout
                .as_mut()
                .expect("stdout")
                .read(&mut chunk)
                .expect("read stdout");
            if n == 0 {
                panic!("tool closed stdout before answering");
            }
            self.stdout_buffer.extend_from_slice(&chunk[..n]);
        }
    }

    fn close_stdin_and_wait(mut self) -> (i32, String) {
        drop(self.child.stdin.take());
        let mut rest = Vec::new();
        if let Some(mut out) = self.child.stdout.take() {
            out.read_to_end(&mut rest).expect("drain stdout");
        }
        self.stdout_buffer.extend_from_slice(&rest);
        let mut err = String::new();
        if let Some(mut stderr) = self.child.stderr.take() {
            stderr.read_to_string(&mut err).expect("drain stderr");
        }
        let status = self.child.wait().expect("wait");
        (status.code().unwrap_or(-1), err)
    }

    fn kill(mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn envelope(id: u64, body: ClientBody) -> Json {
    serde_json::to_value(ClientEnvelope {
        protocol_version: auto_tool_protocol::PROTOCOL_VERSION,
        id,
        body,
    })
    .expect("serialize envelope")
}

fn parse_response(json: &Json) -> ServerEnvelope {
    serde_json::from_value(json.clone()).expect("typed response")
}

fn temp_project() -> (tempfile::TempDir, GenerationRequest) {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(dir.path().join("pac.at"), "name: \"demo\"\nrender: \"vue\"\n").unwrap();
    std::fs::create_dir(dir.path().join("front")).unwrap();
    std::fs::write(dir.path().join("front").join("main.at"), "// demo").unwrap();
    let request = GenerationRequest::from_descriptor(
        &parse_document(
            "name: \"联系 demo\"\nrender: \"vue\"\nwindow: \"1280x720\"\nfront: \"./front\"\nshadcn: off\ndep \"local\" { path: \"./front\" }\n",
            SourceInfo::from_path(dir.path().join("pac.at")),
        )
        .expect("parse"),
    );
    (dir, request)
}

// ---------------------------------------------------------------------------
// V-05 positives
// ---------------------------------------------------------------------------

#[test]
fn describe_returns_real_identity_and_capabilities() {
    let mut tool = ToolProcess::spawn();
    tool.send(&envelope(1, ClientBody::Describe));
    let response = parse_response(&tool.next_response());
    assert_eq!(response.id, 1);
    match response.body {
        ServerBody::DescribeOk { descriptor } => {
            assert_eq!(descriptor.tool_name, "auto-ui-tool");
            assert_eq!(descriptor.protocol_version, 1);
            assert_eq!(
                descriptor
                    .capabilities
                    .iter()
                    .map(|c| c.operation.as_str())
                    .collect::<Vec<_>>(),
                vec!["describe", "validate_project"]
            );
            assert!(descriptor.capabilities.iter().all(|c| !c.cancellable && !c.interactive));
            assert!(descriptor.lifecycle.logs_on_stderr_only);
        }
        other => panic!("expected DescribeOk, got {other:?}"),
    }
    let (code, stderr) = tool.close_stdin_and_wait();
    assert_eq!(code, 0, "clean EOF shutdown");
    assert!(!stderr.is_empty(), "logs go to stderr");
}

#[test]
fn validate_ok_report_echoes_request_without_loss() {
    let (_dir, request) = temp_project();
    let mut tool = ToolProcess::spawn();
    tool.send(&envelope(7, ClientBody::ValidateProject {
        request: WireRequest(request),
    }));
    let response = parse_response(&tool.next_response());
    assert_eq!(response.id, 7);
    match response.body {
        ServerBody::ValidateOk { report } => {
            assert!(report.ok, "findings: {:?}", report.findings);
            let summary = &report.summary;
            assert_eq!(summary.name.as_deref(), Some("联系 demo"));
            assert_eq!(summary.frontends, vec!["vue".to_string()]);
            assert_eq!(summary.window, Some((1280.0, 720.0)));
            assert_eq!(summary.dep_roots, 1);
            assert_eq!(summary.origin, "static");
            assert!(summary.is_evaluated);
            // honesty: scope declarations are explicit and non-empty
            assert_eq!(summary.checked, vec!["metadata", "paths", "request-constraints"]);
            assert!(summary.not_checked.contains(&"generation".to_string()));
            assert!(summary.not_checked.contains(&"build".to_string()));
        }
        other => panic!("expected ValidateOk, got {other:?}"),
    }
    tool.kill();
}

#[test]
fn validate_reports_missing_paths_and_run_constraint() {
    let (_dir, mut request) = temp_project();
    request.project_root = Some(PathBuf::from("Z:/definitely/not/here"));
    request.input_sources.push(PathBuf::from("Z:/missing/main.at"));
    request.mode = RequestMode::Run; // no output_root -> constraint error
    let mut tool = ToolProcess::spawn();
    tool.send(&envelope(2, ClientBody::ValidateProject {
        request: WireRequest(request),
    }));
    let response = parse_response(&tool.next_response());
    match response.body {
        ServerBody::ValidateOk { report } => {
            assert!(!report.ok);
            let codes: Vec<&str> = report.findings.iter().map(|f| f.code.as_str()).collect();
            assert!(codes.contains(&"run-without-output-root"));
            assert!(codes.contains(&"project-root-missing"));
            assert!(codes.contains(&"input-source-missing"));
        }
        other => panic!("expected ValidateOk, got {other:?}"),
    }
    tool.kill();
}

#[test]
fn validate_flags_static_unsupported_without_faking_evaluation() {
    let descriptor = parse_document(
        "name: \"x\"\ntitle: \"a${name}b\"\n",
        SourceInfo::inline(),
    )
    .expect("parse");
    let request = GenerationRequest::from_descriptor(&descriptor);
    assert!(!request.is_evaluated());
    let mut tool = ToolProcess::spawn();
    tool.send(&envelope(3, ClientBody::ValidateProject {
        request: WireRequest(request),
    }));
    let response = parse_response(&tool.next_response());
    match response.body {
        ServerBody::ValidateOk { report } => {
            assert!(!report.summary.is_evaluated);
            assert!(report
                .findings
                .iter()
                .any(|f| f.code == "static-unsupported-constructs" && f.severity == Severity::Warning));
        }
        other => panic!("expected ValidateOk, got {other:?}"),
    }
    tool.kill();
}

// ---------------------------------------------------------------------------
// V-05 negatives: version / capability / field rejections, budget, logs
// ---------------------------------------------------------------------------

#[test]
fn unknown_version_is_rejected_then_tool_keeps_serving() {
    let mut tool = ToolProcess::spawn();
    tool.send(&json!({"protocol_version": 999, "id": 5, "body": {"op": "describe"}}));
    let response = parse_response(&tool.next_response());
    match response.body {
        ServerBody::Error { error } => {
            assert_eq!(
                error.kind,
                ErrorKind::UnknownProtocolVersion { client: 999, server: 1 }
            );
        }
        other => panic!("expected Error, got {other:?}"),
    }
    // stream survives; a valid request still works
    tool.send(&envelope(6, ClientBody::Describe));
    assert!(matches!(
        parse_response(&tool.next_response()).body,
        ServerBody::DescribeOk { .. }
    ));
    tool.kill();
}

#[test]
fn unimplemented_operation_is_rejected_with_zero_side_effects() {
    let output_root = tempfile::tempdir().expect("tempdir");
    let mut tool = ToolProcess::spawn();
    tool.send(&json!({
        "protocol_version": 1,
        "id": 9,
        "body": {"op": "generate_ui", "output_root": output_root.path().to_string_lossy()}
    }));
    let response = parse_response(&tool.next_response());
    match response.body {
        ServerBody::Error { error } => {
            assert_eq!(
                error.kind,
                ErrorKind::UnknownOperation { operation: "generate_ui".into() }
            );
        }
        other => panic!("expected Error, got {other:?}"),
    }
    // capability rejection happened before any side effect: the declared
    // output directory was never created
    assert!(!output_root.path().join("gen").exists());
    assert_eq!(std::fs::read_dir(output_root.path()).expect("dir readable").count(), 0);
    tool.kill();
}

#[test]
fn missing_envelope_fields_are_named() {
    let mut tool = ToolProcess::spawn();
    tool.send(&json!({"protocol_version": 1, "id": 10}));
    match parse_response(&tool.next_response()).body {
        ServerBody::Error { error } => {
            assert_eq!(error.kind, ErrorKind::MissingField { field: "body".into() });
        }
        other => panic!("expected Error, got {other:?}"),
    }
    tool.send(&json!({"protocol_version": 1, "id": 11, "body": {}}));
    match parse_response(&tool.next_response()).body {
        ServerBody::Error { error } => {
            assert_eq!(error.kind, ErrorKind::MissingField { field: "op".into() });
        }
        other => panic!("expected Error, got {other:?}"),
    }
    tool.kill();
}

#[test]
fn cancel_answers_per_operation_and_for_unknown_targets() {
    let mut tool = ToolProcess::spawn();
    tool.send(&envelope(20, ClientBody::Describe));
    let _ = tool.next_response();
    // cancel the completed describe: honest not-cancellable naming the op
    tool.send(&envelope(21, ClientBody::Cancel { target_id: 20 }));
    match parse_response(&tool.next_response()).body {
        ServerBody::Error { error } => {
            assert_eq!(
                error.kind,
                ErrorKind::NotCancellable { operation: "describe".into() }
            );
        }
        other => panic!("expected Error, got {other:?}"),
    }
    // cancel an id that never existed: invalid target, not a silent ok
    tool.send(&envelope(22, ClientBody::Cancel { target_id: 404 }));
    match parse_response(&tool.next_response()).body {
        ServerBody::Error { error } => {
            assert!(matches!(error.kind, ErrorKind::InvalidField { field, .. } if field == "target_id"));
        }
        other => panic!("expected Error, got {other:?}"),
    }
    tool.kill();
}

#[test]
fn oversize_frame_is_stream_fatal_with_stderr_diagnostic() {
    let mut tool = ToolProcess::spawn();
    let big = "x".repeat(9 * 1024 * 1024);
    let message = json!({"protocol_version": 1, "id": 30, "body": {"op": "describe", "pad": big}});
    let mut framed = Vec::new();
    write_frame(&mut framed, &message, 16 * 1024 * 1024).expect("locally frameable");
    tool.send_raw(&framed);
    let (code, stderr) = tool.close_stdin_and_wait();
    assert_eq!(code, 2, "stream-fatal exit; stderr: {stderr}");
    assert!(stderr.contains("exceeds budget"), "diagnostic on stderr: {stderr}");
}

#[test]
fn control_stream_stays_framed_while_logs_flow_to_stderr() {
    let mut tool = ToolProcess::spawn();
    for id in 1..=3u64 {
        tool.send(&envelope(id, ClientBody::Describe));
    }
    for id in 1..=3u64 {
        let response = parse_response(&tool.next_response());
        assert_eq!(response.id, id);
    }
    // every byte the tool wrote to stdout parsed as frames (next_response
    // would have panicked otherwise); no log text leaked into the stream.
    let (code, stderr) = tool.close_stdin_and_wait();
    assert_eq!(code, 0);
    assert!(stderr.lines().any(|l| l.contains("accepted request")), "stderr: {stderr}");
}

#[test]
fn binary_version_flag_is_plain_text_outside_the_protocol() {
    let out = Command::new(BIN).arg("--version").output().expect("run --version");
    assert!(out.status.success());
    let text = String::from_utf8_lossy(&out.stdout);
    assert_eq!(text.trim(), format!("auto-ui-tool {}", env!("CARGO_PKG_VERSION")));
}

//! Real-process consumer tests (PLAN-815 T-12 / V-05): spawn the actual
//! `auto-ui-tool` binary, speak framed I-TOOL over its stdio, and verify —
//! describe identity, validate honesty, unknown version/operation rejection
//! with zero side effects, missing fields, cancel semantics, budget
//! enforcement, log/control separation, and loss-sensitive request echo.

use auto_project_model::descriptor::SourceInfo;
use auto_project_model::{parse_document, GenerationRequest, RequestMode};
use auto_tool_protocol::framing::write_frame;
use auto_tool_protocol::message::{
    ClientBody, ClientEnvelope, ServerBody, ServerEnvelope, WireRequest,
};
use auto_tool_protocol::report::{RunOutcome, Severity};
use auto_tool_protocol::{Budgets, ErrorKind};
use serde_json::{json, Value as Json};
use std::hash::{Hash, Hasher};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};

const BIN: &str = env!("CARGO_BIN_EXE_auto-ui-tool");

struct ToolProcess {
    child: Child,
    stdout_buffer: Vec<u8>,
    read_pos: usize,
    /// The tool's stderr lands in a file (PLAN-815 T-05: run paths and
    /// owned children are chatty; an in-band pipe interacts badly with
    /// inherited-stdio grandchildren under load — a file keeps the log
    /// oracle intact with zero backpressure).
    stderr_path: std::path::PathBuf,
}

impl ToolProcess {
    fn spawn() -> Self {
        Self::spawn_in(None)
    }

    /// Spawn with an explicit working directory (review R1 F-04: path
    /// validation must not depend on the tool's cwd).
    fn spawn_in(cwd: Option<&std::path::Path>) -> Self {
        let mut command = Command::new(BIN);
        let stderr_path = std::env::temp_dir().join(format!(
            "auto-ui-tool-stderr-{}-{}.log",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let stderr_file = std::fs::File::create(&stderr_path).expect("create tool stderr log");
        command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::from(stderr_file));
        if let Some(dir) = cwd {
            command.current_dir(dir);
        }
        let child = command.spawn().expect("spawn auto-ui-tool");
        Self {
            child,
            stdout_buffer: Vec::new(),
            read_pos: 0,
            stderr_path,
        }
    }

    /// The tool's stderr accumulated so far (live file read).
    fn stderr_so_far(&self) -> String {
        std::fs::read_to_string(&self.stderr_path).unwrap_or_default()
    }

    fn send(&mut self, message: &Json) {
        let stdin = self.child.stdin.as_mut().expect("stdin");
        write_frame(stdin, message, Budgets::contract().max_frame_bytes).expect("write frame");
    }

    fn send_raw(&mut self, bytes: &[u8]) {
        // A stream-fatal frame makes the tool exit while we may still be
        // writing; a broken pipe here is the expected consequence, the
        // assertions live on the exit code and stderr.
        let _ = self.child.stdin.as_mut().expect("stdin").write_all(bytes);
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
                let status = self.child.try_wait().expect("tool still running?");
                panic!(
                    "tool closed stdout before answering; exit status: {status:?}; stderr tail: {}",
                    {
                        let text = self.stderr_so_far();
                        let lines: Vec<&str> = text.lines().collect();
                        let n = lines.len();
                        lines[n.saturating_sub(12)..].join(" | ")
                    }
                );
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
        let status = self.child.wait().expect("wait");
        // Give the tool a moment to flush its stderr file on the way out.
        for _ in 0..50 {
            if self
                .stderr_so_far()
                .lines()
                .any(|l| l.contains("exiting with code"))
                || status.code().is_some()
            {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
        let err = self.stderr_so_far();
        let _ = std::fs::remove_file(&self.stderr_path);
        (status.code().unwrap_or(-1), err)
    }

    fn kill(mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
        let _ = std::fs::remove_file(&self.stderr_path);
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
    std::fs::write(
        dir.path().join("pac.at"),
        "name: \"demo\"\nrender: \"vue\"\n",
    )
    .unwrap();
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
                vec![
                    "describe",
                    "validate_project",
                    "generate_ui",
                    "build_ui",
                    "run_ui",
                ]
            );
            // T-05: the bounded A ops stay non-cancellable/non-interactive;
            // run_ui is honestly cancellable AND interactive (Vite reads
            // stdin; children are owned and killed on cancel).
            // T-06: build_ui is cancellable (builds can be long) but not
            // interactive (no stdin-consuming build children).
            for capability in &descriptor.capabilities {
                match capability.operation.as_str() {
                    "run_ui" => assert!(capability.cancellable && capability.interactive),
                    "build_ui" => assert!(capability.cancellable && !capability.interactive),
                    other => {
                        assert!(!capability.cancellable, "{other}");
                        assert!(!capability.interactive, "{other}");
                    }
                }
            }
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
    tool.send(&envelope(
        7,
        ClientBody::ValidateProject {
            request: WireRequest(request),
        },
    ));
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
            assert_eq!(
                summary.checked,
                vec!["metadata", "paths", "request-constraints"]
            );
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
    request
        .input_sources
        .push(PathBuf::from("Z:/missing/main.at"));
    request.mode = RequestMode::Run; // no output_root -> constraint error
    let mut tool = ToolProcess::spawn();
    tool.send(&envelope(
        2,
        ClientBody::ValidateProject {
            request: WireRequest(request),
        },
    ));
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

/// T-07 子批3（F-815-T07-B2-R1-03）字段×规则矩阵补齐：`output_root` /
/// `input_sources` 的逐项消费对照经真实 tool 的 validate_project——每条
/// 规则成对正/负控；相对路径锚定=project_root（非工具 cwd：`front/main.at`
/// 只在工程内存在，cwd 侧解析必产 input-source-missing——区分性 oracle）。
/// 与 `validate_reports_missing_paths_and_run_constraint`（三负控）互补：
/// 本测试补 run 模式正控（output_root 在位）、output-root-is-file、
/// input-source-is-directory、相对 input_source 按请求根解析。
#[test]
fn validate_field_rules_output_root_and_input_sources_boundary() {
    fn validate(
        tool: &mut ToolProcess,
        id: u64,
        request: &GenerationRequest,
    ) -> auto_tool_protocol::report::ValidateProjectReport {
        tool.send(&envelope(
            id,
            ClientBody::ValidateProject {
                request: WireRequest(request.clone()),
            },
        ));
        let response = parse_response(&tool.next_response());
        match response.body {
            ServerBody::ValidateOk { report } => report,
            other => panic!("expected ValidateOk (id {id}), got {other:?}"),
        }
    }
    fn codes(report: &auto_tool_protocol::report::ValidateProjectReport) -> Vec<&str> {
        report.findings.iter().map(|f| f.code.as_str()).collect()
    }

    let (dir, base) = temp_project();
    let root = dir.path().to_path_buf();
    let out_dir = root.join("out-target");
    std::fs::create_dir(&out_dir).unwrap();
    let out_file = root.join("out-file.txt");
    std::fs::write(&out_file, "regular file").unwrap();

    let mut tool = ToolProcess::spawn();

    // (1) run 模式正控：output_root（绝对、目录、在盘）满足约束；相对
    // input_source 按请求根解析命中工程内真实文件（cwd 侧不存在）。
    let mut run_request = base.clone();
    run_request.mode = RequestMode::Run;
    run_request.output_root = Some(out_dir.clone());
    run_request
        .input_sources
        .push(PathBuf::from("front/main.at"));
    let report = validate(&mut tool, 10, &run_request);
    let finding_codes = codes(&report);
    assert!(report.ok, "positive control stays green: {finding_codes:?}");
    assert!(!finding_codes.contains(&"run-without-output-root"));
    assert!(!finding_codes.contains(&"output-root-is-file"));
    assert!(!finding_codes.iter().any(|c| c.starts_with("input-source")));
    assert!(report.summary.has_output_root);
    assert_eq!(report.summary.input_sources, 1);

    // (2) output_root 角色负控：指向普通文件 → 具名 Error；generate 模式
    // 不触发 run-without-output-root（约束是 run-only）。
    let mut file_root_request = base.clone();
    file_root_request.output_root = Some(out_file.clone());
    let report = validate(&mut tool, 11, &file_root_request);
    let finding_codes = codes(&report);
    assert!(!report.ok);
    assert!(finding_codes.contains(&"output-root-is-file"));
    assert!(!finding_codes.contains(&"run-without-output-root"));
    assert_eq!(
        report
            .findings
            .iter()
            .find(|f| f.code == "output-root-is-file")
            .expect("finding present")
            .severity,
        Severity::Error
    );

    // (3) input_source 角色负控：声明目录 → 具名 Error（按请求根解析到
    // 工程内真实目录 front/）。
    let mut dir_source_request = base.clone();
    dir_source_request
        .input_sources
        .push(PathBuf::from("front"));
    let report = validate(&mut tool, 12, &dir_source_request);
    assert!(!report.ok);
    assert!(codes(&report).contains(&"input-source-is-directory"));

    tool.kill();
}

#[test]
fn validate_flags_static_unsupported_without_faking_evaluation() {
    let descriptor =
        parse_document("name: \"x\"\ntitle: \"a${name}b\"\n", SourceInfo::inline()).expect("parse");
    let request = GenerationRequest::from_descriptor(&descriptor);
    assert!(!request.is_evaluated());
    let mut tool = ToolProcess::spawn();
    tool.send(&envelope(
        3,
        ClientBody::ValidateProject {
            request: WireRequest(request),
        },
    ));
    let response = parse_response(&tool.next_response());
    match response.body {
        ServerBody::ValidateOk { report } => {
            assert!(!report.summary.is_evaluated);
            assert!(report
                .findings
                .iter()
                .any(|f| f.code == "static-unsupported-constructs"
                    && f.severity == Severity::Warning));
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
                ErrorKind::UnknownProtocolVersion {
                    client: 999,
                    server: 1
                }
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

/// Review R1 F-03: the reported client version is the actual number sent —
/// `4294967297` must not collapse to a truncated `1`.
#[test]
fn version_rejection_reports_the_actual_client_number() {
    let mut tool = ToolProcess::spawn();
    tool.send(&json!({"protocol_version": 4294967297u64, "id": 31, "body": {"op": "describe"}}));
    match parse_response(&tool.next_response()).body {
        ServerBody::Error { error } => {
            assert_eq!(
                error.kind,
                ErrorKind::UnknownProtocolVersion {
                    client: 4294967297,
                    server: 1
                },
                "client version must not be truncated"
            );
        }
        other => panic!("expected Error, got {other:?}"),
    }
    tool.kill();
}

#[test]
fn unimplemented_operation_is_rejected_with_zero_side_effects() {
    let output_root = tempfile::tempdir().expect("tempdir");
    let mut tool = ToolProcess::spawn();
    tool.send(&json!({
        "protocol_version": 1,
        "id": 9,
        "body": {"op": "teleport_ui", "output_root": output_root.path().to_string_lossy()}
    }));
    let response = parse_response(&tool.next_response());
    match response.body {
        ServerBody::Error { error } => {
            assert_eq!(
                error.kind,
                ErrorKind::UnknownOperation {
                    operation: "teleport_ui".into()
                }
            );
        }
        other => panic!("expected Error, got {other:?}"),
    }
    // capability rejection happened before any side effect: the declared
    // output directory was never created
    assert!(!output_root.path().join("gen").exists());
    assert_eq!(
        std::fs::read_dir(output_root.path())
            .expect("dir readable")
            .count(),
        0
    );
    tool.kill();
}

#[test]
fn missing_envelope_fields_are_named() {
    let mut tool = ToolProcess::spawn();
    tool.send(&json!({"protocol_version": 1, "id": 10}));
    match parse_response(&tool.next_response()).body {
        ServerBody::Error { error } => {
            assert_eq!(
                error.kind,
                ErrorKind::MissingField {
                    field: "body".into()
                }
            );
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
                ErrorKind::NotCancellable {
                    operation: "describe".into()
                }
            );
        }
        other => panic!("expected Error, got {other:?}"),
    }
    // cancel an id that never existed: invalid target, not a silent ok
    tool.send(&envelope(22, ClientBody::Cancel { target_id: 404 }));
    match parse_response(&tool.next_response()).body {
        ServerBody::Error { error } => {
            assert!(
                matches!(error.kind, ErrorKind::InvalidField { ref field, .. } if field == "target_id")
            );
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
    assert!(
        stderr.contains("exceeds budget"),
        "diagnostic on stderr: {stderr}"
    );
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
    assert!(
        stderr.lines().any(|l| l.contains("accepted request")),
        "stderr: {stderr}"
    );
}

#[test]
fn binary_version_flag_is_plain_text_outside_the_protocol() {
    let out = Command::new(BIN)
        .arg("--version")
        .output()
        .expect("run --version");
    assert!(out.status.success());
    let text = String::from_utf8_lossy(&out.stdout);
    assert_eq!(
        text.trim(),
        format!("auto-ui-tool {}", env!("CARGO_PKG_VERSION"))
    );
}

// ---------------------------------------------------------------------------
// Review R1 F-04: path validation anchored to the request's project root,
// never the tool's working directory; role mismatches are findings.
// ---------------------------------------------------------------------------

fn validate_in(
    tool: &mut ToolProcess,
    id: u64,
    request: GenerationRequest,
) -> auto_tool_protocol::ValidateProjectReport {
    tool.send(&envelope(
        id,
        ClientBody::ValidateProject {
            request: WireRequest(request),
        },
    ));
    match parse_response(&tool.next_response()).body {
        ServerBody::ValidateOk { report } => report,
        other => panic!("expected ValidateOk, got {other:?}"),
    }
}

/// Same request (absolute project root + relative input_sources) validated
/// from two different working directories → identical, correct result.
#[test]
fn path_validation_is_independent_of_tool_cwd() {
    let dir = tempfile::tempdir().expect("tempdir");
    let root = dir.path();
    std::fs::write(root.join("pac.at"), "name: \"cwd\"\n").unwrap();
    std::fs::create_dir(root.join("front")).unwrap();
    std::fs::write(root.join("front").join("main.at"), "// x").unwrap();

    let mut request = GenerationRequest::from_descriptor(
        &parse_document(
            "name: \"cwd\"\nrender: \"vue\"\n",
            SourceInfo::from_path(root.join("pac.at")),
        )
        .expect("parse"),
    );
    request.input_sources = vec![PathBuf::from("front/main.at")];

    let foreign = tempfile::tempdir().expect("foreign cwd");
    let report_from_foreign = {
        let mut tool = ToolProcess::spawn_in(Some(foreign.path()));
        let report = validate_in(&mut tool, 1, request.clone());
        tool.kill();
        report
    };
    let report_from_project = {
        let mut tool = ToolProcess::spawn_in(Some(root));
        let report = validate_in(&mut tool, 2, request);
        tool.kill();
        report
    };
    assert!(
        report_from_foreign.ok,
        "findings: {:?}",
        report_from_foreign.findings
    );
    assert!(
        report_from_project.ok,
        "findings: {:?}",
        report_from_project.findings
    );
    assert_eq!(
        report_from_foreign, report_from_project,
        "cwd must not change the report"
    );
}

/// Relative paths with no usable root are reported unverifiable — from any
/// cwd — instead of being probed where the tool happens to start.
#[test]
fn relative_paths_without_root_are_reported_not_cwd_probed() {
    let mut request = GenerationRequest::dynamic(SourceInfo::inline());
    request.identity.name = Some("no-root".into());
    request.input_sources = vec![PathBuf::from("front/main.at")];
    request.output_root = Some(PathBuf::from("out"));

    // both directories exist relative to the PROJECT dir; if the tool
    // probed its cwd, one spawn would silently pass
    let project = tempfile::tempdir().expect("project cwd");
    std::fs::create_dir(project.path().join("front")).unwrap();
    std::fs::write(project.path().join("front").join("main.at"), "// x").unwrap();

    let report_a = {
        let mut tool = ToolProcess::spawn_in(Some(project.path()));
        let report = validate_in(&mut tool, 1, request.clone());
        tool.kill();
        report
    };
    let report_b = {
        let mut tool = ToolProcess::spawn(); // test-harness cwd
        let report = validate_in(&mut tool, 2, request);
        tool.kill();
        report
    };
    for report in [&report_a, &report_b] {
        let codes: Vec<&str> = report.findings.iter().map(|f| f.code.as_str()).collect();
        assert!(
            codes.contains(&"relative-path-without-root"),
            "codes: {codes:?}"
        );
        assert!(
            !codes.contains(&"input-source-missing"),
            "must not probe cwd"
        );
    }
    assert_eq!(report_a, report_b);
}

/// Role checks: project_root must be a directory (C11), output_root must
/// not resolve to a regular file, and relative roots are rejected.
#[test]
fn path_roles_are_checked_and_relative_roots_rejected() {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(dir.path().join("result"), "plain file").unwrap();

    // C11: project_root is a regular file
    let mut request = GenerationRequest::dynamic(SourceInfo::inline());
    request.identity.name = Some("c11".into());
    request.project_root = Some(dir.path().join("result"));
    let mut tool = ToolProcess::spawn();
    let report = validate_in(&mut tool, 1, request);
    let codes: Vec<&str> = report.findings.iter().map(|f| f.code.as_str()).collect();
    assert!(codes.contains(&"project-root-is-file"), "codes: {codes:?}");
    assert!(!report.ok);
    tool.kill();

    // output_root resolving onto an existing regular file, from a foreign cwd
    let mut request = GenerationRequest::dynamic(SourceInfo::inline());
    request.identity.name = Some("role".into());
    request.project_root = Some(dir.path().to_path_buf());
    request.output_root = Some(PathBuf::from("result"));
    let foreign = tempfile::tempdir().expect("foreign");
    let mut tool = ToolProcess::spawn_in(Some(foreign.path()));
    let report = validate_in(&mut tool, 2, request);
    let codes: Vec<&str> = report.findings.iter().map(|f| f.code.as_str()).collect();
    assert!(codes.contains(&"output-root-is-file"), "codes: {codes:?}");
    assert!(!report.ok);
    tool.kill();

    // relative project_root cannot anchor anything
    let mut request = GenerationRequest::dynamic(SourceInfo::inline());
    request.identity.name = Some("relroot".into());
    request.project_root = Some(PathBuf::from("relative/root"));
    let mut tool = ToolProcess::spawn();
    let report = validate_in(&mut tool, 3, request);
    let codes: Vec<&str> = report.findings.iter().map(|f| f.code.as_str()).collect();
    assert!(
        codes.contains(&"project-root-not-absolute"),
        "codes: {codes:?}"
    );
    tool.kill();
}

// ---------------------------------------------------------------------------
// Review R2 regressions: controlled rejection of tagless value payloads
// (F-815-R1-02) and Windows cwd-qualified path forms (F-815-R1-04).
// ---------------------------------------------------------------------------

/// A valid frame carrying {"v":1,"extra":2} in three real value positions
/// gets a controlled invalid-field error (never a panic / exit 101), and
/// the tool keeps serving afterwards.
#[test]
fn r2_tagless_value_payload_is_controlled_not_fatal() {
    for (position, inject) in [
        ("npm_deps", serde_json::json!({"v": 1, "extra": 2})),
        (
            "raw value",
            serde_json::json!([{"key": "k", "value": {"v": 1, "extra": 2}, "line": 1}]),
        ),
        (
            "raw node prop",
            serde_json::json!([{"head": "app", "name": null, "raw_args": null,
                "props": [{"key": "k", "value": {"v": 1, "extra": 2}, "line": 1}],
                "children": [], "line": 1}]),
        ),
    ] {
        let mut tool = ToolProcess::spawn();
        let request = patched_request_with(|wire| match position {
            "npm_deps" => {
                wire["request"]["package_assets"]["npm_deps"] = inject;
            }
            "raw value" => {
                wire["request"]["raw"] = inject;
            }
            _ => {
                wire["request"]["raw_nodes"] = inject;
            }
        });
        tool.send(&request);
        let response = parse_response(&tool.next_response());
        match response.body {
            ServerBody::Error { error } => match error.kind {
                ErrorKind::InvalidField { field, reason } => {
                    assert!(
                        reason.contains("value") || reason.contains("'t'"),
                        "{position}: {field}: {reason}"
                    );
                }
                other => panic!("{position}: expected InvalidField, got {other:?}"),
            },
            other => panic!("{position}: expected Error, got {other:?}"),
        }
        // stream survives: describe answers, clean EOF exit 0 (not 101)
        tool.send(&envelope(99, ClientBody::Describe));
        assert!(matches!(
            parse_response(&tool.next_response()).body,
            ServerBody::DescribeOk { .. }
        ));
        let (code, _) = tool.close_stdin_and_wait();
        assert_eq!(code, 0, "{position}: tool must not die on a bad payload");
    }
}

fn patched_request_with(patch: impl FnOnce(&mut serde_json::Value)) -> Json {
    let dir = tempfile::tempdir().expect("tempdir");
    let request = GenerationRequest::from_descriptor(
        &parse_document(
            "name: \"x\"\nrender: \"vue\"\n",
            SourceInfo::from_path(dir.path().join("pac.at")),
        )
        .expect("parse"),
    );
    let mut wire = auto_project_model::wire::request_to_wire(&request).expect("encode");
    patch(&mut wire);
    json!({
        "protocol_version": 1,
        "id": 40,
        "body": {"op": "validate_project", "request": wire}
    })
}

/// Windows drive-relative (`C:dir`) and rooted-without-prefix (`\dir`)
/// forms are never probed against the per-drive process cwd — the report
/// is identical from any working directory and says cwd-qualified-path.
#[cfg(windows)]
#[test]
fn r2_drive_relative_paths_are_cwd_qualified_not_cwd_probed() {
    let dir = tempfile::tempdir().expect("tempdir");
    let root = dir.path().to_path_buf();
    std::fs::create_dir_all(root.join("front")).unwrap();
    std::fs::write(root.join("front").join("main.at"), "// x").unwrap();
    std::fs::write(root.join("result"), "file").unwrap();

    // derive the real drive prefix from the temp path (no C: assumption)
    let drive = root.to_string_lossy();
    let drive = &drive[..drive.find(':').unwrap() + 1];
    let drive_relative = format!("{drive}front/main.at");
    let rooted = r"\front\main.at";

    for raw in [drive_relative.as_str(), rooted] {
        let mut request = GenerationRequest::dynamic(SourceInfo::inline());
        request.identity.name = Some("drive-rel".into());
        request.project_root = Some(root.clone());
        request.input_sources = vec![PathBuf::from(raw)];

        let same_drive_elsewhere =
            tempfile::tempdir_in(root.parent().unwrap()).expect("same-drive cwd");
        let report_a = {
            let mut tool = ToolProcess::spawn_in(Some(&root));
            let r = validate_in(&mut tool, 1, request.clone());
            tool.kill();
            r
        };
        let report_b = {
            let mut tool = ToolProcess::spawn_in(Some(same_drive_elsewhere.path()));
            let r = validate_in(&mut tool, 2, request.clone());
            tool.kill();
            r
        };
        assert_eq!(report_a, report_b, "'{raw}' result depends on cwd");
        let codes: Vec<&str> = report_a.findings.iter().map(|f| f.code.as_str()).collect();
        assert!(
            codes.contains(&"cwd-qualified-path"),
            "'{raw}' codes: {codes:?}"
        );
        assert!(
            !codes.contains(&"input-source-missing"),
            "'{raw}' probed the cwd"
        );
    }
}

// ---------------------------------------------------------------------------
// PLAN-815 T-05 / V-03 (B stage): real execution over the real binary —
// generate_ui artifacts + progress, unserved-render rejection, busy gate,
// input channel, and the real Vite dev-server lifecycle (launch spec from
// real child stdout, cancel killing owned children, logs on stderr only).
// ---------------------------------------------------------------------------

/// A real minimal UI project (002-counter shape): pac.at + src/front/app.at.
fn vue_ui_project() -> (tempfile::TempDir, GenerationRequest) {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::create_dir_all(dir.path().join("src").join("front")).unwrap();
    std::fs::write(
        dir.path().join("pac.at"),
        "name: \"t05demo\"\nrender: \"vue\"\nwindow: \"640x480\"\nfront: \"./src/front\"\n",
    )
    .unwrap();
    std::fs::write(
        dir.path().join("src").join("front").join("app.at"),
        "widget App {\n    model {\n        var count int = 0\n    }\n    view {\n        col {\n            text `Count: ${.count}`\n            button \"+\" { onclick: () => {.count += 1} }\n        }\n    }\n}\n",
    )
    .unwrap();
    let request = GenerationRequest::from_descriptor(
        &parse_document(
            "name: \"t05demo\"\nrender: \"vue\"\nwindow: \"640x480\"\nfront: \"./src/front\"\n",
            SourceInfo::from_path(dir.path().join("pac.at")),
        )
        .expect("parse"),
    );
    (dir, request)
}

/// Read envelopes until one matches; return (matched, all_seen). Progress
/// events for the same op id are expected interleaved noise.
fn next_until(
    tool: &mut ToolProcess,
    mut predicate: impl FnMut(&ServerEnvelope) -> bool,
) -> (ServerEnvelope, Vec<ServerEnvelope>) {
    let mut seen = Vec::new();
    loop {
        let envelope = parse_response(&tool.next_response());
        if predicate(&envelope) {
            seen.push(envelope.clone());
            return (envelope, seen);
        }
        seen.push(envelope);
    }
}

#[test]
#[ignore = "heavy: real cold generation (debug compile of the real chain, minutes); serial evidence run via --ignored"]
fn generate_ui_reports_real_artifacts_with_progress() {
    let (dir, request) = vue_ui_project();
    let mut tool = ToolProcess::spawn();
    tool.send(&envelope(
        50,
        ClientBody::GenerateUi {
            request: WireRequest(request),
            output: None,
            project: None,
            delivery_root: None,
            strict: None,
        },
    ));
    let (final_envelope, seen) = next_until(&mut tool, |e| {
        matches!(e.body, ServerBody::GenerateOk { .. })
            || matches!(e.body, ServerBody::Error { .. })
    });
    assert_eq!(final_envelope.id, 50);
    match final_envelope.body {
        ServerBody::GenerateOk { report } => {
            assert_eq!(report.frontends, vec!["vue".to_string()]);
            assert!(report.files_written > 10, "real artifacts: {report:?}");
            assert!(report.duration_ms > 0);
            // The generation root exists on disk with real files — the
            // report counts a real walk, not a generator self-report.
            assert!(dir.path().join("gen").join("front").join("vue").exists());
            let real_files = walk_files(&dir.path().join("gen"));
            assert_eq!(real_files, report.files_written, "disk matches report");
        }
        other => panic!("expected GenerateOk, got {other:?}"),
    }
    // Progress events arrived on the control stream (framed, same id).
    let progress: Vec<_> = seen
        .iter()
        .filter(|e| matches!(e.body, ServerBody::Event { .. }))
        .collect();
    assert!(!progress.is_empty(), "no progress events: {seen:?}");
    // The tool keeps serving after a completed work op.
    tool.send(&envelope(51, ClientBody::Describe));
    assert!(matches!(
        parse_response(&tool.next_response()).body,
        ServerBody::DescribeOk { .. }
    ));
    let (code, stderr) = tool.close_stdin_and_wait();
    eprintln!("EXIT CODE = {code}; stderr last lines: {}", {
        let lines: Vec<&str> = stderr.lines().collect();
        let n = lines.len();
        lines[n.saturating_sub(6)..].join(" | ")
    });
    assert_eq!(code, 0);
}

fn walk_files(root: &std::path::Path) -> u64 {
    let mut count = 0u64;
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else {
                count += 1;
            }
        }
    }
    count
}

#[test]
fn generate_ui_rejects_unserved_render_before_side_effects() {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::create_dir_all(dir.path().join("front")).unwrap();
    std::fs::write(
        dir.path().join("pac.at"),
        "name: \"x\"\nrender: \"unreal-engine\"\n",
    )
    .unwrap();
    let request = GenerationRequest::from_descriptor(
        &parse_document(
            "name: \"x\"\nrender: \"unreal-engine\"\n",
            SourceInfo::from_path(dir.path().join("pac.at")),
        )
        .expect("parse"),
    );
    let mut tool = ToolProcess::spawn();
    tool.send(&envelope(
        52,
        ClientBody::GenerateUi {
            request: WireRequest(request),
            output: None,
            project: None,
            delivery_root: None,
            strict: None,
        },
    ));
    match parse_response(&tool.next_response()).body {
        ServerBody::Error { error } => {
            assert!(
                matches!(error.kind, ErrorKind::InvalidField { ref field, .. } if field == "request"),
                "named rejection: {error:?}"
            );
        }
        other => panic!("expected Error, got {other:?}"),
    }
    // Zero side effects: nothing was generated anywhere under the project.
    assert!(!dir.path().join("gen").exists());
    tool.kill();
}

#[test]
fn input_targets_are_answered_honestly() {
    let mut tool = ToolProcess::spawn();
    tool.send(&envelope(60, ClientBody::Describe));
    let _ = tool.next_response();
    // Input to a completed bounded op: already completed, named.
    tool.send(&json!({
        "protocol_version": 1,
        "id": 61,
        "body": {"op": "input", "target_id": 60, "data": [104, 105, 10]}
    }));
    match parse_response(&tool.next_response()).body {
        ServerBody::Error { error } => {
            assert!(
                matches!(error.kind, ErrorKind::InvalidField { ref field, .. } if field == "target_id"),
                "{error:?}"
            );
        }
        other => panic!("expected Error, got {other:?}"),
    }
    // Input to an id that never existed.
    tool.send(&json!({
        "protocol_version": 1,
        "id": 62,
        "body": {"op": "input", "target_id": 4040, "data": [104, 105, 10]}
    }));
    match parse_response(&tool.next_response()).body {
        ServerBody::Error { error } => {
            assert!(
                matches!(error.kind, ErrorKind::InvalidField { ref field, .. } if field == "target_id"),
                "{error:?}"
            );
        }
        other => panic!("expected Error, got {other:?}"),
    }
    let (code, _) = tool.close_stdin_and_wait();
    assert_eq!(code, 0);
}

#[test]
fn run_ui_reports_real_failure_for_broken_source() {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::create_dir_all(dir.path().join("src").join("front")).unwrap();
    std::fs::write(
        dir.path().join("pac.at"),
        "name: \"broken\"\nrender: \"vue\"\n",
    )
    .unwrap();
    std::fs::write(
        dir.path().join("src").join("front").join("app.at"),
        "widget App {\n    model {\n        var count int = 0\n", // unclosed
    )
    .unwrap();
    let request = GenerationRequest::from_descriptor(
        &parse_document(
            "name: \"broken\"\nrender: \"vue\"\n",
            SourceInfo::from_path(dir.path().join("pac.at")),
        )
        .expect("parse"),
    );
    let mut tool = ToolProcess::spawn();
    tool.send(&envelope(
        63,
        ClientBody::RunUi {
            request: WireRequest(request),
            args: Vec::new(),
        },
    ));
    let (envelope, _) = next_until(&mut tool, |e| {
        matches!(e.body, ServerBody::RunOk { .. }) || matches!(e.body, ServerBody::Error { .. })
    });
    match envelope.body {
        ServerBody::RunOk { report } => {
            // Real failure surfaced from the real compile path.
            assert_eq!(report.outcome, auto_tool_protocol::RunOutcome::Failed);
            assert!(report.error.as_deref().is_some_and(|e| !e.is_empty()));
        }
        other => panic!("expected RunOk(failed), got {other:?}"),
    }
    tool.kill();
}

/// The real execution chain on a no-backend vue project: real generation,
/// real `pnpm install` child (hosted stdio, progress events on the control
/// stream), then the honest backend failure surfaced from the real run
/// path. The Launched/URL-sniff positive and the interactive-quit path
/// need a backend-served fixture — they land with the T-06 real-project
/// matrix (V-04); the forwarding mechanism itself is covered by the
/// proc_host real-child unit test.
/// Environment: needs pnpm (warm store / network).
#[test]
#[ignore = "heavy+env: real pnpm install; serial evidence run via --ignored"]
fn run_ui_real_install_chain_then_honest_backend_failure() {
    let (_dir, request) = vue_ui_project();
    let mut tool = ToolProcess::spawn();
    tool.send(&envelope(
        70,
        ClientBody::RunUi {
            request: WireRequest(request),
            args: Vec::new(),
        },
    ));
    // Progress events stream while the real install child runs; the run
    // then fails honestly at the backend step (the fixture declares no
    // back contract — start_api_server refuses, matching legacy).
    let (final_run, seen) = next_until(&mut tool, |e| {
        matches!(e.body, ServerBody::RunOk { .. }) || matches!(e.body, ServerBody::Error { .. })
    });
    let progress_steps: Vec<String> = seen
        .iter()
        .filter_map(|e| match &e.body {
            ServerBody::Event {
                event: auto_tool_protocol::ToolEvent::Progress { message, .. },
            } => Some(message.clone()),
            _ => None,
        })
        .collect();
    assert!(
        progress_steps.iter().any(|m| m.contains("install")),
        "real install step announced: {progress_steps:?}"
    );
    match final_run.body {
        ServerBody::RunOk { report } => {
            assert_eq!(report.outcome, auto_tool_protocol::RunOutcome::Failed);
            let error = report.error.expect("real failure text");
            assert!(error.contains("backend"), "honest backend failure: {error}");
            assert_eq!(report.profile, "vue");
        }
        other => panic!("expected RunOk(failed), got {other:?}"),
    }
    // The tool keeps serving; the log stream carried the real child output
    // (never the control stream — every frame parsed above).
    tool.send(&envelope(73, ClientBody::Describe));
    assert!(matches!(
        parse_response(&tool.next_response()).body,
        ServerBody::DescribeOk { .. }
    ));
    let (code, stderr) = tool.close_stdin_and_wait();
    assert_eq!(code, 0);
    assert!(
        stderr.contains("Installing dependencies"),
        "install phase visible on the log stream: {}",
        stderr.lines().take(6).collect::<Vec<_>>().join(" | ")
    );
}

/// Single work operation at a time: while a real run (with a real install
/// child in flight) is running, a second work op is rejected by name BEFORE
/// side effects; the `input` channel delivers bytes to the real foreground
/// child (InputAck); cancel kills the owned children and the slot frees.
/// Environment: needs pnpm.
#[test]
#[ignore = "heavy+env: real run op with real children; serial evidence run via --ignored"]
fn run_ui_single_flight_input_busy_then_cancel() {
    let (_dir, request) = vue_ui_project();
    let mut tool = ToolProcess::spawn();
    tool.send(&envelope(
        80,
        ClientBody::RunUi {
            request: WireRequest(request.clone()),
            args: Vec::new(),
        },
    ));
    let _ = next_until(&mut tool, |e| match &e.body {
        ServerBody::Event {
            event: auto_tool_protocol::ToolEvent::Progress { message, .. },
        } => message.contains("install"),
        _ => false,
    });
    // Second work op while the first runs: named busy rejection.
    tool.send(&envelope(
        81,
        ClientBody::RunUi {
            request: WireRequest(request),
            args: Vec::new(),
        },
    ));
    match parse_response(&tool.next_response()).body {
        ServerBody::Error { error } => {
            assert!(
                matches!(error.kind, ErrorKind::InvalidField { ref field, .. } if field == "request"),
                "busy rejection: {error:?}"
            );
        }
        other => panic!("expected busy Error, got {other:?}"),
    }
    // Input channel against the in-flight op. With a warm store the
    // install child may already have exited by now, so BOTH answers are
    // honest and expected: InputAck (bytes delivered to the live child's
    // stdin) or the named no-interactive-child error. Which arm fires is
    // timing; that each fires by name is the contract.
    tool.send(&json!({
        "protocol_version": 1,
        "id": 82,
        "body": {"op": "input", "target_id": 80, "data": [113, 10]}
    }));
    // F02 rework: the receipt comes from the dedicated writer thread —
    // progress events may interleave, so wait for the receipt by kind.
    let (receipt, _) = next_until(&mut tool, |e| {
        matches!(e.body, ServerBody::InputAck { .. }) || matches!(e.body, ServerBody::Error { .. })
    });
    match receipt.body {
        ServerBody::InputAck { target_id, bytes } => {
            assert_eq!(target_id, 80);
            assert_eq!(bytes, 2);
        }
        ServerBody::Error { error } => {
            assert!(
                matches!(error.kind, ErrorKind::InvalidField { ref field, .. } if field == "data"),
                "expected the honest no-interactive-child error, got {error:?}"
            );
        }
        other => panic!("expected InputAck or named no-child error, got {other:?}"),
    }
    // Cancel the first: owned children die, the receipt answers, the op
    // completes as cancelled, and the slot frees for another work op.
    tool.send(&envelope(83, ClientBody::Cancel { target_id: 80 }));
    let (cancelled, _) = next_until(&mut tool, |e| {
        matches!(e.body, ServerBody::Cancelled { .. })
    });
    match cancelled.body {
        ServerBody::Cancelled { target_id } => assert_eq!(target_id, 80),
        other => panic!("expected Cancelled, got {other:?}"),
    }
    let (final_run, _) = next_until(&mut tool, |e| matches!(e.body, ServerBody::RunOk { .. }));
    match final_run.body {
        ServerBody::RunOk { report } => {
            assert_eq!(report.outcome, auto_tool_protocol::RunOutcome::Cancelled);
        }
        other => panic!("expected RunOk, got {other:?}"),
    }
    // Describe still answers inline (bounded ops bypass the work slot).
    tool.send(&envelope(84, ClientBody::Describe));
    assert!(matches!(
        parse_response(&tool.next_response()).body,
        ServerBody::DescribeOk { .. }
    ));
    let (code, _) = tool.close_stdin_and_wait();
    assert_eq!(code, 0);
}

/// A real minimal VM UI project (render vm): pac.at + src/front/app.at.
fn vm_ui_project() -> (tempfile::TempDir, GenerationRequest) {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::create_dir_all(dir.path().join("src").join("front")).unwrap();
    std::fs::write(
        dir.path().join("pac.at"),
        "name: \"t05vm\"\nrender: \"vm\"\nwindow: \"480x360\"\n",
    )
    .unwrap();
    std::fs::write(
        dir.path().join("src").join("front").join("app.at"),
        "widget App {\n    model {\n        var count int = 0\n    }\n    view {\n        col {\n            text `Count: ${.count}`\n            button \"+\" { onclick: () => {.count += 1} }\n        }\n    }\n}\n",
    )
    .unwrap();
    let request = GenerationRequest::from_descriptor(
        &parse_document(
            "name: \"t05vm\"\nrender: \"vm\"\nwindow: \"480x360\"\n",
            SourceInfo::from_path(dir.path().join("pac.at")),
        )
        .expect("parse"),
    );
    (dir, request)
}

/// R2-03: the REAL interactive success path. A real VM session in the
/// isolated child process: the tool reports `Launched` with the session's
/// ACCURATE MCP endpoint URL (sniffed from the child's real stderr
/// banner), `Input` with `q` REALLY quits the running UI (the child's
/// stdin watcher -> runtime host-exit -> iced closes), the final report is
/// `completed`, the tool keeps serving, and a SECOND VM session (R2-01)
/// runs and cancels cleanly in the same host process. Environment: opens
/// real (brief) UI windows; needs the local runtime only.
#[test]
#[ignore = "heavy: real iced VM sessions (two windows, brief); serial evidence run via --ignored"]
fn run_ui_vm_real_session_launched_input_quit_and_restart() {
    let (dir, request) = vm_ui_project();
    let mut tool = ToolProcess::spawn();

    // --- Session 1: Launched with the accurate URL, then Input-driven
    // interactive quit.
    tool.send(&envelope(
        90,
        ClientBody::RunUi {
            request: WireRequest(request),
            args: Vec::new(),
        },
    ));
    let (launched, seen) = next_until(&mut tool, |e| {
        matches!(
            &e.body,
            ServerBody::Event {
                event: auto_tool_protocol::ToolEvent::Launched { .. }
            }
        )
    });
    assert_eq!(launched.id, 90);
    let url = match launched.body {
        ServerBody::Event {
            event: auto_tool_protocol::ToolEvent::Launched { url },
        } => url,
        other => panic!("expected Launched, got {other:?}"),
    };
    assert!(
        url.starts_with("http://127.0.0.1:"),
        "accurate MCP endpoint URL from the real session: {url}"
    );
    assert!(
        seen.iter().any(|e| matches!(
            &e.body,
            ServerBody::Event {
                event: auto_tool_protocol::ToolEvent::Progress { .. }
            }
        )),
        "progress before launch: {seen:?}"
    );
    // The real interactive quit: `q` over the Input channel ends the
    // actually-running UI session.
    tool.send(&json!({
        "protocol_version": 1,
        "id": 91,
        "body": {"op": "input", "target_id": 90, "data": [113, 10]}
    }));
    let (final_run, _) = next_until(&mut tool, |e| matches!(e.body, ServerBody::RunOk { .. }));
    match final_run.body {
        ServerBody::RunOk { report } => {
            assert_eq!(report.outcome, auto_tool_protocol::RunOutcome::Completed);
            assert_eq!(report.profile, "vm");
            assert!(report.error.is_none(), "teardown clean: {report:?}");
            assert_eq!(report.launch.as_deref(), Some(url.as_str()));
        }
        other => panic!("expected RunOk(completed), got {other:?}"),
    }

    // --- Session 2 (R2-01): a second VM run in the SAME host process
    // works (process isolation), and cancel still closes the window.
    let (dir2, request2) = vm_ui_project();
    tool.send(&envelope(
        92,
        ClientBody::RunUi {
            request: WireRequest(request2),
            args: Vec::new(),
        },
    ));
    let _ = next_until(&mut tool, |e| {
        matches!(
            &e.body,
            ServerBody::Event {
                event: auto_tool_protocol::ToolEvent::Launched { .. }
            }
        )
    });
    tool.send(&envelope(93, ClientBody::Cancel { target_id: 92 }));
    let (cancelled_receipt, _) = next_until(&mut tool, |e| {
        matches!(e.body, ServerBody::Cancelled { .. })
    });
    match cancelled_receipt.body {
        ServerBody::Cancelled { target_id } => assert_eq!(target_id, 92),
        other => panic!("expected Cancelled, got {other:?}"),
    }
    let (final_run2, _) = next_until(&mut tool, |e| matches!(e.body, ServerBody::RunOk { .. }));
    match final_run2.body {
        ServerBody::RunOk { report } => {
            assert_eq!(report.outcome, auto_tool_protocol::RunOutcome::Cancelled);
        }
        other => panic!("expected RunOk(cancelled), got {other:?}"),
    }

    // The tool keeps serving A requests; clean EOF exit 0.
    tool.send(&envelope(94, ClientBody::Describe));
    assert!(matches!(
        parse_response(&tool.next_response()).body,
        ServerBody::DescribeOk { .. }
    ));
    let (code, stderr) = tool.close_stdin_and_wait();
    assert_eq!(code, 0);
    assert!(
        stderr.contains("AutoUI MCP: listening on"),
        "real MCP banner on the log stream: {}",
        stderr.lines().take(6).collect::<Vec<_>>().join(" | ")
    );
    let _ = dir;
    let _ = dir2;
}

// ---------------------------------------------------------------------------
// PLAN-815 R4 distinguishing matrix (F-815-T05-R4-03/04/06/07):
// multi-profile generation, REAL output roots, request-vs-disk field
// source, honest build failure, and owned-tree build cancellation.
// ---------------------------------------------------------------------------

/// Pure layout oracle (default face): the reported roots are the SAME
/// constants the generators write to — jet/ark/vscode under
/// `gen/front/<profile>` (R4-03's counterexample: the old report claimed
/// `android`/`ohos`/`gen/vscode`, none of which exist); tauri covers BOTH
/// halves (front + gen/back/rust); the aggregate root is the deepest
/// shared ancestor.
#[test]
fn generation_layout_roots_match_real_generator_dirs() {
    let root = std::path::Path::new("D:/proj");
    let dirs = |profile: &str| auto_ui_tools::tool_ops::profile_output_dirs(root, profile);
    assert_eq!(dirs("vue"), vec![root.join("gen/front/vue")]);
    assert_eq!(
        dirs("jet"),
        vec![root.join("gen/front/jet")],
        "not android/"
    );
    assert_eq!(dirs("ark"), vec![root.join("gen/front/ark")], "not ohos/");
    assert_eq!(
        dirs("vscode"),
        vec![root.join("gen/front/vscode")],
        "not gen/vscode"
    );
    assert_eq!(
        dirs("tauri"),
        vec![root.join("gen/front/vue"), root.join("gen/back/rust")],
        "tauri covers the Vue front AND the tauri backend"
    );
    // Shared-ancestor aggregation: tauri -> gen/, [vue, jet] -> gen/front,
    // single dir -> itself.
    let (t_root, _) = auto_ui_tools::tool_ops::aggregate_output_root(root, &dirs("tauri"));
    assert_eq!(t_root, root.join("gen"));
    let mut multi = dirs("vue");
    multi.extend(dirs("jet"));
    let (m_root, _) = auto_ui_tools::tool_ops::aggregate_output_root(root, &multi);
    assert_eq!(m_root, root.join("gen/front"));
    let (s_root, _) = auto_ui_tools::tool_ops::aggregate_output_root(root, &dirs("vue"));
    assert_eq!(s_root, root.join("gen/front/vue"));
}

/// R4-07: a Multi [vue, jet] request generates EVERY declared target (the
/// old code took frontends[0] and reported the full set); the report's
/// output_root is the real shared root and files_written matches a real
/// walk over BOTH per-profile roots. Real generation chain — serial.
#[test]
#[ignore = "heavy: real cold generation (vue+jet); serial evidence run via --ignored"]
fn generate_ui_multi_profile_generates_every_declared_target() {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::create_dir_all(dir.path().join("src").join("front")).unwrap();
    let pac = "name: \"multi\"\nrender: [\"vue\", \"jet\"]\nwindow: \"640x480\"\nfront: \"./src/front\"\n";
    std::fs::write(dir.path().join("pac.at"), pac).unwrap();
    std::fs::write(
        dir.path().join("src").join("front").join("app.at"),
        "widget App {\n    model {\n        var count int = 0\n    }\n    view {\n        col {\n            text `Count: ${.count}`\n            button \"+\" { onclick: () => {.count += 1} }\n        }\n    }\n}\n",
    )
    .unwrap();
    let request = GenerationRequest::from_descriptor(
        &parse_document(pac, SourceInfo::from_path(dir.path().join("pac.at"))).expect("parse"),
    );
    assert_eq!(request.frontend_names(), vec!["vue", "jet"]);

    let mut tool = ToolProcess::spawn();
    tool.send(&envelope(
        60,
        ClientBody::GenerateUi {
            request: WireRequest(request),
            output: None,
            project: None,
            delivery_root: None,
            strict: None,
        },
    ));
    let (final_response, _) = next_until(&mut tool, |e| {
        matches!(e.body, ServerBody::GenerateOk { .. })
            || matches!(e.body, ServerBody::Error { .. })
    });
    match final_response.body {
        ServerBody::GenerateOk { report } => {
            assert_eq!(report.frontends, vec!["vue".to_string(), "jet".to_string()]);
            // BOTH real roots exist with real files (jet previously missing).
            let vue_root = dir.path().join("gen/front/vue");
            let jet_root = dir.path().join("gen/front/jet");
            assert!(vue_root.is_dir(), "vue root missing");
            assert!(
                jet_root.is_dir(),
                "jet root missing (R4-07: only the first target ran)"
            );
            assert!(walk_files(&vue_root) > 5);
            assert!(walk_files(&jet_root) > 5);
            // The reported root is the REAL shared root; the count is a
            // real walk over the union.
            assert!(report.output_root.replace('\\', "/").ends_with("gen/front"));
            assert_eq!(
                report.files_written,
                walk_files(&vue_root) + walk_files(&jet_root),
                "report counts the union of both real roots"
            );
        }
        other => panic!("expected GenerateOk, got {other:?}"),
    }
    let (code, _) = tool.close_stdin_and_wait();
    assert_eq!(code, 0);
}

/// R4-04: vscode fields come from the EVALUATED REQUEST, not a re-read of
/// the disk pac.at (the review's frozen counterexample: request name
/// `request-name`, disk pac `disk-name` — the old chain used disk-name).
#[test]
#[ignore = "heavy: real cold generation (vscode scaffold); serial evidence run via --ignored"]
fn generate_ui_vscode_fields_come_from_request_not_disk_pac() {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::create_dir_all(dir.path().join("src").join("front")).unwrap();
    std::fs::write(
        dir.path().join("pac.at"),
        "name: \"disk-name\"\nrender: \"vscode\"\nfront: \"./src/front\"\n",
    )
    .unwrap();
    std::fs::write(
        dir.path().join("src").join("front").join("app.at"),
        "widget App {\n    view {\n        col { text `hello` }\n    }\n}\n",
    )
    .unwrap();
    // The request is evaluated from a DIFFERENT declaration snapshot than
    // the disk pac (identity.name = request-name).
    let request = GenerationRequest::from_descriptor(
        &parse_document(
            "name: \"request-name\"\nrender: \"vscode\"\nfront: \"./src/front\"\n",
            SourceInfo::from_path(dir.path().join("pac.at")),
        )
        .expect("parse"),
    );
    let mut tool = ToolProcess::spawn();
    tool.send(&envelope(
        61,
        ClientBody::GenerateUi {
            request: WireRequest(request),
            output: None,
            project: None,
            delivery_root: None,
            strict: None,
        },
    ));
    let (final_response, _) = next_until(&mut tool, |e| {
        matches!(e.body, ServerBody::GenerateOk { .. })
            || matches!(e.body, ServerBody::Error { .. })
    });
    match final_response.body {
        ServerBody::GenerateOk { report } => {
            let package = std::fs::read_to_string(dir.path().join("gen/front/vscode/package.json"))
                .expect("generated package.json");
            assert!(
                package.contains("request-name"),
                "package.json must use the REQUEST identity: {package}"
            );
            assert!(
                !package.contains("disk-name"),
                "package.json must not re-read the disk pac: {package}"
            );
            assert!(report.files_written > 5);
        }
        other => panic!("expected GenerateOk, got {other:?}"),
    }
    let (code, _) = tool.close_stdin_and_wait();
    assert_eq!(code, 0);
}

/// T-07 子批2 V-02/V-04 字段矩阵：一个全字段请求——Unicode+空格工程根与
/// 名称、zh-only 标题链、`theme{}`、npm_deps、styles、i18n locale 文件、
/// 本地 dep 根（带独立 front）、front 节点形态——经真实 tool 生成，逐字段
/// 断言落在生成物上。随后删除主 pac.at，用**同一请求**再生成，输出必须
/// 逐字节一致：请求快照是唯一数据源，无第二次主声明读取（V-02 oracle）。
#[test]
#[ignore = "heavy: real generation chain; serial evidence run via --ignored"]
fn generate_ui_field_matrix_and_pac_snapshot_independence() {
    let dir = tempfile::tempdir().expect("tempdir");
    // V-04: Unicode + space project directory is a first-class project root.
    let root = dir.path().join("示例 app");
    std::fs::create_dir_all(root.join("ui").join("front")).unwrap();
    std::fs::create_dir_all(root.join("vendor").join("matrixdep").join("front")).unwrap();
    std::fs::create_dir_all(root.join("src").join("i18n").join("locales")).unwrap();
    let pac = concat!(
        "name: \"示例应用\"\n",
        "title: \"English Fallback\"\n",
        "title_zh: \"中文标题示例\"\n",
        "render: \"vue\"\n",
        "window: \"640x480\"\n",
        "npm_deps: [\"dayjs@^1.11.10\"]\n",
        "styles: [\"./src/extra.css\"]\n",
        "i18n: [\"src/i18n/locales/zh.json\"]\n",
        "theme: { name: \"matrix-theme\", mode: \"dark\", colors: { primary: \"#2050c0\" } }\n",
        "dep \"matrixdep\" { path: \"./vendor/matrixdep\" }\n",
    );
    std::fs::write(root.join("pac.at"), pac).unwrap();
    std::fs::write(
        root.join("ui").join("front").join("app.at"),
        "widget App {\n    view {\n        col { text `matrix-field-probe` }\n    }\n}\n",
    )
    .unwrap();
    std::fs::write(
        root.join("vendor")
            .join("matrixdep")
            .join("front")
            .join("dep.at"),
        "widget MatrixDepWidget {\n    view {\n        col { text `dep-widget-ok` }\n    }\n}\n",
    )
    .unwrap();
    std::fs::write(
        root.join("src").join("extra.css"),
        "/* matrix style */ .extra { color: red; }\n",
    )
    .unwrap();
    let zh_locale = "{\n  \"greeting\": \"你好，字段矩阵\"\n}\n";
    std::fs::write(
        root.join("src")
            .join("i18n")
            .join("locales")
            .join("zh.json"),
        zh_locale,
    )
    .unwrap();

    let mut request = GenerationRequest::from_descriptor(
        &parse_document(pac, SourceInfo::from_path(root.join("pac.at"))).expect("parse"),
    );
    // Adapter-shaped snapshot: the front NODE form (`app("front")` family) is
    // carried by the request (the wire round-trips it) and wins over the prop
    // form — this project declares NO `front:` prop and has NO src/front, so
    // honoring anything else fails loudly instead of silently probing disk.
    request.front_node_form = Some("./ui/front".into());
    // T-07 子批3（F-815-T07-B2-R1-03）output_root 适用性边界钉：请求声明的
    // 交付目标根是 validate 面（角色/run 约束）与消费方交接元数据，
    // generate_ui 的落盘布局是冻结的 project_root 布局（gen/front/<profile>）
    // ——不按 output_root 静默重定向；报告 output_root 如实回实际根。
    let declared_output_root = root.join("elsewhere-delivery");
    request.output_root = Some(declared_output_root.clone());

    let mut tool = ToolProcess::spawn();
    let send_generate = |tool: &mut ToolProcess, id: u64| {
        tool.send(&envelope(
            id,
            ClientBody::GenerateUi {
                request: WireRequest(request.clone()),
                output: None,
                project: None,
                delivery_root: None,
                strict: None,
            },
        ));
        let (final_response, _) = next_until(tool, |e| {
            matches!(e.body, ServerBody::GenerateOk { .. })
                || matches!(e.body, ServerBody::Error { .. })
        });
        match final_response.body {
            ServerBody::GenerateOk { report } => {
                assert_eq!(report.frontends, vec!["vue".to_string()]);
                assert!(report.files_written > 10, "real artifacts: {report:?}");
                // 布局不被 output_root 重定向（冻结语义）；报告根=实际根。
                assert!(
                    report
                        .output_root
                        .replace('\\', "/")
                        .ends_with("gen/front/vue"),
                    "report root is the real layout root: {}",
                    report.output_root
                );
                assert!(
                    !declared_output_root.exists(),
                    "declared output_root must not become a silent redirect target"
                );
            }
            other => panic!("expected GenerateOk (id {id}), got {other:?}"),
        }
    };

    send_generate(&mut tool, 70);

    // ---- Per-field oracle: every request field lands in the artifacts. ----
    let vue_root = root.join("gen").join("front").join("vue");
    assert!(
        vue_root.is_dir(),
        "generation root under the unicode/space path"
    );
    let package =
        std::fs::read_to_string(vue_root.join("package.json")).expect("generated package.json");
    // identity.name (Unicode, verbatim — frozen legacy emission shape)
    assert!(
        package.contains("\"name\": \"示例应用\""),
        "unicode project name lands verbatim: {package}"
    );
    // npm_deps (request array form "pkg@spec")
    assert!(
        package.contains("\"dayjs\": \"^1.11.10\""),
        "npm_deps land in package.json: {package}"
    );
    // i18n enables the vue-i18n dependency
    assert!(
        package.contains("\"vue-i18n\""),
        "i18n declaration lands in package.json: {package}"
    );
    // zh-only title chain: AUTO_LOCALE defaults to zh → title_zh wins, and a
    // zh-only project (title_zh without relying on title) is not lost.
    let index_html =
        std::fs::read_to_string(vue_root.join("index.html")).expect("generated index.html");
    assert!(
        index_html.contains("<title>中文标题示例</title>"),
        "zh title chain lands in index.html: {index_html}"
    );
    // theme{} composes and publishes the declared palette
    assert!(
        index_html.contains("__AUTO_COMPOSED_THEME__"),
        "theme bootstrap present: {index_html}"
    );
    assert!(
        index_html.contains("matrix-theme"),
        "declared theme name published: {index_html}"
    );
    assert!(
        vue_root
            .join("src")
            .join("assets")
            .join("index.css")
            .is_file(),
        "composed theme renders into src/assets/index.css"
    );
    // styles: copied verbatim into src/styles/ and imported from main.ts
    let extra_css = std::fs::read(vue_root.join("src").join("styles").join("extra.css"))
        .expect("style file copied into src/styles");
    assert_eq!(
        extra_css,
        std::fs::read(root.join("src").join("extra.css")).unwrap(),
        "style file copied byte-for-byte"
    );
    let main_ts =
        std::fs::read_to_string(vue_root.join("src").join("main.ts")).expect("generated main.ts");
    assert!(
        main_ts.contains("./styles/extra.css"),
        "style import lands in main.ts: {main_ts}"
    );
    // i18n: locale file copied byte-for-byte + instance module
    assert_eq!(
        std::fs::read(vue_root.join("src").join("locales").join("zh.json")).unwrap(),
        zh_locale.as_bytes(),
        "locale file copied byte-for-byte"
    );
    assert!(
        vue_root.join("src").join("i18n-instance.ts").is_file(),
        "i18n instance module lands next to main.ts"
    );
    // dep root: the dep's own front .at is compiled into the app (cross-file
    // input via the declared dep root, never a main-pac re-read)
    let mut generated = String::new();
    let mut stack = vec![vue_root.clone()];
    while let Some(d) = stack.pop() {
        for entry in std::fs::read_dir(&d).unwrap().flatten() {
            let p = entry.path();
            if p.is_dir() {
                stack.push(p);
            } else if let Ok(text) = std::fs::read_to_string(&p) {
                generated.push_str(&text);
            }
        }
    }
    assert!(
        generated.contains("MatrixDepWidget"),
        "declared dep root's widget is compiled into the app"
    );
    assert!(
        generated.contains("matrix-field-probe"),
        "front NODE form dir is the compiled front source"
    );

    // ---- Snapshot, delete the main pac.at, regenerate the SAME request. ----
    let snapshot_tree = |base: &std::path::Path| -> std::collections::BTreeMap<String, u64> {
        let mut out = std::collections::BTreeMap::new();
        let mut stack = vec![base.to_path_buf()];
        while let Some(d) = stack.pop() {
            for entry in std::fs::read_dir(&d).unwrap().flatten() {
                let p = entry.path();
                if p.is_dir() {
                    stack.push(p);
                } else {
                    let bytes = std::fs::read(&p).unwrap();
                    let mut hasher = std::collections::hash_map::DefaultHasher::new();
                    bytes.hash(&mut hasher);
                    let rel = p
                        .strip_prefix(base)
                        .unwrap()
                        .to_string_lossy()
                        .replace('\\', "/");
                    out.insert(rel, hasher.finish());
                }
            }
        }
        out
    };
    let before = snapshot_tree(&root.join("gen"));
    std::fs::remove_file(root.join("pac.at"))
        .expect("remove the main declaration after the request snapshot");
    send_generate(&mut tool, 71);
    let after = snapshot_tree(&root.join("gen"));
    assert_eq!(
        before, after,
        "regeneration with the same request after deleting pac.at must be byte-identical \
         (the request snapshot is the only source; any diff is a second main-pac read \
         or nondeterministic emission)"
    );

    let (code, _) = tool.close_stdin_and_wait();
    assert_eq!(code, 0);
}

/// R4-06: a jet build whose wrapper exits 7 is a NAMED failure (the old
/// chain printed "Build failed" and returned Ok -> BuildOk.completed); a
/// wrapper that exits 0 without producing the APK fails the artifact
/// oracle. Both use a real generated jet project with a controlled
/// wrapper — the honest distinguishing pair from the review.
#[test]
#[ignore = "heavy: real cold generation (jet); serial evidence run via --ignored"]
fn build_ui_jet_reports_real_failure_and_missing_artifact() {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::create_dir_all(dir.path().join("src").join("front")).unwrap();
    let pac = "name: \"jetbuild\"\nrender: \"jet\"\nwindow: \"640x480\"\nfront: \"./src/front\"\n";
    std::fs::write(dir.path().join("pac.at"), pac).unwrap();
    std::fs::write(
        dir.path().join("src").join("front").join("app.at"),
        "widget App {\n    model {\n        var count int = 0\n    }\n    view {\n        col {\n            text `Count: ${.count}`\n            button \"+\" { onclick: () => {.count += 1} }\n        }\n    }\n}\n",
    )
    .unwrap();
    let request = GenerationRequest::from_descriptor(
        &parse_document(pac, SourceInfo::from_path(dir.path().join("pac.at"))).expect("parse"),
    );
    let mut tool = ToolProcess::spawn();

    // Generate the real jet project first.
    tool.send(&envelope(
        62,
        ClientBody::GenerateUi {
            request: WireRequest(request.clone()),
            output: None,
            project: None,
            delivery_root: None,
            strict: None,
        },
    ));
    let (response, _) = next_until(&mut tool, |e| {
        matches!(e.body, ServerBody::GenerateOk { .. })
            || matches!(e.body, ServerBody::Error { .. })
    });
    assert!(
        matches!(response.body, ServerBody::GenerateOk { .. }),
        "jet generation must succeed: {:?}",
        response.body
    );
    let jet_dir = dir.path().join("gen/front/jet");
    assert!(jet_dir.is_dir());
    // The fresh scaffold ships no wrapper: create the CONTROLLED wrapper
    // the build path will execute.
    let gradlew = if cfg!(windows) {
        jet_dir.join("gradlew.bat")
    } else {
        jet_dir.join("gradlew")
    };

    // Case 1: exit 7 -> named failed (never completed).
    if cfg!(windows) {
        std::fs::write(&gradlew, "@echo off\r\n@exit /b 7\r\n").unwrap();
    } else {
        std::fs::write(&gradlew, "#!/bin/sh\nexit 7\n").unwrap();
        make_executable(&gradlew);
    }
    tool.send(&envelope(
        63,
        ClientBody::BuildUi {
            request: WireRequest(request.clone()),
            strict: None,
        },
    ));
    let (response, _) = next_until(&mut tool, |e| {
        matches!(e.body, ServerBody::BuildOk { .. }) || matches!(e.body, ServerBody::Error { .. })
    });
    match response.body {
        ServerBody::BuildOk { report } => {
            assert_eq!(
                report.outcome,
                RunOutcome::Failed,
                "exit 7 must not complete: {report:?}"
            );
            assert!(
                report.error.as_deref().is_some_and(|e| e.contains('7')),
                "named failure carries the exit code: {report:?}"
            );
        }
        other => panic!("expected BuildOk, got {other:?}"),
    }

    // Case 2: exit 0 but no APK -> the artifact oracle fails it by name.
    if cfg!(windows) {
        std::fs::write(&gradlew, "@echo off\r\n@exit /b 0\r\n").unwrap();
    } else {
        std::fs::write(&gradlew, "#!/bin/sh\nexit 0\n").unwrap();
        make_executable(&gradlew);
    }
    tool.send(&envelope(
        64,
        ClientBody::BuildUi {
            request: WireRequest(request),
            strict: None,
        },
    ));
    let (response, _) = next_until(&mut tool, |e| {
        matches!(e.body, ServerBody::BuildOk { .. }) || matches!(e.body, ServerBody::Error { .. })
    });
    match response.body {
        ServerBody::BuildOk { report } => {
            assert_eq!(
                report.outcome,
                RunOutcome::Failed,
                "exit 0 without the artifact must not complete: {report:?}"
            );
            assert!(
                report
                    .error
                    .as_deref()
                    .is_some_and(|e| e.contains("artifact is missing")),
                "the artifact oracle is named: {report:?}"
            );
        }
        other => panic!("expected BuildOk, got {other:?}"),
    }

    let (code, _) = tool.close_stdin_and_wait();
    assert_eq!(code, 0);
}

#[cfg(unix)]
fn make_executable(path: &std::path::Path) {
    use std::os::unix::fs::PermissionsExt;
    let mut perms = std::fs::metadata(path).unwrap().permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(path, perms).unwrap();
}

#[cfg(windows)]
fn make_executable(_path: &std::path::Path) {}

/// R4-05: the jet build subprocess is an OWNED tree — cancel terminates
/// the wrapper AND its descendant (a real sleeper that outlives the
/// wrapper), the final receipt arrives promptly, and a SERIAL next build
/// works on the same host. The wrapper launches a PowerShell sleeper that
/// records its own PID; after cancel the PID must be gone.
#[test]
#[cfg(windows)]
#[ignore = "heavy: real generation + owned tree cancel; serial evidence run via --ignored"]
fn build_ui_cancel_terminates_owned_build_tree() {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::create_dir_all(dir.path().join("src").join("front")).unwrap();
    let pac = "name: \"jetcancel\"\nrender: \"jet\"\nwindow: \"640x480\"\nfront: \"./src/front\"\n";
    std::fs::write(dir.path().join("pac.at"), pac).unwrap();
    std::fs::write(
        dir.path().join("src").join("front").join("app.at"),
        "widget App {\n    view {\n        col { text `x` }\n    }\n}\n",
    )
    .unwrap();
    let request = GenerationRequest::from_descriptor(
        &parse_document(pac, SourceInfo::from_path(dir.path().join("pac.at"))).expect("parse"),
    );
    let mut tool = ToolProcess::spawn();
    tool.send(&envelope(
        70,
        ClientBody::GenerateUi {
            request: WireRequest(request.clone()),
            output: None,
            project: None,
            delivery_root: None,
            strict: None,
        },
    ));
    let (response, _) = next_until(&mut tool, |e| {
        matches!(e.body, ServerBody::GenerateOk { .. })
            || matches!(e.body, ServerBody::Error { .. })
    });
    assert!(matches!(response.body, ServerBody::GenerateOk { .. }));

    // Controlled wrapper: launches a 30s PowerShell sleeper that records
    // its PID, and the wrapper waits on it (a real owned descendant).
    let jet_dir = dir.path().join("gen/front/jet");
    let pid_file = dir.path().join("sleeper.pid");
    let pid_str = pid_file.to_string_lossy().replace('\\', "/");
    let script = format!("$PID | Out-File -Encoding ascii '{pid_str}'; Start-Sleep -Seconds 30");
    let wrapper = format!("@echo off\r\npowershell -NoProfile -Command \"{script}\"\r\n");
    std::fs::write(jet_dir.join("gradlew.bat"), wrapper).unwrap();

    let build_started = std::time::Instant::now();
    tool.send(&envelope(
        71,
        ClientBody::BuildUi {
            request: WireRequest(request.clone()),
            strict: None,
        },
    ));
    // Wait until the sleeper has recorded its PID (the build reached the
    // wrapper and spawned the descendant).
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
    let sleeper_pid: u32 = loop {
        if let Ok(text) = std::fs::read_to_string(&pid_file) {
            if let Ok(pid) = text.trim().parse::<u32>() {
                break pid;
            }
        }
        assert!(
            std::time::Instant::now() < deadline,
            "sleeper never started; stderr: {}",
            tool.stderr_so_far()
        );
        std::thread::sleep(std::time::Duration::from_millis(100));
    };

    // Cancel: the whole owned tree (wrapper + powershell descendant) dies
    // and the final receipt arrives promptly.
    tool.send(&envelope(72, ClientBody::Cancel { target_id: 71 }));
    let _ = next_until(&mut tool, |e| {
        matches!(e.body, ServerBody::BuildOk { .. })
            || matches!(e.body, ServerBody::Cancelled { .. })
    });
    let (final_response, _) =
        next_until(&mut tool, |e| matches!(e.body, ServerBody::BuildOk { .. }));
    assert!(
        build_started.elapsed() < std::time::Duration::from_secs(25),
        "cancel must not wait out the 30s sleeper"
    );
    match final_response.body {
        ServerBody::BuildOk { report } => {
            assert_eq!(report.outcome, RunOutcome::Cancelled, "{report:?}");
        }
        other => panic!("expected BuildOk(cancelled), got {other:?}"),
    }
    // The sleeper PID is really gone (tree ownership, not a receipt flag).
    let mut gone = false;
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    while std::time::Instant::now() < deadline {
        let out = std::process::Command::new("tasklist")
            .args(["/FI", &format!("PID eq {sleeper_pid}")])
            .output()
            .expect("tasklist");
        if !String::from_utf8_lossy(&out.stdout).contains(&sleeper_pid.to_string()) {
            gone = true;
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(200));
    }
    assert!(
        gone,
        "the owned build tree must kill the sleeper descendant"
    );

    // The host keeps serving and a SERIAL next build works.
    tool.send(&envelope(73, ClientBody::Describe));
    assert!(matches!(
        parse_response(&tool.next_response()).body,
        ServerBody::DescribeOk { .. }
    ));
    std::fs::write(jet_dir.join("gradlew.bat"), "@echo off\r\n@exit /b 7\r\n").unwrap();
    tool.send(&envelope(
        74,
        ClientBody::BuildUi {
            request: WireRequest(request),
            strict: None,
        },
    ));
    let (response, _) = next_until(&mut tool, |e| {
        matches!(e.body, ServerBody::BuildOk { .. }) || matches!(e.body, ServerBody::Error { .. })
    });
    match response.body {
        ServerBody::BuildOk { report } => {
            assert_eq!(report.outcome, RunOutcome::Failed, "{report:?}");
        }
        other => panic!("expected BuildOk, got {other:?}"),
    }
    let (code, _) = tool.close_stdin_and_wait();
    assert_eq!(code, 0);
}

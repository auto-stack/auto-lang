//! PLAN-815 T-12 V-05: external consumer of the M815-A UI tool.
//!
//! Compile-time: uses only the tool package's public face (descriptor +
//! protocol + project model) — no Man/language/VM crate may appear in the
//! closure (pinned by `verify-deps.sh`). Runtime: spawns the real
//! `auto-ui-tool` binary located via `AUTO_UI_TOOL_BIN` and exercises the
//! protocol contract end-to-end, exactly the way the PLAN-816 executor
//! will: describe, honest validation, unknown-version/operation rejection
//! with zero side effects, and control/log stream separation.

#[cfg(test)]
use std::io::Read;
#[cfg(test)]
use std::process::{Command, Stdio};

#[cfg(test)]
use auto_project_model::descriptor::SourceInfo;
#[cfg(test)]
use auto_project_model::{parse_document, GenerationRequest};
#[cfg(test)]
use auto_tool_protocol::framing::{parse_frame, write_frame, FrameParse};
#[cfg(test)]
use auto_tool_protocol::message::{
    ClientBody, ClientEnvelope, ServerBody, ServerEnvelope, WireRequest,
};
#[cfg(test)]
use auto_tool_protocol::{Budgets, ErrorKind, PROTOCOL_VERSION};

#[cfg(test)]
fn tool_bin() -> std::path::PathBuf {
    match std::env::var_os("AUTO_UI_TOOL_BIN") {
        Some(path) => path.into(),
        None => panic!(
            "AUTO_UI_TOOL_BIN is not set — build the tool first: \
             cargo build -p auto-ui-tools"
        ),
    }
}

#[cfg(test)]
struct Tool {
    child: std::process::Child,
    buffer: Vec<u8>,
    pos: usize,
}

#[cfg(test)]
impl Tool {
    fn spawn() -> Self {
        let child = Command::new(tool_bin())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("spawn real auto-ui-tool process");
        Self {
            child,
            buffer: Vec::new(),
            pos: 0,
        }
    }

    fn request(&mut self, envelope: &ClientEnvelope) -> ServerEnvelope {
        let json = serde_json::to_value(envelope).expect("serialize");
        write_frame(
            self.child.stdin.as_mut().expect("stdin"),
            &json,
            Budgets::contract().max_frame_bytes,
        )
        .expect("send frame");
        let limit = Budgets::contract().max_frame_bytes;
        loop {
            match parse_frame(&self.buffer[self.pos..], limit) {
                FrameParse::Complete { message, consumed } => {
                    self.pos += consumed;
                    return serde_json::from_value(message).expect("typed response");
                }
                FrameParse::Malformed { reason } => panic!("control stream polluted: {reason}"),
                FrameParse::TooLarge { .. } => panic!("over-budget frame from tool"),
                FrameParse::Incomplete { .. } => {}
            }
            let mut chunk = [0u8; 8192];
            let n = self
                .child
                .stdout
                .as_mut()
                .expect("stdout")
                .read(&mut chunk)
                .expect("read stdout");
            assert!(n > 0, "tool closed stdout before answering");
            self.buffer.extend_from_slice(&chunk[..n]);
        }
    }

    fn finish(mut self) -> (i32, String) {
        drop(self.child.stdin.take());
        let mut rest = Vec::new();
        if let Some(mut out) = self.child.stdout.take() {
            out.read_to_end(&mut rest).expect("drain stdout");
        }
        let mut stderr = String::new();
        if let Some(mut err) = self.child.stderr.take() {
            err.read_to_string(&mut stderr).expect("drain stderr");
        }
        let status = self.child.wait().expect("wait");
        (status.code().unwrap_or(-1), stderr)
    }

    fn kill(mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// Compile-time face: the descriptor is buildable from the lib alone.
#[test]
fn consumer_reads_tool_descriptor_from_lib() {
    let descriptor = auto_ui_tools::tool_descriptor();
    assert_eq!(descriptor.tool_name, "auto-ui-tool");
    assert_eq!(descriptor.protocol_version, PROTOCOL_VERSION);
    assert!(!descriptor.offers("generate_ui"));
}

/// V-05: describe over a real process, clean EOF shutdown, logs on stderr.
#[test]
fn consumer_describes_real_process() {
    let mut tool = Tool::spawn();
    let response = tool.request(&ClientEnvelope {
        protocol_version: PROTOCOL_VERSION,
        id: 1,
        body: ClientBody::Describe,
    });
    match response.body {
        ServerBody::DescribeOk { descriptor } => {
            assert_eq!(descriptor.tool_name, "auto-ui-tool");
            assert_eq!(
                descriptor.capabilities.iter().map(|c| c.operation.as_str()).collect::<Vec<_>>(),
                vec!["describe", "validate_project"]
            );
        }
        other => panic!("expected DescribeOk, got {other:?}"),
    }
    let (code, stderr) = tool.finish();
    assert_eq!(code, 0);
    assert!(stderr.contains("accepted request"), "stderr: {stderr}");
}

/// V-05: validate against a real temp project; loss-sensitive echo.
#[test]
fn consumer_validates_real_request() {
    let dir = tempfile::tempdir().expect("tempdir");
    let pac = dir.path().join("pac.at");
    std::fs::write(&pac, "name: \"外部 demo\"\nrender: \"vue\"\n").unwrap();
    let request = GenerationRequest::from_descriptor(
        &parse_document(
            "name: \"外部 demo\"\nrender: \"vue\"\n",
            SourceInfo::from_path(&pac),
        )
        .expect("parse"),
    );
    let mut tool = Tool::spawn();
    let response = tool.request(&ClientEnvelope {
        protocol_version: PROTOCOL_VERSION,
        id: 2,
        body: ClientBody::ValidateProject {
            request: WireRequest(request),
        },
    });
    match response.body {
        ServerBody::ValidateOk { report } => {
            assert!(report.ok, "findings: {:?}", report.findings);
            assert_eq!(report.summary.name.as_deref(), Some("外部 demo"));
            assert!(report.summary.not_checked.contains(&"generation".to_string()));
        }
        other => panic!("expected ValidateOk, got {other:?}"),
    }
    tool.kill();
}

/// V-05 negative: unknown version rejected, zero side effects, stream alive.
#[test]
fn consumer_sees_version_rejection_without_side_effects() {
    let out_root = tempfile::tempdir().expect("tempdir");
    let mut tool = Tool::spawn();
    let raw = serde_json::json!({
        "protocol_version": 4242,
        "id": 3,
        "body": {"op": "validate_project", "output_root": out_root.path().to_string_lossy()}
    });
    write_frame(
        tool.child.stdin.as_mut().expect("stdin"),
        &raw,
        Budgets::contract().max_frame_bytes,
    )
    .expect("send");
    let response: ServerEnvelope = match parse_next(&mut tool) {
        Some(json) => serde_json::from_value(json).expect("typed"),
        None => panic!("no response"),
    };
    match response.body {
        ServerBody::Error { error } => assert_eq!(
            error.kind,
            ErrorKind::UnknownProtocolVersion { client: 4242, server: 1 }
        ),
        other => panic!("expected Error, got {other:?}"),
    }
    assert_eq!(std::fs::read_dir(out_root.path()).expect("dir").count(), 0);
    // the process keeps serving after the rejection
    let ok = tool.request(&ClientEnvelope {
        protocol_version: PROTOCOL_VERSION,
        id: 4,
        body: ClientBody::Describe,
    });
    assert!(matches!(ok.body, ServerBody::DescribeOk { .. }));
    tool.kill();
}

#[cfg(test)]
fn parse_next(tool: &mut Tool) -> Option<serde_json::Value> {
    let limit = Budgets::contract().max_frame_bytes;
    loop {
        match parse_frame(&tool.buffer[tool.pos..], limit) {
            FrameParse::Complete { message, consumed } => {
                tool.pos += consumed;
                return Some(message);
            }
            FrameParse::Incomplete { .. } => {}
            FrameParse::Malformed { reason } => panic!("control stream polluted: {reason}"),
            FrameParse::TooLarge { .. } => panic!("over-budget frame"),
        }
        let mut chunk = [0u8; 8192];
        let n = tool
            .child
            .stdout
            .as_mut()
            .expect("stdout")
            .read(&mut chunk)
            .expect("read");
        if n == 0 {
            return None;
        }
        tool.buffer.extend_from_slice(&chunk[..n]);
    }
}

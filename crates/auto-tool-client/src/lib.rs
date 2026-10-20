//! # auto-tool-client
//!
//! PLAN-816 T-12（M816-A）：I-TOOL 的真实客户端。
//!
//! 消费已发布的 M815-A 窄能力：经 [`auto_tool_execution`]（唯一 owned
//! 子进程设施）spawn 真实工具宿主进程，用 [`auto_tool_protocol`] 的单源
//! 帧实现与消息信封通信。A 阶段面向 `describe` / `validate_project`；
//! 宿主拒绝的请求（版本/能力/字段/预算）以 [`ToolError`] 具名透传，
//! 客户端自身失败（spawn/发送/控制流污染/EOF/超时）以 [`ClientError`]
//! 具名分类——不吞、不字符串化、不假装成功。
//!
//! 寿命合同（V-02/V-05）：请求可带 deadline；到期后 owned 进程树被
//! 终止并**观察到真停**（收尸）且 reader 线程 join 之后才返回
//! [`ClientError::Timeout`]。优雅关闭（EOF）按宿主合同期待退出码 0。
//!
//! 边界（§5.3）：不静态链接任何工具实现（auto-ui-tools 等）——只与真实
//! 进程说话；不做工具发现/隐式搜索（T-04 的 discovery 职责），调用方
//! 给出显式二进制路径；日志只从 stderr 读，控制流（stdout）出现任何
//! 非帧字节即具名 [`ClientError::StreamViolation`]，绝不静默解析。

use std::ffi::OsString;
use std::path::PathBuf;
use std::process::{ChildStdin, ChildStdout};
use std::sync::mpsc::{Receiver, RecvTimeoutError, Sender};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use auto_tool_execution::{
    spawn as spawn_owned, ExecutionSpec, OwnedProcess, StdioPolicy, TreeScope,
};
use auto_tool_protocol::framing::{parse_frame, write_frame, FrameError, FrameParse};
use auto_tool_protocol::message::{ClientBody, ClientEnvelope, ServerEnvelope, WireRequest};
use auto_tool_protocol::{
    Budgets, ToolDescriptor, ToolError, ToolEvent, ValidateProjectReport, PROTOCOL_VERSION,
};

// 消费方经本 crate 组装 spec/读结果，底层设施类型在此再导出，消费闭包
// 不需要直接依赖 auto-tool-execution。
pub use auto_tool_execution::ExitKind;

/// 一次工具宿主连接的完整请求。
#[derive(Clone, Debug)]
pub struct ToolClientSpec {
    /// 工具宿主二进制的显式路径（无隐式发现——发现归 T-04）。
    pub tool_path: PathBuf,
    /// 跟在程序之后的参数（如无）。
    pub argv: Vec<OsString>,
    /// 工作目录（None = 继承调用方）。
    pub cwd: Option<PathBuf>,
    /// 在继承环境之上追加/覆盖的显式对。
    pub env: Vec<(OsString, OsString)>,
    /// 客户端声明的协议版本（负控时经 [`ToolClient::request_with_version`]
    /// 偏离）。
    pub protocol_version: u32,
    /// 单请求 deadline；None = 无限期（由调用方策略决定）。
    pub request_timeout: Option<Duration>,
    /// 帧预算（默认 [`Budgets::contract`]）。
    pub frame_budget: usize,
    /// owned 终止的树范围（默认 [`TreeScope::ProcessTree`]：宿主的孙进程
    /// 一并属于本次拥有）。
    pub tree_scope: TreeScope,
    /// 优雅关闭（EOF 后）等待宿主退出的宽限；到期 cancel 收尸。
    pub shutdown_timeout: Duration,
}

impl ToolClientSpec {
    pub fn new(tool_path: impl Into<PathBuf>) -> Self {
        Self {
            tool_path: tool_path.into(),
            argv: Vec::new(),
            cwd: None,
            env: Vec::new(),
            protocol_version: PROTOCOL_VERSION,
            request_timeout: None,
            frame_budget: Budgets::contract().max_frame_bytes,
            tree_scope: TreeScope::ProcessTree,
            shutdown_timeout: Duration::from_secs(10),
        }
    }
}

/// 宿主回答（具名数据，不是异常）：成功结果、进度事件或结构化拒绝。
#[derive(Debug, Clone)]
pub enum Reply {
    Describe(ToolDescriptor),
    Validate(ValidateProjectReport),
    Event(ToolEvent),
    /// 宿主按协议拒绝/报错（版本/能力/字段/预算/内部）——调用方数据，
    /// 不是客户端故障。
    Host(ToolError),
}

/// 连接的终态原因（首个终态错误由触发它的那次调用原样返回；这里只
/// 供后续调用具名拒绝）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrokenReason {
    /// 请求超时，owned 树已被终止/收尸。
    TimedOut,
    /// 控制流违规（非帧字节），流不可再用。
    StreamViolated,
}

/// 客户端自身的具名失败。
#[derive(Debug)]
pub enum ClientError {
    /// 宿主进程派生失败（诊断含程序名）。
    Spawn {
        program: OsString,
        source: Box<std::io::Error>,
    },
    InvalidSpec(String),
    Io {
        stage: &'static str,
        source: Box<std::io::Error>,
    },
    /// 请求帧写出失败（管道断裂/序列化/超预算——预算超限为 `TooLarge`）。
    SendFrame(FrameError),
    /// 控制流出现非帧字节/坏帧：协议违规，流不可再用。
    StreamViolation(FrameError),
    /// 宿主在回答前关闭了控制流。
    StreamClosed { id: u64 },
    /// 回答信封的 id 与请求不符（A 阶段严格请求/应答镜像）。
    IdMismatch { sent: u64, received: u64 },
    /// 回答帧无法按协议类型化。
    MalformedReply { reason: String },
    /// 请求 deadline 到期：owned 进程树已终止并收尸、reader 线程已
    /// join（真停）之后才返回本错误；`limit` 为到期的时限。
    Timeout { id: u64, limit: Duration },
    /// 连接已因先前的超时/控制流违规终止，后续请求不可用（首个终态
    /// 错误已由触发调用返回）。
    Broken(BrokenReason),
}

impl std::fmt::Display for ClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClientError::Spawn { program, source } => write!(
                f,
                "tool host spawn failed for `{}`: {source}",
                program.to_string_lossy()
            ),
            ClientError::InvalidSpec(reason) => write!(f, "invalid client spec: {reason}"),
            ClientError::Io { stage, source } => write!(f, "{stage}: {source}"),
            ClientError::SendFrame(e) => write!(f, "sending request frame failed: {e}"),
            ClientError::StreamViolation(e) => {
                write!(f, "control stream violation (stdout must carry frames only): {e}")
            }
            ClientError::StreamClosed { id } => {
                write!(f, "tool host closed the control stream before answering id={id}")
            }
            ClientError::IdMismatch { sent, received } => {
                write!(f, "reply id mismatch: sent {sent}, received {received}")
            }
            ClientError::MalformedReply { reason } => {
                write!(f, "reply frame is not a valid protocol envelope: {reason}")
            }
            ClientError::Timeout { id, limit } => write!(
                f,
                "request id={id} timed out after {limit:?}; owned tool tree was \
                 terminated, reaped, and readers joined"
            ),
            ClientError::Broken(BrokenReason::TimedOut) => write!(
                f,
                "client connection broken by a prior request timeout"
            ),
            ClientError::Broken(BrokenReason::StreamViolated) => write!(
                f,
                "client connection broken by a prior control-stream violation"
            ),
        }
    }
}

impl std::error::Error for ClientError {}

impl From<auto_tool_execution::ExecutionError> for ClientError {
    fn from(e: auto_tool_execution::ExecutionError) -> Self {
        match e {
            auto_tool_execution::ExecutionError::Spawn { program, source } => {
                ClientError::Spawn {
                    program,
                    source: Box::new(source),
                }
            }
            auto_tool_execution::ExecutionError::InvalidSpec(reason) => {
                ClientError::InvalidSpec(reason)
            }
            auto_tool_execution::ExecutionError::Io { stage, source } => ClientError::Io {
                stage,
                source: Box::new(source),
            },
        }
    }
}

/// reader 线程 → 客户端的控制流消息。
enum FromReader {
    Frame(Box<ServerEnvelope>),
    Violation(FrameError),
    Eof,
}

/// 与一个真实工具宿主进程的连接。
pub struct ToolClient {
    proc: OwnedProcess,
    stdin: Option<ChildStdin>,
    rx: Receiver<FromReader>,
    stdout_reader: Option<std::thread::JoinHandle<()>>,
    stderr_reader: Option<std::thread::JoinHandle<()>>,
    logs: Arc<Mutex<Vec<u8>>>,
    next_id: u64,
    spec: ToolClientSpec,
    broken: Option<BrokenReason>,
}

impl ToolClient {
    /// spawn 真实宿主进程并接线控制/日志流。
    pub fn spawn(spec: ToolClientSpec) -> Result<Self, ClientError> {
        if spec.frame_budget == 0 {
            return Err(ClientError::InvalidSpec("frame_budget must be nonzero".into()));
        }
        let mut esp = ExecutionSpec::new({
            let mut argv = Vec::with_capacity(spec.argv.len() + 1);
            argv.push(spec.tool_path.clone().into_os_string());
            argv.extend(spec.argv.iter().cloned());
            argv
        });
        esp.cwd = spec.cwd.clone();
        esp.env = spec.env.clone();
        esp.stdin = StdioPolicy::Piped;
        esp.stdout = StdioPolicy::Piped;
        esp.stderr = StdioPolicy::Piped;
        esp.tree_scope = spec.tree_scope;
        let mut proc = spawn_owned(&esp)?;

        let stdin = proc.take_stdin();
        let (tx, rx) = std::sync::mpsc::channel::<FromReader>();
        let stdout_reader = proc
            .take_stdout()
            .map(|out| reader_thread(out, tx, spec.frame_budget));
        let logs = Arc::new(Mutex::new(Vec::new()));
        let stderr_reader = proc.take_stderr().map(|err| {
            let sink = Arc::clone(&logs);
            std::thread::spawn(move || {
                use std::io::Read;
                let mut r = err;
                let mut chunk = [0u8; 8192];
                loop {
                    match r.read(&mut chunk) {
                        Ok(0) | Err(_) => break,
                        Ok(n) => sink.lock().unwrap().extend_from_slice(&chunk[..n]),
                    }
                }
            })
        });
        if stdin.is_none() {
            return Err(ClientError::InvalidSpec(
                "tool host stdin was not piped".into(),
            ));
        }
        Ok(Self {
            proc,
            stdin,
            rx,
            stdout_reader,
            stderr_reader,
            logs,
            next_id: 1,
            spec,
            broken: None,
        })
    }

    /// 宿主进程 id（诊断/日志用途）。
    pub fn host_pid(&self) -> u32 {
        self.proc.id()
    }

    /// 日志流（stderr）快照——控制流之外的独立通道。
    pub fn logs(&self) -> String {
        String::from_utf8_lossy(&self.logs.lock().unwrap()).into_owned()
    }

    /// 宿主工具的版本/能力/生命周期描述（M815-A 窄能力正控入口）。
    pub fn describe(&mut self) -> Result<ToolDescriptor, ClientError> {
        match self.request(ClientBody::Describe)? {
            Reply::Describe(descriptor) => Ok(descriptor),
            Reply::Host(e) => Err(ClientError::MalformedReply {
                reason: format!("describe rejected: {e}"),
            }),
            other => Err(ClientError::MalformedReply {
                reason: format!("unexpected reply to describe: {other:?}"),
            }),
        }
    }

    /// 校验一个 I-PROJECT 请求（M815-A 第二个窄能力）。宿主的结构化
    /// 拒绝（若发生）作为内层 `Err` 数据返回。
    pub fn validate_project(
        &mut self,
        request: auto_project_model::GenerationRequest,
    ) -> Result<Result<ValidateProjectReport, ToolError>, ClientError> {
        match self.request(ClientBody::ValidateProject {
            request: WireRequest(request),
        })? {
            Reply::Validate(report) => Ok(Ok(report)),
            Reply::Host(e) => Ok(Err(e)),
            other => Err(ClientError::MalformedReply {
                reason: format!("unexpected reply to validate_project: {other:?}"),
            }),
        }
    }

    /// 对具名目标发送 cancel 控制消息；宿主的诚实回答
    /// （[`ToolError`]：NotCancellable/InvalidField）作为数据返回。
    pub fn cancel_operation(&mut self, target_id: u64) -> Result<ToolError, ClientError> {
        match self.request(ClientBody::Cancel { target_id })? {
            Reply::Host(e) => Ok(e),
            other => Err(ClientError::MalformedReply {
                reason: format!("unexpected reply to cancel: {other:?}"),
            }),
        }
    }

    /// 发送一个类型化请求并等待镜像回答。
    pub fn request(&mut self, body: ClientBody) -> Result<Reply, ClientError> {
        self.round_trip(self.spec.protocol_version, body)
    }

    /// 负控通道 ①：以偏离的协议版本发送类型化请求（宿主须在副作用前
    /// 具名拒绝，流保持可用）。合同验证用，生产路径勿用。
    pub fn request_with_version(
        &mut self,
        version: u32,
        body: ClientBody,
    ) -> Result<Reply, ClientError> {
        self.round_trip(version, body)
    }

    /// 负控通道 ②：发送一个宿主能力表之外的操作名（期望
    /// `unknown-operation`、零副作用、流保持可用）。合同验证用。
    pub fn request_unknown_operation(&mut self, operation: &str) -> Result<Reply, ClientError> {
        if let Some(reason) = self.broken {
            return Err(ClientError::Broken(reason));
        }
        let id = self.next_id;
        self.next_id += 1;
        let raw = serde_json::json!({
            "protocol_version": self.spec.protocol_version,
            "id": id,
            "body": {"op": operation}
        });
        self.send_raw(&raw)?;
        let reply = self.wait_reply(id)?;
        Ok(typed_reply(reply))
    }

    fn round_trip(&mut self, version: u32, body: ClientBody) -> Result<Reply, ClientError> {
        if let Some(reason) = self.broken {
            return Err(ClientError::Broken(reason));
        }
        let id = self.next_id;
        self.next_id += 1;
        let envelope = ClientEnvelope {
            protocol_version: version,
            id,
            body,
        };
        let json = serde_json::to_value(&envelope).map_err(|e| ClientError::Io {
            stage: "serialize request",
            source: Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e)),
        })?;
        self.send_raw(&json)?;
        let reply = self.wait_reply(id)?;
        Ok(typed_reply(reply))
    }

    fn send_raw(&mut self, json: &serde_json::Value) -> Result<(), ClientError> {
        let stdin = self
            .stdin
            .as_mut()
            .ok_or_else(|| ClientError::InvalidSpec("stdin already closed".into()))?;
        write_frame(stdin, json, self.spec.frame_budget).map_err(ClientError::SendFrame)
    }

    fn wait_reply(&mut self, id: u64) -> Result<ServerEnvelope, ClientError> {
        // A 阶段宿主严格请求/应答镜像：一条控制流消息即决定本次结果。
        let msg = match self.spec.request_timeout {
            Some(limit) => match self.rx.recv_timeout(limit) {
                Ok(msg) => msg,
                Err(RecvTimeoutError::Timeout) => return Err(self.abort_for_timeout(id, limit)),
                Err(RecvTimeoutError::Disconnected) => return Err(ClientError::StreamClosed { id }),
            },
            None => match self.rx.recv() {
                Ok(msg) => msg,
                Err(_) => return Err(ClientError::StreamClosed { id }),
            },
        };
        match msg {
            FromReader::Frame(envelope) => {
                let envelope = *envelope;
                if envelope.id == id {
                    Ok(envelope)
                } else {
                    Err(ClientError::IdMismatch {
                        sent: id,
                        received: envelope.id,
                    })
                }
            }
            FromReader::Violation(e) => {
                self.broken = Some(BrokenReason::StreamViolated);
                Err(ClientError::StreamViolation(e))
            }
            FromReader::Eof => Err(ClientError::StreamClosed { id }),
        }
    }

    /// deadline 到期路径：终止 owned 树 → 收尸 → join reader 线程（真停）
    /// 之后才返回。kill 发出不算已停（§5.2）。
    fn abort_for_timeout(&mut self, id: u64, limit: Duration) -> ClientError {
        let _ = self.proc.cancel();
        self.join_readers();
        self.broken = Some(BrokenReason::TimedOut);
        ClientError::Timeout { id, limit }
    }

    fn join_readers(&mut self) {
        if let Some(handle) = self.stdout_reader.take() {
            let _ = handle.join();
        }
        if let Some(handle) = self.stderr_reader.take() {
            let _ = handle.join();
        }
    }

    /// 优雅关闭：关闭 stdin（EOF）→ 宽限内等待宿主退出（合同期待码 0）；
    /// 宽限到期则 cancel 收尸（退出分类非 Success）。reader 线程在返回前
    /// join。返回退出分类与完整日志。
    pub fn shutdown(mut self) -> Result<Shutdown, ClientError> {
        drop(self.stdin.take());
        let status = match self.proc.wait_deadline(self.spec.shutdown_timeout) {
            Ok(Some(status)) => status,
            Ok(None) => self.proc.cancel().map_err(|source| ClientError::Io {
                stage: "cancel after shutdown grace",
                source: Box::new(source),
            })?,
            Err(source) => {
                return Err(ClientError::Io {
                    stage: "wait for host shutdown",
                    source: Box::new(source),
                })
            }
        };
        self.join_readers();
        Ok(Shutdown {
            kind: ExitKind::from_status(&status),
            logs: self.logs(),
        })
    }
}

/// 优雅关闭的结果。
#[derive(Debug, Clone)]
pub struct Shutdown {
    pub kind: ExitKind,
    pub logs: String,
}

fn typed_reply(envelope: ServerEnvelope) -> Reply {
    use auto_tool_protocol::message::ServerBody;
    match envelope.body {
        ServerBody::DescribeOk { descriptor } => Reply::Describe(descriptor),
        ServerBody::ValidateOk { report } => Reply::Validate(report),
        ServerBody::Event { event } => Reply::Event(event),
        ServerBody::Error { error } => Reply::Host(error),
    }
}

/// 控制流 reader 线程：只解析帧；任何非帧字节/坏帧/超预算帧都是具名
/// 违规且终结流（同步已丢失）。已消费的字节及时压实，长会话不累积。
fn reader_thread(
    mut out: ChildStdout,
    tx: Sender<FromReader>,
    max_frame_bytes: usize,
) -> std::thread::JoinHandle<()> {
    use std::io::Read;
    std::thread::spawn(move || {
        let mut buffer = Vec::new();
        let mut pos = 0usize;
        let mut chunk = [0u8; 8192];
        loop {
            match parse_frame(&buffer[pos..], max_frame_bytes) {
                FrameParse::Complete { message, consumed } => {
                    pos += consumed;
                    let envelope = serde_json::from_value(message);
                    match envelope {
                        Ok(envelope) => {
                            if tx.send(FromReader::Frame(Box::new(envelope))).is_err() {
                                return;
                            }
                        }
                        Err(e) => {
                            let _ = tx.send(FromReader::Violation(FrameError::Malformed {
                                reason: format!("reply is not a typed envelope: {e}"),
                            }));
                            return;
                        }
                    }
                    if pos > 0 {
                        buffer.drain(..pos);
                        pos = 0;
                    }
                }
                FrameParse::Incomplete { .. } => {}
                FrameParse::Malformed { reason } => {
                    let _ = tx.send(FromReader::Violation(FrameError::Malformed { reason }));
                    return;
                }
                FrameParse::TooLarge { limit, observed } => {
                    let _ = tx.send(FromReader::Violation(FrameError::TooLarge { limit, observed }));
                    return;
                }
            }
            match out.read(&mut chunk) {
                Ok(0) => {
                    let leftover = &buffer[pos..];
                    if leftover.is_empty() {
                        let _ = tx.send(FromReader::Eof);
                    } else {
                        let _ = tx.send(FromReader::Violation(FrameError::Malformed {
                            reason: format!(
                                "control stream closed mid-frame ({} trailing bytes)",
                                leftover.len()
                            ),
                        }));
                    }
                    return;
                }
                Ok(n) => buffer.extend_from_slice(&chunk[..n]),
                Err(e) => {
                    let _ = tx.send(FromReader::Violation(FrameError::Io(e)));
                    return;
                }
            }
        }
    })
}

// 真实进程正反控（V-05/V-02/V-03 当期）在独立 fixture
// test/fixtures/modularity/tool-client-consumer/ 执行（镜像 815
// ui-tool-consumer 分层：crate 内联测试保持纯数据面，不派生子进程，
// 日常档不依赖外部二进制构建时序；fixture driver 先构建
// auto-ui-tool/te-child 再以 env 指向真实产物）。
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spec_defaults_match_published_contract() {
        let spec = ToolClientSpec::new("some-tool");
        assert_eq!(spec.protocol_version, PROTOCOL_VERSION);
        assert_eq!(spec.frame_budget, Budgets::contract().max_frame_bytes);
        assert_eq!(spec.tree_scope, TreeScope::ProcessTree);
        assert_eq!(spec.request_timeout, None);
        assert_eq!(spec.shutdown_timeout, Duration::from_secs(10));
        assert!(spec.argv.is_empty() && spec.env.is_empty());
        assert_eq!(spec.cwd, None);
    }

    #[test]
    fn typed_reply_maps_every_server_body() {
        use auto_tool_protocol::message::ServerBody;
        use auto_tool_protocol::{Finding, RequestSummary, Severity};
        let mk = |body: ServerBody| ServerEnvelope { id: 7, body };
        assert!(matches!(
            typed_reply(mk(ServerBody::DescribeOk {
                descriptor: ToolDescriptor {
                    tool_name: "t".into(),
                    tool_version: "0".into(),
                    protocol_version: PROTOCOL_VERSION,
                    capabilities: vec![],
                    lifecycle: auto_tool_protocol::LifecyclePolicy::contract(),
                }
            })),
            Reply::Describe(_)
        ));
        let summary = RequestSummary {
            name: None,
            origin: "static".into(),
            is_evaluated: false,
            frontends: vec![],
            mode: String::new(),
            window: None,
            members: 0,
            dep_roots: 0,
            raw_props: 0,
            raw_nodes: 0,
            input_sources: 0,
            has_output_root: false,
            has_theme_decl: false,
            package_assets_present: [false; 3],
            checked: vec![],
            not_checked: vec!["generation".into()],
        };
        assert!(matches!(
            typed_reply(mk(ServerBody::ValidateOk {
                report: ValidateProjectReport {
                    ok: false,
                    findings: vec![Finding::new(Severity::Error, "probe", "probe")],
                    summary,
                }
            })),
            Reply::Validate(_)
        ));
        assert!(matches!(
            typed_reply(mk(ServerBody::Event {
                event: ToolEvent::Progress {
                    message: String::new(),
                    done: 1,
                    total: None
                }
            })),
            Reply::Event(_)
        ));
        assert!(matches!(
            typed_reply(mk(ServerBody::Error {
                error: ToolError::internal("x")
            })),
            Reply::Host(_)
        ));
    }

    #[test]
    fn error_kinds_carry_names() {
        let spawn = ClientError::Spawn {
            program: "no-such-tool.exe".into(),
            source: Box::new(std::io::Error::new(std::io::ErrorKind::NotFound, "gone")),
        };
        assert!(spawn.to_string().contains("no-such-tool.exe"));
        let timeout = ClientError::Timeout {
            id: 3,
            limit: Duration::from_secs(2),
        };
        assert!(timeout.to_string().contains("reaped"));
        assert!(timeout.to_string().contains("2"));
        let violated = ClientError::Broken(BrokenReason::StreamViolated);
        assert!(violated.to_string().contains("control-stream violation"));
    }

    #[test]
    fn execution_error_conversion_preserves_names() {
        let e = ClientError::from(auto_tool_execution::ExecutionError::InvalidSpec(
            "argv must not be empty".into(),
        ));
        assert!(matches!(e, ClientError::InvalidSpec(_)));
        let e = ClientError::from(auto_tool_execution::ExecutionError::Io {
            stage: "wait",
            source: std::io::Error::other("boom"),
        });
        assert!(matches!(e, ClientError::Io { stage: "wait", .. }));
    }
}

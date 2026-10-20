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
//! 寿命合同（V-02/V-05，含 R1 复审修复）：
//! - 请求 deadline 覆盖**整次往返**：帧写入（经 writer 线程，对端不读
//!   导致的管道写阻塞不越过截止）与回答等待共用同一绝对截止
//!   （F-816-R1-01）。
//! - 超时/取消后 owned 进程树被终止并**观察到真停**（收尸）、writer 与
//!   reader 线程有界收尾之后才返回 [`ClientError::Timeout`]；终止/收尸
//!   失败与 reader/writer panic、管道被非拥有进程持有一律具名传播，
//!   不以固定文案谎报（F-816-R1-02/06）。
//! - 宿主根进程退出后、reader 收尾前，仍持管道的 owned 后代先被终止；
//!   宽限内收不到 EOF 即 [`ClientError::PipesHeld`]（F-816-R1-02）。
//! - 控制流只解析帧且**先 drain 已缓冲内容**：已到达的完整帧与坏前缀
//!   不等待新字节即被处理（F-816-R1-03）。
//! - 优雅关闭（EOF）按宿主合同期待退出码 0，宽限到期则 cancel 收尸。
//!
//! 边界（§5.3）：不静态链接任何工具实现（auto-ui-tools 等）——只与真实
//! 进程说话；不做工具发现/隐式搜索（T-04 的 discovery 职责），调用方
//! 给出显式二进制路径；日志只从 stderr 读，控制流（stdout）出现任何
//! 非帧字节即具名 [`ClientError::StreamViolation`]，绝不静默解析。

use std::ffi::OsString;
use std::io::Read;
use std::path::PathBuf;
use std::sync::mpsc::{Receiver, RecvTimeoutError, Sender};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

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

/// writer/reader 收尾的拆除宽限：owned 树终止后管道应立即关闭，给足
/// 调度余量即可；超时即管道被非拥有进程持有（F-816-R1-02/06）。
const TEARDOWN_GRACE: Duration = Duration::from_secs(3);

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
    /// 单请求 deadline——覆盖整次往返（帧写入+回答等待）；None = 无限期
    /// （由调用方策略决定）。
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
    /// stdin writer 失败（管道断裂/序列化/预算），连接不可再用。
    WriterFailed,
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
    StreamClosed {
        id: u64,
    },
    /// 回答信封的 id 与请求不符（A 阶段严格请求/应答镜像）。
    IdMismatch {
        sent: u64,
        received: u64,
    },
    /// 回答帧无法按协议类型化。
    MalformedReply {
        reason: String,
    },
    /// 请求 deadline 到期：owned 进程树已终止并收尸、writer 与 reader
    /// 线程有界收尾之后才返回本错误；`limit` 为到期的时限。
    Timeout {
        id: u64,
        limit: Duration,
    },
    /// 宿主根进程已退出且 owned 树已终止，但管道写端仍被**非本次拥有**
    /// 的进程持有：reader/writer 在宽限内收不到 EOF，按具名失败返回
    /// 而非无限等待（F-816-R1-02）。
    PipesHeld {
        after: Duration,
    },
    /// reader/writer 线程 panic（完成通道无消息即断开）。
    Panicked {
        which: &'static str,
    },
    /// 连接已因先前的超时/控制流违规/writer 失败终止，后续请求不可用
    /// （首个终态错误已由触发调用返回）。
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
                write!(
                    f,
                    "control stream violation (stdout must carry frames only): {e}"
                )
            }
            ClientError::StreamClosed { id } => {
                write!(
                    f,
                    "tool host closed the control stream before answering id={id}"
                )
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
            ClientError::PipesHeld { after } => write!(
                f,
                "host pipes still held by processes outside this connection after \
                 {after:?}; readers could not finish"
            ),
            ClientError::Panicked { which } => {
                write!(f, "{which} panicked; connection state is unreliable")
            }
            ClientError::Broken(BrokenReason::TimedOut) => {
                write!(f, "client connection broken by a prior request timeout")
            }
            ClientError::Broken(BrokenReason::StreamViolated) => write!(
                f,
                "client connection broken by a prior control-stream violation"
            ),
            ClientError::Broken(BrokenReason::WriterFailed) => {
                write!(
                    f,
                    "client connection broken by a prior stdin-writer failure"
                )
            }
        }
    }
}

impl std::error::Error for ClientError {}

impl From<auto_tool_execution::ExecutionError> for ClientError {
    fn from(e: auto_tool_execution::ExecutionError) -> Self {
        match e {
            auto_tool_execution::ExecutionError::Spawn { program, source } => ClientError::Spawn {
                program,
                source: Box::new(source),
            },
            auto_tool_execution::ExecutionError::InvalidSpec(reason) => {
                ClientError::InvalidSpec(reason)
            }
            auto_tool_execution::ExecutionError::Io { stage, source } => ClientError::Io {
                stage,
                source: Box::new(source),
            },
            other => ClientError::Io {
                stage: "execution facility",
                source: Box::new(std::io::Error::other(other.to_string())),
            },
        }
    }
}

/// reader 线程 → 客户端的控制流消息。`Done` 是唯一的不变量标记：所有
/// 非恐慌退出路径的最后一条消息；通道在无 `Done` 的情况下断开 = 线程
/// panic（F-816-R1-06）。
#[derive(Debug)]
enum FromReader {
    Frame(Box<ServerEnvelope>),
    Violation(FrameError),
    Eof,
    Done,
}

/// 与一个真实工具宿主进程的连接。
pub struct ToolClient {
    proc: OwnedProcess,
    writer_tx: Option<Sender<serde_json::Value>>,
    /// 每次写出的结果（writer 线程回执）。
    writer_ack: Receiver<Result<(), FrameError>>,
    /// writer 线程退出标记（stdin 已随之关闭）。
    writer_done: Receiver<()>,
    rx: Receiver<FromReader>,
    /// stderr 日志累计（reader 线程只写不判）。
    logs: Arc<Mutex<Vec<u8>>>,
    stderr_done: Receiver<()>,
    next_id: u64,
    spec: ToolClientSpec,
    broken: Option<BrokenReason>,
    /// writer 完成标记是否已被消费（shutdown/收尾只等一次，重复 recv 会
    /// 把正常终态的通道断开误判为 panic）。
    writer_done_seen: bool,
    /// writer/reader 是否已完成过收尾（Timeout/Violation 路径已 drain
    /// `Done` 后，shutdown 不重复等待——通道断开是正常终态而非 panic）。
    collected: bool,
}

impl ToolClient {
    /// spawn 真实宿主进程并接线控制/日志流。stdin 写入由内部 writer
    /// 线程拥有（管道写阻塞不越过调用方 deadline，F-816-R1-01）。
    pub fn spawn(spec: ToolClientSpec) -> Result<Self, ClientError> {
        if spec.frame_budget == 0 {
            return Err(ClientError::InvalidSpec(
                "frame_budget must be nonzero".into(),
            ));
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

        let stdin = proc
            .take_stdin()
            .ok_or_else(|| ClientError::InvalidSpec("tool host stdin was not piped".into()))?;
        let (writer_tx, writer_rx) = std::sync::mpsc::channel::<serde_json::Value>();
        let (ack_tx, writer_ack) = std::sync::mpsc::channel::<Result<(), FrameError>>();
        let (wdone_tx, writer_done) = std::sync::mpsc::channel::<()>();
        let budget = spec.frame_budget;
        std::thread::spawn(move || {
            let mut stdin = stdin;
            for json in writer_rx {
                let result = write_frame(&mut stdin, &json, budget);
                // 仅真实写出失败(Io)终结连接;预算/序列化拒绝发生在任何
                // 字节写出之前,流保持可用。
                let fatal = matches!(result, Err(FrameError::Io(_)));
                let _ = ack_tx.send(result);
                if fatal {
                    break;
                }
            }
            // 通道关闭（调用方 shutdown/收尾）或致命写失败：drop stdin。
            drop(stdin);
            let _ = wdone_tx.send(());
        });

        let (tx, rx) = std::sync::mpsc::channel::<FromReader>();
        let has_stdout = proc
            .take_stdout()
            .map(|out| {
                std::thread::spawn(move || {
                    let mut boxed: Box<dyn Read + Send> = Box::new(out);
                    reader_session(&mut boxed, &tx, budget);
                })
            })
            .is_some();
        if !has_stdout {
            return Err(ClientError::InvalidSpec(
                "tool host stdout was not piped".into(),
            ));
        }
        let logs = Arc::new(Mutex::new(Vec::new()));
        let (sdone_tx, stderr_done) = std::sync::mpsc::channel::<()>();
        if let Some(err) = proc.take_stderr() {
            let sink = Arc::clone(&logs);
            std::thread::spawn(move || {
                let mut r = err;
                let mut chunk = [0u8; 8192];
                loop {
                    match r.read(&mut chunk) {
                        Ok(0) | Err(_) => break,
                        Ok(n) => sink.lock().unwrap().extend_from_slice(&chunk[..n]),
                    }
                }
                let _ = sdone_tx.send(());
            });
        } else {
            let _ = sdone_tx.send(());
        }
        Ok(Self {
            proc,
            writer_tx: Some(writer_tx),
            writer_ack,
            writer_done,
            rx,
            logs,
            stderr_done,
            next_id: 1,
            spec,
            broken: None,
            writer_done_seen: false,
            collected: false,
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
        let started = Instant::now();
        let limit = self.spec.request_timeout;
        self.send_deadline(&raw, started, limit, id)?;
        let reply = self.wait_reply(id, limit, started)?;
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
        let started = Instant::now();
        let limit = self.spec.request_timeout;
        self.send_deadline(&json, started, limit, id)?;
        let reply = self.wait_reply(id, limit, started)?;
        Ok(typed_reply(reply))
    }

    /// 发送一帧，**绝对 deadline 同时约束排队与阻塞写**（F-816-R1-01）：
    /// `started+limit` 是本次请求的唯一截止；写阻塞到截止即走超时路径
    /// （终止 owned 树 → 管道关闭解除 writer 阻塞 → 有界收尾）。
    fn send_deadline(
        &mut self,
        json: &serde_json::Value,
        started: Instant,
        limit: Option<Duration>,
        id: u64,
    ) -> Result<(), ClientError> {
        let tx = self
            .writer_tx
            .as_ref()
            .ok_or_else(|| ClientError::InvalidSpec("writer already closed".into()))?;
        tx.send(json.clone()).map_err(|_| ClientError::Panicked {
            which: "stdin writer",
        })?;
        let wait = limit.map(|l| (started + l).saturating_duration_since(Instant::now()));
        let ack = match wait {
            Some(w) => self.writer_ack.recv_timeout(w),
            None => self
                .writer_ack
                .recv()
                .map_err(|_| RecvTimeoutError::Disconnected),
        };
        match ack {
            Ok(Ok(())) => Ok(()),
            Ok(Err(e)) => {
                if matches!(e, FrameError::Io(_)) {
                    self.broken = Some(BrokenReason::WriterFailed);
                }
                Err(ClientError::SendFrame(e))
            }
            Err(RecvTimeoutError::Timeout) => {
                Err(self.abort_for_timeout(id, limit.unwrap_or_default()))
            }
            Err(RecvTimeoutError::Disconnected) => Err(ClientError::Panicked {
                which: "stdin writer",
            }),
        }
    }

    fn wait_reply(
        &mut self,
        id: u64,
        limit: Option<Duration>,
        started: Instant,
    ) -> Result<ServerEnvelope, ClientError> {
        let wait = limit.map(|l| (started + l).saturating_duration_since(Instant::now()));
        let msg = match wait {
            Some(w) => match self.rx.recv_timeout(w) {
                Ok(msg) => msg,
                Err(RecvTimeoutError::Timeout) => {
                    return Err(self.abort_for_timeout(id, limit.unwrap_or_default()))
                }
                Err(RecvTimeoutError::Disconnected) => {
                    return Err(ClientError::StreamClosed { id })
                }
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
            FromReader::Eof | FromReader::Done => Err(ClientError::StreamClosed { id }),
        }
    }

    /// deadline 到期路径：终止 owned 树 → 收尸 → writer/reader 有界
    /// 收尾，全部完成后才返回（F-816-R1-01/02/06）。终止失败与收尾失败
    /// 具名传播，不以固定 Timeout 文案谎报"已收尸/reader 已 join"。
    fn abort_for_timeout(&mut self, id: u64, limit: Duration) -> ClientError {
        // 先终止 owned 树(解除可能卡在满管道上的写),再结束 writer
        //(idle writer 阻塞在任务通道上,须 drop 发送端才退出)。
        let cancel_result = self.proc.cancel();
        drop(self.writer_tx.take());
        let collect = self.join_all_bounded(TEARDOWN_GRACE);
        self.broken = Some(BrokenReason::TimedOut);
        if let Err(source) = cancel_result {
            return ClientError::Io {
                stage: "cancel after timeout",
                source: Box::new(source),
            };
        }
        if let Err(e) = collect {
            return e;
        }
        ClientError::Timeout { id, limit }
    }

    /// writer 完成（单次消费：shutdown 与收尾路径共享）。
    fn wait_writer_done(&mut self, grace: Duration) -> Result<(), ClientError> {
        if self.writer_done_seen {
            return Ok(());
        }
        match self.writer_done.recv_timeout(grace) {
            Ok(()) => {
                self.writer_done_seen = true;
                Ok(())
            }
            Err(RecvTimeoutError::Timeout) => Err(ClientError::PipesHeld { after: grace }),
            Err(RecvTimeoutError::Disconnected) => Err(ClientError::Panicked {
                which: "stdin writer",
            }),
        }
    }

    /// writer/stderr/stdout 的有界收尾：owned 树终止后管道应立即关闭。
    /// stdout 侧持续 drain 到 `Done`（缺 `Done` 的断开 = panic）。已收尾
    /// 过的连接（Timeout/Violation 路径）直接成功，不重复消费已完成通道。
    fn join_all_bounded(&mut self, grace: Duration) -> Result<(), ClientError> {
        if self.collected {
            return Ok(());
        }
        // writer：树终止后阻塞写解除，线程随即退出。
        self.wait_writer_done(grace)?;
        match self.stderr_done.recv_timeout(grace) {
            Ok(()) => {}
            Err(RecvTimeoutError::Timeout) => return Err(ClientError::PipesHeld { after: grace }),
            Err(RecvTimeoutError::Disconnected) => {
                return Err(ClientError::Panicked {
                    which: "stderr reader",
                })
            }
        }
        let deadline = Instant::now() + grace;
        loop {
            let remaining = deadline.saturating_duration_since(Instant::now());
            match self.rx.recv_timeout(remaining) {
                Ok(FromReader::Done) => {
                    self.collected = true;
                    return Ok(());
                }
                Ok(_) => continue,
                Err(RecvTimeoutError::Timeout) => {
                    return Err(ClientError::PipesHeld { after: grace })
                }
                Err(RecvTimeoutError::Disconnected) => {
                    return Err(ClientError::Panicked {
                        which: "stdout reader",
                    })
                }
            }
        }
    }

    /// 优雅关闭：结束 writer（EOF）→ 宽限内等待宿主退出（合同期待码 0）；
    /// 根退出后先终止仍持管道的 owned 后代（F-816-R1-02），reader 有界
    /// 收尾；宽限到期则 cancel 收尸。返回退出分类与完整日志。
    pub fn shutdown(mut self) -> Result<Shutdown, ClientError> {
        let deadline = Instant::now() + self.spec.shutdown_timeout;
        drop(self.writer_tx.take());
        let writer_grace = deadline
            .saturating_duration_since(Instant::now())
            .min(TEARDOWN_GRACE);
        self.wait_writer_done(writer_grace)?;
        let status = match self
            .proc
            .wait_deadline(deadline.saturating_duration_since(Instant::now()))
        {
            Ok(Some(status)) => {
                // 根已退出：真实分类保留;仍持管道的 owned 后代先终止。
                if self.proc.tree_scope() == TreeScope::ProcessTree {
                    self.proc.terminate().map_err(|source| ClientError::Io {
                        stage: "terminate after root exit",
                        source: Box::new(source),
                    })?;
                }
                status
            }
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
        self.join_all_bounded(TEARDOWN_GRACE)?;
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

/// 控制流 reader 会话：[`reader_run`] 之后以 `Done` 收尾（panic 时不发
/// ——通道断开可判）。线程包装与故障注入单测共用同一路径。
fn reader_session<R: Read>(r: &mut R, tx: &Sender<FromReader>, max_frame_bytes: usize) {
    reader_run(r, tx, max_frame_bytes);
    let _ = tx.send(FromReader::Done);
}

/// 控制流 reader 主体（泛化 `Read` 以便故障注入单测，F-816-R1-03/06）：
/// 只解析帧；**先 drain 缓冲内全部可解析内容**（完整帧与坏前缀都不
/// 等待新字节），仅 Incomplete 才读取更多；任何非帧字节/坏帧/超预算帧
/// 都是具名违规且终结流（同步/拆包时序不影响结果）。
fn reader_run<R: Read>(r: &mut R, tx: &Sender<FromReader>, max_frame_bytes: usize) {
    let mut buffer = Vec::new();
    let mut pos = 0usize;
    let mut chunk = [0u8; 8192];
    loop {
        loop {
            match parse_frame(&buffer[pos..], max_frame_bytes) {
                FrameParse::Complete { message, consumed } => {
                    pos += consumed;
                    match serde_json::from_value::<ServerEnvelope>(message) {
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
                FrameParse::Incomplete { .. } => break,
                FrameParse::Malformed { reason } => {
                    let _ = tx.send(FromReader::Violation(FrameError::Malformed { reason }));
                    return;
                }
                FrameParse::TooLarge { limit, observed } => {
                    let _ = tx.send(FromReader::Violation(FrameError::TooLarge {
                        limit,
                        observed,
                    }));
                    return;
                }
            }
        }
        match r.read(&mut chunk) {
            Ok(0) => {
                let leftover = buffer.len() - pos;
                if leftover == 0 {
                    let _ = tx.send(FromReader::Eof);
                } else {
                    let _ = tx.send(FromReader::Violation(FrameError::Malformed {
                        reason: format!(
                            "control stream closed mid-frame ({leftover} trailing bytes)"
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
}

// 真实进程正反控（V-05/V-02/V-03 当期）在独立 fixture
// test/fixtures/modularity/tool-client-consumer/ 执行（镜像 815
// ui-tool-consumer 分层：crate 内联测试保持纯数据面+故障注入面，不依赖
// 外部二进制构建时序；fixture driver 先构建 auto-ui-tool/te-child 再以
// env 指向真实产物）。
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
        assert!(ClientError::Broken(BrokenReason::StreamViolated)
            .to_string()
            .contains("control-stream violation"));
        assert!(ClientError::PipesHeld {
            after: Duration::from_millis(10)
        }
        .to_string()
        .contains("outside this connection"));
        assert!(ClientError::Panicked {
            which: "stdout reader"
        }
        .to_string()
        .contains("panicked"));
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
        // 新设施错误变体不再无声穿过（F-816-R1-06）。
        let e = ClientError::from(auto_tool_execution::ExecutionError::PipesHeld {
            after: Duration::from_millis(5),
        });
        assert!(e.to_string().contains("execution facility"));
    }

    // ---- 故障注入面（F-816-R1-03/06 永久回归，无 sleep、无子进程）----

    /// 预置分块的假 Read：依次交付 `chunks`，之后按 `tail` 结束。
    struct Chunked {
        chunks: Vec<Vec<u8>>,
        tail: Tail,
    }

    enum Tail {
        Eof,
        Err(&'static str),
    }

    impl Read for Chunked {
        fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
            if !self.chunks.is_empty() {
                let chunk = self.chunks.remove(0);
                let n = chunk.len().min(buf.len());
                buf[..n].copy_from_slice(&chunk[..n]);
                return Ok(n);
            }
            match self.tail {
                Tail::Eof => Ok(0),
                Tail::Err(msg) => Err(std::io::Error::other(msg)),
            }
        }
    }

    fn frame_bytes(value: &serde_json::Value) -> Vec<u8> {
        let payload = serde_json::to_vec(value).unwrap();
        let mut bytes = (payload.len() as u32).to_be_bytes().to_vec();
        bytes.extend_from_slice(&payload);
        bytes
    }

    fn typed_error_envelope(id: u64) -> serde_json::Value {
        serde_json::to_value(ServerEnvelope::error(
            id,
            ToolError::unknown_operation("probe"),
        ))
        .unwrap()
    }

    /// F-816-R1-03：合法帧 + 坏前缀同批到达——坏前缀在**不读取新字节**
    /// 的情况下被处理（后续 read 直接报错，drain 正确则永不触达）。
    #[test]
    fn buffered_bad_prefix_reported_without_more_reads() {
        let (tx, rx) = std::sync::mpsc::channel::<FromReader>();
        let good = frame_bytes(&typed_error_envelope(1));
        let mut combined = good;
        combined.extend_from_slice(&[0xff, 0xff, 0xff, 0xff]);
        let mut reader = Chunked {
            chunks: vec![combined],
            tail: Tail::Err("read after buffered content — drain regression"),
        };
        reader_session(&mut reader, &tx, Budgets::contract().max_frame_bytes);
        // 第一条：合法类型化错误帧。
        match rx.try_recv().unwrap() {
            FromReader::Frame(env) => assert_eq!(env.id, 1),
            other => panic!("expected buffered frame first, got {other:?}"),
        }
        // 第二条：缓冲坏前缀（0xffffffff > 预算）→ TooLarge，不是 Io。
        match rx.try_recv().unwrap() {
            FromReader::Violation(FrameError::TooLarge { observed, .. }) => {
                assert_eq!(observed, 0xffff_ffff);
            }
            other => panic!("expected buffered TooLarge, got {other:?}"),
        }
        // 收尾：Done 恒为最后一条。
        assert!(matches!(rx.try_recv().unwrap(), FromReader::Done));
        assert!(rx.try_recv().is_err());
    }

    /// F-816-R1-03 相邻面：粘包（一帧读内多帧）全部 drain，再 EOF。
    #[test]
    fn coalesced_frames_all_drained_before_eof() {
        let (tx, rx) = std::sync::mpsc::channel::<FromReader>();
        let mut combined = frame_bytes(&typed_error_envelope(1));
        combined.extend_from_slice(&frame_bytes(
            &serde_json::to_value(ServerEnvelope::error(
                2,
                ToolError::not_cancellable("describe"),
            ))
            .unwrap(),
        ));
        let mut reader = Chunked {
            chunks: vec![combined],
            tail: Tail::Eof,
        };
        reader_session(&mut reader, &tx, Budgets::contract().max_frame_bytes);
        assert!(matches!(rx.try_recv().unwrap(), FromReader::Frame(_)));
        assert!(matches!(rx.try_recv().unwrap(), FromReader::Frame(_)));
        assert!(matches!(rx.try_recv().unwrap(), FromReader::Eof));
        assert!(matches!(rx.try_recv().unwrap(), FromReader::Done));
    }

    /// F-816-R1-06：reader 读错具名传播（不是静默 EOF）。
    #[test]
    fn reader_io_error_surfaces_as_violation() {
        let (tx, rx) = std::sync::mpsc::channel::<FromReader>();
        let mut reader = Chunked {
            chunks: vec![(3u32).to_be_bytes().to_vec()],
            tail: Tail::Err("kernel pipe gone"),
        };
        reader_session(&mut reader, &tx, Budgets::contract().max_frame_bytes);
        match rx.try_recv().unwrap() {
            FromReader::Violation(FrameError::Io(e)) => {
                assert!(e.to_string().contains("kernel pipe gone"))
            }
            other => panic!("expected Io violation, got {other:?}"),
        }
        assert!(matches!(rx.try_recv().unwrap(), FromReader::Done));
    }

    /// EOF 夹带半帧 → 具名"mid-frame"违规。
    #[test]
    fn eof_with_partial_frame_is_named_violation() {
        let (tx, rx) = std::sync::mpsc::channel::<FromReader>();
        let mut partial = (64u32).to_be_bytes().to_vec();
        partial.extend_from_slice(b"{\"id\": 1"); // 不足 64 字节载荷
        let mut reader = Chunked {
            chunks: vec![partial],
            tail: Tail::Eof,
        };
        reader_session(&mut reader, &tx, Budgets::contract().max_frame_bytes);
        match rx.try_recv().unwrap() {
            FromReader::Violation(FrameError::Malformed { reason }) => {
                assert!(reason.contains("mid-frame"), "{reason}")
            }
            other => panic!("expected mid-frame violation, got {other:?}"),
        }
        assert!(matches!(rx.try_recv().unwrap(), FromReader::Done));
    }

    /// 帧内 JSON 合法但不符合信封类型 → 具名"not a typed envelope"。
    #[test]
    fn untyped_envelope_is_named_violation() {
        let (tx, rx) = std::sync::mpsc::channel::<FromReader>();
        let mut reader = Chunked {
            chunks: vec![frame_bytes(&serde_json::json!({"id": 1, "nope": true}))],
            tail: Tail::Eof,
        };
        reader_session(&mut reader, &tx, Budgets::contract().max_frame_bytes);
        match rx.try_recv().unwrap() {
            FromReader::Violation(FrameError::Malformed { reason }) => {
                assert!(reason.contains("typed envelope"), "{reason}")
            }
            other => panic!("expected typed-envelope violation, got {other:?}"),
        }
        assert!(matches!(rx.try_recv().unwrap(), FromReader::Done));
    }
}
